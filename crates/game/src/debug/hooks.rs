//! Level-inspection hooks used by tools and tests: `rand_trace`,
//! `rand_marks`, `load_level`, `prop_list`, `zone_views` and `level_dump`.
//! `level_dump()` dumps a level's build trace, grid, zone purposes, regions,
//! props and placed objects as JSON (`backrooms --dump-level`,
//! tests/reference_levels.rs).
use crate::prelude::*;
use serde_json::{json, Value};

fn r6(x: f64) -> f64 {
    if x.is_finite() {
        crate::base::numeric::round_decimals(x, 6)
    } else {
        x
    }
}
fn v3(v: &Vec3) -> Value {
    json!([r6(v.x), r6(v.y), r6(v.z)])
}

impl Game {
    /// Start (`on`) or stop tracing seeded draws; clears the recorded marks.
    pub fn rand_trace(&mut self, on: bool) {
        self.rand_trace.on = on;
        self.rand_trace.n = crate::base::rng::seeded_draws();
        self.rand_trace.marks.clear();
    }
    /// The trace marks: (label, seeded draws since `rand_trace(true)`)
    pub fn rand_marks(&self) -> Vec<(String, u64)> {
        self.rand_trace.marks.clone()
    }

    /// Build level `n` and start playing it; returns the build time in ms.
    pub fn load_level(&mut self, n: i32) -> f64 {
        let t0 = self.clock_ms();
        self.init_level(n);
        self.state = "playing".into();
        self.clock_ms() - t0
    }

    /// Every placed prop at the top of the world: its placer, transform,
    /// world bounds, mesh count and whether it can be interacted with.
    pub fn prop_list(&self) -> Vec<Value> {
        let mut out = vec![];
        let kids = self.world.children();
        for (k, o) in kids.iter().enumerate() {
            let Some(placer) = o.user().placer.clone() else { continue };
            o.update_world_matrix(true, true);
            let mut bb = Box3::empty();
            bb.set_from_object(o);
            if bb.is_empty() {
                continue;
            }
            let mut meshes = 0;
            o.traverse(&mut |c: &Obj| {
                if c.is_mesh() {
                    meshes += 1;
                }
            });
            let r = o.rot();
            out.push(json!({
                "k": k, "placer": placer, "pos": v3(&o.pos()), "rot": [r6(r.x), r6(r.y), r6(r.z)],
                "min": v3(&bb.min), "max": v3(&bb.max), "meshes": meshes, "interact": !o.user().static_prop,
            }));
        }
        out
    }

    /// A camera pose inside each zone of the layout, for looking at it: the
    /// open tile of the zone with the longest straight run of open floor in
    /// front of it, a step back from its centre, facing down the run.
    /// (zone, x, z, yaw) in world coordinates.
    pub fn zone_views(&self) -> Vec<(usize, f64, f64, f64)> {
        let Some(l) = self.layout.as_ref() else { return vec![] };
        let mut out = vec![];
        for (zi, zn) in l.zones.iter().enumerate() {
            let (cx, cz) = ((zn.x0 + zn.x1) as f64 / 2.0, (zn.z0 + zn.z1) as f64 / 2.0);
            let mut best: Option<(f64, i32, i32, i32, i32)> = None;
            for z in zn.z0..=zn.z1 {
                for x in zn.x0..=zn.x1 {
                    if self.is_wall_tile(x, z) {
                        continue;
                    }
                    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let mut run = 0;
                        while run < 10 && !self.is_wall_tile(x + dx * (run + 1), z + dz * (run + 1)) {
                            run += 1;
                        }
                        // long views, from near the middle of the zone
                        let score = run as f64 - 0.15 * ((x as f64 - cx).abs() + (z as f64 - cz).abs());
                        if best.map(|b| score > b.0).unwrap_or(true) {
                            best = Some((score, x, z, dx, dz));
                        }
                    }
                }
            }
            if let Some((_, x, z, dx, dz)) = best {
                let (px, pz) = (self.wx(x) - dx as f64 * 1.5, self.wz(z) - dz as f64 * 1.5);
                out.push((zi, px, pz, (-(dx as f64)).atan2(-(dz as f64))));
            }
        }
        out
    }

    /// The level as JSON: seed, trace marks, grid, zones, key props, counts,
    /// entities, player, placed objects and barricades.
    pub fn level_dump(&self) -> Value {
        let marks: Vec<Value> = self.rand_marks().into_iter().map(|(l, n)| json!([l, n])).collect();
        let arch = self.layout.as_ref().map(|l| json!({
            "dominant": l.dominant,
            "zones": l.zones.iter().map(|z| json!({ "id": z.id, "arch": z.arch, "x0": z.x0, "z0": z.z0, "x1": z.x1, "z1": z.z1, "ceilH": z.ceil_h, "wall": z.wall, "wallT": z.wall_t })).collect::<Vec<_>>(),
            "zoneOf": l.zone_of,
        }));
        let exit = self.exit_data.as_ref().map(|e| json!({ "x": e.pos.x, "z": e.pos.z }));
        let mut props = serde_json::Map::new();
        let xzn = |w: &Vec3, n: &Vec3| json!({ "x": w.x, "z": w.z, "nx": n.x, "nz": n.z });
        if let Some(t) = &self.tv_data {
            props.insert("tv".into(), xzn(&t.world, &t.normal));
        }
        if let Some(t) = &self.phone_data {
            props.insert("phone".into(), xzn(&t.world, &t.normal));
        }
        if let Some(t) = &self.clock_data {
            props.insert("clock".into(), xzn(&t.world, &t.normal));
        }
        if let Some(m) = &self.man_data {
            let p = m.g.pos();
            props.insert("mannequin".into(), json!({ "x": p.x, "z": p.z }));
        }
        let pick = |v: &Vec<crate::base::records::Pickup>| {
            v.iter()
                .filter(|a| !a.taken)
                .map(|a| {
                    let p = a.g.pos();
                    json!({ "x": p.x, "z": p.z })
                })
                .collect::<Vec<_>>()
        };
        props.insert("almonds".into(), Value::Array(pick(&self.almonds)));
        props.insert("batteries".into(), Value::Array(pick(&self.battery_items)));
        if let Some(e) = &self.exit_data {
            props.insert("exit".into(), xzn(&e.pos, &e.normal));
        }
        let counts = json!({
            "walls": self.tiles.iter().map(|&t| t as i64).sum::<i64>(),
            "panels": self.panels.len(),
            "almonds": self.almonds.iter().filter(|a| !a.taken).count(),
            "batteries": self.battery_items.iter().filter(|a| !a.taken).count(),
            "interactables": self.interactables.len(),
            "animators": self.animators.len(),
        });
        let entities: Vec<Value> = self
            .entities
            .iter()
            .map(|e| {
                let p = e.pos();
                json!({ "x": p.x, "z": p.z, "state": e.state, "type": e.kind })
            })
            .collect();
        let purposes: Vec<Value> = self
            .zone_purposes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                json!({ "zone": i, "area": p.area, "label": p.label, "primary": p.primary, "secondary": p.secondary, "core": p.core })
            })
            .collect();
        let regions = json!({
            "map": self.region_map,
            // wall variant, grime, flags, light
            "info": self.region_info.iter().map(|r| json!([r.wall, r.grime, r.flags, r.lit])).collect::<Vec<_>>(),
        });
        json!({
            "seed": self.run_seed, "level": self.level_num, "marks": marks,
            "grid": { "W": self.w, "H": self.h, "cell": CELL, "tiles": self.tiles },
            "arch": arch, "purposes": purposes, "regions": regions, "scp": self.level_scp, "weather": self.level_weather.key,
            "exit": exit, "props": Value::Object(props), "counts": counts, "entities": entities,
            "player": { "x": self.player.pos.x, "y": self.player.pos.y, "z": self.player.pos.z, "yaw": self.player.yaw, "pitch": self.player.pitch },
            "propStats": { "parts": crate::assets::props::prop_part_names().len() },
            "objects": self.prop_list(),
            "barricades": self.barricade_tiles,
        })
    }

    /// Keep one creature of `kind` standing `dist` m in front of the
    /// camera, facing it, and send the others away (`--showcase`, for
    /// looking at a creature headless). Adds one if the level has none.
    pub fn showcase_entity(&mut self, kind: &str, dist: f64) {
        let ei = match self.entities.iter().position(|e| e.kind == kind) {
            Some(i) => i,
            None => {
                let (tx, tz) = self.nav_tile_at(self.player.pos.x, self.player.pos.z);
                self.add_entity(kind, tx, tz, 0)
            }
        };
        let yaw = self.player.yaw;
        let (fx, fz) = (-yaw.sin(), -yaw.cos());
        for (k, e) in self.entities.iter_mut().enumerate() {
            if k == ei {
                e.g.set_pos(self.player.pos.x + fx * dist, 0.0, self.player.pos.z + fz * dist);
                e.yaw = yaw;
                e.state = "wander".into();
                e.freeze = 1.0;
            } else {
                e.g.set_pos(-1e4, 0.0, -1e4);
                e.freeze = 1.0;
            }
        }
    }
}
