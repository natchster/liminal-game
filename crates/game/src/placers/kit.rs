//! Helpers placers and the core share (`canvas_tex`, `pg`, `joint`, ...).

use crate::base::rng::with_own_stream;
use engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// How many textures each painter (its closure's type) has made since
    /// the level began.
    static MADE: RefCell<HashMap<&'static str, u32>> = RefCell::new(HashMap::new());
}

/// A level begins: each painter's count starts again, so what is painted
/// for each thing on it (a note's paper) comes out the same on every build.
pub fn restart_texture_streams() {
    MADE.with(|m| m.borrow_mut().clear());
}

/// The painter `F`'s own stream for its next texture: from its name (the
/// closure's place in the code), the size, and how many it has made since
/// the level began. Painted on the native stream, a texture had come out as
/// whatever was drawn before it, and shifted every frame after it, so a
/// level's frames depended on what the thread had made before (E10).
fn painter_seed<F>(w: f64, h: f64) -> u32 {
    let name = std::any::type_name::<F>();
    let n = MADE.with(|m| {
        let mut m = m.borrow_mut();
        let c = m.entry(name).or_insert(0);
        *c += 1;
        *c - 1
    });
    // FNV-1a
    let mut x: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.bytes().chain((w as u32).to_le_bytes()).chain((h as u32).to_le_bytes()).chain(n.to_le_bytes()) {
        x ^= u64::from(b);
        x = x.wrapping_mul(0x0100_0000_01b3);
    }
    (x ^ (x >> 32)) as u32
}

/// A repeating sRGB canvas texture. The draw runs on a stream of its own
/// (`painter_seed`), never the level's seeded one nor the native one.
pub fn canvas_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: impl Num, h: impl Num, draw: F) -> Tex {
    canvas_tex_opt(w, h, true, draw)
}
/// [`canvas_tex`] with a choice of repeat wrapping.
pub fn canvas_tex_opt<F: FnOnce(&mut Canvas, f64, f64)>(w: impl Num, h: impl Num, repeat: bool, draw: F) -> Tex {
    let (w, h) = (w.f(), h.f());
    let mut c = Canvas::new(w as u32, h as u32);
    with_own_stream(painter_seed::<F>(w, h), || draw(&mut c, w, h));
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
