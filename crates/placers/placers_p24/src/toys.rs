//! Child-toy / household family.
use crate::{pick_color, pick_open_tile, scatter_pos, scatter_xz, tri_geo};
use game::prelude::*;

// R458 small princess wand: 6% chance per level. Star top + wood
// wand + trailing ribbons. Party-costume family.
pub fn place_small_princess_wands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let star_colors = [0xd8b040, 0xf0d0d0, 0xd050a0, 0x2080d8];
    let star_mat = basic_mat!({ color: pick_color(&star_colors), side: DoubleSide });
    let stick_mat = lambert!({ color: 0xd8b040 });
    let ribbon_colors = [0xd050a0, 0xd82040, 0xd8b040, 0xa020a0];
    let wand = group();
    // Wood stick.
    let stick_len = 0.18;
    let stick = mesh(pg("SmallPrincessWands__stick", cyl_geo!(0.004, 0.004, stick_len, 6)), &stick_mat);
    stick.rotation().z = PI / 2.0;
    stick.position().set(stick_len / 2.0, 0.008, 0);
    wand.add(&stick);
    // 5-point star at end.
    let mut star_verts: Vec<f64> = vec![];
    let outer_r = 0.028;
    let inner_r = 0.012;
    let n_pts = 5;
    for i in 0..n_pts * 2 {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let a = -PI / 2.0 + (i as f64 / (n_pts * 2) as f64) * PI * 2.0;
        star_verts.push(stick_len + a.cos() * r);
        star_verts.push(0.008 + a.sin() * r);
        star_verts.push(0.0);
    }
    // Triangulate as fan from center.
    let center_idx = (n_pts * 2) as u32;
    star_verts.push(stick_len);
    star_verts.push(0.008);
    star_verts.push(0.0);
    let mut indices: Vec<u32> = vec![];
    for i in 0..(n_pts * 2) as u32 {
        indices.push(center_idx);
        indices.push(i);
        indices.push((i + 1) % (n_pts * 2) as u32);
    }
    let star_geo = tri_geo(&star_verts, indices);
    let star = mesh(star_geo, &star_mat);
    // The star is built upright round (stick_len, 0.008), which put its lower
    // points 2 cm under the floor; lay it flat with the wand, turned about its
    // own centre on the stick's tip.
    let star_pivot = group();
    star_pivot.position().set(stick_len, 0.008, 0);
    star_pivot.rotation().x = -PI / 2.0;
    star.position().set(-stick_len, -0.008, 0);
    star_pivot.add(&star);
    wand.add(&star_pivot);
    // 3 trailing ribbons from stick end.
    for i in 0..3 {
        let r_mat = lambert!({ color: pick_color(&ribbon_colors), side: DoubleSide });
        let ribbon_len = 0.08 + rnd() * 0.06;
        let ribbon = mesh(plane_geo!(0.012, ribbon_len), &r_mat);
        ribbon.rotation().x = -PI / 2.0;
        ribbon.rotation().z = -PI / 6.0 + (i as f64 / 2.0) * (PI / 3.0);
        ribbon.position().set(-0.005 - (-PI / 6.0 + (i as f64 / 2.0) * (PI / 3.0)).cos() * ribbon_len / 2.0, 0.008, 0);
        wand.add(&ribbon);
    }
    scatter_pos(g, &wand, x, z, 2.4);
    wand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wand);
}

// R457 small toy robot: 6% chance per level. Blocky humanoid
// robot with head + body + limbs + antenna. Child-toy family.
pub fn place_small_toy_robots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0xc8c8c8, 0x606870, 0xd82040, 0x2050d8, 0xd8b040];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let detail_mat = lambert!({ color: 0x181008 });
    let light_mat = basic_mat!({ color: 0xd82040 });
    let antenna_ball_mat = basic_mat!({ color: 0xd82040 });
    let robot = group();
    // Body: block torso.
    let torso = mesh(pg("SmallToyRobots__torso", box_geo!(0.036, 0.040, 0.024)), &body_mat);
    torso.position().y = 0.038;
    robot.add(&torso);
    // Head: smaller cube.
    let head = mesh(pg("SmallToyRobots__head", box_geo!(0.024, 0.024, 0.022)), &body_mat);
    head.position().y = 0.070;
    robot.add(&head);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.003, 5, 4), &light_mat);
        eye.position().set(s * 0.006, 0.073, 0.012);
        robot.add(&eye);
    }
    // Mouth grill (dark bar).
    let mouth = mesh(box_geo!(0.012, 0.002, 0.001), &detail_mat);
    mouth.position().set(0, 0.065, 0.0112); // back face just inside the head front (z 0.011)
    robot.add(&mouth);
    // Antenna.
    let ant = mesh(pg("SmallToyRobots__antenna", cyl_geo!(0.001, 0.001, 0.020, 4)), &body_mat);
    ant.position().y = 0.092;
    robot.add(&ant);
    let ant_ball = mesh(sphere_geo!(0.004, 5, 4), &antenna_ball_mat);
    ant_ball.position().y = 0.104;
    robot.add(&ant_ball);
    // 2 arms.
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("SmallToyRobots__arm", box_geo!(0.010, 0.028, 0.010)), &body_mat);
        arm.position().set(s * 0.023, 0.038, 0);
        robot.add(&arm);
    }
    // 2 legs.
    for s in [-1.0, 1.0] {
        let leg = mesh(pg("SmallToyRobots__leg", box_geo!(0.014, 0.020, 0.014)), &body_mat);
        leg.position().set(s * 0.008, 0.010, 0);
        robot.add(&leg);
    }
    // Chest button/gauge.
    let chest = mesh(circle_geo!(0.004, 6), &light_mat);
    chest.position().set(0, 0.038, 0.013);
    robot.add(&chest);
    scatter_pos(g, &robot, x, z, 2.4);
    robot.rotation().y = rnd() * PI * 2.0;
    // 20% chance tipped on side.
    if rnd() < 0.20 {
        robot.rotation().z = PI / 2.0;
        robot.position().y = 0.020;
    }
    g.world.add(&robot);
}

// R457 small toy microphone: 8% chance per level. Kid karaoke
// microphone (bulb head + handle). Child-toy family.
pub fn place_small_toy_microphones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let handle_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0x181008];
    let handle_mat = lambert!({ color: pick_color(&handle_colors) });
    let head_mat = lambert!({ color: 0x2a2418 });
    let mic = group();
    // Handle: cylinder.
    let handle = mesh(pg("SmallToyMicrophones__handle", cyl_geo!(0.010, 0.012, 0.10, 8)), &handle_mat);
    // One axis through the grille centre (-0.010, 0.020), sloping down 0.072 rad
    // so the butt rests on the floor: collar at 1.6 cm along it, handle from
    // 1.4 cm (inside the collar) to 11.4 cm.
    let mic_tilt: f64 = 0.072;
    handle.rotation().z = PI / 2.0 - mic_tilt;
    handle.position().set(-0.010 + 0.064 * mic_tilt.cos(), 0.020 - 0.064 * mic_tilt.sin(), 0);
    mic.add(&handle);
    // Head: mesh bulb (dark sphere).
    let head = mesh(pg("SmallToyMicrophones__grille", sphere_geo!(0.020, 12, 10)), &head_mat);
    head.position().set(-0.010, 0.020, 0);
    mic.add(&head);
    // Ring between handle + head.
    let ring = mesh(pg("SmallToyMicrophones__collar", cyl_geo!(0.014, 0.014, 0.004, 8)), &handle_mat);
    ring.rotation().z = PI / 2.0 - mic_tilt;
    ring.position().set(-0.010 + 0.016 * mic_tilt.cos(), 0.020 - 0.016 * mic_tilt.sin(), 0);
    mic.add(&ring);
    scatter_pos(g, &mic, x, z, 2.4);
    mic.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mic);
}

// R456 small toy dinosaur: 6% chance per level. Green T-Rex
// silhouette (body + head + tail + legs). Child-toy family.
pub fn place_small_toy_dinos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let dino_colors = [0x408040, 0x306030, 0x504020, 0x805040, 0x2050d8];
    let dino_mat = lambert!({ color: pick_color(&dino_colors) });
    let eye_mat = lambert!({ color: 0x181008 });
    let dino = group();
    // Torso: elongated ellipsoid.
    let torso = mesh(pg("SmallToyDinos__torso", sphere_geo!(0.028, 10, 8)), &dino_mat);
    torso.scale().set(1.6, 1, 0.9);
    torso.position().y = 0.040;
    dino.add(&torso);
    // Head: smaller sphere at front.
    let head = mesh(pg("SmallToyDinos__head", sphere_geo!(0.020, 8, 6)), &dino_mat);
    head.scale().set(1.5, 1, 1);
    head.position().set(0.048, 0.060, 0);
    dino.add(&head);
    // Snout: box extending forward.
    let snout = mesh(pg("SmallToyDinos__snout", box_geo!(0.020, 0.014, 0.014)), &dino_mat);
    snout.position().set(0.076, 0.058, 0); // muzzle out past the head front (x 0.078), back half in it
    dino.add(&snout);
    // 2 tiny arms (short cylinders).
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("SmallToyDinos__arm", cyl_geo!(0.004, 0.004, 0.012, 5)), &dino_mat);
        arm.position().set(0.030, 0.048, s * 0.020);
        arm.rotation().z = -PI / 6.0;
        dino.add(&arm);
    }
    // 2 big legs.
    for s in [-1.0, 1.0] {
        let leg = mesh(pg("SmallToyDinos__leg", box_geo!(0.014, 0.030, 0.014)), &dino_mat);
        leg.position().set(0.005, 0.020, s * 0.014);
        dino.add(&leg);
    }
    // Tail: elongated cone extending back.
    let tail = mesh(pg("SmallToyDinos__tail", cone_geo!(0.014, 0.06, 6)), &dino_mat);
    tail.rotation().z = PI / 2.0;
    tail.position().set(-0.060, 0.040, 0);
    dino.add(&tail);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(pg("SmallPiggyBanks__eye", sphere_geo!(0.003, 5, 4)), &eye_mat);
        eye.position().set(0.053, 0.070, s * 0.014);
        dino.add(&eye);
    }
    scatter_pos(g, &dino, x, z, 2.4);
    dino.rotation().y = rnd() * PI * 2.0;
    g.world.add(&dino);
}

// R456 small toy phone: 8% chance per level. Toy phone base +
// handset with cord. Child-toy family.
pub fn place_small_toy_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0xd82040, 0xd8b040, 0x2050d8, 0x30a030];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let button_mat = lambert!({ color: 0xf0ece0 });
    let cord_mat = lambert!({ color: 0x201a10 });
    let phone = group();
    // Base: box.
    let base = mesh(pg("SmallToyPhones__base", box_geo!(0.09, 0.020, 0.07)), &body_mat);
    base.position().y = 0.010;
    phone.add(&base);
    // 3×4 dial button grid.
    for bx in 0..3 {
        for bz in 0..4 {
            let btn = mesh(pg("SmallToyPhones__button", cyl_geo!(0.005, 0.005, 0.002, 6)), &button_mat);
            btn.position().set(
                -0.020 + bx as f64 * 0.020,
                0.021,
                -0.029 + bz as f64 * 0.0125, // grid kept in front of the handset
            );
            phone.add(&btn);
        }
    }
    // Handset: curved cylinder on top of base.
    let handset = mesh(pg("SmallToyPhones__handset", cyl_geo!(0.010, 0.010, 0.075, 6)), &body_mat);
    handset.rotation().z = PI / 2.0;
    // Handset and cups rest wholly on the base top (back edge z 0.035, sides x ±0.045).
    handset.position().set(0, 0.030, 0.023);
    phone.add(&handset);
    // Handset earpiece + mouthpiece bulges.
    for s in [-1.0, 1.0] {
        let bulge = mesh(pg("SmallToyPhones__cup", sphere_geo!(0.012, 6, 5)), &body_mat);
        bulge.position().set(s * 0.033, 0.030, 0.023);
        phone.add(&bulge);
    }
    // Cord: thin curved cylinder loop.
    let cord = mesh(torus_geo!(0.010, 0.001, 4, 8, PI * 1.5), &cord_mat);
    cord.position().set(0.037, 0.022, 0.005); // in front of the earpiece cup, clear of the buttons (x <= 0.025)
    cord.rotation().z = PI / 2.0;
    phone.add(&cord);
    scatter_pos(g, &phone, x, z, 2.4);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}

// R455 small piggy bank: 6% chance per level. Pink piggy bank
// body + 4 legs + snout + ears + coin slot. Child-toy family.
pub fn place_small_piggy_banks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let piggy_colors = [0xf0a0a0, 0xf0c0d0, 0xd0e0f0, 0xf0d8c8];
    let piggy_mat = lambert!({ color: pick_color(&piggy_colors) });
    let slot_mat = lambert!({ color: 0x181008 });
    let eye_mat = lambert!({ color: 0x181008 });
    let pig = group();
    // Body: sphere scaled elongated.
    let body = mesh(pg("SmallPiggyBanks__body", sphere_geo!(0.045, 12, 10)), &piggy_mat);
    body.scale().set(1.3, 1, 1);
    body.position().y = 0.048;
    pig.add(&body);
    // 4 legs (small cylinders).
    for (lx, lz) in [(-0.04, -0.02), (0.04, -0.02), (-0.04, 0.02), (0.04, 0.02)] {
        let leg = mesh(pg("SmallPiggyBanks__leg", cyl_geo!(0.008, 0.008, 0.018, 6)), &piggy_mat);
        leg.position().set(lx, 0.009, lz);
        pig.add(&leg);
    }
    // Snout: cylinder at front.
    let snout = mesh(pg("SmallPiggyBanks__snout", cyl_geo!(0.014, 0.014, 0.014, 8)), &piggy_mat);
    snout.rotation().z = PI / 2.0;
    snout.position().set(0.058, 0.048, 0);
    pig.add(&snout);
    // 2 nostrils (tiny dark discs).
    for s in [-1.0, 1.0] {
        let nostril = mesh(circle_geo!(0.002, 5), &slot_mat);
        nostril.rotation().y = PI / 2.0;
        nostril.position().set(0.065, 0.048, s * 0.004);
        pig.add(&nostril);
    }
    // 2 ears (small triangles).
    for s in [-1.0, 1.0] {
        let ear_geo = tri_geo(&[0.0, 0.0, 0.0, 0.012, 0.014, 0.0, -0.010, 0.014, 0.0], vec![0, 1, 2]);
        let ear = mesh(ear_geo, &piggy_mat);
        // Tip up (the triangle is built tip-down) with its base sunk into the
        // back; each ear's front face turned outward so both show from outside.
        ear.position().set(0.020, 0.092, s * 0.024);
        ear.rotation().set(0, if s > 0.0 { 0.4 } else { PI - 0.4 }, PI);
        pig.add(&ear);
    }
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(pg("SmallPiggyBanks__eye", sphere_geo!(0.003, 5, 4)), &eye_mat);
        eye.position().set(0.038, 0.062, s * 0.024);
        pig.add(&eye);
    }
    // Coin slot on back.
    let slot = mesh(pg("SmallPiggyBanks__slot", box_geo!(0.024, 0.001, 0.005)), &slot_mat);
    slot.position().set(-0.02, 0.091, 0);
    pig.add(&slot);
    // Curly tail.
    let tail = mesh(pg("SmallPiggyBanks__tail", torus_geo!(0.008, 0.002, 4, 8, PI * 1.5)), &piggy_mat);
    // Curl stands in the pig's side plane (XY); its thick start (local +x)
    // is sunk 1.5 mm into the rump (surface x -0.057) and it curls up and back.
    tail.position().set(-0.0635, 0.058, 0);
    pig.add(&tail);
    scatter_pos(g, &pig, x, z, 2.4);
    pig.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pig);
}

// R455 small magnifying glass: 8% chance per level. Handle +
// lens ring + tinted lens disc. Detective/kid family.
pub fn place_small_magnifying_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let handle_colors = [0x8a5030, 0x201a10, 0xd8b040, 0x181008];
    let handle_mat = lambert!({ color: pick_color(&handle_colors) });
    let ring_mat = lambert!({ color: 0xd8b040 });
    let lens_mat = basic_mat!({ color: 0xd0e8f0, transparent: true, opacity: 0.35 });
    let glass = group();
    let lens_r = 0.040;
    // Lens ring (torus).
    let ring = mesh(pg("SmallMagnifyingGlasses__rim", torus_geo!(lens_r, 0.005, 6, 16)), &ring_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.006;
    glass.add(&ring);
    // Lens disc (tinted glass).
    let lens = mesh(circle_geo!(lens_r - 0.002, 16), &lens_mat);
    lens.rotation().x = -PI / 2.0;
    lens.position().y = 0.007;
    glass.add(&lens);
    // Handle: extending sideways from lens ring.
    let handle_len = 0.10;
    let handle = mesh(pg("SmallMagnifyingGlasses__handle", cyl_geo!(0.008, 0.010, handle_len, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(lens_r + handle_len / 2.0, 0.006, 0);
    glass.add(&handle);
    // End knob.
    let knob = mesh(pg("SmallMagnifyingGlasses__knob", sphere_geo!(0.012, 8, 6)), &handle_mat);
    knob.position().set(lens_r + handle_len + 0.005, 0.006, 0);
    glass.add(&knob);
    scatter_pos(g, &glass, x, z, 2.4);
    glass.rotation().y = rnd() * PI * 2.0;
    g.world.add(&glass);
}

// R454 small metal jacks: 8% chance per level. 4-8 six-point
// metal jack toys scattered with small rubber ball. Games family.
pub fn place_small_jacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let jack_colors = [0xc8c8c8, 0xd8b040, 0xa02040, 0x2050d8];
    let ball_mat = lambert!({ color: 0xd82040 });
    let grp = group();
    let n_jacks = 4 + (rnd() * 5.0).floor() as i32;
    for i in 0..n_jacks {
        let jack_mat = lambert!({ color: pick_color(&jack_colors) });
        let jack = group();
        // 3 perpendicular pins with round tips (6 tips total = jack).
        for axis in 0..3 {
            let pin = mesh(pg("SmallJacks__pin", cyl_geo!(0.0018, 0.0018, 0.026, 4)), &jack_mat);
            if axis == 1 {
                // Y-axis: no rotation.
            } else if axis == 0 {
                pin.rotation().z = PI / 2.0;
            } else {
                pin.rotation().x = PI / 2.0;
            }
            jack.add(&pin);
            // Ball tips.
            for s in [-1.0, 1.0] {
                let tip = mesh(pg("SmallJacks__knob", sphere_geo!(0.003, 5, 4)), &jack_mat);
                if axis == 1 {
                    tip.position().y = s * 0.013;
                } else if axis == 0 {
                    tip.position().x = s * 0.013;
                } else {
                    tip.position().z = s * 0.013;
                }
                jack.add(&tip);
            }
        }
        // Keep clear of the jacks already down and of the ball (reach 1.6 cm);
        // widen the scatter a little on each retry when it gets crowded.
        let (mut jx, mut jz) = (0.0, 0.0);
        for tries in 0..30 {
            let spread = 1.0 + tries as f64 * 0.06;
            jx = (rnd() - 0.5) * 0.15 * spread;
            jz = (rnd() - 0.5) * 0.12 * spread;
            if hypot2(jx - 0.06, jz - 0.04) >= 0.028
                && grp.children().iter().all(|o| {
                    let p = o.pos();
                    hypot2(p.x - jx, p.z - jz) >= 0.034
                })
            {
                break;
            }
        }
        jack.position().set(jx, 0.013, jz);
        jack.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        grp.add(&jack);
    }
    // Small red rubber ball nearby.
    let ball = mesh(pg("SmallJacks__ball", sphere_geo!(0.010, 8, 6)), &ball_mat);
    ball.position().set(0.06, 0.010, 0.04);
    grp.add(&ball);
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R454 magnetic letters: 10% chance per level. 4-8 plastic fridge letters
// spilled on the floor (Blender batch r17): moulded capitals 38 mm tall,
// a round magnet set in each one's back, in the faded colours of an
// alphabet set; random letters, no word. Some landed on their backs.
pub fn place_magnetic_letters(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let letter_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xe07020];
    // the old colours as an alphabet set's polypropylene, faded
    let plastic: [u32; 6] = [0x983634, 0x35568a, 0xc29a3c, 0x3f7a43, 0x725089, 0xb96a30];
    let wear = fridge_letter_tex();
    let mut mats: Vec<Option<Mat>> = vec![None; plastic.len()];
    let magnet_mat = lambert!({ color: 0x3b3936 });
    let grp = group();
    // where each letter was drawn (the old spacing test runs on these), and
    // where each lies, its reach, its letter
    let mut drawn: Vec<(f64, f64)> = vec![];
    let mut lying: Vec<(f64, f64, f64)> = vec![];
    let mut used = [false; 26];
    let n_letters = 4 + (rnd() * 5.0).floor() as i32;
    for _ in 0..n_letters {
        let col = pick_color(&letter_colors);
        let k = letter_colors.iter().position(|&c| c == col).unwrap_or(0);
        // Keep clear of the tiles already down (half-diagonal 2 cm); widen the
        // scatter a little on each retry when it gets crowded.
        let (mut lx, mut lz) = (0.0, 0.0);
        for tries in 0..30 {
            let spread = 1.0 + tries as f64 * 0.06;
            lx = (rnd() - 0.5) * 0.18 * spread;
            lz = (rnd() - 0.5) * 0.12 * spread;
            if drawn.iter().all(|&(px, pz)| hypot2(px - lx, pz - lz) >= 0.040) {
                break;
            }
        }
        drawn.push((lx, lz));
        let ry = rnd() * PI * 2.0;
        // which letter, and whether it landed on its back, from where it fell
        // (no new draw); no letter twice
        let mut li = ((((lx + 0.5) * 9973.0 + (lz + 0.5) * 7919.0).fract().abs()) * 26.0) as usize % 26;
        while used[li] {
            li = (li + 7) % 26;
        }
        used[li] = true;
        let ch = (b'A' + li as u8) as char;
        let on_back = (ry * 13.7).fract() < 0.2;
        let Some(geo) = prop_part_as_modelled(&format!("Real__fridgeLetter{ch}")) else { continue };
        let bb = geo.bounding_box();
        let reach = 0.5 * hypot2(bb.max.x - bb.min.x, bb.max.z - bb.min.z);
        // clear of the letters already down: pushed off them where they fell
        let (mut qx, mut qz) = (lx, lz);
        for _ in 0..8 {
            for &(ax, az, ar) in &lying {
                let (dx, dz) = (qx - ax, qz - az);
                let d = hypot2(dx, dz);
                let need = reach + ar + 0.003;
                if d < need {
                    let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (1.0, 0.0) };
                    qx += ux * (need - d);
                    qz += uz * (need - d);
                }
            }
        }
        lying.push((qx, qz, reach));
        let mat = mats[k].get_or_insert_with(|| phong!({ color: plastic[k], map: &wear, specular: 0x222222, shininess: 30 })).clone();
        let letter = group();
        let inner = group();
        inner.add(&mesh(geo, &mat));
        if let Some(mg) = prop_part_as_modelled(&format!("Real__fridgeLetterMagnet{ch}")) {
            inner.add(&mesh(mg, &magnet_mat));
        }
        if on_back {
            inner.rotation().z = PI;
            inner.position().y = bb.max.y;
        }
        letter.add(&inner);
        letter.position().set(qx, 0, qz);
        letter.rotation().y = ry;
        grp.add(&letter);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// The letters' wear (multiplies each one's colour): the moulded plastic
/// scuffed pale where it was handled, scratched, grime and grit in it
fn fridge_letter_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#e8e3da");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..12 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 10.0 + rnd() * 24.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(255,252,244,0.30)" } else { "rgba(66,58,46,0.26)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        ctx.set_line_width(0.8);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 3.0 + rnd() * 12.0;
            ctx.set_stroke_style(if rnd() < 0.7 { "rgba(252,248,240,0.4)" } else { "rgba(44,38,32,0.3)" });
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..90 {
            ctx.set_fill_style(format!("rgba(50,44,36,{:.2})", 0.25 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}

// R453 small remote control: 10% chance per level. Rectangular
// remote with buttons + antenna. Household family.
pub fn place_small_remote_controls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0x181008, 0x201a10, 0x606870, 0xc8c8c8];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let button_colors = [0xd82040, 0x2050d8, 0x30a030, 0xd8b040, 0xf0ece0];
    let remote = group();
    // Body: elongated box.
    let body = mesh(pg("SmallRemoteControls__body", box_geo!(0.045, 0.010, 0.15)), &body_mat);
    body.position().y = 0.005;
    remote.add(&body);
    // 4×2 grid of button pips on top face.
    for bx in 0..4usize {
        for bz in 0..2usize {
            let button_mat = lambert!({ color: button_colors[(bx + bz) % button_colors.len()] });
            let btn = mesh(pg("SmallRemoteControls__button", cyl_geo!(0.004, 0.004, 0.002, 6)), &button_mat);
            btn.position().set(-0.014 + bx as f64 * 0.009, 0.011, -0.030 + bz as f64 * 0.008);
            remote.add(&btn);
        }
    }
    // Big rectangular button at bottom (mute/select).
    let big_btn = mesh(pg("SmallRemoteControls__bigbutton", box_geo!(0.024, 0.002, 0.014)), lambert!({ color: 0xd82040 }));
    big_btn.position().set(0, 0.011, 0.036);
    remote.add(&big_btn);
    // Antenna: thin cylinder poking out.
    let antenna_mat = lambert!({ color: 0x808080 });
    let antenna = mesh(pg("SmallRemoteControls__antenna", cyl_geo!(0.002, 0.002, 0.06, 4)), &antenna_mat);
    antenna.position().set(0.014, 0.010, -0.075);
    antenna.rotation().x = PI / 2.0 + 0.4;
    remote.add(&antenna);
    scatter_pos(g, &remote, x, z, 2.4);
    remote.rotation().y = rnd() * PI * 2.0;
    g.world.add(&remote);
}

// R453 kids drawing decal: 6% chance per level. Canvas texture
// with random scribbles simulating child chalk drawing.
game::lazy_static_tex!(
    kids_drawing_texture,
    canvas_tex_opt(384, 384, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // Chalk-ish palette.
        let colors = ["#f0ece0", "#d0d0d0", "#d08080", "#80d080", "#8080d0", "#d0d080"];
        ctx.set_line_cap("round");
        ctx.set_line_width(4);
        // 3-6 scribble strokes.
        let n_scribbles = 3 + (rnd() * 4.0).floor() as i32;
        for _s in 0..n_scribbles {
            ctx.set_stroke_style(colors[(rnd() * colors.len() as f64).floor() as usize]);
            ctx.set_global_alpha(0.4 + rnd() * 0.4);
            ctx.begin_path();
            let mut cx = 40.0 + rnd() * (w - 80.0);
            let mut cy = 40.0 + rnd() * (h - 80.0);
            ctx.move_to(cx, cy);
            let n_pts = 6 + (rnd() * 8.0).floor() as i32;
            for _p in 0..n_pts {
                cx += (rnd() - 0.5) * 80.0;
                cy += (rnd() - 0.5) * 80.0;
                ctx.line_to(cx, cy);
            }
            ctx.stroke();
        }
        // A stick figure or sun.
        ctx.set_global_alpha(0.7);
        ctx.set_stroke_style(colors[(rnd() * colors.len() as f64).floor() as usize]);
        ctx.set_line_width(5);
        let cx = w / 2.0 + (rnd() - 0.5) * 80.0;
        let cy = h / 2.0 + (rnd() - 0.5) * 80.0;
        // Circle (head or sun).
        ctx.begin_path();
        ctx.arc(cx, cy, 30, 0, PI * 2.0);
        ctx.stroke();
        // Rays or body lines.
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * 32.0, cy + a.sin() * 32.0);
            ctx.line_to(cx + a.cos() * 55.0, cy + a.sin() * 55.0);
            ctx.stroke();
        }
    })
);

pub fn place_kids_drawings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let draw_mat = basic_mat!({ map: kids_drawing_texture(), transparent: true, opacity: 0.75, depthWrite: false });
    let decal = mesh(plane_geo!(1.0, 1.0), &draw_mat);
    decal.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    decal.position().set(px, 0.004, pz);
    decal.rotation().z = rnd() * PI * 2.0;
    g.world.add(&decal);
}

// R452 small toy boat: 8% chance per level. Small wooden boat
// with mast + sail. Child-toy family.
pub fn place_small_toy_boats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let hull_colors = [0x8a5030, 0x603820, 0x2050d8, 0xd82040, 0xf0ece0];
    let hull_mat = lambert!({ color: pick_color(&hull_colors) });
    let mast_mat = lambert!({ color: 0x2a2418 });
    let sail_colors = [0xf0ece0, 0xd82040, 0x2050d8, 0xd8b040];
    let sail_mat = lambert!({ color: pick_color(&sail_colors), side: DoubleSide });
    let boat = group();
    // Hull: trapezoid box (wider top).
    let hull = mesh(pg("SmallToyBoats__hull", box_geo!(0.12, 0.030, 0.045)), &hull_mat);
    hull.position().y = 0.015;
    boat.add(&hull);
    // Prow: cone at front.
    let prow = mesh(pg("SmallToyBoats__bow", cone_geo!(0.022, 0.04, 4)), &hull_mat);
    prow.rotation().z = -PI / 2.0;
    prow.position().set(0.070, 0.015, 0);
    boat.add(&prow);
    // Deck plane (lighter color).
    let deck = mesh(pg("SmallToyBoats__deck", plane_geo!(0.10, 0.040)), &hull_mat);
    deck.rotation().x = -PI / 2.0;
    deck.position().y = 0.031;
    boat.add(&deck);
    // Mast.
    let mast = mesh(pg("SmallToyBoats__mast", cyl_geo!(0.003, 0.003, 0.10, 5)), &mast_mat);
    mast.position().set(0, 0.080, 0);
    boat.add(&mast);
    // Triangular sail.
    let sail_geo = tri_geo(&[0.0, 0.032, 0.0, 0.0, 0.130, 0.0, 0.055, 0.032, 0.0], vec![0, 1, 2]);
    let sail = mesh(sail_geo, &sail_mat);
    boat.add(&sail);
    scatter_pos(g, &boat, x, z, 2.4);
    boat.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped on side: rolled about the keel (x) onto the hull side,
    // mast along the floor (YXZ: roll about the boat's own axis, then yaw).
    if rnd() < 0.25 {
        boat.set_rotation_order(EulerOrder::YXZ);
        boat.rotation().x = PI / 2.0;
        boat.position().y = 0.0225;
    }
    g.world.add(&boat);
}

// R452 small children's music box: 6% chance per level. Cube
// wood box + wind-up crank. Child-music family.
pub fn place_small_music_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let box_colors = [0xa06030, 0x8a5030, 0x603820, 0xd82060, 0xd8b040];
    let box_mat = lambert!({ color: pick_color(&box_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let wick_mat = lambert!({ color: 0x201a10 });
    let bx = group();
    let box_size = 0.09;
    // Body: cube.
    let body = mesh(pg("SmallMusicBoxes__body", box_geo!(box_size, box_size * 0.7, box_size)), &box_mat);
    body.position().y = box_size * 0.35;
    bx.add(&body);
    // Brass border trim (thin band around body top).
    let trim = mesh(pg("SmallMusicBoxes__trim", box_geo!(box_size + 0.003, 0.004, box_size + 0.003)), &brass_mat);
    trim.position().y = box_size * 0.7;
    bx.add(&trim);
    // Crank handle on side.
    let shaft = mesh(pg("SmallMusicBoxes__shaft", cyl_geo!(0.003, 0.003, 0.020, 5)), &brass_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(box_size / 2.0 + 0.010, box_size * 0.35, 0);
    bx.add(&shaft);
    let crank_arm = mesh(pg("SmallMusicBoxes__arm", box_geo!(0.004, 0.030, 0.004)), &brass_mat);
    // Arm pinned to the shaft at its lower boss (it was centred on the shaft).
    crank_arm.position().set(box_size / 2.0 + 0.020, box_size * 0.35 + 0.013, 0);
    bx.add(&crank_arm);
    let crank_handle = mesh(pg("SmallMusicBoxes__handle", cyl_geo!(0.004, 0.004, 0.012, 6)), &wick_mat);
    // Knob sticks out sideways (+X) from the arm's upper boss, parallel to the
    // shaft, instead of standing on the arm's line.
    crank_handle.rotation().z = -PI / 2.0;
    crank_handle.position().set(box_size / 2.0 + 0.022 + 0.006, box_size * 0.35 + 0.026, 0);
    bx.add(&crank_handle);
    scatter_pos(g, &bx, x, z, 2.4);
    bx.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bx);
}

// R451 small toy plane: 8% chance per level. Toy airplane with
// fuselage + wings + tail + propeller. Child-toy family.
pub fn place_small_toy_planes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0x606870, 0xf0ece0];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let prop_mat = lambert!({ color: 0x201a10 });
    let plane = group();
    // Fuselage: elongated capsule.
    let fus = mesh(pg("SmallToyPlanes__fuselage", cyl_geo!(0.014, 0.008, 0.12, 8)), &body_mat);
    fus.rotation().z = PI / 2.0;
    fus.position().set(0.06, 0.020, 0);
    plane.add(&fus);
    // Nose cap (rounded).
    let nose = mesh(pg("SmallToyPlanes__nose", sphere_geo!(0.014, 8, 6)), &body_mat);
    nose.position().set(0, 0.020, 0);
    plane.add(&nose);
    // Tail cap.
    let tail = mesh(pg("SmallToyPlanes__tail", sphere_geo!(0.008, 6, 5)), &body_mat);
    tail.position().set(0.12, 0.020, 0);
    plane.add(&tail);
    // Wings: long thin box across.
    let wings = mesh(pg("SmallToyPlanes__wing", box_geo!(0.025, 0.005, 0.14)), &body_mat);
    wings.position().set(0.050, 0.020, 0);
    plane.add(&wings);
    // Tail fin (vertical).
    let fin = mesh(pg("SmallToyPlanes__fin", box_geo!(0.020, 0.024, 0.004)), &body_mat);
    fin.position().set(0.110, 0.028, 0);
    plane.add(&fin);
    // Tail horizontal stabilizer.
    let stab = mesh(pg("SmallToyPlanes__stab", box_geo!(0.020, 0.003, 0.045)), &body_mat);
    stab.position().set(0.110, 0.020, 0);
    plane.add(&stab);
    // Propeller (crossed sticks at nose).
    let prop_pivot = group();
    prop_pivot.position().set(-0.015, 0.020, 0); // just ahead of the nose (front at x -0.014)
    for i in 0..2 {
        let blade = mesh(pg("SmallToyPlanes__blade", box_geo!(0.002, 0.030, 0.005)), &prop_mat);
        blade.rotation().x = i as f64 * PI / 2.0;
        prop_pivot.add(&blade);
    }
    plane.add(&prop_pivot);
    // Propeller slow spin animator.
    let spin_rate = 3.0 + rnd() * 4.0;
    let pp = prop_pivot.clone();
    g.animate(move |_g, _dt, t| {
        pp.rotation().x = t * spin_rate;
    });
    scatter_pos(g, &plane, x, z, 2.4);
    plane.rotation().y = rnd() * PI * 2.0;
    // 30% chance tipped on side: rolled about the fuselage (x), resting on a
    // wing tip (YXZ so the roll is about the plane's own axis, then the yaw).
    if rnd() < 0.30 {
        plane.set_rotation_order(EulerOrder::YXZ);
        plane.rotation().x = PI / 2.0;
        plane.position().y = 0.070;
    }
    g.world.add(&plane);
}
