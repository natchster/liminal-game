//! `place_rubber_ducks`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Rubber duck: 10% chance per level.
pub(crate) fn place_rubber_ducks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_mat = lambert!({ color: 0xf0d020 });
    let beak_mat = lambert!({ color: 0xe07020 });
    let eye_mat = basic_mat!({ color: 0x181410 });
    let duck = group();
    // Body (round).
    let body = mesh(pg("RubberDucks__body", sphere_geo!(0.030, 10, 8)), &body_mat);
    body.scale().set(1.2, 0.9, 1);
    body.position().y = 0.024;
    duck.add(&body);
    // Head.
    let head = mesh(pg("RubberDucks__head", sphere_geo!(0.020, 10, 8)), &body_mat);
    head.position().set(0.024, 0.048, 0);
    duck.add(&head);
    // Beak.
    let beak = mesh(pg("RubberDucks__bill", box_geo!(0.014, 0.006, 0.010)), &beak_mat);
    beak.position().set(0.048, 0.045, 0); // root 3 mm in the head, the rest out
    duck.add(&beak);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.002, 5, 4), &eye_mat);
        eye.position().set(0.0357, 0.0539, s * 0.0144); // half out of the head surface
        duck.add(&eye);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    duck.position().set(px, 0, pz);
    duck.rotation().y = rnd() * PI * 2.0;
    g.world.add(&duck);
}
