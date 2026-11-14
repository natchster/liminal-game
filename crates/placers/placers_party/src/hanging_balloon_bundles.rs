//! `place_hanging_balloon_bundles`: Hanging balloon bundle: 6% chance per level. 3-4 balloons on independent strings converging at ceiling anchor point.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Hanging balloon bundle: 6% chance per level. 3-4 balloons
/// on independent strings converging at ceiling anchor point.
pub(crate) fn place_hanging_balloon_bundles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bal_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let string_mat = lambert!({ color: 0xf0e8d8 });
    let bundle = group();
    let n_balloons = 3 + (rnd() * 2.0).floor() as i32;
    let anchor_y = WALL_H - 0.05;
    for i in 0..n_balloons {
        let angle = (i as f64 / n_balloons as f64) * PI * 2.0 + rnd() * 0.4;
        let spread = 0.15 + rnd() * 0.08;
        let balloon_y = WALL_H - (0.4 + rnd() * 0.4);
        let bx = angle.dcos() * spread;
        let bz = angle.dsin() * spread;
        // String from anchor to balloon tie (the knot on top of the neck-up balloon).
        let dx = bx - 0.0;
        let dy = balloon_y + 0.105 - anchor_y;
        let dz = bz - 0.0;
        let str_len = (dx * dx + dy * dy + dz * dz).sqrt();
        let strg = mesh(pg("HangingBalloonBundles__string", cyl_geo!(0.002, 0.002, str_len, 3)), &string_mat);
        strg.position().set(bx / 2.0, (anchor_y + balloon_y + 0.105) / 2.0, bz / 2.0);
        // Rotate cylinder from Y-up to string direction.
        let mut str_dir = v3(dx, dy, dz);
        str_dir.normalize();
        let y_up = v3(0, 1, 0);
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&y_up, &str_dir);
        strg.quaternion().copy(&q);
        bundle.add(&strg);
        // Balloon body.
        let bal_mat = lambert!({ color: pick(&bal_colors) });
        let bal = mesh(pg("HangingBalloonBundles__balloon", sphere_geo!(0.09, 10, 8)), &bal_mat);
        bal.scale().y = 1.15;
        bal.rotation().z = PI; // neck up, hanging from the knot
        bal.position().set(bx, balloon_y, bz);
        bundle.add(&bal);
        // Balloon tie (small cone), on top of the balloon's neck.
        let tie = mesh(pg("HangingBalloonBundles__tie", cone_geo!(0.008, 0.014, 5)), &bal_mat);
        tie.position().set(bx, balloon_y + 0.100, bz);
        bundle.add(&tie);
    }
    bundle.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0, g.wz(z) + (rnd() - 0.5) * 1.5);
    bundle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bundle);
    // Very subtle whole-bundle sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 8.0;
        let base_rot_z = bundle.rot().z;
        let b = bundle.clone();
        g.animate(move |_g, _dt, t| {
            b.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).dsin() * 0.02;
        });
    }
}
