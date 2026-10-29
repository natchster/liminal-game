//! What the vending machine does (`docs/VENDING.md`, the rules). Each frame
//! (`vending_frame`) it reads where you look (rule 3), runs what it is doing
//! (the coin ringing down, a coil turning, a shove, a good falling, the
//! flap), and keeps its fine detail for when you are near. Its uses are
//! interactables at its face, one verb at a time (`vending_register`).
//!
//! Everything here runs as you play: no draw from the build's stream. What
//! varies per machine was fixed when it was built (`State::drop_on`).
use super::model::*;
use super::paint::{designs, paint_display, Ware, DISPLAY_CHARS};
use crate::prelude::*;

/// How lit a powered machine's tile is at least, in the light map (its
/// glass and the glow on the floor in front of it)
pub const VEND_LIGHT: f64 = 0.5;
/// Its coils, key legends and price strips are not drawn beyond this
/// (a 5 mm wire is under a pixel there)
pub const DETAIL_FAR: f64 = 8.0;
/// How near your eyes must be for its display to follow them (rule 3)
pub const GAZE_REACH: f64 = 2.5;
/// The coin, from the press to its ring in the cup, s
pub const RETURN_T: f64 = 1.4;
/// A coil's turn, s (a geared motor at 20 rpm turns once in 3 s)
pub const VEND_T: f64 = 2.6;
/// A shove: forward and back against its feet, s
pub const SHOVE_T: f64 = 0.7;
/// What falls takes this long to the bin, s
pub const FALL_T: f64 = 0.35;
/// How far along the way what hears hears a coin ring down (rule 4), a
/// shove (rule 6), m
pub const RING_REACH: f64 = 10.0;
pub const BANG_REACH: f64 = 20.0;
/// The silence a coin buys (rule 5): how long, and how near it you walk unheard
pub const SILENCE_T: f64 = 40.0;
pub const SILENCE_REACH: f64 = 10.0;
/// What a snack does for your nerve
pub const SNACK_NERVE: f64 = 6.0;

/// What a machine is doing
#[derive(Clone, Debug, Default)]
pub struct State {
    /// its fine detail is drawn (within `DETAIL_FAR`); None: not yet set
    /// for the meshes it has now (a new merge draws everything)
    pub near: Option<bool>,
    /// the coil your eyes are on (rule 3)
    pub gaze: Option<usize>,
    /// it has given its coin (rule 4), and when the coin rings down
    pub coin_given: bool,
    pub coin_at: Option<f64>,
    /// a coil turning: which, since when
    pub vend: Option<(usize, f64)>,
    /// the coil whose front good hangs at the glass
    pub hanging: Option<usize>,
    /// shoves since the last thing that fell, and how many it takes to
    /// shake what hangs loose (fixed when it was built: 2 to 4)
    pub shoves: u32,
    pub drop_on: u32,
    /// a shove under way: since when, how far it leans
    pub shove: Option<(f64, f64)>,
    /// a good falling to the bin: its pieces' meshes, since when
    pub falling: Option<(Vec<Obj>, f64)>,
    /// what is in the bin (a design of the goods' atlas)
    pub bin: Option<usize>,
    /// the flap pushed in, since when
    pub flap_t: Option<f64>,
    /// until when it, and the place, are silent (rule 5)
    pub silent_until: f64,
    /// what the display says, and since when (it runs across)
    pub says: String,
    pub says_t: f64,
    pub shown: (String, usize, i32),
}

/// One of a machine's verbs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    /// press the coin return with nothing in it (rule 4)
    CoinReturn,
    /// a coin in, for the coil your eyes are on (rule 5)
    InsertCoin,
    /// shove it (rules 6 and 7)
    Rock,
    /// push the flap and take what is in the bin
    ReachIn,
}

impl Game {
    /// The verb machine `i` offers now, if any (one at a time)
    pub fn vending_verb(&self, i: usize) -> Option<Verb> {
        let m = self.vending.get(i)?;
        let s = &m.state;
        if s.vend.is_some() || s.shove.is_some() || s.falling.is_some() || s.coin_at.is_some() {
            return None;
        }
        if s.bin.is_some() {
            return Some(Verb::ReachIn);
        }
        if s.hanging.is_some() || !m.powered {
            return Some(Verb::Rock);
        }
        if self.player.coins > 0 {
            return s.gaze.filter(|&k| m.stock[k].count > 0).map(|_| Verb::InsertCoin);
        }
        Some(if s.coin_given { Verb::Rock } else { Verb::CoinReturn })
    }

    /// Registers machine `i`'s uses and its animator, once it stands where it is
    pub fn vending_register(&mut self, i: usize) {
        for (verb, prompt) in [
            (Verb::CoinReturn, "[E] COIN RETURN"),
            (Verb::InsertCoin, "[E] INSERT COIN"),
            (Verb::Rock, "[E] ROCK IT"),
            (Verb::ReachIn, "[E] REACH IN"),
        ] {
            // the use point: before its face, at the eyes (moved there each frame)
            self.interact(Vec3::default(), 1.5, prompt, move |g| g.vending_verb(i) == Some(verb), move |g| g.vending_use(i, verb));
        }
        let first = self.interactables.len() - 4;
        self.animate(move |g, dt, _t| {
            let at = g.vending.get(i).map(|m| m.world(0.0, EYE, D / 2.0 + 0.55));
            if let Some(p) = at {
                for k in first..first + 4 {
                    if let Some(it) = g.interactables.get_mut(k) {
                        it.pos = p;
                    }
                }
            }
            g.vending_frame(i, dt);
        });
    }

    /// Machine `i` does `verb`
    pub fn vending_use(&mut self, i: usize, verb: Verb) {
        let now = self.perf_t;
        match verb {
            Verb::CoinReturn => {
                let m = &mut self.vending[i];
                m.state.coin_given = true;
                m.state.coin_at = Some(now + RETURN_T);
                self.vending_sound(i, Sfx::Lever);
            }
            Verb::InsertCoin => {
                let Some(k) = self.vending[i].state.gaze else { return };
                self.player.coins -= 1;
                self.vending_sound(i, Sfx::CoinIn);
                let m = &mut self.vending[i];
                m.state.says = format!("CREDIT .25  {}", m.places[k].code);
                m.state.says_t = now;
                m.state.vend = Some((k, now + 0.6));
                m.body.lift(k);
                m.state.near = None;
            }
            Verb::Rock => {
                let m = &mut self.vending[i];
                m.state.shoves += 1;
                let n = m.state.shoves as f64;
                m.state.shove = Some((now, (0.02 + 0.025 * n).min(0.09)));
            }
            Verb::ReachIn => {
                let m = &mut self.vending[i];
                let Some(d) = m.state.bin.take() else { return };
                m.state.flap_t = Some(now);
                self.vending_sound(i, Sfx::Flap);
                let de = designs()[d];
                match de.kind {
                    Ware::Almond => {
                        self.player.bottles += 1;
                        self.show_msg("ALMOND WATER", 1.6);
                    }
                    Ware::Battery => {
                        self.player.batteries += 2;
                        self.show_msg("TWO BATTERIES", 1.6);
                    }
                    _ => {
                        self.player.sanity = (self.player.sanity + SNACK_NERVE).min(SAN_MAX);
                        self.show_msg(&format!("{}. IT TASTES OF NOTHING.", de.name), 2.4);
                    }
                }
            }
        }
    }

    /// Machine `i`, each frame
    pub fn vending_frame(&mut self, i: usize, dt: f64) {
        let now = self.perf_t;
        let _ = dt;
        self.vending_gaze(i);
        // the coin rings down into the cup
        if let Some(t) = self.vending[i].state.coin_at.filter(|&t| now >= t) {
            let _ = t;
            self.vending[i].state.coin_at = None;
            self.player.coins += 1;
            self.vending_sound(i, Sfx::CoinRing);
            self.vending_noise(i, RING_REACH, 0.6);
            self.show_msg("A COIN IN THE CUP", 1.6);
        }
        self.vending_turn(i, now);
        self.vending_shove(i, now);
        self.vending_fall(i, now);
        self.vending_flap(i, now);
        self.vending_display(i, now);
        // the fine detail, near
        let cp = self.camera.pos();
        let m = &mut self.vending[i];
        let p = m.pos();
        let near = hypot2(cp.x - p.x, cp.z - p.z) < DETAIL_FAR;
        if m.state.near != Some(near) {
            m.state.near = Some(near);
            for (s, o) in &m.body.meshes {
                if matches!(s, Slot::Coil | Slot::Legend | Slot::Keys | Slot::Labels) {
                    o.set_visible(near);
                }
            }
        }
    }

    /// Rule 3: the coil under your eyes, through the glass, within reach
    fn vending_gaze(&mut self, i: usize) {
        let m = &self.vending[i];
        if !m.powered {
            self.vending[i].state.gaze = None;
            return;
        }
        let eye = self.camera.pos();
        let dir = self.camera.world_direction();
        let o = m.local(&eye);
        let mut ahead = eye;
        ahead.add(&dir);
        let mut d = m.local(&ahead);
        d.sub(&o);
        let mut gaze = None;
        if d.z < -1e-6 {
            let t = (GLASS_Z - o.z) / d.z;
            let (x, y) = (o.x + d.x * t, o.y + d.y * t);
            if t > 0.0 && t < GAZE_REACH && x > WIN[0] && x < WIN[1] && y > WIN[2] && y < WIN[3] {
                // the tray whose shelf is under the point, then the nearest coil on it
                if let Some(tray) = (0..TRAY_Y.len()).find(|&t| y >= TRAY_Y[t] && (t == 0 || y < TRAY_Y[t - 1])) {
                    gaze = m
                        .places
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.tray == tray)
                        .min_by(|a, b| (a.1.x - x).abs().total_cmp(&(b.1.x - x).abs()))
                        .map(|(k, _)| k);
                }
            }
        }
        self.vending[i].state.gaze = gaze;
    }

    /// A coil turning once (rule 5): the coil spins, its goods move forward a
    /// place, and the front one comes off its end and catches against the
    /// glass, hanging; nothing falls
    fn vending_turn(&mut self, i: usize, now: f64) {
        let Some((k, t0)) = self.vending[i].state.vend else { return };
        if now < t0 {
            return;
        }
        let s = ((now - t0) / VEND_T).min(1.0);
        if s > 0.0 && self.vending[i].state.says_t < t0 {
            self.vending[i].state.says_t = t0;
        }
        let m = &mut self.vending[i];
        let place = m.places[k].clone();
        let Some((_, lifted)) = m.body.lifted.clone() else { return };
        let ease = s * s * (3.0 - 2.0 * s);
        let pitch = place.pitch();
        let end = COIL_Z + COIL_LEN / 2.0;
        for (pi, o) in &lifted {
            let p = &m.body.pieces[*pi];
            let (pos, q, _) = p.m.decompose();
            if p.slot == Slot::Coil {
                let spin = Quat::from_axis_angle(&Vec3::new(0.0, 0.0, 1.0), -PI * 2.0 * ease);
                let mut q2 = q;
                q2.multiply(&spin);
                o.set_quat(&q2);
                continue;
            }
            let front = m.stock_front(k, *pi);
            let mut np = pos;
            np.z += pitch * ease;
            o.set_pos_v(&np);
            if front {
                // past the coil's end it tips forward until its top meets the glass
                let over = (np.z - end + 0.02).max(0.0);
                let h = 0.2;
                let lean = (over / h).min(((GLASS_Z - 0.006 - end) / h).clamp(0.0, 1.0)).asin();
                let tip = Quat::from_axis_angle(&Vec3::new(1.0, 0.0, 0.0), lean);
                let mut q2 = tip;
                q2.multiply(&q);
                o.set_quat(&q2);
            }
        }
        if s < 0.25 {
            return;
        }
        if m.state.says.starts_with("CREDIT") && s > 0.25 {
            m.state.says = String::new();
        }
        if s >= 1.0 {
            m.state.vend = None;
            m.state.hanging = Some(k);
            m.state.shoves = 0;
            m.body.settle();
            m.state.near = None;
            m.state.says = "MAKE ANOTHER SELECTION".into();
            m.state.says_t = now;
            m.state.silent_until = now + SILENCE_T;
            self.vending_sound(i, Sfx::Catch);
        }
    }

    /// A shove (rules 6, 7): it leans toward you on its front feet and
    /// drops back against the wall with a bang; on its count what hangs
    /// shakes loose
    fn vending_shove(&mut self, i: usize, now: f64) {
        let Some((t0, amp)) = self.vending[i].state.shove else { return };
        let s = ((now - t0) / SHOVE_T).min(1.0);
        // up over the first half, back down hard
        let a = if s < 0.55 { amp * (s / 0.55 * PI / 2.0).sin() } else { amp * (1.0 - ((s - 0.55) / 0.45).powi(2)).max(0.0) };
        self.vending[i].tilt.rotation().x = 0.003 + a;
        if s < 1.0 {
            return;
        }
        self.vending[i].tilt.rotation().x = 0.003;
        self.vending[i].state.shove = None;
        self.vending_sound(i, Sfx::Bang);
        self.vending_noise(i, BANG_REACH, 0.8);
        let m = &self.vending[i];
        if let Some(k) = m.state.hanging.filter(|_| m.state.shoves >= m.state.drop_on) {
            self.vending_drop(i, k, now);
        }
    }

    /// What hangs from coil `k` falls: its pieces leave the merge and drop
    /// down the front of the cabinet into the bin
    fn vending_drop(&mut self, i: usize, k: usize, now: f64) {
        let m = &mut self.vending[i];
        m.state.hanging = None;
        m.state.shoves = 0;
        let item = m.front_item(k);
        let Some(item) = item else { return };
        let mut objs = vec![];
        let mut keep = vec![];
        for p in std::mem::take(&mut m.body.pieces) {
            if p.item == Some(item) {
                let o = mesh(merge(std::iter::once(&Piece { m: Mat4::IDENTITY, ..p.clone() })), &m.body.mats[&p.slot]);
                let (pos, q, sc) = p.m.decompose();
                o.set_pos_v(&pos);
                o.set_quat(&q);
                o.set_scale(sc.x, sc.y, sc.z);
                m.body.root.add(&o);
                objs.push(o);
            } else {
                keep.push(p);
            }
        }
        m.body.pieces = keep;
        m.stock[k].count = m.stock[k].count.saturating_sub(1);
        m.items[k].remove(0);
        m.body.draw(None);
        m.state.near = None;
        m.state.bin = Some(m.stock[k].design);
        m.state.falling = Some((objs, now));
    }

    /// The good falling to the bin
    fn vending_fall(&mut self, i: usize, now: f64) {
        let Some((objs, t0)) = self.vending[i].state.falling.clone() else { return };
        let s = ((now - t0) / FALL_T).min(1.0);
        for o in &objs {
            let mut p = o.pos();
            p.y -= (s * s) * 0.08;
            p.z += 0.004;
            o.set_pos_v(&p);
        }
        if s < 1.0 {
            return;
        }
        let m = &mut self.vending[i];
        for o in &objs {
            m.body.root.remove(o);
        }
        m.state.falling = None;
        self.vending_sound(i, Sfx::Thunk);
    }

    /// The flap, pushed in and swinging back
    fn vending_flap(&mut self, i: usize, now: f64) {
        let m = &mut self.vending[i];
        let Some(t0) = m.state.flap_t else { return };
        let s = (now - t0) / 0.9;
        let a = if s < 0.3 { s / 0.3 } else { (1.0 - (s - 0.3) / 0.7).max(0.0) };
        m.flap.rotation().x = -0.9 * a;
        if s >= 1.0 {
            m.flap.rotation().x = 0.0;
            m.state.flap_t = None;
        }
    }

    /// What its display says: the coil under your eyes, its price; or what
    /// it is saying (it runs across); or HAVE A NICE DAY, running across
    fn vending_display(&mut self, i: usize, now: f64) {
        let m = &mut self.vending[i];
        let (text, on) = if !m.powered {
            (String::new(), 0.0)
        } else if !m.state.says.is_empty() {
            (m.state.says.clone(), 1.0)
        } else if let Some(k) = m.state.gaze {
            let c = &m.places[k];
            if m.stock[k].count == 0 {
                (format!("{}  SOLD OUT", c.code), 1.0)
            } else {
                let price = match c.tray {
                    0..=2 => "1.25",
                    3 | 4 => "1.00",
                    _ => "1.50",
                };
                (format!("{:<6}{price}", c.code), 1.0)
            }
        } else {
            ("HAVE A NICE DAY".into(), 1.0)
        };
        // a longer line runs across, three characters a second, round and round
        let n = text.chars().count();
        let scroll = if n > DISPLAY_CHARS {
            let padded = n + 4;
            ((now - m.state.says_t).max(0.0) * 3.0) as usize % padded
        } else {
            0
        };
        let key = (text.clone(), scroll, (on * 10.0) as i32);
        if m.state.shown == key {
            return;
        }
        m.state.shown = key;
        let line = if n > DISPLAY_CHARS { format!("{text}    {text}") } else { text };
        paint_display(&mut m.display_canvas, &line, scroll, on);
        m.display_canvas.update_tex(&m.display_tex);
        m.display_tex.needs_update();
    }

    /// What hears, within `reach` along the way, hears machine `i`: it turns
    /// to it and comes to look near it (a sound leaves a rough guess, not
    /// where you are); `loud` 0..1 is how sure it makes them
    pub fn vending_noise(&mut self, i: usize, reach: f64, loud: f64) {
        let f = self.vending[i].world(0.0, 0.0, D / 2.0 + 0.8);
        let (gx, gz) = self.open_point(f.x, f.z);
        let now = self.perf_t;
        for ei in 0..self.entities.len() {
            let Some(c) = self.creature(ei) else { continue };
            if c.data.hearing <= 0.0 {
                continue;
            }
            let p = self.entities[ei].pos();
            let straight = hypot2(p.x - f.x, p.z - f.z);
            let d = self.path_distance(p.x, p.z, straight);
            if d > reach * c.data.hearing {
                continue;
            }
            if let Some(c) = self.creature_mut(ei) {
                c.brain.mem.heard = Some((gx, gz, now));
                c.brain.mem.awareness = c.brain.mem.awareness.max(loud);
            }
        }
        self.director_note("vending");
    }

    /// Whether a silent machine (rule 5) covers your steps at (x, z)
    pub fn vending_muffles(&self, x: f64, z: f64) -> bool {
        let now = self.perf_t;
        self.vending.iter().any(|m| {
            let p = m.pos();
            m.state.silent_until > now && hypot2(p.x - x, p.z - z) < SILENCE_REACH
        })
    }
    /// How much the machines' silence hushes the place (0 or 1)
    pub fn vending_hush(&self) -> f64 {
        let now = self.perf_t;
        if self.vending.iter().any(|m| m.state.silent_until > now) {
            1.0
        } else {
            0.0
        }
    }

    /// Whether a body of radius `r` at (x, z) is inside a vending machine
    /// (they are solid, as the walls are)
    pub fn hits_vending(&self, x: f64, z: f64, r: f64) -> bool {
        self.vending.iter().any(|m| {
            let p = m.local(&Vec3::new(x, 0.0, z));
            let (cx, cz) = (p.x.clamp(-W / 2.0, W / 2.0), p.z.clamp(-D / 2.0, D / 2.0));
            (p.x - cx).powi(2) + (p.z - cz).powi(2) < r * r
        })
    }

    /// The light the powered machines give the light map: (tile, at least)
    pub fn vending_light(&self) -> Vec<(i32, i32, f64)> {
        self.vending
            .iter()
            .filter(|m| m.powered)
            .map(|m| {
                let f = m.world(0.0, 0.0, D / 2.0 + 0.6);
                (self.tx(f.x), self.tz(f.z), VEND_LIGHT)
            })
            .collect()
    }

    /// The machines stand where they will stay (the level's settling is
    /// done): the places of the hums for the ambience
    pub fn vending_settled(&mut self) {
        self.pp.vending_positions = self.vending.iter().filter(|m| m.powered).map(|m| m.world(0.0, 0.9, 0.0)).collect();
    }
}

/// What a machine sounds like doing what it does (`vending_sound`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Lever,
    CoinRing,
    CoinIn,
    Catch,
    Bang,
    Thunk,
    Flap,
}

impl Game {
    /// Machine `i` makes `what`, from where it stands
    pub fn vending_sound(&mut self, i: usize, what: Sfx) {
        let p = self.vending[i].world(0.25, 1.1, D / 2.0);
        self.play_from(p.x, p.y, p.z, 2.0, 1.0, 0.7, move |g| match what {
            Sfx::Lever => {
                g.noise_burst(0.03, 2400.0, 3.0, 0.06, "bandpass", None, 0.0, 0.0);
                g.blip(900.0, 0.02, "square", 0.02, None, 0.0, 0.0);
            }
            Sfx::CoinRing => {
                // down through the mechanism, then a ring in the steel cup
                for (k, d) in [0.0, 0.07, 0.13, 0.22].iter().enumerate() {
                    g.noise_burst(0.025, 3200.0 - k as f64 * 300.0, 6.0, 0.05, "bandpass", None, *d, 0.0);
                }
                g.blip(4180.0, 0.5, "sine", 0.05, None, 0.30, 0.0);
                g.blip(6230.0, 0.35, "sine", 0.03, None, 0.30, 0.0);
                g.blip(2710.0, 0.6, "sine", 0.025, None, 0.31, 0.0);
            }
            Sfx::CoinIn => {
                g.noise_burst(0.04, 3600.0, 5.0, 0.05, "bandpass", None, 0.0, 0.0);
                g.noise_burst(0.05, 2200.0, 4.0, 0.04, "bandpass", None, 0.12, 0.0);
                g.blip(2000.0, 0.06, "square", 0.012, None, 0.35, 0.0);
            }
            Sfx::Catch => {
                g.noise_burst(0.12, 900.0, 1.2, 0.04, "bandpass", None, 0.0, 0.0);
            }
            Sfx::Bang => {
                g.blip(62.0, 0.30, "sine", 0.35, Some(40.0), 0.0, 0.0);
                g.noise_burst(0.25, 400.0, 0.8, 0.22, "lowpass", Some(120.0), 0.0, 0.0);
                g.noise_burst(0.35, 2600.0, 2.0, 0.05, "bandpass", None, 0.03, 0.0);
            }
            Sfx::Thunk => {
                g.blip(110.0, 0.12, "sine", 0.12, Some(70.0), 0.0, 0.0);
                g.noise_burst(0.08, 700.0, 1.0, 0.06, "lowpass", None, 0.0, 0.0);
            }
            Sfx::Flap => {
                g.noise_burst(0.05, 1500.0, 2.0, 0.05, "bandpass", None, 0.0, 0.0);
                g.noise_burst(0.06, 1100.0, 2.0, 0.04, "bandpass", None, 0.45, 0.0);
            }
        });
    }
}
