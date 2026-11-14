//! `place_stuffed_rabbits`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Stuffed rabbit: 4% chance per level. Similar to bear but w/ long ears +
// smaller body.
pub(crate) fn place_stuffed_rabbits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bunny_colors: [u32; 5] = [0xf0e8d8, 0xd0d0d0, 0xd0a080, 0xf0d0d0, 0xa08060];
    let bunny_mat = lambert!({ color: pick(&bunny_colors) });
    let nose_mat = lambert!({ color: 0xd050a0 });
    let eye_mat = lambert!({ color: 0x101010 });
    let rabbit = group();
    // Body (smaller than bear, more upright).
    let body = mesh(pg("StuffedRabbits__body", sphere_geo!(0.035, 10, 8)), &bunny_mat);
    body.scale().set(0.9, 1, 0.9);
    body.position().y = 0.035;
    rabbit.add(&body);
    // Head sphere.
    let head = mesh(pg("StuffedRabbits__head", sphere_geo!(0.028, 10, 8)), &bunny_mat);
    head.position().y = 0.078;
    rabbit.add(&head);
    // 2 long tall ears (upright ellipsoids).
    // Track for ear twitch animator: (ear, phase, side)
    let mut rabbit_ears: Vec<(Obj, f64, f64)> = vec![];
    for side in [-1.0f64, 1.0] {
        let ear = mesh(pg("StuffedRabbits__ear", sphere_geo!(0.008, 6, 5)), &bunny_mat);
        ear.scale().set(0.7, 2.5, 0.5);
        ear.position().set(side * 0.008, 0.115, -0.005);
        let phase = rnd() * PI * 2.0 + (if side > 0.0 { 1.7 } else { 0.0 });
        {
            let mut u = ear.user();
            u.base_z = 0.0;
            u.phase = phase;
            u.set("side", side);
        }
        rabbit_ears.push((ear.clone(), phase, side));
        rabbit.add(&ear);
    }
    // 2 arms.
    for side in [-1.0, 1.0] {
        let arm = mesh(pg("TeddyBears__arm", sphere_geo!(0.014, 6, 5)), &bunny_mat);
        arm.scale().set(1.2, 0.7, 0.7);
        arm.position().set(side * 0.032, 0.040, 0.006);
        rabbit.add(&arm);
    }
    // 2 legs (larger — bunnies have big hind feet).
    for side in [-1.0, 1.0] {
        let leg = mesh(pg("StuffedRabbits__foot", sphere_geo!(0.018, 6, 5)), &bunny_mat);
        leg.scale().set(1, 0.5, 1.4);
        // under the face (-Z), toes out front and heels under the belly, soles on the floor
        leg.position().set(side * 0.018, 0.009, -0.028);
        rabbit.add(&leg);
    }
    // Small round tail on back.
    let tail = mesh(pg("StuffedRabbits__tail", sphere_geo!(0.008, 6, 5)), lambert!({ color: 0xffffff }));
    tail.position().set(0, 0.028, 0.030);
    rabbit.add(&tail);
    // Face: 2 eyes + pink nose.
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.0025, 5, 4)), &eye_mat);
        eye.position().set(side * 0.009, 0.084, -0.025);
        rabbit.add(&eye);
    }
    let nose = mesh(pg("TeddyBears__nose", sphere_geo!(0.004, 5, 4)), &nose_mat);
    nose.position().set(0, 0.076, -0.028);
    rabbit.add(&nose);
    rabbit.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    rabbit.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rabbit);
    let p = rabbit.pos();
    g.pp.plush_positions.push(v3(p.x, 0, p.z));
    // Subtle ear twitch — rare rapid rotations w/ long idle.
    g.animate(move |_g, _dt, t| {
        for (ear, phase, side) in &rabbit_ears {
            // Base + occasional short twitch (when sin > 0.96, add a rapid wobble).
            let trigger = (t * 0.2 + phase).sin();
            if trigger > 0.96 {
                ear.rotation().z = (t * 30.0).sin() * 0.08 * side;
            } else {
                ear.rotation().z = 0.0;
            }
        }
    });
}
