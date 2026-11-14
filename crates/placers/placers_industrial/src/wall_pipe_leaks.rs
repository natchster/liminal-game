//! `place_wall_pipe_leaks`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::noise::Rng32;
use placers_kit::paint::decal_tex;
use placers_kit::paint::lay;
use placers_kit::paint::mark_mat;
use placers_kit::place::mount;
use placers_kit::place::seed_of;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall pipe leak (`docs/research/props-remodelling/wall_marks_and_wear.md`): a short
// run of 2 in steel pipe out of the wall, along it and back in (batch r06's
// malleable elbows, a coupling, the collars at the wall), painted once and
// rusting; one elbow's thread by the wall weeps: a crust of lime and rust
// under it, a drop hanging, and down the wall from its collar the wet run,
// rust-brown at its edges, and the stain where it spread and dried.
pub(crate) fn place_wall_pipe_leaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let leak = group();
    // the old pipe's axis 2.6 cm off the wall: the group's origin stays there
    let (wall_x, wall_z) = wall_xz(g, c, 0.026);
    mount(&leak, c, wall_x, wall_z, 1.90, Some(0.3));
    let p = leak.pos();
    let mut r = Rng32(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    // the frame: the wall's face at z = WALL; the run's axis 85 mm off it, as
    // high as keeps the collars' tops where the old pipe's top was; its
    // elbows' collars at the old pipe's ends
    const WALL: f64 = -0.026;
    const EA: f64 = 0.057; // a 2 in elbow, centre to end
    const IN: f64 = 0.017; // a thread into its fitting
    let (zp, ya, xe) = (WALL + 0.085, 0.025927 - 0.060, 0.300 - 0.0575);
    let paint = (r.f() * 3.0) as usize % 3;
    let pipe_mat = lambert!({ map: &pipe_paint_tex(paint, false), color: 0xffffff });
    let fit_mat = lambert!({ map: &pipe_paint_tex(paint, true), color: 0xffffff });
    let plate = phong!({ color: 0x77766f, specular: 0x262626, shininess: 26 });
    let part = |name: &str, w: f64| game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, w, w));
    // the run, between the elbows, through a coupling
    let xc = (r.f() - 0.5) * 0.12;
    leak.add(&pipe_length(&pipe_mat, v3(-xe + EA - IN, ya, zp), v3(xc - 0.015, ya, zp)));
    leak.add(&pipe_length(&pipe_mat, v3(xc + 0.015, ya, zp), v3(xe - EA + IN, ya, zp)));
    let cp = mesh(part("Real__pipeCoupling", 0.064), &fit_mat);
    cp.position().set(xc, ya, zp);
    leak.add(&cp);
    for sx in [-1.0, 1.0] {
        let x = sx * xe;
        // the elbow (its legs +x and +y as modelled): one leg along the run,
        // the other back into the wall
        let el = group();
        el.position().set(x, ya, zp);
        el.rotation().y = if sx < 0.0 { 0.0 } else { PI };
        let m = mesh(part("Real__pipeElbow", 0.08), &fit_mat);
        // its +y leg to -z (into the wall); the right one is turned round
        // after (its +x leg to -x), so its +y leg goes to +z first
        m.rotation().x = sx * PI / 2.0;
        el.add(&m);
        leak.add(&el);
        leak.add(&pipe_length(&pipe_mat, v3(x, ya, zp - EA + IN), v3(x, ya, WALL)));
        let col = mesh(part("Real__pipeCollar", 0.115), &plate);
        col.position().set(x, ya, WALL);
        leak.add(&col);
    }
    // the leaking one, at its thread by the wall
    let side = if r.f() < 0.5 { -1.0 } else { 1.0 };
    let lx = side * xe;
    let crust = mesh(part("Real__pipeLeakCrust", 0.03), &lambert!({ color: 0x9c8466 }));
    crust.position().set(lx, ya - 0.0385, zp - EA + 0.006);
    crust.rotation().y = r.f() * 0.6 - 0.3;
    leak.add(&crust);
    let drop = mesh(
        part("Real__pipeLeakDrop", 0.006),
        &phong!({ color: 0x4a5256, specular: 0xd8dcdc, shininess: 120, transparent: true, opacity: 0.55 }),
    );
    drop.position().set(lx - 0.003, ya - 0.0385 - 0.0128, zp - EA + 0.007);
    leak.add(&drop);
    // down the wall from its collar: the wet run and its stain, to where
    // the old stain's foot was
    let (top, bot) = (ya + 0.03, -0.26);
    let (w, hgt) = (0.10, top - bot);
    let run = mesh(plane_geo!(w, hgt), &mark_mat(&pipe_leak_tex(r.u(), w, hgt, top - (ya - 0.0575))));
    run.position().set(lx, (top + bot) / 2.0, WALL + 0.001);
    leak.add(&run);
    g.world.add(&leak);
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// An opaque, repeating surface `tw` x `th` px over `pw` x `ph` m: each
/// pixel `px(x, y)` (m from its top left) in sRGB
fn surface_tex(tw: usize, th: usize, pw: f64, ph: f64, px: impl Fn(f64, f64) -> [f64; 4] + Sync) -> Tex {
    canvas_tex_opt(tw as f64, th as f64, true, move |ctx, cw, ch| {
        let (sx, sy) = (pw / cw, ph / ch);
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, tw * 4, |j, out| {
            let y = (j as f64 + 0.5) * sy;
            for i in 0..tw {
                let p = px((i as f64 + 0.5) * sx, y);
                for (c, v) in p.iter().enumerate() {
                    out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A length of 2 in pipe (60.3 mm) from `a` to `b` along one axis, its
/// paint wrapped round it at its real scale.
fn pipe_length(mat: &Mat, a: Vec3, b: Vec3) -> Obj {
    let d = b - a;
    let len = d.length().max(0.001);
    let geo = cyl_geo!(0.03015, 0.03015, len, 16, 1, true);
    let uv: Vec<f32> = geo.r().uv.clone();
    // the texture is 0.4 m along
    let uv: Vec<f32> = uv.iter().enumerate().map(|(i, &v)| if i % 2 == 1 { v * (len / 0.4) as f32 } else { v }).collect();
    geo.set_uv(uv);
    let m = mesh(geo, mat);
    m.position().set((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0);
    if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
        m.rotation().z = PI / 2.0;
    } else if d.z.abs() > d.y.abs() {
        m.rotation().x = PI / 2.0;
    }
    m
}

/// A pipe's old paint (`paint`: grey, green, cream) flaked off its steel
/// in patches, rust in them and blooming round them, a run of rust along
/// its underside, dust on its top; a fitting's (`fitting`) rustier.
/// Round it 0.19 m, along it 0.4 m (one per kind, shared).
fn pipe_paint_tex(paint: usize, fitting: bool) -> Tex {
    thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; 6]); }
    let i = paint % 3 + if fitting { 3 } else { 0 };
    T.with(|t| {
        if let Some(x) = &t.borrow()[i] {
            return x.clone();
        }
        const PAINTS: [[f64; 3]; 3] = [[0.37, 0.38, 0.37], [0.34, 0.38, 0.31], [0.6, 0.57, 0.48]];
        let pc = PAINTS[paint % 3];
        let rusty = if fitting { 0.56 } else { 0.62 };
        let tex = surface_tex(96, 256, 0.19, 0.4, move |x, y| {
            let seed = 0x9e3 + paint as u32 * 31 + u32::from(fitting) * 7;
            // round the pipe: x 0-0.19 (a run's underside at 0.1425, its
            // top at 0.0475), along it: y
            let under = (-((x - 0.1425) / 0.03).powi(2)).exp();
            let top = (-((x - 0.0475) / 0.03).powi(2)).exp();
            let fl = fbm(seed, x * 70.0, y * 70.0, 4);
            let bare = sstep(rusty, rusty + 0.03, fl + 0.12 * under);
            let bloom = sstep(rusty - 0.12, rusty, fl + 0.12 * under) * (1.0 - bare);
            let rust = mix([0.24, 0.14, 0.08], [0.38, 0.21, 0.1], fbm(seed ^ 0x3, x * 300.0, y * 300.0, 2));
            let mut c = mix(pc, rust, bare);
            c = mix(c, [0.36, 0.22, 0.12], 0.4 * bloom);
            c = mix(c, [0.32, 0.19, 0.1], 0.35 * under);
            // grime
            c = mix(c, [0.2, 0.18, 0.15], 0.25 * fbm(seed ^ 0x7, x * 25.0, y * 25.0, 3));
            c = mix(c, [0.52, 0.5, 0.45], 0.25 * top);
            let m = 0.92 + 0.12 * fbm(seed ^ 0x5, x * 140.0, y * 30.0, 2);
            [c[0] * m, c[1] * m, c[2] * m, 1.0]
        });
        t.borrow_mut()[i] = Some(tex.clone());
        tex
    })
}

/// The run down the wall under a leaking collar, across a plane `pw` x `ph`
/// (y down from its top; the collar's foot `from` down): wet and dark under
/// the collar, rust at its edges, narrowing, then where it spread and
/// dried, a stain with its tide lines.
fn pipe_leak_tex(seed: u32, pw: f64, ph: f64, from: f64) -> Tex {
    let mut r = Rng32(seed | 1);
    let s = r.u();
    const WET: [f64; 3] = [0.22, 0.16, 0.09];
    const STAIN: [f64; 3] = [0.5, 0.37, 0.2];
    const RUST: [f64; 3] = [0.45, 0.25, 0.1];
    const TIDE: [f64; 3] = [0.3, 0.18, 0.08];
    let (tw, th) = (96usize, 288usize);
    let sx = pw / tw as f64;
    let mid = pw / 2.0 + (r.f() - 0.5) * 0.008;
    // how far down it got below the collar's foot (m), in tongues: (x,
    // half width, depth), the deepest first
    let reach = ph - from - 0.02 - 0.03 * r.f();
    let lobes: Vec<(f64, f64, f64)> = (0..2 + (r.f() * 2.0) as usize)
        .map(|k| {
            if k == 0 {
                (mid + (r.f() - 0.5) * 0.008, 0.024 + 0.01 * r.f(), reach)
            } else {
                (mid + (r.f() - 0.5) * 0.05, 0.008 + 0.01 * r.f(), reach * (0.35 + 0.45 * r.f()))
            }
        })
        .collect();
    // each wetting's reach at each column, and its edge's slope; the
    // last (outermost) first, the earlier ones inside it
    let rims: Vec<f64> = vec![1.0, 0.78 + 0.08 * r.f(), 0.55 + 0.1 * r.f()];
    let reach_at = |x: f64, k: usize| -> f64 {
        let mut l = 0.004f64;
        for &(c, half, depth) in &lobes {
            let u = (x - c) / half;
            if u.abs() < 1.0 {
                let tongue = depth * rims[k] * (1.0 - u.abs().powf(2.2)).powf(0.55);
                let kk = 0.012;
                let hh = (0.5 + 0.5 * (tongue - l) / kk).clamp(0.0, 1.0);
                l = l + (tongue - l) * hh + kk * hh * (1.0 - hh);
            }
        }
        let wav = (fbm(s ^ (0x40 + k as u32), x * 40.0, 0.5, 3) - 0.5) * 0.03 + (vn(s ^ 0x41, x * 160.0, k as f64 * 3.0) - 0.5) * 0.004;
        (l + wav * (l / 0.03).min(1.0)).max(0.0)
    };
    let cols: Vec<Vec<(f64, f64)>> = (0..rims.len())
        .map(|k| {
            (0..tw)
                .map(|i| {
                    let x = (i as f64 + 0.5) * sx;
                    let (l0, l1, l) = (reach_at(x - sx, k), reach_at(x + sx, k), reach_at(x, k));
                    (l, (1.0 + ((l1 - l0) / (2.0 * sx)).powi(2)).sqrt())
                })
                .collect()
        })
        .collect();
    decal_tex(tw, th, pw, ph, STAIN, move |x, y, aa| {
        let mut px = [STAIN[0], STAIN[1], STAIN[2], 0.0];
        let i = ((x / sx) as usize).min(tw - 1);
        // the damp round the hole behind the collar, seen past its rim
        let hole = (x - pw / 2.0).hypot(y - (from - 0.0575)) / 0.06;
        lay(&mut px, WET, 0.22 * sstep(1.25, 0.92, hole) * sstep(0.01, 0.03, y) * (0.6 + 0.4 * fbm(s ^ 0x1, x * 80.0, y * 80.0, 2)));
        let dy = y - from;
        // the wetted patch hanging from the collar's foot, and its rims:
        // sharp outside, what the water carried left along them
        let fine = (vn(s ^ 0x51, x * 260.0, y * 260.0) - 0.5) * 0.0015;
        let (l, slope) = cols[0][i];
        let top = (-dy - 0.006).max(0.0);
        let d0 = (dy - l) / slope + fine + top * 4.0 * sstep(0.012, 0.02, (x - mid).abs());
        let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
        let pale = 0.55 + 0.45 * (d0 / 0.012).exp();
        lay(&mut px, STAIN, inside * 0.45 * pale * (0.65 + 0.35 * fbm(s ^ 0x52, x * 50.0, y * 50.0, 2)));
        let tide = |d: f64| (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / 0.0018).exp();
        let along = |k: u32| 0.5 + 0.5 * fbm(s ^ (0x60 + k), x * 30.0, y * 12.0, 2);
        let mut lines = tide(d0) * along(0);
        for k in 1..rims.len() {
            let (lk, sk) = cols[k][i];
            lines = 1.0 - (1.0 - lines) * (1.0 - tide((dy - lk) / sk + fine) * 0.7 * along(k as u32) * inside);
        }
        lay(&mut px, TIDE, lines * 0.7);
        // still wet down its middle under the leak, rust at its edges
        let tt = (dy / reach).clamp(0.0, 1.0);
        let rw = 0.0045 * (1.0 - 0.5 * tt) * (0.6 + 0.8 * fbm(s ^ 0x6, 1.5, y * 20.0, 2));
        let xr = mid + (fbm(s ^ 0x7, 0.5, y * 11.0, 3) - 0.5) * 0.012 * tt;
        let dr = ((x - xr).abs() - rw).max(dy - l * 0.9).max(-dy - 0.004);
        lay(&mut px, WET, (0.5 - dr / aa).clamp(0.0, 1.0) * (0.4 - 0.2 * tt));
        lay(&mut px, RUST, 0.4 * sstep(-0.002, 0.0, dr) * sstep(0.0012, 0.0, dr));
        // nothing at the plane's edges
        px[3] *= sstep(0.0, 0.006, x.min(pw - x).min(y).min(ph - y));
        px
    })
}
