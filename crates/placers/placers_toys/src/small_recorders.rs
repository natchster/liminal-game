//! `place_small_recorders`: Small recorder: 8% chance per level. Beige recorder flute with mouthpiece + 5 finger holes. Child-music family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small recorder: 8% chance per level. Beige recorder flute
/// with mouthpiece + 5 finger holes. Child-music family.
pub(crate) fn place_small_recorders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xf0e0c8, 0xe8d0a8, 0xd8b890, 0x201a12];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let hole_mat = lambert!({ color: 0x100a08 });
    let recorder = group();
    // Body: horizontal cylinder.
    let body_len = 0.24;
    let body = mesh(pg("SmallRecorders__body", cyl_geo!(0.012, 0.012, body_len, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(body_len / 2.0, 0.015, 0);
    recorder.add(&body);
    // Mouthpiece: slight flare at near end.
    let mouth = mesh(pg("SmallRecorders__beak", cyl_geo!(0.014, 0.010, 0.020, 8)), &body_mat);
    mouth.rotation().z = PI / 2.0;
    mouth.position().set(-0.005, 0.015, 0);
    recorder.add(&mouth);
    // Bell flare at far end.
    let bell = mesh(pg("SmallRecorders__foot", cyl_geo!(0.016, 0.012, 0.020, 8)), &body_mat);
    bell.rotation().z = PI / 2.0;
    bell.position().set(body_len - 0.005, 0.015, 0);
    recorder.add(&bell);
    // 5 finger holes on top.
    for i in 0..5 {
        let hole = mesh(circle_geo!(0.003, 8), &hole_mat);
        hole.rotation().x = -PI / 2.0;
        hole.position().set(0.06 + i as f64 * 0.028, 0.0255, 0); // in the body's finger-hole recesses
        recorder.add(&hole);
    }
    recorder.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    recorder.rotation().y = rnd() * PI * 2.0;
    g.world.add(&recorder);
}
