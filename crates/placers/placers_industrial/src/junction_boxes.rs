//! `place_junction_boxes`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Wall junction boxes: 20% chance per level, 1-2 units with conduits.
pub(crate) fn place_junction_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.junction_box_positions.clear(); // reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let box_mat = lambert!({ color: 0x6a6a68 });
    let conduit_mat = lambert!({ color: 0x4a4a48 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let jbox = group();
        // Main box: 0.16 wide × 0.20 tall × 0.06 deep.
        let bx = mesh(pg("JunctionBoxes__box", box_geo!(0.16, 0.20, 0.06)), &box_mat);
        jbox.add(&bx);
        // Small hex screws on 4 corners.
        for (sx, sy) in [(-0.06, 0.08), (0.06, 0.08), (-0.06, -0.08), (0.06, -0.08)] {
            let screw = mesh(pg("JunctionBoxes__screw", cyl_geo!(0.005, 0.005, 0.005, 6)), &conduit_mat);
            screw.rotation().x = PI / 2.0;
            screw.position().set(sx, sy, 0.032);
            jbox.add(&screw);
        }
        // 2-3 conduits leaving the box (dir, y, len).
        let conduit_dirs = [
            ("up", 0.095, 0.35), // starts in the box top (0.10), not 5 cm above it
            ("left", 0.02, 0.28),
            ("right", 0.02, 0.28),
        ];
        // Always up-conduit; 60% chance of side conduits.
        let mut chosen = vec![conduit_dirs[0]];
        if rnd() < 0.6 {
            chosen.push(conduit_dirs[1]);
        }
        if rnd() < 0.6 {
            chosen.push(conduit_dirs[2]);
        }
        for &(dir, cy, len) in &chosen {
            let conduit = mesh(pg("JunctionBoxes__conduit", cyl_geo!(0.015, 0.015, len, 6)), &conduit_mat);
            if dir == "up" {
                conduit.position().set(0, cy + len / 2.0, 0.015);
            } else {
                conduit.rotation().z = PI / 2.0;
                conduit.position().set((if dir == "left" { -1.0 } else { 1.0 }) * (0.08 + len / 2.0), cy, 0.015);
            }
            jbox.add(&conduit);
        }
        // 25% chance: torn wire dangling from bottom of junction box.
        if rnd() < 0.25 {
            let wire_mat = lambert!({ color: 0x181410 });
            let copper_mat = lambert!({ color: 0xa87028 });
            let wire_len = 0.18 + rnd() * 0.22;
            let wire = mesh(pg("JunctionBoxes__wire", cyl_geo!(0.004, 0.004, wire_len, 5)), &wire_mat);
            // Slight tilt so wire hangs "naturally".
            let tilt_z = (rnd() - 0.5) * 0.25;
            wire.rotation().z = tilt_z;
            let wire_drop_x = tilt_z.sin() * (wire_len / 2.0);
            let wpx = (rnd() - 0.5) * 0.08 - wire_drop_x;
            wire.position().set(wpx, -0.10 - wire_len / 2.0, 0.015);
            jbox.add(&wire);
            // Exposed copper tip at bottom.
            let tip = mesh(pg("JunctionBoxes__tip", cyl_geo!(0.005, 0.005, 0.02, 5)), &copper_mat);
            tip.rotation().z = tilt_z;
            let wp = wire.pos();
            tip.position().set(wp.x + tilt_z.sin() * (wire_len / 2.0 + 0.01), wp.y - tilt_z.cos() * (wire_len / 2.0 + 0.01), 0.015);
            jbox.add(&tip);
        }
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.035);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.035);
        let jb_height = 1.6 + rnd() * 0.5;
        if c.dx.abs() == 1 {
            jbox.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            jbox.position().set(wall_x, jb_height, wall_z + (rnd() - 0.5) * 1.2);
        } else {
            jbox.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            jbox.position().set(wall_x + (rnd() - 0.5) * 1.2, jb_height, wall_z);
        }
        g.world.add(&jbox);
        // Track for arc audio events.
        let jp = jbox.pos();
        g.pp.junction_box_positions.push(v3(jp.x, jb_height, jp.z));
        i += 1;
    }
}
