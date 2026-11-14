//! `place_small_suitcases`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage suitcase: 8% chance per level.
pub(crate) fn place_small_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let leather_mat = lambert!({ color: pick(&leather_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let suitcase = group();
    // Body.
    let body = mesh(pg("SmallSuitcases__body", box_geo!(0.22, 0.10, 0.14)), &leather_mat);
    body.position().y = 0.05;
    suitcase.add(&body);
    // Middle divider line (open-close seam).
    let seam = mesh(pg("SmallSuitcases__seam", box_geo!(0.222, 0.003, 0.142)), &brass_mat);
    seam.position().y = 0.05;
    suitcase.add(&seam);
    // Handle (torus arc on top).
    let handle = mesh(pg("SmallSuitcases__handle", torus_geo!(0.024, 0.005, 4, 12, PI)), &leather_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().y = 0.104;
    suitcase.add(&handle);
    // 2 brass buckles on front.
    for s in [-1.0f64, 1.0] {
        let buckle = mesh(pg("SmallSuitcases__buckle", box_geo!(0.014, 0.014, 0.004)), &brass_mat);
        buckle.position().set(s * 0.05, 0.048, 0.072);
        suitcase.add(&buckle);
        // Buckle latch (small strip).
        let latch = mesh(pg("SmallSuitcases__latch", box_geo!(0.008, 0.006, 0.002)), &brass_mat);
        latch.position().set(s * 0.05, 0.048, 0.075);
        suitcase.add(&latch);
    }
    // Corner feet at bottom.
    for (cx, cz) in [(-1.0f64, -1.0f64), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let corner = mesh(pg("SmallSuitcases__foot", cyl_geo!(0.008, 0.008, 0.008, 5)), &brass_mat);
        corner.position().set(cx * 0.095, 0.004, cz * 0.055);
        suitcase.add(&corner);
    }
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}
