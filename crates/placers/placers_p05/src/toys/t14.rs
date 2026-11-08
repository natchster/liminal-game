//! Batch `t14`'s bean-bag elephant, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t14.py`, its notes
//! `t14.notes.md`): its placer and what only it uses. It keeps the old
//! placer's draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use t14_kit::{fbm, fill, mix, put, rgb, salt, ss, vn};

/// The ears' roots on the head's sides (`t14.py` EL_EAR, x either side)
const EL_EAR: (f64, f64, f64) = (0.0290, 0.058, 0.052);
/// How far each ear stands out from the head at rest (radians)
const EAR_OUT: f64 = 0.16;

/// The old body colours, and the plush each stands for (sRGB): a powder
/// blue, a lilac, a fawn, a grey; all faded
const BODY_COLORS: [u32; 4] = [0x8090a0, 0xa090b0, 0xc0a080, 0x606870];
const PLUSH: [u32; 4] = [0x8995a2, 0x9b90a6, 0xb19e80, 0x6c7178];

// Stuffed elephant: 3% chance per level. A bean-bag plush elephant of
// the 1990s, 19 cm long (short plush on a knit backing, its body loosely
// filled with plastic pellets so it lies limp, its head with fibre; big
// floppy ears of two layers, plastic eyes, a thin tail, a law tag sewn in
// at its rear), lying on its belly with its legs splayed. Faded, its plush
// rubbed and matted, grime ground in where it lay. Its ears stir a little
// now and then.
pub fn place_stuffed_elephants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let col = pick(&BODY_COLORS);
    let k = BODY_COLORS.iter().position(|&c| c == col).unwrap_or(0);
    let base = rgb(PLUSH[k]);
    let eleph = group();
    // the ears on their hinges (the flap animator turns them); their phases
    // are the old ones' draws
    let mut eleph_ears: Vec<(Obj, f64, f64, f64)> = vec![];
    let mut ear_names = vec![];
    for side in [-1.0f64, 1.0] {
        let ear = group();
        ear.position().set(side * EL_EAR.0, EL_EAR.1, EL_EAR.2);
        // out from the head: the ear's back edge away from it
        let base_y = -side * EAR_OUT;
        ear.rotation().y = base_y;
        let phase = rnd() * PI * 2.0 + (if side > 0.0 { 0.7 } else { 0.0 });
        {
            let mut u = ear.user();
            u.base_y = base_y;
            u.set("side", side);
            u.phase = phase;
        }
        eleph_ears.push((ear.clone(), base_y, side, phase));
        ear_names.push((ear, if side > 0.0 { "Real__elephantEarL" } else { "Real__elephantEarR" }));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let s = salt(px, pz, 635);
    let fabric = lambert!({ map: &plush_tex(s, base) });
    put(&eleph, "Real__elephantBody", &fabric);
    put(&eleph, "Real__elephantPile", &lambert!({ map: &pile_tex(s, base), alphaTest: 0.5 }));
    for (ear, name) in &ear_names {
        put(ear, name, &fabric);
        eleph.add(ear);
    }
    put(&eleph, "Real__elephantEyes", &phong!({ color: 0x1c1a19, specular: 0x4a4a4a, shininess: 70 }));
    put(&eleph, "Real__elephantTag", &lambert!({ map: &tag_tex(), side: DoubleSide }));
    eleph.position().set(px, 0, pz);
    eleph.rotation().y = rnd() * PI * 2.0;
    g.world.add(&eleph);
    let p = eleph.pos();
    g.pp.plush_positions.push(v3(p.x, 0, p.z));
    // The ears stir: a slow drift, now and then a stronger flap, always out from the head
    g.animate(move |_g, _dt, t| {
        for (ear, base, side, phase) in &eleph_ears {
            let drift = (t * 0.35 + phase).sin() * 0.12;
            let flap = (t * 0.18 + phase * 1.3).sin();
            let big_flap = if flap > 0.92 { ((flap - 0.92) / 0.08 * PI).sin() * 0.30 } else { 0.0 };
            ear.rotation().y = base - side * (drift.max(-0.12) + 0.12 + big_flap) * 0.9;
        }
    });
}

/// Short plush (`b`): its pile in fine streaks, rubbed flat and shiny in
/// patches, faded, grime ground in toward its belly
fn plush_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xe1e;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 150.0, v * 80.0, 3);
            let mut c = mix(b, [b[0] * 0.74, b[1] * 0.72, b[2] * 0.70], ss(0.42, 0.72, streak) * 0.45);
            let tip = vn(seed ^ 2, u * 240.0, v * 200.0);
            c = mix(c, [b[0] * 1.12 + 0.03, b[1] * 1.1 + 0.03, b[2] * 1.08 + 0.03], ss(0.7, 0.95, tip) * 0.35);
            c = mix(c, [b[0] * 0.84, b[1] * 0.82, b[2] * 0.80], matted(seed, u, v) * 0.6);
            let fade = fbm(seed ^ 3, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.66, 0.64, 0.60], ss(0.5, 0.8, fade) * 0.3);
            // grime: from below, where it lay, and in blotches
            let grime = fbm(seed ^ 6, u * 4.0, v * 4.0, 3) + (0.35 - v).max(0.0) * 0.6;
            c = mix(c, [0.34, 0.31, 0.28], ss(0.55, 0.85, grime) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Where the plush is rubbed flat (0 to 1)
fn matted(seed: u32, u: f64, v: f64) -> f64 {
    ss(0.6, 0.72, fbm(seed ^ 4, u * 6.0, v * 6.0, 4))
}

/// The pile's tips (the stippled layer): short, sparse, none where rubbed
fn pile_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xe1e;
        fill(ctx, w, h, |u, v| {
            let n = vn(seed ^ 0x31, u * 190.0, v * 170.0);
            let m = fbm(seed ^ 0x32, u * 20.0, v * 20.0, 2);
            let a = if n > 0.66 - 0.1 * (m - 0.5) + matted(seed, u, v) * 0.6 { 1.0 } else { 0.0 };
            let c = mix([b[0] * 1.08 + 0.02, b[1] * 1.06 + 0.02, b[2] * 1.05 + 0.02], [0.6, 0.58, 0.55], 0.12 + 0.2 * m);
            [c[0], c[1], c[2], a]
        });
    })
}

/// The law tag: white satin gone grey, its print in black and a faded
/// red, the usual words of a stuffed toy's tag (the maker's name worn
/// off), frayed at its fold
fn tag_tex() -> Tex {
    canvas_tex_opt(96, 192, false, |ctx, w, h| {
        ctx.set_fill_style("#c9c4b8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(150,60,56,0.7)");
        ctx.fill_rect(0, 0, w, 22);
        ctx.set_fill_style("rgba(40,38,36,0.85)");
        ctx.set_font("bold 9px Helvetica Neue");
        let lines = [
            "ALL NEW MATERIAL",
            "CONSISTING OF",
            "POLYESTER FIBER",
            "& P.V.C. PELLETS",
            "REG. NO PA. 2287",
            "MADE IN CHINA",
            "SURFACE WASHABLE",
        ];
        for (i, l) in lines.iter().enumerate() {
            ctx.fill_text(l, 6, 36.0 + i as f64 * 15.0);
        }
        // grime and fraying
        for _ in 0..60 {
            ctx.set_fill_style(format!("rgba(90,84,74,{:.2})", 0.1 + rnd() * 0.25));
            ctx.fill_rect(rnd() * w, rnd() * h, 2.0 + rnd() * 6.0, 1.0 + rnd() * 3.0);
        }
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
