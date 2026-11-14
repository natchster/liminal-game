//! `place_ceiling_pipes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Ceiling pipe run: 5% chance per level. Horizontal pipe
// segment attached to ceiling w/ 2 hanger straps.
pub(crate) fn place_ceiling_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let pipe_mat = lambert!({ color: 0x707078 });
    let rust_mat = lambert!({ color: 0x6a3820 });
    let pipe = group();
    let pipe_y = WALL_H - 0.08;
    // Long pipe.
    let body = mesh(pg("CeilingPipes__body", cyl_geo!(0.024, 0.024, 2.0, 8)), &pipe_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = pipe_y;
    pipe.add(&body);
    // Joint band in middle.
    let joint = mesh(pg("WallPipeLeaks__joint", cyl_geo!(0.027, 0.027, 0.020, 8)), &rust_mat);
    joint.rotation().z = PI / 2.0;
    joint.position().y = pipe_y;
    pipe.add(&joint);
    // 2 hanger straps.
    for s in [-0.6, 0.6] {
        let strap = mesh(pg("CeilingPipes__strap", box_geo!(0.014, 0.08, 0.004)), &pipe_mat);
        strap.position().set(s, pipe_y + 0.045, 0);
        pipe.add(&strap);
    }
    scatter(g, &pipe, x, z, 1.0);
    g.world.add(&pipe);
}
