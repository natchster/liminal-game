//! `place_crayons`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Crayon cluster: 8% chance per level. 5-9 short colored sticks.
pub(crate) fn place_crayons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crayon_colors: [u32; 10] = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xe08040, 0xd050a0, 0x804060, 0x202020, 0xffffff, 0xc0a040];
    let wrapper_mat = lambert!({ color: 0xf8f4e8 });
    let cluster = group();
    let count = 5 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let color = pick(&crayon_colors);
        let body_mat = lambert!({ color: color });
        let crayon = group();
        // Wrapper (main cylinder).
        let wrapper = mesh(pg("Crayons__wrapper", cyl_geo!(0.004, 0.004, 0.055, 6)), &wrapper_mat);
        wrapper.rotation().z = PI / 2.0;
        wrapper.position().y = 0.004;
        crayon.add(&wrapper);
        // Colored exposed tip (cone).
        let tip = mesh(pg("Crayons__tip", cone_geo!(0.004, 0.010, 6)), &body_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.032, 0.004, 0);
        crayon.add(&tip);
        // Small colored back-end nub.
        let back = mesh(pg("Crayons__back", cyl_geo!(0.004, 0.004, 0.008, 6)), &body_mat);
        back.rotation().z = PI / 2.0;
        back.position().set(-0.031, 0.004, 0); // butts into the wrapper end (-0.0275)
        crayon.add(&back);
        crayon.position().set((rnd() - 0.5) * 0.14, 0, (rnd() - 0.5) * 0.14);
        crayon.rotation().y = rnd() * PI * 2.0;
        cluster.add(&crayon);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}
