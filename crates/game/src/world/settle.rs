//! Final placement and prop settling.
//!
//! ~700 prop placers were written for 4 m walls standing on tile edges. After
//! they run, `settle_props` reconciles them with the real architecture:
//!   1. faces within reach of anything interactive are pinned to the tile edge
//!   2. anything poking through a low ceiling comes down (wall items) or
//!      lifts that tile's ceiling (doors, tall furniture)
//!   3. ceiling-mounted things follow the ceiling up or down
//!   4. wall-mounted things slide onto the actual wall face; where a partition
//!      does not reach (a thin wall's corner) the face is pinned instead
//!   5. floor clutter caught inside a jamb or tunnel bulge is nudged out
//!
//! Then [`crate::world::placement`] makes sure nothing is sunk, hovering,
//! buried or inside another prop.
use crate::base::numeric::hypot2;
use crate::level::archgen::{self, DIRS as ARCH_DIRS};
use crate::prelude::*;
use crate::world::placement::{self, PlaceObject, PlacementOpts, WallGrid, DEFAULT_TOL};
use std::collections::HashSet;

/// `settle_stats` keys, in reporting order
pub const SETTLE_KEYS: [&str; 8] = ["props", "attached", "moved", "pinned", "ceilingShift", "raised", "pushed", "anchors"];

impl Game {
    /// The world's top-level objects that carry
    /// gameplay (pickups, notes, switches, the TV, phone, clock, figure...)
    pub fn gameplay_objects(&self) -> HashSet<Obj> {
        let world = self.world.clone();
        let mut top: HashSet<Obj> = HashSet::new();
        let mut lift = |o: &Obj| {
            let mut o = o.clone();
            loop {
                let Some(p) = o.parent() else { break };
                if p.ptr_eq(&world) {
                    top.insert(o.clone());
                    break;
                }
                o = p;
            }
        };
        // (interactables hold only a position, no object: nothing to lift)
        for a in &self.almonds {
            lift(&a.g);
        }
        for b in &self.battery_items {
            lift(&b.g);
        }
        for p in &self.papers {
            lift(&p.g);
        }
        for n in &self.lore_notes {
            lift(&n.g);
        }
        // a switch record's only object is its `rocker` (inside the switch's own group)
        for s in &self.light_switches {
            if let Some(r) = s.objs.get("rocker").or(s.g.as_ref()) {
                lift(r);
            }
        }
        // haunted candles (flags and timers) and the safe room (x, z, r,
        // discovered) hold no object: nothing to lift
        // the broken elevator: its light and warning bar
        if let Some(b) = &self.broken_elevator {
            lift(&b.light);
            if let Some(w) = &b.warn_bar {
                lift(w);
            }
        }
        if let Some(t) = &self.tv_data {
            lift(&t.light);
        }
        if let Some(p) = &self.phone_data {
            if let Some(h) = &p.handset {
                lift(h);
            }
        }
        if let Some(c) = &self.clock_data {
            lift(&c.sec_pivot);
        }
        if let Some(m) = &self.man_data {
            lift(&m.g);
        }
        for o in self.world.children() {
            if o.r().user.gameplay {
                top.insert(o);
            }
        }
        top
    }

    /// Pickups (almond water, batteries, paper, notes) whose centre a
    /// jitter or a bulge left inside a wall: moved to the nearest spot clear
    /// of walls (rings of 0.2 m out to 1.6 m, in a fixed order; failing that,
    /// the middle of the nearest open tile that is clear), so a hand can
    /// reach them. The level sweep found one no body could get to
    /// (seed 905325066 level 8, an almond inside a maze wall). No draws.
    pub fn free_buried_pickups(&mut self) {
        let mut objs: Vec<Obj> = self.almonds.iter().chain(self.battery_items.iter()).map(|p| p.g.clone()).collect();
        objs.extend(self.papers.iter().map(|p| p.g.clone()));
        objs.extend(self.lore_notes.iter().map(|n| n.g.clone()));
        for o in objs {
            let p = o.pos();
            if !self.circle_hits_wall(p.x, p.z, 0.05) {
                continue;
            }
            let near = (1..=8)
                .flat_map(|ring| (0..16).map(move |k| (ring as f64 * 0.2, k as f64 / 16.0 * PI * 2.0)))
                .map(|(r, a)| (p.x + a.cos() * r, p.z + a.sin() * r))
                .find(|&(x, z)| !self.circle_hits_wall(x, z, 0.25));
            // a tile the walls fill: the middle of the nearest clear open tile
            let spot = near.or_else(|| {
                let mut best: Option<(f64, f64, f64)> = None;
                for tz in 0..self.h {
                    for tx in 0..self.w {
                        let (x, z) = (self.wx(tx), self.wz(tz));
                        let d = hypot2(x - p.x, z - p.z);
                        if !self.is_wall_tile(tx, tz) && best.is_none_or(|b| d < b.2) && !self.circle_hits_wall(x, z, 0.35) {
                            best = Some((x, z, d));
                        }
                    }
                }
                best.map(|b| (b.0, b.1))
            });
            if let Some((x, z)) = spot {
                o.position().x = x;
                o.position().z = z;
                o.update_world_matrix(true, true);
            }
        }
    }

    pub fn settle_placement(&mut self, from_idx: usize) {
        if self.has_flag("noplace") {
            self.placement_stats.clear();
            self.placement_stats.insert("off".into(), 1.0);
            return;
        }
        let prot = self.gameplay_objects();
        let mut objects = vec![];
        let kids = self.world.children();
        for o in kids.iter().skip(from_idx) {
            let (placer, arch, stat) = {
                let u = &o.r().user;
                (u.placer.clone().unwrap_or_default(), u.arch, u.static_prop)
            };
            if placer.is_empty() || arch || o.is_light() {
                continue;
            }
            let themed_live = placer != "place_props" && !stat;
            objects.push(PlaceObject { obj: o.clone(), protect: prot.contains(o) || themed_live, placer });
        }
        let Some(s) = self.solids.as_ref() else { return };
        let boxes: Vec<f64> = s.boxes.iter().map(|&v| v as f64).collect();
        let grid = WallGrid::from_solids(self.w, self.h, CELL, &boxes, s.count);
        let walls = |bb: &[f64; 6]| grid.query(bb);
        let ceil_at = |x: f64, z: f64| self.ceil_at_world(x, z);
        let in_pool = |x: f64, z: f64| self.near_pool(x, z);
        let st = placement::resolve_placement(&PlacementOpts {
            objects,
            walls: &walls,
            ceil_at: &ceil_at,
            in_pool: if self.pool_bounds.is_some() { Some(&in_pool) } else { None },
            tol: DEFAULT_TOL,
        });
        let mut m = std::collections::BTreeMap::new();
        for (k, v) in [
            ("props", st.props),
            ("lifted", st.lifted),
            ("dropped", st.dropped),
            ("stacked", st.stacked),
            ("unburied", st.unburied),
            ("nudged", st.nudged),
            ("removed", st.removed),
            ("resettled", st.resettled),
            ("skipped", st.skipped),
        ] {
            m.insert(k.to_string(), v as f64);
        }
        self.placement_stats = m;
    }
    /// The pool is built to fit its own basin: neither settle pass may touch it
    pub fn near_pool(&self, x: f64, z: f64) -> bool {
        match &self.pool_bounds {
            Some(p) => x > p.min_x - 0.8 && x < p.max_x + 0.8 && z > p.min_z - 0.8 && z < p.max_z + 0.8,
            None => false,
        }
    }
}

/// A wall face an object hugs: wall tile, side, direction into the wall
/// (DIRS index) and the face's coordinate
#[derive(Clone, Copy, Debug)]
struct Att {
    wt: usize,
    side: usize,
    into: usize,
    bound: f64,
}
struct Info {
    o: Obj,
    bb: Option<Box3>,
    att: Vec<Att>,
}

/// Probes along the face normal from 0.6 tiles out in the room
fn face_delta(s: &archgen::Solids, a: &Att, lat: f64, y: f64) -> Option<f64> {
    let out = if a.into == 0 || a.into == 2 { -1.0 } else { 1.0 }; // sign of the step back into the room
    let into_sign = -out;
    let start = a.bound + out * 0.6;
    let axis_x = a.into < 2;
    let t = if axis_x { archgen::ray_axis(s, start, lat, a.into, 1.8, y) } else { archgen::ray_axis(s, lat, start, a.into, 1.8, y) };
    if !t.is_finite() || t < 0.08 {
        return None;
    }
    Some(((start + into_sign * t) - a.bound) * into_sign) // + = recessed into the wall tile
}

impl Game {
    /// How far the wall face is from where the object hugs it
    /// (tile units, + = recessed), or None where the face is missing / uneven
    fn settle_probe(&self, s: &archgen::Solids, i: &Info, a: &Att) -> Option<f64> {
        let p = i.o.pos();
        let axis_x = a.into < 2;
        let lat = if axis_x { self.to_v(p.z) } else { self.to_u(p.x) };
        let half = match &i.bb {
            Some(bb) => clamp((if axis_x { bb.max.z - bb.min.z } else { bb.max.x - bb.min.x }) / 2.0 / CELL, 0.0, 0.15),
            None => 0.0,
        };
        let y = match &i.bb {
            Some(bb) => clamp((bb.min.y + bb.max.y) / 2.0, 0.15, self.tile_ceil[a.wt] as f64 - 0.15),
            None => clamp(p.y, 0.15, 2.5),
        };
        let d0 = face_delta(s, a, lat, y)?;
        if half > 0.01 {
            for sg in [-1.0, 1.0] {
                match face_delta(s, a, lat + sg * half * 0.9, y) {
                    Some(d) if (d - d0).abs() <= 0.05 => {}
                    _ => return None,
                }
            }
        }
        Some(d0)
    }

    pub fn settle_props(&mut self, from_idx: usize) {
        let mut st: std::collections::BTreeMap<&'static str, f64> = SETTLE_KEYS.iter().map(|&k| (k, 0.0)).collect();
        let mut objs: Vec<Obj> = vec![];
        for o in self.world.children().into_iter().skip(from_idx) {
            if o.is_instanced() || o.r().user.arch {
                continue;
            }
            if !o.is_light() && o.child_count() > 0 && o.pos().length_sq() == 0.0 && o.rot().y == 0.0 {
                for c in o.children() {
                    objs.push(c);
                }
            } else {
                objs.push(o);
            }
        }
        if self.pool_bounds.is_some() {
            let mut i = objs.len();
            while i > 0 {
                i -= 1;
                if objs[i].is_light() {
                    continue;
                }
                let mut b = Box3::default();
                b.set_from_object(&objs[i]);
                if !b.is_empty() && self.near_pool((b.min.x + b.max.x) / 2.0, (b.min.z + b.max.z) / 2.0) {
                    objs.remove(i);
                }
            }
        }
        st.insert("props", objs.len() as f64);
        let mut info: Vec<Info> = objs
            .into_iter()
            .map(|o| {
                let mut bb = None;
                if !o.is_light() {
                    let mut b = Box3::default();
                    b.set_from_object(&o);
                    if !b.is_empty() {
                        bb = Some(b);
                    }
                }
                Info { o, bb, att: vec![] }
            })
            .collect();

        // 1. pinned faces near interactive things
        let mut anchors: HashSet<usize> = HashSet::new();
        let pin_near = |g: &Game, anchors: &mut HashSet<usize>, px: f64, pz: f64, reach: f64| {
            let (x, z) = (g.tx(px), g.tz(pz));
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (ox, oz) = (x + dx, z + dz);
                    if !g.in_grid(ox, oz) || g.tiles[g.t_idx(ox, oz)] != 1 {
                        continue;
                    }
                    for d in 0..4 {
                        let [ddx, ddz] = ARCH_DIRS[d];
                        if g.is_wall_tile(ox + ddx, oz + ddz) {
                            continue;
                        }
                        let (ex, ez) = (g.wx(ox) + ddx as f64 * CELL / 2.0, g.wz(oz) + ddz as f64 * CELL / 2.0);
                        let along = if ddx != 0 { 0f64.max((pz - ez).abs() - CELL / 2.0) } else { 0f64.max((px - ex).abs() - CELL / 2.0) };
                        let across = if ddx != 0 { (px - ex).abs() } else { (pz - ez).abs() };
                        if hypot2(along, across) < reach {
                            anchors.insert(g.t_idx(ox, oz) * 4 + d);
                        }
                    }
                }
            }
        };
        let ipos: Vec<Vec3> = self.interactables.iter().map(|it| it.pos).collect();
        for p in ipos {
            pin_near(self, &mut anchors, p.x, p.z, 1.35);
        }
        // light switches deliberately pin no faces here (their records have no
        // `pos`, only their own x / z)

        // attachments: which wall faces each object hugs (tile units, 0.8 m reach)
        const REACH: f64 = 0.2;
        for it in info.iter_mut() {
            let p = it.o.pos();
            let (u, v) = (self.to_u(p.x), self.to_v(p.z));
            let (xf, zf) = (u.floor(), v.floor());
            let (fu, fv) = (u - xf, v - zf);
            if !self.in_grid(xf, zf) {
                continue;
            }
            let (x, z) = (xf as i32, zf as i32);
            let mut add = |wtx: i32, wtz: i32, side: usize, into: usize, bound: i32| {
                it.att.push(Att { wt: self.t_idx(wtx, wtz), side, into, bound: bound as f64 })
            };
            if self.tiles[self.t_idx(x, z)] == 0 {
                if fu > 1.0 - REACH && self.is_wall_tile(x + 1, z) && self.in_grid(x + 1, z) {
                    add(x + 1, z, 1, 0, x + 1);
                }
                if fu < REACH && self.is_wall_tile(x - 1, z) && self.in_grid(x - 1, z) {
                    add(x - 1, z, 0, 1, x);
                }
                if fv > 1.0 - REACH && self.is_wall_tile(x, z + 1) && self.in_grid(x, z + 1) {
                    add(x, z + 1, 3, 2, z + 1);
                }
                if fv < REACH && self.is_wall_tile(x, z - 1) && self.in_grid(x, z - 1) {
                    add(x, z - 1, 2, 3, z);
                }
            } else {
                const E: f64 = 0.06;
                if fu > 1.0 - E && !self.is_wall_tile(x + 1, z) {
                    add(x, z, 0, 1, x + 1);
                }
                if fu < E && !self.is_wall_tile(x - 1, z) {
                    add(x, z, 1, 0, x);
                }
                if fv > 1.0 - E && !self.is_wall_tile(x, z + 1) {
                    add(x, z, 2, 3, z + 1);
                }
                if fv < E && !self.is_wall_tile(x, z - 1) {
                    add(x, z, 3, 2, z);
                }
            }
            if !it.att.is_empty() {
                *st.get_mut("attached").unwrap() += 1.0;
            }
        }

        // 2 + 3. ceilings
        for it in info.iter_mut() {
            let Some(bb) = it.bb.as_mut() else { continue };
            let p = it.o.pos();
            let t = self.t_idx(self.tx(p.x).clamp(0, self.w - 1), self.tz(p.z).clamp(0, self.h - 1));
            let c = self.tile_ceil[t] as f64;
            if bb.min.y > 0.45 && bb.max.y > WALL_H - 0.2 {
                let dy = c - WALL_H;
                if dy.abs() > 0.005 {
                    it.o.position().y += dy;
                    bb.min.y += dy;
                    bb.max.y += dy;
                    *st.get_mut("ceilingShift").unwrap() += 1.0;
                }
            } else if bb.max.y > c - 0.03 {
                if bb.min.y > 1.0 {
                    let dy = c - 0.05 - bb.max.y;
                    it.o.position().y += dy;
                    bb.min.y += dy;
                    bb.max.y += dy;
                    *st.get_mut("ceilingShift").unwrap() += 1.0;
                } else if bb.max.y < 4.2 {
                    self.tile_ceil[t] = c.max(crate::base::numeric::round_decimals(bb.max.y + 0.12, 2)) as f32;
                    *st.get_mut("raised").unwrap() += 1.0;
                }
            }
        }

        // 4. wall faces
        let s1 = self.build_solids_leaning(Some(&anchors));
        if let Some((s1, lean1)) = &s1 {
            for i in 0..info.len() {
                for k in 0..info[i].att.len() {
                    let a = info[i].att[k];
                    let key = a.wt * 4 + a.side;
                    if anchors.contains(&key) {
                        continue;
                    }
                    // along a partition that leans (its end, the corner it turns
                    // about) a probe finds the wall's side at a graze, not a face
                    let grazes = lean1.iter().any(|w| w.owns(a.wt) && w.along_x == (a.into < 2));
                    if grazes || self.settle_probe(s1, &info[i], &a).is_none() {
                        anchors.insert(key);
                        *st.get_mut("pinned").unwrap() += 1.0;
                    }
                }
            }
        }
        drop(s1);
        self.set_solids_now(Some(&anchors));
        if let Some(solids) = self.solids.take() {
            for it in &info {
                let mut turned = false;
                for a in &it.att {
                    // a partition that leans: onto its face as drawn, and turned with it (once)
                    let p = it.o.pos();
                    let on = |w: &&crate::world::angled::Wall| {
                        let al = w.along(self.to_u(p.x), self.to_v(p.z));
                        w.owns(a.wt)
                            && w.along_x == (a.into >= 2)
                            && w.parts.iter().any(|&(_, q)| al >= w.along(q[0], q[1]) && al <= w.along(q[2], q[3]))
                    };
                    if let Some(wall) = self.angled.iter().find(on) {
                        let side = if a.into == 0 || a.into == 2 { -1.0 } else { 1.0 }; // the room's side of it
                        let d = (wall.face(wall.along(self.to_u(p.x), self.to_v(p.z)), side) - a.bound) * -side;
                        let [dx, dz] = ARCH_DIRS[a.into];
                        let (mx, mz) = (dx as f64 * d * CELL, dz as f64 * d * CELL);
                        {
                            let mut q = it.o.position();
                            q.x += mx;
                            q.z += mz;
                        }
                        *st.get_mut("moved").unwrap() += 1.0;
                        if let (false, Some(bb)) = (turned, &it.bb) {
                            // about its middle
                            let (cx, cz) = ((bb.min.x + bb.max.x) / 2.0 + mx, (bb.min.z + bb.max.z) / 2.0 + mz);
                            let (sn, cs) = wall.phi().sin_cos();
                            let q = it.o.pos();
                            let (rx, rz) = (q.x - cx, q.z - cz);
                            {
                                let mut q = it.o.position();
                                q.x = cx + rx * cs - rz * sn;
                                q.z = cz + rx * sn + rz * cs;
                            }
                            it.o.rotate_on_world_axis(&Vec3::new(0.0, 1.0, 0.0), -wall.phi());
                            turned = true;
                        }
                        continue;
                    }
                    let d = match self.settle_probe(&solids, it, a) {
                        Some(d) if d.abs() >= 0.002 => d,
                        _ => continue,
                    };
                    let [dx, dz] = ARCH_DIRS[a.into];
                    {
                        let mut p = it.o.position();
                        p.x += dx as f64 * d * CELL;
                        p.z += dz as f64 * d * CELL;
                    }
                    *st.get_mut("moved").unwrap() += 1.0;
                }
            }
            // 5. floor clutter inside a jamb / bulge: walk it toward its tile centre
            for it in &info {
                let Some(bb) = &it.bb else { continue };
                if !it.att.is_empty() || bb.min.y > 1.6 {
                    continue;
                }
                let p = it.o.pos();
                let r = clamp((bb.max.x - bb.min.x).max(bb.max.z - bb.min.z) / 2.0, 0.05, 0.5);
                if !archgen::circle_hits(&solids, self.to_u(p.x), self.to_v(p.z), r / CELL, 1.6) {
                    continue;
                }
                let (nx, nz) = self.nav_tile_at(p.x, p.z);
                let (cx, cz) = (self.wx(nx), self.wz(nz));
                for k in 1..=12 {
                    let f = k as f64 / 12.0;
                    let (qx, qz) = (p.x + (cx - p.x) * f, p.z + (cz - p.z) * f);
                    if !archgen::circle_hits(&solids, self.to_u(qx), self.to_v(qz), r / CELL, 1.6) {
                        {
                            let mut pp = it.o.position();
                            pp.x = qx;
                            pp.z = qz;
                        }
                        *st.get_mut("pushed").unwrap() += 1.0;
                        break;
                    }
                }
            }
            self.solids = Some(solids);
        }
        st.insert("anchors", anchors.len() as f64);
        self.settle_stats = st.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    }
}
