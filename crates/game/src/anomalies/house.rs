//! The Borrowed House (a level whose SCP is `house`)
//! (inspired by SCP-1983 "Doorway to Nowhere" by DrEverettMann, CC BY-SA 3.0
//! — docs/LORE.md). Shadows that are fast and deadly while you are lit and
//! slow smoke in the dark; a nest of lamps near the middle that keeps the
//! front door from latching until every lamp is out.
//!
//! Entity keys (`Entity.num`): `solid`, `agit`, `phase`, `huntK`, `touchCD`;
//! `dying` is the typed field; `parts["edge"]` is the pale rim skin. Each nest
//! lamp keeps its glowing shade mesh in `house_state.lamps[i].objs["shade"]`;
//! putting the lamp out dims that mesh's material.
use crate::anomalies::rig::{human_rig, GaitStyle, HumanBuild, HUMAN_RIG_SHADOW};
use crate::anomalies::{or0, truthy};
use crate::base::numeric::hypot2;
use crate::base::records::*;
use crate::prelude::*;
use std::collections::HashMap;

/// The shadows' shader patch (smoke in the dark, a
/// hard black figure with a pale edge in the light). The engine's
/// `Patch::Shadow` reads `SHADOW_U` from `patch_uniforms.f[0]` (uTime) and
/// `f[1]` (uSmoke); `init_anomalies` makes both shadow materials share one.
pub fn shadow_smoke(mat: Mat, edge: bool) -> Mat {
    mat.m().patch = Patch::Shadow { edge };
    mat
}

impl Game {
    /// Write `SHADOW_U` (`g.shadow_u_time`, `g.shadow_u_smoke`) into the shadow materials' uniforms.
    pub fn sync_shadow_u(&self) {
        for m in [&self.shadow_mat, &self.shadow_edge_mat] {
            let pu = m.r().patch_uniforms.clone();
            let mut p = pu.borrow_mut();
            p.f[0] = self.shadow_u_time as f32;
            p.f[1] = self.shadow_u_smoke as f32;
        }
    }

    pub fn house_reset(&mut self) {
        let h = &mut self.house_state;
        h.on = false;
        h.lamps = vec![];
        h.nest = None;
        h.nest_light = None;
        h.out = false;
        h.beat_t = 0.0;
    }
    pub fn house_exit_blocked(&mut self) -> bool {
        if !self.house_state.on || self.house_state.out {
            return false;
        }
        self.show_msg("THE DOOR WILL NOT LATCH", 2.2);
        self.sfx_clank();
        true
    }

    pub fn place_house(&mut self, dist: &[i32]) {
        self.house_state.on = true;
        // the nest: the open tile nearest the middle of the level that is well away
        // from where you arrive and from the front door
        let ex = match &self.exit_data {
            Some(e) => (self.tx(e.pos.x), self.tz(e.pos.z)),
            None => (-99, -99),
        };
        let mut best: Option<(i32, i32, f64)> = None;
        for z in 2..self.h - 2 {
            for x in 2..self.w - 2 {
                let i = self.t_idx(x, z);
                if self.tiles[i] != 0 || (dist[i] as f64) < self.w as f64 * 0.4 || (x - ex.0).abs() + (z - ex.1).abs() < 8 {
                    continue;
                }
                let mut room = 0;
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if self.tiles[self.t_idx(x + dx, z + dz)] == 0 {
                        room += 1;
                    }
                }
                let score = -hypot2(x as f64 - self.w as f64 / 2.0, z as f64 - self.h as f64 / 2.0) + room as f64 * 1.5 + rnd();
                if best.is_none_or(|b| score > b.2) {
                    best = Some((x, z, score));
                }
            }
        }
        let (bx, bz) = match best {
            Some(b) => (b.0, b.1),
            None => ((self.w as f64 / 2.0).floor() as i32, (self.h as f64 / 2.0).floor() as i32),
        };
        let (nx, nz) = (self.wx(bx), self.wz(bz));
        self.house_state.nest = Some((nx, nz));
        let nest = group();
        nest.position().set(nx, 0, nz);
        nest.user().gameplay = true;
        self.world.add(&nest);
        let brass = lambert!({ color: 0x8a6a3a });
        let iron = lambert!({ color: 0x2a2826 });
        let count = 7 + 3.min((self.depth() as f64 / 4.0).floor() as i32);
        for i in 0..count {
            let lamp = group();
            let floor = rnd() < 0.35;
            let shade = lambert!({ color: [0xefe2c4u32, 0xe8c8a0, 0xd8d0c0, 0xc8a878][i as usize % 4], emissive: 0xffc070, emissiveIntensity: 0.9 });
            let s;
            if floor {
                lamp.add(&mesh(pg("Nest__flamp_base", cyl_geo!(0.15, 0.17, 0.03, 16)), &iron));
                let pole = mesh(pg("Nest__flamp_pole", cyl_geo!(0.014, 0.014, 1.4, 8)), &iron);
                pole.position().y = 0.72;
                lamp.add(&pole);
                s = mesh(pg("Nest__flamp_shade", cyl_geo!(0.16, 0.22, 0.28, 16, 1, true)), &shade);
                s.position().y = 1.5;
                lamp.add(&s);
                lamp.rotation().z = rand2(-1.3, 1.3); // heaped, leaning, fallen
            } else {
                lamp.add(&mesh(pg("Nest__lamp_base", cyl_geo!(0.07, 0.09, 0.05, 14)), &brass));
                let stem = mesh(pg("Nest__lamp_stem", cyl_geo!(0.012, 0.012, 0.3, 8)), &brass);
                stem.position().y = 0.19;
                lamp.add(&stem);
                s = mesh(pg("Nest__lamp_shade", cyl_geo!(0.1, 0.16, 0.2, 16, 1, true)), &shade);
                s.position().y = 0.4;
                lamp.add(&s);
                lamp.rotation().z = if rnd() < 0.4 { rand2(-1.4, 1.4) } else { 0.0 };
            }
            let a = (i as f64 / count as f64) * PI * 2.0 + rand2(-0.3, 0.3);
            let r = 0.25 + rnd() * 0.95;
            lamp.position().set(
                a.cos() * r,
                if floor {
                    0.0
                } else if rnd() < 0.5 {
                    0.0
                } else {
                    0.18
                },
                a.sin() * r,
            );
            lamp.rotation().y = rand1(7.0);
            nest.add(&lamp);
            let mut objs = HashMap::new();
            objs.insert("shade".to_string(), s);
            self.house_state.lamps.push(HouseLamp { g: lamp.clone(), out: false, light: None, objs, f: HashMap::new() });
            let li = self.house_state.lamps.len() - 1;
            let lp = lamp.pos();
            let pos = v3(nx + lp.x, EYE, nz + lp.z);
            self.interact(
                pos,
                1.25,
                "[E] PUT OUT THE LAMP",
                move |g| g.house_state.lamps.get(li).is_some_and(|l| !l.out),
                move |g| g.put_out_nest_lamp(li),
            );
        }
        // a crate or two the lamps are heaped on
        for (x, z, s) in [(0.3, -0.2, 0.5), (-0.4, 0.35, 0.42)] {
            let c = mesh(pg("Nest__crate", box_geo!(s, s * 0.7, s)), &lambert!({ color: 0x6a5238 }));
            c.position().set(x, s * 0.35, z);
            c.rotation().y = rand1(7.0);
            nest.add(&c);
        }
        let l = point_light!(0xffc27a, 7, 13, 1.6);
        l.position().set(nx, 1.3, nz);
        self.world.add(&l);
        self.house_state.nest_light = Some(l);
    }

    /// Put out nest lamp `li` (index into `house_state.lamps`).
    pub fn put_out_nest_lamp(&mut self, li: usize) {
        let Some(l) = self.house_state.lamps.get_mut(li) else { return };
        if l.out {
            return;
        }
        l.out = true;
        if let Some(s) = l.objs.get("shade") {
            let m = s.mat();
            m.m().emissive_intensity = 0.0;
            m.m().color.multiply_scalar(0.45);
        }
        let left = self.house_state.lamps.iter().filter(|l| !l.out).count();
        if let Some(nl) = &self.house_state.nest_light {
            nl.set_intensity(7.0 * left as f64 / self.house_state.lamps.len() as f64);
        }
        if self.ac.is_some() {
            self.blip(1800.0, 0.04, "square", 0.05, None, 0.0, 0.0);
            self.noise_burst(0.18, 5200.0, 2.0, 0.08, "highpass", None, 0.0, 0.0);
        }
        // every lamp that goes out tells them where you are
        for e in self.entities.iter_mut() {
            if e.kind == "shadow" {
                e.set_n("agit", 2.6);
                e.state = "hunt".into();
                e.lose_t = 0.0;
            }
        }
        self.player.sanity = clamp(self.player.sanity - 4.0, 0.0, SAN_MAX);
        if left == 0 {
            self.house_let_go();
        }
    }
    pub fn house_let_go(&mut self) {
        self.house_state.out = true;
        if let Some(nl) = &self.house_state.nest_light {
            nl.set_intensity(0.0);
        }
        for e in self.entities.iter_mut() {
            if e.kind == "shadow" {
                e.dying = 1.0;
            }
        }
        if self.ac.is_some() {
            self.noise_burst(2.2, 140.0, 0.7, 0.35, "lowpass", None, 0.0, 0.0);
            self.schedule(1600.0, |g| g.sfx_door());
        }
        self.show_msg("SOMEWHERE, A DOOR SETTLES INTO ITS FRAME", 3.2);
    }

    /// A shadow at tile (x, z): the rig, plus the skinned body and its pale edge when built.
    pub fn make_shadow(&mut self, x: i32, z: i32) -> HumanBuild {
        let g = group();
        // its body: still the HTML version's (`creature_meshes.bin`). A
        // MakeHuman shadow fitted to this rig (E9) lost or tied four blind
        // reviews to it: the old one's long fingers hook out as claws
        let body = crate::assets::creatures::creature_geometry("shBody");
        let sm = self.shadow_mat.clone();
        let mut parts = human_rig(&g, &HUMAN_RIG_SHADOW, if body.is_some() { None } else { Some(&sm) });
        let skin = match &body {
            Some(b) => {
                let s = crate::entities::skin_body(&g, b, &sm);
                parts.skin = s.clone();
                s
            }
            None => None,
        };
        if let (Some(skin), Some(body)) = (&skin, &body) {
            skin.set_render_order(3);
            let edge = skinned_mesh(body, &self.shadow_edge_mat);
            edge.set_render_order(2); // drawn first: the body covers all but the rim
            g.add(&edge);
            // bind the edge to the body's skeleton and bind matrix (the renderer gives
            // every skinned mesh `skin_body`'s generous bounding sphere)
            let sk = match &skin.r().kind {
                Kind::Skinned { skeleton, bind_matrix, .. } => Some((skeleton.clone(), *bind_matrix)),
                _ => None,
            };
            if let Some((skeleton_v, bm)) = sk {
                if let Kind::Skinned { skeleton, bind_matrix, bind_matrix_inverse, .. } = &mut edge.n().kind {
                    *skeleton = skeleton_v;
                    *bind_matrix = bm;
                    *bind_matrix_inverse = bm.inverted();
                }
            }
            parts.edge = Some(edge);
        }
        g.position().set(self.wx(x), 0, self.wz(z));
        self.world.add(&g);
        let face = node();
        self.scene.add(&face);
        HumanBuild { g, face, parts, face_y: 1.9 }
    }

    pub fn spawn_shadows(&mut self, count: i32, dist_from_spawn: &[i32]) {
        let mut open: Vec<(i32, i32)> = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                let i = self.t_idx(x, z);
                if self.tiles[i] == 0 && dist_from_spawn[i] as f64 > self.w as f64 * 0.4 {
                    open.push((x, z));
                }
            }
        }
        let mut i = 0;
        while i < count && !open.is_empty() {
            let (x, z) = open.remove((rnd() * open.len() as f64).floor() as usize);
            let b = self.make_shadow(x, z);
            let mut e = Entity::new("shadow", b.g.clone());
            e.face = Some(b.face);
            e.face_y = 1.9;
            e.eye_y_offset = 0.06;
            e.parts = b.parts.to_map();
            e.yaw = rand1(7.0);
            e.state = "wander".into();
            e.speed = 5.0 + (self.depth() as f64 * 0.06).min(0.6);
            e.set_n("solid", 0.0);
            e.set_n("agit", 0.0);
            e.dying = 0.0;
            e.set_n("phase", rand1(7.0));
            e.set_n("huntK", 0.0);
            self.entities.push(e);
            i += 1;
        }
    }

    /// Is the player lit, as the shadows reckon it?
    pub fn player_lit_for_shadows(&self) -> bool {
        if self.player.flashlight_on {
            return true;
        }
        if self.region_light_at(self.tx(self.player.pos.x), self.tz(self.player.pos.z)) > 0.55 {
            return true;
        }
        match self.house_state.nest {
            Some((nx, nz)) if !self.house_state.out => hypot2(self.player.pos.x - nx, self.player.pos.z - nz) < 5.0,
            _ => false,
        }
    }

    /// → true when the player was caught
    pub fn update_shadow(&mut self, ei: usize, dt: f64, t: f64, d: f64) -> bool {
        let pp = self.player.pos;
        let dying = self.entities[ei].dying;
        if truthy(dying) {
            let nd = (dying - dt * 0.5).max(0.0);
            self.entities[ei].dying = nd;
            self.shadow_u_time = t; // the smoke still boils as it comes apart
            self.sync_shadow_u();
            self.shadow_fade(ei, nd * 0.3, 0.0);
            let g = self.entities[ei].g.clone();
            g.scale().set_scalar(1.0 + (1.0 - nd) * 0.4);
            if nd <= 0.0 {
                self.world.remove(&g);
                g.position().set(1e5, 0, 1e5);
                self.entities[ei].kind = "gone".into();
            }
            return false;
        }
        let lit = self.player_lit_for_shadows();
        let was_solid = self.entities[ei].n("solid");
        let solid = was_solid + ((if lit { 1.0 } else { 0.0 }) - was_solid) * 1f64.min(dt * 3.0);
        self.entities[ei].set_n("solid", solid);
        if was_solid < 0.5 && solid >= 0.5 && d < 18.0 && self.ac.is_some() {
            self.noise_burst(0.35, 3800.0, 0.8, clamp(0.25 - d * 0.012, 0.02, 0.25), "highpass", None, 0.0, 0.0);
        }
        {
            let e = &mut self.entities[ei];
            let a = e.n("agit");
            if a > 0.0 {
                e.set_n("agit", a - dt);
            }
        }
        let ep = self.entities[ei].pos();
        let sees = d < 24.0 && (d < 9.0 || self.los(ep.x, ep.z, pp.x, pp.z));
        if (lit && sees) || self.entities[ei].n("agit") > 0.0 {
            let e = &mut self.entities[ei];
            e.state = "hunt".into();
            e.lose_t = 0.0;
        } else if self.entities[ei].state == "hunt" {
            let e = &mut self.entities[ei];
            e.lose_t += dt;
            if e.lose_t > 3.0 {
                e.state = "wander".into();
                e.w_field = None;
            }
        }
        if self.entities[ei].state == "hunt" && self.player_field.is_some() {
            let (f, sp) = (self.player_field.clone(), if lit { self.entities[ei].speed } else { 0.9 });
            self.entity_step_toward(ei, f, dt, sp, Some(pp));
        } else {
            self.entities[ei].w_timer -= dt;
            if self.entities[ei].w_field.is_none() || self.entities[ei].w_timer <= 0.0 {
                self.entities[ei].w_timer = rand2(5.0, 10.0);
                for _ in 0..8 {
                    let gx = irand(1, self.w as i64 - 2) as i32;
                    let gz = irand(1, self.h as i64 - 2) as i32;
                    if self.tiles[self.t_idx(gx, gz)] == 0 {
                        let f = Rc::new(self.bfs(gx, gz));
                        let e = &mut self.entities[ei];
                        e.w_target = Some((gx, gz));
                        e.w_field = Some(f);
                        break;
                    }
                }
            }
            if let Some(f) = self.entities[ei].w_field.clone() {
                self.entity_step_toward(ei, Some(f), dt, 1.1, None);
            }
        }
        {
            let e = &self.entities[ei];
            let yaw = e.yaw;
            e.g.rotation().y = yaw;
        }
        // the look: smoke that shivers in the dark, a hard black figure with a pale
        // edge in the light
        self.shadow_u_time = t;
        self.sync_shadow_u();
        let phase = self.entities[ei].n("phase");
        let flick = 0.22 + 0.1 * (t * 9.0 + phase).sin() + 0.06 * (t * 23.0 + phase * 3.0).sin();
        let solid = self.entities[ei].n("solid");
        self.shadow_fade(ei, lerp(flick, 0.96, solid), 0.55 * solid);
        if let Some(edge) = self.entities[ei].parts.get("edge") {
            edge.set_visible(self.shadow_edge_mat.opacity_v() > 0.004);
        }
        // the walk (`human_gait`): in the dark a drifting glide that shivers, the
        // head lolling; lit and on the hunt, a long fast stride, arms reaching
        let e = &mut self.entities[ei];
        let hk0 = or0(e.n("huntK"));
        let hk = hk0 + ((if e.state == "hunt" { 1.0 } else { 0.0 }) - hk0) * (1.0 - (-dt * 3.0).exp());
        e.set_n("huntK", hk);
        let st = GaitStyle {
            hunt: 0.7 * hk * solid,
            glide: 1.0 - 0.7 * solid,
            reach: 0.85 * hk * solid,
            look: 0.35 + 0.65 * hk,
            shiver: 1.0 - solid,
            tilt: (t * 1.3 + phase).sin() * 0.2 * (1.0 - solid),
            breath: 0.0,
        };
        self.human_gait(ei, dt, t, &HUMAN_RIG_SHADOW, &st);
        // the touch (in the dark it chills once, then the smoke drifts off a while)
        {
            let e = &mut self.entities[ei];
            let cd = e.n("touchCD");
            if cd > 0.0 {
                e.set_n("touchCD", cd - dt);
            }
        }
        if d < KILL_DIST && lit {
            self.on_death("entity");
            return true;
        }
        if d < KILL_DIST && !(self.entities[ei].n("touchCD") > 0.0) {
            self.entities[ei].set_n("touchCD", 2.5);
            self.player.sanity = clamp(self.player.sanity - 22.0, 0.0, SAN_MAX);
            self.ui.flash.set_opacity(0.5);
            self.schedule(250.0, |g| g.ui.flash.set_opacity(0.0));
            let ep = self.entities[ei].pos();
            let mut back = v3(ep.x - pp.x, 0.0, ep.z - pp.z);
            back.normalize();
            back.multiply_scalar(3.0);
            if !self.circle_hits_wall(ep.x + back.x, ep.z + back.z, 0.4) {
                let g = self.entities[ei].g.clone();
                g.position().x += back.x;
                g.position().z += back.z;
            }
            let e = &mut self.entities[ei];
            e.state = "wander".into();
            e.w_field = None;
            if self.player.sanity <= 0.0 {
                self.on_death("sanity");
                return true;
            }
        }
        false
    }

    /// The body material is shared by every shadow,
    /// so its look follows the one nearest the player
    pub fn shadow_fade(&mut self, ei: usize, body: f64, edge: f64) {
        if self.nearest_shadow() != Some(ei) {
            return;
        }
        self.shadow_mat.set_opacity(body);
        self.shadow_edge_mat.set_opacity(edge);
        let e = &self.entities[ei];
        self.shadow_u_smoke = if truthy(e.dying) { 1.0 } else { 1.0 - clamp(e.n("solid"), 0.0, 1.0) }; // coming apart, it is all smoke
        self.sync_shadow_u();
    }
    /// (index into `entities`)
    pub fn nearest_shadow(&self) -> Option<usize> {
        let (mut best, mut bd) = (None, 1e9);
        let pp = self.player.pos;
        for (i, e) in self.entities.iter().enumerate() {
            if e.kind == "shadow" {
                let p = e.pos();
                let d = (p.x - pp.x).powi(2) + (p.z - pp.z).powi(2);
                if d < bd {
                    bd = d;
                    best = Some(i);
                }
            }
        }
        best
    }
    /// A heartbeat near the lit nest
    pub fn update_house(&mut self, dt: f64) {
        if !self.house_state.on || self.state != "playing" {
            return;
        }
        let Some((nx, nz)) = self.house_state.nest else { return };
        if self.house_state.out {
            return;
        }
        let Some(ss) = self.soundscape() else { return };
        let d = hypot2(self.player.pos.x - nx, self.player.pos.z - nz);
        self.house_state.beat_t -= dt;
        if d < 12.0 && self.house_state.beat_t <= 0.0 {
            self.house_state.beat_t = 3.2;
            ss.heartbeat(60.0 + (12.0 - d) * 4.0, 3, 0.0, None);
        }
    }
}
