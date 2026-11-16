//! Placement rules (`docs/design/ASSETS.md`, *Placement rules*): where an
//! asset may lie, why, and how many, written in its dossier
//! (`assets/src/props/<name>/placement.toml`) and compiled into the bundle
//! embedded here (`assets/placement.json`, `cargo xtask asset write`).
//!
//! Once a floor of the rooms has settled (level 4 and down: the Building's
//! floors place their own), the things the rules place are moved to spots
//! their rules allow: on what they may stand on, against a wall, in a
//! corner, in the open; in a purpose a rule names and none it never fits;
//! where a body can reach them from where you arrive, never behind a
//! barricade, in a wall or in another thing. A thing the rules place is
//! any made with `user.s["by"] = "rules"` and its asset's name as its
//! `kind` (the floor's own almond water and batteries:
//! `core_props::pickups`); its maker decides how many (the rules drop any
//! past `count`'s most), the rules decide where. Which spot is chosen comes
//! from a stream of its own (`SplitMix64`, seeded by the floor and the
//! asset's name), never the level's: no floor's draws move. Each moved thing
//! keeps which rule placed it (`user.s["rule"]`), so a test can hold it to
//! it.
//!
//! Some things are not put down by that pass but made by the floor's layout
//! where its plan has one (a door: the far wall's, a cell's front, a
//! gallery's; `by = "layout"`). Their rules name the place in the layout
//! each stands at (`at`), with its purposes and its why, on any floor its
//! `levels` say (the Building's too). The layout asks `place_by_layout`
//! as it makes one: a rule that allows it is recorded with it
//! (`Game::layout_things`); a thing the layout may leave out is not made
//! where no rule allows it (its maker keeps its draws), and what the floor
//! must have is made and, unrecorded, found by the rules' test.
use super::N4;
use crate::base::rng::SplitMix64;
use crate::prelude::*;
use crate::world::tables::PURPOSE_LABEL;
use serde_json::Value;
use std::sync::OnceLock;

static BUNDLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/placement.json"));
const SALT: u64 = 0x91AC_E50F_17E5_2B33;
/// No two of a kind the rules place nearer than this to each other (m):
/// two in one view read as a trail laid, the game's reason (the playtest of
/// 2026-11-15, `assets/src/props/almond_water/playtest.md`); two kinds
/// stand at least `APART_KINDS` apart
pub const APART: f64 = 6.0;
pub const APART_KINDS: f64 = 1.0;
/// Nor nearer where you arrive (you would take it as you came in)
const FROM_ARRIVAL: f64 = 4.0;
/// How far round a thing set down on a surface must be flat under it (m):
/// a bottle's radius, a cell's half length, and a little
pub const FOOT: f64 = 0.045;
/// The first floor the rules place on (the rooms'; the Building's floors
/// place their own)
pub const FIRST_LEVEL: i32 = 4;

/// Where a rule puts a thing (on a wall, hung: not yet; `parse` refuses it)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    Floor,
    Surface,
    /// made by the floor's layout, at the place its rule names (`Rule::at`)
    Layout,
}
/// Who puts an asset's things where they are
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum By {
    /// the placement pass, once the floor has settled (`place_by_rules`)
    Rules,
    /// the floor's layout, where its plan has one (`place_by_layout`)
    Layout,
}
/// Near what, on the floor
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Near {
    Wall,
    Corner,
    Open,
}

/// One way a thing comes to be somewhere
#[derive(Clone, Debug)]
pub struct Rule {
    pub on: On,
    /// what it stands on (`core_<kind>` or a placer's name), on a surface
    pub of: Vec<String>,
    pub near: Option<Near>,
    /// the purposes it fits ("*": any)
    pub purposes: Vec<String>,
    /// only where the place has no light of its own (a dark region)
    pub dark: bool,
    pub weight: f64,
    pub why: String,
    /// (made by the layout) the place in it the thing stands at: "way down",
    /// "cell", "gallery", …
    pub at: String,
}

/// An asset's rules
#[derive(Clone, Debug)]
pub struct AssetRules {
    pub name: String,
    pub by: By,
    pub count: (usize, usize),
    pub levels: (i32, i32),
    pub rules: Vec<Rule>,
    /// the purposes it never fits
    pub never: Vec<String>,
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect()
}
fn pair<T: Copy>(v: &Value, f: fn(&Value) -> Option<T>) -> Option<(T, T)> {
    let a = v.as_array()?;
    Some((f(a.first()?)?, f(a.get(1)?)?))
}

/// The bundle's assets
pub fn parse(text: &str) -> Result<Vec<AssetRules>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut out = vec![];
    for (name, a) in v["assets"].as_object().ok_or("no assets")? {
        let count = pair(&a["count"], |x| x.as_u64().map(|n| n as usize)).ok_or(format!("{name}: count"))?;
        let by = match a["by"].as_str() {
            None | Some("rules") => By::Rules,
            Some("layout") => By::Layout,
            o => return Err(format!("{name}: by {o:?}: \"rules\" or \"layout\"")),
        };
        let levels = pair(&a["levels"], |x| x.as_i64().map(|n| n as i32)).unwrap_or((FIRST_LEVEL, 999));
        if by == By::Rules && levels.0 < FIRST_LEVEL {
            return Err(format!("{name}: levels from {}: the rules place from level {FIRST_LEVEL} down", levels.0));
        }
        let mut rules = vec![];
        for r in a["rule"].as_array().into_iter().flatten() {
            let at = r["at"].as_str().unwrap_or("").to_string();
            let on = match (by, r["on"].as_str()) {
                (By::Layout, None) if !at.is_empty() => On::Layout,
                (By::Layout, _) => return Err(format!("{name}: made by the layout: each rule says `at` (where in it), not `on`")),
                (By::Rules, _) if !at.is_empty() => return Err(format!("{name}: `at` is for things the layout makes")),
                (By::Rules, Some("floor")) => On::Floor,
                (By::Rules, Some("surface")) => On::Surface,
                (By::Rules, o) => return Err(format!("{name}: on {o:?}: \"floor\" or \"surface\" (hung on a wall: not yet)")),
            };
            let near = match r["near"].as_str() {
                Some("wall") => Some(Near::Wall),
                Some("corner") => Some(Near::Corner),
                Some("open") => Some(Near::Open),
                None => None,
                o => return Err(format!("{name}: near {o:?}")),
            };
            rules.push(Rule {
                on,
                of: strs(&r["of"]),
                near,
                purposes: strs(&r["purposes"]),
                dark: r["dark"].as_bool().unwrap_or(false),
                weight: r["weight"].as_f64().unwrap_or(1.0),
                why: r["why"].as_str().unwrap_or("").to_string(),
                at,
            });
        }
        out.push(AssetRules { name: name.clone(), by, count, levels, rules, never: strs(&a["never"]["purposes"]) });
    }
    Ok(out)
}

/// Every asset's rules, as the game was built with them
pub fn all() -> &'static [AssetRules] {
    static R: OnceLock<Vec<AssetRules>> = OnceLock::new();
    R.get_or_init(|| parse(BUNDLE).unwrap_or_else(|e| panic!("assets/placement.json: {e}")))
}

/// The rules of `name`, if it has any
pub fn of(name: &str) -> Option<&'static AssetRules> {
    all().iter().find(|a| a.name == name)
}

/// Every purpose a rule may name: the zones' (`world::purpose`)
pub fn purposes() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = PURPOSE_LABEL.iter().map(|&(_, l)| l).collect();
    v.extend(["pool hall", "service tunnels", "storage hall", "showroom", "well", "stair halls", "works"]);
    v
}

/// A place a rule allows
#[derive(Clone, Debug)]
pub struct Spot {
    pub at: Vec3,
    /// the rule (its index among the asset's)
    pub rule: usize,
}

fn fits(purposes: &[String], label: &str) -> bool {
    purposes.iter().any(|p| p == "*" || p == label)
}

/// A thing the floor's layout made where a rule of its asset allowed it
/// (`Game::place_by_layout`)
#[derive(Clone, Debug)]
pub struct LayoutThing {
    pub asset: &'static str,
    /// the place in the layout it stands at (its rule's `at`)
    pub at: &'static str,
    pub x: f64,
    pub z: f64,
    /// the purpose of the zone it stands in ("empty" where none is said)
    pub purpose: &'static str,
    /// the rule that allowed it (its index among the asset's)
    pub rule: usize,
}

/// The level faces of `o`'s meshes, in the world (either way up: a face
/// drawn from both sides): what a thing set on it could rest on. An
/// instanced mesh's copies are not walked.
pub fn level_faces(o: &Obj) -> Vec<[Vec3; 3]> {
    o.update_matrix_world(false);
    let mut out = vec![];
    o.traverse(&mut |c| {
        if c.instanced_bounding_box().is_some() {
            return;
        }
        let Some(g) = c.try_geo() else { return };
        let m = c.matrix_world();
        let r = g.r();
        let p = &r.position;
        let v = |i: usize| {
            let mut v = Vec3::new(p[i * 3] as f64, p[i * 3 + 1] as f64, p[i * 3 + 2] as f64);
            v.apply_matrix4(&m);
            v
        };
        let n = r.index.as_ref().map_or(p.len() / 3, |ix| ix.len());
        for t in (0..n / 3).map(|t| t * 3) {
            let (a, b, c) = match &r.index {
                Some(ix) => (ix[t] as usize, ix[t + 1] as usize, ix[t + 2] as usize),
                None => (t, t + 1, t + 2),
            };
            let (va, vb, vc) = (v(a), v(b), v(c));
            let nrm = (vb - va).crossed(vc - va);
            let len = nrm.length();
            // level within about 6 degrees
            if len > 1e-12 && nrm.y.abs() / len > 0.995 {
                out.push([va, vb, vc]);
            }
        }
    });
    out
}

/// The highest of `faces` straight over or under (x, z): what a ray down
/// from above meets first
pub fn top_over(faces: &[[Vec3; 3]], x: f64, z: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for [a, b, c] in faces {
        let d = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
        if d.abs() < 1e-14 {
            continue;
        }
        let l1 = ((b.z - c.z) * (x - c.x) + (c.x - b.x) * (z - c.z)) / d;
        let l2 = ((c.z - a.z) * (x - c.x) + (a.x - c.x) * (z - c.z)) / d;
        let l3 = 1.0 - l1 - l2;
        if l1 < -1e-9 || l2 < -1e-9 || l3 < -1e-9 {
            continue;
        }
        let y = l1 * a.y + l2 * b.y + l3 * c.y;
        if best.is_none_or(|h| y > h) {
            best = Some(y);
        }
    }
    best
}

/// Whether a thing set down at (x, y, z) on `faces` rests there: nothing
/// of them above its base anywhere under it (`FOOT` round: it is not sunk
/// into them), and their top at its base under most of it, on every side
/// (a crate's slats and a box's seams leave gaps; an edge leaves a side
/// with nothing under it)
pub fn rests(faces: &[[Vec3; 3]], x: f64, y: f64, z: f64) -> bool {
    const TOL: f64 = 0.004;
    if top_over(faces, x, z).is_some_and(|h| h > y + TOL) {
        return false;
    }
    let (mut held, mut sides) = (0, [false; 4]);
    for ring in [0.5, 1.0] {
        for k in 0..8 {
            // (off the axes a little: a probe exactly on a seam is luck)
            let a = (k as f64 + 0.3) * std::f64::consts::TAU / 8.0;
            match top_over(faces, x + a.dcos() * FOOT * ring, z + a.dsin() * FOOT * ring) {
                Some(h) if h > y + TOL => return false,
                Some(h) if h > y - TOL => {
                    held += 1;
                    sides[k / 2] = true;
                }
                _ => {}
            }
        }
    }
    held * 10 >= 16 * 7 && sides.iter().all(|&s| s)
}

/// Where on `o` (its world box `bb`) a thing can be set down: the highest
/// level face under it, on which it `rests`, nearest the middle of the
/// box; and how high. None where it has no such top (a vase's mouth, a
/// rim, a slope).
pub fn rest_on(o: &Obj, bb: &Box3) -> Option<Vec3> {
    let faces = level_faces(o);
    let (cx, cz) = ((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
    let step = 0.03;
    let mut at = vec![];
    let mut x = bb.min.x + FOOT;
    while x <= bb.max.x - FOOT {
        let mut z = bb.min.z + FOOT;
        while z <= bb.max.z - FOOT {
            at.push((x, z));
            z += step;
        }
        x += step;
    }
    // (a stable sort: ties keep the grid's order)
    at.sort_by(|a, b| (a.0 - cx).dhypot(a.1 - cz).partial_cmp(&(b.0 - cx).dhypot(b.1 - cz)).unwrap_or(std::cmp::Ordering::Equal));
    at.into_iter().find_map(|(x, z)| {
        // the highest level face under it (its middle may be over a gap)
        let y = [(0.0, 0.0), (FOOT, 0.0), (-FOOT, 0.0), (0.0, FOOT), (0.0, -FOOT)]
            .iter()
            .filter_map(|&(dx, dz)| top_over(&faces, x + dx, z + dz))
            .fold(None, |m: Option<f64>, h| Some(m.map_or(h, |m| m.max(h))))?;
        rests(&faces, x, y, z).then_some(Vec3::new(x, y, z))
    })
}

/// A name's own number (FNV-1a): an asset's stream
fn name_hash(s: &str) -> u32 {
    s.bytes().fold(0x811C_9DC5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

impl Game {
    /// Whether (x, z) is in a region with no light of its own
    pub fn dark_at(&self, x: f64, z: f64) -> bool {
        let (tx, tz) = (self.tx(x), self.tz(z));
        let r = if self.in_grid(tx, tz) { self.region_map.get(self.t_idx(tx, tz)).map_or(-1, |v| *v as i32) } else { -1 };
        r >= 0 && self.region_info.get(r as usize).is_some_and(|r| r.flags & crate::visuals::lighting::RF_DARK != 0)
    }

    /// From (x, z) along (dx, dz), the last place a thing of radius `r`
    /// clears the walls before it meets one (within `max` m): the foot of
    /// the wall that way; None if it meets none
    fn slide(&self, x: f64, z: f64, dx: f64, dz: f64, r: f64, max: f64) -> Option<(f64, f64)> {
        let mut t = 0.0;
        while t < max {
            let (nx, nz) = (x + dx * (t + 0.02), z + dz * (t + 0.02));
            if self.circle_hits_wall(nx, nz, r) {
                return (t > 0.0).then_some((x + dx * t, z + dz * t));
            }
            t += 0.02;
        }
        None
    }

    /// The purpose of the zone (x, z) is in ("empty" where none is said)
    pub fn purpose_label_at(&self, x: f64, z: f64) -> &'static str {
        self.zone_purpose_at(self.tx(x), self.tz(z)).map_or("empty", |p| p.label)
    }

    /// Whether the rules of `asset` (a thing the layout makes) allow one at
    /// `at` in the layout, standing at (x, z) on this floor: the first rule
    /// for `at` whose purposes take the zone's (and none the asset never
    /// fits), on a floor in its levels, under its count's most. One allowed
    /// is recorded with its rule (`layout_things`) and the rule's index
    /// returned. A maker free to leave it out does when none allows it (the
    /// door to nowhere, keeping its draws); a door the floor must have (its
    /// way down, a cell's, a gallery's, a lift's) is made either way, and
    /// `what_the_layout_places_keeps_its_rules` counts what was made against
    /// what was recorded
    pub fn place_by_layout(&mut self, asset: &str, at: &str, x: f64, z: f64) -> Option<usize> {
        let a = of(asset).unwrap_or_else(|| panic!("{asset}: no placement rules (assets/src/props/{asset}/, cargo xtask asset write)"));
        assert!(a.by == By::Layout, "{asset}: placed by the rules pass, not by the layout");
        if self.level_num < a.levels.0 || self.level_num > a.levels.1 {
            return None;
        }
        let purpose = self.purpose_label_at(x, z);
        // (past its most a floor, no more)
        let made = self.layout_things.iter().filter(|t| t.asset == a.name).count();
        if a.never.iter().any(|p| p == purpose) || made >= a.count.1 {
            return None;
        }
        let (rule, r) = a.rules.iter().enumerate().find(|(_, r)| r.at == at && fits(&r.purposes, purpose))?;
        self.layout_things.push(LayoutThing { asset: a.name.as_str(), at: r.at.as_str(), x, z, purpose, rule });
        Some(rule)
    }

    /// The things on the floor (top-level, made by a placer or the core
    /// props), each with its kind and its world box; not those the rules
    /// place (they are being moved)
    fn things_here(&self) -> Vec<(Obj, String, Box3)> {
        let mut out = vec![];
        for o in self.world.children() {
            let kind = {
                let u = o.user();
                if u.s.get("by").is_some_and(|b| b == "rules") {
                    continue;
                }
                match (u.s.get("kind"), u.placer.as_deref()) {
                    (Some(k), _) => format!("core_{k}"),
                    (None, Some(p)) if !p.is_empty() => p.to_string(),
                    _ => continue,
                }
            };
            let mut bb = Box3::empty();
            bb.set_from_object(&o);
            if !bb.is_empty() {
                out.push((o, kind, bb));
            }
        }
        out
    }

    /// Every spot the rules of `a` allow on this floor, and whether each is
    /// reachable; `dist` the walk from where you arrive (`bfs_open`)
    pub fn rule_spots(&self, a: &AssetRules) -> Vec<Spot> {
        // (what the layout makes stands where its plan has it)
        if a.by == By::Layout || self.level_num < a.levels.0 || self.level_num > a.levels.1 {
            return vec![];
        }
        let (sx, sz) = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let dist = self.bfs_open(sx, sz);
        let reach_tile = |x: f64, z: f64| {
            let (tx, tz) = (self.tx(x), self.tz(z));
            self.in_grid(tx, tz) && !self.is_wall_tile(tx, tz) && dist[self.t_idx(tx, tz)] >= 0
        };
        // a body can stand within reach of it
        let reachable = |x: f64, z: f64| {
            (0..9).any(|k| {
                let (px, pz) = if k == 0 {
                    (x, z)
                } else {
                    let ang = k as f64 * std::f64::consts::TAU / 8.0;
                    (x + ang.dcos() * 0.65, z + ang.dsin() * 0.65)
                };
                reach_tile(px, pz) && !self.circle_hits_wall(px, pz, PLAYER_R)
            })
        };
        let things = self.things_here();
        let clear_of_things = |x: f64, y: f64, z: f64, but: Option<&Obj>| {
            !things.iter().any(|(o, _, bb)| {
                but.is_none_or(|b| !b.ptr_eq(o))
                    && x > bb.min.x - 0.06
                    && x < bb.max.x + 0.06
                    && z > bb.min.z - 0.06
                    && z < bb.max.z + 0.06
                    && y + 0.04 > bb.min.y
                    && y + 0.02 < bb.max.y
            })
        };
        let arrival = self.spawn_pos;
        let ok = |x: f64, z: f64, rule: &Rule| {
            let label = self.purpose_label_at(x, z);
            // (its own tile open and reached: a wall's face can lie inside
            // its wall tile, and a thing there would be out of every walk)
            fits(&rule.purposes, label)
                && !a.never.iter().any(|n| n == label)
                && (x - arrival.x).dhypot(z - arrival.z) > FROM_ARRIVAL
                && (!rule.dark || self.dark_at(x, z))
                && reach_tile(x, z)
                && reachable(x, z)
        };
        let mut out = vec![];
        for (ri, rule) in a.rules.iter().enumerate() {
            match rule.on {
                On::Layout => {}
                On::Surface => {
                    for (o, kind, bb) in &things {
                        if !rule.of.iter().any(|k| k == kind) {
                            continue;
                        }
                        let wide = (bb.max.x - bb.min.x).min(bb.max.z - bb.min.z);
                        if wide < 0.18 {
                            continue;
                        }
                        // on its own top where it is level and clear (not
                        // its box's: a vase or a chair's back stands higher)
                        let Some(at) = rest_on(o, bb) else { continue };
                        if !(0.25..=1.6).contains(&(at.y - self.floor_y_at(at.x, at.z))) {
                            continue;
                        }
                        if ok(at.x, at.z, rule) && clear_of_things(at.x, at.y, at.z, Some(o)) {
                            out.push(Spot { at, rule: ri });
                        }
                    }
                }
                On::Floor => {
                    for tz in 0..self.h {
                        for tx in 0..self.w {
                            if self.is_wall_tile(tx, tz) || dist[self.t_idx(tx, tz)] < 0 {
                                continue;
                            }
                            let (cx, cz) = (self.wx(tx), self.wz(tz));
                            let wall = |dx: i32, dz: i32| self.is_wall_tile(tx + dx, tz + dz);
                            // (a wall, not a pillar: the wall tile runs on
                            // across the way; things by a lone pillar read as
                            // left in the open)
                            let run = |dx: i32, dz: i32| wall(dx, dz) && (wall(dx + dz, dz + dx) || wall(dx - dz, dz - dx));
                            let mut here: Vec<(f64, f64)> = vec![];
                            match rule.near.unwrap_or(Near::Wall) {
                                // at the wall's foot (a hand's breadth off it at most)
                                Near::Wall => {
                                    for (dx, dz) in N4 {
                                        if run(dx, dz)
                                            && let Some(p) = self.slide(cx, cz, dx as f64, dz as f64, 0.08, CELL / 2.0 - 0.1)
                                        {
                                            here.push(p);
                                        }
                                    }
                                }
                                // into the corner: to one wall, then along it to the other
                                Near::Corner => {
                                    for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                                        if run(dx, 0)
                                            && run(0, dz)
                                            && let Some((x1, z1)) = self.slide(cx, cz, dx as f64, 0.0, 0.08, CELL / 2.0 - 0.1)
                                            && let Some(p) = self.slide(x1, z1, 0.0, dz as f64, 0.08, CELL / 2.0 - 0.1)
                                        {
                                            here.push(p);
                                        }
                                    }
                                }
                                Near::Open => {
                                    if N4.iter().all(|&(dx, dz)| !wall(dx, dz)) {
                                        here.push((cx, cz));
                                    }
                                }
                            }
                            for (x, z) in here {
                                let y = self.floor_y_at(x, z);
                                // clear of the walls, and (against one) near it
                                if self.circle_hits_wall(x, z, 0.06) {
                                    continue;
                                }
                                if rule.near != Some(Near::Open) && !self.circle_hits_wall(x, z, 0.2) {
                                    continue;
                                }
                                if ok(x, z, rule) && clear_of_things(x, y, z, None) {
                                    out.push(Spot { at: Vec3::new(x, y, z), rule: ri });
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// The things the rules place (`user.s["by"] == "rules"`, their asset
    /// their `kind`), moved to spots their rules allow, after the floor has
    /// settled; any past their count's most, and any their rules allow
    /// nowhere, are not on this floor (taken out of the world and the
    /// pickups' lists). A thing marked with no rules for its kind stays
    /// where it was made, its rule `"unruled"` (the rules test fails on it).
    pub fn place_by_rules(&mut self) {
        let mut gone: Vec<Obj> = vec![];
        // each asset's things, in the order they were made
        let mut kinds: Vec<(&'static AssetRules, Vec<Obj>)> = vec![];
        // what else of those kinds lies here (her camp's): kept as far from
        // as one the rules place
        let mut taken: Vec<(Vec3, &str)> = vec![];
        for o in self.world.children() {
            let (by_rules, kind) = {
                let u = o.user();
                (u.s.get("by").is_some_and(|b| b == "rules"), u.s.get("kind").cloned())
            };
            let a = kind.as_deref().and_then(of);
            match (by_rules, a) {
                (true, Some(a)) => match kinds.iter_mut().find(|(k, _)| std::ptr::eq(*k, a)) {
                    Some((_, v)) => v.push(o.clone()),
                    None => kinds.push((a, vec![o.clone()])),
                },
                (true, None) => {
                    o.user().s.insert("rule".into(), "unruled".into());
                }
                // (one the layout made, a door, is not one of theirs)
                (false, Some(a)) if a.by == By::Rules => taken.push((o.pos(), a.name.as_str())),
                (false, _) => {}
            }
        }
        // each asset's things, its spots and its own stream
        let mut queues = vec![];
        for (a, mut objs) in kinds {
            if objs.len() > a.count.1 {
                for o in objs.split_off(a.count.1) {
                    o.user().s.insert("rule".into(), "none".into());
                    gone.push(o);
                }
            }
            let spots = self.rule_spots(a);
            let r = SplitMix64::new(self.current_seed ^ name_hash(&a.name).wrapping_mul(0x9E37_79B9), SALT);
            queues.push((a, objs.into_iter(), spots, r));
        }
        // the assets in turn, a thing each, so where spots are few (the same
        // tops) each kind has its share
        loop {
            let mut any = false;
            for (a, objs, spots, r) in queues.iter_mut() {
                let Some(o) = objs.next() else { continue };
                any = true;
                let name: &'static str = a.name.as_str();
                let free: Vec<&Spot> = spots
                    .iter()
                    .filter(|s| {
                        taken.iter().all(|(t, k)| (t.x - s.at.x).dhypot(t.z - s.at.z) > if *k == name { APART } else { APART_KINDS })
                    })
                    .collect();
                // a rule by its weight, among those with somewhere left; then a spot of it
                let total: f64 =
                    a.rules.iter().enumerate().filter(|(i, _)| free.iter().any(|s| s.rule == *i)).map(|(_, rl)| rl.weight).sum();
                if total <= 0.0 {
                    // nowhere its rules allow is left: it is not on this floor
                    o.user().s.insert("rule".into(), "none".into());
                    gone.push(o.clone());
                    continue;
                }
                let mut pick = r.next() * total;
                let mut ri = 0;
                for (i, rl) in a.rules.iter().enumerate() {
                    if !free.iter().any(|s| s.rule == i) {
                        continue;
                    }
                    ri = i;
                    pick -= rl.weight;
                    if pick < 0.0 {
                        break;
                    }
                }
                let of_rule: Vec<&&Spot> = free.iter().filter(|s| s.rule == ri).collect();
                let s = of_rule[((r.next() * of_rule.len() as f64) as usize).min(of_rule.len() - 1)];
                o.position().set(s.at.x, s.at.y, s.at.z);
                // a bottle on the floor was dropped: on its side; set down on
                // something, it stands (the playtest: lying, it reads as
                // dropped). The draw is every asset's; the pose, the bottle's
                // builder's.
                let lying = a.rules[ri].on != On::Surface && r.next() < 0.75;
                if name == "almond_water" {
                    crate::core_props::pickups::lay(&o, lying);
                }
                o.update_world_matrix(true, true);
                o.user().s.insert("rule".into(), ri.to_string());
                taken.push((s.at, name));
            }
            if !any {
                break;
            }
        }
        for o in &gone {
            self.world.remove(o);
        }
        self.almonds.retain(|p| !gone.iter().any(|o| o.ptr_eq(&p.g)));
        self.battery_items.retain(|p| !gone.iter().any(|o| o.ptr_eq(&p.g)));
        for p in self.almonds.iter_mut().chain(self.battery_items.iter_mut()) {
            p.base = p.g.pos().y;
        }
    }
}
