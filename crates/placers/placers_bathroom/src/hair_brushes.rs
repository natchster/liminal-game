//! `place_hair_brushes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Hair brush on floor: 8% chance per level.
pub(crate) fn place_hair_brushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let handle_colors = [0xd08040, 0x804020, 0x404040, 0xe0c0a0];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let bristle_mat = lambert!({ color: 0x201810 });
    let brush = group();
    // Bristle pad (flat oval-ish rectangle).
    let pad = mesh(pg("HairBrushes__pad", box_geo!(0.06, 0.010, 0.11)), &handle_mat);
    pad.position().set(0, 0.005, 0);
    brush.add(&pad);
    // Bristles layer (thin dark rectangle on top).
    let bristles = mesh(pg("HairBrushes__bristles", box_geo!(0.055, 0.012, 0.10)), &bristle_mat);
    bristles.position().set(0, 0.014, 0);
    brush.add(&bristles);
    // Handle (elongated cylinder): neck 2 mm into the pad, butt on the floor.
    let handle = mesh(pg("HairBrushes__handle", cyl_geo!(0.014, 0.012, 0.10, 6)), &handle_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().set(0, 0.014, 0.103);
    brush.add(&handle);
    brush.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}
