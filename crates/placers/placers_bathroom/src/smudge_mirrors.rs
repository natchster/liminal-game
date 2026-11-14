//! `place_smudge_mirrors`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t04_put;
use placers_kit::glass::glass_mat as t04_glass_mat;
use placers_kit::noise::fbm as t04_fbm;
use placers_kit::noise::hash as t04_hash;
use placers_kit::noise::sstep as t04_ss;
use placers_kit::paint::fill as t04_fill;
use placers_kit::place::pick_wall;
use placers_kit::place::vary as t04_vary;
use placers_kit::place::wall_mount;

// Soap-smudged wall mirror: 4% chance per level. A 16 x 20 in
// stainless channel-frame washroom mirror (Blender batch `t04`, the Bradley
// 781 class) on its hanger: silvered glass that gives back the room's light
// (`Patch::Glass`), the silver just starting to go at its foot; on its face
// the last cleaning's smears, wiped in arcs with a soapy cloth and left to
// dry in streaks, the splashes from the basin under it, fingers' smudges at
// its edges, flecks of toothpaste, dust over its top.
pub(crate) fn place_smudge_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let mirror = group();
    let mount_y = 1.55;
    wall_mount(g, &mirror, x, z, dx, dz, 0.005);
    let p = mirror.pos();
    let vary = t04_vary(p.x, p.z, mount_y);
    // the parts are modelled with the wall at z = 0: 5 mm behind the spot
    let body = group();
    body.position().set(0, mount_y, -0.005);
    mirror.add(&body);
    t04_put(&body, "Real__smudgeMirrorFrame", &phong!({ color: 0x55585a, specular: 0x8a8d90, shininess: 48 }));
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(SM_GLASS[0] + 0.01, SM_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
    back.position().z = SM_GLASS[2] - 0.0062;
    body.add(&back);
    let glass = mesh(
        plane_geo!(SM_GLASS[0] + SM_UNDER, SM_GLASS[1] + SM_UNDER),
        // (the lamp in it a small hot spot, not a disc of glare: `MIRROR_SHINE`)
        t04_glass_mat(sm_silver_tex(vary), "mirror_glass", 0.13, [0.0; 3], 0.0, 0.86, [1.0, 0.016, 0.015, 0.013], MIRROR_SHINE),
    );
    glass.position().z = SM_GLASS[2] - 0.0001;
    glass.set_render_order(2);
    body.add(&glass);
    let film = mesh(
        plane_geo!(SM_GLASS[0], SM_GLASS[1]),
        lambert!({ map: &sm_film_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    film.position().z = SM_GLASS[2] + 0.0003;
    film.set_render_order(3);
    body.add(&film);
    g.world.add(&mirror);
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn t04_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
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

/// The mirror's glass as seen inside its frame (w, h), its face's distance
/// from the wall (`Real__smudgeMirrorFrame`'s hem, bathroom/washroom_fittings.py SM_*), and how much
/// more of it runs under the frame's lip (both sides)
const SM_GLASS: [f64; 3] = [0.380, 0.482, 0.0172];

const SM_UNDER: f64 = 0.006;

/// Where the silver has gone (grey: how much, to the black paint behind):
/// only begun, a feathered black edge in stretches along its foot and in its
/// lower corners, a few spots ahead of it, pinholes
fn sm_silver_tex(vary: u32) -> Tex {
    canvas_tex_opt(192, 240, false, move |ctx, w, h| {
        let seed = vary ^ 0x5117;
        let (gw, gh) = (SM_GLASS[0] + SM_UNDER, SM_GLASS[1] + SM_UNDER);
        t04_fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * gw, (v - 0.5) * gh);
            // distance in from the nearest edge, the foot's counted double
            let dx = gw / 2.0 - x.abs();
            let dfoot = y + gh / 2.0;
            let wave = t04_fbm(seed, x * 18.0, y * 18.0, 3);
            let deep = 0.004 + 0.010 * t04_ss(0.45, 0.75, t04_fbm(seed ^ 0x33, x * 6.0, 0.5, 3));
            let foot = 1.0 - t04_ss(deep * 0.5, deep * (1.2 + wave), dfoot);
            let corner = 1.0 - t04_ss(0.0, 0.03, (dx * dx + dfoot * dfoot).sqrt() - 0.012);
            let side = (1.0 - t04_ss(0.0015, 0.004 + 0.004 * wave, dx)) * (1.0 - t04_ss(0.05, 0.2, dfoot));
            let spot = if t04_hash(seed ^ 9, (u * 96.0) as i32, (v * 120.0) as i32) < 0.006 * (1.0 - t04_ss(0.0, 0.12, dfoot)) + 0.0006 {
                0.8
            } else {
                0.0
            };
            let g0 = foot.max(corner * 0.9).max(side).max(spot);
            [g0, g0, g0, 1.0]
        });
    })
}

/// The mirror's face (transparent where clean): a soapy cloth's arcs dried
/// in milky streaks, the run of its last wipe, a haze over the lower half
/// with the basin's splashes dried on it, toothpaste flecks, fingers'
/// smudges at its sides, dust settled along the top, the frame's lip seen
/// again in it
fn sm_film_tex(vary: u32) -> Tex {
    canvas_tex_opt(384, 488, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x5aa9_e1f1);
        ctx.clear_rect(0, 0, w, h);
        // the frame's lip seen again in the glass, the glass's edge a dark line
        ctx.set_fill_style("rgba(150,152,150,0.42)");
        for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
            ctx.fill_rect(x, y, rw, rh);
        }
        ctx.set_stroke_style("rgba(18,18,17,0.45)");
        ctx.set_line_width(1.2);
        ctx.stroke_rect(3.6, 3.6, w - 7.2, h - 7.2);
        // a haze of dried soap and water over the lower half
        let gr = ctx.create_linear_gradient(0, h * 0.35, 0, h);
        gr.add_color_stop(0, "rgba(200,198,190,0)");
        gr.add_color_stop(1, "rgba(200,198,190,0.16)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, h * 0.35, w, h * 0.65);
        // the cloth's arcs: overlapping sweeps, each a band of soapy smear
        // and streaks dragged through it, wandering, fading in and out along
        // the stroke, broken where the cloth lifted
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        let sweeps = 5 + (r.next() * 3.0) as usize;
        for _ in 0..sweeps {
            let cx = w * r.range(0.0, 1.0);
            let cy = h * r.range(0.35, 1.1);
            let rad = w * r.range(0.3, 0.6);
            let a0 = r.range(0.0, 6.3);
            let span = r.range(0.6, 1.3);
            let band = r.range(30.0, 70.0);
            let alpha = r.range(0.04, 0.08);
            let (ph1, ph2) = (r.range(0.0, 6.3), r.range(0.0, 6.3));
            let strands = 7 + (r.next() * 6.0) as usize;
            for k in 0..strands {
                let off = band * (r.next() - 0.5);
                let broad = k < 2;
                let lw = if broad { r.range(10.0, 22.0) } else { r.range(0.6, 2.0) };
                let a_k = alpha * if broad { 0.55 } else { r.range(0.4, 1.0) };
                let (t_in, t_out) = (r.range(0.0, 0.25), r.range(0.75, 1.0));
                let gap = r.range(0.2, 0.9);
                let segs = 28;
                ctx.set_line_width(lw);
                let pt = |t: f64| {
                    let a = a0 + span * t;
                    let rr = rad + off + 4.0 * (t * 9.0 + ph1).sin() + 2.5 * (t * 23.0 + ph2 + off * 0.1).sin();
                    (cx + a.cos() * rr, cy + a.sin() * rr)
                };
                for sg in 0..segs {
                    let (t0, t1) = (sg as f64 / segs as f64, (sg + 1) as f64 / segs as f64);
                    if t1 < t_in || t0 > t_out || (!broad && (t0 - gap).abs() < 0.04) {
                        continue;
                    }
                    let env = (PI * ((t0 + t1) / 2.0 - t_in) / (t_out - t_in)).sin().max(0.0);
                    ctx.set_stroke_style(format!("rgba(216,214,206,{:.3})", a_k * env));
                    let (x0, y0) = pt(t0);
                    let (x1, y1) = pt(t1);
                    ctx.begin_path();
                    ctx.move_to(x0, y0);
                    ctx.line_to(x1, y1);
                    ctx.stroke();
                }
            }
        }
        // the last wipe's run: the soapy water gathered at its foot and ran
        // a little way, wandering, beading where it stopped
        for _ in 0..(4 + (r.next() * 5.0) as usize) {
            let x = w * r.range(0.08, 0.92);
            let y0 = h * r.range(0.45, 0.8);
            let len = h * r.range(0.03, 0.14);
            ctx.set_stroke_style(format!("rgba(206,203,194,{:.3})", r.range(0.06, 0.14)));
            ctx.set_line_width(r.range(0.8, 1.6));
            ctx.begin_path();
            ctx.move_to(x, y0);
            let mut px = x;
            for k in 1..=8 {
                px += r.range(-0.8, 0.8);
                ctx.line_to(px, y0 + len * k as f64 / 8.0);
            }
            ctx.stroke();
            t04_blob(ctx, px, y0 + len, 2.2, 1.3, 0.0, &[(0.0, "rgba(212,209,200,0.28)".into()), (1.0, "rgba(212,209,200,0)".into())]);
        }
        // the basin's splashes, dried, thickest low down
        for _ in 0..180 {
            let x = r.next() * w;
            let y = h * (0.4 + 0.6 * r.next().powf(0.55));
            let rr = 0.5 + r.next() * r.next() * 2.8;
            let a = 0.08 + 0.2 * r.next();
            ctx.set_fill_style(format!("rgba(208,205,196,{a:.3})"));
            ctx.begin_path();
            ctx.ellipse(x, y, rr, rr * (0.7 + 0.3 * r.next()), r.next() * 3.0, 0, 7);
            ctx.fill();
            // a ring where it dried
            if r.next() < 0.25 {
                ctx.set_stroke_style(format!("rgba(190,186,176,{:.3})", a * 0.8));
                ctx.set_line_width(0.6);
                ctx.begin_path();
                ctx.arc(x, y, rr + 0.8, 0, 7);
                ctx.stroke();
            }
        }
        // toothpaste: white flecks at a mouth's height over the basin
        for _ in 0..(10 + (r.next() * 16.0) as usize) {
            let x = w * (0.5 + r.range(-0.3, 0.3));
            let y = h * r.range(0.62, 0.92);
            ctx.set_fill_style(format!("rgba(232,234,230,{:.2})", r.range(0.5, 0.85)));
            ctx.begin_path();
            ctx.ellipse(x, y, r.range(0.8, 2.0), r.range(0.6, 1.4), r.range(0.0, 3.0), 0, 7);
            ctx.fill();
        }
        // greasy smudges at its sides where hands set it straight, soft
        for side in [0.06, 0.94] {
            if r.next() < 0.4 {
                continue;
            }
            let (x, y) = (w * (side + r.range(-0.03, 0.03)), h * r.range(0.3, 0.7));
            let turn = r.range(-0.5, 0.5);
            for _ in 0..3 {
                let (dx, dy) = (r.range(-8.0, 8.0), r.range(-18.0, 18.0));
                t04_blob(
                    ctx,
                    x + dx,
                    y + dy,
                    r.range(6.0, 11.0),
                    r.range(0.45, 0.8),
                    turn + r.range(-0.3, 0.3),
                    &[(0.0, format!("rgba(190,186,176,{:.3})", r.range(0.08, 0.16))), (1.0, "rgba(190,186,176,0)".into())],
                );
            }
        }
        // dust along the top, and more of it in the corners
        let gt = ctx.create_linear_gradient(0, 0, 0, h * 0.22);
        gt.add_color_stop(0, "rgba(176,172,162,0.20)");
        gt.add_color_stop(1, "rgba(176,172,162,0)");
        ctx.set_fill_style(&gt);
        ctx.fill_rect(0, 0, w, h * 0.22);
        for _ in 0..10 {
            let (x, y, rr) = (r.next() * w, r.next() * h, w * r.range(0.08, 0.25));
            t04_blob(
                ctx,
                x,
                y,
                rr,
                r.range(0.5, 1.0),
                r.range(0.0, 3.0),
                &[(0.0, format!("rgba(176,172,162,{:.3})", r.range(0.03, 0.07))), (1.0, "rgba(176,172,162,0)".into())],
            );
        }
    })
}
