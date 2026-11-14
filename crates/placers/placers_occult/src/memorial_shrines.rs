//! `place_memorial_shrines`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::salt;

// Small memorial shrine (floor): 2.5% chance per level. A floor shrine
// someone made: a planed pine board laid on the floor, a small black
// hardback on it, and standing on the book on its easel strut a 5 x 7 in
// photograph in a wooden frame (an old portrait, faded); two or three 1/2 in
// chime candles burnt down in brass chime holders before it, some lit; a
// stoneware bud vase of dried rosebuds; half the time a small brass cross
// or a Star of David on its base; now and then a black ribbon bow on the
// frame's corner, its tail stirring.
pub(crate) fn place_memorial_shrines(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let d = draws(g, x, z);
    let salt = salt(d.px, d.pz, 0x5a1);
    let ms = group();
    // the board, stained by the old colour
    put(&ms, "Real__shrineBoard", &lambert!({ map: &board_tex(salt, d.wood) }));
    // the book for a riser, its spine to the back
    let book = group();
    book.position().set(0, BOARD_TOP, -0.03);
    book.rotation().y = PI;
    ms.add(&book);
    let cloth = lambert!({ map: &cloth_tex(salt) });
    put(&book, "Real__bookHbCBoards", &cloth);
    put(&book, "Real__bookHbCSpine", &cloth);
    put(&book, "Real__bookHbCPages", &lambert!({ color: 0xb4a684 }));
    // the photograph standing on the book, leaning back on its strut about
    // its back's bottom edge
    let vary = pictures::vary_of(d.px, d.pz, d.turn);
    let lean = group();
    lean.position().set(0, BOARD_TOP + BOOK_T, 0.010);
    lean.rotation().x = -LEAN;
    ms.add(&lean);
    let body = group();
    body.position().y = FRAME_H / 2.0;
    lean.add(&body);
    let frame_mat = if vary.is_multiple_of(3) {
        phong!({ map: &pictures::moulding_tex(FRAME_OUTER, &WOOD_FACE, Outline::Rect, Finish::Ebony, 1500.0, vary), specular: 0x161412, shininess: 30 })
    } else {
        lambert!({ map: &pictures::moulding_tex(FRAME_OUTER, &WOOD_FACE, Outline::Rect, Finish::Wood([0x5c, 0x3e, 0x28]), 1500.0, vary) })
    };
    let dirt = 0.25 + 0.25 * (vary % 97) as f32 / 97.0;
    let parts: [(&str, Mat); 4] = [
        ("Real__framedPhotoFrameWood", frame_mat),
        ("Real__framedPhotoGlass", pictures::glass_mat(vary, dirt)),
        ("Real__framedPhotoBack", lambert!({ color: 0x3a2e24 })),
        ("Real__shrineEasel", lambert!({ color: 0x1e1c1a })),
    ];
    for (name, mat) in &parts {
        if let Some(m) = put(&body, name, mat) {
            if name.ends_with("Glass") {
                m.set_render_order(2);
            }
        }
    }
    let photo = PHOTOS[((vary >> 3) % PHOTOS.len() as u32) as usize];
    let mut age = Age::photo(vary);
    age.border = if (vary >> 6).is_multiple_of(2) { 0.004 } else { 0.0 };
    let print = mesh(plane_geo!(PRINT.0, PRINT.1), lambert!({ map: &pictures::print_tex(photo, PRINT, 2400.0, age, vary) }));
    print.position().z = ZL - 0.0023;
    body.add(&print);
    // the candles, in a row before it
    let brass = phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x6a5430, shininess: 30 });
    let wax = lambert!({ map: &wax_tex(salt), color: d.wax });
    let n = d.candles.len();
    for (i, &(h, lit)) in d.candles.iter().enumerate() {
        let cx = (i as f64 - (n - 1) as f64 / 2.0) * 0.055;
        let cz = 0.068;
        // the old candle's height says how far this one has burnt
        let (k, top) = CANDLE_TOPS[if h < 0.06 {
            0
        } else if h < 0.07 {
            1
        } else {
            2
        }];
        let c = group();
        c.position().set(cx, BOARD_TOP, cz);
        c.rotation().y = f64::from((salt >> (i * 5)) % 64) * 0.1;
        ms.add(&c);
        put(&c, "Real__shrineChimeHolder", &brass);
        put(&c, &format!("Real__shrineChimeCandle{k}"), &wax);
        if let Some(m) = put(&c, "Real__shrineChimeWick", &lambert!({ color: 0x1c1915 })) {
            m.position().y = top - 0.0028;
        }
        // 50%: lit, as the old one was: its flame (t09's) and its light,
        // flickering as the old one flickered
        if let Some(phase) = lit {
            let root = top + 0.0005;
            let flame = group();
            flame.position().y = root;
            c.add(&flame);
            if let Some(m) = put(&flame, "Real__brassStickFlameA", &basic_mat!({ map: &flame_tex(), transparent: true, depthWrite: false }))
            {
                m.position().y = -FLAME_ROOT;
                m.set_render_order(3);
            }
            let flame_light = point_light!(0xffa040, 0.20, 0.8);
            flame_light.position().y = root + 0.010;
            c.add(&flame_light);
            g.animate(move |_g, _dt, t| {
                let flick = 0.85 + (t * 6.0 + phase).sin() * 0.12 + (t * 9.0 + phase * 1.4).sin() * 0.07;
                flame_light.set_intensity(0.20 * flick);
                flame.scale().set_scalar(flick);
            });
        }
    }
    // the bud vase and its dried rosebuds
    let (vx, vz) = (-0.1225, 0.03);
    if let Some(m) = put(&ms, "Real__shrineVase", &phong!({ map: &glaze_tex(salt), specular: 0x2a2c2e, shininess: 26 })) {
        m.position().set(vx, BOARD_TOP, vz);
    }
    let bud_mat = lambert!({ map: &rose_tex(salt) });
    for &(stem_color, len, stem_lean, foot_dz, ry) in &d.stems {
        // through the vase's mouth, leaning as the old stem leant, turned
        // about the vase by the old bud's turn; it stands in the vase as
        // deep as the old stem was short
        let stand = group();
        stand.position().set(vx, BOARD_TOP + VASE_MOUTH, vz + foot_dz);
        stand.rotation().y = ry;
        ms.add(&stand);
        let tilt = group();
        tilt.rotation().z = stem_lean;
        stand.add(&tilt);
        let vis = 0.050 + (len - 0.06) / 0.03 * 0.030;
        if let Some(m) = put(&tilt, "Real__shrineRoseStem", &lambert!({ color: stem_color })) {
            m.position().y = vis - ROSE_STEM;
        }
        if let Some(m) = put(&tilt, "Real__shrineRoseBud", &bud_mat) {
            m.position().y = vis - 0.0005;
            m.rotation().y = ry * 2.3;
        }
    }
    // 50%: a small brass cross (60%) or a Star of David on its base
    if let Some(is_cross) = d.symbol {
        let name = if is_cross { "Real__shrineCross" } else { "Real__shrineStar" };
        if let Some(m) = put(&ms, name, &brass) {
            m.position().set(0.1225, BOARD_TOP, 0.02);
            m.rotation().y = -0.25;
        }
    }
    // 40%: a black ribbon bow on the frame's top left corner, its long tail
    // stirring in the draught as the old ribbon's did (3 mm)
    if let Some((sway_freq, sway_phase)) = d.ribbon {
        let satin = phong!({ color: 0x1e1c1b, specular: 0x262626, shininess: 18, side: DoubleSide });
        let knot = group();
        knot.position().set(-FRAME_OUTER.0 / 2.0 + 0.008, FRAME_OUTER.1 / 2.0 - 0.010, 0.0198);
        body.add(&knot);
        put(&knot, "Real__shrineRibbonBow", &satin);
        if let Some(tail) = put(&knot, "Real__shrineRibbonTail", &satin) {
            g.animate(move |_g, _dt, t| {
                tail.position().x = (t * sway_freq + sway_phase).sin() * 0.003;
            });
        }
    }
    ms.position().set(d.px, 0, d.pz);
    ms.rotation().y = d.turn;
    g.world.add(&ms);
}

/// The board (`occult/shrine_and_seance_things.py` SB_*: its top), r02's small hardback (its thickness)
/// and t01's 5 x 7 in frame (its height, its outside, the print, the lip)
const BOARD_TOP: f64 = 0.019;

const BOOK_T: f64 = 0.0273;

const FRAME_H: f64 = 0.202;

const FRAME_OUTER: (f64, f64) = (0.151, 0.202);

const PRINT: (f64, f64) = (0.127, 0.178);

const ZL: f64 = 0.0060;

/// How far the frame leans back on its strut (`occult/shrine_and_seance_things.py` shrine_easel)
const LEAN: f64 = 14.0 * PI / 180.0;

/// t01's wooden moulding's face, outer edge to sight (d, z)
static WOOD_FACE: [(f64, f64); 9] = [
    (0.0, 0.0130),
    (0.0006, 0.0152),
    (0.0020, 0.0168),
    (0.0040, 0.0176),
    (0.0075, 0.0178),
    (0.0105, 0.0172),
    (0.0122, 0.0160),
    (0.0158, 0.0100),
    (0.016, 0.0094),
];

/// The photographs a family keeps (portraits, CC0)
const PHOTOS: [&str; 3] = ["graduate", "couple_reading", "baby_chair"];

/// The chime candles burnt down (`occult/shrine_and_seance_things.py` CH_TOPS: their tops above the
/// holder's foot, m) and t09's flame (its root in its own frame)
const CANDLE_TOPS: [(char, f64); 3] = [('C', 0.052), ('B', 0.066), ('A', 0.080)];

const FLAME_ROOT: f64 = 0.133 + 0.069;

/// The bud vase's mouth (`occult/shrine_and_seance_things.py` shrine_vase) and a rose's stem
const VASE_MOUTH: f64 = 0.057;

const ROSE_STEM: f64 = 0.095;

/// What the old placer drew, in its order
struct Draws {
    wood: u32,
    wax: u32,
    candles: Vec<(f64, Option<f64>)>,
    stems: Vec<(u32, f64, f64, f64, f64)>,
    symbol: Option<bool>,
    ribbon: Option<(f64, f64)>,
    px: f64,
    pz: f64,
    turn: f64,
}

/// The old placer's draws after its tile, in its order (its meshes are
/// gone; its draws are this shrine's variety)
fn draws(g: &Game, x: i32, z: i32) -> Draws {
    let wood_colors = [0x2a1810, 0x402015, 0x604020];
    let wood: u32 = choice(&wood_colors);
    let candle_count = 2 + (rnd() * 2.0).floor() as i32;
    let candle_colors = [0xf0e8c8, 0xd0d0d0, 0xe8d0c0]; // ivory, white, cream
    let wax: u32 = choice(&candle_colors);
    let mut candles = vec![];
    for _ in 0..candle_count {
        let h = 0.05 + rnd() * 0.03;
        let lit = if rnd() < 0.5 { Some(rnd() * PI * 2.0) } else { None };
        candles.push((h, lit));
    }
    let dry_stems = [0x606038, 0x704838, 0x504028]; // faded olive, dried brown, sepia
    let mut stems = vec![];
    for _ in 0..4 {
        let color: u32 = choice(&dry_stems);
        let len = 0.06 + rnd() * 0.03;
        let lean = (rnd() - 0.5) * 0.5;
        let foot_dz = (rnd() - 0.5) * 0.006;
        let ry = (rnd() - 0.5) * 6.0;
        stems.push((color, len, lean, foot_dz, ry));
    }
    let symbol = if rnd() < 0.5 { Some(rnd() < 0.6) } else { None };
    let ribbon = if rnd() < 0.4 {
        let sway_freq = 0.30 + rnd() * 0.25;
        let sway_phase = rnd() * PI * 2.0;
        Some((sway_freq, sway_phase))
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 1.8;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.8;
    let turn = rnd() * PI * 2.0;
    Draws { wood, wax, candles, stems, symbol, ribbon, px, pz, turn }
}

/// The board's pine (its top as laid: u along it), stained by the old
/// colour: the grain's late wood darker, a knot, the stain worn pale where
/// things stood and were moved, dust
fn board_tex(salt: u32, stain: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0d;
        let s = [f64::from((stain >> 16) & 0xff) / 255.0, f64::from((stain >> 8) & 0xff) / 255.0, f64::from(stain & 0xff) / 255.0];
        // the stain over pale pine: the old colour lifted toward wood
        let base = mix([0.62, 0.48, 0.32], [s[0] * 1.6 + 0.05, s[1] * 1.6 + 0.04, s[2] * 1.6 + 0.03], 0.7);
        let (kx, ky) = (0.2 + 0.6 * hash(seed, 1, 2), 0.3 + 0.4 * hash(seed, 3, 4));
        fill(ctx, w, h, |u, v| {
            let warp = fbm(seed, u * 3.0, v * 2.0, 2) * 0.6;
            let ring =
                ((v * 26.0 + warp * 6.0 + ((u - kx).powi(2) * 4.0 + (v - ky).powi(2) * 30.0).sqrt() * 8.0).sin() * 0.5 + 0.5).powi(3);
            let mut c = mix(base, [base[0] * 0.62, base[1] * 0.58, base[2] * 0.55], ring * 0.7);
            let knot = ((u - kx).powi(2) * 16.0 + (v - ky).powi(2) * 60.0).sqrt();
            c = mix(c, [0.16, 0.10, 0.06], (1.0 - sstep(0.04, 0.12, knot)) * 0.85);
            let worn = (fbm(seed ^ 3, u * 6.0, v * 3.0, 3) - 0.55).max(0.0) * 1.6;
            c = mix(c, [0.58, 0.47, 0.34], worn);
            c = mix(c, [0.46, 0.44, 0.40], 0.12 + 0.12 * fbm(seed ^ 5, u * 12.0, v * 6.0, 2));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The book's cloth: black buckram gone brown and grey, rubbed at its edges
fn cloth_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xc1f;
        fill(ctx, w, h, |u, v| {
            let weave = ((u * 180.0).sin() * (v * 180.0).sin()).abs() * 0.15;
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix([0.14, 0.12, 0.11], [0.20, 0.17, 0.15], n + weave);
            let edge = u.min(1.0 - u).min(v).min(1.0 - v);
            c = mix(c, [0.34, 0.30, 0.26], (1.0 - sstep(0.0, 0.06, edge)) * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Brass gone dull (grey on the brass's colour: how bright): brown tarnish
/// in patches, darker spots
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xb7d;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix([0.42, 0.38, 0.33], [0.85, 0.82, 0.76], sstep(0.4, 0.75, n));
            let spot = vn(seed ^ 3, u * 40.0, v * 40.0);
            if spot > 0.72 {
                c = mix(c, [0.26, 0.24, 0.21], (spot - 0.72) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The candles' wax (grey on the old colour; laid round each, v up): dust,
/// the runs a shade clearer, soot near the top
fn wax_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a6;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 12.0, 3);
            let mut c = mix([0.97, 0.96, 0.94], [0.88, 0.86, 0.83], n);
            c = mix(c, [0.45, 0.43, 0.40], sstep(0.82, 1.0, v) * 0.4);
            let dust = vn(seed ^ 5, u * 90.0, v * 90.0);
            if dust > 0.8 {
                c = mix(c, [0.70, 0.69, 0.66], (dust - 0.8) * 2.5);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A candle flame (t09's, laid round it, v up its 27 mm): a dim blue root,
/// the bright yellow-white body, orange toward its tip, soft at its edge
fn flame_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(16, 64, false, |ctx, w, h| {
                fill(ctx, w, h, |_u, v| {
                    let root = 1.0 - sstep(0.04, 0.2, v);
                    let tip = sstep(0.35, 1.0, v);
                    let c = mix([1.0, 0.86, 0.56], [1.0, 0.56, 0.22], tip);
                    let c = mix(c, [0.36, 0.40, 0.80], root);
                    let a = (0.85 - 0.5 * root) * (1.0 - 0.55 * sstep(0.7, 1.0, v));
                    [c[0], c[1], c[2], a]
                });
            })
        })
        .clone()
    })
}

/// The bud vase's salt glaze (laid round it, v up): a grey-blue orange-peel
/// glaze thinner and browner on its shoulder, the foot ring bare stoneware,
/// dust on its shoulder
fn glaze_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x61a;
        fill(ctx, w, h, |u, v| {
            let peel = vn(seed, u * 70.0, v * 50.0);
            let mut c = mix([0.30, 0.33, 0.36], [0.22, 0.25, 0.28], fbm(seed ^ 2, u * 6.0, v * 6.0, 3) * 0.7 + peel * 0.3);
            c = mix(c, [0.36, 0.30, 0.24], sstep(0.45, 0.75, v) * 0.35 * fbm(seed ^ 4, u * 4.0, v * 4.0, 2));
            c = mix(c, [0.48, 0.42, 0.34], 1.0 - sstep(0.03, 0.06, v));
            c = mix(c, [0.46, 0.44, 0.41], sstep(0.55, 0.8, v) * (1.0 - sstep(0.85, 0.95, v)) * 0.3);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A dried rosebud (box-mapped): a deep red gone to brown and almost black
/// in its folds, the sepals and hip a dried green-brown at its foot
fn rose_tex(salt: u32) -> Tex {
    canvas_tex_opt(32, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x205;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 8.0, 3);
            let mut c = mix([0.34, 0.10, 0.09], [0.16, 0.06, 0.05], n);
            c = mix(c, [0.24, 0.20, 0.12], 1.0 - sstep(0.12, 0.3, v));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
