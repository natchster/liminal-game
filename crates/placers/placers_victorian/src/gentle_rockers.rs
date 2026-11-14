//! `place_gentle_rockers`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Rocking chair w/ optional gentle rocking animator (floor): 3% chance per level.
pub(crate) fn place_gentle_rockers(g: &mut Game, _dist: &[i32]) {
    g.pp.rocking_chair_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    // Optional cushion color.
    let cushion_colors = [0xa02040, 0x408060, 0xd0a040, 0xf0e0c0, 0x2060a0, 0x804060];
    let cushion_mat = lambert!({ color: choice(&cushion_colors) });
    // Whole chair goes into a group that we rotate for the rocking animation.
    let rock_group = group();
    let chair = group();
    // Seat — flat rectangular box.
    let (seat_w, seat_d, seat_h) = (0.36, 0.32, 0.036);
    let seat = mesh(pg("GentleRockers__seat", box_geo!(seat_w, seat_h, seat_d)), &wood_mat);
    seat.position().y = 0.30;
    chair.add(&seat);
    // 4 legs — thin cylinders at corners.
    for cx in [-seat_w * 0.42, seat_w * 0.42] {
        for cz in [-seat_d * 0.42, seat_d * 0.42] {
            let leg = mesh(pg("GentleRockers__leg", cyl_geo!(0.010, 0.010, 0.28, 5)), &wood_mat);
            leg.scale().y = 0.94; // seat underside down to the runner's top (y ~0.02)
            leg.position().set(cx, 0.1525, cz);
            chair.add(&leg);
        }
    }
    // Back — tall rectangular panel behind seat.
    let (back_w, back_h) = (seat_w, 0.42);
    let back = mesh(pg("GentleRockers__back", box_geo!(back_w, back_h, 0.020)), &wood_mat);
    back.position().set(0, 0.30 + back_h / 2.0, -seat_d * 0.45);
    chair.add(&back);
    // 5 vertical back slats for classic rocker look.
    let slat_w = 0.010;
    for i in 0..5 {
        let sx = -seat_w * 0.35 + i as f64 * (seat_w * 0.7 / 4.0);
        let slat = mesh(pg("GentleRockers__slat", cyl_geo!(0.006, 0.006, back_h - 0.020, 5)), &wood_mat);
        slat.position().set(sx, 0.30 + back_h / 2.0, -seat_d * 0.45 + 0.005);
        chair.add(&slat);
    }
    // Top rail on back.
    let top_rail = mesh(pg("GentleRockers__topRail", box_geo!(seat_w + 0.020, 0.024, 0.020)), &wood_mat);
    top_rail.position().set(0, 0.30 + back_h + 0.010, -seat_d * 0.45);
    chair.add(&top_rail);
    // 2 armrests — horizontal boxes at seat height.
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("GentleRockers__arm", box_geo!(0.024, 0.020, seat_d * 0.7)), &wood_mat);
        arm.position().set(side * seat_w * 0.48, 0.30 + 0.10, -seat_d * 0.05);
        chair.add(&arm);
        // Vertical support for armrest (short cylinder).
        let arm_support = mesh(pg("GentleRockers__armSupport", cyl_geo!(0.008, 0.008, 0.11, 5)), &wood_mat);
        arm_support.position().set(side * seat_w * 0.48, 0.30 + 0.056, seat_d * 0.28);
        chair.add(&arm_support);
    }
    // 2 curved rocker runners — partial torus segments.
    for side in [-1.0f64, 1.0] {
        let runner = mesh(pg("GentleRockers__runner", torus_geo!(0.30, 0.008, 4, 12, PI * 0.75)), &wood_mat);
        // Arc centred on its lowest point, so it runs under the chair from
        // front to back; the rig flattens it into a long low rocker.
        runner.rotation().z = -PI * 0.875;
        runner.rotation().y = PI / 2.0;
        // Position runner so bottom of arc rests on floor plane.
        runner.position().set(0, 0.308, 0);
        let runner_rig = group();
        runner_rig.scale().set(1, 0.6, 1.15);
        runner_rig.position().set(side * seat_w * 0.42, 0, 0);
        runner_rig.add(&runner);
        chair.add(&runner_rig);
    }
    // 55% chance: cushion on seat.
    if rnd() < 0.55 {
        let cushion = mesh(pg("GentleRockers__cushion", box_geo!(seat_w - 0.020, 0.032, seat_d - 0.020)), &cushion_mat);
        cushion.position().set(0, 0.30 + seat_h / 2.0 + 0.016, 0);
        chair.add(&cushion);
        // 30% chance: back cushion too.
        if rnd() < 0.3 {
            let back_cushion = mesh(pg("GentleRockers__backCushion", box_geo!(seat_w - 0.030, back_h * 0.4, 0.024)), &cushion_mat);
            back_cushion.position().set(0, 0.30 + back_h * 0.30, -seat_d * 0.45 + 0.023); // in front of the spindles
            chair.add(&back_cushion);
        }
    }
    rock_group.add(&chair);
    // 50% chance: gentle rocking animator — very slow, subtle.
    let is_rocking = rnd() < 0.5;
    if is_rocking {
        let rock_phase = rnd() * PI * 2.0;
        let rg2 = rock_group.clone();
        g.animate(move |_g, _dt, t| {
            // Very slow gentle rocking — 0.7 Hz, ±3° amplitude.
            rg2.rotation().x = (t * 0.7 + rock_phase).sin() * 0.05;
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    rock_group.position().set(px, 0, pz);
    rock_group.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rock_group);
    // Register rocking chair positions for wood-creak audio.
    if is_rocking {
        let p = rock_group.pos();
        g.pp.rocking_chair_positions.push(v3(p.x, 0, p.z));
    }
}
