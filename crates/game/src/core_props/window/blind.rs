//! A window's blind as a verb (`docs/design/WINDOWS.md` 4, W3). `[E] LOWER THE
//! BLIND` lets it all the way down over a second, its slats rattling down
//! and shut: what hears, near, comes to look. The glass it hangs over is
//! then covered: no look counts through it, no light comes in by it, and
//! nothing comes through it (`Window::covered`). `[E] RAISE THE BLIND` draws
//! it up again. Nothing here draws from a level's stream; its rattle is
//! made only where it is heard (by you, using it).
use super::{model, view, Blind};
use crate::prelude::*;

/// A blind is let down or drawn up in this long, s
pub const BLIND_T: f64 = 1.0;
/// Its rattle carries this far along the way, m
pub const RATTLE_REACH: f64 = 10.0;
/// You reach a blind's cord from within this of its middle, m
const REACH: f64 = 1.3;
/// A blind on its way is drawn afresh this often, s (its slats moved)
const REDRAW: f64 = 1.0 / 15.0;

impl Game {
    /// Window `i`'s blinds' verbs (registered once it is finished)
    pub fn window_blinds_register(&mut self, i: usize) {
        for h in 0..self.windows[i].blinds.len() {
            let p = self.blind_cord(i, h);
            self.interact(p, REACH, "[E] LOWER THE BLIND", move |g| g.blind_verb(i, h) == Some(true), move |g| g.blind_use(i, h, true));
            self.interact(p, REACH, "[E] RAISE THE BLIND", move |g| g.blind_verb(i, h) == Some(false), move |g| g.blind_use(i, h, false));
        }
    }

    /// Where you take blind `h` of window `i` by its cord (world): before
    /// its middle, at your hands' height
    pub fn blind_cord(&self, i: usize, h: usize) -> Vec3 {
        let w = &self.windows[i];
        let at = w.blinds[h].at;
        w.world((at[0] + at[1]) / 2.0, 1.25, w.depth + 0.45)
    }

    /// What blind `h` of window `i` offers now: `Some(true)` to let it
    /// down, `Some(false)` to draw it up, nothing while it moves
    pub fn blind_verb(&self, i: usize, h: usize) -> Option<bool> {
        let w = self.windows.get(i)?;
        let b = w.blinds.get(h)?;
        if b.moving.is_some() {
            return None;
        }
        Some(!w.covered(b.lights[0]))
    }

    /// Let blind `h` of window `i` down (`lower`) or draw it up
    pub fn blind_use(&mut self, i: usize, h: usize, lower: bool) {
        let now = self.perf_t;
        let w = &mut self.windows[i];
        let k = w.blinds[h].lights[0];
        let from = w.lights[k].blind.unwrap_or(Blind { down: 0.0, open: 0.6, skew: 0.0 });
        let to = if lower { Blind { down: 1.0, open: 0.05, skew: 0.0 } } else { Blind { down: 0.0, open: from.open.max(0.5), skew: 0.0 } };
        w.blinds[h].moving = Some((from, to, now));
        self.blind_sound(i, h, lower);
        self.window_noise(i, RATTLE_REACH, 0.5);
    }

    /// Its slats rattling down (or a cord's zip and the slats stacking up):
    /// aluminium, a ticking run quickening, then the bottom rail's knock
    fn blind_sound(&mut self, i: usize, h: usize, lower: bool) {
        let p = self.blind_cord(i, h);
        let pp = self.player.pos;
        if crate::base::numeric::hypot2(pp.x - p.x, pp.z - p.z) > RATTLE_REACH {
            return;
        }
        let p = Vec3::new(p.x, p.y + 0.4, p.z);
        self.play_from(p.x, p.y, p.z, 2.0, 1.0, 0.7, move |g| {
            // the cord through the head rail
            g.noise_burst(0.5, 1900.0, 1.2, 0.03, "bandpass", Some(if lower { 2600.0 } else { 1500.0 }), 0.0, 0.0);
            // the slats clattering against each other (low and dull, well
            // under the pane's tick: the blind is not the glass)
            let mut t = 0.04;
            for k in 0..18 {
                let f = 1500.0 + ((k * 7) % 5) as f64 * 220.0;
                g.noise_burst(0.018, f, 2.5, 0.05, "bandpass", None, t, 0.0);
                g.noise_burst(0.010, f * 0.6, 2.0, 0.03, "bandpass", None, t + 0.008, 0.0);
                t += 0.07 - 0.0025 * k as f64;
            }
            // the bottom rail coming to rest
            g.noise_burst(0.05, 900.0, 2.0, 0.09, "bandpass", None, super::blind::BLIND_T - 0.05, 0.0);
            g.blip(1250.0, 0.12, "triangle", 0.025, Some(1150.0), super::blind::BLIND_T - 0.05, 0.0);
        });
    }

    /// What hears, within `reach` along the way, hears window `i`: it turns
    /// to it and comes to look near it (a sound leaves a rough guess)
    pub fn window_noise(&mut self, i: usize, reach: f64, loud: f64) {
        let w = &self.windows[i];
        let f = w.world(0.0, 0.0, w.depth + 0.8);
        let (gx, gz) = self.open_point(f.x, f.z);
        let now = self.perf_t;
        for ei in 0..self.entities.len() {
            let Some(c) = self.creature(ei) else { continue };
            if c.data.hearing <= 0.0 {
                continue;
            }
            let p = self.entities[ei].pos();
            let straight = crate::base::numeric::hypot2(p.x - f.x, p.z - f.z);
            if self.path_distance(p.x, p.z, straight) > reach * c.data.hearing {
                continue;
            }
            if let Some(c) = self.creature_mut(ei) {
                c.brain.mem.heard = Some((gx, gz, now));
                c.brain.mem.awareness = c.brain.mem.awareness.max(loud);
            }
        }
        self.director_note("blind");
    }

    /// Per frame (`windows_frame`): blinds on their way down or up, drawn
    /// afresh as their slats move; when one comes to rest, its window's
    /// light is thrown in again (a covered light throws none)
    pub(super) fn blinds_frame(&mut self) {
        let now = self.perf_t;
        let mut settled = vec![];
        for i in 0..self.windows.len() {
            for h in 0..self.windows[i].blinds.len() {
                let Some((from, to, t0)) = self.windows[i].blinds[h].moving else { continue };
                let k = ((now - t0) / BLIND_T).clamp(0.0, 1.0);
                let e = k * k * (3.0 - 2.0 * k);
                let b = Blind {
                    down: from.down + (to.down - from.down) * e,
                    open: from.open + (to.open - from.open) * e,
                    skew: from.skew * (1.0 - e),
                };
                let w = &mut self.windows[i];
                let done = k >= 1.0;
                // (its slats drawn afresh a few times a second, and at rest)
                let step = ((now - t0) / REDRAW).floor();
                let last = ((now - t0 - 1.0 / 60.0) / REDRAW).floor();
                if done || step != last {
                    let hg = &mut w.blinds[h];
                    let new = model::hang_blind(&hg.at, &b, &hg.mats);
                    w.root.remove(&hg.obj);
                    w.root.add(&new);
                    hg.obj = new;
                }
                let lights = w.blinds[h].lights.clone();
                for l in lights {
                    w.lights[l].blind = Some(if done { to } else { b });
                }
                if done {
                    w.blinds[h].moving = None;
                    settled.push(i);
                }
            }
        }
        settled.dedup();
        for i in settled {
            view::rethrow(self, i);
        }
    }
}
