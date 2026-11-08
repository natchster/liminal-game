//! Batch `t14`'s plush teddy and small doll, made real (`docs/PROPS.md` P3;
//! the Blender batch `assets/src/blender/props/t14.py`, its notes
//! `t14.notes.md`): their placers and what only they use. Each keeps the
//! old placer's draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use t14_kit::{fbm, fill, hash, mix, put, rgb, salt, ss, vn};

/// The old colours, and the plush each stands for (sRGB): a mid brown, a
/// caramel, a chocolate, a honey
const PLUSH_COLORS: [u32; 4] = [0x6a4020, 0x8a5028, 0x3a2810, 0xa08050];
const PLUSH: [u32; 4] = [0x6c4a33, 0x8a6446, 0x3e2e23, 0xa18565];

/// Plush teddy: 6% chance per level. A machine-made plush teddy of the
/// 1980s-90s, 30 cm standing (acrylic pile on a knit backing, polyester
/// fibre stuffing, sewn to sit; a velour muzzle, inner ears and soles;
/// plastic safety eyes and a moulded nose, the mouth stitched; a satin
/// ribbon in a bow at its neck), slumped where it was left, its head fallen
/// forward. Its pile matted and faded, grimed, the ribbon gone dull.
/// 30%: an eye pulled out, the hole where it went through left.
pub fn place_plush_bears(g: &mut Game, _dist: &[i32]) {
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

/// The head's place in each pose (`t14.py` SD_POSES, printed by its build:
/// `SMALLDOLL`): its parts are in r03's head frame, scaled
const SD_HEAD: [((f64, f64, f64), (f64, f64, f64)); 2] =
    [((0.00008, 0.03005, -0.04119), (-1.38886, 0.35, 0.0)), ((-0.00205, 0.02744, 0.04098), (1.55013, -1.35, 0.0))];
/// r03's head sculpt is scaled by this for the small doll (`t14.py` SD_K)
const SD_K: f64 = 0.47;
/// The small doll's face part's bounds in its head frame (x across, y up):
/// its paint is laid straight on along z
const SD_FACE_LO: (f64, f64) = (-0.02353, -0.00585);
const SD_FACE_HI: (f64, f64) = (0.02381, 0.04522);

/// The old body colours, and the hard plastic each stands for (sRGB): a
/// pale, a tan, a brown, a pink pale; and the old dresses' colours, faded
/// cotton
const SKIN_COLORS: [u32; 4] = [0xd0b898, 0xa08060, 0x684030, 0xd8b8a0];
const SKIN: [u32; 4] = [0xd2b29c, 0xae8a6e, 0x6f4c38, 0xd8bcaa];
const DRESS_COLORS: [u32; 5] = [0xd8a0c0, 0xa0c0d8, 0xc0d8a0, 0xd8c8a0, 0xa0a0d8];
const DRESS: [u32; 5] = [0xbf98a6, 0x92a7b6, 0xa2ad8e, 0xc6b68e, 0x9c98b6];

/// Small doll: 8% chance per level. An 8 in (20 cm) toddler doll of the
/// 1950s-60s, the small walker kind: hard plastic jointed at the neck,
/// shoulders and hips, sleep eyes, a synthetic wig cut in a bob with a
/// fringe, a cotton dress with puffed sleeves (gingham, sprigged or plain),
/// white socks and oilcloth one-strap shoes; its head batch r03's baby's
/// sculpt, smaller, its mouth closed. Found as it was dropped: on its back,
/// its eyes shut (40%), or on its front with its head turned onto its
/// cheek. Its plastic yellowed and scuffed, the wig matted and dusty, the
/// dress faded and grimed.
pub fn place_small_dolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let skin_c = pick(&SKIN_COLORS);
    let dress_c = pick(&DRESS_COLORS);
    let ks = SKIN_COLORS.iter().position(|&c| c == skin_c).unwrap_or(0);
    let kd = DRESS_COLORS.iter().position(|&c| c == dress_c).unwrap_or(0);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    // 40%: on its back; else on its front
    let on_back = rnd() < 0.40;
    let s = salt(px, pz, 417);
    let (p, i) = if on_back { ("A", 0) } else { ("B", 1) };
    let doll = group();
    let skin = phong!({ map: &hard_plastic_tex(s, rgb(SKIN[ks])), specular: 0x1c1a18, shininess: 22 });
    put(&doll, &format!("Real__smallDoll{p}Body"), &skin);
    put(&doll, &format!("Real__smallDoll{p}Dress"), &lambert!({ map: &dress_tex(s, rgb(DRESS[kd])) }));
    put(&doll, &format!("Real__smallDoll{p}Socks"), &lambert!({ map: &sock_tex(s) }));
    put(&doll, &format!("Real__smallDoll{p}Shoes"), &phong!({ color: 0x1d1b1a, specular: 0x2e2e2e, shininess: 28 }));
    // the skull and the wig, laid as the pose lays them
    put(&doll, &format!("Real__smallDoll{p}Skull"), &skin);
    let head = group();
    let ((hx, hy, hz), (rx, ry, rz)) = SD_HEAD[i];
    head.position().set(hx, hy, hz);
    head.rotation().set(rx, ry, rz);
    put(&head, "Real__smallDollFace", &phong!({ map: &small_face_tex(s, rgb(SKIN[ks])), specular: 0x1c1a18, shininess: 22 }));
    let eye = phong!({ map: &small_eye_tex(s), specular: 0x2a2a2a, shininess: 70 });
    put(&head, "Real__smallDollEyeR", &eye);
    put(&head, "Real__smallDollEyeL", &eye);
    put(&head, &format!("Real__smallDoll{p}Lids"), &lambert!({ color: SKIN[ks] }));
    put(&head, &format!("Real__smallDoll{p}Lashes"), &lambert!({ color: 0x2e2620, side: DoubleSide }));
    // the wig: a blonde, a mid brown, an auburn or a dark brown, by where it lies
    let hair = [0xa88c62, 0x7c5e40, 0x7e4a30, 0x46342a][(hash(s, 5, 5) * 4.0) as usize % 4];
    put(
        &doll,
        &format!("Real__smallDoll{p}Wig"),
        &phong!({ map: &wig_tex(s, rgb(hair)), specular: 0x262422, shininess: 14, side: DoubleSide }),
    );
    doll.add(&head);
    doll.position().set(px, 0, pz);
    doll.rotation().y = turn;
    g.world.add(&doll);
}

/// Hard plastic (`b`) gone yellow: faintly uneven, scuffed paler, grime in
/// patches and specks
fn hard_plastic_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4a7;
        fill(ctx, w, h, |u, v| {
            let mut c = mix(b, [b[0] * 0.95, b[1] * 0.9, b[2] * 0.78], 0.5);
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            c = mix(c, [c[0] * 0.85, c[1] * 0.8, c[2] * 0.74], ss(0.45, 0.8, n) * 0.4);
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.38, 0.32, 0.26], ss(0.58, 0.85, grime) * 0.35);
            let scuff = vn(seed ^ 6, u * 30.0, v * 9.0);
            c = mix(c, [0.86, 0.80, 0.72], ss(0.82, 0.95, scuff) * 0.3);
            let speck = vn(seed ^ 7, u * 70.0, v * 70.0);
            if speck > 0.88 {
                c = mix(c, [0.34, 0.28, 0.22], (speck - 0.88) * 4.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The dress's cotton (`b`): a gingham check, a sprig of small flowers or
/// plain (by where it lies), faded unevenly, grimed where it lay, an old
/// stain
fn dress_tex(salt: u32, b: [f64; 3]) -> Tex {
    let kind = (hash(salt, 2, 7) * 3.0) as u32;
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xd2e;
        fill(ctx, w, h, |u, v| {
            let mut c = b;
            match kind {
                0 => {
                    // gingham: white and the colour, their crossings darker
                    let (a, bb) = ((u * 26.0).fract() < 0.5, (v * 26.0).fract() < 0.5);
                    let k = if a && bb {
                        1.0
                    } else if a || bb {
                        0.5
                    } else {
                        0.0
                    };
                    c = mix([0.82, 0.80, 0.76], [b[0] * 0.8, b[1] * 0.8, b[2] * 0.8], k);
                }
                1 => {
                    // sprigged: small pale flowers on the colour
                    let (cu, cv) = ((u * 18.0).floor(), (v * 18.0).floor());
                    let (fu, fv) = ((u * 18.0).fract() - 0.5, (v * 18.0).fract() - 0.5);
                    let o = hash(seed, cu as i32, cv as i32) * 0.3;
                    let d = ((fu - o + 0.15).powi(2) + (fv + o - 0.15).powi(2)).sqrt();
                    c = mix(b, [0.84, 0.80, 0.74], ss(0.16, 0.10, d) * 0.85);
                }
                _ => {}
            }
            // the weave
            let weave = 0.5 + 0.5 * ((u * 900.0).sin() * (v * 900.0).sin());
            c = mix(c, [c[0] * 0.92, c[1] * 0.92, c[2] * 0.92], weave * 0.3);
            let fade = fbm(seed, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.70, 0.68, 0.64], ss(0.45, 0.8, fade) * 0.35);
            let grime = fbm(seed ^ 3, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.36, 0.32, 0.27], ss(0.55, 0.85, grime) * 0.4);
            let (du, dv) = (u - 0.3 - 0.4 * hash(seed, 1, 1), v - 0.3 - 0.4 * hash(seed, 2, 2));
            let d = (du * du + dv * dv).sqrt() / 0.08;
            c = mix(c, [c[0] * 0.78, c[1] * 0.72, c[2] * 0.62], ss(1.0, 0.8, d) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// White cotton socks gone grey, darker at the soles' edge
fn sock_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x50c;
        fill(ctx, w, h, |u, v| {
            let rib = 0.5 + 0.5 * (u * 120.0).sin();
            let mut c = mix([0.80, 0.79, 0.76], [0.70, 0.69, 0.66], rib * 0.4);
            c = mix(c, [0.45, 0.42, 0.38], (ss(0.5, 0.85, fbm(seed, u * 4.0, v * 4.0, 3)) + (0.3 - v).max(0.0)) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The wig's synthetic hair (`b`): combed strands, a soft sheen in their
/// lie, matted and dusty
fn wig_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x816;
        fill(ctx, w, h, |u, v| {
            let strand = fbm(seed, u * 110.0, v * 10.0, 3);
            let mut c = mix(b, [b[0] * 0.6, b[1] * 0.58, b[2] * 0.56], ss(0.4, 0.75, strand) * 0.6);
            let sheen = fbm(seed ^ 1, u * 40.0, v * 6.0, 2);
            c = mix(c, [b[0] * 1.2 + 0.05, b[1] * 1.18 + 0.05, b[2] * 1.15 + 0.04], ss(0.62, 0.8, sheen) * 0.35);
            let mat = fbm(seed ^ 2, u * 6.0, v * 6.0, 3);
            c = mix(c, [b[0] * 0.75, b[1] * 0.72, b[2] * 0.7], ss(0.55, 0.75, mat) * 0.4);
            c = mix(c, [0.50, 0.47, 0.42], ss(0.6, 0.85, fbm(seed ^ 3, u * 3.0, v * 3.0, 3)) * 0.3);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

fn small_face_px(x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let u = (x - SD_FACE_LO.0) / (SD_FACE_HI.0 - SD_FACE_LO.0);
    let v = (y - SD_FACE_LO.1) / (SD_FACE_HI.1 - SD_FACE_LO.1);
    (u * w, (1.0 - v) * h)
}

/// The small doll's face, as the factory sprayed and brushed it (r03's
/// face's places, scaled): blush on its cheeks, fine brows, painted lashes
/// under the eyes, a closed rosebud mouth, the nostrils; rubbed, grimed
fn small_face_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(160, 176, false, move |ctx, w, h| {
        let seed = salt ^ 0xfa2;
        let k = SD_K * w / (SD_FACE_HI.0 - SD_FACE_LO.0); // px per metre of r03's head
        let px = |x: f64, y: f64| small_face_px(x * SD_K, y * SD_K, w, h);
        fill(ctx, w, h, |u, v| {
            let mut c = mix(b, [b[0] * 0.95, b[1] * 0.9, b[2] * 0.78], 0.5);
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            c = mix(c, [c[0] * 0.86, c[1] * 0.82, c[2] * 0.76], ss(0.45, 0.8, n) * 0.35);
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.40, 0.33, 0.27], ss(0.6, 0.85, grime) * 0.28);
            [c[0], c[1], c[2], 1.0]
        });
        let blot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, c0: &str, c1: &str| {
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, c0);
            gr.add_color_stop(1, c1);
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        };
        for s in [-1.0, 1.0] {
            let (cx0, cy0) = px(s * 0.025, 0.038);
            blot(ctx, cx0, cy0, 0.015 * k, "rgba(196,112,100,0.38)", "rgba(196,112,100,0)");
            ctx.set_stroke_style("rgba(110,80,58,0.55)");
            ctx.set_line_width(0.9);
            ctx.begin_path();
            for i in 0..8 {
                let t = f64::from(i) / 7.0;
                let (a, b) = px(s * (0.009 + 0.020 * t), 0.0712 + 0.0035 * (1.0 - (2.0 * t - 0.8).powi(2)));
                if i == 0 {
                    ctx.move_to(a, b);
                } else {
                    ctx.line_to(a, b);
                }
            }
            ctx.stroke();
            ctx.set_stroke_style("rgba(70,50,40,0.6)");
            ctx.set_line_width(0.8);
            for i in 0..5 {
                let t = f64::from(i) / 4.0;
                let (a, b) = px(s * (0.012 + 0.013 * t), 0.0528);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.line_to(a + s * 0.8, b + 2.0);
                ctx.stroke();
            }
            let (a, b) = px(s * 0.0034, 0.0442);
            ctx.set_fill_style("rgba(120,70,60,0.55)");
            ctx.begin_path();
            ctx.ellipse(a, b, 0.9, 0.6, 0, 0, 7);
            ctx.fill();
        }
        // a closed rosebud mouth: the upper lip's two peaks, the lower round
        let (mx, my) = px(0.0, 0.0335);
        ctx.set_fill_style("rgba(168,82,76,0.75)");
        for s in [-1.0, 1.0] {
            ctx.begin_path();
            ctx.ellipse(mx + s * 0.0022 * k, my - 0.0012 * k, 0.0026 * k, 0.0016 * k, s * 0.3, 0, 7);
            ctx.fill();
        }
        ctx.begin_path();
        ctx.ellipse(mx, my + 0.0016 * k, 0.0034 * k, 0.0019 * k, 0, 0, 7);
        ctx.fill();
        ctx.set_fill_style("rgba(90,40,36,0.7)");
        ctx.fill_rect(mx - 0.0034 * k, my, 0.0068 * k, 0.6);
        // the paint rubbed off the nose's tip and a cheek; a scuff
        let (nx, ny) = px(0.0, 0.047);
        blot(ctx, nx, ny, 0.004 * k, "rgba(236,220,204,0.4)", "rgba(236,220,204,0)");
        let sx = if hash(salt, 4, 4) < 0.5 { -1.0 } else { 1.0 };
        let (gx, gy) = px(sx * 0.024, 0.030);
        blot(ctx, gx, gy, 0.010 * k, "rgba(76,68,60,0.3)", "rgba(76,68,60,0)");
    })
}

/// A sleep eye: an ivory ball, a blue or brown iris with a darker ring,
/// the pupil black (the iris faces +z: a quarter of the way along)
fn small_eye_tex(salt: u32) -> Tex {
    let brown = hash(salt, 6, 6) < 0.4;
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        ctx.set_fill_style("#cec4ae");
        ctx.fill_rect(0, 0, w, h);
        let (cx0, cy0) = (w * 0.25, h * 0.5);
        let gr = ctx.create_radial_gradient(cx0, cy0, 0, cx0, cy0, 11);
        if brown {
            gr.add_color_stop(0, "#3a2a1c");
            gr.add_color_stop(0.6, "#664a30");
            gr.add_color_stop(1, "#22180f");
        } else {
            gr.add_color_stop(0, "#3a5266");
            gr.add_color_stop(0.6, "#5f7d96");
            gr.add_color_stop(1, "#25303c");
        }
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 11, 11, 0, 0, 7);
        ctx.fill();
        ctx.set_fill_style("#1a1818");
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 4.5, 4.5, 0, 0, 7);
        ctx.fill();
    })
}

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

/// Where the plush is matted flat (0 to 1): the pile and its tips' layer
/// read it at the same place
fn matted(seed: u32, u: f64, v: f64) -> f64 {
    ss(0.6, 0.72, fbm(seed ^ 4, u * 6.0, v * 6.0, 4))
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

/// What this batch's painters share in this crate
mod t14_kit {
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
    pub(super) fn ss(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// A colour (0xrrggbb) as fractions
    pub(super) fn rgb(c: u32) -> [f64; 3] {
        [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
    }
    /// Every pixel from where it is (u across, v up), on every core
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
    /// A salt from where a thing is (no draw): its painters differ from
    /// floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
