//! Batch `t20`'s abacus, made real (`docs/design/PROPS.md` P3; the Blender batch
//! `assets/src/blender/props/t20.py`, its notes `t20.notes.md`): a small
//! Chinese abacus (suanpan) of 13 rods lying on its back on the floor. It
//! keeps the old placer's draws, call for call.
use super::*;
use t20_kit::*;

// Small vintage abacus: 6% chance per level. A suanpan (Blender batch
// `t20`): hardwood rails strapped with brass at the corners, bamboo rods,
// two beads above the beam and five below on each; cleared, or with a sum
// left on it. The wood dark and dull, the beads polished by fingers, dust
// in the frame's corners.
pub(super) fn place_small_abacuses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old draws, call for call: where it lies and how it is turned
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let abacus = group();
    abacus.position().set(px, 0, pz);
    abacus.rotation().y = turn;
    let vary = vary(px, pz, turn);
    put(&abacus, "Real__abacusFrame", &phong!({ map: &ab_wood_tex(vary), specular: 0x1c1814, shininess: 20 }));
    put(&abacus, "Real__abacusRods", &lambert!({ color: 0x7a6440 }));
    put(&abacus, "Real__abacusBrass", &phong!({ color: 0x5e4a28, specular: 0x4a3c22, shininess: 30 }));
    // cleared, or with the last sum left on it (where it lies, no draw)
    let beads = if hash(vary, 1, 2) < 0.5 { "Real__abacusBeadsA" } else { "Real__abacusBeadsB" };
    put(&abacus, beads, &phong!({ map: &ab_bead_tex(vary), specular: 0x2a2018, shininess: 34 }));
    g.world.add(&abacus);
}

/// The frame's hardwood (u along the rails): near black-brown, its edges
/// rubbed to the red of the wood, dust lodged along the rails' feet
fn ab_wood_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0xab;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, u, v, [0.13, 0.07, 0.045], 14.0, 40.0);
            let rub = ss(0.6, 0.8, fbm(seed ^ 3, u * 12.0, v * 6.0, 3));
            o = mix(o, [0.26, 0.13, 0.08], rub * 0.4);
            let dust = 0.06 + 0.14 * fbm(seed ^ 5, u * 9.0, v * 9.0, 3);
            opaque(mix(o, [0.42, 0.39, 0.34], dust))
        });
    })
}

/// The beads' wood (as their bounds are mapped): red-brown, polished, each
/// a little different
fn ab_bead_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0xbe;
        fill(ctx, w, h, |u, v| {
            let o = wood(seed, u, v, [0.24, 0.11, 0.06], 30.0, 60.0);
            let each = 0.85 + 0.25 * vn(seed ^ 1, u * 16.0, v * 10.0);
            opaque(mix(o.map(|c| c * each), [0.40, 0.37, 0.32], 0.06))
        });
    })
}

/// Batch `t20`'s painting kit (each of its modules has its own copy): the
/// parts placed as modelled, a value of where a thing was put to vary its
/// painters, value noise, and a painter that fills a canvas pixel by pixel
/// on every core (no canvas call and no draw inside the loop)
#[allow(dead_code)]
mod t20_kit {
    use game::assets::props::prop_part_as_modelled;
    use game::prelude::*;

    pub use placers_kit::geometry::put;

    pub use placers_kit::place::vary;

    pub use placers_kit::noise::hash;

    pub use placers_kit::noise::vn;

    pub use placers_kit::noise::fbm;

    pub use placers_kit::noise::sstep as ss;

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    pub use placers_kit::paint::opaque;

    pub use placers_kit::paint::fill;

    pub use placers_kit::noise::crack;

    /// Wood seen along its grain (u along it): the growth rings' streaks and
    /// the figure, `base` its colour (sRGB)
    pub fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
        let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
        let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
        let streak = fbm(seed, u * along * 0.2, v * across, 3);
        let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
        base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
    }

    /// Tarnished metal's mottle (a multiplier near 1)
    pub fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
        0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * ss(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
    }
}
