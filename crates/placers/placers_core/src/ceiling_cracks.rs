//! `place_ceiling_cracks`
use game::prelude::*;
use placers_kit::noise::vn;
use placers_kit::paint::mark_mat;
use placers_kit::paint::mark_tex;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Ceiling cracks: 20% chance per level, 1-2 cracks. A crack is in what
// the ceiling is (`docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`): a suspended ceiling's mineral-fibre tile
// cracks within itself, edge to edge across the one tile (the grid holds
// each tile apart), its two halves a little out of line; a concrete slab or a
// plaster ceiling cracks in a jagged, branching line that thins to nothing at
// its ends, white with leached salts along a slab's. Painted on a
// transparent plane under the ceiling.
pub(crate) fn place_ceiling_cracks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        // the old crack's draws: its heading, then three runs, each its
        // length and the turn after it (the line it follows on a slab or in
        // plaster; its kinks across a tile)
        let mut ang = rnd() * PI * 2.0;
        let mut path = vec![(0.0f64, 0.0f64)];
        let mut turns = vec![];
        for _s in 0..3 {
            let seg_len = 0.35 + rnd() * 0.25;
            let (px, pz) = *path.last().unwrap();
            path.push((px + ang.cos() * seg_len, pz + ang.sin() * seg_len));
            let turn = (rnd() - 0.5) * 1.0;
            turns.push(turn);
            ang += turn;
        }
        let crack_group = group();
        crack_group.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, WALL_H - 0.005, g.wz(z) + (rnd() - 0.5) * 1.6);
        let yaw = rnd() * PI * 2.0;
        let cp = crack_group.pos();
        let ceiling = ceiling_kind(g, x, z);
        if ceiling == 0 {
            // one tile of the grid, the one over where it was (or the next
            // one along, off a fitting); edge to edge at the old heading
            let (cx, cz) = ceiling_cell(g, cp.x, cp.z);
            let side = CELL / 6.0;
            let plane = mesh(plane_geo!(side, side), tile_crack_mat(yaw, &turns, path[1].0.hypot(path[1].1)));
            plane.rotation().x = PI / 2.0;
            plane.position().set(cx - cp.x, 0, cz - cp.z);
            crack_group.add(&plane);
        } else {
            crack_group.rotation().y = yaw;
            let (x0, x1) = path.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
            let (z0, z1) = path.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
            let (pw, ph) = (x1 - x0 + 0.3, z1 - z0 + 0.3);
            let (mx, mz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
            let local: Vec<(f64, f64)> = path.iter().map(|p| (p.0 - mx, p.1 - mz)).collect();
            let plane = mesh(plane_geo!(pw, ph), slab_crack_mat(pw, ph, &local, ceiling == 1));
            plane.rotation().x = PI / 2.0;
            plane.position().set(mx, 0, mz);
            crack_group.add(&plane);
        }
        g.world.add(&crack_group);
        // 30% chance: 2-4 bits fallen from it on the floor below (8-12
        // when a piece came down): of the tile (aged off-white mineral
        // fibre), or chips of the slab or the plaster (`core/marks_on_the_rooms_surfaces.py`)
        if rnd() < 0.30 {
            // (what it was made of, at its own albedo: a tile's aged
            // mineral fibre, the slab's concrete, the plaster's lime)
            let chunk_color = match ceiling {
                0 => 0xb0a998,
                1 => 0x8e897e,
                _ => 0xc2baa8,
            };
            let chunk_mat = lambert!({ color: chunk_color });
            // 10% chance: major-fall cluster (8-12 chunks).
            let is_major = rnd() < 0.10;
            let chunk_count = if is_major { 8 + (rnd() * 5.0).floor() as i32 } else { 2 + (rnd() * 3.0).floor() as i32 };
            let bits = if ceiling == 0 {
                ["Real__ceilingCracksTileBitA", "Real__ceilingCracksTileBitB", "Real__ceilingCracksTileBitC"]
            } else {
                ["Real__ceilingCracksChipA", "Real__ceilingCracksChipB", "Real__ceilingCracksChipC"]
            };
            for _ch in 0..chunk_count {
                // the old chunk's three sizes: now which bit it is, and how
                // big (within 13% of as modelled)
                let (sa, sb, sc) = (rnd(), rnd(), rnd());
                let geo = game::assets::props::prop_part_as_modelled(bits[(sa * 3.0) as usize % 3])
                    .unwrap_or_else(|| box_geo!(0.04, 0.016, 0.04));
                let chunk = mesh(geo, &chunk_mat);
                let s = 0.87 + 0.13 * (sb + sc);
                chunk.scale().set(s, s, s);
                let ch_ang = rnd() * PI * 2.0;
                let ch_rad = 0.08 + rnd() * (if is_major { 0.55 } else { 0.25 });
                let ch_sink = ((rnd() - 0.5) * 0.005).abs();
                chunk.position().set(cp.x + ch_ang.cos() * ch_rad, 0, cp.z + ch_ang.sin() * ch_rad);
                chunk.rotation().set((rnd() - 0.5) * 0.6, rnd() * PI * 2.0, (rnd() - 0.5) * 0.6);
                // rest the tilted chunk's lowest corner on the floor (settled in ≤ 2.5 mm)
                chunk.update_matrix_world(true);
                let min_y = box_from_object(&chunk).min.y;
                chunk.position().y = -min_y - ch_sink;
                g.world.add(&chunk);
            }
        }
        i += 1;
    }
}

/// Bounds of `o` and its subtree (not precise): each geometry's bounding box
/// through its object's world matrix (call after `update_matrix_world`).
fn box_from_object(o: &Obj) -> Box3 {
    let mut b = Box3::empty();
    o.traverse(&mut |c| {
        c.update_world_matrix(false, false);
        if let Some(geo) = c.try_geo() {
            let mut gb = geo.bounding_box();
            gb.apply_matrix4(&c.matrix_world());
            b.union(&gb);
        }
    });
    b
}

/// What the ceiling over tile (x, z) is: 0 mineral-fibre tiles on a T-bar
/// grid, 1 a concrete slab, 2 plaster (`World::ceil_mat_key`)
fn ceiling_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.ceil_mat) {
        Some("slab") => 1,
        Some("plaster") => 2,
        _ => 0,
    }
}

/// The centre of the ceiling tile over world (px, pz) on the T-bar grid
/// (`visuals::textures::CEIL_CELLS`: 6 tiles a 4 m tile, the bars through the
/// tile's edge along x and half a tile off it along z), or the next one along
/// z where a fitting hung at the tile's centre fills it (2 × 1 of them)
fn ceiling_cell(g: &Game, px: f64, pz: f64) -> (f64, f64) {
    let c = CELL / 6.0;
    let (tx, tz) = (g.wx(g.tx(px)), g.wz(g.tz(pz)));
    let cx = tx + ((px - tx) / c).floor() * c + c / 2.0;
    let mut cz = tz + ((pz - tz - c / 2.0) / c).floor() * c + c;
    if (cx - tx).abs() < c && (cz - tz).abs() < c / 2.0 {
        cz += if pz > tz { c } else { -c };
    }
    (cx, cz)
}

/// A jagged path from `a` to `b`: midpoint displacement, `depth` times, each
/// midpoint pushed sideways by up to `rough` of its span (own stream)
fn jagged(a: (f64, f64), b: (f64, f64), depth: u32, rough: f64) -> Vec<(f64, f64)> {
    let mut pts = vec![a, b];
    for _ in 0..depth {
        let mut next = Vec::with_capacity(pts.len() * 2);
        for w in pts.windows(2) {
            let (p, q) = (w[0], w[1]);
            let off = (rnd() - 0.5) * 2.0 * rough;
            next.push(p);
            next.push(((p.0 + q.0) / 2.0 - (q.1 - p.1) * off, (p.1 + q.1) / 2.0 + (q.0 - p.0) * off));
        }
        next.push(b);
        pts = next;
    }
    pts
}

/// Strokes `pts` (px) segment by segment, `width(t)` px wide at t (0..1
/// along it), round-capped, in `style`
fn stroke_tapered(ctx: &mut Canvas, pts: &[(f64, f64)], style: &str, width: impl Fn(f64) -> f64) {
    ctx.set_stroke_style(style);
    ctx.set_line_cap("round");
    let n = pts.len().saturating_sub(1);
    for k in 0..n {
        ctx.set_line_width(width((k as f64 + 0.5) / n as f64).max(0.2));
        ctx.begin_path();
        ctx.move_to(pts[k].0, pts[k].1);
        ctx.line_to(pts[k + 1].0, pts[k + 1].1);
        ctx.stroke();
    }
}

/// Strokes `pts` (px) as one path, `width` px, in `style`
fn stroke_path(ctx: &mut Canvas, pts: &[(f64, f64)], style: &str, width: f64) {
    if pts.len() < 2 {
        return;
    }
    ctx.set_stroke_style(style);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.set_line_width(width);
    ctx.begin_path();
    ctx.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        ctx.line_to(p.0, p.1);
    }
    ctx.stroke();
}

/// A branch off a crack at `pts[k]`, turned by `turn` from the crack's
/// heading there, `len` long (in `pts`' units), jagged
fn crack_branch(pts: &[(f64, f64)], k: usize, turn: f64, len: f64) -> Vec<(f64, f64)> {
    let (p, q) = (pts[k], pts[(k + 1).min(pts.len() - 1)]);
    let a = (q.1 - p.1).atan2(q.0 - p.0) + turn;
    jagged(p, (p.0 + a.cos() * len, p.1 + a.sin() * len), 4, 0.2)
}

/// A crack across one mineral-fibre ceiling tile: its plane is the tile
/// (`CELL / 6` square, the grid's bars at its edges), the crack edge to edge
/// at heading `yaw`, kinked by the old crack's `turns`, its halves a little
/// out of line (a broken edge lit on one side, a shadow on the other)
fn tile_crack_mat(yaw: f64, turns: &[f64], span: f64) -> Mat {
    let side = CELL / 6.0;
    let turns = turns.to_vec();
    let tex = mark_tex(512.0, 512.0, [0.1, 0.09, 0.08], move |ctx, cw, _ch| {
        let s = side / cw; // m a px
        let hs = side / 2.0 - 0.0125; // the tile's face inside the bars' flanges
        let to_px = |p: (f64, f64)| ((p.0 + side / 2.0) / s, (side / 2.0 - p.1) / s);
        // nothing on the bars: only the tile's face
        let inset = (side / 2.0 - hs) / s;
        ctx.begin_path();
        ctx.rect(inset, inset, cw - 2.0 * inset, cw - 2.0 * inset);
        ctx.clip();
        // the line it breaks along: through a point near the middle, at yaw
        let (dx, dz) = (yaw.cos(), yaw.sin());
        let off = (rnd() - 0.5) * 0.3 * hs;
        let c0 = (-dz * off, dx * off);
        // where that line meets the tile's edges either way
        let reach = |sg: f64| {
            let tx = if dx.abs() > 1e-9 { (hs * dx.signum() * sg - c0.0) / (dx * sg) } else { f64::MAX };
            let tz = if dz.abs() > 1e-9 { (hs * dz.signum() * sg - c0.1) / (dz * sg) } else { f64::MAX };
            tx.min(tz)
        };
        let (ta, tb) = (reach(-1.0), reach(1.0));
        let a = (c0.0 - dx * ta, c0.1 - dz * ta);
        let mut b = (c0.0 + dx * tb, c0.1 + dz * tb);
        // sometimes it has not broken through yet: it stops inside the tile
        let open_end = rnd() < 0.4;
        if open_end {
            let k = 0.55 + 0.3 * rnd();
            b = (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
        }
        // kinked where the old crack turned
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let (nx, nz) = (-(b.1 - a.1) / len, (b.0 - a.0) / len);
        let at = |t: f64, kink: f64| (a.0 + (b.0 - a.0) * t + nx * kink * len * 0.12, a.1 + (b.1 - a.1) * t + nz * kink * len * 0.12);
        let macro_pts = [a, at(1.0 / 3.0, turns[0]), at(2.0 / 3.0, turns[1]), b];
        let mut pts = vec![];
        for w in macro_pts.windows(2) {
            let mut seg = jagged(w[0], w[1], 5, 0.16);
            if !pts.is_empty() {
                seg.remove(0);
            }
            pts.extend(seg);
        }
        let clamp_in = |p: (f64, f64)| (p.0.clamp(-hs, hs), p.1.clamp(-hs, hs));
        let pts: Vec<(f64, f64)> = pts.into_iter().map(|p| to_px(clamp_in(p))).collect();
        // how wide it is open: most at the edge it started from, thinning
        // to nothing where it stops inside
        let wmax = (0.0045 + 0.002 * (span - 0.35) / 0.25) / s;
        let width = |t: f64| wmax * if open_end { (1.0 - t).powf(0.7) } else { 0.75 + 0.25 * (PI * t).sin() };
        // the edge of the half that dropped, lit; the shadow it casts
        let lit: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 + nx * 1.6, p.1 - nz * 1.6)).collect();
        let shade: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - nx * 4.0, p.1 + nz * 4.0)).collect();
        let sag: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - nx * 30.0, p.1 + nz * 30.0)).collect();
        // the dropped half a shade darker, most near the break
        stroke_path(ctx, &sag, "rgba(40,34,28,0.07)", 60.0);
        stroke_path(ctx, &shade, "rgba(38,33,27,0.2)", 9.0);
        stroke_tapered(ctx, &lit, "rgba(222,214,196,0.6)", |t| width(t) * 0.8 + 0.8);
        stroke_tapered(ctx, &pts, "rgb(24,21,18)", width);
        // a branch or two, finer
        for _ in 0..(rnd() * 2.6) as usize {
            let k = ((0.2 + 0.6 * rnd()) * pts.len() as f64) as usize;
            let turn = (0.5 + 0.6 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let br: Vec<(f64, f64)> = crack_branch(&pts, k, turn, (0.04 + 0.1 * rnd()) / s)
                .into_iter()
                .map(|p| (p.0.clamp(0.0125 / s, cw - 0.0125 / s), p.1.clamp(0.0125 / s, cw - 0.0125 / s)))
                .collect();
            let w0 = width(k as f64 / pts.len() as f64) * 0.6;
            stroke_tapered(ctx, &br, "rgb(30,26,22)", |t| w0 * (1.0 - t).powf(0.8));
        }
        // crumbs of the broken face along it
        ctx.set_fill_style("rgba(40,35,30,0.6)");
        for _ in 0..(6.0 + 10.0 * rnd()) as usize {
            let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
            ctx.begin_path();
            ctx.arc(p.0 + (rnd() - 0.5) * 4.0, p.1 + (rnd() - 0.5) * 4.0, 0.5 + rnd(), 0, PI * 2.0);
            ctx.fill();
        }
    });
    mark_mat(&tex)
}

/// A crack in a concrete slab (`slab`) or a plaster ceiling, along `path`
/// (m, the plane's own frame), on a plane `pw × ph` m: jagged and branching,
/// widest along its middle and thinning to nothing at its ends; on a slab,
/// white with the salts the water leached out of it here and there, in
/// plaster the paint flaked along its lips
fn slab_crack_mat(pw: f64, ph: f64, path: &[(f64, f64)], slab: bool) -> Mat {
    let s = (pw.max(ph) / 1024.0).max(0.0016);
    let (tw, th) = ((pw / s).round(), (ph / s).round());
    let path = path.to_vec();
    let tex = mark_tex(tw, th, [0.1, 0.09, 0.08], move |ctx, _, _| {
        let to_px = |p: (f64, f64)| ((p.0 + pw / 2.0) / s, (ph / 2.0 - p.1) / s);
        let mut pts = vec![];
        for w in path.windows(2) {
            let mut seg = jagged(to_px(w[0]), to_px(w[1]), 6, 0.13);
            if !pts.is_empty() {
                seg.remove(0);
            }
            pts.extend(seg);
        }
        let seed = (rnd() * 4.0e9) as u32;
        let wmax = (0.003 + 0.0015 * rnd()) / s;
        let width = |t: f64| wmax * (PI * t).sin().powf(0.6) * (0.7 + 0.6 * vn(seed, t * 9.0, 0.5));
        let mut branches = vec![];
        for _ in 0..2 + (rnd() * 3.0) as usize {
            let t = 0.1 + 0.8 * rnd();
            let k = (t * pts.len() as f64) as usize;
            let turn = (0.4 + 0.6 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let br = crack_branch(&pts, k, turn, (0.06 + 0.24 * rnd()) / s);
            let w0 = width(t) * 0.6;
            branches.push((br, w0));
        }
        if slab {
            // the salts: a soft white along stretches of it, a rust stain
            for _ in 0..1 + (rnd() * 2.0) as usize {
                let (a, b) = (rnd(), rnd());
                let (k0, k1) = ((a.min(b) * pts.len() as f64) as usize, (a.max(b) * pts.len() as f64) as usize);
                if k1 > k0 + 1 {
                    stroke_path(ctx, &pts[k0..k1], "rgba(214,209,196,0.12)", 12.0 + 8.0 * rnd());
                    stroke_path(ctx, &pts[k0..k1], "rgba(222,218,206,0.22)", 4.0);
                }
            }
            let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
            let r = 10.0 + 16.0 * rnd();
            let gr = ctx.create_radial_gradient(p.0, p.1, 1, p.0, p.1, r);
            gr.add_color_stop(0, "rgba(110,78,48,0.16)");
            gr.add_color_stop(1, "rgba(110,78,48,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(p.0 - r, p.1 - r, r * 2.0, r * 2.0);
        } else {
            // the paint's lips along it, and flakes of it gone
            let lip: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 + 1.2, p.1 - 1.2)).collect();
            stroke_tapered(ctx, &lip, "rgba(232,226,212,0.4)", |t| width(t) * 0.7 + 0.5);
            ctx.set_fill_style("rgba(150,138,120,0.35)");
            for _ in 0..(4.0 + 6.0 * rnd()) as usize {
                let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
                ctx.begin_path();
                ctx.ellipse(
                    p.0 + (rnd() - 0.5) * 6.0,
                    p.1 + (rnd() - 0.5) * 6.0,
                    1.5 + 3.0 * rnd(),
                    1.0 + 2.0 * rnd(),
                    rnd() * PI,
                    0,
                    PI * 2.0,
                );
                ctx.fill();
            }
        }
        // its shadow side, then the crack itself
        let shade: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - 1.0, p.1 + 1.0)).collect();
        stroke_tapered(ctx, &shade, "rgba(30,27,24,0.25)", |t| width(t) + 1.0);
        let core = if slab { "rgb(26,24,22)" } else { "rgb(34,30,26)" };
        stroke_tapered(ctx, &pts, core, width);
        for (br, w0) in &branches {
            stroke_tapered(ctx, br, core, |t| w0 * (1.0 - t).powf(0.8));
            // and now and then a finer one off the branch
            if rnd() < 0.4 && br.len() > 4 {
                let k = br.len() / 2;
                let sub = crack_branch(br, k, (rnd() - 0.5) * 1.6, (0.03 + 0.06 * rnd()) / s);
                stroke_tapered(ctx, &sub, core, |t| w0 * 0.5 * (1.0 - t));
            }
        }
    });
    mark_mat(&tex)
}
