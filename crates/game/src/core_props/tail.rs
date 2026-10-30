//! `place_props` second half: the weather extras (core_props/weather_fx.rs),
//! light switches, paper piles, lore notes, the pool room, the robot vacuum,
//! the safe room, plant / window zones, the broken elevator, elevators, the
//! floor hole, barricades, wall mirrors, the absurd tableaux, graffiti and
//! the pickup bobbing animator. See `core_props/mod.rs` for `PropsCtx` and
//! the builders.
//!
//! Every call into other modules (`take`, the builders, `wall_adj_spot`,
//! `alt_path_exists`, the lore tables) goes through `core_props::seam`, so
//! the tests can replay a recorded hand-off state.
//!
//! Records other modules read (keys as player.rs expects them):
//! * `g.light_switches[i]`: `g` = the switch group, `x`/`z` = its world
//!   position, `panels` = the controlled panel indices, `objs["rocker"]`,
//!   `f["motionCooldown"]`, `b["wasInRange"]`; the placard text is the
//!   group's `user().s["label"]`. Flipping is `g.flip_light_switch(i, cause)`.
//! * `vacuum_bot`: heading → `dir`, pick timer → `timer`, follow time →
//!   `follow_t`, speed / spin phase / chase / stun time → `f[..]`, the home
//!   LED → `objs["homeBtn"]`. Its interactable's `pos` is a copy, not a
//!   reference, so the bobbing animator keeps it on the bot.
//! * `g.haunted_candles[i]`: see core_props/weather_fx.rs.
use crate::base::numeric::hypot2;
use crate::base::records::*;
use crate::core_props::seam::*;
use crate::core_props::weather_fx::{jitter_pos, off_wall};
use crate::core_props::*;
use crate::prelude::*;
use crate::visuals::textures::{handwritten_tex, text_tex, HandOpts, TextTexOpts};
use std::collections::HashMap;

/// The names hand-written on the switch placards
const SWITCH_PLACARD_NAMES: [&str; 10] =
    ["STORAGE", "HALL A", "HALL B", "BATHROOM", "OFFICE 3", "BASEMENT", "ELEVATOR", "BREAKROOM", "STAIR C", "ARCHIVE"];
/// The owner names on the vacuum's name plate
const VAC_NAMES: [&str; 8] = ["ELIAS", "M. RANDOLPH", "K. YU", "A. BAKER", "CHESTER", "BUDDY", "CARL", "PROPERTY OF 7-B"];
/// The graffiti phrases
const SCRAWL_POOL: &[&str] = &[
    "DON'T LOOK AT IT",
    "IT'S NOT REAL",
    "NO WAY OUT",
    "RUN",
    "IT HUNTS BY SOUND",
    "THE WALLS BREATHE",
    "SIX FLOORS DEEPER",
    "HELP ME",
    "THE CLOCK LIES",
    "I'M STILL HERE",
    "ALMOND WATER SAVES",
    "DO NOT ANSWER THE PHONE",
    "BLINK AND YOU DIE",
    "IT WEARS OUR FACES",
    "GOODNIGHT",
    "FOLLOW THE HUM",
    "THE EXIT IS A LIE",
    "MOTHER WHERE ARE YOU",
    "I SEE YOU THROUGH THE MIRROR",
    "THIS IS FLOOR ZERO",
    "HE COUNTS OUR STEPS",
    "PLEASE LEAVE A LIGHT ON",
    "HELLO ?",
    "DAY 47",
    "DAY 219",
    "DAY ???",
    "THE LIGHTS LIE",
    "IT SPEAKS IN CHILD",
    "MY NAME IS ELIAS",
    "I MADE IT TO LEVEL 3",
    "GOD IS NOT HERE",
    "FOUR CORNERS FOUR WATCHERS",
    "ALWAYS TURN LEFT",
    "NEVER TURN RIGHT",
    "THE HUM IS A HYMN",
    "MY REFLECTION SMILED FIRST",
    "STOP RUNNING IT LIKES IT",
];
/// The number of graffiti hand styles
const HAND_STYLE_COUNT: i64 = 7;
/// Graffiti hand style `k` — style 0 draws its `drip` chance when called
fn hand_style(k: i64) -> HandOpts {
    let o = |color: &str, thickness: f64, jitter: f64, gap: f64, wobble: f64, double_stroke: bool, drip: bool| HandOpts {
        color: color.to_string(),
        thickness,
        jitter,
        gap,
        wobble,
        double_stroke,
        drip,
        ..Default::default()
    };
    match k {
        0 => {
            let drip = rnd() < 0.35;
            o("rgba(140,20,15,0.95)", 6.0, 1.6, 0.02, 1.2, true, drip)
        }
        1 => o("rgba(230,225,210,0.90)", 4.0, 1.9, 0.05, 1.5, false, false), // chalk
        2 => o("rgba(15,10,8,0.95)", 3.0, 1.2, 0.01, 0.8, true, false),      // pen
        3 => o("rgba(220,180,40,0.92)", 5.0, 1.4, 0.03, 1.4, false, false),  // aerosol yellow
        4 => o("rgba(100,10,10,0.98)", 8.0, 2.4, 0.06, 2.0, false, true),    // heavy blood
        5 => o("rgba(30,50,120,0.95)", 4.0, 1.5, 0.03, 1.1, false, false),   // blue pen
        _ => o("rgba(35,30,25,0.88)", 3.0, 1.7, 0.04, 1.6, false, false),    // pencil
    }
}
/// the hand-written tape label style of the switch placards / vacuum name plate
fn tape_hand(w: f64, h: f64) -> HandOpts {
    HandOpts { color: "rgba(30,20,10,0.95)".into(), thickness: 3.0, jitter: 1.4, wobble: 1.1, w, h, ..Default::default() }
}

impl Game {
    /// The second half of `place_props`.
    pub fn place_props_tail(&mut self, cx: &mut PropsCtx, dist: &[i32]) {
        let _ = dist;
        self.tail_weather_extras(cx);
        self.tail_light_switches(cx);
        self.tail_paper_piles(cx);
        self.tail_lore_notes(cx);
        self.tail_pool_room(cx);
        let vac = self.tail_robot_vacuum(cx);
        self.tail_safe_room(cx);
        self.tail_plant_zone(cx);
        // the windows (`core_props::window`): chosen here, finished on the final walls
        crate::core_props::window::place_windows(self, cx);
        self.tail_broken_elevator(cx);
        self.tail_elevators(cx);
        self.tail_floor_hole(cx);
        self.tail_barricades(cx);
        self.tail_wall_mirrors(cx);
        self.tail_tableaux(cx);
        self.tail_graffiti(cx);
        self.tail_bobbing(vac);
    }

    /// Flip light switch `i`:
    /// toggle `on`, the rocker, the controlled panels' `switched_off`, the click,
    /// and 'THE SWITCH FLIPPED ON ITS OWN' when `cause == "motion"`. (Called by player.rs.)
    pub fn flip_light_switch(&mut self, i: usize, cause: &str) {
        let Some(sw) = self.light_switches.get_mut(i) else { return };
        sw.on = !sw.on;
        let on = sw.on;
        if let Some(r) = sw.objs.get("rocker") {
            r.rotation().x = if on { -0.35 } else { 0.35 };
        }
        let idxs = sw.panels.clone();
        for idx in idxs {
            let Some(pn) = self.panels.get_mut(idx) else { continue };
            pn.switched_off = !on;
        }
        if self.ac.is_some() {
            self.noise_burst(0.02, 3200.0, 4.0, 0.10, "bandpass", None, 0.0, 0.0);
            self.blip(400.0, 0.04, "square", 0.05, None, 0.0, 0.0);
        }
        // motion-triggered flips add a small drama beat
        if cause == "motion" {
            self.show_msg("THE SWITCH FLIPPED ON ITS OWN", 1.4);
        }
    }

    /// LIGHT SWITCHES: 1-2 wall switches, each controls the four nearest
    /// ceiling panels; ~30% are faulty (flip by themselves, see `update_player`).
    fn tail_light_switches(&mut self, cx: &mut PropsCtx) {
        let switch_count = irand(1, 2);
        for _ in 0..switch_count {
            let Some(c) = s_take(self, cx, true) else { break };
            let Some(spot) = s_wall_adj_spot(self, c.x, c.z) else { continue };
            let g = group();
            g.set_pos_v(&off_wall(&spot, 0.02));
            g.position().y = 1.42;
            g.rotation().y = spot.yaw;
            // wall plate (cream plastic)
            let plate = mesh(pg("Props__switch_plate", box_geo!(0.10, 0.16, 0.012)), lambert!({ color: 0xe6ddc4 }));
            g.add(&plate);
            // recessed rocker housing
            let housing = mesh(pg("Props__switch_housing", box_geo!(0.06, 0.10, 0.006)), lambert!({ color: 0x2a2418 }));
            housing.position().z = 0.010;
            g.add(&housing);
            // the rocker itself. Rest position: tilted "on"
            let rocker = mesh(pg("Props__switch_rocker", box_geo!(0.05, 0.08, 0.020)), lambert!({ color: 0xf2e9c8, emissive: 0x0a0805 }));
            rocker.position().z = 0.017;
            rocker.rotation().x = -0.35;
            g.add(&rocker);
            // tiny screw dots on the plate corners
            for (sx, sy) in [(-0.04, 0.06), (0.04, 0.06), (-0.04, -0.06), (0.04, -0.06)] {
                let screw = mesh(circle_geo!(0.0035, 6), lambert!({ color: 0x8a806c }));
                screw.set_pos(sx, sy, 0.007);
                g.add(&screw);
            }
            // hand-written placard just above the plate
            let placard_name = choice(&SWITCH_PLACARD_NAMES);
            let placard_tex = handwritten_tex(placard_name, &tape_hand(256.0, 64.0));
            let placard = mesh(plane_geo!(0.12, 0.035), basic_mat!({ map: placard_tex, transparent: true }));
            placard.set_pos(0, 0.11, 0.011);
            g.add(&placard);
            // masking-tape strip behind the placard
            let tape = mesh(plane_geo!(0.13, 0.04), basic_mat!({ color: 0xe8dcb8, transparent: true, opacity: 0.75 }));
            tape.set_pos(0, 0.11, 0.010);
            tape.rotation().z = rand2(-0.10, 0.10);
            g.add(&tape);
            // placard slightly in front of the tape, with its tilt
            placard.position().z = 0.012;
            placard.rotation().z = tape.rot().z;
            self.world.add(&g);
            // controlled panels — 4 nearest, captured at build time (stable sort)
            let (px, pz) = (g.pos().x, g.pos().z);
            let mut by_d: Vec<(usize, f64)> =
                self.panels.iter().enumerate().map(|(idx, p)| (idx, (p.wx - px) * (p.wx - px) + (p.wz - pz) * (p.wz - pz))).collect();
            by_d.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let nearest: Vec<usize> = by_d.iter().take(4).map(|x| x.0).collect();
            let is_faulty = rnd() < 0.3;
            let i_sw = self.light_switches.len();
            let mut sw = LightSwitch {
                g: Some(g.clone()),
                x: g.pos().x,
                z: g.pos().z,
                on: true,
                faulty: is_faulty,
                panels: nearest,
                ..Default::default()
            };
            sw.objs.insert("rocker".into(), rocker.clone());
            sw.f.insert("motionCooldown".into(), 0.0);
            sw.b.insert("wasInRange".into(), false);
            g.user().s.insert("label".into(), placard_name.to_string());
            self.light_switches.push(sw);
            let mut use_pos = g.pos();
            use_pos.add_scaled_vector(&spot.normal, 0.7);
            use_pos.y = EYE;
            let prompt = if is_faulty { "[E] LIGHT SWITCH  (bad wiring)" } else { "[E] LIGHT SWITCH" };
            self.interact(use_pos, 1.2, prompt, |_| true, move |g| g.flip_light_switch(i_sw, "player"));
        }
    }

    /// PAPER PILES: rare (0-2 per level), picked up in `update_player`
    fn tail_paper_piles(&mut self, cx: &mut PropsCtx) {
        let paper_count = if rnd() < 0.65 {
            1
        } else if rnd() < 0.35 {
            2
        } else {
            0
        };
        for _ in 0..paper_count {
            let Some(c) = s_take(self, cx, false) else { break };
            let built = s_build(self, cx, TailBuild::PaperPile);
            built.g.set_pos_v(&jitter_pos(self, &c, 0.0));
            built.g.rotation().y = rand1(PI * 2.0);
            self.world.add(&built.g);
            self.papers.push(PaperPile { g: built.g.clone(), taken: false });
        }
    }

    /// A torn page on the floor, one corner curled up, a few lines of
    /// writing on it: lit only as the floor is (no glow). A note that
    /// continues a story already begun is written in that story's ink
    /// (`ink_of` its colour), which only a near look tells.
    pub(crate) fn build_lore_note_mesh(&mut self, boosted: bool, arc_color: Option<u32>) -> Obj {
        let g = group();
        // torn sheet at floor level
        let sheet = mesh(pg("Props__loreNote_sheet", box_geo!(0.26, 0.006, 0.32)), &self.mats.paper);
        sheet.position().y = 0.005;
        g.add(&sheet);
        // one corner curled up
        let curl = mesh(pg("Props__loreNote_curl", box_geo!(0.10, 0.005, 0.10)), &self.mats.paper);
        curl.set_pos(0.08, 0.03, 0.11);
        curl.rotation().x = -0.5;
        curl.rotation().z = 0.3;
        g.add(&curl);
        // its lines of writing
        let ink = match arc_color {
            Some(c) if boosted && c != 0 => ink_of(c),
            _ => INK,
        };
        let ink_mat = lambert!({ color: ink });
        for l in 0..4 {
            let line = mesh(box_geo!(0.20, 0.001, 0.006), &ink_mat);
            line.set_pos(0, 0.010, -0.10 + l as f64 * 0.06);
            g.add(&line);
        }
        g
    }

    /// A note you have read (when every note here was read): its curled
    /// corner turned down flat over it, as you left it
    pub(crate) fn turn_down_note(g: &Obj) {
        if let Some(curl) = g.children().get(1) {
            curl.set_pos(0.08, 0.0115, 0.11);
            curl.rotation().x = 0.0;
            curl.rotation().z = 0.0;
        }
    }

    /// A note's writing, in `color`'s ink
    pub(crate) fn ink_note(g: &Obj, color: u32) {
        let m = lambert!({ color: ink_of(color) });
        for line in g.children().iter().skip(2) {
            line.set_mat(m.clone());
        }
    }

    /// LORE NOTES: 2–4 per level (plus the level's anomaly notes first), each
    /// an unread id when possible, with a wall clue (arrow / READ check).
    fn tail_lore_notes(&mut self, cx: &mut PropsCtx) {
        let n = s_lore_count(self);
        let has_scp = |g: &Game, i: usize| s_lore_scp(g, i).map(|s| !s.is_empty()).unwrap_or(false);
        // prefer ids the player hasn't found; anomaly notes never join the pool
        let mut unread = vec![];
        for i in 0..n {
            if !self.lore_found.contains(&i) && !has_scp(self, i) {
                unread.push(i);
            }
        }
        // a lore entry's id is its index in `LORE_ENTRIES`
        let mut pool: Vec<usize> = if !unread.is_empty() { unread } else { (0..n).filter(|&i| !has_scp(self, i)).collect() };
        let mut i = pool.len() as i64 - 1;
        while i > 0 {
            let j = (rnd() * (i + 1) as f64).floor() as usize;
            pool.swap(i as usize, j);
            i -= 1;
        }
        let scp_notes: Vec<usize> = match self.level_scp.clone().filter(|s| !s.is_empty()) {
            Some(l) => (0..n).filter(|&i| s_lore_scp(self, i).as_deref() == Some(l.as_str())).collect(),
            None => vec![],
        };
        let mut all = scp_notes.clone();
        all.extend(pool);
        let pool = all;
        let note_count = irand(2, 4) + scp_notes.len() as i64;

        let mut i = 0i64;
        while i < note_count {
            let c = s_take(self, cx, false);
            let Some(c) = c else { break };
            if i as usize >= pool.len() {
                break;
            }
            let id = pool[i as usize];
            i += 1;
            let arc_for_note = s_lore_arc_for(self, id);
            let arc_color = arc_for_note.map(|a| s_lore_arc_color(self, a));
            let boosted = s_lore_note_boosted(self, id);
            let m = self.build_lore_note_mesh(boosted, arc_color);
            m.set_pos_v(&jitter_pos(self, &c, 0.0));
            m.rotation().y = rand1(PI * 2.0);
            self.world.add(&m);
            self.lore_notes.push(LoreNote { g: m.clone(), taken: false, id });
            // (the draws of the wall's mark that once pointed at it, kept so
            // nothing placed after it moves: a note lies as paper lies)
            if s_wall_adj_spot(self, c.x, c.z).is_some() {
                let _ = (rand2(-0.05, 0.05), rand2(-0.10, 0.10));
            }
        }
    }

    /// POOL ROOM at the location `build_world` reserved (`g.pool_plan`):
    /// tiled sunken floor and walls, curb, ladder, a waving water surface,
    /// sheen and floating beach balls. Walkable: tiles reserved, grid open.
    fn tail_pool_room(&mut self, cx: &mut PropsCtx) {
        let Some(plan) = self.pool_plan.take() else { return };
        let (x, z, pw, ph) = (plan.x, plan.z, plan.pw, plan.ph);
        let trim_w = 0.20; // width of the poured-tile perimeter curb
        let min_x = self.wx(x) - CELL / 2.0 + trim_w;
        let max_x = self.wx(x + pw - 1) + CELL / 2.0 - trim_w;
        let min_z = self.wz(z) - CELL / 2.0 + trim_w;
        let max_z = self.wz(z + ph - 1) + CELL / 2.0 - trim_w;
        let pool_width = max_x - min_x;
        let pool_depth = max_z - min_z;
        let (pcx, pcz) = ((min_x + max_x) / 2.0, (min_z + max_z) / 2.0);
        let water_y = 0.08;
        let bottom_y: f64 = -0.55; // visually sunken pool floor
                                   // ---- tiled pool bottom (big 4x4 aqua tiles with grout + stains) ----
        let pool_bottom_tex = canvas_tex(512, 512, |ctx, w, h| {
            ctx.set_fill_style("#3d7a95");
            ctx.fill_rect(0, 0, w, h);
            let tile_size = 128.0;
            let mut y = 0.0;
            while y < h {
                let mut x = 0.0;
                while x < w {
                    let shade = irand(-8, 8);
                    ctx.set_fill_style(format!("rgb({},{},{})", 61 + shade, 122 + shade, 149 + shade));
                    ctx.fill_rect(x + 3.0, y + 3.0, tile_size - 6.0, tile_size - 6.0);
                    for _ in 0..6 {
                        ctx.set_fill_style(format!("rgba(0,20,30,{})", rand2(0.15, 0.30)));
                        ctx.fill_rect(x + rand2(4.0, tile_size - 4.0), y + rand2(4.0, tile_size - 4.0), rand2(2.0, 5.0), rand2(2.0, 5.0));
                    }
                    x += tile_size;
                }
                y += tile_size;
            }
            // grout lines
            ctx.set_fill_style("#1a3244");
            let mut y = 0.0;
            while y <= h {
                ctx.fill_rect(0, y - 3.0, w, 6);
                y += tile_size;
            }
            let mut x = 0.0;
            while x <= w {
                ctx.fill_rect(x - 3.0, 0, 6, h);
                x += tile_size;
            }
            // moldy patches at grout intersections
            for _ in 0..6 {
                let gx = rand2(0.0, w / tile_size).floor() * tile_size;
                let gy = rand2(0.0, h / tile_size).floor() * tile_size;
                ctx.set_fill_style(format!("rgba(50,90,30,{})", rand2(0.2, 0.4)));
                ctx.begin_path();
                ctx.arc(gx, gy, rand2(6.0, 18.0), 0, 7);
                ctx.fill();
            }
            ctx.set_fill_style("#0a1a24");
            ctx.set_font("bold 24px sans-serif");
            ctx.set_text_align("center");
            ctx.fill_text("3 FT", 64, 68);
        });
        pool_bottom_tex.set_repeat_wrapping();
        pool_bottom_tex.t().repeat.set(pool_width / 1.5, pool_depth / 1.5);
        let pool_floor = mesh(plane_geo!(pool_width, pool_depth), lambert!({ map: pool_bottom_tex }));
        pool_floor.rotation().x = -PI / 2.0;
        pool_floor.set_pos(pcx, bottom_y, pcz);
        self.world.add(&pool_floor);
        // ---- pool side walls (4 panels from y=0 down to `bottom_y`) ----
        let wall_tile_tex = canvas_tex(256, 256, |ctx, w, h| {
            ctx.set_fill_style("#6a9db0");
            ctx.fill_rect(0, 0, w, h);
            let t = 64.0; // tile size on walls
            let mut y = 0.0;
            while y < h {
                let mut x = 0.0;
                while x < w {
                    let shade = irand(-6, 6);
                    ctx.set_fill_style(format!("rgb({},{},{})", 106 + shade, 157 + shade, 176 + shade));
                    ctx.fill_rect(x + 2.0, y + 2.0, t - 4.0, t - 4.0);
                    x += t;
                }
                y += t;
            }
            ctx.set_fill_style("#2a4658");
            let mut y = 0.0;
            while y <= h {
                ctx.fill_rect(0, y - 2.0, w, 4);
                y += t;
            }
            let mut x = 0.0;
            while x <= w {
                ctx.fill_rect(x - 2.0, 0, 4, h);
                x += t;
            }
        });
        wall_tile_tex.set_repeat_wrapping();
        let wall_depth = bottom_y.abs();
        let _pool_wall_mat = lambert!({ map: wall_tile_tex.clone() });
        // four pool side walls: [pos x, y, z, size x, y, z, u repeat]
        let side_configs = [
            [pcx, bottom_y / 2.0, min_z, pool_width, wall_depth, 0.02, pool_width / 0.7],
            [pcx, bottom_y / 2.0, max_z, pool_width, wall_depth, 0.02, pool_width / 0.7],
            [min_x, bottom_y / 2.0, pcz, 0.02, wall_depth, pool_depth, pool_depth / 0.7],
            [max_x, bottom_y / 2.0, pcz, 0.02, wall_depth, pool_depth, pool_depth / 0.7],
        ];
        for [px, py, pz, sx, sy, sz, ur] in side_configs {
            // a texture clone per wall so each has its own UV repeat
            let t = wall_tile_tex.duplicate();
            t.needs_update();
            t.set_repeat_wrapping();
            t.t().repeat.set(ur, wall_depth / 0.7);
            let mat = lambert!({ map: t });
            let wall = mesh(box_geo!(sx, sy, sz), mat);
            wall.set_pos(px, py, pz);
            self.world.add(&wall);
        }
        // ---- perimeter curb (thick ceramic tile trim around the pool) ----
        let curb_tex = canvas_tex(128, 128, |ctx, w, h| {
            ctx.set_fill_style("#e0dabf");
            ctx.fill_rect(0, 0, w, h);
            let t = 32.0;
            let mut y = 0.0;
            while y < h {
                let mut x = 0.0;
                while x < w {
                    let shade = irand(-6, 6);
                    ctx.set_fill_style(format!("rgb({},{},{})", 224 + shade, 218 + shade, 191 + shade));
                    ctx.fill_rect(x + 2.0, y + 2.0, t - 4.0, t - 4.0);
                    x += t;
                }
                y += t;
            }
            ctx.set_fill_style("#8a856a");
            let mut y = 0.0;
            while y <= h {
                ctx.fill_rect(0, y - 1.0, w, 2);
                y += t;
            }
            let mut x = 0.0;
            while x <= w {
                ctx.fill_rect(x - 1.0, 0, 2, h);
                x += t;
            }
            // occasional puddle-wet stains
            for _ in 0..6 {
                ctx.set_fill_style(format!("rgba(80,80,60,{})", rand2(0.10, 0.25)));
                ctx.begin_path();
                ctx.arc(rand1(w), rand1(h), rand2(6.0, 14.0), 0, 7);
                ctx.fill();
            }
        });
        curb_tex.set_repeat_wrapping();
        let _curb_mat = lambert!({ map: curb_tex.clone() });
        let world = self.world.clone();
        let draw_curb = |x0: f64, z0: f64, x1: f64, z1: f64| {
            let len = hypot2(x1 - x0, z1 - z0);
            let t = curb_tex.duplicate();
            t.needs_update();
            t.set_repeat_wrapping();
            t.t().repeat.set(len / 0.3, 1.0);
            let mat = lambert!({ map: t });
            // 4 cm-tall curb, 20 cm wide
            let strip = mesh(pg("Props__pool_curb", box_geo!(len, 0.06, trim_w)), mat);
            strip.set_pos((x0 + x1) / 2.0, 0.03, (z0 + z1) / 2.0);
            strip.rotation().y = (z1 - z0).atan2(x1 - x0);
            world.add(&strip);
        };
        // top/bottom curbs
        draw_curb(min_x - trim_w / 2.0, min_z - trim_w / 2.0, max_x + trim_w / 2.0, min_z - trim_w / 2.0);
        draw_curb(min_x - trim_w / 2.0, max_z + trim_w / 2.0, max_x + trim_w / 2.0, max_z + trim_w / 2.0);
        // left/right curbs (length along Z)
        draw_curb(min_x - trim_w / 2.0, min_z + trim_w / 2.0, min_x - trim_w / 2.0, max_z - trim_w / 2.0);
        draw_curb(max_x + trim_w / 2.0, min_z + trim_w / 2.0, max_x + trim_w / 2.0, max_z - trim_w / 2.0);
        // depth marker painted on the closest curb
        let depth_mark = mesh(
            plane_geo!(0.20, 0.05),
            basic_mat!({
                map: text_tex("3 FT", &TextTexOpts { w: 128.0, h: 32.0, font: "bold 28px sans-serif".into(), color: "#0a1a24".into(), glow: 0.0, ..Default::default() }),
                transparent: true,
            }),
        );
        depth_mark.rotation().x = -PI / 2.0;
        depth_mark.set_pos(pcx, 0.062, min_z - trim_w / 2.0);
        self.world.add(&depth_mark);
        // the depth under the water: a dark teal backing plane 3 cm down
        let pool_tint_mat = basic_mat!({ color: 0x1f3d45, transparent: true, opacity: 0.55, side: DoubleSide, depthWrite: false });
        let pool_tint = mesh(plane_geo!(pool_width * 0.98, pool_depth * 0.98), pool_tint_mat);
        pool_tint.rotation().x = -PI / 2.0;
        pool_tint.set_pos(pcx, water_y - 0.03, pcz);
        self.world.add(&pool_tint);
        // water surface: still, chlorinated, greyed by the room's light, a faint glow so it reads in the dark
        let water_mat = lambert!({ color: 0x527f88, emissive: 0x0b181c, transparent: true, opacity: 0.62, side: DoubleSide });
        let water = mesh(plane_geo!(pool_width, pool_depth, 24, 24), water_mat);
        water.rotation().x = -PI / 2.0;
        water.set_pos(pcx, water_y, pcz);
        self.world.add(&water);
        let wgeo = water.geo();
        let water_base: Vec<f32> = wgeo.g().position.clone();
        self.animate(move |_g, _dt, t| {
            let mut gg = wgeo.g();
            let count = water_base.len() / 3;
            for i in 0..count {
                let (bx, bz) = (water_base[i * 3] as f64, water_base[i * 3 + 2] as f64);
                gg.position[i * 3 + 2] = bz as f32;
                // two crossing wave patterns (written into the plane's local y)
                gg.position[i * 3 + 1] =
                    ((t * 1.8 + bx * 2.2 + bz * 1.3).sin() * 0.025 + (t * 1.1 + bx * 0.9 - bz * 1.7).sin() * 0.020) as f32;
            }
            gg.touch();
        });
        // R769 reflective sheen: two additive bands
        let sheen_mat_a = basic_mat!({ color: 0xd0f0ff, transparent: true, opacity: 0.14, blending: AdditiveBlending, depthWrite: false });
        let sheen_a = mesh(plane_geo!(pool_width * 0.85, pool_depth * 0.55), &sheen_mat_a);
        sheen_a.rotation().x = -PI / 2.0;
        sheen_a.set_pos(pcx - pool_width * 0.05, water_y + 0.006, pcz + pool_depth * 0.08);
        self.world.add(&sheen_a);
        let sheen_mat_b = basic_mat!({ color: 0xf0f8ff, transparent: true, opacity: 0.10, blending: AdditiveBlending, depthWrite: false });
        let sheen_b = mesh(plane_geo!(pool_width * 0.55, pool_depth * 0.25), &sheen_mat_b);
        sheen_b.rotation().x = -PI / 2.0;
        sheen_b.set_pos(pcx + pool_width * 0.15, water_y + 0.008, pcz - pool_depth * 0.12);
        self.world.add(&sheen_b);
        // slow shimmer: position drift + opacity pulse
        self.animate(move |_g, _dt, t| {
            sheen_a.position().x = pcx - pool_width * 0.05 + (t * 0.4).sin() * 0.30;
            sheen_a.position().z = pcz + pool_depth * 0.08 + (t * 0.35).cos() * 0.25;
            sheen_mat_a.set_opacity(0.10 + (t * 0.6).sin() * 0.05);
            sheen_b.position().x = pcx + pool_width * 0.15 + (t * 0.45).cos() * 0.25;
            sheen_b.position().z = pcz - pool_depth * 0.12 + (t * 0.38).sin() * 0.20;
            sheen_mat_b.set_opacity(0.07 + (t * 0.55).cos() * 0.05);
        });
        // ---- floating beach balls ----
        struct Ball {
            mesh: Obj,
            radius: f64,
            bx: f64,
            bz: f64,
            phase: f64,
            vx: f64,
            vz: f64,
        }
        let ball_colors: [u32; 7] = [0xd93a3a, 0x3aa8d9, 0xd9c93a, 0x40c96a, 0xa040c9, 0xf0873a, 0xf0f0f0];
        let ball_count = irand(5, 8);
        let mut balls = vec![];
        for _ in 0..ball_count {
            // beach-ball sized, sits half-submerged
            let radius = rand2(0.28, 0.40);
            let bmat = lambert!({ color: choice(&ball_colors), emissive: 0x0c0c0c });
            let ball = mesh(pg("Props__pool_ball", sphere_geo!(radius, 20, 14)), bmat);
            let bx = rand2(min_x + radius + 0.2, max_x - radius - 0.2);
            let bz = rand2(min_z + radius + 0.2, max_z - radius - 0.2);
            ball.set_pos(bx, water_y + radius * 0.5, bz);
            self.world.add(&ball);
            // stripes (beach ball look)
            let stripe_mat = lambert!({ color: 0xffffff });
            for s in 0..3 {
                let stripe = mesh(pg("Props__pool_ballStripe", torus_geo!(radius + 0.001, radius * 0.06, 6, 16)), &stripe_mat);
                stripe.set_rot(s as f64 * 1.1, s as f64 * 0.7, 0);
                ball.add(&stripe);
            }
            let phase = rand1(7.0);
            let vx = rand2(-0.10, 0.10);
            let vz = rand2(-0.10, 0.10);
            balls.push(Ball { mesh: ball, radius, bx, bz, phase, vx, vz });
        }
        self.animate(move |_g, dt, t| {
            for b in balls.iter_mut() {
                b.bx += b.vx * dt;
                b.bz += b.vz * dt;
                if b.bx < min_x + b.radius + 0.1 {
                    b.bx = min_x + b.radius + 0.1;
                    b.vx = -b.vx;
                }
                if b.bx > max_x - b.radius - 0.1 {
                    b.bx = max_x - b.radius - 0.1;
                    b.vx = -b.vx;
                }
                if b.bz < min_z + b.radius + 0.1 {
                    b.bz = min_z + b.radius + 0.1;
                    b.vz = -b.vz;
                }
                if b.bz > max_z - b.radius - 0.1 {
                    b.bz = max_z - b.radius - 0.1;
                    b.vz = -b.vz;
                }
                b.mesh.position().x = b.bx;
                b.mesh.position().z = b.bz;
                b.mesh.position().y = water_y + b.radius * 0.5 + (t * 2.1 + b.phase).sin() * 0.04;
                b.mesh.rotation().x += dt * 0.6;
                b.mesh.rotation().z += dt * 0.4;
            }
        });
        // walkable: reserve the tiles so props don't spawn inside
        for zz in 0..ph {
            for xx in 0..pw {
                cx.taken.push(Cand { x: x + xx, z: z + zz, d: 0 });
            }
        }
        self.pool_bounds = Some(PoolBounds { min_x, max_x, min_z, max_z, water_y, splash_y: 0.0, tiles: [x, z, pw, ph] });
        // ---- pool ladder along one edge ----
        let ladder_mat = lambert!({ color: 0xb0b8c0, emissive: 0x101418 });
        let ladder_cz = pcz;
        // two side rails that arc over the curb into the water
        for s in [-0.14, 0.14] {
            // above-water part
            let rail_top = mesh(pg("Props__poolLadder_rail", cyl_geo!(0.022, 0.022, 0.55, 10)), &ladder_mat);
            rail_top.set_pos(min_x + 0.05, 0.35, ladder_cz + s);
            self.world.add(&rail_top);
            // the bend over the curb, then a post down onto the curb
            let arc = mesh(pg("Props__poolLadder_bend", cyl_geo!(0.022, 0.022, 0.18, 10)), &ladder_mat);
            arc.set_pos(min_x - 0.034, 0.6575, ladder_cz + s);
            arc.rotation().z = 1.2;
            self.world.add(&arc);
            let arc_down = mesh(arc.geo(), &ladder_mat);
            arc_down.scale().y = 3.5; // 0.63: from the bend's end (0.69) to the curb top (0.06)
            arc_down.set_pos(min_x - 0.118, 0.375, ladder_cz + s);
            self.world.add(&arc_down);
            // below-water part descending into the pool
            let rail_below = mesh(pg("Props__poolLadder_railLow", cyl_geo!(0.022, 0.022, wall_depth + 0.1, 10)), &ladder_mat);
            rail_below.set_pos(min_x + 0.10, bottom_y / 2.0 + 0.05, ladder_cz + s);
            self.world.add(&rail_below);
        }
        // hand-hold above water
        let grip = mesh(pg("Props__poolLadder_grip", cyl_geo!(0.022, 0.022, 0.32, 10)), &ladder_mat);
        grip.rotation().x = PI / 2.0;
        grip.set_pos(min_x + 0.06, 0.62, ladder_cz);
        self.world.add(&grip);
        // rungs (4 evenly spaced, some below water)
        for r in 0..4 {
            let rung_y = 0.30 - r as f64 * 0.20;
            let rung = mesh(pg("Props__poolLadder_rung", cyl_geo!(0.018, 0.018, 0.30, 8)), &ladder_mat);
            rung.rotation().x = PI / 2.0;
            rung.set_pos(min_x + 0.10, rung_y, ladder_cz);
            self.world.add(&rung);
        }
    }

    /// ROBOT VACUUM: one per level, wanders the carpet (`update_vacuum_bot`);
    /// E-tap: 60% pet (follows 15 s), 30% chase (6 s), 10% kick (stun 8 s,
    /// alerts entities within 20 m). Returns the interactable index and the
    /// bot group so the bobbing animator can keep the prompt on the bot.
    fn tail_robot_vacuum(&mut self, cx: &mut PropsCtx) -> Option<(usize, Obj)> {
        let cand = s_take(self, cx, false)?;
        let g = group();
        // white plastic disc body
        let body_mat = lambert!({ color: 0xece6d4, emissive: 0x141210 });
        let body = mesh(pg("Props__vacuum_body", cyl_geo!(0.18, 0.19, 0.08, 24)), body_mat);
        body.position().y = 0.055;
        g.add(&body);
        // top ring and center recessed sensor
        let ring = mesh(pg("Props__vacuum_ring", torus_geo!(0.16, 0.008, 6, 24)), lambert!({ color: 0x3a3a3a }));
        ring.rotation().x = PI / 2.0;
        ring.position().y = 0.10;
        g.add(&ring);
        // center touch panel
        let panel = mesh(pg("Props__vacuum_panel", cyl_geo!(0.08, 0.08, 0.005, 20)), lambert!({ color: 0x141a20, emissive: 0x0a1218 }));
        panel.position().y = 0.10;
        g.add(&panel);
        // "home" button (glowing green)
        let home_btn = mesh(circle_geo!(0.02, 12), basic_mat!({ color: 0x60ff90 }));
        home_btn.rotation().x = -PI / 2.0;
        home_btn.set_pos(0, 0.104, 0);
        g.add(&home_btn);
        // status LEDs
        for i in 0..3 {
            let led = mesh(circle_geo!(0.008, 8), basic_mat!({ color: [0xff4020u32, 0xffc040, 0x40ff40][i] }));
            led.rotation().x = -PI / 2.0;
            led.set_pos(-0.05 + i as f64 * 0.05, 0.104, 0.05);
            g.add(&led);
        }
        // front bumper (arc)
        let bumper = mesh(
            pg("Props__vacuum_bumper", cyl_geo!(0.185, 0.185, 0.06, 24, 1, false, -PI / 4.0, PI / 2.0)),
            lambert!({ color: 0x2a2a2a }),
        );
        bumper.position().y = 0.05;
        g.add(&bumper);
        // two side-brush bristles poking out
        let brist_mat = lambert!({ color: 0xf0d840 });
        for s in [-1.0, 1.0] {
            let brush_hub = mesh(pg("Props__vacuum_brushHub", cyl_geo!(0.02, 0.02, 0.02, 8)), lambert!({ color: 0x3a3a3a }));
            brush_hub.set_pos(s * 0.14, 0.015, 0.11);
            g.add(&brush_hub);
            for b in 0..5 {
                let bristle = mesh(box_geo!(0.002, 0.002, 0.06), &brist_mat);
                let a = (b as f64 / 5.0) * PI * 2.0;
                bristle.set_pos(s * 0.14 + a.cos() * 0.02, 0.015, 0.11 + a.sin() * 0.02);
                bristle.rotation().y = a;
                g.add(&bristle);
            }
        }
        // rear main brush
        let main_brush = mesh(pg("Props__vacuum_roller", cyl_geo!(0.02, 0.02, 0.20, 8)), lambert!({ color: 0x2a2018 }));
        main_brush.rotation().z = PI / 2.0;
        main_brush.set_pos(0, 0.015, -0.05);
        g.add(&main_brush);
        // dust bin lid line
        let lid_line = mesh(box_geo!(0.15, 0.003, 0.005), &self.mats.dark);
        lid_line.set_pos(0, 0.104, -0.08);
        g.add(&lid_line);
        // brand label
        let label = mesh(
            plane_geo!(0.10, 0.02),
            basic_mat!({
                map: text_tex("CLEAN 3000", &TextTexOpts { w: 256.0, h: 64.0, font: "bold 26px sans-serif".into(), color: "#4a5a70".into(), glow: 2.0, ..Default::default() }),
                transparent: true,
            }),
        );
        label.rotation().x = -PI / 2.0;
        label.set_pos(0, 0.105, -0.03);
        g.add(&label);
        // owner name plate — hand-written tape on top
        let name_lbl = mesh(
            plane_geo!(0.16, 0.024),
            basic_mat!({
                map: handwritten_tex(choice(&VAC_NAMES), &tape_hand(512.0, 64.0)),
                transparent: true,
            }),
        );
        name_lbl.rotation().x = -PI / 2.0;
        name_lbl.set_pos(0, 0.106, 0.06);
        // tape applied off-square
        name_lbl.rotation().z = rand2(-0.15, 0.15);
        g.add(&name_lbl);
        // yellowed masking tape under the name
        let tape = mesh(plane_geo!(0.17, 0.028), basic_mat!({ color: 0xe8dcb8, transparent: true, opacity: 0.75 }));
        tape.rotation().x = -PI / 2.0;
        tape.set_pos(0, 0.1055, 0.06);
        tape.rotation().z = name_lbl.rot().z;
        g.add(&tape);
        name_lbl.position().y = 0.1058;
        g.set_pos(self.wx(cand.x), 0, self.wz(cand.z));
        self.world.add(&g);
        // a random cardinal heading
        let start_yaw = irand(0, 3) as f64 * (PI / 2.0);
        g.rotation().y = start_yaw;
        let mut v = VacuumBot {
            g: g.clone(),
            world_pos: g.pos(),
            dir: start_yaw,
            timer: 0.0,
            follow_t: 0.0,
            objs: HashMap::new(),
            f: HashMap::new(),
            b: HashMap::new(),
        };
        v.f.insert("speed".into(), 0.45); // slow so it feels peaceful
        for k in ["spinPhase", "chaseT", "stunT"] {
            v.f.insert(k.into(), 0.0);
        }
        v.objs.insert("homeBtn".into(), home_btn.clone());
        self.vacuum_bot = Some(v);
        let fv = |v: &VacuumBot, k: &str| v.f.get(k).copied().unwrap_or(0.0);
        let idx = self.interactables.len();
        self.interact(
            g.pos(),
            1.2,
            "[E] TAP THE VACUUM",
            move |gm| gm.vacuum_bot.as_ref().map(|v| fv(v, "chaseT") <= 0.0 && v.follow_t <= 0.0 && fv(v, "stunT") <= 0.0).unwrap_or(false),
            |gm| {
                if gm.vacuum_bot.is_none() {
                    return;
                }
                let roll = rnd();
                if roll < 0.60 {
                    // pet — friendly chirp + start following
                    gm.vacuum_bot.as_mut().unwrap().follow_t = 15.0;
                    if gm.ac.is_some() {
                        gm.blip(1200.0, 0.10, "sine", 0.11, None, 0.0, 0.0);
                        gm.blip(1600.0, 0.12, "sine", 0.09, None, 0.08, 0.0);
                        gm.blip(2000.0, 0.08, "sine", 0.06, None, 0.18, 0.0);
                    }
                    gm.show_msg("THE VACUUM LIKES YOU", 1.6);
                } else if roll < 0.90 {
                    // chase — playful pursuit
                    gm.vacuum_bot.as_mut().unwrap().f.insert("chaseT".into(), 6.0);
                    if gm.ac.is_some() {
                        gm.blip(880.0, 0.08, "square", 0.10, None, 0.0, 0.0);
                        gm.blip(1320.0, 0.10, "square", 0.08, None, 0.06, 0.0);
                    }
                    gm.show_msg("THE VACUUM NOTICED YOU", 1.4);
                } else {
                    // kick — loud clank, stun the bot, alert nearby entities
                    gm.vacuum_bot.as_mut().unwrap().f.insert("stunT".into(), 8.0);
                    if gm.ac.is_some() {
                        gm.noise_burst(0.20, 500.0, 1.5, 0.35, "lowpass", None, 0.0, 0.0);
                        gm.blip(160.0, 0.20, "sawtooth", 0.20, Some(90.0), 0.0, 0.0);
                    }
                    let vp = gm.vacuum_bot.as_ref().unwrap().g.pos();
                    for ent in gm.entities.iter_mut() {
                        if ent.kind == "watcher" {
                            continue;
                        }
                        let ep = ent.pos();
                        let d_ent = hypot2(ep.x - vp.x, ep.z - vp.z);
                        if d_ent < 20.0 && ent.state != "hunt" {
                            ent.state = "hunt".into();
                            ent.lose_t = 0.0;
                        }
                    }
                    gm.show_msg("YOU KICKED IT — SOMETHING HEARD", 2.2);
                }
            },
        );
        Some((idx, g))
    }

    /// SAFE ROOM: one candlelit alcove far from spawn where sanity regens.
    fn tail_safe_room(&mut self, cx: &mut PropsCtx) {
        // the first free candidate at least 12 steps from spawn (not removed from cands)
        let mut picked = None;
        for c in cx.cands.iter() {
            if cx.taken.iter().any(|t| t.x == c.x && t.z == c.z) {
                continue;
            }
            if c.d < 12 {
                continue;
            }
            picked = Some(*c);
            break;
        }
        let Some(picked) = picked else { return };
        cx.taken.push(picked);
        let (px, pz) = (self.wx(picked.x), self.wz(picked.z));
        let g = group();
        g.set_pos(px, 0, pz);
        // small dark-red round rug
        let rug_tex = canvas_tex(128, 128, |ctx, w, h| {
            ctx.set_fill_style("#5a1a1a");
            ctx.fill_rect(0, 0, w, h);
            // frayed edge + fabric weave
            for _ in 0..400 {
                let (r, gg, b, a) = (irand(90, 140), irand(20, 40), irand(20, 40), rand2(0.15, 0.5));
                ctx.set_fill_style(format!("rgba({},{},{},{})", r, gg, b, a));
                ctx.fill_rect(rand1(w), rand1(h), rand2(1.0, 2.0), rand2(1.0, 2.0));
            }
            // gold border
            ctx.set_stroke_style("#c8a040");
            ctx.set_line_width(6);
            ctx.begin_path();
            ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 6.0, 0, PI * 2.0);
            ctx.stroke();
        });
        let rug = mesh(circle_geo!(1.2, 20), lambert!({ map: rug_tex }));
        rug.rotation().x = -PI / 2.0;
        rug.position().y = 0.005;
        g.add(&rug);
        // wooden bench
        let wood_t = lambert!({ color: 0x5a3a1a });
        let bench_top = mesh(pg("Props__safeRoom_benchTop", box_geo!(0.9, 0.05, 0.28)), &wood_t);
        bench_top.set_pos(0, 0.42, -0.7);
        g.add(&bench_top);
        for s in [-0.4, 0.4] {
            let leg = mesh(pg("Props__safeRoom_benchLeg", box_geo!(0.06, 0.42, 0.24)), &wood_t);
            leg.set_pos(s, 0.21, -0.7);
            g.add(&leg);
        }
        // three candles on the bench, two-tier flames
        let mut flames = vec![];
        let mut cores = vec![];
        let wax_mat = lambert!({ color: 0xe8dcb8 });
        let flame_mat = basic_mat!({ color: 0xffcc66, transparent: true, opacity: 0.9, blending: AdditiveBlending, depthWrite: false });
        let core_mat = basic_mat!({ color: 0xffe8b0, transparent: true, opacity: 0.85, blending: AdditiveBlending, depthWrite: false });
        for ccx in [-0.28, 0.0, 0.28] {
            let wax = mesh(pg("Props__safeRoom_candle", cyl_geo!(0.028, 0.032, 0.14 + rand2(-0.02, 0.02), 8)), &wax_mat);
            wax.set_pos(ccx, 0.52, -0.7);
            g.add(&wax);
            let wick = mesh(pg("Props__safeRoom_wick", box_geo!(0.005, 0.02, 0.005)), lambert!({ color: 0x1a1408 }));
            wick.set_pos(ccx, 0.60, -0.7);
            g.add(&wick);
            let flame = mesh(plane_geo!(0.08, 0.14), &flame_mat);
            flame.set_pos(ccx, 0.66, -0.7);
            g.add(&flame);
            flames.push(flame);
            let core = mesh(plane_geo!(0.045, 0.085), &core_mat);
            core.set_pos(ccx, 0.655, -0.7);
            g.add(&core);
            cores.push(core);
        }
        // warm point light above the candles
        let warm_light = point_light!(0xffb060, 3.0, 5, 1.4);
        warm_light.set_pos(0, 1.4, -0.5);
        g.add(&warm_light);
        self.world.add(&g);
        // flicker + light pulse, synced to the safe-room LFO breath
        self.animate(move |gm, _dt, t| {
            let lfo_hz =
                gm.audio.safe_room_lfo.as_ref().map(|o| o.frequency().value()).filter(|v| *v != 0.0 && !v.is_nan()).unwrap_or(0.35);
            let flame_amp = crate::base::tables::flame_breath_amp_by_weather(gm.weather_key())
                .or(crate::base::tables::flame_breath_amp_by_weather("normal"))
                .unwrap()
                * gm.descent_boost_mul(0.4);
            let room_breath = 1.0 + (t * lfo_hz * PI * 2.0 * gm.descent_freq_boost()).sin() * flame_amp;
            let flicker = (0.85 + (t * 14.0).sin() * 0.10 + (t * 23.7).sin() * 0.05) * room_breath;
            let candle_boost = if gm.player.flashlight_on { 1.0 + 0.15 * (1.0 - gm.beam_battery_scale(0.0)) } else { 1.0 };
            warm_light.set_intensity(3.0 * flicker * candle_boost * (0.65 + 0.35 * gm.level_depth_mul()));
            let cam = gm.camera.pos();
            for i in 0..flames.len() {
                let fi = i as f64;
                flames[i].scale().y = 1.0 + (t * 12.0 + fi * 1.7).sin() * 0.15;
                flames[i].look_at_v(&cam);
                cores[i].scale().y = 1.0 + (t * 15.0 + fi * 2.3).sin() * 0.22;
                cores[i].scale().x = 1.0 + (t * 19.0 + fi * 3.1).sin() * 0.06;
                cores[i].look_at_v(&cam);
            }
        });
        self.safe_room = Some(SafeRoom { x: px, z: pz, r: 1.5, discovered: false, ..Default::default() });
    }

    /// PLANT ZONE: a "greenhouse" tile, a big plant ringed by five small ones
    fn tail_plant_zone(&mut self, cx: &mut PropsCtx) {
        let Some(c) = s_take(self, cx, false) else { return };
        let big = s_build(self, cx, TailBuild::FloorPlant);
        big.g.set_pos(self.wx(c.x), 0, self.wz(c.z));
        self.world.add(&big.g);
        for i in 0..5 {
            let a = (i as f64 / 5.0) * PI * 2.0;
            let p = s_build(self, cx, TailBuild::PottedPlant);
            p.g.set_pos(self.wx(c.x) + a.cos() * 1.2, 0, self.wz(c.z) + a.sin() * 1.2);
            p.g.rotation().y = rand1(PI * 2.0);
            self.world.add(&p.g);
        }
    }

    /// BROKEN ELEVATOR (55%): jammed doors onto black, a pulsing red light,
    /// a bloody handprint trail to a corpse pile, and a peer-in scare.
    fn tail_broken_elevator(&mut self, cx: &mut PropsCtx) {
        if rnd() >= 0.55 {
            return;
        }
        let Some(c) = s_take(self, cx, true) else { return };
        let Some(spot) = s_wall_adj_spot(self, c.x, c.z) else { return };
        let g = group();
        g.set_pos_v(&off_wall(&spot, 0.075));
        g.rotation().y = spot.yaw;
        // outer frame — battered darker metal
        let frame_mat = lambert!({ color: 0x2a2e34 });
        let outer = mesh(pg("Props__brokenElevator_frame", box_geo!(1.7, 2.5, 0.15)), &frame_mat);
        outer.position().y = 1.25;
        g.add(&outer);
        // pitch-black shaft behind the jammed doors
        let shaft = mesh(box_geo!(1.35, 2.1, 0.06), basic_mat!({ color: 0x000000 }));
        shaft.set_pos(0, 1.10, 0.10);
        g.add(&shaft);
        // two doors wedged half-open — left mostly closed, right mostly open
        let door_mat = lambert!({ color: 0x5a5f65, emissive: 0x0a0d10 });
        let door_l = mesh(pg("Props__brokenElevator_doorL", box_geo!(0.65, 2.05, 0.05)), &door_mat);
        door_l.set_pos(-0.40, 1.05, 0.13);
        g.add(&door_l);
        let door_r = mesh(pg("Props__brokenElevator_doorR", box_geo!(0.30, 2.05, 0.05)), &door_mat);
        door_r.set_pos(0.62, 1.05, 0.13);
        g.add(&door_r);
        // dents on both doors
        for _ in 0..5 {
            let dent = mesh(sphere_geo!(rand2(0.06, 0.1), 6, 4), lambert!({ color: 0x1a1c22 }));
            dent.scale().set(1, 1, 0.15);
            let side = choice(&[door_l.clone(), door_r.clone()]);
            dent.set_pos(rand2(-0.25, 0.25), rand2(-0.7, 0.7), 0.032);
            side.add(&dent);
        }
        // dangling wires from the top of the shaft
        let wire_mat = lambert!({ color: 0x2a1a08 });
        for _ in 0..4 {
            let wire = mesh(pg("Props__brokenElevator_wire", cyl_geo!(0.006, 0.006, rand2(0.4, 0.9), 4)), &wire_mat);
            wire.set_pos(rand2(-0.4, 0.4), 1.85 - rand2(0.0, 0.4), 0.11);
            wire.rotation().z = rand2(-0.15, 0.15);
            g.add(&wire);
        }
        // red emergency warning bar above the doors
        let warn_bar = mesh(box_geo!(1.5, 0.08, 0.04), basic_mat!({ color: 0xff2020 }));
        warn_bar.set_pos(0, 2.28, 0.11);
        g.add(&warn_bar);
        // pulsing red glow in front of the shaft (driven by `update_broken_elevator`)
        let red_light = point_light!(0xff2020, 0, 4, 1.8);
        red_light.set_pos(0, 2.28, 0.4);
        g.add(&red_light);
        // hand-written out-of-service placard
        let oos = mesh(
            plane_geo!(1.0, 0.22),
            basic_mat!({
                map: handwritten_tex("DO NOT ENTER", &HandOpts {
                    color: "rgba(160,30,20,0.98)".into(), thickness: 6.0, jitter: 1.9, wobble: 1.4,
                    w: 1024.0, h: 220.0, double_stroke: true, drip: true, ..Default::default()
                }),
                transparent: true,
            }),
        );
        oos.set_pos(0, 1.9, 0.14);
        g.add(&oos);
        // the side call panel is dead
        let panel = mesh(pg("Props__elevator_callPanel", box_geo!(0.15, 0.28, 0.03)), lambert!({ color: 0x1a1a1a }));
        panel.set_pos(0.94, 1.4, 0.12);
        g.add(&panel);
        let dead_btn = mesh(pg("Props__elevator_button", cyl_geo!(0.03, 0.03, 0.01, 12)), lambert!({ color: 0x3a3025 }));
        dead_btn.rotation().x = PI / 2.0;
        dead_btn.set_pos(0.94, 1.4, 0.14);
        g.add(&dead_btn);
        // scorch marks around the door edges
        let scorch_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..12 {
                let grad = ctx.create_radial_gradient(rand1(w), rand1(h), 2, rand1(w), rand1(h), rand2(8.0, 22.0));
                grad.add_color_stop(0, "rgba(10,10,10,0.85)");
                grad.add_color_stop(1, "rgba(10,10,10,0)");
                ctx.set_fill_style(&grad);
                ctx.fill_rect(0, 0, w, h);
            }
        });
        let scorch = mesh(plane_geo!(1.6, 2.2), basic_mat!({ map: scorch_tex, transparent: true }));
        scorch.set_pos(0, 1.1, 0.14);
        g.add(&scorch);
        self.world.add(&g);
        // ---- bloody handprints trailing along the wall away from the shaft ----
        let hand_tex_palm = canvas_tex_opt(128, 128, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_fill_style("rgba(120,15,15,0.92)");
            ctx.begin_path();
            ctx.ellipse(56, 78, 28, 32, 0, 0, PI * 2.0);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(28, 90, 10, 18, -0.6, 0, PI * 2.0);
            ctx.fill();
            let finger = |ctx: &mut Canvas, fx: f64, fy: f64, ang: f64, len: f64| {
                ctx.save();
                ctx.translate(fx, fy);
                ctx.rotate(ang);
                ctx.begin_path();
                ctx.ellipse(0, -len / 2.0, 7, len / 2.0, 0, 0, PI * 2.0);
                ctx.fill();
                ctx.begin_path();
                ctx.arc(0, -len, 8, 0, PI * 2.0);
                ctx.fill();
                ctx.restore();
            };
            finger(ctx, 38.0, 55.0, -0.35, 34.0);
            finger(ctx, 52.0, 48.0, -0.10, 40.0);
            finger(ctx, 66.0, 48.0, 0.10, 40.0);
            finger(ctx, 80.0, 54.0, 0.35, 32.0);
            ctx.set_fill_style("rgba(120,15,15,0.55)");
            for i in 0..6 {
                let (dx, dy) = (90.0 + i as f64 * 6.0, 76.0 + rand2(-6.0, 6.0));
                ctx.begin_path();
                ctx.ellipse(dx, dy, rand2(4.0, 8.0), rand2(2.0, 5.0), 0, 0, PI * 2.0);
                ctx.fill();
            }
            ctx.set_fill_style("rgba(90,10,10,0.7)");
            for d in 0..3 {
                let x = 30.0 + d as f64 * 20.0;
                ctx.fill_rect(x, 90.0 + rand2(-2.0, 2.0), 2, rand2(15.0, 30.0));
            }
        });
        // scrape variant: four parallel drag streaks with fingertip stubs
        let hand_tex_scrape = canvas_tex_opt(128, 128, false, |ctx, w, h| {
            ctx.set_fill_style("rgba(0,0,0,0)");
            ctx.fill_rect(0, 0, w, h);
            for (sy, angle) in [(42.0, -0.06), (58.0, -0.02), (74.0, 0.02), (90.0, 0.06)] {
                ctx.save();
                ctx.translate(28, sy);
                ctx.rotate(angle);
                let grad = ctx.create_linear_gradient(0, 0, 78, 0);
                grad.add_color_stop(0, "rgba(140,20,15,0.92)");
                grad.add_color_stop(0.55, "rgba(120,15,15,0.55)");
                grad.add_color_stop(1, "rgba(90,10,10,0.05)");
                ctx.set_fill_style(&grad);
                ctx.fill_rect(0, -4, 78, 8);
                ctx.set_fill_style("rgba(140,20,15,0.90)");
                ctx.begin_path();
                ctx.ellipse(0, 0, 7, 5, 0, 0, PI * 2.0);
                ctx.fill();
                ctx.restore();
            }
            ctx.set_fill_style("rgba(90,10,10,0.75)");
            for d in 0..4 {
                let x = 30.0 + d as f64 * 20.0 + rand2(-4.0, 4.0);
                ctx.fill_rect(x, 105.0 + rand2(-2.0, 2.0), 2, rand2(8.0, 22.0));
            }
        });
        let pick_hand_tex = || if rnd() < 0.6 { hand_tex_palm.clone() } else { hand_tex_scrape.clone() };
        // tangent along the wall
        let (tan_x, tan_z) = (-spot.normal.z, spot.normal.x);
        // 3–5 prints, chest height first, fading toward the floor
        let print_count = irand(3, 5);
        let gp = g.pos();
        for p in 0..print_count {
            let pf = p as f64;
            let along = 1.1 + pf * 0.55;
            let side = if p % 2 == 0 { 1.0 } else { -1.0 };
            let print_mat = basic_mat!({
                map: pick_hand_tex(), transparent: true, opacity: clamp(0.9 - pf * 0.14, 0.3, 0.95),
                depthWrite: false,
            });
            let hp = mesh(plane_geo!(0.30, 0.30), print_mat);
            hp.set_pos(gp.x + tan_x * along * side, 1.35 - pf * 0.14 + rand2(-0.05, 0.05), gp.z + tan_z * along * side);
            // face away from the wall
            hp.rotation().y = spot.yaw;
            // mirrored hands on one side
            if side < 0.0 {
                hp.scale().x = -1.0;
            }
            hp.rotation().z = rand2(-0.08, 0.08);
            // nudge out from the wall against z-fighting
            hp.position().x += spot.normal.x * 0.055;
            hp.position().z += spot.normal.z * 0.055;
            self.world.add(&hp);
        }
        // ---- peer-in interactable ----
        let mut use_pos = gp;
        use_pos.add_scaled_vector(&spot.normal, 1.0);
        use_pos.y = EYE;
        self.interact(
            use_pos,
            1.4,
            "[E] PEER INTO THE SHAFT",
            |gm| !gm.broken_elevator.as_ref().map(|b| b.peered).unwrap_or(false),
            |gm| {
                let Some(be) = gm.broken_elevator.as_mut() else { return };
                be.peered = true;
                // brief tension chime + soft breath from below
                gm.sfx_chime();
                gm.schedule(500.0, |g| g.sfx_breath());
                gm.shake_t = 0.4;
                // a small camera lens dip synced to the breath
                gm.schedule(500.0, |g| {
                    g.camera_dip_t = 0.25;
                });
                gm.ui.aberration.set_opacity(0.85);
                gm.schedule(900.0, |g| {
                    let v = clamp((1.0 - g.player.sanity / SAN_MAX) * 0.7, 0.0, 1.0);
                    g.ui.aberration.set_opacity(v);
                });
                let m = choice(&["SOMETHING WAS DOWN THERE.", "IT LOOKED BACK.", "YOU SHOULD NOT HAVE LOOKED.", "THE DARK BREATHED."]);
                gm.show_msg(m, 2.4);
                // small sanity penalty for the scare
                gm.player.sanity = clamp(gm.player.sanity - 12.0, 0.0, SAN_MAX);
            },
        );
        // ---- corpse pile at the end of the handprint trail ----
        {
            let corpse_side = 1.0;
            let corpse_along = 1.1 + (3f64.max(5.0) - 1.0) * 0.55 + 0.4;
            let corpse_x = gp.x + (-spot.normal.z) * corpse_along * corpse_side + spot.normal.x * 0.35;
            let corpse_z = gp.z + spot.normal.x * corpse_along * corpse_side + spot.normal.z * 0.35;
            let cg = group();
            cg.set_pos(corpse_x, 0, corpse_z);
            cg.rotation().y = spot.yaw + rand2(-0.6, 0.6);
            let bone_mat = lambert!({ color: 0xd8ceb0, emissive: 0x0a0a08 });
            // skull with two dark eye sockets
            let skull = mesh(pg("Props__corpse_skull", sphere_geo!(0.10, 12, 8)), &bone_mat);
            skull.scale().set(1, 0.85, 1.1);
            skull.set_pos(0, 0.10, 0.14);
            cg.add(&skull);
            for s in [-1.0, 1.0] {
                let eye = mesh(circle_geo!(0.020, 8), basic_mat!({ color: 0x000000 }));
                eye.rotation().y = -0.05 * s;
                eye.set_pos(s * 0.035, 0.12, 0.24);
                cg.add(&eye);
            }
            let jaw = mesh(pg("Props__corpse_jaw", box_geo!(0.11, 0.03, 0.05)), &bone_mat);
            jaw.set_pos(0.01, 0.04, 0.20);
            cg.add(&jaw);
            // rib cage — 4 arches
            for r in 0..4 {
                let rib = mesh(pg("Props__corpse_rib", torus_geo!(0.09, 0.008, 4, 12, PI)), &bone_mat);
                rib.rotation().z = PI / 2.0;
                rib.rotation().y = PI / 2.0;
                rib.set_pos(-0.05 - r as f64 * 0.03, 0.09, 0);
                cg.add(&rib);
            }
            // spine
            for v in 0..7 {
                let vert = mesh(pg("Props__corpse_vertebra", cyl_geo!(0.016, 0.014, 0.025, 6)), &bone_mat);
                vert.rotation().z = PI / 2.0;
                vert.set_pos(-0.02 - v as f64 * 0.03, 0.04, 0.005 + rand2(-0.01, 0.01));
                cg.add(&vert);
            }
            let pelvis = mesh(pg("Props__corpse_pelvis", box_geo!(0.06, 0.05, 0.09)), &bone_mat);
            pelvis.set_pos(-0.22, 0.03, 0);
            cg.add(&pelvis);
            // femurs sprawled
            let fem1 = mesh(pg("Props__corpse_femurA", cyl_geo!(0.014, 0.014, 0.24, 6)), &bone_mat);
            fem1.rotation().z = PI / 2.0 + 0.35;
            fem1.set_pos(-0.30, 0.02, -0.10);
            cg.add(&fem1);
            let fem2 = mesh(pg("Props__corpse_femurB", cyl_geo!(0.014, 0.014, 0.22, 6)), &bone_mat);
            fem2.rotation().z = PI / 2.0 - 0.20;
            fem2.set_pos(-0.28, 0.02, 0.11);
            cg.add(&fem2);
            // an arm bone with a torn cloth strip
            let arm = mesh(pg("Props__corpse_armBone", cyl_geo!(0.011, 0.011, 0.22, 6)), &bone_mat);
            arm.rotation().z = PI / 2.0 + 0.6;
            arm.set_pos(0.05, 0.03, 0.15);
            cg.add(&arm);
            let cloth = mesh(pg("Props__corpse_rag", box_geo!(0.20, 0.005, 0.08)), lambert!({ color: 0x3a2418 }));
            cloth.set_pos(-0.05, 0.007, 0.05);
            cloth.rotation().y = 0.4;
            cg.add(&cloth);
            // small blood pool underneath
            let blood_tex = canvas_tex_opt(64, 64, false, |ctx, w, h| {
                ctx.set_fill_style("rgba(0,0,0,0)");
                ctx.fill_rect(0, 0, w, h);
                let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 4, w / 2.0, h / 2.0, w / 2.0);
                grad.add_color_stop(0, "rgba(90,10,10,0.85)");
                grad.add_color_stop(1, "rgba(50,5,5,0)");
                ctx.set_fill_style(&grad);
                ctx.fill_rect(0, 0, w, h);
            });
            let pool = mesh(circle_geo!(0.35, 16), basic_mat!({ map: blood_tex, transparent: true, depthWrite: false }));
            pool.rotation().x = -PI / 2.0;
            pool.set_pos(-0.05, 0.004, 0.05);
            cg.add(&pool);
            // ID card with a name that ties to the lore (drawn on the native RNG)
            let id_names = ["ELIAS", "M. RANDOLPH", "K. YU", "A. BAKER", "7-B STAFF"];
            let name_tex = canvas_tex_opt(256, 128, false, |ctx, w, h| {
                ctx.set_fill_style("#e0dabf");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_stroke_style("#2a1a08");
                ctx.set_line_width(3);
                ctx.stroke_rect(4, 4, w - 8.0, h - 8.0);
                ctx.set_fill_style("#5a2a1a");
                ctx.fill_rect(4, 4, w - 8.0, 22);
                ctx.set_fill_style("#e0dabf");
                ctx.set_font("bold 14px monospace");
                ctx.fill_text("BACKROOMS STAFF ID", 12, 20);
                ctx.set_fill_style("#2a2a2a");
                ctx.fill_rect(12, 34, 60, 74);
                // dark X across the portrait — this person is gone
                ctx.set_stroke_style("rgba(120,15,15,0.9)");
                ctx.set_line_width(5);
                ctx.begin_path();
                ctx.move_to(12, 34);
                ctx.line_to(72, 108);
                ctx.stroke();
                ctx.begin_path();
                ctx.move_to(72, 34);
                ctx.line_to(12, 108);
                ctx.stroke();
                ctx.set_fill_style("#1a1408");
                ctx.set_font("bold 20px monospace");
                ctx.fill_text(choice(&id_names), 82, 60);
                ctx.set_font("12px monospace");
                ctx.fill_text(&format!("LEVEL {}", irand(1, 9)), 82, 82);
                ctx.fill_text(&format!("ID#{}", irand(1000, 9999)), 82, 100);
            });
            let card = mesh(pg("Props__corpse_idCard", box_geo!(0.16, 0.003, 0.10)), lambert!({ color: 0xe0dabf }));
            card.set_pos(0.05, 0.005, -0.10);
            card.rotation().y = rand2(0.0, PI * 2.0);
            cg.add(&card);
            // the printed face on top
            let card_face = mesh(plane_geo!(0.16, 0.10), basic_mat!({ map: name_tex, transparent: true }));
            card_face.rotation().x = -PI / 2.0;
            card_face.set_pos_v(&card.pos());
            card_face.position().y = 0.008;
            card_face.rotation().z = card.rot().y;
            cg.add(&card_face);
            self.world.add(&cg);
        }
        // proximity data so the frame loop can pulse the light + gate scrape audio
        let next_scrape = rand2(4.0, 9.0);
        self.broken_elevator = Some(BrokenElevator {
            light: red_light,
            warn_bar: Some(warn_bar),
            world: gp,
            normal: spot.normal,
            next_scrape,
            peered: false,
            objs: HashMap::new(),
            f: HashMap::new(),
            b: HashMap::new(),
        });
    }

    /// ELEVATORS: 1-2; the first works (an extra route to the next level)
    fn tail_elevators(&mut self, cx: &mut PropsCtx) {
        let mut elevator_placed = 0;
        let mut attempt = 0;
        while attempt < 6 && elevator_placed < 2 {
            attempt += 1;
            let Some(c) = s_take(self, cx, true) else { break };
            let Some(spot) = s_wall_adj_spot(self, c.x, c.z) else { continue };
            let functional = elevator_placed == 0; // first one always works
            let built = s_build(self, cx, TailBuild::Elevator(functional));
            // flush with the wall
            built.g.set_pos_v(&off_wall(&spot, 0.075));
            built.g.rotation().y = spot.yaw;
            self.world.add(&built.g);
            if functional {
                let mut use_pos = off_wall(&spot, 1.0);
                use_pos.y = EYE;
                self.interact(use_pos, 1.8, "[E] CALL THE ELEVATOR", |_| true, |gm| gm.next_level_via_elevator());
            }
            elevator_placed += 1;
        }
    }

    /// HOLE IN FLOOR (35%): a shortcut down; `update_player` drops the player in
    fn tail_floor_hole(&mut self, cx: &mut PropsCtx) {
        if rnd() >= 0.35 {
            return;
        }
        let Some(c) = s_take(self, cx, false) else { return };
        let built = s_build(self, cx, TailBuild::FloorHole);
        built.g.set_pos(self.wx(c.x), 0, self.wz(c.z));
        self.world.add(&built.g);
        self.floor_holes.push(FloorHole { x: self.wx(c.x), z: self.wz(c.z), r: 0.75 });
    }

    /// BARRICADES: block up to 4 corridor tiles with stacked props, only
    /// where spawn → exit stays solvable.
    fn tail_barricades(&mut self, cx: &mut PropsCtx) {
        let Some(exit_pos) = self.exit_data.as_ref().map(|e| e.pos) else { return };
        let attempts = 4;
        for _ in 0..attempts {
            // a corridor tile: exactly 2 open neighbours, on opposite sides
            let mut candidate: Option<(i32, i32)> = None;
            let mut tries = 0;
            while tries < 30 && candidate.is_none() {
                tries += 1;
                let gx = irand(2, (self.w - 3) as i64) as i32;
                let gz = irand(2, (self.h - 3) as i64) as i32;
                if self.is_wall_tile(gx, gz) {
                    continue;
                }
                let nb: Vec<bool> =
                    [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().map(|&(dx, dz)| !self.is_wall_tile(gx + dx, gz + dz)).collect();
                let open_count = nb.iter().filter(|&&b| b).count();
                let opposite = (nb[0] && nb[1]) || (nb[2] && nb[3]);
                if open_count == 2 && opposite && hypot2(self.wx(gx) - self.spawn_pos.x, self.wz(gz) - self.spawn_pos.z) > 6.0 {
                    candidate = Some((gx, gz));
                }
            }
            let Some((bx, bz)) = candidate else { break };
            // solvability: spawn → exit with the candidate blocked
            let blocked_idx = self.t_idx(bx, bz);
            let mut blocked = vec![blocked_idx];
            blocked.extend(self.barricade_tiles.iter().copied());
            let alt = s_alt_path_exists(self, 1, 1, self.tx(exit_pos.x), self.tz(exit_pos.z), &blocked);
            if !alt {
                continue;
            }
            self.barricade_tiles.push(blocked_idx);
            // a visual barricade at that tile
            let (ccx, ccz) = (self.wx(bx), self.wz(bz));
            let cab = s_build(self, cx, TailBuild::FilingCabinet);
            cab.g.set_pos(ccx, 0, ccz);
            cab.g.rotation().y = rand1(PI * 2.0);
            self.world.add(&cab.g);
            // books perched on top
            let chair_on_top = s_build(self, cx, TailBuild::BookStack);
            chair_on_top.g.set_pos(ccx + 0.1, cab.top, ccz + 0.05);
            self.world.add(&chair_on_top.g);
            let box_side = s_build(self, cx, TailBuild::CardboardStack);
            box_side.g.set_pos(ccx + rand2(-0.4, 0.4), 0, ccz + rand2(-0.4, 0.4));
            self.world.add(&box_side.g);
            // a cone leaning
            let cone = s_build(self, cx, TailBuild::TrafficCone);
            cone.g.set_pos(ccx + rand2(-0.6, 0.6), 0, ccz + rand2(-0.6, 0.6));
            cone.g.rotation().z = rand2(-0.3, 0.3);
            self.world.add(&cone.g);
        }
    }

    /// wall-mounted broken mirrors and empty frames
    fn tail_wall_mirrors(&mut self, cx: &mut PropsCtx) {
        for _ in 0..2 {
            let Some(c) = s_take(self, cx, true) else { break };
            let Some(spot) = s_wall_adj_spot(self, c.x, c.z) else { continue };
            let built = s_build(self, cx, TailBuild::BrokenMirror(spot.yaw));
            built.g.set_pos_v(&off_wall(&spot, 0.05));
            built.g.position().y = 1.4;
            built.g.rotation().y = spot.yaw;
            self.world.add(&built.g);
        }
    }

    /// the absurd tableaux: a baby doll in a wheelchair, traffic cones lined
    /// up like sentries, a cardboard box pyramid
    fn tail_tableaux(&mut self, cx: &mut PropsCtx) {
        if let Some(c) = s_take(self, cx, false) {
            let chair = s_build(self, cx, TailBuild::Wheelchair);
            chair.g.set_pos(self.wx(c.x) + rand2(-0.6, 0.6), 0, self.wz(c.z) + rand2(-0.6, 0.6));
            chair.g.rotation().y = rand1(PI * 2.0);
            self.world.add(&chair.g);
            let doll = s_build(self, cx, TailBuild::BabyDoll);
            let cp = chair.g.pos();
            doll.g.set_pos(cp.x, 0.55, cp.z);
            doll.g.rotation().y = chair.g.rot().y;
            self.world.add(&doll.g);
        }
        if let Some(c) = s_take(self, cx, false) {
            let base_yaw = rand1(PI * 2.0);
            let (dir_x, dir_z) = (base_yaw.cos(), base_yaw.sin());
            for i in 0..4 {
                let fi = i as f64;
                let cone = s_build(self, cx, TailBuild::TrafficCone);
                cone.g.set_pos(self.wx(c.x) + dir_x * fi * 0.7, 0, self.wz(c.z) + dir_z * fi * 0.7);
                self.world.add(&cone.g);
            }
        }
        if let Some(c) = s_take(self, cx, false) {
            let (ccx, ccz) = (self.wx(c.x), self.wz(c.z));
            let layers = 3;
            for l in 0..layers {
                let per = layers - l;
                for i in 0..per {
                    let b = mesh(pg("Props__cardboard_box", box_geo!(0.5, 0.32, 0.5)), &self.mats.cardboard);
                    b.set_pos(ccx + (i as f64 - (per - 1) as f64 / 2.0) * 0.52, 0.16 + l as f64 * 0.32, ccz);
                    b.rotation().y = rand2(-0.15, 0.15);
                    self.world.add(&b);
                }
            }
        }
    }

    /// hand-written graffiti (stroke-based, per-glyph jitter), up to 10
    fn tail_graffiti(&mut self, cx: &mut PropsCtx) {
        let graffiti_count = 10;
        for _ in 0..graffiti_count {
            let Some(c) = s_take(self, cx, true) else { break };
            let Some(spot) = s_wall_adj_spot(self, c.x, c.z) else { continue };
            let style = hand_style(irand(0, HAND_STYLE_COUNT - 1));
            let text = choice(SCRAWL_POOL);
            let tex = handwritten_tex(text, &style);
            let (plane_w, plane_h) = (2.6, 0.6);
            let p = mesh(plane_geo!(plane_w, plane_h), basic_mat!({ map: tex, transparent: true }));
            p.set_pos_v(&off_wall(&spot, 0.045));
            p.position().y = rand2(1.1, 2.1);
            p.rotation().y = spot.yaw;
            self.world.add(&p);
        }
    }

    /// almond water + battery bobbing. Also keeps the vacuum's interactable
    /// on the bot (the interactable holds a copy of the position, not a reference).
    fn tail_bobbing(&mut self, vac: Option<(usize, Obj)>) {
        self.animate(move |g, _dt, t| {
            for a in &g.almonds {
                if a.taken {
                    continue;
                }
                a.g.position().y = a.base + (t * 2.0 + a.phase).sin() * 0.05;
                a.g.rotation().y = t * 1.2 + a.phase;
            }
            for b in &g.battery_items {
                if b.taken {
                    continue;
                }
                b.g.position().y = b.base + (t * 2.4 + b.phase).sin() * 0.04;
                b.g.rotation().y = t * 1.4 + b.phase;
                b.g.rotation().z = PI / 2.0;
            }
            if let Some((idx, vg)) = &vac {
                let p = vg.pos();
                if let Some(it) = g.interactables.get_mut(*idx) {
                    it.pos = p;
                }
            }
        });
    }
}

/// The wall mark over a note already read: a pencil check and READ.
/// The ink of an ordinary note
const INK: u32 = 0x1a1408;
/// A story's colour as ink: dark, as ink dries on paper
pub(crate) fn ink_of(color: u32) -> u32 {
    let ch = |k: u32| (((color >> k) & 0xff) as f64 * 0.42 + 8.0).min(255.0) as u32;
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}
