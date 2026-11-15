//! Placement rules (`docs/design/ASSETS.md`, *Placement rules*): where an
//! asset may lie, why, and how many, written in its dossier
//! (`assets/src/props/<name>/placement.toml`) and compiled into the bundle
//! embedded here (`assets/placement.json`, `cargo xtask asset write`).
//!
//! Once a floor has settled, the things the rules place (the floor's own
//! almond water and batteries: `core_props::pickups`) are moved to spots
//! their rules allow: on what they may stand on, against a wall, in a
//! corner, in the open; in a purpose a rule names and none it never fits;
//! where a body can reach them from where you arrive, never behind a
//! barricade, in a wall or in another thing. Which spot is chosen comes from
//! a stream of its own (`SplitMix64`, seeded by the floor and the asset),
//! never the level's: no floor's draws move. Each moved thing keeps which
//! rule placed it (`user.s["rule"]`), so a test can hold it to it.
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

/// Where a rule puts a thing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    Floor,
    Surface,
    Wall,
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
}

/// An asset's rules
#[derive(Clone, Debug)]
pub struct AssetRules {
    pub name: String,
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
        let levels = pair(&a["levels"], |x| x.as_i64().map(|n| n as i32)).unwrap_or((1, 999));
        let mut rules = vec![];
        for r in a["rule"].as_array().into_iter().flatten() {
            let on = match r["on"].as_str() {
                Some("floor") => On::Floor,
                Some("surface") => On::Surface,
                Some("wall") => On::Wall,
                o => return Err(format!("{name}: on {o:?}")),
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
            });
        }
        out.push(AssetRules { name: name.clone(), count, levels, rules, never: strs(&a["never"]["purposes"]) });
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

    /// The things on the floor (top-level, made by a placer or the core
    /// props), each with its kind and its world box
    fn things_here(&self) -> Vec<(Obj, String, Box3)> {
        let mut out = vec![];
        for o in self.world.children() {
            let kind = {
                let u = o.user();
                match (u.s.get("kind"), u.placer.as_deref()) {
                    (Some(k), _) => format!("core_{k}"),
                    (None, Some(p)) if !p.is_empty() => p.to_string(),
                    _ => continue,
                }
            };
            if kind == "core_almond_water" || kind == "core_battery" {
                continue;
            }
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
        if self.level_num < a.levels.0 || self.level_num > a.levels.1 {
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
                On::Surface => {
                    for (o, kind, bb) in &things {
                        if !rule.of.iter().any(|k| k == kind) {
                            continue;
                        }
                        let (cx, cz) = ((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
                        let floor = self.floor_y_at(cx, cz);
                        let top = bb.max.y;
                        let wide = (bb.max.x - bb.min.x).min(bb.max.z - bb.min.z);
                        if !(0.25..=1.6).contains(&(top - floor)) || wide < 0.18 {
                            continue;
                        }
                        if ok(cx, cz, rule) && clear_of_things(cx, top, cz, Some(o)) {
                            out.push(Spot { at: Vec3::new(cx, top, cz), rule: ri });
                        }
                    }
                }
                On::Floor | On::Wall => {
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

    /// The things the rules place (`user.s["by"] == "rules"`), moved to spots
    /// their rules allow, after the floor has settled
    pub fn place_by_rules(&mut self) {
        let mut taken: Vec<(Vec3, &str)> = vec![];
        let mut gone: Vec<Obj> = vec![];
        // each asset's things, its spots and its own stream
        let mut queues = vec![];
        for (list, name) in [(0, "almond_water"), (1, "battery")] {
            let Some(a) = of(name) else { continue };
            let objs: Vec<Obj> = if list == 0 {
                self.almonds.iter().map(|p| p.g.clone()).collect()
            } else {
                self.battery_items.iter().map(|p| p.g.clone()).collect()
            };
            let objs: Vec<Obj> = objs.into_iter().filter(|o| o.user().s.get("by").is_some_and(|b| b == "rules")).collect();
            if objs.is_empty() {
                continue;
            }
            let spots = self.rule_spots(a);
            let r = SplitMix64::new(self.current_seed ^ (name.len() as u32).wrapping_mul(0x9E37_79B9), SALT);
            queues.push((name, a, objs.into_iter(), spots, r));
        }
        // the assets in turn, a thing each, so where spots are few (the same
        // tops) each kind has its share
        loop {
            let mut any = false;
            for (name, a, objs, spots, r) in queues.iter_mut() {
                let Some(o) = objs.next() else { continue };
                any = true;
                let name: &str = name;
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
                // something, it stands (the playtest: lying, it reads as dropped)
                let lying = a.rules[ri].on != On::Surface && r.next() < 0.75;
                if name == "almond_water" {
                    crate::core_props::pickups::lay(&o, lying);
                }
                o.update_world_matrix(true, true);
                o.user().s.insert("rule".into(), ri.to_string());
                taken.push((s.at, if name == "almond_water" { "almond_water" } else { "battery" }));
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
