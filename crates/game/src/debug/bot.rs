//! Scripted players for tests and the bench.
use crate::replay::{FrameInput, Quantizer};
use engine::dmath::DMath;

/// A restless player: mostly walking, turning, now and then running,
/// crouching or switching the flashlight (a fixed pseudo-random script, its
/// look rounded as the window rounds a mouse). The tests' bot, and the
/// bench's (`--bench-replay`).
pub fn restless(frames: usize, seed: u32) -> Vec<FrameInput> {
    let mut s = u64::from(seed) | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 1000) as f64 / 1000.0
    };
    let mut out = vec![FrameInput::default(); frames];
    let mut held: Vec<&str> = vec![];
    let mut q = Quantizer::default();
    for (k, f) in out.iter_mut().enumerate() {
        if k % 30 == 0 {
            for h in held.drain(..) {
                f.up(h);
            }
            let r = next();
            let keys: &[&str] = if r < 0.55 {
                &["KeyW"]
            } else if r < 0.7 {
                &["KeyW", "ShiftLeft"]
            } else if r < 0.8 {
                &["KeyA"]
            } else if r < 0.88 {
                &["KeyS"]
            } else {
                &["KeyW", "ControlLeft"]
            };
            for key in keys {
                f.down(key);
                held.push(key);
            }
            if next() < 0.15 {
                f.down("KeyF");
                f.up("KeyF");
            }
        }
        // rounded where it enters, as the window rounds a mouse (`Quantizer`)
        let (dx, dy) = q.look((next() - 0.5) * 18.0, (next() - 0.5) * 2.0);
        f.look(dx, dy);
    }
    out
}

/// A player who looks where they go (`--record-bot`): walks on, turns
/// before a wall it sees ahead (towards the more open side), turns about
/// when it has stopped getting anywhere, and now and then runs, glances
/// aside or switches its light. It reads the world only to choose its
/// inputs, which are what a recording keeps, so its sessions replay as a
/// person's do; the same seed and floor give the same walk.
pub struct Wanderer {
    s: u64,
    q: Quantizer,
    /// a turn under way: frames left, and the turn a frame (radians)
    turn: Option<(u32, f64)>,
    walking: bool,
    /// the frame a run ends at
    running_until: Option<u32>,
    /// where it stood at the last check, to see it gets somewhere
    at: (f64, f64),
    k: u32,
    next_glance: u32,
}

impl Wanderer {
    pub fn new(seed: u64) -> Wanderer {
        Wanderer {
            s: seed | 1,
            q: Quantizer::default(),
            turn: None,
            walking: false,
            running_until: None,
            at: (0.0, 0.0),
            k: 0,
            next_glance: 240,
        }
    }
    fn rnd(&mut self) -> f64 {
        self.s ^= self.s << 13;
        self.s ^= self.s >> 7;
        self.s ^= self.s << 17;
        (self.s % 10_000) as f64 / 10_000.0
    }
    /// A turn of `by` radians (to the left as it grows), over `frames`.
    fn turn_by(&mut self, by: f64, frames: u32) {
        self.turn = Some((frames, by / f64::from(frames)));
    }
    /// The next frame's input, from where play stands now.
    pub fn next(&mut self, g: &crate::Game) -> FrameInput {
        let mut f = FrameInput::default();
        if !self.walking {
            f.down("KeyW");
            self.walking = true;
        }
        let (k, p) = (self.k, &g.player);
        self.k += 1;
        // stopped getting anywhere: about
        if k % 60 == 0 {
            let moved = (p.pos.x - self.at.0).dhypot(p.pos.z - self.at.1);
            self.at = (p.pos.x, p.pos.z);
            if k > 0 && moved < 0.8 && self.turn.is_none() {
                let side = if self.rnd() < 0.5 { 1.0 } else { -1.0 };
                let by = side * std::f64::consts::PI * (0.75 + 0.5 * self.rnd());
                self.turn_by(by, 30);
            }
        }
        if self.turn.is_none() {
            let ahead = |a: f64, d: f64| {
                let yaw = p.yaw + a;
                (p.pos.x - yaw.dsin() * d, p.pos.z - yaw.dcos() * d)
            };
            let (x, z) = ahead(0.0, 1.6);
            if g.circle_hits_wall(x, z, 0.35) {
                // a wall ahead: towards the side with more room
                let room = |a: f64| {
                    (1..=6)
                        .take_while(|i| {
                            let (x, z) = ahead(a, f64::from(*i) * 0.6);
                            !g.circle_hits_wall(x, z, 0.35)
                        })
                        .count()
                };
                let (l, r) = (room(1.0), room(-1.0));
                let side = match l.cmp(&r) {
                    std::cmp::Ordering::Greater => 1.0,
                    std::cmp::Ordering::Less => -1.0,
                    std::cmp::Ordering::Equal if self.rnd() < 0.5 => 1.0,
                    std::cmp::Ordering::Equal => -1.0,
                };
                let by = side * (0.9 + 0.8 * self.rnd());
                self.turn_by(by, 20);
            } else if k >= self.next_glance {
                // a glance aside, now and then
                let by = (self.rnd() - 0.5) * 1.6;
                self.turn_by(by, 30);
                self.next_glance = k + 240 + (self.rnd() * 360.0) as u32;
            }
        }
        // the mouse that turns it (and brings the look back to level)
        let k_look = 0.0022 * g.settings.look;
        let dx = match self.turn {
            Some((n, by)) => {
                self.turn = (n > 1).then_some((n - 1, by));
                -by / k_look
            }
            None => 0.0,
        };
        let dy = (p.pitch / k_look).clamp(-20.0, 20.0) * 0.2 * if g.settings.invert_y { -1.0 } else { 1.0 };
        let (qx, qy) = self.q.look(dx, dy);
        if qx != 0.0 || qy != 0.0 {
            f.look(qx, qy);
        }
        // a run of a few seconds, when there is breath for it
        let roll = self.rnd();
        match self.running_until {
            Some(t) if k >= t => {
                f.up("ShiftLeft");
                self.running_until = None;
            }
            None if p.stamina > 80.0 && roll < 0.004 => {
                f.down("ShiftLeft");
                self.running_until = Some(k + 60 + (self.rnd() * 120.0) as u32);
            }
            _ => {}
        }
        if self.rnd() < 0.0015 {
            f.down("KeyF");
            f.up("KeyF");
        }
        f
    }
}
