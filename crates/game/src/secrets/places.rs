//! The hiding places (`docs/design/SECRETS.md` 4): where each kind fits on a
//! level, what it looks like (its tell, the same all run), its verb, and
//! what happens when it is opened; the scratched sign near some of them.
use super::{PlaceKind, SecretPlan, Thing};
use crate::level::archgen;
use crate::prelude::*;

/// The four ways from a tile, as tile steps
const SIDES: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Where a cell of a tiled ceiling's 6 × 6 grid is, from its tile's
/// centre, m: the bars run through the tile's edge across z and half a
/// cell off it across x (`textures::ceiling_grid`, laid with u along z), so
/// a cell's centre is a whole cell off the edge in x and half one in z; one
/// clear of the panel hung at the tile's centre
const TILE_CELL: (f64, f64) = (-4.0 / 3.0, -1.0);

impl Game {
    fn walls_beside(&self, t: (i32, i32)) -> usize {
        SIDES.iter().filter(|(dx, dz)| self.is_wall_tile(t.0 + dx, t.1 + dz)).count()
    }

    /// The zone's archetype and purpose at a tile ("office", "office wing")
    fn place_of(&self, t: (i32, i32)) -> (&'static str, &'static str) {
        let z = self.zone_at_tile(t.0, t.1);
        let arch = z.and_then(|z| self.layout.as_ref().map(|l| l.zones[z].arch)).unwrap_or("");
        let label = z.and_then(|z| self.zone_purposes.get(z)).map(|p| p.label).unwrap_or("");
        (arch, label)
    }

    /// A flat wall's face beside tile `t` at height `y`, the first side (in
    /// the stream's order) where one stands: (face x, z, normal out of it)
    fn wall_beside(&self, t: (i32, i32), y: f64, r: &mut SplitMix64) -> Option<(f64, f64, (f64, f64))> {
        let (cx, cz) = (self.wx(t.0), self.wz(t.1));
        let k = (r.next() * 4.0) as usize;
        for i in 0..4 {
            let side = SIDES[(k + i) % 4];
            if let Some((fx, fz)) = self.mark_wall(cx, cz, side, y, CELL * 0.55) {
                return Some((fx, fz, (-side.0 as f64, -side.1 as f64)));
            }
        }
        None
    }

    /// A place of `kind` at tile `t` holding `thing`, if one fits there.
    pub(crate) fn place_at(&self, kind: PlaceKind, thing: Thing, t: (i32, i32), r: &mut SplitMix64) -> Option<SecretPlan> {
        let (cx, cz) = (self.wx(t.0), self.wz(t.1));
        let building = !self.ground.is_empty();
        let (arch, label) = self.place_of(t);
        let plan = |x: f64, z: f64, y: f64, normal: (f64, f64)| SecretPlan {
            kind,
            thing,
            tile: t,
            x,
            z,
            y,
            normal,
            drawer: None,
            sign: None,
            crossed: false,
        };
        match kind {
            PlaceKind::Tile => {
                let tiled = self.ceil_mat_key(self.t_idx(t.0, t.1)).starts_with("tiles:");
                let (x, z) = (cx + TILE_CELL.0, cz + TILE_CELL.1);
                let ceil = self.ceil_at_world(x, z);
                // under a tiled ceiling a hand on a chair reaches, room round the chair
                if building || !tiled || ceil > 3.4 || self.walls_beside(t) > 2 || self.circle_hits_wall(x, z, 0.5) {
                    return None;
                }
                Some(plan(x, z, ceil, (0.0, 0.0)))
            }
            PlaceKind::Vent | PlaceKind::Hole | PlaceKind::Hook | PlaceKind::Radiator | PlaceKind::Table => {
                let y = match kind {
                    PlaceKind::Vent => 0.3,
                    PlaceKind::Hole => 1.0,
                    PlaceKind::Hook => 1.65,
                    PlaceKind::Radiator => 0.35,
                    _ => 0.5,
                };
                // where each is: a radiator on the Building's floors, a hook
                // in an office wing, a table in a room someone used, a vent
                // where a building has them
                let fits = match kind {
                    PlaceKind::Radiator => building,
                    PlaceKind::Hook => !building && (arch == "office" || label == "office wing"),
                    PlaceKind::Table => !building && matches!(arch, "office" | "lobby" | "hall") && label != "washrooms",
                    PlaceKind::Vent => !building && arch != "pool",
                    _ => true,
                };
                if !fits {
                    return None;
                }
                let (fx, fz, n) = self.wall_beside(t, y, r)?;
                // a table needs the floor before the wall clear
                if kind == PlaceKind::Table && self.circle_hits_wall(fx + n.0 * 0.4, fz + n.1 * 0.4, 0.3) {
                    return None;
                }
                Some(plan(fx, fz, y, n))
            }
            PlaceKind::Cell => {
                let b = self.building.as_ref()?;
                let c = b.cells.iter().find(|c| c.tile == t)?;
                if self.story.camp.as_ref().is_some_and(|camp| camp.tile == t) {
                    return None;
                }
                // on its bench, along the back wall
                let back = archgen::DIRS[c.door ^ 1];
                let along = r.range(-0.8, 0.8);
                let (bx, bz) = (back[0] as f64, back[1] as f64);
                let x = cx + bx * (CELL / 2.0 - 0.3) - bz * along;
                let z = cz + bz * (CELL / 2.0 - 0.3) + bx * along;
                Some(plan(x, z, 0.44, (-bx, -bz)))
            }
            PlaceKind::Drawer => {
                let k = self.drawer_props.iter().position(|d| {
                    let p = d.cabinet.pos();
                    (self.tx(p.x), self.tz(p.z)) == t && d.drawers.len() >= 2
                })?;
                let p = self.drawer_props[k].cabinet.pos();
                let mut s = plan(p.x, p.z, 0.0, (0.0, 0.0));
                s.drawer = Some((k, 1));
                Some(s)
            }
            PlaceKind::Corner => {
                let walls = self.walls_beside(t);
                let wet = matches!(arch, "lobby" | "pool") || matches!(label, "overgrown atrium" | "washrooms");
                let ok = match thing {
                    // a dead end
                    Thing::Seeds => walls >= 3,
                    // the wet carpet
                    Thing::PaleMushrooms | Thing::BrownMushrooms => walls >= 2 && wet,
                    // a corridor's floor
                    Thing::Envelope => walls == 2,
                    _ => walls >= 2,
                };
                if !ok {
                    return None;
                }
                // the corner between two walls of it (or the middle of a corridor)
                let w: Vec<(i32, i32)> = SIDES.iter().copied().filter(|(dx, dz)| self.is_wall_tile(t.0 + dx, t.1 + dz)).collect();
                let corner = w.iter().flat_map(|a| w.iter().map(move |b| (*a, *b))).find(|(a, b)| a.0 * b.0 + a.1 * b.1 == 0);
                let (x, z) = match (thing, corner) {
                    (Thing::Envelope, _) | (_, None) => (cx + r.range(-0.5, 0.5), cz + r.range(-0.5, 0.5)),
                    (_, Some((a, b))) => (cx + (a.0 + b.0) as f64 * 1.45, cz + (a.1 + b.1) as f64 * 1.45),
                };
                if self.circle_hits_wall(x, z, 0.2) || !self.los(cx, cz, x, z) {
                    return None;
                }
                Some(plan(x, z, 0.0, (0.0, 0.0)))
            }
        }
    }

    /// A scratched sign near tile `t`, on a flat wall seen from the tile's
    /// middle, at 1.4–1.6 m: (x, z, y, normal)
    pub(crate) fn sign_near(&self, t: (i32, i32), r: &mut SplitMix64) -> Option<(f64, f64, f64, (f64, f64))> {
        let y = r.range(1.4, 1.6);
        let (cx, cz) = (self.wx(t.0), self.wz(t.1));
        let mut near: Vec<(i32, i32)> = vec![];
        for dz in -2..=2 {
            for dx in -2i32..=2 {
                let n = (t.0 + dx, t.1 + dz);
                if dx.abs() + dz.abs() >= 1 && self.in_grid(n.0, n.1) && !self.is_wall_tile(n.0, n.1) {
                    near.push(n);
                }
            }
        }
        near.sort_by_key(|n| (n.0 - t.0).abs() + (n.1 - t.1).abs());
        for n in near {
            let mut rr = *r;
            let Some((fx, fz, nn)) = self.wall_beside(n, y, &mut rr) else { continue };
            let (sx, sz) = (fx + nn.0 * 0.3, fz + nn.1 * 0.3);
            if self.los(cx, cz, sx, sz) {
                *r = rr;
                return Some((fx + nn.0 * crate::story::marks::ON_WALL, fz + nn.1 * crate::story::marks::ON_WALL, y, nn));
            }
        }
        None
    }
}
