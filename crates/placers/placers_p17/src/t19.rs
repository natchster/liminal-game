//! Batch `t19`'s empty picture outline, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t19.py`, its notes
//! `t19.notes.md`): where a picture hung for years and was taken down, a
//! mark, as batch `r07`'s are: the wall behind it kept cleaner and less
//! faded than round it, a dark ghost of dust and soot along its edges
//! (thickest along its top, where the warm air rising off the wall laid
//! it), and the two picture hooks it hung from, their pins still in the
//! wall; now and then the cut end of its wire still caught on a hook, or a
//! scrape in the paint below where its corner caught. It keeps the old
//! placer's draws, call for call, and its plane is the old patch's size
//! with its border (`settle_props` probes the wall across it as before).
use crate::{mount_on_wall, wall_candidates};
use game::prelude::*;

/// How far in from the decal's edge the picture's footprint begins: the
/// ghost fades out across this margin
const MARGIN: f64 = 0.025;

// R760 empty painting outline (wall): 3% chance per level. Absence-as-prop.
pub(crate) fn place_empty_painting_outlines(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let ep = group();
    // Random landscape / portrait orientation.
    let is_landscape = rnd() < 0.5;
    let outline_w = if is_landscape { 0.28 + rnd() * 0.14 } else { 0.20 + rnd() * 0.10 };
    let outline_h = if is_landscape { 0.20 + rnd() * 0.10 } else { 0.30 + rnd() * 0.14 };
    // the old patch and its 5 mm border: the decal's size
    let (pw, ph) = (outline_w + 0.010, outline_h + 0.010);
    // 40%: the cut end of its wire still caught on one hook
    let wire = rnd() < 0.4;
    // 30%: a scrape in the paint below where its corner caught
    let scrape = if rnd() < 0.3 {
        let sx = (rnd() - 0.5) * outline_w * 0.7;
        let sy = -outline_h / 2.0 - 0.03 - rnd() * 0.04;
        let rot = (rnd() - 0.5) * 0.3;
        Some((sx, sy, rot))
    } else {
        None
    };
    mount_on_wall(g, &ep, &c, 0.002, 1.60, 0.5);
    let p = ep.pos();
    let k = salt(p.x, p.z, 37);
    let decal = |tex: &Tex| lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let ghost = mesh(plane_geo!(pw, ph), &decal(&ghost_tex(k, pw, ph)));
    ghost.position().set(0, 0, -0.0015);
    ep.add(&ghost);
    // the two picture hooks at the old hooks' places, their pins in the wall
    let brass = phong!({ color: 0x6a5838, specular: 0x4a4030, shininess: 36 });
    let pin = phong!({ color: 0x3c3a36, specular: 0x3a3a38, shininess: 40 });
    for side in [-1.0, 1.0] {
        let hook = group();
        hook.position().set(side * outline_w * 0.32, outline_h * 0.34, -0.002);
        put(&hook, "Real__outlineHook", &brass);
        put(&hook, "Real__outlinePin", &pin);
        if wire && side < 0.0 {
            put(&hook, "Real__outlineWire", &phong!({ color: 0x4a4640, specular: 0x3a3834, shininess: 30 }));
        }
        ep.add(&hook);
    }
    if let Some((sx, sy, rot)) = scrape {
        let s = mesh(plane_geo!(0.015, 0.06), &decal(&scrape_tex(k)));
        s.position().set(sx, sy, -0.0015);
        s.rotation().z = rot;
        ep.add(&s);
    }
    g.world.add(&ep);
}

/// The mark (the decal `pw` x `ph` m): inside the picture's footprint the
/// wall a shade lighter and fresher than the faded wall round it, its edge
/// sharp; outside it a ghost of grey-brown dust along each edge, darkest
/// and widest along the top, faint along the foot, fading into the wall
fn ghost_tex(salt: u32, pw: f64, ph: f64) -> Tex {
    let ppm = 500.0;
    canvas_tex_opt((pw * ppm).round(), (ph * ppm).round(), false, move |ctx, w, h| {
        let seed = salt ^ 0x9d;
        let (a, b) = (pw / 2.0 - MARGIN, ph / 2.0 - MARGIN);
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * pw, (v - 0.5) * ph);
            let (ox, oy) = (x.abs() - a, y.abs() - b);
            // signed distance from the footprint's edge (m, outside > 0)
            let d = if ox > 0.0 && oy > 0.0 { (ox * ox + oy * oy).sqrt() } else { ox.max(oy) };
            let edge = (pw / 2.0 - x.abs()).min(ph / 2.0 - y.abs());
            let fade = sstep(0.0, 0.008, edge);
            let mottle = fbm(seed, x * 30.0, y * 30.0, 3);
            if d < 0.0 {
                // the fresher wall: lighter, its edge as sharp as the frame's
                let k = sstep(0.0, 0.0012, -d);
                let al = (0.10 + 0.05 * mottle) * k;
                return [0.93, 0.90, 0.83, al];
            }
            let top = oy > ox && y > 0.0;
            let foot = oy > ox && y < 0.0;
            let (line, wide) = if top { (0.34, 0.010) } else if foot { (0.10, 0.004) } else { (0.18, 0.006) };
            let ghost = line * (-d / 0.0035).exp() + 0.10 * (-d / wide).exp();
            let al = ghost * (0.55 + 0.7 * mottle) * fade;
            [0.22, 0.19, 0.15, al.min(0.6)]
        });
    })
}

/// A scrape through the paint to the plaster, its edges grimed, thinning
/// to nothing at its ends
fn scrape_tex(salt: u32) -> Tex {
    canvas_tex_opt(24, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0xb5;
        fill(ctx, w, h, |u, v| {
            let mid = 0.5 + 0.12 * (fbm(seed, v * 3.0, 0.5, 2) - 0.5);
            let width = 0.10 * (v * PI).sin().max(0.0).powf(0.7);
            let d = (u - mid).abs();
            if d < width * 0.55 {
                return [0.84, 0.81, 0.74, 0.75 * sstep(0.0, 0.15, v.min(1.0 - v))];
            }
            let lip = 1.0 - sstep(width * 0.55, width * 1.4, d);
            [0.24, 0.20, 0.16, 0.4 * lip * (v * PI).sin()]
        });
    })
}

/// Value noise, a pixel fill on every core (pure), a salt from where a
/// thing is, a part placed (as batch `t19`'s other painters)
fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}
fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
/// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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
fn salt(x: f64, z: f64, k: u32) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
}
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
