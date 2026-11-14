//! `place_small_radios`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Small transistor radio: 6% chance per level. Small box
// + dial + speaker grille + antenna.
pub(crate) fn place_small_radios(g: &mut Game, _dist: &[i32]) {
    g.pp.radio_positions = vec![]; // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let case_colors = [0x8a5030, 0x604020, 0x181410, 0xa02040];
    let case_mat = lambert!({ color: choice(&case_colors) });
    let grille_mat = lambert!({ color: 0x2a2418 });
    let dial_mat = lambert!({ color: 0xe8dcc0 });
    let ant_mat = lambert!({ color: 0xc0c0c0 });
    let radio = group();
    // Body.
    let body = mesh(pg("SmallRadios__body", box_geo!(0.12, 0.07, 0.04)), &case_mat);
    body.position().y = 0.035;
    radio.add(&body);
    // Speaker grille (dark disc on front-left).
    let grille = mesh(circle_geo!(0.024, 12), &grille_mat);
    grille.position().set(-0.030, 0.035, 0.021);
    radio.add(&grille);
    // Dial (small cream rectangle on front-right).
    let dial = mesh(plane_geo!(0.036, 0.020), &dial_mat);
    dial.position().set(0.030, 0.045, 0.021);
    radio.add(&dial);
    // Tuning knob.
    let knob = mesh(pg("SmallRadios__knob", cyl_geo!(0.006, 0.006, 0.005, 8)), &grille_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0.045, 0.020, 0.020);
    radio.add(&knob);
    // Antenna (extended from top).
    let ant = mesh(pg("SmallRadios__antenna", cyl_geo!(0.001, 0.001, 0.12, 4)), &ant_mat);
    ant.rotation().z = 0.4;
    // lower end (centre + (0.06 sin 0.4, −0.06 cos 0.4)) 2 mm into the case top
    // near its right end at (0.05, 0.068); it floated ~5 mm off the corner
    ant.position().set(0.05 - 0.06 * 0.4f64.sin(), 0.068 + 0.06 * 0.4f64.cos(), 0);
    radio.add(&ant);
    scatter(g, &radio, x, z, 2.4);
    g.world.add(&radio);
    // Track for AM static audio.
    let p = radio.pos();
    g.pp.radio_positions.push(v3(p.x, 0.035, p.z));
}
