//! The walkway with no end (the way in, `docs/design/WAY_IN.md`).
//!
//! Past the entrance's two modules the bore is laid as one period of
//! [`PERIOD`] modules, over and over, further than the haze lets you see
//! on either side ([`VIEW`]). Walking on, at the far seam you are carried
//! back a whole period; walking back, at the near one, on a whole period:
//! what you see is the same on both sides of a seam, so it cannot be seen.
//! The near seam waits until you have gone in far enough that the arch
//! behind you is lost in the haze ([`Ring::step`]): from then on the way
//! back comes round too.
//!
//! Pure numbers: no random draw, no clock, nothing of the eye.

/// One module's length (m), as the set is built.
pub const MOD: f64 = 6.0;

/// The ring's period, the module at each place in it: the plain bay (A),
/// a hatch and a way-out sign (B), a dead bar (D), the hydrant cabinet and
/// the first escape plate (C), jet fans (E), the escape door (F; G once it
/// has changed).
pub const PERIOD: [&str; 20] = ["A", "B", "A", "A", "D", "A", "C", "A", "B", "A", "A", "E", "A", "A", "B", "A", "F", "A", "A", "A"];

/// The first module of the ring (the entrance's two come before it).
pub const FIRST: usize = 2;

/// How many modules the haze lets you see, either way: at 252 m the haze
/// of the way in (`Set::HAZE_RING`) leaves 1% of what is there.
pub const VIEW: usize = 42;

/// The modules built: the ring's period with what can be seen past either
/// end of it.
pub const MODULES: usize = FIRST + VIEW + PERIOD.len() + VIEW;

/// The module standing at index `k` of the bore (from `FIRST`).
pub fn variant(k: usize) -> &'static str {
    PERIOD[(k - FIRST) % PERIOD.len()]
}

/// The period's length (m).
pub fn period() -> f64 {
    PERIOD.len() as f64 * MOD
}

/// Where the ring comes round behind you (z, the engine's: into the
/// tunnel is -z): past here the arch is out of sight.
pub fn near_seam() -> f64 {
    -((FIRST + VIEW) as f64) * MOD
}

/// Where the ring carries you back a period, walking on.
pub fn far_seam() -> f64 {
    near_seam() - period()
}

/// The ring's state: whether the way back has begun to come round.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ring {
    pub closed: bool,
}

impl Ring {
    /// After a step to `z` (the feet's): the shift along z to carry you
    /// by, if you passed a seam. Before you are a module past the near
    /// seam the ring stays open behind you (the arch is still to be seen).
    pub fn step(&mut self, z: f64) -> Option<f64> {
        if !self.closed {
            if z < near_seam() - MOD {
                self.closed = true;
            } else {
                return None;
            }
        }
        if z < far_seam() {
            Some(period())
        } else if z > near_seam() {
            Some(-period())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_repeats_and_is_built_past_its_seams_by_what_can_be_seen() {
        for k in FIRST..MODULES - PERIOD.len() {
            assert_eq!(variant(k), variant(k + PERIOD.len()), "module {k}");
        }
        // walking on from the far seam you see VIEW modules, all built
        let ahead = (-far_seam() / MOD) as usize + VIEW;
        assert!(ahead <= MODULES, "{ahead} > {MODULES}");
        // carried back to the near seam you see VIEW modules behind, all ring
        let behind = (-near_seam() / MOD) as usize - VIEW;
        assert_eq!(behind, FIRST);
        // one escape door a period
        assert_eq!(PERIOD.iter().filter(|v| **v == "F").count(), 1);
    }

    #[test]
    fn walking_on_and_back_comes_round() {
        let mut r = Ring::default();
        // near the arch: open behind you
        assert_eq!(r.step(-20.0), None);
        assert_eq!(r.step(near_seam() + 1.0), None);
        assert!(!r.closed);
        // past the near seam it closes
        assert_eq!(r.step(near_seam() - MOD - 0.1), None);
        assert!(r.closed);
        // on past the far seam: back a period, to just short of the near one
        let z = far_seam() - 0.05;
        let dz = r.step(z).unwrap();
        assert_eq!(dz, period());
        assert!(z + dz < near_seam() && z + dz > far_seam());
        // back past the near seam: on a period
        let z = near_seam() + 0.05;
        let dz = r.step(z).unwrap();
        assert_eq!(dz, -period());
        assert!(z + dz > far_seam() && z + dz < near_seam());
        // and the arch never comes back: walk back a long way
        let mut z = -300.0;
        for _ in 0..20_000 {
            z += 0.05;
            if let Some(dz) = r.step(z) {
                z += dz;
            }
            assert!(z < near_seam() + 0.06);
        }
    }
}
