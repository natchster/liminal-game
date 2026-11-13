//! The helpers the placers share: noise, painting, placing and building,
//! each kept once (it had been copied from crate to crate, up to 67 times).
//! A placer crate uses them by name; a placer's own helpers stay with it.

pub mod geometry;
pub mod noise;
pub mod paint;
pub mod place;
