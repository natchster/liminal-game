//! Native macOS engine for the Backrooms:
//! * a scene graph of transformed nodes (groups, meshes, instanced and skinned
//!   meshes, sprites, points, lines, lights, cameras),
//! * geometry generators (boxes, cylinders, spheres, lathes, shapes, ...) and
//!   materials (basic, Lambert, Phong, points, sprites, lines),
//! * a Metal renderer with per-fragment Lambert / Phong / basic shading and
//!   exponential-squared (FogExp2-style) fog,
//! * a 2D canvas API on CoreGraphics / CoreText for textures and the HUD,
//! * an audio node graph rendered on CoreAudio (or offline, for recording),
//! * winit windowing, input and the frame loop.

pub mod app;
pub mod audio;
pub mod canvas;
pub mod cg;
pub mod css;
pub mod earcut;
pub mod geometry;
pub mod material;
pub mod math;
pub mod png;
pub mod render;
pub mod scene;
pub mod texture;
pub mod wav;

/// Everything game code needs: math, geometry, textures, materials, the scene
/// graph, the canvas and the audio graph, plus the constructor macros.
pub mod prelude {
    pub use crate::audio::{AudioBuffer, AudioContext, AudioListener, AudioNode, AudioParam};
    pub use crate::canvas::{Canvas, Gradient, ImageData, Style, TextMetrics};
    pub use crate::geometry::*;
    pub use crate::material::*;
    pub use crate::math::*;
    pub use crate::scene::*;
    pub use crate::texture::*;
    pub use crate::{basic_mat, lambert, line_mat, phong, point_light, points_mat, spot_light, sprite_mat, standard_mat};
    pub use crate::{
        box_geo, capsule_geo, circle_geo, cone_geo, cyl_geo, dodeca_geo, icosa_geo, lathe_geo, octa_geo, plane_geo, ring_geo, shape_geo,
        sphere_geo, tetra_geo, torus_geo,
    };
    pub use std::f64::consts::PI as MATH_PI;
}
