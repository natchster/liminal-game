//! `place_hobby_horses`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Old wooden hobby horse child's toy (floor): 3% chance per level.
pub(crate) fn place_hobby_horses(g: &mut Game, _dist: &[i32]) {
    g.pp.hobby_horse_wheeled_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // 4-colour wood pool.
    let wood_colors = [0x604020, 0x2a1810, 0x8b5a2b, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    // 4-colour mane/hair pool (yarn or horsehair).
    let mane_colors = [0x181818, 0x604020, 0xd0a060, 0x8b5a2b];
    let mane_mat = lambert!({ color: choice(&mane_colors) });
    // Rope reins colour.
    let reins_mat = lambert!({ color: 0xa08050 });
    let hh = group();
    // Pole — long horizontal cylinder laid on floor.
    let pole_len = 0.80;
    let pole = mesh(pg("HobbyHorses__pole", cyl_geo!(0.010, 0.010, pole_len, 6)), &wood_mat);
    pole.rotation().z = PI / 2.0;
    pole.position().set(0, 0.014, 0); // resting on the knob
    hh.add(&pole);
    // Small end cap / handle on trailing end.
    let handle = mesh(pg("HobbyHorses__knob", sphere_geo!(0.014, 6, 5)), &wood_mat);
    handle.position().set(-pole_len / 2.0, 0.014, 0);
    hh.add(&handle);
    // Horse head at leading end — angled up slightly, standing on the back
    // corner of its jaw.
    let head_group = group();
    head_group.position().set(pole_len / 2.0, 0.0412, 0);
    head_group.rotation().z = 0.25; // nose up
    hh.add(&head_group);
    // Main head shape.
    let head_body = mesh(pg("HobbyHorses__head", box_geo!(0.14, 0.08, 0.05)), &wood_mat);
    head_body.position().set(0.06, 0, 0);
    head_group.add(&head_body);
    // Snout/nose.
    let snout = mesh(pg("HobbyHorses__muzzle", box_geo!(0.05, 0.055, 0.04)), &wood_mat);
    snout.position().set(0.14, -0.010, 0);
    head_group.add(&snout);
    // 2 ears.
    for side in [-1.0, 1.0] {
        let ear = mesh(pg("HobbyHorses__ear", cone_geo!(0.008, 0.030, 4)), &wood_mat);
        ear.position().set(0.02, 0.040, side * 0.015);
        ear.rotation().z = -0.3;
        head_group.add(&ear);
    }
    // 2 painted/carved eye dots.
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.003, 5, 4)), lambert!({ color: 0x080808 }));
        eye.position().set(0.10, 0.010, side * 0.026);
        head_group.add(&eye);
    }
    // 2 painted nostril dots on snout.
    for side in [-1.0, 1.0] {
        let nostril = mesh(sphere_geo!(0.002, 4, 4), lambert!({ color: 0x1a1010 }));
        nostril.position().set(0.16, -0.015, side * 0.010);
        head_group.add(&nostril);
    }
    // Mane — 8-12 small yarn strands hung from the crest down one side,
    // hanging plumb in the world (-0.25 undoes the head tilt).
    let mane_strand_count = 8 + (rnd() * 5.0).floor() as i32;
    for i in 0..mane_strand_count {
        let strand_len = 0.030 + rnd() * 0.025;
        let strand = mesh(pg("HobbyHorses__strand", cyl_geo!(0.0015, 0.0015, strand_len, 3)), &mane_mat);
        let along = 0.015 + (i as f64 - mane_strand_count as f64 / 2.0) * 0.004 * rnd();
        let side_z = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let hang = -0.25 + (rnd() - 0.5) * 0.5;
        strand.rotation().z = hang;
        strand.position().set(along + (strand_len / 2.0) * hang.sin(), 0.036 - (strand_len / 2.0) * hang.cos(), side_z * 0.0265);
        head_group.add(&strand);
    }
    // Rope reins — from snout side down to the floor.
    for side in [-1.0, 1.0] {
        // Attachment ring on snout side.
        let ring = mesh(pg("HobbyHorses__ring", torus_geo!(0.004, 0.001, 3, 6)), &reins_mat);
        ring.rotation().y = PI / 2.0;
        ring.position().set(0.15, -0.010, side * 0.025);
        head_group.add(&ring);
        // Rein cord from the ring back along the cheek to the floor.
        let rein = mesh(pg("HobbyHorses__rein", cyl_geo!(0.0015, 0.0015, 0.10, 3)), &reins_mat);
        rein.rotation().z = -1.084;
        rein.position().set(0.1058, -0.0334, side * 0.027);
        head_group.add(&rein);
    }
    // 30%: small wooden wheel at trailing end.
    let is_wheeled = rnd() < 0.3;
    if is_wheeled {
        for side in [-1.0, 1.0] {
            let wheel = mesh(pg("HobbyHorses__wheel", cyl_geo!(0.025, 0.025, 0.010, 10)), &wood_mat);
            wheel.rotation().x = PI / 2.0; // axle across the pole (Z)
            wheel.position().set(-pole_len / 2.0 + 0.030, 0.025, side * 0.020);
            hh.add(&wheel);
            // Small axle.
            let axle = mesh(pg("HobbyHorses__axle", cyl_geo!(0.002, 0.002, 0.050, 4)), &wood_mat);
            axle.rotation().x = PI / 2.0;
            axle.position().set(-pole_len / 2.0 + 0.030, 0.025, 0);
            hh.add(&axle);
        }
        // the pole's trailing end rides on the axle
        pole.position().y = 0.025;
        handle.position().y = 0.025;
    }
    // 40%: dust patina on pole.
    if rnd() < 0.4 {
        let dust_mat = lambert!({ color: 0x605030, transparent: true, opacity: 0.35 });
        let dust = mesh(cyl_geo!(0.012, 0.012, 0.15, 6), &dust_mat);
        dust.rotation().z = PI / 2.0;
        dust.position().set(0, pole.pos().y, 0); // round the pole
        hh.add(&dust);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    hh.position().set(px, 0, pz);
    hh.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hh);
    if is_wheeled {
        let p = hh.pos();
        g.pp.hobby_horse_wheeled_positions.push(v3(p.x, 0, p.z));
    }
}
