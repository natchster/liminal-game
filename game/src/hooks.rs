//! The parts of backrooms.js's `window.__game` test hooks (70421–71228) that
//! tools use to compare a native build of a level with the browser's:
//! `randTrace`, `loadLevel`, `grid`, `arch`, `exit`, `props`, `counts`,
//! `entities`, `propList` and `barricades`. `level_dump()` bundles them in the
//! same shape as `tools/levelcheck/js-capture.mjs` writes.
use crate::prelude::*;
use serde_json::{json, Value};

fn r6(x: f64) -> f64 { if x.is_finite() { crate::archgen::to_fixed_num(x, 6) } else { x } }
fn v3(v: &Vec3) -> Value { json!([r6(v.x), r6(v.y), r6(v.z)]) }

impl Game {
    /// `__game.randTrace(on)`
    pub fn rand_trace(&mut self, on: bool) {
        self.rand_trace.on = on;
        self.rand_trace.n = crate::rng::seeded_draws();
        self.rand_trace.marks.clear();
    }
    /// `__game.randMarks` — (label, seeded draws since `rand_trace(true)`)
    pub fn rand_marks(&self) -> Vec<(String, u64)> { self.rand_trace.marks.clone() }

    /// `__game.loadLevel(n)`
    pub fn load_level(&mut self, n: i32) -> f64 {
        let t0 = self.performance_now();
        self.init_level(n);
        self.state = "playing".into();
        self.performance_now() - t0
    }

    /// `__game.propList()`
    pub fn prop_list(&self) -> Vec<Value> {
        let mut out = vec![];
        let kids = self.world.children();
        for (k, o) in kids.iter().enumerate() {
            let Some(placer) = o.user().placer.clone() else { continue };
            o.update_world_matrix(true, true);
            let mut bb = Box3::empty();
            bb.set_from_object(o);
            if bb.is_empty() { continue; }
            let mut meshes = 0;
            o.traverse(&mut |c: &Obj| { if c.is_mesh() { meshes += 1; } });
            let r = o.rot();
            out.push(json!({
                "k": k, "placer": placer, "pos": v3(&o.pos()), "rot": [r6(r.x), r6(r.y), r6(r.z)],
                "min": v3(&bb.min), "max": v3(&bb.max), "meshes": meshes, "interact": !o.user().static_prop,
            }));
        }
        out
    }

    /// The level as `tools/levelcheck/js-capture.mjs` dumps it from the browser.
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
        if let Some(t) = &self.tv_data { props.insert("tv".into(), xzn(&t.world, &t.normal)); }
        if let Some(t) = &self.phone_data { props.insert("phone".into(), xzn(&t.world, &t.normal)); }
        if let Some(t) = &self.clock_data { props.insert("clock".into(), xzn(&t.world, &t.normal)); }
        if let Some(m) = &self.man_data { let p = m.g.pos(); props.insert("mannequin".into(), json!({ "x": p.x, "z": p.z })); }
        let pick = |v: &Vec<crate::records::Pickup>| v.iter().filter(|a| !a.taken).map(|a| { let p = a.g.pos(); json!({ "x": p.x, "z": p.z }) }).collect::<Vec<_>>();
        props.insert("almonds".into(), Value::Array(pick(&self.almonds)));
        props.insert("batteries".into(), Value::Array(pick(&self.battery_items)));
        if let Some(e) = &self.exit_data { props.insert("exit".into(), xzn(&e.pos, &e.normal)); }
        let counts = json!({
            "walls": self.tiles.iter().map(|&t| t as i64).sum::<i64>(),
            "panels": self.panels.len(),
            "almonds": self.almonds.iter().filter(|a| !a.taken).count(),
            "batteries": self.battery_items.iter().filter(|a| !a.taken).count(),
            "interactables": self.interactables.len(),
            "animators": self.animators.len(),
        });
        let entities: Vec<Value> = self.entities.iter().map(|e| { let p = e.pos(); json!({ "x": p.x, "z": p.z, "state": e.state, "type": e.kind }) }).collect();
        json!({
            "seed": self.run_seed, "level": self.level_num, "marks": marks,
            "grid": { "W": self.w, "H": self.h, "cell": CELL, "tiles": self.tiles },
            "arch": arch, "scp": self.level_scp, "weather": self.level_weather.key,
            "exit": exit, "props": Value::Object(props), "counts": counts, "entities": entities,
            "player": { "x": self.player.pos.x, "y": self.player.pos.y, "z": self.player.pos.z, "yaw": self.player.yaw, "pitch": self.player.pitch },
            "propStats": { "parts": crate::props::prop_part_names().len() },
            "objects": self.prop_list(),
            "barricades": self.barricade_tiles,
        })
    }
}
