//! The player: input actions, item use, crying, guide arrows, the vacuum
//! bot, the broken elevator and `update_player`.
//! CONTRACT: the signatures below are called from other modules; keep them.
//!
//! The player record is `g.player` (`state::Player`), the held keys are
//! `g.keys`, and mouse look is `session/frame.rs::mouse_move`.
//!
//! Per-object state without a typed field lives in the records' maps:
//! * `vacuum_bot`: heading → `v.dir`, pick timer → `v.timer`, follow time →
//!   `v.follow_t`; speed, spin phase, chase / stun time and beep timer →
//!   `v.f[..]` (keys `speed`, `spinPhase`, `chaseT`, `stunT`, `beepTimer`);
//!   the home LED → `v.objs["homeBtn"]`.
//! * `broken_elevator`: linger time → `be.f["lingerT"]`, descent fired →
//!   `be.b["descentFired"]`.
//! * `g.light_switches[i]`: position → `sw.x`/`sw.z`, panel indices →
//!   `sw.panels`, motion cooldown / arm time → `sw.f[..]` (`motionCooldown`,
//!   `armT`), in range → `sw.b["wasInRange"]`; flipping is
//!   `g.flip_light_switch(i, cause)`.
//! * `g.lore_notes[i]`: the whisper cooldown and "was near" flag live in the
//!   note object's user data (`n.g.user()`), which has the note's lifetime.
use crate::base::records::*;
use crate::base::tables::*;
use crate::prelude::*;
use crate::visuals::lighting::RF_DARK;

// ---------------------------------------------------------------- item use
/// Same bonus the old walk-over pickup granted
pub const DRINK_STAMINA: f64 = 45.0;
pub const SWAP_BATTERY: f64 = 55.0;
/// Refuse to waste an item above this fraction
pub const NEARLY_FULL: f64 = 0.95;

/// Per-weather table lookup for weather key `wk`, falling back to the "normal" entry.
fn wpick<T>(f: fn(&str) -> Option<T>, wk: &str) -> T {
    f(wk).or_else(|| f("normal")).unwrap()
}

/// The dark zone takes your nerve no lower than this, and gives it back up
/// to this (a spent nerve, under 25, is the creatures' doing alone, and
/// passes: `creature::creature_contact`). Over 25 by more than the
/// strongest sign of a stalker (12, `director.rs`): at 30, the dark and one
/// sign had spent a nerve no creature had touched (E8).
pub const DARK_FLOOR: f64 = 40.0;
/// How fast a change of your size comes over you, m/s
pub const SIZE_EASE: f64 = 0.025;

/// The place's answer to a cry (`Game::show_the_way`): how far along the
/// way the lights answer, m, and how long after the last each one stutters, s
pub const WAY_LIGHTS_REACH: f64 = 30.0;
pub const WAY_LIGHT_STEP: f64 = 0.25;
/// Its footprints: how many, the first this far along the way, a pace
/// apart, each this long after the last, and how long one takes to dry, s
pub const WAY_PRINTS: usize = 16;
pub const WAY_PRINT_FIRST: f64 = 1.6;
pub const WAY_PACE: f64 = 0.72;
pub const WAY_PRINT_STEP: f64 = 0.45;
pub const WAY_PRINT_LIFE: f64 = 20.0;

/// How wet a print looks with `life` s left: up in its first half second,
/// then drying away
pub fn way_print_opacity(life: f64) -> f64 {
    let age = WAY_PRINT_LIFE - life;
    let up = (age / 0.5).clamp(0.0, 1.0);
    let dry = (life / WAY_PRINT_LIFE).clamp(0.0, 1.0).powf(0.7);
    0.85 * up * dry
}

/// A wet bare footprint, toes toward the top (−z once laid flat): heel,
/// the outer edge of the sole, the ball and five toes, darker where it
/// pressed, a sheen at its rim
fn wet_print_tex() -> Tex {
    canvas_tex_opt(64, 152, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // water soaks out a little round where the foot pressed: each part
        // a soft rim under a darker core
        let blot = |ctx: &mut Canvas, x: f64, y: f64, rx: f64, ry: f64, rot: f64, a: f64| {
            for (k, s) in [(0.35, 1.18), (0.7, 1.0)] {
                ctx.set_fill_style(format!("rgba(16,14,12,{:.3})", a * k));
                ctx.begin_path();
                ctx.ellipse(x, y, rx * s, ry * s, rot, 0.0, PI * 2.0);
                ctx.fill();
            }
        };
        // a right foot (its arch on the left): heel, the outer edge of the
        // sole joining it to the ball, the ball
        blot(ctx, w * 0.54, h * 0.83, w * 0.22, h * 0.1, 0.0, 0.8);
        blot(ctx, w * 0.66, h * 0.62, w * 0.12, h * 0.17, -0.12, 0.62);
        blot(ctx, w * 0.52, h * 0.38, w * 0.29, h * 0.1, -0.15, 0.82);
        // toes, the big one first
        for (k, (x, y, r)) in
            [(0.28, 0.2, 0.11), (0.47, 0.16, 0.075), (0.61, 0.17, 0.066), (0.73, 0.2, 0.058), (0.83, 0.25, 0.05)].iter().enumerate()
        {
            blot(ctx, w * x, h * y, w * r, w * r * 1.15, 0.0, 0.72 - k as f64 * 0.05);
        }
    })
}

impl Game {
    /// start-up work (paper marker / guide arrow geometry, ...)
    pub fn init_player(&mut self) {
        self.paper_marker_geo = box_geo!(0.22, 0.008, 0.28);
        self.paper_marker_mat = lambert!({ color: 0xf2ead0, emissive: 0x0a0906 });
        // a wet bare footprint (the place's answer to a cry: `show_the_way`)
        self.print_geo = {
            let geo = plane_geo!(0.11, 0.26);
            geo.rotate_x(-PI / 2.0);
            geo
        };
        self.print_mat = lambert!({
            map: wet_print_tex(), color: 0xffffff, transparent: true, opacity: 0.0, depthWrite: false,
            polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, side: DoubleSide,
        });
    }

    /// A key (or a mouse button, while the pointer is locked) went down: its
    /// actions (`session::input::BINDINGS`), then the notebook's page keys
    /// and the keys that go on from the death and ending cards.
    pub fn on_key_down(&mut self, code: &str) {
        // a menu takes its keys first (Mute still works under it)
        if self.menu_key(code) {
            return;
        }
        // the cards take a key only once they are up (no skipping them by
        // mashing a key through the moment that leads to them)
        let card_up = !self.ui.overlay.has_class("hidden") || self.live_menu().is_some();
        let go_on = matches!(code, "KeyE" | "Enter" | "Space" | "KeyR" | "Mouse0");
        let mute = self.acts_of(code).contains(&crate::session::input::Act::Mute);
        if self.state == "dead" {
            if go_on && card_up {
                self.restart();
            }
            if mute {
                self.on_action(crate::session::input::Act::Mute);
            }
            return;
        }
        if self.state == "ending" {
            if go_on && card_up && code != "KeyR" {
                self.end_ending();
            }
            return;
        }
        // a cutscene has your hands: Escape, Mute and the skip key
        if self.hands_taken() && self.live_menu().is_none() {
            return self.cutscene_key(code);
        }
        // under the title, the pause list or Settings only Mute reaches the game
        if self.live_menu().is_some() {
            if mute {
                self.on_action(crate::session::input::Act::Mute);
            }
            return;
        }
        // the notebook turns its pages with the arrows (and the wheel, and the mouse buttons)
        if self.journal_open {
            match code {
                "ArrowLeft" | "Mouse2" => return self.journal_prev(),
                "ArrowRight" | "Mouse0" => return self.journal_next(),
                _ => {}
            }
        }
        for a in self.acts_of(code) {
            self.on_action(a);
        }
    }

    /// The first bottle and the first battery of a run
    /// explain themselves, shown just after the pickup line.
    pub fn hint_first_use(&mut self, kind: &str) {
        // (with the key as Settings bind it)
        let key = |g: &Game, a: crate::session::input::Act| {
            g.settings.keys(a).first().map(|k| crate::session::input::key_label(k)).unwrap_or_default()
        };
        if kind == "bottle" && !self.hinted_bottle {
            self.hinted_bottle = true;
            self.schedule(1700.0, move |g| {
                if g.state == "playing" {
                    let s = format!("{} to drink it.", key(g, crate::session::input::Act::Drink));
                    g.show_msg(&s, 2.4);
                }
            });
        } else if kind == "battery" && !self.hinted_battery {
            self.hinted_battery = true;
            self.schedule(1700.0, move |g| {
                if g.state == "playing" {
                    let s = format!("{} to change it.", key(g, crate::session::input::Act::Battery));
                    g.show_msg(&s, 2.4);
                }
            });
        }
    }

    pub fn drink_almond_water(&mut self) -> bool {
        if self.state != "playing" {
            return false;
        }
        if self.player.bottles <= 0 {
            self.show_msg("NO ALMOND WATER", 1.0);
            self.sfx_clank();
            return false;
        }
        if self.player.stamina >= self.body.stam_max * NEARLY_FULL {
            self.show_msg("YOU ARE NOT TIRED YET", 1.2);
            return false;
        }
        self.player.bottles -= 1;
        self.last_used = Some(crate::secrets::Thing::Almond);
        self.player.stamina = clamp(self.player.stamina + DRINK_STAMINA * self.body.drink, 0.0, self.body.stam_max);
        self.player.stam_delay = 0.0; // drinking cancels the post-sprint regen pause
        self.sfx_drip();
        let t = self.pick_text("almondDrink");
        self.show_msg(&t, 1.5);
        true
    }

    pub fn swap_battery(&mut self) -> bool {
        if self.state != "playing" {
            return false;
        }
        if self.player.batteries <= 0 {
            self.show_msg("NO SPARE BATTERIES", 1.0);
            self.sfx_clank();
            return false;
        }
        if self.player.battery >= BAT_MAX * NEARLY_FULL {
            self.show_msg("THE CELL IS STILL GOOD", 1.2);
            return false;
        }
        self.player.batteries -= 1;
        self.last_used = Some(crate::secrets::Thing::Battery);
        self.player.battery = clamp(self.player.battery + SWAP_BATTERY, 0.0, BAT_MAX);
        self.sfx_clank();
        let t = self.pick_text("batterySwap");
        self.show_msg(&t, 1.5);
        true
    }

    pub fn toggle_flashlight(&mut self) {
        if self.state != "playing" {
            return;
        }
        if !self.player.flashlight_on && self.player.battery <= 0.0 {
            self.show_msg("BATTERY DEAD", 1.2);
            self.sfx_clank();
            return;
        }
        self.player.flashlight_on = !self.player.flashlight_on;
        self.sfx_clank();
    }

    /// A small folded sheet dropped on the floor to mark
    /// territory; stays across the level until the player moves on.
    pub fn drop_paper_marker(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.player.papers <= 0 {
            self.show_msg("NO PAPER LEFT", 1.0);
            return;
        }
        // don't drop right on top of an existing one — keeps trails readable
        for m in &self.paper_markers {
            let mp = m.mesh.pos();
            let (dx, dz) = (mp.x - self.player.pos.x, mp.z - self.player.pos.z);
            if dx * dx + dz * dz < 0.35 {
                return;
            }
        }
        self.player.papers -= 1;
        let m = mesh(self.paper_marker_geo.clone(), &self.paper_marker_mat);
        // drop just in front of the player's feet, slight random tilt
        let (fx, fz) = (-self.player.yaw.sin(), -self.player.yaw.cos());
        let (px, pz) = (self.player.pos.x + fx * 0.35, self.player.pos.z + fz * 0.35);
        m.position().set(px, self.floor_y_at(px, pz) + 0.008, pz);
        m.rotation().y = self.player.yaw + rand2(-0.4, 0.4);
        m.rotation().x = rand2(-0.05, 0.05);
        m.rotation().z = rand2(-0.05, 0.05);
        self.world.add(&m);
        let p = m.pos();
        self.paper_markers.push(PaperMarker { mesh: m, world_x: p.x, world_z: p.z });
        self.sfx_paper_rustle();
    }

    /// Whether you stand in a region with no light of its own (a dark zone)
    pub fn in_dark_region(&self) -> bool {
        let (ptx, ptz) = (self.tx(self.player.pos.x), self.tz(self.player.pos.z));
        let r: i32 = if self.in_grid(ptx, ptz) { self.region_map.get(self.t_idx(ptx, ptz)).map(|v| *v as i32).unwrap_or(-1) } else { -1 };
        r >= 0 && self.region_info.get(r as usize).map(|r| r.flags & RF_DARK != 0).unwrap_or(false)
    }

    /// H = shout for help. Coin flip (biased by level): either the place
    /// shows the way to the door (`show_the_way`), or nearby entities aggro
    /// on the player. Higher levels shift the odds toward the entities.
    pub fn cry_for_help(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.cry_cooldown > 0.0 {
            let s = format!("WAIT {}s", fmt_num(self.cry_cooldown.ceil()));
            self.show_msg(&s, 1.0);
            return;
        }
        self.cry_cooldown = 22.0;
        // your voice (a line of the persona, docs/VOICES.md); the formant cry
        // if none can be said
        if !self.say_cry() {
            self.sfx_cry();
        }
        self.show_msg("YOU CRY OUT...", 1.6);
        // odds by level: level 1 => 50/50 the way/aggro. Every level -8% the way, +8% aggro.
        // (what you wear changes how readily the place answers: `secrets::keeps`)
        let way_chance = (clamp(0.5 - (self.depth() - 1) as f64 * 0.08, 0.05, 0.5) * self.body.cry_mul + self.body.cry_add).min(0.6);
        if rnd() < way_chance {
            self.schedule(900.0, |g| {
                g.show_the_way();
            });
        } else {
            self.schedule(600.0, |g| g.alert_entities_to_player());
        }
    }

    pub fn alert_entities_to_player(&mut self) {
        if self.state != "playing" {
            return;
        }
        self.show_msg("SOMETHING HEARD YOU", 2.0);
        self.sfx_distant_scream();
        let pp = self.player.pos;
        for i in 0..self.entities.len() {
            if crate::entities::creature::on_new_brain(&self.entities[i].kind) {
                // a cry is the loudest thing you can do: they come to it
                self.creature_alert(i, pp.x, pp.z);
            }
        }
        // extra spawn at random far open tile — capped at 1 to keep it survivable
        let mut open: Vec<(i32, i32)> = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                if self.is_wall_tile(x, z) {
                    continue;
                }
                let d = hypot2(self.wx(x) - self.player.pos.x, self.wz(z) - self.player.pos.z);
                if d > 10.0 && d < 26.0 {
                    open.push((x, z));
                }
            }
        }
        if !open.is_empty() && rnd() < 0.75 {
            let (x, z) = choice(&open);
            let kind = if self.depth() >= 3 && rnd() < 0.5 { "hound" } else { "smiler" };
            let ei = self.add_entity(kind, x, z, 0);
            let pp = self.player.pos;
            self.entities[ei].freeze = 0.5;
            self.creature_alert(ei, pp.x, pp.z);
        }
    }

    /// The way from here to the door, tile by tile (BFS from the exit,
    /// greedy descent from the player); empty without an exit or a way.
    pub fn way_to_the_door(&self) -> Vec<(i32, i32)> {
        let Some(ep) = self.exit_data.as_ref().map(|e| e.pos) else { return vec![] };
        let (exit_x, exit_z) = (self.tx(ep.x), self.tz(ep.z));
        // (round a barricade, never into one)
        let field = self.bfs_open(exit_x, exit_z);
        let (mut cx, mut cz) = (self.tx(self.player.pos.x), self.tz(self.player.pos.z));
        if self.is_wall_tile(cx, cz) {
            return vec![];
        }
        let mut path = vec![(cx, cz)];
        while (cx != exit_x || cz != exit_z) && path.len() < 800 {
            let here = field.get(self.t_idx(cx, cz)).copied().unwrap_or(-1);
            if here <= 0 {
                break;
            }
            let mut best: Option<(i32, i32, i32)> = None;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if !self.in_grid(nx, nz) || self.is_wall_tile(nx, nz) || self.barricade_tiles.contains(&self.t_idx(nx, nz)) {
                    continue;
                }
                let nd = field.get(self.t_idx(nx, nz)).copied().unwrap_or(-1);
                if nd >= 0 && nd < here && (best.is_none() || nd < best.unwrap().2) {
                    best = Some((nx, nz, nd));
                }
            }
            let Some(b) = best else { break };
            (cx, cz) = (b.0, b.1);
            path.push((cx, cz));
        }
        path
    }

    /// The panels that would answer along `path`: lit ones (not dead, not
    /// switched off) over its first `WAY_LIGHTS_REACH` m, within 2.2 m of
    /// it, in the order you would walk under them.
    fn way_lights(&self, path: &[(i32, i32)]) -> Vec<usize> {
        let mut out: Vec<(usize, usize)> = vec![];
        let tiles = ((WAY_LIGHTS_REACH / CELL).ceil() as usize + 1).min(path.len());
        for (i, p) in self.panels.iter().enumerate() {
            if p.dead || p.switched_off {
                continue;
            }
            let near = path[..tiles]
                .iter()
                .enumerate()
                .filter(|(_, t)| hypot2(self.wx(t.0) - p.wx, self.wz(t.1) - p.wz) < 2.2)
                .map(|(k, _)| k)
                .next();
            if let Some(k) = near.filter(|&k| k > 0) {
                out.push((k, i));
            }
        }
        out.sort_by_key(|&(k, i)| {
            let t = path[k];
            (k, (hypot2(self.panels[i].wx - self.wx(t.0), self.panels[i].wz - self.wz(t.1)) * 1000.0) as i64)
        });
        out.into_iter().map(|(_, i)| i).collect()
    }

    /// How the place would answer a cry here: the lights along the way where
    /// at least three lit panels hang over it (and the last answer was not
    /// the lights), else wet footprints (on the Building's floors, where
    /// nothing walks, the door's own call). None without a way to the door.
    pub fn way_answer_here(&self) -> Option<WayAnswer> {
        let path = self.way_to_the_door();
        if path.len() < 2 {
            return None;
        }
        let lights = self.way_lights(&path).len() >= 3;
        // on the Building's floors nothing walks: no prints there, the door calls
        let walked = if self.ground.is_empty() { WayAnswer::Prints } else { WayAnswer::Call };
        Some(if lights && self.last_way != Some(WayAnswer::Lights) { WayAnswer::Lights } else { walked })
    }

    /// The place shows the way to the door (a cry heard, the payphone's
    /// hint), never by a marker put there for you, and not always the same way: the lit panels
    /// over the way stutter one after another away from you toward it (each
    /// relay ticking where it hangs), or (on the Building's floors, where
    /// nothing walks) the door calls at once, louder, or wet bare footprints appear on the
    /// floor ahead, a pace apart, walking toward it, and dry away. No line
    /// and no chime; the rooms darken a little while it lasts. Returns how
    /// it answered.
    pub fn show_the_way(&mut self) -> Option<WayAnswer> {
        if self.state != "playing" {
            return None;
        }
        let answer = self.way_answer_here()?;
        let path = self.way_to_the_door();
        // (the last answer's prints dry at once)
        for a in self.way_prints.drain(..) {
            self.world.remove(&a.mesh);
        }
        self.cry_dim_t = 8.0;
        self.last_way = Some(answer);
        self.ways_shown += 1;
        match answer {
            WayAnswer::Lights => {
                for (k, i) in self.way_lights(&path).into_iter().enumerate() {
                    let (wx, wz) = (self.panels[i].wx, self.panels[i].wz);
                    self.schedule(k as f64 * WAY_LIGHT_STEP * 1000.0, move |g| {
                        if g.state != "playing" {
                            return;
                        }
                        // (only the same panel: a rebuilt level's panels are not it)
                        if let Some(p) = g.panels.get_mut(i).filter(|p| p.wx == wx && p.wz == wz) {
                            p.flicker = 0.45;
                            let y = if p.y > 0.0 { p.y } else { WALL_H };
                            g.play_from(wx, y, wz, 3.0, 0.9, 0.6, |g| g.sfx_relay());
                        }
                    });
                }
            }
            WayAnswer::Call => self.exit_call_now(),
            WayAnswer::Prints => {
                // along the way: from where you stand through the middle of each tile
                let mut pts: Vec<(f64, f64)> = vec![(self.player.pos.x, self.player.pos.z)];
                pts.extend(path[1..].iter().map(|t| (self.wx(t.0), self.wz(t.1))));
                let mut prints = vec![];
                let (mut along, mut next) = (0.0, WAY_PRINT_FIRST);
                for w in pts.windows(2) {
                    let (dx, dz) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
                    let len = dx.hypot(dz);
                    if len < 1e-6 {
                        continue;
                    }
                    while next <= along + len && prints.len() < WAY_PRINTS {
                        let t = (next - along) / len;
                        let side = if prints.len() % 2 == 0 { -1.0 } else { 1.0 };
                        let (ux, uz) = (dx / len, dz / len);
                        let (x, z) = (w[0].0 + dx * t - uz * side * 0.1, w[0].1 + dz * t + ux * side * 0.1);
                        prints.push((x, z, (-ux).atan2(-uz), side));
                        next += WAY_PACE;
                    }
                    along += len;
                }
                for (k, (x, z, yaw, side)) in prints.into_iter().enumerate() {
                    if self.circle_hits_wall(x, z, 0.08) {
                        continue;
                    }
                    self.schedule(k as f64 * WAY_PRINT_STEP * 1000.0, move |g| {
                        if g.state != "playing" {
                            return;
                        }
                        let m = mesh(g.print_geo.clone(), g.print_mat.duplicate());
                        m.position().set(x, g.floor_y_at(x, z) + 0.006, z);
                        m.rotation().y = yaw;
                        // the left foot is the right one turned over
                        m.scale().x = side;
                        g.world.add(&m);
                        g.way_prints.push(WayPrint { mesh: m, life: WAY_PRINT_LIFE, index: k });
                        g.play_from(x, 0.05, z, 3.0, 1.0, 0.6, |g| g.sfx_wet_step());
                    });
                }
            }
        }
        Some(answer)
    }

    /// Pulsing red light + occasional scrape
    /// audio, and a one-off linger scare.
    pub fn update_broken_elevator(&mut self, dt: f64, t: f64) {
        let Some(be) = self.broken_elevator.as_ref() else { return };
        let (bw, light) = (be.world, be.light.clone());
        let (dx, dz) = (bw.x - self.player.pos.x, bw.z - self.player.pos.z);
        let d = hypot2(dx, dz);
        // slow red pulse (~2 Hz) plus a stutter every few seconds
        let pulse = 0.5 + 0.5 * (t * 2.4).sin();
        let stutter = if (t * 5.0).sin() > 0.92 { 0.2 } else { 1.0 }; // brief brownout every ~1.3s
        light.set_intensity(pulse * stutter * 1.4);
        // occasional scrape sfx, only when player is close enough to hear
        let ns = self.broken_elevator.as_ref().unwrap().next_scrape - dt;
        self.broken_elevator.as_mut().unwrap().next_scrape = ns;
        if ns <= 0.0 {
            self.broken_elevator.as_mut().unwrap().next_scrape = rand2(6.0, 14.0);
            if d < 22.0 && self.state == "playing" {
                // loudness falls off with distance
                let loud = clamp(0.18 - d * 0.006, 0.02, 0.18);
                self.sfx_scrape(loud);
            }
        }
        // ---- linger scare: if player stays close for ≥15s, one distant clang +
        // falling scream. Fires once per level. ----
        let fired = self.broken_elevator.as_ref().unwrap().b.get("descentFired").copied().unwrap_or(false);
        if !fired {
            let linger = self.broken_elevator.as_ref().unwrap().f.get("lingerT").copied().unwrap_or(0.0);
            // (not while a cutscene has you: the shaft's own look)
            if d < 4.0 && self.state == "playing" && !self.hands_taken() {
                let linger = linger + dt;
                self.broken_elevator.as_mut().unwrap().f.insert("lingerT".into(), linger);
                if linger >= 15.0 {
                    self.broken_elevator.as_mut().unwrap().b.insert("descentFired".into(), true);
                    // metallic clang echoes...
                    self.schedule(0.0, |g| g.sfx_door_slam());
                    self.schedule(300.0, |g| g.sfx_clank());
                    self.schedule(620.0, |g| g.sfx_clank());
                    // ...then a distant falling scream fading away
                    self.schedule(900.0, |g| g.sfx_distant_scream());
                    // camera lens dip synced to the third clang — brief physical beat
                    self.schedule(620.0, |g| {
                        g.camera_dip_t = 0.35;
                    });
                    // a small sanity dip (no line says what fell: LORE.md, A broken lift)
                    self.player.sanity = clamp(self.player.sanity - 6.0, 0.0, SAN_MAX);
                }
            } else {
                // walked away — decay linger so a slow pass-through doesn't accumulate
                self.broken_elevator.as_mut().unwrap().f.insert("lingerT".into(), (linger - dt * 2.0).max(0.0));
            }
        }
    }

    /// Gentle wander + hum modulation by distance to
    /// the player; follow / chase / stun modes after an E-tap.
    pub fn update_vacuum_bot(&mut self, dt: f64) {
        let Some(mut v) = self.vacuum_bot.take() else {
            if let Some(n) = self.audio.vacuum_gain.as_ref() {
                let gv = n.gain().value();
                n.gain().set_value(gv + (0.0 - gv) * (dt * 3.0).min(1.0));
            }
            return;
        };
        self.vacuum_bot_step(&mut v, dt);
        self.vacuum_bot = Some(v);
    }

    fn vacuum_bot_step(&mut self, v: &mut VacuumBot, dt: f64) {
        fn fv(v: &VacuumBot, k: &str) -> f64 {
            v.f.get(k).copied().unwrap_or(0.0)
        }
        fn set(v: &mut VacuumBot, k: &str, x: f64) {
            v.f.insert(k.to_string(), x);
        }
        let (ppx, ppz) = (self.player.pos.x, self.player.pos.z);
        v.timer -= dt; // pick timer (next heading change)
                       // Tick down follow/stun timers.
        if v.follow_t > 0.0 {
            v.follow_t -= dt;
        }
        if fv(v, "stunT") > 0.0 {
            set(v, "stunT", fv(v, "stunT") - dt);
        }
        // ---- stun mode: kicked — sits still, occasionally sad beep ----
        if fv(v, "stunT") > 0.0 {
            set(v, "beepTimer", fv(v, "beepTimer") - dt);
            if fv(v, "beepTimer") <= 0.0 {
                set(v, "beepTimer", rand2(1.5, 3.0));
                if self.ac.is_some() {
                    self.blip(400.0, 0.10, "sawtooth", 0.05, None, 0.0, 0.0);
                    self.blip(300.0, 0.12, "sawtooth", 0.04, None, 0.10, 0.0);
                }
            }
            // Tremble subtly.
            v.g.rotation().y += (self.perf_t * 8.0).sin() * 0.005;
            return;
        }
        // ---- follow mode: after petting, follow player at safe distance ----
        let following = v.follow_t > 0.0;
        if following {
            let gp = v.g.pos();
            let (dxp, dzp) = (ppx - gp.x, ppz - gp.z);
            let dp = hypot2(dxp, dzp);
            // Approach if far, stop if close.
            if dp > 1.8 {
                v.dir = dxp.atan2(dzp);
                let follow_speed = 0.9;
                let step = follow_speed * dt;
                let (fx, fz) = (v.dir.sin(), v.dir.cos());
                let nx = gp.x + fx * step;
                let nz = gp.z + fz * step;
                if !self.circle_hits_wall(nx, nz, 0.20) {
                    v.g.position().x = nx;
                    v.g.position().z = nz;
                }
            }
            set(v, "beepTimer", fv(v, "beepTimer") - dt);
            if fv(v, "beepTimer") <= 0.0 {
                set(v, "beepTimer", rand2(2.0, 3.5));
                if self.ac.is_some() {
                    self.blip(1400.0, 0.06, "sine", 0.05, None, 0.0, 0.0);
                }
            }
            let ry = v.g.rot().y;
            v.g.rotation().y = lerp(ry, v.dir, (dt * 6.0).min(1.0));
            set(v, "spinPhase", fv(v, "spinPhase") + dt * 6.0);
            return;
        }
        // ---- chase mode: for a few seconds after E-tap, follow the player. ----
        let chasing = fv(v, "chaseT") > 0.0;
        if chasing {
            set(v, "chaseT", fv(v, "chaseT") - dt);
            // aim at the player, faster than idle wander
            let gp = v.g.pos();
            let (dxp, dzp) = (ppx - gp.x, ppz - gp.z);
            let dp = hypot2(dxp, dzp);
            if dp > 0.001 {
                v.dir = dxp.atan2(dzp);
            }
            let chase_speed = 1.7;
            let step = chase_speed * dt;
            let (fx, fz) = (v.dir.sin(), v.dir.cos());
            let nx = gp.x + fx * step;
            let nz = gp.z + fz * step;
            if !self.circle_hits_wall(nx, nz, 0.20) {
                v.g.position().x = nx;
                v.g.position().z = nz;
            }
            // occasional beep while chasing
            set(v, "beepTimer", fv(v, "beepTimer") - dt);
            if fv(v, "beepTimer") <= 0.0 {
                set(v, "beepTimer", rand2(0.8, 1.3));
                if self.ac.is_some() {
                    self.blip(1200.0 + rand2(-100.0, 100.0), 0.05, "square", 0.06, None, 0.0, 0.0);
                }
            }
            let ry = v.g.rot().y;
            v.g.rotation().y = lerp(ry, v.dir, (dt * 8.0).min(1.0));
            // when chase ends, resume wandering by nudging the pick timer
            if fv(v, "chaseT") <= 0.0 {
                v.timer = 0.0;
            }
            // hum stays audible during chase — reuse the standard proximity code below
        } else {
            let speed = v.f.get("speed").copied().unwrap_or(0.45);
            let step = speed * dt;
            let (fx, fz) = (v.dir.sin(), v.dir.cos());
            let gp = v.g.pos();
            let nx = gp.x + fx * step;
            let nz = gp.z + fz * step;
            // check collision against walls / barricades using the standard grid check
            let wall_hit = self.circle_hits_wall(nx, nz, 0.20);
            // avoid the player so the vacuum reads as autonomous, not chasing
            let (dxp, dzp) = (nx - ppx, nz - ppz);
            let near_player = (dxp * dxp + dzp * dzp) < 0.7 * 0.7;
            if wall_hit || near_player || v.timer <= 0.0 {
                // pick a new heading (small chance of a 45° offset, otherwise a cardinal)
                let new_yaw = irand(0, 3) as f64 * (PI / 2.0) + if rnd() < 0.15 { PI / 4.0 } else { 0.0 };
                v.dir = new_yaw;
                v.timer = rand2(2.5, 5.5);
            } else {
                v.g.position().x = nx;
                v.g.position().z = nz;
            }
            // face movement direction; keep upright
            let ry = v.g.rot().y;
            v.g.rotation().y = lerp(ry, v.dir, (dt * 4.0).min(1.0));
        }
        // spin the side brush hubs
        set(v, "spinPhase", fv(v, "spinPhase") + dt * 6.0);
        // Home-LED colour swap: green when idle, yellow while chasing; blinks
        // slowly during chase so it reads as "active".
        if let Some(btn) = v.objs.get("homeBtn") {
            if let Some(m) = btn.try_mat() {
                if fv(v, "chaseT") > 0.0 {
                    let blink = 0.5 + 0.5 * (fv(v, "spinPhase") * 1.5).sin();
                    let r = 0.75 + blink * 0.25;
                    let g_ch = 0.45 + blink * 0.30;
                    let b_ch = 0.10;
                    m.m().color.set_rgb(r, g_ch, b_ch);
                } else {
                    m.m().color.set_hex(0x60ff90);
                }
            }
        }
        // hum volume ramps with proximity (soothing, never overpowering)
        if let Some(n) = self.audio.vacuum_gain.as_ref() {
            let gp = v.g.pos();
            let (dx, dz) = (gp.x - ppx, gp.z - ppz);
            let d = hypot2(dx, dz);
            let target = clamp(0.11 - d * 0.011, 0.0, 0.11);
            let gv = n.gain().value();
            n.gain().set_value(gv + (target - gv) * (dt * 3.0).min(1.0));
        }
    }

    /// The place's footprints dry away (they come up wet in a breath and
    /// fade over their life); also ticks the cry cooldown and the cry-dim
    /// envelope.
    pub fn update_way(&mut self, dt: f64) {
        if self.cry_cooldown > 0.0 {
            self.cry_cooldown -= dt;
        }
        if self.cry_dim_t > 0.0 {
            self.cry_dim_t -= dt;
        }
        // Cry-dim envelope: rise fast (0.7 s to full), fall slow (2 s)
        let cry_dim_target = if self.cry_dim_t > 0.0 { 1.0 } else { 0.0 };
        let cry_dim_k = if cry_dim_target > self.cry_dim_lerp { (dt / 0.7).min(1.0) } else { (dt / 2.0).min(1.0) };
        self.cry_dim_lerp += (cry_dim_target - self.cry_dim_lerp) * cry_dim_k;
        let mut i = self.way_prints.len();
        while i > 0 {
            i -= 1;
            self.way_prints[i].life -= dt;
            if self.way_prints[i].life <= 0.0 {
                let a = self.way_prints.remove(i);
                self.world.remove(&a.mesh);
                continue;
            }
            let a = &self.way_prints[i];
            a.mesh.mat().set_opacity(way_print_opacity(a.life));
        }
    }

    pub fn update_player(&mut self, dt: f64) {
        let wk = self.weather_key();
        use crate::session::input::Act;
        // a cutscene has your hands: your body stays as it is (no step, no
        // run, the crouch kept), and only its timers and camera go on
        let still = self.hands_taken();
        let (mut mx, mut mz) = (0.0f64, 0.0f64);
        if self.held(Act::Forward) && !still {
            mz += 1.0;
        }
        if self.held(Act::Back) && !still {
            mz -= 1.0;
        }
        // (with the notebook up the arrows turn its pages instead)
        let strafe =
            |g: &Game, a: Act| g.settings.keys(a).iter().any(|k| g.keys.contains(k) && !(g.journal_open && k.starts_with("Arrow")));
        if strafe(self, Act::Left) && !still {
            mx -= 1.0;
        }
        if strafe(self, Act::Right) && !still {
            mx += 1.0;
        }
        // a run by a press lasts until you stop
        if mx == 0.0 && mz == 0.0 && !self.touch_input.active && !still {
            self.input.run_latched = false;
        }
        // crouch: Ctrl held, or C until C again; running stands you up
        let run_key = self.wants_sprint() && (mx != 0.0 || mz != 0.0);
        if run_key && self.input.crouch_latched && !self.held(Act::Crouch) {
            self.input.crouch_latched = false;
        }
        if !still {
            self.player.crouch = self.wants_crouch();
        }
        let sprinting = !still && !self.player.crouch && self.wants_sprint() && self.sprint_ready();
        // touch stick is analog: it replaces the keys rather than adding to them
        if self.touch_input.active && !still {
            mx = self.touch_input.x;
            mz = self.touch_input.z;
        }
        let moving = mx != 0.0 || mz != 0.0;
        // detect whether the player is currently inside the level's pool
        let in_pool = match self.pool_bounds {
            Some(pb) => {
                self.player.pos.x > pb.min_x && self.player.pos.x < pb.max_x && self.player.pos.z > pb.min_z && self.player.pos.z < pb.max_z
            }
            None => false,
        };
        // water slows movement and dampens sprint; your size (`secrets::keeps`)
        let water_mul = if in_pool { 0.55 } else { 1.0 } * self.body.pace;
        let speed = (if sprinting && moving {
            SPRINT
        } else if self.player.crouch {
            CROUCH_WALK
        } else {
            WALK
        }) * water_mul;
        // Audible range — how far entities hear the player.
        let mut aud = 0.0;
        if moving {
            if self.player.crouch {
                aud = 3.5;
            } else if sprinting {
                aud = 13.0;
            } else {
                aud = 6.5;
            }
        }
        if in_pool && moving {
            aud += 3.0;
        }
        self.player.audible_range = aud;
        // splash on entering the pool
        if in_pool && !self.player.was_in_pool {
            self.sfx_splash(true);
        }
        if !in_pool && self.player.was_in_pool {
            self.sfx_splash(false);
        }
        self.player.was_in_pool = in_pool;
        if moving {
            let l = hypot2(mx, mz);
            // analog throttle: how far the touch stick is pushed. 1 for the keyboard.
            let analog = if self.touch_input.active { l.min(1.0) } else { 1.0 };
            mx /= l;
            mz /= l;
            let yaw = self.player.yaw;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let (rx, rz) = (yaw.cos(), -yaw.sin());
            let dx = (fx * mz + rx * mx) * speed * analog * dt;
            let dz = (fz * mz + rz * mx) * speed * analog * dt;
            let mut p = self.player.pos;
            self.collide_move(&mut p, dx, dz, PLAYER_R);
            // on the Building's floors the body stands on the floor under it
            // (a stair's run, a gallery); the rooms are all at y = 0
            if !self.ground.is_empty() {
                p.y = self.floor_y_at(p.x, p.z) + EYE;
            }
            self.player.pos = p;
            // Bob tempo increases with descent
            self.player.bob_phase += dt * speed * 2.1 * self.descent_boost_mul(0.15);
            // Bob amplitude scaled per weather + descent boost.
            let bob_mul = bob_amp_mul_by_weather(wk).unwrap_or(1.0) * self.descent_amp_boost();
            let bob_target = (if sprinting {
                0.075
            } else if self.player.crouch {
                0.025
            } else {
                0.045
            }) * bob_mul;
            self.player.bob_amp = lerp(self.player.bob_amp, bob_target, dt * 6.0);
            // step sfx: crouched has a subtle shuffle; pool = splash; else walk/run.
            // Crouch step cadence per weather.
            let crouch_step_len = if self.player.crouch { Some(wpick(crouch_step_len_by_weather, wk)) } else { None };
            let step_len = if in_pool {
                2.2
            } else if sprinting {
                3.4
            } else if self.player.crouch {
                crouch_step_len.unwrap_or(0.0)
            } else {
                2.6
            };
            if self.player.bob_phase - self.last_step > step_len {
                self.last_step = self.player.bob_phase;
                self.footstep_fx(wk, in_pool, sprinting);
            }
        } else {
            self.player.bob_amp = lerp(self.player.bob_amp, 0.0, dt * 8.0);
        }
        // stamina
        if sprinting && moving {
            self.player.stamina -= STAM_DRAIN * dt;
            self.player.stam_delay = 0.8;
        } else if self.player.stam_delay > 0.0 {
            self.player.stam_delay -= dt;
        } else {
            self.player.stamina += STAM_REGEN * self.body.wind * dt;
        }
        self.player.stamina = clamp(self.player.stamina, 0.0, self.body.stam_max);

        // flashlight battery drain and auto-off when depleted
        if self.player.flashlight_on {
            self.player.battery -= BAT_DRAIN * dt;
            if self.player.battery <= 0.0 {
                self.player.battery = 0.0;
                self.player.flashlight_on = false;
                self.show_msg("BATTERY DEAD", 1.5);
                // your asides: the lamp gone, where it is dark
                if self.in_dark_region() {
                    self.aside(crate::audio::asides::Use::Dark, "");
                }
            }
        }
        // smooth flashlight intensity (brighter so it cuts through the dark sections)
        let target_int = if self.player.flashlight_on { 14.0 } else { 0.0 };
        let fi = self.flashlight.intensity();
        self.flashlight.set_intensity(fi + (target_int - fi) * (dt * 12.0).min(1.0));
        // eye height (crouch); hiding (the Showroom) lowers the eye further
        let eye_target = match self.player.hide_eye {
            Some(e) => e,
            None => {
                if self.player.crouch {
                    CROUCH_EYE
                } else {
                    EYE
                }
            }
        };
        // your size (`secrets::keeps`): it comes over you over a few seconds
        let size_to = if self.player.crouch { self.body.crouch_eye } else { self.body.eye };
        self.size_eye += (size_to - self.size_eye).clamp(-SIZE_EASE * dt, SIZE_EASE * dt);
        let eye_target = if self.player.hide_eye.is_some() { eye_target } else { eye_target + self.size_eye };
        let eye_k = (dt * if self.player.hide_eye.is_some() { 5.0 } else { 9.0 }).min(1.0);
        self.player.eye_h = lerp(self.player.eye_h, eye_target, eye_k);

        // camera
        // Settings: head bob and camera shake scale (the draws stay, so the
        // per-frame stream does not depend on them)
        let (bob_k, shake_k) = (self.settings.bob, if self.settings.shake { 1.0 } else { 0.0 });
        let bob_y = self.player.bob_phase.sin().abs() * self.player.bob_amp * bob_k;
        let insanity_shake =
            if self.player.sanity < 40.0 { rand2(-1.0, 1.0) * (1.0 - self.player.sanity / 40.0) * 0.02 * shake_k } else { 0.0 };
        // Idle breathing: weather-scaled amp+freq; at low sanity
        // the breath quickens and gets shallower.
        let idle_t = self.clock_ms() / 1000.0;
        let idle_weight = (1.0 - self.player.bob_amp / 0.02).max(0.0); // 1 at idle, 0 walking
        let idle_spec = wpick(idle_breath_by_weather, wk);
        let san_frac = self.player.sanity / SAN_MAX;
        let panic_mag = if san_frac < 0.30 { 1.0 - san_frac / 0.30 } else { 0.0 };
        let idle_freq_mul = 1.0 + panic_mag * 0.6;
        let idle_amp_mul = 1.0 - panic_mag * 0.30;
        let idle_freq_dsc = self.descent_freq_boost();
        let idle_breath = (idle_t * idle_spec.freq * idle_freq_mul * idle_freq_dsc).sin()
            * idle_spec.amp
            * idle_amp_mul
            * idle_weight
            * self.descent_amp_boost();
        let cam = self.camera.clone();
        {
            let cx = self.player.pos.x + (self.player.bob_phase * 0.5).cos() * self.player.bob_amp * 0.4 * bob_k;
            let shake = if self.shake_t > 0.0 { rand2(-1.0, 1.0) * self.shake_t * 0.06 * shake_k } else { 0.0 };
            let cy =
                self.floor_y_at(self.player.pos.x, self.player.pos.z) + self.player.eye_h + bob_y + idle_breath + shake + insanity_shake;
            cam.position().set(cx, cy, self.player.pos.z);
        }
        cam.rotation().y = self.player.yaw;
        // camera pitch, plus a brief downward "lens dip" — decays over ~0.35 s
        let mut pitch_dip = 0.0;
        if self.camera_dip_t > 0.0 {
            self.camera_dip_t -= dt;
            let k = (self.camera_dip_t / 0.35).max(0.0);
            pitch_dip = -0.22 * k * k * shake_k; // 12.6° max downward
        }
        // Crouch tilt: weather magnitude + sanity boost
        let crouch_tilt_mag_target = wpick(crouch_tilt_by_weather, wk);
        self.crouch_tilt_mag_current += (crouch_tilt_mag_target - self.crouch_tilt_mag_current) * (dt * 1.5).min(1.0);
        let san_frac2 = self.player.sanity / SAN_MAX;
        let sanity_crouch_boost = 1.0 + if san_frac2 < 0.30 { (1.0 - san_frac2 / 0.30) * 0.50 } else { 0.0 };
        // Low-stamina wobble while crouched
        let mut stamina_crouch_wobble = 0.0;
        if self.player.crouch && self.player.stamina < 40.0 {
            let fatigue_mag = 1.0 - self.player.stamina / 40.0;
            let wob_amp = wpick(stamina_wobble_amp_by_weather, wk);
            let wob_freq = wpick(stamina_wobble_freq_by_weather, wk);
            stamina_crouch_wobble = (self.perf_t * wob_freq).sin() * wob_amp * fatigue_mag;
        }
        let crouch_pitch_target =
            if self.player.crouch { self.crouch_tilt_mag_current * sanity_crouch_boost + stamina_crouch_wobble } else { 0.0 };
        self.player.crouch_pitch_lerp += (crouch_pitch_target - self.player.crouch_pitch_lerp) * (dt * 5.0).min(1.0);
        // Low-stamina bent-over dip when standing
        let dip_mag = wpick(stamina_dip_mag_by_weather, wk);
        let stamina_dip =
            if !self.player.crouch && self.player.stamina < 30.0 { (1.0 - self.player.stamina / 30.0) * dip_mag } else { 0.0 };
        cam.rotation().x = self.player.pitch + pitch_dip - self.player.crouch_pitch_lerp - stamina_dip;
        // Sanity roll amp / freq per weather × descent boost
        let sanity_roll_amp_target = wpick(sanity_roll_amp_by_weather, wk) * self.descent_amp_boost();
        let sanity_roll_freq_target = wpick(sanity_roll_freq_by_weather, wk) * self.descent_freq_boost();
        let k_sanity = (dt * 1.5).min(1.0);
        self.sanity_roll_amp_current += (sanity_roll_amp_target - self.sanity_roll_amp_current) * k_sanity;
        self.sanity_roll_freq_current += (sanity_roll_freq_target - self.sanity_roll_freq_current) * k_sanity;
        let roll_t = mx * (if sprinting { -0.02 } else { -0.012 })
            + if self.player.sanity < 50.0 {
                (self.perf_t * self.sanity_roll_freq_current).sin() * (1.0 - self.player.sanity / 50.0) * self.sanity_roll_amp_current
            } else {
                0.0
            };
        let rz = cam.rot().z;
        cam.rotation().z = lerp(rz, roll_t, dt * 8.0);
        // Settings' field of view is the walking one; running widens it by 7°, crouching narrows it by 6°
        let base_fov = self.settings.fov
            + if sprinting && moving {
                7.0
            } else if self.player.crouch {
                -6.0
            } else {
                0.0
            };
        // Reality-shift FOV nudge
        let shift_spec_fov = wpick(shift_bend_by_weather, wk);
        let shift_fov_delta = if self.reality_shift_t > 0.0 {
            ((2.0 - self.reality_shift_t) * PI * 1.2).sin() * (self.reality_shift_t / 2.0) * shift_spec_fov.fov
        } else {
            0.0
        };
        let aspect = cam.camera().aspect;
        let target_fov = crate::session::settings::vertical_fov(base_fov + shift_fov_delta, aspect);
        let fov = cam.camera().fov;
        if (fov - target_fov).abs() > 0.05 {
            cam.camera().fov = lerp(fov, target_fov, dt * 5.0); // the renderer derives the projection matrix from it
        }
        self.player_light.position().set(self.player.pos.x, 2.15, self.player.pos.z);
        // flashlight rig follows camera
        let cp = cam.pos();
        self.flashlight.position().copy(&cp);
        let fwd = self.player.fwd();
        let pitch_cos = self.player.pitch.cos();
        // hands that shake (a third sweet: `secrets::keeps`): the beam
        // trembles, a hand's few hertz on the game's clock, no draws
        let (tx, ty) = if self.body.shake {
            let t = self.perf_t;
            (0.035 * ((t * 7.1).sin() + 0.6 * (t * 11.3).sin()), 0.03 * ((t * 8.3).sin() + 0.5 * (t * 13.7).cos()))
        } else {
            (0.0, 0.0)
        };
        self.flashlight_target.position().set(
            cp.x + fwd.x * pitch_cos * 5.0 - fwd.z * tx,
            cp.y + self.player.pitch.sin() * 5.0 + ty,
            cp.z + fwd.z * pitch_cos * 5.0 + fwd.x * tx,
        );

        // almond water pickup
        for i in 0..self.almonds.len() {
            if self.almonds[i].taken {
                continue;
            }
            let ap = self.almonds[i].g.pos();
            let (adx, adz) = (ap.x - self.player.pos.x, ap.z - self.player.pos.z);
            if adx * adx + adz * adz < 0.95 {
                self.almonds[i].taken = true;
                self.almonds[i].g.set_visible(false);
                self.player.bottles += 1;
                self.sfx_pickup();
                let t = self.pick_text("almondPickup");
                self.show_msg(&t, 1.6);
                self.hint_first_use("bottle");
            }
        }
        // battery pickup
        for i in 0..self.battery_items.len() {
            if self.battery_items[i].taken {
                continue;
            }
            let bp = self.battery_items[i].g.pos();
            let (bdx, bdz) = (bp.x - self.player.pos.x, bp.z - self.player.pos.z);
            if bdx * bdx + bdz * bdz < 0.95 {
                self.battery_items[i].taken = true;
                self.battery_items[i].g.set_visible(false);
                self.player.batteries += 1;
                self.sfx_pickup();
                let t = self.pick_text("batteryPickup");
                self.show_msg(&t, 1.6);
                self.hint_first_use("battery");
            }
        }
        // floor hole check — step onto one and you fall a level
        for i in 0..self.floor_holes.len() {
            let hole = self.floor_holes[i];
            let (hx, hz) = (hole.x - self.player.pos.x, hole.z - self.player.pos.z);
            if hx * hx + hz * hz < hole.r * hole.r {
                self.begin_fall_down();
                return;
            }
        }
        // paper pickup
        for i in 0..self.papers.len() {
            if self.papers[i].taken {
                continue;
            }
            let pp = self.papers[i].g.pos();
            let (pdx, pdz) = (pp.x - self.player.pos.x, pp.z - self.player.pos.z);
            if pdx * pdx + pdz * pdz < 0.95 {
                self.papers[i].taken = true;
                self.papers[i].g.set_visible(false);
                let gained = irand(3, 8);
                self.player.papers += gained as i32;
                self.sfx_pickup();
                self.show_msg(&format!("PAPER STACK — {} SHEETS", gained), 1.6);
            }
        }
        // lore note pickup — first-time reads show the body immediately + short chime
        self.lore_note_pickups(dt);
        // dark-zone effect: no lights in region + no flashlight = sanity drain + heartbeat
        let in_dark = self.in_dark_region();
        // safe-room bonus: standing in the candlelit alcove regenerates sanity fast
        let mut in_safe_room = false;
        if let Some(sr) = self.safe_room.as_ref() {
            let (sdx, sdz) = (sr.x - self.player.pos.x, sr.z - self.player.pos.z);
            in_safe_room = (sdx * sdx + sdz * sdz) < sr.r * sr.r;
            let discovered = sr.discovered;
            // found: nothing chimes and nothing is said (what the candles do
            // is learned there, never told: `docs/LORE.md`, *What still
            // burns*; PILLARS.md 4, no glow, chime or line)
            if in_safe_room && !discovered {
                self.safe_room.as_mut().unwrap().discovered = true;
            }
            // While inside, what a nerve near its end hears holds off: the
            // whispers' 10 s never builds up, and the heartbeat's countdown
            // is held above zero (reset to zero, it had beaten every frame)
            if in_safe_room {
                self.crit_sanity_accum = 0.0;
                self.sanity_beat_t = self.sanity_beat_t.max(1.0);
                self.sanity_whisper_t = 0.0;
            }
        }
        // (no glow on the screen: the candles' own light is the warmth, and
        // the screen at rest is the place alone, PILLARS.md 4)
        // Safe-room hum, from the bench: full within 1.5 m of its candles,
        // gone by 3 m, duller through a wall, panned to the bench's side;
        // with the flashlight on and a low battery the target dips up to 25%.
        if self.audio.safe_room_gain.is_some() {
            let (near, pan) = match self.safe_room.as_ref() {
                Some(sr) => {
                    // (the sight line, a walk of the level's walls, only
                    // within the 3 m it is heard in)
                    let (x, z) = (sr.x, sr.z - 0.7);
                    let (d, pan) = self.ear_pan(x, z);
                    let clear = d < 3.0 && self.ear(x, z).2;
                    (clamp((3.0 - d) / 1.5, 0.0, 1.0) * if clear { 1.0 } else { 0.45 }, pan)
                }
                None => (0.0, 0.0),
            };
            let batt_dip = if self.player.flashlight_on { self.beam_battery_scale(0.75) } else { 1.0 };
            // (√2: the equal-power pan takes 3 dB off a centred mono hum)
            let target = 0.08 * std::f64::consts::SQRT_2 * near * batt_dip;
            let k = (dt * 2.5).min(1.0);
            let n = self.audio.safe_room_gain.as_ref().unwrap();
            let gv = n.gain().value();
            n.gain().set_value(gv + (target - gv) * k);
            if let Some(p) = self.audio.safe_room_panner.as_ref() {
                let pv = p.pan().value();
                p.pan().set_value(pv + (pan - pv) * k);
            }
        }
        if in_dark && !self.player.flashlight_on {
            // the dark frightens you; it does not leave you defenceless: it
            // takes your nerve down to `DARK_FLOOR`, and only the creatures
            // take it further, for a while (it comes back to the floor as it
            // does anywhere). A dead lamp in a dark zone had spent the nerve
            // of every bot that died, leaving any touch fatal (E8)
            let floor = self.body.dark_floor;
            if self.player.sanity > floor {
                self.player.sanity = (self.player.sanity - 6.5 * self.body.dark * dt).max(floor);
            } else {
                self.player.sanity = (self.player.sanity + SAN_REGEN * self.body.nerve * dt).min(floor);
            }
            self.dark_heart_t += dt;
            if self.dark_heart_t > 0.85 {
                self.dark_heart_t = 0.0;
                self.sfx_thud();
            }
        } else if in_safe_room {
            // ~3x the normal regen while standing in the safe room
            self.player.sanity = clamp(self.player.sanity + SAN_REGEN * 3.2 * self.body.nerve * dt, 0.0, SAN_MAX);
        } else if self.player.sanity < SAN_MAX {
            self.player.sanity = clamp(self.player.sanity + SAN_REGEN * self.body.nerve * dt, 0.0, SAN_MAX);
        }
        // motion-triggered light switches (only faulty ones)
        self.light_switch_motion(dt);
        if self.shake_t > 0.0 {
            self.shake_t -= dt;
        }
    }

    /// The footstep block of `update_player`: one step's sfx
    /// and the per-weather footstep effects.
    fn footstep_fx(&mut self, wk: &'static str, in_pool: bool, sprinting: bool) {
        let crouch = self.player.crouch;
        if in_pool {
            self.sfx_splash(false);
        } else if !crouch {
            if let Some(s) = self.soundscape() {
                let surf = self.floor_surface_at(self.player.pos.x, self.player.pos.z);
                s.footstep(surf, if sprinting { 1.0 } else { 0.62 }, None);
            } else {
                self.sfx_step(sprinting);
            }
        }
        // Crouch footstep: very quiet drag/shuffle noise on each crouched step
        else if !in_pool && self.ac.is_some() {
            let crouch_step_gain = wpick(crouch_step_gain_by_weather, wk);
            let crouch_step_freq = wpick(crouch_step_freq_by_weather, wk);
            self.noise_burst(0.06, crouch_step_freq, 0.6, crouch_step_gain, "lowpass", None, 0.0, 0.0);
            // Crouch dust puff (derelict only)
            if wk == "derelict" {
                if let Some(i) = self.free_scuff() {
                    let s = self.scuffs[i].obj.clone();
                    s.mat().m().color.set_hex(0xa08060);
                    let x = self.player.pos.x + rand2(-0.10, 0.10);
                    let z = self.player.pos.z + rand2(-0.10, 0.10);
                    s.position().set(x, 0.08, z);
                    s.set_visible(true);
                    s.mat().set_opacity(0.20);
                    s.scale().set_scalar(0.15);
                    self.scuffs[i].t = 0.5;
                    self.crouch_fx_tally.derelict += 1.0;
                }
            }
            // Crouch wet ripple
            if wk == "wet" {
                if let Some(i) = self.free_ripple() {
                    let r = self.ripples[i].obj.clone();
                    let x = self.player.pos.x + rand2(-0.08, 0.08);
                    let z = self.player.pos.z + rand2(-0.08, 0.08);
                    r.position().set(x, 0.015, z);
                    r.set_visible(true);
                    r.mat().set_opacity(0.15);
                    r.scale().set(0.15, 0.15, 1);
                    self.ripples[i].t = 0.7;
                }
                self.crouch_fx_tally.wet += 1.0;
            }
            // Crouch electric spark: very rare (12%) and quieter
            if wk == "electric" && rnd() < 0.12 {
                if let Some(i) = self.free_scuff() {
                    let s = self.scuffs[i].obj.clone();
                    s.mat().m().color.set_hex(0xc8e0ff);
                    let x = self.player.pos.x + rand2(-0.08, 0.08);
                    let z = self.player.pos.z + rand2(-0.08, 0.08);
                    s.position().set(x, 0.06, z);
                    s.set_visible(true);
                    s.mat().set_opacity(0.35);
                    s.scale().set_scalar(0.10);
                    self.scuffs[i].t = 0.25;
                }
                self.blip(2200.0 + rand2(-80.0, 80.0), 0.03, "square", 0.006, None, 0.0, 0.0);
                self.crouch_fx_tally.electric += 1.0;
            }
            // Crouch haunted echo: 10%, half-pan, halved gain
            if wk == "haunted" && rnd() < 0.10 {
                let echo_pan = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.35 + rnd() * 0.15);
                self.blip(90.0 + rand2(-6.0, 6.0), 0.09, "sine", 0.010, Some(55.0), 0.30, echo_pan);
                self.noise_burst(0.04, 160.0, 0.8, 0.006, "lowpass", None, 0.32, echo_pan);
                self.crouch_fx_tally.haunted += 1.0;
            }
            // Crouch still hush
            if wk == "still" {
                self.noise_burst(0.05, 160.0, 0.5, 0.007, "lowpass", None, 0.0, 0.0);
                self.crouch_fx_tally.still += 1.0;
            }
            // Crouch normal click: 4%
            if wk == "normal" && rnd() < 0.04 {
                self.blip(3000.0 + rand2(-150.0, 150.0), 0.02, "square", 0.006, None, 0.0, 0.0);
                self.crouch_fx_tally.normal += 1.0;
            }
        }
        // Wet-weather footstep ripple + quiet splish; a sprint intensifies
        if !crouch && !in_pool && wk == "wet" {
            let s_mul = if sprinting { 1.0 + 0.5 * (self.player.stamina / STAM_MAX) } else { 1.0 };
            if let Some(i) = self.free_ripple() {
                let r = self.ripples[i].obj.clone();
                let x = self.player.pos.x + rand2(-0.15, 0.15);
                let z = self.player.pos.z + rand2(-0.15, 0.15);
                r.position().set(x, 0.015, z);
                r.set_visible(true);
                r.mat().set_opacity(0.30 * s_mul);
                r.scale().set(0.4 * s_mul, 0.4 * s_mul, 1);
                self.ripples[i].t = 0.9;
            }
            self.noise_burst(0.05, 900.0 + rand2(-100.0, 100.0), 1.2, 0.020 * s_mul, "bandpass", None, 0.0, 0.0);
            self.footstep_fx_tally.wet += 1.0;
        }
        // Derelict-weather footstep dust scuff; a sprint makes a bigger cloud.
        if !crouch && !in_pool && wk == "derelict" {
            let s_mul = if sprinting { 1.0 + 0.5 * (self.player.stamina / STAM_MAX) } else { 1.0 };
            if let Some(i) = self.free_scuff() {
                let s = self.scuffs[i].obj.clone();
                s.mat().m().color.set_hex(0xa08060);
                let x = self.player.pos.x + rand2(-0.15, 0.15);
                let z = self.player.pos.z + rand2(-0.15, 0.15);
                s.position().set(x, 0.12, z);
                s.set_visible(true);
                s.mat().set_opacity(0.55 * s_mul);
                s.scale().set_scalar(0.35 * s_mul);
                self.scuffs[i].t = 0.7;
            }
            self.noise_burst(0.06, 1600.0 + rand2(-100.0, 100.0), 1.5, 0.014 * s_mul, "highpass", None, 0.0, 0.0);
            self.footstep_fx_tally.derelict += 1.0;
        }
        // Electric-weather footstep spark; sprint and stamina scale it
        if !crouch && !in_pool && wk == "electric" {
            let stam_f = if sprinting { self.player.stamina / STAM_MAX } else { 0.0 };
            let sprint_prob = 0.35 + 0.20 * stam_f; // 0.35–0.55
            if rnd() < sprint_prob {
                let s_mul = 1.0 + 0.3 * stam_f; // 1.0–1.3
                if let Some(i) = self.free_scuff() {
                    let s = self.scuffs[i].obj.clone();
                    s.mat().m().color.set_hex(0xc8e0ff);
                    let x = self.player.pos.x + rand2(-0.10, 0.10);
                    let z = self.player.pos.z + rand2(-0.10, 0.10);
                    s.position().set(x, 0.10, z);
                    s.set_visible(true);
                    s.mat().set_opacity(0.80 * s_mul);
                    s.scale().set_scalar(0.20 * s_mul);
                    self.scuffs[i].t = 0.35;
                }
                self.blip(2400.0 + rand2(-100.0, 100.0), 0.04, "square", 0.014 * s_mul, None, 0.0, 0.0);
                self.footstep_fx_tally.electric += 1.0;
            }
        }
        // Haunted-weather footstep echo: a shadow step behind the player,
        // delayed 250 ms and panned hard to one side (35% of steps).
        if !crouch && !in_pool && wk == "haunted" && rnd() < 0.35 {
            let echo_pan = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.60 + rnd() * 0.25);
            self.blip(95.0 + rand2(-8.0, 8.0), 0.10, "sine", 0.018, Some(55.0), 0.25, echo_pan);
            self.noise_burst(0.05, 180.0, 0.8, 0.010, "lowpass", None, 0.27, echo_pan);
            self.footstep_fx_tally.haunted += 1.0;
        }
        // Still-weather footstep hush (every step)
        if !crouch && !in_pool && wk == "still" {
            self.noise_burst(0.10, 180.0, 0.5, 0.015, "lowpass", None, 0.0, 0.0);
            self.footstep_fx_tally.still += 1.0;
        }
        // Normal-weather footstep click: rare (1 in 8)
        if !crouch && !in_pool && wk == "normal" && rnd() < 0.125 {
            self.blip(3200.0 + rand2(-200.0, 200.0), 0.025, "square", 0.014, None, 0.0, 0.0);
            self.footstep_fx_tally.normal += 1.0;
        }
    }

    /// The first free (expired) dust scuff.
    fn free_scuff(&self) -> Option<usize> {
        self.scuffs.iter().position(|s| s.t <= 0.0)
    }
    /// The first free (expired) ripple.
    fn free_ripple(&self) -> Option<usize> {
        self.ripples.iter().position(|r| r.t <= 0.0)
    }

    /// The lore-note block of `update_player`.
    fn lore_note_pickups(&mut self, dt: f64) {
        let mut i = 0;
        while i < self.lore_notes.len() {
            let ni = i;
            i += 1;
            if self.lore_notes[ni].taken {
                continue;
            }
            let ng = self.lore_notes[ni].g.clone();
            let id = self.lore_notes[ni].id;
            let np = ng.pos();
            let (ndx, ndz) = (np.x - self.player.pos.x, np.z - self.player.pos.z);
            let dist2 = ndx * ndx + ndz * ndz;
            // Read-note whisper: approaching a note already read plays a faint
            // whisper of its title once per approach ("was near" flag + cooldown).
            if self.lore_found.contains(&id) {
                let mut cd = ng.r().user.get("whisperCooldown");
                if cd > 0.0 {
                    cd -= dt;
                    ng.user().set("whisperCooldown", cd);
                }
                let near_now = dist2 < 3.0; // ~1.7 m — wider than pickup radius
                let was_near = ng.r().user.get("wasNearRead") != 0.0;
                if near_now && !was_near && cd <= 0.0 && self.ac.is_some() {
                    ng.user().set("whisperCooldown", 15.0);
                    ng.user().set("wasNearRead", 1.0);
                    // Faint low-freq whisper — deniable.
                    self.noise_burst(0.6, 900.0, 2.5, 0.04, "bandpass", None, 0.0, 0.0);
                    self.blip(160.0 + (id % 40) as f64, 0.5, "sine", 0.035, Some(90.0), 0.15, 0.0);
                    // Subtle title flash — only if no HUD toast is up.
                    if self.msg_t <= 0.0 {
                        let arc = self.lore_arc_for(id);
                        let w = self.whisper_for(arc, Some(id));
                        if let Some(text) = w.text {
                            self.ui.msg.set_text(&text);
                            self.msg_text = text;
                            self.ui.msg.set_opacity(w.opacity);
                            if let Some(glow) = self.whisper_glow_for(arc) {
                                self.ui.msg.set_style("color", &glow.hex);
                                let ts = format!(
                                    "0 0 {}px rgba({}, {}, {}, {})",
                                    fmt_fixed(glow.blur, 1),
                                    glow.r,
                                    glow.g,
                                    glow.b,
                                    fmt_fixed(glow.alpha, 2)
                                );
                                self.ui.msg.set_style("textShadow", &ts);
                            }
                            self.msg_t = w.duration;
                        }
                    }
                }
                if !near_now {
                    ng.user().set("wasNearRead", 0.0);
                }
                continue;
            }
            if dist2 < 0.9 {
                self.lore_notes[ni].taken = true;
                ng.set_visible(false);
                self.lore_found.insert(id);
                self.sfx_pickup();
                self.sfx_chime();
                let title = self.lore_entry_title(id);
                self.show_msg(&format!("NOTE FOUND — {} (J to read)", title), 2.2);
                // Arc completion: if this pickup finished an arc, a special toast + chord.
                for (name, ids) in self.lore_arc_list() {
                    if self.lore_arcs_completed.contains(&name) {
                        continue;
                    }
                    if ids.iter().all(|aid| self.lore_found.contains(aid)) {
                        self.lore_arcs_completed.insert(name.clone());
                        self.schedule(2400.0, move |g| {
                            g.show_msg(&format!("STORY UNCOVERED — {}", name), 3.0);
                            if g.ac.is_some() {
                                // three-note rising chord distinct from the standard chime
                                g.blip(660.0, 0.9, "sine", 0.10, None, 0.0, 0.0);
                                g.blip(880.0, 1.1, "sine", 0.09, None, 0.10, 0.0);
                                g.blip(1320.0, 1.3, "sine", 0.07, None, 0.22, 0.0);
                            }
                        });
                        break; // only fire one per pickup
                    }
                }
                self.save_game();
                let (entries, arcs) = (self.lore_entries_len(), self.lore_arc_list().len());
                // Achievement: player has read every lore entry
                if self.lore_found.len() >= entries {
                    self.fire_archivist_achievement();
                }
                // Achievement: every lore arc completed (after the arc toast)
                if self.lore_arcs_completed.len() >= arcs && !self.storyteller_shown {
                    self.schedule(4500.0, |g| g.fire_storyteller_achievement());
                }
                // *ARCHIVE* recognition — the first time both gates hold.
                if !self.archive_milestone_shown && self.lore_found.len() >= entries && self.lore_arcs_completed.len() >= arcs {
                    self.archive_milestone_shown = true;
                    self.schedule(8000.0, |g| g.show_msg("THE ARCHIVE IS COMPLETE", 2.6));
                }
            }
        }
    }

    /// The light-switch block of `update_player`: two-phase
    /// trigger with a fair-play warning flicker, until the level's faulty trip
    /// count reaches 3 (then they flip at once).
    fn light_switch_motion(&mut self, dt: f64) {
        fn sf(sw: &LightSwitch, k: &str) -> f64 {
            sw.f.get(k).copied().unwrap_or(0.0)
        }
        let mut si = 0;
        while si < self.light_switches.len() {
            let i = si;
            si += 1;
            if !self.light_switches[i].faulty {
                continue;
            }
            let (px, pz) = (self.player.pos.x, self.player.pos.z);
            let (in_range, was_in_range, mc, arm_t, panel_idx) = {
                let sw = &mut self.light_switches[i];
                let mc = sf(sw, "motionCooldown");
                if mc > 0.0 {
                    sw.f.insert("motionCooldown".into(), mc - dt);
                }
                let (dxs, dzs) = (sw.x - px, sw.z - pz);
                let in_range = (dxs * dxs + dzs * dzs) < 4.0; // 2 m radius
                (in_range, sw.b.get("wasInRange").copied().unwrap_or(false), sf(sw, "motionCooldown"), sf(sw, "armT"), sw.panels.clone())
            };
            if in_range && !was_in_range && mc <= 0.0 && !(arm_t > 0.0) {
                if self.faulty_trip_count >= 3.0 {
                    // ESCALATED: no warning flicker, flip immediately with a small shake
                    self.flip_light_switch(i, "motion");
                    if let Some(sw) = self.light_switches.get_mut(i) {
                        sw.f.insert("motionCooldown".into(), 8.0);
                    }
                    self.shake_t = self.shake_t.max(0.25);
                    self.faulty_trip_count += 1.0;
                    self.maybe_fire_escalation_toast();
                } else {
                    // Normal: arm the warning phase (panels flicker as a fair-play cue)
                    if let Some(sw) = self.light_switches.get_mut(i) {
                        sw.f.insert("armT".into(), 0.75);
                    }
                    for idx in panel_idx {
                        if let Some(pn) = self.panels.get_mut(idx) {
                            pn.flicker = 0.75;
                        }
                    }
                }
            }
            let arm_t = self.light_switches.get(i).map(|sw| sf(sw, "armT")).unwrap_or(0.0);
            if arm_t > 0.0 {
                let a = arm_t - dt;
                if let Some(sw) = self.light_switches.get_mut(i) {
                    sw.f.insert("armT".into(), a);
                }
                if a <= 0.0 {
                    self.flip_light_switch(i, "motion");
                    if let Some(sw) = self.light_switches.get_mut(i) {
                        sw.f.insert("motionCooldown".into(), 8.0);
                    }
                    self.faulty_trip_count += 1.0;
                    self.maybe_fire_escalation_toast();
                }
            }
            if let Some(sw) = self.light_switches.get_mut(i) {
                sw.b.insert("wasInRange".into(), in_range);
            }
        }
    }
}
