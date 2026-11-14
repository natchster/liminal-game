//! `place_wingback_armchairs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Wingback armchair (floor): 2% chance per level.
pub(crate) fn place_wingback_armchairs(g: &mut Game, _dist: &[i32]) {
    g.pp.wingback_armchair_positions.clear(); // reset per level
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
        // Hanging subgroup — pivots at top for pendulum sway.
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
        // Breeze-sway animator — very slight pendulum motion.
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
    g.pp.wingback_armchair_positions.push(v3(p.x, 0, p.z));
}
