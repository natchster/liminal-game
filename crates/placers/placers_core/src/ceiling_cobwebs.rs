//! `place_ceiling_cobwebs`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::mark_tex;
use placers_kit::paint::paint_px_indexed;
use placers_kit::paint::rgb;
use placers_kit::place::mount_slid_ref;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Ceiling cobwebs: 25% chance per level, 1-3 cobwebs. An old cobweb in
// the angle of a wall and the ceiling (`docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`): a cobweb spider's
// tangle, long abandoned, its threads grey with the dust they caught, slung
// across the angle and sagging, thick where the tangle is and thin out to
// the few anchor threads that hold it to the wall and the ceiling. Its veil
// and its threads painted on a sagging sheet across the angle, torn and
// ragged at its edges, seen from both sides.
pub(crate) fn place_ceiling_cobwebs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    // Per-weather cobweb tint (pale greys), greyed by the dust in it
    let dusty = rgb(cobweb_color_by_weather(g.weather_key())).map(|c| c * 0.92);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // the old web's draws: its size (now its reach along the wall,
        // 0.34-0.6 m, and across the angle), where along the wall, its
        // threads' anchors, the sway, a caught insect and its pose, and
        // smaller webs beside it
        let size = 0.20 + rnd() * 0.15;
        let web = group();
        // Position at ceiling corner along the wall face.
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        mount_slid_ref(&web, &c, wall_x, wall_z, WALL_H - 0.02, 1.5);
        let strand_count = 3 + (rnd() * 2.0).floor() as i32;
        let anchors: Vec<f64> = (0..strand_count).map(|_| 0.1 + rnd() * 0.85).collect();
        // 20% chance: very slow sway in the air that moves through
        let sway = if rnd() < 0.20 { Some((rnd() * PI * 2.0, 12.0 + rnd() * 8.0)) } else { None };
        // 5% chance: a dead insect caught in it (its turn, and where:
        // the old draws, in the old order for the wall it is on)
        let insect = if rnd() < 0.05 {
            let (spin, a, b) = (rnd(), rnd(), rnd());
            let (drop, along) = if c.dx.abs() == 1 { (a, b) } else { (b, a) };
            Some((spin, drop, along))
        } else {
            None
        };
        // 25% chance: 1-2 smaller webs in the same stretch of the angle
        let mut sibs = vec![];
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                let sib_size = 0.10 + rnd() * 0.10;
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.20 + rnd() * 0.20);
                let drop = rnd() * 0.08;
                sibs.push((sib_size, sib_off, drop));
            }
        }
        // as long along the wall as the old one and as deep in all (its
        // middle where the old one's was: what `settle_props` probes with)
        web.add(&web_sheet(size, size * 0.62, size * 0.5, -0.9 * size - 0.018, dusty, &anchors, insect, 320.0));
        g.world.add(&web);
        if let Some((sway_phase, sway_period)) = sway {
            let base_rot_y = web.rot().y;
            let w2 = web.clone();
            g.animate(move |_g, _dt, t| {
                w2.rotation().y = base_rot_y + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
            });
        }
        for (sib_size, sib_off, drop) in sibs {
            // anchored under the same ceiling: its drop is how much further
            // down the wall it reaches
            let low = -2.0 * drop - 0.9 * sib_size - 0.018;
            let sib = web_sheet(sib_size, sib_size * 0.62 + drop, sib_size * 0.5, low, dusty, &[], None, 192.0);
            sib.rotation().y = web.rot().y;
            let wp = web.pos();
            if c.dx.abs() == 1 {
                sib.position().set(wp.x, wp.y, wp.z + sib_off);
            } else {
                sib.position().set(wp.x + sib_off, wp.y, wp.z);
            }
            g.world.add(&sib);
        }
        i += 1;
    }
}

/// A sheet across the angle of a wall and the ceiling, from x = 0 to `len`
/// along the wall: from the wall `down` below the ceiling to the ceiling
/// `out` from the wall, narrowing to the angle at both ends and sagging into
/// the room between its anchors, deepened or shallowed so its lowest point
/// is at `low` (its box is then the old web's, what `settle_props` probes
/// the wall with). In `wall_mount`'s frame (x along the wall, y up, z out of
/// it) with its origin `gap` off the wall and the ceiling; UVs u along the
/// wall, v from the wall (0) to the ceiling (1)
fn web_geo(len: f64, down: f64, out: f64, gap: f64, low: f64) -> Geo {
    let (nu, nv) = (12usize, 6usize);
    let (mut pos, mut uv, mut idx): (Vec<[f64; 3]>, Vec<f32>, Vec<u32>) = (vec![], vec![], vec![]);
    for j in 0..=nv {
        for i in 0..=nu {
            let (u, v) = (i as f64 / nu as f64, j as f64 / nv as f64);
            let env = 0.4 + 0.6 * (PI * u).sin().powf(0.45);
            let (dn, ot) = (down * env, out * env);
            // straight from the wall (2 mm off it) to the ceiling (2 mm under it)
            let (wy, wz) = (-dn, -gap + 0.002);
            let (cy, cz) = (gap - 0.002, ot);
            let sag = 0.2 * dn.hypot(ot) * 4.0 * v * (1.0 - v);
            let y = wy + (cy - wy) * v - sag * std::f64::consts::FRAC_1_SQRT_2;
            let z = wz + (cz - wz) * v + sag * std::f64::consts::FRAC_1_SQRT_2;
            pos.push([u * len, y, z]);
            uv.extend([u as f32, v as f32]);
        }
    }
    let (top, bottom) = (gap - 0.002, pos.iter().fold(f64::MAX, |m, p| m.min(p[1])));
    let k = (low - top) / (bottom - top);
    let pos: Vec<f32> = pos.iter().flat_map(|p| [p[0] as f32, (top + (p[1] - top) * k) as f32, p[2] as f32]).collect();
    for j in 0..nv {
        for i in 0..nu {
            let a = (j * (nu + 1) + i) as u32;
            let (b, c) = (a + 1, a + (nu + 1) as u32);
            idx.extend([a, b, c, b, c + 1, c]);
        }
    }
    let geo = Geo::new();
    geo.set_position(pos);
    geo.set_uv(uv);
    geo.set_index(idx);
    geo.compute_vertex_normals();
    geo
}

/// An old cobweb across the angle of a wall and the ceiling (`web_geo`), its
/// silk `col` (sRGB) gone grey with the dust it caught: a cobweb spider's
/// tangle, long abandoned. A veil of silk too fine to see thread by thread,
/// thickest in the tangle and torn in places; taut threads criss-crossing it
/// from junction to junction, out to anchors on the wall and the ceiling
/// (some at `anchors`, 0..1 along it); the dust beaded along them and matted
/// in clumps; and `insect` (turn, drop, along: 0..1) caught in it. `tw`: its
/// texture's width (px).
#[allow(clippy::too_many_arguments)]
fn web_sheet(len: f64, down: f64, out: f64, low: f64, col: [f64; 3], anchors: &[f64], insect: Option<(f64, f64, f64)>, tw: f64) -> Obj {
    let geo = web_geo(len, down, out, 0.02, low);
    let th = (tw * down.hypot(out) * 1.2 / len).round().clamp(64.0, 256.0);
    let anchors = anchors.to_vec();
    let tex = mark_tex(tw, th, col, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let mm = cw / (len * 1000.0); // px a millimetre
        let shade = |k: f64| col.map(|c| (c * k).min(1.0));
        let rgba = |k: f64, a: f64| {
            let c = shade(k);
            format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a)
        };
        // where the tangle is thick (the canvas's top row is the ceiling's
        // edge of the sheet, its bottom the wall's)
        let (bx, by) = (cw * (0.42 + 0.16 * rnd()), ch * (0.35 + 0.3 * rnd()));
        let (brx, bry) = (cw * (0.44 + 0.12 * rnd()), ch * (0.4 + 0.14 * rnd()));
        let in_blob = |k: f64| {
            let (a, r) = (rnd() * PI * 2.0, rnd().powf(0.6) * k);
            ((bx + a.cos() * r * brx).clamp(0.0, cw), (by + a.sin() * r * bry).clamp(0.0, ch))
        };
        // 1. the veil: silk too fine to see thread by thread, thickest in the
        // tangle, thin out to the ends, wispy along the way it was spun
        let veil = shade(0.97);
        let (wa, wb) = (rnd() * PI, rnd() * PI);
        paint_px_indexed(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) / cw, (j as f64 + 0.5) / ch);
            let q = ((x * cw - bx) / brx).hypot((y * ch - by) / bry);
            let dens = (-q * q * 0.8).exp();
            // its edges ragged: torn short of the wall (the bottom) and of its ends
            let rag = fbm(seed ^ 0x99, x * 7.0, y * 2.0, 3) - 0.5;
            let ends = sstep(0.0, 0.16, x.min(1.0 - x) + rag * 0.12) * sstep(0.0, 0.3, 1.0 - y + rag * 0.45);
            // wisps: noise stretched along two ways it was spun
            let (u1, u2) = (x * wa.cos() + y * wa.sin(), -x * wa.sin() + y * wa.cos());
            let (v1, v2) = (x * wb.cos() + y * wb.sin(), -x * wb.sin() + y * wb.cos());
            let wisp = 0.5 * vn(seed ^ 0x5e, u1 * 3.0, u2 * 50.0) + 0.5 * vn(seed ^ 0x6f, v1 * 3.0, v2 * 50.0);
            [veil[0], veil[1], veil[2], ((0.06 + 0.62 * dens) * ends * (0.35 + 1.1 * wisp * wisp)).min(0.72)]
        });
        ctx.set_line_cap("round");
        let thread = |ctx: &mut Canvas, a: (f64, f64), b: (f64, f64), sag: f64, style: &str, w: f64| {
            let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            let l = (b.0 - a.0).hypot(b.1 - a.1).max(1e-6);
            let (nx, ny) = (-(b.1 - a.1) / l, (b.0 - a.0) / l);
            ctx.set_stroke_style(style);
            ctx.set_line_width(w);
            ctx.begin_path();
            ctx.move_to(a.0, a.1);
            ctx.quadratic_curve_to(mx + nx * sag * l, my + ny * sag * l, b.0, b.1);
            ctx.stroke();
        };
        let px_w = |w_mm: f64| (w_mm * mm).max(0.8);
        // 2. the strands: long, nearly straight, crossing at shallow angles
        // along the two ways it was spun, so many over each other in the
        // tangle that they felt into a grey mat
        let span = cw.max(ch);
        for k in 0..140 + (rnd() * 80.0) as usize {
            let c = in_blob(1.1);
            let a = if k % 2 == 0 { wa } else { wb } + (rnd() - 0.5) * 0.9;
            let l = span * (0.15 + 0.5 * rnd());
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            thread(
                ctx,
                (c.0 - dx, c.1 - dy),
                (c.0 + dx, c.1 + dy),
                (rnd() - 0.5) * 0.08,
                &rgba(1.08 + 0.12 * rnd(), 0.12 + 0.24 * rnd()),
                px_w(0.6 + 0.6 * rnd()),
            );
        }
        // the lines it hangs by: from the tangle out to anchors on the
        // ceiling (top) and the wall (bottom), a few heavy with dust (thicker,
        // greyer, sagging)
        let mut fixed: Vec<(f64, f64)> = anchors.iter().map(|&t| (t * cw, 0.0)).collect();
        for _ in 0..8 + (rnd() * 8.0) as usize {
            fixed.push((cw * (0.04 + 0.92 * rnd()), if rnd() < 0.5 { 0.0 } else { ch }));
        }
        for &f in &fixed {
            let a = in_blob(0.6);
            let dusty = rnd() < 0.3;
            let (k_col, al, w, sag) = if dusty {
                (0.84, 0.3 + 0.2 * rnd(), 1.0 + 0.9 * rnd(), 0.05 + 0.06 * rnd())
            } else {
                (1.12, 0.25 + 0.25 * rnd(), 0.6 + 0.5 * rnd(), (rnd() - 0.5) * 0.04)
            };
            thread(ctx, a, f, sag, &rgba(k_col, al), px_w(w));
        }
        // 3. the dust: specks caught all through the tangle, matted in
        // clumps where it is thickest
        for _ in 0..90 + (rnd() * 80.0) as usize {
            let p = in_blob(1.0);
            ctx.set_fill_style(rgba(0.85 + 0.25 * rnd(), 0.15 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.arc(p.0, p.1, px_w(0.25 + 0.6 * rnd()), 0, PI * 2.0);
            ctx.fill();
        }
        for _ in 0..3 + (rnd() * 5.0) as usize {
            let p = in_blob(0.7);
            let r = (3.0 + 7.0 * rnd()) * mm;
            let gr = ctx.create_radial_gradient(p.0, p.1, 0, p.0, p.1, r);
            gr.add_color_stop(0, &rgba(0.92, 0.4));
            gr.add_color_stop(0.5, &rgba(0.92, 0.18));
            gr.add_color_stop(1, &rgba(0.92, 0.0));
            ctx.set_fill_style(&gr);
            ctx.fill_rect(p.0 - r, p.1 - r, r * 2.0, r * 2.0);
        }
        // 4. torn: holes through all of it where it has come away
        let mut img = ctx.get_image_data(0, 0, cw, ch);
        let (iw, ih) = (cw as usize, ch as usize);
        for j in 0..ih {
            for i in 0..iw {
                let (x, y) = ((i as f64 + 0.5) / cw, (j as f64 + 0.5) / ch);
                let q = ((x * cw - bx) / brx).hypot((y * ch - by) / bry);
                let keep = sstep(0.3, 0.46, fbm(seed ^ 0x7a, x * 4.0, y * 2.5, 4) + 0.18 * (-q * q).exp());
                let k = (j * iw + i) * 4 + 3;
                img.data[k] = (f64::from(img.data[k]) * keep).round() as u8;
            }
        }
        ctx.put_image_data(&img, 0, 0);
        // a fly, long dead, wrapped in it
        if let Some((spin, drop, along)) = insect {
            let p = (bx + (along - 0.5) * brx, by + (drop - 0.5) * bry);
            ctx.save();
            ctx.translate(p.0, p.1);
            ctx.rotate(spin * PI * 2.0);
            ctx.set_stroke_style("rgba(36,30,24,0.8)");
            ctx.set_line_width(0.5 * mm);
            for k in 0..6 {
                let (sx, sy) = (if k < 3 { -1.0 } else { 1.0 }, (k % 3) as f64 - 1.0);
                ctx.begin_path();
                ctx.move_to(sx * 0.8 * mm, sy * 1.2 * mm);
                ctx.line_to(sx * 2.6 * mm, sy * 2.2 * mm);
                ctx.stroke();
            }
            ctx.set_fill_style("rgba(190,186,176,0.35)");
            ctx.begin_path();
            ctx.ellipse(0.0, 1.8 * mm, 1.4 * mm, 2.6 * mm, 0.3, 0, PI * 2.0);
            ctx.fill();
            ctx.set_fill_style("rgba(34,28,22,0.92)");
            ctx.begin_path();
            ctx.ellipse(0.0, 0.0, 1.5 * mm, 2.6 * mm, 0.0, 0, PI * 2.0);
            ctx.fill();
            ctx.restore();
        }
    });
    mesh(geo, &lambert!({ map: &tex, transparent: true, depthWrite: false, side: DoubleSide }))
}
