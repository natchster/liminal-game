//! What several occult placers share.
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;

/// A small stream of its own for a painter's layout (no level draw)
pub(crate) struct Rng32(pub(crate) u32);

impl Rng32 {
    pub(super) fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        f64::from(self.0) / 4_294_967_296.0
    }
}
