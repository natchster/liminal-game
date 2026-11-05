//! Prop placers: small curios and antiques — scrap papers, chalk marks, keys and
//! padlocks, vintage cameras, backpacks, gift boxes, party horns, scales,
//! abacuses, telegraph keys, sewing machines, brass lanterns, suitcases,
//! phonographs, rotary phones, typewriters, globes, telescopes, scrolls,
//! compasses and pocket watches.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

pub mod consts;
use consts::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_scrap_papers", place_scrap_papers));
    r.push(("place_chalk_marks", place_chalk_marks));
    r.push(("place_small_keys", place_small_keys));
    r.push(("place_vintage_cameras", place_vintage_cameras));
    r.push(("place_backpacks", place_backpacks));
    r.push(("place_small_gift_boxes", place_small_gift_boxes));
    r.push(("place_small_party_horns", place_small_party_horns));
    r.push(("place_small_scales", place_small_scales));
    r.push(("place_small_abacuses", place_small_abacuses));
    r.push(("place_small_telegraph_keys", place_small_telegraph_keys));
    r.push(("place_small_sewing_machines", place_small_sewing_machines));
    r.push(("place_hanging_brass_lanterns", place_hanging_brass_lanterns));
    r.push(("place_small_suitcases", place_small_suitcases));
    r.push(("place_small_phonographs", place_small_phonographs));
    r.push(("place_small_rotary_phones", place_small_rotary_phones));
    r.push(("place_small_typewriters", place_small_typewriters));
    r.push(("place_small_globes", place_small_globes));
    r.push(("place_small_telescopes", place_small_telescopes));
    r.push(("place_small_padlock_keys", place_small_padlock_keys));
    r.push(("place_parchment_scrolls", place_parchment_scrolls));
    r.push(("place_small_compasses", place_small_compasses));
    r.push(("place_small_pocket_watches", place_small_pocket_watches));
}

// ------------------------------------------------------------------ helpers
/// Every open interior tile, row by row (the `open_tiles` loop every placer repeats).
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}
/// In-place Fisher-Yates exactly loops do it.
fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}
/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(v: &[T]) -> T {
    v[(rnd() * v.len() as f64).floor() as usize]
}

/// A wall-facing slot: open tile (x, z) with a wall at (x + dx, z + dz).
#[derive(Clone, Copy)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// Every (open tile, wall direction) pair whose neighbour is a wall or off the grid.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// `a.clone().sub(b)`
fn vsub(a: Vec3, b: Vec3) -> Vec3 {
    let mut v = a;
    v.sub(&b);
    v
}

// R246 discarded scrap papers: 1-3 sheets of office paper per level, balled up
// and thrown away or smoothed out again and dropped (Blender batch r08: a
// sheet of A4 balled tight, one crumpled loosely, one flattened out; printed).
fn place_scrap_papers(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    let count = 1 + (rnd() * 3.0).floor() as usize; // 1-3 scraps
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let mut wad_mat: Option<Mat> = None;
    let mut page_mat: Option<Mat> = None;
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // the old crumple's sizes: which scrap it is (x), how big (z)
        let size_x = 0.06 + rnd() * 0.05;
        let size_y = 0.04 + rnd() * 0.03;
        let size_z = 0.06 + rnd() * 0.05;
        let u = (size_x - 0.06) / 0.05;
        let (name, sheet) = if u < 0.42 {
            ("Real__paperBall", false)
        } else if u < 0.78 {
            ("Real__paperWad", false)
        } else {
            ("Real__paperSheet", true)
        };
        let mat =
            if sheet { page_mat.get_or_insert_with(scrap_page_mat).clone() } else { wad_mat.get_or_insert_with(scrap_wad_mat).clone() };
        let geo = prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(size_x, size_y, size_z));
        let scrap = mesh(geo, &mat);
        // the old tilts are kept as draws: a ball rests on its flattest side,
        // a sheet lies where it fell
        let _rx = rnd();
        let ry = rnd() * PI * 2.0;
        let _rz = rnd();
        // a ball a little bigger or smaller (a sheet is A4)
        let k = if sheet { 1.0 } else { 0.9 + (size_z - 0.06) / 0.05 * 0.2 };
        scrap.scale().set(k, k, k);
        scrap.rotation().y = ry;
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        scrap.position().set(g.wx(x) + ox, 0, g.wz(z) + oz);
        g.world.add(&scrap);
        i += 1;
    }
}

/// Office paper balled up: fragments of its print turned every way, the
/// creases' grime, a footprint's dust (laid on the ball by its box faces).
fn scrap_wad_mat() -> Mat {
    let tex = canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("#e4e0d4");
        ctx.fill_rect(0, 0, w, h);
        // the paper's own unevenness
        for _ in 0..260 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 212.0 + rnd() * 30.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.18)", v - 4.0, v - 14.0));
            ctx.fill_rect(x, y, 3.0 + rnd() * 9.0, 3.0 + rnd() * 9.0);
        }
        // the print: patches of type, each a facet of the sheet turned its own way
        for _ in 0..16 {
            let (cx, cy) = (rnd() * w, rnd() * h);
            let r = 20.0 + rnd() * 30.0;
            ctx.save();
            ctx.begin_path();
            let n = 5 + (rnd() * 3.0).floor() as usize;
            for k in 0..n {
                let t = (k as f64 + rnd() * 0.4) / n as f64 * PI * 2.0;
                let rr = r * (0.6 + 0.4 * rnd());
                if k == 0 {
                    ctx.move_to(cx + t.cos() * rr, cy + t.sin() * rr);
                } else {
                    ctx.line_to(cx + t.cos() * rr, cy + t.sin() * rr);
                }
            }
            ctx.close_path();
            ctx.clip();
            ctx.translate(cx, cy);
            ctx.rotate(rnd() * PI * 2.0);
            paper_type_lines(ctx, -r - 10.0, -r - 10.0, 2.0 * r + 20.0, 2.0 * r + 20.0, 9.0, 0.35 + rnd() * 0.3);
            ctx.restore();
        }
        // crease lines: grime in the valleys
        ctx.set_line_width(0.8);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 10.0 + rnd() * 40.0;
            ctx.set_stroke_style(format!("rgba(70,64,52,{:.2})", 0.08 + rnd() * 0.12));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // dust from the floor
        for _ in 0..16 {
            let at = (rnd() * w, rnd() * h);
            scrap_soft(ctx, at, 8.0 + rnd() * 20.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (96, 86, 66, 0.06 + rnd() * 0.12));
        }
    });
    lambert!({ map: &tex, color: 0xffffff })
}

/// The flattened sheet: a stock count printed on it (a title, a ruled table
/// of stationery), its creases, a shoe's dusty print across it.
fn scrap_page_mat() -> Mat {
    let tex = canvas_tex_opt(384, 544, false, |ctx, w, h| {
        ctx.set_fill_style("#e2ded2");
        ctx.fill_rect(0, 0, w, h);
        let ink = "rgba(36,34,32,0.86)";
        ctx.set_fill_style(ink);
        ctx.set_text_align("left");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 17px Helvetica");
        ctx.fill_text("Stock Count Sheet", 34, 58);
        ctx.set_font("10px Helvetica");
        ctx.fill_text("Stationery store, ground floor", 34, 74);
        ctx.fill_text("Counted by: ____________   Date: ___ / ___ / ______", 34, 92);
        // the table
        let rows = [
            ("1043-A", "Copy paper A4, 80 g/m2, ream", "112"),
            ("1043-B", "Copy paper A3, ream", "14"),
            ("2210", "Toner cartridge, black", "6"),
            ("2211", "Toner cartridge, cyan", "2"),
            ("3105", "Staples 26/6, box of 5000", "38"),
            ("3112", "Binder clips 32 mm, box", "21"),
            ("3120", "Paper clips, box of 100", "45"),
            ("4001", "Folders, manila, pack of 50", "9"),
            ("4007", "Lever arch files, A4", "27"),
            ("5020", "Ballpoint pens, blue, box", "16"),
            ("5021", "Ballpoint pens, black, box", "11"),
            ("5040", "Highlighters, assorted", "7"),
            ("6110", "Envelopes C4, box of 250", "4"),
            ("6115", "Envelopes DL, box of 500", "5"),
            ("7002", "Labels, address, sheet", "300"),
            ("7010", "Tape, clear, 19 mm", "24"),
            ("7015", "Sticky notes 76 x 76", "60"),
            ("8001", "Correction fluid", "13"),
        ];
        let (x0, x1, x2, x3, x4) = (34.0, 92.0, 270.0, 312.0, w - 30.0);
        let top = 112.0;
        let rh = 19.0;
        ctx.set_stroke_style("rgba(40,38,36,0.7)");
        ctx.set_line_width(1.0);
        for k in 0..=rows.len() + 1 {
            let y = top + k as f64 * rh;
            ctx.begin_path();
            ctx.move_to(x0, y);
            ctx.line_to(x4, y);
            ctx.stroke();
        }
        for x in [x0, x1, x2, x3, x4] {
            ctx.begin_path();
            ctx.move_to(x, top);
            ctx.line_to(x, top + (rows.len() + 1) as f64 * rh);
            ctx.stroke();
        }
        ctx.set_font("bold 10px Helvetica");
        for (x, t) in [(x0, "Item"), (x1, "Description"), (x2, "Qty"), (x3, "Counted")] {
            ctx.fill_text(t, x + 4.0, top + 13.5);
        }
        ctx.set_font("10px Times");
        for (k, (a, b, c)) in rows.iter().enumerate() {
            let y = top + (k + 1) as f64 * rh + 13.5;
            ctx.fill_text(a, x0 + 4.0, y);
            ctx.fill_text(b, x1 + 4.0, y);
            ctx.fill_text(c, x2 + 4.0, y);
        }
        ctx.set_font("9px Helvetica");
        ctx.fill_text("Page 2 of 3", w / 2.0 - 22.0, h - 26.0);
        // the creases: thin lines darker on one side, lighter on the other
        for _ in 0..34 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 30.0 + rnd() * 140.0;
            let (dx, dy) = (a.cos() * l, a.sin() * l);
            ctx.set_line_width(1.0);
            ctx.set_stroke_style(format!("rgba(80,72,58,{:.2})", 0.10 + rnd() * 0.15));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + dx, y + dy);
            ctx.stroke();
            ctx.set_stroke_style("rgba(255,255,250,0.10)");
            ctx.begin_path();
            ctx.move_to(x + 1.2, y + 1.2);
            ctx.line_to(x + dx + 1.2, y + dy + 1.2);
            ctx.stroke();
        }
        // a shoe stepped on it: the dusty print of a sole's tread, half of it
        let (sx, sy, sa) = (w * (0.35 + rnd() * 0.3), h * (0.45 + rnd() * 0.3), rnd() * PI);
        ctx.save();
        ctx.translate(sx, sy);
        ctx.rotate(sa);
        ctx.begin_path();
        ctx.ellipse(0, -40.0, 46.0, 62.0, 0, 0, 7);
        ctx.ellipse(0, 70.0, 34.0, 38.0, 0, 0, 7);
        ctx.clip();
        for k in 0..26 {
            let y = -110.0 + k as f64 * 8.0;
            ctx.set_fill_style(format!("rgba(92,82,64,{:.2})", 0.10 + rnd() * 0.10));
            ctx.fill_rect(-50.0 + rnd() * 6.0, y, 100.0, 4.0);
        }
        ctx.restore();
        // dirt at its edges, where it lay longest on the floor
        for _ in 0..40 {
            let (x, y) = if rnd() < 0.5 {
                (rnd() * w, if rnd() < 0.5 { rnd() * 30.0 } else { h - rnd() * 30.0 })
            } else {
                (if rnd() < 0.5 { rnd() * 24.0 } else { w - rnd() * 24.0 }, rnd() * h)
            };
            scrap_soft(ctx, (x, y), 8.0 + rnd() * 18.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (96, 86, 66, 0.08 + rnd() * 0.14));
        }
    });
    lambert!({ map: &tex, color: 0xffffff, side: DoubleSide })
}
/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
fn scrap_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Lines of small printed type (words of an office's papers) filling the box
/// (x, y, w, h), `size` px, its ink `alpha`.
fn paper_type_lines(ctx: &mut Canvas, x: f64, y: f64, w: f64, h: f64, size: f64, alpha: f64) {
    const WORDS: [&str; 32] = [
        "the", "and", "of", "to", "for", "account", "total", "date", "please", "report", "per", "unit", "page", "item", "order", "review",
        "with", "that", "office", "copy", "received", "amount", "number", "dept.", "on", "is", "be", "as", "by", "this", "invoice",
        "meeting",
    ];
    ctx.set_fill_style(format!("rgba(34,32,30,{alpha:.2})"));
    ctx.set_font(&format!("{size:.0}px Times"));
    ctx.set_text_align("left");
    let mut yy = y + size;
    while yy < y + h {
        let mut xx = x + rnd() * 6.0;
        let end = x + w - rnd() * w * 0.25;
        while xx < end {
            let word = WORDS[(rnd() * WORDS.len() as f64) as usize % WORDS.len()];
            ctx.fill_text(word, xx, yy);
            xx += ctx.measure_text(word).width + size * 0.35;
        }
        yy += size * 1.3;
    }
}

// R246 chalk marks: 0-3 marks in chalk on random walls, in a stranger's hand:
// a number underlined, a looping scribble where someone tried the chalk, a
// fitter's measure (a line with a tick at each end, its length over it);
// never a lone stroke, a tally, an arrow or a circle (hers and the place's
// are those), the grain of the chalk dragged over the wall, its dust under
// it. The mark is laid on the wall in upright strips each no wider than the
// old streak's settling probe takes in at once (`CHALK_STRIP`), so settling
// finds the same wall face for it and pins nothing new.
fn place_chalk_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.35 {
        return;
    } // 35% chance of NO chalk this level
    let count = 1 + (rnd() * 3.0).floor() as usize; // 1-3 marks
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let chalk_color = chalk_color_by_weather(g.weather_key());
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // the old streak's width and height: which scrawl, and its size
        let w = 0.012 + rnd() * 0.006;
        let h = 0.10 + rnd() * 0.06;
        let u = (w - 0.012) / 0.006;
        let kind = ((u * 3.0).floor() as usize).min(2);
        let code = ((u * 1.0e6) as u64) ^ (((h - 0.10) / 0.06 * 1.0e6) as u64).rotate_left(17);
        let full = CHALK_PLANE * (0.88 + (h - 0.10) / 0.06 * 0.24);
        let tex = chalk_scrawl_tex(kind, code);
        let chalk_mat = lambert!({
            map: &tex,
            color: chalk_color,
            transparent: true,
            depthWrite: false,
            side: DoubleSide,
            polygonOffset: true,
            polygonOffsetFactor: -2,
            polygonOffsetUnits: -2
        });
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        let mid_height = 0.6 + rnd() * 0.8; // 0.6 - 1.4 m
        let (turn, mx, mz) = if c.dx.abs() == 1 {
            (if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 }, wall_x, wall_z + (rnd() - 0.5) * 1.4)
        } else {
            (if c.dz > 0 { PI } else { 0.0 }, wall_x + (rnd() - 0.5) * 1.4, wall_z)
        };
        // where the wall's face steps (a jamb, a pillar, a corner) within its
        // width, a smaller one, so none of it hangs off an edge
        let (ax, az) = (turn.cos(), -turn.sin());
        let (px, pz) = (mx - c.dx as f64 * 0.01, mz - c.dz as f64 * 0.01);
        let inside = |off: f64| g.circle_hits_wall(px + ax * off, pz + az * off, 0.01);
        let mid = inside(0.0);
        let mut size = full;
        while size > 0.09 && (inside(-size / 2.0) != mid || inside(size / 2.0) != mid) {
            size *= 0.8;
        }
        // the strips, left to right across the wall, each at the old streak's
        // place with its slice of the scrawl beside it (a group at the origin:
        // settling takes each strip by itself, where it took the streak)
        let mark = group();
        let n = (size / CHALK_STRIP).ceil().max(1.0) as usize;
        let sw = size / n as f64;
        for k in 0..n {
            let geo = plane_geo!(sw, size);
            let (u0, u1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
            geo.set_uv(vec![u0, 1.0, u1, 1.0, u0, 0.0, u1, 0.0]);
            geo.translate((k as f64 + 0.5) * sw - size / 2.0, 0, 0);
            let strip = mesh(geo, &chalk_mat);
            strip.rotation().y = turn;
            strip.position().set(mx, mid_height, mz);
            mark.add(&strip);
        }
        g.world.add(&mark);
        i += 1;
    }
}

/// The chalk mark's square (m): its scrawl in the upper part, the dust below.
const CHALK_PLANE: f64 = 0.22;
/// The widest strip a mark is laid in (m): settling probes a wall face only
/// at the middle of anything narrower than 8 cm, as it did the old streak.
const CHALK_STRIP: f64 = 0.07;

/// A scrawl in chalk on a wall (`kind` 0 a number underlined, 1 a looping
/// scribble, 2 a fitter's measure), drawn with a stick of blackboard chalk
/// (10 mm: a stroke 6-8 mm wide), its grain broken by the wall's tooth, its
/// dust fallen below it. White, tinted by the material.
fn chalk_scrawl_tex(kind: usize, code: u64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // 1 px = 0.86 mm; the scrawl in the upper two thirds
        let mut strokes: Vec<Vec<(f64, f64)>> = vec![];
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        // a number in a quick upright hand, its digits `dw` x `dh`, centred on cx
        let number = |strokes: &mut Vec<Vec<(f64, f64)>>, n: u64, cx: f64, y0: f64, dw: f64, dh: f64, slant: f64| -> f64 {
            let digits: Vec<u32> = n.to_string().chars().filter_map(|c| c.to_digit(10)).collect();
            let gap = dw * 0.28;
            let total = digits.len() as f64 * (dw + gap) - gap;
            let mut x0 = cx - total / 2.0;
            for d in digits {
                for st in chalk_digit(d) {
                    strokes.push(st.iter().map(|&(u, v)| jig((x0 + u * dw + (1.0 - v) * dh * slant, y0 + v * dh), 2.0)).collect());
                }
                x0 += dw + gap;
            }
            total
        };
        match kind {
            0 => {
                // a number, 9 cm tall, underlined quickly
                let slant = 0.06 + (code % 7) as f64 * 0.015;
                let total = number(&mut strokes, 2 + code % 97, w / 2.0, 28.0, 56.0, 100.0, slant);
                let ul_y = 28.0 + 100.0 + 14.0;
                strokes.push(vec![((w - total) / 2.0 - 10.0, ul_y + 3.0), (w / 2.0, ul_y), ((w + total) / 2.0 + 14.0, ul_y - 4.0)]);
            }
            1 => {
                // a looping scribble along a line, as a hand tries the chalk
                let loops = 5.0 + (code % 3) as f64;
                let mut pts = vec![];
                let (x0, x1, cy) = (34.0, w - 34.0, 92.0 + (code % 13) as f64);
                for k in 0..=220 {
                    let t = k as f64 / 220.0;
                    let a = t * loops * PI * 2.0;
                    let r = 24.0 + 9.0 * (t * 7.0 + code as f64 * 0.1).sin();
                    pts.push((x0 + (x1 - x0) * t + r * 0.55 * a.cos(), cy - r * 0.8 * a.sin() + 10.0 * (t * 3.0).sin()));
                }
                strokes.push(pts);
            }
            _ => {
                // a fitter's measure: a line across with a tick at each end,
                // its length in centimetres written over it
                let (x0, x1) = (30.0 + (code % 11) as f64, w - 28.0 - (code % 7) as f64);
                let y = 132.0 + (code % 9) as f64;
                strokes.push(vec![jig((x0, y), 2.0), jig(((x0 + x1) / 2.0, y + 1.5), 2.0), jig((x1, y - 2.0), 2.0)]);
                strokes.push(vec![jig((x0 + 1.5, y - 17.0), 1.5), jig((x0 - 1.0, y + 15.0), 1.5)]);
                strokes.push(vec![jig((x1 + 1.5, y - 19.0), 1.5), jig((x1 - 1.0, y + 13.0), 1.5)]);
                number(&mut strokes, 60 + code % 140, (x0 + x1) / 2.0, y - 78.0, 34.0, 58.0, 0.08);
            }
        }
        let mut low = 0.0f64;
        let (mut lx, mut hx) = (w, 0.0f64);
        for st in &strokes {
            let pts = chalk_smooth(st);
            chalk_stroke(ctx, &pts, 9.5);
            for &(x, y) in &pts {
                low = low.max(y);
                lx = lx.min(x);
                hx = hx.max(x);
            }
        }
        // its dust: a faint powder down the wall under the strokes, and specks
        for _ in 0..14 {
            let x = lx + rnd() * (hx - lx);
            let y = low + 4.0 + rnd() * 26.0;
            chalk_soft(ctx, x, y, 7.0 + rnd() * 10.0, 1.6 + rnd(), 0.05 + rnd() * 0.06);
        }
        for _ in 0..160 {
            let x = lx - 6.0 + rnd() * (hx - lx + 12.0);
            let y = low - 10.0 + rnd() * rnd() * (h - low + 6.0);
            ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.15 + rnd() * 0.45));
            ctx.fill_rect(x, y, 0.8 + rnd(), 0.8 + rnd());
        }
    })
}

/// A soft round of chalk dust at (x, y), `r` px, drawn out `k` times as tall
/// as it is wide (the context squashed, so its fade is never cut).
fn chalk_soft(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, alpha: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba(255,255,255,{alpha:.3})"));
    gr.add_color_stop(1, "rgba(255,255,255,0)");
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The strokes of a digit in a unit box (u across, v down), as a quick hand
/// draws it.
fn chalk_digit(d: u32) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
    };
    match d {
        0 => vec![arc(0.5, 0.5, 0.48, 0.5, -1.4, 4.95)],
        1 => vec![vec![(0.22, 0.2), (0.55, 0.0), (0.55, 1.0)]],
        2 => {
            let mut s = arc(0.5, 0.28, 0.45, 0.28, 3.4, 6.6);
            s.extend([(0.05, 1.0), (1.0, 0.98)]);
            vec![s]
        }
        3 => {
            let mut s = arc(0.45, 0.25, 0.45, 0.25, 3.6, 7.6);
            s.extend(arc(0.45, 0.73, 0.5, 0.27, -1.4, 2.6));
            vec![s]
        }
        4 => vec![vec![(0.7, 1.0), (0.7, 0.0), (0.0, 0.68), (1.0, 0.68)]],
        5 => {
            let mut s = vec![(0.9, 0.0), (0.18, 0.0), (0.12, 0.43)];
            s.extend(arc(0.45, 0.68, 0.5, 0.31, -2.4, 2.7));
            vec![s]
        }
        6 => {
            let mut s = vec![(0.85, 0.05), (0.45, 0.2), (0.08, 0.62)];
            s.extend(arc(0.5, 0.72, 0.43, 0.27, 3.1, 9.4));
            vec![s]
        }
        7 => vec![vec![(0.0, 0.02), (0.95, 0.0), (0.35, 1.0)]],
        8 => {
            let mut s = arc(0.5, 0.25, 0.38, 0.25, 1.6, 7.6);
            s.extend(arc(0.5, 0.74, 0.46, 0.26, -1.6, 4.7));
            vec![s]
        }
        _ => {
            let mut s = arc(0.48, 0.3, 0.44, 0.3, 0.2, 6.6);
            s.extend([(0.9, 0.4), (0.62, 1.0)]);
            vec![s]
        }
    }
}

/// A hand's line through the points (Chaikin, twice).
fn chalk_smooth(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut p = p.to_vec();
    for _ in 0..2 {
        if p.len() < 3 {
            break;
        }
        let mut q = vec![p[0]];
        for k in 0..p.len() - 1 {
            let (a, b) = (p[k], p[k + 1]);
            q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
            q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
        }
        q.push(p[p.len() - 1]);
        p = q;
    }
    p
}

/// The wall's tooth at (x, y) px: what catches the chalk (0..1), fixed for
/// the wall so every stroke skips in the same places.
fn chalk_tooth(x: f64, y: f64) -> f64 {
    let (i, j) = ((x / 1.6).floor() as i64, (y / 1.6).floor() as i64);
    let mut v = (i.wrapping_mul(374_761_393) ^ j.wrapping_mul(668_265_263)) as u64;
    v = (v ^ (v >> 13)).wrapping_mul(1_274_126_177);
    ((v ^ (v >> 16)) & 0xffff) as f64 / 65535.0
}

/// A stroke of chalk along the points: grains caught on the wall's tooth,
/// heavier where pressed, broken where it skipped, its ends lifting.
fn chalk_stroke(ctx: &mut Canvas, pts: &[(f64, f64)], width: f64) {
    let mut len = 0.0;
    for k in 1..pts.len() {
        len += ((pts[k].0 - pts[k - 1].0).powi(2) + (pts[k].1 - pts[k - 1].1).powi(2)).sqrt();
    }
    let mut s = 0.0;
    let wob = rnd() * 6.0;
    for k in 1..pts.len() {
        let (a, b) = (pts[k - 1], pts[k]);
        let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if seg < 1e-6 {
            continue;
        }
        let (tx, ty) = ((b.0 - a.0) / seg, (b.1 - a.1) / seg);
        let (nx, ny) = (-ty, tx);
        let steps = (seg / 0.7).ceil() as usize;
        for j in 0..steps {
            let t = j as f64 / steps as f64;
            let (cx, cy) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let at = (s + seg * t) / len.max(1e-6);
            let ends = (at / 0.06).min(1.0) * ((1.0 - at) / 0.1).min(1.0);
            let press = (0.62 + 0.25 * (at * 9.0 + wob).sin() + 0.13 * (at * 23.0 + wob).sin()) * (0.4 + 0.6 * ends);
            for _ in 0..6 {
                let off = (rnd() - 0.5) * width * (0.7 + 0.3 * ends);
                let (x, y) = (cx + nx * off, cy + ny * off);
                let edge = 1.0 - (2.0 * off / width).abs();
                if chalk_tooth(x, y) + 0.3 * rnd() < press * (0.75 + 0.6 * edge) {
                    ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.55 + 0.45 * rnd()));
                    ctx.fill_rect(x - 0.8, y - 0.8, 1.4 + rnd() * 1.0, 1.4 + rnd() * 1.0);
                }
            }
        }
        s += seg;
    }
}

// R290 small key prop: 25% chance per level, 1-3 tiny gold keys on the floor.
fn place_small_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let key_color = key_color_by_weather(g.weather_key());
    let key_mat = lambert!({ color: key_color });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let key = group();
        // Ring at head of key.
        let ring = mesh(pg("SmallKeys__bow", torus_geo!(0.014, 0.003, 4, 10)), &key_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().x = -0.015;
        key.add(&ring);
        // Shaft.
        let shaft = mesh(pg("SmallKeys__blade", cyl_geo!(0.003, 0.003, 0.028, 5)), &key_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().x = 0.012;
        key.add(&shaft);
        // Tooth (tiny bump on shaft end): bitting in the key's horizontal plane.
        let tooth = mesh(pg("SmallKeys__bitting", box_geo!(0.008, 0.006, 0.003)), &key_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.019, 0, 0.005);
        key.add(&tooth);
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        key.position().set(g.wx(x) + ox, 0.003, g.wz(z) + oz);
        key.rotation().y = rnd() * PI * 2.0;
        g.world.add(&key);
        i += 1;
    }
}

// R407 vintage camera: 10% chance per level.
fn place_vintage_cameras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_mat = lambert!({ color: 0x181410 });
    let lens_mat = lambert!({ color: 0x2a2418 });
    let glass_mat = lambert!({ color: 0x1a2028, transparent: true, opacity: 0.65 });
    let cam = group();
    // Body: small box.
    let body = mesh(pg("VintageCameras__body", box_geo!(0.10, 0.06, 0.05)), &body_mat);
    body.position().y = 0.03;
    cam.add(&body);
    // Lens: cylinder protruding front.
    let lens = mesh(pg("VintageCameras__lens", cyl_geo!(0.020, 0.020, 0.03, 10)), &lens_mat);
    lens.rotation().x = PI / 2.0;
    lens.position().set(0, 0.03, 0.035);
    cam.add(&lens);
    // Glass front on lens.
    let glass = mesh(circle_geo!(0.018, 10), &glass_mat);
    glass.position().set(0, 0.03, 0.051);
    cam.add(&glass);
    // Viewfinder: small dark rectangle bump on top.
    let vf = mesh(pg("VintageCameras__finder", box_geo!(0.02, 0.015, 0.015)), &body_mat);
    vf.position().set(-0.03, 0.068, 0);
    cam.add(&vf);
    cam.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cam.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cam);
    // R408 25% chance: small film canister cylinder near camera.
    if rnd() < 0.25 {
        let can_colors = [0x8a2020, 0x282838, 0x2a2820, 0x505050];
        let can_body = lambert!({ color: pick(&can_colors) });
        let can_cap = lambert!({ color: 0x282020 });
        let canister = group();
        let can_b = mesh(pg("VintageCameras__can", cyl_geo!(0.018, 0.018, 0.06, 8)), &can_body);
        can_b.position().y = 0.030;
        canister.add(&can_b);
        let c_cap = mesh(pg("VintageCameras__cap", cyl_geo!(0.019, 0.019, 0.008, 8)), &can_cap);
        c_cap.position().y = 0.064;
        canister.add(&c_cap);
        let c_ang = rnd() * PI * 2.0;
        let c_rad = 0.08 + rnd() * 0.12;
        let cp = cam.pos();
        canister.position().set(cp.x + c_ang.cos() * c_rad, 0, cp.z + c_ang.sin() * c_rad);
        canister.rotation().y = rnd() * PI * 2.0;
        g.world.add(&canister);
    }
}

// R411 small backpack: 12% chance per level.
fn place_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x203040, 0x2a1810, 0x4a3018, 0x484028, 0x381818];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let strap_mat = lambert!({ color: 0x181410 });
    let bp = group();
    // Body: upright box.
    let body = mesh(pg("Backpacks__body", box_geo!(0.28, 0.36, 0.16)), &body_mat);
    body.position().y = 0.18;
    bp.add(&body);
    // 2 vertical shoulder straps on back.
    for sx_off in [-0.08, 0.08] {
        let strap = mesh(pg("Backpacks__strap", plane_geo!(0.04, 0.35)), &strap_mat);
        strap.rotation().y = PI; // face out of the back (it faced into the body)
        strap.position().set(sx_off, 0.18, -0.081);
        bp.add(&strap);
    }
    // Top handle: half-torus.
    let handle = mesh(pg("Backpacks__handle", torus_geo!(0.04, 0.006, 4, 8, PI)), &strap_mat);
    handle.rotation().z = 0.0; // loop over the top, ends on it (was a sideways C)
    handle.position().y = 0.36;
    bp.add(&handle);
    // Small front pocket flap.
    let pocket = mesh(pg("Backpacks__pocket", plane_geo!(0.16, 0.10)), &strap_mat);
    pocket.position().set(0, 0.08, 0.081);
    bp.add(&pocket);
    bp.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bp.rotation().y = rnd() * PI * 2.0;
    // Slight tilt for organic slump.
    bp.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&bp);
}

// R427 small gift box: 10% chance per level.
fn place_small_gift_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let box_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0];
    let ribbon_colors = [0xf0e0d0, 0xd0d0d0, 0xd8b820, 0x2050d8];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors) });
    let gift = group();
    let bs = 0.10 + rnd() * 0.04; // 10-14 cm cube
                                  // Body: cube.
    let body = mesh(pg("SmallGiftBoxes__body", box_geo!(bs, bs, bs)), &box_mat);
    body.position().y = bs / 2.0;
    gift.add(&body);
    // 2 ribbon strips crossing top + wrapping sides.
    let r_wide = 0.010;
    let r_top1 = mesh(pg("SmallGiftBoxes__ribbonX", box_geo!(bs + 0.002, 0.003, r_wide)), &ribbon_mat);
    r_top1.position().y = bs + 0.001;
    gift.add(&r_top1);
    let r_top2 = mesh(pg("SmallGiftBoxes__ribbonZ", box_geo!(r_wide, 0.003, bs + 0.002)), &ribbon_mat);
    r_top2.position().y = bs + 0.001;
    gift.add(&r_top2);
    // Side ribbon wraps (front + back + left + right).
    for (rw, rh, rp, px, py, pz) in [
        (bs + 0.002, bs, r_wide, 0.0, 0.0, bs / 2.0 + 0.001),
        (bs + 0.002, bs, r_wide, 0.0, 0.0, -bs / 2.0 - 0.001),
        (r_wide, bs, bs + 0.002, bs / 2.0 + 0.001, 0.0, 0.0),
        (r_wide, bs, bs + 0.002, -bs / 2.0 - 0.001, 0.0, 0.0),
    ] {
        let r_side = mesh(box_geo!(rw, rh, if rw == r_wide { rp } else { rw }), &ribbon_mat);
        // Scale the box to a 1 cm x 2 mm strip down the middle of its face,
        // continuing the top ribbon, and add it (it was never added).
        let gd = if rw == r_wide { rp } else { rw };
        r_side.scale().set(if px == 0.0 { r_wide / rw } else { 0.002 / rw }, 1, if px == 0.0 { 0.002 / gd } else { r_wide / gd });
        r_side.position().set(px, bs / 2.0 + py, pz);
        gift.add(&r_side);
    }
    // Bow: 2 small triangles at ribbon crossing.
    for bow_side in [-1.0f64, 1.0] {
        let bow_geo = Geo::new();
        let bow_verts: Vec<f32> = [0.0, bs + 0.005, 0.0, bow_side * 0.025, bs + 0.020, 0.0, bow_side * 0.020, bs + 0.005, bow_side * 0.010]
            .iter()
            .map(|&v| v as f32)
            .collect();
        bow_geo.set_position(bow_verts);
        bow_geo.set_index(vec![0, 1, 2]);
        bow_geo.compute_vertex_normals();
        let bow = mesh(bow_geo, &ribbon_mat);
        gift.add(&bow);
    }
    gift.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    gift.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gift);
}

// R427 small party horn: batch `t12`'s (`t12.rs`)
mod t12;
use t12::place_small_party_horns;
// the hanging brass lantern: batch `t21`'s (`t21.rs`)
mod t21;
use t21::place_hanging_brass_lanterns;

// R478 small vintage brass scales: 5% chance per level, gentle sway.
fn place_small_scales(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let scales = group();
    // Base disc.
    let base = mesh(pg("SmallScales__base", cyl_geo!(0.045, 0.055, 0.010, 10)), &brass_mat);
    base.position().y = 0.005;
    scales.add(&base);
    // Central pillar.
    let pillar = mesh(pg("SmallScales__pillar", cyl_geo!(0.005, 0.005, 0.10, 6)), &brass_mat);
    pillar.position().y = 0.060;
    scales.add(&pillar);
    // Cross-beam group (rotates as a whole).
    let beam = group();
    let beam_bar = mesh(pg("SmallScales__beam", cyl_geo!(0.003, 0.003, 0.14, 4)), &brass_mat);
    beam_bar.rotation().z = PI / 2.0;
    beam.add(&beam_bar);
    // 2 chains hanging + 2 pans.
    for s in [-1.0f64, 1.0] {
        for ci in 0..3 {
            let chain = mesh(pg("SmallScales__chain", cyl_geo!(0.001, 0.001, 0.030, 3)), &brass_mat);
            // One straight run from inside the beam end down into the pan.
            chain.position().set(s * 0.065, -0.013 - ci as f64 * 0.030, 0);
            beam.add(&chain);
        }
        let pan = mesh(pg("SmallScales__pan", cyl_geo!(0.024, 0.026, 0.004, 10)), &brass_mat);
        // Hung 1.1 cm higher: the chains reach it and it clears the base.
        pan.position().set(s * 0.065, -0.087, 0);
        beam.add(&pan);
    }
    beam.position().y = 0.112;
    scales.add(&beam);
    // Very slight balance sway.
    let sway_phase = rnd() * PI * 2.0;
    let sway_period = 6.0 + rnd() * 4.0;
    let beam2 = beam.clone();
    g.animate(move |_g, _dt, t| {
        beam2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.05;
    });
    scales.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scales.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scales);
}

// R478 small abacus: batch `t20`'s (`t20.rs`)
mod t20;
use t20::place_small_abacuses;

// R477 small telegraph key: 8% chance per level.
fn place_small_telegraph_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x603820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let knob_mat = lambert!({ color: 0x181008 });
    let key = group();
    // Base wood plate.
    let base = mesh(pg("SmallTelegraphKeys__base", box_geo!(0.08, 0.010, 0.06)), &wood_mat);
    base.position().y = 0.005;
    key.add(&base);
    // Brass pivot posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallTelegraphKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(0, 0.020, s * 0.010);
        key.add(&post);
    }
    // Key lever arm.
    let lever = mesh(pg("SmallTelegraphKeys__lever", box_geo!(0.06, 0.004, 0.008)), &brass_mat);
    lever.position().set(0.008, 0.024, 0);
    key.add(&lever);
    // Bakelite knob on top.
    let knob = mesh(pg("SmallTelegraphKeys__knob", sphere_geo!(0.010, 8, 6)), &knob_mat);
    knob.scale().y = 0.5;
    knob.position().set(0.028, 0.028, 0);
    key.add(&knob);
    // 2 brass terminals on base.
    for s in [-1.0f64, 1.0] {
        let term = mesh(pg("SmallTelegraphKeys__terminal", cyl_geo!(0.004, 0.004, 0.008, 5)), &brass_mat);
        term.position().set(-0.030, 0.014, s * 0.020);
        key.add(&term);
    }
    // Wire trailing off: from between the terminals, over the base's end and
    // down onto the floor.
    let wire = mesh(pg("SmallTelegraphKeys__wire", cyl_geo!(0.001, 0.001, 0.10, 3)), &knob_mat);
    let wire_a = v3(-0.030, 0.012, 0);
    let wire_b = v3(-0.126, 0.001, 0.012);
    let dir = vsub(wire_b, wire_a).normalized();
    wire.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
    wire.scale().y = wire_a.distance_to(&wire_b) / 0.10;
    {
        let mut p = wire.position();
        p.copy(&wire_a).add(&wire_b).multiply_scalar(0.5);
    }
    key.add(&wire);
    key.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    key.rotation().y = rnd() * PI * 2.0;
    g.world.add(&key);
}

// R477 a portable sewing machine on the floor (Blender batch `r16`): a
// black-japanned machine of the Singer 221 class (378 x 184 x 248 mm), its
// gold decals and its maker's name (HALDANE, no real maker) rubbed thin, its
// bright parts tarnished, a spool of thread still on its pin; its handwheel
// turning slowly by itself. 4% chance per level.
fn place_small_sewing_machines(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.sewing_positions.clear(); // R481 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let sewing = group();
    let japan = phong!({ map: &sew_japan_tex(), color: 0xffffff, specular: 0x3a3a3a, shininess: 38 });
    let bright = phong!({ map: &sew_bright_tex(), color: 0xffffff, specular: 0x8a8a88, shininess: 64 });
    for (name, mat) in [("Real__sewBed", &japan), ("Real__sewArm", &japan), ("Real__sewSteel", &bright)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            sewing.add(&mesh(geo, mat));
        }
    }
    // the handwheel and its knob on their axle (x), so it can turn
    let (ax, ay, az) = SEW_AXLE;
    let wheel = group();
    wheel.position().set(ax, ay, az);
    for (name, mat) in [("Real__sewWheel", &japan), ("Real__sewWheelKnob", &bright)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(-ax, -ay, -az);
            wheel.add(&m);
        }
    }
    sewing.add(&wheel);
    // the gold decals, laid on its faces: the maker's name on the arm,
    // scrolls on the bed's front corners, the badge on the pillar
    let decal = |tex: &Tex| lambert!({ map: tex, color: 0xffffff, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let name = mesh(plane_geo!(0.140, 0.034), decal(&sew_name_tex()));
    name.position().set(-0.030, 0.2078, SEW_ARM_FRONT + 0.0004);
    sewing.add(&name);
    for side in [-1.0, 1.0] {
        let scroll = mesh(plane_geo!(0.070, 0.030), decal(&sew_scroll_tex()));
        scroll.rotation().x = -PI / 2.0;
        scroll.scale().x = side;
        scroll.position().set(0.022 + side * 0.058, SEW_TOP + 0.0004, 0.031);
        sewing.add(&scroll);
    }
    let badge = mesh(plane_geo!(0.030, 0.030), decal(&sew_badge_tex()));
    badge.position().set(0.101, 0.142, SEW_ARM_FRONT + 0.0004);
    sewing.add(&badge);
    // a spool of thread on the pin (its colour from where it stands)
    let (px, pz) = (g.wx(x), g.wz(z));
    let thread = [0xd8d2c0, 0x2a2a2c, 0x5a3a2a, 0x2c3448, 0x8a8478][((px * 3.7 + pz * 5.3).abs() as usize) % 5];
    if let Some(geo) = prop_part_as_modelled("Real__sewSpool") {
        sewing.add(&mesh(geo, lambert!({ map: &sew_thread_tex(), color: thread })));
    }
    // the wheel turns slowly by itself (the old rate's draw)
    let rot_rate = 0.3 + rnd() * 0.3;
    let wheel2 = wheel.clone();
    g.animate(move |_g, _dt, t| {
        wheel2.rotation().x = -t * rot_rate;
    });
    sewing.position().set(px + (rnd() - 0.5) * 1.8, 0, pz + (rnd() - 0.5) * 1.8);
    sewing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sewing);
    // R481 track for whirr audio.
    let sp = sewing.pos();
    g.pp.sewing_positions.push(v3(sp.x, 0.10, sp.z));
}

/// The machine as modelled (r16.py): the handwheel's axle, the bed's top,
/// the arm and pillar's front face
const SEW_AXLE: (f64, f64, f64) = (0.1405, 0.1960, -0.0310);
const SEW_TOP: f64 = 0.038;
const SEW_ARM_FRONT: f64 = -0.002;

fn sew_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}
fn sew_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (sew_hash(seed, xi, yi), sew_hash(seed, xi + 1, yi));
    let (c, d) = (sew_hash(seed, xi, yi + 1), sew_hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Black japanning, years on: its gloss dulled by dust in blotches, fine
/// scratches, chips to the grey metal at edges and knocks (one texture for
/// every face, whatever way it is mapped: nothing in it has a direction)
fn sew_japan_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 256, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64, j as f64);
                        let d = sew_noise(41, x / 24.0, y / 24.0) * 0.6 + sew_noise(42, x / 6.0, y / 6.0) * 0.4;
                        // the dust of years on the black, heavier in blotches
                        let g0 = 32.0 + 36.0 * (d - 0.4).max(0.0);
                        out[i * 4..i * 4 + 4].copy_from_slice(&[(g0 + 3.0) as u8, (g0 + 2.0) as u8, g0 as u8, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
                ctx.set_line_width(0.6);
                for _ in 0..40 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let a = rnd() * PI;
                    let l = 3.0 + rnd() * 16.0;
                    ctx.set_stroke_style("rgba(74,72,68,0.35)");
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
                for _ in 0..10 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 0.6 + rnd() * 1.4;
                    ctx.set_fill_style("rgba(96,94,90,0.8)");
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.5 + rnd() * 0.5), rnd() * PI, 0, PI * 2.0);
                    ctx.fill();
                }
            })
        })
        .clone()
    })
}

/// Nickel plate tarnished: dull, brown-grey in blotches, rust pricks
fn sew_bright_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64, j as f64);
                        let t = sew_noise(51, x / 14.0, y / 14.0);
                        let p = (sew_noise(52, x / 1.8, y / 1.8) - 0.82).max(0.0) * 5.0;
                        let g0 = 150.0 - 40.0 * t;
                        let c = [g0 + 4.0 * t - 60.0 * p, g0 - 2.0 * t - 80.0 * p, g0 - 10.0 * t - 96.0 * p];
                        out[i * 4..i * 4 + 4].copy_from_slice(&[c[0].max(0.0) as u8, c[1].max(0.0) as u8, c[2].max(0.0) as u8, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

/// Gold leaf rubbed thin: where a decal's gold is worn through to the black
/// (alpha), crazed, more worn where hands and cloth went
fn sew_rub(ctx: &mut Canvas, w: f64, h: f64, seed: u32, wear: f64) {
    let n = w as usize;
    let mut img = ctx.get_image_data(0, 0, w, h);
    for j in 0..h as usize {
        for i in 0..n {
            let a = img.data[(j * n + i) * 4 + 3] as f64;
            if a == 0.0 {
                continue;
            }
            let (x, y) = (i as f64, j as f64);
            let worn = sew_noise(seed, x / 9.0, y / 9.0) * 0.7 + sew_noise(seed ^ 1, x / 2.0, y / 2.0) * 0.3;
            let craze = (1.0 - ((sew_noise(seed ^ 2, x / 3.0, y / 3.0) - 0.5).abs() * 18.0).min(1.0)) * 0.5;
            let keep = (1.0 - ((worn - (1.0 - wear)) * 3.0).max(0.0) - craze).clamp(0.0, 1.0);
            img.data[(j * n + i) * 4 + 3] = (a * keep) as u8;
        }
    }
    ctx.put_image_data(&img, 0, 0);
}

/// The maker's name in gold on the arm, between scrolls (no real maker)
fn sew_name_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(512, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let gold = "rgba(196,160,84,0.95)";
                ctx.set_fill_style(gold);
                ctx.set_stroke_style(gold);
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.set_font("italic bold 64px Baskerville");
                ctx.fill_text_max("Haldane", w / 2.0, h * 0.50, w * 0.56);
                // the scrolls either side: S-curves and leaves
                ctx.set_line_width(3.0);
                for side in [-1.0, 1.0] {
                    let x0 = w / 2.0 + side * w * 0.31;
                    ctx.begin_path();
                    ctx.move_to(x0, h * 0.5);
                    ctx.bezier_curve_to(x0 + side * 30.0, h * 0.15, x0 + side * 60.0, h * 0.85, x0 + side * 92.0, h * 0.45);
                    ctx.stroke();
                    for k in 0..4 {
                        let t = 0.2 + k as f64 * 0.2;
                        let (lx, ly) = (x0 + side * 92.0 * t, h * (0.5 + 0.18 * (t * 9.0).sin()));
                        ctx.begin_path();
                        ctx.ellipse(lx, ly, 9.0, 3.5, side * 0.7 + t, 0, PI * 2.0);
                        ctx.fill();
                    }
                    ctx.begin_path();
                    ctx.arc(x0 + side * 92.0, h * 0.45, 5.0, 0, PI * 2.0);
                    ctx.fill();
                }
                // the thin rule under it
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(w * 0.22, h * 0.86);
                ctx.line_to(w * 0.78, h * 0.86);
                ctx.stroke();
                sew_rub(ctx, w, h, 61, 0.45);
            })
        })
        .clone()
    })
}

/// A scroll on the bed's corner: a border of curls and leaves (drawn for the
/// right corner; the left is its mirror)
fn sew_scroll_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 112, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let gold = "rgba(196,160,84,0.9)";
                ctx.set_fill_style(gold);
                ctx.set_stroke_style(gold);
                ctx.set_line_width(3.0);
                ctx.begin_path();
                ctx.move_to(8.0, h * 0.7);
                for k in 0..5 {
                    let x = 8.0 + k as f64 * 46.0;
                    ctx.bezier_curve_to(x + 12.0, h * 0.2, x + 34.0, h * 0.2, x + 46.0, h * 0.7);
                }
                ctx.stroke();
                for k in 0..5 {
                    let x = 31.0 + k as f64 * 46.0;
                    ctx.begin_path();
                    ctx.arc(x, h * 0.55, 7.0, 0, PI * 1.6);
                    ctx.stroke();
                    ctx.begin_path();
                    ctx.ellipse(x + 10.0, h * 0.82, 8.0, 3.0, 0.5, 0, PI * 2.0);
                    ctx.fill();
                }
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(4.0, h * 0.93);
                ctx.line_to(w - 4.0, h * 0.93);
                ctx.stroke();
                sew_rub(ctx, w, h, 62, 0.6);
            })
        })
        .clone()
    })
}

/// The badge on the pillar: a round enamel plate, a gilt ring, a monogram
fn sew_badge_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let (cx, cy) = (w / 2.0, h / 2.0);
                ctx.set_fill_style("rgba(170,140,78,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, 60.0, 0, PI * 2.0);
                ctx.fill();
                ctx.set_fill_style("rgba(92,30,30,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, 50.0, 0, PI * 2.0);
                ctx.fill();
                ctx.set_fill_style("rgba(196,164,92,1)");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.set_font("bold 60px Baskerville");
                ctx.fill_text("H", cx, cy + 3.0);
                ctx.set_font("bold 11px Helvetica");
                ctx.fill_text("TRADE MARK", cx, cy + 38.0);
                sew_rub(ctx, w, h, 63, 0.3);
            })
        })
        .clone()
    })
}

/// Thread wound on a spool: fine turns round it, a little dust on the top
fn sew_thread_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 64, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let t = ((j as f64 * 1.7 + i as f64 * 0.05).sin() * 0.5 + 0.5) * 0.25 + 0.75;
                        let v = (255.0 * t) as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

// R476 small vintage suitcase: 8% chance per level.
fn place_small_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let leather_mat = lambert!({ color: pick(&leather_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let suitcase = group();
    // Body.
    let body = mesh(pg("SmallSuitcases__body", box_geo!(0.22, 0.10, 0.14)), &leather_mat);
    body.position().y = 0.05;
    suitcase.add(&body);
    // Middle divider line (open-close seam).
    let seam = mesh(pg("SmallSuitcases__seam", box_geo!(0.222, 0.003, 0.142)), &brass_mat);
    seam.position().y = 0.05;
    suitcase.add(&seam);
    // Handle (torus arc on top).
    let handle = mesh(pg("SmallSuitcases__handle", torus_geo!(0.024, 0.005, 4, 12, PI)), &leather_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().y = 0.104;
    suitcase.add(&handle);
    // 2 brass buckles on front.
    for s in [-1.0f64, 1.0] {
        let buckle = mesh(pg("SmallSuitcases__buckle", box_geo!(0.014, 0.014, 0.004)), &brass_mat);
        buckle.position().set(s * 0.05, 0.048, 0.072);
        suitcase.add(&buckle);
        // Buckle latch (small strip).
        let latch = mesh(pg("SmallSuitcases__latch", box_geo!(0.008, 0.006, 0.002)), &brass_mat);
        latch.position().set(s * 0.05, 0.048, 0.075);
        suitcase.add(&latch);
    }
    // Corner feet at bottom.
    for (cx, cz) in [(-1.0f64, -1.0f64), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let corner = mesh(pg("SmallSuitcases__foot", cyl_geo!(0.008, 0.008, 0.008, 5)), &brass_mat);
        corner.position().set(cx * 0.095, 0.004, cz * 0.055);
        suitcase.add(&corner);
    }
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}

// R475 small vintage phonograph: 4% chance per level.
fn place_small_phonographs(g: &mut Game, _dist: &[i32]) {
    g.pp.phonograph_positions.clear(); // R480 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x6a3820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let record_mat = lambert!({ color: 0x201a10 });
    let label_mat = lambert!({ color: 0xa02040 });
    let phono = group();
    // Wood base cube.
    let base = mesh(pg("SmallPhonographs__base", box_geo!(0.16, 0.06, 0.14)), &wood_mat);
    base.position().y = 0.03;
    phono.add(&base);
    // Turntable disc.
    let turntable = mesh(pg("SmallPhonographs__turntable", cyl_geo!(0.055, 0.055, 0.006, 12)), &record_mat);
    turntable.position().y = 0.063;
    phono.add(&turntable);
    // Record on turntable (slightly smaller).
    let record = mesh(pg("SmallPhonographs__record", cyl_geo!(0.052, 0.052, 0.002, 14)), &record_mat);
    record.position().y = 0.067;
    phono.add(&record);
    // Record label.
    let label = mesh(circle_geo!(0.015, 8), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0685;
    phono.add(&label);
    // Tone arm: from its pivot (collar end, -Y) on the cabinet's back-right
    // corner to the sound box (+Y end) on the record.
    let arm_pivot = v3(0.076, 0.064, -0.068);
    let arm_tip = v3(0.032, 0.0745, 0);
    let arm = mesh(pg("SmallPhonographs__arm", cyl_geo!(0.003, 0.003, 0.08, 4)), &brass_mat);
    arm.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &vsub(arm_tip, arm_pivot).normalized());
    arm.scale().y = arm_tip.distance_to(&arm_pivot) / 0.08;
    {
        let mut p = arm.position();
        p.copy(&arm_pivot).add(&arm_tip).multiply_scalar(0.5);
    }
    phono.add(&arm);
    let arm_head = mesh(pg("SmallPhonographs__head", box_geo!(0.010, 0.006, 0.008)), &brass_mat);
    arm_head.position().set(0.032, 0.070, 0);
    phono.add(&arm_head);
    // Large brass horn: throat (apex, +Y) at the arm pivot, the flared mouth up and back.
    let horn = mesh(pg("SmallPhonographs__horn", cone_geo!(0.08, 0.20, 12, 1, true)), &brass_mat);
    let horn_dir = v3(-0.30, 0.90, -0.30).normalized();
    horn.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &(-horn_dir));
    {
        let mut p = horn.position();
        p.copy(&arm_pivot).add_scaled_vector(&horn_dir, 0.10);
    }
    phono.add(&horn);
    // Turntable slow rotation animator.
    let rot_rate = 0.5 + rnd() * 0.5;
    {
        let (turntable, record, label) = (turntable.clone(), record.clone(), label.clone());
        g.animate(move |_g, _dt, t| {
            turntable.rotation().y = t * rot_rate;
            record.rotation().y = t * rot_rate;
            label.rotation().z = t * rot_rate;
        });
    }
    phono.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    phono.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phono);
    // R480 track for vinyl crackle audio.
    let pp = phono.pos();
    g.pp.phonograph_positions.push(v3(pp.x, 0.10, pp.z));
}

// R475 small rotary phone: 6% chance per level.
fn place_small_rotary_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x181008, 0x502818, 0xa02040];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let dial_mat = lambert!({ color: 0xc0c0c0 });
    let cord_mat = lambert!({ color: 0x201a10 });
    let phone = group();
    // Base body.
    let base = mesh(pg("SmallRotaryPhones__base", box_geo!(0.13, 0.05, 0.10)), &body_mat);
    base.position().y = 0.025;
    phone.add(&base);
    // Rotary dial (disc on top).
    let dial = mesh(pg("SmallRotaryPhones__dial", cyl_geo!(0.028, 0.028, 0.006, 14)), &dial_mat);
    dial.position().set(0, 0.053, 0.008);
    phone.add(&dial);
    // 10 dial finger holes.
    for i in 0..10 {
        let a = (i as f64 / 10.0) * PI * 2.0;
        let hole = mesh(pg("SmallRotaryPhones__hole", cyl_geo!(0.003, 0.003, 0.008, 5)), &body_mat);
        hole.position().set(a.cos() * 0.020, 0.055, 0.008 + a.sin() * 0.020);
        phone.add(&hole);
    }
    // 2 handset cradle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallRotaryPhones__post", cyl_geo!(0.008, 0.010, 0.014, 6)), &body_mat);
        post.position().set(s * 0.045, 0.056, -0.028);
        phone.add(&post);
    }
    // Handset sitting on cradle.
    let handset = mesh(pg("SmallRotaryPhones__handset", cyl_geo!(0.010, 0.010, 0.11, 8)), &body_mat);
    handset.rotation().z = PI / 2.0;
    handset.position().set(0, 0.070, -0.028);
    phone.add(&handset);
    // Ear/mouth cups.
    for s in [-1.0f64, 1.0] {
        let cup = mesh(pg("SmallRotaryPhones__cup", cyl_geo!(0.014, 0.012, 0.010, 8)), &body_mat);
        cup.rotation().z = PI / 2.0;
        cup.position().set(s * 0.048, 0.070, -0.028);
        phone.add(&cup);
    }
    // Curly cord (torus loop).
    let cord = mesh(pg("SmallRotaryPhones__cord", torus_geo!(0.020, 0.002, 4, 12, PI * 1.5)), &cord_mat);
    cord.rotation().x = PI / 2.0;
    cord.rotation().z = PI / 2.0;
    cord.position().set(0.070, 0.030, -0.028);
    phone.add(&cord);
    phone.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}

// R474 small vintage typewriter: 5% chance per level.
fn place_small_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_mat = lambert!({ color: 0x201a10 });
    let key_mat = lambert!({ color: 0xf0ece0 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let roll_mat = lambert!({ color: 0x2a2418 });
    let tw = group();
    // Base body.
    let base = mesh(pg("SmallTypewriters__base", box_geo!(0.16, 0.02, 0.14)), &body_mat);
    base.position().y = 0.010;
    tw.add(&base);
    // Angled key deck (top-front slope).
    let deck = mesh(pg("SmallTypewriters__deck", box_geo!(0.16, 0.010, 0.08)), &body_mat);
    deck.rotation().x = 0.35; // low at the typist (+z), rising toward the roll
    deck.position().set(0, 0.036, 0.03);
    tw.add(&deck);
    // 4x10 key grid, sitting on the deck's recessed floor (row 0 at the front).
    for ky in 0..4 {
        for kx in 0..10 {
            let key = mesh(pg("VintageTypewriters__key", cyl_geo!(0.005, 0.005, 0.004, 5)), &key_mat);
            let kkx = -0.070 + kx as f64 * 0.0155;
            let kkz = 0.0567 - ky as f64 * 0.0169;
            key.rotation().x = 0.35;
            key.position().set(kkx, 0.0305 + ky as f64 * 0.0062, kkz);
            tw.add(&key);
        }
    }
    // Back platen (paper roll cylinder), resting on the base.
    let roll = mesh(pg("SmallTypewriters__roll", cyl_geo!(0.020, 0.020, 0.13, 8)), &roll_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(0, 0.040, -0.045);
    tw.add(&roll);
    // Paper coming out top.
    let paper = mesh(plane_geo!(0.12, 0.07), &paper_mat);
    paper.position().set(0, 0.078, -0.045);
    paper.rotation().x = -0.15;
    tw.add(&paper);
    // Return lever on side: foot on the roll's left end, leaning up and out.
    let lever = mesh(pg("VintageTypewriters__lever", cyl_geo!(0.003, 0.003, 0.030, 4)), &roll_mat);
    lever.rotation().z = 0.5;
    lever.position().set(-0.073, 0.058, -0.045);
    tw.add(&lever);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
}

// R474 small vintage globe texture.
game::lazy_static_tex!(
    globe_texture,
    canvas_tex(256, 128, |ctx, w, h| {
        // Ocean blue.
        ctx.set_fill_style("#4080b0");
        ctx.fill_rect(0, 0, w, h);
        // Continent blobs (5 land shapes).
        ctx.set_fill_style("#508050");
        let conts: [(f64, f64, f64, f64); 5] = [
            (40.0, 60.0, 30.0, 22.0),
            (100.0, 50.0, 25.0, 30.0),
            (160.0, 40.0, 20.0, 18.0),
            (200.0, 70.0, 26.0, 24.0),
            (130.0, 90.0, 22.0, 16.0),
        ];
        for &(cx, cy, rx, ry) in conts.iter() {
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx, ry, rnd() * 0.5, 0, PI * 2.0);
            ctx.fill();
            // Extra bumps for irregular shape.
            for _ in 0..4 {
                let a = rnd() * PI * 2.0;
                let r = rx.min(ry) * 0.4;
                ctx.begin_path();
                ctx.arc(cx + a.cos() * r * 1.4, cy + a.sin() * r * 1.4, r * 0.8, 0, PI * 2.0);
                ctx.fill();
            }
        }
        // Equator + meridian faint lines.
        ctx.set_stroke_style("rgba(80,40,20,0.4)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(0, h / 2.0);
        ctx.line_to(w, h / 2.0);
        ctx.stroke();
        for i in 1..6 {
            let fi = i as f64;
            ctx.begin_path();
            ctx.move_to(fi * w / 6.0, 0);
            ctx.line_to(fi * w / 6.0, h);
            ctx.stroke();
        }
    })
);

// R474 small vintage globe: 5% chance per level, slow spin.
fn place_small_globes(g: &mut Game, _dist: &[i32]) {
    g.pp.globe_positions.clear(); // R482 reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x8a5030 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let globe_mat = lambert!({ map: globe_texture() });
    let globe = group();
    // Base disc.
    let base = mesh(pg("SmallGlobes__base", cyl_geo!(0.045, 0.055, 0.010, 10)), &wood_mat);
    base.position().y = 0.005;
    globe.add(&base);
    // Column.
    let col = mesh(pg("SmallGlobes__column", cyl_geo!(0.006, 0.008, 0.050, 6)), &wood_mat);
    col.position().y = 0.035;
    globe.add(&col);
    // Meridian ring (torus).
    let meridian = mesh(pg("SmallGlobes__meridian", torus_geo!(0.055, 0.003, 4, 20)), &brass_mat);
    meridian.rotation().z = 0.4;
    meridian.position().y = 0.115;
    globe.add(&meridian);
    // Sphere with map; its axis is tilted with the meridian so its poles meet
    // the meridian's pivot pins, and the spin turns it about that axis.
    let sphere = mesh(pg("SmallGlobes__sphere", sphere_geo!(0.050, 16, 12)), &globe_mat);
    let globe_axis = group();
    globe_axis.rotation().z = 0.4;
    globe_axis.position().y = 0.115;
    globe_axis.add(&sphere);
    globe.add(&globe_axis);
    // Slow rotation animator.
    let spin_rate = 0.15 + rnd() * 0.15;
    let sphere2 = sphere.clone();
    g.animate(move |_g, _dt, t| {
        sphere2.rotation().y = t * spin_rate;
    });
    globe.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    globe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&globe);
    // R482 track for wood creak audio.
    let gp = globe.pos();
    g.pp.globe_positions.push(v3(gp.x, 0.12, gp.z));
}

// R473 small brass telescope: 6% chance per level.
fn place_small_telescopes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let leather_mat = lambert!({ color: 0x6a3820 });
    let lens_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.75 });
    let scope = group();
    // 3 sections (largest to smallest, telescoped): (len, r, x).
    let sections: [(f64, f64, f64); 3] = [(0.10, 0.020, 0.05), (0.09, 0.015, 0.145), (0.08, 0.011, 0.230)];
    for &(len, r, sx) in sections.iter() {
        let body = mesh(pg("SmallTelescopes__tube", cyl_geo!(r, r, len, 10)), &brass_mat);
        body.rotation().z = PI / 2.0;
        body.position().set(sx, 0.020, 0);
        scope.add(&body);
    }
    // Leather grip on largest section.
    let grip = mesh(pg("SmallTelescopes__grip", cyl_geo!(0.021, 0.021, 0.020, 10)), &leather_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(0.028, 0.020, 0);
    scope.add(&grip);
    // Eyepiece at large end.
    let eye = mesh(circle_geo!(0.018, 8), &lens_mat);
    eye.rotation().y = -PI / 2.0;
    eye.position().set(-0.001, 0.020, 0);
    scope.add(&eye);
    // Lens at small end.
    let lens = mesh(circle_geo!(0.010, 8), &lens_mat);
    lens.rotation().y = PI / 2.0;
    lens.position().set(0.271, 0.020, 0);
    scope.add(&lens);
    scope.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scope);
}

// R473 small padlock + key: 10% chance per level.
fn place_small_padlock_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = lambert!({ color: 0x2a1810 });
    let grp = group();
    // === PADLOCK ===
    let lock = group();
    // Body: rounded box.
    let body = mesh(pg("SmallPadlockKeys__body", box_geo!(0.040, 0.048, 0.020)), &brass_mat);
    body.position().y = 0.028;
    lock.add(&body);
    // Keyhole dark circle on front.
    let keyhole = mesh(circle_geo!(0.005, 6), &dark_mat);
    keyhole.position().set(0, 0.024, 0.011);
    lock.add(&keyhole);
    // Shackle (U-shape arch): an inverted U standing on the two posts.
    let shackle = mesh(pg("SmallPadlockKeys__shackle", torus_geo!(0.014, 0.004, 5, 12, PI)), &brass_mat);
    shackle.position().y = 0.056;
    lock.add(&shackle);
    // 2 shackle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallPadlockKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(s * 0.014, 0.048, 0);
        lock.add(&post);
    }
    lock.position().set(-0.06, -0.004, 0); // body bottom (0.004) on the floor
    grp.add(&lock);
    // === KEY === (lying flat: shaft, bow and bit all on the floor)
    let key = group();
    // Shaft.
    let shaft = mesh(pg("SmallPadlockKeys__shaft", cyl_geo!(0.002, 0.002, 0.080, 4)), &brass_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.040, 0.002, 0);
    key.add(&shaft);
    // Bow (ring at end), flat in the shaft's plane.
    let bow = mesh(pg("SmallPadlockKeys__bow", torus_geo!(0.012, 0.003, 4, 10)), &brass_mat);
    bow.rotation().x = PI / 2.0;
    bow.position().set(-0.005, 0.003, 0);
    key.add(&bow);
    // 2 teeth on other end, lying flat off the shaft's side.
    for i in 0..2 {
        let tooth = mesh(pg("SmallPadlockKeys__tooth", box_geo!(0.006, 0.008, 0.002)), &brass_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.070 + i as f64 * 0.008, 0.002, 0.005);
        key.add(&tooth);
    }
    key.position().set(0.02, 0, 0.05);
    key.rotation().y = -PI / 6.0;
    grp.add(&key);
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R472 small parchment scroll: 8% chance per level.
fn place_parchment_scrolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let parchment_mat = lambert!({ color: 0xe8dcc0 });
    let shadow_mat = lambert!({ color: 0xc0b090 });
    let ribbon_colors = [0xa02040, 0x4030c0, 0x2a2418, 0x8a2040];
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors), side: DoubleSide });
    let scroll = group();
    let scroll_len = 0.22;
    let scroll_r = 0.020;
    // Main roll.
    let roll = mesh(pg("ParchmentScrolls__roll", cyl_geo!(scroll_r, scroll_r, scroll_len, 12)), &parchment_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&roll);
    // Inner shadow discs at ends (roll depth).
    for s in [-1.0f64, 1.0] {
        let shadow_disc = mesh(circle_geo!(scroll_r * 0.75, 8), &shadow_mat);
        shadow_disc.rotation().y = PI / 2.0;
        shadow_disc.position().set(if s > 0.0 { scroll_len - 0.001 } else { 0.001 }, scroll_r, 0);
        if s < 0.0 {
            shadow_disc.rotation().y = -PI / 2.0;
        }
        scroll.add(&shadow_disc);
    }
    // Ribbon wrapped around middle, snug on the roll (radius 0.021).
    let ribbon = mesh(cyl_geo!(scroll_r + 0.003, scroll_r + 0.003, 0.012, 12, 1, true), &ribbon_mat);
    ribbon.scale().set((scroll_r + 0.001) / (scroll_r + 0.003), 1, (scroll_r + 0.001) / (scroll_r + 0.003));
    ribbon.rotation().z = PI / 2.0;
    ribbon.position().set(scroll_len / 2.0, scroll_r, 0);
    scroll.add(&ribbon);
    // 2 ribbon tail planes hanging.
    for s in [-1.0f64, 1.0] {
        let tail = mesh(plane_geo!(0.008, 0.030), &ribbon_mat);
        tail.rotation().y = PI / 2.0;
        tail.position().set(scroll_len / 2.0, scroll_r + 0.008, s * 0.024);
        tail.rotation().z = s * 0.2;
        scroll.add(&tail);
    }
    scroll.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    scroll.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scroll);
}

// R472 compass face texture.
game::lazy_static_tex!(
    compass_face_texture,
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece0");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(2);
        ctx.stroke();
        // Cardinal N/E/S/W.
        ctx.set_fill_style("#181008");
        ctx.set_font("bold 16px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text("N", w / 2.0, 14);
        ctx.fill_text("E", w - 14.0, h / 2.0);
        ctx.fill_text("S", w / 2.0, h - 14.0);
        ctx.fill_text("W", 14, h / 2.0);
        // Cardinal marks (short lines).
        ctx.set_line_width(3);
        for i in 0..16 {
            let a = -PI / 2.0 + (i as f64 / 16.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(w / 2.0 + a.cos() * 40.0, h / 2.0 + a.sin() * 40.0);
            ctx.line_to(w / 2.0 + a.cos() * 48.0, h / 2.0 + a.sin() * 48.0);
            ctx.stroke();
        }
        // Needle (red-tip pointing north).
        ctx.set_fill_style("#d82040");
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0 - 30.0);
        ctx.line_to(w / 2.0 - 4.0, h / 2.0);
        ctx.line_to(w / 2.0 + 4.0, h / 2.0);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style("#404040");
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0 + 30.0);
        ctx.line_to(w / 2.0 - 4.0, h / 2.0);
        ctx.line_to(w / 2.0 + 4.0, h / 2.0);
        ctx.close_path();
        ctx.fill();
        // Center pin.
        ctx.set_fill_style("#181008");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 4, 0, PI * 2.0);
        ctx.fill();
    })
);

// R472 small vintage compass: 8% chance per level.
fn place_small_compasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: compass_face_texture(), side: DoubleSide });
    let compass = group();
    // Case (disc).
    let case_ = mesh(pg("SmallCompasses__case", cyl_geo!(0.036, 0.036, 0.012, 14)), &brass_mat);
    case_.position().y = 0.006;
    compass.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.030, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0125;
    compass.add(&face);
    // Hinge: barrel along Z (the lid's pivot axis) at the case's top edge.
    let hinge = mesh(pg("SmallCompasses__hinge", cyl_geo!(0.005, 0.005, 0.014, 6)), &brass_mat);
    hinge.rotation().x = PI / 2.0;
    hinge.position().set(0.039, 0.011, 0);
    compass.add(&hinge);
    // Cover leaning open: the closed lid swung 120 deg about the hinge.
    let cover = mesh(pg("SmallCompasses__cover", cyl_geo!(0.036, 0.036, 0.006, 14)), &brass_mat);
    cover.position().set(0.039 + 0.0230, 0.011 + 0.0318, 0);
    cover.rotation().z = -2.0 * PI / 3.0;
    compass.add(&cover);
    compass.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    compass.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compass);
}

// R481 small pocket watch: 8% chance per level.
fn place_small_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.pocket_watch_positions.clear(); // R481 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case (disc).
    let case_ = mesh(pg("SmallPocketWatches__case", cyl_geo!(0.028, 0.028, 0.010, 14)), &brass_mat);
    case_.position().y = 0.005;
    watch.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.024, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0105;
    watch.add(&face);
    // Crown on side.
    let crown = mesh(pg("SmallPocketWatches__crown", cyl_geo!(0.004, 0.004, 0.006, 6)), &brass_mat);
    crown.rotation().z = PI / 2.0;
    crown.position().set(0.031, 0.005, 0);
    watch.add(&crown);
    // Chain of tiny torus links.
    for i in 0..6 {
        let fi = i as f64;
        let link = mesh(pg("SmallPocketWatches__link", torus_geo!(0.005, 0.001, 4, 8)), &brass_mat);
        link.position().set(0.036 + fi * 0.008, 0.005 + (fi * 0.4).sin() * 0.003, 0);
        link.rotation().x = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
        watch.add(&link);
    }
    watch.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    watch.rotation().y = rnd() * PI * 2.0;
    g.world.add(&watch);
    // R481 track for tick audio.
    let wp = watch.pos();
    g.pp.pocket_watch_positions.push(v3(wp.x, 0.01, wp.z));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
