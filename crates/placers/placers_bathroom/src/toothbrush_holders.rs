//! `place_toothbrush_holders`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Toothbrush holder cup: 6% chance per level, 1-2 toothbrushes.
pub(crate) fn place_toothbrush_holders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let shelf_mat = lambert!({ color: 0xc0b8a8 });
    let cup_colors = [0xffffff, 0x40a0c0, 0xe08080, 0xf0d040];
    let cup_mat = lambert!({ color: pick(&cup_colors) });
    let brush_colors = [0x40a0e0, 0xe04040, 0x60c060, 0xe0c040];
    let bristle_mat = lambert!({ color: 0xf0e8d8 });
    let holder = group();
    let shelf_y = 1.25;
    // Small shelf.
    let shelf = mesh(pg("ToothbrushHolders__shelf", box_geo!(0.16, 0.015, 0.10)), &shelf_mat);
    shelf.position().y = shelf_y;
    holder.add(&shelf);
    // Cup.
    let cup = mesh(pg("ToothbrushHolders__cup", cyl_geo!(0.035, 0.030, 0.10, 10)), &cup_mat);
    cup.position().set(0, shelf_y + 0.058, 0);
    holder.add(&cup);
    // 1-2 toothbrushes sticking up.
    let brush_count = 1 + (rnd() * 2.0).floor() as i32;
    for b in 0..brush_count {
        let b_color = pick(&brush_colors);
        let brush_mat = lambert!({ color: b_color });
        let handle = mesh(pg("ToothbrushHolders__handle", box_geo!(0.008, 0.11, 0.008)), &brush_mat);
        handle.position().set(
            if brush_count == 2 {
                if b == 0 {
                    -0.014
                } else {
                    0.014
                }
            } else {
                0.0
            },
            shelf_y + 0.14,
            if brush_count == 2 {
                if b == 0 {
                    -0.008
                } else {
                    0.008
                }
            } else {
                0.0
            },
        );
        handle.rotation().z = (rnd() - 0.5) * 0.2; // slight tilt
        holder.add(&handle);
        // Bristle head.
        let head = mesh(pg("ToothbrushHolders__head", box_geo!(0.010, 0.020, 0.012)), &bristle_mat);
        head.position().copy(&handle.pos());
        head.position().y += 0.055;
        let hr = handle.rot();
        head.rotation().copy(&hr);
        holder.add(&head);
    }
    holder.rotation().y = face_wall(c.dx, c.dz);
    holder.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.050), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.050));
    g.world.add(&holder);
}
