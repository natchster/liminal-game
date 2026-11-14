//! `place_single_earrings`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Single earring: 9% chance per level.
pub(crate) fn place_single_earrings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    } // 9% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let metal_colors = [0xffd700, 0xe0e0e0, 0xd08040];
    let metal_mat = lambert!({ color: pick(&metal_colors) });
    let gem_colors = [0xff4040, 0x4080ff, 0x40ff80, 0xff40ff, 0xffffff];
    let gem_mat = lambert!({ color: pick(&gem_colors) });
    let earring = group();
    // Tiny loop (torus).
    let lp = mesh(pg("SingleEarrings__loop", torus_geo!(0.010, 0.0015, 4, 12)), &metal_mat);
    lp.rotation().x = PI / 2.0;
    lp.position().y = 0.0015;
    earring.add(&lp);
    // Small gem/stud on the loop.
    let gem = mesh(pg("SingleEarrings__gem", octa_geo!(0.005)), &gem_mat);
    gem.position().set(0, 0.005, 0.010);
    earring.add(&gem);
    earring.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    earring.rotation().y = rnd() * PI * 2.0;
    g.world.add(&earring);
}
