//! `place_broken_whole_dolls`
use game::prelude::*;
use placers_kit::geometry::set_rot_order;
use placers_kit::place::open_tiles;

// Whole broken porcelain doll on floor (floor): 3% chance per level.
pub(crate) fn place_broken_whole_dolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Porcelain — aged white w/ slight yellow tint.
    let porcelain_colors = [0xe8e0d0, 0xd8d0c0, 0xf0e8d8];
    let porcelain_mat = lambert!({ color: choice(&porcelain_colors) });
    // Dress fabric — 5-colour pool (tea-stained pastels).
    let dress_colors = [0xa06060, 0x6080a0, 0xa0a060, 0x808040, 0x8060a0];
    let dress_mat = lambert!({ color: choice(&dress_colors) });
    let hair_colors = [0xd0a060, 0x604020, 0x8b5a2b, 0x2a1810];
    let hair_mat = lambert!({ color: choice(&hair_colors) });
    let crack_mat = lambert!({ color: 0x2a1810 });
    let bd = group();
    // Body — small oval torso, laid flat.
    let body = mesh(pg("BrokenWholeDolls__torso", sphere_geo!(0.05, 10, 7)), &porcelain_mat);
    body.scale().set(0.85, 1.6, 0.85);
    body.rotation().z = PI / 2.0; // Lay horizontal.
    body.position().set(0, 0.045, 0);
    bd.add(&body);
    // Dress covering torso: narrow bodice toward the head, wide hem toward
    // the legs; flattened so the hem rests on the floor.
    let dress = mesh(pg("BrokenWholeDolls__dress", cyl_geo!(0.055, 0.070, 0.14, 10)), &dress_mat);
    dress.rotation().z = -PI / 2.0;
    dress.scale().x = 0.71;
    dress.position().set(-0.02, 0.0497, 0);
    bd.add(&dress);
    // Dress fringe fold at hem (3 tiny box bumps).
    for i in 0..3 {
        let fringe = mesh(pg("BrokenWholeDolls__ruffle", box_geo!(0.008, 0.015, 0.015)), &dress_mat);
        fringe.position().set(-0.093, 0.045 - 0.02, -0.020 + i as f64 * 0.020); // on the hem's end
        bd.add(&fringe);
    }
    // Head — sphere on top-end of body.
    let head = mesh(pg("BrokenWholeDolls__head", sphere_geo!(0.038, 10, 7)), &porcelain_mat);
    head.position().set(0.080, 0.038, 0);
    bd.add(&head);
    // Hair on top of head (curled ringlets).
    for i in 0..5 {
        let fi = i as f64;
        let curl = mesh(pg("BrokenWholeDolls__curl", sphere_geo!(0.010, 6, 5)), &hair_mat);
        curl.position().set(0.080 + (fi * 1.2).cos() * 0.020, 0.070 + (fi * 0.7).sin() * 0.008, (fi * 1.2).sin() * 0.020);
        bd.add(&curl);
    }
    // 2 painted eye dots, in the eye sockets of the face (+X).
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.002, 5, 4)), lambert!({ color: 0x1a1030 }));
        eye.position().set(0.1091, 0.0409, side * 0.0122);
        bd.add(&eye);
    }
    // Painted mouth — tiny dark bar across the face on the lips.
    let mouth = mesh(box_geo!(0.006, 0.001, 0.001), lambert!({ color: 0x8b1010 }));
    mouth.rotation().y = PI / 2.0;
    mouth.position().set(0.1123, 0.0264, 0);
    bd.add(&mouth);
    // Rosy cheek dots, on the cheeks and facing out of them.
    for side in [-1.0, 1.0] {
        let cheek = mesh(circle_geo!(0.005, 6), lambert!({ color: 0xd08080 }));
        set_rot_order(&cheek, 0.3026, 0.844f64.atan2(side * 0.447), 0.0, EulerOrder::YXZ);
        cheek.position().set(0.1110, 0.0271, side * 0.0164);
        bd.add(&cheek);
    }
    // Legs — 2 cylinders (one might be missing).
    // 0 = left leg, 1 = right leg, 2 = left arm, 3 = right arm
    let missing_limb: i32 = if rnd() < 0.4 { (rnd() * 4.0).floor() as i32 } else { -1 };
    for i in 0..2 {
        if missing_limb == i {
            continue;
        }
        let leg = mesh(pg("BrokenWholeDolls__leg", cyl_geo!(0.014, 0.014, 0.075, 6)), &porcelain_mat);
        leg.rotation().z = PI / 2.0;
        leg.position().set(-0.11, 0.014, -0.020 + i as f64 * 0.040); // out of the hem, lying on the floor
        bd.add(&leg);
        // Tiny shoe at end.
        let shoe = mesh(pg("BrokenWholeDolls__shoe", sphere_geo!(0.010, 6, 5)), lambert!({ color: 0x2a1810 }));
        shoe.scale().set(1.0, 0.6, 1.0);
        shoe.position().set(-0.15, 0.006, -0.020 + i as f64 * 0.040); // on the floor round the ankle
        bd.add(&shoe);
    }
    // Arms — from the shoulder inside the side of the dress down and out to
    // the hand on the floor.
    let arm_dir = |i: i32| -> Vec3 {
        let a = if i == 0 { 0.25f64 } else { 0.6f64 };
        let mut v = v3(-0.912 * a.cos(), -0.411, (if i == 0 { -1.0 } else { 1.0 }) * 0.912 * a.sin());
        v.normalize();
        v
    };
    let arm_shoulder = |i: i32| -> Vec3 { v3(0.035, 0.040, (if i == 0 { -1.0 } else { 1.0 }) * 0.042) };
    for i in 0..2 {
        if missing_limb == i + 2 {
            continue;
        }
        let arm = mesh(pg("BrokenWholeDolls__arm", cyl_geo!(0.010, 0.010, 0.065, 6)), &porcelain_mat);
        let u = arm_dir(i);
        arm.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &u); // wrist (+Y) outward
        place_along(&arm, &arm_shoulder(i), &u, 0.0325);
        bd.add(&arm);
        // Tiny hand sphere, on the wrist, resting on the floor.
        let hand = mesh(pg("BrokenWholeDolls__hand", sphere_geo!(0.010, 6, 5)), &porcelain_mat);
        place_along(&hand, &arm_shoulder(i), &u, 0.073);
        bd.add(&hand);
    }
    // If a limb is missing, place it detached nearby.
    if missing_limb >= 0 {
        let (detached_geo, is_leg) = if missing_limb < 2 {
            (pg("BrokenWholeDolls__leg", cyl_geo!(0.014, 0.014, 0.075, 6)), true)
        } else {
            (pg("BrokenWholeDolls__arm", cyl_geo!(0.010, 0.010, 0.065, 6)), false)
        };
        let detached = mesh(detached_geo, &porcelain_mat);
        let det_angle = rnd() * PI * 2.0;
        let dx = det_angle.cos() * (0.15 + rnd() * 0.10);
        let dz = det_angle.sin() * (0.15 + rnd() * 0.10);
        detached.position().set(dx, if is_leg { 0.014 } else { 0.010 }, dz);
        // Lying flat on the floor; the old tilt draw now only adds to its heading.
        let det_turn = rnd() * 0.8;
        detached.rotation().z = PI / 2.0;
        detached.rotation().y = rnd() * PI + det_turn;
        bd.add(&detached);
        // Dark socket where the limb was.
        let socket = mesh(circle_geo!(0.010, 6), &crack_mat);
        socket.rotation().y = -PI / 2.0;
        if missing_limb == 0 {
            socket.position().set(-0.0915, 0.014, -0.020);
        } else if missing_limb == 1 {
            socket.position().set(-0.0915, 0.014, 0.020);
        } else {
            let ai = missing_limb - 2;
            let u = arm_dir(ai);
            place_along(&socket, &arm_shoulder(ai), &u, if ai == 0 { 0.036 } else { 0.022 });
            set_rot_order(&socket, -u.y.asin(), u.x.atan2(u.z), 0.0, EulerOrder::YXZ);
        }
        bd.add(&socket);
    }
    // 3-4 visible cracks on head/body.
    let crack_count = 3 + (rnd() * 2.0).floor() as i32;
    for _i in 0..crack_count {
        let crack = mesh(box_geo!(0.030 + rnd() * 0.020, 0.001, 0.001), &crack_mat);
        // Random position on head or body.
        let on_head = rnd() < 0.6;
        if on_head {
            let cx = 0.090 + (rnd() - 0.5) * 0.030;
            let cz = (rnd() - 0.5) * 0.030;
            crack.position().set(cx, 0.040, cz);
        } else {
            let cx = -0.02 + (rnd() - 0.5) * 0.06;
            let cz = (rnd() - 0.5) * 0.04;
            crack.position().set(cx, 0.060, cz);
        }
        crack.rotation().y = rnd() * PI;
        crack.rotation().z = (rnd() - 0.5) * 0.8;
        bd.add(&crack);
    }
    // 40%: tea-stain on dress.
    if rnd() < 0.4 {
        let stain_mat = lambert!({ color: 0x603818, transparent: true, opacity: 0.55 });
        let stain = mesh(circle_geo!(0.020, 6), &stain_mat);
        stain.rotation().x = -PI / 2.0;
        let sx = -0.020 + (rnd() - 0.5) * 0.030;
        let sz = (rnd() - 0.5) * 0.030;
        stain.position().set(sx, 0.0955, sz); // on the dress top
        bd.add(&stain);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bd.position().set(px, 0, pz);
    bd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bd);
}

/// Place `o` at `a + d * s`.
fn place_along(o: &Obj, a: &Vec3, d: &Vec3, s: f64) {
    let mut p = o.position();
    p.copy(a);
    p.add_scaled_vector(d, s);
}
