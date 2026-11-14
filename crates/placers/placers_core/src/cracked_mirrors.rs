//! `place_cracked_mirrors`
use game::prelude::*;
use placers_kit::mirror_frame::blob;
use placers_kit::mirror_frame::moulding_grain_tex;
use placers_kit::place::mount_on_wall;
use placers_kit::place::vary as cm_vary;
use placers_kit::place::wall_candidates;

// Cracked wall mirror: 4% chance per level. A small mirror in a wooden
// frame, struck and cracked (family `six_things_hung_on_a_wall`).
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
    let brk = CmBreak::struck(CM_GLASS[0], CM_GLASS[1], (u64::from(vary) << 7) | 0x2d);
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
/// distance from the wall (`Real__crackedMirrorFrame`, industrial/six_things_hung_on_a_wall.py)
const CM_GLASS: [f64; 3] = [0.300, 0.400, 0.0080];

/// Its moulding's width, and how much more glass runs under its lip (both sides)
const CM_MOULD: f64 = 0.024;

const CM_UNDER: f64 = 0.010;

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
                q = [q[0] + a.dcos() * step, q[1] + a.dsin() * step];
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
            let start = [p[0] + a.dcos() * crush, p[1] + a.dsin() * crush];
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
        let at = |pts: &Vec<[f64; 2]>, rad: f64| pts.iter().find(|q| (q[0] - p[0]).dhypot(q[1] - p[1]) >= rad).copied();
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
                blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
            };
            // the black edge along each side (how deep at most in px, the side's length)
            let sides: [(f64, f64); 4] = [(13.0, w), (6.0, h), (6.0, h), (4.0, w)]; // foot, left, right, top
            for (si, &(deep, len)) in sides.iter().enumerate() {
                let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
                let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
                let depth = |t: f64| -> f64 {
                    let wave = 0.5
                        + 0.22 * (fr[0] * t * 6.28 + ph[0]).dsin()
                        + 0.16 * (fr[1] * t * 6.28 + ph[1]).dsin()
                        + 0.1 * (fr[2] * t * 6.28 + ph[2]).dsin();
                    let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                    let corner = 1.0 + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).dexp();
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
                    let d = depth(t).max(1.5) * (1.3 + 2.5 * rnd().dpowi(2));
                    let (x, y) = at(t, d);
                    spot(ctx, x, y, 0.6 + rnd() * rnd() * 3.0, 0.4 + 0.5 * rnd());
                }
            }
            // pinholes, all over but most low down
            for _ in 0..60 {
                let (x, y) = (rnd() * w, h * rnd().dpowf(0.6));
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
                    spot(ctx, x + a.dcos() * d, y + a.dsin() * d, 1.0 + rnd() * rr * 0.8, 0.4 + 0.4 * rnd());
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
                let (x, y, r) = (rnd() * w, h * rnd().dpowf(0.7), w * (0.12 + 0.3 * rnd()));
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
                    blob(ctx, x0 + (rnd() - 0.5) * 4.0, y0 + f as f64 * 9.0 + rnd() * 3.0, 4.0 + rnd() * 2.0, 0.7, 1.3, &stops);
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
                        ctx.move_to(x + a.dcos() * q, y + a.dsin() * q);
                    } else {
                        ctx.line_to(x + a.dcos() * q, y + a.dsin() * q);
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
                    ctx.move_to(x + a.dcos() * r0, y + a.dsin() * r0);
                    ctx.line_to(x + a.dcos() * r1, y + a.dsin() * r1);
                    ctx.stroke();
                }
            }
        })
    })
}
