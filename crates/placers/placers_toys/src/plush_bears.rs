//! `place_plush_bears`: Plush teddy: 6% chance per level. A machine-made plush teddy of the 1980s-90s, 30 cm standing (acrylic pile on a knit backing, polyester fibre stuffing, sewn to sit; a velour muzzle, inner ears and soles; plastic safety eyes and a moulded nose, the mouth stitched; a satin ribbon in a bow at its neck), slumped where it was left, its head fallen forward. Its pile matted and faded, grimed, the ribbon gone dull. 30%: an eye pulled out, the hole where it went through left.
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::matted;
use placers_kit::noise::mix;
use placers_kit::noise::sstep as ss;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

/// Plush teddy: 6% chance per level. A machine-made plush teddy of the
/// 1980s-90s, 30 cm standing (acrylic pile on a knit backing, polyester
/// fibre stuffing, sewn to sit; a velour muzzle, inner ears and soles;
/// plastic safety eyes and a moulded nose, the mouth stitched; a satin
/// ribbon in a bow at its neck), slumped where it was left, its head fallen
/// forward. Its pile matted and faded, grimed, the ribbon gone dull.
/// 30%: an eye pulled out, the hole where it went through left.
pub(crate) fn place_plush_bears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let col = pick(&PLUSH_COLORS);
    let k = PLUSH_COLORS.iter().position(|&c| c == col).unwrap_or(0);
    // 30% chance: only 1 eye; the old kept the one at -x (its right)
    // when its second draw fell under a half
    let missing_eye = rnd() < 0.30;
    let keep_right = if missing_eye { Some(rnd() < 0.5) } else { None };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let s = salt(px, pz, 411);
    let bear = group();
    let base = rgb(PLUSH[k]);
    put(&bear, "Real__plushBearFur", &lambert!({ map: &plush_tex(s, base) }));
    put(&bear, "Real__plushBearPile", &lambert!({ map: &plush_pile_tex(s, base), alphaTest: 0.5 }));
    // the velour: a cream, or a tan on the pale bear
    let velour = if k == 3 { [0.70, 0.60, 0.48] } else { [0.76, 0.68, 0.57] };
    put(&bear, "Real__plushBearAccent", &lambert!({ map: &velour_tex(s, velour) }));
    // safety eyes and the moulded nose: black plastic, scuffed dull
    let plastic = phong!({ color: 0x1c1b1a, specular: 0x484848, shininess: 60 });
    if keep_right != Some(false) {
        put(&bear, "Real__plushBearEyeR", &plastic);
    }
    if keep_right != Some(true) {
        put(&bear, "Real__plushBearEyeL", &plastic);
    }
    put(&bear, "Real__plushBearEyeHoles", &lambert!({ color: 0x1c1512 }));
    put(&bear, "Real__plushBearNose", &phong!({ color: 0x1c1a19, specular: 0x303030, shininess: 30 }));
    put(&bear, "Real__plushBearMouth", &lambert!({ color: 0x1d1a18 }));
    // satin ribbon gone dull: a faded red or a faded blue, by where it lies
    let ribbon = if hash(s, 3, 1) < 0.6 { 0x7e3f3a } else { 0x4b5a6e };
    put(&bear, "Real__plushBearBow", &phong!({ color: ribbon, specular: 0x2a2826, shininess: 22 }));
    bear.position().set(px, 0, pz);
    bear.rotation().y = turn;
    g.world.add(&bear);
}

/// The old colours, and the plush each stands for (sRGB): a mid brown, a
/// caramel, a chocolate, a honey
const PLUSH_COLORS: [u32; 4] = [0x6a4020, 0x8a5028, 0x3a2810, 0xa08050];

const PLUSH: [u32; 4] = [0x6c4a33, 0x8a6446, 0x3e2e23, 0xa18565];

/// Acrylic plush pile (`b`): short, laid in fine streaks, matted into felted
/// patches where it was hugged and washed, faded, grime and an old stain
fn plush_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 140.0, v * 70.0, 3);
            let mut c = mix(b, [b[0] * 0.66, b[1] * 0.64, b[2] * 0.62], ss(0.42, 0.72, streak) * 0.45);
            let tip = vn(seed ^ 2, u * 220.0, v * 180.0);
            c = mix(c, [b[0] * 1.18 + 0.03, b[1] * 1.16 + 0.03, b[2] * 1.14 + 0.03], ss(0.7, 0.95, tip) * 0.4);
            // matted: flattened pile, darker and smoother in patches
            let mat = matted(seed, u, v);
            c = mix(c, [b[0] * 0.78, b[1] * 0.76, b[2] * 0.74], mat * 0.6);
            let fade = fbm(seed ^ 3, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.60, 0.57, 0.52], ss(0.5, 0.8, fade) * 0.25);
            let grime = fbm(seed ^ 6, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.32, 0.28, 0.24], ss(0.6, 0.85, grime) * 0.35);
            // an old stain, dried at its edge
            let (du, dv) = (u - 0.3 - 0.4 * hash(seed, 1, 1), v - 0.3 - 0.3 * hash(seed, 2, 2));
            let d = (du * du + dv * dv).sqrt() / 0.07;
            c = mix(
                c,
                [c[0] * 0.8, c[1] * 0.74, c[2] * 0.66],
                (ss(1.0, 0.85, d) * 0.4 + ss(0.75, 0.95, d) * ss(1.05, 0.95, d) * 0.4).min(0.6),
            );
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The pile's tips (the stippled layer over the plush): a third of the
/// fibres' ends, none where it is matted flat
fn plush_pile_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let n = vn(seed ^ 0x31, u * 170.0, v * 150.0);
            let m = fbm(seed ^ 0x32, u * 20.0, v * 20.0, 2);
            let a = if n > 0.6 - 0.12 * (m - 0.5) + matted(seed, u, v) * 0.6 { 1.0 } else { 0.0 };
            let c = mix([b[0] * 1.1 + 0.02, b[1] * 1.08 + 0.02, b[2] * 1.06 + 0.02], [0.55, 0.52, 0.47], 0.12 + 0.2 * m);
            [c[0], c[1], c[2], a]
        });
    })
}

/// Velour (`b`): short and flat, its nap catching the light in patches,
/// grimed most where it was touched (the muzzle) and stood on (the soles)
fn velour_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x7e1;
        fill(ctx, w, h, |u, v| {
            let nap = fbm(seed, u * 9.0, v * 9.0, 3);
            let mut c = mix(b, [b[0] * 0.82, b[1] * 0.8, b[2] * 0.78], ss(0.4, 0.7, nap) * 0.6);
            let grime = fbm(seed ^ 1, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.38, 0.33, 0.28], ss(0.5, 0.8, grime) * 0.45);
            let speck = vn(seed ^ 2, u * 70.0, v * 70.0);
            if speck > 0.88 {
                c = mix(c, [0.30, 0.26, 0.22], (speck - 0.88) * 4.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
