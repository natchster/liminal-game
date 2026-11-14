//! `place_small_toy_robots`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_pos;

// Small toy robot: 6% chance per level. Blocky humanoid
// robot with head + body + limbs + antenna. Child-toy family.
pub(crate) fn place_small_toy_robots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0xc8c8c8, 0x606870, 0xd82040, 0x2050d8, 0xd8b040];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let detail_mat = lambert!({ color: 0x181008 });
    let light_mat = basic_mat!({ color: 0xd82040 });
    let antenna_ball_mat = basic_mat!({ color: 0xd82040 });
    let robot = group();
    // Body: block torso.
    let torso = mesh(pg("SmallToyRobots__torso", box_geo!(0.036, 0.040, 0.024)), &body_mat);
    torso.position().y = 0.038;
    robot.add(&torso);
    // Head: smaller cube.
    let head = mesh(pg("SmallToyRobots__head", box_geo!(0.024, 0.024, 0.022)), &body_mat);
    head.position().y = 0.070;
    robot.add(&head);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.003, 5, 4), &light_mat);
        eye.position().set(s * 0.006, 0.073, 0.012);
        robot.add(&eye);
    }
    // Mouth grill (dark bar).
    let mouth = mesh(box_geo!(0.012, 0.002, 0.001), &detail_mat);
    mouth.position().set(0, 0.065, 0.0112); // back face just inside the head front (z 0.011)
    robot.add(&mouth);
    // Antenna.
    let ant = mesh(pg("SmallToyRobots__antenna", cyl_geo!(0.001, 0.001, 0.020, 4)), &body_mat);
    ant.position().y = 0.092;
    robot.add(&ant);
    let ant_ball = mesh(sphere_geo!(0.004, 5, 4), &antenna_ball_mat);
    ant_ball.position().y = 0.104;
    robot.add(&ant_ball);
    // 2 arms.
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("SmallToyRobots__arm", box_geo!(0.010, 0.028, 0.010)), &body_mat);
        arm.position().set(s * 0.023, 0.038, 0);
        robot.add(&arm);
    }
    // 2 legs.
    for s in [-1.0, 1.0] {
        let leg = mesh(pg("SmallToyRobots__leg", box_geo!(0.014, 0.020, 0.014)), &body_mat);
        leg.position().set(s * 0.008, 0.010, 0);
        robot.add(&leg);
    }
    // Chest button/gauge.
    let chest = mesh(circle_geo!(0.004, 6), &light_mat);
    chest.position().set(0, 0.038, 0.013);
    robot.add(&chest);
    scatter_pos(g, &robot, x, z, 2.4);
    robot.rotation().y = rnd() * PI * 2.0;
    // 20% chance tipped on side.
    if rnd() < 0.20 {
        robot.rotation().z = PI / 2.0;
        robot.position().y = 0.020;
    }
    g.world.add(&robot);
}
