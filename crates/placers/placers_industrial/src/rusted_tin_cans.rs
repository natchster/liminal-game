//! `place_rusted_tin_cans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;

// Rusted tin can: 8% chance per level. Small rusted can, sometimes tipped.
pub(crate) fn place_rusted_tin_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let can_colors = [0x6a4028, 0x804830, 0x8a3020, 0x503020];
    let can_mat = lambert!({ color: pick_hex(&can_colors) });
    let label_mat = lambert!({ color: 0xa08a70 });
    let can = group();
    // Can body.
    let body = mesh(pg("RustedTinCans__body", cyl_geo!(0.028, 0.028, 0.075, 12)), &can_mat);
    body.position().y = 0.038;
    can.add(&body);
    // Faded paper label band.
    let label = mesh(pg("RustedTinCans__label", cyl_geo!(0.029, 0.029, 0.035, 12, 1, true)), &label_mat);
    label.position().y = 0.038;
    can.add(&label);
    // Rim discs top + bottom.
    for yy in [0.003, 0.072] {
        let rim = mesh(cyl_geo!(0.028, 0.028, 0.002, 12), &can_mat);
        rim.position().y = yy;
        can.add(&rim);
    }
    // 40% tipped on side.
    if rnd() < 0.40 {
        can.rotation().z = PI / 2.0;
        can.position().y = 0.028;
    }
    can.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    can.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}
