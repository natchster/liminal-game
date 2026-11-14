//! `place_cardboard_boxes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Cardboard boxes: 0-3 boxes per level (30% chance).
pub(crate) fn place_cardboard_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.cardboard_box_positions.clear(); // reset per level
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let box_mat = lambert!({ color: 0x9a7248 });
    let tape_mat = lambert!({ color: 0xd8ccb0 });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // Box dimensions vary — small shipping box scale.
        let sx = 0.30 + rnd() * 0.20;
        let sy = 0.22 + rnd() * 0.15;
        let sz = 0.30 + rnd() * 0.20;
        let bx = group();
        let body = mesh(pg("CardboardBoxes__body", box_geo!(sx, sy, sz)), &box_mat);
        body.position().y = sy / 2.0;
        bx.add(&body);
        // Packing tape stripe across top (30% chance).
        if rnd() < 0.30 {
            let tape = mesh(plane_geo!(sx * 0.9, 0.04), &tape_mat);
            tape.rotation().x = -PI / 2.0;
            tape.position().y = sy + 0.001;
            bx.add(&tape);
        }
        // 25% chance tipped on side.
        if rnd() < 0.25 {
            bx.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
            bx.position().y = sx / 2.0; // approximate rest height
        }
        let ox = (rnd() - 0.5) * 2.2;
        let oz = (rnd() - 0.5) * 2.2;
        bx.position().x = g.wx(x) + ox;
        bx.position().z = g.wz(z) + oz;
        bx.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bx);
        // Track for shifting-audio events.
        let p = bx.pos();
        g.pp.cardboard_box_positions.push(v3(p.x, sy / 2.0, p.z));
        i += 1;
    }
}
