//! `place_elevator_buttons`
use game::prelude::*;
use placers_kit::place::mount_slid as mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Elevator call button: 4% chance per level. Small panel
// with up/down arrows + LED.
pub(crate) fn place_elevator_buttons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let panel_mat = lambert!({ color: 0x808088 });
    let button_mat = lambert!({ color: 0x181410 });
    let led_mat = basic_mat!({ color: 0xd82040 });
    let panel = group();
    // Panel plate.
    let plate = mesh(pg("ElevatorButtons__plate", box_geo!(0.06, 0.12, 0.012)), &panel_mat);
    plate.position().set(0, 0, -0.006);
    panel.add(&plate);
    // 2 buttons (up + down triangles).
    for (by, ay) in [(0.025, PI), (-0.025, 0.0)] {
        let btn = mesh(pg("ElevatorButtons__button", cyl_geo!(0.016, 0.016, 0.005, 8)), &button_mat);
        btn.rotation().x = PI / 2.0;
        btn.position().set(0, by, 0.002);
        panel.add(&btn);
        // Triangle glyph.
        let glyph_geo = Geo::new();
        let gs: f64 = 0.008;
        let verts: Vec<f32> = vec![0.0, gs as f32, 0.0, -gs as f32, -gs as f32, 0.0, gs as f32, -gs as f32, 0.0];
        glyph_geo.set_position(verts);
        glyph_geo.set_index(vec![0, 1, 2]);
        glyph_geo.compute_vertex_normals();
        let glyph = mesh(glyph_geo, basic_mat!({ color: 0xe0e0e0 }));
        glyph.position().set(0, by, 0.005);
        glyph.rotation().z = ay;
        panel.add(&glyph);
    }
    // Small red LED.
    let led = mesh(sphere_geo!(0.003, 5, 4), &led_mat);
    led.position().set(0.020, 0, 0.004);
    panel.add(&led);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&panel, c, wall_x, wall_z, 1.30, 0.6);
    g.world.add(&panel);
}
