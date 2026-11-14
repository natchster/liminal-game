//! `place_worn_teddy_bears`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;

// Worn teddy bear: 6% chance per level. Old teddy with a missing eye.
pub(crate) fn place_worn_teddy_bears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let fur_colors = [0x8a5030, 0x604020, 0x503020];
    let fur_mat = lambert!({ color: pick_hex(&fur_colors) });
    let eye_mat = lambert!({ color: 0x181008 });
    let nose_mat = lambert!({ color: 0x2a1810 });
    let bear = group();
    // Body sphere.
    let body = mesh(pg("WornTeddyBears__body", sphere_geo!(0.045, 12, 10)), &fur_mat);
    body.scale().set(1, 1.2, 0.9);
    body.position().y = 0.045;
    bear.add(&body);
    // Head sphere.
    let head = mesh(pg("WornTeddyBears__head", sphere_geo!(0.035, 12, 10)), &fur_mat);
    head.position().y = 0.100;
    bear.add(&head);
    // Snout.
    let snout = mesh(pg("WornTeddyBears__muzzle", sphere_geo!(0.018, 8, 6)), &fur_mat);
    snout.scale().set(1, 0.8, 1);
    snout.position().set(0, 0.092, 0.028);
    bear.add(&snout);
    // Nose (dark).
    let nose = mesh(pg("WornTeddyBears__nose", sphere_geo!(0.005, 6, 5)), &nose_mat);
    nose.position().set(0, 0.094, 0.044);
    bear.add(&nose);
    // 2 ears (small semispheres), tilted out so the flat base sits on the head.
    for s in [-1.0, 1.0] {
        let ear = mesh(pg("WornTeddyBears__ear", sphere_geo!(0.012, 8, 6, 0, PI * 2.0, 0, PI / 2.0)), &fur_mat);
        ear.position().set(s * 0.032 * (0.6f64).sin(), 0.100 + 0.032 * (0.6f64).cos(), 0);
        ear.rotation().z = -s * 0.6;
        bear.add(&ear);
    }
    // Present eye (still there).
    let eye1 = mesh(pg("WornTeddyBears__eye", sphere_geo!(0.004, 5, 4)), &eye_mat);
    eye1.position().set(0.010, 0.108, 0.030);
    bear.add(&eye1);
    // Missing eye: dark socket hole.
    let socket_mat = basic_mat!({ color: 0x080604 });
    let socket = mesh(circle_geo!(0.005, 6), &socket_mat);
    socket.position().set(-0.010, 0.108, 0.032);
    socket.look_at(-0.010, 0.108, 0.10);
    bear.add(&socket);
    // 4 short limbs (arms + legs).
    for (lx, ly, lz) in [(-0.035, 0.045, 0.0), (0.035, 0.045, 0.0), (-0.020, 0.020, 0.010), (0.020, 0.020, 0.010)] {
        let limb = mesh(pg("WornTeddyBears__limb", sphere_geo!(0.014, 8, 6)), &fur_mat);
        limb.scale().set(1, 1.3, 1);
        limb.position().set(lx, ly, lz);
        bear.add(&limb);
    }
    // 30% chance tipped.
    if rnd() < 0.30 {
        bear.rotation().z = PI / 2.0;
        bear.position().y = 0.035;
    }
    bear.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bear.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bear.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bear);
}
