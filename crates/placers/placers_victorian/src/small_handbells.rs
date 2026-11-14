//! `place_small_handbells`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Small handbell: 10% chance per level. Brass dome + wooden
// handle + tiny inner clapper.
pub(crate) fn place_small_handbells(g: &mut Game, _dist: &[i32]) {
    g.pp.handbell_positions = vec![]; // reset per level
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let brass_mat = lambert!({ color: 0xa8802a });
    let handle_mat = lambert!({ color: 0x4a2818 });
    let clapper_mat = lambert!({ color: 0x2a2418 });
    let bell = group();
    // Dome: half-sphere brass.
    let dome = mesh(pg("SmallHandbells__dome", sphere_geo!(0.045, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &brass_mat);
    // The upper hemisphere as built already opens downward: mouth (rim) at
    // y 0.005, crown at 0.05 under the collar and handle.
    dome.position().y = 0.005;
    bell.add(&dome);
    // Small ring at bell shoulder.
    let ring = mesh(pg("SmallHandbells__ring", torus_geo!(0.008, 0.002, 4, 8)), &brass_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.052;
    bell.add(&ring);
    // Handle: short vertical cylinder above dome.
    let handle = mesh(pg("SmallHandbells__handle", cyl_geo!(0.008, 0.010, 0.08, 6)), &handle_mat);
    handle.position().y = 0.095;
    bell.add(&handle);
    // Tiny clapper: dark sphere hanging inside dome.
    let clapper = mesh(pg("SmallHandbells__clapper", sphere_geo!(0.006, 6, 4)), &clapper_mat);
    clapper.position().y = 0.02;
    bell.add(&clapper);
    // Bell is usually lying on side or upright: 50/50.
    if rnd() < 0.50 {
        bell.rotation().z = PI / 2.0;
        bell.position().y = 0.045; // rest lying on rim
    }
    bell.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bell.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bell.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bell);
    // Track handbell position for rare chime audio events.
    let bp = bell.pos();
    g.pp.handbell_positions.push(v3(bp.x, 0.05, bp.z));
}
