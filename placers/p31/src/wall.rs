//! Wall-mounted props of chunk p31.
use game::prelude::*;
use crate::textures::{exit_sign_texture_2, poster_textures, thermostat_texture};
use crate::{pick, poster_tint_by_weather, shuffle, wall_candidates};

// ------------------------------------------------------------------ line 55128
// R264 fire alarm pull-station: 25% chance per level, 1 unit.
pub fn place_fire_alarms(g: &mut Game, _dist: &[i32]) {
    g.pp.fire_alarm_positions.clear(); // R276 reset per level
    if rnd() < 0.75 { return; } // 25% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let red_mat = lambert!({ color: 0xa02010 });
    let white_mat = lambert!({ color: 0xf0e8e0 });
    let dark_mat = lambert!({ color: 0x2a1810 });
    let alarm = group();
    // Main box: 0.12 wide × 0.16 tall × 0.04 deep.
    let body = mesh(pg("FireAlarms__body", box_geo!(0.12, 0.16, 0.04)), &red_mat);
    alarm.add(&body);
    // White label rectangle upper.
    let label = mesh(plane_geo!(0.10, 0.05), &white_mat);
    label.position().set(0, 0.045, 0.021);
    alarm.add(&label);
    // Pull handle (dark horizontal bar).
    let handle = mesh(pg("FireAlarms__handle", box_geo!(0.09, 0.015, 0.015)), &dark_mat);
    handle.position().set(0, -0.02, 0.028);
    alarm.add(&handle);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.023);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.023);
    let alarm_height = 1.3 + rnd() * 0.3; // 1.3-1.6 m (accessibility standard)
    if c.dx.abs() == 1 {
        alarm.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        alarm.position().set(wall_x, alarm_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        alarm.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        alarm.position().set(wall_x + (rnd() - 0.5) * 1.2, alarm_height, wall_z);
    }
    g.world.add(&alarm);
    // R276 track for beep audio events.
    let ap = alarm.pos();
    g.pp.fire_alarm_positions.push(v3(ap.x, alarm_height, ap.z));
    // R295 15% chance: broken glass shards scattered on floor below.
    if rnd() < 0.15 {
        let glass_mat = lambert!({ color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8 });
        let shard_count = 3 + (rnd() * 4.0).floor() as i32;
        for s in 0..shard_count {
            let shard_w = 0.015 + rnd() * 0.020;
            let shard_h = 0.020 + rnd() * 0.020;
            let shard = mesh(plane_geo!(shard_w, shard_h), &glass_mat);
            shard.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.4;
            shard.rotation().z = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.05 + rnd() * 0.25;
            shard.position().set(ap.x + ang.cos() * rad, 0.005, ap.z + ang.sin() * rad);
            g.world.add(&shard);
        }
    }
}

// ------------------------------------------------------------------ line 55251
// R263 coat hook rows: 20% chance per level, 1 row of 3-5 hooks.
pub fn place_coat_hook_rows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let bar_mat = lambert!({ color: 0x3a2818 });
    let hook_mat = lambert!({ color: 0x5a4830 });
    let row = group();
    // Wall-mount rail: horizontal 0.7 m box.
    let rail = mesh(pg("CoatHookRows__rail", box_geo!(0.70, 0.03, 0.04)), &bar_mat);
    row.add(&rail);
    // 3-5 hooks along rail.
    let hook_count = 3 + (rnd() * 3.0).floor() as i32;
    // R264: 15% chance one hook has a coat still hanging.
    let coat_idx = if rnd() < 0.15 { (rnd() * hook_count as f64).floor() as i32 } else { -1 };
    let coat_colors = [0x2a1810, 0x1a1a2a, 0x2a1a1a, 0x1a2a1a];
    for h in 0..hook_count {
        let x_off = -0.30 + (h as f64 / (hook_count - 1) as f64) * 0.60;
        let hook = mesh(pg("CoatHookRows__hook", cyl_geo!(0.006, 0.006, 0.05, 6)), &hook_mat);
        // peg out of the rail's front face, knob end tilted up 30°
        hook.rotation().x = PI / 3.0;
        hook.position().set(x_off, 0.0125, 0.0367);
        row.add(&hook);
        if h == coat_idx {
            // Draped coat: crumpled dark cloth as tapered box hanging below.
            let coat_mat = lambert!({ color: pick(&coat_colors) });
            let coat = group();
            // Torso: wider top narrowing.
            let torso = mesh(pg("CoatHookRows__torso", cyl_geo!(0.09, 0.13, 0.45, 6)), &coat_mat);
            torso.position().y = -0.24;
            coat.add(&torso);
            // Sleeves: 2 short cylinders angled outward.
            for side in [-1.0, 1.0] {
                let sleeve = mesh(pg("CoatHookRows__sleeve", cyl_geo!(0.03, 0.04, 0.25, 6)), &coat_mat);
                sleeve.rotation().z = side * PI / 8.0;
                sleeve.position().set(side * 0.10, -0.14, 0.02);
                coat.add(&sleeve);
            }
            coat.position().set(x_off, 0.04, 0.11); // collar over the peg, back clear of the wall
            row.add(&coat);
        }
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.025);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.025);
    let row_height = 1.7 + rnd() * 0.2; // 1.7-1.9 m
    if c.dx.abs() == 1 {
        row.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        row.position().set(wall_x, row_height, wall_z + (rnd() - 0.5) * 0.8);
    } else {
        row.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        row.position().set(wall_x + (rnd() - 0.5) * 0.8, row_height, wall_z);
    }
    g.world.add(&row);
}

// ------------------------------------------------------------------ line 55325
// R262 missing photo hook: 25% chance per level, 1-2 hooks with a faded
// rectangle where a photo used to hang.
pub fn place_missing_photo_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 { return; } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let faded_mat = basic_mat!({ color: 0xe0d8c0, transparent: true, opacity: 0.45, depthWrite: false, side: DoubleSide });
    let hook_mat = lambert!({ color: 0x2a2018 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Faded rectangle showing photo absence.
        let rect_w = 0.22 + rnd() * 0.10;
        let rect_h = 0.28 + rnd() * 0.10;
        let faded = mesh(plane_geo!(rect_w, rect_h), &faded_mat);
        // Tiny hook (small dark box near top edge of faded rectangle).
        let hook = mesh(pg("MissingPhotoHooks__hook", box_geo!(0.015, 0.02, 0.012)), &hook_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let center_h = 1.5 + rnd() * 0.5; // 1.5 - 2.0 m
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            faded.rotation().y = rot;
            hook.rotation().y = rot;
            let jit_z = (rnd() - 0.5) * 1.2;
            faded.position().set(wall_x, center_h, wall_z + jit_z);
            hook.position().set(wall_x + c.dx as f64 * 0.008, center_h + rect_h * 0.42, wall_z + jit_z);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            faded.rotation().y = rot;
            hook.rotation().y = rot;
            let jit_x = (rnd() - 0.5) * 1.2;
            faded.position().set(wall_x + jit_x, center_h, wall_z);
            hook.position().set(wall_x + jit_x, center_h + rect_h * 0.42, wall_z + c.dz as f64 * 0.008);
        }
        g.world.add(&faded);
        g.world.add(&hook);
        i += 1;
    }
}

// ------------------------------------------------------------------ line 55454
// R260 water fountains: 15% chance per level. Wall-mounted steel unit.
pub fn place_water_fountains(g: &mut Game, _dist: &[i32]) {
    g.pp.fountain_positions.clear(); // R272 reset per level
    if rnd() < 0.85 { return; } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let steel_mat = lambert!({ color: 0x9a9890 });
    let basin_mat = lambert!({ color: 0x606058 });
    let button_mat = lambert!({ color: 0x2a2822 });
    let spout_mat = lambert!({ color: 0xa0a094 });
    let fountain = group();
    // Main body: 0.28 wide × 0.38 tall × 0.20 deep box protruding from wall.
    let body = mesh(pg("WaterFountains__body", box_geo!(0.28, 0.38, 0.20)), &steel_mat);
    body.position().y = 0.75; // top edge at ~0.94 m
    fountain.add(&body);
    // Basin cavity — small darker box slightly recessed.
    let basin = mesh(pg("WaterFountains__basin", box_geo!(0.22, 0.03, 0.14)), &basin_mat);
    basin.position().set(0, 0.95, 0.03);
    fountain.add(&basin);
    // Spout: small angled cylinder near back of basin.
    let spout = mesh(pg("WaterFountains__spout", cyl_geo!(0.008, 0.012, 0.06, 6)), &spout_mat);
    spout.rotation().x = -PI / 6.0; // pointing up and slightly forward
    spout.position().set(0, 0.98, -0.02);
    fountain.add(&spout);
    // Button on front lower.
    let button = mesh(pg("WaterFountains__button", cyl_geo!(0.02, 0.02, 0.008, 8)), &button_mat);
    button.rotation().x = PI / 2.0;
    button.position().set(0.07, 0.66, 0.104);
    fountain.add(&button);
    // Grille below basin (thin dark rectangle).
    let grille = mesh(plane_geo!(0.20, 0.03), &basin_mat);
    grille.position().set(0, 0.93, 0.101);
    fountain.add(&grille);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.10);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.10);
    if c.dx.abs() == 1 {
        fountain.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        fountain.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        fountain.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        fountain.position().set(wall_x + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&fountain);
    // R272 track for drip-audio events.
    let fp = fountain.pos();
    g.pp.fountain_positions.push(v3(fp.x, 0.94, fp.z));
    // R261 fountain leak stain: 40% chance a wet stain on the floor below.
    if rnd() < 0.40 {
        let r = 0.35 + rnd() * 0.20;
        let stain_geo = circle_geo!(r, 12);
        // Slight stretch for organic irregular pool.
        let sx = 1.0 + rnd() * 0.3;
        let sy = 1.0 - rnd() * 0.2;
        stain_geo.scale(sx, sy, 1);
        let stain_mat = basic_mat!({ color: 0x0e1a20, transparent: true, opacity: 0.55, depthWrite: false });
        let stain = mesh(stain_geo, &stain_mat);
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        // Position: slightly in front of fountain along the wall-normal.
        let fry = fountain.rot().y;
        let fx = -fry.sin(); // fountain-front direction
        let fz = -fry.cos();
        stain.position().set(fp.x + fx * 0.25, 0.004, fp.z + fz * 0.25);
        g.world.add(&stain);
    }
}

// ------------------------------------------------------------------ line 55540
// R260 wall junction boxes: 20% chance per level, 1-2 units with conduits.
pub fn place_junction_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.junction_box_positions.clear(); // R281 reset per level
    if rnd() < 0.80 { return; } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let box_mat = lambert!({ color: 0x6a6a68 });
    let conduit_mat = lambert!({ color: 0x4a4a48 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let jbox = group();
        // Main box: 0.16 wide × 0.20 tall × 0.06 deep.
        let bx = mesh(pg("JunctionBoxes__box", box_geo!(0.16, 0.20, 0.06)), &box_mat);
        jbox.add(&bx);
        // Small hex screws on 4 corners.
        for (sx, sy) in [(-0.06, 0.08), (0.06, 0.08), (-0.06, -0.08), (0.06, -0.08)] {
            let screw = mesh(pg("JunctionBoxes__screw", cyl_geo!(0.005, 0.005, 0.005, 6)), &conduit_mat);
            screw.rotation().x = PI / 2.0;
            screw.position().set(sx, sy, 0.032);
            jbox.add(&screw);
        }
        // 2-3 conduits leaving the box (dir, y, len).
        let conduit_dirs = [
            ("up", 0.095, 0.35), // starts in the box top (0.10), not 5 cm above it
            ("left", 0.02, 0.28),
            ("right", 0.02, 0.28),
        ];
        // Always up-conduit; 60% chance of side conduits.
        let mut chosen = vec![conduit_dirs[0]];
        if rnd() < 0.6 { chosen.push(conduit_dirs[1]); }
        if rnd() < 0.6 { chosen.push(conduit_dirs[2]); }
        for &(dir, cy, len) in &chosen {
            let conduit = mesh(pg("JunctionBoxes__conduit", cyl_geo!(0.015, 0.015, len, 6)), &conduit_mat);
            if dir == "up" {
                conduit.position().set(0, cy + len / 2.0, 0.015);
            } else {
                conduit.rotation().z = PI / 2.0;
                conduit.position().set((if dir == "left" { -1.0 } else { 1.0 }) * (0.08 + len / 2.0), cy, 0.015);
            }
            jbox.add(&conduit);
        }
        // R305 25% chance: torn wire dangling from bottom of junction box.
        if rnd() < 0.25 {
            let wire_mat = lambert!({ color: 0x181410 });
            let copper_mat = lambert!({ color: 0xa87028 });
            let wire_len = 0.18 + rnd() * 0.22;
            let wire = mesh(pg("JunctionBoxes__wire", cyl_geo!(0.004, 0.004, wire_len, 5)), &wire_mat);
            // Slight tilt so wire hangs "naturally".
            let tilt_z = (rnd() - 0.5) * 0.25;
            wire.rotation().z = tilt_z;
            let wire_drop_x = tilt_z.sin() * (wire_len / 2.0);
            let wpx = (rnd() - 0.5) * 0.08 - wire_drop_x;
            wire.position().set(wpx, -0.10 - wire_len / 2.0, 0.015);
            jbox.add(&wire);
            // Exposed copper tip at bottom.
            let tip = mesh(pg("JunctionBoxes__tip", cyl_geo!(0.005, 0.005, 0.02, 5)), &copper_mat);
            tip.rotation().z = tilt_z;
            let wp = wire.pos();
            tip.position().set(
                wp.x + tilt_z.sin() * (wire_len / 2.0 + 0.01),
                wp.y - tilt_z.cos() * (wire_len / 2.0 + 0.01),
                0.015,
            );
            jbox.add(&tip);
        }
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.035);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.035);
        let jb_height = 1.6 + rnd() * 0.5;
        if c.dx.abs() == 1 {
            jbox.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            jbox.position().set(wall_x, jb_height, wall_z + (rnd() - 0.5) * 1.2);
        } else {
            jbox.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            jbox.position().set(wall_x + (rnd() - 0.5) * 1.2, jb_height, wall_z);
        }
        g.world.add(&jbox);
        // R281 track for arc audio events.
        let jp = jbox.pos();
        g.pp.junction_box_positions.push(v3(jp.x, jb_height, jp.z));
        i += 1;
    }
}

// ------------------------------------------------------------------ line 55642
// R258 bulletin boards: 15% chance per level, 1 corkboard with pinned notes.
pub fn place_bulletin_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let board = group();
    // Corkboard base — brown-tan.
    let board_mat = lambert!({ color: 0x6a4a2e });
    let base = mesh(pg("BulletinBoards__cork", plane_geo!(1.0, 0.6)), &board_mat);
    board.add(&base);
    // Wood frame — 4 thin bars around edges.
    let frame_mat = lambert!({ color: 0x3a2818 });
    let frame_thick = 0.03;
    for (ox, oy, sx, sy) in [
        (0.0, 0.31, 1.04, 0.03), (0.0, -0.31, 1.04, 0.03), // top + bottom
        (-0.51, 0.0, 0.03, 0.6), (0.51, 0.0, 0.03, 0.6),   // left + right
    ] {
        let fr = mesh(pg("BulletinBoards__framebar", plane_geo!(sx, sy)), &frame_mat);
        fr.position().set(ox, oy, 0.001);
        board.add(&fr);
    }
    // Mini notes: 3-6 small rectangles pinned across the board.
    let note_count = 3 + (rnd() * 4.0).floor() as i32;
    let note_colors = [0xf0e8c8, 0xe0d0a0, 0xd8c0a4, 0xf8f0e0, 0xc8c8b0];
    for n in 0..note_count {
        let nw = 0.13 + rnd() * 0.08;
        let nh = 0.10 + rnd() * 0.06;
        let note_mat = lambert!({ color: pick(&note_colors) });
        let note = mesh(pg("BulletinBoards__note", plane_geo!(nw, nh)), &note_mat);
        // Position within the board area (leave 0.1 margin from edge).
        let nx = (rnd() - 0.5) * 0.72;
        let ny = (rnd() - 0.5) * 0.32;
        note.position().set(nx, ny, 0.002);
        let base_note_rot_z = (rnd() - 0.5) * 0.25; // slight pin-tilt
        note.rotation().z = base_note_rot_z;
        board.add(&note);
        // Tiny pin (dark dot at top-center).
        let pin = mesh(plane_geo!(0.008, 0.008), basic_mat!({ color: 0x2a1a10 }));
        let np = note.pos();
        pin.position().set(np.x, np.y + nh * 0.42, 0.003);
        board.add(&pin);
        // R298 10% chance this note flutters — subtle sway animator.
        if rnd() < 0.10 {
            let flutter_phase = rnd() * PI * 2.0;
            let note2 = note.clone();
            g.animate(move |_g, _dt, t| {
                note2.rotation().z = base_note_rot_z + (t * 0.7 * PI + flutter_phase).sin() * 0.03;
            });
        }
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
    let board_height = 1.4 + rnd() * 0.3;
    if c.dx.abs() == 1 {
        board.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        board.position().set(wall_x, board_height, wall_z + (rnd() - 0.5) * 0.8);
    } else {
        board.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        board.position().set(wall_x + (rnd() - 0.5) * 0.8, board_height, wall_z);
    }
    g.world.add(&board);
}

// ------------------------------------------------------------------ line 55861
// R256 exit signs: 15% chance per level, at most 1 sign, mounted high
// (2.4-2.7 m). Faded green glow, optionally flickering.
pub fn place_exit_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    // Frame: dark box behind the glowing plane.
    let frame_mat = lambert!({ color: 0x181812 });
    let frame = mesh(pg("ExitSigns__housing", box_geo!(0.42, 0.20, 0.04)), &frame_mat);
    // Glowing plane in front. R259: per-sign material for animator flicker.
    let glow_mat = basic_mat!({ map: exit_sign_texture_2(), transparent: true, opacity: 0.85, side: DoubleSide });
    // R259 flicker animator: 30% chance the sign flickers — stable at 0.85,
    // briefly dips to 0.15 for 40 ms at random 3-8 s intervals.
    if rnd() < 0.30 {
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let gm = glow_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                gm.m().opacity = if dip_t > 0.0 { 0.15 } else { 0.85 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.04; // 40 ms brief dip
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
    let glow = mesh(plane_geo!(0.40, 0.18), &glow_mat);
    // housing back on the wall (4 cm deep); glow 1 mm in front of its face
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
    let sign_height = 2.4 + rnd() * 0.3;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        frame.rotation().y = rot;
        glow.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        frame.position().set(wall_x, sign_height, wall_z + jit_z);
        glow.position().set(wall_x - c.dx as f64 * 0.021, sign_height, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        frame.rotation().y = rot;
        glow.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        frame.position().set(wall_x + jit_x, sign_height, wall_z);
        glow.position().set(wall_x + jit_x, sign_height, wall_z - c.dz as f64 * 0.021);
    }
    g.world.add(&frame);
    g.world.add(&glow);
}

// ------------------------------------------------------------------ line 55965
// R256 thermostats: 20% chance per level, 1 small wall unit (1.3-1.5 m).
pub fn place_thermostats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    // Housing box + dial face.
    let housing_mat = lambert!({ color: 0x9a8a70 });
    let housing = mesh(pg("Thermostats__housing", box_geo!(0.12, 0.12, 0.03)), &housing_mat);
    let dial_mat = lambert!({ map: thermostat_texture(), transparent: true, side: DoubleSide });
    let dial = mesh(plane_geo!(0.10, 0.10), &dial_mat);
    // housing back on the wall (3 cm deep); dial 0.5 mm in front of its face
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let th_height = 1.3 + rnd() * 0.2;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        housing.rotation().y = rot;
        dial.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        housing.position().set(wall_x, th_height, wall_z + jit_z);
        dial.position().set(wall_x - c.dx as f64 * 0.0155, th_height, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        housing.rotation().y = rot;
        dial.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        housing.position().set(wall_x + jit_x, th_height, wall_z);
        dial.position().set(wall_x + jit_x, th_height, wall_z - c.dz as f64 * 0.0155);
    }
    g.world.add(&housing);
    g.world.add(&dial);
}

// ------------------------------------------------------------------ line 56014
// R255 radiator: 0-1 cast-iron ridged heater against a random wall.
pub fn place_radiators(g: &mut Game, _dist: &[i32]) {
    g.pp.radiator_positions.clear(); // R272 reset per level
    if rnd() < 0.75 { return; } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let body_color = vent_color_by_weather(g.weather_key());
    let body_mat = lambert!({ color: body_color });
    let radiator = group();
    // Main body: 0.7 wide × 0.5 tall × 0.10 deep flat box.
    let body = mesh(pg("Radiators__body", box_geo!(0.70, 0.50, 0.10)), &body_mat);
    body.position().y = 0.30;
    radiator.add(&body);
    // 8 vertical fins on front face — each a thin tall box.
    for f in 0..8 {
        let fin = mesh(pg("Radiators__fin", box_geo!(0.06, 0.48, 0.02)), &body_mat);
        fin.position().set(-0.31 + f as f64 * 0.085, 0.30, 0.06);
        radiator.add(&fin);
    }
    // Top slat (cap piece).
    let cap = mesh(pg("Radiators__cap", box_geo!(0.72, 0.03, 0.12)), &body_mat);
    cap.position().y = 0.565;
    radiator.add(&cap);
    // Position: against the wall.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.10);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.10);
    if c.dx.abs() == 1 {
        radiator.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        radiator.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        radiator.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        radiator.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&radiator);
    // R272 track for steam-hiss audio events.
    let rp = radiator.pos();
    g.pp.radiator_positions.push(v3(rp.x, 0.30, rp.z));
}

// ------------------------------------------------------------------ line 56066
// R255 fire extinguisher: 0-1 red cylinder mounted in bracket on a random wall.
pub fn place_fire_extinguishers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 { return; } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let red_mat = lambert!({ color: 0xa02010 });
    let metal_mat = lambert!({ color: 0x3a3028 });
    let black_mat = lambert!({ color: 0x1a1410 });
    let extg = group();
    // Body: cylinder 0.09 r × 0.42 tall.
    let body = mesh(pg("FireExtinguishers__body", cyl_geo!(0.09, 0.09, 0.42, 12)), &red_mat);
    body.position().y = 0.21;
    extg.add(&body);
    // Neck/valve: smaller metal cylinder on top.
    let neck = mesh(pg("FireExtinguishers__valve", cyl_geo!(0.03, 0.05, 0.06, 8)), &metal_mat);
    neck.position().y = 0.45;
    extg.add(&neck);
    // Handle bar (horizontal short box on top of neck).
    let handle = mesh(pg("FireExtinguishers__lever", box_geo!(0.10, 0.02, 0.03)), &black_mat);
    handle.position().y = 0.485;
    extg.add(&handle);
    // Nozzle short cylinder on the front.
    let nozzle = mesh(pg("FireExtinguishers__hose", cyl_geo!(0.008, 0.008, 0.14, 6)), &black_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(0.09, 0.32, 0);
    extg.add(&nozzle);
    // Wall bracket (small dark box behind body).
    let bracket = mesh(pg("FireExtinguishers__bracket", box_geo!(0.15, 0.12, 0.03)), &metal_mat);
    bracket.position().set(-0.09, 0.25, 0);
    extg.add(&bracket);
    // the group's front is +X (nozzle) and the bracket plate is its back
    // (local x −0.165): turn −X to the wall and put that plate on it
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.165);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.165);
    extg.rotation().y = (c.dz as f64).atan2(-(c.dx as f64));
    if c.dx.abs() == 1 {
        extg.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        extg.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&extg);
}

// ------------------------------------------------------------------ line 56135
// R254 rusted pipes: 0-2 horizontal pipes per level along random walls
// at low height (0.15-0.35 m).
pub fn place_rusted_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.60 { return; } // 40% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let pipe_mat = lambert!({ color: 0x5a3820 });
    let bracket_mat = lambert!({ color: 0x3a2818 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Pipe: 0.04 radius, 2.8-3.4 m long — spans nearly full cell.
        let pipe_len = 2.8 + rnd() * 0.6;
        let geo = pg("RustedPipes__pipe", cyl_geo!(0.04, 0.04, pipe_len, 8));
        // CylinderGeometry axis is Y — rotate to lie along wall direction.
        let pipe = mesh(geo, &pipe_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.08);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.08);
        let pipe_height = 0.15 + rnd() * 0.20;
        if c.dx.abs() == 1 {
            // Wall runs along Z axis. Pipe should also run along Z.
            pipe.rotation().z = PI / 2.0;
            pipe.rotation().y = PI / 2.0;
            pipe.position().set(wall_x, pipe_height, g.wz(c.z));
        } else {
            // Wall runs along X axis. Pipe runs along X.
            pipe.rotation().z = PI / 2.0; // Y→X axis
            pipe.position().set(g.wx(c.x), pipe_height, wall_z);
        }
        g.world.add(&pipe);
        // Occasional bracket midway (30% chance).
        if rnd() < 0.30 {
            let bg = pg("RustedPipes__bracket", box_geo!(0.06, 0.05, 0.08));
            let bracket = mesh(bg, &bracket_mat);
            // wall standoff: bolted foot (-y) against the wall, pad against the pipe's back
            bracket.position().copy(&pipe.pos());
            if c.dx.abs() == 1 {
                bracket.rotation().z = c.dx as f64 * PI / 2.0;
                bracket.position().x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
            } else {
                bracket.rotation().x = -(c.dz as f64) * PI / 2.0;
                bracket.position().z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
            }
            g.world.add(&bracket);
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 56332
// R253 stopped wall clock: 20% chance per level — circular wall prop with a
// frozen cryptic time (1.9-2.4 m).
pub fn place_wall_clock(g: &mut Game, _dist: &[i32]) {
    g.pp.wall_clock_positions.clear(); // R308 reset per level
    g.pp.cracked_clock_positions.clear(); // R342 reset per level
    if rnd() < 0.80 { return; } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = pick(&candidates);
    let geo = circle_geo!(0.20, 24);
    // R308: pick one of 6 clock face variants; R310 cracked variant is the last.
    let clock_textures = clock_textures();
    let clock_tex_idx = (rnd() * clock_textures.len() as f64).floor() as usize;
    let clock_tex = clock_textures[clock_tex_idx].clone();
    let is_cracked = clock_tex_idx == clock_textures.len() - 1;
    let mat = lambert!({ map: clock_tex, transparent: true, side: DoubleSide });
    let clock = mesh(geo, &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let clock_height = 1.9 + rnd() * 0.5;
    if c.dx.abs() == 1 {
        clock.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        clock.position().set(wall_x, clock_height, wall_z + (rnd() - 0.5) * 1.4);
    } else {
        clock.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        clock.position().set(wall_x + (rnd() - 0.5) * 1.4, clock_height, wall_z);
    }
    // R337 slight ±0.03 rad Z tilt for crooked-hang aesthetic.
    clock.rotation().z = (rnd() - 0.5) * 0.06;
    g.world.add(&clock);
    // R308 track for tick audio events.
    let cp = clock.pos();
    g.pp.wall_clock_positions.push(v3(cp.x, clock_height, cp.z));
    // R339 cracked variant sways slightly (0.10 Hz, ±0.015 rad).
    if is_cracked {
        let base_sway_z = clock.rot().z;
        let sway_phase = rnd() * PI * 2.0;
        let clock2 = clock.clone();
        g.animate(move |_g, _dt, t| {
            clock2.rotation().z = base_sway_z + (t * 0.2 * PI + sway_phase).sin() * 0.015;
        });
        // R342 track cracked clock position for periodic creak audio.
        g.pp.cracked_clock_positions.push(v3(cp.x, clock_height, cp.z));
    }
    // R332 cracked-clock floor shards.
    if is_cracked {
        let shard_mat = basic_mat!({
            color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8,
            depthWrite: false, side: DoubleSide,
        });
        let sh_count = 2 + (rnd() * 3.0).floor() as i32;
        for sh in 0..sh_count {
            let sw = 0.012 + rnd() * 0.020;
            let shh = 0.016 + rnd() * 0.022;
            let shard = mesh(plane_geo!(sw, shh), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.10 + rnd() * 0.30;
            shard.position().set(cp.x + sh_ang.cos() * sh_rad, 0.003, cp.z + sh_ang.sin() * sh_rad);
            g.world.add(&shard);
        }
    }
}

// ------------------------------------------------------------------ line 56418
// R253 cracked wall segments: visible crack in the drywall with a dark hole behind.
pub fn place_cracked_wall_holes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.65 { return; } // 35% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let hole_mat = basic_mat!({ color: 0x000000 });
    let rim_mat = lambert!({ color: 0x2a2018 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Hole size varies: 0.10-0.22 wide × 0.08-0.18 tall (irregular).
        let w = 0.10 + rnd() * 0.12;
        let hole_h = 0.08 + rnd() * 0.10;
        let hole_plane = mesh(plane_geo!(w, hole_h), &hole_mat);
        // Rim: slightly larger tan-brown rectangle behind hole.
        let rim_plane = mesh(pg("CrackedWallHoles__rim", plane_geo!(w + 0.02, hole_h + 0.02)), &rim_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
        let h_height = 0.6 + rnd() * 1.4; // 0.6 - 2.0 m (waist to head)
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            hole_plane.rotation().y = rot;
            rim_plane.rotation().y = rot;
            let jit_z = (rnd() - 0.5) * 1.4;
            rim_plane.position().set(wall_x, h_height, wall_z + jit_z);
            // hole sits inside the rim's dish (2.5 mm back, still 2.5 mm off the wall)
            hole_plane.position().set(wall_x + c.dx as f64 * 0.0025, h_height, wall_z + jit_z);
            hole_plane.scale().set(0.66, 0.66, 1);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            hole_plane.rotation().y = rot;
            rim_plane.rotation().y = rot;
            let jit_x = (rnd() - 0.5) * 1.4;
            rim_plane.position().set(wall_x + jit_x, h_height, wall_z);
            hole_plane.position().set(wall_x + jit_x, h_height, wall_z + c.dz as f64 * 0.0025);
            hole_plane.scale().set(0.66, 0.66, 1);
        }
        g.world.add(&rim_plane);
        g.world.add(&hole_plane);
        i += 1;
    }
}

// ------------------------------------------------------------------ line 56665
// R250 weathered posters: 0-2 faded posters per level on random walls at
// eye-level (1.2-1.9 m).
pub fn place_weathered_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.55 { return; } // 45% chance of posters
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // 0.4 wide × 0.6 tall poster.
        let geo = plane_geo!(0.4, 0.6);
        let posters = poster_textures();
        let tex = posters[(rnd() * posters.len() as f64).floor() as usize].clone();
        // R338 per-weather color tint via material multiply.
        let poster_tint = poster_tint_by_weather(g.weather_key());
        let mat = lambert!({
            map: tex, color: poster_tint,
            transparent: true, opacity: 0.85 + rnd() * 0.10,
            side: DoubleSide,
        });
        let poster = mesh(geo, &mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.01);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.01);
        let mid_height = 1.2 + rnd() * 0.7; // 1.2 - 1.9 m
        if c.dx.abs() == 1 {
            poster.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            poster.position().set(wall_x, mid_height, wall_z + (rnd() - 0.5) * 1.4);
        } else {
            poster.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            poster.position().set(wall_x + (rnd() - 0.5) * 1.4, mid_height, wall_z);
        }
        // Slight tilt for aged/torn hang.
        poster.rotation().z = (rnd() - 0.5) * 0.06;
        g.world.add(&poster);
        // R316 8% chance: cracked-glass overlay atop the poster.
        if rnd() < 0.08 {
            let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
            let crack = mesh(plane_geo!(0.4, 0.6), &crack_mat);
            crack.position().copy(&poster.pos());
            crack.rotation().copy(&poster.rot());
            // Nudge slightly outward from wall so overlay renders in front.
            crack.position().x -= c.dx as f64 * 0.002;
            crack.position().z -= c.dz as f64 * 0.002;
            g.world.add(&crack);
            // R327 3-5 tiny floor shards beneath cracked poster.
            let ps_mat = basic_mat!({
                color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8,
                depthWrite: false, side: DoubleSide,
            });
            let ps_count = 3 + (rnd() * 3.0).floor() as i32;
            let pp = poster.pos();
            for sh in 0..ps_count {
                let sw = 0.014 + rnd() * 0.022;
                let shh = 0.018 + rnd() * 0.022;
                let shard = mesh(plane_geo!(sw, shh), &ps_mat);
                shard.rotation().x = -PI / 2.0;
                shard.rotation().z = rnd() * PI * 2.0;
                let sh_ang = rnd() * PI * 2.0;
                let sh_rad = 0.10 + rnd() * 0.35;
                shard.position().set(pp.x + sh_ang.cos() * sh_rad, 0.003, pp.z + sh_ang.sin() * sh_rad);
                g.world.add(&shard);
            }
        }
        i += 1;
    }
}
