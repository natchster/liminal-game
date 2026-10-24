//! Toys, children's items and restroom/luggage fixtures (R408-R421).
use crate::{buf_geo, open_tiles, pick, set_rot_order, wall_candidates};
use game::prelude::*;

/// R421 small pacifier: 8% chance per level. Nipple cylinder +
/// guard disc + ring handle.
pub fn place_small_pacifiers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let guard_colors = [0xd8a0c0, 0xa0c0d8, 0xd8d0a0, 0xf0f0f0];
    let guard_mat = lambert!({ color: pick(&guard_colors) });
    let nip_mat = lambert!({ color: 0xe8dcc0 });
    let paci = group();
    // Nipple: soft small cylinder.
    let nipple = mesh(pg("SmallPacifiers__nipple", cyl_geo!(0.008, 0.010, 0.020, 6)), &nip_mat);
    nipple.rotation().z = PI / 2.0;
    nipple.position().set(0.015, 0.014, 0);
    paci.add(&nipple);
    // Guard disc: shield behind nipple.
    let guard = mesh(pg("SmallPacifiers__shield", cyl_geo!(0.020, 0.020, 0.005, 10)), &guard_mat);
    guard.rotation().z = PI / 2.0;
    guard.position().set(0.005, 0.014, 0);
    paci.add(&guard);
    // Ring handle behind guard (half-torus).
    let ring = mesh(pg("SmallPacifiers__ring", torus_geo!(0.010, 0.002, 4, 10)), &guard_mat);
    ring.position().set(-0.008, 0.014, 0);
    paci.add(&ring);
    paci.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    paci.rotation().y = rnd() * PI * 2.0;
    g.world.add(&paci);
}

/// R421 small face mask: 6% chance per level. Oval face plate +
/// 2 dark eye holes + strap.
pub fn place_small_masks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance (rarest)
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let face_colors = [0xd0b898, 0xa08060, 0xd0d0c0, 0xe8ccb0, 0xa0a0a0];
    let face_mat = lambert!({ color: pick(&face_colors) });
    let hole_mat = basic_mat!({ color: 0x080604 });
    let strap_mat = lambert!({ color: 0x2a1810 });
    let mask = group();
    // Face plate: oval ellipsoid (scaled sphere).
    let face = mesh(pg("SmallMasks__face", sphere_geo!(0.06, 10, 6, 0.0, PI * 2.0, 0.0, PI / 2.0)), &face_mat);
    face.scale().set(1, 0.15, 1.3);
    face.position().y = 0.010;
    mask.add(&face);
    // 2 dark eye holes.
    for ex_off in [-0.020, 0.020] {
        let eye = mesh(circle_geo!(0.008, 6), &hole_mat);
        eye.rotation().x = -PI / 2.0;
        eye.position().set(ex_off, 0.019, -0.01);
        mask.add(&eye);
    }
    // Elastic strap behind (under) the face-up plate at eye level.
    let strap = mesh(pg("SmallMasks__strap", cyl_geo!(0.002, 0.002, 0.14, 4)), &strap_mat);
    strap.rotation().z = PI / 2.0;
    strap.position().set(0, 0.008, -0.01);
    mask.add(&strap);
    mask.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mask.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mask);
}

/// R420 small toy drum: 8% chance per level. Cylinder body + top +
/// bottom membrane discs + loose strap.
pub fn place_toy_drums(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let mem_mat = lambert!({ color: 0xd8ccb0 });
    let strap_mat = lambert!({ color: 0x4a3018 });
    let drum = group();
    // Body: vertical cylinder.
    let body = mesh(pg("ToyDrums__body", cyl_geo!(0.06, 0.06, 0.09, 12)), &body_mat);
    body.position().y = 0.045;
    drum.add(&body);
    // Top membrane disc.
    let top_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    top_m.rotation().x = -PI / 2.0;
    top_m.position().y = 0.091;
    drum.add(&top_m);
    // Bottom membrane disc.
    let bot_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    bot_m.rotation().x = PI / 2.0;
    bot_m.position().y = -0.001;
    drum.add(&bot_m);
    // Loose carry strap lying face-up on the floor, one end tucked under the
    // drum's rim at +X, trailing off 0.3 rad from the tangent.
    let strap = mesh(plane_geo!(0.015, 0.20), &strap_mat);
    strap.rotation().x = -PI / 2.0;
    strap.position().set(0.06 + 0.3f64.sin() * 0.095, 0.0015, 0.3f64.cos() * 0.095);
    strap.rotation().z = 0.3;
    drum.add(&strap);
    drum.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    drum.rotation().y = rnd() * PI * 2.0;
    g.world.add(&drum);
}

/// R420 small hat: 10% chance per level. Cylinder crown + flat brim.
pub fn place_small_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hat_colors = [0x181410, 0x4a3018, 0x2a2418, 0x604828, 0x681818];
    let hat_mat = lambert!({ color: pick(&hat_colors) });
    let band_mat = lambert!({ color: 0x2a2418 });
    let hat = group();
    // Brim: wider flat disc.
    let brim = mesh(pg("SmallHats__brim", cyl_geo!(0.10, 0.10, 0.008, 14)), &hat_mat);
    brim.position().y = 0.004;
    hat.add(&brim);
    // Crown: taller narrower cylinder.
    let crown = mesh(pg("SmallHats__crown", cyl_geo!(0.055, 0.055, 0.08, 12)), &hat_mat);
    crown.position().y = 0.048;
    hat.add(&crown);
    // Small band ring around crown base.
    let band = mesh(pg("SmallHats__band", torus_geo!(0.055, 0.005, 4, 12)), &band_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.014;
    hat.add(&band);
    hat.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hat.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped on side (brim flat instead of vertical).
    if rnd() < 0.25 {
        hat.rotation().z = PI / 2.0;
        hat.position().y = 0.10;
    }
    g.world.add(&hat);
}

/// R419 small yo-yo: 10% chance per level. 2 flat colored discs +
/// connecting axle + short dangling string.
pub fn place_small_yo_yos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let yoyo_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8];
    let yoyo_mat = lambert!({ color: pick(&yoyo_colors) });
    let axle_mat = lambert!({ color: 0x2a2418 });
    let string_mat = lambert!({ color: 0xe8e0d0 });
    let yoyo = group();
    // 2 flat discs (yo-yo halves).
    for x_off in [-0.010, 0.010] {
        let disc = mesh(pg("SmallYoYos__half", cyl_geo!(0.024, 0.024, 0.006, 12)), &yoyo_mat);
        disc.rotation().z = PI / 2.0;
        disc.position().set(x_off, 0.024, 0);
        yoyo.add(&disc);
    }
    // Central axle cylinder.
    let axle = mesh(pg("SmallYoYos__axle", cyl_geo!(0.005, 0.005, 0.020, 6)), &axle_mat);
    axle.rotation().z = PI / 2.0;
    axle.position().y = 0.024;
    yoyo.add(&axle);
    // Short dangling string: leaves the underside of the axle between the
    // halves and slopes down to the floor 10 cm out.
    let string = mesh(pg("SmallYoYos__string", cyl_geo!(0.001, 0.001, 0.10, 4)), &string_mat);
    string.position().set(0, 0.0105, 0.049);
    string.rotation().x = PI / 2.0 + 0.18;
    yoyo.add(&string);
    yoyo.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    yoyo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&yoyo);
}

/// R418 small kite: 8% chance per level. Diamond-shape plane + 3
/// small bow ribbons on tail. Laid flat on floor.
pub fn place_small_kites(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let kite_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0];
    let k_mat = lambert!({ color: pick(&kite_colors), side: DoubleSide });
    let bow_mat = lambert!({ color: pick(&kite_colors) });
    let string_mat = lambert!({ color: 0x181410 });
    let kite = group();
    // Diamond kite: 2 triangles arranged in diamond (custom geometry).
    let k_geo = buf_geo(
        &[
            0.0, 0.0, 0.12, // top
            -0.09, 0.0, 0.0, // left
            0.0, 0.0, -0.14, // bottom (longer)
            0.09, 0.0, 0.0, // right
        ],
        &[0, 1, 2, 0, 2, 3],
    );
    let k_mesh = mesh(k_geo, &k_mat);
    k_mesh.position().y = 0.003;
    kite.add(&k_mesh);
    // Tail: thin cylinder extending from bottom of kite.
    let tail = mesh(pg("SmallKites__tail", cyl_geo!(0.002, 0.002, 0.25, 4)), &string_mat);
    tail.rotation().x = PI / 2.0;
    tail.position().set(0, 0.004, -0.24);
    kite.add(&tail);
    // 3 bow ribbons on tail.
    for b in 0..3 {
        let bow = mesh(pg("SmallKites__bow", plane_geo!(0.02, 0.015)), &bow_mat);
        bow.rotation().x = -PI / 2.0;
        bow.rotation().z = rnd() * PI * 2.0;
        bow.position().set(0, 0.005, -0.15 - b as f64 * 0.06);
        kite.add(&bow);
    }
    kite.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    kite.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kite);
}

/// R418 small pinwheel: 10% chance per level. 4 colored triangles
/// arranged around central hub on a thin stick.
pub fn place_small_pinwheels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let stick_mat = lambert!({ color: 0x6a4820 });
    let hub_mat = lambert!({ color: 0x2a2418 });
    let pw_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let pinwheel = group();
    // Stick.
    let stick = mesh(pg("SmallPinwheels__stick", cyl_geo!(0.005, 0.005, 0.28, 5)), &stick_mat);
    stick.position().set(0, 0.14, -0.0055); // just behind the sail plane (z 0)
    pinwheel.add(&stick);
    // Central hub cylinder.
    let hub = mesh(pg("SmallPinwheels__hub", cyl_geo!(0.010, 0.010, 0.008, 6)), &hub_mat);
    hub.rotation().x = PI / 2.0;
    hub.position().set(0, 0.27, 0.004); // pin head in front of the sails' centre
    pinwheel.add(&hub);
    // 4 triangle sails in a sub-group (stood up in the XY plane facing +Z,
    // centred on the hub) so the spin animator's rotation.y spins them about
    // the hub axis.
    let sails_group = group();
    sails_group.rotation().x = -PI / 2.0;
    sails_group.position().set(0, 0.27, 0);
    for s in 0..4 {
        let sail_color = pw_colors[s];
        let sail_mat = lambert!({ color: sail_color, side: DoubleSide });
        let ang = (s as f64 / 4.0) * PI * 2.0;
        let sail_geo = buf_geo(
            &[
                0.0,
                0.0,
                0.0, // center
                ang.cos() * 0.05,
                0.0,
                ang.sin() * 0.05,
                (ang + PI / 4.0).cos() * 0.06,
                0.0,
                (ang + PI / 4.0).sin() * 0.06,
            ],
            &[0, 1, 2],
        );
        let sail = mesh(sail_geo, &sail_mat); // flat in the group's XZ plane
        sails_group.add(&sail);
    }
    pinwheel.add(&sails_group);
    pinwheel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pinwheel.rotation().y = rnd() * PI * 2.0;
    // Slight lean.
    pinwheel.rotation().z = (rnd() - 0.5) * 0.15;
    g.world.add(&pinwheel);
    // R419 30% chance: slow spin animator, period 4-8 s.
    if rnd() < 0.30 {
        let spin_period = 4.0 + rnd() * 4.0;
        let spin_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let sg = sails_group.clone();
        g.animate(move |_g, _dt, t| {
            sg.rotation().y = t * (PI * 2.0 / spin_period) * spin_dir;
        });
    }
}

/// R417 small doll: 8% chance per level. Humanoid stick doll —
/// body cylinder + head sphere + 4 short limb cylinders.
pub fn place_small_dolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd0b898, 0xa08060, 0x684030, 0xd8b8a0];
    let dress_colors = [0xd8a0c0, 0xa0c0d8, 0xc0d8a0, 0xd8c8a0, 0xa0a0d8];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let dress_mat = lambert!({ color: pick(&dress_colors) });
    let eye_mat = basic_mat!({ color: 0x080604 });
    let doll = group();
    // Torso.
    let torso = mesh(pg("SmallDolls__dress", cyl_geo!(0.020, 0.020, 0.07, 8)), &dress_mat);
    torso.position().y = 0.075;
    doll.add(&torso);
    // Head sphere.
    let head = mesh(pg("SmallDolls__head", sphere_geo!(0.028, 8, 6)), &body_mat);
    head.position().y = 0.136; // chin 2 mm into the torso top (0.110)
    doll.add(&head);
    // 2 tiny dot eyes.
    for e_off in [-0.010, 0.010] {
        let eye = mesh(circle_geo!(0.0025, 6), &eye_mat);
        eye.position().set(e_off, 0.141, 0.026);
        doll.add(&eye);
    }
    // 2 arm cylinders sticking out.
    for a_off in [-0.030, 0.030] {
        let arm = mesh(pg("SmallDolls__arm", cyl_geo!(0.008, 0.008, 0.06, 6)), &body_mat);
        arm.rotation().z = PI / 2.0 + (if a_off > 0.0 { -0.4 } else { 0.4 });
        arm.position().set(a_off, 0.09, 0);
        doll.add(&arm);
    }
    // 2 leg cylinders extending down.
    for l_off in [-0.012, 0.012] {
        let leg = mesh(pg("SmallDolls__leg", cyl_geo!(0.009, 0.009, 0.06, 6)), &body_mat);
        leg.position().set(l_off, 0.03, 0);
        doll.add(&leg);
    }
    doll.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    doll.rotation().y = rnd() * PI * 2.0;
    // 40% chance doll lies on its back, face up, resting on the back of its head.
    if rnd() < 0.40 {
        doll.set_rotation_order(EulerOrder::YXZ);
        doll.rotation().x = -PI / 2.0;
        doll.position().y = 0.028;
    }
    g.world.add(&doll);
}

/// R416 small toy car: 10% chance per level. Body + roof + 4 wheels
/// — classic Matchbox-style toy car silhouette.
pub fn place_toy_cars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd0d0d0, 0x181818];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let wheel_mat = lambert!({ color: 0x181410 });
    let car = group();
    // Body: main box.
    let body = mesh(pg("ToyCars__body", box_geo!(0.08, 0.02, 0.05)), &body_mat);
    body.position().y = 0.023;
    car.add(&body);
    // Roof: smaller box on top.
    let roof = mesh(pg("ToyCars__roof", box_geo!(0.04, 0.018, 0.045)), &body_mat);
    roof.position().y = 0.042;
    car.add(&roof);
    // 4 wheels: small cylinders at corners.
    for (wx1, wz1) in [(-0.028, -0.020), (0.028, -0.020), (-0.028, 0.020), (0.028, 0.020)] {
        let wheel = mesh(pg("ToyCars__wheel", cyl_geo!(0.010, 0.010, 0.008, 8)), &wheel_mat);
        wheel.rotation().x = PI / 2.0; // axle across the car (Z), in the body's wheel arches
        wheel.position().set(wx1, 0.010, wz1);
        car.add(&wheel);
    }
    car.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    car.rotation().y = rnd() * PI * 2.0;
    // R417 25% chance: car flipped on side (roll about its length axis, then yaw).
    if rnd() < 0.25 {
        car.set_rotation_order(EulerOrder::YXZ);
        car.rotation().x = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        car.position().y = 0.025; // sits on side, body half-width above floor
    }
    g.world.add(&car);
}

/// R415 marble scatter: 10% chance per level. 4-7 tiny colored
/// spheres scattered on floor — "spilled marbles".
pub fn place_marble_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let marble_colors = [0x2050d8, 0xd82020, 0xd8b820, 0x30a030, 0xa0a0a0, 0x8020d8];
    let cx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let cz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let count = 4 + (rnd() * 4.0).floor() as i32;
    for _ in 0..count {
        let m_col = pick(&marble_colors);
        let marble_mat = lambert!({ color: m_col });
        let marble = mesh(pg("MarbleScatters__marble", sphere_geo!(0.010, 6, 4)), &marble_mat);
        let m_ang = rnd() * PI * 2.0;
        let m_rad = rnd() * 0.30;
        marble.position().set(cx + m_ang.cos() * m_rad, 0.010, cz + m_ang.sin() * m_rad);
        g.world.add(&marble);
    }
}

/// R415 small building block: 10% chance per level. Colored cube with
/// 4 stud dots on top (LEGO-style silhouette).
pub fn place_building_blocks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let block_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xf0a020];
    let block_mat = lambert!({ color: pick(&block_colors) });
    let block = group();
    // Body: cube.
    let body = mesh(pg("BuildingBlocks__body", box_geo!(0.06, 0.03, 0.06)), &block_mat);
    body.position().y = 0.015;
    block.add(&body);
    // 4 stud dots on top (2×2 grid).
    for sx_off in [-0.015, 0.015] {
        for sz_off in [-0.015, 0.015] {
            let stud = mesh(pg("BuildingBlocks__stud", cyl_geo!(0.008, 0.008, 0.006, 6)), &block_mat);
            stud.position().set(sx_off, 0.033, sz_off);
            block.add(&stud);
        }
    }
    block.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    block.rotation().y = rnd() * PI * 2.0;
    g.world.add(&block);
    // R416 25% chance: 1-2 sibling blocks stacked on top.
    if rnd() < 0.25 {
        let stack_count = 1 + (rnd() * 2.0).floor() as i32;
        for s in 1..=stack_count {
            let sib_col = pick(&block_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_block = block.duplicate();
            // Replace materials on all children.
            sib_block.traverse(&mut |child| {
                if child.is_mesh() {
                    child.set_mat(&sib_mat);
                }
            });
            sib_block.position().y = 0.036 * s as f64; // stack height per block
            sib_block.position().x += (rnd() - 0.5) * 0.015;
            sib_block.position().z += (rnd() - 0.5) * 0.015;
            sib_block.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_block);
        }
    }
}

/// R414 small children's shoe: 10% chance per level. Small shoe
/// silhouette (elongated body + toe bump + sole).
pub fn place_children_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shoe_colors = [0xd82020, 0x2050d8, 0xf0f0f0, 0x181818, 0xd8b820];
    let shoe_mat = lambert!({ color: pick(&shoe_colors) });
    let sole_mat = lambert!({ color: 0x181410 });
    let shoe = group();
    // Body: elongated box.
    let body = mesh(pg("ChildrenShoes__upper", box_geo!(0.14, 0.05, 0.06)), &shoe_mat);
    body.position().y = 0.025;
    shoe.add(&body);
    // Toe bump: rounded box at front.
    let toe = mesh(pg("ChildrenShoes__toe", sphere_geo!(0.03, 6, 4)), &shoe_mat);
    toe.scale().set(1, 0.7, 1);
    toe.position().set(0.04, 0.025, 0); // ends flush with the sole's front (x 0.07)
    shoe.add(&toe);
    // Sole: thin dark box at bottom.
    let sole = mesh(pg("ChildrenShoes__sole", box_geo!(0.14, 0.008, 0.06)), &sole_mat);
    sole.position().y = 0.004;
    shoe.add(&sole);
    shoe.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    shoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shoe);
}

/// R413 small crayon: 10% chance per level. Thin colored cylinder +
/// small pointed cone tip.
pub fn place_small_crayons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crayon_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8, 0xd05038, 0xd82060];
    let c_col = pick(&crayon_colors);
    let cray_mat = lambert!({ color: c_col });
    let crayon = group();
    // Body: thin cylinder.
    let body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &cray_mat);
    body.rotation().z = PI / 2.0;
    crayon.add(&body);
    // Pointed tip: cone at one end.
    let tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &cray_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().x = 0.037;
    crayon.add(&tip);
    crayon.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.008, g.wz(z) + (rnd() - 0.5) * 2.4);
    crayon.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crayon);
    // R414 30% chance: 1-3 sibling crayons scattered nearby.
    if rnd() < 0.30 {
        let sib_count = 1 + (rnd() * 3.0).floor() as i32;
        for _ in 0..sib_count {
            let sib_col = pick(&crayon_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_crayon = group();
            let sib_body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &sib_mat);
            sib_body.rotation().z = PI / 2.0;
            sib_crayon.add(&sib_body);
            let sib_tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &sib_mat);
            sib_tip.rotation().z = -PI / 2.0;
            sib_tip.position().x = 0.037;
            sib_crayon.add(&sib_tip);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.06 + rnd() * 0.12;
            let cp = crayon.pos();
            sib_crayon.position().set(cp.x + s_ang.cos() * s_rad, 0.008, cp.z + s_ang.sin() * s_rad);
            sib_crayon.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_crayon);
        }
    }
}

// R413 drawing paper texture — cream paper with random crayon-color
// scribble strokes. Built once.
game::lazy_static_tex!(
    drawing_paper_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        // Random crayon scribble strokes.
        let scribble_colors = ["#c82020", "#2050c8", "#c8b020", "#30a030", "#8020c8", "#d05038"];
        let strokes = 4 + (rnd() * 4.0).floor() as i32;
        for _ in 0..strokes {
            ctx.set_stroke_style(pick(&scribble_colors));
            ctx.set_line_width(2.0 + rnd() * 2.0);
            ctx.begin_path();
            let start_x = 10.0 + rnd() * (w - 20.0);
            let start_y = 10.0 + rnd() * (h - 20.0);
            ctx.move_to(start_x, start_y);
            // Wiggly line via 4 quadratic curves.
            let (mut cur_x, mut cur_y) = (start_x, start_y);
            for _ in 0..4 {
                let cp_x = cur_x + (rnd() - 0.5) * 30.0;
                let cp_y = cur_y + (rnd() - 0.5) * 30.0;
                let end_x = cur_x + (rnd() - 0.5) * 40.0;
                let end_y = cur_y + (rnd() - 0.5) * 40.0;
                ctx.quadratic_curve_to(cp_x, cp_y, end_x, end_y);
                cur_x = end_x;
                cur_y = end_y;
            }
            ctx.stroke();
        }
        // Slight grunge.
        for _ in 0..15 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
);

/// Drawing paper (R413): 10% chance per level.
pub fn place_drawing_papers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let paper_mat = lambert!({ map: drawing_paper_texture(), side: DoubleSide });
    let paper = mesh(plane_geo!(0.22, 0.28), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = rnd() * PI * 2.0;
    paper.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.005, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&paper);
}

/// R412 small rubber ball: 10% chance per level. Sphere with 2-color
/// hemisphere split (top/bottom halves in different colors).
pub fn place_rubber_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let ball_colors_a = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let ball_colors_b = [0xf0f0f0, 0x181818, 0xd8b820, 0x8020d8];
    let c_a = pick(&ball_colors_a);
    let c_b = pick(&ball_colors_b);
    let top_mat = lambert!({ color: c_a });
    let bot_mat = lambert!({ color: c_b });
    let ball = group();
    // Top half sphere.
    let top = mesh(pg("RubberBalls__half", sphere_geo!(0.05, 10, 6, 0.0, PI * 2.0, 0.0, PI / 2.0)), &top_mat);
    top.position().y = 0.05;
    ball.add(&top);
    // Bottom half sphere.
    let bot = mesh(pg("RubberBalls__half", sphere_geo!(0.05, 10, 6, 0.0, PI * 2.0, 0.0, PI / 2.0)), &bot_mat);
    bot.rotation().x = PI;
    bot.position().y = 0.05;
    ball.add(&bot);
    ball.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}

/// R411 plush toy bear: 6% chance per level. Stubby brown plush
/// silhouette — very rare, deeply unsettling in institutional space.
pub fn place_plush_bears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bear_colors = [0x6a4020, 0x8a5028, 0x3a2810, 0xa08050];
    let bear_mat = lambert!({ color: pick(&bear_colors) });
    let eye_mat = basic_mat!({ color: 0x080604 });
    let bear = group();
    // Body: chunky rounded box (sphere approximation).
    let body = mesh(pg("PlushBears__body", sphere_geo!(0.06, 8, 6)), &bear_mat);
    body.scale().set(1.1, 1.3, 0.9);
    body.position().y = 0.08;
    bear.add(&body);
    // Head: smaller sphere on top.
    let head = mesh(pg("PlushBears__head", sphere_geo!(0.045, 8, 6)), &bear_mat);
    head.position().y = 0.19;
    bear.add(&head);
    // 2 ears: tiny spheres on head.
    for ex in [-0.030, 0.030] {
        let ear = mesh(pg("PlushBears__ear", sphere_geo!(0.015, 6, 4)), &bear_mat);
        ear.position().set(ex, 0.225, 0);
        bear.add(&ear);
    }
    // R412 30% chance: only 1 eye (missing damage variant).
    let missing_eye = rnd() < 0.30;
    let eye_offsets: Vec<f64> = if missing_eye { vec![if rnd() < 0.5 { -0.014 } else { 0.014 }] } else { vec![-0.014, 0.014] };
    for &ex in &eye_offsets {
        let eye = mesh(circle_geo!(0.004, 6), &eye_mat);
        eye.position().set(ex, 0.195, 0.043);
        bear.add(&eye);
    }
    // 2 short arms (small spheres on sides).
    for ax in [-0.065, 0.065] {
        let arm = mesh(pg("PlushBears__arm", sphere_geo!(0.028, 6, 4)), &bear_mat);
        arm.position().set(ax, 0.08, 0);
        bear.add(&arm);
    }
    bear.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    bear.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bear);
}

/// R410 small handbasket: 10% chance per level. Shallow open box +
/// 2 arched handles. Reads as shopping-style handbasket.
pub fn place_handbaskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a2020, 0x2050a0, 0x686050, 0x2a2a2a];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let inside_mat = basic_mat!({ color: 0x0a0806 });
    let handle_mat = lambert!({ color: 0x1a1610 });
    let basket = group();
    // Body: shallow open box.
    let body = mesh(pg("Handbaskets__body", box_geo!(0.22, 0.12, 0.16)), &body_mat);
    body.position().y = 0.06;
    basket.add(&body);
    // Inside dark disc (open top).
    let inside = mesh(plane_geo!(0.20, 0.14), &inside_mat);
    inside.rotation().x = -PI / 2.0;
    inside.position().y = 0.121;
    basket.add(&inside);
    // 2 arched handles across top: each half-ring (in XY, arching up) stands
    // on the rim of a long wall (z ±0.075), pivot hubs on the rim.
    for hx_off in [-0.09, 0.09] {
        let handle = mesh(pg("Handbaskets__handle", torus_geo!(0.05, 0.006, 4, 8, PI)), &handle_mat);
        handle.position().set(0, 0.118, hx_off * (0.075 / 0.09));
        basket.add(&handle);
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}

/// R410 small suitcase: 10% chance per level. Larger box + top handle
/// + 2 side latches.
pub fn place_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0x4a2010 });
    let handle_mat = lambert!({ color: 0x2a1810 });
    let latch_mat = lambert!({ color: 0xa08040 });
    let suitcase = group();
    // Body: taller than briefcase.
    let body = mesh(pg("Suitcases__body", box_geo!(0.40, 0.28, 0.14)), &body_mat);
    body.position().y = 0.14;
    suitcase.add(&body);
    // Top handle: half-torus.
    let handle = mesh(pg("Suitcases__handle", torus_geo!(0.06, 0.008, 4, 8, PI)), &handle_mat);
    handle.rotation().z = 0.0; // arch over the top, end blocks on the lid
    handle.position().y = 0.286;
    suitcase.add(&handle);
    // 2 brass latches on top edge.
    for lx_off in [-0.12, 0.12] {
        let latch = mesh(pg("Suitcases__latch", box_geo!(0.03, 0.012, 0.024)), &latch_mat);
        latch.position().set(lx_off, 0.27, 0.075);
        suitcase.add(&latch);
    }
    // Small "sticker" plate on front-right.
    let sticker = mesh(pg("Suitcases__plate", plane_geo!(0.06, 0.04)), &latch_mat);
    sticker.position().set(0.10, 0.18, 0.0715); // plate back on the front face
    suitcase.add(&sticker);
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}

/// R409 wall paper towel dispenser: 12% chance per level. Tall
/// gray box + horizontal dispensing slot + small hanging paper end.
pub fn place_paper_towel_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xa0a098 });
    let slot_mat = basic_mat!({ color: 0x0a0806 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let ptd = group();
    // Body: tall box.
    let body = mesh(pg("PaperTowelDispensers__body", box_geo!(0.24, 0.34, 0.10)), &body_mat);
    body.position().y = 0.0;
    ptd.add(&body);
    // Dispensing slot: dark horizontal recess near bottom.
    let slot = mesh(plane_geo!(0.18, 0.02), &slot_mat);
    slot.position().set(0, -0.15, 0.051);
    ptd.add(&slot);
    // Small paper towel hanging out of slot: top edge inside the slot recess,
    // hanging out over its lip.
    let paper_end = mesh(pg("PaperTowelDispensers__towel", plane_geo!(0.10, 0.06)), &paper_mat);
    paper_end.position().set(0, -0.178, 0.053);
    paper_end.rotation().x = -0.15;
    ptd.add(&paper_end);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.05);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.05);
    let ptd_height = 1.4 + rnd() * 0.15;
    if cdx.abs() == 1 {
        ptd.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        ptd.position().set(wall_x, ptd_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        ptd.rotation().y = if cdz > 0 { PI } else { 0.0 };
        ptd.position().set(wall_x + (rnd() - 0.5) * 0.6, ptd_height, wall_z);
    }
    g.world.add(&ptd);
}

/// R409 tissue box: 12% chance per level. Small box with oval hole
/// on top + tiny tissue poking up.
pub fn place_tissue_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let box_colors = [0xe8dcc0, 0xa0d0e8, 0xd0a0d8, 0xa0a0a0];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let hole_mat = basic_mat!({ color: 0x0a0806 });
    let tis_mat = lambert!({ color: 0xf0f0e8 });
    let tissue = group();
    // Body.
    let body = mesh(pg("TissueBoxes__box", box_geo!(0.20, 0.06, 0.10)), &box_mat);
    body.position().y = 0.030;
    tissue.add(&body);
    // Oval hole on top (approximated by dark plane).
    let hole = mesh(plane_geo!(0.10, 0.03), &hole_mat);
    hole.rotation().x = -PI / 2.0;
    hole.position().y = 0.061;
    tissue.add(&hole);
    // Tiny tissue triangle poking up from hole.
    let t_geo = buf_geo(&[-0.02, 0.0, 0.0, 0.02, 0.0, 0.0, 0.0, 0.025, 0.0], &[0, 1, 2]);
    let t_poke = mesh(t_geo, &tis_mat);
    t_poke.position().y = 0.065;
    t_poke.rotation().z = (rnd() - 0.5) * 0.3;
    tissue.add(&t_poke);
    tissue.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tissue.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tissue);
}

/// R408 wall electric hand dryer: 12% chance per level. Large white
/// box + curved bottom nozzle — restroom institutional fixture.
pub fn place_hand_dryers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xd0c8b8 });
    let nozzle_mat = lambert!({ color: 0x2a2418 });
    let dryer = group();
    // Main body: box.
    let body = mesh(pg("HandDryers__body", box_geo!(0.32, 0.30, 0.14)), &body_mat);
    body.position().y = 0.0;
    dryer.add(&body);
    // Curved bottom nozzle: short slanted box at bottom.
    let nozzle = mesh(pg("HandDryers__nozzle", box_geo!(0.12, 0.04, 0.06)), &nozzle_mat);
    nozzle.rotation().x = -0.5;
    nozzle.position().set(0, -0.17, 0.06);
    dryer.add(&nozzle);
    // Small round emergency button on right side.
    let btn = mesh(pg("HandDryers__button", cyl_geo!(0.015, 0.015, 0.005, 8)), &nozzle_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0.12, 0.05, 0.071);
    dryer.add(&btn);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.07);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.07);
    let dryer_height = 1.2 + rnd() * 0.15;
    if cdx.abs() == 1 {
        dryer.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        dryer.position().set(wall_x, dryer_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        dryer.rotation().y = if cdz > 0 { PI } else { 0.0 };
        dryer.position().set(wall_x + (rnd() - 0.5) * 0.6, dryer_height, wall_z);
    }
    g.world.add(&dryer);
}
