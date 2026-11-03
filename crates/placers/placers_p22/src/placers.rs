//! The placers of this crate, in registration order.
use crate::textures::*;
use crate::{buffer_geo, frame_bars, mount_on_wall, open_tiles, scatter_on_floor, wall_candidates, WallCand};
use game::prelude::*;

/// A uniformly random element (one `rnd()` draw). for colour tables.
fn pick_hex(a: &[u32]) -> u32 {
    a[(rnd() * a.len() as f64).floor() as usize]
}

// R505 overturned tricycle: 4% chance per level. Small kid tricycle tipped on side.
pub(crate) fn place_overturned_tricycles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030];
    let frame_mat = lambert!({ color: pick_hex(&frame_colors) });
    let wheel_mat = lambert!({ color: 0x181008 });
    let seat_mat = lambert!({ color: 0xa02040 });
    let handle_mat = lambert!({ color: 0xf0ece0 });
    let tri = group();
    // Main frame bar (from front to back).
    let main_bar = mesh(pg("OverturnedTricycles__mainbar", cyl_geo!(0.010, 0.010, 0.20, 6)), &frame_mat);
    main_bar.rotation().z = PI / 2.0;
    main_bar.position().set(0, 0.10, 0);
    tri.add(&main_bar);
    // Front fork (angled up); dropout end on the front axle (0.11, 0.055).
    let fork = mesh(pg("OverturnedTricycles__fork", cyl_geo!(0.008, 0.008, 0.14, 6)), &frame_mat);
    fork.rotation().z = 0.3;
    fork.position().set(0.11 - 0.07 * (0.3f64).sin(), 0.055 + 0.07 * (0.3f64).cos(), 0);
    tri.add(&fork);
    // Handlebars, clamped on the fork top.
    let handle_bar = mesh(pg("OverturnedTricycles__handlebar", cyl_geo!(0.008, 0.008, 0.12, 6)), &frame_mat);
    handle_bar.rotation().x = PI / 2.0;
    handle_bar.position().set(0.069, 0.186, 0);
    tri.add(&handle_bar);
    // Handle grips.
    for s in [-1.0, 1.0] {
        let grip = mesh(pg("OverturnedTricycles__grip", cyl_geo!(0.010, 0.010, 0.024, 6)), &handle_mat);
        grip.rotation().x = PI / 2.0;
        grip.position().set(0.069, 0.186, s * 0.070);
        tri.add(&grip);
    }
    // Seat, on the main bar.
    let seat = mesh(pg("OverturnedTricycles__seat", box_geo!(0.070, 0.014, 0.050)), &seat_mat);
    seat.position().set(-0.06, 0.117, 0);
    tri.add(&seat);
    // Front wheel (bigger).
    let front_wheel = mesh(pg("OverturnedTricycles__frontwheel", cyl_geo!(0.055, 0.055, 0.020, 12)), &wheel_mat);
    front_wheel.rotation().x = PI / 2.0;
    front_wheel.position().set(0.11, 0.055, 0);
    tri.add(&front_wheel);
    // 2 back wheels (smaller).
    for s in [-1.0, 1.0] {
        let wheel = mesh(pg("OverturnedTricycles__rearwheel", cyl_geo!(0.035, 0.035, 0.020, 10)), &wheel_mat);
        wheel.rotation().x = PI / 2.0;
        wheel.position().set(-0.09, 0.035, s * 0.060);
        tri.add(&wheel);
    }
    // Tip on side: rolled about the front-back axis (x); 'YXZ' rolls first, then the yaw.
    tri.set_rotation_order(EulerOrder::YXZ);
    tri.rotation().x = PI / 2.0;
    tri.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0.082, g.wz(z) + (rnd() - 0.5) * 1.8);
    tri.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tri);
}

// R503 old CRT TV: 4% chance per level. Boxy TV with static screen + antennae.
pub(crate) fn place_old_crt_tvs(g: &mut Game, _dist: &[i32]) {
    g.pp.crt_positions.clear(); // R504 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let case_mat = lambert!({ color: 0x8a7060 });
    let dark_mat = lambert!({ color: 0x181410 });
    let screen_mat = basic_mat!({ map: crt_static_texture() });
    let antenna_mat = lambert!({ color: 0x606870 });
    let tv = group();
    let (bw, bh, bd) = (0.28, 0.22, 0.24);
    // Case body.
    let body = mesh(pg("OldCRTTVs__body", box_geo!(bw, bh, bd)), &case_mat);
    body.position().y = bh / 2.0;
    tv.add(&body);
    // Screen (dark bezel).
    let bezel = mesh(pg("OldCRTTVs__bezel", box_geo!(bw * 0.85, bh * 0.75, 0.005)), &dark_mat);
    bezel.position().set(0, bh / 2.0, bd / 2.0 + 0.001);
    tv.add(&bezel);
    // Screen face (canvas static).
    let screen = mesh(plane_geo!(bw * 0.78, bh * 0.68), &screen_mat);
    screen.position().set(0, bh / 2.0, bd / 2.0 + 0.006);
    tv.add(&screen);
    // 3 knob dials on side.
    for i in 0..3 {
        let knob = mesh(pg("OldCRTTVs__knob", cyl_geo!(0.008, 0.008, 0.006, 6)), &dark_mat);
        knob.rotation().z = PI / 2.0;
        knob.position().set(bw / 2.0 + 0.003, bh / 2.0 - 0.02 - i as f64 * 0.04, bd / 2.0 - 0.04);
        tv.add(&knob);
    }
    // 2 antennae (rabbit ears).
    for s in [-1.0, 1.0] {
        let ant = mesh(pg("OldCRTTVs__antenna", cyl_geo!(0.002, 0.002, 0.18, 3)), &antenna_mat);
        ant.rotation().z = s * 0.5;
        ant.position().set(s * 0.06, bh + 0.09, -bd / 2.0 + 0.03);
        tv.add(&ant);
    }
    // Faint self-lit screen glow point light.
    let glow = point_light!(0x506080, 0.20, 0.6, 2);
    glow.position().set(0, bh / 2.0, bd / 2.0 + 0.10);
    tv.add(&glow);
    // Static flicker animator (slight intensity wobble).
    let flicker_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = t * 12.0 + flicker_phase;
        glow.set_intensity(0.20 + p.sin() * 0.06 + (p * 3.3).sin() * 0.04);
    });
    scatter_on_floor(g, &tv, x, z, 1.6);
    g.world.add(&tv);
    // R504 track for static hiss audio.
    let p = tv.pos();
    g.pp.crt_positions.push(v3(p.x, 0.20, p.z));
}

// R503 overflowing ashtray: 6% chance per level. Ashtray + scattered butts.
pub(crate) fn place_overflowing_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let tray_mat = lambert!({ color: 0x604838 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let filter_mat = lambert!({ color: 0xa08040 });
    let ash_mat = lambert!({ color: 0x2a2418 });
    let grp = group();
    // Tray body: shallow dish.
    let tray = mesh(pg("OverflowingAshtrays__tray", cyl_geo!(0.050, 0.040, 0.015, 12)), &tray_mat);
    tray.position().y = 0.0075;
    grp.add(&tray);
    // Ash pile in center.
    let ash_pile = mesh(pg("OverflowingAshtrays__ash", cone_geo!(0.030, 0.015, 8)), &ash_mat);
    ash_pile.position().y = 0.020;
    grp.add(&ash_pile);
    // 10-16 butts scattered inside + around tray.
    let n_butts = 10 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_butts {
        // Paper part (white).
        let paper = mesh(pg("OverflowingAshtrays__paper", cyl_geo!(0.002, 0.002, 0.014, 4)), &paper_mat);
        paper.rotation().z = PI / 2.0;
        // Filter (tan).
        let filter = mesh(pg("OverflowingAshtrays__filter", cyl_geo!(0.002, 0.002, 0.006, 4)), &filter_mat);
        filter.rotation().z = PI / 2.0;
        // Position: some inside tray, some around it.
        let in_tray = (i as f64) < n_butts as f64 * 0.6;
        let r = if in_tray { rnd() * 0.035 } else { 0.06 + rnd() * 0.15 };
        let a = rnd() * PI * 2.0;
        let px = a.cos() * r;
        let pz = a.sin() * r;
        let py = if in_tray { 0.018 } else { 0.002 };
        let ang = rnd() * PI * 2.0;
        paper.position().set(px, py, pz);
        paper.rotation().y = ang;
        // on the paper's axis: rotation.y = ang lays the tube along (cos, 0, -sin)
        filter.position().set(px + ang.cos() * 0.010, py, pz - ang.sin() * 0.010);
        filter.rotation().y = ang;
        grp.add(&paper);
        grp.add(&filter);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R502 peeling wallpaper (`assets/src/blender/props/r11b.notes.md`): the
// corner of a drop come away at its seam, lifted off the wall, turned over
// and hanging, the paper's pale back to the room; behind it the bare wall
// it was pasted to, stained with the old paste, a pencil plumb line by the
// seam, scraps of the paper's backing still stuck; its torn edges white with
// the paper's core. On a papered wall it is the wall's own paper; on any
// other, a scrap of old paper left on it, torn round, peeling the same way.
pub(crate) fn place_peeling_wallpapers(g: &mut Game, _dist: &[i32]) {
    use r11b_kit::*;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let wallpaper = group();
    let (pw, ph) = (0.30, 0.50);
    // which drop came away, the one to the left of the seam or to the right
    // (the old peeled corner's side)
    let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    mount_on_wall(g, &wallpaper, &c, 0.005, 1.4, 0.5);
    // its layout, from where it is (no draws)
    let p = wallpaper.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    let key = wall_key_at(g, c.x, c.z);
    let papered = key.starts_with("wp:");
    // the seam (x, along the wall) and the line the paper still holds along
    let seam = -side * (0.005 + 0.025 * r.f());
    let foot = -0.03 + 0.05 * r.f();
    let tex = peeling_wallpaper_tex(r.u(), pw, ph, side, seam, foot, papered);
    let decal = mesh(plane_geo!(pw, ph), &decal_mat(&tex));
    decal.position().z = -0.004;
    wallpaper.add(&decal);
    // the corner come away: its printed face (the wall's own paper, or the
    // scrap's), its back
    let face_mat = if papered { g.arch_mat(&key).duplicate() } else { lambert!({ color: OLD_PAPER_HEX }) };
    let back_mat = lambert!({ map: &paper_back_tex(), color: 0xffffff });
    let hand = if side > 0.0 { "R" } else { "L" };
    for (part, mat) in [("Front", &face_mat), ("Back", &back_mat)] {
        let name = format!("Real__peelingWallpaperCurl{hand}{part}");
        let geo = game::assets::props::prop_part_as_modelled(&name).unwrap_or_else(|| box_geo!(0.12, 0.1, 0.07));
        let m = mesh(geo, mat);
        m.position().set(seam, foot, -0.005);
        wallpaper.add(&m);
    }
    // a mark on the wall: the placement pass leaves it where the settle put it
    wallpaper.user().floats = true;
    g.world.add(&wallpaper);
}

// R502 spilled pill bottle: 8% chance per level. Tipped bottle + scattered capsules.
pub(crate) fn place_spilled_pill_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bottle_mat = basic_mat!({ color: 0xe07030, transparent: true, opacity: 0.85 });
    let cap_mat = lambert!({ color: 0xf0ece0 });
    let pill_colors = [0xf0ece0, 0xe0a0a0, 0xa0d0e0, 0xd0d0d0];
    let grp = group();
    // Bottle body (tipped on side).
    let body = mesh(cyl_geo!(0.014, 0.014, 0.045, 10), &bottle_mat);
    body.rotation().z = PI / 2.0 - 0.045; // neck end raised onto the wider cap
    body.position().set(-0.04, 0.015, 0);
    grp.add(&body);
    // White cap on side, pushed over the bottle's neck end.
    let cap = mesh(pg("SpilledPillBottles__cap", cyl_geo!(0.016, 0.016, 0.010, 10)), &cap_mat);
    cap.rotation().z = PI / 2.0;
    cap.position().set(-0.0645, 0.016, 0);
    grp.add(&cap);
    // 8-14 scattered pills (capsules).
    let n_pills = 8 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_pills {
        let pill_mat = lambert!({ color: pick_hex(&pill_colors) });
        let pill = mesh(pg("SpilledPillBottles__pill", cyl_geo!(0.003, 0.003, 0.008, 5)), &pill_mat);
        pill.rotation().z = PI / 2.0;
        pill.rotation().y = rnd() * PI * 2.0;
        pill.position().set((rnd() - 0.5) * 0.20, 0.003, (rnd() - 0.5) * 0.18);
        grp.add(&pill);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R501 religious triptych: 4% chance per level. 3-panel wall hanging w/ icons.
pub(crate) fn place_religious_triptychs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0xd8b040 });
    let panel_mat = lambert!({ map: triptych_texture(), side: DoubleSide });
    let triptych = group();
    let (w, h, t, fw) = (0.42, 0.34, 0.020, 0.024);
    // Frame.
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("ReligiousTriptychs__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        triptych.add(&bar);
    }
    // 2 dividers between panels.
    for px in [-w / 6.0, w / 6.0] {
        let div = mesh(pg("ReligiousTriptychs__divider", box_geo!(0.008, h, t)), &frame_mat);
        div.position().set(px, 0, 0);
        triptych.add(&div);
    }
    // Canvas, recessed 8 mm behind the frame's face.
    let canvas = mesh(plane_geo!(w, h), &panel_mat);
    canvas.position().set(0, 0, t / 2.0 - 0.008);
    triptych.add(&canvas);
    mount_on_wall(g, &triptych, &c, 0.014, 1.68, 0.6);
    g.world.add(&triptych);
}

// R501 shattered vase: 6% chance per level. Broken pottery + upright base.
pub(crate) fn place_shattered_vases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pot_colors = [0xe8dcc0, 0x8a5030, 0x4a2818, 0xa02040];
    let pot_mat = lambert!({ color: pick_hex(&pot_colors) });
    let grp = group();
    // Intact base stub (bottom of vase remaining).
    let base = mesh(pg("ShatteredVases__base", cyl_geo!(0.030, 0.035, 0.020, 12)), &pot_mat);
    base.position().y = 0.010;
    grp.add(&base);
    // Broken jagged rim on base (short irregular tabs, tangent to the rim).
    for i in 0..5 {
        let a = (i as f64 / 5.0) * PI * 2.0 + rnd() * 0.4;
        let tab = mesh(pg("ShatteredVases__shard", box_geo!(0.010, 0.014 + rnd() * 0.010, 0.004)), &pot_mat);
        tab.position().set(a.cos() * 0.028, 0.024, a.sin() * 0.028);
        tab.rotation().y = PI / 2.0 - a;
        grp.add(&tab);
    }
    // 6-8 scattered shard triangles on floor.
    let n_shards = 6 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_shards {
        let s = 0.020 + rnd() * 0.020;
        let shard_geo = buffer_geo(&[0.0, 0.0, 0.0, s, 0.0, s * 0.5 * (rnd() - 0.5), s * 0.5, 0.0, s * (0.5 + rnd() * 0.5)], vec![0, 1, 2]);
        let shard = mesh(shard_geo, &pot_mat);
        shard.position().set((rnd() - 0.5) * 0.30, 0.002, (rnd() - 0.5) * 0.30);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R500 worn teddy bear: 6% chance per level. Old teddy with a missing eye.
pub(crate) fn place_worn_teddy_bears(g: &mut Game, _dist: &[i32]) {
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
    let eye_mat = lambert!({ color: 0x181008 });
    let nose_mat = lambert!({ color: 0x2a1810 });
    let bear = group();
    // Body sphere.
    let body = mesh(pg("WornTeddyBears__body", sphere_geo!(0.045, 12, 10)), &fur_mat);
    body.scale().set(1, 1.2, 0.9);
    body.position().y = 0.045;
    bear.add(&body);
    // Head sphere.
    let head = mesh(pg("WornTeddyBears__head", sphere_geo!(0.035, 12, 10)), &fur_mat);
    head.position().y = 0.100;
    bear.add(&head);
    // Snout.
    let snout = mesh(pg("WornTeddyBears__muzzle", sphere_geo!(0.018, 8, 6)), &fur_mat);
    snout.scale().set(1, 0.8, 1);
    snout.position().set(0, 0.092, 0.028);
    bear.add(&snout);
    // Nose (dark).
    let nose = mesh(pg("WornTeddyBears__nose", sphere_geo!(0.005, 6, 5)), &nose_mat);
    nose.position().set(0, 0.094, 0.044);
    bear.add(&nose);
    // 2 ears (small semispheres), tilted out so the flat base sits on the head.
    for s in [-1.0, 1.0] {
        let ear = mesh(pg("WornTeddyBears__ear", sphere_geo!(0.012, 8, 6, 0, PI * 2.0, 0, PI / 2.0)), &fur_mat);
        ear.position().set(s * 0.032 * (0.6f64).sin(), 0.100 + 0.032 * (0.6f64).cos(), 0);
        ear.rotation().z = -s * 0.6;
        bear.add(&ear);
    }
    // Present eye (still there).
    let eye1 = mesh(pg("WornTeddyBears__eye", sphere_geo!(0.004, 5, 4)), &eye_mat);
    eye1.position().set(0.010, 0.108, 0.030);
    bear.add(&eye1);
    // Missing eye: dark socket hole.
    let socket_mat = basic_mat!({ color: 0x080604 });
    let socket = mesh(circle_geo!(0.005, 6), &socket_mat);
    socket.position().set(-0.010, 0.108, 0.032);
    socket.look_at(-0.010, 0.108, 0.10);
    bear.add(&socket);
    // 4 short limbs (arms + legs).
    for (lx, ly, lz) in [(-0.035, 0.045, 0.0), (0.035, 0.045, 0.0), (-0.020, 0.020, 0.010), (0.020, 0.020, 0.010)] {
        let limb = mesh(pg("WornTeddyBears__limb", sphere_geo!(0.014, 8, 6)), &fur_mat);
        limb.scale().set(1, 1.3, 1);
        limb.position().set(lx, ly, lz);
        bear.add(&limb);
    }
    // 30% chance tipped.
    if rnd() < 0.30 {
        bear.rotation().z = PI / 2.0;
        bear.position().y = 0.035;
    }
    bear.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bear.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bear.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bear);
}

// R500 dead moth: 5% chance per level. Small dead moth on floor.
pub(crate) fn place_dead_moths(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wing_mat = lambert!({ color: 0xa08a70, side: DoubleSide });
    let body_mat = lambert!({ color: 0x302010 });
    let moth = group();
    // Body: tiny elongated cylinder.
    let body = mesh(pg("DeadMoths__body", cyl_geo!(0.002, 0.001, 0.014, 4)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = 0.003;
    moth.add(&body);
    // 2 wings (triangle planes) laid flat.
    for s in [-1.0, 1.0] {
        let wing_geo = buffer_geo(&[0.0, 0.0, 0.0, 0.008, 0.0, s * 0.012, -0.008, 0.0, s * 0.012], vec![0, 1, 2]);
        let wing = mesh(wing_geo, &wing_mat);
        wing.position().y = 0.002;
        moth.add(&wing);
    }
    // 2 tiny antennae.
    for s in [-1.0, 1.0] {
        let ant = mesh(cyl_geo!(0.0002, 0.0002, 0.006, 3), &body_mat);
        ant.position().set(0.008, 0.003, s * 0.001);
        ant.rotation().z = 0.6;
        ant.rotation().y = s * 0.3;
        moth.add(&ant);
    }
    scatter_on_floor(g, &moth, x, z, 2.4);
    g.world.add(&moth);
}

// R499 doll body parts: 4% chance per level. 2-3 severed doll limbs scattered.
pub(crate) fn place_doll_body_parts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let porcelain_mat = lambert!({ color: 0xf0d8c0 });
    let joint_mat = lambert!({ color: 0x8a3020 });
    let grp = group();
    let n_parts = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_parts {
        let is_arm = rnd() < 0.5;
        let part = group();
        // Limb main cylinder (thicker for leg, thinner for arm).
        let limb_r = if is_arm { 0.010 } else { 0.014 };
        let limb_len = if is_arm { 0.065 } else { 0.080 };
        let limb = mesh(pg("DollBodyParts__limb", cyl_geo!(limb_r, limb_r, limb_len, 8)), &porcelain_mat);
        limb.rotation().z = PI / 2.0;
        part.add(&limb);
        // End joint (dark socket).
        let socket = mesh(pg("DollBodyParts__socket", cyl_geo!(limb_r + 0.002, limb_r + 0.002, 0.006, 8)), &joint_mat);
        socket.rotation().z = PI / 2.0;
        socket.position().x = -limb_len / 2.0 - 0.002;
        part.add(&socket);
        // Hand/foot ball at other end.
        let ext = mesh(pg("DollBodyParts__hand", sphere_geo!(limb_r * 1.4, 6, 5)), &porcelain_mat);
        ext.position().x = limb_len / 2.0 + limb_r * 0.5;
        part.add(&ext);
        part.position().set((rnd() - 0.5) * 0.30, limb_r, (rnd() - 0.5) * 0.20);
        part.rotation().y = rnd() * PI * 2.0;
        part.rotation().z = (rnd() - 0.5) * 0.4;
        // Height from the tilt: whichever end is lower rests on the floor.
        let rz = part.rot().z;
        let (tsn, tcs) = (rz.sin(), rz.cos());
        let py = (limb_r * 1.4 - ext.pos().x * tsn).max((limb_r + 0.002) * tcs + 0.003 * tsn.abs() - socket.pos().x * tsn);
        part.position().y = py;
        grp.add(&part);
    }
    scatter_on_floor(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

// R499 single child shoe: 6% chance per level. One small child's shoe on floor.
pub(crate) fn place_single_child_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let shoe_colors = [0xd82040, 0x2050d8, 0xd8b040, 0xf0ece0, 0x201a10];
    let shoe_mat = lambert!({ color: pick_hex(&shoe_colors) });
    let sole_mat = lambert!({ color: 0x201a10 });
    let shoe = group();
    // Sole.
    let sole = mesh(pg("SingleChildShoes__sole", box_geo!(0.10, 0.010, 0.05)), &sole_mat);
    sole.position().y = 0.005;
    shoe.add(&sole);
    // Upper toe (rounded).
    let toe = mesh(pg("SingleChildShoes__toe", sphere_geo!(0.028, 8, 6)), &shoe_mat);
    toe.scale().set(1, 0.6, 0.9);
    toe.position().set(0.02, 0.020, 0);
    shoe.add(&toe);
    // Heel.
    let heel = mesh(pg("SingleChildShoes__heel", box_geo!(0.040, 0.024, 0.048)), &shoe_mat);
    heel.position().set(-0.024, 0.022, 0);
    shoe.add(&heel);
    // 2 lace X marks on top.
    for i in 0..2 {
        for j in 0..2 {
            let lace_mat = lambert!({ color: 0xf0ece0 });
            let lace = mesh(pg("SingleChildShoes__lace", box_geo!(0.020, 0.001, 0.002)), &lace_mat);
            lace.position().set(0.01 + i as f64 * 0.010, 0.030, 0);
            lace.rotation().y = if j == 0 { 0.3 } else { -0.3 };
            shoe.add(&lace);
        }
    }
    scatter_on_floor(g, &shoe, x, z, 2.4);
    // 25% tipped on its side ('YXZ': roll about the shoe's own axis, then yaw).
    if rnd() < 0.25 {
        shoe.set_rotation_order(EulerOrder::YXZ);
        shoe.rotation().x = PI / 2.0;
        shoe.position().y = 0.025;
    }
    g.world.add(&shoe);
}

// R498 dead flowers in vase: 6% chance per level. Vase + dried stems + petals.
pub(crate) fn place_dead_flower_vases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let vase_colors = [0xe8dcc0, 0x8a5030, 0x4a2818, 0xa0a0a0];
    let vase_mat = lambert!({ color: pick_hex(&vase_colors) });
    let stem_mat = lambert!({ color: 0x604020 });
    let petal_mat = lambert!({ color: 0x8a5030, side: DoubleSide });
    let vase = group();
    // Vase: tapered cylinder + narrow neck.
    let body = mesh(pg("DeadFlowerVases__body", cyl_geo!(0.035, 0.028, 0.070, 12)), &vase_mat);
    body.position().y = 0.035;
    vase.add(&body);
    let neck = mesh(pg("DeadFlowerVases__neck", cyl_geo!(0.022, 0.028, 0.020, 10)), &vase_mat);
    neck.position().y = 0.080;
    vase.add(&neck);
    // 4-6 dried curled stems.
    let n_stems = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_stems {
        let a = (i as f64 / n_stems as f64) * PI * 2.0 + rnd() * 0.5;
        let stem_len = 0.08 + rnd() * 0.06;
        let tilt = rnd() * 0.4 + 0.2;
        let stem = mesh(pg("DeadFlowerVases__stem", cyl_geo!(0.0015, 0.0015, stem_len, 4)), &stem_mat);
        // Foot 1 cm down inside the neck (mouth at 0.090).
        stem.position().set(
            a.cos() * 0.010 + a.cos() * stem_len / 2.0 * tilt.sin(),
            0.080 + stem_len / 2.0 * tilt.cos(),
            a.sin() * 0.010 + a.sin() * stem_len / 2.0 * tilt.sin(),
        );
        stem.rotation().z = -a.cos() * tilt;
        stem.rotation().x = a.sin() * tilt;
        vase.add(&stem);
        // Drooping tip petal.
        let petal = mesh(circle_geo!(0.006, 5), &petal_mat);
        petal.position().set(
            a.cos() * 0.010 + a.cos() * stem_len * tilt.sin(),
            0.080 + stem_len * tilt.cos() - 0.005,
            a.sin() * 0.010 + a.sin() * stem_len * tilt.sin(),
        );
        petal.rotation().x = -PI / 3.0;
        vase.add(&petal);
    }
    // 3 fallen petals on floor.
    for i in 0..3 {
        let fallen = mesh(circle_geo!(0.006, 5), &petal_mat);
        fallen.rotation().x = -PI / 2.0;
        fallen.position().set((rnd() - 0.5) * 0.10, 0.002, (rnd() - 0.5) * 0.10);
        fallen.rotation().z = rnd() * PI * 2.0;
        vase.add(&fallen);
    }
    scatter_on_floor(g, &vase, x, z, 2.4);
    g.world.add(&vase);
}

// R498 baby crib fragment: 3% chance per level. Wooden crib section with bars.
pub(crate) fn place_baby_crib_fragments(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0xf0e0d0, 0xe8dcc0, 0x8a5030];
    let wood_mat = lambert!({ color: pick_hex(&wood_colors) });
    let crib = group();
    // Base rail (bottom).
    let base_rail = mesh(pg("BabyCribFragments__baserail", box_geo!(0.44, 0.016, 0.020)), &wood_mat);
    base_rail.position().set(0, 0.008, 0);
    crib.add(&base_rail);
    // Top rail.
    let top_rail = mesh(pg("BabyCribFragments__toprail", box_geo!(0.44, 0.016, 0.020)), &wood_mat);
    top_rail.position().set(0, 0.42, 0);
    crib.add(&top_rail);
    // 10 vertical bars.
    let n_bars = 10;
    for i in 0..n_bars {
        let bx = -0.20 + i as f64 * (0.40 / (n_bars - 1) as f64);
        let bar = mesh(pg("BabyCribFragments__spindle", cyl_geo!(0.005, 0.005, 0.40, 5)), &wood_mat);
        bar.position().set(bx, 0.22, 0);
        crib.add(&bar);
    }
    // 2 corner posts (taller).
    for s in [-1.0, 1.0] {
        let post = mesh(pg("BabyCribFragments__post", cyl_geo!(0.014, 0.014, 0.50, 5)), &wood_mat);
        post.position().set(s * 0.22, 0.25, 0);
        crib.add(&post);
        // Post cap knob.
        let knob = mesh(pg("BabyCribFragments__finial", sphere_geo!(0.018, 8, 6)), &wood_mat);
        knob.position().set(s * 0.22, 0.51, 0);
        crib.add(&knob);
    }
    scatter_on_floor(g, &crib, x, z, 1.6);
    g.world.add(&crib);
}

// R497 crystal chandelier: 3% chance per level. 6 arms + dangles + inner light.
pub(crate) fn place_crystal_chandeliers(g: &mut Game, _dist: &[i32]) {
    g.pp.chandelier_positions.clear(); // R524 reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let brass_mat = lambert!({ color: 0xc89040 });
    let crystal_mat = basic_mat!({ color: 0xf0f0ff, transparent: true, opacity: 0.55 });
    let chain_mat = lambert!({ color: 0x8a7050 });
    let chandelier = group();
    let chain_len = 0.35 + rnd() * 0.15;
    let center_y = WALL_H - chain_len - 0.05;
    // Hanging chain, from the ceiling down into the cap.
    let chain = mesh(pg("CrystalChandeliers__chain", cyl_geo!(0.003, 0.003, chain_len, 4)), &chain_mat);
    chain.position().y = WALL_H - chain_len / 2.0;
    chandelier.add(&chain);
    // Top brass cap.
    let top_cap = mesh(pg("CrystalChandeliers__cap", cyl_geo!(0.030, 0.020, 0.010, 8)), &brass_mat);
    top_cap.position().y = center_y + 0.055;
    chandelier.add(&top_cap);
    // Central body sphere.
    let body = mesh(pg("CrystalChandeliers__body", sphere_geo!(0.030, 12, 10)), &brass_mat);
    body.position().y = center_y + 0.020;
    chandelier.add(&body);
    // 6 arms extending outward.
    let n_arms = 6;
    for i in 0..n_arms {
        let a = (i as f64 / n_arms as f64) * PI * 2.0;
        let arm_x = a.cos() * 0.09;
        let arm_z = a.sin() * 0.09;
        // Arm cylinder; tip socket under the cup's foot.
        let arm = mesh(pg("CrystalChandeliers__arm", cyl_geo!(0.003, 0.003, 0.10, 4)), &brass_mat);
        arm.position().set(arm_x / 2.0, center_y + 0.013, arm_z / 2.0);
        arm.rotation().z = -(0.0f64).atan2(0.09);
        // Just orient toward outward direction.
        let arm_dir = v3(arm_x, 0.005, arm_z).normalized();
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&v3(0, 1, 0), &arm_dir);
        arm.quaternion().copy(&q);
        chandelier.add(&arm);
        // Small candle cup at end of arm.
        let cup = mesh(pg("CrystalChandeliers__cup", cyl_geo!(0.008, 0.006, 0.006, 6)), &brass_mat);
        cup.position().set(arm_x, center_y + 0.020, arm_z);
        chandelier.add(&cup);
        // Small candle.
        let candle = mesh(pg("CrystalChandeliers__candle", cyl_geo!(0.005, 0.005, 0.024, 5)), lambert!({ color: 0xf0e8d8 }));
        candle.position().set(arm_x, center_y + 0.034, arm_z);
        chandelier.add(&candle);
        // Small flame + halo (all lit).
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.9 });
        let flame = mesh(cone_geo!(0.003, 0.008, 5), &flame_mat);
        flame.position().set(arm_x, center_y + 0.050, arm_z);
        chandelier.add(&flame);
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.016, 6, 5), &halo_mat);
        halo.position().set(arm_x, center_y + 0.050, arm_z);
        chandelier.add(&halo);
    }
    // 8 crystal dangles below body.
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        let dr = 0.07;
        let dangle = mesh(cone_geo!(0.005, 0.030, 4), &crystal_mat);
        dangle.position().set(a.cos() * dr, center_y - 0.030, a.sin() * dr);
        dangle.rotation().x = PI;
        chandelier.add(&dangle);
    }
    // Single central light.
    let glow = point_light!(0xffb060, 0.9, 2.0, 2);
    glow.position().y = center_y + 0.020;
    chandelier.add(&glow);
    let flicker_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = t * 5.0 + flicker_phase;
        glow.set_intensity(0.9 + p.sin() * 0.06 + (p * 1.7).sin() * 0.04);
    });
    scatter_on_floor(g, &chandelier, x, z, 1.5);
    // R524 track for chime audio.
    let p = chandelier.pos();
    g.pp.chandelier_positions.push(v3(p.x, center_y, p.z));
    // Slow sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 12.0 + rnd() * 6.0;
        // swing about the chain's ceiling point (local 0, WALL_H, 0), not the floor
        // under it: shift the group by the pivot's displacement each frame
        let mut sway_off = [0.0f64; 3];
        let ch = chandelier.clone();
        g.animate(move |_g, _dt, t| {
            let rz = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.015;
            ch.rotation().z = rz;
            let s = rz.sin() * WALL_H;
            let ry = ch.rot().y;
            let o = [s * ry.cos(), (1.0 - rz.cos()) * WALL_H, -s * ry.sin()];
            {
                let mut pp = ch.position();
                pp.x += o[0] - sway_off[0];
                pp.y += o[1] - sway_off[1];
                pp.z += o[2] - sway_off[2];
            }
            sway_off = o;
        });
    }
    g.world.add(&chandelier);
}

// R496 cracked wall mirror: 4% chance per level. Spiderweb-cracked mirror + frame.
pub(crate) fn place_cracked_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x2a1810 });
    let mirror_mat = lambert!({ map: cracked_mirror_texture(), side: DoubleSide });
    let mirror = group();
    let (w, h, t, fw) = (0.30, 0.40, 0.020, 0.024);
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("CrackedMirrors__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        mirror.add(&bar);
    }
    let surface = mesh(plane_geo!(w, h), &mirror_mat);
    surface.position().set(0, 0, t / 2.0 + 0.001);
    mirror.add(&surface);
    mount_on_wall(g, &mirror, &c, 0.014, 1.65, 0.6);
    mirror.rotation().z = (rnd() - 0.5) * 0.06;
    g.world.add(&mirror);
}

// R496 rocking chair: 4% chance per level. Empty rocking chair, slow rock animator.
pub(crate) fn place_rocking_chairs(g: &mut Game, _dist: &[i32]) {
    g.pp.chair_positions.clear(); // R497 reset per level
    g.pp.chair_periods.clear();
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x603820, 0x8a5030, 0x201a10];
    let wood_mat = lambert!({ color: pick_hex(&wood_colors) });
    let chair = group();
    // Seat plank.
    let seat = mesh(pg("RockingChairs__seat", box_geo!(0.30, 0.020, 0.24)), &wood_mat);
    seat.position().y = 0.24;
    chair.add(&seat);
    // 4 legs.
    for (lx, lz) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let leg = mesh(pg("RockingChairs__leg", cyl_geo!(0.010, 0.010, 0.22, 5)), &wood_mat);
        leg.position().set(lx as f64 * 0.12, 0.12, lz as f64 * 0.10);
        if lz == 1 {
            // Front legs run on up through the seat as the arm posts (to y 0.344).
            leg.scale().y = 0.334 / 0.22;
            leg.position().y = 0.177;
        }
        chair.add(&leg);
    }
    // Back rest bars (5 vertical bars).
    let back_rail = mesh(pg("RockingChairs__rail", box_geo!(0.30, 0.020, 0.014)), &wood_mat);
    back_rail.position().set(0, 0.44, -0.11);
    chair.add(&back_rail);
    for i in 0..5 {
        let rx = -0.12 + i as f64 * 0.06;
        let bar = mesh(pg("RockingChairs__spindle", cyl_geo!(0.006, 0.006, 0.20, 4)), &wood_mat);
        bar.position().set(rx, 0.35, -0.11);
        chair.add(&bar);
    }
    // 2 rocker rails, front to back under each pair of legs, level (the modelled rocker is curved).
    for s in [-1.0, 1.0] {
        let rocker = mesh(pg("RockingChairs__rocker", box_geo!(0.36, 0.018, 0.030)), &wood_mat);
        rocker.position().set(s * 0.12, 0.009, 0);
        rocker.rotation().y = PI / 2.0;
        chair.add(&rocker);
    }
    // 2 armrests, front to back at each side.
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("RockingChairs__arm", cyl_geo!(0.008, 0.008, 0.20, 4)), &wood_mat);
        arm.rotation().x = PI / 2.0;
        arm.position().set(s * 0.12, 0.34, -0.005);
        chair.add(&arm);
    }
    // Slow rock animator (very subtle unsettling motion).
    let rock_phase = rnd() * PI * 2.0;
    let rock_period = 5.0 + rnd() * 3.0;
    {
        let chair = chair.clone();
        g.animate(move |_g, _dt, t| {
            chair.rotation().x = (t * (PI * 2.0 / rock_period) + rock_phase).sin() * 0.04;
        });
    }
    scatter_on_floor(g, &chair, x, z, 1.8);
    // Yaw first, so the animator's rotation.x rocks the chair along its own rockers.
    chair.set_rotation_order(EulerOrder::YXZ);
    g.world.add(&chair);
    // R497 track for creak audio (store period for sync: the ambient creak timer
    // reads it from the parallel `chair_periods`).
    let p = chair.pos();
    g.pp.chair_positions.push(v3(p.x, 0.30, p.z));
    g.pp.chair_periods.push(rock_period);
}

// R495 scattered pages: 8% chance per level. 5-10 loose sheets with writing.
pub(crate) fn place_scattered_pages(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let page_mat = lambert!({ map: scattered_page_texture(), side: DoubleSide });
    let grp = group();
    let n_pages = 5 + (rnd() * 6.0).floor() as i32;
    for i in 0..n_pages {
        let page = mesh(plane_geo!(0.14, 0.18), &page_mat);
        page.rotation().x = -PI / 2.0;
        page.position().set((rnd() - 0.5) * 0.7, 0.003 + i as f64 * 0.0004, (rnd() - 0.5) * 0.6);
        page.rotation().z = rnd() * PI * 2.0;
        // 20% chance curled (small tilt).
        if rnd() < 0.20 {
            page.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.15;
        }
        grp.add(&page);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R495 wall chalkboard: 4% chance per level. Green board, wood frame, chalk writing.
pub(crate) fn place_wall_chalkboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x603820 });
    let board_mat = lambert!({ map: chalkboard_texture(), side: DoubleSide });
    let board = group();
    let (w, h, t, fw) = (0.40, 0.30, 0.015, 0.020);
    // Frame.
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("WallChalkboards__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        board.add(&bar);
    }
    // Chalkboard surface.
    let surface = mesh(plane_geo!(w, h), &board_mat);
    surface.position().set(0, 0, t / 2.0 + 0.001);
    board.add(&surface);
    mount_on_wall(g, &board, &c, 0.010, 1.60, 0.6);
    g.world.add(&board);
}

// R494 severed porcelain doll head: 3% chance per level.
pub(crate) fn place_doll_heads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let porcelain_mat = lambert!({ color: 0xf0d8c0 });
    let hair_colors = [0x604020, 0x201810, 0xd8a040, 0x8a2020];
    let hair_mat = lambert!({ color: pick_hex(&hair_colors) });
    let eye_mat = basic_mat!({ color: 0x0080a0 });
    let mouth_mat = lambert!({ color: 0x802040 });
    let doll = group();
    // Head sphere.
    let head = mesh(pg("DollHeads__head", sphere_geo!(0.045, 12, 10)), &porcelain_mat);
    head.scale().set(1, 1.1, 0.95);
    head.position().y = 0.045;
    doll.add(&head);
    // 2 glass eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.006, 6, 5), &eye_mat);
        eye.position().set(s * 0.014, 0.052, 0.038);
        doll.add(&eye);
    }
    // Small mouth (dark curve).
    let mouth = mesh(plane_geo!(0.020, 0.006), &mouth_mat);
    mouth.position().set(0, 0.030, 0.043);
    doll.add(&mouth);
    // Hair patch (curved top).
    let hair = mesh(pg("DollHeads__hair", sphere_geo!(0.040, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &hair_mat);
    hair.position().y = 0.058;
    hair.scale().set(1.1, 0.9, 1.1);
    doll.add(&hair);
    // Neck stub (dark, torn), poking 5 mm out under the head.
    let neck_mat = lambert!({ color: 0x8a3020 });
    let neck = mesh(pg("DollHeads__neck", cyl_geo!(0.014, 0.010, 0.010, 8)), &neck_mat);
    neck.position().y = -0.003;
    doll.add(&neck);
    // Tipped on side often.
    if rnd() < 0.60 {
        doll.rotation().z = PI / 2.0;
        doll.position().y = 0.030;
    }
    doll.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    doll.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    doll.rotation().y = rnd() * PI * 2.0;
    g.world.add(&doll);
}

// R494 wall coat hooks with robe: 5% chance per level. 3 hooks + optional robe.
pub(crate) fn place_coat_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let board_mat = lambert!({ color: 0x603820 });
    let hook_mat = lambert!({ color: 0x606870 });
    let robe_colors = [0x2a2418, 0x502010, 0x201a10, 0x8a4030, 0x304050];
    let hook_group = group();
    // Wood board.
    let board = mesh(pg("CoatHooks__board", box_geo!(0.30, 0.05, 0.014)), &board_mat);
    board.position().y = 0.025;
    hook_group.add(&board);
    // 3 hook posts extending forward.
    for i in 0..3 {
        let hx = -0.10 + i as f64 * 0.10;
        // Base post.
        let post = mesh(pg("CoatHooks__post", cyl_geo!(0.004, 0.004, 0.030, 4)), &hook_mat);
        post.rotation().x = PI / 2.0;
        post.position().set(hx, 0.025, 0.020);
        hook_group.add(&post);
        // Hook tip (small angled), rising from the post's front end.
        let tip = mesh(pg("CoatHooks__tip", cyl_geo!(0.003, 0.003, 0.014, 4)), &hook_mat);
        tip.position().set(hx, 0.0315, 0.0357);
        tip.rotation().x = 0.4;
        hook_group.add(&tip);
    }
    // Optional robe on middle hook (70% chance).
    if rnd() < 0.70 {
        let robe_mat = lambert!({ color: pick_hex(&robe_colors), side: DoubleSide });
        // Robe body: trapezoid plane.
        let robe_geo = buffer_geo(&[-0.06, 0.0, 0.0, 0.06, 0.0, 0.0, 0.10, -0.35, 0.0, -0.10, -0.35, 0.0], vec![0, 1, 2, 0, 2, 3]);
        let robe = mesh(robe_geo, &robe_mat);
        robe.position().set(0, 0.020, 0.035);
        hook_group.add(&robe);
    }
    mount_on_wall(g, &hook_group, &c, 0.007, 2.10, 0.6);
    g.world.add(&hook_group);
}

// R493 hanging censer: 4% chance per level. Brass censer on 3 chains + gentle sway.
pub(crate) fn place_hanging_censers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let brass_mat = lambert!({ color: 0xc89040 });
    let chain_mat = lambert!({ color: 0x8a7050 });
    let censer = group();
    let chain_len = 0.5 + rnd() * 0.5;
    let bowl_y = WALL_H - chain_len - 0.05;
    // 3 chains from ceiling to bowl: 2 mm into the ceiling down to the rim bead.
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let anchor_top = v3(a.cos() * 0.020, WALL_H + 0.002, a.sin() * 0.020);
        let anchor_bot = v3(a.cos() * 0.035, bowl_y + 0.002, a.sin() * 0.035);
        let mut chain_dir = Vec3::ZERO;
        chain_dir.sub_vectors(&anchor_bot, &anchor_top);
        let chain_len_actual = chain_dir.length();
        let chain = mesh(pg("HangingCensers__chain", cyl_geo!(0.002, 0.002, chain_len_actual, 3)), &chain_mat);
        chain.position().set((anchor_top.x + anchor_bot.x) / 2.0, (anchor_top.y + anchor_bot.y) / 2.0, (anchor_top.z + anchor_bot.z) / 2.0);
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&v3(0, 1, 0), &chain_dir.normalized());
        chain.quaternion().copy(&q);
        censer.add(&chain);
    }
    // Bowl (half-sphere).
    let bowl = mesh(pg("HangingCensers__bowl", sphere_geo!(0.035, 12, 6, 0, PI * 2.0, PI / 2.0, PI / 2.0)), &brass_mat);
    bowl.position().y = bowl_y;
    censer.add(&bowl);
    // Rim.
    let rim = mesh(pg("HangingCensers__rim", torus_geo!(0.035, 0.004, 5, 12)), &brass_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = bowl_y;
    censer.add(&rim);
    // Lid (small dome).
    let lid = mesh(pg("HangingCensers__lid", sphere_geo!(0.030, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &brass_mat);
    lid.position().y = bowl_y;
    censer.add(&lid);
    // Handle knob on top.
    let knob = mesh(pg("HangingCensers__knob", sphere_geo!(0.008, 6, 5)), &brass_mat);
    knob.position().y = bowl_y + 0.033;
    censer.add(&knob);
    // Faint ember glow underneath.
    let glow = point_light!(0xff8030, 0.15, 0.4, 2);
    glow.position().y = bowl_y;
    censer.add(&glow);
    // Slow sway.
    let sway_phase = rnd() * PI * 2.0;
    let sway_period = 10.0 + rnd() * 6.0;
    {
        let censer = censer.clone();
        g.animate(move |_g, _dt, t| {
            censer.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.03;
        });
    }
    scatter_on_floor(g, &censer, x, z, 1.8);
    g.world.add(&censer);
}

// R493 broken wall clock: 5% chance per level. Clock face frozen at an odd time.
pub(crate) fn place_broken_wall_clocks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let case_mat = lambert!({ color: 0x2a1810 });
    let face_mat = lambert!({ map: broken_clock_texture(), side: DoubleSide });
    let clock = group();
    let (r, t) = (0.14, 0.020);
    // Case cylinder.
    let case_r = mesh(pg("BrokenWallClocks__case", cyl_geo!(r, r, t, 16)), &case_mat);
    case_r.rotation().x = PI / 2.0;
    clock.add(&case_r);
    // Rim (small torus).
    let rim = mesh(pg("BrokenWallClocks__bezel", torus_geo!(r, 0.008, 6, 16)), &case_mat);
    rim.rotation().x = 0.0;
    rim.position().z = t / 2.0 + 0.001;
    clock.add(&rim);
    // Face.
    let face = mesh(circle_geo!(r - 0.008, 16), &face_mat);
    face.position().z = t / 2.0 + 0.003;
    clock.add(&face);
    mount_on_wall(g, &clock, &c, 0.014, 1.90, 0.6);
    clock.rotation().z = (rnd() - 0.5) * 0.06;
    g.world.add(&clock);
}

// R492 ancient book stack: 8% chance per level. 3-4 leather-bound tomes stacked.
pub(crate) fn place_ancient_book_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x502010, 0x3a1808, 0x6a3820, 0x201a10, 0x8a2020];
    let page_mat = lambert!({ color: 0xe8dcc0 });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let stack = group();
    let n_books = 3 + (rnd() * 2.0).floor() as i32;
    let mut y = 0.0;
    for i in 0..n_books {
        let leather_mat = lambert!({ color: pick_hex(&leather_colors) });
        let book = group();
        let bw = 0.09 + rnd() * 0.04;
        let bh = 0.020 + rnd() * 0.008;
        let bd = 0.06 + rnd() * 0.02;
        // Cover.
        let cover = mesh(pg("AncientBookStacks__cover", box_geo!(bw, bh, bd)), &leather_mat);
        cover.position().y = bh / 2.0;
        book.add(&cover);
        // Page block (slightly smaller inset).
        let pages = mesh(pg("AncientBookStacks__pages", box_geo!(bw - 0.003, bh - 0.006, bd - 0.003)), &page_mat);
        pages.position().y = bh / 2.0;
        book.add(&pages);
        // Gold spine bands, across the spine at 0.2 and 0.8 of its length.
        for sy in [bh * 0.3, bh * 0.7] {
            let band = mesh(pg("AncientBookStacks__band", box_geo!(0.001, 0.003, bd)), &gold_mat);
            band.rotation().x = PI / 2.0;
            band.scale().z = 0.6 * bh / bd;
            band.position().set(-bw / 2.0 - 0.0004, bh / 2.0, (sy / bh - 0.5) * bd * 1.5);
            book.add(&band);
        }
        book.position().y = y;
        book.position().x = (rnd() - 0.5) * 0.010;
        book.position().z = (rnd() - 0.5) * 0.010;
        book.rotation().y = (rnd() - 0.5) * 0.15;
        stack.add(&book);
        y += bh; // each book rests on the one below
    }
    scatter_on_floor(g, &stack, x, z, 2.4);
    g.world.add(&stack);
}

/// Batch r11b's painting kit (the marks on the walls): hashed noise, a seeded
/// generator for a mark's layout (no level draws), and a per-pixel painter
/// filled on every core (`par_rows`).
mod r11b_kit {
    use game::prelude::*;

    /// A small generator for a mark's own layout, seeded from where it is
    pub struct Rng(pub u32);
    impl Rng {
        pub fn u(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
        /// in [0, 1)
        pub fn f(&mut self) -> f64 {
            f64::from(self.u()) / 4_294_967_296.0
        }
    }

    /// A seed from values already fixed (a position, a side): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    /// A hash of (seed, i, j) in [0, 1)
    pub fn h01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    /// Smooth value noise in [0, 1]
    pub fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (h01(seed, i, j), h01(seed, i + 1, j));
        let (c, d) = (h01(seed, i, j + 1), h01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    /// Fractal value noise of `oct` octaves, in [0, 1]
    pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }

    /// 0 below `e0`, 1 above `e1`, smooth between
    pub fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    /// `c` at alpha `a` over straight colour and alpha `dst`
    pub fn lay(dst: &mut [f64; 4], c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let out = a + dst[3] * (1.0 - a);
        if out > 1e-6 {
            for k in 0..3 {
                dst[k] = (c[k] * a + dst[k] * dst[3] * (1.0 - a)) / out;
            }
        }
        dst[3] = out;
    }

    /// Specks: a dot in each `cell` (m) with likelihood `like`, its radius
    /// `r0` + up to `r1`, anti-aliased over `aa`: (coverage, the dot's hash)
    #[allow(clippy::too_many_arguments)]
    pub fn specks(seed: u32, x: f64, y: f64, cell: f64, like: f64, r0: f64, r1: f64, aa: f64) -> (f64, f64) {
        let (gi, gj) = ((x / cell).floor() as i32, (y / cell).floor() as i32);
        let (mut best, mut hk) = (0.0f64, 0.0);
        for k in 0..9 {
            let (ci, cj) = (gi + k % 3 - 1, gj + k / 3 - 1);
            let h = h01(seed, ci, cj);
            if h >= like {
                continue;
            }
            let (px, py) = ((f64::from(ci) + h01(seed ^ 0x77, ci, cj)) * cell, (f64::from(cj) + h01(seed ^ 0x78, ci, cj)) * cell);
            let r = r0 + r1 * h01(seed ^ 0x79, ci, cj);
            let cov = (0.5 - ((x - px).hypot(y - py) - r) / aa).clamp(0.0, 1.0);
            if cov > best {
                best = cov;
                hk = h01(seed ^ 0x7a, ci, cj);
            }
        }
        (best, hk)
    }

    /// A transparent mark `tw` x `th` px across a plane `pw` x `ph` m: each
    /// pixel `px(x, y, aa)` (m from the plane's top left, y down; `aa` a
    /// pixel's size) in straight sRGB and alpha. Where it is clear its colour
    /// is `bleed`, so a mark's edge does not filter toward black.
    pub fn decal_tex(tw: usize, th: usize, pw: f64, ph: f64, bleed: [f64; 3], px: impl Fn(f64, f64, f64) -> [f64; 4] + Sync) -> Tex {
        let tex = canvas_tex_opt(tw as f64, th as f64, false, move |ctx, cw, ch| {
            let (sx, sy) = (pw / cw, ph / ch);
            let aa = sx.max(sy) * 1.1;
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, tw * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..tw {
                    let x = (i as f64 + 0.5) * sx;
                    let p = px(x, y, aa);
                    for (c, v) in p.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        });
        // what is clear (it came back black through the canvas) takes the
        // bleed colour, and what is nearly clear goes toward it
        {
            let img = tex.image();
            let mut im = img.borrow_mut();
            for p in im.data.chunks_exact_mut(4) {
                let t = smooth(0.0, 0.06, f64::from(p[3]) / 255.0);
                for (c, b) in p.iter_mut().zip(bleed) {
                    *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * t).round().clamp(0.0, 255.0) as u8;
                }
            }
            im.version += 1;
        }
        tex
    }

    /// The decal material: lit, see-through, drawn over the wall
    pub fn decal_mat(tex: &Tex) -> Mat {
        lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
    }

    /// The surface of the wall seen from open tile (`x`, `z`): its arch
    /// material's key (`wp:<variant>` a papered wall), as the walls are drawn
    pub fn wall_key_at(g: &Game, x: i32, z: i32) -> String {
        if g.layout.is_none() || !g.in_grid(x, z) {
            return "wp:0".into();
        }
        g.wall_mat_key(g.t_idx(x, z))
    }

    /// An old wallpaper's ground: cream gone yellow (the scrap's, and its curl's face)
    pub const OLD_PAPER: [f64; 3] = [0.77, 0.71, 0.56];

    pub const OLD_PAPER_HEX: u32 = 0xc4b58f;

    /// The wall where a drop's corner came away, across a plane `pw` x `ph`
    /// (x right from its middle, y up from its middle, as the group is): the
    /// seam at `seam`, the drop on `side` of it, pasted still below `foot`;
    /// on a `papered` wall the paper round it is the wall's own (clear here),
    /// else a scrap of old paper is painted round it.
    pub fn peeling_wallpaper_tex(seed: u32, pw: f64, ph: f64, side: f64, seam: f64, foot: f64, papered: bool) -> Tex {
        let mut r = Rng(seed | 1);
        let s = r.u();
        // (sRGB) plaster, the old paste on it, the paper's backing left stuck,
        // the paper's white core at a tear, pencil
        const PLASTER: [f64; 3] = [0.64, 0.61, 0.55];
        const PASTE: [f64; 3] = [0.62, 0.5, 0.31];
        const BACKING: [f64; 3] = [0.84, 0.81, 0.74];
        const CORE: [f64; 3] = [0.9, 0.88, 0.82];
        const SHADE: [f64; 3] = [0.05, 0.045, 0.04];
        const STRIPE: [f64; 3] = [0.6, 0.5, 0.38];
        let (cw, cl) = (0.12, 0.19);
        // the scrap's torn outline: its half width and half height, ragged
        let (sw, sh) = (0.1 + 0.02 * r.f(), 0.17 + 0.03 * r.f());
        let stripe_at = r.f() * 0.04;
        let tex = decal_tex(288, 480, pw, ph, if papered { PLASTER } else { OLD_PAPER }, move |cx, cy, aa| {
            // the group's frame: x right, y up, from the middle
            let (x, y) = (cx - pw / 2.0, ph / 2.0 - cy);
            let mut px = [PLASTER[0], PLASTER[1], PLASTER[2], 0.0];
            let u = (x - seam) * side; // across the drop from the seam
                                       // the bare patch: from the seam to the torn far side, from the foot
                                       // to the torn top; how far inside it (m, < 0 outside)
            let far = cw + 0.006 * (fbm(s ^ 0x21, y * 60.0, 0.5, 3) - 0.5) * 2.0 + 0.002 * (vnoise(s ^ 0x22, y * 700.0, 0.5) - 0.5);
            let top = foot + cl + 0.01 * (fbm(s ^ 0x23, x * 45.0, 0.5, 3) - 0.5) * 2.0 + 0.002 * (vnoise(s ^ 0x24, x * 700.0, 0.5) - 0.5);
            let inside = u.min(far - u).min(y - foot).min(top - y);
            // the scrap round it (another wall): its torn outline
            let scrap = if papered {
                1.0
            } else {
                // a rounded, torn outline: bitten into deeply here and there
                let (u, v) = (x / sw, y / sh);
                let q = (u.abs().powf(2.6) + v.abs().powf(2.6)).powf(1.0 / 2.6);
                let bite = 0.28 * (fbm(s ^ 0x31, x * 9.0, y * 9.0, 3) - 0.5) * 2.0 + 0.05 * (fbm(s ^ 0x32, x * 60.0, y * 60.0, 2) - 0.5);
                (1.0 - q + bite) * sw.min(sh)
            };
            if !papered && scrap > -0.003 && inside < 0.0 {
                // the old paper: faded stripes in pairs and a small sprig between,
                // yellowed, foxed, grimier low
                let xs = (x - stripe_at).rem_euclid(0.04);
                let st = (-((xs - 0.004) / 0.0007).powi(2)).exp() + (-((xs - 0.009) / 0.0007).powi(2)).exp();
                let (gi, gj) = (((x - stripe_at) / 0.04 - 0.5).floor(), (y / 0.05).floor());
                let (sx0, sy0) = ((gi + 0.5 + 0.5) * 0.04 + stripe_at, (gj + 0.5) * 0.05 + if (gi as i32) % 2 == 0 { 0.0 } else { 0.025 });
                let sprig = (0.5 - (((x - sx0) * 1.4).hypot(y - sy0 - 0.0) - 0.0035) / aa).clamp(0.0, 1.0);
                let fox = smooth(0.62, 0.8, fbm(s ^ 0x41, x * 40.0, y * 40.0, 3));
                let mut c = mix(OLD_PAPER, STRIPE, st * 0.55 + sprig * 0.4);
                c = mix(c, [0.55, 0.42, 0.27], fox * 0.35 + 0.1 * smooth(0.0, 0.2, -y));
                px = [c[0], c[1], c[2], (0.5 + scrap / aa).clamp(0.0, 1.0)];
                // its torn edge: the white core of the paper
                let fr = 0.0012 + 0.0012 * vnoise(s ^ 0x42, x * 300.0, y * 300.0);
                let a = px[3];
                lay(&mut px, CORE, smooth(fr, 0.0, scrap) * 0.85 * a);
            }
            // the paste left on the scrap's wall round it
            if !papered && scrap > -0.012 && scrap < 0.0 {
                lay(&mut px, PASTE, 0.25 * smooth(-0.012, 0.0, scrap) * fbm(s ^ 0x43, x * 50.0, y * 12.0, 2));
            }
            if inside > -0.004 {
                let fill = (0.5 + inside / aa).clamp(0.0, 1.0);
                // the wall it was pasted to: plaster on a papered wall, the
                // wall's own face on another
                if papered {
                    let m = 0.92 + 0.12 * fbm(s ^ 0x51, x * 90.0, y * 90.0, 3);
                    lay(&mut px, [PLASTER[0] * m, PLASTER[1] * m, PLASTER[2] * m], fill);
                }
                // the paste, in the brush's strokes, and stains where it soaked
                let stroke = fbm(s ^ 0x52, x * 70.0 + y * 8.0, y * 9.0, 3);
                lay(&mut px, PASTE, fill * (0.15 + 0.4 * smooth(0.45, 0.75, stroke)));
                // scraps of the paper's backing torn off and left stuck, mostly at the edges
                let edge = smooth(0.03, 0.0, inside);
                let back = fbm(s ^ 0x53, x * 24.0, y * 24.0, 4) + 0.2 * edge;
                let bk = smooth(0.6, 0.62, back);
                lay(&mut px, BACKING, fill * bk * 0.92);
                // and the white of its torn edge
                lay(&mut px, CORE, fill * 0.6 * (bk * (1.0 - bk) * 4.0).min(1.0));
                // the pencil line the paperhanger hung the drop to, by the seam
                let pl = (u - 0.0025).abs();
                lay(
                    &mut px,
                    [0.42, 0.42, 0.44],
                    fill * 0.35 * (0.5 - (pl - 0.0003) / aa).clamp(0.0, 1.0) * smooth(0.3, 0.6, vnoise(s ^ 0x54, 0.5, y * 40.0)),
                );
                // the shadow the lifted corner throws in, under its turn
                let sh = smooth(0.08, 0.0, y - foot) * smooth(0.0, 0.015, u) * smooth(0.0, 0.015, far - u);
                lay(&mut px, SHADE, fill * 0.3 * sh);
            }
            // the paper's torn edge round the patch: the white core, a hair proud
            if inside < 0.0 && inside > -0.004 && u > -0.001 {
                let fr = 0.0011 + 0.0016 * vnoise(s ^ 0x55, x * 260.0, y * 260.0);
                let k = smooth(fr, 0.0, -inside);
                // not along the seam (the next drop's cut edge) nor the foot (the turn)
                let torn = smooth(0.002, 0.006, u) * smooth(0.0, 0.004, y - foot);
                lay(&mut px, CORE, k * torn * 0.85);
            }
            // the seam: the next drop's cut edge, lifted a hair: a lit lip and its shadow
            let ds = (x - seam) * side;
            let near = smooth(0.0, 0.07, ph / 2.0 - y.abs()) * if papered { 1.0 } else { smooth(-0.003, 0.003, scrap) };
            lay(&mut px, SHADE, 0.35 * near * (0.5 - (ds - 0.0008).abs() / 0.0008).clamp(0.0, 1.0));
            lay(&mut px, CORE, 0.25 * near * (0.5 - (ds + 0.0006).abs() / 0.0006).clamp(0.0, 1.0));
            // under the lifted corner's hanging end, the wall in its shade
            let hang = smooth(0.0, 0.02, u) * smooth(0.0, 0.02, cw - u) * (-((y - (foot - 0.05)) / 0.05).powi(2)).exp();
            lay(&mut px, SHADE, 0.22 * hang);
            // nothing at the plane's edges
            px[3] *= smooth(0.0, 0.01, (pw / 2.0 - x.abs()).min(ph / 2.0 - y.abs()));
            px
        });
        tex
    }

    /// The back of a wallpaper: off-white paper, the paste dried on it in
    /// strokes, grey with the wall's dust (one, shared).
    pub fn paper_back_tex() -> Tex {
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| {
            c.get_or_init(|| {
                decal_tex(128, 128, 0.12, 0.12, [0.8, 0.77, 0.68], |x, y, _| {
                    let stroke = fbm(0x9a9e, x * 60.0 + y * 6.0, y * 10.0, 3);
                    let mut c = mix([0.75, 0.71, 0.61], [0.64, 0.54, 0.37], 0.6 * smooth(0.4, 0.8, stroke));
                    c = mix(c, [0.6, 0.6, 0.58], 0.25 * smooth(0.55, 0.8, fbm(0x9a9f, x * 140.0, y * 140.0, 2)));
                    [c[0], c[1], c[2], 1.0]
                })
            })
            .clone()
        })
    }
}
