//! Where you can stand: boxes of walkable ground (the manifest's `walk`),
//! each at one height: the walkway, the plaza, every tread of the stairs,
//! the street above. A move is allowed when the feet and a ring round them
//! at the body's radius all stand on ground within a step of the feet; a
//! blocked move slides along whichever axis is free. Walls, the balustrade,
//! the parapets and the glass are simply where there is no ground. No
//! random draw, no clock: the same moves make the same walk.
use crate::data::WalkBox;
use engine::dmath::DMath;

/// The tallest step taken without climbing (a stair's riser is 0.175 m).
pub const STEP: f64 = 0.30;
/// The body's radius.
pub const RADIUS: f64 = 0.24;

pub struct Walk {
    boxes: Vec<WalkBox>,
}

impl Walk {
    pub fn new(boxes: Vec<WalkBox>) -> Walk {
        Walk { boxes }
    }

    /// The ground under (x, z) nearest below or within a step above `feet`.
    pub fn ground(&self, x: f64, z: f64, feet: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        for b in &self.boxes {
            if x >= b.x[0] - 1e-6 && x <= b.x[1] + 1e-6 && z >= b.z[0] - 1e-6 && z <= b.z[1] + 1e-6 && b.y <= feet + STEP {
                best = Some(best.map_or(b.y, |g| g.max(b.y)));
            }
        }
        best
    }

    /// Whether (x, z) is open, over ground at any height, by `r` all round
    /// (a camera's clearance: not in a wall).
    pub fn open(&self, x: f64, z: f64, r: f64) -> bool {
        let any = |x: f64, z: f64| self.ground(x, z, f64::INFINITY).is_some();
        any(x, z)
            && (0..8).all(|k| {
                let (s, c) = (k as f64 * std::f64::consts::FRAC_PI_4).dsin_cos();
                any(x + c * r, z + s * r)
            })
    }

    /// Whether a body stands at (x, z) with its feet at `feet`: the ground
    /// under every point of its ring within a step of the feet.
    pub fn fits(&self, x: f64, z: f64, feet: f64) -> bool {
        let Some(g) = self.ground(x, z, feet) else { return false };
        if (g - feet).abs() > STEP {
            return false;
        }
        for k in 0..8 {
            let (s, c) = (k as f64 * std::f64::consts::FRAC_PI_4).dsin_cos();
            match self.ground(x + c * RADIUS, z + s * RADIUS, feet) {
                Some(h) if (h - feet).abs() <= STEP => {}
                _ => return false,
            }
        }
        true
    }

    /// Move the feet at `p` (x, y, z) by (dx, dz), sliding along what
    /// blocks; the feet follow the ground.
    pub fn step(&self, p: [f64; 3], dx: f64, dz: f64) -> [f64; 3] {
        // long moves in short pieces, so a thin wall is never stepped over
        let n = ((dx.abs().max(dz.abs()) / 0.1).ceil() as usize).max(1);
        let (sx, sz) = (dx / n as f64, dz / n as f64);
        let mut q = p;
        for _ in 0..n {
            for (mx, mz) in [(sx, sz), (sx, 0.0), (0.0, sz)] {
                if mx == 0.0 && mz == 0.0 {
                    continue;
                }
                let (x, z) = (q[0] + mx, q[2] + mz);
                if let Some(g) = self.ground(x, z, q[1])
                    && self.fits(x, z, g)
                {
                    q = [x, g, z];
                    break;
                }
            }
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk() -> Walk {
        let b = |x0: f64, x1: f64, z0: f64, z1: f64, y: f64| WalkBox { x: [x0, x1], z: [z0, z1], y, name: String::new() };
        let mut v = vec![b(0.0, 2.0, -50.0, 0.0, 0.2), b(0.0, 6.0, 0.0, 10.0, 0.2)];
        // a stair up along +z from z = 10: risers 0.175
        for k in 0..10 {
            v.push(b(0.0, 1.5, 10.0 + k as f64 * 0.3, 10.3 + k as f64 * 0.3, 0.2 + 0.175 * (k + 1) as f64));
        }
        Walk::new(v)
    }

    #[test]
    fn walls_stop_and_slide() {
        let w = walk();
        let p = w.step([1.0, 0.2, -5.0], 3.0, -1.0);
        assert!(p[0] <= 2.0 - RADIUS + 1e-9, "{p:?}");
        assert!((p[2] - -6.0).abs() < 1e-9, "slid along the wall: {p:?}");
    }

    #[test]
    fn stairs_are_climbed_and_descended() {
        let w = walk();
        let mut p = [0.75, 0.2, 9.0];
        for _ in 0..40 {
            p = w.step(p, 0.0, 0.1);
        }
        assert!(p[1] > 1.5, "climbed: {p:?}");
        for _ in 0..40 {
            p = w.step(p, 0.0, -0.1);
        }
        assert!((p[1] - 0.2).abs() < 1e-9, "back down: {p:?}");
    }

    #[test]
    fn no_falling_off_a_flight() {
        let w = walk();
        // halfway up, step sideways off the stair toward the plaza side's air
        let mut p = [0.75, 0.2, 9.0];
        for _ in 0..25 {
            p = w.step(p, 0.0, 0.1);
        }
        let q = w.step(p, 2.0, 0.0);
        assert!(q[0] <= 1.5 - RADIUS + 1e-9, "{q:?}");
    }
}
