//! Prop placers: study / parlour props — wingback armchairs, books on chair arms,
//! silhouette portraits, ottomans, humidors, brandy snifters, inkwells and
//! letters, silver trays, antique maps and pocket watches.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod t02;
use t02::place_antique_maps;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_wingback_armchairs", place_wingback_armchairs));
    r.push(("place_chair_arm_books", place_chair_arm_books));
    r.push(("place_silhouette_portraits", place_silhouette_portraits));
    r.push(("place_ottomans", place_ottomans));
    r.push(("place_humidors", place_humidors));
    r.push(("place_brandy_snifters", place_brandy_snifters));
    r.push(("place_inkwell_letters", place_inkwell_letters));
    r.push(("place_silver_trays", place_silver_trays));
    r.push(("place_antique_maps", place_antique_maps));
    r.push(("place_pocket_watches", place_pocket_watches));
}

/// A uniformly random element (one `rnd()` draw). index
#[inline]
fn ri(len: usize) -> usize {
    (rnd() * len as f64).floor() as usize
}

/// Every open (tile 0) interior tile, row-major loops.
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

// R743 wingback armchair (floor): 2% chance per level.
fn place_wingback_armchairs(g: &mut Game, _dist: &[i32]) {
    g.pp.wingback_armchair_positions.clear(); // R747 reset per level
    if rnd() < 0.98 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let uphol_colors: [u32; 7] = [0x603030, 0x2a4a60, 0x604040, 0x603520, 0x304020, 0x604060, 0x505050];
    let uphol_mat = lambert!({ color: uphol_colors[ri(uphol_colors.len())] });
    let wood_mat = lambert!({ color: 0x604020 });
    let wa = group();
    // Seat cushion — wide flat box.
    let (seat_w, seat_d, seat_t) = (0.55, 0.48, 0.12);
    let seat_y = 0.42;
    let seat = mesh(pg("WingbackArmchairs__seat", box_geo!(seat_w, seat_t, seat_d)), &uphol_mat);
    seat.position().y = seat_y;
    wa.add(&seat);
    // Base skirt — box beneath cushion (hem 3 cm off the floor so the feet show).
    let skirt = mesh(pg("WingbackArmchairs__base", box_geo!(seat_w + 0.020, 0.36, seat_d + 0.020)), &uphol_mat);
    skirt.scale().y = 0.33 / 0.36;
    skirt.position().y = 0.03 + 0.165;
    wa.add(&skirt);
    // Back panel — tall vertical box.
    let back_h = 0.75;
    let back = mesh(pg("WingbackArmchairs__back", box_geo!(seat_w, back_h, 0.16)), &uphol_mat);
    back.position().set(0, seat_y + seat_t / 2.0 + back_h / 2.0, -seat_d * 0.5 + 0.06);
    wa.add(&back);
    // 2 winged sides at top of back — tilted forward panels.
    for side in [-1.0f64, 1.0] {
        let wing = mesh(pg("WingbackArmchairs__wing", box_geo!(0.11, back_h * 0.75, 0.28)), &uphol_mat);
        wing.position().set(side * (seat_w * 0.5 - 0.02), seat_y + seat_t / 2.0 + back_h * 0.5, -seat_d * 0.5 + 0.18);
        wing.rotation().y = side * -0.10; // Wings angled slightly forward.
        wa.add(&wing);
    }
    // 2 armrests — arm bodies + rolled tops.
    for side in [-1.0f64, 1.0] {
        let arm_body = mesh(pg("WingbackArmchairs__arm", box_geo!(0.15, 0.26, seat_d - 0.04)), &uphol_mat);
        arm_body.position().set(side * (seat_w * 0.5 + 0.045), seat_y + seat_t / 2.0 + 0.10, 0.02);
        wa.add(&arm_body);
        let roll = mesh(pg("WingbackArmchairs__roll", cyl_geo!(0.075, 0.075, seat_d - 0.04, 8)), &uphol_mat);
        roll.rotation().x = PI / 2.0;
        roll.position().set(side * (seat_w * 0.5 + 0.045), seat_y + seat_t / 2.0 + 0.22, 0.02);
        wa.add(&roll);
    }
    // 4 wooden feet near the base's corners.
    for cx in [-seat_w * 0.46, seat_w * 0.46] {
        for cz in [-seat_d * 0.44, seat_d * 0.44] {
            let foot = mesh(pg("WingbackArmchairs__foot", cyl_geo!(0.014, 0.020, 0.030, 6)), &wood_mat);
            foot.position().set(cx, 0.015, cz);
            wa.add(&foot);
        }
    }
    // 50% chance: throw blanket draped over one arm.
    if rnd() < 0.5 {
        let throw_colors: [u32; 6] = [0xa02040, 0x408060, 0xd0a040, 0xf0e0c0, 0x606080, 0x804060];
        let throw_mat = lambert!({ color: throw_colors[ri(throw_colors.len())] });
        let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
        // Draped horizontal segment on top of the arm roll.
        let throw_h = mesh(pg("WingbackArmchairs__throwTop", box_geo!(0.13, 0.010, seat_d * 0.7)), &throw_mat);
        throw_h.position().set(side * (seat_w * 0.5 + 0.070), seat_y + seat_t / 2.0 + 0.300, 0.02);
        wa.add(&throw_h);
        // R748 Hanging subgroup — pivots at top for pendulum sway.
        let hang_group = group();
        hang_group.position().set(side * (seat_w * 0.5 + 0.130), seat_y + seat_t / 2.0 + 0.300, 0.02);
        wa.add(&hang_group);
        let throw_v = mesh(pg("WingbackArmchairs__throwHang", box_geo!(0.008, 0.20, seat_d * 0.6)), &throw_mat);
        throw_v.position().y = -0.10;
        hang_group.add(&throw_v);
        // Fringe/tassel bumps at bottom of hanging segment.
        for i in 0..3 {
            let tassel = mesh(pg("WingbackArmchairs__tassel", box_geo!(0.008, 0.020, 0.008)), &throw_mat);
            tassel.position().set(0, -0.22, (i as f64 - 1.0) * 0.014);
            hang_group.add(&tassel);
        }
        // R748 breeze-sway animator — very slight pendulum motion.
        let sway_freq = 0.35 + rnd() * 0.25;
        let sway_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            hang_group.rotation().z = (t * sway_freq + sway_phase).sin() * 0.028;
        });
    }
    // 40% chance: small pillow on seat.
    if rnd() < 0.4 {
        let pillow_colors: [u32; 5] = [0xa02040, 0x408060, 0x2060a0, 0xd0a040, 0xf0e0c0];
        let pillow_mat = lambert!({ color: pillow_colors[ri(pillow_colors.len())] });
        let pillow = mesh(pg("WingbackArmchairs__pillow", box_geo!(0.28, 0.06, 0.20)), &pillow_mat);
        pillow.position().set((rnd() - 0.5) * 0.10, seat_y + seat_t / 2.0 + 0.030, -seat_d * 0.25);
        pillow.rotation().y = (rnd() - 0.5) * 0.30;
        wa.add(&pillow);
    }
    wa.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    wa.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wa);
    let p = wa.pos();
    g.pp.wingback_armchair_positions.push(v3(p.x, 0, p.z)); // R747
}

// R745 open book on chair arm (floor): 5% chance per level.
fn place_chair_arm_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let wood_mat = lambert!({ color: 0x604020 });
    let cab = group();
    // Small side stand / narrow table.
    let (stand_w, stand_d, stand_h) = (0.34, 0.34, 0.55);
    let stand = mesh(pg("ChairArmBooks__top", box_geo!(stand_w, 0.014, stand_d)), &wood_mat);
    stand.position().y = stand_h;
    cab.add(&stand);
    // 4 tapered legs.
    for cx in [-stand_w * 0.42, stand_w * 0.42] {
        for cz in [-stand_d * 0.42, stand_d * 0.42] {
            let leg = mesh(pg("ChairArmBooks__leg", cyl_geo!(0.010, 0.014, stand_h - 0.007, 5)), &wood_mat);
            leg.position().set(cx, (stand_h - 0.007) / 2.0, cz);
            cab.add(&leg);
        }
    }
    // Cross-brace between the back pair of legs.
    let brace = mesh(pg("ChairArmBooks__brace", cyl_geo!(0.006, 0.006, stand_w * 0.75, 4)), &wood_mat);
    brace.rotation().z = PI / 2.0;
    brace.scale().y = (stand_w * 0.84) / (stand_w * 0.75);
    brace.position().set(0, stand_h * 0.35, -stand_d * 0.42);
    cab.add(&brace);
    // Open book face-down on stand.
    let book_colors: [u32; 5] = [0x603030, 0x2a4a60, 0x408030, 0x604040, 0x201050];
    let book_mat = lambert!({ color: book_colors[ri(book_colors.len())] });
    let page_mat = lambert!({ color: 0xf0e8d0 });
    let (book_w, book_d) = (0.12, 0.15);
    let left_page = mesh(pg("ChairArmBooks__pageL", box_geo!(book_w, 0.005, book_d)), &page_mat);
    left_page.position().set(-book_w / 2.0 + 0.004, stand_h + 0.014, 0);
    left_page.rotation().z = -0.10;
    cab.add(&left_page);
    let right_page = mesh(pg("ChairArmBooks__pageR", box_geo!(book_w, 0.005, book_d)), &page_mat);
    right_page.position().set(book_w / 2.0 - 0.004, stand_h + 0.014, 0);
    right_page.rotation().z = 0.10;
    cab.add(&right_page);
    // Central spine under the gutter.
    let spine = mesh(pg("ChairArmBooks__spine", cyl_geo!(0.003, 0.003, book_d, 4)), &book_mat);
    spine.rotation().x = PI / 2.0;
    spine.position().set(0, stand_h + 0.0045, 0);
    cab.add(&spine);
    // Cover flaps peeking from outer edges.
    let left_cover = mesh(pg("ChairArmBooks__cover", box_geo!(0.003, 0.007, book_d)), &book_mat);
    left_cover.position().set(-book_w + 0.002, stand_h + 0.016, 0);
    left_cover.rotation().z = -0.10;
    cab.add(&left_cover);
    let right_cover = mesh(pg("ChairArmBooks__cover", box_geo!(0.003, 0.007, book_d)), &book_mat);
    right_cover.position().set(book_w - 0.002, stand_h + 0.016, 0);
    right_cover.rotation().z = 0.10;
    cab.add(&right_cover);
    // 6 text lines per open page.
    let ink_mat = lambert!({ color: 0x1a1a1a });
    for side in 0..2 {
        for line in 0..6 {
            let line_w = book_w * (0.55 + rnd() * 0.35);
            let text_line = mesh(box_geo!(line_w, 0.002, 0.001), &ink_mat);
            let base_x = if side == 0 { -book_w * 0.5 } else { book_w * 0.5 };
            text_line.position().set(base_x + (rnd() - 0.5) * 0.010, stand_h + 0.0181, -book_d * 0.35 + line as f64 * (book_d * 0.11));
            text_line.rotation().z = if side == 0 { -0.10 } else { 0.10 };
            cab.add(&text_line);
        }
    }
    // 50% chance: bookmark ribbon peeking from spine.
    if rnd() < 0.5 {
        let ribbon_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xc0a040, 0x603030, 0x408060];
        let ribbon_mat = lambert!({ color: ribbon_colors[ri(ribbon_colors.len())] });
        let ribbon = mesh(pg("ChairArmBooks__ribbon", box_geo!(0.008, 0.002, 0.05)), &ribbon_mat);
        ribbon.position().set(0, stand_h + 0.0085, book_d * 0.5 + 0.025);
        cab.add(&ribbon);
    }
    // 40% chance: reading glasses beside book on stand.
    if rnd() < 0.4 {
        let frame_colors: [u32; 4] = [0x101010, 0x604020, 0xa02040, 0xc0a040];
        let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())] });
        let lens_mat = basic_mat!({ color: 0xf0f0f8, transparent: true, opacity: 0.4 });
        let lens_r = 0.018;
        for side in [-1.0f64, 1.0] {
            let ring = mesh(pg("ChairArmBooks__rim", torus_geo!(lens_r, 0.002, 4, 10)), &frame_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(side * lens_r * 1.2 + book_w * 0.5, stand_h + 0.009, 0.125);
            cab.add(&ring);
            let lens = mesh(circle_geo!(lens_r - 0.002, 10), &lens_mat);
            lens.rotation().x = -PI / 2.0;
            lens.position().set(side * lens_r * 1.2 + book_w * 0.5, stand_h + 0.0095, 0.125);
            cab.add(&lens);
        }
        // Bridge.
        let bridge = mesh(pg("ChairArmBooks__bridge", box_geo!(0.012, 0.002, 0.003)), &frame_mat);
        bridge.position().set(book_w * 0.5, stand_h + 0.009, 0.125 - 0.005);
        cab.add(&bridge);
        // Folded temples.
        for side in [-1.0f64, 1.0] {
            let temple = mesh(pg("ChairArmBooks__temple", cyl_geo!(0.001, 0.001, 0.038, 3)), &frame_mat);
            temple.rotation().z = PI / 2.0;
            temple.rotation().y = side * -0.4;
            temple.position().set(side * (lens_r * 1.2 + 0.019) + book_w * 0.5, stand_h + 0.008, 0.125 + 0.014);
            cab.add(&temple);
        }
    }
    cab.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    cab.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cab);
}

// R745 framed Victorian silhouette portrait texture.
game::lazy_static_tex!(
    silhouette_portrait_texture,
    canvas_tex(180, 220, |ctx, w, h| {
        // Cream background.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(0, 0, w, h);
        // Very subtle vignette darkening corners.
        let rg = ctx.create_radial_gradient(w / 2.0, h / 2.0, 30, w / 2.0, h / 2.0, w);
        rg.add_color_stop(0, "rgba(0,0,0,0)");
        rg.add_color_stop(1, "rgba(80,60,40,0.25)");
        ctx.set_fill_style(&rg);
        ctx.fill_rect(0, 0, w, h);
        // Central profile silhouette — black.
        ctx.set_fill_style("#0a0a0a");
        let (cx, cy) = (w / 2.0, h * 0.5);
        let variant = (rnd() * 3.0).floor() as i32;
        ctx.begin_path();
        if variant == 0 {
            // Adult male profile — angular jaw + short hair.
            ctx.move_to(cx - 5.0, cy - 55.0);
            ctx.quadratic_curve_to(cx + 28.0, cy - 60.0, cx + 32.0, cy - 30.0);
            ctx.quadratic_curve_to(cx + 34.0, cy - 15.0, cx + 25.0, cy - 12.0);
            ctx.quadratic_curve_to(cx + 40.0, cy - 5.0, cx + 42.0, cy);
            ctx.quadratic_curve_to(cx + 30.0, cy + 5.0, cx + 26.0, cy + 8.0);
            ctx.quadratic_curve_to(cx + 32.0, cy + 15.0, cx + 24.0, cy + 20.0);
            ctx.quadratic_curve_to(cx + 30.0, cy + 30.0, cx + 20.0, cy + 38.0);
            ctx.quadratic_curve_to(cx + 10.0, cy + 45.0, cx - 15.0, cy + 50.0);
            ctx.line_to(cx - 30.0, cy + 70.0);
            ctx.line_to(cx - 30.0, cy + 100.0);
            ctx.line_to(cx - 40.0, cy + 100.0);
            ctx.line_to(cx - 35.0, cy + 55.0);
            ctx.line_to(cx - 20.0, cy + 20.0);
            ctx.line_to(cx - 22.0, cy - 30.0);
            ctx.line_to(cx - 20.0, cy - 50.0);
            ctx.close_path();
        } else if variant == 1 {
            // Adult female profile — softer curves + updo hair bun.
            ctx.move_to(cx - 8.0, cy - 55.0);
            ctx.quadratic_curve_to(cx + 25.0, cy - 60.0, cx + 30.0, cy - 30.0);
            ctx.quadratic_curve_to(cx + 34.0, cy - 10.0, cx + 40.0, cy - 5.0);
            ctx.quadratic_curve_to(cx + 36.0, cy + 3.0, cx + 28.0, cy + 5.0);
            ctx.quadratic_curve_to(cx + 32.0, cy + 12.0, cx + 24.0, cy + 16.0);
            ctx.quadratic_curve_to(cx + 30.0, cy + 26.0, cx + 22.0, cy + 32.0);
            ctx.quadratic_curve_to(cx + 12.0, cy + 40.0, cx - 10.0, cy + 45.0);
            ctx.line_to(cx - 30.0, cy + 65.0);
            ctx.line_to(cx - 35.0, cy + 100.0);
            ctx.line_to(cx - 50.0, cy + 100.0);
            ctx.line_to(cx - 45.0, cy + 55.0);
            ctx.line_to(cx - 30.0, cy + 15.0);
            // Hair bun — small circle at back-top of head.
            ctx.line_to(cx - 34.0, cy - 30.0);
            ctx.quadratic_curve_to(cx - 42.0, cy - 50.0, cx - 30.0, cy - 55.0);
            ctx.close_path();
        } else {
            // Child profile — smaller, rounder head + shorter neck.
            ctx.move_to(cx - 5.0, cy - 40.0);
            ctx.quadratic_curve_to(cx + 22.0, cy - 45.0, cx + 26.0, cy - 20.0);
            ctx.quadratic_curve_to(cx + 30.0, cy - 5.0, cx + 34.0, cy);
            ctx.quadratic_curve_to(cx + 28.0, cy + 6.0, cx + 20.0, cy + 8.0);
            ctx.quadratic_curve_to(cx + 24.0, cy + 15.0, cx + 18.0, cy + 18.0);
            ctx.quadratic_curve_to(cx + 22.0, cy + 26.0, cx + 15.0, cy + 30.0);
            ctx.line_to(cx - 8.0, cy + 38.0);
            ctx.line_to(cx - 22.0, cy + 55.0);
            ctx.line_to(cx - 22.0, cy + 85.0);
            ctx.line_to(cx - 32.0, cy + 85.0);
            ctx.line_to(cx - 28.0, cy + 50.0);
            ctx.line_to(cx - 15.0, cy + 15.0);
            ctx.line_to(cx - 18.0, cy - 25.0);
            ctx.close_path();
        }
        ctx.fill();
        // Oval-shaped mat frame.
        ctx.set_stroke_style("#a06030");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.ellipse(cx, cy, w * 0.42, h * 0.42, 0, 0, PI * 2.0);
        ctx.stroke();
        // Gold rim inside oval.
        ctx.set_stroke_style("#c0a060");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.ellipse(cx, cy, w * 0.40, h * 0.40, 0, 0, PI * 2.0);
        ctx.stroke();
        // Signature/name at bottom.
        ctx.set_fill_style("#402020");
        ctx.set_font("italic 10px cursive");
        ctx.set_text_align("center");
        let names = [
            "Charles W., aet 34",
            "Elizabeth, aet 28",
            "Little Sarah",
            "Master James",
            "Mrs. Anderson",
            "Papa, 1889",
            "Mother, 1902",
            "Young Thomas",
        ];
        ctx.fill_text(names[ri(names.len())], w / 2.0, h - 12.0);
    })
);

fn place_silhouette_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let mut wall_tiles: Vec<(i32, i32, i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        wall_tiles.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[ri(wall_tiles.len())];
    let frame_colors: [u32; 4] = [0x604020, 0xd0a040, 0x2a1810, 0xc0a060];
    let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())] });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let portrait_mat = lambert!({ map: silhouette_portrait_texture() });
    let sp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.18, 0.22);
    let frame = mesh(pg("SilhouettePortraits__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    sp.add(&frame);
    // Gold inner mat.
    let gold_inner = mesh(pg("SilhouettePortraits__mat", box_geo!(frame_w - 0.010, frame_h - 0.010, 0.002)), &gold_mat);
    gold_inner.position().set(0, mount_y, 0.018);
    sp.add(&gold_inner);
    // Portrait face plane.
    let portrait = mesh(plane_geo!(frame_w - 0.024, frame_h - 0.024), &portrait_mat);
    portrait.position().set(0, mount_y, 0.020);
    sp.add(&portrait);
    sp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    sp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    sp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&sp);
}

// R746 ottoman / footstool (floor): 4% chance per level.
fn place_ottomans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let uphol_colors: [u32; 7] = [0x603030, 0x2a4a60, 0x604040, 0x603520, 0x304020, 0x604060, 0x505050];
    let uphol_mat = lambert!({ color: uphol_colors[ri(uphol_colors.len())] });
    let wood_mat = lambert!({ color: 0x604020 });
    let ot = group();
    // Bimodal: round tufted ottoman vs square upholstered footstool.
    let is_round = rnd() < 0.5;
    let ott_h = 0.32;
    if is_round {
        let ot_r = 0.24;
        let cushion = mesh(pg("Ottomans__drum", cyl_geo!(ot_r, ot_r - 0.010, 0.18, 12)), &uphol_mat);
        cushion.position().y = 0.16 + 0.09; // sits on the skirt
        ot.add(&cushion);
        // 5 button-tuft indents on top.
        let button_mat = lambert!({ color: 0x1a1a1a });
        let button_r = 0.006;
        for i in 0..5 {
            let ang = if i == 0 { 0.0 } else { (i as f64 - 1.0) / 4.0 * PI * 2.0 };
            let rr = if i == 0 { 0.0 } else { ot_r * 0.55 };
            let btn = mesh(circle_geo!(button_r, 5), &button_mat);
            btn.rotation().x = -PI / 2.0;
            btn.position().set(ang.cos() * rr, 0.16 + 0.18 - (if i == 0 { 0.011 } else { 0.020 }), ang.sin() * rr);
            ot.add(&btn);
        }
        // Skirt below cushion.
        let skirt = mesh(pg("Ottomans__drumSkirt", cyl_geo!(ot_r - 0.005, ot_r - 0.005, 0.14, 12)), &uphol_mat);
        skirt.position().y = 0.09;
        ot.add(&skirt);
        // 4 small wooden feet.
        for i in 0..4 {
            let ang = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
            let foot = mesh(pg("Ottomans__bunFoot", cyl_geo!(0.014, 0.020, 0.020, 5)), &wood_mat);
            foot.position().set(ang.cos() * (ot_r - 0.020), 0.010, ang.sin() * (ot_r - 0.020));
            ot.add(&foot);
        }
    } else {
        // Square footstool — rectangular cushion + 4 tapered legs.
        let (ot_w, ot_d) = (0.38, 0.30);
        let cushion = mesh(pg("Ottomans__cushion", box_geo!(ot_w, 0.14, ot_d)), &uphol_mat);
        cushion.position().y = 0.22 + 0.07;
        ot.add(&cushion);
        let skirt = mesh(pg("Ottomans__apron", box_geo!(ot_w - 0.008, 0.10, ot_d - 0.008)), &uphol_mat);
        skirt.position().y = 0.17; // top meets the cushion
        ot.add(&skirt);
        for cx in [-ot_w * 0.42, ot_w * 0.42] {
            for cz in [-ot_d * 0.42, ot_d * 0.42] {
                let leg = mesh(pg("Ottomans__leg", cyl_geo!(0.010, 0.014, 0.14, 5)), &wood_mat);
                leg.position().set(cx, 0.07, cz);
                ot.add(&leg);
            }
        }
    }
    // 30% chance: throw pillow on top.
    if rnd() < 0.3 {
        let pillow_colors: [u32; 5] = [0xa02040, 0x408060, 0xd0a040, 0xf0e0c0, 0x2060a0];
        let pillow_mat = lambert!({ color: pillow_colors[ri(pillow_colors.len())] });
        let pillow = mesh(pg("Ottomans__pillow", box_geo!(0.16, 0.04, 0.16)), &pillow_mat);
        pillow.position().set(0, if is_round { 0.16 + 0.18 + 0.020 } else { 0.36 + 0.020 }, 0);
        pillow.rotation().y = (rnd() - 0.5) * 0.4;
        ot.add(&pillow);
    }
    ot.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    ot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ot);
}

// R746 humidor cigar box w/ scattered cigars (floor): 3% chance per level.
fn place_humidors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let box_colors: [u32; 4] = [0x603020, 0x8b5a2b, 0x2a1810, 0x604040];
    let box_mat = lambert!({ color: box_colors[ri(box_colors.len())] });
    let lining_colors: [u32; 3] = [0x603030, 0x201050, 0x1a1a1a];
    let lining_mat = lambert!({ color: lining_colors[ri(lining_colors.len())] });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let cigar_colors: [u32; 5] = [0x8b5a2b, 0x604020, 0x704a20, 0xa07040, 0x502010];
    let band_colors: [u32; 4] = [0xd0a060, 0xa02040, 0x2060a0, 0xd0d040];
    let hm = group();
    // Bottom half — rectangular wooden box.
    let (box_w, box_d, box_h) = (0.20, 0.14, 0.055);
    let bottom = mesh(pg("Humidors__box", box_geo!(box_w, box_h, box_d)), &box_mat);
    bottom.position().y = box_h / 2.0 + 0.002;
    hm.add(&bottom);
    // Gold trim strips on top edges of box.
    let trim_top = mesh(pg("Humidors__trim", box_geo!(box_w + 0.003, 0.003, box_d + 0.003)), &gold_mat);
    trim_top.position().y = box_h + 0.002;
    hm.add(&trim_top);
    // Interior velvet lining.
    let lining = mesh(pg("Humidors__lining", box_geo!(box_w - 0.010, 0.003, box_d - 0.010)), &lining_mat);
    lining.position().y = box_h + 0.002;
    hm.add(&lining);
    // Top lid opened about its hinge on the box's back top edge.
    let lid_hinge = group();
    lid_hinge.position().set(0, box_h + 0.002, -box_d * 0.5);
    lid_hinge.rotation().x = -PI / 3.0;
    hm.add(&lid_hinge);
    let top = mesh(pg("Humidors__lid", box_geo!(box_w, box_h * 0.35, box_d)), &box_mat);
    top.position().set(0, box_h * 0.175, box_d * 0.5);
    lid_hinge.add(&top);
    let lid_trim = mesh(pg("Humidors__trim", box_geo!(box_w + 0.003, 0.003, box_d + 0.003)), &gold_mat);
    lid_trim.position().set(0, box_h * 0.35, box_d * 0.5);
    lid_hinge.add(&lid_trim);
    // Small hinge cylinder + brass clasp.
    let hinge = mesh(pg("Humidors__hinge", cyl_geo!(0.003, 0.003, box_w * 0.85, 5)), &gold_mat);
    hinge.rotation().z = PI / 2.0;
    hinge.position().set(0, box_h + 0.005, -box_d * 0.5);
    hm.add(&hinge);
    let clasp = mesh(pg("Humidors__clasp", box_geo!(0.010, 0.005, 0.003)), &gold_mat);
    clasp.position().set(0, box_h * 0.7, box_d * 0.5 + 0.002);
    hm.add(&clasp);
    // 4-6 cigars scattered on lining + spilled on floor.
    let cigar_count = 4 + (rnd() * 3.0).floor() as i32;
    struct RockingCigar {
        cigar: Obj,
        band: Obj,
        base_y: f64,
        band_off: f64,
        phase: f64,
        freq: f64,
        amp: f64,
    }
    let mut rocking_cigars: Vec<RockingCigar> = vec![]; // R747 spilled cigars that rock slowly
    for i in 0..cigar_count {
        let cigar_mat = lambert!({ color: cigar_colors[ri(cigar_colors.len())] });
        let cigar_len = 0.11 + rnd() * 0.03;
        let cigar_r = 0.007 + rnd() * 0.002;
        let cigar = mesh(pg("Humidors__cigar", cyl_geo!(cigar_r, cigar_r * 0.9, cigar_len, 6)), &cigar_mat);
        cigar.rotation().z = PI / 2.0;
        cigar.rotation().y = (rnd() - 0.5) * 1.2;
        let is_inside = i < 3;
        if is_inside {
            let px = (rnd() - 0.5) * (box_w - 0.030);
            let pz = (rnd() - 0.5) * (box_d - 0.030);
            cigar.position().set(px, box_h + 0.010, pz);
        } else {
            let px = box_w * 0.5 + 0.02 + rnd() * 0.10;
            let pz = (rnd() - 0.5) * 0.10;
            cigar.position().set(px, cigar_r + 0.002, pz);
        }
        hm.add(&cigar);
        // Colored band (label) near center of cigar.
        let band_mat = lambert!({ color: band_colors[ri(band_colors.len())] });
        let band = mesh(pg("Humidors__band", cyl_geo!(cigar_r + 0.0004, cigar_r + 0.0004, 0.010, 6)), &band_mat);
        band.rotation().z = PI / 2.0;
        let cry = cigar.rot().y;
        band.rotation().y = cry;
        band.position().copy(&cigar.pos());
        // Along the cigar's axis: rotation.y sends its length (X) to (cos, 0, -sin).
        let dx1 = cry.cos() * cigar_len * 0.25;
        let dz1 = -cry.sin() * cigar_len * 0.25;
        band.position().x += dx1;
        band.position().z += dz1;
        hm.add(&band);
        // R747 spilled cigars only: 50% chance to slowly rock in place.
        if !is_inside && rnd() < 0.5 {
            let phase = rnd() * PI * 2.0;
            let freq = 0.35 + rnd() * 0.4;
            let amp = 0.05 + rnd() * 0.05;
            rocking_cigars.push(RockingCigar {
                cigar: cigar.clone(),
                band: band.clone(),
                base_y: cry,
                band_off: cigar_len * 0.25,
                phase,
                freq,
                amp,
            });
        }
    }
    // R747 cigar rocking animator.
    if !rocking_cigars.is_empty() {
        g.animate(move |_g, _dt, t| {
            for rc in &rocking_cigars {
                let yr = rc.base_y + (t * rc.freq + rc.phase).sin() * rc.amp;
                rc.cigar.rotation().y = yr;
                rc.band.rotation().y = yr;
                // keep the band on the cigar as it swings about its centre
                let cp = rc.cigar.pos();
                rc.band.position().x = cp.x + yr.cos() * rc.band_off;
                rc.band.position().z = cp.z - yr.sin() * rc.band_off;
            }
        });
    }
    // 40% chance: metal cigar cutter beside humidor.
    if rnd() < 0.4 {
        let cutter_mat = lambert!({ color: 0xa0a0a8 });
        let cutter = mesh(pg("Humidors__cutter", cyl_geo!(0.020, 0.020, 0.006, 8)), &cutter_mat);
        cutter.rotation().z = PI / 2.0;
        cutter.position().set(-box_w * 0.5 - 0.06, 0.020, 0.04);
        hm.add(&cutter);
        let cutter_hole = mesh(cyl_geo!(0.008, 0.008, 0.008, 6), lambert!({ color: 0x0a0a0a }));
        cutter_hole.rotation().z = PI / 2.0;
        cutter_hole.position().set(-box_w * 0.5 - 0.06, 0.020, 0.04);
        hm.add(&cutter_hole);
    }
    // 30% chance: matchbook — small paper folder.
    if rnd() < 0.3 {
        let matchbook_mat = lambert!({ color: 0xa02020 });
        let matchbook = mesh(pg("Humidors__matchbook", box_geo!(0.030, 0.006, 0.024)), &matchbook_mat);
        matchbook.position().set(box_w * 0.5 + 0.05, 0.003, -0.06);
        matchbook.rotation().y = (rnd() - 0.5) * 0.5;
        hm.add(&matchbook);
        let text_line = mesh(box_geo!(0.020, 0.001, 0.001), lambert!({ color: 0xf0e8c0 }));
        text_line.position().set(box_w * 0.5 + 0.05, 0.007, -0.06);
        text_line.rotation().y = matchbook.rot().y;
        hm.add(&text_line);
    }
    hm.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    hm.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hm);
}

// R748 brandy snifter: batch `t10`'s (`t10.rs`)
mod t10;
use t10::place_brandy_snifters;

// R749 partial letter texture: mid-word interruption pattern.
game::lazy_static_tex!(
    partial_letter_texture,
    canvas_tex(180, 220, |ctx, w, h| {
        ctx.set_fill_style("#f4ecd0");
        ctx.fill_rect(0, 0, w, h);
        // Faint blue-ruled lines.
        ctx.set_stroke_style("rgba(120,140,180,0.28)");
        ctx.set_line_width(0.7);
        let mut y = 24.0;
        while y < h - 10.0 {
            ctx.begin_path();
            ctx.move_to(8, y);
            ctx.line_to(w - 8.0, y);
            ctx.stroke();
            y += 14.0;
        }
        // Red left margin.
        ctx.set_stroke_style("rgba(200,60,60,0.35)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(18, 8);
        ctx.line_to(18, h - 8.0);
        ctx.stroke();
        // Date top right.
        let dates = ["August 3, 1958", "April 17, 1971", "November 9, 1965", "February 24, 1982"];
        ctx.set_fill_style("#302010");
        ctx.set_font("italic 11px cursive");
        ctx.set_text_align("right");
        ctx.fill_text(dates[ri(dates.len())], w - 12.0, 20);
        // Greeting.
        let names = ["Sarah", "Michael", "Emma", "James", "Rose", "Thomas", "Anna", "David"];
        ctx.set_text_align("left");
        ctx.set_font("italic 16px cursive");
        ctx.fill_text(&format!("Dear {},", names[ri(names.len())]), 24, 44);
        // 3-4 lines of wavy cursive — then cut off.
        ctx.set_stroke_style("rgba(50,35,20,0.75)");
        ctx.set_line_width(1.2);
        let lines_filled = 3 + (rnd() * 2.0).floor() as i32;
        let (mut last_x, mut last_y) = (24.0, 66.0);
        for i in 0..lines_filled {
            let y = 66.0 + i as f64 * 14.0;
            let is_last = i == lines_filled - 1;
            let line_len = if is_last { 20.0 + rnd() * 40.0 } else { 100.0 + rnd() * 40.0 };
            ctx.begin_path();
            ctx.move_to(24, y);
            let mut cx = 24.0;
            while cx < 24.0 + line_len {
                let seg_len = 6.0 + rnd() * 8.0;
                let nx = cx + seg_len;
                let dy = (rnd() - 0.5) * 4.0;
                ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.5, nx, y + dy);
                cx = nx;
            }
            ctx.stroke();
            if is_last {
                last_x = 24.0 + line_len;
                last_y = y;
            }
        }
        // Big ink blot at cut-off.
        ctx.set_fill_style("rgba(20,15,10,0.92)");
        ctx.begin_path();
        let r = 4.0 + rnd() * 2.0;
        ctx.arc(last_x + 3.0, last_y + 1.0, r, 0, PI * 2.0);
        ctx.fill();
        // Splatter — 3-5 small satellite droplets.
        let splatter_n = 3 + (rnd() * 3.0).floor() as i32;
        for _ in 0..splatter_n {
            let sx = last_x + 3.0 + (rnd() - 0.5) * 18.0;
            let sy = last_y + 1.0 + (rnd() - 0.5) * 12.0;
            ctx.begin_path();
            let r = 0.8 + rnd() * 1.5;
            ctx.arc(sx, sy, r, 0, PI * 2.0);
            ctx.fill();
        }
        // Trailing ink smear from blot.
        ctx.set_stroke_style("rgba(20,15,10,0.5)");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(last_x + 3.0, last_y + 1.0);
        ctx.quadratic_curve_to(last_x + 12.0, last_y + 6.0, last_x + 24.0, last_y + 3.0);
        ctx.stroke();
        // Fold crease.
        ctx.set_stroke_style("rgba(120,90,60,0.30)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w, h * 0.5);
        ctx.stroke();
    })
);

// R749 fountain pen + inkwell + partially-written letter (floor): 4% chance.
fn place_inkwell_letters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let iw = group();
    // Letter sheet on floor with partial-writing texture.
    let paper_mat = lambert!({ map: partial_letter_texture() });
    let (sheet_w, sheet_d) = (0.16, 0.20);
    let sheet = mesh(pg("InkwellLetters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &paper_mat);
    sheet.position().y = 0.001;
    sheet.rotation().y = (rnd() - 0.5) * 0.3;
    iw.add(&sheet);
    // Fountain pen laid diagonally across letter.
    let pen_colors: [u32; 5] = [0x1a1a2a, 0x2a1a10, 0x501010, 0x2a3a4a, 0x604020];
    let pen_mat = lambert!({ color: pen_colors[ri(pen_colors.len())] });
    let gold_mat = lambert!({ color: 0xd4b060 });
    let silver_mat = lambert!({ color: 0xc0c0c8 });
    let pen = group();
    let barrel_len = 0.11;
    let barrel = mesh(pg("InkwellLetters__barrel", cyl_geo!(0.006, 0.005, barrel_len, 8)), &pen_mat);
    barrel.rotation().z = PI / 2.0;
    pen.add(&barrel);
    // Metal cap section.
    let use_gold = rnd() < 0.5;
    let trim_mat = if use_gold { gold_mat.clone() } else { silver_mat.clone() };
    let cap_sec = mesh(pg("InkwellLetters__capband", cyl_geo!(0.0065, 0.0065, 0.020, 8)), &trim_mat);
    cap_sec.rotation().z = PI / 2.0;
    cap_sec.position().x = -barrel_len * 0.45;
    pen.add(&cap_sec);
    // Clip on cap.
    let clip = mesh(pg("InkwellLetters__clip", box_geo!(0.016, 0.003, 0.003)), &trim_mat);
    clip.position().set(-barrel_len * 0.45, 0.007, 0);
    pen.add(&clip);
    // Nib section — tapered gold tip.
    let nib = mesh(pg("InkwellLetters__nib", cyl_geo!(0.0015, 0.004, 0.014, 8)), &trim_mat);
    nib.rotation().z = -PI / 2.0;
    nib.position().x = barrel_len * 0.55;
    pen.add(&nib);
    // Tiny black ink drop hanging from nib tip.
    let ink_drop = mesh(pg("InkwellLetters__drop", sphere_geo!(0.002, 5, 4)), lambert!({ color: 0x080810 }));
    let ink_drop_base_y = -0.001;
    ink_drop.position().set(barrel_len * 0.62, ink_drop_base_y, 0);
    pen.add(&ink_drop);
    // R750 ink drop grow+drop animator.
    let drop_cycle = 6.0 + rnd() * 4.0;
    let drop_phase = rnd() * drop_cycle;
    g.animate(move |_g, _dt, t| {
        let local = ((t + drop_phase) % drop_cycle) / drop_cycle;
        if local < 0.85 {
            let s = 1.0 + local * 0.9;
            ink_drop.scale().set(s, s + local * 0.4, s);
            ink_drop.position().y = ink_drop_base_y - local * 0.0008;
            ink_drop.set_visible(true);
        } else if local < 0.92 {
            let fall_local = (local - 0.85) / 0.07;
            ink_drop.position().y = ink_drop_base_y - 0.001 - fall_local * 0.010;
            ink_drop.scale().set_scalar(1.9 * (1.0 - fall_local * 0.5));
        } else {
            ink_drop.set_visible(false);
        }
    });
    // Position pen laid diagonally across letter.
    pen.position().set(sheet_w * 0.15, 0.008, sheet_d * 0.10);
    pen.rotation().y = -PI / 3.0 + (rnd() - 0.5) * 0.4;
    iw.add(&pen);
    // Inkwell — small glass jar with dark liquid + cap.
    let inkwell = group();
    let ink_glass_mat = lambert!({ color: 0x505860, transparent: true, opacity: 0.55 });
    let jar_r = 0.024;
    let jar = mesh(cyl_geo!(jar_r, jar_r * 0.95, 0.028, 12), &ink_glass_mat);
    jar.position().y = 0.014;
    inkwell.add(&jar);
    // Ink inside — black / blue-black / sepia-brown.
    let ink_colors: [u32; 3] = [0x080810, 0x0a1428, 0x1a1005];
    let ink_color = ink_colors[ri(ink_colors.len())];
    let ink_mat = lambert!({ color: ink_color, transparent: true, opacity: 0.95 });
    let ink_vol = mesh(cyl_geo!(jar_r - 0.003, jar_r - 0.003, 0.020, 10), &ink_mat);
    ink_vol.position().y = 0.011;
    inkwell.add(&ink_vol);
    let ink_surf = mesh(circle_geo!(jar_r - 0.003, 12), &ink_mat);
    ink_surf.rotation().x = -PI / 2.0;
    ink_surf.position().y = 0.021;
    inkwell.add(&ink_surf);
    // Neck ring (top of jar).
    let neck = mesh(pg("InkwellLetters__neck", torus_geo!(jar_r, 0.002, 4, 12)), &trim_mat);
    neck.rotation().x = PI / 2.0;
    neck.position().y = 0.028;
    inkwell.add(&neck);
    // Optional open cap on floor beside — 50% chance.
    if rnd() < 0.5 {
        let cap = mesh(pg("InkwellLetters__cap", cyl_geo!(jar_r + 0.001, jar_r + 0.001, 0.008, 12)), &trim_mat);
        cap.rotation().z = (rnd() - 0.5) * 0.3;
        let crz = cap.rot().z;
        cap.position().set(-0.030, 0.004 + (jar_r + 0.001) * crz.sin().abs(), -0.043);
        inkwell.add(&cap);
    }
    // Beside the letter, clear of its left edge.
    inkwell.position().set(-sheet_w * 0.5 - jar_r - 0.014, 0, sheet_d * 0.15);
    iw.add(&inkwell);
    // 30% chance: small ink smudge stain on the paper.
    if rnd() < 0.3 {
        let smudge_mat = lambert!({ color: ink_color, transparent: true, opacity: 0.5 });
        let smudge = mesh(circle_geo!(0.008, 5), &smudge_mat);
        smudge.rotation().x = -PI / 2.0;
        smudge.position().set(sheet_w * 0.30, 0.003, -sheet_d * 0.15);
        iw.add(&smudge);
    }
    // 25% chance: second blank sheet of stationery beside.
    if rnd() < 0.25 {
        let blank_mat = lambert!({ color: 0xf4ecd0 });
        let blank = mesh(pg("InkwellLetters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &blank_mat);
        blank.position().set(sheet_w + 0.02, 0.001, 0);
        blank.rotation().y = (rnd() - 0.5) * 0.3;
        iw.add(&blank);
    }
    iw.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    iw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&iw);
}

// R750 silver serving tray with optional teacups + biscuits + sugar bowl (floor): 4%.
fn place_silver_trays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Tray metal 3-color pool: silver / pewter / brass.
    let tray_mats = [
        lambert!({ color: 0xd8d8e0, emissive: 0x161818 }), // polished silver
        lambert!({ color: 0x909098 }),                     // pewter
        lambert!({ color: 0xc0a060, emissive: 0x201810 }), // brass
    ];
    let tray_mat = tray_mats[ri(tray_mats.len())].clone();
    let st = group();
    // Main tray body.
    let tray_r = 0.14;
    let tray = mesh(pg("SilverTrays__tray", cyl_geo!(tray_r, tray_r, 0.008, 20)), &tray_mat);
    tray.position().y = 0.008;
    st.add(&tray);
    // Raised rim.
    let rim = mesh(pg("SilverTrays__rim", torus_geo!(tray_r, 0.006, 5, 20)), &tray_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.014;
    st.add(&rim);
    // 2 side handles.
    for side in [-1.0f64, 1.0] {
        let handle = mesh(pg("SilverTrays__handle", torus_geo!(0.020, 0.004, 4, 8, PI)), &tray_mat);
        handle.rotation().x = PI / 2.0;
        handle.rotation().z = -PI / 2.0;
        handle.rotation().y = if side > 0.0 { 0.0 } else { PI };
        handle.position().set(side * (tray_r + 0.002), 0.015, 0);
        st.add(&handle);
    }
    // Engraved center ring (decorative).
    let center_ring = mesh(pg("SilverTrays__engraving", torus_geo!(tray_r * 0.45, 0.001, 4, 20)), lambert!({ color: 0xa0a0a8 }));
    center_ring.rotation().x = PI / 2.0;
    center_ring.position().y = 0.0114;
    st.add(&center_ring);
    // Item pool.
    let has_tea = rnd() < 0.60;
    let tea_colors: [u32; 4] = [0xf0e8d8, 0xe0d8c0, 0xf0d0d0, 0xd0e0f0];
    if has_tea {
        let teacup_count = 1 + (rnd() * 2.0).floor() as i32;
        for i in 0..teacup_count {
            let fi = i as f64;
            let cup_mat = lambert!({ color: tea_colors[ri(tea_colors.len())] });
            let cup = mesh(pg("SilverTrays__cup", cyl_geo!(0.024, 0.020, 0.032, 12)), &cup_mat);
            cup.position().set((fi - 0.5) * 0.08, 0.0292, -0.03);
            st.add(&cup);
            let saucer = mesh(pg("SilverTrays__saucer", cyl_geo!(0.038, 0.038, 0.003, 14)), &cup_mat);
            saucer.position().set((fi - 0.5) * 0.08, 0.0131, -0.03);
            st.add(&saucer);
            let cup_handle = mesh(pg("SilverTrays__cupHandle", torus_geo!(0.008, 0.002, 4, 8)), &cup_mat);
            cup_handle.position().set((fi - 0.5) * 0.08 + 0.0295, 0.0282, -0.03);
            st.add(&cup_handle);
            // Tea inside — dark liquid disc.
            let tea_mat = lambert!({ color: 0x604020, transparent: true, opacity: 0.9 });
            let tea_surf = mesh(circle_geo!(0.020, 12), &tea_mat);
            tea_surf.rotation().x = -PI / 2.0;
            tea_surf.position().set((fi - 0.5) * 0.08, 0.0422, -0.03);
            st.add(&tea_surf);
            // R751 steam wisp animator (60% per cup).
            if rnd() < 0.6 {
                let tea_steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.16 });
                struct Wisp {
                    mesh: Obj,
                    phase: f64,
                    base_y: f64,
                    cup_cx: f64,
                }
                let mut tea_wisps: Vec<Wisp> = vec![];
                let cup_cx = (fi - 0.5) * 0.08;
                for j in 0..2 {
                    let wisp = mesh(sphere_geo!(0.008, 6, 5), &tea_steam_mat);
                    wisp.scale().y = 2.0;
                    let px = cup_cx + (rnd() - 0.5) * 0.008;
                    let py = 0.055 + j as f64 * 0.020;
                    let pz = -0.03 + (rnd() - 0.5) * 0.008;
                    wisp.position().set(px, py, pz);
                    let phase = rnd() * PI * 2.0;
                    tea_wisps.push(Wisp { mesh: wisp.clone(), phase, base_y: wisp.pos().y, cup_cx });
                    st.add(&wisp);
                }
                g.animate(move |_g, _dt, t| {
                    for w in &tea_wisps {
                        {
                            let mut p = w.mesh.position();
                            p.y = w.base_y + (t * 0.7 + w.phase).sin() * 0.010;
                            p.x = w.cup_cx + (t * 0.5 + w.phase).sin() * 0.010;
                            p.z = -0.03 + (t * 0.6 + w.phase * 1.3).cos() * 0.010;
                        }
                        let s = 0.85 + (t * 0.9 + w.phase).sin() * 0.18;
                        w.mesh.scale().set(s, 2.0 * s, s);
                    }
                });
            }
            // 40% chance small spoon leaning against saucer.
            if rnd() < 0.4 {
                let spoon_mat = lambert!({ color: 0xd8d8e0 });
                let spoon = mesh(pg("SilverTrays__spoon", cyl_geo!(0.0015, 0.0015, 0.030, 4)), &spoon_mat);
                spoon.position().set((fi - 0.5) * 0.08 + 0.030, 0.027, -0.045);
                spoon.rotation().z = 0.6;
                st.add(&spoon);
                let spoon_bowl = mesh(pg("SilverTrays__spoonBowl", sphere_geo!(0.004, 6, 4)), &spoon_mat);
                spoon_bowl.scale().set(1.4, 0.4, 1.0);
                spoon_bowl.position().set((fi - 0.5) * 0.08 + 0.043, 0.0132, -0.045);
                st.add(&spoon_bowl);
            }
        }
    }
    // 50% chance biscuits on small plate.
    if rnd() < 0.5 {
        let plate_mat = lambert!({ color: 0xf0e8d8 });
        let plate = mesh(pg("SilverTrays__plate", cyl_geo!(0.040, 0.040, 0.003, 14)), &plate_mat);
        plate.position().set(0.04, 0.0131, 0.06);
        st.add(&plate);
        let biscuit_count = 3 + (rnd() * 2.0).floor() as i32;
        let biscuit_colors: [u32; 3] = [0xc09060, 0xa07040, 0xd0a080];
        for i in 0..biscuit_count {
            let biscuit_mat = lambert!({ color: biscuit_colors[ri(biscuit_colors.len())] });
            let biscuit = mesh(pg("SilverTrays__biscuit", cyl_geo!(0.011, 0.011, 0.004, 8)), &biscuit_mat);
            let stacked = i < 2;
            if stacked {
                biscuit.position().set(0.04, 0.0151 + i as f64 * 0.004, 0.06);
            } else {
                let px = 0.04 + (rnd() - 0.5) * 0.045;
                let pz = 0.06 + (rnd() - 0.5) * 0.045;
                biscuit.position().set(px, 0.0156, pz);
            }
            st.add(&biscuit);
        }
    }
    // 40% chance sugar bowl (small covered pot).
    if rnd() < 0.4 {
        let sugar_mat = lambert!({ color: tea_colors[ri(tea_colors.len())] });
        let sugar = mesh(pg("SilverTrays__sugar", cyl_geo!(0.018, 0.020, 0.024, 10)), &sugar_mat);
        sugar.position().set(-0.07, 0.0236, 0.03);
        st.add(&sugar);
        let lid = mesh(pg("SilverTrays__lid", sphere_geo!(0.020, 8, 5, 0, PI * 2.0, 0, PI * 0.4)), &sugar_mat);
        lid.position().set(-0.07, 0.0291, 0.03);
        st.add(&lid);
        let knob = mesh(pg("SilverTrays__knob", sphere_geo!(0.004, 5, 4)), &sugar_mat);
        knob.position().set(-0.07, 0.0527, 0.03);
        st.add(&knob);
    }
    // 25% chance small folded cloth napkin on tray.
    if rnd() < 0.25 {
        // 0xc0a08040 is a 32-bit value; Color::hex keeps only the low 24 bits (0xa08040), deliberately.
        let nap_colors: [u32; 3] = [0xf0f0e8, 0xe0e0d8, 0xc0a08040];
        let nap_mat = lambert!({ color: nap_colors[ri(nap_colors.len())] });
        let napkin = mesh(pg("SilverTrays__napkin", box_geo!(0.045, 0.005, 0.030)), &nap_mat);
        napkin.position().set(0.0, 0.0141, -0.095);
        napkin.rotation().y = (rnd() - 0.5) * 0.4;
        st.add(&napkin);
    }
    st.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    st.rotation().y = rnd() * PI * 2.0;
    g.world.add(&st);
}

// R751 open pocket watch on chain (floor): 4% chance per level.
fn place_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.pocket_watch_positions_r755.clear(); // R755 reset per level
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Metal case pool — gold / silver / rose gold / gunmetal.
    let case_mats = [
        lambert!({ color: 0xd4b060, emissive: 0x201810 }), // gold
        lambert!({ color: 0xd8d8e0, emissive: 0x161818 }), // silver
        lambert!({ color: 0xc08070, emissive: 0x201810 }), // rose gold
        lambert!({ color: 0x606068 }),                     // gunmetal
    ];
    let case_mat = case_mats[ri(case_mats.len())].clone();
    let pw = group();
    // Watch body — thin cylinder, axis vertical.
    let watch_r = 0.030;
    let watch_h = 0.010;
    let body = mesh(pg("PocketWatches__body", cyl_geo!(watch_r, watch_r, watch_h, 20)), &case_mat);
    body.position().set(0, watch_h / 2.0, 0);
    pw.add(&body);
    // Rim (torus).
    let rim = mesh(pg("PocketWatches__rim", torus_geo!(watch_r, 0.002, 5, 20)), &case_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().set(0, watch_h + 0.001, 0);
    pw.add(&rim);
    // Dial face — off-white porcelain.
    let dial_mat = lambert!({ color: 0xf0e8d0 });
    let dial = mesh(circle_geo!(watch_r - 0.003, 20), &dial_mat);
    dial.rotation().x = -PI / 2.0;
    dial.position().set(0, watch_h + 0.0015, 0);
    pw.add(&dial);
    // 12 hour tick marks around dial edge.
    let tick_mat = lambert!({ color: 0x1a1a10 });
    for i in 0..12 {
        let angle = (i as f64 / 12.0) * PI * 2.0;
        let is_major = i % 3 == 0;
        let tick = mesh(box_geo!(0.0012, 0.0006, if is_major { 0.006 } else { 0.004 }), &tick_mat);
        tick.position().set(angle.cos() * (watch_r - 0.006), watch_h + 0.002, angle.sin() * (watch_r - 0.006));
        tick.rotation().y = -angle - PI / 2.0; // radial, like the hands
        pw.add(&tick);
    }
    // Hour hand — frozen at a specific time.
    let hour_angle = rnd() * PI * 2.0;
    let hour_hand = mesh(pg("PocketWatches__hourHand", box_geo!(0.0018, 0.0008, 0.014)), &tick_mat);
    hour_hand.position().set(hour_angle.cos() * 0.007, watch_h + 0.0025, hour_angle.sin() * 0.007);
    hour_hand.rotation().y = -hour_angle - PI / 2.0;
    pw.add(&hour_hand);
    // Minute hand — long, thin.
    let min_angle = rnd() * PI * 2.0;
    let min_hand = mesh(pg("PocketWatches__minHand", box_geo!(0.0012, 0.0008, 0.020)), &tick_mat);
    min_hand.position().set(min_angle.cos() * 0.010, watch_h + 0.0030, min_angle.sin() * 0.010);
    min_hand.rotation().y = -min_angle - PI / 2.0;
    pw.add(&min_hand);
    // Center pin.
    let center_pin = mesh(pg("PocketWatches__pin", cyl_geo!(0.0015, 0.0015, 0.002, 6)), &case_mat);
    center_pin.position().set(0, watch_h + 0.004, 0);
    pw.add(&center_pin);
    // Winding crown on top edge.
    let crown = mesh(pg("PocketWatches__crown", cyl_geo!(0.0035, 0.0035, 0.005, 8)), &case_mat);
    crown.rotation().x = -PI / 2.0;
    crown.position().set(0, watch_h / 2.0 + 0.0002, -watch_r - 0.002);
    pw.add(&crown);
    // Hinged cover swung 150° open about a hinge on the case's back edge.
    let cover = mesh(pg("PocketWatches__cover", cyl_geo!(watch_r, watch_r, 0.003, 20)), &case_mat);
    cover.rotation().x = PI + (PI / 2.0 + PI / 3.0);
    cover.position().set(0, watch_h + 0.0015 + watch_r * 0.5, watch_r + watch_r * (PI / 6.0).cos());
    pw.add(&cover);
    // Trailing chain.
    let chain_mat = case_mat.clone();
    let chain_start_x = watch_r + 0.005;
    let chain_curve_end_x = chain_start_x + 0.10 + rnd() * 0.08;
    let chain_curve_end_z = -0.03 - rnd() * 0.04;
    let chain_len = 2 * 4i64.max(round_half_up(hypot2(chain_curve_end_x - chain_start_x, chain_curve_end_z) / 0.013) as i64);
    for i in 0..chain_len {
        let t01 = i as f64 / (chain_len - 1) as f64;
        let link = mesh(pg("PocketWatches__link", torus_geo!(0.004, 0.001, 4, 6)), &chain_mat);
        let lx = chain_start_x + t01 * (chain_curve_end_x - chain_start_x);
        let lz = t01 * chain_curve_end_z + (t01 * PI).sin() * 0.008;
        // Even links flat on the floor, odd ones on edge through them.
        link.position().set(lx, if i % 2 == 0 { 0.001 } else { 0.005 }, lz);
        link.set_rotation_order(EulerOrder::YXZ);
        link.rotation().y = (-(chain_curve_end_z + (t01 * PI).cos() * PI * 0.008)).atan2(chain_curve_end_x - chain_start_x);
        link.rotation().x = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
        link.rotation().z = if i % 2 == 0 { 0.0 } else { PI / 2.0 };
        pw.add(&link);
    }
    // End of chain: fob or key.
    let chain_end_roll = rnd();
    if chain_end_roll < 0.60 {
        let fob = mesh(pg("PocketWatches__fob", box_geo!(0.014, 0.004, 0.004)), &case_mat);
        fob.position().set(chain_curve_end_x + 0.007, 0.002, chain_curve_end_z);
        fob.rotation().y = -PI / 4.0;
        pw.add(&fob);
    } else if chain_end_roll < 0.90 {
        let key_shaft = mesh(pg("PocketWatches__keyShaft", cyl_geo!(0.0012, 0.0012, 0.014, 5)), &case_mat);
        key_shaft.rotation().z = PI / 2.0;
        key_shaft.position().set(chain_curve_end_x + 0.0185, 0.0012, chain_curve_end_z);
        pw.add(&key_shaft);
        let key_head = mesh(pg("PocketWatches__keyHead", torus_geo!(0.005, 0.001, 4, 8)), &case_mat);
        key_head.rotation().x = PI / 2.0;
        key_head.position().set(chain_curve_end_x + 0.0065, 0.001, chain_curve_end_z);
        pw.add(&key_head);
        let key_bit = mesh(pg("PocketWatches__keyBit", box_geo!(0.0025, 0.002, 0.0025)), &case_mat);
        key_bit.position().set(chain_curve_end_x + 0.0265, 0.001, chain_curve_end_z);
        pw.add(&key_bit);
    }
    // 30% chance: initials engraved on cover.
    if rnd() < 0.3 {
        let eng_mat = lambert!({ color: 0x603818 });
        for i in 0..2 {
            let eng = mesh(box_geo!(0.006, 0.0005, 0.001), &eng_mat);
            let cover_y = watch_h + 0.0015 + watch_r * 0.5;
            let cover_z = watch_r + watch_r * (PI / 6.0).cos();
            eng.position().set(-0.006 + i as f64 * 0.008, cover_y + 0.0011 * (PI / 6.0).cos(), cover_z - 0.0011 * 0.5);
            eng.rotation().x = -PI / 6.0;
            pw.add(&eng);
        }
    }
    pw.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pw);
    let p = pw.pos();
    g.pp.pocket_watch_positions_r755.push(v3(p.x, 0, p.z)); // R755
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
