//! `place_folded_umbrellas`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Folded umbrella: 8% chance per level. Compact folded
// umbrella w/ handle and strap.
pub(crate) fn place_folded_umbrellas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let cloth_colors = [0x181410, 0x2050a0, 0xa02040, 0x304050, 0x603820];
    let cloth_mat = lambert!({ color: choice(&cloth_colors) });
    let handle_mat = lambert!({ color: 0x2a1810 });
    let strap_mat = lambert!({ color: 0x8a7050 });
    let umb = group();
    // Main body (folded cylinder).
    let body = mesh(pg("FoldedUmbrellas__body", cyl_geo!(0.020, 0.020, 0.32, 8)), &cloth_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.020, 0);
    umb.add(&body);
    // Ridged folds (small dark stripes).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let stripe = mesh(pg("FoldedUmbrellas__seam", cyl_geo!(0.021, 0.021, 0.32, 3)), &handle_mat);
        stripe.rotation().z = PI / 2.0;
        stripe.rotation().x = a;
        stripe.scale().set(1, 1, 0.03);
        stripe.position().set(0, 0.020, 0);
        umb.add(&stripe);
    }
    // Tip cap (pointy).
    let tip = mesh(pg("FoldedUmbrellas__tip", cone_geo!(0.014, 0.020, 6)), &cloth_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(0.170, 0.020, 0);
    umb.add(&tip);
    // Curved handle.
    let handle = mesh(pg("FoldedUmbrellas__handle", torus_geo!(0.020, 0.005, 4, 8, PI)), &handle_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().set(-0.175, 0.020, 0);
    umb.add(&handle);
    // Strap loop.
    let strap = mesh(torus_geo!(0.008, 0.001, 3, 6), &strap_mat);
    // A band round the furled canopy (YZ plane, R ~0.021), not buried inside it.
    strap.rotation().y = PI / 2.0;
    strap.scale().set_scalar(2.6);
    strap.position().set(0, 0.020, 0);
    umb.add(&strap);
    scatter(g, &umb, x, z, 2.0);
    g.world.add(&umb);
}
