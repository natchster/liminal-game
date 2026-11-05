//! Batch `t14`'s doll's head and its arms and legs, made real
//! (`docs/PROPS.md` P3; the Blender batch `assets/src/blender/props/t14.py`,
//! its notes `t14.notes.md`): a vinyl baby doll's (batch `r03`'s) head and
//! limbs come off at their joints, found on the floor. Their placers and
//! what only they use; each keeps the old placer's draws, call for call.
use super::*;
use t14_kit::{fbm, fill, hash, mix, put, salt, ss, vn};

/// r03's head frame in its lying body's (`r03.py` DOLL_NECK, DOLL_TURN):
/// where the head's own parts are laid when it lies as the doll's does
const DOLL_NECK: (f64, f64, f64) = (0.050, 0.0592, 0.0);
const DOLL_TURN: f64 = -0.30;
/// How far a head standing on its neck stands up: its neck's rim under the
/// head frame's origin (`t14.py` `head_neck_lip`)
const NECK_LIFT: f64 = 0.0139;

// R494 a doll's head: 3% chance per level. A vinyl baby doll's head
// (batch r03's: 10 cm across, moulded hair, sleep eyes with lashes, the
// mouth open for its bottle) come off at its neck: the vinyl rolled in at
// the rim, the hollow dark inside. Lying on the back of its head as the
// doll lay, its lids down (the left stuck half open), or standing on its
// neck, its eyes open (the left stuck half shut); yellowed, grimed, the
// paint rubbed.
pub(crate) fn place_doll_heads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old hair colours: the paint of its moulded hair
    let hair_colors = [0x604020, 0x201810, 0xd8a040, 0x8a2020];
    let hair = pick_hex(&hair_colors);
    let lying = rnd() < 0.60;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let k = salt(px, pz, 494);
    let vinyl = lambert!({ map: &vinyl_tex(k, 0xd6b19a) });
    let doll = group();
    // the parts r03 laid into its lying body's frame
    let skull = group();
    put(&skull, "Real__dollHead", &vinyl);
    put(
        &skull,
        "Real__dollHair",
        &lambert!({ map: &hair_tex(k, hair), polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
    );
    // and those in the head's own frame (face +z, crown +y)
    let head = group();
    put(&head, "Real__dollFace", &lambert!({ map: &face_tex(k) }));
    let eye = phong!({ map: &eye_tex(k), specular: 0x2a2a2a, shininess: 70 });
    put(&head, "Real__dollEyeR", &eye);
    put(&head, "Real__dollEyeL", &eye);
    let lid = lambert!({ color: 0xbf9a86 });
    let lash = lambert!({ color: 0x3a312b, side: DoubleSide });
    if lying {
        put(&head, "Real__dollLids", &lid);
        put(&head, "Real__dollLashes", &lash);
    } else {
        put(&head, "Real__dollHeadLidsUp", &lid);
        put(&head, "Real__dollHeadLashesUp", &lash);
    }
    put(&head, "Real__dollHeadNeckLip", &vinyl);
    put(&head, "Real__dollHeadNeckHole", &lambert!({ color: 0x0d0a09 }));
    if lying {
        // as the doll lies: on the back of its head, rolled toward its right
        head.position().set(DOLL_NECK.0, DOLL_NECK.1, DOLL_NECK.2);
        head.rotation().set(DOLL_TURN - PI / 2.0, 0, -PI / 2.0);
        doll.add(&skull);
        doll.add(&head);
    } else {
        // standing on its neck: the lying frame's parts brought back into
        // the head's (the inverse of r03's laying down)
        let a = group();
        a.rotation().z = PI / 2.0;
        let b = group();
        b.rotation().x = PI / 2.0 - DOLL_TURN;
        skull.position().set(-DOLL_NECK.0, -DOLL_NECK.1, -DOLL_NECK.2);
        b.add(&skull);
        a.add(&b);
        head.add(&a);
        head.position().y = NECK_LIFT;
        doll.add(&head);
    }
    doll.position().set(px, 0, pz);
    doll.rotation().y = turn;
    g.world.add(&doll);
}

// R499 a doll's arms and legs: 4% chance per level. Two or three of a vinyl
// baby doll's limbs (batch r03's doll: an arm bent at the elbow, its hand's
// fingers moulded together; a leg bent at the knee, its foot with toes),
// come off its cloth body: each limb's top a groove the cloth was tied in
// and a lip round its open end, the hollow dark inside. Lying where they
// fell, apart.
pub(crate) fn place_doll_body_parts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let k = salt(g.wx(x), g.wz(z), 499);
    // one doll's: the same vinyl, yellowed alike
    let vinyl = lambert!({ map: &vinyl_tex(k, 0xd3ae97) });
    let dark = lambert!({ color: 0x0d0a09 });
    let grp = group();
    let n_parts = 2 + (rnd() * 2.0).floor() as i32;
    let mut placed: Vec<(f64, f64)> = vec![];
    for _ in 0..n_parts {
        let is_arm = rnd() < 0.5;
        let part = group();
        let limb = group();
        let (name, hole, half) = if is_arm {
            ("Real__dollLimbArm", "Real__dollLimbArmHole", 0.052)
        } else {
            ("Real__dollLimbLeg", "Real__dollLimbLegHole", 0.056)
        };
        put(&limb, name, &vinyl);
        put(&limb, hole, &dark);
        // turned about its middle, as the old limb was
        limb.position().x = -half;
        part.add(&limb);
        let (mut ox, mut oz) = ((rnd() - 0.5) * 0.30, (rnd() - 0.5) * 0.20);
        let yaw = rnd() * PI * 2.0;
        // (the old tilt's draw: a little more turn)
        let tilt = (rnd() - 0.5) * 0.4;
        // apart: a limb never lies through another (no draw)
        for &(qx, qz) in &placed {
            let (dx, dz) = (ox - qx, oz - qz);
            let d = (dx * dx + dz * dz).sqrt();
            if d < 0.105 {
                let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (1.0, 0.0) };
                ox = qx + ux * 0.105;
                oz = qz + uz * 0.105;
            }
        }
        placed.push((ox, oz));
        part.position().set(ox, 0, oz);
        part.rotation().y = yaw + tilt;
        grp.add(&part);
    }
    scatter_on_floor(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

/// Soft vinyl gone yellow (`base`): faintly uneven, grime gathered in
/// soft patches and specks, scuffed paler where it rubbed on the floor, a
/// child's ballpoint here and there
fn vinyl_tex(salt: u32, base: u32) -> Tex {
    let b = [f64::from((base >> 16) & 255) / 255.0, f64::from((base >> 8) & 255) / 255.0, f64::from(base & 255) / 255.0];
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x51d;
        fill(ctx, w, h, |u, v| {
            let mut c = b;
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            c = mix(c, [b[0] * 0.84, b[1] * 0.78, b[2] * 0.70], ss(0.45, 0.8, n) * 0.45);
            let pale = fbm(seed ^ 3, u * 7.0, v * 7.0, 3);
            c = mix(c, [0.88, 0.78, 0.70], ss(0.6, 0.85, pale) * 0.2);
            // grime: blotches and the dirt of years in its pores
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.40, 0.33, 0.27], ss(0.55, 0.8, grime) * 0.3);
            let speck = vn(seed ^ 7, u * 70.0, v * 70.0);
            if speck > 0.88 {
                c = mix(c, [0.36, 0.30, 0.24], (speck - 0.88) * 4.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
        // a ballpoint's strokes, faded
        if hash(salt, 1, 2) < 0.5 {
            ctx.set_stroke_style("rgba(60,72,120,0.4)");
            ctx.set_line_width(1.0);
            let (x0, y0) = (w * (0.2 + 0.6 * hash(salt, 3, 4)), h * (0.2 + 0.6 * hash(salt, 5, 6)));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            for i in 1..6 {
                let fi = f64::from(i);
                ctx.line_to(x0 + fi * 3.0 + rnd() * 2.0, y0 + if i % 2 == 0 { 0.0 } else { 3.0 + rnd() * 2.0 });
            }
            ctx.stroke();
        }
    })
}

/// Moulded hair's paint (`col`): combed in fine streaks, rubbed back to the
/// vinyl on the crown and the waves' crests, dust in the grooves
fn hair_tex(salt: u32, col: u32) -> Tex {
    let p = [f64::from((col >> 16) & 255) / 255.0, f64::from((col >> 8) & 255) / 255.0, f64::from(col & 255) / 255.0];
    // the paint of a doll's hair, not a toy's: muted toward a brown
    let p = mix(p, [0.42, 0.33, 0.24], 0.35);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4a1;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 60.0, v * 6.0, 3);
            let mut c = mix(p, [p[0] * 0.6, p[1] * 0.6, p[2] * 0.6], ss(0.45, 0.75, streak) * 0.6);
            c = mix(c, [p[0] * 1.2 + 0.05, p[1] * 1.2 + 0.04, p[2] * 1.15 + 0.03], ss(0.6, 0.8, fbm(seed ^ 2, u * 30.0, v * 4.0, 2)) * 0.3);
            // rubbed through to the vinyl
            let rub = fbm(seed ^ 3, u * 16.0, v * 16.0, 3);
            c = mix(c, [0.72, 0.58, 0.48], ss(0.70, 0.78, rub) * 0.55);
            // dust in the grooves
            c = mix(c, [0.48, 0.45, 0.40], ss(0.55, 0.75, fbm(seed ^ 9, u * 4.0, v * 4.0, 2)) * 0.25);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The face part's bounds in the head's frame (x across, y up; r03.py,
/// `Real__dollFace`): its paint is laid straight on along z
const FACE_LO: (f64, f64) = (-0.0501, -0.0125);
const FACE_HI: (f64, f64) = (0.0501, 0.0961);
fn face_px(x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let u = (x - FACE_LO.0) / (FACE_HI.0 - FACE_LO.0);
    let v = (y - FACE_LO.1) / (FACE_HI.1 - FACE_LO.1);
    (u * w, (1.0 - v) * h)
}

/// The doll's face, as r03's (the same mould, the same factory's paint),
/// worn its own way: blush sprayed and rubbed off the cheeks' crowns, fine
/// brows, painted lower lashes, the lips round the bottle's hole, the
/// nostrils; dirt in the creases and specks, a grey scuff, paint rubbed
/// off the nose's tip
fn face_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 288, false, move |ctx, w, h| {
        let seed = salt ^ 0xfa;
        let k = w / (FACE_HI.0 - FACE_LO.0); // px per metre
        let px = |x: f64, y: f64| face_px(x, y, w, h);
        fill(ctx, w, h, |u, v| {
            let b = [0.84, 0.69, 0.60];
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            let mut c = mix(b, [b[0] * 0.84, b[1] * 0.78, b[2] * 0.70], ss(0.45, 0.8, n) * 0.45);
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.40, 0.33, 0.27], ss(0.6, 0.85, grime) * 0.25);
            let speck = vn(seed ^ 7, u * 80.0, v * 90.0);
            if speck > 0.88 {
                c = mix(c, [0.36, 0.30, 0.24], (speck - 0.88) * 4.0);
            }
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
            blot(ctx, cx0, cy0, 0.016 * k, "rgba(190,116,100,0.36)", "rgba(190,116,100,0)");
            blot(ctx, cx0 + s * 2.0, cy0 - 3.0, 0.006 * k, "rgba(214,180,160,0.4)", "rgba(214,180,160,0)");
            ctx.set_stroke_style("rgba(118,86,60,0.5)");
            ctx.set_line_width(1.2);
            for i in 0..9 {
                let t = f64::from(i) / 8.0;
                let x = s * (0.008 + 0.022 * t);
                let y = 0.0715 + 0.0035 * (1.0 - (2.0 * t - 0.8).powi(2));
                let (a, b) = px(x, y);
                ctx.begin_path();
                ctx.move_to(a, b + 1.0);
                ctx.line_to(a + s * 3.0, b - 1.4);
                ctx.stroke();
            }
            ctx.set_stroke_style("rgba(78,56,42,0.55)");
            ctx.set_line_width(0.9);
            for i in 0..6 {
                let t = f64::from(i) / 5.0;
                let x = s * (0.0125 + 0.0135 * t);
                let (a, b) = px(x, 0.0532 - 0.0012 * (1.0 - (2.0 * t - 1.0).powi(2)) + s * (x - s * 0.019) * 0.1);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.line_to(a + s * 1.2, b + 3.2);
                ctx.stroke();
            }
            let (ex, ey) = px(s * 0.019, 0.0585);
            ctx.set_stroke_style("rgba(80,62,48,0.4)");
            ctx.set_line_width(2.4);
            ctx.begin_path();
            ctx.ellipse(ex, ey, 0.0102 * k, 0.0060 * k, 0, 0, 7);
            ctx.stroke();
            let (a, b) = px(s * 0.0034, 0.0442);
            ctx.set_fill_style("rgba(96,58,48,0.6)");
            ctx.begin_path();
            ctx.ellipse(a, b, 1.6, 1.1, s * 0.4, 0, 7);
            ctx.fill();
        }
        // the lips, worn pale at their crowns, round the dark bottle hole
        let (mx, my) = px(0.0, 0.0333);
        ctx.set_fill_style("rgba(160,98,90,0.5)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0060 * k, 0.0051 * k, 0, 0, 7);
        ctx.fill();
        blot(ctx, mx, my - 0.0042 * k, 0.0022 * k, "rgba(214,170,150,0.5)", "rgba(214,170,150,0)");
        ctx.set_fill_style("rgba(48,24,22,0.95)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0027 * k, 0.0027 * k, 0, 0, 7);
        ctx.fill();
        // the nose's tip rubbed, a grey scuff where it lay on its face once
        let (nx, ny) = px(0.0, 0.047);
        blot(ctx, nx, ny, 0.004 * k, "rgba(232,212,196,0.45)", "rgba(232,212,196,0)");
        let sx = if hash(salt, 9, 9) < 0.5 { -1.0 } else { 1.0 };
        let (gx, gy) = px(sx * (0.020 + 0.012 * hash(salt, 1, 9)), 0.020 + 0.03 * hash(salt, 2, 9));
        blot(ctx, gx, gy, 0.012 * k, "rgba(70,64,58,0.34)", "rgba(70,64,58,0)");
    })
}

/// A sleep eye: an ivory ball yellowed, its iris (grey-blue or brown, by
/// the head) with a darker ring and streaks, the pupil black (the iris
/// faces +z: a quarter of the way along)
fn eye_tex(salt: u32) -> Tex {
    let brown = hash(salt, 7, 1) < 0.35;
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        ctx.set_fill_style("#cfc4ad");
        ctx.fill_rect(0, 0, w, h);
        let (cx0, cy0) = (w * 0.25, h * 0.5);
        let gr = ctx.create_radial_gradient(cx0, cy0, 0, cx0, cy0, 21);
        if brown {
            gr.add_color_stop(0, "#3e2c1e");
            gr.add_color_stop(0.55, "#6a4c32");
            gr.add_color_stop(0.85, "#4e3826");
            gr.add_color_stop(1, "#241a12");
        } else {
            gr.add_color_stop(0, "#3d5468");
            gr.add_color_stop(0.55, "#62809a");
            gr.add_color_stop(0.85, "#4b677e");
            gr.add_color_stop(1, "#26323e");
        }
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 21, 21, 0, 0, 7);
        ctx.fill();
        ctx.set_stroke_style("rgba(28,30,34,0.45)");
        ctx.set_line_width(1);
        for i in 0..28 {
            let a = f64::from(i) / 28.0 * 6.283;
            ctx.begin_path();
            ctx.move_to(cx0 + a.cos() * 9.0, cy0 + a.sin() * 9.0);
            ctx.line_to(cx0 + a.cos() * 19.0, cy0 + a.sin() * 19.0);
            ctx.stroke();
        }
        ctx.set_fill_style("#1c1a1a");
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 8.5, 8.5, 0, 0, 7);
        ctx.fill();
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
