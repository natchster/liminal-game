//! The self-assembly (`docs/MEGASTRUCTURE.md`, design 4): what comes on
//! the Building's floors, from far off. Nothing walks there; instead, now
//! and then, a siren from the loudspeakers on the walls, and out of the
//! void below a violet fog rises, its surface seen from above, the lamps
//! under it going out, until it stands among the walks; it holds there a
//! while, and sinks, and the all-clear sounds. Shelter is a cell: inside,
//! the wheel turned, the door sealed, you hear it at the door and it does
//! not come in. Out in it, your mind goes fast, and then you are gone.
//!
//! The director runs it on the Building's floors in place of the creatures
//! (calm: the quiet; build: the siren, the fog far below; peak: the fog
//! among the walks; relax: the fall and the all-clear), so the pacing log
//! reads the same. Taking her page sounds the siren within seconds.
//! Per-frame draws come from the native stream; nothing here touches a
//! level's build. Every floor tile has a cell within reach of the warning
//! at a walk (`level::building::CELL_REACH`, `tests/assembly.rs`).
use crate::entities::director::Phase;
use crate::prelude::*;

/// How long the siren sounds before the fog reaches the lowest walk, s
pub const WARNING: f64 = 42.0;
/// How fast the fog rises among the walks, and how fast it sinks, m/s
pub const RISE: f64 = 0.6;
pub const SINK: f64 = 1.2;
/// How long it stands at its crest, s (sealed in, you wait out a minute or
/// two, not more)
pub const CREST: (f64, f64) = (15.0, 22.0);
/// The quiet between two, s (the first on a floor is longer, and on the
/// first floor longer again while her camp is there to find)
pub const QUIET: (f64, f64) = (75.0, 115.0);
pub const FIRST_QUIET: (f64, f64) = (95.0, 130.0);
pub const FIRST_FLOOR_QUIET: (f64, f64) = (120.0, 160.0);
/// Out in the fog: how fast your mind goes, per second, eyes under it or
/// only feet; and how long you last in it at most, s
pub const DRAIN: f64 = 11.0;
pub const WADE_DRAIN: f64 = 3.0;
pub const LAST: f64 = 10.0;
/// How deep below the lowest walk the fog first shows, m
const DEEP: f64 = 30.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// nothing; the countdown to the next
    #[default]
    Quiet,
    /// the siren, the fog rising from far below toward the lowest walk
    Siren,
    /// the fog rising among the walks
    Rising,
    /// standing at its crest
    Crest,
    /// sinking away; the all-clear when it is gone
    Falling,
}

/// The self-assembly on this floor.
#[derive(Clone, Debug, Default)]
pub struct Assembly {
    pub stage: Stage,
    /// seconds left in the stage (Quiet, Siren, Crest)
    pub t: f64,
    /// the fog's surface, m (`f64::NEG_INFINITY` while there is none)
    pub y: f64,
    /// how high it comes this time, m
    pub crest: f64,
    /// the floor's lowest and highest walks, m
    pub low: f64,
    pub high: f64,
    /// how many have come on this floor
    pub count: u32,
    /// seconds to the siren's next wail, and to the fog's next sound
    pub wail_t: f64,
    pub sound_t: f64,
    /// seconds spent out in the fog, this time
    pub exposure: f64,
    /// the cell you are sealed in (`BuildingPlan::cells` index)
    pub sealed: Option<usize>,
    /// seconds to the next knock at a sealed door
    pub knock_t: f64,
}

impl Assembly {
    /// Whether the fog is anywhere on the floor
    pub fn up(&self) -> bool {
        matches!(self.stage, Stage::Siren | Stage::Rising | Stage::Crest | Stage::Falling)
    }
}

impl Game {
    /// A floor of the Building has begun: the quiet before the first.
    pub fn reset_assembly(&mut self) {
        let walks: Vec<f64> = self.ground.iter().filter(|g| g.walkable()).map(|g| g.h0.min(g.h1) as f64).collect();
        let low = walks.iter().cloned().fold(f64::INFINITY, f64::min);
        let high = walks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let first = if self.level_num == 1 && self.story.camp.is_some() { FIRST_FLOOR_QUIET } else { FIRST_QUIET };
        // the director's first quiet (its one draw, 45–80 s on these floors),
        // spread over this one: another draw at a floor's start would move
        // every frame after it (CLAUDE.md invariant 1)
        let u = ((self.director.t - 45.0) / 35.0).clamp(0.0, 1.0);
        let t = first.0 + u * (first.1 - first.0);
        self.assembly = Assembly { stage: Stage::Quiet, t, y: f64::NEG_INFINITY, low, high, ..Default::default() };
        self.director = crate::entities::director::Director { t: self.assembly.t, ..Default::default() };
    }

    /// Her page was taken, or a trail that is not hers ran out: the siren
    /// within `secs` (a quiet is cut short; already coming, nothing changes)
    pub fn assembly_wake(&mut self, secs: (f64, f64)) {
        if self.assembly.stage == Stage::Quiet {
            self.assembly.t = self.assembly.t.min(rand2(secs.0, secs.1));
        }
    }

    /// Where the fog is over (x, z) and whether you are in it
    fn in_fog(&self) -> (bool, bool) {
        if !self.assembly.up() || self.assembly.sealed.is_some() {
            return (false, false);
        }
        let p = self.player.pos;
        let foot = self.floor_y_at(p.x, p.z);
        let eye = foot + self.player.eye_h;
        (eye < self.assembly.y, foot < self.assembly.y - 0.1)
    }

    /// One step of the self-assembly (the director's, on the Building's floors)
    pub fn update_assembly(&mut self, dt: f64) {
        let a = &mut self.assembly;
        let stage = a.stage;
        match stage {
            Stage::Quiet => {
                a.t -= dt;
                if a.t <= 0.0 {
                    a.stage = Stage::Siren;
                    a.t = WARNING;
                    a.count += 1;
                    a.y = a.low - DEEP;
                    a.crest = a.low + (a.high - a.low) * rand2(0.55, 1.05) + 2.0;
                    a.wail_t = 0.0;
                    a.sound_t = 2.0;
                    a.exposure = 0.0;
                    self.director.phase = Phase::Build;
                    self.director_note("build");
                    self.director_note("assembly:siren");
                }
            }
            Stage::Siren => {
                a.t -= dt;
                // from far below to the lowest walk in the warning
                a.y += DEEP / WARNING * dt;
                if a.t <= 0.0 {
                    // (it rises on without a break)
                    a.stage = Stage::Rising;
                    self.director.phase = Phase::Peak;
                    self.director_note("peak");
                }
            }
            Stage::Rising => {
                a.y += RISE * dt;
                if a.y >= a.crest {
                    a.y = a.crest;
                    a.stage = Stage::Crest;
                    a.t = rand2(CREST.0, CREST.1);
                }
            }
            Stage::Crest => {
                a.t -= dt;
                if a.t <= 0.0 {
                    a.stage = Stage::Falling;
                    self.director.phase = Phase::Relax;
                    self.director_note("relax");
                }
            }
            Stage::Falling => {
                a.y -= SINK * dt;
                if a.y < a.low - DEEP {
                    a.stage = Stage::Quiet;
                    a.y = f64::NEG_INFINITY;
                    a.t = rand2(QUIET.0, QUIET.1);
                    self.director.phase = Phase::Calm;
                    self.director_note("assembly:clear");
                    self.assembly_all_clear();
                }
            }
        }
        self.director.t = self.assembly.t;
        // the siren wails from the speakers while it is coming and up
        if matches!(self.assembly.stage, Stage::Siren | Stage::Rising) {
            self.assembly.wail_t -= dt;
            if self.assembly.wail_t <= 0.0 {
                self.assembly.wail_t = if self.assembly.stage == Stage::Siren { 9.0 } else { 16.0 };
                self.assembly_wail();
            }
        }
        // the fog's own sound, from its surface under you or round you
        if self.assembly.up() {
            self.assembly.sound_t -= dt;
            if self.assembly.sound_t <= 0.0 {
                self.assembly.sound_t = rand2(2.5, 4.5);
                let p = self.player.pos;
                let d = (p.y - EYE + 1.0 - self.assembly.y).abs();
                if d < 28.0 && self.ac.is_some() {
                    let y = self.assembly.y;
                    self.play_from(p.x + rand2(-6.0, 6.0), y, p.z + rand2(-6.0, 6.0), 12.0, 1.4, 0.0, |g| g.sfx_fog());
                }
            }
        }
        // in a sealed cell while it is up: something at the door
        if let (Some(ci), true) = (self.assembly.sealed, matches!(self.assembly.stage, Stage::Rising | Stage::Crest)) {
            self.assembly.knock_t -= dt;
            if self.assembly.knock_t <= 0.0 {
                self.assembly.knock_t = rand2(7.0, 15.0);
                let at = self.cell_door_at(ci);
                if let (Some(ss), Some(at)) = (self.soundscape(), at) {
                    ss.knock(2 + (native_rnd() * 3.0) as i32, at.x, at.y, at.z, 0.45);
                }
            }
        }
        // out in it
        let (under, wading) = self.in_fog();
        if under || wading {
            self.assembly.exposure += dt;
            self.player.sanity -= dt * if under { DRAIN } else { WADE_DRAIN };
            if self.player.sanity <= 0.0 || self.assembly.exposure > LAST {
                self.player.sanity = self.player.sanity.max(0.0);
                self.on_death("assembly");
            }
        }
        // how close it feels: the fog's surface under your eye
        let eye = self.player.pos.y - EYE + self.player.eye_h;
        let near = if self.assembly.up() { clamp(1.0 - (eye - self.assembly.y) / 18.0, 0.0, 1.0) } else { 0.0 };
        let target = if self.assembly.sealed.is_some() { near * 0.4 } else { near };
        let rate = if target > self.director.tension { 1.2 } else { 0.3 };
        self.director.tension += (target - self.director.tension) * (dt * rate).min(1.0);
        self.update_fog_surface();
    }

    /// The siren: a wail from the loudspeakers within earshot
    fn assembly_wail(&mut self) {
        if self.ac.is_none() {
            return;
        }
        let p = self.player.pos;
        let mut speakers = self.building_speakers.clone();
        speakers.sort_by(|a, b| a.distance_to(&p).partial_cmp(&b.distance_to(&p)).unwrap_or(std::cmp::Ordering::Equal));
        for s in speakers.iter().take(2) {
            self.play_from(s.x, s.y, s.z, 40.0, 5.0, 0.3, |g| g.sfx_siren());
        }
    }

    /// The all-clear: one long steady tone from the speakers
    fn assembly_all_clear(&mut self) {
        if self.ac.is_none() {
            return;
        }
        let p = self.player.pos;
        if let Some(s) = self
            .building_speakers
            .iter()
            .min_by(|a, b| a.distance_to(&p).partial_cmp(&b.distance_to(&p)).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
        {
            self.play_from(s.x, s.y, s.z, 40.0, 4.0, 0.3, |g| g.sfx_all_clear());
        }
    }
}

/// One cell's hermetic door, as it swings: the leaf and its wheel turn on
/// the hinge (0 shut, `OPEN` open into the room)
#[derive(Clone, Debug)]
pub struct CellDoor {
    pub tile: (i32, i32),
    pub leaf: Obj,
    pub open: f64,
    pub target: f64,
    /// where it stands: the doorway's middle on the floor, world
    pub at: Vec3,
    /// the doorway, world (x0, z0, x1, z1): shut, it stops a body
    pub span: [f64; 4],
}
/// How far a cell's door stands open, radians
pub const OPEN: f64 = 1.75;

impl Game {
    /// Where cell `ci`'s door is (for what knocks at it)
    pub fn cell_door_at(&self, ci: usize) -> Option<Vec3> {
        self.cell_doors.get(ci).map(|d| Vec3::new(d.at.x, d.at.y + 1.2, d.at.z))
    }

    /// Whether a body at (px, pz) of radius r meets a shut cell door
    pub fn hits_cell_door(&self, px: f64, pz: f64, r: f64) -> bool {
        self.cell_doors.iter().filter(|d| d.target < 0.5).any(|d| {
            let [x0, z0, x1, z1] = d.span;
            let (cx, cz) = (px.clamp(x0, x1), pz.clamp(z0, z1));
            (px - cx).powi(2) + (pz - cz).powi(2) < r * r
        })
    }

    /// Turn the wheel of cell `ci` (shut it, or open it)
    pub fn turn_cell_wheel(&mut self, ci: usize, shut: bool) {
        let Some(d) = self.cell_doors.get_mut(ci) else { return };
        d.target = if shut { 0.0 } else { OPEN };
        let at = d.at;
        self.assembly.sealed = if shut { Some(ci) } else { None };
        self.director_note(if shut { "assembly:sealed" } else { "assembly:opened" });
        if self.ac.is_some() {
            self.play_from(at.x, at.y + 1.1, at.z, 4.0, 1.0, 0.0, |g| g.sfx_wheel(shut));
        }
    }

    /// The doors swing toward where their wheel was turned
    pub fn update_cell_doors(&mut self, dt: f64) {
        for d in self.cell_doors.iter_mut() {
            if (d.open - d.target).abs() < 1e-4 {
                continue;
            }
            d.open += (d.target - d.open) * (dt * 2.6).min(1.0);
            if (d.open - d.target).abs() < 0.01 {
                d.open = d.target;
            }
            let yaw = d.leaf.r().user.get("yaw");
            d.leaf.rotation().y = yaw + d.open;
        }
    }

    /// The fog's surface where it is, and the lamps under it out
    pub fn update_fog_surface(&mut self) {
        let up = self.assembly.up();
        let y = self.assembly.y;
        for (k, o) in self.fog_layers.iter().enumerate() {
            o.set_visible(up);
            if up {
                o.position().y = y - k as f64 * 0.8;
                o.rotation().z += 0.0006 * (k as f64 + 1.0);
            }
        }
        // the lamps the fog has come over go out, and come back as it goes
        let mut changed = false;
        for p in self.panels.iter_mut() {
            let under = up && p.y < y;
            let was = p.f.get("fogged").copied().unwrap_or(0.0) > 0.5;
            if under != was {
                p.f.insert("fogged".into(), if under { 1.0 } else { 0.0 });
                p.switched_off = under;
                changed = true;
            }
        }
        // (the light map follows the lamps switched off: `update_lights`)
        let _ = changed;
    }
}
