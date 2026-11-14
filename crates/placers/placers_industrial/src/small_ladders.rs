//! `place_small_ladders`
use game::prelude::*;
use placers_kit::place::ri;
use placers_kit::place::wall_candidates;

// Small wooden ladder: 12% chance per level. 2 vertical rails
// + 4 horizontal rungs, leaning against wall.
pub(crate) fn place_small_ladders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    let wood_mat = lambert!({ color: 0x6a4020 });
    let ladder = group();
    let r_h = 1.6; // total ladder height
    // Rails: 2 vertical cylinders 0.35 m apart.
    for rx_off in [-0.175, 0.175] {
        let rail = mesh(pg("SmallLadders__rail", cyl_geo!(0.012, 0.012, r_h, 6)), &wood_mat);
        rail.position().set(rx_off, r_h / 2.0, 0);
        ladder.add(&rail);
    }
    // 4 rungs horizontal.
    for rung in 0..4 {
        let r_y = 0.3 + rung as f64 * 0.35;
        let rung_mesh = mesh(pg("SmallLadders__rung", cyl_geo!(0.010, 0.010, 0.35, 6)), &wood_mat);
        rung_mesh.rotation().z = PI / 2.0;
        rung_mesh.position().set(0, r_y, 0);
        ladder.add(&rung_mesh);
    }
    // Lean ~12° toward wall: lean in the ladder's own frame, then yaw, and
    // stand the feet far enough out that the rail tops just touch the wall.
    ladder.set_rotation_order(EulerOrder::YXZ);
    ladder.rotation().x = -0.20 - rnd() * 0.10;
    let foot_out = r_h * (-ladder.rot().x).sin() + 0.012;
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - foot_out);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - foot_out);
    if c.dx.abs() == 1 {
        ladder.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        ladder.position().set(wall_x, 0, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        ladder.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        ladder.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&ladder);
    // 25% chance: small paint bucket at ladder base.
    if rnd() < 0.25 {
        let bucket_colors = [0x8a8880, 0x707070, 0xa08040];
        let paint_bucket_mat = lambert!({ color: bucket_colors[ri(bucket_colors.len())] });
        let paint_handle_mat = lambert!({ color: 0x2a2018 });
        let paint_bucket = group();
        // Body: shallow tapered cylinder.
        let pb_body = mesh(pg("SmallLadders__pail", cyl_geo!(0.08, 0.07, 0.12, 10)), &paint_bucket_mat);
        pb_body.position().y = 0.06;
        paint_bucket.add(&pb_body);
        // Rim.
        let pb_rim = mesh(pg("SmallLadders__pailRim", torus_geo!(0.08, 0.004, 4, 10)), &paint_bucket_mat);
        pb_rim.rotation().x = PI / 2.0;
        pb_rim.position().y = 0.12;
        paint_bucket.add(&pb_rim);
        // Wire handle: half-torus above.
        let pb_handle = mesh(pg("SmallLadders__pailBail", torus_geo!(0.075, 0.003, 4, 8, PI)), &paint_handle_mat);
        pb_handle.rotation().z = 0.0; // bail over the top, ends on the rim
        pb_handle.position().y = 0.12;
        paint_bucket.add(&pb_handle);
        // Position at ladder base with slight forward offset.
        let pfx = -c.dx as f64 * 0.20;
        let pfz = -c.dz as f64 * 0.20;
        let lp = ladder.pos();
        paint_bucket.position().set(lp.x + pfx + (rnd() - 0.5) * 0.15, 0, lp.z + pfz + (rnd() - 0.5) * 0.15);
        paint_bucket.rotation().y = rnd() * PI * 2.0;
        g.world.add(&paint_bucket);
        // 25% chance: colored paint spill stain 0.2-0.4 m from bucket.
        if rnd() < 0.25 {
            let paint_colors = [0xe0e0e0, 0x505050, 0x8a4028, 0xe0d0a0]; // white, gray, rust, tan
            let spill_mat =
                basic_mat!({ color: paint_colors[ri(paint_colors.len())], transparent: true, opacity: 0.75, depthWrite: false });
            let spill = mesh(circle_geo!(0.08 + rnd() * 0.06, 12), &spill_mat);
            spill.rotation().x = -PI / 2.0;
            spill.rotation().z = rnd() * PI * 2.0;
            spill.scale().set(1.4, 0.85, 1);
            let sp_ang = rnd() * PI * 2.0;
            let sp_rad = 0.20 + rnd() * 0.20;
            let pp = paint_bucket.pos();
            spill.position().set(pp.x + sp_ang.cos() * sp_rad, 0.004, pp.z + sp_ang.sin() * sp_rad);
            g.world.add(&spill);
        }
    }
}
