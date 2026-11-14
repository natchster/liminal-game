//! `place_childrens_tea_sets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Children's tea set: 5% chance per level. 3-4 mini cups + teapot.
pub(crate) fn place_childrens_tea_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let set_colors = [0xf0c0d0, 0xd0d8f0, 0xf0e8d8, 0xd0f0d8];
    let set_mat = lambert!({ color: pick_hex(&set_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let grp = group();
    // Teapot in center: sphere + spout + handle + lid.
    let pot = mesh(pg("ChildrensTeaSets__pot", sphere_geo!(0.024, 10, 8)), &set_mat);
    pot.scale().y = 0.8;
    pot.position().set(0, 0.019, 0);
    grp.add(&pot);
    let spout = mesh(pg("ChildrensTeaSets__spout", cyl_geo!(0.005, 0.003, 0.020, 5)), &set_mat);
    spout.rotation().z = -0.5;
    spout.position().set(0.024, 0.024, 0);
    grp.add(&spout);
    let handle = mesh(pg("ChildrensTeaSets__handle", torus_geo!(0.008, 0.002, 4, 8, PI)), &set_mat);
    handle.rotation().z = PI / 2.0; // C in the XY plane, bulging out behind (-X)
    handle.position().set(-0.0215, 0.020, 0); // both ends in the pot wall
    grp.add(&handle);
    let lid = mesh(pg("ChildrensTeaSets__lid", sphere_geo!(0.006, 6, 5)), &set_mat);
    lid.position().set(0, 0.038, 0);
    grp.add(&lid);
    // 3-4 mini cups around.
    let n_cups = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_cups {
        let a = (i as f64 / n_cups as f64) * PI * 2.0;
        let cup = mesh(pg("ChildrensTeaSets__cup", cyl_geo!(0.011, 0.008, 0.015, 8, 1, true)), &set_mat);
        cup.position().set(a.cos() * 0.06, 0.008, a.sin() * 0.06);
        grp.add(&cup);
        // Saucer.
        let saucer = mesh(pg("ChildrensTeaSets__saucer", cyl_geo!(0.018, 0.018, 0.002, 10)), &set_mat);
        saucer.position().set(a.cos() * 0.06, 0.001, a.sin() * 0.06);
        grp.add(&saucer);
        // Tea residue.
        if rnd() < 0.5 {
            let tea = mesh(circle_geo!(0.008, 8), &tea_mat);
            tea.rotation().x = -PI / 2.0;
            tea.position().set(a.cos() * 0.06, 0.013, a.sin() * 0.06);
            grp.add(&tea);
        }
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}
