//! `place_small_pumpkins`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small pumpkin: 5% chance per level. A small carving pumpkin (19 cm)
// carved into a jack-o'-lantern and left: gone soft and slumped, its top
// sunk, the lid fallen in askew, its skin dull, rotting brown in patches
// and furred with mould, the cut flesh dried brown. Six in ten still have a
// tea light burning in them (a flame: its light through the face); the
// rest, a tea light burnt out.
pub(crate) fn place_small_pumpkins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let is_lit = rnd() < 0.60;
    let pumpkin = group();
    if is_lit {
        let glow = point_light!(0xff9a40, 0.4, 0.8, 2);
        glow.position().set(PK_WICK.0, PK_WICK.1 + 0.02, PK_WICK.2);
        pumpkin.add(&glow);
        // the flame over the wick, turned to the eye
        let geo = plane_geo!(0.010, 0.026);
        geo.translate(0, 0.012, 0);
        let flame = mesh(geo, basic_mat!({ map: flame_tex(), transparent: true, opacity: 0.9, depthWrite: false }));
        flame.position().set(PK_WICK.0, PK_WICK.1, PK_WICK.2);
        pumpkin.add(&flame);
        // its flicker (the old draws: phase and rate)
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 4.0 + rnd() * 3.0;
        g.animate(move |g, _dt, t| {
            let wobble = (t * flicker_rate + flicker_phase).sin() * 0.5 + (t * flicker_rate * 1.7 + flicker_phase).sin() * 0.3;
            glow.set_intensity(0.4 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.18;
            let m = flame.matrix_world();
            let c = g.camera.pos();
            flame.look_at(c.x, m.e[13], c.z);
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 7);
    put(&pumpkin, "Real__pumpkinSkin", &phong!({ map: &pumpkin_skin_tex(salt), specular: 0x1c1a16, shininess: 12 }));
    put(&pumpkin, "Real__pumpkinFlesh", &lambert!({ map: &pumpkin_flesh_tex(salt) }));
    put(&pumpkin, "Real__pumpkinStem", &lambert!({ color: 0x6c6250 }));
    put(&pumpkin, "Real__pumpkinTealight", &phong!({ color: 0x8c8c8a, specular: 0x9a9a9a, shininess: 60 }));
    put(&pumpkin, "Real__pumpkinWax", &lambert!({ color: if is_lit { 0xd6cdb8 } else { 0xb8ae98 } }));
    pumpkin.position().set(px, 0, pz);
    pumpkin.rotation().y = turn;
    g.world.add(&pumpkin);
}

/// The tea light in the pumpkin (family `halloween_things`): its wick's tip
const PK_WICK: (f64, f64, f64) = (0.0032, 0.0095 + 0.0105, 0.004);

/// A carved pumpkin's skin gone soft (laid round it: u round, v from its
/// bottom up): a dull orange gone brown, darker down its ten grooves, soft
/// brown rot in patches (more on its sunken top and where it sits), white
/// and blue-green mould in colonies, wrinkled
fn pumpkin_skin_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 10.0, v * 5.0, 4);
            let mut c = mix([0.70, 0.42, 0.18], [0.60, 0.33, 0.14], n);
            // the grooves between the lobes
            let gr = ((u - 0.05) * 10.0 + 0.04 * vn(seed ^ 3, v * 8.0, 0.0)).rem_euclid(1.0);
            let groove = (1.0 - ((gr - 0.5).abs() * 2.0)).powi(3);
            c = mix(c, [0.42, 0.25, 0.12], (1.0 - groove) * 0.0 + (1.0 - ((gr.min(1.0 - gr)) / 0.08).min(1.0)) * 0.45);
            // the rot: soft brown patches, more on the top and the bottom
            let rot = fbm(seed ^ 5, u * 6.0, v * 4.0, 5) + 0.25 * sstep(0.7, 1.0, v) + 0.2 * (1.0 - sstep(0.0, 0.25, v));
            c = mix(c, [0.40, 0.25, 0.13], sstep(0.56, 0.72, rot) * 0.85);
            c = mix(c, [0.22, 0.15, 0.09], sstep(0.76, 0.9, rot) * 0.9);
            // wrinkles
            let wr = vn(seed ^ 7, u * 60.0, v * 140.0);
            c = mix(c, [0.30, 0.19, 0.10], sstep(0.75, 0.9, wr) * 0.35);
            // mould: white fur and blue-green colonies, on the rot
            let m = fbm(seed ^ 9, u * 40.0, v * 20.0, 3) + 0.25 * sstep(0.6, 0.8, rot);
            let fur = vn(seed ^ 13, u * 400.0, v * 200.0);
            c = mix(c, [0.70, 0.69, 0.63], sstep(0.80, 0.95, m) * (0.35 + 0.35 * fur));
            let bg = fbm(seed ^ 11, u * 50.0, v * 25.0, 3) + 0.25 * sstep(0.65, 0.85, rot);
            c = mix(c, [0.40, 0.47, 0.40], sstep(0.86, 0.97, bg) * 0.55);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cut flesh and the inside (laid round it): dried to a tan leather,
/// browned, the fibres' strings, mould on the cut faces
fn pumpkin_flesh_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xf1e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let mut c = mix([0.42, 0.29, 0.15], [0.28, 0.18, 0.09], n);
            let str_ = vn(seed ^ 3, u * 20.0, v * 120.0);
            c = mix(c, [0.20, 0.13, 0.07], sstep(0.7, 0.85, str_) * 0.5);
            let m = fbm(seed ^ 5, u * 30.0, v * 30.0, 3);
            c = mix(c, [0.62, 0.61, 0.56], sstep(0.76, 0.9, m) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A tea light's flame as it is seen, on a plane whose foot is the flame's
/// (as the haunted floors' candles draw theirs): a teardrop, yellow-white
/// through its middle, orange at its edge, blue at its foot, a faint glow
fn flame_tex() -> Tex {
    canvas_tex_opt(32, 64, false, move |ctx, w, h| {
        fill(ctx, w, h, |uu, v| {
            let u = uu * 2.0 - 1.0;
            let (v0, v1, half) = (0.07, 0.84, 0.46);
            let t = (v - v0) / (v1 - v0);
            let hw = if t <= 0.0 || t >= 1.0 {
                0.0
            } else if t < 0.3 {
                half * (t / 0.3).sqrt()
            } else {
                half * ((1.0 - t) / 0.7).powf(1.2)
            };
            let q = if hw > 1e-6 { u.abs() / hw } else { 9.0 };
            let body = 1.0 - sstep(0.55, 1.05, q);
            let edge = sstep(0.3, 1.0, q).max(sstep(0.55, 1.0, t));
            let (r, g, b) = (1.0, 0.9 - 0.32 * edge, 0.7 - 0.52 * edge);
            let foot = (1.0 - sstep(0.04, 0.24, t)) * (0.35 + 0.65 * sstep(0.2, 0.8, q));
            let dark = (1.0 - sstep(0.0, 0.3, t)) * (1.0 - sstep(0.0, 0.5, q)) * 0.6;
            let (r, g, b) = (r + (0.35 - r) * foot, g + (0.5 - g) * foot, b + (1.0 - b) * foot);
            let a = body * (1.0 - dark) * (1.0 - 0.45 * foot);
            let d = (q - 1.0).max(0.0) * hw.max(0.2) + (v0 - v).max(0.0) * 1.5 + (v - v1).max(0.0) * 1.2;
            let halo = 0.14 * (-(d / 0.22).powi(2)).exp();
            [r, g, b, a + halo * (1.0 - a)]
        });
    })
}
