//! `place_cassette_tapes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Cassette tape: 8% chance per level. Rectangle w/ 2 spool holes + label.
pub(crate) fn place_cassette_tapes(g: &mut Game, _dist: &[i32]) {
    g.pp.cassette_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_colors = [0x181410, 0x201810, 0x4a2818, 0x503060];
    let body_mat = lambert!({ color: pick_hex(&body_colors) });
    let label_colors = [0xf0ece0, 0xd8b040, 0xa02040, 0xd0d0d0];
    let label_mat = lambert!({ color: pick_hex(&label_colors) });
    let hole_mat = basic_mat!({ color: 0x080604 });
    let cass = group();
    // Body: rectangular block.
    let body = mesh(pg("CassetteTapes__body", box_geo!(0.10, 0.010, 0.06)), &body_mat);
    body.position().y = 0.005;
    cass.add(&body);
    // Label on top.
    let label = mesh(plane_geo!(0.08, 0.035), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.011, -0.010);
    cass.add(&label);
    // 2 spool holes.
    for s in [-1.0, 1.0] {
        let hole = mesh(circle_geo!(0.008, 8), &hole_mat);
        hole.rotation().x = -PI / 2.0;
        hole.position().set(s * 0.024, 0.011, 0.014);
        cass.add(&hole);
    }
    scatter(g, &cass, x, z, 2.4);
    g.world.add(&cass);
    // Track for hiss audio.
    let p = cass.pos();
    g.pp.cassette_positions.push(v3(p.x, 0.01, p.z));
}
