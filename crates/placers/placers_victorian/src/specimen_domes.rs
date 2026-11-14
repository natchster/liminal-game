//! `place_specimen_domes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Victorian glass-dome specimen display (floor): 3% chance per level.
pub(crate) fn place_specimen_domes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Base wood pool.
    let wood_colors: [u32; 4] = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let gold_mat = lambert!({ color: 0xc0a060, emissive: 0x201810 });
    // Very transparent glass for dome.
    let glass_mat = lambert!({ color: 0xd8e0e8, transparent: true, opacity: 0.20 });
    let sd = group();
    // Wooden base — round platform w/ small foot.
    let base_r = 0.075;
    let base = mesh(pg("SpecimenDomes__base", cyl_geo!(base_r, base_r + 0.006, 0.015, 16)), &wood_mat);
    base.position().y = 0.008;
    sd.add(&base);
    // Ornamental base ring (torus).
    let base_ring = mesh(pg("SpecimenDomes__baseRing", torus_geo!(base_r, 0.004, 5, 16)), &gold_mat);
    base_ring.rotation().x = PI / 2.0;
    base_ring.position().y = 0.016;
    sd.add(&base_ring);
    // Small gold plaque on the plinth's sloping face.
    let plaque_mat = gold_mat.clone();
    let plaque = mesh(pg("SpecimenDomes__plaque", box_geo!(0.036, 0.010, 0.002)), &plaque_mat);
    let plaque_cant = 0.4f64.atan();
    plaque.position().set(0, 0.008, base_r + 0.0040);
    plaque.rotation().x = -plaque_cant;
    sd.add(&plaque);
    // Tiny engraving lines on plaque.
    let eng_mat = lambert!({ color: 0x402020 });
    for i in 0..2 {
        let eng_line = mesh(box_geo!(0.024, 0.001, 0.0005), &eng_mat);
        let d = (i as f64 - 0.5) * 0.004;
        eng_line.position().set(
            0,
            0.008 + d * plaque_cant.cos() + 0.0011 * plaque_cant.sin(),
            base_r + 0.0040 - d * plaque_cant.sin() + 0.0011 * plaque_cant.cos(),
        );
        eng_line.rotation().x = -plaque_cant;
        sd.add(&eng_line);
    }
    // Glass hemisphere dome (top-half sphere).
    let dome_r = base_r - 0.008;
    let dome = mesh(sphere_geo!(dome_r, 16, 10, 0, PI * 2.0, 0, PI / 2.0), &glass_mat);
    dome.position().y = 0.016;
    sd.add(&dome);
    // Faint rim outline where dome meets base.
    let dome_rim = mesh(pg("SpecimenDomes__domeRim", torus_geo!(dome_r, 0.002, 4, 20)), &gold_mat);
    dome_rim.rotation().x = PI / 2.0;
    dome_rim.position().y = 0.017;
    sd.add(&dome_rim);
    // 3-variant specimen inside — bird / butterfly / beetle.
    let spec_variant = (rnd() * 3.0).floor() as i32;
    if spec_variant == 0 {
        // Taxidermy bird on perch.
        let bird_colors: [u32; 5] = [0x8b4513, 0x6b8060, 0x502010, 0x605040, 0x804020];
        let bird_color = bird_colors[ri(bird_colors.len())];
        let bird_mat = lambert!({ color: bird_color });
        // Small wooden perch lying on the plinth top, across the bird.
        let perch = mesh(pg("SpecimenDomes__perch", cyl_geo!(0.003, 0.003, 0.040, 5)), &wood_mat);
        perch.rotation().x = PI / 2.0;
        perch.position().set(0.005, 0.0185, 0);
        sd.add(&perch);
        // Bird body, long along X like the head-tail line.
        let bird_body = mesh(pg("SpecimenDomes__birdBody", sphere_geo!(0.014, 10, 7)), &bird_mat);
        bird_body.scale().set(1.3, 0.85, 1.0);
        bird_body.position().y = 0.0325;
        sd.add(&bird_body);
        // Head (smaller sphere).
        let bird_head = mesh(pg("SpecimenDomes__birdHead", sphere_geo!(0.009, 8, 6)), &bird_mat);
        bird_head.position().set(0.015, 0.0415, 0);
        sd.add(&bird_head);
        // Tiny beak (cone), base 1.5 mm into the face.
        let beak = mesh(pg("SpecimenDomes__beak", cone_geo!(0.002, 0.006, 5)), lambert!({ color: 0xa07030 }));
        beak.rotation().z = -PI / 2.0;
        beak.position().set(0.0255, 0.0415, 0);
        sd.add(&beak);
        // Tail (small flat cone) fanning out flat, root in the rump.
        let tail = mesh(pg("SpecimenDomes__tail", cone_geo!(0.007, 0.016, 4)), &bird_mat);
        tail.scale().z = 0.4;
        tail.rotation().x = -PI / 2.0;
        tail.rotation().z = PI / 2.0;
        tail.position().set(-0.023, 0.0305, 0);
        sd.add(&tail);
        // 2 wings.
        for side in [-1.0f64, 1.0] {
            let wing = mesh(pg("SpecimenDomes__wing", sphere_geo!(0.010, 8, 5)), &bird_mat);
            wing.scale().set(0.4, 0.5, 1.3);
            wing.rotation().y = PI / 2.0;
            wing.position().set(-0.002, 0.0345, side * 0.0125);
            sd.add(&wing);
        }
        // 2 legs, standing on the perch.
        for side in [-1.0f64, 1.0] {
            let leg = mesh(cyl_geo!(0.0008, 0.0008, 0.008, 3), lambert!({ color: 0xa07030 }));
            leg.position().set(0.005, 0.0235, side * 0.005);
            sd.add(&leg);
        }
        // Tiny eye dot.
        let eye = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 4, 4)), lambert!({ color: 0x080810 }));
        eye.position().set(0.020, 0.0435, 0.006);
        sd.add(&eye);
    } else if spec_variant == 1 {
        // Butterfly on pin.
        let wing_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let wing_color = wing_colors[ri(wing_colors.len())];
        let wing_mat = lambert!({ color: wing_color, side: DoubleSide });
        let body_mat = lambert!({ color: 0x181818 });
        // Vertical pin (silver needle).
        let pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0005, 0.0005, 0.030, 4)), lambert!({ color: 0xa0a0a8 }));
        pin.position().y = 0.030;
        sd.add(&pin);
        // Pin head.
        let pin_head = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 5, 4)), lambert!({ color: 0xa0a0a8 }));
        pin_head.position().y = 0.046;
        sd.add(&pin_head);
        // Central body.
        let bf_body = mesh(pg("SpecimenDomes__bfBody", cyl_geo!(0.001, 0.001, 0.016, 4)), &body_mat);
        bf_body.rotation().z = PI / 2.0;
        bf_body.position().y = 0.036;
        sd.add(&bf_body);
        // Head.
        let bf_head = mesh(pg("SpecimenDomes__bfHead", sphere_geo!(0.002, 4, 4)), &body_mat);
        bf_head.position().set(0.010, 0.036, 0);
        sd.add(&bf_head);
        // 4 wing plates (2 upper, 2 lower, symmetric).
        for side in [-1.0f64, 1.0] {
            let upper_wing = mesh(circle_geo!(0.014, 5), &wing_mat);
            upper_wing.scale().set(1.0, 0.7, 1.0);
            upper_wing.rotation().x = -PI / 2.0;
            upper_wing.rotation().z = side * 0.3;
            upper_wing.position().set(0, 0.036, side * 0.014);
            sd.add(&upper_wing);
            let lower_wing = mesh(circle_geo!(0.010, 5), &wing_mat);
            lower_wing.scale().set(1.0, 0.5, 1.0);
            lower_wing.rotation().x = -PI / 2.0;
            lower_wing.rotation().z = side * -0.2;
            lower_wing.position().set(-0.003, 0.036, side * 0.012);
            sd.add(&lower_wing);
            let marking = mesh(circle_geo!(0.003, 5), lambert!({ color: 0x181818, side: DoubleSide }));
            marking.rotation().x = -PI / 2.0;
            marking.position().set(0, 0.0362, side * 0.014);
            sd.add(&marking);
        }
        // 2 antennae.
        for side in [-1.0f64, 1.0] {
            let ant = mesh(pg("SpecimenDomes__antenna", cyl_geo!(0.0004, 0.0004, 0.010, 3)), &body_mat);
            ant.position().set(0.014, 0.038, side * 0.002);
            ant.rotation().z = -0.6;
            sd.add(&ant);
        }
    } else {
        // Beetle display — 3-4 beetles pinned in a row.
        let beetle_count = 3 + (rnd() * 2.0).floor() as i32;
        let beetle_colors: [u32; 4] = [0x1a1a08, 0x2a3010, 0x502010, 0x102030];
        for i in 0..beetle_count {
            let beetle_color = beetle_colors[ri(beetle_colors.len())];
            let beetle_mat = lambert!({ color: beetle_color, emissive: 0x080810 });
            let beetle_group = group();
            // Oval body (squashed sphere).
            let beetle = mesh(pg("SpecimenDomes__beetle", sphere_geo!(0.006, 8, 6)), &beetle_mat);
            beetle.scale().set(1.0, 0.4, 1.6);
            beetle.position().y = 0.022;
            beetle_group.add(&beetle);
            // Small head bump.
            let b_head = mesh(pg("SpecimenDomes__beetleHead", sphere_geo!(0.003, 5, 4)), &beetle_mat);
            b_head.position().set(0, 0.024, 0.009);
            beetle_group.add(&b_head);
            // 6 tiny legs (3 per side).
            for side in [-1.0f64, 1.0] {
                for j in 0..3 {
                    let leg = mesh(cyl_geo!(0.0004, 0.0004, 0.006, 3), &beetle_mat);
                    leg.rotation().z = side * 0.5;
                    leg.position().set(side * 0.006, 0.021, -0.004 + j as f64 * 0.004);
                    beetle_group.add(&leg);
                }
            }
            // Tiny elytra split line down back.
            let split = mesh(box_geo!(0.0004, 0.001, 0.010), lambert!({ color: 0x080810 }));
            split.position().set(0, 0.024, 0);
            beetle_group.add(&split);
            // Pin from the plinth top up through the beetle.
            let b_pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0004, 0.0004, 0.014, 3)), lambert!({ color: 0xa0a0a8 }));
            b_pin.position().set(0, 0.0225, 0);
            beetle_group.add(&b_pin);
            // Position beetles in a row.
            beetle_group.position().x = (i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020;
            sd.add(&beetle_group);
        }
        // Small paper label in front of each beetle.
        for i in 0..beetle_count {
            let label = mesh(pg("SpecimenDomes__label", box_geo!(0.010, 0.0008, 0.004)), lambert!({ color: 0xf0e8c0 }));
            label.position().set((i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020, 0.0159, -0.020);
            sd.add(&label);
        }
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}
