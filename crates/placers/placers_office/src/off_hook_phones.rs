//! `place_off_hook_phones`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Telephone off-hook w/ dial tone hum (floor): 4% chance per level.
pub(crate) fn place_off_hook_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.off_hook_phone_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let phone_colors = [0x101010, 0xa02020, 0x2060a0, 0x604020, 0xc0a060];
    let phone_mat = lambert!({ color: pick(&phone_colors) });
    let dial_mat = lambert!({ color: 0xe0e0d0 });
    let cord_mat = lambert!({ color: 0x201010 });
    let p = group();
    // Base — rounded rectangular box.
    let base = mesh(pg("OffHookPhones__base", box_geo!(0.14, 0.028, 0.10)), &phone_mat);
    base.position().y = 0.014;
    p.add(&base);
    // Dial disc — flat cylinder on top.
    let dial = mesh(pg("OffHookPhones__dial", cyl_geo!(0.038, 0.038, 0.006, 12)), &dial_mat);
    dial.position().set(-0.02, 0.031, 0);
    p.add(&dial);
    // Dial center hole.
    let dial_hole = mesh(pg("OffHookPhones__hub", cyl_geo!(0.006, 0.006, 0.007, 8)), &phone_mat);
    dial_hole.position().set(-0.02, 0.032, 0);
    p.add(&dial_hole);
    // 10 finger holes around dial edge.
    for i in 0..10 {
        let ang = (i as f64 / 10.0) * PI * 2.0;
        let hole = mesh(pg("OffHookPhones__fingerhole", cyl_geo!(0.0038, 0.0038, 0.007, 5)), &phone_mat);
        hole.position().set(-0.02 + ang.dcos() * 0.028, 0.033, ang.dsin() * 0.028);
        p.add(&hole);
    }
    // 2 hook cradle bumps on top (empty — handset is off).
    for off in [-0.045, 0.045] {
        let bump = mesh(pg("OffHookPhones__prong", sphere_geo!(0.005, 5, 4)), &phone_mat);
        bump.position().set(0.050, 0.030, off);
        p.add(&bump);
    }
    // Handset — lying beside base.
    let (handset_off_x, handset_off_z) = (0.15, 0.06);
    let handset = group();
    // Earpiece cylinder + shaft + mouthpiece cylinder.
    let earpiece = mesh(pg("OffHookPhones__earcup", cyl_geo!(0.015, 0.015, 0.020, 8)), &phone_mat);
    earpiece.rotation().z = PI / 2.0;
    earpiece.position().set(-0.045, 0, 0);
    handset.add(&earpiece);
    let shaft = mesh(pg("OffHookPhones__grip", box_geo!(0.08, 0.014, 0.020)), &phone_mat);
    shaft.position().set(0, 0, 0);
    handset.add(&shaft);
    let mouthpiece = mesh(pg("OffHookPhones__mouthcup", cyl_geo!(0.015, 0.015, 0.020, 8)), &phone_mat);
    mouthpiece.rotation().z = PI / 2.0;
    mouthpiece.position().set(0.045, 0, 0);
    handset.add(&mouthpiece);
    // Small speaker holes on earpiece + mouthpiece (tiny dark discs).
    let speaker_mat = lambert!({ color: 0x0a0a0a });
    for (xoff, dir) in [(-0.055, -1.0), (0.055, 1.0)] {
        let speaker = mesh(circle_geo!(0.010, 8), &speaker_mat);
        speaker.rotation().y = dir * PI / 2.0;
        speaker.position().set(xoff, 0, 0);
        handset.add(&speaker);
    }
    handset.position().set(handset_off_x, 0.015, handset_off_z);
    handset.rotation().y = 0.4;
    p.add(&handset);
    // Curly cord connecting phone to handset — 8 tiny torus rings in a chain.
    let (cord_start_x, cord_start_z) = (0.06, 0.045);
    for i in 0..8 {
        let t = i as f64 / 7.0;
        let cx = cord_start_x + (handset_off_x - cord_start_x - 0.04) * t;
        let cz = cord_start_z + (handset_off_z - cord_start_z) * t;
        let cy = 0.010 + (t * PI).dsin() * 0.010;
        let ring = mesh(pg("OffHookPhones__cord", torus_geo!(0.005, 0.001, 3, 8)), &cord_mat);
        ring.rotation().z = PI / 2.0;
        ring.position().set(cx, cy, cz);
        p.add(&ring);
    }
    p.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    p.rotation().y = rnd() * PI * 2.0;
    g.world.add(&p);
    // Track position — for handset dial-tone audio.
    let (pp, ry) = (p.pos(), p.rot().y);
    let handset_world_x = pp.x + handset_off_x * ry.dcos() - handset_off_z * ry.dsin();
    let handset_world_z = pp.z + handset_off_x * ry.dsin() + handset_off_z * ry.dcos();
    g.pp.off_hook_phone_positions.push(v3(handset_world_x, 0, handset_world_z));
}
