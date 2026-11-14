//! `place_teddy_paws`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::salt;
use placers_kit::place::scatter;

// A teddy's arm: 6% chance per level. A plush teddy's arm (a 40 cm
// bear's: acrylic pile sewn round its edge, a velour pad on its paw) torn
// off at its shoulder seam, lying on its side: the plush ragged at the
// tear, the seam's threads broken, the polyester fibre stuffing pulled out
// of it, three tufts of it lying near. Matted, faded, grimed.
pub(crate) fn place_teddy_paws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let col = pick_hex(&FUR_COLORS);
    let k = FUR_COLORS.iter().position(|&c| c == col).unwrap_or(0);
    let s = salt(g.wx(x), g.wz(z), 515);
    let base = rgb(PLUSH[k]);
    let paw = group();
    let arm = group();
    put(&arm, "Real__teddyArmFur", &lambert!({ map: &plush_tex(s, base) }));
    put(&arm, "Real__teddyArmPile", &lambert!({ map: &plush_pile_tex(s, base), alphaTest: 0.5 }));
    put(&arm, "Real__teddyArmPad", &lambert!({ map: &velour_tex(s, [0.70, 0.62, 0.51]) }));
    let fibre = lambert!({ map: &stuffing_tex(s) });
    let wisps = lambert!({ map: &wisps_tex(s), alphaTest: 0.5 });
    put(&arm, "Real__teddyArmStuffing", &fibre);
    put(&arm, "Real__teddyArmStuffingWisps", &wisps);
    paw.add(&arm);
    // the stuffing's tufts (the old puffs' draws: their size and turn, and
    // where they lie, past the torn end)
    for _ in 0..3 {
        let (a, b) = (rnd(), rnd());
        let (tx, tz) = ((rnd() - 0.5) * 0.08, 0.04 + (rnd() - 0.5) * 0.04);
        let tuft = group();
        put(&tuft, "Real__teddyArmTuft", &fibre);
        put(&tuft, "Real__teddyArmTuftWisps", &wisps);
        tuft.scale().set_scalar(0.8 + 0.4 * a);
        tuft.rotation().y = b * PI * 2.0;
        tuft.position().set(-0.115 + tx * 0.9, 0, (tz - 0.04) * 3.0);
        paw.add(&tuft);
    }
    scatter(g, &paw, x, z, 2.4);
    // (the old tilt's draw: the arm turned a little from its tufts)
    arm.rotation().y = (rnd() - 0.5) * 0.4;
    g.world.add(&paw);
}

/// The old fur colours, and the plush each stands for (sRGB): a caramel, a
/// dark brown, a chocolate
const FUR_COLORS: [u32; 3] = [0x8a5030, 0x604020, 0x503020];

const PLUSH: [u32; 3] = [0x86603f, 0x5c4331, 0x4a3628];

/// Acrylic plush pile (`b`): short, in fine streaks, matted flat in
/// patches, faded, grimed
fn plush_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 70.0, v * 35.0, 3);
            let mut c = mix(b, [b[0] * 0.66, b[1] * 0.64, b[2] * 0.62], sstep(0.42, 0.72, streak) * 0.45);
            let tip = vn(seed ^ 2, u * 110.0, v * 90.0);
            c = mix(c, [b[0] * 1.18 + 0.03, b[1] * 1.16 + 0.03, b[2] * 1.14 + 0.03], sstep(0.7, 0.95, tip) * 0.4);
            c = mix(c, [b[0] * 0.78, b[1] * 0.76, b[2] * 0.74], matted(seed, u, v) * 0.6);
            let fade = fbm(seed ^ 3, u * 2.0, v * 2.0, 3);
            c = mix(c, [0.60, 0.57, 0.52], sstep(0.5, 0.8, fade) * 0.25);
            let grime = fbm(seed ^ 6, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.32, 0.28, 0.24], sstep(0.55, 0.85, grime) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Where the plush is matted flat (0 to 1)
fn matted(seed: u32, u: f64, v: f64) -> f64 {
    sstep(0.6, 0.72, fbm(seed ^ 4, u * 3.0, v * 3.0, 4))
}

/// The pile's tips (the stippled layer): a third of the fibres' ends, none
/// where it is matted flat
fn plush_pile_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let n = vn(seed ^ 0x31, u * 85.0, v * 75.0);
            let m = fbm(seed ^ 0x32, u * 10.0, v * 10.0, 2);
            let a = if n > 0.6 - 0.12 * (m - 0.5) + matted(seed, u, v) * 0.6 { 1.0 } else { 0.0 };
            let c = mix([b[0] * 1.1 + 0.02, b[1] * 1.08 + 0.02, b[2] * 1.06 + 0.02], [0.55, 0.52, 0.47], 0.12 + 0.2 * m);
            [c[0], c[1], c[2], a]
        });
    })
}

/// The paw pad's velour (`b`): its nap in patches, grimed
fn velour_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x7e1;
        fill(ctx, w, h, |u, v| {
            let nap = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix(b, [b[0] * 0.82, b[1] * 0.8, b[2] * 0.78], sstep(0.4, 0.7, nap) * 0.6);
            let grime = fbm(seed ^ 1, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.38, 0.33, 0.28], sstep(0.45, 0.8, grime) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Polyester fibre stuffing: white gone grey with the floor's dust, its
/// fibres in fine curled wisps, darker in its folds
fn stuffing_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5f1;
        fill(ctx, w, h, |u, v| {
            let b = [0.84, 0.83, 0.80];
            let wisp = fbm(seed, u * 40.0 + 6.0 * fbm(seed ^ 1, u * 6.0, v * 6.0, 2), v * 40.0, 3);
            let mut c = mix(b, [0.70, 0.68, 0.64], sstep(0.35, 0.7, wisp) * 0.5);
            let dust = fbm(seed ^ 2, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.55, 0.52, 0.47], sstep(0.55, 0.8, dust) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The stuffing's loose fibres (its stippled outer layer): wisps, most of
/// it open
fn wisps_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5f3;
        fill(ctx, w, h, |u, v| {
            let f = fbm(seed, u * 30.0 + 8.0 * fbm(seed ^ 1, u * 5.0, v * 5.0, 2), v * 90.0, 2);
            let a = if (f - 0.5).abs() < 0.06 { 1.0 } else { 0.0 };
            let c = mix([0.86, 0.85, 0.82], [0.62, 0.60, 0.56], vn(seed ^ 2, u * 8.0, v * 8.0) * 0.7);
            [c[0], c[1], c[2], a]
        });
    })
}

/// The part `name` of a Blender batch as modelled, added to `grp`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
