//! `place_trowels`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Gardening trowel: 6% chance per level. Metal scoop blade + handle.
pub(crate) fn place_trowels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0x8b5a2b, 0x404040, 0xc02020, 0x2060c0];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let trowel = group();
    // Scoop blade (curved via 2 rotated planes + a cylinder half).
    // Simpler: a slightly tapered half cylinder.
    let blade = mesh(pg("Trowels__blade", cyl_geo!(0.020, 0.014, 0.080, 6, 1, false, 0, PI)), &blade_mat);
    // Scoop lying hollow-up: the half-cylinder's round back (+X) down, its
    // width across (Z), the tip end (+Y) toward -X; the belly rests on the
    // floor and the rim is at y 0.020.
    blade.rotation().x = PI;
    blade.rotation().z = PI / 2.0;
    blade.position().set(-0.030, 0.020, 0);
    trowel.add(&blade);
    // Blade tip cone (pointed end): flattened (0.35) to a point continuing
    // the scoop's floor, lying on the floor.
    let tip = mesh(pg("Trowels__tip", cone_geo!(0.015, 0.020, 6)), &blade_mat);
    tip.rotation().z = -PI / 2.0;
    tip.scale().x = 0.35;
    tip.position().set(-0.080, 0.0055, 0);
    trowel.add(&tip);
    // Shaft (thin cylinder from blade to handle).
    let shaft = mesh(pg("Trowels__shaft", cyl_geo!(0.005, 0.005, 0.040, 6)), &blade_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.008, 0.010, 0);
    trowel.add(&shaft);
    // Wooden/plastic handle grip.
    let handle = mesh(pg("Trowels__handle", cyl_geo!(0.012, 0.014, 0.060, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.056, 0.012, 0);
    trowel.add(&handle);
    // Small end cap.
    let cap = mesh(pg("Trowels__cap", sphere_geo!(0.014, 6, 5)), &handle_mat);
    cap.position().set(0.090, 0.012, 0);
    trowel.add(&cap);
    trowel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trowel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&trowel);
}
