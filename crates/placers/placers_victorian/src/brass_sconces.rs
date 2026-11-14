//! `place_brass_sconces`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Wall-mounted brass sconce w/ candle (wall): 4% chance per level.
pub(crate) fn place_brass_sconces(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let candle_colors = [0xf0e8c8, 0xd0d0d0, 0xe8d0c0];
    let candle_mat = lambert!({ color: choice(&candle_colors) });
    let sc = group();
    // Wall mounting plate.
    let mount_plate = mesh(pg("BrassSconces__plate", box_geo!(0.045, 0.070, 0.006)), &brass_mat);
    mount_plate.position().set(0, 0, 0.003);
    sc.add(&mount_plate);
    // 2 small mounting screws.
    for side in [-1.0, 1.0] {
        let screw = mesh(pg("BrassSconces__screw", cyl_geo!(0.002, 0.002, 0.003, 4)), &brass_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(0, side * 0.028, 0.006);
        sc.add(&screw);
    }
    // Ornate scrolled arm: plate boss -> knop -> under the drip cup.
    let arm_a = v3(0, -0.004, 0.003);
    let arm_k = v3(0.030, 0.015, 0.062);
    let arm_c = v3(0.056, 0.078, 0.090);
    let arm_seg1 = mesh(pg("BrassSconces__arm", cyl_geo!(0.006, 0.006, 0.045, 5)), &brass_mat);
    arm_span(&arm_seg1, &arm_a, &arm_k, 0.045);
    sc.add(&arm_seg1);
    // Small mid decorative sphere.
    let arm_mid = mesh(pg("BrassSconces__knop", sphere_geo!(0.010, 6, 5)), &brass_mat);
    arm_mid.position().copy(&arm_k);
    sc.add(&arm_mid);
    // Upper arm segment.
    let arm_seg2 = mesh(pg("BrassSconces__arm", cyl_geo!(0.005, 0.005, 0.045, 5)), &brass_mat);
    arm_span(&arm_seg2, &arm_k, &arm_c, 0.045);
    sc.add(&arm_seg2);
    // Drip cup.
    let drip_cup = mesh(pg("BrassSconces__dripCup", cyl_geo!(0.025, 0.020, 0.010, 12)), &brass_mat);
    drip_cup.position().set(0.056, 0.080, 0.090);
    sc.add(&drip_cup);
    let drip_rim = mesh(pg("BrassSconces__dripRim", torus_geo!(0.023, 0.002, 3, 12)), &brass_mat);
    drip_rim.rotation().x = PI / 2.0;
    drip_rim.position().set(0.056, 0.086, 0.090); // on the cup's lip
    sc.add(&drip_rim);
    // Candle sitting in drip cup (foot 2 mm down in the socket).
    let candle_h = 0.10 + rnd() * 0.04;
    let candle = mesh(pg("BrassSconces__candle", cyl_geo!(0.012, 0.013, candle_h, 8)), &candle_mat);
    candle.position().set(0.056, 0.083 + candle_h / 2.0, 0.090);
    sc.add(&candle);
    // Wick.
    let wick = mesh(pg("BrassSconces__wick", cyl_geo!(0.0008, 0.0008, 0.004, 3)), lambert!({ color: 0x101010 }));
    wick.position().set(0.056, 0.083 + candle_h + 0.002, 0.090);
    sc.add(&wick);
    // 55%: candle is lit — flame + tiny PointLight + flicker animator.
    if rnd() < 0.55 {
        let flame_mat = basic_mat!({ color: 0xffa040 });
        let flame = mesh(sphere_geo!(0.006, 6, 5), &flame_mat);
        flame.scale().y = 2.0;
        flame.position().set(0.056, 0.083 + candle_h + 0.010, 0.090);
        sc.add(&flame);
        let flame_light = point_light!(0xffa040, 0.30, 1.2);
        flame_light.position().set(0.056, 0.083 + candle_h + 0.012, 0.090);
        sc.add(&flame_light);
        let phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let flick = 0.85 + (t * 7.0 + phase).sin() * 0.11 + (t * 11.0 + phase * 1.3).sin() * 0.06;
            flame_light.set_intensity(0.30 * flick);
            flame.scale().set(flick, 2.0 * flick, flick);
        });
        // Small wax drip melting down.
        if rnd() < 0.5 {
            let wax = mesh(pg("BrassSconces__wax", cyl_geo!(0.001, 0.002, 0.014, 4)), &candle_mat);
            wax.position().set(0.078, 0.078, 0.090);
            sc.add(&wax);
        }
    }
    // 30%: small chain hanging from the knop.
    if rnd() < 0.3 {
        for i in 0..3 {
            let chain_link = mesh(pg("BrassSconces__link", torus_geo!(0.004, 0.001, 3, 6)), &brass_mat);
            // Upright links, each turned 90° to the last.
            chain_link.rotation().y = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
            chain_link.position().set(arm_k.x, arm_k.y - 0.012 - i as f64 * 0.006, arm_k.z);
            sc.add(&chain_link);
        }
    }
    mount_on_wall(g, &sc, &c, 0.003, 1.55, 0.5);
    g.world.add(&sc);
}

/// Stretch a Y-axis segment between two points (`arm_span`).
fn arm_span(m: &Obj, a: &Vec3, b: &Vec3, len: f64) {
    {
        let mut p = m.position();
        p.copy(a);
        p.add(b);
        p.multiply_scalar(0.5);
    }
    let mut d = *b;
    d.sub(a);
    d.normalize();
    m.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
    m.scale().y = a.distance_to(b) / len;
}
