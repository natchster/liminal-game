//! Wall-mounted props of this crate.
use crate::textures::{exit_sign_texture_2, thermostat_texture};
use crate::{pick, poster_tint_by_weather, shuffle, wall_candidates};
use game::prelude::*;

// R264 fire alarm pull-station: 25% chance per level, 1 unit (Blender batch
// `r06`): a die-cast single-action station on its red surface back box, FIRE
// raised over its lever, PULL DOWN on the lever, the glass rod across it, the
// cover's screws, the conduit up the wall to a junction box. Now and then it
// has been pulled: the lever down, the rod snapped, its glass on the floor.
pub fn place_fire_alarms(g: &mut Game, _dist: &[i32]) {
    g.pp.fire_alarm_positions.clear(); // R276 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let alarm = group();
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
    let ceil = g.ceil_at_world(g.wx(c.x), g.wz(c.z));
    // R295 15% chance: it was pulled; the glass rod's pieces on the floor below.
    let pulled = rnd() < 0.15;
    let th = alarm.rot().y;
    let (along, out) = ((th.cos(), -th.sin()), (th.sin(), th.cos()));
    if pulled {
        let glass = lambert!({ color: 0xc4d2cc, transparent: true, opacity: 0.55 });
        let shard_count = 3 + (rnd() * 4.0).floor() as i32;
        for _ in 0..shard_count {
            let shard_w = 0.015 + rnd() * 0.020;
            let shard_h = 0.020 + rnd() * 0.020;
            let tip = (rnd() - 0.5) * 0.4;
            let turn = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.05 + rnd() * 0.25;
            // a piece of the 5 mm rod, lying in front of the wall
            let shard = mesh(real_part("Real__rodShard", 0.02, 0.005, 0.005), &glass);
            let k = ((shard_w + shard_h) / 0.045).clamp(0.85, 1.15);
            shard.scale().set(k, k, k);
            shard.rotation().set(tip * 0.2, turn, 0);
            // in front of the wall, not in it
            let (a, n) = (ang.cos() * rad, 0.03 + ang.sin().abs() * rad);
            shard.position().set(ap.x + along.0 * a + out.0 * n, 0.0, ap.z + along.1 * a + out.1 * n);
            g.world.add(&shard);
        }
    }
    let st = build_pull_station(pulled, alarm_height, ceil);
    alarm.add(&st);
}

// R263 coat hook rows: 20% chance per level, 1 rail of 3-5 hooks (Blender
// batch `r06`): an oak rail screwed to the wall, cast iron double coat-and-
// hat hooks along it, now and then an overcoat still hanging from one.
pub fn place_coat_hook_rows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let row = group();
    // the rail on the wall (its back 25 mm behind the row's origin)
    let oak = lambert!({ map: &rail_wood_tex() });
    let rail = mesh(real_part("Real__hookRail", 0.70, 0.095, 0.019), &oak);
    rail.position().z = -0.025;
    row.add(&rail);
    let steel = phong!({ color: 0x5e5c57, specular: 0x2a2a2a, shininess: 30 });
    for sx in [-0.335, 0.335] {
        let sc = mesh(real_part("Real__hookScrew", 0.01, 0.01, 0.002), &steel);
        sc.position().set(sx, 0, -0.025 + 0.019);
        row.add(&sc);
    }
    // 3-5 hooks along it
    let hook_count = 3 + (rnd() * 3.0).floor() as i32;
    // R264: 15% chance one hook has a coat still hanging.
    let coat_idx = if rnd() < 0.15 { (rnd() * hook_count as f64).floor() as i32 } else { -1 };
    let coat_colors = [0x2a1810, 0x1a1a2a, 0x2a1a1a, 0x1a2a1a];
    let iron = phong!({ color: 0x1c1b1a, specular: 0x1a1a1a, shininess: 22 });
    for h in 0..hook_count {
        let x_off = -0.30 + (h as f64 / (hook_count - 1) as f64) * 0.60;
        let hook = mesh(real_part("Real__coatHook", 0.026, 0.13, 0.1), &iron);
        hook.position().set(x_off, 0.0, -0.025 + 0.019);
        row.add(&hook);
        if h == coat_idx {
            // the old palette's pick (the same draw): an overcoat in wool of that hue
            let old = pick(&coat_colors);
            let wool = match old {
                0x2a1810 => 0x7a6249, // camel gone brown
                0x1a1a2a => 0x47506a, // navy
                0x2a1a1a => 0x66403c, // oxblood
                _ => 0x5a5e4a,        // loden
            };
            let coat = mesh(real_part("Real__coatHung", 0.5, 1.0, 0.2), lambert!({ map: &wool_tex(), color: wool, side: DoubleSide }));
            // hung by its loop on the coat hook's cradle
            coat.position().set(x_off, -0.047, -0.006 + 0.058);
            row.add(&coat);
            let buttons =
                mesh(real_part("Real__coatButtons", 0.03, 0.4, 0.01), phong!({ color: 0x2a2018, specular: 0x202020, shininess: 40 }));
            buttons.position().copy(&coat.pos());
            row.add(&buttons);
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

// R262 missing photo hook: 25% chance per level, 1-2 hooks with a faded
// rectangle where a photo used to hang.
pub fn place_missing_photo_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
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

// R260 water fountains: 15% chance per level. Wall-mounted steel unit.
pub fn place_water_fountains(g: &mut Game, _dist: &[i32]) {
    g.pp.fountain_positions.clear(); // R272 reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R260 wall junction boxes: 20% chance per level, 1-2 units with conduits.
pub fn place_junction_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.junction_box_positions.clear(); // R281 reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
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
        if rnd() < 0.6 {
            chosen.push(conduit_dirs[1]);
        }
        if rnd() < 0.6 {
            chosen.push(conduit_dirs[2]);
        }
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
            tip.position().set(wp.x + tilt_z.sin() * (wire_len / 2.0 + 0.01), wp.y - tilt_z.cos() * (wire_len / 2.0 + 0.01), 0.015);
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

// R258 bulletin boards: 15% chance per level, 1 corkboard with pinned notes.
pub fn place_bulletin_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
        (0.0, 0.31, 1.04, 0.03),
        (0.0, -0.31, 1.04, 0.03), // top + bottom
        (-0.51, 0.0, 0.03, 0.6),
        (0.51, 0.0, 0.03, 0.6), // left + right
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

// R256 exit signs: 15% chance per level, at most 1 sign, mounted high
// (2.4-2.7 m). Faded green glow, optionally flickering.
pub fn place_exit_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R256 thermostats: 20% chance per level, 1 small wall unit (1.3-1.5 m).
pub fn place_thermostats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R255 radiator: 0-1 cast-iron ridged heater against a random wall.
pub fn place_radiators(g: &mut Game, _dist: &[i32]) {
    g.pp.radiator_positions.clear(); // R272 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R255 fire extinguisher: 0-1 red cylinder mounted in bracket on a random wall.
pub fn place_fire_extinguishers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R254 rusted pipes: 0-2 runs of 2 in steel pipe per level along walls,
// low (0.15-0.35 m) (Blender batch `r06`): malleable iron fittings, strut
// clamps, now and then a gate valve, the old paint flaking off them and rust
// weeping from the joints down the wall.
pub fn place_rusted_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.60 {
        return;
    } // 40% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // its run: 2.8-3.4 m along the wall, its axis 0.08 m off it
        let pipe_len = 2.8 + rnd() * 0.6;
        let pipe_height = 0.15 + rnd() * 0.20;
        // (the old bracket's 30% chance, the same draw): a gate valve on the run
        let valve = rnd() < 0.30;
        let (rot, _) = wall_turn(&c);
        let held = wall_runs_on(g, &c);
        let run = build_pipe_run(pipe_len, pipe_height, valve, held);
        run.rotation().y = rot;
        let (fx, fz) = wall_face(g, &c);
        if c.dx.abs() == 1 {
            run.position().set(fx - c.dx as f64 * 0.08, pipe_height, g.wz(c.z));
        } else {
            run.position().set(g.wx(c.x), pipe_height, fz - c.dz as f64 * 0.08);
        }
        g.world.add(&run);
        i += 1;
    }
}

// R253 stopped wall clock: 20% chance per level — circular wall prop with a
// frozen cryptic time (1.9-2.4 m).
pub fn place_wall_clock(g: &mut Game, _dist: &[i32]) {
    g.pp.wall_clock_positions.clear(); // R308 reset per level
    g.pp.cracked_clock_positions.clear(); // R342 reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R253 cracked wall segments: a hole kicked or punched through the wall
// (Blender batch `r06`): the face paper torn back, the gypsum broken through
// its 12.7 mm, chunks hanging in on their paper, the stud bay behind it (a
// steel stud, the back of the far board), cracks out from it, the crumbs on
// the floor below. The wall is not cut: the hole's mask is drawn first and
// writes depth only, so the wall is not drawn inside it.
pub fn place_cracked_wall_holes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.65 {
        return;
    } // 35% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // its size: 0.10-0.22 wide x 0.08-0.18 tall (the same draws as ever)
        let w = 0.10 + rnd() * 0.12;
        let hole_h = 0.08 + rnd() * 0.10;
        let h_height = 0.6 + rnd() * 1.4; // 0.6 - 2.0 m (waist to head)
        let (rot, along) = wall_turn(&c);
        let jit = (rnd() - 0.5) * 1.4;
        let (wx, wz) = wall_face(g, &c);
        let hole = build_wall_hole(g, &c, w, hole_h, h_height, jit);
        hole.rotation().y = rot;
        hole.position().set(wx + along.0 * jit, h_height, wz + along.1 * jit);
        g.world.add(&hole);
        i += 1;
    }
}

// R250 weathered posters: 0-2 per level on random walls at eye level
// (1.2-1.9 m) (Blender batch `r06`): what offices and halls put up (a safety
// poster, a dance's bill, a bake sale, a recycling notice, flyers with their
// tear-off tabs), printed as real ones are, sun-faded and foxed, taped at
// the corners, one corner curled off the wall and one torn away under its
// tape. Now and then one is behind the glass of a clip frame, and the glass
// is broken.
pub fn place_weathered_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.55 {
        return;
    } // 45% chance of posters
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // which poster (the old texture choice's draw)
        let design = ((rnd() * POSTER_DESIGNS as f64).floor() as usize).min(POSTER_DESIGNS - 1);
        // R338 per-weather tint, multiplied on the print
        let poster_tint = poster_tint_by_weather(g.weather_key());
        // how long the sun has had it (the old opacity's draw)
        let faded = rnd() < 0.5;
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.01);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.01);
        let mid_height = 1.2 + rnd() * 0.7; // 1.2 - 1.9 m
        let poster = group();
        if c.dx.abs() == 1 {
            poster.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            poster.position().set(wall_x, mid_height, wall_z + (rnd() - 0.5) * 1.4);
        } else {
            poster.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            poster.position().set(wall_x + (rnd() - 0.5) * 1.4, mid_height, wall_z);
        }
        // Slight tilt: hung by hand.
        let tilt = (rnd() - 0.5) * 0.06;
        // R316 8% chance: in a clip frame whose glass is broken
        let framed = rnd() < 0.08;
        let sheet = build_poster(design, faded, poster_tint, framed);
        sheet.rotation().z = tilt;
        sheet.position().z = -0.01;
        poster.add(&sheet);
        g.world.add(&poster);
        if framed {
            // R327 3-5 pieces of its glass on the floor beneath
            let glass = lambert!({ color: 0xb8c6c0, transparent: true, opacity: 0.45 });
            let ps_count = 3 + (rnd() * 3.0).floor() as i32;
            let pp = poster.pos();
            let th = poster.rot().y;
            let (along, out) = ((th.cos(), -th.sin()), (th.sin(), th.cos()));
            for _ in 0..ps_count {
                let sw = 0.014 + rnd() * 0.022;
                let shh = 0.018 + rnd() * 0.022;
                let turn = rnd() * PI * 2.0;
                let sh_ang = rnd() * PI * 2.0;
                let sh_rad = 0.10 + rnd() * 0.35;
                let shard = mesh(real_part("Real__glassShard", 0.03, 0.003, 0.02), &glass);
                let k = ((sw + shh) / 0.05).clamp(0.85, 1.15);
                shard.scale().set(k, k, k);
                shard.rotation().y = turn;
                // in front of the wall, not in it
                let (a, n) = (sh_ang.cos() * sh_rad, 0.02 + sh_ang.sin().abs() * sh_rad);
                shard.position().set(pp.x + along.0 * a + out.0 * n, 0.001, pp.z + along.1 * a + out.1 * n);
                g.world.add(&shard);
            }
        }
        i += 1;
    }
}

// ------------------------------------------------------------ r06 helpers

/// A Blender part of batch `r06` as modelled (a scale of one), or a box
/// standing in for it when the props are switched off.
fn real_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// The turn that faces a wall prop's +z out of candidate `c`'s wall, and
/// the direction along the wall its jitter moves it.
fn wall_turn(c: &crate::Cand) -> (f64, (f64, f64)) {
    if c.dx.abs() == 1 {
        (if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 }, (0.0, 1.0))
    } else {
        (if c.dz > 0 { PI } else { 0.0 }, (1.0, 0.0))
    }
}

/// Where candidate `c`'s wall face is (x, z), on the line through its tile's centre.
fn wall_face(g: &Game, c: &crate::Cand) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * CELL * 0.5, g.wz(c.z) + c.dz as f64 * CELL * 0.5)
}

/// What the wall behind candidate `c` is made of: its arch material's key.
fn wall_key_of(g: &Game, c: &crate::Cand) -> String {
    let (nx, nz) = (c.x + c.dx, c.z + c.dz);
    if !g.in_grid(nx, nz) || g.layout.is_none() {
        return "wp:0".into();
    }
    g.wall_mat_key(g.t_idx(nx, nz))
}

/// A cheap hash of a few floats already drawn, for a choice that must not
/// draw (in 0..1).
fn mix01(a: f64, b: f64) -> f64 {
    let v = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    v - v.floor()
}

/// The hole: its group's origin on the wall's face at the hole's centre,
/// +z out of the wall, y up (the floor at -h_height).
fn build_wall_hole(g: &mut Game, c: &crate::Cand, w: f64, hole_h: f64, h_height: f64, jit: f64) -> Obj {
    // the board (1/2 in) and the bay of a 3-5/8 in stud behind it
    const BOARD: f64 = 0.0127;
    const BAY: f64 = 0.092;
    // the size: the nearest of the three modelled holes, scaled a little
    let size = (w * hole_h).sqrt();
    let kinds = [("A", 0.1015), ("B", 0.1373), ("C", 0.1833)];
    let (k, nominal) = kinds.iter().copied().min_by(|a, b| (a.1 - size).abs().total_cmp(&(b.1 - size).abs())).unwrap();
    let scale = (size / nominal).clamp(0.85, 1.15);
    let key = wall_key_of(g, c);
    let masonry = !(key.starts_with("wp:") || key.starts_with("drywall") || key.starts_with("paint"));
    let root = group();
    // it is in the wall on purpose: the placement pass must not push it out
    // (the settle pass still slides it onto the wall's real face)
    root.user().floats = true;
    let hg = group();
    hg.scale().set(scale, scale, scale);
    root.add(&hg);
    // drawn before the walls: what is inside the wall, then the mask
    let inside = |o: &Obj| {
        o.set_render_order(-3);
    };
    // what the broken edge is: gypsum, or the masonry the wall is built of
    let edge = if !masonry {
        0xaba496
    } else if key.starts_with("brick") {
        0x5e3428
    } else if key.starts_with("tile") {
        0xa29d91
    } else {
        0x77736b
    };
    let gypsum = lambert!({ color: edge });
    let gy = mesh(real_part(&format!("Real__hole{k}Gypsum"), 0.2, 0.15, 0.013), &gypsum);
    inside(&gy);
    hg.add(&gy);
    if !masonry {
        // the face paper: the wall's own surface, torn and pushed in, in the
        // shade of the hole
        let paper_mat = g.arch_mat(&key).duplicate();
        paper_mat.set_color(0x6a6a6a);
        let pa = mesh(real_part(&format!("Real__hole{k}Paper"), 0.2, 0.15, 0.01), &paper_mat);
        inside(&pa);
        hg.add(&pa);
        // the stud bay: a steel stud where the hole happens to be by one
        let off = -0.20 + mix01(jit, h_height) * 0.40;
        let stud_mat = lambert!({ color: 0x3c3e3f });
        for sx in [-0.406, 0.0, 0.406] {
            let x = off + sx;
            if x.abs() > 0.38 {
                continue;
            }
            let st = mesh(real_part("Real__holeStud", 0.04, 0.5, 0.092), &stud_mat);
            st.position().x = x;
            inside(&st);
            hg.add(&st);
        }
    }
    // the back of the bay: the far board's paper back (or the rough core
    // of a block), dark away from the light coming in through the hole
    let back_depth = if masonry { 0.035 } else { BOARD + BAY };
    let back_tint = if key.starts_with("brick") { 0xb07060 } else { 0xffffff };
    let back = mesh(plane_geo!(0.8, 0.5), lambert!({ map: &hole_back_tex(masonry), color: back_tint }));
    back.position().z = -back_depth;
    inside(&back);
    hg.add(&back);
    // the mask: the torn outline, writing depth only
    let mask = mesh(real_part(&format!("Real__hole{k}Mask"), 0.2, 0.15, 0.001), basic_mat!({ colorWrite: false }));
    mask.set_render_order(-2);
    hg.add(&mask);
    // the cracks out from it, the dent, the scuffs: on the wall, behind the mask
    let ki = match k {
        "A" => 0,
        "B" => 1,
        _ => 2,
    };
    let decal = mesh(plane_geo!(0.56, 0.56), lambert!({ map: &hole_decal_tex(ki), transparent: true, depthWrite: false }));
    decal.position().z = 0.0008;
    hg.add(&decal);
    // what fell out of it, on the floor at the wall's foot
    let chips = mesh(real_part("Real__holeChips", 0.25, 0.02, 0.12), &gypsum);
    chips.position().set(0, -h_height, 0.01);
    root.add(&chips);
    let dust = mesh(
        plane_geo!(0.42, 0.22),
        lambert!({ map: &hole_dust_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    dust.rotation().x = -PI / 2.0;
    dust.position().set(0, -h_height + 0.002, 0.10);
    root.add(&dust);
    root
}

/// The far board's back seen through a hole: grey-brown kraft paper with
/// its dust, lit only where the hole lets the light in (or the rough grey
/// of a broken block's core).
fn hole_back_tex(masonry: bool) -> Tex {
    thread_local! { static T: std::cell::OnceCell<[Tex; 2]> = const { std::cell::OnceCell::new() }; }
    let pair = T.with(|c| {
        c.get_or_init(|| {
            let paper = canvas_tex_opt(256, 320, false, |ctx, w, h| {
                ctx.set_fill_style("#4c463d");
                ctx.fill_rect(0, 0, w, h);
                // the paper's fibre and the dust run down it
                for _ in 0..900 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    ctx.set_fill_style(&format!(
                        "rgba({},{},{},{})",
                        60 + (rnd() * 60.0) as i32,
                        55 + (rnd() * 50.0) as i32,
                        45 + (rnd() * 40.0) as i32,
                        0.15 + rnd() * 0.2
                    ));
                    ctx.fill_rect(x, y, 1.0 + rnd() * 2.0, 1.0 + rnd() * 5.0);
                }
                // the maker's line printed along the back paper, faint
                ctx.set_fill_style("rgba(40,52,70,0.45)");
                ctx.set_font("bold 13px Helvetica");
                ctx.set_text_align("left");
                ctx.fill_text("1/2 IN. REGULAR GYPSUM BOARD   ASTM C1396   1/2 IN. REGULAR", -30, h * 0.47);
                // gypsum dust settled where the board's crumbs fell, lower down
                let gr = ctx.create_linear_gradient(0, h * 0.55, 0, h);
                gr.add_color_stop(0, "rgba(220,214,200,0)");
                gr.add_color_stop(1, "rgba(220,214,200,0.35)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, h * 0.55, w, h * 0.45);
                // the light: only what comes in through the hole
                let lg = ctx.create_radial_gradient(w / 2.0, h / 2.0, 6, w / 2.0, h / 2.0, w * 0.55);
                lg.add_color_stop(0, "rgba(0,0,0,0.55)");
                lg.add_color_stop(0.22, "rgba(0,0,0,0.75)");
                lg.add_color_stop(0.6, "rgba(0,0,0,0.88)");
                lg.add_color_stop(1, "rgba(0,0,0,0.95)");
                ctx.set_fill_style(lg);
                ctx.fill_rect(0, 0, w, h);
            });
            let block = canvas_tex_opt(256, 320, false, |ctx, w, h| {
                ctx.set_fill_style("#6f6b64");
                ctx.fill_rect(0, 0, w, h);
                // the aggregate broken through
                for _ in 0..700 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 1.0 + rnd() * 4.0;
                    let v = (60.0 + rnd() * 90.0) as i32;
                    ctx.set_fill_style(&format!("rgba({},{},{},{})", v, v - 4, v - 10, 0.5 + rnd() * 0.4));
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.6 + rnd() * 0.4), rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
                let lg = ctx.create_radial_gradient(w / 2.0, h / 2.0, 6, w / 2.0, h / 2.0, w * 0.55);
                lg.add_color_stop(0, "rgba(0,0,0,0.05)");
                lg.add_color_stop(0.4, "rgba(0,0,0,0.35)");
                lg.add_color_stop(1, "rgba(0,0,0,0.75)");
                ctx.set_fill_style(lg);
                ctx.fill_rect(0, 0, w, h);
            });
            [paper, block]
        })
        .clone()
    });
    pair[masonry as usize].clone()
}

/// Round a hole on the wall: hairline cracks out from its rim, the dent the
/// blow left, grey smudges of hands, a shoe's black scuff under the big one
/// (transparent where unmarked). `k` the hole's size: 0 a fist, 1, 2 a kick.
fn hole_decal_tex(k: usize) -> Tex {
    thread_local! { static T: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            [(0.056, 0.046), (0.076, 0.062), (0.105, 0.080)]
                .iter()
                .enumerate()
                .map(|(ki, &(rx, ry))| {
                    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
                        ctx.clear_rect(0, 0, w, h);
                        let px = w / 0.56; // pixels a metre
                        let (cx, cy) = (w / 2.0, h / 2.0);
                        let (ex, ey) = (rx * px, ry * px);
                        // the dent: the board pushed in round the hole, its shade below
                        let dent = ctx.create_radial_gradient(cx, cy + ey * 0.15, ex * 0.9, cx, cy + ey * 0.15, ex * 1.9);
                        dent.add_color_stop(0, "rgba(30,24,16,0.22)");
                        dent.add_color_stop(1, "rgba(30,24,16,0)");
                        ctx.set_fill_style(dent);
                        ctx.fill_rect(0, 0, w, h);
                        // the cracks: from the rim, wandering, now and then forking
                        let n = 4 + ki * 2;
                        for j in 0..n {
                            let a0 = j as f64 / n as f64 * std::f64::consts::TAU + rnd() * 0.6;
                            let mut x = cx + a0.cos() * ex * 0.95;
                            let mut y = cy + a0.sin() * ey * 0.95;
                            let mut a = a0;
                            let len = (0.35 + rnd() * 0.9) * ex;
                            let steps = 14;
                            let mut pts = vec![(x, y)];
                            for _ in 0..steps {
                                a += (rnd() - 0.5) * 0.9;
                                let l = len / steps as f64;
                                x += a.cos() * l;
                                y += a.sin() * l * 0.9 + l * 0.08; // they run a little downward
                                pts.push((x, y));
                            }
                            // its lit lip, then the crack itself
                            for (col, lw, dx) in [("rgba(250,245,230,0.22)", 1.4, 1.0), ("rgba(28,22,16,0.6)", 1.0, 0.0)].iter() {
                                ctx.set_stroke_style(*col);
                                ctx.begin_path();
                                for (q, &(x, y)) in pts.iter().enumerate() {
                                    let fade = 1.0 - q as f64 / pts.len() as f64;
                                    ctx.set_line_width(lw * (0.35 + 0.9 * fade));
                                    if q == 0 {
                                        ctx.move_to(x + dx, y + dx);
                                    } else {
                                        ctx.line_to(x + dx, y + dx);
                                    }
                                }
                                ctx.stroke();
                            }
                            if rnd() < 0.5 {
                                let (bx, by) = pts[5 + (rnd() * 6.0) as usize];
                                let b = a0 + (rnd() - 0.5) * 2.0;
                                ctx.set_stroke_style("rgba(28,22,16,0.6)");
                                ctx.set_line_width(0.8);
                                ctx.begin_path();
                                ctx.move_to(bx, by);
                                let (mut x2, mut y2, mut a2) = (bx, by, b);
                                for _ in 0..6 {
                                    a2 += (rnd() - 0.5) * 1.0;
                                    x2 += a2.cos() * len / 22.0;
                                    y2 += a2.sin() * len / 22.0;
                                    ctx.line_to(x2, y2);
                                }
                                ctx.stroke();
                            }
                        }
                        // hands: grey smudges
                        for _ in 0..5 {
                            let a = rnd() * 6.3;
                            let r = (1.3 + rnd() * 1.4) * ex;
                            let (x, y) = (cx + a.cos() * r, cy + a.sin() * r * 0.9);
                            let s = ctx.create_radial_gradient(x, y, 0, x, y, 10.0 + rnd() * 16.0);
                            s.add_color_stop(0, "rgba(50,44,38,0.16)");
                            s.add_color_stop(1, "rgba(50,44,38,0)");
                            ctx.set_fill_style(s);
                            ctx.fill_rect(x - 30.0, y - 30.0, 60, 60);
                        }
                        // the kick's shoe, black rubber dragged down under it
                        if ki == 2 {
                            ctx.set_stroke_style("rgba(16,14,12,0.45)");
                            ctx.set_line_cap("round");
                            for k in 0..4 {
                                ctx.set_line_width(2.0 + rnd() * 3.0);
                                let x = cx - 20.0 + k as f64 * 12.0 + rnd() * 6.0;
                                ctx.begin_path();
                                ctx.move_to(x, cy + ey * 1.15);
                                ctx.bezier_curve_to(x + 4.0, cy + ey * 1.5, x - 3.0, cy + ey * 1.8, x + 2.0, cy + ey * (2.0 + rnd() * 0.6));
                                ctx.stroke();
                            }
                        }
                    })
                })
                .collect()
        })[k]
            .clone()
    })
}

/// Gypsum dust on the floor under a hole: fine white powder, heaviest at the
/// wall (the plane's top edge), with crumbs in it.
fn hole_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for _ in 0..40 {
                    let (x, y) = (w * (0.2 + 0.6 * rnd()), h * (0.05 + 0.55 * rnd() * rnd()));
                    let r = 8.0 + rnd() * 26.0;
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0, "rgba(226,220,206,0.22)");
                    gr.add_color_stop(1, "rgba(226,220,206,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                for _ in 0..160 {
                    let (x, y) = (w * (0.15 + 0.7 * rnd()), h * rnd() * rnd() * 0.9);
                    ctx.set_fill_style(&format!("rgba(236,230,216,{})", 0.4 + rnd() * 0.5));
                    ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
                }
            })
        })
        .clone()
    })
}

// ------------------------------------------------------------ the pipe run

/// A straight length of 2 in pipe from `a` to `b` (along one axis), its
/// paint and rust laid a metre to a repeat whatever its length.
fn pipe_length(mat: &Mat, a: Vec3, b: Vec3) -> Obj {
    let d = b - a;
    let len = d.length().max(0.001);
    let geo = cyl_geo!(0.03015, 0.03015, len, 16, 1, true);
    let uv: Vec<f32> = geo.r().uv.clone();
    let uv: Vec<f32> = uv.iter().enumerate().map(|(i, &v)| if i % 2 == 1 { v * len as f32 } else { v }).collect();
    geo.set_uv(uv);
    let m = mesh(geo, mat);
    m.position().set((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0);
    if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
        m.rotation().z = PI / 2.0;
    } else if d.z.abs() > d.y.abs() {
        m.rotation().x = PI / 2.0;
    }
    m
}

/// A fitting part at `at`, turned by `outer_y` then (inside that) by
/// `inner` (x, z).
fn pipe_fitting(name: &str, mat: &Mat, at: Vec3, outer_y: f64, inner: (f64, f64)) -> Obj {
    let o = group();
    o.position().copy(&at);
    o.rotation().y = outer_y;
    let m = mesh(real_part(name, 0.1, 0.1, 0.08), mat);
    m.rotation().x = inner.0;
    m.rotation().z = inner.1;
    o.add(&m);
    o
}

/// Whether the wall behind candidate `c` runs on past its tile to the run's
/// left (-x) and right (+x), as the run's group is turned.
fn wall_runs_on(g: &Game, c: &crate::Cand) -> [bool; 2] {
    let (rot, _) = wall_turn(c);
    let (ux, uz) = (rot.cos().round() as i32, (-rot.sin()).round() as i32);
    let (nx, nz) = (c.x + c.dx, c.z + c.dz);
    [g.is_wall_tile(nx - ux, nz - uz), g.is_wall_tile(nx + ux, nz + uz)]
}

/// The run: its group's origin on the pipe's axis at the middle of the run,
/// x along the wall, y up, z out of the wall (the wall at z = -0.08, the
/// floor at y = -h). Each end turns into the wall, down through the floor,
/// or up the wall and into it at 1.0-1.6 m; a coupling, a union, two strut
/// clamps, a gate valve when `valve`; rust weeping from the joints.
fn build_pipe_run(len: f64, h: f64, valve: bool, held: [bool; 2]) -> Obj {
    const EA: f64 = 0.057; // centre to end of a 2 in elbow
    const IN: f64 = 0.017; // how far a pipe's thread goes into its fitting
    const WALL: f64 = -0.08;
    let run = group();
    let k1 = mix01(len, h);
    let k2 = mix01(h * 1.7, len * 0.3);
    let paint = ((k1 * 7.0).fract() * 3.0) as usize;
    let pipe_mat = lambert!({ map: &pipe_tex(paint, false) });
    let fit_mat = lambert!({ map: &pipe_tex(paint, true) });
    let galv = lambert!({ map: &pipe_tex(3, true), color: 0xffffff });
    let plate = phong!({ color: 0x77766f, specular: 0x262626, shininess: 26 });
    let xe = len / 2.0;
    // the run itself, through its inline fittings
    run.add(&pipe_length(&pipe_mat, v3(-xe + EA - IN, 0, 0), v3(xe - EA + IN, 0, 0)));
    // where the wall runs on past an end, the pipe may go into it or up it;
    // where it stops (a pillar, a wall's end) the pipe goes down into the floor
    let ends = [if held[0] { (k1 * 3.0) as usize % 3 } else { 1 }, if held[1] { (k2 * 3.0) as usize % 3 } else { 1 }];
    let rise = 1.0 + (k1 * 13.0).fract() * 0.6 - h; // the top of a riser, above the run
    let mut joints: Vec<f64> = vec![];
    for (side, &end) in ends.iter().enumerate() {
        let sx = if side == 0 { -1.0 } else { 1.0 };
        let x = sx * xe;
        let oy = if side == 0 { 0.0 } else { PI };
        joints.push(x);
        match end {
            0 => {
                // into the wall: the elbow's other leg back, a nipple, its collar
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (-sx * PI / 2.0, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, 0, -EA + IN), v3(x, 0, WALL)));
                let col = mesh(real_part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.position().set(x, 0, WALL);
                run.add(&col);
            }
            1 => {
                // down through the floor, its collar on the floor
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (PI, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, -EA + IN, 0), v3(x, -h, 0)));
                let col = mesh(real_part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.rotation().x = -PI / 2.0;
                col.position().set(x, -h, 0);
                run.add(&col);
            }
            _ => {
                // up the wall on a clamp, then into it
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (0.0, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, EA - IN, 0), v3(x, rise - EA + IN, 0)));
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, rise, 0), PI / 2.0, (0.0, -PI / 2.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, rise, -EA + IN), v3(x, rise, WALL)));
                let col = mesh(real_part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.position().set(x, rise, WALL);
                run.add(&col);
                let cl = mesh(real_part("Real__pipeClamp", 0.04, 0.16, 0.12), &galv);
                cl.rotation().z = PI / 2.0;
                cl.position().set(x, rise * 0.55, 0);
                run.add(&cl);
            }
        }
    }
    // two strut clamps on the run
    for (side, sx) in [-1.0f64, 1.0].iter().enumerate() {
        let cl = mesh(real_part("Real__pipeClamp", 0.04, 0.16, 0.12), &galv);
        cl.position().x = sx * if held[side] { xe - 0.45 } else { 0.3 };
        run.add(&cl);
    }
    // a coupling where two lengths were joined, a union by the valve's place
    let xc = -xe + 0.9 + (k2 * 5.0).fract() * 0.5;
    run.add(&pipe_fitting("Real__pipeCoupling", &fit_mat, v3(xc, 0, 0), 0.0, (0.0, 0.0)));
    joints.push(xc);
    // now and then a tee, its branch back into the wall
    if (k1 * 17.0).fract() < 0.6 {
        let xt = xc + 0.55;
        run.add(&pipe_fitting("Real__pipeTee", &fit_mat, v3(xt, 0, 0), 0.0, (-PI / 2.0, 0.0)));
        run.add(&pipe_length(&pipe_mat, v3(xt, 0, -EA + IN), v3(xt, 0, WALL)));
        let col = mesh(real_part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
        col.position().set(xt, 0, WALL);
        run.add(&col);
        joints.push(xt);
    }
    let xv = xe - 0.85;
    if valve {
        let bronze = lambert!({ color: 0x5e4c32 });
        run.add(&pipe_fitting("Real__pipeValve", &bronze, v3(xv, 0, 0), 0.0, (0.0, 0.0)));
        let wheel = mesh(real_part("Real__pipeWheel", 0.115, 0.03, 0.115), lambert!({ color: 0x5a2219 }));
        wheel.position().set(xv, 0.150, 0);
        wheel.rotation().y = k2 * 1.5;
        run.add(&wheel);
        joints.push(xv);
        run.add(&pipe_fitting("Real__pipeUnion", &fit_mat, v3(xv - 0.16, 0, 0), 0.0, (0.0, 0.0)));
    } else {
        run.add(&pipe_fitting("Real__pipeUnion", &fit_mat, v3(xv, 0, 0), 0.0, (0.0, 0.0)));
        joints.push(xv);
    }
    // the rust that wept from the joints: down the wall, and dried on the floor
    let weep = lambert!({ map: &rust_weep_tex(), transparent: true, depthWrite: false });
    for (j, &x) in joints.iter().enumerate() {
        if (k1 * 31.0 + j as f64 * 0.37).fract() < 0.25 {
            continue; // not every joint leaks
        }
        let d = mesh(plane_geo!(0.12, 0.40), &weep);
        d.position().set(x + 0.01, -0.21, WALL + 0.0008);
        run.add(&d);
    }
    let stain = mesh(
        plane_geo!(0.34, 0.24),
        lambert!({ map: &rust_floor_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    stain.rotation().x = -PI / 2.0;
    stain.position().set(joints[(k2 * 9.0) as usize % joints.len()], -h + 0.002, WALL + 0.12);
    run.add(&stain);
    run
}

/// A pipe's old paint over its steel (`paint`: 0 grey, 1 green, 2 cream),
/// flaked off in patches, rust in them and blooming round them; a fitting's
/// (`fitting`) rustier still. 128 px round, a metre along.
fn pipe_tex(paint: usize, fitting: bool) -> Tex {
    thread_local! { static T: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            let mut v = vec![];
            for fit in [false, true] {
                for (base, edge) in [("#5f625e", "#4c4f4b"), ("#56614f", "#454f40"), ("#9a917a", "#7f7864"), ("#7a7c79", "#5d5f5c")] {
                    v.push(canvas_tex(128, 512, move |ctx, w, h| {
                        ctx.set_fill_style(base);
                        ctx.fill_rect(0, 0, w, h);
                        // brush marks and grime in the paint
                        for _ in 0..140 {
                            ctx.set_fill_style(&format!("rgba(30,26,20,{})", 0.04 + rnd() * 0.08));
                            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 6.0 + rnd() * 30.0);
                        }
                        // the flakes: paint lifted off, the steel rusted beneath
                        let n = if fit { 120 } else { 85 };
                        for _ in 0..n {
                            let (x, y) = (rnd() * w, rnd() * h);
                            let r = 2.0 + rnd() * if fit { 14.0 } else { 12.0 };
                            // the rust bloom round it
                            let gr = ctx.create_radial_gradient(x, y, r * 0.4, x, y, r * 2.2);
                            gr.add_color_stop(0, "rgba(110,56,24,0.55)");
                            gr.add_color_stop(1, "rgba(110,56,24,0)");
                            ctx.set_fill_style(gr);
                            ctx.fill_rect(x - r * 2.2, y - r * 2.2, r * 4.4, r * 4.4);
                            // the flake's edge, then the bare rust inside it
                            ctx.set_fill_style(edge);
                            ctx.begin_path();
                            let k = 7;
                            for q in 0..k {
                                let a = q as f64 / k as f64 * 6.283;
                                let rr = r * (0.7 + rnd() * 0.5) + 1.0;
                                if q == 0 {
                                    ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                                } else {
                                    ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                                }
                            }
                            ctx.close_path();
                            ctx.fill();
                            ctx.set_fill_style(if rnd() < 0.5 { "#5a301a" } else { "#3e2618" });
                            ctx.begin_path();
                            for q in 0..k {
                                let a = q as f64 / k as f64 * 6.283;
                                let rr = r * (0.55 + rnd() * 0.45);
                                if q == 0 {
                                    ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                                } else {
                                    ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                                }
                            }
                            ctx.close_path();
                            ctx.fill();
                        }
                        // rust run along the lower side (a quarter of the way round)
                        let gr = ctx.create_linear_gradient(w * 0.62, 0, w * 0.88, 0);
                        gr.add_color_stop(0, "rgba(96,50,22,0)");
                        gr.add_color_stop(0.5, if fit { "rgba(96,50,22,0.55)" } else { "rgba(96,50,22,0.3)" });
                        gr.add_color_stop(1, "rgba(96,50,22,0)");
                        ctx.set_fill_style(gr);
                        ctx.fill_rect(0, 0, w, h);
                        // dust on what faces up
                        let gd = ctx.create_linear_gradient(w * 0.1, 0, w * 0.4, 0);
                        gd.add_color_stop(0, "rgba(150,140,120,0)");
                        gd.add_color_stop(0.5, "rgba(150,140,120,0.18)");
                        gd.add_color_stop(1, "rgba(150,140,120,0)");
                        ctx.set_fill_style(gd);
                        ctx.fill_rect(0, 0, w, h);
                    }));
                }
            }
            v
        })[paint % 4 + if fitting { 4 } else { 0 }]
        .clone()
    })
}

/// Rust wept from a joint down a wall: thin runs from the top, wider and
/// paler as they spread, a tide line where each dried (transparent elsewhere).
fn rust_weep_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(96, 320, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..7 {
                    let x0 = w * (0.3 + 0.4 * rnd());
                    let len = h * (0.25 + 0.7 * rnd());
                    let a = 0.22 + 0.30 * rnd();
                    let mut x = x0;
                    let steps = 40;
                    for q in 0..steps {
                        let t = q as f64 / steps as f64;
                        let y = t * len;
                        x += (rnd() - 0.5) * 1.6;
                        let wd = 1.5 + t * (4.0 + k as f64 * 0.6);
                        ctx.set_fill_style(&format!("rgba(104,52,22,{})", a * (1.0 - t * 0.7)));
                        ctx.fill_rect(x - wd / 2.0, y, wd, len / steps as f64 + 1.0);
                    }
                    // the tide line where it stopped
                    ctx.set_fill_style(&format!("rgba(70,34,14,{})", a * 0.9));
                    ctx.begin_path();
                    ctx.ellipse(x, len, 3.0 + rnd() * 3.0, 2.0, 0, 0, 7);
                    ctx.fill();
                }
                let gr = ctx.create_linear_gradient(0, 0, 0, 30);
                gr.add_color_stop(0, "rgba(84,42,18,0.5)");
                gr.add_color_stop(1, "rgba(84,42,18,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(w * 0.2, 0, w * 0.6, 30);
            })
        })
        .clone()
    })
}

/// Where the drips dried on the floor: rings of rust, darker at their rims.
fn rust_floor_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(192, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for _ in 0..9 {
                    let (x, y) = (w * (0.3 + 0.4 * rnd()), h * (0.15 + 0.5 * rnd()));
                    let r = 8.0 + rnd() * 26.0;
                    let gr = ctx.create_radial_gradient(x, y, r * 0.2, x, y, r);
                    gr.add_color_stop(0, "rgba(98,52,24,0.20)");
                    gr.add_color_stop(0.85, "rgba(90,46,20,0.32)");
                    gr.add_color_stop(1, "rgba(90,46,20,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.6 + 0.3 * rnd()), rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
            })
        })
        .clone()
    })
}

// ------------------------------------------------------------ the pull station

/// The station: its group's origin 23 mm off the wall at the cover's middle
/// (the wall at z = -0.023), the conduit from its back box up to a junction
/// box under the ceiling `ceil` (absolute; the station at `at_y`).
fn build_pull_station(pulled: bool, at_y: f64, ceil: f64) -> Obj {
    let st = group();
    let o = group();
    o.position().z = -0.023;
    st.add(&o);
    let red = lambert!({ map: &alarm_paint_tex(), color: 0xffffff });
    let red_box = lambert!({ color: 0x7c1a13 });
    let white = lambert!({ color: 0xc4bfb1 });
    let steel = phong!({ color: 0x6a6964, specular: 0x2c2c2c, shininess: 34 });
    let zinc = phong!({ color: 0x7c7d79, specular: 0x262626, shininess: 22 });
    let glass = lambert!({ color: 0xc8d6d0, transparent: true, opacity: 0.5 });
    let add = |name: &str, mat: &Mat, x: f64, y: f64, z: f64| {
        let m = mesh(real_part(name, 0.1, 0.14, 0.03), mat);
        m.position().set(x, y, z);
        o.add(&m);
        m
    };
    add("Real__pullBox", &red_box, 0.0, 0.0, 0.0);
    add("Real__pullBody", &red, 0.0, 0.0, 0.0);
    add("Real__pullLetters", &white, 0.0, 0.0, 0.0);
    add("Real__pullScrews", &steel, 0.0, 0.0, 0.0);
    add("Real__pullLock", &steel, 0.030, -0.050, 0.035 + 0.030 + 0.0035);
    // the lever on its pivot, at the top of its well; down when pulled
    let pivot = group();
    pivot.position().set(0, 0.004 + 0.020 - 0.004, 0.035 + 0.030);
    pivot.rotation().x = if pulled { -0.75 } else { 0.0 };
    o.add(&pivot);
    let lever = mesh(real_part("Real__pullHandle", 0.07, 0.03, 0.016), &red);
    pivot.add(&lever);
    let text = mesh(real_part("Real__pullHandleText", 0.06, 0.008, 0.001), &white);
    pivot.add(&text);
    // the glass rod across it in its lugs (snapped when it was pulled)
    let rod = mesh(real_part(if pulled { "Real__pullRodBroken" } else { "Real__pullRod" }, 0.09, 0.005, 0.005), &glass);
    rod.position().set(0, 0.002, 0.035 + 0.030 + 0.0045);
    o.add(&rod);
    // the conduit: a connector in the box's knockout, EMT up the wall on its
    // straps, into a junction box under the ceiling
    let top = (ceil - 0.10).min(2.85) - at_y; // the junction box's top, above the station
    let jb_c = top - 0.051;
    add("Real__emtConnector", &zinc, 0.0, 0.070, 0.0175);
    let c0 = 0.070 + 0.030;
    let c1 = jb_c - 0.051;
    if c1 > c0 + 0.05 {
        let len = c1 - c0;
        let emt = mesh(cyl_geo!(0.00895, 0.00895, len, 10, 1, true), &zinc);
        emt.position().set(0, (c0 + c1) / 2.0, 0.0175);
        o.add(&emt);
        let n = ((len / 1.0).ceil() as i32).max(1);
        for k in 0..n {
            let y = c0 + (k as f64 + 0.5) * len / n as f64;
            add("Real__emtStrap", &zinc, 0.0, y, 0.0);
        }
        let jb = mesh(box_geo!(0.102, 0.102, 0.054), lambert!({ color: 0x75766f }));
        jb.position().set(0, jb_c, 0.027);
        o.add(&jb);
        let cover = mesh(box_geo!(0.106, 0.106, 0.004), lambert!({ color: 0x6c6d66 }));
        cover.position().set(0, jb_c, 0.056);
        o.add(&cover);
    }
    st
}

/// The station's cover: red enamel over die-cast, rubbed to the metal at its
/// edges and round the lever's well where hands go, grimy, a few chips.
/// (Laid on its front by the part's box projection: the whole texture is the face.)
fn alarm_paint_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 170, false, |ctx, w, h| {
                ctx.set_fill_style("#8a1c14");
                ctx.fill_rect(0, 0, w, h);
                // the enamel's mottle
                for _ in 0..400 {
                    ctx.set_fill_style(&format!("rgba({},0,0,{})", if rnd() < 0.5 { 40 } else { 200 }, 0.03 + rnd() * 0.05));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
                }
                // rubbed thin at the edges, chipped to grey metal at the corners
                ctx.set_stroke_style("rgba(150,120,110,0.35)");
                ctx.set_line_width(3);
                ctx.stroke_rect(1.5, 1.5, w - 3.0, h - 3.0);
                for &(x, y) in &[(3.0, 3.0), (w - 4.0, 3.0), (3.0, h - 4.0), (w - 4.0, h - 4.0), (w * 0.5, h - 3.0), (w - 3.0, h * 0.6)] {
                    ctx.set_fill_style("rgba(120,116,110,0.85)");
                    ctx.begin_path();
                    ctx.ellipse(x, y, 2.0 + rnd() * 4.0, 1.5 + rnd() * 3.0, rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
                // hands: grime round the lever's well and below it
                let gr = ctx.create_radial_gradient(w * 0.5, h * 0.52, 8, w * 0.5, h * 0.55, w * 0.62);
                gr.add_color_stop(0, "rgba(30,20,16,0.30)");
                gr.add_color_stop(1, "rgba(30,20,16,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
                // dust on its top edge, the grime of years low down
                let gd = ctx.create_linear_gradient(0, 0, 0, h);
                gd.add_color_stop(0, "rgba(160,150,130,0.22)");
                gd.add_color_stop(0.08, "rgba(160,150,130,0)");
                gd.add_color_stop(0.8, "rgba(40,30,24,0)");
                gd.add_color_stop(1, "rgba(40,30,24,0.25)");
                ctx.set_fill_style(gd);
                ctx.fill_rect(0, 0, w, h);
                // scratches
                ctx.set_line_width(0.8);
                for _ in 0..14 {
                    let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 6.0 + rnd() * 20.0);
                    ctx.set_stroke_style(&format!("rgba(170,140,130,{})", 0.2 + rnd() * 0.3));
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            })
        })
        .clone()
    })
}

// ------------------------------------------------------------ the coat hooks

/// Old oak, its varnish darkened: straight grain along the rail, the rays.
fn rail_wood_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(512, 64, |ctx, w, h| {
                ctx.set_fill_style("#6a4e33");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..70 {
                    let y = rnd() * h;
                    let a = 0.08 + rnd() * 0.18;
                    ctx.set_stroke_style(&format!("rgba(40,26,14,{a})"));
                    ctx.set_line_width(0.6 + rnd() * 1.6);
                    ctx.begin_path();
                    ctx.move_to(0, y);
                    let mut yy = y;
                    let mut x = 0.0;
                    while x < w {
                        x += 16.0;
                        yy += (rnd() - 0.5) * 0.9;
                        ctx.line_to(x, yy);
                    }
                    ctx.stroke();
                }
                // the rays: short pale flecks across the grain
                for _ in 0..60 {
                    ctx.set_fill_style(&format!("rgba(170,130,90,{})", 0.08 + rnd() * 0.12));
                    ctx.fill_rect(rnd() * w, rnd() * h, 3.0 + rnd() * 8.0, 1.0);
                }
                // hands' grime along the middle
                let gr = ctx.create_linear_gradient(0, 0, 0, h);
                gr.add_color_stop(0, "rgba(20,14,8,0.15)");
                gr.add_color_stop(0.5, "rgba(20,14,8,0)");
                gr.add_color_stop(1, "rgba(20,14,8,0.2)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            })
        })
        .clone()
    })
}

/// Wool cloth: its fine twill, lint and the dust settled on it (grey, laid
/// over the coat's own colour).
fn wool_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(256, 256, |ctx, w, h| {
                ctx.set_fill_style("#f2f2f2");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_line_width(1);
                let mut d = -h;
                while d < w {
                    ctx.set_stroke_style(&format!("rgba(0,0,0,{})", 0.06 + rnd() * 0.06));
                    ctx.begin_path();
                    ctx.move_to(d, 0);
                    ctx.line_to(d + h, h);
                    ctx.stroke();
                    d += 3.0;
                }
                for _ in 0..500 {
                    let v = if rnd() < 0.5 { "255,255,255" } else { "0,0,0" };
                    ctx.set_fill_style(&format!("rgba({v},{})", 0.05 + rnd() * 0.1));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
                }
                for _ in 0..30 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 6.0 + rnd() * 20.0;
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0, "rgba(255,250,235,0.10)");
                    gr.add_color_stop(1, "rgba(255,250,235,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
            })
        })
        .clone()
    })
}

// ------------------------------------------------------------ the posters

/// How many posters there are to choose from.
const POSTER_DESIGNS: usize = 6;

/// Each design's sheet (0 A2, 1 tabloid, 2 letter).
const POSTER_SHEET: [usize; POSTER_DESIGNS] = [0, 0, 1, 1, 2, 2];

/// Sheet sizes (m), the corner each part has curled and the corner torn
/// away under its tape (0 bottom left, 1 bottom right, 2 top right, 3 top left).
const SHEETS: [(&str, f64, f64, usize, usize); 3] =
    [("Real__posterA2", 0.420, 0.594, 2, 0), ("Real__posterTabloid", 0.279, 0.432, 0, 2), ("Real__posterLetter", 0.216, 0.279, 1, 3)];

/// A poster on the wall: its group's origin on the wall's face at the
/// sheet's middle, +z out. `framed`: in a clip frame, flat behind its glass
/// (broken), the clips at its edges.
fn build_poster(design: usize, faded: bool, tint: u32, framed: bool) -> Obj {
    let pg_ = group();
    let (part, w, h, curl, _) = SHEETS[POSTER_SHEET[design]];
    let paper = lambert!({ map: &poster_tex(design, faded, framed), color: tint, alphaTest: 0.5 });
    let corners = [(-w / 2.0, -h / 2.0), (w / 2.0, -h / 2.0), (w / 2.0, h / 2.0), (-w / 2.0, h / 2.0)];
    if framed {
        let back = mesh(box_geo!(w, h, 0.003), lambert!({ color: 0x4a3a2a }));
        back.position().z = 0.0015;
        pg_.add(&back);
        let sheet = mesh(plane_geo!(w, h), &paper);
        sheet.position().z = 0.0032;
        pg_.add(&sheet);
        let glass = mesh(plane_geo!(w, h), lambert!({ map: &cracked_pane_tex(), transparent: true, depthWrite: false }));
        glass.position().z = 0.0046;
        pg_.add(&glass);
        let clip_mat = phong!({ color: 0x8a8a86, specular: 0x333333, shininess: 40 });
        for (cx, cy, rz) in [(0.0, h / 2.0, 0.0), (0.0, -h / 2.0, PI), (-w / 2.0, h * 0.2, PI / 2.0), (w / 2.0, -h * 0.2, -PI / 2.0)] {
            let cl = mesh(real_part("Real__posterClip", 0.016, 0.02, 0.012), &clip_mat);
            cl.rotation().z = rz;
            cl.position().set(cx, cy, 0.0);
            pg_.add(&cl);
        }
        return pg_;
    }
    let sheet = mesh(real_part(part, w, h, 0.002), &paper);
    pg_.add(&sheet);
    let back = mesh(real_part(&format!("{part}Back"), w, h, 0.002), lambert!({ color: 0xb5ad98 }));
    pg_.add(&back);
    // masking tape at the corners: over the paper where it holds, on the
    // wall alone where the corner curled away from it
    let tape = lambert!({ color: 0x9c8762 });
    for (k, &(cx, cy)) in corners.iter().enumerate() {
        let t = mesh(real_part("Real__posterTape", 0.048, 0.019, 0.0005), &tape);
        let (ix, iy) = (-cx.signum() * 0.008, -cy.signum() * 0.008);
        t.position().set(cx + ix, cy + iy, if k == curl { 0.0004 } else { 0.0022 });
        t.rotation().z = if (k % 2 == 0) == (cx * cy > 0.0) { PI / 4.0 } else { -PI / 4.0 };
        pg_.add(&t);
    }
    pg_
}

/// What a poster says, as real ones are set: title, body, small print, and
/// its wear (sun, damp, foxing, a fold, hands at its foot); the torn corner
/// and torn-off tabs cut away (alpha). Nothing about the place, no name.
fn poster_tex(design: usize, faded: bool, framed: bool) -> Tex {
    thread_local! { static T: std::cell::RefCell<std::collections::BTreeMap<(usize, bool, bool), Tex>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) }; }
    if let Some(t) = T.with(|c| c.borrow().get(&(design, faded, framed)).cloned()) {
        return t;
    }
    let (_, sw, sh, _, torn) = SHEETS[POSTER_SHEET[design]];
    let ppm = 1200.0; // pixels a metre
    let (pw, ph) = ((sw * ppm).round(), (sh * ppm).round());
    let t = canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let s = w / 504.0; // a design's units: the A2 sheet is 504 wide
                           // the paper, yellowed
        ctx.set_fill_style(if faded { "#d9cfb2" } else { "#e2dcc7" });
        ctx.fill_rect(0, 0, w, h);
        let ink = |a: f64| format!("rgba(26,22,18,{a})");
        let fade = if faded { 0.55 } else { 0.8 };
        match design {
            0 => {
                // a safety poster: LIFT SAFELY, the figure squatting to the box, three rules
                ctx.set_fill_style(&format!("rgba(34,92,52,{fade})"));
                ctx.fill_rect(0, 0, w, 120.0 * s);
                ctx.set_fill_style("rgba(240,236,224,0.95)");
                ctx.set_text_align("center");
                ctx.set_font(&format!("bold {}px Helvetica", 30.0 * s));
                ctx.fill_text_max("SAFETY FIRST", w / 2.0, 44.0 * s, w - 40.0 * s);
                ctx.set_font(&format!("bold {}px Helvetica", 58.0 * s));
                ctx.fill_text_max("LIFT SAFELY", w / 2.0, 102.0 * s, w - 40.0 * s);
                // the figure: squatting, back straight, the box held close
                ctx.set_stroke_style(&ink(fade));
                ctx.set_fill_style(&ink(fade));
                ctx.set_line_cap("round");
                ctx.set_line_join("round");
                ctx.set_line_width(16.0 * s);
                let (fx, fy) = (w * 0.44, 300.0 * s);
                ctx.begin_path();
                ctx.arc(fx, fy - 108.0 * s, 20.0 * s, 0, 7);
                ctx.fill();
                ctx.begin_path(); // the back, upright
                ctx.move_to(fx, fy - 80.0 * s);
                ctx.line_to(fx - 6.0 * s, fy);
                ctx.stroke();
                ctx.begin_path(); // the legs, bent
                ctx.move_to(fx - 6.0 * s, fy);
                ctx.line_to(fx + 46.0 * s, fy + 14.0 * s);
                ctx.line_to(fx + 40.0 * s, fy + 74.0 * s);
                ctx.move_to(fx - 6.0 * s, fy);
                ctx.line_to(fx + 30.0 * s, fy + 30.0 * s);
                ctx.line_to(fx + 8.0 * s, fy + 76.0 * s);
                ctx.stroke();
                ctx.begin_path(); // the arms to the box
                ctx.move_to(fx - 2.0 * s, fy - 66.0 * s);
                ctx.line_to(fx + 40.0 * s, fy - 30.0 * s);
                ctx.line_to(fx + 62.0 * s, fy - 10.0 * s);
                ctx.stroke();
                ctx.fill_rect(fx + 58.0 * s, fy - 34.0 * s, 70.0 * s, 56.0 * s);
                ctx.set_line_width(5.0 * s);
                ctx.begin_path();
                ctx.move_to(40.0 * s, fy + 82.0 * s);
                ctx.line_to(w - 40.0 * s, fy + 82.0 * s);
                ctx.stroke();
                // the tick by it
                ctx.set_stroke_style(&format!("rgba(34,92,52,{fade})"));
                ctx.set_line_width(14.0 * s);
                ctx.begin_path();
                ctx.move_to(w * 0.80, fy - 70.0 * s);
                ctx.line_to(w * 0.84, fy - 40.0 * s);
                ctx.line_to(w * 0.92, fy - 110.0 * s);
                ctx.stroke();
                ctx.set_text_align("left");
                ctx.set_fill_style(&ink(fade + 0.1));
                let rules = [
                    ("1", "Bend your knees, not your back."),
                    ("2", "Keep the load close to your body."),
                    ("3", "Turn with your feet. Never twist."),
                ];
                for (k, (n, line)) in rules.iter().enumerate() {
                    let y = 470.0 * s + k as f64 * 62.0 * s;
                    ctx.set_font(&format!("bold {}px Helvetica", 34.0 * s));
                    ctx.fill_text(n, 40.0 * s, y);
                    ctx.set_font(&format!("{}px Helvetica", 25.0 * s));
                    ctx.fill_text_max(line, 82.0 * s, y - 4.0 * s, w - 120.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 21.0 * s));
                ctx.set_text_align("center");
                ctx.fill_text_max("Ask for help with anything over 50 lb.", w / 2.0, 680.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 12.0 * s));
                ctx.fill_text_max("Report every injury, however small, to your supervisor the same day.", w / 2.0, 702.0 * s, w - 60.0 * s);
            }
            1 => {
                // a dance's bill: two inks, a slab serif
                ctx.set_fill_style(&format!("rgba(150,40,30,{fade})"));
                ctx.fill_rect(24.0 * s, 24.0 * s, w - 48.0 * s, h - 48.0 * s);
                ctx.set_fill_style(if faded { "#d9cfb2" } else { "#e2dcc7" });
                ctx.fill_rect(36.0 * s, 36.0 * s, w - 72.0 * s, h - 72.0 * s);
                ctx.set_text_align("center");
                ctx.set_fill_style(&ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Rockwell", 40.0 * s));
                ctx.fill_text_max("SATURDAY NIGHT", w / 2.0, 120.0 * s, w - 100.0 * s);
                ctx.set_fill_style(&format!("rgba(150,40,30,{fade})"));
                ctx.set_font(&format!("bold {}px Rockwell", 128.0 * s));
                ctx.fill_text_max("DANCE", w / 2.0, 250.0 * s, w - 90.0 * s);
                ctx.set_fill_style(&ink(fade + 0.1));
                ctx.fill_rect(80.0 * s, 290.0 * s, w - 160.0 * s, 4.0 * s);
                ctx.set_font(&format!("bold {}px Rockwell", 44.0 * s));
                ctx.fill_text_max("LIVE MUSIC", w / 2.0, 360.0 * s, w - 100.0 * s);
                ctx.set_font(&format!("{}px Rockwell", 34.0 * s));
                ctx.fill_text_max("8 PM till midnight", w / 2.0, 420.0 * s, w - 100.0 * s);
                ctx.fill_rect(80.0 * s, 450.0 * s, w - 160.0 * s, 4.0 * s);
                ctx.set_font(&format!("{}px Rockwell", 28.0 * s));
                ctx.fill_text_max("Admission $5 at the door", w / 2.0, 510.0 * s, w - 100.0 * s);
                ctx.fill_text_max("Refreshments", w / 2.0, 550.0 * s, w - 100.0 * s);
                ctx.set_fill_style(&format!("rgba(150,40,30,{fade})"));
                ctx.set_font(&format!("bold {}px Rockwell", 30.0 * s));
                ctx.fill_text_max("EVERYONE WELCOME", w / 2.0, 640.0 * s, w - 100.0 * s);
            }
            2 => {
                // a bake sale, from an office printer
                ctx.set_text_align("center");
                ctx.set_fill_style(&ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Futura", 92.0 * s));
                ctx.fill_text_max("BAKE SALE", w / 2.0, 120.0 * s, w - 50.0 * s);
                ctx.set_font(&format!("bold {}px Futura", 64.0 * s));
                ctx.fill_text_max("FRIDAY", w / 2.0, 240.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 52.0 * s));
                ctx.fill_text_max("11:30 – 2:00", w / 2.0, 320.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 36.0 * s));
                ctx.fill_text_max("in the break room", w / 2.0, 380.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 30.0 * s));
                ctx.fill_text_max("Cookies · Brownies · Pie · Coffee", w / 2.0, 500.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("italic {}px Times New Roman", 28.0 * s));
                ctx.fill_text_max("All proceeds go to the holiday fund", w / 2.0, 640.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Times New Roman", 24.0 * s));
                ctx.fill_text_max("Thank you!", w / 2.0, 690.0 * s, w - 60.0 * s);
            }
            3 => {
                // a recycling notice
                ctx.set_fill_style(&format!("rgba(30,60,110,{fade})"));
                ctx.fill_rect(0, 0, w, 150.0 * s);
                ctx.set_text_align("center");
                ctx.set_fill_style("rgba(240,236,224,0.95)");
                ctx.set_font(&format!("bold {}px Helvetica", 96.0 * s));
                ctx.fill_text_max("PAPER ONLY", w / 2.0, 110.0 * s, w - 40.0 * s);
                ctx.set_text_align("left");
                ctx.set_fill_style(&ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Helvetica", 40.0 * s));
                ctx.fill_text("YES", 40.0 * s, 230.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 30.0 * s));
                for (k, l) in ["Office paper", "Envelopes and folders", "Newspapers, magazines", "Flattened boxes"].iter().enumerate() {
                    ctx.fill_text_max(l, 70.0 * s, 280.0 * s + k as f64 * 42.0 * s, w - 100.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 40.0 * s));
                ctx.fill_text("NO", 40.0 * s, 490.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 30.0 * s));
                for (k, l) in ["Food or drink", "Cups and lids", "Plastic wrap", "Batteries"].iter().enumerate() {
                    ctx.fill_text_max(l, 70.0 * s, 540.0 * s + k as f64 * 42.0 * s, w - 100.0 * s);
                }
                ctx.set_font(&format!("italic {}px Helvetica", 22.0 * s));
                ctx.set_text_align("center");
                ctx.fill_text_max("Thank you for keeping it clean.", w / 2.0, 735.0 * s, w - 60.0 * s);
            }
            _ => {
                // a flyer with tear-off tabs
                let (title, lines, tab): (&str, [&str; 3], &str) = if design == 4 {
                    (
                        "GUITAR LESSONS",
                        ["Beginners welcome. All ages.", "Acoustic and electric.", "$15 a half hour."],
                        "Guitar lessons  555-0186",
                    )
                } else {
                    (
                        "ROOM FOR RENT",
                        ["Quiet, furnished, by the bus line.", "$300 a month, utilities included.", "Non-smoker, please."],
                        "Room for rent  555-0142",
                    )
                };
                ctx.set_text_align("center");
                ctx.set_fill_style(&ink(fade + 0.15));
                ctx.set_font(&format!("bold {}px Helvetica", 70.0 * s));
                ctx.fill_text_max(title, w / 2.0, 110.0 * s, w - 40.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 40.0 * s));
                for (k, l) in lines.iter().enumerate() {
                    ctx.fill_text_max(l, w / 2.0, 210.0 * s + k as f64 * 60.0 * s, w - 40.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 44.0 * s));
                ctx.fill_text_max(if design == 4 { "Call 555-0186" } else { "Call 555-0142" }, w / 2.0, 420.0 * s, w - 40.0 * s);
                // the tabs: eight, cut apart, the number on each turned up the tab
                let ty = h - 150.0 * s;
                let n = 8;
                let tw = w / n as f64;
                ctx.set_stroke_style(&ink(0.5));
                ctx.set_line_width(1.0 * s);
                ctx.set_line_dash(&[6.0 * s, 4.0 * s]);
                ctx.begin_path();
                ctx.move_to(0, ty);
                ctx.line_to(w, ty);
                ctx.stroke();
                ctx.set_line_dash(&[]);
                for k in 0..n {
                    ctx.save();
                    ctx.translate((k as f64 + 0.5) * tw + 9.0 * s, h - 8.0 * s);
                    ctx.rotate(-PI / 2.0);
                    ctx.set_text_align("left");
                    ctx.set_font(&format!("{}px Helvetica", 19.0 * s));
                    ctx.fill_text_max(tab, 0, 0, 136.0 * s);
                    ctx.restore();
                }
                // three already taken: torn off along the line, a stub left
                ctx.set_global_composite_operation("destination-out");
                for k in [1usize, 4, 5] {
                    let x0 = k as f64 * tw;
                    ctx.begin_path();
                    ctx.move_to(x0 + 1.0, h);
                    ctx.line_to(x0 + 1.0, ty + 18.0 * s + rnd() * 20.0 * s);
                    let mut x = x0 + 1.0;
                    while x < x0 + tw - 1.0 {
                        x += 4.0 * s;
                        ctx.line_to(x.min(x0 + tw - 1.0), ty + 10.0 * s + rnd() * 26.0 * s);
                    }
                    ctx.line_to(x0 + tw - 1.0, h);
                    ctx.close_path();
                    ctx.fill();
                }
                ctx.set_global_composite_operation("source-over");
                // the slits between the tabs that are left
                ctx.set_global_composite_operation("destination-out");
                for k in 1..n {
                    ctx.fill_rect(k as f64 * tw - 1.0, ty, 2, h - ty);
                }
                ctx.set_global_composite_operation("source-over");
            }
        }
        // ---- its years on the wall
        // the sun: the ink and paper bleached toward the light
        if faded {
            ctx.set_global_composite_operation("source-atop");
            ctx.set_fill_style("rgba(226,216,190,0.22)");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_global_composite_operation("source-over");
        }
        ctx.set_global_composite_operation("source-atop");
        // foxing: small brown spots
        for _ in 0..(40.0 * w / 504.0) as i32 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 1.0 + rnd() * 4.0 * s;
            ctx.set_fill_style(&format!("rgba(120,80,40,{})", 0.10 + rnd() * 0.25));
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.7 + rnd() * 0.3), 0, 0, 7);
            ctx.fill();
        }
        // damp: a tide line across its foot
        let ty = h * (0.82 + rnd() * 0.08);
        let gr = ctx.create_linear_gradient(0, ty - 30.0 * s, 0, h);
        gr.add_color_stop(0, "rgba(140,110,60,0)");
        gr.add_color_stop(0.25, "rgba(140,110,60,0.18)");
        gr.add_color_stop(0.3, "rgba(110,84,44,0.25)");
        gr.add_color_stop(1, "rgba(140,110,60,0.12)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, ty - 30.0 * s, w, h);
        // the folds it was posted in (in half each way): a pale crease
        for (x0, y0, x1, y1) in [(w / 2.0, 0.0, w / 2.0, h), (0.0, h / 2.0, w, h / 2.0)] {
            ctx.set_stroke_style("rgba(255,250,235,0.20)");
            ctx.set_line_width(2.0 * s);
            ctx.begin_path();
            ctx.move_to(x0 + 1.0, y0);
            ctx.line_to(x1 + 1.0, y1);
            ctx.stroke();
            ctx.set_stroke_style("rgba(60,46,30,0.16)");
            ctx.set_line_width(1.2 * s);
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x1, y1);
            ctx.stroke();
        }
        // grime: hands at its foot, dust settled along its top edge
        for _ in 0..8 {
            let (x, y) = (rnd() * w, h * (0.75 + rnd() * 0.25));
            let r = (20.0 + rnd() * 40.0) * s;
            let gs = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gs.add_color_stop(0, "rgba(60,48,34,0.12)");
            gs.add_color_stop(1, "rgba(60,48,34,0)");
            ctx.set_fill_style(gs);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // the paper's edge darkened where it met the air
        for (x0, y0, x1, y1) in [(0.0, 0.0, 14.0, 0.0), (w, 0.0, w - 14.0, 0.0)] {
            let ge = ctx.create_linear_gradient(x0 * s.max(1.0), y0, x1 * s.max(1.0), y1);
            ge.add_color_stop(0, "rgba(120,96,60,0.25)");
            ge.add_color_stop(1, "rgba(120,96,60,0)");
            ctx.set_fill_style(ge);
            ctx.fill_rect(0, 0, w, h);
        }
        ctx.set_global_composite_operation("source-over");
        // the torn corner: gone but for the scrap still under its tape
        if !framed {
            let cx = if torn == 0 || torn == 3 { 0.0 } else { w };
            let cy = if torn == 0 || torn == 1 { h } else { 0.0 };
            let (dx, dy) = (if cx == 0.0 { 1.0 } else { -1.0 }, if cy == 0.0 { 1.0 } else { -1.0 });
            let ppm = w / sw;
            let (a, b) = (0.024 * ppm, 0.065 * ppm);
            ctx.set_global_composite_operation("destination-out");
            ctx.begin_path();
            // a jagged band from edge to edge, between the scrap and the sheet
            let steps = 18;
            let mut pts = vec![];
            for q in 0..=steps {
                let t = q as f64 / steps as f64;
                let r = b + (rnd() - 0.5) * 0.012 * ppm;
                pts.push((cx + dx * r * (1.0 - t), cy + dy * r * t));
            }
            for q in (0..=steps).rev() {
                let t = q as f64 / steps as f64;
                let r = a + (rnd() - 0.5) * 0.008 * ppm;
                pts.push((cx + dx * r * (1.0 - t), cy + dy * r * t));
            }
            for (q, &(x, y)) in pts.iter().enumerate() {
                if q == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.close_path();
            ctx.fill();
            ctx.set_global_composite_operation("source-over");
        }
    });
    T.with(|c| c.borrow_mut().insert((design, faded, framed), t.clone()));
    t
}

/// A clip frame's broken glass: the cracks from where it was struck, rings
/// round it, a few panes of it dirtier (transparent but for the cracks).
fn cracked_pane_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 360, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                // dust on the glass
                ctx.set_fill_style("rgba(200,196,180,0.10)");
                ctx.fill_rect(0, 0, w, h);
                let (cx, cy) = (w * 0.62, h * 0.38);
                let mut ends = vec![];
                for k in 0..11 {
                    let mut a = k as f64 / 11.0 * 6.283 + rnd() * 0.4;
                    let (mut x, mut y) = (cx, cy);
                    ctx.set_stroke_style("rgba(235,240,236,0.55)");
                    ctx.set_line_width(1.0);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    let steps = 10 + (rnd() * 10.0) as i32;
                    for _ in 0..steps {
                        a += (rnd() - 0.5) * 0.35;
                        x += a.cos() * 14.0;
                        y += a.sin() * 14.0;
                        ctx.line_to(x, y);
                    }
                    ctx.stroke();
                    ends.push(a);
                }
                for r in [18.0, 44.0, 80.0] {
                    ctx.set_stroke_style("rgba(235,240,236,0.4)");
                    ctx.begin_path();
                    let n = 11;
                    for k in 0..=n {
                        let a = k as f64 / n as f64 * 6.283;
                        let rr = r * (0.8 + rnd() * 0.4);
                        if k == 0 {
                            ctx.move_to(cx + a.cos() * rr, cy + a.sin() * rr);
                        } else {
                            ctx.line_to(cx + a.cos() * rr, cy + a.sin() * rr);
                        }
                    }
                    ctx.stroke();
                }
                // the impact: crushed white
                let gr = ctx.create_radial_gradient(cx, cy, 0, cx, cy, 10);
                gr.add_color_stop(0, "rgba(240,240,236,0.7)");
                gr.add_color_stop(1, "rgba(240,240,236,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(cx - 10.0, cy - 10.0, 20, 20);
            })
        })
        .clone()
    })
}
