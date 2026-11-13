//! Batch `r13a`'s props in this crate, made real (`docs/design/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Cracked wall mirror: 4% chance per level. A small mirror in a wooden
// frame, struck and cracked (batch r13a).
pub(crate) fn place_cracked_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // A 12 x 16 in mirror in a stained half-round moulding (Blender batch
    // `r13a`), hung from one nail by the sawtooth hanger on its back, so it
    // hangs a little askew: silvered glass that gives back the room's light
    // (`Patch::Glass`), its silver going black from the edges, struck once
    // and cracked from the blow, dust on it (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let mirror = group();
    mount_on_wall(g, &mirror, &c, 0.014, 1.65, 0.6);
    // the old tilt's draw: askew on its nail
    let tilt = (rnd() - 0.5) * 0.06;
    mirror.rotation().z = tilt;
    let p = mirror.pos();
    let vary = cm_vary(p.x, p.z, tilt);
    // the parts are modelled with the wall's face at z = 0: 14 mm behind the spot
    let body = group();
    body.position().z = -0.014;
    mirror.add(&body);
    if let Some(geo) = prop_part_as_modelled("Real__crackedMirrorFrame") {
        let tex = moulding_grain_tex(CM_GLASS[0] + 2.0 * CM_MOULD, CM_GLASS[1] + 2.0 * CM_MOULD, CM_MOULD, [0x4a, 0x2e, 0x1c], vary, 900.0);
        body.add(&mesh(geo, lambert!({ map: &tex })));
    }
    let brk = CmBreak::struck(CM_GLASS[0], CM_GLASS[1], u64::from(vary) << 7 | 0x2d);
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(CM_GLASS[0] + 0.01, CM_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
    back.position().z = 0.0040;
    body.add(&back);
    let glass = mesh(plane_geo!(CM_GLASS[0] + CM_UNDER, CM_GLASS[1] + CM_UNDER), cm_glass_mat(cm_silver_tex(&brk, vary)));
    glass.position().z = CM_GLASS[2] - 0.0001;
    glass.set_render_order(2);
    body.add(&glass);
    let film_mat = lambert!({
        map: &cm_film_tex(&brk, vary), transparent: true, depthWrite: false,
        polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
    });
    let film = mesh(plane_geo!(CM_GLASS[0], CM_GLASS[1]), &film_mat);
    film.position().z = CM_GLASS[2] + 0.0003;
    film.set_render_order(3);
    body.add(&film);
    g.world.add(&mirror);
}

/// The small mirror's glass as seen in its frame (w, h) and its face's
/// distance from the wall (`Real__crackedMirrorFrame`, r13a.py)
const CM_GLASS: [f64; 3] = [0.300, 0.400, 0.0080];

/// Its moulding's width, and how much more glass runs under its lip (both sides)
const CM_MOULD: f64 = 0.024;

const CM_UNDER: f64 = 0.010;

use placers_kit::place::vary as cm_vary;

/// A mirror's breakage, in metres on its glass (x right and y up from its
/// middle): each crack's points and weight (1 a fine one), where it was
/// struck and how wide the crushed spot is. From a seed (no level draws), so
/// the silver and the film agree
#[derive(Clone, Default)]
struct CmBreak {
    lines: Vec<(Vec<[f64; 2]>, f64)>,
    hits: Vec<([f64; 2], f64)>,
    /// the radial cracks in turn round the blow
    radials: Vec<Vec<[f64; 2]>>,
}

impl CmBreak {
    /// struck once: radial cracks from the blow to the edges (some stopping
    /// short, a few branching), rings between them near it
    fn struck(gw: f64, gh: f64, seed: u64) -> CmBreak {
        let (hw, hh) = (gw / 2.0, gh / 2.0);
        let mut r = SplitMix64::new(0, seed);
        let mut b = CmBreak::default();
        let inside = |p: [f64; 2]| p[0].abs() < hw + 0.003 && p[1].abs() < hh + 0.003;
        let run = |r: &mut SplitMix64, p: [f64; 2], a: f64, reach: f64| -> Vec<[f64; 2]> {
            let mut pts = vec![p];
            let (mut q, mut a, mut l) = (p, a, 0.0);
            while l < reach && inside(q) {
                let step = r.range(0.008, 0.022);
                a += r.range(-0.12, 0.12);
                q = [q[0] + a.cos() * step, q[1] + a.sin() * step];
                l += step;
                pts.push(q);
            }
            pts
        };
        // the blow: chest high, a little to one side
        let p = [r.range(-0.28, 0.28) * gw, r.range(-0.1, 0.3) * gh];
        let crush = r.range(0.005, 0.01);
        b.hits.push((p, crush));
        let n = 7 + (r.next() * 5.0) as usize;
        let a0 = r.range(0.0, PI * 2.0);
        let mut dirs: Vec<Vec<[f64; 2]>> = vec![];
        for i in 0..n {
            let a = a0 + PI * 2.0 * i as f64 / n as f64 + r.range(-0.25, 0.25);
            let start = [p[0] + a.cos() * crush, p[1] + a.sin() * crush];
            let reach = if r.next() < 0.25 { r.range(0.05, 0.16) } else { 9.0 };
            let pts = run(&mut r, start, a, reach);
            if r.next() < 0.4 && pts.len() > 5 {
                let k = pts.len() * (3 + (r.next() * 3.0) as usize) / 10;
                let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
                let turn = a + side * r.range(0.3, 0.6);
                let reach = if r.next() < 0.4 { r.range(0.04, 0.12) } else { 9.0 };
                let br = run(&mut r, pts[k], turn, reach);
                b.lines.push((br, 0.8));
            }
            dirs.push(pts.clone());
            b.lines.push((pts, 1.25));
        }
        let at = |pts: &Vec<[f64; 2]>, rad: f64| pts.iter().find(|q| (q[0] - p[0]).hypot(q[1] - p[1]) >= rad).copied();
        let rings = [r.range(0.018, 0.035), r.range(0.05, 0.09)];
        for (ri, &rad) in rings.iter().enumerate() {
            let chance = if ri == 0 { 0.55 } else { 0.3 };
            for i in 0..n {
                let j = (i + 1) % n;
                if r.next() > chance {
                    continue;
                }
                let r0 = rad * r.range(0.75, 1.3);
                let r1 = r0 * r.range(0.8, 1.2);
                let (Some(q0), Some(q1)) = (at(&dirs[i], r0), at(&dirs[j], r1)) else { continue };
                let m = [(q0[0] + q1[0]) / 2.0 + r.range(-0.002, 0.002), (q0[1] + q1[1]) / 2.0 + r.range(-0.002, 0.002)];
                b.lines.push((vec![q0, m, q1], 0.75));
            }
        }
        b.radials = dirs;
        b
    }
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn cm_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The glass: silver behind it (`Patch::Glass`, nearly all of the light
/// back), its map where the silver has gone to the black paint behind
fn cm_glass_mat(silver: Tex) -> Mat {
    // the lamp seen in it as a lamp is in a mirror: a small hot spot the
    // size of its lens, not a disc of glare round it (at 4000 the highlight
    // read as bloom: M8)
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: MIRROR_SHINE, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(silver);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off what it faces), nothing behind it
        f[0] = 0.13;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the map's grey is the silver gone, to the black paint behind (linear)
        f[8] = 1.0;
        f[9] = 0.016;
        f[10] = 0.015;
        f[11] = 0.013;
        f[12] = 0.0;
        // silvered: most of the light comes back at any angle
        f[13] = 0.86;
    }
    m
}

/// Where the silver has gone (grey: how much, to the black paint behind):
/// a black edge creeping in from the frame in stretches with a wandering,
/// feathered front (deepest along the foot and in the corners), spots of it
/// ahead, pinholes; the silver flaked along the cracks and round the blow
fn cm_silver_tex(b: &CmBreak, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(240, 328, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (CM_GLASS[0] + CM_UNDER, CM_GLASS[1] + CM_UNDER);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.set_fill_style("#000");
            ctx.fill_rect(0, 0, w, h);
            // the room in it, darker low down
            let gr = ctx.create_linear_gradient(0, 0, 0, h);
            gr.add_color_stop(0, "rgba(255,255,255,0)");
            gr.add_color_stop(0.4, "rgba(255,255,255,0.05)");
            gr.add_color_stop(1, "rgba(255,255,255,0.26)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, w, h);
            let spot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, a: f64| {
                let stops = [
                    (0.0, format!("rgba(255,255,255,{a:.3})")),
                    (0.5, format!("rgba(255,255,255,{:.3})", a * 0.75)),
                    (1.0, "rgba(255,255,255,0)".into()),
                ];
                cm_blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
            };
            // the black edge along each side (how deep at most in px, the side's length)
            let sides: [(f64, f64); 4] = [(13.0, w), (6.0, h), (6.0, h), (4.0, w)]; // foot, left, right, top
            for (si, &(deep, len)) in sides.iter().enumerate() {
                let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
                let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
                let depth = |t: f64| -> f64 {
                    let wave = 0.5
                        + 0.22 * (fr[0] * t * 6.28 + ph[0]).sin()
                        + 0.16 * (fr[1] * t * 6.28 + ph[1]).sin()
                        + 0.1 * (fr[2] * t * 6.28 + ph[2]).sin();
                    let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                    let corner = 1.0 + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).exp();
                    deep * (on * on * (3.0 - 2.0 * on)) * wave * corner
                };
                let at = |t: f64, d: f64| -> (f64, f64) {
                    match si {
                        0 => (t * w, h - d),
                        1 => (d, h - t * h),
                        2 => (w - d, h - t * h),
                        _ => (t * w, d),
                    }
                };
                ctx.set_fill_style("rgba(255,255,255,0.85)");
                ctx.begin_path();
                let (x0, y0) = at(0.0, 0.0);
                ctx.move_to(x0, y0);
                let n = (len / 3.0) as usize;
                for k in 0..=n {
                    let t = k as f64 / n as f64;
                    let (x, y) = at(t, depth(t));
                    ctx.line_to(x, y);
                }
                let (x1, y1) = at(1.0, 0.0);
                ctx.line_to(x1, y1);
                ctx.close_path();
                ctx.fill();
                for _ in 0..(len / 2.5) as usize {
                    let t = rnd();
                    let d = depth(t);
                    if d < 0.5 {
                        continue;
                    }
                    let (x, y) = at(t, d * (0.8 + 0.4 * rnd()));
                    spot(ctx, x, y, 1.0 + rnd() * d.min(10.0) * 0.45, 0.3 + 0.4 * rnd());
                }
                for _ in 0..(len / 6.0) as usize {
                    let t = rnd();
                    let d = depth(t).max(1.5) * (1.3 + 2.5 * rnd().powi(2));
                    let (x, y) = at(t, d);
                    spot(ctx, x, y, 0.6 + rnd() * rnd() * 3.0, 0.4 + 0.5 * rnd());
                }
            }
            // pinholes, all over but most low down
            for _ in 0..60 {
                let (x, y) = (rnd() * w, h * rnd().powf(0.6));
                ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.5 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.arc(x, y, 0.35 + rnd() * rnd() * 1.1, 0, 7);
                ctx.fill();
            }
            // along the cracks the silver flaked; round the blow more of it
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(255,255,255,0.55)");
                ctx.set_line_width(0.9 * k);
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.stroke();
            }
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                for _ in 0..10 {
                    let (a, d) = (rnd() * 7.0, rr * (1.0 + 2.5 * rnd()));
                    spot(ctx, x + a.cos() * d, y + a.sin() * d, 1.0 + rnd() * rr * 0.8, 0.4 + 0.4 * rnd());
                }
            }
        })
    })
}

/// The mirror's face: a film of dust, a wiped smear and finger marks, the
/// frame's wood seen again round its edge, and the breakage (each crack a
/// dark line where the silver broke and a bright one where its edge catches
/// the light, the crushed spot of the blow); clear elsewhere
fn cm_film_tex(b: &CmBreak, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(300, 400, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (CM_GLASS[0], CM_GLASS[1]);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.clear_rect(0, 0, w, h);
            // the moulding seen again in the glass, the glass's thickness a dark line between
            ctx.set_fill_style("rgba(70,46,30,0.5)");
            for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
                ctx.fill_rect(x, y, rw, rh);
            }
            ctx.set_stroke_style("rgba(18,16,14,0.45)");
            ctx.set_line_width(1.1);
            ctx.stroke_rect(3.5, 3.5, w - 7.0, h - 7.0);
            // dust, unevenly, and more of it lower down
            for _ in 0..14 {
                let (x, y, r) = (rnd() * w, h * rnd().powf(0.7), w * (0.12 + 0.3 * rnd()));
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(176,172,162,{:.3})", 0.05 + 0.06 * rnd()));
                g.add_color_stop(1, "rgba(176,172,162,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            // a wiped arc, and fingertips where it was straightened on its nail
            ctx.set_line_cap("round");
            for _ in 0..2 {
                let (x, y, r) = (w * (0.2 + 0.6 * rnd()), h * (0.3 + 0.5 * rnd()), w * (0.15 + 0.2 * rnd()));
                ctx.set_stroke_style(format!("rgba(186,182,172,{:.3})", 0.02 + 0.03 * rnd()));
                ctx.set_line_width(8.0 + 12.0 * rnd());
                ctx.begin_path();
                let a = rnd() * 6.3;
                ctx.arc(x, y, r, a, a + 0.8 + rnd() * 1.4);
                ctx.stroke();
            }
            for side in [0.0, 1.0] {
                let x0 = if side < 0.5 { w * (0.04 + 0.06 * rnd()) } else { w * (0.9 - 0.06 * rnd()) };
                let y0 = h * (0.35 + 0.3 * rnd());
                for f in 0..3 {
                    let stops = [
                        (0.0, "rgba(196,190,178,0.16)".to_string()),
                        (0.7, "rgba(196,190,178,0.08)".into()),
                        (1.0, "rgba(196,190,178,0)".into()),
                    ];
                    cm_blob(ctx, x0 + (rnd() - 0.5) * 4.0, y0 + f as f64 * 9.0 + rnd() * 3.0, 4.0 + rnd() * 2.0, 0.7, 1.3, &stops);
                }
            }
            // the cracks: dark where the silver broke, bright where the edge catches the light
            ctx.set_line_join("round");
            let path = |ctx: &mut Canvas, pts: &[[f64; 2]], dx: f64, dy: f64| {
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x + dx, y + dy);
                    } else {
                        ctx.line_to(x + dx, y + dy);
                    }
                }
                ctx.stroke();
            };
            // each piece between the radials sits a hair off the others and
            // gives back the room a little differently
            let nr = b.radials.len();
            for i in 0..nr {
                let (ra, rb) = (&b.radials[i], &b.radials[(i + 1) % nr]);
                let v = if rnd() < 0.5 { "rgba(220,218,210," } else { "rgba(10,10,9," };
                ctx.set_fill_style(format!("{v}{:.3})", 0.02 + 0.035 * rnd()));
                ctx.begin_path();
                for (k, q) in ra.iter().chain(rb.iter().rev()).enumerate() {
                    let (x, y) = px(*q);
                    if k == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(12,11,10,0.7)");
                ctx.set_line_width(1.2 * k);
                path(ctx, pts, 0.0, 0.0);
                ctx.set_stroke_style(format!("rgba(226,224,216,{:.2})", 0.28 + 0.22 * rnd()));
                ctx.set_line_width(0.6 * k);
                path(ctx, pts, 0.6, -0.5);
            }
            // the crushed spot: glass ground to white, splinters out of it
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                ctx.set_fill_style("rgba(214,212,204,0.85)");
                ctx.begin_path();
                for i in 0..14 {
                    let a = i as f64 / 14.0 * PI * 2.0;
                    let q = rr * (0.6 + 0.6 * rnd());
                    if i == 0 {
                        ctx.move_to(x + a.cos() * q, y + a.sin() * q);
                    } else {
                        ctx.line_to(x + a.cos() * q, y + a.sin() * q);
                    }
                }
                ctx.close_path();
                ctx.fill();
                for _ in 0..28 {
                    let a = rnd() * 7.0;
                    let (r0, r1) = (rr * rnd(), rr * (1.0 + 1.6 * rnd()));
                    let v = if rnd() < 0.5 { 22 } else { 226 };
                    ctx.set_stroke_style(format!("rgba({v},{v},{},0.7)", v - 6));
                    ctx.set_line_width(0.6 + 0.4 * rnd());
                    ctx.begin_path();
                    ctx.move_to(x + a.cos() * r0, y + a.sin() * r0);
                    ctx.line_to(x + a.cos() * r1, y + a.sin() * r1);
                    ctx.stroke();
                }
            }
        })
    })
}

/// A moulded frame's wood as its front is mapped (the frame's bounds, `w` x
/// `h` m, its rails `mould` m wide, `ppm` px a metre): each rail's grain
/// along it, mitred at the corners, the joints opened a hair; stained `base`
/// (sRGB), its figure darker, the stain rubbed thin along its outer edge and
/// its sight edge, knocks and scratches, dust along the top rail. `vary`
/// mixes the frame's place into its stream
fn moulding_grain_tex(w: f64, h: f64, mould: f64, base: [u8; 3], vary: u32, ppm: f64) -> Tex {
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let m = mould * ppm;
            let [r0, g0, b0] = base.map(f64::from);
            let col = |k: f64, a: f64| format!("rgba({:.0},{:.0},{:.0},{a:.3})", r0 * k, g0 * k, b0 * k);
            ctx.set_fill_style(col(1.0, 1.0));
            ctx.fill_rect(0, 0, cw, ch);
            // the four rails: (outline, along x?)
            let rails: [([(f64, f64); 4], bool); 4] = [
                ([(0.0, 0.0), (cw, 0.0), (cw - m, m), (m, m)], true),
                ([(0.0, ch), (cw, ch), (cw - m, ch - m), (m, ch - m)], true),
                ([(0.0, 0.0), (m, m), (m, ch - m), (0.0, ch)], false),
                ([(cw, 0.0), (cw - m, m), (cw - m, ch - m), (cw, ch)], false),
            ];
            for (poly, along_x) in rails.iter() {
                ctx.save();
                ctx.begin_path();
                ctx.move_to(poly[0].0, poly[0].1);
                for q in &poly[1..] {
                    ctx.line_to(q.0, q.1);
                }
                ctx.close_path();
                ctx.clip();
                // each rail its own board: a shade of its own
                ctx.set_fill_style(col(0.88 + 0.24 * rnd(), 0.5));
                ctx.fill_rect(0, 0, cw, ch);
                let len = if *along_x { cw } else { ch };
                let (lo, hi) = match (poly[0], along_x) {
                    ((_, y), true) if y < 1.0 => (0.0, m),
                    (_, true) => (ch - m, ch),
                    ((x, _), false) if x < 1.0 => (0.0, m),
                    _ => (cw - m, cw),
                };
                // the grain: long wavering lines along the rail, darker figure among them
                let lines = (m * 2.2) as usize;
                for _ in 0..lines {
                    let across = lo + rnd() * (hi - lo);
                    let (amp, fr, ph) = (0.4 + rnd() * 1.6, 1.0 + rnd() * 3.0, rnd() * 6.3);
                    let dark = rnd() < 0.3;
                    ctx.set_stroke_style(col(if dark { 0.55 } else { 0.72 + 0.2 * rnd() }, if dark { 0.35 } else { 0.18 + 0.2 * rnd() }));
                    ctx.set_line_width(if dark { 0.8 + rnd() * 1.4 } else { 0.4 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = (len / 8.0) as usize + 2;
                    for k in 0..=n {
                        let t = k as f64 / n as f64 * len;
                        let a = across + amp * (t / len * fr * 6.28 + ph).sin();
                        let (x, y) = if *along_x { (t, a) } else { (a, t) };
                        if k == 0 {
                            ctx.move_to(x, y);
                        } else {
                            ctx.line_to(x, y);
                        }
                    }
                    ctx.stroke();
                }
                // pores, short dark flecks along the grain
                for _ in 0..(len * m / 60.0) as usize {
                    let (t, a) = (rnd() * len, lo + rnd() * (hi - lo));
                    let l = 1.5 + rnd() * 4.0;
                    ctx.set_stroke_style(col(0.45, 0.25 + 0.2 * rnd()));
                    ctx.set_line_width(0.6);
                    ctx.begin_path();
                    let (x, y, dx, dy) = if *along_x { (t, a, l, 0.0) } else { (a, t, 0.0, l) };
                    ctx.move_to(x, y);
                    ctx.line_to(x + dx, y + dy);
                    ctx.stroke();
                }
                ctx.restore();
            }
            // the mitres: each joint opened a hair, grime in it
            ctx.set_stroke_style("rgba(20,14,10,0.55)");
            ctx.set_line_width(0.9);
            for (a, b) in [((0.0, 0.0), (m, m)), ((cw, 0.0), (cw - m, m)), ((0.0, ch), (m, ch - m)), ((cw, ch), (cw - m, ch - m))] {
                ctx.begin_path();
                ctx.move_to(a.0, a.1);
                ctx.line_to(b.0, b.1);
                ctx.stroke();
            }
            // the stain rubbed thin along the outer edge and the sight edge
            ctx.set_line_width(2.4);
            for inset in [1.2, m - 1.4] {
                ctx.set_stroke_style("rgba(222,196,160,0.14)");
                ctx.stroke_rect(inset, inset, cw - 2.0 * inset, ch - 2.0 * inset);
            }
            // knocks and scratches
            for _ in 0..(cw + ch) as usize / 14 {
                let edge = rnd() < 0.7;
                let (x, y) = if rnd() < 0.5 {
                    (
                        rnd() * cw,
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            ch - 1.0
                        } else {
                            ch - rnd() * m
                        },
                    )
                } else {
                    (
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            cw - 1.0
                        } else {
                            cw - rnd() * m
                        },
                        rnd() * ch,
                    )
                };
                if rnd() < 0.5 {
                    let stops = [(0.0, "rgba(214,182,140,0.45)".to_string()), (1.0, "rgba(214,182,140,0)".into())];
                    cm_blob(ctx, x, y, 0.8 + rnd() * 2.2, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
                } else {
                    let a = rnd() * 6.3;
                    let l = 3.0 + rnd() * 12.0;
                    ctx.set_stroke_style(format!("rgba(200,170,130,{:.2})", 0.12 + 0.18 * rnd()));
                    ctx.set_line_width(0.5);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            }
            // dust along the top rail, and grime where hands set it straight
            let gd = ctx.create_linear_gradient(0, 0, 0, m);
            gd.add_color_stop(0, "rgba(168,162,150,0.32)");
            gd.add_color_stop(1, "rgba(168,162,150,0.08)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(m * 0.4, 0, cw - m * 0.8, m);
            for x in [m * 0.6, cw - m * 0.6] {
                let stops = [(0.0, "rgba(28,20,14,0.22)".to_string()), (1.0, "rgba(28,20,14,0)".into())];
                cm_blob(ctx, x, ch * (0.55 + 0.2 * rnd()), m * 1.4, 0.6, PI / 2.0, &stops);
            }
        })
    })
}

// Wall chalkboard: 4% chance per level. A 30 x 40 cm wall board in a
// pine frame, a note left on it in chalk (batch r13a).
pub(crate) fn place_wall_chalkboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // The common 30 x 40 cm wall chalkboard (Blender batch `r13a`): a
    // painted board in a varnished pine frame 22 mm wide, flat on the wall;
    // on it a note in someone's chalk capitals over the ghost of the one
    // wiped before it, the eraser's swipes, dust (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let board = group();
    mount_on_wall(g, &board, &c, 0.010, 1.60, 0.6);
    let p = board.pos();
    let vary = cm_vary(p.x, p.z, 0.5);
    // the parts are modelled with the wall's face at z = 0: 10 mm behind the spot
    let body = group();
    body.position().z = -0.010;
    board.add(&body);
    if let Some(geo) = prop_part_as_modelled("Real__chalkboardFrame") {
        let tex = moulding_grain_tex(CB_SIZE[0], CB_SIZE[1], CB_MOULD, [0x9c, 0x76, 0x4c], vary, 1000.0);
        body.add(&mesh(geo, lambert!({ map: &tex })));
    }
    let (fw, fh) = (CB_SIZE[0] - 2.0 * CB_MOULD, CB_SIZE[1] - 2.0 * CB_MOULD);
    let face = mesh(plane_geo!(fw + 0.002, fh + 0.002), lambert!({ map: &chalkboard_face_tex(fw, fh, vary) }));
    face.position().z = CB_FACE;
    body.add(&face);
    g.world.add(&board);
}

/// The chalkboard: its size, its frame's width, its painted face's distance
/// from the wall (`Real__chalkboardFrame`, r13a.py)
const CB_SIZE: [f64; 2] = [0.400, 0.300];

const CB_MOULD: f64 = 0.022;

const CB_FACE: f64 = 0.0071;

/// What was left on it, in capitals: the week's business, nothing more
const CB_NOTES: [&[&str]; 6] = [
    &["MON 3/11", "STAFF MTG 9:30", "ROOM 2B"],
    &["ORDER:", "TONER x2", "A4 PAPER x5", "STAPLES"],
    &["J. HALE OUT", "TILL THE 14TH", "CALLS TO EXT 214"],
    &["PHOTOCOPIER", "OUT OF ORDER", "TECH WEDS"],
    &["MILK", "COFFEE", "SUGAR", "CUPS x20"],
    &["DELIVERY THURS", "CHAIRS x12", "TABLES x4"],
];

use placers_kit::noise::hash as cb_hash01;

use placers_kit::noise::vn as cb_vnoise;

use placers_kit::noise::sstep as cb_sstep;

/// An arc of an ellipse (centre, radii, from a0 to a1 radians, y up)
fn cb_arc(cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64) -> Vec<(f64, f64)> {
    let n = ((a1 - a0).abs() / 0.35).ceil().max(2.0) as usize;
    (0..=n).map(|k| a0 + (a1 - a0) * k as f64 / n as f64).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
}

/// A capital as a hand writes it in chalk: its width, and its strokes in the
/// order and direction they are made (x right, y up, the cap height 1)
fn cb_glyph(c: char) -> (f64, Vec<Vec<(f64, f64)>>) {
    let v = |p: &[(f64, f64)]| p.to_vec();
    match c {
        'A' => (0.66, vec![v(&[(0.0, 0.0), (0.33, 1.0), (0.66, 0.0)]), v(&[(0.14, 0.38), (0.52, 0.4)])]),
        'B' => (
            0.58,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.34, 1.0), (0.5, 0.93), (0.53, 0.78), (0.45, 0.6), (0.28, 0.54), (0.02, 0.54)]),
                v(&[(0.28, 0.54), (0.5, 0.47), (0.6, 0.3), (0.56, 0.12), (0.4, 0.02), (0.0, 0.0)]),
            ],
        ),
        'C' => (0.62, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 0.75, 5.55)]),
        'D' => (
            0.62,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.24, 1.0), (0.48, 0.9), (0.62, 0.62), (0.6, 0.3), (0.45, 0.08), (0.2, 0.0), (0.0, 0.0)]),
            ],
        ),
        'E' => (0.55, vec![v(&[(0.55, 1.0), (0.0, 1.0), (0.0, 0.0), (0.56, 0.0)]), v(&[(0.0, 0.52), (0.4, 0.53)])]),
        'F' => (0.52, vec![v(&[(0.55, 1.0), (0.0, 1.0), (0.0, 0.0)]), v(&[(0.0, 0.52), (0.4, 0.53)])]),
        'G' => (0.68, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 0.75, 5.9), v(&[(0.4, 0.42), (0.7, 0.42), (0.68, 0.06)])]),
        'H' => (0.6, vec![v(&[(0.0, 1.0), (0.0, 0.0)]), v(&[(0.6, 1.0), (0.6, 0.0)]), v(&[(0.0, 0.52), (0.6, 0.53)])]),
        'I' => (0.14, vec![v(&[(0.07, 1.0), (0.07, 0.0)])]),
        'J' => (0.5, vec![v(&[(0.5, 1.0), (0.5, 0.26), (0.42, 0.06), (0.25, 0.0), (0.08, 0.06), (0.0, 0.22)])]),
        'K' => (0.58, vec![v(&[(0.0, 1.0), (0.0, 0.0)]), v(&[(0.56, 1.0), (0.02, 0.42)]), v(&[(0.2, 0.6), (0.6, 0.0)])]),
        'L' => (0.5, vec![v(&[(0.0, 1.0), (0.0, 0.0), (0.5, 0.0)])]),
        'M' => (0.76, vec![v(&[(0.0, 0.0), (0.04, 1.0), (0.38, 0.28), (0.71, 1.0), (0.76, 0.0)])]),
        'N' => (0.62, vec![v(&[(0.0, 0.0), (0.0, 1.0), (0.62, 0.0), (0.62, 1.0)])]),
        'O' => (0.72, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 1.75, 1.75 + 6.45)]),
        'P' => {
            (0.56, vec![v(&[(0.0, 0.0), (0.0, 1.0)]), v(&[(0.0, 1.0), (0.36, 1.0), (0.54, 0.88), (0.55, 0.66), (0.4, 0.52), (0.02, 0.5)])])
        }
        'Q' => (0.74, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 1.75, 1.75 + 6.4), v(&[(0.42, 0.24), (0.74, -0.06)])]),
        'R' => (
            0.6,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.36, 1.0), (0.54, 0.88), (0.55, 0.66), (0.4, 0.52), (0.02, 0.5)]),
                v(&[(0.22, 0.5), (0.6, 0.0)]),
            ],
        ),
        'S' => (
            0.58,
            vec![v(&[
                (0.55, 0.86),
                (0.42, 0.99),
                (0.2, 1.0),
                (0.04, 0.88),
                (0.06, 0.66),
                (0.3, 0.53),
                (0.52, 0.4),
                (0.6, 0.2),
                (0.5, 0.04),
                (0.28, 0.0),
                (0.08, 0.06),
                (0.0, 0.17),
            ])],
        ),
        'T' => (0.64, vec![v(&[(0.0, 1.0), (0.64, 1.0)]), v(&[(0.32, 1.0), (0.32, 0.0)])]),
        'U' => (0.6, vec![v(&[(0.0, 1.0), (0.0, 0.3), (0.08, 0.08), (0.29, 0.0), (0.5, 0.08), (0.59, 0.3), (0.6, 1.0)])]),
        'V' => (0.62, vec![v(&[(0.0, 1.0), (0.31, 0.0), (0.62, 1.0)])]),
        'W' => (0.86, vec![v(&[(0.0, 1.0), (0.2, 0.0), (0.43, 0.72), (0.65, 0.0), (0.86, 1.0)])]),
        'X' => (0.58, vec![v(&[(0.0, 1.0), (0.58, 0.0)]), v(&[(0.58, 1.0), (0.0, 0.0)])]),
        'Y' => (0.6, vec![v(&[(0.0, 1.0), (0.3, 0.5)]), v(&[(0.6, 1.0), (0.3, 0.5), (0.3, 0.0)])]),
        'Z' => (0.58, vec![v(&[(0.0, 1.0), (0.58, 1.0), (0.0, 0.0), (0.6, 0.0)])]),
        '0' => (0.58, vec![cb_arc(0.29, 0.5, 0.29, 0.5, 1.7, 1.7 + 6.4)]),
        '1' => (0.32, vec![v(&[(0.02, 0.8), (0.24, 1.0), (0.24, 0.0)])]),
        '2' => {
            (0.58, vec![v(&[(0.03, 0.78), (0.12, 0.95), (0.32, 1.0), (0.5, 0.92), (0.55, 0.72), (0.42, 0.46), (0.0, 0.0), (0.58, 0.0)])])
        }
        '3' => (
            0.56,
            vec![v(&[
                (0.04, 0.9),
                (0.24, 1.0),
                (0.46, 0.95),
                (0.52, 0.77),
                (0.38, 0.58),
                (0.2, 0.54),
                (0.38, 0.54),
                (0.55, 0.42),
                (0.58, 0.2),
                (0.42, 0.03),
                (0.2, 0.0),
                (0.02, 0.1),
            ])],
        ),
        '4' => (0.62, vec![v(&[(0.46, 0.0), (0.46, 1.0), (0.0, 0.3), (0.62, 0.3)])]),
        '5' => (
            0.56,
            vec![v(&[
                (0.52, 1.0),
                (0.1, 1.0),
                (0.06, 0.56),
                (0.3, 0.62),
                (0.5, 0.52),
                (0.58, 0.3),
                (0.48, 0.08),
                (0.26, 0.0),
                (0.04, 0.08),
            ])],
        ),
        '6' => (
            0.56,
            vec![v(&[
                (0.5, 0.95),
                (0.3, 1.0),
                (0.1, 0.85),
                (0.02, 0.5),
                (0.04, 0.2),
                (0.2, 0.01),
                (0.42, 0.02),
                (0.56, 0.2),
                (0.54, 0.4),
                (0.38, 0.55),
                (0.18, 0.52),
                (0.04, 0.38),
            ])],
        ),
        '7' => (0.56, vec![v(&[(0.0, 1.0), (0.58, 1.0), (0.2, 0.0)])]),
        '8' => (
            0.58,
            vec![v(&[
                (0.3, 0.54),
                (0.08, 0.66),
                (0.07, 0.88),
                (0.3, 1.0),
                (0.52, 0.88),
                (0.5, 0.66),
                (0.3, 0.54),
                (0.06, 0.4),
                (0.04, 0.14),
                (0.3, 0.0),
                (0.56, 0.14),
                (0.54, 0.4),
                (0.3, 0.54),
            ])],
        ),
        '9' => (
            0.56,
            vec![v(&[
                (0.54, 0.62),
                (0.4, 0.48),
                (0.18, 0.48),
                (0.04, 0.64),
                (0.06, 0.88),
                (0.26, 1.0),
                (0.48, 0.93),
                (0.56, 0.7),
                (0.54, 0.3),
                (0.42, 0.05),
                (0.2, 0.0),
                (0.06, 0.08),
            ])],
        ),
        '.' => (0.12, vec![v(&[(0.05, 0.0), (0.07, 0.03)])]),
        ':' => (0.14, vec![v(&[(0.06, 0.62), (0.08, 0.65)]), v(&[(0.06, 0.06), (0.08, 0.09)])]),
        '-' => (0.36, vec![v(&[(0.0, 0.45), (0.36, 0.47)])]),
        '/' => (0.42, vec![v(&[(0.0, -0.04), (0.42, 1.04)])]),
        'x' => (0.4, vec![v(&[(0.0, 0.52), (0.38, 0.0)]), v(&[(0.38, 0.52), (0.0, 0.0)])]),
        _ => (0.42, vec![]),
    }
}

/// The note's strokes laid out on a board `w` x `h` px: lines of capitals
/// `cap` px high from (x0, y0), as one hand writes them (each letter a
/// little bigger or smaller, leaning, off the line; the lines drifting)
fn cb_layout(lines: &[&str], w: f64, cap: f64, x0: f64, y0: f64, pitch: f64, r: &mut SplitMix64) -> Vec<Vec<(f64, f64)>> {
    let mut out = vec![];
    let lean = r.range(0.04, 0.14);
    for (li, line) in lines.iter().enumerate() {
        let mut x = x0 + r.range(-0.15, 0.25) * cap;
        let base = y0 + pitch * li as f64 + r.range(-0.08, 0.08) * cap;
        let drift = r.range(-0.035, 0.03);
        let size = cap * r.range(0.94, 1.06);
        for ch in line.chars() {
            if ch == ' ' {
                x += size * r.range(0.45, 0.62);
                continue;
            }
            let (gw, strokes) = cb_glyph(ch);
            let k = size * r.range(0.93, 1.07);
            let (dy, turn) = (r.range(-0.05, 0.05) * cap + (x - x0) * drift, r.range(-0.05, 0.05));
            let (sn, cs) = turn.sin_cos();
            for s in strokes {
                let mut p: Vec<(f64, f64)> = s
                    .iter()
                    .map(|&(gx, gy)| {
                        let (lx, ly) = ((gx + gy * lean) * k, gy * k);
                        let (rx, ry) = (lx * cs - ly * sn, lx * sn + ly * cs);
                        (x + rx, base + dy - ry)
                    })
                    .collect();
                // the hand overshoots where a stroke starts, now and then
                if p.len() > 1 && r.next() < 0.3 {
                    let (a, b) = (p[0], p[1]);
                    let e = r.range(0.04, 0.1);
                    p.insert(0, (a.0 - (b.0 - a.0) * e, a.1 - (b.1 - a.1) * e));
                }
                out.push(p);
            }
            x += (gw + r.range(0.16, 0.26)) * k;
            if x > w - cap * 0.6 {
                break;
            }
        }
    }
    out
}

/// How much chalk each pixel of a `w` x `h` board holds (0..1) from
/// `lines` (px; each smoothed, pressed heavier where it lands, skipping where
/// the stick rode over the board's grain), a stick `width` px wide. No draws
fn cb_chalk(lines: &[Vec<(f64, f64)>], n: usize, m: usize, width: f64, seed: u32) -> Vec<f32> {
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        // smoothed (Chaikin, twice) between its sharp turns, which a hand
        // makes as turns (an A's apex, an E's corners)
        let mut p: Vec<(f64, f64)> = vec![];
        let mut run: Vec<(f64, f64)> = vec![];
        for k in 0..l.len() {
            run.push(l[k]);
            let sharp = k > 0 && k + 1 < l.len() && {
                let (a, b, c) = (l[k - 1], l[k], l[k + 1]);
                let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
                let cos = (u.0 * v.0 + u.1 * v.1) / (u.0.hypot(u.1) * v.0.hypot(v.1)).max(1e-9);
                cos < 0.45
            };
            if sharp || k + 1 == l.len() {
                for _ in 0..2 {
                    if run.len() < 3 {
                        break;
                    }
                    let mut q = vec![run[0]];
                    for j in 0..run.len() - 1 {
                        let (a, b) = (run[j], run[j + 1]);
                        q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
                        q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
                    }
                    q.push(run[run.len() - 1]);
                    run = q;
                }
                if !p.is_empty() {
                    run.remove(0);
                }
                p.append(&mut run);
                run.push(l[k]);
            }
        }
        let mut d = vec![];
        for k in 0..p.len().saturating_sub(1) {
            let (a, b) = (p[k], p[k + 1]);
            let c = ((a.0 - b.0).hypot(a.1 - b.1) / 2.5).ceil().max(1.0) as usize;
            for i in 0..c {
                let t = i as f64 / c as f64;
                d.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        if let Some(&e) = p.last() {
            d.push(e);
        }
        if d.len() == 1 {
            d.push((d[0].0 + 0.6, d[0].1 + 0.4));
        }
        let nd = d.len() - 1;
        let ls = seed.wrapping_add(li as u32 * 97);
        let press = |k: usize| {
            let t = k as f64 / nd as f64;
            (0.78 + 0.22 * (t * 5.0 + li as f64).sin()) * (1.0 + 0.25 * (-t * 14.0).exp()) * ((1.0 - t) / 0.1).clamp(0.45, 1.0)
        };
        for k in 0..nd {
            segs.push((d[k], d[k + 1], press(k), press(k + 1), ls));
        }
    }
    let r = width / 2.0;
    const CELL: usize = 16;
    let (cw, chh) = (n.div_ceil(CELL), m.div_ceil(CELL));
    let mut cells: Vec<Vec<u32>> = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - r - 1.0, cw), cell(a.0.max(b.0) + r + 1.0, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - r - 1.0, chh), cell(a.1.max(b.1) + r + 1.0, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CELL) * cw + i / CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                let rr = r * (0.82 + 0.3 * cb_vnoise(ls ^ 0x5a, x * 0.3, y * 0.3));
                let cover = cb_sstep(rr + 0.7, rr - 0.7, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = cb_sstep(0.18, 0.45, cb_vnoise(ls ^ 0x77, (a.0 + ex * t) * 0.09, (a.1 + ey * t) * 0.09));
                let press = (pa + (pb - pa) * t) * (0.45 + 0.55 * skip) * (1.0 - 0.35 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            // chalk holds on the board's grain, finer than on a wall
            let grain = cb_hash01(seed ^ 0x1d, i as i32, j as i32);
            let tooth = 0.75 * grain + 0.25 * cb_vnoise(seed ^ 0x2e, x * 0.2, y * 0.2);
            let a = cb_sstep(tooth - 0.18, tooth + 0.12, keep * 0.9) * keep.min(1.0).powf(0.5) * 0.92;
            out[i * 4] = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    });
    (0..n * m).map(|k| f32::from(bytes[k * 4]) / 255.0).collect()
}

/// The board's face, `w` x `h` m: its black-green paint, worn glossy where
/// it was wiped most; the ghost of the last note under the eraser's
/// swipes; chalk dust gathered low; the note itself in chalk capitals.
/// Which note, and the hand, from `vary` (where the board hangs)
fn chalkboard_face_tex(w: f64, h: f64, vary: u32) -> Tex {
    let ppm = 1600.0;
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let (n, m) = (cw as usize, ch as usize);
        let mut r = SplitMix64::new(seed, 0x0c4a_1b0a);
        let note = (vary as usize) % CB_NOTES.len();
        let ghost = (note + 1 + (vary as usize >> 8) % (CB_NOTES.len() - 1)) % CB_NOTES.len();
        // the note: capitals 2.4 cm high, lines 4.6 cm apart
        let cap = 0.024 * ppm * r.range(0.9, 1.1);
        let lines = cb_layout(CB_NOTES[note], cw, cap, 0.025 * ppm, 0.045 * ppm + cap * 0.2, 0.046 * ppm * r.range(0.95, 1.05), &mut r);
        let text = cb_chalk(&lines, n, m, 0.0034 * ppm, seed ^ 0x51);
        // the last note, wiped: thicker, fainter, smeared sideways by the eraser
        let ghost_lines = cb_layout(CB_NOTES[ghost], cw, cap * 1.1, 0.03 * ppm, 0.05 * ppm, 0.05 * ppm, &mut r);
        let ghost_c = cb_chalk(&ghost_lines, n, m, 0.0062 * ppm, seed ^ 0x52);
        // the eraser's swipes: long gentle arcs across the board, some overlapping
        let swipes: Vec<(f64, f64, f64, f64, f64)> = (0..7)
            .map(|_| (r.range(0.0, ch), r.range(0.03, 0.08) * ch, r.range(-0.08, 0.08), r.range(0.4, 1.0), r.range(0.025, 0.06) * ppm))
            .collect();
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64 + 0.5;
            for i in 0..n {
                let x = i as f64 + 0.5;
                let k = j * n + i;
                // the paint: a dark green-black, mottled, glossier where wiped most
                let mot = cb_vnoise(seed ^ 0x61, x * 0.02, y * 0.02) * 0.6 + cb_vnoise(seed ^ 0x62, x * 0.09, y * 0.09) * 0.4;
                let base = [34.0 + 9.0 * mot, 41.0 + 9.0 * mot, 37.0 + 8.0 * mot];
                // what the eraser left: dust along its swipes, smearing the ghost sideways
                let mut haze = 0.05 + 0.05 * cb_vnoise(seed ^ 0x63, x * 0.015, y * 0.03);
                let mut wiped = 0.0f64;
                for &(sy, amp, tilt, str_, wd) in &swipes {
                    let cy = sy + tilt * (x - cw / 2.0) + amp * ((x / cw) * 3.1).sin();
                    let d = (y - cy) / wd;
                    let band = (-d * d).exp() * str_;
                    let streak = 0.6 + 0.4 * cb_vnoise(seed ^ 0x64, x * 0.01, y * 0.6);
                    haze += band * 0.07 * streak;
                    wiped += band * 0.9;
                }
                // dust settled low on the board
                haze += 0.08 * cb_sstep(0.7, 1.0, y / ch) * (0.6 + 0.4 * cb_vnoise(seed ^ 0x65, x * 0.05, y * 0.2));
                let smear = ghost_c[j * n + ((i as f64 + 6.0 * cb_vnoise(seed ^ 0x66, x * 0.02, y * 0.05)) as usize).min(n - 1)] as f64;
                // most of it gone under the swipes: what is left, in patches
                let left = cb_sstep(0.3, 0.65, cb_vnoise(seed ^ 0x67, x * 0.025, y * 0.06)) * (1.0 - 0.75 * wiped.min(1.0));
                let g = 0.24 * smear * left;
                let t = text[k] as f64 * 0.95;
                let a = (1.0 - (1.0 - haze.min(0.4)) * (1.0 - g) * (1.0 - t)).clamp(0.0, 1.0);
                let chalk = [224.0, 224.0, 216.0];
                for c in 0..3 {
                    out[i * 4 + c] = (base[c] * (1.0 - a) + chalk[c] * a).round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
