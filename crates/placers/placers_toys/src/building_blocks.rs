//! `place_building_blocks`: Small building block: 10% chance per level. Colored cube with 4 stud dots on top (LEGO-style silhouette).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small building block: 10% chance per level. Colored cube with
/// 4 stud dots on top (LEGO-style silhouette).
pub(crate) fn place_building_blocks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let block_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xf0a020];
    let block_mat = lambert!({ color: pick(&block_colors) });
    let block = group();
    // Body: cube.
    let body = mesh(pg("BuildingBlocks__body", box_geo!(0.06, 0.03, 0.06)), &block_mat);
    body.position().y = 0.015;
    block.add(&body);
    // 4 stud dots on top (2×2 grid).
    for sx_off in [-0.015, 0.015] {
        for sz_off in [-0.015, 0.015] {
            let stud = mesh(pg("BuildingBlocks__stud", cyl_geo!(0.008, 0.008, 0.006, 6)), &block_mat);
            stud.position().set(sx_off, 0.033, sz_off);
            block.add(&stud);
        }
    }
    block.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    block.rotation().y = rnd() * PI * 2.0;
    g.world.add(&block);
    // 25% chance: 1-2 sibling blocks stacked on top.
    if rnd() < 0.25 {
        let stack_count = 1 + (rnd() * 2.0).floor() as i32;
        for s in 1..=stack_count {
            let sib_col = pick(&block_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_block = block.duplicate();
            // Replace materials on all children.
            sib_block.traverse(&mut |child| {
                if child.is_mesh() {
                    child.set_mat(&sib_mat);
                }
            });
            sib_block.position().y = 0.036 * s as f64; // stack height per block
            sib_block.position().x += (rnd() - 0.5) * 0.015;
            sib_block.position().z += (rnd() - 0.5) * 0.015;
            sib_block.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_block);
        }
    }
}
