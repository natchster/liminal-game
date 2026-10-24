//! Native macOS engine for the Backrooms: a three.js-compatible scene graph
//! (r160 semantics), a Metal renderer, the HTML canvas on CoreGraphics and a
//! WebAudio-style graph on CoreAudio.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod math;
pub mod css;
pub mod earcut;
pub mod geometry;
pub mod texture;
pub mod material;
pub mod scene;
pub mod cg;
pub mod canvas;
pub mod audio;
pub mod png;
#[cfg(feature = "wip")]
pub mod render;
#[cfg(feature = "wip")]
pub mod app;

/// Everything a ported three.js / canvas / WebAudio call site needs.
pub mod prelude {
    pub use crate::math::*;
    pub use crate::geometry::*;
    pub use crate::texture::*;
    pub use crate::material::*;
    pub use crate::scene::*;
    pub use crate::canvas::{Canvas, Gradient, ImageData, Style, TextMetrics};
    pub use crate::audio::{AudioBuffer, AudioContext, AudioListener, AudioNode, AudioParam};
    pub use crate::{box_geo, cyl_geo, cone_geo, sphere_geo, plane_geo, circle_geo, torus_geo, capsule_geo, octa_geo, icosa_geo, dodeca_geo, tetra_geo, ring_geo, lathe_geo, shape_geo};
    pub use crate::{lambert, basic_mat, phong, standard_mat, points_mat, sprite_mat, line_mat, point_light, spot_light};
    pub use std::f64::consts::PI as MATH_PI;
}
