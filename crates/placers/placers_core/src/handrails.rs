//! `place_handrails`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Handrail: 0-2 wall-mounted horizontal metal bars per level.
pub(crate) fn place_handrails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.60 {
        return;
    } // 40% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut cands = wall_candidates(g);
    shuffle(&mut cands);
    let rail_mat = lambert!({ color: 0x8a8880 });
    let bracket_mat = lambert!({ color: 0x5a5850 });
    let mut i = 0;
    while i < count && i < cands.len() {
        let c = cands[i];
        // Rail: 2.8-3.4 m long along wall.
        let rail_len = 2.8 + rnd() * 0.6;
        let rail = mesh(pg("Handrails__rail", cyl_geo!(0.02, 0.02, rail_len, 8)), &rail_mat);
        if c.dx.abs() == 1 {
            rail.rotation().x = PI / 2.0; // Y-axis cylinder → Z-axis
        } else {
            rail.rotation().z = PI / 2.0; // Y-axis cylinder → X-axis
        }
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.08);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.08);
        let rail_height = 0.9;
        if c.dx.abs() == 1 {
            rail.position().set(wall_x, rail_height, g.wz(c.z));
        } else {
            rail.position().set(g.wx(c.x), rail_height, wall_z);
        }
        g.world.add(&rail);
        // 2 brackets attaching rail to wall.
        for bs in [-1.0, 1.0] {
            let bracket = mesh(pg("Handrails__bracket", box_geo!(0.03, 0.06, 0.06)), &bracket_mat);
            // block runs from the wall to 6 mm into the rail's back
            let b_off = CELL * 0.5 - 0.033;
            bracket.scale().z = 1.1;
            if c.dx.abs() == 1 {
                bracket.rotation().y = PI / 2.0;
                bracket.position().set(g.wx(c.x) + c.dx as f64 * b_off, rail_height, g.wz(c.z) + bs * (rail_len / 2.0 - 0.15));
            } else {
                bracket.position().set(g.wx(c.x) + bs * (rail_len / 2.0 - 0.15), rail_height, g.wz(c.z) + c.dz as f64 * b_off);
            }
            g.world.add(&bracket);
        }
        // Worn-spot variant: 15% chance a darker band at the rail midpoint.
        if rnd() < 0.15 {
            let worn_mat = lambert!({ color: 0x4a4640 });
            let worn = mesh(pg("Handrails__worn", cyl_geo!(0.022, 0.022, 0.30, 8)), &worn_mat);
            if c.dx.abs() == 1 {
                worn.rotation().x = PI / 2.0;
            } else {
                worn.rotation().z = PI / 2.0;
            }
            worn.position().copy(&rail.pos());
            g.world.add(&worn);
        }
        i += 1;
    }
}
