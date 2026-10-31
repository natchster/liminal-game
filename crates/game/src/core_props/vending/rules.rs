//! What the vending machine does (`docs/VENDING.md`, the rules). Each frame
//! (`vending_frame`) it keeps its rhythm and goes dark when something comes
//! (rules 1, 2), reads where you look (rule 3), runs what it is doing (the
//! coin ringing down, a coil turning, a shove, a good falling, the flap, its
//! fall), restocks in its dark (rule 8), lifts its flap from inside the
//! dead ones (rule 9), and keeps its fine detail for when you are near. Its
//! uses are interactables at its face, one verb at a time
//! (`vending_register`).
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
/// The silence a coin buys (rule 5): how long, and how near it you walk
/// unheard (the room hushed round it, fading out to there)
pub const SILENCE_T: f64 = 40.0;
pub const SILENCE_REACH: f64 = 10.0;
/// What a snack does for your nerve
pub const SNACK_NERVE: f64 = 6.0;
/// Rule 2: how near something that is after you, or sent for you, puts it
/// out, and how long after it has gone it comes back, m and s
pub const DARK_REACH: f64 = 16.0;
pub const DARK_HOLD: f64 = 3.0;
/// Rule 7: the shoves, with nothing hanging, of the creak and of the fall;
/// the fall's time, s; what it leaves of your nerve
pub const CREAK_ON: u32 = 2;
pub const FALL_ON: u32 = 3;
pub const TOPPLE_T: f64 = 0.9;
pub const TOPPLE_NERVE: f64 = 22.0;
/// Rule 9: how near, and how long, you stand before a dead one's glass
/// before its flap lifts, m and s; and the lift (up, held, let fall), s
pub const INSIDE_REACH: f64 = 2.0;
pub const INSIDE_WAIT: f64 = 1.5;
pub const LIFT_T: f64 = 2.6;
/// Its sounds are made only within this of you, m (its fall anywhere)
pub const HEARD_WITHIN: f64 = 30.0;
/// How much of its lit colour the inside keeps when it is dark (a dead
/// one's inside is built at 0.30 of a lit one's 0.82)
pub const DARK_INSIDE: f64 = 0.30 / 0.82;

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
    /// rule 1: its rhythm (on, dark, where in it it began), fixed when it
    /// was built; whether it is lit now
    pub rhythm: (f64, f64, f64),
    pub lit: bool,
    /// its first frame has been (it began as it is)
    pub started: bool,
    /// rule 2: dark until then (something came); dark now for that (it
    /// went out, and comes back, without a sound)
    pub dark_until: f64,
    pub hushed_dark: bool,
    /// rule 7: shoves while nothing hung; it creaked; falling since; down
    pub free_shoves: u32,
    pub creaked: bool,
    pub toppling: Option<f64>,
    pub down: bool,
    /// rule 8: how many times it restocked (in its dark)
    pub restocks: u32,
    /// rule 9 (a dead one): since when you have stood close before it, and
    /// since when its flap was lifted from inside (once)
    pub close_since: Option<f64>,
    pub lifted: Option<f64>,
    /// a coil turning by itself (`vending_phantom`): which, from when
    pub spin: Option<(usize, f64)>,
    /// rule 9: a dead one with something in it (half of them, from its seed)
    pub inhabited: bool,
    /// the last sound it made, and when (what was heard of it)
    pub last_sound: Option<(Sfx, f64)>,
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
        if s.down || s.toppling.is_some() {
            return None;
        }
        if s.vend.is_some() || s.spin.is_some() || s.shove.is_some() || s.falling.is_some() || s.coin_at.is_some() {
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
                self.vending_sound(i, Sfx::Motor);
            }
            Verb::Rock => {
                let m = &mut self.vending[i];
                m.state.shoves += 1;
                if m.state.hanging.is_none() {
                    m.state.free_shoves += 1;
                }
                let n = m.state.shoves.max(m.state.free_shoves) as f64;
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
                        let t = crate::base::rng::with_native(|| self.pick_text("snackEat"));
                        self.show_msg(&t, 2.4);
                    }
                }
            }
        }
    }

    /// Machine `i`, each frame
    pub fn vending_frame(&mut self, i: usize, dt: f64) {
        let now = self.perf_t;
        self.vending_lights(i, now);
        self.vending_hum(i, dt, now);
        self.vending_topple(i, now);
        self.vending_inside(i, now);
        self.vending_gaze(i);
        // the coin rings down into the cup
        // (the coin you wear: no coin return rings for you, `secrets::keeps`)
        if let Some(t) = self.vending[i].state.coin_at.filter(|&t| now >= t && !self.body.no_coin_return) {
            let _ = t;
            self.vending[i].state.coin_at = None;
            self.player.coins += 1;
            self.vending_sound(i, Sfx::CoinRing);
            self.vending_noise(i, RING_REACH, 0.6);
            self.show_msg("A COIN IN THE CUP", 1.6);
        }
        if self.body.no_coin_return && self.vending[i].state.coin_at.is_some_and(|t| now >= t) {
            self.vending[i].state.coin_at = None;
        }
        self.vending_turn(i, now);
        self.vending_spin(i, now);
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
        // rule 7: with nothing hanging, the second shove creaks, the third
        // is past its balance
        let free = self.vending[i].state.free_shoves;
        if free >= FALL_ON {
            self.vending[i].state.toppling = Some(now);
            return;
        }
        if free >= CREAK_ON && !self.vending[i].state.creaked {
            self.vending[i].state.creaked = true;
            self.vending_sound(i, Sfx::Creak);
        }
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
        let (text, on) = if !m.powered || !m.state.lit || m.state.down {
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
    /// How much a silent machine (rule 5) hushes the room round it at
    /// (x, z): all of it within half its reach, nothing past its reach
    pub fn vending_hush(&self, x: f64, z: f64) -> f64 {
        let now = self.perf_t;
        self.vending
            .iter()
            .filter(|m| m.state.silent_until > now)
            .map(|m| {
                let p = m.pos();
                ((SILENCE_REACH - hypot2(p.x - x, p.z - z)) / (SILENCE_REACH * 0.5)).clamp(0.0, 1.0)
            })
            .fold(0.0, f64::max)
    }

    /// Whether a body of radius `r` at (x, z) is inside a vending machine
    /// (they are solid, as the walls are)
    pub fn hits_vending(&self, x: f64, z: f64, r: f64) -> bool {
        self.vending.iter().any(|m| {
            let p = m.local(&Vec3::new(x, 0.0, z));
            // standing, its footprint; fallen, it lies on its face before it
            let (z0, z1) = if m.state.down { (FACE, FACE + H) } else { (-D / 2.0, D / 2.0) };
            let (cx, cz) = (p.x.clamp(-W / 2.0, W / 2.0), p.z.clamp(z0, z1));
            (p.x - cx).powi(2) + (p.z - cz).powi(2) < r * r
        })
    }

    /// The light the powered machines give the light map: (tile, at least)
    pub fn vending_light(&self) -> Vec<(i32, i32, f64)> {
        self.vending
            .iter()
            .filter(|m| m.powered && m.state.lit && !m.state.down)
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
    Click,
    Thump,
    Motor,
    Creak,
    Crash,
    Lift,
}

impl Game {
    /// Machine `i` makes `what`, from where it stands
    pub fn vending_sound(&mut self, i: usize, what: Sfx) {
        let p = self.vending[i].world(0.25, 1.1, D / 2.0);
        // past where it could be heard it makes nothing (a sound played
        // draws from the frame stream: a far machine's click had moved
        // every later ambient event of a floor); its fall is heard anywhere
        let pp = self.player.pos;
        if what != Sfx::Crash && hypot2(pp.x - p.x, pp.z - p.z) > HEARD_WITHIN {
            return;
        }
        self.vending[i].state.last_sound = Some((what, self.perf_t));
        self.play_from(p.x, p.y, p.z, 2.0, 1.0, 0.7, move |g| match what {
            Sfx::Lever => {
                g.noise_burst(0.03, 2400.0, 3.0, 0.06, "bandpass", None, 0.0, 0.0);
                g.blip(900.0, 0.02, "square", 0.02, None, 0.0, 0.0);
            }
            Sfx::CoinRing => {
                // the mechanism letting it go, the coin's fall through its
                // chute (each tick a little later and duller), then the cup:
                // a small steel bowl rung once (its partials inharmonic, the
                // higher ones gone first), the coin rattling still in it
                g.noise_burst(0.02, 2400.0, 3.0, 0.04, "bandpass", None, 0.0, 0.0);
                for (k, d) in [0.18, 0.26, 0.31, 0.40, 0.44].iter().enumerate() {
                    g.noise_burst(0.02, 4200.0 - k as f64 * 380.0, 7.0, 0.045 - k as f64 * 0.004, "bandpass", None, *d, 0.0);
                }
                for (f, a, len) in [(2873.0, 0.045, 0.9), (4411.0, 0.03, 0.55), (6260.0, 0.02, 0.32), (8120.0, 0.012, 0.2)] {
                    g.blip(f, len, "sine", a, None, 0.52, 0.0);
                }
                for d in [0.64, 0.71, 0.76, 0.80] {
                    g.noise_burst(0.015, 5200.0, 8.0, 0.02, "bandpass", None, d, 0.0);
                }
            }
            Sfx::Motor => {
                // a geared DC motor turning the spiral once: its whine,
                // its gears' teeth, and the spiral ticking past its stop
                if let (Some(ss), Some(dest)) = (g.soundscape(), g.sfx_route.as_ref().and_then(|r| r.dest.clone())) {
                    let t = g.ac.as_ref().map(|a| a.current_time()).unwrap_or(0.0);
                    ss.note(&dest, t + 0.6, VEND_T, 92.0, 0.035, "sawtooth", Some(86.0), 0.08, 0.15);
                    ss.note(&dest, t + 0.6, VEND_T, 184.0, 0.018, "square", Some(172.0), 0.08, 0.15);
                    ss.note(&dest, t + 0.6, VEND_T, 1380.0, 0.004, "sine", Some(1290.0), 0.1, 0.2);
                }
                for k in 0..12 {
                    g.noise_burst(0.012, 3000.0, 6.0, 0.012, "bandpass", None, 0.65 + k as f64 * VEND_T / 12.0, 0.0);
                }
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
                // 300 kg of steel dropping back on its feet: the thud, the
                // cabinet's sheet ringing, the glass in its frame, the goods
                // shifting on their spirals
                g.blip(58.0, 0.35, "sine", 0.38, Some(38.0), 0.0, 0.0);
                g.noise_burst(0.28, 380.0, 0.8, 0.24, "lowpass", Some(110.0), 0.0, 0.0);
                g.blip(212.0, 0.5, "triangle", 0.03, Some(198.0), 0.01, 0.0);
                g.blip(331.0, 0.4, "triangle", 0.02, Some(319.0), 0.01, 0.0);
                g.noise_burst(0.3, 3400.0, 3.0, 0.05, "bandpass", None, 0.02, 0.0);
                g.noise_burst(0.35, 1500.0, 1.0, 0.03, "bandpass", Some(900.0), 0.08, 0.0);
            }
            Sfx::Thunk => {
                g.blip(110.0, 0.12, "sine", 0.12, Some(70.0), 0.0, 0.0);
                g.noise_burst(0.08, 700.0, 1.0, 0.06, "lowpass", None, 0.0, 0.0);
            }
            Sfx::Flap => {
                g.noise_burst(0.05, 1500.0, 2.0, 0.05, "bandpass", None, 0.0, 0.0);
                g.noise_burst(0.06, 1100.0, 2.0, 0.04, "bandpass", None, 0.45, 0.0);
            }
            Sfx::Click => {
                // the ballast's relay letting go
                g.noise_burst(0.015, 3000.0, 4.0, 0.05, "bandpass", None, 0.0, 0.0);
                g.blip(1800.0, 0.012, "square", 0.015, None, 0.0, 0.0);
            }
            Sfx::Thump => {
                // the tube striking again
                g.blip(70.0, 0.16, "sine", 0.06, Some(55.0), 0.0, 0.0);
                g.noise_burst(0.02, 2500.0, 3.0, 0.03, "bandpass", None, 0.0, 0.0);
            }
            Sfx::Creak => {
                // sheet steel and its feet, past their balance
                g.blip(180.0, 0.6, "sawtooth", 0.025, Some(130.0), 0.05, 0.0);
                g.noise_burst(0.5, 600.0, 6.0, 0.035, "bandpass", Some(420.0), 0.05, 0.0);
            }
            Sfx::Crash => {
                g.blip(48.0, 0.9, "sine", 0.6, Some(30.0), 0.0, 0.0);
                g.noise_burst(0.8, 300.0, 0.7, 0.5, "lowpass", Some(90.0), 0.0, 0.0);
                // the glass, and the goods
                g.noise_burst(0.6, 5200.0, 1.5, 0.18, "highpass", None, 0.04, 0.0);
                for k in 0..6 {
                    g.noise_burst(0.05, 2000.0 + k as f64 * 400.0, 4.0, 0.05, "bandpass", None, 0.25 + k as f64 * 0.11, 0.0);
                }
            }
            Sfx::Lift => {
                // the flap's hinge, pushed round slowly from inside (a thin
                // dry squeal), and at the end the flap let fall on its frame
                g.noise_burst(0.8, 2600.0, 9.0, 0.012, "bandpass", Some(3300.0), 0.0, 0.0);
                g.blip(1900.0, 0.6, "triangle", 0.004, Some(2250.0), 0.05, 0.0);
                g.noise_burst(0.05, 1500.0, 2.0, 0.05, "bandpass", None, LIFT_T - 0.2, 0.0);
                g.blip(150.0, 0.08, "sine", 0.05, Some(105.0), LIFT_T - 0.2, 0.0);
            }
        });
    }
}

impl Game {
    /// Rules 1 and 2: lit in its rhythm, dark while something that is after
    /// you or sent for you is near it (and a moment after); the light map,
    /// its tube, its lights and its display follow
    fn vending_lights(&mut self, i: usize, now: f64) {
        let m = &self.vending[i];
        if !m.powered {
            return;
        }
        let (on, off, phase) = m.state.rhythm;
        let in_rhythm = m.state.down || on <= 0.0 || (now + phase).rem_euclid(on + off) < on;
        let p = m.pos();
        let stalker = self.director.stalker;
        let comes = self.entities.iter().enumerate().any(|(ei, e)| {
            let after = self.creature(ei).map(|c| c.brain.mode.after_you()).unwrap_or(false) || stalker == Some(ei);
            after && hypot2(e.pos().x - p.x, e.pos().z - p.z) < DARK_REACH
        });
        if comes {
            self.vending[i].state.dark_until = now + DARK_HOLD;
        }
        let m = &self.vending[i];
        let lit = !m.state.down && in_rhythm && now >= m.state.dark_until;
        let (first, was) = (!m.state.started, m.state.lit);
        self.vending[i].state.started = true;
        if lit == was {
            return;
        }
        self.vending[i].state.lit = lit;
        // its own beat clicks out and thumps back; a dark for something
        // coming (rule 2) goes out and comes back without a sound: that is
        // the tell. As the level begins it is simply as it is, and in the
        // silence a coin bought it makes no sound either
        let s = &mut self.vending[i].state;
        let quiet = if lit { s.hushed_dark } else { now < s.dark_until };
        s.hushed_dark = !lit && now < s.dark_until;
        if !first && !quiet && now >= s.silent_until {
            self.vending_sound(i, if lit { Sfx::Thump } else { Sfx::Click });
        }
        // rule 8: what was taken from it is back when its light comes back
        if lit && !first {
            self.vending_restock(i);
        }
        self.vending_light_up(i, lit);
        self.build_light_map();
    }

    /// Machine `i`'s tube, lights and display on or off
    pub fn vending_light_up(&mut self, i: usize, lit: bool) {
        let m = &mut self.vending[i];
        m.tubes.set_color(if lit { 0xf2f6ff } else { 0x4a4c4e });
        for (k, l) in m.lights.iter().enumerate() {
            l.set_intensity(if lit { m.light_power[k] } else { 0.0 });
        }
        // inside, it goes as dim as a dead one (the room's light comes in
        // only through the glass): the colours as the look settled them,
        // kept the first time it goes dark
        if m.inside.is_empty() {
            for (s, mat) in &m.body.mats {
                if matches!(s, Slot::Coil | Slot::Tray | Slot::Liner | Slot::Goods | Slot::Bottle | Slot::Labels) {
                    m.inside.push((mat.clone(), mat.color_v(), mat.emissive_v()));
                }
            }
        }
        for (mat, c, e) in &m.inside {
            if lit {
                mat.m().color = *c;
                mat.m().emissive = *e;
            } else {
                let k = DARK_INSIDE;
                mat.m().color.set_rgb(c.r * k, c.g * k, c.b * k);
                mat.m().emissive = Color::default();
            }
        }
        m.state.shown = (String::from("~"), 0, -1);
    }

    /// Rule 7: past its balance it falls on its face, about its front edge
    fn vending_topple(&mut self, i: usize, now: f64) {
        let Some(t0) = self.vending[i].state.toppling else { return };
        let s = ((now - t0) / TOPPLE_T).min(1.0);
        // slow off its feet, then all at once
        self.vending[i].tilt.rotation().x = 0.003 + (PI / 2.0 - 0.003) * s * s * s;
        if s < 1.0 {
            return;
        }
        let m = &mut self.vending[i];
        m.state.toppling = None;
        m.state.down = true;
        m.state.lit = false;
        m.state.hanging = None;
        m.state.bin = None;
        // the glass bursts against the floor
        if let Some(o) = m.body.meshes.get(&Slot::Glass) {
            o.set_visible(false);
        }
        self.vending_light_up(i, false);
        self.vending_sound(i, Sfx::Crash);
        // heard all over the floor: whatever hears comes to where it fell
        self.vending_noise(i, 1e6, 1.0);
        self.director_note("vending-fall");
        self.vending_spill(i);
        // you, before it
        let pp = self.player.pos;
        let q = self.vending[i].local(&pp);
        if q.x.abs() < W / 2.0 + 0.35 && q.z > FACE - 0.1 && q.z < FACE + H + 0.3 {
            let nerve = TOPPLE_NERVE.min((self.player.sanity - crate::entities::creature::KNOCKED_LEAVES).max(0.0));
            self.knock_down(nerve);
            // out from under it: beside it, or past its top
            let m = &self.vending[i];
            let spots = [
                (W / 2.0 + 0.5, FACE + 0.9),
                (-(W / 2.0 + 0.5), FACE + 0.9),
                (0.0, FACE + H + 0.5),
                (q.x.signum() * (W / 2.0 + 0.5), FACE + 0.3),
            ];
            if let Some(p) = spots.iter().map(|&(x, z)| m.world(x, 0.0, z)).find(|p| !self.circle_hits_wall(p.x, p.z, 0.3)) {
                self.player.pos.x = p.x;
                self.player.pos.z = p.z;
            }
        }
        self.build_light_map();
    }

    /// Its goods across the floor as it fell: the almond water and the
    /// batteries it held lie by it to be taken (two of each at most)
    fn vending_spill(&mut self, i: usize) {
        self.vending_scatter(i);
        let d = designs();
        let m = &self.vending[i];
        let mut almond = 0;
        let mut battery = 0;
        for (k, s) in m.stock.iter().enumerate() {
            match d[s.design].kind {
                Ware::Almond => almond += s.count.min(1),
                Ware::Battery => battery += s.count.min(2),
                _ => {}
            }
            let _ = k;
        }
        let spots: Vec<Vec3> = [(-0.8, 0.6), (0.8, 0.9), (-0.75, 1.5), (0.85, 1.7)]
            .iter()
            .map(|&(x, z)| m.world(x, 0.0, FACE + z))
            .filter(|p| !self.circle_hits_wall(p.x, p.z, 0.15))
            .collect();
        let mut at = spots.into_iter();
        for _ in 0..almond.min(2) {
            let Some(p) = at.next() else { break };
            let ag = group();
            ag.position().set(p.x, 0.15, p.z);
            ag.add(&mesh(pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10)), lambert!({ color: 0x9fc4d8, emissive: 0x16323e })));
            let cap = mesh(pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8)), &self.mats.metal);
            cap.position().y = 0.18;
            ag.add(&cap);
            self.world.add(&ag);
            self.almonds.push(crate::base::records::Pickup { g: ag, taken: false, base: 0.15, phase: p.x });
        }
        for _ in 0..battery.min(2) {
            let Some(p) = at.next() else { break };
            let bg = group();
            bg.position().set(p.x, 0.11, p.z);
            bg.add(&mesh(pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10)), lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 })));
            let tip = mesh(pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8)), lambert!({ color: 0xdad3c2 }));
            tip.position().y = 0.105;
            bg.add(&tip);
            self.world.add(&bg);
            self.battery_items.push(crate::base::records::Pickup { g: bg, taken: false, base: 0.11, phase: p.z });
        }
    }

    /// Rule 8: what was taken from it is back after its dark (each coil as
    /// full as it was built), called as its light comes back
    fn vending_restock(&mut self, i: usize) {
        let m = &mut self.vending[i];
        if m.state.down || m.state.vend.is_some() || m.state.hanging.is_some() || m.state.falling.is_some() || m.body.lifted.is_some() {
            return;
        }
        let short: Vec<usize> = (0..m.stock.len()).filter(|&k| m.stock[k].count < m.full[k]).collect();
        if short.is_empty() {
            return;
        }
        m.state.restocks += 1;
        let seed = m.seed ^ m.state.restocks.wrapping_mul(0x9E37_79B9);
        for k in short {
            m.restock(k, seed);
        }
        m.body.draw(None);
        m.state.near = None;
    }

    /// Rule 9: stand close before a dead one's glass a moment and, once,
    /// if something is in it (half of them), its flap lifts inward by
    /// itself: slowly, held up on the dark of its bin,
    /// let fall; while it is held a hand print comes up slowly on the inside
    /// of its glass, and stays
    fn vending_inside(&mut self, i: usize, now: f64) {
        let m = &self.vending[i];
        if m.powered || m.state.down || !m.state.inhabited {
            return;
        }
        if let Some(t0) = m.state.lifted {
            let s = ((now - t0) / LIFT_T).min(1.0);
            // up over its first third, held, let fall at the end
            let a = if s < 0.35 {
                let u = s / 0.35;
                u * u * (3.0 - 2.0 * u)
            } else if s < 0.92 {
                1.0
            } else {
                (1.0 - s) / 0.08
            };
            let m = &mut self.vending[i];
            if m.state.flap_t.is_none() {
                m.flap.rotation().x = -0.5 * a;
            }
            if let Some(h) = &m.print {
                h.set_visible(true);
                h.mat().m().opacity = ((s - 0.2) / 0.6).clamp(0.0, 1.0);
            }
            return;
        }
        // before its glass, with nothing between (not from behind its
        // wall); whichever way you face
        let p = m.world(0.0, 0.9, D / 2.0);
        let eye = self.player.pos;
        let before = m.local(&eye).z > D / 2.0;
        let close = before && hypot2(p.x - eye.x, p.z - eye.z) < INSIDE_REACH && self.los(eye.x, eye.z, p.x, p.z);
        let s = &mut self.vending[i].state;
        match (close, s.close_since) {
            (false, _) => s.close_since = None,
            (true, None) => s.close_since = Some(now),
            (true, Some(t)) if now - t >= INSIDE_WAIT => {
                s.lifted = Some(now);
                self.vending_sound(i, Sfx::Lift);
            }
            _ => {}
        }
    }
}

impl Game {
    /// Bags and bars thrown out across the floor round it as it fell (to
    /// look at: what can be taken lies among them, `vending_spill`); their
    /// places from its own seed
    fn vending_scatter(&mut self, i: usize) {
        let m = &self.vending[i];
        let d = designs();
        let stock: Vec<usize> = m
            .stock
            .iter()
            .filter(|s| s.count > 0 && matches!(d[s.design].kind, Ware::Bag | Ware::Bar | Ware::Pastry))
            .map(|s| s.design)
            .collect();
        if stock.is_empty() {
            return;
        }
        let mat = m.body.mats[&Slot::Goods].clone();
        let seed = m.seed ^ 0xFA11;
        let at = m.world(0.0, 0.0, 0.0);
        let yaw = m.yaw();
        let pieces: Vec<(Geo, Mat4)> = crate::base::rng::with_own_stream(seed, || {
            (0..10)
                .map(|n| {
                    let de = d[stock[n % stock.len()]];
                    let geo = match de.kind {
                        Ware::Bar => super::goods::pouch(0.058, 0.135, 0.02, 0.05, rand1(0.4), true, rand1(6.0), true),
                        Ware::Pastry => super::goods::pouch(0.125, 0.125, 0.03, 0.06, rand1(0.6), false, rand1(6.0), true),
                        _ => super::goods::pouch(0.15, 0.205, rand2(0.03, 0.045), 0.055, rand1(0.9), false, rand1(6.0), true),
                    };
                    // UVs into its cell
                    let c = super::paint::cell_uv(de.cell);
                    {
                        let mut gg = geo.g();
                        for k in 0..gg.uv.len() / 2 {
                            gg.uv[k * 2] = (c[0] + gg.uv[k * 2] as f64 * c[2]) as f32;
                            gg.uv[k * 2 + 1] = (c[1] + gg.uv[k * 2 + 1] as f64 * c[3]) as f32;
                        }
                    }
                    // flat on the floor round it, most toward its sides and its top
                    let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
                    let (x, z) = if rnd() < 0.7 {
                        (side * rand2(0.55, 1.1), FACE + rand2(0.1, H))
                    } else {
                        (rand2(-0.6, 0.6), FACE + H + rand2(0.1, 0.6))
                    };
                    let q = Quat::from_euler(&Euler::new(-PI / 2.0, rand1(PI * 2.0), 0.0));
                    let mut qy = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw);
                    qy.multiply(&q);
                    let (s, cs) = yaw.sin_cos();
                    let p = Vec3::new(at.x + x * cs + z * s, 0.012, at.z - x * s + z * cs);
                    (geo, Mat4::compose(&p, &qy, &Vec3::new(1.0, 1.0, 1.0)))
                })
                .collect()
        });
        for (geo, mx) in pieces {
            let (p, q, _) = mx.decompose();
            if self.circle_hits_wall(p.x, p.z, 0.05) {
                continue;
            }
            let o = mesh(geo, &mat);
            o.set_pos_v(&p);
            o.set_quat(&q);
            self.world.add(&o);
        }
    }
}

impl Game {
    /// Its hum, from its top behind the glass: while it is lit and not
    /// silent (rule 5), softened in the director's hush; made the first
    /// time it is heard (no soundscape, no hum)
    fn vending_hum(&mut self, i: usize, dt: f64, now: f64) {
        let m = &self.vending[i];
        if !m.powered {
            return;
        }
        let level =
            if m.state.lit && !m.state.down && now >= m.state.silent_until { 1.0 - 0.75 * self.amb_hush.clamp(0.0, 1.0) } else { 0.0 };
        if m.hum.is_none() {
            if level <= 0.0 {
                return;
            }
            let Some(ss) = self.soundscape() else { return };
            self.vending[i].hum = Some(ss.machine_hum());
        }
        let m = &self.vending[i];
        let p = m.world(CAB[0], 1.55, D / 2.0 - 0.1);
        if let Some(h) = &m.hum {
            h.update(dt, p.x, p.y, p.z, level);
        }
    }

    /// Now and then, near a lit machine, one of its coils turns by itself
    /// (the ambience's `vending-hum`, rule 8's other side: something else
    /// uses it): the motor's whine from it, and the spiral turning, if it
    /// is lit and doing nothing else; nothing comes forward
    pub fn vending_phantom(&mut self, near: Vec3) {
        let Some(i) = (0..self.vending.len()).filter(|&i| self.vending[i].powered).min_by(|&a, &b| {
            let (pa, pb) = (self.vending[a].pos(), self.vending[b].pos());
            hypot2(pa.x - near.x, pa.z - near.z).total_cmp(&hypot2(pb.x - near.x, pb.z - near.z))
        }) else {
            return;
        };
        let m = &self.vending[i];
        let s = &m.state;
        if !s.lit || s.down || s.vend.is_some() || s.toppling.is_some() || s.falling.is_some() || m.body.lifted.is_some() {
            return;
        }
        // (not in the silence a coin bought: rule 5)
        if self.perf_t < s.silent_until {
            return;
        }
        let n = m.places.len();
        let k = (m.seed as usize).wrapping_add(m.state.restocks as usize * 7 + (self.perf_t * 10.0) as usize) % n;
        let now = self.perf_t;
        self.vending_sound(i, Sfx::Motor);
        let m = &mut self.vending[i];
        m.body.lift(k);
        m.state.near = None;
        m.state.spin = Some((k, now + 0.6));
    }

    /// A coil turning by itself (`vending_phantom`): it spins once, its
    /// goods stay where they are
    fn vending_spin(&mut self, i: usize, now: f64) {
        let Some((k, t0)) = self.vending[i].state.spin else { return };
        let _ = k;
        if now < t0 {
            return;
        }
        let s = ((now - t0) / VEND_T).min(1.0);
        let m = &mut self.vending[i];
        let Some((_, lifted)) = m.body.lifted.clone() else { return };
        let ease = s * s * (3.0 - 2.0 * s);
        for (pi, o) in &lifted {
            let p = &m.body.pieces[*pi];
            if p.slot == Slot::Coil {
                let (_, q, _) = p.m.decompose();
                let spin = Quat::from_axis_angle(&Vec3::new(0.0, 0.0, 1.0), -PI * 2.0 * ease);
                let mut q2 = q;
                q2.multiply(&spin);
                o.set_quat(&q2);
            }
        }
        if s >= 1.0 {
            m.state.spin = None;
            m.body.settle();
            m.state.near = None;
        }
    }
}
