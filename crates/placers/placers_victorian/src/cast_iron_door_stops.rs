//! `place_cast_iron_door_stops`
use game::prelude::*;
use placers_kit::place::open_tiles;
use std::collections::HashSet;

// Cast iron door-stop (floor): 4% chance per level. dog / boot / owl.
pub(crate) fn place_cast_iron_door_stops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let iron_colors = [0x181818, 0x2a1a10, 0x502010]; // fresh iron, rust-brown, rusted red
    let iron_mat = lambert!({ color: choice(&iron_colors) });
    let ds = group();
    // Base plate.
    let base = mesh(pg("CastIronDoorStops__base", box_geo!(0.11, 0.010, 0.09)), &iron_mat);
    base.position().y = 0.005;
    ds.add(&base);
    let variant = (rnd() * 3.0).floor() as i32;
    if variant == 0 {
        // Dog figurine — spaniel-like silhouette.
        let body = mesh(pg("CastIronDoorStops__dogBody", sphere_geo!(0.030, 8, 6)), &iron_mat);
        body.scale().set(1.5, 1.0, 0.9);
        body.position().set(0, 0.030, 0);
        ds.add(&body);
        let head = mesh(pg("CastIronDoorStops__dogHead", sphere_geo!(0.022, 8, 6)), &iron_mat);
        head.position().set(0.033, 0.045, 0);
        ds.add(&head);
        let snout = mesh(pg("CastIronDoorStops__dogMuzzle", cone_geo!(0.010, 0.018, 6)), &iron_mat);
        snout.rotation().z = -PI / 2.0;
        snout.position().set(0.055, 0.042, 0);
        ds.add(&snout);
        // 2 floppy ears.
        for side in [-1.0, 1.0] {
            let ear = mesh(pg("CastIronDoorStops__dogEar", sphere_geo!(0.008, 6, 5)), &iron_mat);
            ear.scale().set(0.5, 1.4, 0.7);
            ear.position().set(0.030, 0.040, side * 0.016);
            ds.add(&ear);
        }
        // 4 stubby legs.
        for lx in [-0.020, 0.020] {
            for lz in [-0.014, 0.014] {
                let leg = mesh(pg("CastIronDoorStops__dogLeg", cyl_geo!(0.006, 0.006, 0.020, 5)), &iron_mat);
                leg.position().set(lx, 0.020, lz);
                ds.add(&leg);
            }
        }
        // Tail (curled up).
        let tail = mesh(pg("CastIronDoorStops__dogTail", cyl_geo!(0.004, 0.004, 0.020, 4)), &iron_mat);
        tail.position().set(-0.030, 0.050, 0);
        tail.rotation().z = 0.6;
        ds.add(&tail);
        // 2 tiny eyes.
        for side in [-1.0, 1.0] {
            let eye = mesh(sphere_geo!(0.002, 4, 4), lambert!({ color: 0x0a0a0a }));
            eye.position().set(0.045, 0.052, side * 0.008);
            ds.add(&eye);
        }
    } else if variant == 1 {
        // Victorian boot.
        let foot = mesh(pg("CastIronDoorStops__bootFoot", box_geo!(0.075, 0.030, 0.038)), &iron_mat);
        foot.position().set(0, 0.025, 0);
        ds.add(&foot);
        let toe = mesh(pg("CastIronDoorStops__bootToe", sphere_geo!(0.018, 8, 6)), &iron_mat);
        toe.scale().set(1.0, 0.8, 0.9);
        toe.position().set(0.038, 0.0245, 0); // rests on the base plate (top 0.010)
        ds.add(&toe);
        let heel = mesh(pg("CastIronDoorStops__bootHeel", box_geo!(0.018, 0.014, 0.030)), &iron_mat);
        heel.position().set(-0.030, 0.017, 0);
        ds.add(&heel);
        let shin = mesh(pg("CastIronDoorStops__bootShaft", box_geo!(0.038, 0.075, 0.038)), &iron_mat);
        shin.position().set(-0.010, 0.075, 0);
        ds.add(&shin);
        // 5 tiny laces across the shaft's front face (+X, the toe side).
        for i in 0..5 {
            let lace = mesh(pg("CastIronDoorStops__lace", cyl_geo!(0.0015, 0.0015, 0.006, 4)), lambert!({ color: 0x080808 }));
            lace.rotation().x = PI / 2.0;
            lace.position().set(0.0095, 0.050 + i as f64 * 0.010, 0);
            ds.add(&lace);
        }
        // Boot cuff/top opening.
        let cuff = mesh(pg("CastIronDoorStops__bootCuff", torus_geo!(0.015, 0.003, 4, 10)), &iron_mat);
        cuff.rotation().x = PI / 2.0;
        cuff.position().set(-0.010, 0.115, 0);
        ds.add(&cuff);
    } else {
        // Owl figurine.
        let body = mesh(pg("CastIronDoorStops__owlBody", sphere_geo!(0.035, 10, 7)), &iron_mat);
        body.scale().set(1.0, 1.2, 0.9);
        body.position().set(0, 0.040, 0);
        ds.add(&body);
        let head = mesh(pg("CastIronDoorStops__owlHead", sphere_geo!(0.024, 10, 7)), &iron_mat);
        head.position().set(0, 0.080, 0);
        ds.add(&head);
        // 2 large staring owl eyes.
        for side in [-1.0, 1.0] {
            let eye_ring = mesh(pg("CastIronDoorStops__owlEyeRing", torus_geo!(0.008, 0.002, 4, 10)), &iron_mat);
            eye_ring.rotation().y = PI / 2.0;
            eye_ring.position().set(0.020, 0.083, side * 0.009);
            ds.add(&eye_ring);
            let eye_pupil = mesh(sphere_geo!(0.003, 5, 4), lambert!({ color: 0x0a0a0a }));
            eye_pupil.position().set(0.022, 0.083, side * 0.009);
            ds.add(&eye_pupil);
        }
        // Small triangular beak between eyes.
        let beak = mesh(pg("CastIronDoorStops__owlBeak", cone_geo!(0.004, 0.008, 4)), &iron_mat);
        beak.rotation().z = -PI / 2.0;
        beak.position().set(0.025, 0.076, 0);
        ds.add(&beak);
        // 2 wing outlines.
        for side in [-1.0, 1.0] {
            let wing = mesh(pg("CastIronDoorStops__owlWing", sphere_geo!(0.014, 6, 5)), &iron_mat);
            wing.scale().set(0.6, 1.2, 0.5);
            wing.position().set(0, 0.045, side * 0.028);
            ds.add(&wing);
        }
        // 2 small feet.
        for side in [-1.0, 1.0] {
            let foot = mesh(pg("CastIronDoorStops__owlFoot", cyl_geo!(0.005, 0.005, 0.008, 5)), &iron_mat);
            foot.position().set(0, 0.014, side * 0.010);
            ds.add(&foot);
        }
        // Ear tufts.
        for side in [-1.0, 1.0] {
            let tuft = mesh(pg("CastIronDoorStops__owlTuft", cone_geo!(0.004, 0.010, 4)), &iron_mat);
            tuft.position().set(-0.005, 0.098, side * 0.008);
            tuft.rotation().z = side * 0.3;
            ds.add(&tuft);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    ds.position().set(px, 0, pz);
    ds.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ds);
}
