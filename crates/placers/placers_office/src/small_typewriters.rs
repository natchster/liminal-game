//! `place_small_typewriters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage typewriter: 5% chance per level.
pub(crate) fn place_small_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_mat = lambert!({ color: 0x201a10 });
    let key_mat = lambert!({ color: 0xf0ece0 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let roll_mat = lambert!({ color: 0x2a2418 });
    let tw = group();
    // Base body.
    let base = mesh(pg("SmallTypewriters__base", box_geo!(0.16, 0.02, 0.14)), &body_mat);
    base.position().y = 0.010;
    tw.add(&base);
    // Angled key deck (top-front slope).
    let deck = mesh(pg("SmallTypewriters__deck", box_geo!(0.16, 0.010, 0.08)), &body_mat);
    deck.rotation().x = 0.35; // low at the typist (+z), rising toward the roll
    deck.position().set(0, 0.036, 0.03);
    tw.add(&deck);
    // 4x10 key grid, sitting on the deck's recessed floor (row 0 at the front).
    for ky in 0..4 {
        for kx in 0..10 {
            let key = mesh(pg("VintageTypewriters__key", cyl_geo!(0.005, 0.005, 0.004, 5)), &key_mat);
            let kkx = -0.070 + kx as f64 * 0.0155;
            let kkz = 0.0567 - ky as f64 * 0.0169;
            key.rotation().x = 0.35;
            key.position().set(kkx, 0.0305 + ky as f64 * 0.0062, kkz);
            tw.add(&key);
        }
    }
    // Back platen (paper roll cylinder), resting on the base.
    let roll = mesh(pg("SmallTypewriters__roll", cyl_geo!(0.020, 0.020, 0.13, 8)), &roll_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(0, 0.040, -0.045);
    tw.add(&roll);
    // Paper coming out top.
    let paper = mesh(plane_geo!(0.12, 0.07), &paper_mat);
    paper.position().set(0, 0.078, -0.045);
    paper.rotation().x = -0.15;
    tw.add(&paper);
    // Return lever on side: foot on the roll's left end, leaning up and out.
    let lever = mesh(pg("VintageTypewriters__lever", cyl_geo!(0.003, 0.003, 0.030, 4)), &roll_mat);
    lever.rotation().z = 0.5;
    lever.position().set(-0.073, 0.058, -0.045);
    tw.add(&lever);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
}
