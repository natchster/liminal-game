//! `place_mops`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Mops: 30% chance per level, 1 mop leaning against a wall.
pub(crate) fn place_mops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let stick_mat = lambert!({ color: 0x6a4a28 });
    let head_mat = lambert!({ color: 0x8a8878 });
    let mop = group();
    let stick = mesh(pg("Mops__stick", cyl_geo!(0.015, 0.015, 1.4, 6)), &stick_mat);
    stick.position().y = 0.70;
    mop.add(&stick);
    let head = mesh(pg("Mops__head", cyl_geo!(0.08, 0.06, 0.14, 8)), &head_mat);
    head.position().y = 0.07;
    mop.add(&head);
    // Lean stick ~10° toward wall (subtle).
    mop.rotation().z = -PI / 18.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.15);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.15);
    if c.dx.abs() == 1 {
        mop.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mop.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        mop.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mop.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&mop);
    // 40% chance mop bucket beside — janitorial co-location.
    if rnd() < 0.40 {
        let bucket_mat = lambert!({ color: 0xd8b820 });
        let bucket = group();
        let body = mesh(pg("Mops__bucket", cyl_geo!(0.14, 0.11, 0.22, 12)), &bucket_mat);
        body.position().y = 0.11;
        bucket.add(&body);
        let rim = mesh(pg("Mops__rim", torus_geo!(0.14, 0.006, 4, 12)), &bucket_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().y = 0.22;
        bucket.add(&rim);
        // Wire handle (thin arch — 2 small cylinders).
        let handle_mat = lambert!({ color: 0x3a3830 });
        for hs in [-1.0, 1.0] {
            let h_strand = mesh(pg("Mops__wire", cyl_geo!(0.003, 0.003, 0.10, 4)), &handle_mat);
            h_strand.position().set(hs * 0.12, 0.28, 0);
            bucket.add(&h_strand);
        }
        let handle_top = mesh(pg("Mops__handleTop", cyl_geo!(0.003, 0.003, 0.24, 4)), &handle_mat);
        handle_top.rotation().z = PI / 2.0;
        handle_top.position().y = 0.33;
        bucket.add(&handle_top);
        // 30% chance bucket contains dark water surface.
        if rnd() < 0.30 {
            let water_mat = basic_mat!({ color: 0x1a1a20, transparent: true, opacity: 0.75, depthWrite: false });
            let water = mesh(circle_geo!(0.12, 12), &water_mat);
            water.rotation().x = -PI / 2.0;
            water.position().y = 0.18; // ~80% full
            bucket.add(&water);
        }
        // Position 0.3-0.6 m from mop.
        let dist = 0.3 + rnd() * 0.3;
        let ang = rnd() * PI * 2.0;
        let mp = mop.pos();
        bucket.position().set(mp.x + ang.cos() * dist, 0, mp.z + ang.sin() * dist);
        g.world.add(&bucket);
    }
}
