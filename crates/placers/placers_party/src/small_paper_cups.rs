//! `place_small_paper_cups`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Small paper cup: 12% chance per level. A 9 oz printed paper cup of
// the party kind (coated cupstock, 76 mm across its rolled rim, 89 mm
// tall), its print faded and scuffed, what was in it dried to a crust and
// a tide line inside, a run of it dried down the outside; standing where
// it was put down, two or three still nested as they came, squeezed in a
// fist, or knocked over and the drink dried into the floor at its mouth.
pub(crate) fn place_small_paper_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: which print (white with confetti, pink with
    // dots, blue striped, yellow with zigzags, green with dots, cream with
    // its greeting)
    let cup_colors = [0xf0ece4, 0xf0c0d0, 0xd0d8f0, 0xf0d8a0, 0xd8f0d0, 0xe8d8c8];
    let col = pick(&cup_colors);
    let look = cup_colors.iter().position(|&c| c == col).unwrap_or(0);
    // the old 25%: knocked over, lying on its side
    let tipped = rnd() < 0.25;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let yaw = rnd() * PI * 2.0;
    // what else: from where it is (no draw)
    let salt = salt(px, pz, 13);
    let how = if tipped {
        Cup::Lying
    } else {
        match salt % 6 {
            0..=2 => Cup::Standing,
            3 => Cup::Nested(2 + (salt >> 8) % 2),
            _ => Cup::Squeezed,
        }
    };
    let drink = ((salt >> 4) % 3) as usize;
    let used = !matches!(how, Cup::Nested(_));
    let fill = 0.040 + 0.022 * f64::from((salt >> 12) % 100) / 100.0;
    let mats = CupMats {
        outer: lambert!({ map: &print_tex(look, salt, used) }),
        inner: lambert!({ map: &inside_tex(drink, fill, salt, how) }),
        bottom: lambert!({ map: &bottom_tex(drink, salt, how) }),
        rim: lambert!({ color: 0xb4ae9f }),
    };
    let cup = group();
    match how {
        Cup::Squeezed => one_cup(&cup, "Real__paperCupSqueezed", &mats, None),
        Cup::Nested(n) => {
            // the inside of all but the top one is out of sight: the top
            // one's is clean, dusty
            for i in 0..n {
                let one = group();
                one.position().y = f64::from(i) * CUP_NEST;
                one.rotation().y = f64::from(i) * 1.9 + f64::from(salt % 7);
                one_cup(&one, "Real__paperCup", &mats, None);
                cup.add(&one);
            }
        }
        Cup::Standing => one_cup(&cup, "Real__paperCup", &mats, None),
        Cup::Lying => {
            // on its side, resting on the curl and the base's edge: its
            // mouth toward -x, the axis rising to it (the parts themselves
            // turned: the placement measures a turned part by its turned
            // box, whose corner would dip into the floor and lift it)
            let theta = CUP_CURL.1.datan2(CUP_CURL.0 - CUP_BASE);
            one_cup(&cup, "Real__paperCup", &mats, Some((theta, CUP_BASE * theta.dsin())));
            // what was in it ran out of the mouth and dried into the floor
            let stain = mesh(
                plane_geo!(0.20, 0.13),
                lambert!({
                    map: &spill_tex(drink, salt),
                    color: 0xffffff,
                    transparent: true,
                    depthWrite: false,
                    polygonOffset: true,
                    polygonOffsetFactor: -2,
                    polygonOffsetUnits: -2
                }),
            );
            stain.rotation().x = -PI / 2.0;
            stain.position().set(-0.135, 0.0012, 0.0);
            cup.add(&stain);
        }
    }
    cup.position().set(px, 0, pz);
    cup.rotation().y = yaw;
    g.world.add(&cup);
}

/// The cup as modelled (`party/a_party_tables_leavings.py`): the outside's height and half width
/// (its bounds), the inside of the wall's heights, the bottom's half width
/// and the floor's height
const CUP_OUT: (f64, f64) = (0.0870, 0.0352);

const CUP_IN: (f64, f64) = (0.0086, 0.088_025);

const CUP_BOTTOM: f64 = 0.026;

const CUP_FLOOR: f64 = 0.0066;

/// Lying on its side: the curl's outermost radius and height, and the
/// base's edge; it rests on both
const CUP_CURL: (f64, f64) = (0.037_65, 0.088_03);

const CUP_BASE: f64 = 0.0254;

/// How far a cup stands up out of the one it is nested in
const CUP_NEST: f64 = 0.011;

struct CupMats {
    outer: Mat,
    inner: Mat,
    bottom: Mat,
    rim: Mat,
}

/// A cup's four parts (`Real__paperCup…` or `Real__paperCupSqueezed…`),
/// upright or turned about z by `lie`'s angle and lifted by its height
fn one_cup(grp: &Obj, stem: &str, m: &CupMats, lie: Option<(f64, f64)>) {
    let geos = [
        (part_split(&format!("{stem}Outer")), &m.outer),
        (part(&format!("{stem}Inner")), &m.inner),
        (part(&format!("{stem}Rim")), &m.rim),
        (part_top(&format!("{stem}Bottom"), CUP_BOTTOM), &m.bottom),
    ];
    for (geo, mat) in geos {
        let Some(geo) = geo else { continue };
        let geo = match lie {
            Some((theta, lift)) => turned(&geo, theta, lift),
            None => geo,
        };
        grp.add(&mesh(geo, mat.clone()));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Cup {
    Standing,
    Nested(u32),
    Squeezed,
    Lying,
}

/// The drinks a party has (sRGB, 0..1): a cola, a red fruit punch, an orange
/// squash; dried, each goes darker and browner
const DRINK: [[f64; 3]; 3] = [[0.26, 0.14, 0.07], [0.42, 0.11, 0.10], [0.66, 0.38, 0.10]];

/// The board's white, gone dull
const BOARD: [f64; 3] = [0.83, 0.80, 0.74];

/// The printed outside, laid as `part_split` lays it: its wall round (u,
/// from the seam) and up (v below a half), the foot's underside above. The
/// print as the party cups are printed: a solid ground and a pattern in
/// white, or confetti on white, or a greeting; faded, rubbed off at the
/// foot and where it was held, dust, and a dried run of the drink down
/// from the rim.
fn print_tex(look: usize, salt: u32, used: bool) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x51c0;
        let mut r = SplitMix64::new(salt, 0x7c11);
        let drip: Vec<(f64, f64)> = (0..if used { 2 } else { 0 }).map(|_| (r.next(), 0.02 + 0.05 * r.next())).collect();
        let held = (r.next(), 0.035 + 0.02 * r.next());
        let around = 2.0 * PI * 0.031;
        fill(ctx, w, h, |u, v| {
            if v >= 0.5 {
                // the foot's underside: the board, floor grime
                let g = vn(seed, u * 30.0, v * 30.0);
                let c = mix([0.62, 0.59, 0.53], [0.40, 0.37, 0.32], 0.4 + 0.4 * g);
                return [c[0], c[1], c[2], 1.0];
            }
            let y = v * 2.0 * CUP_OUT.0;
            // across the seam the print does not meet itself
            let seam = if u < 0.0318 { 0.0024 } else { 0.0 };
            let rad = CUP_BASE + (CUP_OUT.1 - CUP_BASE) * y / CUP_OUT.0;
            let mut c = pattern(look, u * around + seam, u * 2.0 * PI * rad + seam, y, seed);
            // the margins: the print stops short of the foot and the curl
            let margin = sstep(0.0035, 0.0025, y) + sstep(0.0835, 0.0855, y);
            c = mix(c, BOARD, margin);
            // the seam's edge: a hair of shadow
            if (u - 0.0318).abs() < 0.0012 {
                c = mix(c, [0.30, 0.28, 0.25], 0.35);
            }
            // the ink faded
            c = mix(c, [0.86, 0.84, 0.79], 0.22);
            // rubbed where a hand held it, and at the foot
            let du = ((u - held.0 + 0.5).rem_euclid(1.0) - 0.5) * around;
            let rub = (1.0 - ((du / 0.020).dpowi(2) + ((y - held.1) / 0.016).dpowi(2))).max(0.0);
            let n = fbm(seed ^ 3, u * 40.0, v * 12.0, 3);
            c = mix(c, [0.76, 0.73, 0.67], rub * (0.3 + 0.5 * n));
            c = mix(c, [0.42, 0.39, 0.34], sstep(0.010, 0.0, y) * (0.35 + 0.35 * n));
            // the drink run down from the rim, dried
            for &(d0, len) in &drip {
                let du = ((u - d0 + 0.5).rem_euclid(1.0) - 0.5) * around + 0.0012 * (y * 300.0).dsin();
                let top = CUP_OUT.0 - 0.001;
                if y > top - len && y < top {
                    let wdt = 0.0022 * (1.0 + 0.5 * ((top - y) / len));
                    let k = sstep(wdt, 0.0, du.abs()) * (0.18 + 0.2 * sstep(top - len + 0.005, top - len, y));
                    c = mix(c, [0.40, 0.27, 0.16], k);
                }
            }
            // dust and grime in the board's grain
            let d = vn(seed ^ 9, u * 260.0, v * 90.0);
            c = mix(c, [0.58, 0.56, 0.51], 0.10 + 0.12 * fbm(seed ^ 5, u * 8.0, v * 4.0, 2) + 0.2 * sstep(0.75, 1.0, d));
            [c[0], c[1], c[2], 1.0]
        });
        if look == 5 {
            // the greeting, twice round, in a printer's script, faded and
            // rubbed (a pixel is 0.38 mm round and 0.68 mm up: the type
            // squeezed to stand true)
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            for k in 0..2 {
                let cx = (0.27 + 0.5 * f64::from(k)) * w;
                for (text, font, ink, vy) in [
                    ("Happy Birthday", "italic bold 34px Georgia", "rgba(150,62,58,0.70)", 0.56),
                    ("\u{2605}   \u{2605}   \u{2605}", "bold 18px Helvetica", "rgba(70,96,140,0.55)", 0.40),
                ] {
                    ctx.save();
                    ctx.translate(cx, h * (1.0 - 0.5 * vy));
                    ctx.scale(1, 0.38 / 0.68);
                    ctx.set_font(font);
                    ctx.set_fill_style(ink);
                    ctx.fill_text(text, 0, 0);
                    ctx.restore();
                }
            }
            // dust over the ink as over the rest
            ctx.set_fill_style("rgba(150,144,132,0.16)");
            ctx.fill_rect(0, h * 0.5, w, h * 0.5);
        }
    })
}

/// The print at (round the wall from the seam: `xa` as the blank's rays
/// run, at the wall's middle; `x` at this height; y up), before its wear
fn pattern(look: usize, xa: f64, x: f64, y: f64, seed: u32) -> [f64; 3] {
    let white = [0.90, 0.88, 0.84];
    match look {
        // confetti: short dashes and dots of four inks on white
        0 => {
            let cell = 0.0072;
            let (ci, cj) = ((x / cell).floor(), (y / cell).floor());
            let (fx, fy) = (x / cell - ci, y / cell - cj);
            let h = |k: u32| hash(seed ^ k, ci as i32, cj as i32);
            let (ox, oy, ang) = (0.3 + 0.4 * h(1), 0.3 + 0.4 * h(2), h(3) * PI);
            let (dx, dy) = ((fx - ox) * cell, (fy - oy) * cell);
            let along = dx * ang.dcos() + dy * ang.dsin();
            let across = -dx * ang.dsin() + dy * ang.dcos();
            let dot = h(5) < 0.3;
            let ink = if dot { (dx * dx + dy * dy).sqrt() < 0.0011 } else { along.abs() < 0.0016 && across.abs() < 0.0006 };
            if ink && h(6) < 0.8 {
                [[0.74, 0.30, 0.32], [0.30, 0.44, 0.68], [0.86, 0.70, 0.30], [0.38, 0.60, 0.42]][(h(4) * 4.0) as usize % 4]
            } else {
                white
            }
        }
        // a pink ground, white polka dots in a staggered grid
        1 => {
            let s = 0.0090;
            let row = (y / s).floor();
            let xo = if row as i64 % 2 == 0 { 0.0 } else { s / 2.0 };
            let (fx, fy) = (((x + xo) / s).fract() - 0.5, (y / s).fract() - 0.5);
            if (fx * fx + fy * fy).sqrt() * s < 0.0022 { white } else { [0.84, 0.60, 0.64] }
        }
        // blue, white stripes up it
        2 => {
            if (xa / 0.012).fract() < 0.36 {
                white
            } else {
                [0.52, 0.63, 0.80]
            }
        }
        // yellow, white zigzags round it
        3 => {
            let p = 0.010;
            let zz = ((xa / p).fract() - 0.5).abs() * 0.007;
            if ((y + zz) / 0.016).fract() < 0.22 { white } else { [0.90, 0.77, 0.44] }
        }
        // green, small white dots, a white band near the rim
        4 => {
            let s = 0.0062;
            let (fx, fy) = ((x / s).fract() - 0.5, (y / s).fract() - 0.5);
            if (fx * fx + fy * fy).sqrt() * s < 0.0012 || (y > 0.072 && y < 0.077) { white } else { [0.62, 0.77, 0.58] }
        }
        // cream, its greeting drawn on after; a red band at the rim
        _ => {
            if y > 0.0765 && y < 0.0805 {
                [0.66, 0.30, 0.28]
            } else {
                [0.90, 0.84, 0.72]
            }
        }
    }
}

/// The inside of the wall (round from the seam, and up): white coated
/// board, dusty; drunk from: the drink's film dried down the wall from
/// where it stood, its tide lines, darker low; a lip's grime under the rim
/// (lying: dried along the side it lay on, out to the mouth)
fn inside_tex(drink: usize, level: f64, salt: u32, how: Cup) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x1a5e;
        let dc = DRINK[drink];
        let used = !matches!(how, Cup::Nested(_));
        let lying = how == Cup::Lying;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 30.0, v * 12.0, 3);
            let mut c = mix(BOARD, [0.74, 0.71, 0.65], 0.3 * n);
            let y = CUP_IN.0 + v * (CUP_IN.1 - CUP_IN.0);
            if used && !lying {
                // the film from where it stood down, darker low
                let below = sstep(level + 0.0008, level - 0.0008, y);
                let k = below * (0.22 + 0.30 * sstep(level, CUP_FLOOR, y)) * (0.75 + 0.25 * n);
                c = mix(c, dc, k);
                // the tide lines: where it stood, and where it stood for a
                // while as it went down
                for (i, step) in [0.0, 0.006, 0.013].iter().enumerate() {
                    let i = i as f64;
                    let ty = level - step + 0.0006 * (u * PI * 6.0 + i).dsin();
                    c = mix(c, mix(dc, [0.0, 0.0, 0.0], 0.3), sstep(0.0007, 0.0, (y - ty).abs()) * (0.55 - 0.15 * i));
                }
            }
            if lying {
                // along the side it lay on (-x: u = 3/4), out to the mouth
                let du = (u - 0.75).abs();
                c = mix(c, dc, sstep(0.09 + 0.02 * n, 0.05, du) * 0.5);
                c = mix(c, mix(dc, [0.0, 0.0, 0.0], 0.3), sstep(0.006, 0.0, (du - 0.07 - 0.02 * n).abs()) * 0.5);
            }
            // dust settled low; grime under the rim where lips were
            c = mix(c, [0.60, 0.58, 0.53], 0.12 * sstep(0.05, 0.01, y));
            if used {
                c = mix(c, [0.62, 0.55, 0.48], 0.3 * sstep(0.080, 0.087, y) * n);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The floor inside, from above (u along x, v against z; the bottom
/// disc's underside sees the same): drunk from, a dark crust crackled
/// into flakes, the board showing between; lying, a stain toward the side
/// it lay on; dust in it
fn bottom_tex(drink: usize, salt: u32, how: Cup) -> Tex {
    canvas_tex_opt(96, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0xb077;
        let dc = DRINK[drink];
        fill(ctx, w, h, |u, v| {
            let (x, z) = ((u * 2.0 - 1.0) * CUP_BOTTOM, ((1.0 - v) * 2.0 - 1.0) * CUP_BOTTOM);
            let n = fbm(seed, u * 8.0, v * 8.0, 3);
            let mut c = mix([0.88, 0.86, 0.80], [0.78, 0.75, 0.69], 0.3 * n);
            match how {
                Cup::Standing | Cup::Squeezed => {
                    let cr = (1.0 - ((vn(seed ^ 4, x * 700.0, z * 700.0) - 0.5).abs() * 9.0).min(1.0)).dpowi(2);
                    c = mix(c, mix(dc, [0.0, 0.0, 0.0], 0.45), 0.85 * (0.8 + 0.2 * n));
                    c = mix(c, mix(dc, BOARD, 0.5), cr * 0.6);
                }
                Cup::Lying => c = mix(c, dc, sstep(0.006, -0.008, x) * 0.5),
                Cup::Nested(_) => {}
            }
            // the dust that settled in it
            c = mix(c, [0.58, 0.55, 0.50], 0.22 + 0.2 * vn(seed ^ 6, x * 400.0, z * 400.0));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// What ran out of a knocked-over cup and dried into the floor: a tide-lined
/// stain, darkest at its edge, from the mouth outward (+x of the plane is
/// toward the cup)
fn spill_tex(drink: usize, salt: u32) -> Tex {
    canvas_tex_opt(160, 104, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b1;
        let dc = DRINK[drink];
        fill(ctx, w, h, |u, v| {
            // its shape: fed from the mouth (u = 1), spread out and ragged
            let (x, y) = (1.0 - u, v - 0.5);
            let edge = 0.30 + 0.14 * sstep(0.0, 0.7, x) + 0.10 * (fbm(seed, u * 5.0, v * 5.0, 3) - 0.5);
            let reach = 0.85 + 0.1 * (fbm(seed ^ 2, v * 4.0, 0.3, 2) - 0.5);
            // a soft round end, not a box's
            let d = ((y / edge).dpowi(2) + (x / reach).dpowi(4)).sqrt();
            if d >= 1.0 {
                return [dc[0], dc[1], dc[2], 0.0];
            }
            let tide = sstep(0.72, 0.92, d) * (1.0 - sstep(0.92, 1.0, d));
            let a = 0.10 + 0.22 * tide + 0.08 * fbm(seed ^ 3, u * 14.0, v * 14.0, 2);
            let c = mix(mix(dc, [0.12, 0.09, 0.06], 0.45), [0.0, 0.0, 0.0], 0.3 * tide);
            [c[0], c[1], c[2], a * sstep(1.0, 0.9, d) * sstep(0.0, 0.06, x)]
        });
    })
}

/// A copy of `geo` turned about z by `theta` and lifted by `lift`, its
/// texture coordinates kept and its bounds its own
fn turned(geo: &Geo, theta: f64, lift: f64) -> Geo {
    let out = geo.duplicate();
    {
        let mut gg = out.g();
        let (s, c) = (theta.dsin() as f32, theta.dcos() as f32);
        let turn = |arr: &mut Vec<f32>, dy: f32| {
            for p in arr.chunks_mut(3) {
                let (x, y) = (p[0], p[1]);
                p[0] = x * c - y * s;
                p[1] = x * s + y * c + dy;
            }
        };
        turn(&mut gg.position, lift as f32);
        turn(&mut gg.normal, 0.0);
        gg.compute_bounding_box();
        gg.compute_bounding_sphere();
        gg.touch();
    }
    out
}

/// The same, its texture coordinates laid from above at its real scale
/// (u along x, v against z, `half` m from its middle to the map's edge)
fn part_top(name: &str, half: f64) -> Option<Geo> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        for i in 0..n {
            let (x, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
            gg.uv[i * 2] = ((x / half + 1.0) * 0.5) as f32;
            gg.uv[i * 2 + 1] = (1.0 - (z / half + 1.0) * 0.5) as f32;
        }
        gg.touch();
    }
    Some(geo)
}

/// The same, its texture coordinates folded so its sides and its tops
/// each get their own place on one map: a triangle that mostly looks up
/// or down takes the disc the game lays on a cylinder's caps (from
/// above), folded into the upper left quarter; any other the round and
/// up of its sides, folded into the lower half. A corner the two kinds
/// share is doubled, so no triangle runs across the map from one to
/// the other.
fn part_split(name: &str) -> Option<Geo> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let idx = gg.index.clone()?;
        let n = gg.position.len() / 3;
        let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
        for i in 0..n {
            for k in 0..3 {
                let p = f64::from(gg.position[i * 3 + k]);
                lo[k] = lo[k].min(p);
                hi[k] = hi[k].max(p);
            }
        }
        let ex = [0, 1, 2].map(|k| if hi[k] - lo[k] > 1e-9 { hi[k] - lo[k] } else { 1.0 });
        let ny = |i: usize| gg.normal[i * 3 + 1];
        let cap = |i: usize| ny(i).abs() > 0.75;
        let (mut pos, mut nrm) = (gg.position.clone(), gg.normal.clone());
        let mut uv: Vec<f32> = Vec::with_capacity(gg.uv.len());
        for i in 0..n {
            let (u, v) = (gg.uv[i * 2], gg.uv[i * 2 + 1]);
            if cap(i) {
                uv.extend_from_slice(&[u * 0.5, 0.5 + v * 0.5]);
            } else {
                uv.extend_from_slice(&[u, v * 0.5]);
            }
        }
        let mut out = Vec::with_capacity(idx.len());
        let mut dup: std::collections::HashMap<(u32, u8), u32> = std::collections::HashMap::new();
        for t in idx.chunks(3) {
            let tri_cap = t.iter().map(|&i| ny(i as usize).abs()).sum::<f32>() / 3.0 > 0.6;
            let up = t.iter().map(|&i| ny(i as usize)).sum::<f32>() >= 0.0;
            let side_u = t.iter().map(|&i| i as usize).find(|&i| !cap(i)).map(|i| uv[i * 2]);
            for &i in t {
                let iu = i as usize;
                if cap(iu) == tri_cap {
                    out.push(i);
                    continue;
                }
                let f = [0, 1, 2].map(|k| (f64::from(gg.position[iu * 3 + k]) - lo[k]) / ex[k]);
                let (u, v, key) = if tri_cap {
                    (f[2] * 0.5, 0.5 + if up { f[0] } else { 1.0 - f[0] } * 0.5, u8::from(up))
                } else {
                    let mut u = ((f[0] * 2.0 - 1.0).datan2(f[2] * 2.0 - 1.0) / (2.0 * PI) + 1.0) % 1.0;
                    let mut key = 2;
                    if let Some(m) = side_u.map(f64::from) {
                        if u - m > 0.5 {
                            u -= 1.0;
                            key = 3;
                        } else if m - u > 0.5 {
                            u += 1.0;
                            key = 4;
                        }
                    }
                    (u, f[1] * 0.5, key)
                };
                let d = *dup.entry((i, key)).or_insert_with(|| {
                    pos.extend_from_slice(&[gg.position[iu * 3], gg.position[iu * 3 + 1], gg.position[iu * 3 + 2]]);
                    nrm.extend_from_slice(&[gg.normal[iu * 3], gg.normal[iu * 3 + 1], gg.normal[iu * 3 + 2]]);
                    uv.extend_from_slice(&[u as f32, v as f32]);
                    (pos.len() / 3 - 1) as u32
                });
                out.push(d);
            }
        }
        gg.position = pos;
        gg.normal = nrm;
        gg.uv = uv;
        gg.index = Some(out);
        gg.touch();
    }
    Some(geo)
}
