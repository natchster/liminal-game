//! `place_small_paper_crowns`: Small paper crown: 8% chance per level. A party crown die-cut from foil card or printed card (Blender batch `t12`): 89 mm tall, eight points over a 30 mm band, a printed jewel in each point; fastened round by its tab and trodden (pressed oval, creased, three points bent over), or come unfastened and sprung open in a curl. The foil dulled, scuffed through to the card, cracked white where it bent; dust, grime at its foot.
use crate::shared::put_with;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

/// Small paper crown: 8% chance per level. A party crown die-cut from
/// foil card or printed card (Blender batch `t12`): 89 mm tall, eight points
/// over a 30 mm band, a printed jewel in each point; fastened round by its
/// tab and trodden (pressed oval, creased, three points bent over), or come
/// unfastened and sprung open in a curl. The foil dulled, scuffed through
/// to the card, cracked white where it bent; dust, grime at its foot.
pub(crate) fn place_small_paper_crowns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour: the crown's foil or card
    let crown_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let cc = pick(&crown_colors);
    let design = crown_colors.iter().position(|&c| c == cc).unwrap_or(0);
    let crown = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    crown.position().set(px, 0, pz);
    crown.rotation().y = rnd() * PI * 2.0;
    let salt = salt_of(px, pz);
    let kind = usize::from(salt % 5 < 2);
    let tex = crown_print_tex(design, kind, salt);
    // foil (rose, gold, pink) dark in its diffuse and coloured in the
    // highlight; or matt printed card
    let mat = match design {
        0 => phong!({ map: &tex, specular: 0x8a5a62, shininess: 30 }),
        2 => phong!({ map: &tex, specular: 0x9a8650, shininess: 30 }),
        5 => phong!({ map: &tex, specular: 0x8a6a7c, shininess: 30 }),
        _ => lambert!({ map: &tex }),
    };
    put_with(&crown, ["Real__paperCrown", "Real__paperCrownOpen"][kind], &mat, |gg| {
        let n = gg.position.len() / 3;
        let length = CR_C + CR_OV;
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let (nx, nz) = (f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 2]));
            // where it is along the card, and the card's own normal there
            let (u, ex, ez, sx, sz) = if kind == 0 {
                let th = (x / CR_AX).atan2(z / CR_AZ).rem_euclid(2.0 * PI);
                let (ex, ez) = (CR_AX * th.sin(), CR_AZ * th.cos());
                let (a, b) = (CR_AZ * th.sin(), CR_AX * th.cos());
                let l = a.hypot(b);
                (th / (2.0 * PI) * CR_C, ex, ez, a / l, b / l)
            } else {
                let span = length / CR_OPEN_R;
                let th0 = PI + (2.0 * PI - span) / 2.0;
                let th = x.atan2(z);
                let u = ((th - th0).rem_euclid(2.0 * PI) * CR_OPEN_R).min(length);
                let r = CR_OPEN_R * (1.0 + 0.10 * (2.0 * u / length - 1.0).abs().powi(3));
                (u, r * th.sin(), r * th.cos(), th.sin(), th.cos())
            };
            let off = (x - ex) * sx + (z - ez) * sz;
            // a bent point: how far up the card it was, unfolded
            let v = if y > CR_VF { CR_VF + (y - CR_VF).hypot(off) } else { y };
            let front = nx * sx + nz * sz > 0.0;
            gg.uv[i * 2] = (u / length) as f32;
            gg.uv[i * 2 + 1] = if front { (v / (CR_HB + CR_HP) * PRINT_V).min(PRINT_V - 0.002) } else { 0.97 } as f32;
        }
    });
    g.world.add(&crown);
}

// ------------------------------------------------------------- the crown
/// The crown's card (`t12.py` CR_*): its length round (fastened) and its
/// tab, the band and the points, how many, where a bent point folds
const CR_C: f64 = 0.530;

const CR_OV: f64 = 0.045;

const CR_HB: f64 = 0.030;

const CR_HP: f64 = 0.059;

const CR_NP: usize = 8;

const CR_VF: f64 = CR_HB + 0.004;

/// the fastened ring pressed oval (half axes along x and z); the open one's
/// curl
const CR_AX: f64 = 0.094;

const CR_AZ: f64 = 0.075;

const CR_OPEN_R: f64 = 0.130;

/// The points bent over in each (the print cracks at their folds), and the
/// ring's crease across its band (u along the card)
const CR_FOLDED: [&[usize]; 2] = [&[1, 4, 6], &[2, 5]];

const CR_KINK: f64 = 0.205;

/// The print's share of the texture; above it the grey back of the card
const PRINT_V: f64 = 0.92;

/// The crown's print along its card (u along it, v up) and its wear; the
/// back of the card above `PRINT_V`
fn crown_print_tex(design: usize, kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(1024, 192, false, move |ctx, w, h| {
        let length = CR_C + CR_OV;
        let pitch = (CR_C - CR_OV) / CR_NP as f64;
        let foil = matches!(design, 0 | 2 | 5);
        // ground, its printed edging, the jewels' inks
        let (ground, edge) = match design {
            0 => ([0.50, 0.24, 0.27], [0.62, 0.52, 0.30]),
            1 => ([0.30, 0.36, 0.52], [0.80, 0.68, 0.36]),
            2 => ([0.56, 0.45, 0.21], [0.36, 0.20, 0.16]),
            3 => ([0.33, 0.47, 0.32], [0.80, 0.68, 0.36]),
            4 => ([0.42, 0.29, 0.47], [0.80, 0.68, 0.36]),
            _ => ([0.58, 0.40, 0.48], [0.62, 0.62, 0.64]),
        };
        let jewels = [[0.55, 0.14, 0.16], [0.14, 0.42, 0.26], [0.16, 0.24, 0.52]];
        let card = [0.82, 0.80, 0.74];
        let folded = CR_FOLDED[kind];
        fill(ctx, w, h, |tu, tv| {
            if tv > PRINT_V {
                let n = fbm(salt ^ 0x51, tu * 40.0, tv * 8.0, 3);
                let c = mix([0.66, 0.64, 0.59], [0.55, 0.53, 0.48], n);
                return [c[0], c[1], c[2], 1.0];
            }
            let u = tu * length;
            let v = tv / PRINT_V * (CR_HB + CR_HP);
            let mut c = ground;
            // the card's outline here: its top edge
            let in_points = u > CR_OV && u < CR_C;
            let f = ((u - CR_OV) / pitch).rem_euclid(1.0);
            let top = if in_points { CR_HB + CR_HP * (1.0 - (2.0 * f - 1.0).abs()) } else { CR_HB };
            // printed edging: along the band's foot and top, and up each
            // point's sides
            let line = |d: f64, wd: f64| 1.0 - sstep(wd * 0.6, wd, d.abs());
            let mut e = line(v - 0.004, 0.0014).max(line(v - (CR_HB - 0.003), 0.0012));
            if in_points && v > CR_HB - 0.003 {
                let side = (top - v) * (pitch / 2.0) / CR_HP;
                e = e.max(line(side - 0.0035, 0.0012));
            }
            // a row of dots along the band
            let dk = (u / 0.012).floor();
            let dd = (u - (dk + 0.5) * 0.012).hypot(v - CR_HB * 0.5);
            e = e.max(1.0 - sstep(0.0016, 0.0022, dd));
            c = mix(c, edge, e);
            // a jewel in each point, a pearl at its tip
            if in_points {
                let k = ((u - CR_OV) / pitch).floor() as usize;
                let cu = CR_OV + (k as f64 + 0.5) * pitch;
                let (du, dv) = ((u - cu) / 0.0075, (v - (CR_HB + 0.014)) / 0.0105);
                let r = du.hypot(dv);
                if r < 1.25 {
                    let jw = jewels[k % 3];
                    let rim = 1.0 - sstep(1.0, 1.25, r);
                    c = mix(c, edge, rim);
                    let inner = 1.0 - sstep(0.85, 1.0, r);
                    // its facets printed: lighter up and left
                    let lit = (0.5 - 0.35 * du + 0.45 * dv).clamp(0.0, 1.0);
                    let jc = mix(jw, [0.85, 0.82, 0.78], 0.35 * lit * lit);
                    c = mix(c, jc, inner);
                }
                let pd = (u - cu).hypot(v - (CR_HB + CR_HP - 0.008));
                c = mix(c, [0.80, 0.78, 0.72], 1.0 - sstep(0.0026, 0.0034, pd));
            }
            // wear: foil dulled in patches, scuffed through to the card
            c = mix(c, [0.50, 0.48, 0.43], (0.08 + 0.14 * fbm(salt, u * 40.0, v * 40.0, 3)) * if foil { 1.0 } else { 0.6 });
            let sc = vn(salt ^ 3, u * 1100.0, v * 1100.0) * 0.55 + fbm(salt ^ 4, u * 140.0, v * 260.0, 2) * 0.45;
            if sc > 0.71 {
                c = mix(c, card, ((sc - 0.71) * 6.0).min(0.9));
            }
            // cracked white where it folded: a bent point's root, the crease
            let mut crack = 0.0;
            if in_points && folded.contains(&(((u - CR_OV) / pitch).floor() as usize)) {
                crack = (1.0 - sstep(0.0002, 0.0010, (v - CR_VF).abs())) * f64::from(u8::from(vn(salt ^ 5, u * 1500.0, 2.0) > 0.25));
            }
            if kind == 0 {
                crack = crack
                    .max((1.0 - sstep(0.0002, 0.0012, (u - CR_KINK).abs())) * f64::from(u8::from(vn(salt ^ 6, v * 1500.0, 3.0) > 0.3)));
            }
            c = mix(c, card, crack * 0.8);
            // grime at its foot, dust settled on it
            c = mix(c, [0.24, 0.21, 0.17], (1.0 - sstep(0.0, 0.012, v)) * 0.55);
            let dust = fbm(salt ^ 7, u * 16.0, v * 16.0, 4);
            c = mix(c, [0.58, 0.55, 0.50], 0.10 + 0.22 * sstep(0.35, 0.75, dust));
            [c[0], c[1], c[2], 1.0]
        });
    })
}
