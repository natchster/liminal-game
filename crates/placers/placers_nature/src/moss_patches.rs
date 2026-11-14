//! `place_moss_patches`
use crate::shared::floor_mark;
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_tex {
    ($w:expr_2021, $h:expr_2021, $repeat:expr_2021, $paint:expr_2021) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

// Moss patch: 5% chance per level. Moss grown on a floor kept damp
// (Blender family `bits_of_the_outside_indoors`): a felted mat painted on the floor, cushions of
// packed shoots rising out of it, here and there their setae and nodding
// capsules.
pub(crate) fn place_moss_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let moss_colors = [0x407030, 0x506040, 0x304018, 0x607040];
    let patch = group();
    // the old mat's colour: the mat's
    let base_color = choice(&moss_colors);
    let kb = moss_colors.iter().position(|&c| c == base_color).unwrap_or(0);
    let shoots = moss_shoots_tex();
    let setae = lambert!({ color: 0x5a3424 });
    let mut tints: [Option<Mat>; 4] = Default::default();
    // the old twelve tufts: each one's colour, size and where it is (now a cushion's)
    let mut first = true;
    for _i in 0..12 {
        let c = choice(&moss_colors);
        let kc = moss_colors.iter().position(|&m| m == c).unwrap_or(0);
        let bump_r = 0.008 + rnd() * 0.010;
        let a = rnd() * PI * 2.0;
        let r = rnd() * 0.10;
        let (name, nominal) = if bump_r < 0.0113 {
            ("Real__mossCushionC", 0.0095)
        } else if bump_r < 0.0147 {
            ("Real__mossCushionB", 0.013)
        } else {
            ("Real__mossCushionA", 0.0175)
        };
        let Some(geo) = part(name) else { continue };
        let tint = MOSS_RGB[kc];
        let mat = tints[kc]
            .get_or_insert_with(|| {
                // the shoots' texture is about (0.52, 0.55, 0.40): the tint brings it to the colour
                let t = (((tint[0] / 0.52 * 255.0).min(255.0) as u32) << 16)
                    | (((tint[1] / 0.55 * 255.0).min(255.0) as u32) << 8)
                    | (tint[2] / 0.40 * 255.0).min(255.0) as u32;
                lambert!({ map: &shoots, color: t })
            })
            .clone();
        let cushion = group();
        cushion.add(&mesh(geo, &mat));
        // the old tuft's colour: setae on the first and on the green ones, in fruit
        if kc == 0 || first {
            if let Some(sg) = part("Real__mossSetae") {
                let s = mesh(sg, &setae);
                s.position().y = 0.002;
                cushion.add(&s);
            }
            first = false;
        }
        cushion.scale().set_scalar((bump_r / nominal).clamp(0.85, 1.15));
        cushion.position().set(a.dcos() * r, 0, a.dsin() * r);
        cushion.rotation().y = a * 3.0;
        patch.add(&cushion);
    }
    let (px, pz) = jitter(g, x, z, 2.2);
    patch.position().set(px, 0, pz);
    patch.rotation().y = rnd() * PI * 2.0;
    let mat_tex = moss_mat_tex(MOSS_RGB[kb], salt_of(px, pz));
    patch.add(&floor_mark(&mat_tex, 0.36, 0.36, 0.0, 0.0, 0.0));
    g.world.add(&patch);
}

// =================================================================== moss
/// The moss colours the old placer chose among: green, olive, dark, dry
/// (sRGB, as the shoots' tips are)
const MOSS_RGB: [[f64; 3]; 4] = [[0.44, 0.54, 0.24], [0.50, 0.52, 0.28], [0.34, 0.42, 0.20], [0.58, 0.56, 0.36]];

/// A cushion's shoots seen close: tiny leaves packed into a fur, the tips
/// paler (silvery where they are dry), darker in the gaps between
fn moss_shoots_tex() -> Tex {
    cached_tex!(128, 128, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let fine = vn(0x3055, x / 1.6, y / 1.6);
            let mid = fbm(0x3056, x / 7.0, y / 7.0, 3);
            let k = 0.55 + 0.6 * fine * fine + 0.3 * (mid - 0.5);
            let tip = sstep(0.75, 0.95, fine);
            let c = mix_unclamped([0.62 * k, 0.66 * k, 0.48 * k], [0.86, 0.86, 0.76], 0.35 * tip);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The mat the cushions grow out of, painted on the floor (`salt`: where it
/// is): a felt of shoots, green at its heart, browner and patchy out to a
/// ragged edge where it thins into the floor's dust; silt dried round it
fn moss_mat_tex(base: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w - 0.5, y / h - 0.5);
            let r = u.dhypot(v) * 2.0;
            let a = v.datan2(u);
            let edge = 0.62 + 0.22 * (fbm(salt, a.dcos() * 2.0 + 5.0, a.dsin() * 2.0 + 5.0, 3) - 0.5) * 2.0;
            let rr = r + 0.35 * (fbm(salt ^ 0x5, u * 7.0, v * 7.0, 4) - 0.5);
            // out at its edge it breaks into small colonies, then single tufts
            let tufts = sstep(0.62, 0.7, vn(salt ^ 0x15, x / 4.0, y / 4.0)) * (1.0 - sstep(edge + 0.02, edge + 0.16, rr));
            let cover = (1.0 - sstep(edge - 0.1, edge + 0.0, rr)).max(tufts);
            // the shoots: each a speck, its tip catching the light
            let shoots = vn(salt ^ 0x9, x / 1.1, y / 1.1);
            let tips = sstep(0.62, 0.8, vn(salt ^ 0xa, x / 0.8, y / 0.8));
            // colonies, with bare floor between some of them
            let patch = fbm(salt ^ 0x3, u * 5.0, v * 5.0, 4);
            let mut c = mix_unclamped(base, [0.46, 0.42, 0.28], sstep(0.4, 0.95, r) * 0.6 + 0.3 * sstep(0.55, 0.75, patch));
            c = mix_unclamped([c[0] * 0.45, c[1] * 0.45, c[2] * 0.45], c, 0.3 + 0.7 * shoots);
            c = mix_unclamped(c, [0.78, 0.80, 0.62], 0.35 * tips);
            // the silt the damp brought, dried in a ring round it
            let silt = sstep(edge - 0.1, edge + 0.1, r) * (1.0 - sstep(edge + 0.1, edge + 0.32, r));
            let a_moss = cover * (0.7 + 0.3 * shoots) * sstep(0.32, 0.4, patch + 0.25 * (1.0 - r));
            let a_silt = 0.2 * silt;
            let al = a_moss + a_silt * (1.0 - a_moss);
            let col = if al > 0.0 { mix_unclamped([0.36, 0.32, 0.25], c, a_moss / al) } else { c };
            [col[0], col[1], col[2], al]
        });
    })
}
