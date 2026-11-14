//! `place_small_jewelry_boxes`: Small jewelry box: 6% chance per level. Small hinged box with visible gems inside (60% open). Dressing-table family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small jewelry box: 6% chance per level. Small hinged box
/// with visible gems inside (60% open). Dressing-table family.
pub(crate) fn place_small_jewelry_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let box_colors = [0x6a3820, 0x8a5030, 0x201a10, 0xd8b040];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let felt_mat = lambert!({ color: 0x8a2040 });
    let gem_colors = [0xd82060, 0x2080d8, 0x30c060, 0xd050a0, 0xd8b040];
    let jewel = group();
    let is_open = rnd() < 0.60;
    // Body: cube.
    let body = mesh(pg("SmallJewelryBoxes__body", box_geo!(0.10, 0.04, 0.07)), &box_mat);
    body.position().y = 0.020;
    jewel.add(&body);
    if is_open {
        // Felt inner floor.
        let felt = mesh(pg("SmallJewelryBoxes__felt", plane_geo!(0.09, 0.06)), &felt_mat);
        felt.rotation().x = -PI / 2.0;
        felt.position().y = 0.041;
        jewel.add(&felt);
        // 3-5 tiny gems scattered on felt.
        let n_gems = 3 + (rnd() * 3.0).floor() as i32;
        for _ in 0..n_gems {
            let gem_mat = basic_mat!({ color: pick(&gem_colors), transparent: true, opacity: 0.9 });
            let gem = mesh(sphere_geo!(0.006, 6, 5), &gem_mat);
            gem.position().set((rnd() - 0.5) * 0.07, 0.045, (rnd() - 0.5) * 0.05);
            jewel.add(&gem);
        }
        // Lid open behind (leaning back).
        let lid = mesh(pg("SmallJewelryBoxes__lidOpen", box_geo!(0.10, 0.03, 0.07)), &box_mat);
        lid.position().set(0, 0.078, -0.0305); // hinge edge on the box's back top edge
        lid.rotation().x = -PI / 3.0;
        jewel.add(&lid);
    } else {
        // Closed lid on top.
        let lid = mesh(pg("SmallJewelryBoxes__lid", box_geo!(0.10, 0.014, 0.07)), &box_mat);
        lid.position().y = 0.047;
        jewel.add(&lid);
        // Small brass hinge on back.
        let hinge = mesh(pg("SmallJewelryBoxes__hinge", cyl_geo!(0.003, 0.003, 0.02, 4)), lambert!({ color: 0xd8b040 }));
        hinge.rotation().z = PI / 2.0;
        hinge.position().set(0, 0.043, -0.033);
        jewel.add(&hinge);
    }
    jewel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    jewel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&jewel);
}
