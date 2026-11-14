//! `place_dead_moths`
use crate::shared::buffer_geo;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Dead moth: 5% chance per level. Small dead moth on floor.
pub(crate) fn place_dead_moths(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wing_mat = lambert!({ color: 0xa08a70, side: DoubleSide });
    let body_mat = lambert!({ color: 0x302010 });
    let moth = group();
    // Body: tiny elongated cylinder.
    let body = mesh(pg("DeadMoths__body", cyl_geo!(0.002, 0.001, 0.014, 4)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = 0.003;
    moth.add(&body);
    // 2 wings (triangle planes) laid flat.
    for s in [-1.0, 1.0] {
        let wing_geo = buffer_geo(&[0.0, 0.0, 0.0, 0.008, 0.0, s * 0.012, -0.008, 0.0, s * 0.012], vec![0, 1, 2]);
        let wing = mesh(wing_geo, &wing_mat);
        wing.position().y = 0.002;
        moth.add(&wing);
    }
    // 2 tiny antennae.
    for s in [-1.0, 1.0] {
        let ant = mesh(cyl_geo!(0.0002, 0.0002, 0.006, 3), &body_mat);
        ant.position().set(0.008, 0.003, s * 0.001);
        ant.rotation().z = 0.6;
        ant.rotation().y = s * 0.3;
        moth.add(&ant);
    }
    scatter(g, &moth, x, z, 2.4);
    g.world.add(&moth);
}
