//! Batch `t19`'s carved armorial shield and roe skull, made real
//! (`docs/PROPS.md` P3; the Blender batch `assets/src/blender/props/t19.py`,
//! its notes `t19.notes.md`): a carved oak shield of arms of no real family,
//! its field painted and its ordinary carved proud and painted, the bead
//! round it gilt; a roe's skull boiled and bleached on a carved oak shield
//! or an oval board (a buck's with its antlers, batch `t11`'s; a doe's
//! without), a brass plate under it. Each keeps the old placer's draws,
//! call for call, and hangs where the old one hung.
use crate::{mount_on_wall, wall_candidates};
use game::prelude::*;
use t19_kit::*;

/// The burrs' bases on the skull plaque (`t19.py` `SP_O`), and the antlers'
/// turn about x there: the skull's (its frontal to the room, 90 degrees)
/// less the antlers' lean off the frontal (52 degrees), as batch `t11`'s
const SKULL_ANTLERS: [f64; 3] = [0.0, 0.072, 0.0883];
const SKULL_ANTLER_TURN: f64 = 0.663_225;

/// The crest's outline (`t19.py` `crest_outline`): half its width, its top
/// and point, its top corners' radius; the arcs' radius and centre
const CREST_HW: f64 = 0.168;
const CREST_TOP: f64 = 0.208;
const CREST_RC: f64 = 0.022;
const CREST_R: f64 = 0.212_762;
const CREST_CX: f64 = CREST_HW - CREST_R;
/// The field part's bounds (x, y): its texture is laid across them
const FIELD_X: (f64, f64) = (-0.1490, 0.1490);
const FIELD_Y: (f64, f64) = (-0.1883, 0.1890);

// R765 framed heraldic coat of arms: 2.5% chance per level. A carved
// armorial plaque of oak (33.6 x 41.6 cm, the old frame's size): a heater
// shield, its edge moulded with a bead and a cove into the field; the
// field gessoed and painted, the ordinary carved proud of it and painted,
// its charges painted; the bead gilt over red bole. Invented arms, of no
// real family: vert a chevron between three acorns or; gules a bend
// argent between two mullets or; azure a cross or between four annulets
// argent. The paint crazed, chipped to the gesso at its knocks, the gold
// rubbed to the bole, dust on the top of the bead.
pub(crate) fn place_heraldic_crests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old frame's colour: the oak's stain
    let frame_colors = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let stain = match choice(&frame_colors) {
        0x2a1810 => [0.16, 0.10, 0.06],
        0x402015 => [0.24, 0.14, 0.08],
        0x604020 => [0.32, 0.21, 0.12],
        _ => [0.44, 0.32, 0.20],
    };
    let hc = group();
    mount_on_wall(g, &hc, &c, 0.007, 1.60, 0.4);
    hc.rotation().z = (rnd() - 0.5) * 0.020;
    let p = hc.pos();
    let k = salt(p.x, p.z, 31);
    let arms = (k / 7 % 3) as usize;
    put(&hc, "Real__crestBoard", &phong!({ map: &crest_board_tex(k, stain), specular: 0x1e1a12, shininess: 20 }));
    put(&hc, "Real__crestField", &lambert!({ map: &crest_field_tex(k, arms) }));
    let (ordinary, metal) = match arms {
        0 => ("Real__crestChevron", true),
        1 => ("Real__crestBend", false),
        _ => ("Real__crestCross", true),
    };
    let tex = crest_tincture_tex(k, metal);
    let mat = if metal { phong!({ map: &tex, specular: 0x4a3c22, shininess: 30 }) } else { lambert!({ map: &tex }) };
    put(&hc, ordinary, &mat);
    g.world.add(&hc);
}

/// How far in from the crest's outer edge a point of its face is (m)
fn crest_in(x: f64, y: f64) -> f64 {
    let ax = x.abs();
    let side = if y > 0.0 {
        if ax > CREST_HW - CREST_RC && y > CREST_TOP - CREST_RC {
            CREST_RC - ((ax - (CREST_HW - CREST_RC)).powi(2) + (y - (CREST_TOP - CREST_RC)).powi(2)).sqrt()
        } else {
            CREST_HW - ax
        }
    } else {
        CREST_R - ((ax - CREST_CX).powi(2) + y * y).sqrt()
    };
    side.min(CREST_TOP - y)
}

/// The board (u across its 33.6 cm, v up its 41.6 cm): oak under its old
/// stain on the edge and in the cove, the bead gilt over red bole, the
/// gold rubbed through on its crest and darkened in the hollow beside it,
/// dust along the top
fn crest_board_tex(salt: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(224, 278, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * 2.0 * CREST_HW, (v - 0.5) * 2.0 * CREST_TOP);
            let d = crest_in(x, y);
            let grain = fbm(seed, x * 6.0, y * 140.0 + fbm(seed ^ 3, x * 9.0, y * 3.0, 2) * 4.0, 3);
            let mut c = mix(stain, stain.map(|k| k * 0.6), sstep(0.45, 0.72, grain) * 0.7);
            let bead = sstep(0.0030, 0.0040, d) * (1.0 - sstep(0.0100, 0.0112, d));
            if bead > 0.0 {
                let crest = 1.0 - ((d - 0.0066) / 0.0035).abs().min(1.0);
                let mut gold = [0.50, 0.39, 0.19];
                gold = mix(gold, [0.36, 0.28, 0.14], fbm(seed ^ 5, x * 40.0, y * 40.0, 2) * 0.5);
                let worn = crest * sstep(0.45, 0.7, fbm(seed ^ 7, x * 90.0, y * 90.0, 2));
                gold = mix(gold, [0.40, 0.19, 0.11], worn * 0.8);
                c = mix(c, gold, bead);
            }
            // grime in the cove's hollow
            let cove = sstep(0.0105, 0.0125, d) * (1.0 - sstep(0.0150, 0.0175, d));
            c = c.map(|k| k * (1.0 - 0.3 * cove));
            // dust on what faces up: the top's bead and edge
            let top = 1.0 - sstep(0.0, 0.012, CREST_TOP - y);
            let dd = fbm(seed ^ 9, x * 50.0, y * 50.0, 2);
            c = mix(c, [0.52, 0.49, 0.44], (0.05 + 0.3 * top) * (0.6 + 0.8 * dd));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The field (u, v across the field part's bounds): its tincture over
/// gesso, its charges painted with a dark line round them, then aged: the
/// paint crazed into islands, chipped to the gesso, darkened, dust
fn crest_field_tex(salt: u32, arms: usize) -> Tex {
    let ppm = 700.0;
    let (fw, fh) = (FIELD_X.1 - FIELD_X.0, FIELD_Y.1 - FIELD_Y.0);
    canvas_tex_opt((fw * ppm).round(), (fh * ppm).round(), false, move |ctx, w, h| {
        let px = |x: f64| (x - FIELD_X.0) * ppm;
        let py = |y: f64| (FIELD_Y.1 - y) * ppm;
        let (field, charge) = match arms {
            0 => ("#2f4a2c", "#a88a48"),
            1 => ("#7a2621", "#a88a48"),
            _ => ("#23365a", "#c8c2b0"),
        };
        ctx.set_fill_style(field);
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style(charge);
        ctx.set_stroke_style("rgba(20,14,10,0.8)");
        ctx.set_line_width(1.2);
        match arms {
            0 => {
                // three acorns, slipped: two in chief, one in base
                for &(x, y) in &[(-0.080, 0.120), (0.080, 0.120), (0.0, -0.088)] {
                    acorn(ctx, px(x), py(y), 0.030 * ppm);
                }
            }
            1 => {
                // two mullets: in sinister chief and in dexter base
                for &(x, y) in &[(0.078, 0.110), (-0.074, -0.080)] {
                    mullet(ctx, px(x), py(y), 0.036 * ppm);
                }
            }
            _ => {
                // four annulets, one in each quarter
                for &(x, y) in &[(-0.088, 0.112), (0.088, 0.112), (-0.074, -0.058), (0.074, -0.058)] {
                    ctx.set_fill_style(charge);
                    ctx.begin_path();
                    ctx.arc(px(x), py(y), 0.020 * ppm, 0, PI * 2.0);
                    ctx.arc_dir(px(x), py(y), 0.0125 * ppm, PI * 2.0, 0, true);
                    ctx.fill();
                    ctx.stroke();
                }
            }
        }
        age_paint(ctx, w, h, salt ^ 0x71, 1.0);
    })
}

/// An acorn on its stalk, the cup hatched
fn acorn(ctx: &mut Canvas, x: f64, y: f64, s: f64) {
    ctx.begin_path();
    ctx.ellipse(x, y + s * 0.15, s * 0.42, s * 0.62, 0, 0, PI * 2.0);
    ctx.fill();
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(x - s * 0.55, y - s * 0.30);
    ctx.quadratic_curve_to(x, y - s * 0.95, x + s * 0.55, y - s * 0.30);
    ctx.quadratic_curve_to(x, y - s * 0.05, x - s * 0.55, y - s * 0.30);
    ctx.fill();
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(x, y - s * 0.72);
    ctx.quadratic_curve_to(x + s * 0.1, y - s * 1.05, x + s * 0.35, y - s * 1.15);
    ctx.set_line_width(s * 0.12);
    ctx.stroke();
    ctx.set_line_width(1.2);
    // the cup's scales
    ctx.set_stroke_style("rgba(40,28,14,0.55)");
    for k in 0..4 {
        let t = -0.4 + k as f64 * 0.27;
        ctx.begin_path();
        ctx.move_to(x + s * t, y - s * 0.62);
        ctx.line_to(x + s * (t + 0.22), y - s * 0.30);
        ctx.stroke();
    }
    ctx.set_stroke_style("rgba(20,14,10,0.8)");
}

/// A five-pointed mullet
fn mullet(ctx: &mut Canvas, x: f64, y: f64, r: f64) {
    ctx.begin_path();
    for k in 0..10 {
        let a = -PI / 2.0 + k as f64 * PI / 5.0;
        let rr = if k % 2 == 0 { r } else { r * 0.40 };
        if k == 0 {
            ctx.move_to(x + rr * a.cos(), y + rr * a.sin());
        } else {
            ctx.line_to(x + rr * a.cos(), y + rr * a.sin());
        }
    }
    ctx.close_path();
    ctx.fill();
    ctx.stroke();
}

/// The ordinary's paint: gold (an oil gilding, dulled) or silver (white
/// lead gone cream), crazed, chipped, dusty
fn crest_tincture_tex(salt: u32, metal: bool) -> Tex {
    canvas_tex_opt(128, 160, false, move |ctx, w, h| {
        ctx.set_fill_style(if metal { "#9c7e42" } else { "#c8c2b0" });
        ctx.fill_rect(0, 0, w, h);
        age_paint(ctx, w, h, salt ^ 0x83, 0.6);
    })
}

/// Paint on gesso over oak, aged: crazed into islands along fine dark
/// cracks, chipped to the white gesso (and through it to the wood) here and
/// there, darkened unevenly, dust; `scale` how many pixels a centimetre
fn age_paint(ctx: &mut Canvas, w: f64, h: f64, seed: u32, scale: f64) {
    let src = ctx.get_image_data(0, 0, w, h);
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let k = (j * n + i) * 4;
            let mut c = [0, 1, 2].map(|q| f64::from(src.data[k + q]) / 255.0);
            let (x, y) = (i as f64 / (7.0 * scale), j as f64 / (7.0 * scale));
            let crack = (vn(seed, x * 1.3, y * 1.3) - 0.5).abs().min((vn(seed ^ 3, x * 2.7, y * 2.7) - 0.5).abs() * 1.4);
            c = mix(c, [0.12, 0.09, 0.06], (1.0 - sstep(0.0, 0.025, crack)) * 0.55);
            let chip = fbm(seed ^ 5, x * 1.7, y * 1.7, 3) * (0.75 + 0.25 * fbm(seed ^ 6, x * 0.2, y * 0.2, 2));
            c = mix(c, [0.64, 0.60, 0.51], sstep(0.705, 0.715, chip));
            c = mix(c, [0.34, 0.24, 0.16], sstep(0.735, 0.745, chip));
            let dark = fbm(seed ^ 7, x * 0.3, y * 0.3, 3);
            c = c.map(|q| q * (0.82 + 0.25 * dark));
            let d = vn(seed ^ 9, x * 4.0, y * 4.0);
            c = mix(c, [0.52, 0.50, 0.46], sstep(0.75, 0.95, d) * 0.25 + 0.06);
            for q in 0..3 {
                out[i * 4 + q] = (c[q].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

// R766 mounted animal skull on a plaque (wall): 2.5% chance. A roe deer's
// skull, boiled and bleached (the European mount: the whole skull bar the
// jaw, its palate to the board, its nose down, its frontal to the room)
// on a carved oak shield or an oval board (the old plaque's shape draw); a
// buck's with its pedicles and the antlers they carried (batch `t11`'s),
// or a doe's (the old fox and raven); a small brass plate under it. The
// bone ivory (the old bone colour's pick), yellowed toward the occiput and
// the antlers' sap, grimed in its sutures and pits, dust on all of it.
pub(crate) fn place_skull_plaques(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old plaque wood's pick: the oak's stain
    let plaque_colors = [0x2a1810, 0x402015, 0x1a0805, 0x502010];
    let stain = match choice(&plaque_colors) {
        0x2a1810 => [0.30, 0.20, 0.12],
        0x402015 => [0.36, 0.24, 0.14],
        0x1a0805 => [0.22, 0.14, 0.08],
        _ => [0.40, 0.27, 0.15],
    };
    // the old bone colour's pick: how far it has yellowed
    let bone_colors = [0xe0d8c0, 0xd8d0b8, 0xf0e8d0];
    let bone = match choice(&bone_colors) {
        0xe0d8c0 => [0.70, 0.65, 0.54],
        0xd8d0b8 => [0.64, 0.58, 0.46],
        _ => [0.75, 0.72, 0.63],
    };
    let sp = group();
    let is_shield_shape = rnd() < 0.5;
    // the old fox, deer and raven: a doe's skull, a buck's, a doe's
    let variant = (rnd() * 3.0).floor() as i32;
    let buck = variant == 1;
    mount_on_wall(g, &sp, &c, 0.006, 1.70, 0.4);
    sp.rotation().z = (rnd() - 0.5) * 0.010;
    let p = sp.pos();
    let k = salt(p.x, p.z, 29);
    let board = if is_shield_shape { "Real__skullPlaqueShield" } else { "Real__skullPlaqueOval" };
    put(&sp, board, &phong!({ map: &oak_tex(k, stain), specular: 0x141210, shininess: 16 }));
    let skull = if buck { "Real__skullPlaqueBuck" } else { "Real__skullPlaqueDoe" };
    put(&sp, skull, &lambert!({ map: &skull_bone_tex(k, bone, buck) }));
    put(&sp, "Real__skullPlaqueTeeth", &phong!({ color: 0x4e4232, specular: 0x1a1814, shininess: 30 }));
    put(&sp, "Real__skullPlaquePlate", &phong!({ map: &plate_tex(k), specular: 0x5a4a2c, shininess: 40 }));
    if buck {
        let antlers = group();
        antlers.position().set(SKULL_ANTLERS[0], SKULL_ANTLERS[1], SKULL_ANTLERS[2]);
        antlers.rotation().x = SKULL_ANTLER_TURN;
        put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
        put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
        sp.add(&antlers);
    }
    g.world.add(&sp);
}

/// Oak under an old stain and varnish (batch `t11`'s): the grain's lines,
/// the rays' pale flecks across them, the stain uneven, paler where hands
/// rubbed the edges, the varnish crazed, dust in the pores
fn oak_tex(salt: u32, base: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 176, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, base.map(|k| k * 0.6), sstep(0.45, 0.72, grain) * 0.5);
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, base.map(|k| k * 1.4), sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, base.map(|k| k * 1.25), (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, base.map(|k| k * 1.35), (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The skull's bone as its face is laid (u across, v up the plaque: the
/// occiput at the top, the nose at the foot): ivory, yellowed toward the
/// top and (a buck's) brown up round the pedicles where the antlers' sap
/// ran, the sutures dark wandering lines (the coronal across behind the
/// pedicles, the frontonasal ahead of the orbits, the midline), grime in
/// its pits, the orbits' edges darker, dust
fn skull_bone_tex(salt: u32, base: [f64; 3], buck: bool) -> Tex {
    canvas_tex_opt(96, 224, false, move |ctx, w, h| {
        let seed = salt ^ 0x0b1;
        fill(ctx, w, h, |u, v| {
            let mut c = base;
            let n = fbm(seed, u * 6.0, v * 14.0, 3);
            c = mix(c, base.map(|k| k * 0.82), sstep(0.4, 0.75, n) * 0.7);
            c = mix(c, [0.55, 0.46, 0.32], sstep(0.55, 1.0, v) * 0.35);
            if buck {
                let ped = (-(((u - 0.26).powi(2) + (v - 0.68).powi(2)) / 0.004)).exp() + (-(((u - 0.74).powi(2) + (v - 0.68).powi(2)) / 0.004)).exp();
                c = mix(c, [0.40, 0.30, 0.19], ped * 0.7);
            }
            let x = (u - 0.5) * 0.083;
            let y = v * 0.2006 - 0.064;
            let wob = |s: u32, t: f64| (fbm(seed ^ s, t * 140.0, 0.5, 2) - 0.5) * 0.004;
            let d1 = (y - 0.082 - wob(21, x)).abs();
            let d2 = (y - 0.032 - wob(23, x)).abs();
            let d3 = if y > -0.03 { (x - wob(25, y)).abs() } else { 1.0 };
            let sut = 1.0 - sstep(0.0, 0.0009, d1.min(d2).min(d3));
            c = mix(c, [0.36, 0.30, 0.22], sut * 0.45);
            let pit = vn(seed ^ 3, u * 70.0, v * 160.0);
            c = mix(c, [0.50, 0.44, 0.34], sstep(0.84, 0.95, pit) * 0.35);
            let side = (u - 0.5).abs();
            c = mix(c, [0.30, 0.25, 0.19], sstep(0.40, 0.5, side) * 0.5);
            let d = vn(seed ^ 5, u * 70.0, v * 150.0);
            c = mix(c, [0.62, 0.60, 0.55], sstep(0.82, 0.95, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam; batch `t11`'s): the burr and the pearls
/// dark with old sap, their crowns rubbed paler, the gutters darker
fn antler_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x0c7;
        fill(ctx, w, h, |u, v| {
            let low = 1.0 - sstep(0.15, 0.6, v);
            let mut c = mix([0.40, 0.30, 0.20], [0.20, 0.14, 0.09], low);
            let gut = fbm(seed, u * 34.0, v * 3.0, 3);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.55, 0.75, gut) * 0.6);
            let pearl = vn(seed ^ 3, u * 60.0, v * 110.0);
            c = mix(c, [0.62, 0.54, 0.42], sstep(0.72, 0.9, pearl) * (0.35 + 0.4 * low));
            let d = vn(seed ^ 5, u * 90.0, v * 160.0);
            c = mix(c, [0.55, 0.52, 0.46], sstep(0.85, 0.97, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The brass plate: the day it was shot engraved on it, the engraving
/// grimed dark, the brass tarnished brown at its edges
fn plate_tex(salt: u32) -> Tex {
    canvas_tex_opt(168, 48, false, move |ctx, w, h| {
        let seed = salt ^ 0x0e3;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.46, 0.37, 0.19];
            c = mix(c, [0.24, 0.18, 0.10], sstep(0.45, 0.85, fbm(seed, u * 6.0, v * 2.0, 3)) * 0.6);
            let edge = (u.min(1.0 - u) * 3.5).min(v.min(1.0 - v));
            c = mix(c, [0.20, 0.15, 0.09], (1.0 - sstep(0.0, 0.12, edge)) * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
        let months = ["V", "VI", "VII", "VIII", "IX", "X"];
        let day = 1 + salt % 28;
        let date = format!("{}. {}. 19{}", day, months[(salt / 28 % 6) as usize], 52 + salt / 168 % 17);
        ctx.set_fill_style("rgba(28,20,12,0.85)");
        ctx.set_font("italic 22px serif");
        ctx.set_text_align("center");
        ctx.fill_text(&date, w / 2.0, h * 0.68);
    })
}

/// The painting kit of batch `t19`'s painters (as batches `t09`'s and
/// `t11`'s): value noise and a per-pixel fill (pure, so `par_rows` may run
/// it on every core), a salt from where a thing is, a part placed.
mod t19_kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
