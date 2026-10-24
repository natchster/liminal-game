//! Toys, nursery items, wall thermometer, plush animals (R625–R635).
use crate::{face_yaw, open_tiles, pick, set_rot_order, wall_tiles};
use game::prelude::*;

// R625 crayon cluster: 8% chance per level. 5-9 short colored sticks.
pub fn place_crayons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crayon_colors: [u32; 10] = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xe08040, 0xd050a0, 0x804060, 0x202020, 0xffffff, 0xc0a040];
    let wrapper_mat = lambert!({ color: 0xf8f4e8 });
    let cluster = group();
    let count = 5 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let color = pick(&crayon_colors);
        let body_mat = lambert!({ color: color });
        let crayon = group();
        // Wrapper (main cylinder).
        let wrapper = mesh(pg("Crayons__wrapper", cyl_geo!(0.004, 0.004, 0.055, 6)), &wrapper_mat);
        wrapper.rotation().z = PI / 2.0;
        wrapper.position().y = 0.004;
        crayon.add(&wrapper);
        // Colored exposed tip (cone).
        let tip = mesh(pg("Crayons__tip", cone_geo!(0.004, 0.010, 6)), &body_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.032, 0.004, 0);
        crayon.add(&tip);
        // Small colored back-end nub.
        let back = mesh(pg("Crayons__back", cyl_geo!(0.004, 0.004, 0.008, 6)), &body_mat);
        back.rotation().z = PI / 2.0;
        back.position().set(-0.031, 0.004, 0); // butts into the wrapper end (-0.0275)
        crayon.add(&back);
        crayon.position().set((rnd() - 0.5) * 0.14, 0, (rnd() - 0.5) * 0.14);
        crayon.rotation().y = rnd() * PI * 2.0;
        cluster.add(&crayon);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

// R626 puzzle piece: 9% chance per level. Small irregular cardboard cutout.
pub fn place_puzzle_pieces(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    } // 9% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let piece_colors: [u32; 5] = [0x8ba0c0, 0xa0805a, 0x60a060, 0xd0a850, 0xc06070];
    let piece_mat = lambert!({ color: pick(&piece_colors), side: DoubleSide });
    let piece = group();
    // Body: base square + 2 bumps + 2 notches.
    let base = mesh(box_geo!(0.038, 0.003, 0.038), &piece_mat);
    base.position().y = 0.0015;
    piece.add(&base);
    // Bump on one side (positive tab).
    let bump = mesh(box_geo!(0.014, 0.003, 0.014), &piece_mat);
    bump.position().set(0.026, 0.0015, 0);
    piece.add(&bump);
    // Round bump extension (semi-circle nub).
    let round = mesh(cyl_geo!(0.008, 0.008, 0.003, 6), &piece_mat);
    round.position().set(0.033, 0.0015, 0);
    piece.add(&round);
    // Optional 2nd tab on adjacent side.
    if rnd() < 0.5 {
        let bump2 = mesh(cyl_geo!(0.008, 0.008, 0.003, 6), &piece_mat);
        bump2.position().set(0, 0.0015, 0.026);
        piece.add(&bump2);
    }
    piece.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    piece.rotation().y = rnd() * PI * 2.0;
    g.world.add(&piece);
}

// R627 plastic action figure: 4% chance per level. Head + torso + limbs.
pub fn place_action_figures(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 5] = [0x2060c0, 0x40a040, 0xc02020, 0x606060, 0xd0a040];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let skin_mat = lambert!({ color: 0xd8b090 });
    let fig = group();
    // 60% chance lying down (fallen posture), 40% chance upright.
    let is_down = rnd() < 0.60;
    // Torso.
    let torso = mesh(pg("ActionFigures__torso", box_geo!(0.024, 0.032, 0.014)), &body_mat);
    // lying on its back: every part's top (+Y) turns to -Z, toward the head,
    // and each rests on the floor
    torso.position().y = if is_down { 0.007 } else { 0.040 };
    if is_down {
        torso.rotation().x = -PI / 2.0;
    }
    fig.add(&torso);
    // Head.
    let head = mesh(pg("ActionFigures__head", sphere_geo!(0.011, 8, 6)), &skin_mat);
    head.position().y = if is_down { 0.011 } else { 0.068 };
    head.position().z = if is_down { -0.024 } else { 0.0 };
    if is_down {
        head.rotation().x = -PI / 2.0;
    } // neck peg to the torso
    fig.add(&head);
    // 2 arms.
    for side in [-1.0, 1.0] {
        let arm = mesh(pg("ActionFigures__arm", box_geo!(0.006, 0.024, 0.006)), &body_mat);
        arm.position().set(side * 0.014, if is_down { 0.003 } else { 0.040 }, if is_down { -0.004 } else { 0.0 });
        if is_down {
            arm.rotation().x = -PI / 2.0;
        }
        fig.add(&arm);
    }
    // 2 legs.
    for side in [-1.0, 1.0] {
        let leg = mesh(pg("ActionFigures__leg", box_geo!(0.008, 0.024, 0.008)), &body_mat);
        leg.position().set(side * 0.006, if is_down { 0.004 } else { 0.014 }, if is_down { 0.018 } else { 0.0 });
        if is_down {
            leg.rotation().x = -PI / 2.0;
        }
        fig.add(&leg);
    }
    fig.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    fig.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fig);
}

// R628 baby rattle: 6% chance per level. Ball head on handle w/ ring.
pub fn place_baby_rattles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rattle_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xf0e0a0, 0xa0e0c0];
    let rattle_mat = lambert!({ color: pick(&rattle_colors) });
    let handle_mat = lambert!({ color: 0xf8f0e8 });
    let rattle = group();
    // Ball head.
    let ball = mesh(pg("BabyRattles__ball", sphere_geo!(0.020, 8, 6)), &rattle_mat);
    ball.position().set(-0.030, 0.020, 0);
    rattle.add(&ball);
    // Handle stem.
    let handle = mesh(pg("BabyRattles__handle", cyl_geo!(0.005, 0.005, 0.055, 6)), &handle_mat);
    // aimed at the ball centre (its end 7 mm inside the ball), sloping down
    // to the grip ring, whose near side it enters
    handle.rotation().z = PI / 2.0 - 0.222;
    handle.position().set(0.0095, 0.0111, 0);
    rattle.add(&handle);
    // Grip ring at end, lying flat on the floor.
    let ring = mesh(pg("BabyRattles__ring", torus_geo!(0.014, 0.004, 5, 12)), &handle_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().set(0.0483, 0.004, 0);
    rattle.add(&ring);
    // 3 small stars/dots on ball for decoration.
    for i in 0..3 {
        let dot = mesh(circle_geo!(0.003, 6), lambert!({ color: 0xffffff }));
        let a = (i as f64 / 3.0) * PI * 2.0;
        // on the ball's crown (they were 3 mm inside it), tilted with its curve
        dot.position().set(-0.030 + a.cos() * 0.007, 0.0389, a.sin() * 0.007);
        set_rot_order(&dot, -PI / 2.0 + 0.42, PI / 2.0 - a, 0.0, EulerOrder::YXZ);
        rattle.add(&dot);
    }
    rattle.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    rattle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rattle);
}

// R628 toy dinosaur: 4% chance per level. Body + legs + tail + head.
pub fn place_toy_dinosaurs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let dino_colors: [u32; 5] = [0x408040, 0x2060c0, 0xa04020, 0xc0a040, 0x804080];
    let dino_mat = lambert!({ color: pick(&dino_colors) });
    let dino = group();
    // Body (ellipsoid via scaled sphere).
    let body = mesh(pg("ToyDinosaurs__body", sphere_geo!(0.020, 8, 6)), &dino_mat);
    body.scale().set(1.8, 0.9, 1);
    body.position().y = 0.020;
    dino.add(&body);
    // Head (smaller ellipsoid at front).
    let head = mesh(pg("ToyDinosaurs__head", sphere_geo!(0.012, 8, 6)), &dino_mat);
    head.scale().set(1.3, 1, 1);
    head.position().set(-0.032, 0.028, 0);
    dino.add(&head);
    // Tail (cone stretching back).
    let tail = mesh(pg("ToyDinosaurs__tail", cone_geo!(0.010, 0.045, 6)), &dino_mat);
    tail.rotation().z = -PI / 2.0;
    tail.position().set(0.045, 0.018, 0);
    dino.add(&tail);
    // 4 stubby legs.
    for i in 0..4 {
        let leg = mesh(pg("ToyDinosaurs__leg", cyl_geo!(0.006, 0.006, 0.014, 6)), &dino_mat);
        leg.position().set(if i < 2 { -0.015 } else { 0.015 }, 0.007, if i % 2 == 0 { -0.010 } else { 0.010 });
        dino.add(&leg);
    }
    // Optional 3 back spikes (50% chance = stegosaurus variant).
    if rnd() < 0.5 {
        for s in 0..3 {
            let spike = mesh(pg("ToyDinosaurs__plate", cone_geo!(0.005, 0.010, 4)), &dino_mat);
            spike.position().set(-0.010 + s as f64 * 0.010, 0.038, 0);
            dino.add(&spike);
        }
    }
    dino.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    dino.rotation().y = rnd() * PI * 2.0;
    g.world.add(&dino);
}

// R630 gum-ball dispenser: 3% chance per level. Wall-mounted glass globe on
// stand + coin slot.
pub fn place_gum_ball_dispensers(g: &mut Game, _dist: &[i32]) {
    g.pp.gumball_positions.clear(); // R669 reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let base_mat = lambert!({ color: 0xa02020 });
    let glass_mat = basic_mat!({ color: 0xe8f0f8, transparent: true, opacity: 0.35 });
    let gum_colors: [u32; 7] = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xd050a0, 0xe08040, 0xffffff];
    let dispenser = group();
    let mount_y = 1.30;
    // Base pedestal (red metal).
    let base = mesh(pg("GumBallDispensers__base", box_geo!(0.14, 0.24, 0.14)), &base_mat);
    base.position().y = mount_y;
    dispenser.add(&base);
    // Glass sphere on top w/ gumballs inside.
    let globe = mesh(sphere_geo!(0.09, 12, 10), &glass_mat);
    globe.position().set(0, mount_y + 0.20, 0);
    dispenser.add(&globe);
    // 15 colored gumballs inside sphere.
    for _i in 0..15 {
        let gc = pick(&gum_colors);
        let gum_mat = lambert!({ color: gc });
        let ball = mesh(pg("GumBallDispensers__ball", sphere_geo!(0.012, 6, 5)), &gum_mat);
        // Random position inside globe (bias downward — gravity settled).
        let r = rnd() * 0.065;
        let theta = rnd() * PI * 2.0;
        let phi = PI * 0.5 + rnd() * PI * 0.4; // biased lower hemisphere
        ball.position().set(phi.sin() * theta.cos() * r, mount_y + 0.20 + phi.cos() * r, phi.sin() * theta.sin() * r);
        dispenser.add(&ball);
    }
    // Coin slot (small dark rectangle on base).
    let slot = mesh(box_geo!(0.014, 0.004, 0.002), lambert!({ color: 0x100808 }));
    slot.position().set(0, mount_y + 0.04, 0.072);
    dispenser.add(&slot);
    // Dispensing knob.
    let knob = mesh(pg("GumBallDispensers__knob", cyl_geo!(0.010, 0.010, 0.015, 8)), &base_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0, mount_y - 0.04, 0.078);
    dispenser.add(&knob);
    dispenser.rotation().y = face_yaw(dx, dz);
    dispenser.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&dispenser);
    let p = dispenser.pos();
    g.pp.gumball_positions.push(v3(p.x, 0, p.z)); // R669
}

// R631 pacifier: 7% chance per level. Nipple + shield + ring.
pub fn place_pacifiers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shield_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xf0e0a0, 0xa0e0c0];
    let shield_mat = lambert!({ color: pick(&shield_colors) });
    let nipple_mat = lambert!({ color: 0xf0d8c0 });
    let paci = group();
    // Shield (flat disc lying on floor).
    let shield = mesh(pg("Pacifiers__shield", cyl_geo!(0.018, 0.018, 0.004, 10)), &shield_mat);
    shield.rotation().x = PI / 2.0;
    shield.position().set(0, 0.018, 0);
    paci.add(&shield);
    // Nipple (small elongated ellipsoid).
    let nipple = mesh(pg("Pacifiers__teat", sphere_geo!(0.006, 6, 5)), &nipple_mat);
    nipple.scale().set(1, 1, 1.6);
    nipple.position().set(0, 0.018, -0.010);
    paci.add(&nipple);
    // Ring on back (grip torus).
    let ring = mesh(pg("Pacifiers__ring", torus_geo!(0.010, 0.003, 5, 12)), &shield_mat);
    // hinged at the back centre of the shield (top of the ring there), swung
    // down and back until its far edge rests on the floor
    ring.rotation().x = -0.67;
    ring.position().set(0, 0.0102, 0.0102);
    paci.add(&ring);
    paci.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    paci.rotation().y = rnd() * PI * 2.0;
    g.world.add(&paci);
}

// R631 sippy cup: 6% chance per level. Colored cup + lid + spout + 2 handles.
pub fn place_sippy_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 5] = [0xc02020, 0x2060c0, 0x40a040, 0xd050a0, 0xf0e020];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let lid_mat = lambert!({ color: 0xf8f4e8 });
    let spout_mat = lambert!({ color: 0xf0d8c0 });
    let cup = group();
    // Cup body (tapered cylinder).
    let body = mesh(pg("SippyCups__body", cyl_geo!(0.030, 0.026, 0.070, 12)), &body_mat);
    body.position().y = 0.035;
    cup.add(&body);
    // Lid (wider disc on top).
    let lid = mesh(pg("SippyCups__lid", cyl_geo!(0.032, 0.032, 0.010, 12)), &lid_mat);
    lid.position().y = 0.075;
    cup.add(&lid);
    // Spout (small extended cylinder).
    let spout = mesh(pg("SippyCups__spout", cyl_geo!(0.006, 0.008, 0.020, 6)), &spout_mat);
    spout.rotation().z = 0.4;
    spout.position().set(0.020, 0.088, 0);
    cup.add(&spout);
    // 2 side handles (half-torus each).
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("SippyCups__handle", torus_geo!(0.014, 0.005, 5, 8, PI)), &body_mat);
        // C in the radial plane, bulging out sideways; both ends in the wall
        handle.rotation().z = -side * PI / 2.0;
        handle.position().set(side * 0.026, 0.035, 0);
        cup.add(&handle);
    }
    // 40% chance tipped on side: laid over about X so the handles stay level,
    // resting on the lid rim and the foot (Y last keeps the random heading).
    let tipped = rnd() < 0.4;
    if tipped {
        cup.set_rotation_order(EulerOrder::YXZ);
        cup.rotation().x = PI / 2.0 - 0.074;
    }
    cup.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, if tipped { 0.026 } else { 0.0 }, g.wz(z) + (rnd() - 0.5) * 2.4);
    cup.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cup);
}

// R632 wall thermometer strip: 4% chance per level. Long thin wall device w/
// liquid column + tick marks.
game::lazy_static_tex!(
    therm_texture,
    canvas_tex(28, 200, |ctx, w, h| {
        // Cream background.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        // Central red liquid column (partial fill).
        let fill_h = h * (0.4 + rnd() * 0.3);
        ctx.set_fill_style("#c02020");
        ctx.fill_rect(w * 0.42, h - fill_h, w * 0.16, fill_h);
        // Tick marks along right side.
        ctx.set_stroke_style("#202020");
        ctx.set_line_width(1);
        for i in 0..20 {
            let y = 10.0 + i as f64 * ((h - 20.0) / 19.0);
            let long = i % 5 == 0;
            ctx.begin_path();
            ctx.move_to(w * 0.62, y);
            ctx.line_to(w * (if long { 0.90 } else { 0.78 }), y);
            ctx.stroke();
        }
    })
);

pub fn place_thermometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let back_mat = lambert!({ color: 0x604020 });
    let face_mat = lambert!({ map: therm_texture() });
    let bulb_mat = lambert!({ color: 0xc02020 });
    let therm = group();
    let mount_y = 1.55;
    // Wooden backboard.
    let back = mesh(pg("Thermometers__board", box_geo!(0.030, 0.32, 0.008)), &back_mat);
    back.position().y = mount_y;
    therm.add(&back);
    // Face plate w/ scale texture.
    let face = mesh(plane_geo!(0.024, 0.28), &face_mat);
    face.position().set(0, mount_y, 0.006);
    therm.add(&face);
    // Bulb at bottom (red sphere).
    let bulb = mesh(pg("Thermometers__bulb", sphere_geo!(0.010, 8, 6)), &bulb_mat);
    bulb.position().set(0, mount_y - 0.15, 0.007);
    therm.add(&bulb);
    therm.rotation().y = face_yaw(dx, dz);
    therm.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.006), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.006));
    g.world.add(&therm);
}

// R632 baby bottle: 6% chance per level. Body + nipple + cap ring.
pub fn place_baby_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0xf8f0e0, transparent: true, opacity: 0.65 });
    let content_mat = lambert!({ color: 0xf0e8d8 });
    let ring_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xa0e0c0, 0xf0e0a0];
    let ring_mat = lambert!({ color: pick(&ring_colors) });
    let nipple_mat = lambert!({ color: 0xf0d8c0 });
    let bottle = group();
    // Translucent body.
    let body = mesh(cyl_geo!(0.020, 0.024, 0.075, 12), &body_mat);
    body.position().y = 0.037;
    bottle.add(&body);
    // Interior content (milk column visible through glass).
    let content = mesh(pg("BabyBottles__milk", cyl_geo!(0.017, 0.021, 0.045, 10)), &content_mat);
    content.position().y = 0.024;
    bottle.add(&content);
    // Colored cap ring on top.
    let ring = mesh(pg("BabyBottles__collar", cyl_geo!(0.022, 0.022, 0.010, 12)), &ring_mat);
    ring.position().y = 0.080;
    bottle.add(&ring);
    // Nipple on top.
    let nipple = mesh(pg("BabyBottles__teat", sphere_geo!(0.010, 6, 5)), &nipple_mat);
    nipple.scale().y = 1.4;
    nipple.position().y = 0.096;
    bottle.add(&nipple);
    // 30% tipped over.
    let is_tipped = rnd() < 0.30;
    if is_tipped {
        bottle.rotation().z = PI / 2.0;
    }
    bottle.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
    // R675 milk sloshing animator (tipped bottles slosh more).
    let slphase = rnd() * PI * 2.0;
    let amp = if is_tipped { 0.06 } else { 0.02 }; // tipped bottles slosh visibly more
    g.animate(move |_g, _dt, t| {
        // Wobble the milk cylinder scale x/z slightly, and tilt scale y a bit.
        let s = 1.0 + (t * 1.8 + slphase).sin() * amp * 0.5;
        let sy = 1.0 + (t * 1.8 + slphase + PI).sin() * amp * 0.3;
        content.scale().set(s, sy, s);
    });
}

// R634 8-track tape cartridge: 5% chance per level. Chunky blocky cartridge w/
// label + notch cutout.
pub fn place_8_track_tapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shell_colors: [u32; 6] = [0x2040a0, 0xa02020, 0x408040, 0xc0a040, 0x606060, 0xe0d0a0];
    let shell_mat = lambert!({ color: pick(&shell_colors) });
    let label_colors: [u32; 4] = [0xffffff, 0xf0e020, 0xd050a0, 0x40a0e0];
    let label_mat = lambert!({ color: label_colors[(rnd() * 4.0).floor() as usize] });
    let cartridge = group();
    // Chunky main body.
    let body = mesh(pg("EightTrackTapes__body", box_geo!(0.108, 0.024, 0.140)), &shell_mat);
    body.position().y = 0.012;
    cartridge.add(&body);
    // Large paper label on top.
    let label = mesh(plane_geo!(0.090, 0.110), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.025, 0);
    cartridge.add(&label);
    // Front notch (dark rectangular cutout at one edge).
    let notch = mesh(box_geo!(0.070, 0.020, 0.014), lambert!({ color: 0x0a0a0a }));
    notch.position().set(0, 0.012, 0.063);
    cartridge.add(&notch);
    // 2 small tape hub circles visible through notch.
    for side in [-1.0, 1.0] {
        let hub = mesh(pg("EightTrackTapes__hub", cyl_geo!(0.008, 0.008, 0.002, 8)), lambert!({ color: 0x606060 }));
        hub.position().set(side * 0.024, 0.023, 0.058);
        cartridge.add(&hub);
    }
    cartridge.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cartridge.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cartridge);
}

// R634 stuffed rabbit: 4% chance per level. Similar to bear but w/ long ears +
// smaller body.
pub fn place_stuffed_rabbits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bunny_colors: [u32; 5] = [0xf0e8d8, 0xd0d0d0, 0xd0a080, 0xf0d0d0, 0xa08060];
    let bunny_mat = lambert!({ color: pick(&bunny_colors) });
    let nose_mat = lambert!({ color: 0xd050a0 });
    let eye_mat = lambert!({ color: 0x101010 });
    let rabbit = group();
    // Body (smaller than bear, more upright).
    let body = mesh(pg("StuffedRabbits__body", sphere_geo!(0.035, 10, 8)), &bunny_mat);
    body.scale().set(0.9, 1, 0.9);
    body.position().y = 0.035;
    rabbit.add(&body);
    // Head sphere.
    let head = mesh(pg("StuffedRabbits__head", sphere_geo!(0.028, 10, 8)), &bunny_mat);
    head.position().y = 0.078;
    rabbit.add(&head);
    // 2 long tall ears (upright ellipsoids).
    // R664 track for ear twitch animator: (ear, phase, side)
    let mut rabbit_ears: Vec<(Obj, f64, f64)> = vec![];
    for side in [-1.0f64, 1.0] {
        let ear = mesh(pg("StuffedRabbits__ear", sphere_geo!(0.008, 6, 5)), &bunny_mat);
        ear.scale().set(0.7, 2.5, 0.5);
        ear.position().set(side * 0.008, 0.115, -0.005);
        let phase = rnd() * PI * 2.0 + (if side > 0.0 { 1.7 } else { 0.0 });
        {
            let mut u = ear.user();
            u.base_z = 0.0;
            u.phase = phase;
            u.set("side", side);
        }
        rabbit_ears.push((ear.clone(), phase, side));
        rabbit.add(&ear);
    }
    // 2 arms.
    for side in [-1.0, 1.0] {
        let arm = mesh(pg("TeddyBears__arm", sphere_geo!(0.014, 6, 5)), &bunny_mat);
        arm.scale().set(1.2, 0.7, 0.7);
        arm.position().set(side * 0.032, 0.040, 0.006);
        rabbit.add(&arm);
    }
    // 2 legs (larger — bunnies have big hind feet).
    for side in [-1.0, 1.0] {
        let leg = mesh(pg("StuffedRabbits__foot", sphere_geo!(0.018, 6, 5)), &bunny_mat);
        leg.scale().set(1, 0.5, 1.4);
        // under the face (-Z), toes out front and heels under the belly, soles on the floor
        leg.position().set(side * 0.018, 0.009, -0.028);
        rabbit.add(&leg);
    }
    // Small round tail on back.
    let tail = mesh(pg("StuffedRabbits__tail", sphere_geo!(0.008, 6, 5)), lambert!({ color: 0xffffff }));
    tail.position().set(0, 0.028, 0.030);
    rabbit.add(&tail);
    // Face: 2 eyes + pink nose.
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.0025, 5, 4)), &eye_mat);
        eye.position().set(side * 0.009, 0.084, -0.025);
        rabbit.add(&eye);
    }
    let nose = mesh(pg("TeddyBears__nose", sphere_geo!(0.004, 5, 4)), &nose_mat);
    nose.position().set(0, 0.076, -0.028);
    rabbit.add(&nose);
    rabbit.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    rabbit.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rabbit);
    let p = rabbit.pos();
    g.pp.plush_positions.push(v3(p.x, 0, p.z)); // R666
                                                // R664 subtle ear twitch — rare rapid rotations w/ long idle.
    g.animate(move |_g, _dt, t| {
        for (ear, phase, side) in &rabbit_ears {
            // Base + occasional short twitch (when sin > 0.96, add a rapid wobble).
            let trigger = (t * 0.2 + phase).sin();
            if trigger > 0.96 {
                ear.rotation().z = (t * 30.0).sin() * 0.08 * side;
            } else {
                ear.rotation().z = 0.0;
            }
        }
    });
}

// R635 stuffed elephant: 3% chance per level. Body + head + trunk + big ears +
// 4 legs + 2 tusks.
pub fn place_stuffed_elephants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 4] = [0x8090a0, 0xa090b0, 0xc0a080, 0x606870];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let tusk_mat = lambert!({ color: 0xf0e8d0 });
    let eye_mat = lambert!({ color: 0x101010 });
    let eleph = group();
    // Body (large ellipsoid).
    let body = mesh(pg("StuffedElephants__body", sphere_geo!(0.045, 10, 8)), &body_mat);
    body.scale().set(1.6, 1, 1);
    body.position().y = 0.045;
    eleph.add(&body);
    // Head sphere.
    let head = mesh(pg("StuffedElephants__head", sphere_geo!(0.030, 10, 8)), &body_mat);
    head.position().set(-0.055, 0.070, 0);
    eleph.add(&head);
    // Trunk (3 tapered segments curling down).
    for i in 0..3 {
        let t = i as f64 / 2.0;
        let r = 0.014 * (1.0 - t * 0.4);
        let seg = mesh(pg("StuffedElephants__trunk", cyl_geo!(r, r * 0.85, 0.014, 6)), &body_mat);
        seg.rotation().z = PI / 6.0 - i as f64 * 0.3;
        seg.position().set(-0.075 - i as f64 * 0.008, 0.055 - i as f64 * 0.011, 0);
        eleph.add(&seg);
    }
    // 2 big ears (flat discs).
    // R669 track for ear flap animator: (pivot, base y, side, phase)
    let mut eleph_ears: Vec<(Obj, f64, f64, f64)> = vec![];
    for side in [-1.0f64, 1.0] {
        let ear_mesh = mesh(pg("StuffedElephants__ear", circle_geo!(0.024, 8)), &body_mat);
        // hinged at its root, 4.5 mm inside the side of the head (the pivot
        // group the flap animator turns); the whole disc reaches out sideways
        let ear = group();
        ear_mesh.position().x = -0.024; // local -X is outward on both sides
        ear.add(&ear_mesh);
        ear.rotation().y = side * PI / 2.0;
        ear.position().set(-0.050, 0.078, side * 0.023);
        let base_y = side * PI / 2.0;
        let phase = rnd() * PI * 2.0 + (if side > 0.0 { 0.7 } else { 0.0 });
        {
            let mut u = ear.user();
            u.base_y = base_y;
            u.set("side", side);
            u.phase = phase;
        }
        eleph_ears.push((ear.clone(), base_y, side, phase));
        eleph.add(&ear);
    }
    // 4 stumpy legs.
    for i in 0..4 {
        let leg = mesh(pg("StuffedElephants__leg", cyl_geo!(0.012, 0.012, 0.025, 6)), &body_mat);
        leg.position().set(if i < 2 { -0.030 } else { 0.030 }, 0.012, if i % 2 == 0 { -0.020 } else { 0.020 });
        eleph.add(&leg);
    }
    // 2 tusks.
    for side in [-1.0, 1.0] {
        let tusk = mesh(pg("StuffedElephants__tusk", cone_geo!(0.003, 0.014, 5)), &tusk_mat);
        tusk.rotation().z = PI / 2.0;
        tusk.position().set(-0.086, 0.056, side * 0.010); // base 1 mm into the head
        eleph.add(&tusk);
    }
    // 2 eyes.
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.003, 5, 4)), &eye_mat);
        eye.position().set(-0.0716, 0.0811, side * 0.0200); // 2 mm proud of the head (was inside it)
        eleph.add(&eye);
    }
    eleph.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    eleph.rotation().y = rnd() * PI * 2.0;
    g.world.add(&eleph);
    let p = eleph.pos();
    g.pp.plush_positions.push(v3(p.x, 0, p.z)); // R666
                                                // R669 gentle ear flap — very slow rotation w/ occasional stronger flap.
    g.animate(move |_g, _dt, t| {
        for (ear, base, side, phase) in &eleph_ears {
            // Base rest rotation + slow drift (±0.15 rad).
            let drift = (t * 0.35 + phase).sin() * 0.15;
            // Occasional bigger flap when high sin threshold met.
            let flap = (t * 0.18 + phase * 1.3).sin();
            let big_flap = if flap > 0.92 { ((flap - 0.92) / 0.08 * PI).sin() * 0.35 * side } else { 0.0 };
            ear.rotation().y = base + drift + big_flap;
        }
    });
}
