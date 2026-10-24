//! The placers of this crate, in registration order.
use crate::textures::*;
use crate::{frame_bars, mount_on_wall, open_tiles, scatter_on_floor, tri_geo, wall_candidates, WallCand};
use game::prelude::*;

/// A uniformly random element (one `rnd()` draw). for colour tables.
fn pick_hex(a: &[u32]) -> u32 {
    a[(rnd() * a.len() as f64).floor() as usize]
}

// R521 hospital cross: 4% chance per level. Red + cross box wall-mounted.
pub(crate) fn place_hospital_crosses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let white_mat = basic_mat!({ color: 0xf0ece0 });
    let red_mat = basic_mat!({ color: 0xd82030 });
    let cross = group();
    let s = 0.14;
    // White backing.
    let back = mesh(pg("HospitalCrosses__back", box_geo!(s, s, 0.010)), &white_mat);
    back.position().z = -0.005;
    cross.add(&back);
    // Red vertical bar.
    let v_bar = mesh(pg("HospitalCrosses__vbar", box_geo!(s * 0.3, s * 0.9, 0.008)), &red_mat);
    v_bar.position().z = 0.002;
    cross.add(&v_bar);
    // Red horizontal bar.
    let h_bar = mesh(pg("HospitalCrosses__hbar", box_geo!(s * 0.9, s * 0.3, 0.008)), &red_mat);
    h_bar.position().z = 0.002;
    cross.add(&h_bar);
    mount_on_wall(g, &cross, &c, 0.005, 2.0, 0.6);
    g.world.add(&cross);
}

// R521 children's tea set: 5% chance per level. 3-4 mini cups + teapot.
pub(crate) fn place_childrens_tea_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let set_colors = [0xf0c0d0, 0xd0d8f0, 0xf0e8d8, 0xd0f0d8];
    let set_mat = lambert!({ color: pick_hex(&set_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let grp = group();
    // Teapot in center: sphere + spout + handle + lid.
    let pot = mesh(pg("ChildrensTeaSets__pot", sphere_geo!(0.024, 10, 8)), &set_mat);
    pot.scale().y = 0.8;
    pot.position().set(0, 0.019, 0);
    grp.add(&pot);
    let spout = mesh(pg("ChildrensTeaSets__spout", cyl_geo!(0.005, 0.003, 0.020, 5)), &set_mat);
    spout.rotation().z = -0.5;
    spout.position().set(0.024, 0.024, 0);
    grp.add(&spout);
    let handle = mesh(pg("ChildrensTeaSets__handle", torus_geo!(0.008, 0.002, 4, 8, PI)), &set_mat);
    handle.rotation().z = PI / 2.0; // C in the XY plane, bulging out behind (-X)
    handle.position().set(-0.0215, 0.020, 0); // both ends in the pot wall
    grp.add(&handle);
    let lid = mesh(pg("ChildrensTeaSets__lid", sphere_geo!(0.006, 6, 5)), &set_mat);
    lid.position().set(0, 0.038, 0);
    grp.add(&lid);
    // 3-4 mini cups around.
    let n_cups = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_cups {
        let a = (i as f64 / n_cups as f64) * PI * 2.0;
        let cup = mesh(pg("ChildrensTeaSets__cup", cyl_geo!(0.011, 0.008, 0.015, 8, 1, true)), &set_mat);
        cup.position().set(a.cos() * 0.06, 0.008, a.sin() * 0.06);
        grp.add(&cup);
        // Saucer.
        let saucer = mesh(pg("ChildrensTeaSets__saucer", cyl_geo!(0.018, 0.018, 0.002, 10)), &set_mat);
        saucer.position().set(a.cos() * 0.06, 0.001, a.sin() * 0.06);
        grp.add(&saucer);
        // Tea residue.
        if rnd() < 0.5 {
            let tea = mesh(circle_geo!(0.008, 8), &tea_mat);
            tea.rotation().x = -PI / 2.0;
            tea.position().set(a.cos() * 0.06, 0.013, a.sin() * 0.06);
            grp.add(&tea);
        }
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R520 coin scatter: 10% chance per level. 5-12 loose coins on floor.
pub(crate) fn place_coin_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let coin_colors = [0xd8b040, 0xa08050, 0xc0c0c0, 0xe0a060];
    let grp = group();
    let n_coins = 5 + (rnd() * 8.0).floor() as i32;
    for i in 0..n_coins {
        let coin_mat = lambert!({ color: pick_hex(&coin_colors) });
        let coin = mesh(pg("CoinScatters__coin", cyl_geo!(0.008, 0.008, 0.001, 10)), &coin_mat);
        coin.position().set((rnd() - 0.5) * 0.30, 0.001, (rnd() - 0.5) * 0.30);
        coin.rotation().y = rnd() * PI * 2.0;
        // 20% chance edge-on tilted.
        if rnd() < 0.20 {
            coin.rotation().z = PI / 2.0;
            coin.position().y = 0.008;
        }
        grp.add(&coin);
    }
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R519 wedding ring: 6% chance per level. Small gold torus on floor.
pub(crate) fn place_wedding_rings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let gold_colors = [0xd8b040, 0xe8c860, 0xd0d0d0, 0xf0d8d8];
    let gold_mat = lambert!({ color: pick_hex(&gold_colors) });
    let gem_mat = basic_mat!({ color: 0xe0f0ff, transparent: true, opacity: 0.9 });
    let ring = group();
    // Band torus.
    let band = mesh(pg("WeddingRings__band", torus_geo!(0.010, 0.002, 5, 14)), &gold_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.002;
    ring.add(&band);
    // 40% chance small diamond gem on top (an octahedron).
    if rnd() < 0.40 {
        let gem = mesh(octa_geo!(0.003), &gem_mat);
        gem.position().y = 0.006;
        ring.add(&gem);
    }
    scatter_on_floor(g, &ring, x, z, 2.4);
    g.world.add(&ring);
}

// R519 water puddle: 8% chance per level. Dark disc with a radial ripple animator.
pub(crate) fn place_water_puddles(g: &mut Game, _dist: &[i32]) {
    g.pp.puddle_positions.clear(); // R520 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let water_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.65, depthWrite: false });
    let ripple_mat = basic_mat!({ color: 0xa0c0d0, transparent: true, opacity: 0.4, depthWrite: false });
    let puddle = group();
    let p_radius = 0.10 + rnd() * 0.10;
    // Main puddle disc.
    let water = mesh(circle_geo!(p_radius, 14), &water_mat);
    water.rotation().x = -PI / 2.0;
    water.position().y = 0.003;
    puddle.add(&water);
    // Ripple ring (small torus at surface).
    let ripple = mesh(torus_geo!(p_radius * 0.3, 0.002, 4, 14), &ripple_mat);
    ripple.rotation().x = PI / 2.0;
    ripple.position().y = 0.004;
    puddle.add(&ripple);
    // Ripple radial expansion animator.
    let ripple_phase = rnd() * PI * 2.0;
    let ripple_speed = 0.5 + rnd() * 0.3;
    {
        let ripple = ripple.clone();
        g.animate(move |_g, _dt, t| {
            let frac = ((t * ripple_speed + ripple_phase) % 4.0) / 4.0;
            ripple.scale().set_scalar(0.3 + frac * 2.5);
            ripple.mat().set_opacity((0.4 * (1.0 - frac)).max(0.0));
        });
    }
    scatter_on_floor(g, &puddle, x, z, 2.4);
    g.world.add(&puddle);
    // R520 track for drip audio.
    let p = puddle.pos();
    g.pp.puddle_positions.push(v3(p.x, 0.003, p.z));
}

// R518 porcelain cup + saucer: 7% chance per level.
pub(crate) fn place_porcelain_cup_saucers(g: &mut Game, _dist: &[i32]) {
    g.pp.teacup_positions.clear(); // R525 reset per level
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let porcelain_colors = [0xe8dcc0, 0xf0d8c0, 0xd8d8e0, 0xe8c0a0];
    let porcelain_mat = lambert!({ color: pick_hex(&porcelain_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let pattern_mat = lambert!({ color: 0x4030a0 });
    let set = group();
    // Saucer (flat disc + slight rim).
    let saucer = mesh(pg("PorcelainCupSaucers__saucer", cyl_geo!(0.045, 0.045, 0.004, 14)), &porcelain_mat);
    saucer.position().y = 0.002;
    set.add(&saucer);
    let saucer_rim = mesh(pg("PorcelainCupSaucers__saucerRim", torus_geo!(0.043, 0.002, 5, 14)), &porcelain_mat);
    saucer_rim.rotation().x = PI / 2.0;
    saucer_rim.position().y = 0.005;
    set.add(&saucer_rim);
    // Cup (short cylinder).
    let cup = mesh(pg("PorcelainCupSaucers__cup", cyl_geo!(0.024, 0.020, 0.030, 12, 1, true)), &porcelain_mat);
    cup.position().y = 0.019;
    set.add(&cup);
    // Cup bottom.
    let cup_bottom = mesh(circle_geo!(0.020, 12), &porcelain_mat);
    cup_bottom.rotation().x = -PI / 2.0;
    cup_bottom.position().y = 0.005;
    set.add(&cup_bottom);
    // Cup handle (torus arc): loop in the radial plane with its gap on the cup wall.
    let handle = mesh(pg("PorcelainCupSaucers__handle", torus_geo!(0.010, 0.002, 4, 8, PI * 1.4)), &porcelain_mat);
    handle.rotation().z = -2.2;
    handle.rotation().y = 0.0;
    handle.position().set(0.028, 0.020, 0);
    set.add(&handle);
    // Tea residue (60% chance).
    if rnd() < 0.60 {
        let tea = mesh(circle_geo!(0.018, 12), &tea_mat);
        tea.rotation().x = -PI / 2.0;
        tea.position().y = 0.026;
        set.add(&tea);
    }
    // Small blue pattern band on cup (60% chance).
    if rnd() < 0.60 {
        let band = mesh(cyl_geo!(0.0245, 0.0245, 0.003, 12, 1, true), &pattern_mat);
        band.scale().set(0.935, 1, 0.935); // painted on the cup wall, not floating off it
        band.position().y = 0.010;
        set.add(&band);
    }
    scatter_on_floor(g, &set, x, z, 2.4);
    g.world.add(&set);
    // R525 track for clink audio.
    let p = set.pos();
    g.pp.teacup_positions.push(v3(p.x, 0.02, p.z));
}

// R518 broken alarm bell: 4% chance per level. Wall bell dome + dangling clapper.
pub(crate) fn place_alarm_bells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let plate_mat = lambert!({ color: 0x6a3820 });
    let bell_mat = lambert!({ color: 0xc02818 });
    let clapper_mat = lambert!({ color: 0x181410 });
    let bell = group();
    // Back plate.
    let plate = mesh(pg("AlarmBells__plate", box_geo!(0.12, 0.16, 0.014)), &plate_mat);
    plate.position().set(0, 0, -0.007);
    bell.add(&plate);
    // Bell dome (half-sphere).
    let dome = mesh(pg("AlarmBells__gong", sphere_geo!(0.055, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &bell_mat);
    dome.position().set(0, 0.020, 0.020);
    dome.rotation().x = PI / 2.0; // gong faces the room, apex at the nut
    bell.add(&dome);
    // Bell top small button.
    let top = mesh(pg("AlarmBells__nut", cyl_geo!(0.008, 0.008, 0.008, 6)), &bell_mat);
    top.rotation().x = PI / 2.0;
    top.position().set(0, 0.020, 0.075);
    bell.add(&top);
    // Dangling clapper (rusted, off-center).
    let clapper_pivot = group();
    clapper_pivot.position().set(0, 0.020, 0.080); // hangs from the centre nut
    let clapper_arm = mesh(pg("AlarmBells__arm", cyl_geo!(0.001, 0.001, 0.04, 3)), &clapper_mat);
    clapper_arm.position().y = -0.020;
    clapper_pivot.add(&clapper_arm);
    let ball = mesh(pg("AlarmBells__ball", sphere_geo!(0.006, 6, 5)), &clapper_mat);
    ball.position().y = -0.042;
    clapper_pivot.add(&ball);
    clapper_pivot.rotation().z = 0.4; // hanging tilted, broken
    bell.add(&clapper_pivot);
    mount_on_wall(g, &bell, &c, 0.007, 2.20, 0.6);
    g.world.add(&bell);
}

// R517 rust stain streak: 5% chance per level. Vertical rust drip decal on wall.
pub(crate) fn place_rust_stain_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: rust_stain_texture(), transparent: true, opacity: 0.9, depthWrite: false });
    let decal = mesh(plane_geo!(0.24, 0.60), &mat);
    mount_on_wall(g, &decal, &c, 0.005, 1.5, 0.5);
    g.world.add(&decal);
}

// R517 forgotten wristwatch: 6% chance per level. Watch face + leather band on floor.
pub(crate) fn place_forgotten_wristwatches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let band_colors = [0x2a1810, 0x603820, 0x404040, 0xd8b040];
    let band_mat = lambert!({ color: pick_hex(&band_colors) });
    let case_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case disc.
    let case_ = mesh(pg("ForgottenWristwatches__case", cyl_geo!(0.020, 0.020, 0.006, 12)), &case_mat);
    case_.position().y = 0.003;
    watch.add(&case_);
    // Face.
    let face = mesh(circle_geo!(0.018, 12), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0065;
    watch.add(&face);
    // Band left half (curled); each strap starts at a lug on the case edge.
    let band_l = mesh(pg("ForgottenWristwatches__strapHoles", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_l.rotation().x = -PI / 2.0;
    band_l.position().set(-0.030, 0.005, -0.020);
    watch.add(&band_l);
    // Band right half.
    let band_r = mesh(pg("ForgottenWristwatches__strapKeeper", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_r.set_rotation_order(EulerOrder::YXZ); // lie flat first, then turn half round
    band_r.rotation().x = -PI / 2.0;
    band_r.rotation().y = PI;
    band_r.position().set(0.030, 0.005, 0.020);
    watch.add(&band_r);
    scatter_on_floor(g, &watch, x, z, 2.4);
    g.world.add(&watch);
}

// R516 wall claw marks: 4% chance per level. 3 diagonal scratch lines on wall.
pub(crate) fn place_claw_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: claw_marks_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(0.50, 0.70), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 1.4, 0.5);
    g.world.add(&decal);
}

// R516 rusted nails: 10% chance per level. 6-12 rusted nails scattered on floor.
pub(crate) fn place_rusted_nails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let rust_colors = [0x6a3820, 0x504028, 0x8a4030];
    let grp = group();
    let n_nails = 6 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_nails {
        let nail_mat = lambert!({ color: pick_hex(&rust_colors) });
        let nail = group();
        // Shaft.
        let shaft = mesh(pg("RustedNails__shaft", cyl_geo!(0.001, 0.001, 0.032, 4)), &nail_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().set(0.010, 0.001, 0);
        nail.add(&shaft);
        // Head.
        let head = mesh(pg("RustedNails__head", cyl_geo!(0.003, 0.003, 0.002, 6)), &nail_mat);
        head.rotation().z = PI / 2.0;
        head.position().set(-0.007, 0.001, 0);
        nail.add(&head);
        // Tip point.
        let tip = mesh(cone_geo!(0.001, 0.005, 4), &nail_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.028, 0.001, 0);
        nail.add(&tip);
        // rests on its head rim and point: tilt the shank down toward the tip
        nail.position().set((rnd() - 0.5) * 0.25, 0.0016, (rnd() - 0.5) * 0.20);
        nail.rotation().z = -0.055;
        nail.rotation().y = rnd() * PI * 2.0;
        grp.add(&nail);
    }
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R515 biohazard sign: 4% chance per level. Yellow trefoil warning on wall.
pub(crate) fn place_biohazard_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: biohazard_texture() });
    let sign = mesh(pg("BiohazardSigns__plate", plane_geo!(0.20, 0.20)), &mat);
    mount_on_wall(g, &sign, &c, 0.005, 1.70, 0.6);
    g.world.add(&sign);
}

// R515 teddy paw: 6% chance per level. Dismembered plush paw + torn stuffing.
pub(crate) fn place_teddy_paws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let fur_colors = [0x8a5030, 0x604020, 0x503020];
    let fur_mat = lambert!({ color: pick_hex(&fur_colors) });
    let stuffing_mat = lambert!({ color: 0xe8e0d0, transparent: true, opacity: 0.85 });
    let seam_mat = lambert!({ color: 0x2a1810 });
    let paw = group();
    // Paw ball (sphere).
    let ball = mesh(pg("TeddyPaws__paw", sphere_geo!(0.020, 8, 6)), &fur_mat);
    ball.scale().set(1, 1.4, 1);
    ball.position().y = 0.020;
    paw.add(&ball);
    // Torn edge: faces up into the ragged crater on the paw's top.
    let torn = mesh(circle_geo!(0.018, 8), &seam_mat);
    torn.rotation().x = -PI / 2.0;
    torn.scale().set_scalar(0.62);
    torn.position().y = 0.0445;
    paw.add(&torn);
    // Stuffing puffs (3 small clouds nearby).
    for i in 0..3 {
        let puff = mesh(sphere_geo!(0.008, 6, 5), &stuffing_mat);
        puff.scale().set(1.0 + rnd() * 0.5, 0.6, 1.0 + rnd() * 0.5);
        puff.position().set((rnd() - 0.5) * 0.08, 0.006, 0.04 + (rnd() - 0.5) * 0.04);
        paw.add(&puff);
    }
    scatter_on_floor(g, &paw, x, z, 2.4);
    paw.rotation().z = (rnd() - 0.5) * 0.4;
    g.world.add(&paw);
}

// R514 caution tape strip: 5% chance per level. Striped tape on wall.
pub(crate) fn place_caution_tapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = lambert!({ map: caution_tape_texture(), side: DoubleSide });
    let tape = mesh(plane_geo!(2.2, 0.05), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let center_y = 1.1;
    if c.dx.abs() == 1 {
        tape.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tape.position().set(wall_x, center_y, wall_z);
    } else {
        tape.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        tape.position().set(wall_x, center_y, wall_z);
    }
    // Small tilt for organic hang.
    tape.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&tape);
}

// R514 broken green bottle: 6% chance per level. Base + shards + neck fragment.
pub(crate) fn place_broken_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let glass_colors = [0x306040, 0x603820, 0x304060];
    let glass_mat = basic_mat!({ color: pick_hex(&glass_colors), transparent: true, opacity: 0.7 });
    let grp = group();
    // Base stub (bottom cylinder).
    let base = mesh(cyl_geo!(0.024, 0.028, 0.020, 10), &glass_mat);
    base.position().y = 0.010;
    grp.add(&base);
    // Jagged rim tabs on base.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + rnd() * 0.4;
        let tab = mesh(box_geo!(0.008, 0.010 + rnd() * 0.008, 0.003), &glass_mat);
        tab.position().set(a.cos() * 0.024, 0.025, a.sin() * 0.024);
        tab.rotation().y = -a;
        grp.add(&tab);
    }
    // Separated neck fragment nearby.
    let neck = mesh(cyl_geo!(0.008, 0.011, 0.035, 8), &glass_mat);
    neck.rotation().z = PI / 2.0;
    neck.position().set(0.08, 0.008, 0.05);
    grp.add(&neck);
    // scattered shards.
    for i in 0..6 {
        let s1 = 0.014 + rnd() * 0.016;
        let shard_geo = tri_geo([0.0, 0.0, 0.0, s1, 0.0, s1 * (rnd() - 0.5), s1 * 0.5, 0.0, s1 * (0.5 + rnd() * 0.5)]);
        let shard = mesh(shard_geo, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.20, 0.002, (rnd() - 0.5) * 0.20);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R513 chalk arrow: 5% chance per level. Wall chalk arrow + cryptic word.
pub(crate) fn place_chalk_arrows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: chalk_arrow_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(0.50, 0.25), &mat);
    // Random flip so arrow points either direction.
    if rnd() < 0.5 {
        decal.rotation().y = PI;
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let center_y = 1.55;
    let wall_offset = if decal.rot().y == PI { PI } else { 0.0 };
    if c.dx.abs() == 1 {
        decal.rotation().y += if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        decal.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        decal.rotation().y += if c.dz > 0 { PI } else { 0.0 };
        decal.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
    g.world.add(&decal);
}

// R513 shattered eyeglasses: 6% chance per level. Frame + lenses + broken temple.
pub(crate) fn place_shattered_eyeglasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x181008 });
    let lens_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.4 });
    let glasses = group();
    // 2 lens frames (torus).
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("ShatteredEyeglasses__rim", torus_geo!(0.020, 0.002, 4, 12)), &frame_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(s * 0.024, 0.002, 0);
        glasses.add(&rim);
        // Lens disc.
        let lens = mesh(circle_geo!(0.018, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(s * 0.024, 0.003, 0);
        glasses.add(&lens);
    }
    // Bridge between lenses.
    let bridge = mesh(pg("ShatteredEyeglasses__bridge", cyl_geo!(0.001, 0.001, 0.010, 3)), &frame_mat);
    bridge.rotation().z = PI / 2.0;
    bridge.position().set(0, 0.002, 0);
    glasses.add(&bridge);
    // 1 temple attached (other broken off).
    let temple_side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    let temple = mesh(pg("ShatteredEyeglasses__temple", cyl_geo!(0.001, 0.001, 0.10, 3)), &frame_mat);
    temple.rotation().x = PI / 2.0;
    temple.rotation().z = 0.1;
    temple.position().set(temple_side * 0.044, 0.002, -0.05);
    glasses.add(&temple);
    // Broken temple laying separately.
    let broken = mesh(pg("ShatteredEyeglasses__broken", cyl_geo!(0.001, 0.001, 0.06, 3)), &frame_mat);
    broken.rotation().z = PI / 3.0;
    broken.position().set(-temple_side * 0.10, 0.002, 0.04);
    glasses.add(&broken);
    scatter_on_floor(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}

// R512 wooden mouse trap: 7% chance per level. Wood base + spring bar + cheese.
pub(crate) fn place_mouse_traps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0xa08050 });
    let wire_mat = lambert!({ color: 0xc0c0c0 });
    let cheese_mat = lambert!({ color: 0xf0d040 });
    let trap = group();
    // Base plank.
    let base = mesh(pg("MouseTraps__base", box_geo!(0.06, 0.006, 0.038)), &wood_mat);
    base.position().y = 0.003;
    trap.add(&base);
    // Spring bar (U-shape wire).
    let spring_loop = mesh(pg("MouseTraps__snap", torus_geo!(0.016, 0.001, 4, 8)), &wire_mat);
    spring_loop.position().set(-0.015, 0.010, 0);
    spring_loop.rotation().x = PI / 2.0;
    trap.add(&spring_loop);
    // Bar wire across base.
    let bar = mesh(pg("MouseTraps__bar", cyl_geo!(0.001, 0.001, 0.036, 4)), &wire_mat);
    bar.rotation().x = PI / 2.0;
    bar.position().set(0.010, 0.007, 0);
    trap.add(&bar);
    // Trigger tab.
    let trigger = mesh(pg("MouseTraps__trigger", box_geo!(0.006, 0.002, 0.014)), &wire_mat);
    trigger.position().set(0.020, 0.008, 0);
    trap.add(&trigger);
    // Cheese bait cube.
    let cheese = mesh(pg("MouseTraps__cheese", box_geo!(0.006, 0.005, 0.005)), &cheese_mat);
    cheese.position().set(0.020, 0.009, 0);
    trap.add(&cheese);
    scatter_on_floor(g, &trap, x, z, 2.4);
    g.world.add(&trap);
}

// R512 wall crayon graffiti: 5% chance per level. Child-drawn scene on wall.
pub(crate) fn place_crayon_graffiti(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: crayon_graffiti_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(0.60, 0.50), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 0.9, 0.6); // low-mounted (child-height)
    g.world.add(&decal);
}

// R511 family portrait: 5% chance per level. Framed photo w/ 3-4 silhouettes.
pub(crate) fn place_family_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x6a3820 });
    let photo_mat = lambert!({ map: family_portrait_texture(), side: DoubleSide });
    let portrait = group();
    let (w, h, t, fw) = (0.20, 0.16, 0.015, 0.014);
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("FamilyPortraits__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        portrait.add(&bar);
    }
    let photo = mesh(plane_geo!(w, h), &photo_mat);
    photo.position().set(0, 0, t / 2.0 + 0.001);
    portrait.add(&photo);
    mount_on_wall(g, &portrait, &c, 0.010, 1.60, 0.6);
    portrait.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&portrait);
}

// R511 cassette tape: 8% chance per level. Rectangle w/ 2 spool holes + label.
pub(crate) fn place_cassette_tapes(g: &mut Game, _dist: &[i32]) {
    g.pp.cassette_positions.clear(); // R524 reset per level
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
    scatter_on_floor(g, &cass, x, z, 2.4);
    g.world.add(&cass);
    // R524 track for hiss audio.
    let p = cass.pos();
    g.pp.cassette_positions.push(v3(p.x, 0.01, p.z));
}

// R510 mop bucket: 6% chance per level. Plastic bucket with a mop in it.
pub(crate) fn place_mop_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bucket_colors = [0x2050c0, 0x304050, 0xa02040];
    let bucket_mat = lambert!({ color: pick_hex(&bucket_colors) });
    let handle_mat = lambert!({ color: 0x606870 });
    let stick_mat = lambert!({ color: 0x8a5030 });
    let mop_mat = lambert!({ color: 0xa08a70 });
    let water_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.7 });
    let bucket = group();
    // Bucket body (open cylinder, wider at top).
    let body = mesh(pg("MopBuckets__body", cyl_geo!(0.075, 0.055, 0.15, 12, 1, true)), &bucket_mat);
    body.position().y = 0.075;
    bucket.add(&body);
    // Base disc.
    let base = mesh(circle_geo!(0.055, 12), &bucket_mat);
    base.rotation().x = -PI / 2.0;
    base.position().y = 0.001;
    bucket.add(&base);
    // Water inside (60% chance).
    if rnd() < 0.60 {
        let water = mesh(circle_geo!(0.070, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.scale().set(0.93, 0.93, 1); // inside the wall at this height
        water.position().y = 0.10;
        bucket.add(&water);
    }
    // Metal handle (torus arc).
    let handle = mesh(pg("MopBuckets__handle", torus_geo!(0.075, 0.003, 4, 12, PI)), &handle_mat);
    handle.position().y = 0.150;
    handle.rotation().x = PI / 2.0;
    bucket.add(&handle);
    // Mop leaning against bucket rim: foot in the mop head inside the bucket.
    let stick_len = 0.60;
    let stick = mesh(pg("MopBuckets__stick", cyl_geo!(0.008, 0.008, stick_len, 6)), &stick_mat);
    stick.rotation().z = -0.5;
    stick.position().set(0.012 + (0.5f64).sin() * stick_len / 2.0, 0.05 + (0.5f64).cos() * stick_len / 2.0, 0);
    bucket.add(&stick);
    // Mop head (dirty bundle on the stick's foot, sitting in the bucket).
    let mop_head = mesh(pg("MopBuckets__mopHead", sphere_geo!(0.045, 8, 6)), &mop_mat);
    mop_head.scale().set(0.9, 1.2, 0.9);
    mop_head.position().set(0.012, 0.058, 0);
    bucket.add(&mop_head);
    scatter_on_floor(g, &bucket, x, z, 1.6);
    g.world.add(&bucket);
}

// R510 dust bunnies: 10% chance per level. 5-10 small gray fuzz balls.
pub(crate) fn place_dust_bunnies(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let dust_colors = [0x707070, 0x606060, 0x808080];
    let grp = group();
    let n_bunnies = 5 + (rnd() * 6.0).floor() as i32;
    for i in 0..n_bunnies {
        let dust_mat = lambert!({ color: pick_hex(&dust_colors), transparent: true, opacity: 0.65 });
        let bunny = mesh(sphere_geo!(0.008 + rnd() * 0.006, 6, 5), &dust_mat);
        bunny.scale().set(1.0 + rnd() * 0.4, 0.5 + rnd() * 0.3, 1.0 + rnd() * 0.4);
        bunny.position().set((rnd() - 0.5) * 0.40, 0.006, (rnd() - 0.5) * 0.40);
        grp.add(&bunny);
    }
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R509 open coloring book: 6% chance per level. Cover + open spread + crayon.
pub(crate) fn place_coloring_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let cover_mat = lambert!({ color: 0xd82060 });
    let page_mat = lambert!({ map: coloring_texture(), side: DoubleSide });
    let book = group();
    // Cover under.
    let cover = mesh(pg("ColoringBooks__cover", box_geo!(0.22, 0.003, 0.14)), &cover_mat);
    cover.position().y = 0.002;
    book.add(&cover);
    // Pages.
    let pages = mesh(plane_geo!(0.20, 0.13), &page_mat);
    pages.rotation().x = -PI / 2.0;
    pages.position().y = 0.005;
    book.add(&pages);
    // Crayon nearby.
    let crayon_colors = [0xd82040, 0x2050d8, 0xd8b040];
    let crayon_mat = lambert!({ color: pick_hex(&crayon_colors) });
    let crayon = mesh(pg("ColoringBooks__crayon", cyl_geo!(0.004, 0.004, 0.05, 6)), &crayon_mat);
    crayon.rotation().z = PI / 2.0;
    crayon.position().set(0.08, 0.004, -0.05);
    crayon.rotation().y = 0.5;
    book.add(&crayon);
    scatter_on_floor(g, &book, x, z, 2.0);
    g.world.add(&book);
}

// R509 loose vinyl record: 8% chance per level. Black disc w/ coloured label.
pub(crate) fn place_vinyl_record_discs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let vinyl_mat = lambert!({ color: 0x101008 });
    let label_colors = [0xa02040, 0x2050d8, 0xd8b040, 0x30a030];
    let label_mat = lambert!({ color: pick_hex(&label_colors) });
    let disc = group();
    // Main vinyl.
    let body = mesh(pg("VinylRecordDiscs__disc", cyl_geo!(0.078, 0.078, 0.002, 20)), &vinyl_mat);
    body.position().y = 0.001;
    disc.add(&body);
    // Label.
    let label = mesh(circle_geo!(0.025, 12), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0025;
    disc.add(&label);
    // Center hole (tiny dark spot).
    let hole = mesh(circle_geo!(0.003, 6), &vinyl_mat);
    hole.rotation().x = -PI / 2.0;
    hole.position().y = 0.0028;
    disc.add(&hole);
    scatter_on_floor(g, &disc, x, z, 2.4);
    g.world.add(&disc);
}

// R508 broken picture frame: 6% chance per level. Frame face-down + glass shards.
pub(crate) fn place_broken_picture_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_colors = [0x603820, 0x8a5030, 0x2a1810, 0xd8b040];
    let frame_mat = lambert!({ color: pick_hex(&frame_colors) });
    let glass_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.5 });
    let back_mat = lambert!({ color: 0xe8dcc0 });
    let grp = group();
    let (w, h, t, fw) = (0.22, 0.28, 0.015, 0.024);
    // Frame face-down (positive Y = back visible).
    let frame = group();
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("BrokenPictureFrames__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        frame.add(&bar);
    }
    // Paper backing.
    let back = mesh(plane_geo!(w, h), &back_mat);
    back.rotation().y = PI; // faces out of the back (-z)
    back.position().set(0, 0, -t / 2.0 - 0.001);
    frame.add(&back);
    // face-down as intended: front (+z) to the floor, backing on top
    frame.rotation().x = PI / 2.0;
    frame.position().y = 0.008;
    grp.add(&frame);
    // 5-8 scattered glass shards around frame.
    let n_shards = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_shards {
        let s1 = 0.020 + rnd() * 0.020;
        let shard_geo = tri_geo([0.0, 0.0, 0.0, s1, 0.0, s1 * (rnd() - 0.5) * 0.5, s1 * 0.5, 0.0, s1 * (0.5 + rnd() * 0.5)]);
        let shard = mesh(shard_geo, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.36, 0.002, (rnd() - 0.5) * 0.36);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

// R507 opened suitcase: 5% chance per level. Base + lifted lid + clothes piles.
pub(crate) fn place_opened_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let leather_mat = lambert!({ color: pick_hex(&leather_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let cloth_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xf0ece0, 0x2a2418];
    let suitcase = group();
    let (bw, bh, bd) = (0.22, 0.05, 0.14);
    // Base (bottom half).
    let base = mesh(pg("OpenedSuitcases__base", box_geo!(bw, bh, bd)), &leather_mat);
    base.position().y = bh / 2.0;
    suitcase.add(&base);
    // Lid (raised, tilted open), pivoting on its back bottom edge (the hinge).
    let lid = group();
    let lid_box = mesh(pg("OpenedSuitcases__lid", box_geo!(bw, bh, bd)), &leather_mat);
    lid_box.position().set(0, bh / 2.0, bd / 2.0);
    lid.add(&lid_box);
    lid.position().set(0, bh, -bd / 2.0);
    lid.rotation().x = -PI / 3.0;
    suitcase.add(&lid);
    // Brass corner reinforcements at 4 corners of base.
    for (cx, cz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let corner = mesh(pg("OpenedSuitcases__stud", cyl_geo!(0.008, 0.008, 0.008, 5)), &brass_mat);
        corner.position().set(cx * (bw / 2.0 - 0.004), 0.004, cz * (bd / 2.0 - 0.004));
        suitcase.add(&corner);
    }
    // 4-6 clothes piles inside base.
    let n_clothes = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_clothes {
        let cloth_mat = lambert!({ color: pick_hex(&cloth_colors) });
        // Wrinkled cloth.
        let cloth =
            mesh(pg("OpenedSuitcases__cloth", box_geo!(0.05 + rnd() * 0.04, 0.008 + rnd() * 0.006, 0.03 + rnd() * 0.03)), &cloth_mat);
        cloth.position().set((rnd() - 0.5) * (bw - 0.05), bh + 0.005 + i as f64 * 0.003, (rnd() - 0.5) * (bd - 0.03));
        cloth.rotation().y = rnd() * PI * 2.0;
        cloth.rotation().z = (rnd() - 0.5) * 0.2;
        suitcase.add(&cloth);
    }
    scatter_on_floor(g, &suitcase, x, z, 1.8);
    g.world.add(&suitcase);
}

// R506 rusted tin can: 8% chance per level. Small rusted can, sometimes tipped.
pub(crate) fn place_rusted_tin_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let can_colors = [0x6a4028, 0x804830, 0x8a3020, 0x503020];
    let can_mat = lambert!({ color: pick_hex(&can_colors) });
    let label_mat = lambert!({ color: 0xa08a70 });
    let can = group();
    // Can body.
    let body = mesh(pg("RustedTinCans__body", cyl_geo!(0.028, 0.028, 0.075, 12)), &can_mat);
    body.position().y = 0.038;
    can.add(&body);
    // Faded paper label band.
    let label = mesh(pg("RustedTinCans__label", cyl_geo!(0.029, 0.029, 0.035, 12, 1, true)), &label_mat);
    label.position().y = 0.038;
    can.add(&label);
    // Rim discs top + bottom.
    for yy in [0.003, 0.072] {
        let rim = mesh(cyl_geo!(0.028, 0.028, 0.002, 12), &can_mat);
        rim.position().y = yy;
        can.add(&rim);
    }
    // 40% tipped on side.
    if rnd() < 0.40 {
        can.rotation().z = PI / 2.0;
        can.position().y = 0.028;
    }
    can.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    can.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}

// R505 wall taxidermy deer head: 4% chance per level. Deer bust on a plaque.
pub(crate) fn place_deer_head_mounts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let plaque_mat = lambert!({ color: 0x603820 });
    let deer_mat = lambert!({ color: 0x8a6540 });
    let antler_mat = lambert!({ color: 0xe8dcc0 });
    let eye_mat = basic_mat!({ color: 0x100804 });
    let mount = group();
    // Wood plaque (shield shape approximation).
    let plaque = mesh(pg("DeerHeadMounts__plaque", box_geo!(0.28, 0.32, 0.020)), &plaque_mat);
    plaque.position().set(0, 0, -0.014);
    mount.add(&plaque);
    // Deer head (elongated sphere).
    let head = mesh(pg("DeerHeadMounts__head", sphere_geo!(0.070, 12, 10)), &deer_mat);
    head.scale().set(0.85, 1.2, 0.90);
    head.position().set(0, 0, 0.040);
    mount.add(&head);
    // Snout.
    let snout = mesh(pg("DeerHeadMounts__snout", sphere_geo!(0.040, 8, 6)), &deer_mat);
    snout.scale().set(0.7, 0.7, 1);
    snout.position().set(0, -0.055, 0.075);
    mount.add(&snout);
    // Nose.
    let nose = mesh(sphere_geo!(0.008, 6, 5), &eye_mat);
    nose.position().set(0, -0.075, 0.105);
    mount.add(&nose);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.006, 5, 4), &eye_mat);
        eye.position().set(s * 0.024, 0.010, 0.088);
        mount.add(&eye);
    }
    // 2 ears (small triangles): tips lean outwards, base on the head's crown.
    for s in [-1.0, 1.0] {
        let ear = mesh(pg("DeerHeadMounts__ear", cone_geo!(0.014, 0.030, 4)), &deer_mat);
        ear.position().set(s * 0.050, 0.065, 0.020);
        ear.rotation().z = -s * 0.5;
        mount.add(&ear);
    }
    // 2 antlers - each with branches (Y-shape approximation).
    for s in [-1.0, 1.0] {
        // Main stem.
        let stem = mesh(pg("DeerHeadMounts__beam", cyl_geo!(0.005, 0.005, 0.10, 4)), &antler_mat);
        stem.position().set(s * 0.025, 0.100, 0.020);
        stem.rotation().z = s * -0.3;
        mount.add(&stem);
        // Branches.
        for b in 0..3 {
            let b = b as f64;
            let branch = mesh(pg("DeerHeadMounts__tine", cyl_geo!(0.003, 0.003, 0.05, 4)), &antler_mat);
            branch.position().set(s * (0.040 + b * 0.010), 0.100 + b * 0.020 - 0.010, 0.020);
            branch.rotation().z = s * -0.9;
            mount.add(&branch);
        }
    }
    mount_on_wall(g, &mount, &c, 0.005, 2.00, 0.6);
    g.world.add(&mount);
}
