//! What several nature placers share.
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mark_mat;
use placers_kit::paint::paint_px;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_tex {
    ($w:expr_2021, $h:expr_2021, $repeat:expr_2021, $paint:expr_2021) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

/// `rgba(…)` of an sRGB colour (0-1) at alpha `a`
pub(crate) fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
pub(crate) fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &css(c, a));
    gr.add_color_stop(1, &css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A thin part (grass blades, stems) lit as one: drawn from both sides, its
/// normals turned up toward the light above, so that the side turned from
/// a lamp does not go black (a blade lets the light through)
pub(crate) fn thin_lit(name: &str) -> Option<Geo> {
    thread_local! { static C: RefCell<Vec<(String, Geo)>> = const { RefCell::new(vec![]) }; }
    if let Some(g) = C.with(|c| c.borrow().iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())) {
        return Some(g);
    }
    let g = part(name)?.duplicate();
    {
        let mut gg = g.g();
        for n in gg.normal.as_chunks_mut::<3>().0 {
            let (x, y, z) = (n[0] * 0.35, n[1].abs() * 0.35 + 0.75, n[2] * 0.35);
            let l = (x * x + y * y + z * z).sqrt().max(1e-6);
            n.copy_from_slice(&[x / l, y / l, z / l]);
        }
        if let Some(idx) = gg.index.as_mut() {
            let back: Vec<u32> = idx.as_chunks::<3>().0.iter().flat_map(|t| [t[0], t[2], t[1]]).collect();
            idx.extend(back);
        }
        gg.version += 1;
    }
    C.with(|c| c.borrow_mut().push((name.to_string(), g.clone())));
    Some(g)
}

/// A painted mark lying flat on the floor, `w` × `d` m, its middle at (x, z)
pub(crate) fn floor_mark(tex: &Tex, w: f64, d: f64, x: f64, z: f64, turn: f64) -> Obj {
    let m = mesh(plane_geo!(w, d), mark_mat(tex));
    m.rotation().x = -PI / 2.0;
    m.rotation().z = turn;
    m.position().set(x, 0.0015, z);
    m
}

/// Oak bark on a one-year twig (u along it): grey-brown, fine streaks along
/// it, pale lenticels (short dashes), a bloom of grey-green algae here and
/// there; one texture for every twig, laid along each one's length
pub(crate) fn bark_tex() -> Tex {
    cached_tex!(256, 64, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w, y / h);
            let streak = fbm(0x0b1, u * 40.0, v * 3.0, 3);
            let mut c = mix_unclamped([0.50, 0.44, 0.37], [0.36, 0.31, 0.26], streak);
            // lenticels: short pale dashes across the twig
            let (cx, cy) = ((u * 70.0).floor(), (v * 9.0).floor());
            let hh = hash(0x1e7, cx as i32, cy as i32);
            if hh > 0.72 {
                let (fx, fy) = (u * 70.0 - cx - 0.5, v * 9.0 - cy - 0.5);
                let d = (fx / 0.18).hypot(fy / 0.42);
                c = mix_unclamped(c, [0.66, 0.60, 0.50], 0.8 * (1.0 - sstep(0.6, 1.0, d)));
            }
            // algae and lichen, grey-green, dry
            let alg = sstep(0.62, 0.75, fbm(0x2a9, u * 9.0, v * 2.0, 4));
            c = mix_unclamped(c, [0.46, 0.48, 0.40], 0.5 * alg);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Part `name` with texture coordinates of its own: `f(p, n)` gives each
/// vertex's (u, v) from where it is and which way it faces in the part's
/// frame. Where `wrap_u`, u runs round the thing (0..1): a triangle across
/// the seam gets its low corners again at u + 1. Made once a thread.
pub(crate) fn uv_part(name: &str, wrap_u: bool, f: impl Fn([f64; 3], [f64; 3]) -> (f64, f64)) -> Option<Geo> {
    thread_local! { static C: RefCell<Vec<(String, Geo)>> = const { RefCell::new(vec![]) }; }
    if let Some(g) = C.with(|c| c.borrow().iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())) {
        return Some(g);
    }
    let g = part(name)?.duplicate();
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        let mut pos = gg.position.clone();
        let mut nrm = gg.normal.clone();
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let p = [f64::from(pos[i * 3]), f64::from(pos[i * 3 + 1]), f64::from(pos[i * 3 + 2])];
            let q = [f64::from(nrm[i * 3]), f64::from(nrm[i * 3 + 1]), f64::from(nrm[i * 3 + 2])];
            let (u, v) = f(p, q);
            uv.push(u as f32);
            uv.push(v as f32);
        }
        let mut idx: Vec<u32> = gg.index.clone().unwrap_or_else(|| (0..n as u32).collect());
        if wrap_u {
            let mut dup: Vec<(u32, u32)> = vec![];
            for t in idx.as_chunks_mut::<3>().0 {
                let us = [uv[t[0] as usize * 2], uv[t[1] as usize * 2], uv[t[2] as usize * 2]];
                let (lo, hi) = (us.iter().copied().fold(9.0f32, f32::min), us.iter().copied().fold(-9.0f32, f32::max));
                if hi - lo <= 0.5 {
                    continue;
                }
                for k in t.iter_mut() {
                    if uv[*k as usize * 2] < 0.5 {
                        let d = match dup.iter().find(|(a, _)| *a == *k) {
                            Some(&(_, d)) => d,
                            None => {
                                let d = (pos.len() / 3) as u32;
                                let s = *k as usize;
                                pos.extend_from_slice(&[pos[s * 3], pos[s * 3 + 1], pos[s * 3 + 2]]);
                                nrm.extend_from_slice(&[nrm[s * 3], nrm[s * 3 + 1], nrm[s * 3 + 2]]);
                                uv.extend_from_slice(&[uv[s * 2] + 1.0, uv[s * 2 + 1]]);
                                dup.push((*k, d));
                                d
                            }
                        };
                        *k = d;
                    }
                }
            }
        }
        gg.position = pos;
        gg.normal = nrm;
        gg.uv = uv;
        gg.index = Some(idx);
        gg.touch();
    }
    C.with(|c| c.borrow_mut().push((name.to_string(), g.clone())));
    Some(g)
}

/// Things laid in a cluster keep a little apart: `at` moved out from those
/// already down (each `r` round) until it is clear of them (no draw)
pub(crate) fn clear_of(at: (f64, f64), r: f64, down: &[(f64, f64, f64)]) -> (f64, f64) {
    let (mut x, mut z) = at;
    for _ in 0..6 {
        let mut moved = false;
        for &(ox, oz, or) in down {
            let (dx, dz) = (x - ox, z - oz);
            let d = dx.hypot(dz);
            let need = (r + or) * 0.92;
            if d < need {
                let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (0.6, 0.8) };
                x = ox + ux * need;
                z = oz + uz * need;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    (x, z)
}

/// Fill the canvas pixel by pixel: `f(x, y)` (pixel centres) gives a
/// straight sRGB colour. Pure (no draws, no canvas calls), so its rows are
/// filled on every core
pub(crate) fn paint_px_rgb(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let c = f(i as f64 + 0.5, j as f64 + 0.5);
            for k in 0..3 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// fbm that repeats every `p` (whole numbers) in x and y
pub(crate) fn fbm_tile(seed: u32, x: f64, y: f64, p: f64, oct: u32) -> f64 {
    let (u, v) = (x / p, y / p);
    let a = fbm(seed, x, y, oct);
    let b = fbm(seed, x - p, y, oct);
    let c = fbm(seed, x, y - p, oct);
    let d = fbm(seed, x - p, y - p, oct);
    let (fu, fv) = (u.fract(), v.fract());
    (a * (1.0 - fu) + b * fu) * (1.0 - fv) + (c * (1.0 - fu) + d * fu) * fv
}

/// What is painted already, changed pixel by pixel on every core: `f(u, v,
/// rgb)` (as [`fill`]) gives the new colour
pub(crate) fn over(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64, [f64; 3]) -> [f64; 3] + Sync) {
    let iw = w as usize;
    let mut img = ctx.get_image_data(0, 0, w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = [f64::from(out[i * 4]) / 255.0, f64::from(out[i * 4 + 1]) / 255.0, f64::from(out[i * 4 + 2]) / 255.0];
            let o = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h, c);
            for k in 0..3 {
                out[i * 4 + k] = (o[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}
