//! `place_small_padlock_keys`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small padlock + key: 10% chance per level.
pub(crate) fn place_small_padlock_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = lambert!({ color: 0x2a1810 });
    let grp = group();
    // === PADLOCK ===
    let lock = group();
    // Body: rounded box.
    let body = mesh(pg("SmallPadlockKeys__body", box_geo!(0.040, 0.048, 0.020)), &brass_mat);
    body.position().y = 0.028;
    lock.add(&body);
    // Keyhole dark circle on front.
    let keyhole = mesh(circle_geo!(0.005, 6), &dark_mat);
    keyhole.position().set(0, 0.024, 0.011);
    lock.add(&keyhole);
    // Shackle (U-shape arch): an inverted U standing on the two posts.
    let shackle = mesh(pg("SmallPadlockKeys__shackle", torus_geo!(0.014, 0.004, 5, 12, PI)), &brass_mat);
    shackle.position().y = 0.056;
    lock.add(&shackle);
    // 2 shackle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallPadlockKeys__post", cyl_geo!(0.004, 0.004, 0.020, 5)), &brass_mat);
        post.position().set(s * 0.014, 0.048, 0);
        lock.add(&post);
    }
    lock.position().set(-0.06, -0.004, 0); // body bottom (0.004) on the floor
    grp.add(&lock);
    // === KEY === (lying flat: shaft, bow and bit all on the floor)
    let key = group();
    // Shaft.
    let shaft = mesh(pg("SmallPadlockKeys__shaft", cyl_geo!(0.002, 0.002, 0.080, 4)), &brass_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.040, 0.002, 0);
    key.add(&shaft);
    // Bow (ring at end), flat in the shaft's plane.
    let bow = mesh(pg("SmallPadlockKeys__bow", torus_geo!(0.012, 0.003, 4, 10)), &brass_mat);
    bow.rotation().x = PI / 2.0;
    bow.position().set(-0.005, 0.003, 0);
    key.add(&bow);
    // 2 teeth on other end, lying flat off the shaft's side.
    for i in 0..2 {
        let tooth = mesh(pg("SmallPadlockKeys__tooth", box_geo!(0.006, 0.008, 0.002)), &brass_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.070 + i as f64 * 0.008, 0.002, 0.005);
        key.add(&tooth);
    }
    key.position().set(0.02, 0, 0.05);
    key.rotation().y = -PI / 6.0;
    grp.add(&key);
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}
