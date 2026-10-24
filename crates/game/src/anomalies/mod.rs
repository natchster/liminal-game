//! Anomaly levels: the Showroom, the human rig and its walk, the Borrowed
//! House.
//!
//! This file: the level's anomaly (`scp_on_level`, the omen), the Showroom
//! (`store_state`, `place_store`, the boards, the music, the announcements, the
//! hours and the hiding), the exit door (`use_exit_door`), `update_scp`,
//! `level_title` and the start-up work. The human rig, its walk and the
//! staff are in `rig.rs`; the Borrowed House in `house.rs`; the Showroom's
//! hours and furniture kit in `store_kit.rs`.
//!
//! CONTRACT: the signatures below are called from other modules; keep them.
pub mod house;
pub mod rig;
pub mod store_kit;

use crate::base::records::*;
use crate::prelude::*;
use std::collections::HashMap;
use store_kit::{Phase, StoreKit, TagText};

/// A display bed or a wardrobe a crouching player can hide in: the object
/// plus its [`store_kit::HideSpot`] fields. Kept in `g.store_hide_spots`.
#[derive(Clone, Debug)]
pub struct StoreHideSpot {
    pub obj: Obj,
    pub eye: f64,
    pub r: f64,
    pub kind: String,
    pub front: Option<f64>,
}

// ------------------------------------------------------------- anomalies ---
/// The anomalies forced by the launch flags (`--flags 'scp=store@3,door@5'`,
/// for tests and tools); `none@4` clears a level. Entries are in first-seen
/// order (a repeated level keeps its first slot and takes the last key). The
/// flags never change during a run, so they are parsed on demand.
pub fn scp_forced(flags: &str) -> Vec<(f64, String)> {
    let mut m: Vec<(f64, String)> = vec![];
    // the value of the first `scp=` flag
    let Some(h) = flags.split('&').find_map(|kv| kv.strip_prefix("scp=")) else { return m };
    if h.is_empty() {
        return m;
    }
    for part in percent_decode(h).split(',') {
        let mut it = part.split('@');
        let k = it.next().unwrap_or("");
        let n = it.next().map(parse_number).unwrap_or(f64::NAN);
        if !k.is_empty() && n > 0.0 {
            if let Some(e) = m.iter_mut().find(|e| e.0 == n) {
                e.1 = k.to_string();
            } else {
                m.push((n, k.to_string()));
            }
        }
    }
    m
}

/// Percent-decode `s` (as UTF-8). A malformed escape or invalid UTF-8 leaves
/// the string as it stands.
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = |c: u8| (c as char).to_digit(16);
            match (b.get(i + 1).and_then(|&c| hex(c)), b.get(i + 2).and_then(|&c| hex(c))) {
                (Some(h), Some(l)) => {
                    out.push((h * 16 + l) as u8);
                    i += 3;
                    continue;
                }
                _ => return s.to_string(),
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// The number, or 0 when it is NaN (unset)
#[inline]
pub(crate) fn or0(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v
    }
}
/// A number as a flag: true unless it is 0 or NaN
#[inline]
pub(crate) fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

// ------------------------------------------------------------- the Showroom ---
/// A room-set's main piece (`bed`, `sofa`, ...) → its price-tag description; None for the other pieces
fn main_what(name: &str) -> Option<&'static str> {
    Some(match name {
        "bed" => "Bed frame, 160×200",
        "sofa" => "3-seat sofa",
        "wardrobe" => "Wardrobe",
        "desk" => "Desk",
        "diningTable" => "Table, 6 seats",
        "kitchenRun" => "Kitchen, complete",
        "bunkBed" => "Bunk bed frame",
        "armchair" => "Armchair",
        "bookcase" => "Bookcase",
        _ => return None,
    })
}
/// The store music's chords: four slow chords, three voices each
const STORE_CHORDS: [[f64; 3]; 4] = [[261.6, 329.6, 392.0], [220.0, 277.2, 329.6], [246.9, 293.7, 370.0], [196.0, 246.9, 293.7]];
/// One step of the store music: glide every voice to chord `k`
fn store_music_step(ac: &AudioContext, oscs: &[AudioNode], k: usize) {
    let c = STORE_CHORDS[k % STORE_CHORDS.len()];
    let t = ac.current_time();
    // oscillators in creation order: voice v in 0..3, each detuned -4 then +4 cents
    for (i, o) in oscs.iter().enumerate() {
        let v = i / 2;
        o.frequency().set_target_at_time(c[v] * if v == 0 { 0.5 } else { 1.0 }, t, 0.25);
    }
}

impl Game {
    /// start-up work (staff/shadow materials, the shadow uniforms, ...)
    pub fn init_anomalies(&mut self) {
        // the staff and shadow bodies: decoded once at start-up (`creature_geometry` caches them)
        let _ = crate::assets::creatures::creature_geometry("stBody");
        let _ = crate::assets::creatures::creature_geometry("shBody");
        // the shadow uniforms (time 0, smoke 1), shared by both shadow materials
        self.shadow_u_time = 0.0;
        self.shadow_u_smoke = 1.0;
        let body = crate::anomalies::house::shadow_smoke(
            basic_mat!({ color: 0x040405, transparent: true, opacity: 0.3, depthWrite: false }),
            false,
        );
        let edge = crate::anomalies::house::shadow_smoke(
            basic_mat!({ color: 0x9aa2b4, transparent: true, opacity: 0.0, side: BackSide, depthWrite: false }),
            true,
        );
        let u = body.r().patch_uniforms.clone();
        edge.m().patch_uniforms = u;
        self.shadow_mat = body;
        self.shadow_edge_mat = edge;
        self.sync_shadow_u();
        self.omen_timers.clear();
    }

    /// The level's anomaly key, if any
    pub fn scp_on_level(&self, n: i32) -> Option<String> {
        let forced = scp_forced(&self.launch_flags);
        if let Some((_, k)) = forced.iter().find(|e| e.0 == n as f64) {
            return if k == "none" { None } else { Some(k.clone()) };
        }
        let k = crate::level::scp::scp_for_level(self.run_seed, n)?;
        // a forced anomaly is not also met on its planned level
        if forced.iter().any(|e| e.1 == k) {
            None
        } else {
            Some(k.to_string())
        }
    }

    pub fn show_omen(&mut self, text: &str, delay: f64, dur: f64) {
        if text.is_empty() {
            return;
        }
        for id in std::mem::take(&mut self.omen_timers) {
            self.cancel_scheduled(id);
        }
        self.ui.omen.set_opacity(0.0);
        let t = text.to_string();
        let a = self.schedule(delay * 1000.0, move |g| {
            g.ui.omen.set_text(&t);
            g.ui.omen.set_opacity(0.85);
        });
        let b = self.schedule((delay + dur) * 1000.0, |g| {
            g.ui.omen.set_opacity(0.0);
        });
        self.omen_timers = vec![a, b];
    }

    pub fn store_reset(&mut self) {
        if let Some(m) = self.store_state.music.take() {
            self.store_music_stop(&m);
        }
        let s = &mut self.store_state;
        s.on = false;
        s.t = 0.0;
        s.phase = "day".into();
        s.cycle = 0.0;
        s.hides = vec![];
        s.hidden = false;
        s.lights_out = 0.0;
        s.panels_off = false;
        s.board_text = String::new();
        // (the board texture, its canvas and the arrow material are kept)
        self.store_hide_spots = vec![];
        self.player.hide_eye = None;
    }

    /// The archetype of tile `i`'s zone ('' without a layout)
    pub fn zone_arch_of(&self, i: usize) -> &'static str {
        match &self.layout {
            Some(l) => l.zones[l.zone_of[i] as usize].arch,
            None => "",
        }
    }

    pub fn place_store(&mut self, dist: &[i32]) {
        let _ = dist;
        self.store_state.on = true;
        let mats: Rc<RefCell<HashMap<u32, Mat>>> = Rc::new(RefCell::new(HashMap::new()));
        let mat = {
            let mats = mats.clone();
            Rc::new(move |hex: u32| -> Mat { mats.borrow_mut().entry(hex).or_insert_with(|| lambert!({ color: hex })).clone() })
        };
        // price tags share a pool of ten textures (each a canvas upload of its own
        // otherwise, and each a draw call that can never be instanced)
        let mut tag_pool: HashMap<String, Vec<Mat>> = HashMap::new(); // kind of item -> up to three tags
        let tag = move |text: &TagText| -> Mat {
            let pool = tag_pool.entry(text.what.clone()).or_default();
            if pool.len() >= 3 {
                return pool[irand(0, pool.len() as i64 - 1) as usize].clone();
            }
            let (name, what, price) = (text.name.clone(), text.what.clone(), text.price.clone());
            let t = canvas_tex_opt(256, 160, false, move |ctx, w, h| {
                ctx.set_fill_style("#f2dc3a");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_fill_style("#1a1a1a");
                ctx.set_font("bold 34px Helvetica, Arial, sans-serif");
                ctx.fill_text(&name, 14, 44);
                ctx.set_font("18px Helvetica, Arial, sans-serif");
                ctx.fill_text(&what, 14, 72);
                ctx.set_font("bold 56px Helvetica, Arial, sans-serif");
                ctx.fill_text(&price, 14, 138);
            });
            let m = basic_mat!({ map: t });
            pool.push(m.clone());
            m
        };
        let mut kit = {
            let m = mat.clone();
            StoreKit::game(Box::new(move |h| m(h)), Box::new(tag))
        };
        // one shared board texture: OPEN and the time by day, CLOSED at night
        // (a 256×128 canvas kept in `store_state.board_canvas`, redrawn by `draw_store_board`)
        let canvas = Canvas::new(256, 128);
        let board_tex = canvas.to_tex();
        {
            let mut tt = board_tex.t();
            tt.color_space = ColorSpace::Srgb;
            tt.anisotropy = 4;
        }
        self.store_state.board_canvas = Some(Rc::new(RefCell::new(canvas)));
        self.store_state.board_tex = Some(board_tex.clone());
        let board_mat = basic_mat!({ map: board_tex });
        self.draw_store_board("OPEN", Some("10:00"));
        let mut sign_mat: HashMap<&'static str, Mat> = HashMap::new();
        let mut sign_for = |text: &'static str| -> Mat {
            sign_mat
                .entry(text)
                .or_insert_with(|| {
                    let tex = crate::visuals::textures::text_tex(
                        text,
                        &crate::visuals::textures::TextTexOpts {
                            w: 512.0,
                            h: 128.0,
                            font: "bold 60px Helvetica, Arial, sans-serif".into(),
                            color: "#1c2a44".into(),
                            glow: 0.0,
                            rot: 0.0,
                        },
                    );
                    lambert!({ map: tex, color: 0xffffff, emissive: 0x303030 })
                })
                .clone()
        };
        let sign_back = mat(0xf2f0e8);
        let pick_theme = || -> &'static str {
            let total = store_kit::ROOM_SET_WEIGHTS.iter().fold(0.0, |a, (_, w)| a + w);
            let mut r = rnd() * total;
            for (k, w) in store_kit::ROOM_SET_WEIGHTS {
                r -= w;
                if r <= 0.0 {
                    return k;
                }
            }
            "living"
        };
        let ex = self.exit_data.as_ref().map(|e| (self.tx(e.pos.x), self.tz(e.pos.z)));
        let zones: Vec<crate::level::archgen::Zone> = self.layout.as_ref().map(|l| l.zones.clone()).unwrap_or_default();
        for zn in &zones {
            if zn.arch != "store" {
                continue;
            }
            for c in crate::level::archgen::store_cells(zn) {
                if c.x1 - c.x0 < 1 || c.z1 - c.z0 < 1 {
                    continue;
                } // a sliver: leave it as aisle
                if let Some(ex) = ex {
                    if ex.0 >= c.x0 - 1 && ex.0 <= c.x1 + 1 && ex.1 >= c.z0 - 1 && ex.1 <= c.z1 + 1 {
                        continue;
                    }
                } // keep the way out clear
                let theme = pick_theme();
                let cx = (self.wx(c.x0) + self.wx(c.x1)) / 2.0;
                let cz = (self.wz(c.z0) + self.wz(c.z1)) / 2.0;
                // face the side of the cell with the most openings in its partition
                let mut best: Option<(i32, i32, f64)> = None;
                for (dx, dz) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let mut open = 0;
                    if dz != 0 {
                        let zz = if dz > 0 { c.z1 + 1 } else { c.z0 - 1 };
                        for x in c.x0..=c.x1 {
                            if self.in_grid(x, zz) && self.tiles[self.t_idx(x, zz)] == 0 {
                                open += 1;
                            }
                        }
                    } else {
                        let xx = if dx > 0 { c.x1 + 1 } else { c.x0 - 1 };
                        for z in c.z0..=c.z1 {
                            if self.in_grid(xx, z) && self.tiles[self.t_idx(xx, z)] == 0 {
                                open += 1;
                            }
                        }
                    }
                    let score = open as f64 + rnd() * 0.5;
                    if best.is_none_or(|b| score > b.2) {
                        best = Some((dx, dz, score));
                    }
                }
                let best = best.unwrap();
                let yaw = (best.0 as f64).atan2(best.1 as f64);
                let (cs, sn) = (yaw.cos(), yaw.sin());
                let at = |lx: f64, lz: f64| (cx + lx * cs + lz * sn, cz - lx * sn + lz * cs);
                for p in store_kit::room_set(theme, &mut || rnd()).unwrap() {
                    let obj = kit.build(p.name, p.a1, p.a2).unwrap_or_else(|| panic!("store kit has no builder {}", p.name));
                    let (px, pz) = at(p.x, p.z);
                    obj.position().set(px, p.y_off, pz);
                    obj.rotation().y = yaw + p.yaw;
                    self.world.add(&obj);
                    if let Some(h) = store_kit::hide_spot(&obj) {
                        self.store_state.hides.push(StoreHide {
                            x: px,
                            z: pz,
                            eye: v3(px, h.eye, pz),
                            f: [("r".to_string(), h.r)].into_iter().chain(h.front.map(|f| ("front".to_string(), f))).collect(),
                        });
                        self.store_hide_spots.push(StoreHideSpot {
                            obj: obj.clone(),
                            eye: h.eye,
                            r: h.r,
                            kind: h.kind.clone(),
                            front: h.front,
                        });
                    }
                    if let Some(what) = main_what(p.name) {
                        if rnd() < 0.6 {
                            let name = store_kit::product_name(&mut || rnd());
                            let price = format!("{}.–", irand(2, 99) * 10 - 1);
                            let t = kit.price_tag(&TagText { name, what: what.to_string(), price });
                            let (qx, qz) = at(p.x + 0.9, p.z + 1.0);
                            t.position().set(qx, 0, qz);
                            t.rotation().y = yaw;
                            self.world.add(&t);
                        }
                    }
                }
                // a department sign hung from the slab over the front of the set
                if rnd() < 0.55 {
                    let s = group();
                    let face = mesh(plane_geo!(1.8, 0.45), sign_for(store_kit::department_sign(theme).unwrap()));
                    face.position().z = 0.021;
                    s.add(&face);
                    s.add(&mesh(pg("Store__sign_board", box_geo!(1.86, 0.5, 0.04)), &sign_back));
                    // hung from the nominal ceiling (WALL_H) like every ceiling prop:
                    // `settle_props` then lifts the whole sign to the real slab
                    let drop = 0.85;
                    for x in [-0.8, 0.8] {
                        let w = mesh(cyl_geo!(0.004, 0.004, drop - 0.25, 4), mat(0x777777));
                        w.position().set(x, 0.25 + (drop - 0.25) / 2.0, 0);
                        s.add(&w);
                    }
                    let (px, pz) = at(0.0, 2.6);
                    s.position().set(px, WALL_H - drop, pz);
                    s.rotation().y = yaw;
                    self.world.add(&s);
                }
                // now and then a board with the store's hours on a post
                if rnd() < 0.34 {
                    let b = group();
                    b.add(&mesh(pg("Store__board_post", cyl_geo!(0.03, 0.03, 2.3, 8)), mat(0x3a3a3a)));
                    b.child(0).unwrap().position().y = 1.15;
                    let frame = mesh(pg("Store__board_frame", box_geo!(0.84, 0.46, 0.06)), mat(0x222222));
                    frame.position().y = 2.42;
                    b.add(&frame);
                    let face = mesh(plane_geo!(0.76, 0.38), &board_mat);
                    face.position().set(0, 2.42, 0.031);
                    b.add(&face);
                    let (px, pz) = at(2.4, 2.7);
                    b.position().set(px, 0, pz);
                    b.rotation().y = yaw;
                    self.world.add(&b);
                }
            }
        }
        // floor arrows: every other aisle tile points one step closer to the way out
        if let Some(ex) = ex {
            let to_exit = self.bfs(ex.0, ex.1);
            let arrow_tex = canvas_tex_opt(128, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                ctx.set_fill_style("rgba(255,248,225,0.92)");
                ctx.begin_path();
                ctx.move_to(64, 10);
                ctx.line_to(112, 62);
                ctx.line_to(80, 62);
                ctx.line_to(80, 118);
                ctx.line_to(48, 118);
                ctx.line_to(48, 62);
                ctx.line_to(16, 62);
                ctx.close_path();
                ctx.fill();
            });
            let arrow_mat =
                basic_mat!({ map: arrow_tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2 });
            self.store_state.arrow_mat = Some(arrow_mat.clone());
            struct Arrow {
                x: f64,
                z: f64,
                yaw: f64,
            }
            let mut list: Vec<Arrow> = vec![];
            for z in 1..self.h - 1 {
                for x in 1..self.w - 1 {
                    let i = self.t_idx(x, z);
                    if self.tiles[i] != 0 || self.zone_arch_of(i) != "store" || to_exit[i] <= 1 || (x + z) % 2 != 0 {
                        continue;
                    }
                    let mut best_d = to_exit[i];
                    let mut dir: Option<(i32, i32)> = None;
                    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let n = self.t_idx(x + dx, z + dz);
                        if self.tiles[n] == 0 && to_exit[n] >= 0 && to_exit[n] < best_d {
                            best_d = to_exit[n];
                            dir = Some((dx, dz));
                        }
                    }
                    if let Some((dx, dz)) = dir {
                        list.push(Arrow { x: self.wx(x), z: self.wz(z), yaw: (-(dx as f64)).atan2(-(dz as f64)) });
                    }
                }
            }
            if !list.is_empty() {
                let im = instanced_mesh(plane_geo!(0.8, 0.8), &arrow_mat, list.len());
                for (k, a) in list.iter().enumerate() {
                    let e = Euler::with_order(-PI / 2.0, a.yaw, 0.0, EulerOrder::YXZ);
                    let q = Quat::from_euler(&e);
                    let m4 = Mat4::compose(&v3(a.x, 0.012, a.z), &q, &v3(1, 1, 1));
                    im.set_matrix_at(k, &m4);
                }
                im.user().arch = true;
                self.world.add(&im);
            }
        }
        // abandoned trolleys in the aisles
        for _ in 0..6 {
            let x = irand(1, self.w as i64 - 2) as i32;
            let z = irand(1, self.h as i64 - 2) as i32;
            let i = self.t_idx(x, z);
            if self.tiles[i] != 0 || self.zone_arch_of(i) != "store" {
                continue;
            }
            let t = kit.trolley();
            t.position().set(self.wx(x) + rand2(-1.0, 1.0), 0, self.wz(z) + rand2(-1.0, 1.0));
            t.rotation().y = rand1(7.0);
            self.world.add(&t);
        }
    }

    pub fn draw_store_board(&mut self, word: &str, time: Option<&str>) {
        let Some(c) = self.store_state.board_canvas.clone() else { return };
        let text = format!("{}{}", word, time.unwrap_or(""));
        if text == self.store_state.board_text {
            return;
        }
        self.store_state.board_text = text;
        let (w, h) = (256.0, 128.0);
        let mut ctx = c.borrow_mut();
        ctx.set_fill_style("#050505");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_text_align("center");
        ctx.set_fill_style(if word == "OPEN" { "#3ee07a" } else { "#ff3a2a" });
        ctx.set_font("bold 54px \"Courier New\", monospace");
        ctx.fill_text(word, w / 2.0, if time.is_some() { 58.0 } else { 82.0 });
        if let Some(t) = time {
            ctx.set_fill_style("#e8d8a0");
            ctx.set_font("bold 38px \"Courier New\", monospace");
            ctx.fill_text(t, w / 2.0, 108);
        }
        if let Some(t) = &self.store_state.board_tex {
            ctx.update_tex(t);
        }
    }

    /// Soft shop music, four slow chords on two detuned voices,
    /// low-passed, as if from ceiling speakers a long way off. The voice's
    /// `nodes` are the six oscillators, `gain` the output; `set` / `stop` are
    /// [`Game::store_music_set`] / [`Game::store_music_stop`].
    pub fn store_music(&mut self) -> Option<Voice> {
        let ac = self.ac.clone()?;
        let out = ac.create_gain();
        out.gain().set_value(0.0);
        let lp = ac.create_biquad_filter();
        lp.set_type("lowpass");
        lp.frequency().set_value(1400.0);
        out.connect(&lp);
        match self.soundscape() {
            Some(s) => {
                lp.connect(&s.world);
            }
            None => {
                if let Some(m) = &self.master {
                    lp.connect(m);
                }
            }
        }
        let mut oscs: Vec<AudioNode> = vec![];
        for v in 0..3 {
            for det in [-4.0, 4.0] {
                let o = ac.create_oscillator();
                o.set_type(if v != 0 { "triangle" } else { "sine" });
                o.detune().set_value(det);
                o.connect(&out);
                o.start(0.0);
                oscs.push(o);
            }
        }
        store_music_step(&ac, &oscs, 0);
        // the next chord every 3.2 s
        let id = self.store_music_tick(oscs.clone(), 1);
        self.store_music_timer = Some(id);
        out.gain().set_target_at_time(0.022, ac.current_time(), 1.5);
        Some(Voice { nodes: oscs, gain: Some(out) })
    }
    /// one beat of the store music: the next chord in 3.2 s, then the one after
    fn store_music_tick(&mut self, oscs: Vec<AudioNode>, k: usize) -> u64 {
        self.schedule(3200.0, move |g| {
            if let Some(ac) = g.ac.clone() {
                store_music_step(&ac, &oscs, k);
            }
            let id = g.store_music_tick(oscs, k + 1);
            g.store_music_timer = Some(id);
        })
    }
    /// Fade the store music in (`on`) or out.
    pub fn store_music_set(&mut self, m: &Voice, on: bool) {
        let t = self.ac_time();
        if let Some(out) = &m.gain {
            out.gain().set_target_at_time(if on { 0.022 } else { 0.0 }, t, if on { 1.5 } else { 0.08 });
        }
    }
    /// Stop the store music: cancel its beat, fade out, then stop and disconnect the oscillators.
    pub fn store_music_stop(&mut self, m: &Voice) {
        if let Some(id) = self.store_music_timer.take() {
            self.cancel_scheduled(id);
        }
        let t = self.ac_time();
        if let Some(out) = &m.gain {
            out.gain().set_target_at_time(0.0, t, 0.05);
        }
        let m = m.clone();
        self.schedule(400.0, move |g| {
            let t = g.ac_time();
            for o in &m.nodes {
                o.stop(t);
            }
            if let Some(out) = &m.gain {
                out.disconnect();
            }
        });
    }

    /// The public address, a two-tone chime, then a voice through a tinny horn
    pub fn store_announce(&mut self, kind: &str) {
        let Some(ac) = self.ac.clone() else { return };
        if kind == "close" {
            self.blip(784.0, 0.7, "sine", 0.05, None, 0.0, 0.0);
            self.blip(659.0, 0.8, "sine", 0.045, None, 0.42, 0.0);
            self.blip(523.0, 1.3, "sine", 0.04, None, 0.84, 0.0);
        } else {
            self.blip(523.0, 0.7, "sine", 0.04, None, 0.0, 0.0);
            self.blip(659.0, 0.8, "sine", 0.04, None, 0.4, 0.0);
            self.blip(784.0, 1.2, "sine", 0.04, None, 0.8, 0.0);
        }
        if kind != "close" {
            return;
        }
        let Some(ss) = self.soundscape() else { return };
        let horn = ac.create_biquad_filter();
        horn.set_type("bandpass");
        horn.frequency().set_value(1300.0);
        horn.q().set_value(1.4);
        let shaper = ac.create_wave_shaper();
        let curve: Vec<f32> = (0..256)
            .map(|i| {
                let x = i as f64 / 128.0 - 1.0;
                (x * 3.0).tanh() as f32
            })
            .collect();
        shaper.set_curve(curve);
        horn.connect(&shaper);
        shaper.connect(&ss.world);
        let v = ss.create_child_voice(Some(&horn));
        v.set_level(0.5, 0.05);
        let t0 = ac.current_time() + 1.9;
        let d1 = v.say("please", Some(t0), 0.42, 1.0);
        let d2 = v.say("please", Some(t0 + d1 + 0.6), 0.4, 1.0);
        self.schedule((1.9 + d1 + d2 + 1.2) * 1000.0, move |g| {
            v.stop();
            g.schedule(1500.0, move |_| {
                horn.disconnect();
                shaper.disconnect();
            });
        });
    }

    pub fn on_store_phase(&mut self, from: &str, to: &str) {
        let _ = from;
        if to == "warn" {
            self.store_announce("close");
            for i in 0..self.panels.len() {
                if rnd() < 0.3 {
                    self.panels[i].flicker = rand2(0.2, 0.6);
                }
            }
            self.show_msg("THE STORE WILL CLOSE SHORTLY", 3.0);
        } else if to == "night" {
            // the relay: a deep thunk, then every light in the building goes
            self.noise_burst(0.35, 90.0, 1.0, 0.5, "lowpass", None, 0.0, 0.0);
            self.blip(48.0, 0.6, "sine", 0.3, Some(30.0), 0.0, 0.0);
            self.sfx_clank();
            for p in self.panels.iter_mut() {
                p.switched_off = true;
            }
            self.store_state.panels_off = true;
            if let Some(m) = self.store_state.music.clone() {
                self.store_music_set(&m, false);
            }
            self.shake_t = self.shake_t.max(0.25);
        } else if to == "day" {
            self.store_announce("open");
            for i in 0..self.panels.len() {
                self.panels[i].switched_off = false;
                self.panels[i].flicker = rand2(0.1, 1.6);
            }
            self.store_state.panels_off = false;
            if let Some(m) = self.store_state.music.clone() {
                self.store_music_set(&m, true);
            }
            for e in self.entities.iter_mut() {
                if e.kind == "staff" {
                    e.state = "wander".into();
                    e.lose_t = 0.0;
                }
            }
        }
    }

    pub fn update_store(&mut self, dt: f64) {
        if !self.store_state.on || self.state != "playing" {
            return;
        }
        if self.store_state.music.is_none() && self.ac.is_some() {
            self.store_state.music = self.store_music();
        }
        self.store_state.t += dt;
        let p = store_kit::store_phase_default(self.store_state.t);
        if p.phase.as_str() != self.store_state.phase {
            let from = std::mem::replace(&mut self.store_state.phase, p.phase.as_str().to_string());
            self.store_state.cycle = p.cycle as f64;
            self.on_store_phase(&from, p.phase.as_str());
        }
        let night = p.phase == Phase::Night;
        let lo = self.store_state.lights_out;
        self.store_state.lights_out = lo + ((if night { 1.0 } else { 0.0 }) - lo) * 1f64.min(dt * if night { 4.0 } else { 0.9 });
        if let Some(m) = &self.store_state.arrow_mat {
            let v = 1.0 - 0.96 * self.store_state.lights_out;
            m.m().color.set_scalar(v);
        }
        let clock = store_kit::store_clock(&p);
        self.draw_store_board(if night { "CLOSED" } else { "OPEN" }, clock.as_deref());
        // hiding: crouched under a display bed, or inside a wardrobe
        let mut spot: Option<usize> = None;
        if self.player.crouch {
            for (k, h) in self.store_hide_spots.iter().enumerate() {
                if h.obj.parent().is_none() {
                    continue;
                }
                let o = h.obj.pos();
                let (dx, dz) = (self.player.pos.x - o.x, self.player.pos.z - o.z);
                // into the object's frame (its front is +z)
                let ry = h.obj.rot().y;
                let (c, s) = ((-ry).cos(), (-ry).sin());
                let (lx, lz) = (dx * c + dz * s, -dx * s + dz * c);
                let front = match h.front {
                    Some(f) if truthy(f) => f,
                    _ => 0.35,
                };
                let inside = if h.kind == "bed" { lx.abs() < 0.75 && lz.abs() < 1.0 } else { lx.abs() < 0.55 && lz > -0.3 && lz < front };
                if inside {
                    spot = Some(k);
                    break;
                }
            }
        }
        let was = self.store_state.hidden;
        self.store_state.hidden = spot.is_some();
        self.player.hide_eye = spot.map(|k| self.store_hide_spots[k].eye);
        if self.store_state.hidden != was {
            self.ui.crouch_ind.set_text(if self.store_state.hidden { "— HIDDEN —" } else { "— CROUCHING —" });
        }
    }

    pub fn use_exit_door(&mut self, to_stairs: bool) {
        let scp = self.level_scp.clone();
        if scp.as_deref() == Some("house") && self.house_exit_blocked() {
            return;
        }
        if scp.as_deref() == Some("loop") {
            return self.enter_loop();
        }
        if to_stairs {
            self.enter_stairwell()
        } else {
            self.next_level()
        }
    }
    pub fn update_scp(&mut self, dt: f64) {
        match self.level_scp.as_deref() {
            Some("store") => self.update_store(dt),
            Some("house") => self.update_house(dt),
            _ => {}
        }
    }

    /// The lore name of the level (its dominant zone archetype)
    pub fn level_title(&self) -> String {
        if let Some(l) = &self.layout {
            if l.dominant == "house" {
                return "ROOMS FROM ELSEWHERE".into();
            }
            if let Some(a) = crate::level::archgen::archetype(l.dominant) {
                return a.title.to_string();
            }
        }
        self.pick_text("levelSubtitle")
    }
}
