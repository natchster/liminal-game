//! `place_leaning_frames`
use game::prelude::*;
use placers_kit::place::ri;
use placers_kit::place::wall_candidates;

// Empty picture frame leaning against wall: 15% chance per
// level. 4 thin frame planes + hollow center.
pub(crate) fn place_leaning_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    let frame_mat = lambert!({ color: 0x4a3018 });
    let back_mat = lambert!({ color: 0x1a1210 });
    let frame = group();
    // Overall frame 0.30 wide × 0.42 tall × 0.03 deep.
    let (fw, fh, bar) = (0.30, 0.42, 0.025);
    // 4 thin border bars.
    for (sx, sy, w, h) in [
        (0.0, fh / 2.0 - bar / 2.0, fw, bar),              // top
        (0.0, -fh / 2.0 + bar / 2.0, fw, bar),             // bottom
        (-fw / 2.0 + bar / 2.0, 0.0, bar, fh - 2.0 * bar), // left
        (fw / 2.0 - bar / 2.0, 0.0, bar, fh - 2.0 * bar),  // right
    ] {
        let seg = mesh(box_geo!(w, h, 0.03), &frame_mat);
        seg.position().set(sx, sy, 0);
        frame.add(&seg);
    }
    // Dark backing plane inset (visible through empty center).
    let backing = mesh(pg("LeaningFrames__backing", plane_geo!(fw - 2.0 * bar - 0.01, fh - 2.0 * bar - 0.01)), &back_mat);
    backing.position().z = -0.005;
    frame.add(&backing);
    // Lean frame against wall: tilt backward ~10-15°.
    let lean_angle = 0.20 + rnd() * 0.10;
    // Lean about the frame's own horizontal axis (order YXZ), pivoting on the
    // bottom-back edge: that edge rests on the floor and the top-back edge
    // (half height fh/2, half depth 0.015) touches the wall.
    frame.set_rotation_order(EulerOrder::YXZ);
    let lean_y = (fh / 2.0) * lean_angle.cos() + 0.015 * lean_angle.sin();
    let lean_off = (fh / 2.0) * lean_angle.sin() + 0.015 * lean_angle.cos();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - lean_off);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - lean_off);
    if c.dx.abs() == 1 {
        frame.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        // Tilt so top leans TOWARD the wall (rotate around X of local frame).
        frame.rotation().x = -lean_angle * (if c.dx > 0 { 1.0 } else { 1.0 });
        frame.position().set(wall_x, lean_y, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        frame.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        frame.rotation().x = -lean_angle;
        frame.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, lean_y, wall_z);
    }
    g.world.add(&frame);
}
