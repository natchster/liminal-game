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

    /// A camera pose looking at the exit door down the floor plan: of the
    /// open tiles 3–7 tiles' walk from the door that can see it and stand in
    /// front of it (within 60° of the way it faces), the furthest in a
    /// straight line, facing the door. (x, z, yaw) in world coordinates;
    /// None without an exit or a tile that sees it.
    pub fn exit_view(&self) -> Option<(f64, f64, f64)> {
        let e = self.exit_data.as_ref()?;
        let (dx, dz) = (e.pos.x - e.normal.x * 0.9, e.pos.z - e.normal.z * 0.9);
        let d = self.bfs(self.tx(e.pos.x), self.tz(e.pos.z));
        let mut best: Option<(f64, f64, f64)> = None;
        for z in 0..self.h {
            for x in 0..self.w {
                let k = d[(z * self.w + x) as usize];
                if !(3..=7).contains(&k) || self.is_wall_tile(x, z) {
                    continue;
                }
                let (wx, wz) = (self.wx(x), self.wz(z));
                let far = (wx - dx).hypot(wz - dz);
                let front = ((wx - dx) * e.normal.x + (wz - dz) * e.normal.z) / far.max(1e-6);
                if front > 0.5 && self.los(wx, wz, e.pos.x, e.pos.z) && best.is_none_or(|b| far > b.2) {
                    best = Some((wx, wz, far));
                }
            }
        }
        best.map(|(x, z, _)| (x, z, (-(dx - x)).atan2(-(dz - z))))
    }

    /// Camera poses on her story: looking at her camp from a few tiles back
    /// along her walk, and standing at one of her sheets looking on along
    /// the trail to the next (the third sheet, or the last). (name, x, z,
    /// yaw, pitch).
    pub fn story_views(&self) -> Vec<(&'static str, f64, f64, f64, f64)> {
        let s = &self.story;
        let mut out = vec![];
        let face = |x: f64, z: f64, tx: f64, tz: f64| (-(tx - x)).atan2(-(tz - z));
        if let Some(c) = &s.camp {
            // out from her wall, as far as still sees her things, looking down
            let back = [2.4, 1.8, 1.2].into_iter().find(|d| {
                let (x, z) = (c.at.x + c.normal.x * d, c.at.z + c.normal.z * d);
                !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, c.at.x, c.at.z)
            });
            let d = back.unwrap_or(1.0);
            let (x, z) = (c.at.x + c.normal.x * d, c.at.z + c.normal.z * d);
            out.push(("camp", x, z, face(x, z, c.at.x, c.at.z), -0.55));
        }
        if s.trail.len() >= 2 {
            let i = 2.min(s.trail.len() - 2);
            let (a, b) = (&s.trail[i], &s.trail[i + 1]);
            // a step behind the sheet, so it is at your feet in the frame
            let yaw = face(a.x, a.z, b.x, b.z);
            let back = [2.2, 1.6, 1.0].into_iter().find(|d| {
                let (x, z) = (a.x + yaw.sin() * d, a.z + yaw.cos() * d);
                !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, a.x, a.z)
            });
            let d = back.unwrap_or(0.6);
            let (x, z) = (a.x + yaw.sin() * d, a.z + yaw.cos() * d);
            out.push(("trail", x, z, yaw, -0.42));
        }
        // and down a trail that is not hers, from her walk
        if let Some(f) = s.false_trails.first().filter(|f| f.trail.len() >= 2) {
            let (a, b) = (&f.trail[0], &f.trail[1]);
            let yaw = face(a.x, a.z, b.x, b.z);
            let back = [2.2, 1.6, 1.0].into_iter().find(|d| {
                let (x, z) = (a.x + yaw.sin() * d, a.z + yaw.cos() * d);
                !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, a.x, a.z)
            });
            let d = back.unwrap_or(0.6);
            out.push(("false", a.x + yaw.sin() * d, a.z + yaw.cos() * d, yaw, -0.42));
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

    /// Measures of the built level for the expressive-range report
    /// (`cargo xtask range`), chosen to be *other* than the generator's
    /// inputs: how the space plays and reads, not what the dials were.
    ///
    /// On the 4-neighbour graph of open tiles: `loops` (independent cycles
    /// per open tile, not counting the trivial ones of open floor), `dead_ends`
    /// (share of tiles with one open neighbour), `depth_mean` / `depth_max` /
    /// `depth_exit` (steps from spawn), `integration` (space syntax: the
    /// inverse mean depth, averaged over sample tiles). And: `rooms` and
    /// `room_area` (mean tiles per room), `sight_mean` / `sight_max` (the
    /// longest straight open run from a tile, capped at 20), `light_density`
    /// (panels per open tile), `prop_density` and `empty_prop_density` (themed
    /// props per open tile, overall and in zones whose purpose is empty),
    /// `themes_per_zone`, `dark_share` (open tiles in dark regions) and
    /// `grime` (mean region grime).
    pub fn level_metrics(&self) -> Value {
        let (w, h) = (self.w, self.h);
        let open = |x: i32, z: i32| self.in_grid(x, z) && !self.is_wall_tile(x, z);
        let idx = |x: i32, z: i32| (z * w + x) as usize;
        let tiles: Vec<(i32, i32)> = (0..h).flat_map(|z| (0..w).map(move |x| (x, z))).filter(|&(x, z)| open(x, z)).collect();
        let v = tiles.len().max(1) as f64;
        let mut edges = 0usize;
        let (mut squares, mut dead) = (0usize, 0usize);
        for &(x, z) in &tiles {
            let n = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|(dx, dz)| open(x + dx, z + dz)).count();
            if n == 1 {
                dead += 1;
            }
            edges += usize::from(open(x + 1, z)) + usize::from(open(x, z + 1));
            if open(x + 1, z) && open(x, z + 1) && open(x + 1, z + 1) {
                squares += 1;
            }
        }
        let bfs = |sx: i32, sz: i32| -> Vec<i32> {
            let mut d = vec![-1i32; (w * h) as usize];
            if !open(sx, sz) {
                return d;
            }
            let mut q = std::collections::VecDeque::from([(sx, sz)]);
            d[idx(sx, sz)] = 0;
            while let Some((x, z)) = q.pop_front() {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, nz) = (x + dx, z + dz);
                    if open(nx, nz) && d[idx(nx, nz)] < 0 {
                        d[idx(nx, nz)] = d[idx(x, z)] + 1;
                        q.push_back((nx, nz));
                    }
                }
            }
            d
        };
        // components, for the cycle count
        let mut comp = 0usize;
        let mut seen = vec![false; (w * h) as usize];
        for &(x, z) in &tiles {
            if !seen[idx(x, z)] {
                comp += 1;
                for (i, d) in bfs(x, z).iter().enumerate() {
                    if *d >= 0 {
                        seen[i] = true;
                    }
                }
            }
        }
        let cycles = (edges + comp).saturating_sub(tiles.len()).saturating_sub(squares);
        let (sx, sz) = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let from_spawn = bfs(sx, sz);
        let reached: Vec<i32> = from_spawn.iter().copied().filter(|d| *d >= 0).collect();
        let depth_mean = reached.iter().map(|&d| d as f64).sum::<f64>() / reached.len().max(1) as f64;
        let depth_max = reached.iter().copied().max().unwrap_or(0);
        let depth_exit =
            self.exit_data.as_ref().map(|e| from_spawn.get(idx(self.tx(e.pos.x), self.tz(e.pos.z))).copied().unwrap_or(-1)).unwrap_or(-1);
        let step = (tiles.len() / 24).max(1);
        let integration = {
            let ints: Vec<f64> = tiles
                .iter()
                .step_by(step)
                .map(|&(x, z)| {
                    let d: Vec<i32> = bfs(x, z).into_iter().filter(|d| *d > 0).collect();
                    let m = d.iter().map(|&d| d as f64).sum::<f64>() / d.len().max(1) as f64;
                    if m > 0.0 {
                        1.0 / m
                    } else {
                        0.0
                    }
                })
                .collect();
            ints.iter().sum::<f64>() / ints.len().max(1) as f64
        };
        let rooms = self.layout.as_ref().map(|l| l.rooms.len()).unwrap_or(0);
        let room_area = self
            .layout
            .as_ref()
            .map(|l| l.rooms.iter().map(|r| ((r.x1 - r.x0 + 1) * (r.z1 - r.z0 + 1)) as f64).sum::<f64>() / l.rooms.len().max(1) as f64)
            .unwrap_or(0.0);
        let sight: Vec<i32> = tiles
            .iter()
            .map(|&(x, z)| {
                [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .map(|(dx, dz)| {
                        let mut r = 0;
                        while r < 20 && open(x + dx * (r + 1), z + dz * (r + 1)) {
                            r += 1;
                        }
                        r
                    })
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let sight_mean = sight.iter().map(|&s| s as f64).sum::<f64>() / v;
        let sight_max = sight.iter().copied().max().unwrap_or(0);
        // themed props, by zone
        let theme_of: std::collections::HashMap<&str, &str> = crate::world::tables::PLACER_THEME.iter().copied().collect();
        let zone_of = |x: i32, z: i32| self.layout.as_ref().and_then(|l| l.zone_of.get(idx(x, z)).copied()).map(|z| z as usize);
        let mut props = 0usize;
        let mut empty_props = 0usize;
        let mut zone_themes: std::collections::BTreeMap<usize, std::collections::BTreeSet<&str>> = Default::default();
        for c in self.world.children() {
            let placer = c.r().user.placer.clone();
            let Some(placer) = placer.filter(|p| p != "place_props") else { continue };
            let p = c.pos();
            let (tx, tz) = (self.tx(p.x), self.tz(p.z));
            props += 1;
            if let Some(zi) = zone_of(tx, tz) {
                if self.zone_purposes.get(zi).is_some_and(|zp| zp.label == "empty") {
                    empty_props += 1;
                }
                if let Some(t) = theme_of.get(placer.as_str()) {
                    zone_themes.entry(zi).or_default().insert(t);
                }
            }
        }
        let empty_tiles = tiles
            .iter()
            .filter(|&&(x, z)| zone_of(x, z).and_then(|zi| self.zone_purposes.get(zi)).is_some_and(|zp| zp.label == "empty"))
            .count();
        let themes_per_zone = zone_themes.values().map(|t| t.len() as f64).sum::<f64>() / zone_themes.len().max(1) as f64;
        let region = |x: i32, z: i32| self.region_map.get(idx(x, z)).and_then(|&r| self.region_info.get(r as usize));
        let dark = tiles.iter().filter(|&&(x, z)| region(x, z).is_some_and(|r| r.flags & crate::visuals::lighting::RF_DARK != 0)).count();
        let grime = tiles.iter().map(|&(x, z)| region(x, z).map(|r| r.grime as f64).unwrap_or(0.0)).sum::<f64>() / v;
        let r4 = |x: f64| (x * 10000.0).round() / 10000.0;
        json!({
            "seed": self.run_seed, "level": self.level_num, "scp": self.level_scp, "weather": self.level_weather.key,
            "open": tiles.len(),
            "loops": r4(cycles as f64 / v), "dead_ends": r4(dead as f64 / v),
            "depth_mean": r4(depth_mean), "depth_max": depth_max, "depth_exit": depth_exit,
            "integration": r4(integration),
            "rooms": rooms, "room_area": r4(room_area),
            "sight_mean": r4(sight_mean), "sight_max": sight_max,
            "light_density": r4(self.panels.len() as f64 / v),
            "prop_density": r4(props as f64 / v),
            "empty_prop_density": if empty_tiles > 0 { json!(r4(empty_props as f64 / empty_tiles as f64)) } else { Value::Null },
            "themes_per_zone": r4(themes_per_zone),
            "dark_share": r4(dark as f64 / v), "grime": r4(grime),
        })
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

    /// Headless `--screen NAME`: put a list up over the level as it is (the
    /// pause list, the question before leaving, Settings with the
    /// brightness line chosen, Controls, or the title)
    pub fn debug_screen(&mut self, name: &str) {
        use crate::ui::menu::Screen;
        // the self-assembly at its crest on a floor of the Building: its
        // surface below you, or you in it (`world::assembly`)
        if (name == "fog-below" || name == "fog-in") && self.building.is_some() {
            let p = self.player.pos;
            let f = self.floor_y_at(p.x, p.z);
            let a = &mut self.assembly;
            a.stage = crate::world::assembly::Stage::Crest;
            a.y = if name == "fog-below" { f - 7.0 } else { f + 3.0 };
            a.crest = a.y;
            a.t = 60.0;
            self.god_mode = true;
            // at the edge with the longest view over the void, looking out and down
            if name == "fog-below" {
                let mut best = None;
                for z in 1..self.h - 1 {
                    for x in 1..self.w - 1 {
                        let g = self.ground[self.t_idx(x, z)];
                        if g.kind != crate::level::archgen::GROUND_FLOOR {
                            continue;
                        }
                        for (d, dd) in crate::level::archgen::DIRS.iter().enumerate() {
                            let n = (1..16)
                                .take_while(|k| {
                                    let (vx, vz) = (x + dd[0] * k, z + dd[1] * k);
                                    self.in_grid(vx, vz) && self.ground[self.t_idx(vx, vz)].kind == crate::level::archgen::GROUND_VOID
                                })
                                .count();
                            if best.is_none_or(|b: (usize, i32, i32, usize)| n > b.0) {
                                best = Some((n, x, z, d));
                            }
                        }
                    }
                }
                if let Some((_, x, z, d)) = best {
                    let dd = crate::level::archgen::DIRS[d];
                    let (wx, wz) = (self.wx(x) + dd[0] as f64 * 1.2, self.wz(z) + dd[1] as f64 * 1.2);
                    self.player.pos = Vec3::new(wx, self.floor_y_at(wx, wz) + EYE, wz);
                    self.player.yaw = (-(dd[0] as f64)).atan2(-(dd[1] as f64));
                    self.player.pitch = -0.4;
                    let f = self.floor_y_at(wx, wz);
                    self.assembly.y = f - 7.0;
                    self.assembly.crest = self.assembly.y;
                }
            }
            return;
        }
        if name == "title" {
            self.state = "menu".into();
            self.open_menu(Screen::Title);
            return;
        }
        // the death card and her last card, as they stand over black
        if name == "dead" || name == "ending" {
            self.ui.fade.set_style("background", if name == "dead" { "#000" } else { "#050403" });
            self.ui.fade.set_opacity(1.0);
            self.state = name.into();
            if name == "dead" {
                self.open_menu(Screen::Dead);
                self.menu.heading = "The hum never stops.".into();
            } else {
                let (t, b) = self.ending_card();
                self.open_menu(Screen::Ending);
                self.menu.heading = t;
                self.menu.note = b;
            }
            return;
        }
        // the notebook open in your hands: her first page, or a typed note
        if name == "notebook" || name == "notebook-typed" {
            self.notebook.insert("f1".into());
            self.lore_found.extend([7usize, 24]);
            if !self.journal_open {
                self.toggle_journal();
            }
            let ids = self.journal_entries_in_order();
            let want = if name == "notebook" { ids.iter().position(|&i| i <= -100) } else { ids.iter().position(|&i| i == 7) };
            self.journal_cursor = want.unwrap_or(0) as f64;
            self.journal_render();
            return;
        }
        self.pause();
        match name {
            "leave" => self.open_menu(Screen::ConfirmLeave),
            "settings" => self.open_menu(Screen::Settings),
            "brightness" => {
                self.open_menu(Screen::Settings);
                self.menu.sel = self.menu.items.iter().position(|i| i.id == "set:brightness").unwrap_or(0);
            }
            "controls" => self.open_menu(Screen::Controls),
            _ => {}
        }
    }
}
