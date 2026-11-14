//! `place_boot_trays`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::salt;

// Boot tray w/ 1-2 pairs of boots (floor): 4% chance per level. A 32 x
// 16 in moulded rubber boot tray, ribbed (batch `t07`), a pair or two of
// lace-up work boots on it (r14a's), the second pair's one fallen over;
// dried grit and the salt's tide lines on it, and six times in ten the
// dirty water they dripped still standing between its ribs.
pub(crate) fn place_boot_trays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old colour draw: recycled rubber, black, brown, charcoal, blue-black
    let tray_colors = [0x1a1a1a, 0x604020, 0x2a2a2a, 0x202030];
    let tc = choice(&tray_colors);
    let ti = tray_colors.iter().position(|&c| c == tc).unwrap_or(0);
    let rubber = [[0.115, 0.112, 0.108], [0.20, 0.15, 0.11], [0.17, 0.168, 0.162], [0.12, 0.12, 0.135]][ti];
    let salt = salt(g.wx(x), g.wz(z), ti as u32);
    let bt = group();
    put(&bt, "Real__bootTray", &lambert!({ map: &tray_tex(rubber, salt) }));
    // the wet left in it (60%): dirty water standing between the ribs
    if rnd() < 0.6 {
        let wet = mesh(
            plane_geo!(0.78, 0.38),
            phong!({ map: &tray_water_tex(salt), specular: 0x4a4a48, shininess: 80, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        wet.rotation().x = -PI / 2.0;
        wet.position().y = 0.0068;
        bt.add(&wet);
    }
    // the old boots' colours (rubber boots' toy colours) as a work boot's
    // leathers: oxblood, dark brown oiled, wheat nubuck, black
    let boot_colors = [0xa02040, 0x408060, 0xd0a040, 0x2060a0, 0x101010, 0xf0e0a0, 0x804020];
    let leathers = [
        (0x46282a, 0x34262a, 0x262422, 0x3a2c24, 0x3c3832),
        (0x4e3626, 0x3a2c22, 0x2a2826, 0x7a6448, 0x4a4236),
        (0x8a6a48, 0x5a4836, 0x3a332c, 0x8c7a5c, 0x5c5040),
        (0x2e2b28, 0x2a2522, 0x262422, 0x2a2724, 0x3c3832),
    ];
    let boot_pairs = 1 + (rnd() * 2.0).floor() as i32;
    for p in 0..boot_pairs {
        let bc = choice(&boot_colors);
        let cols = leathers[[0, 1, 2, 3, 3, 2, 1][boot_colors.iter().position(|&c| c == bc).unwrap_or(0)]];
        let pair_x = if p == 0 { -0.162 } else { 0.162 };
        let vary = salt ^ (p as u32).wrapping_mul(0x2545_f491);
        // the second pair's right boot has fallen over onto its outer side
        let fallen = p == 1;
        for side in 0..2 {
            let right = side == 1;
            let boot = if fallen && right {
                boot_footwear("boot", "Side", cols, vary, false)
            } else {
                boot_footwear("boot", "", cols, vary ^ side as u32, !right)
            };
            let (bx, yaw) = match (fallen, right) {
                (true, true) => (pair_x + 0.165, 0.12),
                (true, false) => (pair_x - 0.09, -0.06),
                (false, r) => (pair_x + if r { 0.064 } else { -0.064 }, if r { -0.05 } else { 0.07 }),
            };
            boot.position().set(bx, TRAY_RIB_TOP, if fallen { 0.01 } else { -0.005 });
            boot.rotation().y = yaw;
            bt.add(&boot);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    bt.position().set(px, 0, pz);
    bt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bt);
}

/// The boot tray as batch `t07` models it: its floor's top and its ribs'
const TRAY_RIB_TOP: f64 = 0.0055 + 0.0040;

/// The tray's rubber (laid on it from above: u along it, v across): moulded
/// recycled rubber, grey with grit and dust trodden in, the white tide lines
/// of the road salt where meltwater dried, mud crusted along the ribs
fn tray_tex(rubber: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x7e7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 20.0, v * 10.0, 4);
            let mut c = mix(rubber, [rubber[0] * 1.5, rubber[1] * 1.5, rubber[2] * 1.5], n * 0.6);
            let grit = vn(seed ^ 3, u * 400.0, v * 200.0);
            if grit > 0.83 {
                c = mix(c, [0.42, 0.39, 0.34], (grit - 0.83) * 4.0);
            }
            // the tide lines: rings of salt about the boots' places
            for (cu, cv, r) in [(0.30, 0.5, 0.17), (0.70, 0.52, 0.19), (0.5, 0.4, 0.35)] {
                let d = ((u - cu) * 2.0).hypot(v - cv) / r + (fbm(seed ^ 5, u * 8.0, v * 8.0, 3) - 0.5) * 0.5;
                let line = (1.0 - ((d - 1.0).abs() / 0.06)).max(0.0).powf(1.5);
                c = mix(c, [0.52, 0.51, 0.48], line * 0.3 * (0.5 + fbm(seed ^ 13, u * 40.0, v * 20.0, 2)));
                if d < 1.0 {
                    c = mix(c, [0.40, 0.38, 0.34], 0.18 * fbm(seed ^ 9, u * 30.0, v * 30.0, 2));
                }
            }
            let mud = sstep(0.55, 0.75, fbm(seed ^ 11, u * 6.0, v * 3.0, 4));
            c = mix(c, [0.24, 0.20, 0.15], mud * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The water standing in it: grey-brown, darker where deeper, its edges
/// ragged where the ribs break it, a scum of grit (transparent round it)
fn tray_water_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a7;
        fill(ctx, w, h, |u, v| {
            let edge = sstep(0.0, 0.06, u.min(1.0 - u)) * sstep(0.0, 0.1, v.min(1.0 - v));
            let depth = fbm(seed, u * 5.0, v * 2.5, 4);
            let a = sstep(0.42, 0.6, depth) * edge;
            let c = mix([0.30, 0.28, 0.24], [0.16, 0.15, 0.13], depth);
            let scum = vn(seed ^ 3, u * 160.0, v * 80.0);
            let c = if scum > 0.86 { mix(c, [0.45, 0.42, 0.36], 0.6) } else { c };
            [c[0], c[1], c[2], a * 0.8]
        });
    })
}

// ---- the boots of batch r14a (the work boot, standing or fallen over) ----
/// The part `name` of a Blender batch, as modelled, added to `grp` in `mat`.
fn boot_add(grp: &Obj, name: &str, mat: &Mat, mirror: bool) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(if mirror { mirror_x(&geo) } else { geo }, mat.clone()));
    }
}

/// A copy of `geo` mirrored across x (a left boot of a right one): its
/// triangles turned so they still face out
fn mirror_x(geo: &Geo) -> Geo {
    let out = geo.duplicate();
    {
        let mut gg = out.g();
        for i in 0..gg.position.len() / 3 {
            gg.position[i * 3] = -gg.position[i * 3];
        }
        for i in 0..gg.normal.len() / 3 {
            gg.normal[i * 3] = -gg.normal[i * 3];
        }
        if let Some(idx) = gg.index.as_mut() {
            for t in idx.chunks_mut(3) {
                t.swap(1, 2);
            }
        }
        gg.touch();
    }
    out.compute_bounding_box();
    out.compute_bounding_sphere();
    out
}

fn boot_rgb(c: u32) -> (f64, f64, f64) {
    (((c >> 16) & 255) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64)
}

fn boot_css(c: (f64, f64, f64), a: f64) -> String {
    let k = |v: f64| v.clamp(0.0, 255.0).round();
    format!("rgba({},{},{},{:.3})", k(c.0), k(c.1), k(c.2), a.clamp(0.0, 1.0))
}

fn boot_mix(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
/// fading out from its middle (the context squashed, so no edge cuts it).
fn boot_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &boot_css(c, a));
    gr.add_color_stop(1, &boot_css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A short wandering line from `at`, `len` px long at angle `ang`.
fn boot_wander(ctx: &mut Canvas, at: (f64, f64), len: f64, ang: f64, c: (f64, f64, f64), a: f64, lw: f64) {
    ctx.set_stroke_style(boot_css(c, a));
    ctx.set_line_width(lw);
    ctx.begin_path();
    ctx.move_to(at.0, at.1);
    let (mut x, mut y, mut t) = (at.0, at.1, ang);
    for _ in 0..5 {
        t += (rnd() - 0.5) * 0.5;
        x += t.cos() * len / 5.0;
        y += t.sin() * len / 5.0;
        ctx.line_to(x, y);
    }
    ctx.stroke();
}

/// Leather after years, in its colour `base` (sRGB): mottled where it was
/// oiled and dried, its grain, creased, rubbed pale and scuffed (most low
/// on it, v < 0.35: the welt's edge all round, the toe's foot), a film of
/// dust. Painted on a stream of its own varied by `vary`.
fn boot_leather_tex(base: u32, size: u32, vary: u32) -> Tex {
    canvas_tex(size, size, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || boot_paint_leather(ctx, w, h, base))
    })
}

fn boot_paint_leather(ctx: &mut Canvas, w: f64, h: f64, base: u32) {
    let c = boot_rgb(base);
    let s = w / 256.0;
    ctx.set_fill_style(boot_css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.62, c.1 * 0.58, c.2 * 0.55);
    let pale = boot_mix(c, (196.0, 184.0, 166.0), 0.42);
    for _ in 0..36 {
        let at = (rnd() * w, rnd() * h);
        let (col, a) = if rnd() < 0.55 { (dark, 0.10 + rnd() * 0.14) } else { (pale, 0.06 + rnd() * 0.10) };
        boot_soft(ctx, at, (10.0 + rnd() * 34.0) * s, 0.5 + rnd() * 0.5, rnd() * 3.0, col, a);
    }
    // the grain: pores and a fine pebble
    for _ in 0..(2600.0 * s * s) as i32 {
        let col = if rnd() < 0.7 { dark } else { pale };
        ctx.set_fill_style(boot_css(col, 0.12 + rnd() * 0.16));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
    }
    // creases, each with its pale lip
    for _ in 0..(48.0 * s) as i32 {
        let at = (rnd() * w, rnd() * h);
        let len = (6.0 + rnd() * 24.0) * s;
        let ang = (rnd() - 0.5) * 1.0 + if rnd() < 0.25 { PI / 2.0 } else { 0.0 };
        boot_wander(ctx, at, len, ang, (c.0 * 0.4, c.1 * 0.36, c.2 * 0.34), 0.45, 0.8 * s.max(0.7));
        boot_wander(ctx, (at.0, at.1 + 1.0), len * 0.8, ang, pale, 0.18, 0.7);
    }
    // rubbed pale and scuffed, most of it low: the dye worn through to the
    // crust in ragged streaks along the rub, scratched
    for i in 0..30 {
        let y = if i < 22 { h * (0.66 + rnd() * 0.34) } else { rnd() * h };
        let at = (rnd() * w, y);
        let crust = boot_mix(c, (190.0, 178.0, 160.0), 0.40 + rnd() * 0.2);
        let ang = (rnd() - 0.5) * 0.5;
        for _ in 0..(3 + (rnd() * 5.0) as i32) {
            let a2 = (at.0 + (rnd() - 0.5) * 10.0 * s, at.1 + (rnd() - 0.5) * 4.0 * s);
            boot_wander(
                ctx,
                a2,
                (2.0 + rnd() * 9.0) * s,
                ang + (rnd() - 0.5) * 0.3,
                crust,
                0.18 + rnd() * 0.3,
                (0.5 + rnd() * 0.9) * s.max(0.6),
            );
        }
        boot_soft(ctx, at, (4.0 + rnd() * 8.0) * s, 0.25, ang, crust, 0.10 + rnd() * 0.10);
    }
    // dust, and dried water's tide marks
    for _ in 0..12 {
        boot_soft(ctx, (rnd() * w, rnd() * h), (18.0 + rnd() * 40.0) * s, 0.6, rnd() * 3.0, (150.0, 144.0, 132.0), 0.08 + rnd() * 0.08);
    }
    for _ in 0..3 {
        let (x, y, r) = (rnd() * w, rnd() * h, (10.0 + rnd() * 26.0) * s);
        ctx.set_stroke_style(boot_css(pale, 0.22));
        ctx.set_line_width(1.2 * s);
        ctx.begin_path();
        ctx.arc(x, y, r, rnd() * 3.0, 3.0 + rnd() * 3.0);
        ctx.stroke();
    }
}

/// The shoe or boot of batch r14a, `kind` "shoe" or "boot": `var` "R", "L",
/// "Side" (the right one fallen on its outer side, as it lies) or "" (the
/// boot standing). Colours sRGB: the leather, the lining, the sole's
/// rubber or leather, the laces, the eyelets' and hooks' metal.
fn boot_footwear(kind: &str, var: &str, cols: (u32, u32, u32, u32, u32), vary: u32, mirror: bool) -> Obj {
    let (leather, lining, sole, lace, metal) = cols;
    let grp = group();
    let name = |p: &str| format!("Real__{kind}{p}{var}");
    let boot = kind == "boot";
    let upper = lambert!({ color: 0xffffff, map: &boot_leather_tex(leather, 256, vary) });
    boot_add(&grp, &name("Upper"), &upper, mirror);
    boot_add(&grp, &name("Lining"), &lambert!({ color: lining }), mirror);
    let edge = lambert!({ color: 0xffffff, map: &boot_sole_tex(sole, false, vary ^ 0x91) });
    boot_add(&grp, &name("Sole"), &edge, mirror);
    let heel =
        if boot || var == "Side" { edge.clone() } else { lambert!({ color: 0xffffff, map: &boot_sole_tex(0x3e2c22, true, vary ^ 0x93) }) };
    boot_add(&grp, &name("Heel"), &heel, mirror);
    // the tread shows only on one that has fallen over
    let tread = if var == "Side" {
        lambert!({ color: 0xffffff, map: &boot_tread_tex(boot, sole, vary ^ 0x97) })
    } else {
        lambert!({ color: sole })
    };
    boot_add(&grp, &name("Tread"), &tread, mirror);
    boot_add(&grp, &name("Laces"), &lambert!({ color: lace }), mirror);
    let met = phong!({ color: metal, specular: 0x4a4a46, shininess: 40 });
    boot_add(&grp, &name(if boot { "Hooks" } else { "Eyelets" }), &met, mirror);
    grp
}

/// A sole's edge and welt (or `stack`, a heel of leather lifts on a rubber
/// top piece, its layers across): inked and burnished, scuffed pale, dirt.
fn boot_sole_tex(base: u32, stack: bool, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = boot_rgb(base);
            ctx.set_fill_style(boot_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            if stack {
                // the lifts (each 4-5 mm of the heel's 22), the top piece below
                let mut y = 0.0;
                while y < h * 0.76 {
                    ctx.set_fill_style(boot_css((c.0 * 0.55, c.1 * 0.5, c.2 * 0.48), 0.7));
                    ctx.fill_rect(0, y, w, 1.0);
                    ctx.set_fill_style(boot_css(boot_mix(c, (150.0, 120.0, 96.0), 0.3), 0.3));
                    ctx.fill_rect(0, y + 1.0, w, 1.0);
                    y += h * 0.19;
                }
                ctx.set_fill_style("rgb(40,38,36)");
                ctx.fill_rect(0, h * 0.77, w, h * 0.23);
            }
            for _ in 0..40 {
                ctx.set_fill_style(boot_css((c.0 * 0.7, c.1 * 0.68, c.2 * 0.66), 0.2));
                ctx.fill_rect(rnd() * w, rnd() * h, 4.0 + rnd() * 18.0, 1.0);
            }
            for _ in 0..22 {
                let crust = boot_mix(c, (150.0, 140.0, 126.0), 0.5);
                boot_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 6.0, 0.4, (rnd() - 0.5) * 0.5, crust, 0.35);
            }
            for _ in 0..10 {
                boot_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 20.0, 0.5, 0.0, (110.0, 102.0, 90.0), 0.12);
            }
        })
    })
}

/// The tread of one fallen on its side, as it lies (u from the toe, at u =
/// 0, back to the heel; v from the outer edge, on the floor, up to the inner
/// one): a shoe's leather sole worn dark at the ball and frayed at the toe,
/// paler in the waist where it never touched, the heel's rubber top piece
/// worn at its back; a boot's lugs, worn grey on their faces, grit in the
/// grooves.
fn boot_tread_tex(boot: bool, base: u32, vary: u32) -> Tex {
    canvas_tex(256, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // the heel's breast, as modelled (domestic/things_worn_and_dropped.py): the shoe's at 78% of
            // the tread from the toe, the boot's at 77%
            let br = if boot { 0.771 } else { 0.781 };
            if !boot {
                let leather = (122.0, 96.0, 70.0);
                ctx.set_fill_style(boot_css(leather, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // worn: the ball dark and smooth, the toe frayed paler
                boot_soft(ctx, (w * 0.34, h * 0.5), w * 0.22, 0.6, 0.0, (64.0, 54.0, 44.0), 0.6);
                boot_soft(ctx, (w * 0.04, h * 0.5), w * 0.07, 1.4, 0.0, (150.0, 132.0, 108.0), 0.5);
                // the waist, never on the floor: paler, the size stamped
                boot_soft(ctx, (w * 0.66, h * 0.5), w * 0.12, 0.6, 0.0, (160.0, 132.0, 100.0), 0.45);
                ctx.save();
                ctx.translate(w * 0.66, h * 0.5);
                ctx.rotate(-PI / 2.0);
                ctx.set_fill_style("rgba(60,46,34,0.55)");
                ctx.set_font("bold 11px Helvetica");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("9", 0, -6);
                ctx.set_font("7px Helvetica");
                ctx.fill_text("LEATHER", 0, 7);
                ctx.restore();
                for _ in 0..140 {
                    boot_wander(ctx, (rnd() * w * br, rnd() * h), 3.0 + rnd() * 9.0, (rnd() - 0.5) * 6.3, (70.0, 58.0, 46.0), 0.25, 0.6);
                }
                // the heel's top piece, black rubber, worn at the back
                ctx.set_fill_style("rgb(44,41,38)");
                ctx.fill_rect(w * br, 0, w * (1.0 - br), h);
                boot_soft(ctx, (w * 1.0, h * 0.35), w * 0.06, 1.8, 0.0, (96.0, 88.0, 78.0), 0.6);
                for k in 0..5 {
                    ctx.set_fill_style("rgba(120,112,100,0.7)");
                    ctx.fill_rect(w * (br + 0.03 + 0.035 * (k as f64 % 2.0)), h * (0.2 + 0.15 * k as f64), 2.0, 2.0);
                }
            } else {
                let c = boot_rgb(base);
                let groove = (c.0 * 0.55, c.1 * 0.55, c.2 * 0.55);
                ctx.set_fill_style(boot_css(groove, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // lugs in rows across the sole, chevroned; few in the waist
                let face = boot_mix(c, (120.0, 114.0, 106.0), 0.35);
                let mut x = 6.0;
                let mut row = 0;
                while x < w - 4.0 {
                    let waist = x / w > 0.56 && x / w < br;
                    if x / w > br - 0.012 && x / w < br + 0.012 {
                        x += 6.0;
                        continue;
                    }
                    let n = if waist { 2 } else { 4 };
                    for k in 0..n {
                        let y = h * (0.12 + 0.76 * (k as f64 + 0.5 * (row % 2) as f64) / n as f64);
                        ctx.save();
                        ctx.translate(x, y);
                        ctx.rotate(if k * 2 < n { 0.5 } else { -0.5 });
                        ctx.set_fill_style(boot_css(face, 1.0));
                        ctx.fill_rect(-4.0, -9.0, 8.0, 18.0);
                        ctx.restore();
                    }
                    x += if waist { 16.0 } else { 11.0 };
                    row += 1;
                }
                // the faces worn grey, grit in the grooves
                boot_soft(ctx, (w * 0.34, h * 0.5), w * 0.2, 0.6, 0.0, (130.0, 124.0, 114.0), 0.25);
                boot_soft(ctx, (w * 0.92, h * 0.5), w * 0.1, 0.8, 0.0, (130.0, 124.0, 114.0), 0.25);
                for _ in 0..260 {
                    ctx.set_fill_style(boot_css((150.0, 140.0, 124.0), 0.35));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
                }
            }
        })
    })
}
