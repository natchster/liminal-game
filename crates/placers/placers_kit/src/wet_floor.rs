//! What water leaves on a floor: which floor it is on, and a pixel of a puddle's map.
use game::prelude::*;

/// What the floor of tile (x, z) is: 0 carpet (or carpet tiles), 1 concrete,
/// 2 glazed tile
pub fn floor_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.floor_mat) {
        Some("concrete") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// A pixel's straight sRGB colour and alpha, built up by laying marks over it
#[derive(Clone, Copy)]
pub struct Px {
    pub c: [f64; 3],
    pub a: f64,
}

impl Px {
    pub const CLEAR: Px = Px { c: [0.0; 3], a: 0.0 };
    /// `c` at alpha `a` laid over what is there
    pub fn lay(&mut self, c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        let out = a + self.a * (1.0 - a);
        if out > 1e-6 {
            for (k, v) in self.c.iter_mut().enumerate() {
                *v = (c[k] * a + *v * self.a * (1.0 - a)) / out;
            }
        }
        self.a = out;
    }
    pub fn rgba(self) -> [f64; 4] {
        [self.c[0], self.c[1], self.c[2], self.a]
    }
}
