//! The washroom mirror (batch `r05`): a stainless channel frame, its
//! silvered glass, the silver gone from the edges, cracked from a blow.
use super::*;

/// A broken washroom mirror on the wall (Blender batch `r05`; `yaw` is unused)
pub fn build_broken_mirror(g: &mut Game, cx: &mut PropsCtx, yaw: f64) -> Built {
    let _ = (g, cx, yaw);
    // A 24 x 36 in stainless channel-frame mirror (the Bobrick B-165 class)
    // on its hanger, flat on the wall: silvered glass that gives back the
    // room's light (`Patch::Glass`), its silver going black from the edges
    // and in pinholes, dust and dried splashes on it, broken where something
    // hit it (radial cracks from the blow, rings between them, now and then
    // a shard gone), every crack inside the glass (docs/PROPS.md).
    use crate::assets::props::prop_part_as_modelled;
    // the placer stands the group 0.11 m out from the wall's face (the wall
    // spot's 6 cm and its own 5)
    const WALL: f64 = -0.11;
    // the old cracks' draws: four, a start and five steps each; they break the glass
    let mut d = [[0.0f64; 12]; 4];
    for c in d.iter_mut() {
        c[0] = rand2(-0.3, 0.3);
        c[1] = rand2(-0.5, 0.5);
        for s in 0..5 {
            c[2 + 2 * s] = rand2(-0.15, 0.15);
            c[3 + 2 * s] = rand2(-0.2, 0.2);
        }
    }
    let brk = MirrorBreak::from_draws(&d);
    let mg = group();
    // the frame: bright-polished stainless gone dull
    if let Some(geo) = prop_part_as_modelled("Real__mirrorFrame") {
        let frame = mesh(geo, phong!({ color: 0x55585a, specular: 0x8a8d90, shininess: 48 }));
        frame.position().z = WALL;
        mg.add(&frame);
    }
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(MIRROR_GLASS[0] + 0.01, MIRROR_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
    back.position().z = WALL + 0.0112;
    mg.add(&back);
    // the glass, its edges under the frame's lip
    let glass = mesh(plane_geo!(MIRROR_GLASS[0] + MIRROR_UNDER, MIRROR_GLASS[1] + MIRROR_UNDER), mirror_glass_mat(&brk));
    glass.position().z = WALL + MIRROR_GLASS[2];
    glass.set_render_order(2);
    mg.add(&glass);
    // what is on its face: dust, dried splashes, the cracks catching the light
    let film_mat = lambert!({
        map: &mirror_film_tex(&brk), transparent: true, depthWrite: false,
        polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
    });
    let film = mesh(plane_geo!(MIRROR_GLASS[0], MIRROR_GLASS[1]), &film_mat);
    film.position().z = WALL + MIRROR_GLASS[2] + 0.0003;
    film.set_render_order(3);
    mg.add(&film);
    Built::new(mg, 1.4)
}

/// The mirror's glass as seen inside its frame (width, height) and its
/// face's distance from the wall (`Real__mirrorFrame`'s hem, r05.py)
const MIRROR_GLASS: [f64; 3] = [0.584, 0.888, 0.0172];
/// How much more of the glass runs under the frame's lip (both sides)
const MIRROR_UNDER: f64 = 0.006;

/// A mirror's breakage, in metres on its glass (x right and y up from its
/// middle), worked out from the old cracks' draws and a generator seeded
/// by them (no draws of its own), so the silver and the film agree
#[derive(Clone, Default)]
struct MirrorBreak {
    /// each crack: its points, and how heavy its line is (1 a fine one)
    lines: Vec<(Vec<[f64; 2]>, f64)>,
    /// where it was struck, and how wide the crushed spot is
    hits: Vec<([f64; 2], f64)>,
    /// a shard fallen out by the blow (empty if none)
    hole: Vec<[f64; 2]>,
}

impl MirrorBreak {
    fn from_draws(d: &[[f64; 12]; 4]) -> MirrorBreak {
        let (hw, hh) = (MIRROR_GLASS[0] / 2.0, MIRROR_GLASS[1] / 2.0);
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in d.iter().flatten() {
            h ^= v.to_bits();
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut r = SplitMix64::new(0, h);
        let mut b = MirrorBreak::default();
        let inside = |p: [f64; 2]| p[0].abs() < hw + 0.004 && p[1].abs() < hh + 0.004;
        // a crack run from `p` toward `a`, wandering a little, to the glass's
        // edge or until it stops (`reach`)
        let run = |r: &mut SplitMix64, p: [f64; 2], a: f64, reach: f64| -> Vec<[f64; 2]> {
            let mut pts = vec![p];
            let (mut q, mut a, mut l) = (p, a, 0.0);
            while l < reach && inside(q) {
                let step = r.range(0.012, 0.03);
                a += r.range(-0.11, 0.11);
                q = [q[0] + a.cos() * step, q[1] + a.sin() * step];
                l += step;
                pts.push(q);
            }
            pts
        };
        let kind = (d[1][0] + 0.3) / 0.6;
        if kind < 0.74 {
            // struck: the blow's centre, chest high or a little above
            let p = [d[0][0] * 0.62, d[0][1] * 0.5 + 0.05];
            let crush = r.range(0.007, 0.014);
            b.hits.push((p, crush));
            let n = 6 + ((d[1][1] + 0.5) * 5.0).floor() as usize;
            let a0 = d[2][0] / 0.3 * PI;
            let mut dirs = vec![];
            for i in 0..n {
                let a = a0 + PI * 2.0 * i as f64 / n as f64 + d[i / 5 % 4][2 + i % 5 * 2] / 0.15 * 0.28;
                let start = [p[0] + a.cos() * crush, p[1] + a.sin() * crush];
                let reach = if r.next() < 0.22 { r.range(0.07, 0.24) } else { 9.0 };
                let pts = run(&mut r, start, a, reach);
                // a branch now and then, splitting off part way
                if r.next() < 0.38 && pts.len() > 6 {
                    let k = pts.len() * (3 + (r.next() * 3.0) as usize) / 10;
                    let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
                    let turn = a + side * r.range(0.3, 0.6);
                    let reach = if r.next() < 0.4 { r.range(0.05, 0.15) } else { 9.0 };
                    let br = run(&mut r, pts[k], turn, reach);
                    b.lines.push((br, 0.8));
                }
                dirs.push((a, pts.clone()));
                b.lines.push((pts, 1.3));
            }
            // the rings between the radials, near the blow
            let rings = [r.range(0.022, 0.045), r.range(0.065, 0.12)];
            let at = |pts: &Vec<[f64; 2]>, rad: f64| -> Option<[f64; 2]> {
                pts.iter().find(|q| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt() >= rad).copied()
            };
            // (nearly straight from one radial to the next, each at its own
            // distance, in some sectors only, now and then stopping short)
            for (ri, &rad) in rings.iter().enumerate() {
                let chance = if ri == 0 { 0.5 } else { 0.28 };
                for i in 0..n {
                    let j = (i + 1) % n;
                    if r.next() > chance {
                        continue;
                    }
                    let r0 = rad * r.range(0.75, 1.3);
                    let (Some(q0), Some(q1)) = (at(&dirs[i].1, r0), at(&dirs[j].1, r0 * r.range(0.8, 1.2))) else { continue };
                    let reach = if r.next() < 0.3 { r.range(0.5, 0.8) } else { 1.0 };
                    let q1 = [q0[0] + (q1[0] - q0[0]) * reach, q0[1] + (q1[1] - q0[1]) * reach];
                    let m = [(q0[0] + q1[0]) / 2.0 + r.range(-0.003, 0.003), (q0[1] + q1[1]) / 2.0 + r.range(-0.003, 0.003)];
                    b.lines.push((vec![q0, m, q1], 0.8));
                }
            }
            // a shard gone between two radials
            if d[3][1] > 0.12 {
                let i = ((d[3][2] + 0.15) / 0.3 * n as f64).floor() as usize % n;
                let j = (i + 1) % n;
                let rad = rings[(d[3][3] > 0.0) as usize];
                if let (Some(q0), Some(q1)) = (at(&dirs[i].1, rad), at(&dirs[j].1, rad)) {
                    let (a0, a1) = (dirs[i].0, dirs[j].0);
                    b.hole = vec![
                        [p[0] + a0.cos() * crush, p[1] + a0.sin() * crush],
                        q0,
                        q1,
                        [p[0] + a1.cos() * crush, p[1] + a1.sin() * crush],
                    ];
                }
            }
            // a second, lighter blow
            if d[3][0] > 0.15 {
                let p2 = [(d[2][1] * 0.5).clamp(-hw + 0.06, hw - 0.06), (-d[1][1] * 0.7).clamp(-hh + 0.06, hh - 0.06)];
                let crush2 = r.range(0.004, 0.008);
                b.hits.push((p2, crush2));
                let n2 = 4 + (r.next() * 3.0) as usize;
                let a2 = r.range(0.0, PI * 2.0);
                for i in 0..n2 {
                    let a = a2 + PI * 2.0 * i as f64 / n2 as f64 + r.range(-0.3, 0.3);
                    let start = [p2[0] + a.cos() * crush2, p2[1] + a.sin() * crush2];
                    let reach = if r.next() < 0.6 { r.range(0.04, 0.16) } else { 9.0 };
                    b.lines.push((run(&mut r, start, a, reach), 1.0));
                }
            }
        } else {
            // a crack run across it from the bottom rail (a slammed door, the hanger)
            let x0 = d[0][0] / 0.3 * (hw - 0.05);
            let up = PI / 2.0 + (d[0][1] / 0.5) * 0.75;
            let pts = run(&mut r, [x0, -hh - 0.003], up, 9.0);
            let k = pts.len() / 2;
            if k > 2 {
                let side = if d[2][0] < 0.0 { -1.0 } else { 1.0 };
                let turn = up + side * r.range(0.35, 0.7);
                let br = run(&mut r, pts[k], turn, 9.0);
                b.lines.push((br, 0.9));
            }
            b.lines.push((pts, 1.1));
            if d[3][0] > 0.0 {
                let y0 = d[1][1] * 0.6;
                let side = if d[3][2] < 0.0 { -1.0 } else { 1.0 };
                let a = if side < 0.0 { r.range(-0.4, 0.4) } else { PI + r.range(-0.4, 0.4) };
                let reach = r.range(0.12, 0.35);
                let pts = run(&mut r, [side * (hw + 0.003), y0], a, reach);
                b.lines.push((pts, 0.9));
            }
        }
        b
    }
}

/// The mirror's glass: silver behind it (`Patch::Glass`, nearly all of the
/// light back), its map where the silver has gone to the black paint
fn mirror_glass_mat(b: &MirrorBreak) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 4000, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(mirror_silver_tex(b));
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off the walls and floor
        // it faces: a surface of about a third albedo), and nothing behind it
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
/// the black edge, a band creeping in from the frame with a wandering,
/// feathered front (deepest along the bottom, where the cleaning ran down,
/// and in its corners), spots of it near the front, pinholes; the silver
/// flaked along the cracks and round the blow. And, faintly, the darker
/// room low in it (the floor and the lower walls it gives back)
fn mirror_silver_tex(b: &MirrorBreak) -> Tex {
    let b = b.clone();
    canvas_tex_opt(384, 578, false, move |ctx, w, h| {
        let (gw, gh) = (MIRROR_GLASS[0] + MIRROR_UNDER, MIRROR_GLASS[1] + MIRROR_UNDER);
        let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
        ctx.set_fill_style("#000");
        ctx.fill_rect(0, 0, w, h);
        // the room in it, darker low down
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(255,255,255,0)");
        gr.add_color_stop(0.4, "rgba(255,255,255,0.06)");
        gr.add_color_stop(1, "rgba(255,255,255,0.3)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
        let spot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, a: f64| {
            let stops = [
                (0.0, format!("rgba(255,255,255,{a:.3})")),
                (0.5, format!("rgba(255,255,255,{:.3})", a * 0.75)),
                (1.0, "rgba(255,255,255,0)".into()),
            ];
            mirror_blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
        };
        // the black edge along each side: (how deep it runs in px at most, along the
        // side's length in px); its front wanders (three waves of their own) and
        // runs deeper into the corners
        let sides: [(f64, f64); 4] = [(16.0, w), (7.0, h), (7.0, h), (4.0, w)]; // bottom, left, right, top
        for (si, &(deep, len)) in sides.iter().enumerate() {
            let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
            let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
            let depth = |t: f64| -> f64 {
                let wave = 0.5
                    + 0.22 * (fr[0] * t * 6.28 + ph[0]).sin()
                    + 0.16 * (fr[1] * t * 6.28 + ph[1]).sin()
                    + 0.1 * (fr[2] * t * 6.28 + ph[2]).sin();
                // in stretches, not all along: where the wave is low the silver held
                let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                let corner = 1.0
                    + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).exp()
                    + if si == 0 { 0.0 } else { 1.5 * (-t * 7.0).exp() * (si != 3) as u8 as f64 };
                deep * (on * on * (3.0 - 2.0 * on)) * wave * corner
            };
            // where on the canvas a point `d` px in from the side, `t` along it, is
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
            // the front feathered, and spots of it ahead of the front
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
        for _ in 0..90 {
            let (x, y) = (rnd() * w, h * rnd().powf(0.6));
            ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.5 + 0.5 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.35 + rnd() * rnd() * 1.2, 0, 7);
            ctx.fill();
        }
        // along the cracks, the silver flaked; round the blow more of it
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
}

/// The mirror's face: a film of dust, dried splashes and drips on its lower
/// half, a wiped smear, and the breakage (each crack a dark line where the
/// silver broke and a bright one where its edge catches the light, the
/// crushed spot of the blow, a shard gone to the steel back); clear elsewhere
fn mirror_film_tex(b: &MirrorBreak) -> Tex {
    let b = b.clone();
    canvas_tex_opt(512, 778, false, move |ctx, w, h| {
        let (gw, gh) = (MIRROR_GLASS[0], MIRROR_GLASS[1]);
        let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
        ctx.clear_rect(0, 0, w, h);
        // the frame's lip seen again in the glass, round its edge, the glass's
        // thickness a dark line between
        ctx.set_fill_style("rgba(150,152,150,0.42)");
        for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
            ctx.fill_rect(x, y, rw, rh);
        }
        ctx.set_stroke_style("rgba(18,18,17,0.45)");
        ctx.set_line_width(1.2);
        ctx.stroke_rect(3.6, 3.6, w - 7.2, h - 7.2);
        // dust, unevenly
        for _ in 0..16 {
            let (x, y, r) = (rnd() * w, rnd() * h, w * (0.1 + 0.25 * rnd()));
            let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
            g.add_color_stop(0, &format!("rgba(176,172,162,{:.3})", 0.04 + 0.05 * rnd()));
            g.add_color_stop(1, "rgba(176,172,162,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
        // a wiped smear, an arc of a cloth or a palm
        ctx.set_line_cap("round");
        for _ in 0..3 {
            let (x, y, r) = (w * (0.2 + 0.6 * rnd()), h * (0.3 + 0.5 * rnd()), w * (0.12 + 0.2 * rnd()));
            ctx.set_stroke_style(format!("rgba(186,182,172,{:.3})", 0.04 + 0.04 * rnd()));
            ctx.set_line_width(8.0 + 14.0 * rnd());
            ctx.begin_path();
            let a = rnd() * 6.3;
            ctx.arc(x, y, r, a, a + 0.8 + rnd() * 1.4);
            ctx.stroke();
        }
        // dried splashes on the lower half, a few run down
        for _ in 0..170 {
            let x = rnd() * w;
            let y = h * (0.45 + 0.55 * rnd().powf(0.6));
            let r = 0.5 + rnd() * rnd() * 2.6;
            let a = 0.12 + 0.28 * rnd();
            ctx.set_fill_style(format!("rgba(206,203,194,{a:.3})"));
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.7 + 0.3 * rnd()), rnd() * 3.0, 0, 7);
            ctx.fill();
            if rnd() < 0.08 {
                ctx.set_stroke_style(format!("rgba(200,197,188,{:.3})", a * 0.5));
                ctx.set_line_width(r * 0.9);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + rnd() * 2.0 - 1.0, y + 10.0 + rnd() * 50.0);
                ctx.stroke();
            }
        }
        // the shard gone: the galvanised back, its paint left in patches
        if b.hole.len() > 2 {
            ctx.save();
            ctx.begin_path();
            for (i, p) in b.hole.iter().enumerate() {
                let (x, y) = px(*p);
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.close_path();
            ctx.set_fill_style("#3f403f");
            ctx.fill();
            ctx.clip();
            for _ in 0..40 {
                let (x, y) = px(b.hole[(rnd() * b.hole.len() as f64) as usize % b.hole.len()]);
                ctx.set_fill_style(format!("rgba(22,21,20,{:.2})", 0.4 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.ellipse(x + rnd() * 30.0 - 15.0, y + rnd() * 30.0 - 15.0, 2.0 + rnd() * 9.0, 2.0 + rnd() * 6.0, rnd() * 3.0, 0, 7);
                ctx.fill();
            }
            ctx.restore();
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
        for (pts, k) in &b.lines {
            ctx.set_stroke_style("rgba(14,13,12,0.85)");
            ctx.set_line_width(1.9 * k);
            path(ctx, pts, 0.0, 0.0);
            ctx.set_stroke_style(format!("rgba(236,234,228,{:.2})", 0.6 + 0.25 * rnd()));
            ctx.set_line_width(1.1 * k);
            path(ctx, pts, 0.9, -0.8);
        }
        if b.hole.len() > 2 {
            let mut edge = b.hole.clone();
            edge.push(b.hole[0]);
            ctx.set_stroke_style("rgba(236,234,228,0.8)");
            ctx.set_line_width(1.2);
            path(ctx, &edge, 0.0, 0.0);
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
            for _ in 0..34 {
                let a = rnd() * 7.0;
                let (r0, r1) = (rr * rnd(), rr * (1.0 + 1.6 * rnd()));
                let v = if rnd() < 0.5 { 22 } else { 226 };
                ctx.set_stroke_style(format!("rgba({v},{v},{},0.7)", v - 6));
                ctx.set_line_width(0.6 + 0.5 * rnd());
                ctx.begin_path();
                ctx.move_to(x + a.cos() * r0, y + a.sin() * r0);
                ctx.line_to(x + a.cos() * r1, y + a.sin() * r1);
                ctx.stroke();
            }
        }
    })
}
