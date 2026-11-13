//! The rule-follower (`docs/design/ENTITIES.md` design 7): what a player who knows
//! every creature's counter does about the creatures it can perceive, and
//! how it moves (the keys, as a player presses them). Shared by
//! `tests/beatable.rs` (level by level) and `tests/playthrough.rs` (the whole
//! run, live).
#![allow(dead_code)]
use game::entities::brain::Mode;
use game::Game;

pub fn press(g: &mut Game, code: &str, down: bool) {
    if down {
        g.keys.insert(code.to_string());
    } else {
        g.keys.remove(code);
    }
}

/// How you would move this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gait {
    Still,
    Crouch,
    Walk,
    Run,
}

/// What the rule-follower makes of the creatures it can perceive: how to
/// move, where to look (None: along the way), the lamp, and a sheet to drop.
pub struct Counter {
    pub gait: Gait,
    pub face: Option<(f64, f64)>,
    pub away_from: Option<(f64, f64)>,
    pub lamp: Option<bool>,
    pub paper: bool,
    pub why: &'static str,
}

pub fn counter(g: &Game, last_drop: f64, t: f64) -> Counter {
    let pp = g.player.pos;
    let mut c = Counter { gait: Gait::Walk, face: None, away_from: None, lamp: None, paper: false, why: "on its way" };
    for (i, e) in g.entities.iter().enumerate() {
        let Some(cr) = g.creature(i) else { continue };
        let m = cr.brain.mode;
        let p = e.pos();
        let d = ((p.x - pp.x).powi(2) + (p.z - pp.z).powi(2)).sqrt();
        // the one behind the glass: never stare out of its window (it is
        // far out on the track while its glass is beside you); in the room,
        // keep out of its light
        if e.kind == "glass" {
            if game::entities::glass::outside(g, i) {
                if game::entities::glass::eyes_on_glass(g) && cr.get("stare") > 1.0 {
                    if let Some(b) = g.behind_glass.as_ref() {
                        let n = g.windows[b.win].normal;
                        c.face = Some((pp.x + n.x * 3.0, pp.z + n.z * 3.0));
                        c.why = "a window: eyes off it";
                    }
                }
            } else if d < 10.0 {
                c.away_from = Some((p.x, p.z));
                let lit = g.behind_glass.as_ref().is_some_and(|b| g.windows[b.win].lit_at(pp.x, pp.z));
                if lit {
                    c.gait = Gait::Run;
                }
                c.why = "the one behind the glass: out of its light";
            }
            continue;
        }
        if d > 30.0 {
            continue;
        }
        let line = g.los(pp.x, pp.z, p.x, p.z);
        let after = matches!(m, Mode::Alert | Mode::Hunt | Mode::Strike);
        match e.kind.as_str() {
            // lamp out once it stirs; still in the dark while it is after
            // you; never run near it
            "smiler" if (after || m == Mode::Search) && d < 18.0 || d < 7.0 && line => {
                c.lamp = Some(false);
                if after && d < 14.0 {
                    c.gait = Gait::Crouch.min(c.gait);
                    c.why = "a smiler: dark and still";
                } else if d < 7.0 {
                    // its breath, close (it creeps on when sent): creep away
                    // from it, never on into it
                    c.gait = Gait::Crouch.min(c.gait);
                    c.away_from = Some((p.x, p.z));
                    c.why = "a smiler close: creep away";
                } else if c.gait == Gait::Run && d < 10.0 {
                    c.gait = Gait::Walk;
                }
            }
            // move when it moves, freeze when it listens; out of a charge
            "hound" if d < 16.0 => {
                let moving = (e.vx * e.vx + e.vz * e.vz).sqrt() > game::entities::hound::MOVING;
                if m == Mode::Hunt && moving && d < 8.0 {
                    c.gait = Gait::Run;
                    c.away_from = Some((p.x, p.z));
                    c.why = "a hound charges: out of its way";
                } else if !moving && c.gait != Gait::Run {
                    c.gait = Gait::Still;
                    c.why = "a hound listens: still";
                }
            }
            // keep it in sight once it is about (its ring, or seen)
            "watcher" if g.path_distance(p.x, p.z, d) < game::entities::watcher::RING_REACH || (line && d < 20.0 && after) => {
                c.face = Some((p.x, p.z));
                c.lamp = Some(true);
                c.why = "a watcher: kept in sight";
            }
            // never look at its face: look away when the hum stops; run if it comes
            "corner" if d < 20.0 && line && (m == Mode::Alert || m == Mode::Hunt) => {
                c.face = Some((2.0 * pp.x - p.x, 2.0 * pp.z - p.z));
                c.lamp = Some(false);
                if m == Mode::Hunt {
                    c.gait = Gait::Run;
                    c.away_from = Some((p.x, p.z));
                }
                c.why = "the one in the corner: eyes away";
            }
            // give it paper when it comes
            "reader" if after && d < 12.0 => {
                if g.player.papers > 0 && t - last_drop > 2.5 {
                    c.paper = true;
                    c.why = "a reader: a sheet behind you";
                } else if g.player.papers == 0 && m == Mode::Hunt && d < 8.0 {
                    c.gait = Gait::Run;
                    c.away_from = Some((p.x, p.z));
                    c.why = "a reader and no paper: run";
                }
            }
            _ => {}
        }
    }
    c
}

/// Move toward `to` while looking at `face` (W/A/S/D as a player does).
pub fn drive(g: &mut Game, to: Option<(f64, f64)>, face: Option<(f64, f64)>, gait: Gait) {
    let pp = g.player.pos;
    if let Some((fx, fz)) = face {
        g.player.yaw = (-(fx - pp.x)).atan2(-(fz - pp.z));
    }
    let (mut w, mut s, mut a, mut dkey) = (false, false, false, false);
    if let (Some((tx, tz)), true) = (to, gait != Gait::Still) {
        let (ux, uz) = (tx - pp.x, tz - pp.z);
        let l = (ux * ux + uz * uz).sqrt();
        if l > 1e-6 {
            let (ux, uz) = (ux / l, uz / l);
            if face.is_none() {
                g.player.yaw = (-ux).atan2(-uz);
            }
            let yaw = g.player.yaw;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let (rx, rz) = (yaw.cos(), -yaw.sin());
            let (mz, mx) = (ux * fx + uz * fz, ux * rx + uz * rz);
            w = mz > 0.38;
            s = mz < -0.38;
            dkey = mx > 0.38;
            a = mx < -0.38;
        }
    }
    press(g, "KeyW", w);
    press(g, "KeyS", s);
    press(g, "KeyA", a);
    press(g, "KeyD", dkey);
    press(g, "ShiftLeft", gait == Gait::Run && g.player.stamina > 15.0);
    press(g, "ControlLeft", gait == Gait::Crouch);
}
