//! Helpers placers and the core share (`canvas_tex`, `pg`, `joint`, ...).

use crate::base::rng::with_native;
use engine::prelude::*;

/// A repeating sRGB canvas texture. The draw runs on
/// the native RNG, never the level's seeded one.
pub fn canvas_tex(w: impl Num, h: impl Num, draw: impl FnOnce(&mut Canvas, f64, f64)) -> Tex {
    canvas_tex_opt(w, h, true, draw)
}
/// [`canvas_tex`] with a choice of repeat wrapping.
pub fn canvas_tex_opt(w: impl Num, h: impl Num, repeat: bool, draw: impl FnOnce(&mut Canvas, f64, f64)) -> Tex {
    let (w, h) = (w.f(), h.f());
    let mut c = Canvas::new(w as u32, h as u32);
    with_native(|| draw(&mut c, w, h));
    let t = c.to_tex();
    {
        let mut tt = t.t();
        tt.color_space = ColorSpace::Srgb;
        if repeat {
            tt.wrap_s = Wrap::Repeat;
            tt.wrap_t = Wrap::Repeat;
        }
        tt.anisotropy = 4;
    }
    t
}

/// The Blender-made prop part `name` fitted to the primitive `geo`.
pub fn pg(name: &str, geo: Geo) -> Geo {
    crate::assets::props::prop_part(name, geo, false)
}
/// Like [`pg`], but a private copy the caller may modify.
pub fn pg_own(name: &str, geo: Geo) -> Geo {
    crate::assets::props::prop_part(name, geo, true)
}

/// A named bone at
/// (x, y, z) under `parent`, carrying `geo` as a mesh when given.
pub fn joint(geo: Option<Geo>, mat: &Mat, parent: Option<&Obj>, x: f64, y: f64, z: f64, name: &str) -> Obj {
    let pivot = bone();
    pivot.set_name(name);
    pivot.set_pos(x, y, z);
    if let Some(g) = geo {
        pivot.add(&mesh(g, mat));
    }
    if let Some(p) = parent {
        p.add(&pivot);
    }
    pivot
}

/// Lazily created shared texture, built once on first use. Initialization runs
/// on the native RNG so it can never shift a level's seeded stream.
#[macro_export]
macro_rules! lazy_static_tex {
    ($fname:ident, $init:expr) => {
        pub fn $fname() -> engine::texture::Tex {
            thread_local! { static V: std::cell::OnceCell<engine::texture::Tex> = const { std::cell::OnceCell::new() }; }
            V.with(|c| c.get_or_init(|| $crate::base::rng::with_native(|| $init)).clone())
        }
    };
}
