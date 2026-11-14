//! `place_fallen_ceiling_tiles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Fallen ceiling tile: 15% chance per level. Small flat
// rectangle on floor representing a dropped acoustic-ceiling tile.
pub(crate) fn place_fallen_ceiling_tiles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let tile_mat = lambert!({ color: 0xd8d0c0 });
    let tile = mesh(pg("FallenCeilingTiles__tile", box_geo!(0.60, 0.02, 0.60)), &tile_mat);
    tile.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0.01, g.wz(z) + (rnd() - 0.5) * 1.6);
    tile.rotation().y = rnd() * PI * 2.0;
    tile.rotation().x = (rnd() - 0.5) * 0.20;
    tile.rotation().z = (rnd() - 0.5) * 0.20;
    g.world.add(&tile);
    // Dark rectangle in ceiling directly above tile position.
    let gap_mat = basic_mat!({ color: 0x0a0806 });
    let gap = mesh(plane_geo!(0.55, 0.55), &gap_mat);
    gap.rotation().x = PI / 2.0; // face down from ceiling
    let tp = tile.pos();
    gap.position().set(tp.x, WALL_H - 0.01, tp.z);
    g.world.add(&gap);
}
