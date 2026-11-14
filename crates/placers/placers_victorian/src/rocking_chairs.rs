//! `place_rocking_chairs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Rocking chair: 4% chance per level. Empty rocking chair, slow rock animator.
pub(crate) fn place_rocking_chairs(g: &mut Game, _dist: &[i32]) {
    g.pp.chair_positions.clear(); // reset per level
    g.pp.chair_periods.clear();
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x603820, 0x8a5030, 0x201a10];
    let wood_mat = lambert!({ color: pick_hex(&wood_colors) });
    let chair = group();
    // Seat plank.
    let seat = mesh(pg("RockingChairs__seat", box_geo!(0.30, 0.020, 0.24)), &wood_mat);
    seat.position().y = 0.24;
    chair.add(&seat);
    // 4 legs.
    for (lx, lz) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let leg = mesh(pg("RockingChairs__leg", cyl_geo!(0.010, 0.010, 0.22, 5)), &wood_mat);
        leg.position().set(lx as f64 * 0.12, 0.12, lz as f64 * 0.10);
        if lz == 1 {
            // Front legs run on up through the seat as the arm posts (to y 0.344).
            leg.scale().y = 0.334 / 0.22;
            leg.position().y = 0.177;
        }
        chair.add(&leg);
    }
    // Back rest bars (5 vertical bars).
    let back_rail = mesh(pg("RockingChairs__rail", box_geo!(0.30, 0.020, 0.014)), &wood_mat);
    back_rail.position().set(0, 0.44, -0.11);
    chair.add(&back_rail);
    for i in 0..5 {
        let rx = -0.12 + i as f64 * 0.06;
        let bar = mesh(pg("RockingChairs__spindle", cyl_geo!(0.006, 0.006, 0.20, 4)), &wood_mat);
        bar.position().set(rx, 0.35, -0.11);
        chair.add(&bar);
    }
    // 2 rocker rails, front to back under each pair of legs, level (the modelled rocker is curved).
    for s in [-1.0, 1.0] {
        let rocker = mesh(pg("RockingChairs__rocker", box_geo!(0.36, 0.018, 0.030)), &wood_mat);
        rocker.position().set(s * 0.12, 0.009, 0);
        rocker.rotation().y = PI / 2.0;
        chair.add(&rocker);
    }
    // 2 armrests, front to back at each side.
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("RockingChairs__arm", cyl_geo!(0.008, 0.008, 0.20, 4)), &wood_mat);
        arm.rotation().x = PI / 2.0;
        arm.position().set(s * 0.12, 0.34, -0.005);
        chair.add(&arm);
    }
    // Slow rock animator (very subtle unsettling motion).
    let rock_phase = rnd() * PI * 2.0;
    let rock_period = 5.0 + rnd() * 3.0;
    {
        let chair = chair.clone();
        g.animate(move |_g, _dt, t| {
            chair.rotation().x = (t * (PI * 2.0 / rock_period) + rock_phase).sin() * 0.04;
        });
    }
    scatter_on_floor(g, &chair, x, z, 1.8);
    // Yaw first, so the animator's rotation.x rocks the chair along its own rockers.
    chair.set_rotation_order(EulerOrder::YXZ);
    g.world.add(&chair);
    // Track for creak audio (store period for sync: the ambient creak timer
    // reads it from the parallel `chair_periods`).
    let p = chair.pos();
    g.pp.chair_positions.push(v3(p.x, 0.30, p.z));
    g.pp.chair_periods.push(rock_period);
}
