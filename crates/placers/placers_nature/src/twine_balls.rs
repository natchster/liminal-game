//! `place_twine_balls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Twine ball: 7% chance per level. Wound sphere w/ visible thread wraps +
// trailing end.
pub(crate) fn place_twine_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let twine_colors: [u32; 4] = [0xc8a870, 0xa08050, 0xe0d8b8, 0x806040];
    let twine_mat = lambert!({ color: pick(&twine_colors) });
    let twine = group();
    // Main ball sphere.
    let ball = mesh(pg("TwineBalls__ball", sphere_geo!(0.038, 12, 10)), &twine_mat);
    ball.position().y = 0.038;
    twine.add(&ball);
    // 6 thin wrap lines (rings around ball) for wound look.
    for _i in 0..6 {
        let wrap = mesh(pg("TwineBalls__wrap", torus_geo!(0.038, 0.001, 3, 16)), lambert!({ color: 0x806040 }));
        wrap.rotation().x = rnd() * PI;
        wrap.rotation().y = rnd() * PI;
        wrap.position().y = 0.038;
        twine.add(&wrap);
    }
    // Trailing end (short segment extending out).
    let end = mesh(pg("TwineBalls__end", cyl_geo!(0.001, 0.001, 0.070, 4)), &twine_mat);
    // Leaves the ball's lower flank (+Y end on its surface) and slopes down
    // to its frayed free end (-Y) lying on the floor.
    let end_a = v3(0.030, 0.016, 0.010);
    let end_b = v3(0.030 + 0.0684 * 0.4f64.dcos(), 0.0012, 0.010 - 0.0684 * 0.4f64.dsin());
    let mut dir = end_a;
    dir.sub(&end_b);
    dir.normalize();
    end.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
    {
        let mut p = end.position();
        p.copy(&end_a);
        p.add(&end_b);
        p.multiply_scalar(0.5);
    }
    twine.add(&end);
    twine.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    twine.rotation().y = rnd() * PI * 2.0;
    g.world.add(&twine);
}
