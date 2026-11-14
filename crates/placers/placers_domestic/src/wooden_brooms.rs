//! `place_wooden_brooms`
use game::prelude::*;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wooden broom: 12% chance per level. Thin wooden pole +
// bristle head, leaning against wall.
pub(crate) fn place_wooden_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let pole_mat = lambert!({ color: 0x6a4820 });
    let bristle_mat = lambert!({ color: 0x3a2818 });
    let broom = group();
    // Pole: thin vertical cylinder.
    let pole = mesh(pg("WoodenBrooms__pole", cyl_geo!(0.012, 0.012, 1.3, 6)), &pole_mat);
    pole.position().y = 0.72;
    broom.add(&pole);
    // Bristle head: flat rectangular box at bottom.
    let head = mesh(pg("WoodenBrooms__bristles", box_geo!(0.20, 0.08, 0.05)), &bristle_mat);
    head.position().y = 0.06;
    broom.add(&head);
    // Small connector wood block between pole and head.
    let connector = mesh(pg("WoodenBrooms__block", box_geo!(0.05, 0.03, 0.05)), &pole_mat);
    connector.position().y = 0.11;
    broom.add(&connector);
    // Lean toward the wall (local -Z after the yaw; order YXZ applies the lean
    // before the yaw). The foot stands out so the pole's top rests on the wall.
    let lean = 0.12 + rnd() * 0.08;
    broom.set_rotation_order(EulerOrder::YXZ);
    broom.rotation().x = -lean;
    let foot_off = 1.37 * lean.sin() + 0.012 * lean.cos();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - foot_off);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - foot_off);
    if c.dx.abs() == 1 {
        broom.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        broom.position().set(wall_x, 0, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        broom.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        broom.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    // Head (bottom at 0.02) down on the floor: its wall-side bottom edge is lowest.
    broom.position().y = -(0.02 * lean.cos() - 0.025 * lean.sin());
    g.world.add(&broom);
    // 25% chance: small dark dust pile beside bristle head.
    if rnd() < 0.25 {
        let dust_mat = lambert!({ color: 0x3a3028 });
        let dust_pile = mesh(pg("WoodenBrooms__dust", sphere_geo!(0.05 + rnd() * 0.03, 6, 4)), &dust_mat);
        dust_pile.scale().set(1.2, 0.3, 1.5); // wide flat pile
                                              // Place beside broom in front of bristle, forward from wall.
        let dfx = -c.dx as f64 * 0.20;
        let dfz = -c.dz as f64 * 0.20;
        let bp = broom.pos();
        dust_pile.position().set(bp.x + dfx + (rnd() - 0.5) * 0.15, 0.015, bp.z + dfz + (rnd() - 0.5) * 0.15);
        g.world.add(&dust_pile);
    }
}
