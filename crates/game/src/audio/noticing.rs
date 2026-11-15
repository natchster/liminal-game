//! What you come upon (GitHub issue #10): the things of a floor your asides
//! may answer the first time you come upon one: a thing out of place
//! (`Use::Odd`, at most one a floor), a window, the vacuum, the floor's
//! door, the working lift and the broken one (`Use::Noticed`), one of the
//! Showroom's beds (`Use::Found`). The director decides as for any event
//! (`docs/design/VOICES.md`, *Your asides*): every kind once a run, its
//! chance falling with what was said, a word only when your breath is not
//! held.
//!
//! The list is made once a floor after its build, from what was built: no
//! draw, so a floor is the same whatever is noticed on it. Things out of
//! place are kept as their passes are tagged (`Game::tag_placed`), before
//! instancing takes the static ones out of the world, and read once the
//! floor is fitted. The list is looked through four times a second of play.

use crate::audio::asides::Use;
use crate::base::game::Game;
use crate::base::numeric::hypot2;
use engine::math::Vec3;

/// A thing is come upon within this, m: near enough to take it in
pub const NEAR: f64 = 5.0;
/// ... the vacuum, smaller and moving, within this, m
pub const VACUUM_NEAR: f64 = 4.0;
/// How often the floor's things are looked through, s
const EVERY: f64 = 0.25;
/// Within this, a thing is seen whatever the light and wherever you look, m
const BESIDE: f64 = 1.5;

/// A thing of the floor not yet come upon
#[derive(Clone, Debug, PartialEq)]
pub struct Notable {
    /// where it is seen from (in the room before it, at its foot)
    pub at: Vec3,
    pub use_: Use,
    pub subject: &'static str,
}

/// What a floor holds to be come upon, and what has been
#[derive(Clone, Debug, Default)]
pub struct Noticing {
    /// the floor's things not yet come upon
    pub things: Vec<Notable>,
    /// the vacuum was met this floor
    pub vacuum_met: bool,
    /// a thing out of place was come upon this floor
    pub odd_done: bool,
    /// time to the next look, s
    wait: f64,
}

/// What a thing out of place is about, by its placer and theme (`None`:
/// a thing an office has, or might)
pub fn odd_subject(placer: &str, theme: &str) -> Option<&'static str> {
    match (placer, theme) {
        ("place_traffic_cones", _) => Some("cones"),
        (_, "occult") => Some("occult"),
        (_, "toys") => Some("toys"),
        (_, "party") => Some("party"),
        _ => None,
    }
}

impl Game {
    /// The floor's things to come upon, from what was built (once a floor,
    /// after it is fitted; no draw)
    pub fn fit_noticing(&mut self) {
        let mut things = vec![];
        let mut add = |at: Vec3, use_: Use, subject: &'static str| things.push(Notable { at, use_, subject });
        for (o, subject) in std::mem::take(&mut self.odd_spots) {
            let p = o.pos();
            add(Vec3::new(p.x, p.y + 0.5, p.z), Use::Odd, subject);
        }
        if let Some(b) = &self.pool_bounds {
            add(Vec3::new((b.min_x + b.max_x) / 2.0, b.water_y, (b.min_z + b.max_z) / 2.0), Use::Odd, "pool");
        }
        for w in &self.windows {
            add(w.world(0.0, 1.4, w.depth + 0.6), Use::Noticed, "window");
        }
        // the floor's door, where it is only a door: the Loop's way in has its
        // sign hidden and the Borrowed House's door may not open, so a word
        // there would point at it (lore-keeper, #10)
        if let Some(e) = self.exit_data.as_ref().filter(|_| self.level_scp.is_none()) {
            add(e.pos, Use::Noticed, "exit");
        }
        if let Some(at) = self.lift_working_at {
            add(at, Use::Noticed, "lift");
        }
        if let Some(b) = &self.broken_elevator {
            add(Vec3::new(b.world.x + b.normal.x, b.world.y, b.world.z + b.normal.z), Use::Noticed, "lift_broken");
        }
        for s in self.store_hide_spots.iter().filter(|s| s.kind == "bed") {
            let p = s.obj.pos();
            add(Vec3::new(p.x, p.y + 0.5, p.z), Use::Found, "bed");
        }
        self.noticing = Noticing { things, ..Default::default() };
    }

    /// Whether you take in a thing at `at` from where you stand: within
    /// `near` on your storey, in sight, in front of you, and lit (or your
    /// lamp on, or close); beside it, always
    pub fn takes_in(&self, at: Vec3, near: f64) -> bool {
        let p = self.player.pos;
        let d = hypot2(at.x - p.x, at.z - p.z);
        if d > near || (at.y - p.y).abs() > 2.5 {
            return false;
        }
        if d < BESIDE {
            return true;
        }
        let mut to = Vec3::new(at.x - p.x, 0.0, at.z - p.z);
        to.normalize();
        let lit = d < 3.0 || self.player.flashlight_on || self.e_region_light_at(self.tx(at.x), self.tz(at.z)) > 0.25;
        self.player.fwd().dot(&to) > 0.6 && lit && self.e_los(p.x, p.z, at.x, at.z)
    }

    /// Four times a second of play: the things come upon tell your asides,
    /// each once a floor; a thing out of place only the first of the floor
    pub fn update_noticing(&mut self, dt: f64) {
        self.noticing.wait -= dt;
        if self.noticing.wait > 0.0 {
            return;
        }
        self.noticing.wait = EVERY;
        let mut come = vec![];
        let mut k = 0;
        while k < self.noticing.things.len() {
            if self.takes_in(self.noticing.things[k].at, NEAR) {
                come.push(self.noticing.things.remove(k));
            } else {
                k += 1;
            }
        }
        for n in come {
            if n.use_ == Use::Odd {
                // one thing out of place a floor: the rest of them are let go
                if self.noticing.odd_done {
                    continue;
                }
                self.noticing.odd_done = true;
                self.noticing.things.retain(|t| t.use_ != Use::Odd);
            }
            self.aside(n.use_, n.subject);
        }
        if !self.noticing.vacuum_met
            && let Some(at) = self.vacuum_bot.as_ref().map(|v| v.g.pos())
            && self.takes_in(Vec3::new(at.x, at.y + 0.2, at.z), VACUUM_NEAR)
        {
            self.noticing.vacuum_met = true;
            self.aside(Use::Noticed, "vacuum");
        }
    }
}
