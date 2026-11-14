//! `place_small_telegraph_keys`
use crate::shared::vsub;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small telegraph key: 8% chance per level.
pub(crate) fn place_small_telegraph_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x603820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let knob_mat = lambert!({ color: 0x181008 });
    let key = group();
    // Base wood plate.
    let base = mesh(pg("SmallTelegraphKeys__base", box_geo!(0.08, 0.010, 0.06)), &wood_mat);
    base.position().y = 0.005;
    key.add(&base);
    // Brass pivot posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallTelegraphKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(0, 0.020, s * 0.010);
        key.add(&post);
    }
    // Key lever arm.
    let lever = mesh(pg("SmallTelegraphKeys__lever", box_geo!(0.06, 0.004, 0.008)), &brass_mat);
    lever.position().set(0.008, 0.024, 0);
    key.add(&lever);
    // Bakelite knob on top.
    let knob = mesh(pg("SmallTelegraphKeys__knob", sphere_geo!(0.010, 8, 6)), &knob_mat);
    knob.scale().y = 0.5;
    knob.position().set(0.028, 0.028, 0);
    key.add(&knob);
    // 2 brass terminals on base.
    for s in [-1.0f64, 1.0] {
        let term = mesh(pg("SmallTelegraphKeys__terminal", cyl_geo!(0.004, 0.004, 0.008, 5)), &brass_mat);
        term.position().set(-0.030, 0.014, s * 0.020);
        key.add(&term);
    }
    // Wire trailing off: from between the terminals, over the base's end and
    // down onto the floor.
    let wire = mesh(pg("SmallTelegraphKeys__wire", cyl_geo!(0.001, 0.001, 0.10, 3)), &knob_mat);
    let wire_a = v3(-0.030, 0.012, 0);
    let wire_b = v3(-0.126, 0.001, 0.012);
    let dir = vsub(wire_b, wire_a).normalized();
    wire.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
    wire.scale().y = wire_a.distance_to(&wire_b) / 0.10;
    {
        let mut p = wire.position();
        p.copy(&wire_a).add(&wire_b).multiply_scalar(0.5);
    }
    key.add(&wire);
    key.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    key.rotation().y = rnd() * PI * 2.0;
    g.world.add(&key);
}
