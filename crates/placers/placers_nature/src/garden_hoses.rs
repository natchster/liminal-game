//! `place_garden_hoses`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Garden hose coil: 5% chance per level. Concentric tori forming coiled hose.
pub(crate) fn place_garden_hoses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hose_colors: [u32; 4] = [0x408040, 0x2060c0, 0x606060, 0x804040];
    let hose_mat = lambert!({ color: pick(&hose_colors) });
    let nozzle_mat = lambert!({ color: 0xd0a040 });
    let hose = group();
    // 4 concentric torus loops for coiled read.
    // Lying flat on the floor, each turn pressed against the next (1.4 cm
    // pitch for a 1.6 cm hose), so they read as one flat coil.
    for i in 0..4 {
        let r = 0.090 - i as f64 * 0.014;
        let lp = mesh(pg("GardenHoses__loop", torus_geo!(r, 0.008, 5, 16)), &hose_mat);
        lp.rotation().x = PI / 2.0;
        lp.rotation().z = i as f64 * 1.3; // turn each loop's stripe/bump round
        lp.position().y = 0.008;
        hose.add(&lp);
    }
    // Brass spray nozzle at end.
    // The hose end leaves the outer turn tangentially at (0.09, 0): the
    // coupling (+Y) sits on the hose there, the front points along +Z.
    let nozzle = mesh(pg("GardenHoses__nozzle", cyl_geo!(0.010, 0.012, 0.060, 8)), &nozzle_mat);
    nozzle.rotation().x = -PI / 2.0;
    nozzle.position().set(0.094, 0.011, 0.028);
    hose.add(&nozzle);
    // Nozzle tip cone.
    let tip = mesh(pg("GardenHoses__tip", cone_geo!(0.008, 0.014, 6)), &nozzle_mat);
    tip.rotation().x = PI / 2.0;
    tip.position().set(0.094, 0.011, 0.064);
    hose.add(&tip);
    hose.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    hose.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hose);
}
