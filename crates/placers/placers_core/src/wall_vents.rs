//! `place_wall_vents`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Wall vents: 0-2 rusted grille boxes per level on random walls
// at head height (1.6-2.1 m).
pub(crate) fn place_wall_vents(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.45 {
        return;
    } // 45% chance NO vents this level
    let count = 1 + (rnd() * 2.0).floor() as usize; // 1-2
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let frame_color = vent_color_by_weather(g.weather_key());
    let frame_mat = lambert!({ color: frame_color });
    let opening_mat = basic_mat!({ color: 0x080604 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.03);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.03);
        let mid_height = 1.6 + rnd() * 0.5; // 1.6 - 2.1 m
                                            // Vent frame (metallic box, proud of wall).
        let frame = mesh(pg("WallVents__grille", box_geo!(0.30, 0.20, 0.04)), &frame_mat);
        // Recessed opening (darker inset).
        let opening = mesh(plane_geo!(0.24, 0.14), &opening_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            // Position opening just proud of the frame face.
            opening.position().set(wall_x + c.dx as f64 * 0.021, mid_height, wall_z);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            opening.position().set(wall_x, mid_height, wall_z + c.dz as f64 * 0.021);
        }
        g.world.add(&frame);
        g.world.add(&opening);
        i += 1;
    }
}
