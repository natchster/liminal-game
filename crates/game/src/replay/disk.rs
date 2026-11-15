//! Room on the disk for a recording (GitHub issue #9). A recording begins
//! only with enough of the disk free, and while it runs the disk is looked
//! at again every so often: below the line it ends, kept as it stands, and
//! the REC mark says why. The line and the interval are the player's
//! (`Settings::replay_free_pct`, `replay_check_min`, Settings › Replays); the
//! app measures the disk (`statvfs` on the replays folder) only when this
//! says a look is due, and acts on the answer. Times are the game's clock
//! (ms): a test runs minutes of it in a moment. Nothing here is read by
//! play, so a recording's frames never depend on the disk.

/// What a look at the disk found
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Disk {
    /// enough of it free, or it could not be measured (a recording is not
    /// refused, or ended, for what cannot be known)
    Room,
    /// too little: the share of it free (0..1)
    Low(f64),
}

/// When the disk is looked at while a recording runs
#[derive(Clone, Debug, Default)]
pub struct DiskGuard {
    /// the game's clock (ms) at the last look; None while nothing records
    last_ms: Option<f64>,
}

impl DiskGuard {
    /// A recording asked to begin, `free` the share of the disk free: always
    /// looked at, whatever the interval. `Room` begins it (and its looks)
    pub fn start(&mut self, free: Option<f64>, min_free_pct: i32, now_ms: f64) -> Disk {
        self.look(free, min_free_pct, now_ms)
    }
    /// Whether a look is due: `every_min` minutes since the last (read each
    /// frame, so a changed interval counts at once)
    pub fn due(&self, every_min: i32, now_ms: f64) -> bool {
        self.last_ms.is_some_and(|t| now_ms - t >= f64::from(every_min.max(1)) * 60_000.0)
    }
    /// The look while recording: `Low` ends the looks (the app ends the
    /// recording); `Room` counts the interval again from now
    pub fn check(&mut self, free: Option<f64>, min_free_pct: i32, now_ms: f64) -> Disk {
        self.look(free, min_free_pct, now_ms)
    }
    /// `free` (a share of the disk, or None) against the line (% of the
    /// disk): with room, the next look counts from now; without, none
    fn look(&mut self, free: Option<f64>, min_free_pct: i32, now_ms: f64) -> Disk {
        let d = match free {
            Some(f) if f * 100.0 < f64::from(min_free_pct) => Disk::Low(f),
            _ => Disk::Room,
        };
        self.last_ms = (d == Disk::Room).then_some(now_ms);
        d
    }
    /// The recording ended (the player ended it, or the guard)
    pub fn stop(&mut self) {
        self.last_ms = None;
    }
    /// Whether the looks are running (a recording began and is not over)
    pub fn watching(&self) -> bool {
        self.last_ms.is_some()
    }
}

/// A share of the disk as the notice says it ("3%", "less than 1%")
fn share(free: f64) -> String {
    if free < 0.01 { "less than 1%".into() } else { format!("{:.0}%", (free * 100.0).floor()) }
}

/// The REC mark's notice when a recording cannot begin
pub fn refused(free: f64) -> String {
    format!("Not recording: {} of the disk free", share(free))
}

/// The REC mark's notice when the disk ended a recording
pub fn stopped(free: f64) -> String {
    format!("Recording stopped: {} of the disk free", share(free))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: f64 = 60_000.0;

    /// The shares of the disk free a test pretends to see: never the disk
    /// itself (GitHub issue #9: "use safe values for disk space")
    const PLENTY: Option<f64> = Some(0.40);
    const SCANT: Option<f64> = Some(0.034);

    #[test]
    fn a_recording_begins_only_with_room_and_always_looks_first() {
        let mut g = DiskGuard::default();
        assert_eq!(g.start(SCANT, 5, 0.0), Disk::Low(0.034), "3.4% free is under a 5% line");
        assert!(!g.watching(), "a recording refused starts no looks");
        assert_eq!(g.start(SCANT, 3, 0.0), Disk::Room, "3.4% free is over a 3% line");
        assert!(g.watching());
        // the line itself is room enough; a disk that cannot be measured
        // refuses nothing
        assert_eq!(DiskGuard::default().start(Some(0.05), 5, 0.0), Disk::Room);
        assert_eq!(DiskGuard::default().start(None, 20, 0.0), Disk::Room);
        // the look at the start is made whatever the interval: `start` is
        // called at once, never `due`
        assert!(!DiskGuard::default().due(1, 10.0 * MIN), "nothing is due while nothing records");
    }

    #[test]
    fn the_disk_is_looked_at_again_every_minute_or_two() {
        let mut g = DiskGuard::default();
        g.start(PLENTY, 5, 1_000.0);
        // every minute (the default)
        assert!(!g.due(1, 1_000.0 + MIN - 1.0));
        assert!(g.due(1, 1_000.0 + MIN));
        assert_eq!(g.check(PLENTY, 5, 1_000.0 + MIN), Disk::Room);
        assert!(!g.due(1, 1_000.0 + MIN + 30_000.0), "a look counts the interval again from it");
        // every two minutes
        let t = 1_000.0 + MIN;
        assert!(!g.due(2, t + MIN));
        assert!(g.due(2, t + 2.0 * MIN));
        // a changed interval counts at once: from two minutes back to one
        assert!(g.due(1, t + MIN), "the interval is read at every frame");
    }

    #[test]
    fn a_disk_running_low_ends_the_recording_and_the_looks() {
        let mut g = DiskGuard::default();
        g.start(PLENTY, 5, 0.0);
        assert!(g.due(2, 2.0 * MIN));
        assert_eq!(g.check(Some(0.049), 5, 2.0 * MIN), Disk::Low(0.049));
        assert!(!g.watching() && !g.due(2, 10.0 * MIN), "nothing more to look at: the recording has ended");
        // the player's own stop ends the looks too
        g.start(PLENTY, 5, 0.0);
        g.stop();
        assert!(!g.watching());
    }

    #[test]
    fn the_notices_say_how_much_is_free() {
        assert_eq!(refused(0.034), "Not recording: 3% of the disk free");
        assert_eq!(stopped(0.049), "Recording stopped: 4% of the disk free");
        assert_eq!(refused(0.004), "Not recording: less than 1% of the disk free");
    }
}
