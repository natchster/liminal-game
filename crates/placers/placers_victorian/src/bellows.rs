//! `place_bellows`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::crack;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::vary;

// Hearth bellows: 4% chance per level. An English fireside bellows
// (Blender batch `t20`): two oak boards, the top one carved with a roundel,
// joined by a gusset of leather nailed round their edges with brass nails,
// a turned brass nozzle in the nose block. The oak gone dark, rubbed pale
// at the handles; the leather dried and cracked, its pleats dusty; the
// brass gone brown.
pub(crate) fn place_bellows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old draws, call for call: where it lies and how it is turned
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let bel = group();
    bel.position().set(px, 0, pz);
    bel.rotation().y = turn;
    let vary = vary(px, pz, turn);
    put(&bel, "Real__bellowsBoards", &phong!({ map: &bw_oak_tex(vary), specular: 0x141210, shininess: 12 }));
    put(&bel, "Real__bellowsLeather", &lambert!({ map: &bw_leather_tex(vary) }));
    put(&bel, "Real__bellowsTacks", &phong!({ color: 0x6a5530, specular: 0x4a3c22, shininess: 30 }));
    put(&bel, "Real__bellowsNozzle", &phong!({ map: &bw_brass_tex(vary), specular: 0x5a4a2a, shininess: 36 }));
    g.world.add(&bel);
}

/// English oak gone dark under old wax, as the boards are mapped (u along
/// them, nose to handle; v across): its grain and the silver flecks of its
/// rays, the handles rubbed pale (u > 0.72), dust and soot in the grain
fn bw_oak_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, u, v, [0.32, 0.21, 0.12], 12.0, 26.0);
            // the rays: short pale flecks across the grain
            let ray = sstep(0.78, 0.9, vn(seed ^ 7, u * 60.0, v * 9.0)) * 0.6;
            o = mix(o, [0.36, 0.25, 0.14], ray * 0.5);
            // rubbed at the handles by hands, the wax worn through
            let rub = sstep(0.70, 0.85, u) * (0.5 + 0.5 * fbm(seed ^ 3, u * 20.0, v * 8.0, 2));
            o = mix(o, [0.40, 0.29, 0.17], rub * 0.45);
            // soot from the grate near the nose, dust over all of it
            o = o.map(|c| c * (1.0 - 0.35 * (1.0 - sstep(0.0, 0.18, u))));
            let dust = 0.06 + 0.12 * fbm(seed ^ 5, u * 9.0, v * 5.0, 3);
            opaque(mix(o, [0.42, 0.39, 0.34], dust))
        });
    })
}

/// Old leather dried out: dark brown, crazed into small islands that have
/// gone pale and grey at their edges, scuffed, dust settled in its folds
fn bw_leather_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x1ea;
        fill(ctx, w, h, |u, v| {
            let g = fbm(seed, u * 10.0, v * 6.0, 4);
            let mut o = [0.19, 0.115, 0.07].map(|c| c * (0.75 + 0.5 * g));
            // crazed: the cracks dark, the islands' lips gone pale
            let d = crack(seed ^ 3, u * 70.0, v * 34.0);
            let dry = sstep(0.45, 0.75, fbm(seed ^ 11, u * 4.0, v * 3.0, 3));
            o = mix(o, [0.36, 0.30, 0.25], dry * sstep(0.10, 0.02, d) * 0.6);
            o = o.map(|c| c * (1.0 - 0.55 * (1.0 - sstep(0.0, 0.035, d)) * (0.4 + 0.6 * dry)));
            // scuffs where it was pumped
            let scuff = sstep(0.82, 0.92, vn(seed ^ 5, u * 30.0, v * 4.0));
            o = mix(o, [0.33, 0.24, 0.17], scuff * 0.5);
            let dust = 0.10 + 0.14 * fbm(seed ^ 9, u * 6.0, v * 6.0, 3);
            opaque(mix(o, [0.44, 0.41, 0.36], dust))
        });
    })
}

/// Brass gone brown and dull, its tarnish mottled, rubbed brighter on its
/// rings, a little green where the damp sat
fn bw_brass_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0xb5;
        fill(ctx, w, h, |u, v| {
            let k = tarnish(seed, u, v, 6.0);
            let mut o = [0.40, 0.31, 0.16].map(|c| c * k);
            let verd = sstep(0.68, 0.82, fbm(seed ^ 4, u * 9.0, v * 9.0, 3));
            o = mix(o, [0.27, 0.32, 0.22], verd * 0.5);
            opaque(o)
        });
    })
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Wood seen along its grain (u along it): the growth rings' streaks and
/// the figure, `base` its colour (sRGB)
fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
    let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
    let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
    let streak = fbm(seed, u * along * 0.2, v * across, 3);
    let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
    base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
}

/// Tarnished metal's mottle (a multiplier near 1)
fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
    0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * sstep(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
}
