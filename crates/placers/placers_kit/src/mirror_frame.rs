//! A mirror's or board's frame: the moulding's grain and soft blots.
use game::prelude::*;

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
pub fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
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

/// A moulded frame's wood as its front is mapped (the frame's bounds, `w` x
/// `h` m, its rails `mould` m wide, `ppm` px a metre): each rail's grain
/// along it, mitred at the corners, the joints opened a hair; stained `base`
/// (sRGB), its figure darker, the stain rubbed thin along its outer edge and
/// its sight edge, knocks and scratches, dust along the top rail. `vary`
/// mixes the frame's place into its stream
pub fn moulding_grain_tex(w: f64, h: f64, mould: f64, base: [u8; 3], vary: u32, ppm: f64) -> Tex {
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
                        let a = across + amp * (t / len * fr * 6.28 + ph).dsin();
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
                            if edge { 1.0 } else { rnd() * m }
                        } else if edge {
                            ch - 1.0
                        } else {
                            ch - rnd() * m
                        },
                    )
                } else {
                    (
                        if rnd() < 0.5 {
                            if edge { 1.0 } else { rnd() * m }
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
                    blob(ctx, x, y, 0.8 + rnd() * 2.2, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
                } else {
                    let a = rnd() * 6.3;
                    let l = 3.0 + rnd() * 12.0;
                    ctx.set_stroke_style(format!("rgba(200,170,130,{:.2})", 0.12 + 0.18 * rnd()));
                    ctx.set_line_width(0.5);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
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
                blob(ctx, x, ch * (0.55 + 0.2 * rnd()), m * 1.4, 0.6, PI / 2.0, &stops);
            }
        })
    })
}
