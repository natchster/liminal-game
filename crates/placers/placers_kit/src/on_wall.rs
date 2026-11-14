//! Things on a wall's face: which way the face turns, where it is, a clamped mix.
use crate::place::WallCand as Cand;
use game::prelude::*;

/// The turn that faces a wall prop's +z out of candidate `c`'s wall, and
/// the direction along the wall its jitter moves it.
pub fn wall_turn(c: &Cand) -> (f64, (f64, f64)) {
    if c.dx.abs() == 1 {
        (if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 }, (0.0, 1.0))
    } else {
        (if c.dz > 0 { PI } else { 0.0 }, (1.0, 0.0))
    }
}

/// Where candidate `c`'s wall face is (x, z), on the line through its tile's centre.
pub fn wall_face(g: &Game, c: &Cand) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * CELL * 0.5, g.wz(c.z) + c.dz as f64 * CELL * 0.5)
}

/// A cheap hash of a few floats already drawn, for a choice that must not
/// draw (in 0..1).
pub fn mix01(a: f64, b: f64) -> f64 {
    let v = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    v - v.floor()
}
