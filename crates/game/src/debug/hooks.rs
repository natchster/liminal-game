//! Level-inspection hooks used by tools and tests: `rand_trace`,
//! `rand_marks`, `load_level`, `prop_list`, `zone_views` and `level_dump`.
//! `level_dump()` dumps a level's build trace, grid, zone purposes, regions,
//! props and placed objects as JSON (`backrooms --dump-level`,
//! tests/reference_levels.rs).
use crate::prelude::*;
use serde_json::{Value, json};

fn r6(x: f64) -> f64 {
    if x.is_finite() { crate::base::numeric::round_decimals(x, 6) } else { x }
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
            let kind = o.user().s.get("kind").cloned();
            out.push(json!({
                "k": k, "placer": placer, "kind": kind, "pos": v3(&o.pos()), "rot": [r6(r.x), r6(r.y), r6(r.z)],
                "min": v3(&bb.min), "max": v3(&bb.max), "meshes": meshes, "interact": !o.user().static_prop,
            }));
        }
        out
    }

    /// The prop census (`--prop-census`): for each kind of placed prop (a
    /// core prop's kind, else its placer), up to `per_label` of them, a
    /// camera that sees it whole: (label, the prop's index in the world,
    /// eye x, y, z, yaw, pitch). The camera stands off the prop's face
    /// where it can (else round it), clear of the walls and in sight of it,
    /// far enough for it to fill about half the frame, its eye lowered for
    /// a small thing as you would crouch to it.
    pub fn census_shots(&self, per_label: usize) -> Vec<(String, usize, f64, f64, f64, f64, f64)> {
        let mut by: std::collections::BTreeMap<String, Vec<(usize, Obj)>> = Default::default();
        for (k, o) in self.world.children().iter().enumerate() {
            let u = o.user();
            let Some(placer) = u.placer.clone() else { continue };
            let label = u.s.get("kind").map(|k| format!("core_{k}")).unwrap_or(placer);
            drop(u);
            by.entry(label).or_default().push((k, o.clone()));
        }
        let mut out = vec![];
        // a barricade from 3.2 m along the way on each side (`PROPS.md` 2):
        // its own slab hides it from the framing below
        let bar = self.barricades.clone();
        if let Some(objs) = by.remove("core_barricades") {
            for (i, b) in bar.iter().enumerate() {
                let (cx, cz) = ((b.x0 + b.x1) / 2.0, (b.z0 + b.z1) / 2.0);
                let k = objs
                    .iter()
                    .map(|(k, o)| (*k, o.pos()))
                    .min_by(|a, c| (a.1.x - cx).hypot(a.1.z - cz).total_cmp(&(c.1.x - cx).hypot(c.1.z - cz)))
                    .map(|a| a.0)
                    .unwrap_or(0);
                for side in [1.0, -1.0] {
                    let (x, z) = if b.along_x { (cx + side * 3.2, cz) } else { (cx, cz + side * 3.2) };
                    if self.circle_hits_wall(x, z, 0.2)
                        || out.iter().filter(|s: &&(String, usize, f64, f64, f64, f64, f64)| s.0 == "core_barricades").count()
                            >= per_label * 2
                    {
                        continue;
                    }
                    let y = self.floor_y_at(x, z) + crate::prelude::EYE;
                    let yaw = (-(cx - x)).atan2(-(cz - z));
                    let pitch = (1.1 - crate::prelude::EYE).atan2(3.2);
                    let _ = i;
                    // (each side its own shot: the object's index, then 1 or 2)
                    out.push(("core_barricades".to_string(), k * 10 + if side > 0.0 { 1 } else { 2 }, x, y, z, yaw, pitch));
                }
            }
        }
        for (label, objs) in by {
            let n = objs.len();
            let picks: Vec<usize> = if per_label >= n { (0..n).collect() } else { (0..per_label).map(|i| i * n / per_label).collect() };
            for i in picks {
                let (k, o) = &objs[i];
                o.update_world_matrix(true, true);
                let mut bb = Box3::empty();
                bb.set_from_object(o);
                if bb.is_empty() {
                    continue;
                }
                let c = Vec3::new((bb.min.x + bb.max.x) / 2.0, (bb.min.y + bb.max.y) / 2.0, (bb.min.z + bb.max.z) / 2.0);
                let s = (bb.max.x - bb.min.x).max(bb.max.y - bb.min.y).max(bb.max.z - bb.min.z).max(0.05);
                let floor = self.floor_y_at(c.x, c.z);
                let ry = o.rot().y;
                let face = (ry.sin(), ry.cos());
                let mut dirs: Vec<(f64, f64)> = (0..16).map(|j| j as f64 * PI / 8.0).map(|a| (a.sin(), a.cos())).collect();
                dirs.sort_by(|a, b| (b.0 * face.0 + b.1 * face.1).total_cmp(&(a.0 * face.0 + a.1 * face.1)));
                let mut pose = None;
                'find: for scale in [1.0, 0.75, 0.55] {
                    let d = (s * 1.8 * scale).clamp(0.6, 6.0) + (bb.max.x - bb.min.x).max(bb.max.z - bb.min.z) / 2.0;
                    for &(dx, dz) in &dirs {
                        let (x, z) = (c.x + dx * d, c.z + dz * d);
                        // (in sight of its near side: a barricade's own slab hides its middle)
                        if !self.circle_hits_wall(x, z, 0.2) && self.los(x, z, c.x + dx * 0.4, c.z + dz * 0.4) {
                            pose = Some((x, z, d));
                            break 'find;
                        }
                    }
                }
                let Some((x, z, d)) = pose else { continue };
                let y = (c.y + 0.25 + 0.35 * d).clamp(floor + 0.45, floor + crate::prelude::EYE);
                let yaw = (-(c.x - x)).atan2(-(c.z - z));
                let pitch = (c.y - y).atan2(d).clamp(-1.3, 1.0);
                out.push((label.clone(), *k, x, y, z, yaw, pitch));
            }
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
        // the first of each kind of her marks, from a few steps short of it,
        // looking at it
        let mut seen = vec![];
        for m in &s.trail {
            if seen.contains(&m.kind) {
                continue;
            }
            seen.push(m.kind);
            let (sx, sz) = m.seen_at();
            let (fx, fz) = (-m.yaw.sin(), -m.yaw.cos());
            // a mark on a wall from out in the room, a little short of it;
            // one on the floor from a few steps short of it
            let at = |d: f64| {
                if m.kind.on_wall() {
                    (m.x + m.normal.0 * d * 0.85 - fx * d * 0.5, m.z + m.normal.1 * d * 0.85 - fz * d * 0.5)
                } else {
                    (sx - fx * d, sz - fz * d)
                }
            };
            let back = [2.6, 2.0, 1.4, 0.9].into_iter().find(|&d| {
                let (x, z) = at(d);
                !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, sx, sz)
            });
            let (x, z) = at(back.unwrap_or(0.6));
            let (dx, dz) = (m.x - x, m.z - z);
            let pitch = ((m.y + 0.02) - crate::prelude::EYE).atan2(dx.hypot(dz).max(0.3)).clamp(-1.2, 0.3);
            out.push((m.kind.view_name(), x, z, face(x, z, m.x, m.z), pitch));
        }
        // each hiding place (`secrets`): a place on a wall from out in the
        // room, a ceiling tile from under its edge, a thing on the floor
        // from a step or two off
        for (k, s) in self.secrets.list.iter().enumerate() {
            let p = &s.plan;
            let kind = match p.kind {
                crate::secrets::PlaceKind::Tile => "secret_tile",
                crate::secrets::PlaceKind::Vent => "secret_vent",
                crate::secrets::PlaceKind::Drawer => "secret_drawer",
                crate::secrets::PlaceKind::Radiator => "secret_radiator",
                crate::secrets::PlaceKind::Cell => "secret_cell",
                crate::secrets::PlaceKind::Hole => "secret_hole",
                crate::secrets::PlaceKind::Corner => "secret_corner",
                crate::secrets::PlaceKind::Table => "secret_table",
                crate::secrets::PlaceKind::Hook => "secret_hook",
            };
            // (a view's name lives as long as the program: a debug view's few bytes)
            let name: &'static str = Box::leak(format!("{kind}{k}").into_boxed_str());
            let (tx, tz) = (self.wx(p.tile.0), self.wz(p.tile.1));
            let (ox, oz) = if p.normal != (0.0, 0.0) {
                p.normal
            } else {
                let (dx, dz) = (tx - p.x, tz - p.z);
                let l = dx.hypot(dz).max(1e-6);
                (dx / l, dz / l)
            };
            let at = [2.2, 1.7, 1.2]
                .into_iter()
                .map(|d| (p.x + ox * d, p.z + oz * d))
                .find(|&(x, z)| !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, p.x + ox * 0.3, p.z + oz * 0.3));
            let Some((x, z)) = at else { continue };
            let y = match p.kind {
                crate::secrets::PlaceKind::Tile => p.y - 0.1,
                crate::secrets::PlaceKind::Drawer => 0.6,
                _ => p.y.max(0.05),
            };
            let d = (p.x - x).hypot(p.z - z).max(0.3);
            let pitch = (y - crate::prelude::EYE).atan2(d).clamp(-1.2, 1.0);
            out.push((name, x, z, face(x, z, p.x, p.z), pitch));
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
        // the rooms off the walks (G4): from the walk outside the doorway,
        // looking in
        if let Some(plan) = &self.building {
            for (k, r) in plan.rooms.iter().enumerate().take(3) {
                let (x, z) = r.tile;
                let (dx, dz) = (crate::level::archgen::DIRS[r.door][0] as f64, crate::level::archgen::DIRS[r.door][1] as f64);
                let (cx, cz) = (self.wx(x) + dx * 4.4, self.wz(z) + dz * 4.4);
                out.push((["room0", "room1", "room2"][k], cx, cz, face(cx, cz, self.wx(x), self.wz(z)), -0.2));
            }
        }
        out
    }

    /// Camera poses along her walk (x, z, yaw): every eighth step from the
    /// fifth, at most `n`, in the middle of its tile looking two tiles on.
    /// The walk is planned before her marks, so the same poses serve a
    /// build before and after a change to them (`--views-at`).
    pub fn walk_views(&self, n: usize) -> Vec<(f64, f64, f64)> {
        let w = &self.story.walk;
        (5..w.len().saturating_sub(2))
            .step_by(8)
            .take(n)
            .map(|k| {
                let (a, b) = (w[k], w[k + 2]);
                let (x, z) = (self.wx(a.0), self.wz(a.1));
                (x, z, (-(self.wx(b.0) - x)).atan2(-(self.wz(b.1) - z)))
            })
            .collect()
    }

    /// Camera poses facing each hiding place from 3 m off it (x, z, yaw),
    /// for frames of the same spot with and without it (`--views-at`)
    pub fn secret_poses(&self) -> Vec<(f64, f64, f64)> {
        let mut out = vec![];
        for s in &self.secrets.list {
            let p = &s.plan;
            let (tx, tz) = (self.wx(p.tile.0), self.wz(p.tile.1));
            let (ox, oz) = if p.normal != (0.0, 0.0) {
                p.normal
            } else {
                let (dx, dz) = (tx - p.x, tz - p.z);
                let l = dx.hypot(dz).max(1e-6);
                (dx / l, dz / l)
            };
            let at = [3.0, 2.4, 1.8]
                .into_iter()
                .map(|d| (p.x + ox * d, p.z + oz * d))
                .find(|&(x, z)| !self.circle_hits_wall(x, z, 0.3) && self.los(x, z, p.x + ox * 0.3, p.z + oz * 0.3));
            if let Some((x, z)) = at {
                out.push((x, z, (-(p.x - x)).atan2(-(p.z - z))));
            }
        }
        out
    }

    /// Camera poses on the partitions that lean (`world::angled`), the
    /// steepest first (at most `n`): out in the room beside one, level with
    /// its free half, looking back at it and what it turns about, so its
    /// angle shows against the ceiling's grid. (x, z, yaw) in
    /// world coordinates; a pose where a body does not fit, or that does not
    /// see what it looks at, is left out.
    pub fn lean_views(&self, n: usize) -> Vec<(f64, f64, f64)> {
        let mut walls: Vec<&crate::world::angled::Wall> = self.angled.iter().collect();
        walls.sort_by(|a, b| b.theta.abs().total_cmp(&a.theta.abs()));
        let mut out = vec![];
        for w in walls {
            // the free end: the end of its parts farthest from the pivot
            let ends = w.parts.iter().flat_map(|&(_, p)| [w.along(p[0], p[1]), w.along(p[2], p[3])]);
            let Some(far) = ends.max_by(|a, b| (a - w.pivot).abs().total_cmp(&(b - w.pivot).abs())) else { continue };
            for side in [1.0, -1.0] {
                // 5 m out from its face, beyond its free end's middle; looking at its pivot's third
                let (al, look) = (w.pivot + (far - w.pivot) * 0.85, w.pivot + (far - w.pivot) * 0.3);
                let (ac, tac) = (w.face(al, side) + side * 1.25, w.face(look, side));
                let (u, v, tu, tv) = if w.along_x { (al, ac, look, tac) } else { (ac, al, tac, look) };
                let (x, z) = ((u - self.w as f64 / 2.0) * CELL, (v - self.h as f64 / 2.0) * CELL);
                let (tx, tz) = ((tu - self.w as f64 / 2.0) * CELL, (tv - self.h as f64 / 2.0) * CELL);
                if self.circle_hits_wall(x, z, 0.4) || !self.los(x, z, tx, tz) {
                    continue;
                }
                out.push((x, z, (-(tx - x)).atan2(-(tz - z))));
                break;
            }
            if out.len() >= n {
                break;
            }
        }
        out
    }

    /// Camera poses on the vending machines (`core_props::vending`): for
    /// each, one square in front of it 2.2 m out, and one from 1.2 m off its
    /// face and to its left, looking at its glass (its state: `vending_states`). (x, z, yaw) in world
    /// coordinates; a pose where a body does not fit, or that does not see
    /// the machine's face, is left out.
    /// Each vending machine: lit or dead
    pub fn vending_states(&self) -> Vec<&'static str> {
        self.vending.iter().map(|m| if m.powered { "lit" } else { "dead" }).collect()
    }
    /// Camera poses on the windows (`core_props::window`): for each row,
    /// one square in front of its middle 2.4 m out from its wall, and one
    /// 1.6 m out and 1.1 m to its side looking at its glass. (x, z, yaw) in
    /// world coordinates; a pose where a body does not fit, or that does not
    /// see the glass, is left out.
    pub fn window_views(&self) -> Vec<(f64, f64, f64)> {
        let mut out = vec![];
        for w in &self.windows {
            let k = w.lights.len() / 2;
            let target = w.light_centre(k);
            for (x, z) in [(0.0, 2.4), (-1.1, 1.6)] {
                let p = w.world((w.lights[k].rect[0] + w.lights[k].rect[1]) / 2.0 + x, 0.0, w.depth + z);
                let face = w.world((w.lights[k].rect[0] + w.lights[k].rect[1]) / 2.0, 0.0, w.depth + 0.15);
                if self.circle_hits_wall(p.x, p.z, 0.3) || !self.los(p.x, p.z, face.x, face.z) {
                    continue;
                }
                out.push((p.x, p.z, (-(target.x - p.x)).atan2(-(target.z - p.z))));
            }
        }
        out
    }

    pub fn vending_views(&self) -> Vec<(f64, f64, f64)> {
        let mut out = vec![];
        for m in &self.vending {
            let target = m.world(-0.1, 0.0, crate::core_props::vending::model::D / 2.0);
            for (x, z) in [(0.0, 2.2), (-0.9, 1.25)] {
                let p = m.world(x, 0.0, crate::core_props::vending::model::D / 2.0 + z);
                let face = m.world(0.0, 0.0, crate::core_props::vending::model::D / 2.0 + 0.1);
                if self.circle_hits_wall(p.x, p.z, 0.3) || !self.los(p.x, p.z, face.x, face.z) {
                    continue;
                }
                out.push((p.x, p.z, (-(target.x - p.x)).atan2(-(target.z - p.z))));
            }
        }
        out
    }

    /// Plays every lit vending machine to a state, for its frames: `hang`
    /// (a coin in, for a bag on its second tray or else its first stocked
    /// coil: the good hangs at the glass), `bin` (and then rocked until it
    /// falls into the bin), `fall` (rocked past its balance), `dark` (in its
    /// rhythm's dark beat), `print` (a dead one's hand on its glass),
    /// `inside` (a dead one's flap held up from inside, the hand coming up).
    /// Time runs for the machines alone.
    pub fn vending_demo(&mut self, what: &str) {
        use crate::core_props::vending::rules::{LIFT_T, SHOVE_T, VEND_T, Verb};
        let run = |g: &mut Game, i: usize, secs: f64| {
            for _ in 0..(secs * 60.0) as usize {
                g.perf_t += 1.0 / 60.0;
                g.vending_frame(i, 1.0 / 60.0);
            }
        };
        for i in 0..self.vending.len() {
            // a dead one: its hand print (rule 9)
            if what == "print" {
                if let Some(h) = &self.vending[i].print {
                    h.set_visible(true);
                    h.mat().m().opacity = 1.0;
                }
                continue;
            }
            // a dead one: its flap held up from inside (rule 9)
            if what == "inside" {
                if !self.vending[i].powered {
                    self.vending[i].state.lifted = Some(self.perf_t - LIFT_T * 0.6);
                    self.vending_frame(i, 1.0 / 60.0);
                }
                continue;
            }
            if !self.vending[i].powered {
                continue;
            }
            match what {
                // in its dark beat (rule 1)
                "dark" => {
                    self.vending[i].state.rhythm = (0.001, 1e9, 0.0);
                    run(self, i, 0.1);
                    continue;
                }
                // rocked past its balance (rule 7)
                "fall" => {
                    self.vending[i].state.coin_given = true;
                    for _ in 0..crate::core_props::vending::rules::FALL_ON {
                        self.vending_use(i, Verb::Rock);
                        run(self, i, SHOVE_T + 0.2);
                    }
                    run(self, i, crate::core_props::vending::rules::TOPPLE_T + 0.3);
                    continue;
                }
                _ => {}
            }
            let d = crate::core_props::vending::paint::designs();
            let m = &self.vending[i];
            // a bag on the second tray, where it shows best; else any
            let pick = (0..m.places.len())
                .find(|&k| {
                    m.places[k].tray == 1
                        && m.stock[k].count > 0
                        && d[m.stock[k].design].kind == crate::core_props::vending::paint::Ware::Bag
                })
                .or_else(|| (0..m.places.len()).find(|&k| m.stock[k].count > 0));
            let Some(k) = pick else { continue };
            self.vending[i].state.gaze = Some(k);
            self.player.coins += 1;
            self.vending_use(i, Verb::InsertCoin);
            run(self, i, VEND_T + 0.8);
            if what == "bin" {
                for _ in 0..self.vending[i].state.drop_on {
                    self.vending_use(i, Verb::Rock);
                    run(self, i, SHOVE_T + 0.6);
                }
            }
        }
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
                    if m > 0.0 { 1.0 / m } else { 0.0 }
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
            // the settled walls a body meets (`settle_props` pins a face where a
            // wall prop's probe misses, and a pillar's end becomes a wall): how
            // many boxes, and a hash of every coordinate
            "solidBoxes": self.solids.as_ref().map(|s| s.count).unwrap_or(0),
            "solidsHash": self.solids.as_ref().map(|s| {
                let mut x: u64 = 0xcbf2_9ce4_8422_2325;
                for v in &s.boxes[..s.count * 8] {
                    for b in v.to_bits().to_le_bytes() {
                        x ^= u64::from(b);
                        x = x.wrapping_mul(0x0100_0000_01b3);
                    }
                }
                format!("{x:016x}")
            }).unwrap_or_default(),
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

    /// The creature bench's scene (`--bench-creatures`): `per_kind` of every
    /// kind that roams the rooms, on open tiles 6–14 m round you, all of them
    /// hunting you, and you in god mode. → how many were placed. Placement
    /// draws from the native stream; the level's build is untouched.
    pub fn bench_pack(&mut self, per_kind: usize) -> usize {
        self.god_mode = true;
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        // open tiles 6–14 m round you, the ring widened (to 30 m) until every
        // creature of the pack has one (level 4's arrival has only five
        // within 14 m: the pack had stopped there, leaving kinds out)
        let wanted = per_kind * crate::entities::ROAMING_KINDS.len() + 1;
        let mut spots = vec![];
        let mut far = 14.0;
        while spots.len() < wanted && far <= 30.0 {
            spots.clear();
            for z in 1..self.h - 1 {
                for x in 1..self.w - 1 {
                    let d = ((self.wx(x) - px).powi(2) + (self.wz(z) - pz).powi(2)).sqrt();
                    if !self.is_wall_tile(x, z) && (6.0..far).contains(&d) {
                        spots.push((x, z));
                    }
                }
            }
            far += 4.0;
        }
        let mut placed = 0;
        for kind in crate::entities::ROAMING_KINDS {
            for _ in 0..per_kind {
                if spots.is_empty() {
                    return placed;
                }
                let (x, z) = spots.remove(with_native(|| (rnd() * spots.len() as f64) as usize).min(spots.len() - 1));
                let ei = self.add_entity(kind, x, z, 0);
                self.entities[ei].freeze = 0.0;
                let d = hypot2(self.entities[ei].pos().x - px, self.entities[ei].pos().z - pz);
                self.entity_notice(ei, d);
                placed += 1;
            }
        }
        // and the one in the corner (at most one a level, as on a level), coming
        if let Some((x, z)) = spots.pop() {
            let ei = self.add_entity("corner", x, z, 0);
            self.entities[ei].freeze = 0.0;
            let now = self.perf_t;
            if let Some(c) = self.creature_mut(ei) {
                c.brain.set(crate::entities::brain::Mode::Hunt, "the bench sends it", now);
                c.ease.insert("knows_until", now + 1e9);
            }
            placed += 1;
        }
        placed
    }

    /// Keep one creature of `kind` standing `dist` m in front of the
    /// camera, facing it, and send the others away (`--showcase`, for
    /// looking at a creature headless). Adds one if the level has none.
    pub fn showcase_entity(&mut self, kind: &str, dist: f64) {
        // the one behind the glass as you meet it: out of its window
        if kind == "glass" && crate::entities::glass::showcase(self, dist).is_some() {
            return;
        }
        let ei = match self.entities.iter().position(|e| e.kind == kind) {
            Some(i) => i,
            None => {
                let (tx, tz) = self.nav_tile_at(self.player.pos.x, self.player.pos.z);
                self.add_entity(kind, tx, tz, 0)
            }
        };
        let yaw = self.player.yaw;
        let (fx, fz) = (-yaw.sin(), -yaw.cos());
        // the Watcher among its furniture: a wingback and a chair under the
        // same old sheets, beside it and a little behind (as you see it)
        if kind == "watcher" && !self.showcase_company {
            self.showcase_company = true;
            let (cx, cz) = (self.player.pos.x + fx * dist, self.player.pos.z + fz * dist);
            let (rx, rz) = (-fz, fx);
            for (name, side, back, turn) in [("sheet_wingback", 1.3, 0.7, 0.6), ("sheet_chair", -1.4, 1.1, -0.9)] {
                if let Some(rig) = crate::entities::body::build_body(name, &crate::entities::watcher::material) {
                    rig.root.position().set(cx + rx * side + fx * back, 0.0, cz + rz * side + fz * back);
                    rig.root.rotation().y = yaw + turn;
                    self.world.add(&rig.root);
                }
            }
        }
        for (k, e) in self.entities.iter_mut().enumerate() {
            if k == ei {
                e.g.set_pos(self.player.pos.x + fx * dist, 0.0, self.player.pos.z + fz * dist);
                // (the one in the corner as you meet it: its back to you)
                e.yaw = if kind == "corner" { yaw + PI } else { yaw };
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
        // a cutscene with your hands (the bars, the HUD gone), and with a
        // key pressed and the skip key held half its time (the way out)
        if name == "cutscene" || name == "cutscene-skip" {
            let p = self.player.pos;
            let (s, c) = self.player.yaw.sin_cos();
            let at = [p.x - s * 2.0, self.floor_y_at(p.x, p.z), p.z - c * 2.0];
            let thing = crate::cutscene::play::Frame::thing(at, self.player.yaw);
            let anchors = crate::cutscene::play::Anchors { thing: Some(thing), killer: None, seen_as: None };
            self.play_cutscene_data(glance(), anchors);
            if name == "cutscene-skip"
                && let Some(q) = self.cut.as_mut()
            {
                q.hint = crate::cutscene::play::HINT_FOR;
                q.skip_held = crate::cutscene::play::SKIP_HOLD * 0.5;
            }
            return;
        }
        // the one behind the glass's window: its blinds let down (at once),
        // or its lamp hung low as when it has learned (`--showcase glass:…`)
        if name == "blind-down" || name == "lamp-low" {
            if let Some(b) = self.behind_glass.clone() {
                if name == "blind-down" {
                    let now = self.perf_t;
                    for h in 0..self.windows[b.win].blinds.len() {
                        let k = self.windows[b.win].blinds[h].lights[0];
                        let from = self.windows[b.win].lights[k].blind.unwrap_or(crate::core_props::window::Blind {
                            down: 0.0,
                            open: 0.6,
                            skew: 0.0,
                        });
                        let to = crate::core_props::window::Blind { down: 1.0, open: 0.05, skew: 0.0 };
                        self.windows[b.win].blinds[h].moving = Some((from, to, now - 2.0));
                    }
                } else {
                    crate::core_props::window::view::set_lamp_drop(self, b.win, crate::entities::glass::LAMP_DROP);
                    crate::core_props::window::view::rethrow(self, b.win);
                }
            }
            return;
        }
        // the place's answer to a cry, as footprints, all of them down and
        // wet, and you turned to look along them (`show_the_way`)
        if name == "way-prints" {
            self.last_way = Some(crate::base::records::WayAnswer::Lights);
            let path = self.way_to_the_door();
            if let Some(&t) = path.get(3).or(path.last()) {
                let (dx, dz) = (self.wx(t.0) - self.player.pos.x, self.wz(t.1) - self.player.pos.z);
                self.player.yaw = (-dx).atan2(-dz);
                self.player.pitch = -0.38;
            }
            self.state = "playing".into();
            self.show_the_way();
            self.now_ms += 20_000.0;
            self.run_scheduled();
            for a in &self.way_prints {
                a.mesh.mat().set_opacity(crate::player::way_print_opacity(crate::player::WAY_PRINT_LIFE - 2.0));
            }
            return;
        }
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
        // sealed in the first cell with the fog over it, something answered at
        // the door: the moment before its first pull, nothing of the door's
        // to use yet (`cell-answered`), its wheel turning (`cell`), or the
        // door given and the fog coming in (`cell-gave`), seen from the
        // middle of the cell, the lamp in hand (`world::cell_knock`)
        if matches!(name, "cell" | "cell-answered" | "cell-gave") && self.building.is_some() && !self.cell_doors.is_empty() {
            use crate::world::cell_knock::{AtTheDoor, Visit};
            let d = self.cell_doors[0].clone();
            let (tx, tz) = d.tile;
            let (cx, cz) = (self.wx(tx), self.wz(tz));
            let h = self.floor_y_of(tx, tz);
            // a step behind the cell's middle, looking at the door
            let (dx, dz) = (d.at.x - cx, d.at.z - cz);
            let l = dx.hypot(dz).max(1e-6);
            let (px, pz) = (cx - dx / l * 0.6, cz - dz / l * 0.6);
            self.player.pos = Vec3::new(px, h + EYE, pz);
            self.player.yaw = (-dx).atan2(-dz) + 0.08;
            self.player.pitch = -0.2;
            // (the cell is dark inside: its lamp hangs outside, over the door)
            self.player.flashlight_on = true;
            self.god_mode = true;
            self.turn_cell_wheel(0, true);
            // (shut at once, and drawn so: a door at rest is not redrawn)
            let d = &mut self.cell_doors[0];
            d.open = 0.0;
            d.turned = d.turn_to;
            let yaw = d.leaf.r().user.get("yaw");
            d.leaf.rotation().y = yaw;
            d.wheel.rotation().z = d.turned;
            let a = &mut self.assembly;
            a.stage = crate::world::assembly::Stage::Crest;
            // (over the cell's roof while it is sealed; to the knee once it gave)
            a.y = h + if name == "cell-gave" { 0.7 } else { 5.0 };
            a.crest = a.y;
            a.t = 60.0;
            a.knock_t = 60.0;
            let wait = if name == "cell-answered" { 60.0 } else { 0.0 };
            a.door = AtTheDoor { visit: Visit::Knocking, cell: Some(0), patience: 9, wait, tries: 60.0, ..Default::default() };
            self.answer_the_door(0);
            if name == "cell-gave" {
                self.assembly.door.give = 1.0;
                self.assembly.door.t = 0.0;
                self.update_at_the_door(1.0 / 60.0);
                self.cell_doors[0].open = crate::world::cell_knock::GAVE_OPEN * 0.8;
            }
            return;
        }
        // the Loop's seated figure on the lap it comes, from 2.5 m (or 1.6:
        // nearer, as it is when it has come for you)
        if name == "loop-figure" || name == "loop-figure-near" {
            let dist = if name == "loop-figure" { 2.5 } else { 1.6 };
            self.debug_loop_figure(crate::modes::loop_rooms::LOOP::FIGURE_LAP, dist);
            return;
        }
        // a figure far across the void, looked at (`world::far_figures`)
        if name == "figure" && self.building.is_some() {
            self.show_far_figure();
            return;
        }
        // the Well's lift cage, from the walk nearest the arrival it is seen from
        if name == "lift" {
            let (Some(at), Some(plan)) = (self.lift_at, self.building.clone()) else { return };
            let mut best: Option<(i32, Vec3)> = None;
            for z in 0..self.h {
                for x in 0..self.w {
                    if !self.ground[self.t_idx(x, z)].walkable() {
                        continue;
                    }
                    let eye = Vec3::new(self.wx(x), self.floor_y_of(x, z) + EYE, self.wz(z));
                    let k = (x - plan.spawn.0).abs() + (z - plan.spawn.1).abs();
                    let seen = self.view_clear(eye, Vec3::new(at.x, at.y + 1.3, at.z));
                    if seen && best.is_none_or(|b| k < b.0) {
                        best = Some((k, eye));
                    }
                }
            }
            if let Some((_, eye)) = best {
                self.player.pos = eye;
                let (dx, dy, dz) = (at.x - eye.x, at.y + 1.3 - eye.y, at.z - eye.z);
                // a little to its side, clear of the crosshair and a prompt
                self.player.yaw = (-dx).atan2(-dz) - 0.12;
                self.player.pitch = dy.atan2((dx * dx + dz * dz).sqrt());
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

/// A glance at a thing two metres ahead and back (`--screen cutscene`)
fn glance() -> crate::cutscene::Cutscene {
    let key = |t: f64, anchor: &str, pos: [f64; 3], look: Option<[f64; 3]>, sway: f64| {
        serde_json::json!({ "t": t, "anchor": anchor, "pos": pos, "look": look, "look_at": null, "yaw": 0.0, "pitch": 0.0, "roll": 0.0,
            "fov": 0.0, "ease": "in_out", "cut": false, "sway": sway, "walk": 0.0, "jolt": false })
    };
    let v = serde_json::json!({
        "id": "glance", "about": "a glance at a thing two metres ahead", "world": "live", "then": "back", "skip": "hold",
        "breaks_off": false, "length": 3.4, "blend_in": 0.0, "blend_out": 1.0, "repeat_from": 0.0, "stages": ["ahead"], "finally": [],
        "bars": { "in": 0.0, "out": 2.9 },
        "camera": [key(0.0, "start", [0.0; 3], None, 0.0), key(1.2, "thing", [0.0, 1.5, 1.1], Some([0.0, 1.2, 0.0]), 0.3), key(2.4, "eyes", [0.0; 3], None, 0.0)],
        "fade": [], "sound": [], "event": [], "say": [],
    });
    crate::cutscene::Cutscene::from_json(&v).expect("the glance")
}
