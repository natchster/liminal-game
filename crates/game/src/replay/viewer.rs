//! Watching a replay (`docs/engineering/REPLAYS.md`, *Watching*): a
//! recording played back by the window's clock at a speed, paused, moved
//! ten seconds either way, with the line that says where it is. Moving back
//! starts a fresh game at the anchor before the place and simulates to it;
//! a seek runs a slice a window frame, unheard. Nothing here touches the
//! window: the app maps its keys onto a `Viewer` and draws its `bar`.
use super::{Player, ReplayFile};
use crate::base::game::CLOCK_TICKS_PER_SECOND;
use crate::prelude::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The speeds a replay plays at, slowest to fastest.
pub const SPEEDS: [f64; 5] = [0.25, 0.5, 1.0, 2.0, 4.0];

/// Where each frame of a recording is in time (from its first frame).
pub struct Timeline {
    first: u64,
    /// the clock's ticks at the end of each frame
    ends: Vec<u64>,
}

impl Timeline {
    pub fn of(f: &ReplayFile) -> Timeline {
        let first = f.segments.first().map_or(0, |s| s.first_frame);
        let mut t = 0u64;
        let ends = f
            .segments
            .iter()
            .flat_map(|s| s.frames.iter())
            .map(|(k, _)| {
                t += u64::from(*k);
                t
            })
            .collect();
        Timeline { first, ends }
    }
    /// The recording's clock, in seconds, as frame `f` begins.
    pub fn seconds(&self, f: u64) -> f64 {
        let i = f.saturating_sub(self.first) as usize;
        let t = if i == 0 { 0 } else { self.ends.get(i - 1).or(self.ends.last()).copied().unwrap_or(0) };
        t as f64 / f64::from(CLOCK_TICKS_PER_SECOND)
    }
    /// The frame that begins at or before `s` seconds.
    pub fn frame_at(&self, s: f64) -> u64 {
        let t = (s * f64::from(CLOCK_TICKS_PER_SECOND)).max(0.0) as u64;
        self.first + self.ends.partition_point(|&e| e <= t) as u64
    }
    /// How long frame `f` lasts, in seconds.
    pub fn interval(&self, f: u64) -> f64 {
        self.seconds(f + 1) - self.seconds(f)
    }
}

/// What the line under a replay shows (`ui::draw`): set by the app from its
/// `Viewer`, drawn only.
#[derive(Clone, Debug, Default)]
pub struct PlaybackBar {
    /// seconds from the in-point, and the recording's length
    pub at: f64,
    pub total: f64,
    pub speed: f64,
    pub paused: bool,
    /// a seek under way: how far it has come (0–1)
    pub seeking: Option<f64>,
    /// what the player should know: made by another build; where the
    /// replay left its recording
    pub note: String,
}

/// One recording on the Replays screen (the app reads the folder).
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayLine {
    pub path: PathBuf,
    /// "Floor 4"
    pub label: String,
    /// "14 Nov, 20:43 · 2:31", and "another build" when it was
    pub detail: String,
}

/// A recording being watched.
pub struct Viewer {
    pub player: Player,
    timeline: Timeline,
    /// what is shown: from the in-point to the out-point
    from: u64,
    to: u64,
    pub speed: f64,
    pub paused: bool,
    /// a seek under way: the frame it set out from, and the one it goes to
    seeking: Option<(u64, u64)>,
    /// the window's time not yet played (at the speed)
    owed: f64,
    /// whether the sound was last left hushed
    hushed: bool,
}

impl Viewer {
    /// `file` opened to be watched: its game at its first segment's start,
    /// seeking to its in-point (`make` as `Player::start` takes it).
    pub fn open(file: ReplayFile, make: impl FnOnce(&str) -> Game) -> Result<(Game, Viewer), String> {
        let timeline = Timeline::of(&file);
        let (from, to) = (file.in_point(), file.out_point());
        let (g, player) = Player::start(file, 0, make)?;
        let seeking = (player.frame < from || player.pending_entry()).then_some((player.frame, from));
        let v = Viewer { player, timeline, from, to, speed: 1.0, paused: false, seeking, owed: 0.0, hushed: false };
        Ok((g, v))
    }

    /// One window frame: a slice of a seek (until `budget` is spent), or
    /// the recorded frames the window's `dt` reaches at the speed (at most
    /// 16: a window that falls behind slows the replay, it does not race).
    /// False once the recording is done.
    pub fn frame(&mut self, g: &mut Game, dt: f64, budget: Duration) -> bool {
        let quiet = self.quiet();
        if let Some(ac) = g.ac.as_ref().filter(|_| quiet != self.hushed) {
            ac.set_muted(quiet || g.muted);
            self.hushed = quiet;
        }
        if let Some((_, to)) = self.seeking {
            let t0 = Instant::now();
            while self.player.frame < to || self.player.pending_entry() {
                if !self.player.step(g) {
                    self.seeking = None;
                    return false;
                }
                // what the seek made is cut as it goes (a minute of
                // footsteps would sound at once)
                if self.player.frame.is_multiple_of(120) {
                    cut(g);
                    if t0.elapsed() > budget {
                        return true;
                    }
                }
            }
            cut(g);
            self.seeking = None;
            self.owed = 0.0;
            return true;
        }
        if self.paused {
            return true;
        }
        self.owed += dt * self.speed;
        for _ in 0..16 {
            if self.player.frame >= self.to {
                return false;
            }
            let step = self.timeline.interval(self.player.frame);
            if self.owed < step {
                return true;
            }
            self.owed -= step;
            if !self.player.step(g) {
                return false;
            }
        }
        self.owed = self.owed.min(self.timeline.interval(self.player.frame));
        self.player.frame < self.to
    }

    /// Ten seconds (or `by`) on or back: on, by simulating; back, from the
    /// anchor before the place, in a fresh game (`make`) put in `g`.
    pub fn jump(&mut self, g: &mut Game, by: f64, make: impl FnOnce(&str) -> Game) -> Result<(), String> {
        let here = self.timeline.seconds(self.player.frame);
        let target = self.timeline.frame_at(here + by).clamp(self.from, self.to.saturating_sub(1).max(self.from));
        if target < self.player.frame {
            let seg = self.player.file.segments.iter().rposition(|s| s.first_frame <= target).unwrap_or(0);
            let (fresh, player) = Player::start(self.player.file.clone(), seg, make)?;
            *g = fresh;
            self.player = player;
            self.hushed = false;
        }
        self.seeking = Some((self.player.frame, target));
        self.owed = 0.0;
        Ok(())
    }

    pub fn faster(&mut self) {
        self.speed = SPEEDS.into_iter().find(|&s| s > self.speed).unwrap_or(self.speed);
    }
    pub fn slower(&mut self) {
        self.speed = SPEEDS.into_iter().rev().find(|&s| s < self.speed).unwrap_or(self.speed);
    }

    /// Whether the sound is hushed: seeking, paused, or not at its speed.
    pub fn quiet(&self) -> bool {
        self.seeking.is_some() || self.paused || self.speed != 1.0
    }

    /// The line under the replay, as it stands (the app adds its note).
    pub fn bar(&self) -> PlaybackBar {
        let s0 = self.timeline.seconds(self.from);
        let seeking = self.seeking.map(|(a, b)| if b > a { (self.player.frame.saturating_sub(a)) as f64 / (b - a) as f64 } else { 1.0 });
        let note = match &self.player.diverged {
            Some(d) => format!(
                "The replay left its recording at {} ({}): another build, or a fault",
                clock(self.timeline.seconds(d.frame) - s0),
                d.parts.join(", ")
            ),
            None => String::new(),
        };
        PlaybackBar {
            at: (self.timeline.seconds(self.player.frame) - s0).max(0.0),
            total: self.timeline.seconds(self.to) - s0,
            speed: self.speed,
            paused: self.paused,
            seeking,
            note,
        }
    }
}

/// `m:ss`.
pub fn clock(s: f64) -> String {
    let s = s.max(0.0) as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

fn cut(g: &Game) {
    if let Some(ac) = &g.ac {
        ac.cut_one_shots();
    }
}
