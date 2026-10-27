//! Zone purposes: what kind of place each part of a level is.
//!
//! Zones of one archetype that open into each other with no wall between
//! them read as one space, an *area*. Each area is given a purpose once per
//! level: a primary prop theme drawn by how welcome the themes are in its
//! archetype (`THEME_ALLOW`), sometimes a compatible second theme
//! (`THEME_COMPAT`), or no theme at all (`PURPOSE_EMPTY`); every zone of the
//! area keeps it. Each zone also draws whether the building's own fittings
//! (the "core" theme) are there. `themed` lets a placer use only the floor
//! whose purpose welcomes its theme, so an area's props tell one story.
use crate::level::archgen::{Layout, Zone};
use crate::prelude::*;
use crate::world::build::theme_allow;
use crate::world::tables::{PURPOSE_EMPTY, PURPOSE_LABEL, SECONDARY_CHANCE, THEME_ALLOW, THEME_COMPAT};

/// What a zone is for
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZonePurpose {
    /// the area (lowest zone id in it) the zone belongs to
    pub area: usize,
    /// what the place reads as ("office wing", "empty", ...)
    pub label: &'static str,
    /// the main prop theme (None: left bare)
    pub primary: Option<&'static str>,
    /// a compatible second theme
    pub secondary: Option<&'static str>,
    /// the building's own fittings (stains, vents, sockets, signs ...)
    pub core: bool,
}
impl ZonePurpose {
    /// Placers of `theme` may use this zone
    pub fn allows(&self, theme: &str) -> bool {
        if theme == "core" {
            return self.core;
        }
        self.primary == Some(theme) || self.secondary == Some(theme)
    }
    /// One of the zone's themes is among `themes`
    pub fn has_any(&self, themes: &[&str]) -> bool {
        themes.iter().any(|t| self.primary == Some(t) || self.secondary == Some(t))
    }
}

fn empty_chance(arch: &str) -> f64 {
    PURPOSE_EMPTY.iter().find(|(a, _)| *a == arch).map(|&(_, v)| v).unwrap_or(0.0)
}
fn label_of(arch: &str, primary: Option<&str>) -> &'static str {
    match (arch, primary) {
        ("pool", Some("bathroom")) => "pool hall",
        ("tunnels" | "electrical", Some("industrial")) => "service tunnels",
        ("hall", Some("industrial")) => "storage hall",
        ("store", None) => "showroom",
        // the Building's floors are what they are, nothing more
        ("well", _) => "well",
        ("halls", _) => "stair halls",
        ("works", _) => "works",
        _ => {
            let k = primary.unwrap_or("");
            PURPOSE_LABEL.iter().find(|(t, _)| *t == k).map(|&(_, l)| l).unwrap_or("empty")
        }
    }
}
/// A weighted pick (one draw); None when nothing has weight
fn pick(list: &[(&'static str, f64)]) -> Option<&'static str> {
    let total: f64 = list.iter().map(|&(_, w)| w.max(0.0)).sum();
    if total <= 0.0 {
        return None;
    }
    let mut r = rnd() * total;
    for &(k, w) in list {
        r -= w.max(0.0);
        if r < 0.0 {
            return Some(k);
        }
    }
    list.last().map(|&(k, _)| k)
}

/// The purpose of an area of archetype `arch` (its zones then draw `core`
/// each). `bare`: no themed props whatever the draw (the first room of the
/// first level).
fn pick_purpose(arch: &'static str, area: usize, bare: bool) -> ZonePurpose {
    let empty = rnd() < empty_chance(arch) || bare;
    let themes: Vec<(&'static str, f64)> = THEME_ALLOW
        .iter()
        .find(|(a, _)| *a == arch)
        .map(|(_, l)| l.iter().copied().filter(|&(t, _)| t != "core").collect())
        .unwrap_or_default();
    let primary = if empty { None } else { pick(&themes) };
    let secondary = match primary {
        Some(p) if rnd() < SECONDARY_CHANCE => {
            // compatible themes, those the archetype welcomes first
            let compat: Vec<(&'static str, f64)> = THEME_COMPAT
                .iter()
                .find(|(t, _)| *t == p)
                .map(|(_, l)| l.iter().map(|&(t, w)| (t, w * if theme_allow(arch, t) > 0.0 { 1.0 } else { 0.3 })).collect())
                .unwrap_or_default();
            pick(&compat)
        }
        _ => None,
    };
    ZonePurpose { area, label: label_of(arch, primary), primary, secondary, core: false }
}

/// Share of wall tiles on the column (`along_z`) or row `line`, between `a`
/// and `b` (inclusive)
fn wall_share(tiles: &[u8], w: i32, along_z: bool, line: i32, a: i32, b: i32) -> f64 {
    let walls = (a..=b).filter(|&k| if along_z { tiles[(k * w + line) as usize] } else { tiles[(line * w + k) as usize] } != 0).count();
    walls as f64 / (b - a + 1).max(1) as f64
}

/// A real wall runs between the two touching zones (either side's edge is
/// mostly wall), rather than one space carrying on across the zone line
pub fn walled_apart(l: &Layout, tiles: &[u8], a: &Zone, b: &Zone) -> bool {
    let share = if b.x0 == a.x1 + 1 || a.x0 == b.x1 + 1 {
        // side by side: the west zone's last column, the east zone's first
        let (west, east) = if b.x0 == a.x1 + 1 { (a, b) } else { (b, a) };
        let (z0, z1) = (a.z0.max(b.z0), a.z1.min(b.z1));
        wall_share(tiles, l.w, true, west.x1, z0, z1).max(wall_share(tiles, l.w, true, east.x0, z0, z1))
    } else {
        let (north, south) = if b.z0 == a.z1 + 1 { (a, b) } else { (b, a) };
        let (x0, x1) = (a.x0.max(b.x0), a.x1.min(b.x1));
        wall_share(tiles, l.w, false, north.z1, x0, x1).max(wall_share(tiles, l.w, false, south.z0, x0, x1))
    };
    share >= 0.5
}

/// The area of every zone: the lowest zone id among the zones of its
/// archetype it opens into without a wall between them
pub fn zone_areas(l: &Layout, tiles: &[u8]) -> Vec<usize> {
    let n = l.zones.len();
    let mut area: Vec<usize> = (0..n).collect();
    fn root(g: &mut [usize], mut i: usize) -> usize {
        while g[i] != i {
            g[i] = g[g[i]];
            i = g[i];
        }
        i
    }
    for a in 0..n {
        for b in a + 1..n {
            let (za, zb) = (&l.zones[a], &l.zones[b]);
            if za.arch == zb.arch && za.touches(zb) && !walled_apart(l, tiles, za, zb) {
                let (ra, rb) = (root(&mut area, a), root(&mut area, b));
                area[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    (0..n).map(|i| root(&mut area, i)).collect()
}

impl Game {
    /// Gives every zone of the layout its purpose (one per area, drawn in
    /// zone order; the building's fittings are drawn zone by zone). Without
    /// a layout there are no zones and no purposes.
    pub fn pick_zone_purposes(&mut self) {
        self.zone_purposes.clear();
        let Some(l) = self.layout.as_ref() else { return };
        let areas = zone_areas(l, &self.tiles);
        let spawn_zone = l.zone_of[l.idx(1, 1)] as usize;
        let mut out: Vec<ZonePurpose> = Vec::with_capacity(l.zones.len());
        for (zi, z) in l.zones.iter().enumerate() {
            let a = areas[zi];
            let mut p = if a < zi {
                out[a].clone()
            } else {
                // the first room of the first level is the photograph: nothing in it
                pick_purpose(z.arch, a, self.level_num == crate::level::building::FIRST_ROOMS_LEVEL && areas[spawn_zone] == a)
            };
            p.core = rnd() < theme_allow(z.arch, "core");
            out.push(p);
        }
        self.zone_purposes = out;
    }

    /// The purpose of the zone at tile (x, z) (None before `build_world`
    /// picked them, or without a layout)
    pub fn zone_purpose_at(&self, x: i32, z: i32) -> Option<&ZonePurpose> {
        self.zone_purposes.get(self.zone_at_tile(x, z)?)
    }
}
