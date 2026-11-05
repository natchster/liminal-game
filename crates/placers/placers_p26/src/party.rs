//! Party-remnant and children's-playtime props (R422-R430).
use crate::{buf_geo, open_tiles, pick, set_rot_order};
use game::prelude::*;

// the skipping rope: batch `t16`'s (`party/t16.rs`)
mod t16;
pub use t16::place_skipping_ropes;

/// R430 small cupcake wrapper: 10% chance per level. Empty pleated
/// paper wrapper ("who ate the cupcake?").
pub fn place_small_cupcake_wrappers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wrap_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let wrap_mat = lambert!({ color: pick(&wrap_colors), side: DoubleSide });
    let wrap = group();
    // Pleated body: 12-sided cylinder gives visible pleats.
    let body = mesh(cyl_geo!(0.030, 0.022, 0.028, 12, 1, true), &wrap_mat);
    body.position().y = 0.014;
    wrap.add(&body);
    // Bottom.
    let bottom = mesh(circle_geo!(0.022, 12), &wrap_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    wrap.add(&bottom);
    // 40% flattened / crushed variant.
    if rnd() < 0.40 {
        wrap.scale().y = 0.35;
    }
    wrap.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    wrap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wrap);
}

/// R429 small paper napkin: 12% chance per level. A two-ply luncheon
/// napkin left from a party (Blender batch r15: 33 cm, folded in four): with
/// three of the old crumples dropped half open, with four scrunched into a
/// wad; its embossed border and printed spots, a smear of cake, dust. The
/// old draws are kept call for call.
pub fn place_small_paper_napkins(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: white, cream, pink, blue, yellow or green napkins
    let nap_colors = [0xf0ece4, 0xe8d8c8, 0xf0c0d0, 0xd0d8f0, 0xf0d8a0, 0xd8f0d0];
    let col = pick(&nap_colors);
    let look = nap_colors.iter().position(|&c| c == col).unwrap_or(0);
    let napkin = group();
    let n_crumples = 3 + (rnd() * 2.0).floor() as i32;
    for _ in 0..n_crumples {
        // the old crumple's size, heights, corners, turn and place: kept as draws
        for _ in 0..8 {
            let _ = rnd();
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    napkin.position().set(px, 0, pz);
    napkin.rotation().y = rnd() * PI * 2.0;
    let salt = ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32;
    let name = if n_crumples == 3 { "Real__napkinCrumpled" } else { "Real__napkinWad" };
    if let Some(geo) = prop_part_as_modelled(name) {
        napkin.add(&mesh(geo, &lambert!({ map: &r15_napkin::napkin_tex(look, salt), side: DoubleSide })));
    }
    g.world.add(&napkin);
}

// R428 small paper plate: batch `t13`'s (`party/t13.rs`)
mod t13;
pub use t13::place_small_paper_plates;

/// R428 small birthday candle: 8% chance per level. Wax cylinder +
/// wick + optional tiny flame with faint point light. Party-remnant.
pub fn place_small_birthday_candles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wax_colors = [0xf0e8d8, 0xf0c0d0, 0xd8f0d0, 0xd0d8f0, 0xf0d8a0, 0xe0b0e0];
    let wax_mat = lambert!({ color: pick(&wax_colors) });
    let wick_mat = lambert!({ color: 0x2a2418 });
    let candle = group();
    // Body: thin tall cylinder.
    let body = mesh(pg("SmallBirthdayCandles__body", cyl_geo!(0.008, 0.008, 0.06, 8)), &wax_mat);
    body.position().y = 0.03;
    candle.add(&body);
    // Wick: tiny dark cylinder on top.
    let wick = mesh(pg("SmallBirthdayCandles__wick", cyl_geo!(0.001, 0.001, 0.008, 4)), &wick_mat);
    wick.position().y = 0.064;
    candle.add(&wick);
    // 40% chance lit: tiny teardrop flame + faint point light.
    let is_lit = rnd() < 0.40;
    if is_lit {
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
        let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
        flame.position().y = 0.074;
        candle.add(&flame);
        let flame_light = point_light!(0xffb060, 0.35, 0.9, 2);
        flame_light.position().y = 0.075;
        candle.add(&flame_light);
        // R483 warm halo sphere for readable bloom.
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.020, 6, 5), &halo_mat);
        halo.position().y = 0.076;
        candle.add(&halo);
        // R429 subtle flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 6.0 + rnd() * 4.0;
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3 + (p * 3.1).sin() * 0.2;
            flame_light.set_intensity(0.35 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
            halo.scale().set_scalar(1.0 + wobble * 0.15);
        });
    }
    // Occasional tipped variant.
    if !is_lit && rnd() < 0.20 {
        candle.rotation().z = PI / 2.0;
        candle.position().y = 0.008;
    }
    candle.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    candle.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    candle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&candle);
}

/// R426 confetti scatter: 10% chance per level. 10-15 tiny colored
/// planes scattered on floor in ~0.35 m radius circle.
pub fn place_confetti_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let conf_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0, 0xf0a0a0];
    let cx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let cz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let count = 10 + (rnd() * 6.0).floor() as i32;
    for _ in 0..count {
        let c_col = pick(&conf_colors);
        let conf_mat = basic_mat!({ color: c_col, transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
        let conf = mesh(plane_geo!(0.008, 0.014), &conf_mat);
        conf.rotation().x = -PI / 2.0;
        conf.rotation().z = rnd() * PI * 2.0;
        let c_ang = rnd() * PI * 2.0;
        let c_rad = rnd() * 0.35;
        conf.position().set(cx + c_ang.cos() * c_rad, 0.003, cz + c_ang.sin() * c_rad);
        g.world.add(&conf);
    }
}

/// R426 birthday party hat: batch `t12`'s (`t12.rs`)
#[path = "t12.rs"]
mod t12;
pub use t12::place_birthday_hats;

/// R425 curled gift ribbon: 10% chance per level. Curling ribbon cut from
/// round a present (Blender batch r14b: 3/16 in polypropylene, crimped and
/// glossy): the knot with four tails rolled in ringlets, or a single cut
/// length curled at its ends; faded, trodden flat in places.
pub fn place_small_ribbons(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: the ribbon's colour, faded
    let ribbon_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xf0a0a0];
    let c = pick(&ribbon_colors);
    let k = ribbon_colors.iter().position(|&r| r == c).unwrap_or(0);
    let ribbon = group();
    ribbon.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ribbon.rotation().y = rnd() * PI * 2.0;
    let p = ribbon.pos();
    let seed = r14b_kit::seed_of(&[p.x, p.z]);
    let faded = [0xa85464, 0x56689a, 0xb8a052, 0x5a8458, 0x865684, 0xcaa29e][k];
    let mat = phong!({ color: faded, specular: 0x585858, shininess: 52, side: DoubleSide });
    mat.m().map = Some(r14b_kit::faded_tex(seed));
    let name = if seed % 10 < 7 { "Real__ribbonKnot" } else { "Real__ribbonLength" };
    if let Some(geo) = prop_part_as_modelled(name) {
        ribbon.add(&mesh(geo, &mat));
    }
    g.world.add(&ribbon);
}

/// R423 small balloon: 10% chance per level. Colored sphere floating
/// low with string extending down to floor. Children's party remnant.
pub fn place_small_balloons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let balloon_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0];
    let balloon_mat = lambert!({ color: pick(&balloon_colors) });
    let string_mat = lambert!({ color: 0xd8d8d8 });
    let balloon = group();
    // R425 25% chance: fully deflated variant — flat rubber puddle.
    let is_deflated = rnd() < 0.25;
    let body = if is_deflated {
        mesh(pg("SmallBalloons__skin", circle_geo!(0.08 + rnd() * 0.04, 10)), &balloon_mat)
    } else {
        mesh(pg("SmallBalloons__body", sphere_geo!(0.10, 12, 10)), &balloon_mat)
    };
    if is_deflated {
        body.rotation().x = -PI / 2.0;
        body.rotation().z = rnd() * PI * 2.0;
        body.scale().set(1.3, 0.85, 1);
        body.position().y = 0.003;
    } else {
        body.scale().set(1, 1.15, 1);
        body.position().y = 0.5 + rnd() * 0.7;
    }
    let float_y = body.pos().y;
    balloon.add(&body);
    if !is_deflated {
        // Small tie-off pinch at bottom.
        let tie = mesh(pg("SmallBalloons__knot", cone_geo!(0.015, 0.03, 5)), &balloon_mat);
        tie.rotation().x = PI;
        tie.position().y = float_y - 0.11;
        balloon.add(&tie);
        // String from tie down to floor.
        let string_len = float_y - 0.14;
        let string = mesh(pg("SmallBalloons__string", cyl_geo!(0.002, 0.002, string_len, 4)), &string_mat);
        string.position().y = float_y - 0.14 - string_len / 2.0;
        balloon.add(&string);
    }
    balloon.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    g.world.add(&balloon);
    // R424 30% chance: slow sway animator (only on inflated variant).
    if !is_deflated && rnd() < 0.30 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let bp = body.pos();
        let (base_x, base_z) = (bp.x, bp.z);
        let b = body.clone();
        g.animate(move |_g, _dt, t| {
            let wobble = (t * (PI * 2.0 / sway_period) + sway_phase).sin();
            {
                let mut p = b.position();
                p.x = base_x + wobble * 0.020;
                p.z = base_z + (t * (PI * 2.0 / sway_period) + sway_phase).cos() * 0.015;
            }
            b.rotation().z = wobble * 0.05;
        });
    }
    // R424 25% chance: 1-2 sibling balloons at different heights.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _ in 0..sib_count {
            let sib_color = pick(&balloon_colors);
            let sib_mat = lambert!({ color: sib_color });
            let sib_balloon = group();
            let sib_body = mesh(pg("SmallBalloons__body", sphere_geo!(0.10, 12, 10)), &sib_mat);
            sib_body.scale().set(1, 1.15, 1);
            let sib_float_y = 0.5 + rnd() * 0.7;
            sib_body.position().y = sib_float_y;
            sib_balloon.add(&sib_body);
            let sib_tie = mesh(pg("SmallBalloons__knot", cone_geo!(0.015, 0.03, 5)), &sib_mat);
            sib_tie.rotation().x = PI;
            sib_tie.position().y = sib_float_y - 0.11;
            sib_balloon.add(&sib_tie);
            let sib_string_len = sib_float_y - 0.14;
            let sib_string = mesh(pg("SmallBalloons__string", cyl_geo!(0.002, 0.002, sib_string_len, 4)), &string_mat);
            sib_string.position().y = sib_float_y - 0.14 - sib_string_len / 2.0;
            sib_balloon.add(&sib_string);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.20 + rnd() * 0.20;
            let bp = balloon.pos();
            sib_balloon.position().set(bp.x + s_ang.cos() * s_rad, 0, bp.z + s_ang.sin() * s_rad);
            g.world.add(&sib_balloon);
        }
    }
}

/// R423 small tambourine: 8% chance per level. Torus rim + membrane
/// surface + tiny jingle dots around rim.
pub fn place_small_tambourines(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rim_mat = lambert!({ color: 0x6a4820 });
    let mem_mat = lambert!({ color: 0xd8ccb0 });
    let jingle_mat = lambert!({ color: 0xa8a840 });
    let tamb = group();
    // Rim: torus laid flat.
    let rim = mesh(pg("SmallTambourines__rim", torus_geo!(0.09, 0.010, 4, 12)), &rim_mat);
    rim.rotation().x = -PI / 2.0;
    rim.position().y = 0.010;
    tamb.add(&rim);
    // Membrane surface: cream disc inside rim.
    let mem = mesh(circle_geo!(0.086, 12), &mem_mat);
    mem.rotation().x = -PI / 2.0;
    mem.position().y = 0.011;
    tamb.add(&mem);
    // 6 tiny brass jingle dots around rim.
    for j in 0..6 {
        let j_ang = (j as f64 / 6.0) * PI * 2.0;
        let jingle = mesh(pg("SmallTambourines__jingle", cyl_geo!(0.008, 0.008, 0.004, 6)), &jingle_mat);
        jingle.position().set(j_ang.cos() * 0.09, 0.014, j_ang.sin() * 0.09);
        tamb.add(&jingle);
    }
    tamb.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tamb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tamb);
}

// R422 children's book texture — colorful cartoon cover simplification
// (bright bg + big geometric shape center).
game::lazy_static_tex!(
    child_book_texture,
    canvas_tex_opt(96, 128, false, |ctx, w, h| {
        let bg_colors = ["#d8b0e0", "#a0d8f0", "#f0d0a0", "#c0e8a0"];
        ctx.set_fill_style(pick(&bg_colors));
        ctx.fill_rect(0, 0, w, h);
        // Large central colorful shape (circle, star, or triangle).
        let shape_colors = ["#d82020", "#2050d8", "#d8b820", "#30a030", "#8020d8"];
        ctx.set_fill_style(pick(&shape_colors));
        let shape = (rnd() * 3.0).floor() as i32;
        let (cx, cy) = (w / 2.0, h * 0.4);
        if shape == 0 {
            // Circle.
            ctx.begin_path();
            ctx.arc(cx, cy, 30, 0, PI * 2.0);
            ctx.fill();
        } else if shape == 1 {
            // Triangle.
            ctx.begin_path();
            ctx.move_to(cx, cy - 30.0);
            ctx.line_to(cx - 30.0, cy + 25.0);
            ctx.line_to(cx + 30.0, cy + 25.0);
            ctx.close_path();
            ctx.fill();
        } else {
            // Star (rough 5-point via 10 vertices).
            ctx.begin_path();
            for i in 0..10 {
                let a = -PI / 2.0 + (i as f64 / 10.0) * PI * 2.0;
                let r = if i % 2 == 0 { 30.0 } else { 14.0 };
                let px = cx + a.cos() * r;
                let py = cy + a.sin() * r;
                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
            }
            ctx.close_path();
            ctx.fill();
        }
        // Fake title text: 2 dark horizontal bars near bottom.
        ctx.set_fill_style("#3a2c1a");
        ctx.fill_rect(15, h - 30.0, w - 30.0, 4);
        ctx.fill_rect(20, h - 20.0, w - 40.0, 4);
    })
);

/// Children's book (R422): 10% chance per level.
pub fn place_childrens_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cover_mat = lambert!({ map: child_book_texture(), side: DoubleSide });
    let pages_mat = lambert!({ color: 0xd8ccb0 });
    let cbook = group();
    // Slightly smaller than R380 adult book (10 × 14 cm).
    let cover = mesh(box_geo!(0.10, 0.005, 0.14), &cover_mat);
    cover.position().y = 0.020;
    cbook.add(&cover);
    let pages = mesh(pg("ChildrensBooks__pages", box_geo!(0.095, 0.016, 0.135)), &pages_mat);
    pages.position().y = 0.010;
    cbook.add(&pages);
    let back = mesh(box_geo!(0.10, 0.005, 0.14), &cover_mat);
    back.position().y = -0.001;
    cbook.add(&back);
    cbook.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    cbook.rotation().y = rnd() * PI * 2.0;
    cbook.rotation().x = (rnd() - 0.5) * 0.12;
    g.world.add(&cbook);
}

/// Batch r14b's kit (the ribbon): a seed from where it lies, and the fading
/// painted over the ribbon's colour.
mod r14b_kit {
    use game::prelude::*;

    /// A seed from values already fixed (a position): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    /// Fading over the ribbon's colour (multiplied): bleached paler in
    /// patches where the light lay, grimy where it was trodden
    pub fn faded_tex(seed: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            ctx.set_fill_style("#e8e8e8");
            ctx.fill_rect(0, 0, w, h);
            let mut r = SplitMix64::new(seed, 0x21bb);
            for _ in 0..10 {
                let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(6.0, 18.0));
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, "rgba(255,255,255,0.7)");
                gr.add_color_stop(1, "rgba(255,255,255,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
            }
            for _ in 0..8 {
                let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(4.0, 12.0));
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, "rgba(70,60,50,0.55)");
                gr.add_color_stop(1, "rgba(70,60,50,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
            }
            for _ in 0..40 {
                ctx.set_fill_style(&format!("rgba(40,34,28,{:.2})", r.range(0.2, 0.6)));
                ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), r.range(0.6, 1.6), r.range(0.6, 1.6));
            }
        })
    }
}

/// The napkin's print and wear (`place_small_paper_napkins`), seeded by where
/// it lies.
mod r15_napkin {
    use game::prelude::*;

    /// Tissue napkins (sRGB), by the old colour, gone dull
    const TISSUE: [(f64, f64, f64); 6] = [
        (224.0, 220.0, 210.0),
        (214.0, 198.0, 178.0),
        (212.0, 174.0, 180.0),
        (178.0, 190.0, 210.0),
        (218.0, 196.0, 146.0),
        (186.0, 208.0, 180.0),
    ];

    fn rgba(c: (f64, f64, f64), a: f64) -> String {
        format!("rgba({:.0},{:.0},{:.0},{a:.3})", c.0, c.1, c.2)
    }

    fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &rgba(c, a));
        gr.add_color_stop(1, &rgba(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
    }

    /// The napkin from above (165 mm, folded in four): the tissue's fine
    /// emboss all over, the border pressed in a band 12 mm from the edges,
    /// spots printed on a coloured one, the folds' lines; a smear of
    /// chocolate cake and a grease spot where it was used; the floor's dust
    pub fn napkin_tex(look: usize, salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x4a9c);
            let c = TISSUE[look];
            ctx.set_fill_style(rgba(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            // the emboss: a fine grid of pressed dots, lit from one side (a
            // 64 px repeat, laid as the embossing roller lays it)
            let mut tile = Canvas::new(64, 64);
            let mut y = 1.5;
            while y < 64.0 {
                let mut x = if (y as i32 / 4) % 2 == 0 { 1.5 } else { 3.5 };
                while x < 64.0 {
                    tile.set_fill_style("rgba(255,255,250,0.10)");
                    tile.fill_rect(x, y, 1.0, 1.0);
                    tile.set_fill_style("rgba(0,0,0,0.06)");
                    tile.fill_rect(x + 1.0, y + 1.0, 1.0, 1.0);
                    x += 4.0;
                }
                y += 4.0;
            }
            for ty in 0..4 {
                for tx in 0..4 {
                    ctx.draw_image(&tile, tx as f64 * 64.0, ty as f64 * 64.0);
                }
            }
            // the border's pressed band: a wave between two lines
            let b = 18.0;
            ctx.set_line_width(1.0);
            for (dx, col) in [(0.0, "rgba(255,255,250,0.22)"), (1.0, "rgba(0,0,0,0.14)")] {
                ctx.set_stroke_style(col);
                for inset in [b - 6.0, b + 6.0] {
                    ctx.stroke_rect(inset + dx, inset + dx, w - 2.0 * inset, h - 2.0 * inset);
                }
                ctx.begin_path();
                let mut t = 0.0;
                let per = 4.0 * (w - 2.0 * b);
                while t <= per {
                    let (side, s) = ((t / (w - 2.0 * b)) as i32, t % (w - 2.0 * b));
                    let off = 3.5 * (t / 6.0).sin();
                    let (x, y) = match side {
                        0 => (b + s, b + off),
                        1 => (w - b + off, b + s),
                        2 => (w - b - s, h - b + off),
                        _ => (b + off, h - b - s),
                    };
                    if t == 0.0 {
                        ctx.move_to(x + dx, y + dx);
                    } else {
                        ctx.line_to(x + dx, y + dx);
                    }
                    t += 2.0;
                }
                ctx.stroke();
            }
            // spots printed on the coloured ones (a white one is plain)
            if look >= 2 {
                let ink = [(240.0, 236.0, 228.0), (200.0, 120.0, 120.0), (110.0, 140.0, 190.0), (230.0, 230.0, 220.0)][(salt % 4) as usize];
                ctx.set_fill_style(rgba(ink, 0.55));
                let mut j = 0;
                let mut y = 30.0;
                while y < h - 26.0 {
                    let mut x = if j % 2 == 0 { 30.0 } else { 44.0 };
                    while x < w - 26.0 {
                        ctx.begin_path();
                        ctx.arc(x, y, 3.2, 0, 7);
                        ctx.fill();
                        x += 28.0;
                    }
                    y += 24.0;
                    j += 1;
                }
            }
            // the quarter folds, crossing it
            ctx.set_stroke_style("rgba(0,0,0,0.10)");
            ctx.set_line_width(1.4);
            ctx.begin_path();
            ctx.move_to(w / 2.0, 0);
            ctx.line_to(w / 2.0 + 3.0, h);
            ctx.move_to(0, h / 2.0);
            ctx.line_to(w, h / 2.0 - 2.0);
            ctx.stroke();
            // used: a smear of chocolate cake, a grease spot gone translucent
            let (sx, sy) = (60.0 + r.next() * 130.0, 60.0 + r.next() * 130.0);
            for k in 0..5 {
                let at = (sx + k as f64 * 6.0 + r.next() * 4.0, sy + k as f64 * 3.0 + r.next() * 4.0);
                soft(ctx, at, 7.0 + r.next() * 6.0, 0.6, 0.4, (86.0, 58.0, 40.0), 0.35 + 0.2 * r.next());
            }
            soft(
                ctx,
                (40.0 + r.next() * 170.0, 40.0 + r.next() * 170.0),
                18.0 + r.next() * 10.0,
                0.8,
                r.next() * 3.0,
                (150.0, 136.0, 110.0),
                0.22,
            );
            // the floor's dust
            for _ in 0..14 {
                let at = (r.next() * w, r.next() * h);
                soft(ctx, at, 12.0 + r.next() * 30.0, 0.5 + 0.5 * r.next(), r.next() * 3.0, (110.0, 100.0, 84.0), 0.08 + 0.12 * r.next());
            }
            ctx.set_fill_style("rgba(120,112,98,0.16)");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..400 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.25 * r.next()));
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
        })
    }
}
