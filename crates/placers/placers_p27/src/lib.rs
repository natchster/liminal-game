//! Prop placers: wall-mounted devices (flashlights, barometers, watches, digital
//! clocks, no-smoking signs, gauges, calendars, thermostats), typewriters, alarm
//! clocks and clock radios, ceramics, trophies, leaning and broken window
//! frames, brooms, ladders, handbells and wooden crates.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_wall_flashlights", place_wall_flashlights));
    r.push(("place_typewriters", place_typewriters));
    r.push(("place_alarm_clocks", place_alarm_clocks));
    r.push(("place_clock_radios", place_clock_radios));
    r.push(("place_wall_barometers", place_wall_barometers));
    r.push(("place_wall_watches", place_wall_watches));
    r.push(("place_digital_wall_clocks", place_digital_wall_clocks));
    r.push(("place_no_smoking_signs", place_no_smoking_signs));
    r.push(("place_ceramic_bowls", place_ceramic_bowls));
    r.push(("place_ceramic_mugs", place_ceramic_mugs));
    r.push(("place_small_trophies", place_small_trophies));
    r.push(("place_wall_gauges", place_wall_gauges));
    r.push(("place_wall_calendars", place_wall_calendars));
    r.push(("place_wall_thermostats", place_wall_thermostats));
    r.push(("place_leaning_frames", place_leaning_frames));
    r.push(("place_broken_window_frames", place_broken_window_frames));
    r.push(("place_wooden_brooms", place_wooden_brooms));
    r.push(("place_small_ladders", place_small_ladders));
    r.push(("place_small_handbells", place_small_handbells));
    r.push(("place_wooden_crates", place_wooden_crates));
}

// ------------------------------------------------------------------ helpers

/// `{ x, z, dx, dz }`: an open tile with a wall (or the grid edge) on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The common wall-candidate scan: every open interior tile, once per side
/// that is a wall tile or out of the grid, in z, x, [[1,0],[-1,0],[0,1],[0,-1]] order.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut c = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    c.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    c
}

/// Every open interior tile (`tiles[t_idx(x, z)] == 0`), z-major.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// A deep clone whose Euler angles are re-derived from each node's quaternion
/// (a yaw past π/2 comes back as (±π, π - θ, ±π)), so later
/// `clone.rotation().y = v` writes act on those angles (kept on purpose).
/// `Obj::duplicate` keeps the Euler as it was, hence this wrapper.
fn clone_obj(o: &Obj) -> Obj {
    let c = o.duplicate();
    c.traverse(&mut |n| {
        let q = n.quat();
        *n.quaternion() = q;
    });
    c
}

/// `floor(`rnd()` * len)`
#[inline]
fn pick(len: usize) -> usize {
    (rnd() * len as f64).floor() as usize
}

/// The shared wall-mount branch (rotation from the wall side, one random
/// draw for the offset along the wall).
fn wall_mount(o: &Obj, c: &WallCand, wall_x: f64, wall_z: f64, y: f64, spread: f64) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, y, wall_z + (rnd() - 0.5) * spread);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * spread, y, wall_z);
    }
}

/// A one-triangle geometry: f32 positions `v`, `set_index(idx)`,
/// `compute_vertex_normals()`.
fn tri_geo(v: [f64; 9], idx: [u32; 3]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

// R407 wall emergency flashlight: 12% chance per level. Small dark
// bracket + flashlight cylinder held horizontally on wall.
fn place_wall_flashlights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let bracket_mat = lambert!({ color: 0x2a2418 });
    let flash_mat = lambert!({ color: 0x181820 });
    let lens_glass = lambert!({ color: 0xe8e8c0 });
    let fl = group();
    // Bracket: small dark box against wall.
    let bracket = mesh(pg("WallFlashlights__bracket", box_geo!(0.06, 0.06, 0.015)), &bracket_mat);
    bracket.position().z = 0.008;
    fl.add(&bracket);
    // Flashlight body: horizontal cylinder held by bracket.
    let fl_body = mesh(pg("WallFlashlights__barrel", cyl_geo!(0.025, 0.025, 0.16, 8)), &flash_mat);
    fl_body.rotation().z = PI / 2.0;
    fl_body.position().z = 0.040;
    fl.add(&fl_body);
    // Head: slightly wider cylinder at front tip.
    let fl_head = mesh(pg("WallFlashlights__head", cyl_geo!(0.030, 0.028, 0.03, 8)), &flash_mat);
    fl_head.rotation().z = PI / 2.0;
    fl_head.position().set(0.088, 0, 0.040);
    fl.add(&fl_head);
    // Lens glass at tip.
    let fl_lens = mesh(circle_geo!(0.025, 10), &lens_glass);
    fl_lens.rotation().y = PI / 2.0;
    fl_lens.position().set(0.104, 0, 0.040);
    fl.add(&fl_lens);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let fl_height = 1.6 + rnd() * 0.15;
    wall_mount(&fl, &c, wall_x, wall_z, fl_height, 1.0);
    g.world.add(&fl);
}

// R405 vintage typewriter: 10% chance per level. Box + slanted key
// area + paper roller + platen. Very institutional/office vintage.
fn place_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let body_mat = lambert!({ color: 0x282020 });
    let key_mat = lambert!({ color: 0x4a3a30 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let platen_mat = lambert!({ color: 0x1a1410 });
    let tw = group();
    // Body: main block.
    let body = mesh(pg("Typewriters__body", box_geo!(0.28, 0.10, 0.22)), &body_mat);
    body.position().y = 0.05;
    tw.add(&body);
    // Key area: slanted rectangle on front-top.
    let key_area = mesh(pg("Typewriters__keys", box_geo!(0.22, 0.03, 0.10)), &key_mat);
    key_area.rotation().x = -0.15;
    key_area.position().set(0, 0.10, 0.06);
    tw.add(&key_area);
    // Platen: horizontal cylinder across top-back for paper roller.
    let platen = mesh(pg("Typewriters__platen", cyl_geo!(0.020, 0.020, 0.24, 10)), &platen_mat);
    platen.rotation().z = PI / 2.0;
    platen.position().set(0, 0.12, -0.06);
    tw.add(&platen);
    // Paper sheet: vertical thin rectangle poking up from platen.
    let paper = mesh(pg("Typewriters__paper", plane_geo!(0.18, 0.14)), &paper_mat);
    paper.position().set(0, 0.19, -0.06);
    paper.rotation().x = -0.10;
    tw.add(&paper);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
    // R406 30% chance: 3-4 typed paper sheets scattered on floor.
    if rnd() < 0.30 {
        let psheet_count = 3 + (rnd() * 2.0).floor() as i32;
        for _ps in 0..psheet_count {
            let psheet = mesh(pg("Typewriters__sheet", plane_geo!(0.18, 0.24)), &paper_mat);
            psheet.rotation().x = -PI / 2.0;
            psheet.rotation().z = rnd() * PI * 2.0;
            let p_ang = rnd() * PI * 2.0;
            let p_rad = 0.25 + rnd() * 0.35;
            let tp = tw.pos();
            psheet.position().set(tp.x + p_ang.cos() * p_rad, 0.004, tp.z + p_ang.sin() * p_rad);
            g.world.add(&psheet);
        }
    }
}

// R405 twin-bell alarm clock: 12% chance per level. Round face +
// legs + 2 dome bells on top. Classic bedside clock silhouette.
fn place_alarm_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.alarm_clock_positions = vec![]; // R406 reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let face_mat = lambert!({ color: 0xd8d0c0 });
    let shell_mat = lambert!({ color: 0x707068 });
    let hand_mat = basic_mat!({ color: 0x1a1408 });
    let clock = group();
    // Round shell body.
    let shell = mesh(pg("AlarmClocks__shell", cyl_geo!(0.06, 0.06, 0.05, 12)), &shell_mat);
    shell.rotation().x = PI / 2.0;
    shell.position().y = 0.07;
    clock.add(&shell);
    // Face disc on front.
    let face = mesh(circle_geo!(0.052, 16), &face_mat);
    face.position().set(0, 0.07, 0.026);
    clock.add(&face);
    // 2 hands on face.
    let h_hand = mesh(plane_geo!(0.003, 0.028), &hand_mat);
    h_hand.position().set(0, 0.07, 0.028);
    h_hand.rotation().z = 0.6;
    clock.add(&h_hand);
    let m_hand = mesh(plane_geo!(0.002, 0.040), &hand_mat);
    m_hand.position().set(0, 0.07, 0.029);
    m_hand.rotation().z = -0.9;
    clock.add(&m_hand);
    // 2 dome bells on top.
    for b_off in [-0.035, 0.035] {
        let bell = mesh(pg("AlarmClocks__bell", sphere_geo!(0.025, 8, 6, 0, PI * 2.0, 0, PI / 2.0)), &shell_mat);
        bell.position().set(b_off, 0.13, 0);
        clock.add(&bell);
    }
    // 2 legs (small blocks under body).
    for l_off in [-0.035, 0.035] {
        let leg = mesh(pg("AlarmClocks__leg", box_geo!(0.015, 0.02, 0.02)), &shell_mat);
        leg.position().set(l_off, 0.01, 0);
        clock.add(&leg);
    }
    clock.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    clock.rotation().y = rnd() * PI * 2.0;
    g.world.add(&clock);
    // R406 track alarm clock position for rare ring audio events.
    let cp = clock.pos();
    g.pp.alarm_clock_positions.push(v3(cp.x, 0.08, cp.z));
}

// R404 small clock radio: 12% chance per level. Block with tuning
// dial + LED digital display. Classic bedside device.
fn place_clock_radios(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let case_mat = lambert!({ color: 0x2a2418 });
    let dial_mat = lambert!({ color: 0xd8d0c0 });
    let led_mat = basic_mat!({ color: 0xe02818 });
    let knob_mat = lambert!({ color: 0x8a7828 });
    let radio = group();
    // Case: box.
    let body = mesh(pg("ClockRadios__case", box_geo!(0.20, 0.08, 0.12)), &case_mat);
    body.position().y = 0.04;
    radio.add(&body);
    // Tuning dial: cream horizontal rectangle on front.
    let dial = mesh(plane_geo!(0.12, 0.03), &dial_mat);
    dial.position().set(-0.03, 0.055, 0.061);
    radio.add(&dial);
    // Small LED digital display on right side of front.
    let led = mesh(plane_geo!(0.04, 0.025), &led_mat);
    led.position().set(0.07, 0.055, 0.061);
    radio.add(&led);
    // Small knob (right side of case).
    let knob = mesh(pg("ClockRadios__knob", cyl_geo!(0.012, 0.012, 0.006, 6)), &knob_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0.09, 0.025, 0.061);
    radio.add(&knob);
    radio.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    radio.rotation().y = rnd() * PI * 2.0;
    g.world.add(&radio);
}

// R404 wall barometer: 10% chance per level. Tall dark-wood case
// with dial face + 2 hanging weights below.
fn place_wall_barometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let case_mat = lambert!({ color: 0x4a2818 });
    let dial_face_mat = lambert!({ color: 0xd8d0c0 });
    let weight_mat = lambert!({ color: 0xa08040 });
    let barometer = group();
    // Case: tall thin dark-wood rectangle.
    let case_b = mesh(pg("WallBarometers__case", box_geo!(0.18, 0.60, 0.05)), &case_mat);
    case_b.position().y = 0.0;
    barometer.add(&case_b);
    // Dial face: cream circle near top.
    let dial_f = mesh(circle_geo!(0.070, 16), &dial_face_mat);
    dial_f.position().set(0, 0.20, 0.026);
    barometer.add(&dial_f);
    // Small dark ring around dial.
    let dial_ring = mesh(pg("WallBarometers__ring", torus_geo!(0.070, 0.003, 4, 16)), &case_mat);
    dial_ring.rotation().x = 0.0;
    dial_ring.position().set(0, 0.20, 0.028);
    barometer.add(&dial_ring);
    // Tiny needle inside dial.
    let needle_mat = basic_mat!({ color: 0x8a2020 });
    let needle = mesh(plane_geo!(0.003, 0.055), &needle_mat);
    needle.position().set(0.010, 0.213, 0.030);
    needle.rotation().z = 0.5;
    barometer.add(&needle);
    // 2 hanging brass weights (small cylinders) below dial.
    for w_off in [-0.04, 0.04] {
        let w_cord = mesh(pg("WallBarometers__cord", cyl_geo!(0.002, 0.002, 0.28, 4)), &case_mat);
        w_cord.position().set(w_off, -0.08, 0.026);
        barometer.add(&w_cord);
        let weight = mesh(pg("WallBarometers__weight", cyl_geo!(0.013, 0.013, 0.05, 8)), &weight_mat);
        weight.position().set(w_off, -0.24, 0.026);
        barometer.add(&weight);
    }
    // Case back (0.05 deep) flush on the wall; it was 5 mm off it.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.025);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.025);
    let bar_height = 1.5 + rnd() * 0.15;
    wall_mount(&barometer, &c, wall_x, wall_z, bar_height, 0.8);
    g.world.add(&barometer);
}

// R402 small wall watch: 12% chance per level. Tiny 8-cm circle
// on wall — differs from R253 20-cm analog clock and R400 digital.
fn place_wall_watches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let watch_mat = lambert!({ map: wall_watch_texture(), transparent: true, side: DoubleSide });
    let watch = mesh(circle_geo!(0.04, 16), &watch_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let watch_height = 1.6 + rnd() * 0.30;
    wall_mount(&watch, &c, wall_x, wall_z, watch_height, 1.0);
    g.world.add(&watch);
    // R403 10% chance: cracked-glass overlay atop watch face.
    if rnd() < 0.10 {
        let wc_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let wcrack = mesh(plane_geo!(0.08, 0.08), &wc_mat);
        let wp = watch.pos();
        let wr = watch.rot();
        wcrack.position().copy(&wp);
        wcrack.rotation().copy(&wr);
        wcrack.position().x -= c.dx as f64 * 0.001;
        wcrack.position().z -= c.dz as f64 * 0.001;
        g.world.add(&wcrack);
    }
}

// R400 digital wall clock: 15% chance per level. Small black
// rectangle with red LED time display. Distinct from R253 analog
// wall clock.
fn place_digital_wall_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.digital_clock_positions = vec![]; // R402 reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    // Use a basic (unlit) material so the LED display stays lit regardless of scene light.
    let dc_mat = basic_mat!({ map: digital_clock_texture(), side: DoubleSide });
    let dc = mesh(plane_geo!(0.20, 0.10), &dc_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let dc_height = 1.9 + rnd() * 0.30;
    wall_mount(&dc, &c, wall_x, wall_z, dc_height, 1.0);
    dc.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&dc);
    // R402 track digital clock position for periodic mains-hum audio.
    let dp = dc.pos();
    g.pp.digital_clock_positions.push(v3(dp.x, dc_height, dp.z));
    // R401 40% chance: brief LED flicker animator (opacity/tint dip
    // simulating glitching digital display).
    if rnd() < 0.40 {
        // Convert to transparent material for opacity control.
        dc_mat.m().transparent = true;
        dc_mat.m().opacity = 1.0;
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let m = dc_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                m.m().opacity = if dip_t > 0.0 { 0.25 } else { 1.0 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.05;
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
    // R403 25% chance: 1-2 sibling digital clocks 0.28-0.36 m in row.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        for s in 1..=sib_count {
            let sib_clock = clone_obj(&dc);
            let sib_off = sib_dir * (0.28 + rnd() * 0.08) * s as f64;
            if c.dx.abs() == 1 {
                sib_clock.position().z += sib_off;
            } else {
                sib_clock.position().x += sib_off;
            }
            g.world.add(&sib_clock);
            let sp = sib_clock.pos();
            g.pp.digital_clock_positions.push(v3(sp.x, dc_height, sp.z));
        }
    }
    // R401 50% chance: small power cord from clock bottom running
    // down to floor. Reads as "clock is plugged into unseen socket".
    if rnd() < 0.50 {
        let cord_mat = lambert!({ color: 0x181410 });
        let cord_len = dc_height - 0.05; // reach to just above floor
        let cord = mesh(pg("DigitalWallClocks__cord", cyl_geo!(0.004, 0.004, cord_len, 4)), &cord_mat);
        let dp = dc.pos();
        cord.position().set(dp.x, dc_height - 0.05 - cord_len / 2.0, dp.z);
        // Offset cord slightly out from wall.
        cord.position().x -= c.dx as f64 * 0.005;
        cord.position().z -= c.dz as f64 * 0.005;
        g.world.add(&cord);
    }
}

// R395 wall no-smoking sign: 15% chance per level. Small round sign
// with pictogram + red slash. Institutional signage.
fn place_no_smoking_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let sign_mat = lambert!({ map: no_smoking_texture(), transparent: true, side: DoubleSide });
    let sign = mesh(circle_geo!(0.09, 20), &sign_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let sign_height = 1.65 + rnd() * 0.20;
    wall_mount(&sign, &c, wall_x, wall_z, sign_height, 1.0);
    sign.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&sign);
}

// R398 small ceramic bowl: 10% chance per level. Shallow wide
// cylinder with dark interior disc.
fn place_ceramic_bowls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let bowl_colors = [0xf0f0e8, 0x304868, 0x684030, 0x707078, 0xd0a850];
    let bowl_mat = lambert!({ color: bowl_colors[pick(bowl_colors.len())] });
    let inner_mat = lambert!({ color: 0x0a0806 });
    let bowl = group();
    // Body: shallow wide cylinder.
    let body = mesh(pg("CeramicBowls__bowl", cyl_geo!(0.06, 0.045, 0.04, 12)), &bowl_mat);
    body.position().y = 0.020;
    bowl.add(&body);
    // R399 40% chance liquid disc replaces empty interior.
    if rnd() < 0.40 {
        let b_liq_colors = [0x8a5030, 0xa06840, 0x605a30, 0x3a2818]; // soup/tea/water/broth
        let b_liq_mat = lambert!({ color: b_liq_colors[pick(b_liq_colors.len())] });
        let b_liq = mesh(circle_geo!(0.056, 12), &b_liq_mat);
        b_liq.rotation().x = -PI / 2.0;
        b_liq.position().y = 0.033;
        bowl.add(&b_liq);
    } else {
        let inner = mesh(circle_geo!(0.056, 12), &inner_mat);
        inner.rotation().x = -PI / 2.0;
        inner.position().y = 0.038;
        bowl.add(&inner);
    }
    bowl.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    bowl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bowl);
}

// R396 ceramic mug: 12% chance per level. Straight-walled cylinder
// mug + curved handle.
fn place_ceramic_mugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let mug_colors = [0xf0f0e8, 0x304868, 0x684030, 0x505050, 0xd0a850];
    let mug_mat = lambert!({ color: mug_colors[pick(mug_colors.len())] });
    let inside_mat = lambert!({ color: 0x0a0806 });
    let mug = group();
    // R399 25% chance: sit mug on a matching saucer plate.
    let has_saucer = rnd() < 0.25;
    let mut body_y = 0.045;
    if has_saucer {
        let saucer = mesh(pg("CeramicMugs__saucer", cyl_geo!(0.06, 0.055, 0.008, 12)), &mug_mat);
        saucer.position().y = 0.004;
        mug.add(&saucer);
        body_y += 0.008; // lift mug atop saucer
    }
    // Body: straight cylinder.
    let body = mesh(pg("CeramicMugs__body", cyl_geo!(0.036, 0.036, 0.09, 12)), &mug_mat);
    body.position().y = body_y;
    mug.add(&body);
    // R397 40% chance liquid disc replaces dark empty interior.
    let has_liquid = rnd() < 0.40;
    if has_liquid {
        let liquid_colors = [0x2a1810, 0x4a3018, 0x1a0a08, 0x8a6030]; // coffee/tea/dark/light-tea
        let liquid_mat = lambert!({ color: liquid_colors[pick(liquid_colors.len())] });
        let liquid = mesh(circle_geo!(0.032, 10), &liquid_mat);
        liquid.rotation().x = -PI / 2.0;
        liquid.position().y = body_y + 0.035; // slightly below rim
        mug.add(&liquid);
    } else {
        let inside = mesh(circle_geo!(0.032, 10), &inside_mat);
        inside.rotation().x = -PI / 2.0;
        inside.position().y = body_y + 0.042;
        mug.add(&inside);
    }
    // Handle: half-torus attached to side.
    let handle = mesh(pg("CeramicMugs__handle", torus_geo!(0.020, 0.005, 4, 8, PI)), &mug_mat);
    handle.rotation().z = -PI / 2.0; // arc bulges out of the mug, not into it
    handle.position().set(0.036, body_y, 0);
    mug.add(&handle);
    mug.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mug.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mug);
    // R398 25% chance: small tea/coffee stain on floor nearby.
    if rnd() < 0.25 {
        let stain_colors = [0x2a1810, 0x4a3018, 0x1a0a08, 0x5a3818];
        let stain_mat = basic_mat!({ color: stain_colors[pick(stain_colors.len())], transparent: true, opacity: 0.55, depthWrite: false });
        let tea_stain = mesh(circle_geo!(0.04 + rnd() * 0.03, 10), &stain_mat);
        tea_stain.rotation().x = -PI / 2.0;
        tea_stain.rotation().z = rnd() * PI * 2.0;
        tea_stain.scale().set(1.3, 0.85, 1);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.10 + rnd() * 0.12;
        let mp = mug.pos();
        tea_stain.position().set(mp.x + s_ang.cos() * s_rad, 0.004, mp.z + s_ang.sin() * s_rad);
        g.world.add(&tea_stain);
    }
    // R397 20% chance: 1-2 sibling mugs nearby (break room scene).
    if rnd() < 0.20 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _s in 0..sib_count {
            let sib_mug = clone_obj(&mug);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.15 + rnd() * 0.20;
            sib_mug.position().x += s_ang.cos() * s_rad;
            sib_mug.position().z += s_ang.sin() * s_rad;
            sib_mug.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_mug);
        }
    }
}

// R395 small trophy: 10% chance per level. Gold cup + base on floor.
fn place_small_trophies(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let gold_mat = lambert!({ color: 0xc8a028 });
    let base_mat = lambert!({ color: 0x3a2018 });
    let plaque_mat = lambert!({ color: 0xc8a028 });
    let trophy = group();
    // Base: dark wooden square.
    let base = mesh(pg("SmallTrophies__base", box_geo!(0.10, 0.03, 0.10)), &base_mat);
    base.position().y = 0.015;
    trophy.add(&base);
    // R400 small gold plaque on front face with tiny dark engraving lines.
    let plaque = mesh(pg("SmallTrophies__plaque", plane_geo!(0.07, 0.020)), &plaque_mat);
    plaque.position().set(0, 0.020, 0.051);
    trophy.add(&plaque);
    // Tiny dark engraving lines on plaque.
    let engrave_mat = basic_mat!({ color: 0x2a2018, transparent: true, opacity: 0.85, depthWrite: false });
    for el in 0..3 {
        let e_line = mesh(plane_geo!(0.04, 0.002), &engrave_mat);
        e_line.position().set((rnd() - 0.5) * 0.010, 0.015 + el as f64 * 0.005, 0.0515);
        trophy.add(&e_line);
    }
    // Stem: narrow gold cylinder.
    let stem = mesh(pg("SmallTrophies__stem", cyl_geo!(0.015, 0.015, 0.10, 6)), &gold_mat);
    stem.position().y = 0.08;
    trophy.add(&stem);
    // Cup: inverted cone gold.
    let cup = mesh(pg("SmallTrophies__cup", cyl_geo!(0.05, 0.025, 0.06, 8)), &gold_mat);
    cup.position().y = 0.16;
    trophy.add(&cup);
    // 2 handles: small torus sections on cup sides.
    for h_off in [-0.06f64, 0.06] {
        let handle = mesh(pg("SmallTrophies__handle", torus_geo!(0.020, 0.003, 4, 8, PI)), &gold_mat);
        // A C in the XZ-radial plane (XY): ends on the cup wall, bow outward.
        // The chord leans 0.395 rad with the cup's flare (r 0.025 -> 0.05).
        let hs = sign_of(h_off);
        handle.rotation().z = -hs * (PI / 2.0 + 0.395);
        handle.position().set(hs * 0.0365, 0.16, 0);
        trophy.add(&handle);
    }
    trophy.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trophy.rotation().y = rnd() * PI * 2.0;
    // R396 30% chance: trophy lies on side (Z-rotated 90°).
    if rnd() < 0.30 {
        trophy.rotation().z = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        trophy.position().y = 0.10; // roughly cup diameter above floor
    }
    g.world.add(&trophy);
}

// R392 wall analog gauge: 15% chance per level. Small circular
// gauge on wall — pressure/temperature reading dial.
fn place_wall_gauges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let g_mat = lambert!({ map: gauge_texture(), transparent: true, side: DoubleSide });
    let gauge = mesh(circle_geo!(0.10, 20), &g_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let gauge_height = 1.5 + rnd() * 0.30;
    wall_mount(&gauge, &c, wall_x, wall_z, gauge_height, 1.0);
    g.world.add(&gauge);
    // R394 30% chance: 1-2 sibling gauges 0.24-0.30 m apart in row.
    if rnd() < 0.30 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        for s in 1..=sib_count {
            let sib_gauge = clone_obj(&gauge);
            let sib_off = sib_dir * (0.24 + rnd() * 0.06) * s as f64;
            if c.dx.abs() == 1 {
                sib_gauge.position().z += sib_off;
            } else {
                sib_gauge.position().x += sib_off;
            }
            g.world.add(&sib_gauge);
        }
    }
}

// R390 wall calendar: 20% chance per level, 1 unit. Small
// paper-hanging calendar with visible date grid.
fn place_wall_calendars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let cal_mat = lambert!({ map: calendar_texture_2(), transparent: true, side: DoubleSide });
    let cal = mesh(plane_geo!(0.24, 0.32), &cal_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let cal_height = 1.6 + rnd() * 0.20;
    wall_mount(&cal, &c, wall_x, wall_z, cal_height, 1.0);
    // Slight crooked hang.
    cal.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&cal);
    // R391 30% chance: X marks across some day cells + torn bottom-
    // corner triangle. X marks read as "counting down days".
    if rnd() < 0.30 {
        // 4-6 X marks in random cells.
        let x_mat = basic_mat!({ color: 0x1a0a08, transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
        let x_count = 4 + (rnd() * 3.0).floor() as i32;
        let cell_w = 0.24 / 7.0;
        let cell_h = 0.32 / 5.0; // approx grid size
        for _xi in 0..x_count {
            let cell_col = (rnd() * 7.0).floor() - 3.0; // -3..3
            let cell_row = (rnd() * 5.0).floor() - 2.0; // -2..2
            let cell_x = cell_col * cell_w + cell_w * 0.5;
            let cell_y = -cell_row * cell_h - cell_h * 0.5;
            // 2 diagonal strokes = X.
            for stroke in 0..2 {
                let line = mesh(plane_geo!(0.020, 0.002), &x_mat);
                let cry = cal.rot().y;
                line.rotation().z = if stroke == 0 { PI / 4.0 } else { -PI / 4.0 };
                line.rotation().y = cry;
                // Local offset in calendar frame; nudge slightly outward.
                let cp = cal.pos();
                let world_x = cp.x + cry.cos() * 0.0 - cry.sin() * cell_x;
                let world_z = cp.z - cry.sin() * 0.0 - cry.cos() * cell_x;
                line.position().set(world_x - c.dx as f64 * 0.001, cp.y + cell_y, world_z - c.dz as f64 * 0.001);
                g.world.add(&line);
            }
        }
        // Torn bottom-right corner triangle.
        let torn_mat = basic_mat!({ color: 0x1a1610, transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let size = 0.05 + rnd() * 0.03;
        let torn_geo = tri_geo([0.12 - size, -0.16, 0.0, 0.12, -0.16, 0.0, 0.12, -0.16 + size, 0.0], [0, 2, 1]);
        let torn = mesh(torn_geo, &torn_mat);
        let cry = cal.rot().y;
        let cp = cal.pos();
        torn.rotation().y = cry;
        torn.position().copy(&cp);
        torn.position().x -= c.dx as f64 * 0.001;
        torn.position().z -= c.dz as f64 * 0.001;
        g.world.add(&torn);
    }
}

// R356 wall thermostat: 20% chance per level, 1 unit. Small rectangular
// beige control panel with tiny dial.
fn place_wall_thermostats(g: &mut Game, _dist: &[i32]) {
    g.pp.thermostat_positions = vec![]; // R357 reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let body_mat = lambert!({ color: 0xd8d0c0 });
    let dial_mat = lambert!({ color: 0x2a2418 });
    let readout_mat = basic_mat!({ color: 0x304028 });
    let thermo = group();
    // Body: 8×12 cm cream rectangle.
    let body = mesh(pg("WallThermostats__body", box_geo!(0.08, 0.12, 0.02)), &body_mat);
    thermo.add(&body);
    // Dial: small circular knob near top.
    let dial = mesh(pg("WallThermostats__knob", cyl_geo!(0.018, 0.018, 0.006, 8)), &dial_mat);
    dial.rotation().x = PI / 2.0;
    dial.position().set(0, 0.02, 0.013); // back on the body's face (z 0.010)
    thermo.add(&dial);
    // Tiny readout window at bottom.
    let readout = mesh(plane_geo!(0.05, 0.02), &readout_mat);
    readout.position().set(0, -0.035, 0.011);
    thermo.add(&readout);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.011);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.011);
    let thermo_height = 1.4 + rnd() * 0.15;
    wall_mount(&thermo, &c, wall_x, wall_z, thermo_height, 1.0);
    g.world.add(&thermo);
    // R357 track thermostat position for click audio events.
    let tp = thermo.pos();
    g.pp.thermostat_positions.push(v3(tp.x, thermo_height, tp.z));
    // R357 30% chance: subtle readout flicker animator — brief dim
    // dips simulating an aged LCD glitching.
    if rnd() < 0.30 {
        let _readout_base_op = readout_mat.opacity_v();
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let m = readout_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                let on = dip_t > 0.0;
                m.m().color.set_rgb(
                    if on { 0.10 } else { 0x30 as f64 / 255.0 },
                    if on { 0.15 } else { 0x40 as f64 / 255.0 },
                    if on { 0.10 } else { 0x28 as f64 / 255.0 },
                );
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.04;
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
}

// R358 empty picture frame leaning against wall: 15% chance per
// level. 4 thin frame planes + hollow center.
fn place_leaning_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let frame_mat = lambert!({ color: 0x4a3018 });
    let back_mat = lambert!({ color: 0x1a1210 });
    let frame = group();
    // Overall frame 0.30 wide × 0.42 tall × 0.03 deep.
    let (fw, fh, bar) = (0.30, 0.42, 0.025);
    // 4 thin border bars.
    for (sx, sy, w, h) in [
        (0.0, fh / 2.0 - bar / 2.0, fw, bar),              // top
        (0.0, -fh / 2.0 + bar / 2.0, fw, bar),             // bottom
        (-fw / 2.0 + bar / 2.0, 0.0, bar, fh - 2.0 * bar), // left
        (fw / 2.0 - bar / 2.0, 0.0, bar, fh - 2.0 * bar),  // right
    ] {
        let seg = mesh(box_geo!(w, h, 0.03), &frame_mat);
        seg.position().set(sx, sy, 0);
        frame.add(&seg);
    }
    // Dark backing plane inset (visible through empty center).
    let backing = mesh(pg("LeaningFrames__backing", plane_geo!(fw - 2.0 * bar - 0.01, fh - 2.0 * bar - 0.01)), &back_mat);
    backing.position().z = -0.005;
    frame.add(&backing);
    // Lean frame against wall: tilt backward ~10-15°.
    let lean_angle = 0.20 + rnd() * 0.10;
    // Lean about the frame's own horizontal axis (order YXZ), pivoting on the
    // bottom-back edge: that edge rests on the floor and the top-back edge
    // (half height fh/2, half depth 0.015) touches the wall.
    frame.set_rotation_order(EulerOrder::YXZ);
    let lean_y = (fh / 2.0) * lean_angle.cos() + 0.015 * lean_angle.sin();
    let lean_off = (fh / 2.0) * lean_angle.sin() + 0.015 * lean_angle.cos();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - lean_off);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - lean_off);
    if c.dx.abs() == 1 {
        frame.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        // Tilt so top leans TOWARD the wall (rotate around X of local frame).
        frame.rotation().x = -lean_angle * (if c.dx > 0 { 1.0 } else { 1.0 });
        frame.position().set(wall_x, lean_y, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        frame.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        frame.rotation().x = -lean_angle;
        frame.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, lean_y, wall_z);
    }
    g.world.add(&frame);
}

// R375 broken window frame: 15% chance per level. Rectangular
// frame + dark inner void + jagged glass shard remnants.
fn place_broken_window_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let frame_mat = lambert!({ color: 0x2a2418 });
    let void_mat = basic_mat!({ color: 0x040404 });
    let shard_mat_col = shard_color_by_weather(g.weather_key());
    let shard_mat = basic_mat!({ color: shard_mat_col, transparent: true, opacity: 0.7, side: DoubleSide });
    let win = group();
    // Overall window 0.5 × 0.6 m frame at 1.4 m center height.
    let (fw, fh, bar) = (0.50, 0.60, 0.03);
    // 4 border bars.
    for (sx, sy, w, h) in [
        (0.0, fh / 2.0 - bar / 2.0, fw, bar),
        (0.0, -fh / 2.0 + bar / 2.0, fw, bar),
        (-fw / 2.0 + bar / 2.0, 0.0, bar, fh - 2.0 * bar),
        (fw / 2.0 - bar / 2.0, 0.0, bar, fh - 2.0 * bar),
    ] {
        let seg = mesh(box_geo!(w, h, 0.025), &frame_mat);
        seg.position().set(sx, sy, 0);
        win.add(&seg);
    }
    // Dark void plane inside (looking into blackness).
    let void_plane = mesh(plane_geo!(fw - 2.0 * bar - 0.005, fh - 2.0 * bar - 0.005), &void_mat);
    void_plane.position().z = -0.008;
    win.add(&void_plane);
    // 3-5 jagged shard triangles clinging to frame edges.
    let shard_count = 3 + (rnd() * 3.0).floor() as i32;
    for _s in 0..shard_count {
        let s_size = 0.04 + rnd() * 0.05;
        // Random triangle vertex, pointing inward from a frame edge.
        let edge = (rnd() * 4.0).floor() as i32;
        let (sx, sy, dx_v, dy_v);
        if edge == 0 {
            // top
            sx = (rnd() - 0.5) * (fw - 2.0 * bar);
            sy = fh / 2.0 - bar;
            dx_v = (rnd() - 0.5) * s_size;
            dy_v = -s_size;
        } else if edge == 1 {
            // bottom
            sx = (rnd() - 0.5) * (fw - 2.0 * bar);
            sy = -fh / 2.0 + bar;
            dx_v = (rnd() - 0.5) * s_size;
            dy_v = s_size;
        } else if edge == 2 {
            // left
            sx = -fw / 2.0 + bar;
            sy = (rnd() - 0.5) * (fh - 2.0 * bar);
            dx_v = s_size;
            dy_v = (rnd() - 0.5) * s_size;
        } else {
            // right
            sx = fw / 2.0 - bar;
            sy = (rnd() - 0.5) * (fh - 2.0 * bar);
            dx_v = -s_size;
            dy_v = (rnd() - 0.5) * s_size;
        }
        let sh_geo = tri_geo([sx, sy, 0.001, sx + s_size * 0.4, sy + s_size * 0.1, 0.001, sx + dx_v, sy + dy_v, 0.001], [0, 1, 2]);
        let shard = mesh(sh_geo, &shard_mat);
        win.add(&shard);
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.010);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.010);
    wall_mount(&win, &c, wall_x, wall_z, 1.4, 1.0);
    g.world.add(&win);
    // R379 30% chance: wall streak running down from window bottom
    // to floor — implies rain/moisture ingressed through open frame.
    let mut has_streak_under_window = false;
    if rnd() < 0.30 {
        let ws_color = by_weather(g.weather_key(), 0x1a2028, 0x1a1408, 0x180a08, 0x1a1610, 0x181410, 0x181410);
        let ws_mat = basic_mat!({ color: ws_color, transparent: true, opacity: 0.55, depthWrite: false, side: DoubleSide });
        let ws_w = 0.07 + rnd() * 0.05;
        let ws_h = 1.0 + rnd() * 0.2;
        let ws = mesh(plane_geo!(ws_w, ws_h), &ws_mat);
        ws.rotation().y = win.rot().y;
        // Position aligned with window center, starting just below frame.
        let wp = win.pos();
        if c.dx.abs() == 1 {
            ws.position().set(wp.x, 1.4 - fh / 2.0 - ws_h / 2.0 + 0.05, wp.z);
        } else {
            ws.position().set(wp.x, 1.4 - fh / 2.0 - ws_h / 2.0 + 0.05, wp.z);
        }
        g.world.add(&ws);
        has_streak_under_window = true;
    }
    // R380 40% chance (if streak present): small 3D puddle at streak base.
    if has_streak_under_window && rnd() < 0.40 {
        let (wp_color, wp_emissive) = by_weather(
            g.weather_key(),
            (0x1a2830, 0x08101a),
            (0x201808, 0x100804),
            (0x281010, 0x100404),
            (0x181410, 0x080604),
            (0x101820, 0x040810),
            (0x101820, 0x040810),
        );
        let wp_mat = lambert!({ color: wp_color, emissive: wp_emissive, transparent: true, opacity: 0.85 });
        let wpuddle = mesh(sphere_geo!(0.13 + rnd() * 0.08, 10, 5, 0, PI * 2.0, 0, PI / 2.0), &wp_mat);
        wpuddle.scale().y = 0.07;
        // Puddle at floor beneath streak — nudge slightly out from wall.
        let wp = win.pos();
        let puddle_x = wp.x - c.dx as f64 * 0.16;
        let puddle_z = wp.z - c.dz as f64 * 0.16;
        wpuddle.position().set(puddle_x, 0.001, puddle_z);
        wpuddle.rotation().y = rnd() * PI * 2.0;
        g.world.add(&wpuddle);
    }
    // R376 spawn 4-7 floor shards beneath broken window.
    let floor_shard_mat = basic_mat!({ color: shard_mat_col, transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
    let ws_count = 4 + (rnd() * 4.0).floor() as i32;
    for _sh in 0..ws_count {
        let wshard = mesh(plane_geo!(0.014 + rnd() * 0.026, 0.018 + rnd() * 0.028), &floor_shard_mat);
        wshard.rotation().x = -PI / 2.0;
        wshard.rotation().z = rnd() * PI * 2.0;
        let sh_ang = rnd() * PI * 2.0;
        let sh_rad = 0.12 + rnd() * 0.45;
        let wp = win.pos();
        wshard.position().set(wp.x + sh_ang.cos() * sh_rad, 0.003, wp.z + sh_ang.sin() * sh_rad);
        g.world.add(&wshard);
    }
}

// R393 wooden broom: 12% chance per level. Thin wooden pole +
// bristle head, leaning against wall.
fn place_wooden_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let pole_mat = lambert!({ color: 0x6a4820 });
    let bristle_mat = lambert!({ color: 0x3a2818 });
    let broom = group();
    // Pole: thin vertical cylinder.
    let pole = mesh(pg("WoodenBrooms__pole", cyl_geo!(0.012, 0.012, 1.3, 6)), &pole_mat);
    pole.position().y = 0.72;
    broom.add(&pole);
    // Bristle head: flat rectangular box at bottom.
    let head = mesh(pg("WoodenBrooms__bristles", box_geo!(0.20, 0.08, 0.05)), &bristle_mat);
    head.position().y = 0.06;
    broom.add(&head);
    // Small connector wood block between pole and head.
    let connector = mesh(pg("WoodenBrooms__block", box_geo!(0.05, 0.03, 0.05)), &pole_mat);
    connector.position().y = 0.11;
    broom.add(&connector);
    // Lean toward the wall (local -Z after the yaw; order YXZ applies the lean
    // before the yaw). The foot stands out so the pole's top rests on the wall.
    let lean = 0.12 + rnd() * 0.08;
    broom.set_rotation_order(EulerOrder::YXZ);
    broom.rotation().x = -lean;
    let foot_off = 1.37 * lean.sin() + 0.012 * lean.cos();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - foot_off);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - foot_off);
    if c.dx.abs() == 1 {
        broom.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        broom.position().set(wall_x, 0, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        broom.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        broom.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    // Head (bottom at 0.02) down on the floor: its wall-side bottom edge is lowest.
    broom.position().y = -(0.02 * lean.cos() - 0.025 * lean.sin());
    g.world.add(&broom);
    // R394 25% chance: small dark dust pile beside bristle head.
    if rnd() < 0.25 {
        let dust_mat = lambert!({ color: 0x3a3028 });
        let dust_pile = mesh(pg("WoodenBrooms__dust", sphere_geo!(0.05 + rnd() * 0.03, 6, 4)), &dust_mat);
        dust_pile.scale().set(1.2, 0.3, 1.5); // wide flat pile
                                              // Place beside broom in front of bristle, forward from wall.
        let dfx = -c.dx as f64 * 0.20;
        let dfz = -c.dz as f64 * 0.20;
        let bp = broom.pos();
        dust_pile.position().set(bp.x + dfx + (rnd() - 0.5) * 0.15, 0.015, bp.z + dfz + (rnd() - 0.5) * 0.15);
        g.world.add(&dust_pile);
    }
}

// R391 small wooden ladder: 12% chance per level. 2 vertical rails
// + 4 horizontal rungs, leaning against wall.
fn place_small_ladders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let wood_mat = lambert!({ color: 0x6a4020 });
    let ladder = group();
    let r_h = 1.6; // total ladder height
                   // Rails: 2 vertical cylinders 0.35 m apart.
    for rx_off in [-0.175, 0.175] {
        let rail = mesh(pg("SmallLadders__rail", cyl_geo!(0.012, 0.012, r_h, 6)), &wood_mat);
        rail.position().set(rx_off, r_h / 2.0, 0);
        ladder.add(&rail);
    }
    // 4 rungs horizontal.
    for rung in 0..4 {
        let r_y = 0.3 + rung as f64 * 0.35;
        let rung_mesh = mesh(pg("SmallLadders__rung", cyl_geo!(0.010, 0.010, 0.35, 6)), &wood_mat);
        rung_mesh.rotation().z = PI / 2.0;
        rung_mesh.position().set(0, r_y, 0);
        ladder.add(&rung_mesh);
    }
    // Lean ~12° toward wall: lean in the ladder's own frame, then yaw, and
    // stand the feet far enough out that the rail tops just touch the wall.
    ladder.set_rotation_order(EulerOrder::YXZ);
    ladder.rotation().x = -0.20 - rnd() * 0.10;
    let foot_out = r_h * (-ladder.rot().x).sin() + 0.012;
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - foot_out);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - foot_out);
    if c.dx.abs() == 1 {
        ladder.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        ladder.position().set(wall_x, 0, g.wz(c.z) + (rnd() - 0.5) * 1.0);
    } else {
        ladder.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        ladder.position().set(g.wx(c.x) + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&ladder);
    // R392 25% chance: small paint bucket at ladder base.
    if rnd() < 0.25 {
        let bucket_colors = [0x8a8880, 0x707070, 0xa08040];
        let paint_bucket_mat = lambert!({ color: bucket_colors[pick(bucket_colors.len())] });
        let paint_handle_mat = lambert!({ color: 0x2a2018 });
        let paint_bucket = group();
        // Body: shallow tapered cylinder.
        let pb_body = mesh(pg("SmallLadders__pail", cyl_geo!(0.08, 0.07, 0.12, 10)), &paint_bucket_mat);
        pb_body.position().y = 0.06;
        paint_bucket.add(&pb_body);
        // Rim.
        let pb_rim = mesh(pg("SmallLadders__pailRim", torus_geo!(0.08, 0.004, 4, 10)), &paint_bucket_mat);
        pb_rim.rotation().x = PI / 2.0;
        pb_rim.position().y = 0.12;
        paint_bucket.add(&pb_rim);
        // Wire handle: half-torus above.
        let pb_handle = mesh(pg("SmallLadders__pailBail", torus_geo!(0.075, 0.003, 4, 8, PI)), &paint_handle_mat);
        pb_handle.rotation().z = 0.0; // bail over the top, ends on the rim
        pb_handle.position().y = 0.12;
        paint_bucket.add(&pb_handle);
        // Position at ladder base with slight forward offset.
        let pfx = -c.dx as f64 * 0.20;
        let pfz = -c.dz as f64 * 0.20;
        let lp = ladder.pos();
        paint_bucket.position().set(lp.x + pfx + (rnd() - 0.5) * 0.15, 0, lp.z + pfz + (rnd() - 0.5) * 0.15);
        paint_bucket.rotation().y = rnd() * PI * 2.0;
        g.world.add(&paint_bucket);
        // R393 25% chance: colored paint spill stain 0.2-0.4 m from bucket.
        if rnd() < 0.25 {
            let paint_colors = [0xe0e0e0, 0x505050, 0x8a4028, 0xe0d0a0]; // white, gray, rust, tan
            let spill_mat =
                basic_mat!({ color: paint_colors[pick(paint_colors.len())], transparent: true, opacity: 0.75, depthWrite: false });
            let spill = mesh(circle_geo!(0.08 + rnd() * 0.06, 12), &spill_mat);
            spill.rotation().x = -PI / 2.0;
            spill.rotation().z = rnd() * PI * 2.0;
            spill.scale().set(1.4, 0.85, 1);
            let sp_ang = rnd() * PI * 2.0;
            let sp_rad = 0.20 + rnd() * 0.20;
            let pp = paint_bucket.pos();
            spill.position().set(pp.x + sp_ang.cos() * sp_rad, 0.004, pp.z + sp_ang.sin() * sp_rad);
            g.world.add(&spill);
        }
    }
}

// R389 small handbell: 10% chance per level. Brass dome + wooden
// handle + tiny inner clapper.
fn place_small_handbells(g: &mut Game, _dist: &[i32]) {
    g.pp.handbell_positions = vec![]; // R390 reset per level
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let brass_mat = lambert!({ color: 0xa8802a });
    let handle_mat = lambert!({ color: 0x4a2818 });
    let clapper_mat = lambert!({ color: 0x2a2418 });
    let bell = group();
    // Dome: half-sphere brass.
    let dome = mesh(pg("SmallHandbells__dome", sphere_geo!(0.045, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &brass_mat);
    // The upper hemisphere as built already opens downward: mouth (rim) at
    // y 0.005, crown at 0.05 under the collar and handle.
    dome.position().y = 0.005;
    bell.add(&dome);
    // Small ring at bell shoulder.
    let ring = mesh(pg("SmallHandbells__ring", torus_geo!(0.008, 0.002, 4, 8)), &brass_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.052;
    bell.add(&ring);
    // Handle: short vertical cylinder above dome.
    let handle = mesh(pg("SmallHandbells__handle", cyl_geo!(0.008, 0.010, 0.08, 6)), &handle_mat);
    handle.position().y = 0.095;
    bell.add(&handle);
    // Tiny clapper: dark sphere hanging inside dome.
    let clapper = mesh(pg("SmallHandbells__clapper", sphere_geo!(0.006, 6, 4)), &clapper_mat);
    clapper.position().y = 0.02;
    bell.add(&clapper);
    // Bell is usually lying on side or upright: 50/50.
    if rnd() < 0.50 {
        bell.rotation().z = PI / 2.0;
        bell.position().y = 0.045; // rest lying on rim
    }
    bell.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bell.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bell.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bell);
    // R390 track handbell position for rare chime audio events.
    let bp = bell.pos();
    g.pp.handbell_positions.push(v3(bp.x, 0.05, bp.z));
}

// R386 wooden crate: 15% chance per level. Small brown crate box
// with 3 dark slat lines on each visible face.
fn place_wooden_crates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let wood_colors = [0x704028, 0x5a3018, 0x6a3820, 0x503020];
    let wood_mat = lambert!({ color: wood_colors[pick(wood_colors.len())] });
    let slat_mat = lambert!({ color: 0x2a1810 });
    let crate_ = group();
    let cs = 0.35 + rnd() * 0.10; // 35-45 cm cubes
                                  // Body: main cube.
    let body = mesh(pg("WoodenCrates__body", box_geo!(cs, cs, cs)), &wood_mat);
    body.position().y = cs / 2.0;
    crate_.add(&body);
    // 3 thin dark slat lines on each of the 4 visible side faces.
    for (rot_y, _off_z) in [(0.0, cs / 2.0 + 0.001), (PI / 2.0, 0.0), (PI, cs / 2.0 + 0.001), (-PI / 2.0, 0.0)] {
        for sl in 0..3 {
            let slat = mesh(plane_geo!(cs, 0.008), &slat_mat);
            slat.rotation().y = rot_y;
            // Position slats along face height with even spacing.
            let rel_y = cs * 0.25 + sl as f64 * cs * 0.25 - cs / 2.0;
            if rot_y.sin().abs() < 0.5 {
                // Facing Z axis.
                slat.position().set(0, rel_y + cs / 2.0, sign_of(rot_y.cos()) * (cs / 2.0 + 0.001));
            } else {
                // Facing X axis.
                slat.position().set(sign_of(rot_y.sin()) * (cs / 2.0 + 0.001), rel_y + cs / 2.0, 0);
            }
            crate_.add(&slat);
        }
    }
    crate_.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    crate_.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crate_);
    // R388 20% chance: 1-2 dark broken-slat holes on random visible face.
    let has_broken_holes = rnd() < 0.20;
    if has_broken_holes {
        let hole_mat = basic_mat!({ color: 0x0a0806 });
        let hole_count = 1 + (rnd() * 2.0).floor() as i32;
        for _hh in 0..hole_count {
            let hole = mesh(plane_geo!(cs * 0.30, cs * 0.14), &hole_mat);
            // Random face pick (Z+, Z-, X+, X-).
            let face = (rnd() * 4.0).floor() as usize;
            let (hr_y, hz_off, hx_off) = [
                (0.0, cs / 2.0 + 0.003, 0.0),
                (PI, -cs / 2.0 - 0.003, 0.0),
                (PI / 2.0, 0.0, cs / 2.0 + 0.003),
                (-PI / 2.0, 0.0, -cs / 2.0 - 0.003),
            ][face];
            hole.rotation().y = hr_y;
            let hy = cs * 0.3 + rnd() * cs * 0.4;
            // Position hole in crate local space, then apply to crate.
            let cp = crate_.pos();
            let cry = crate_.rot().y;
            hole.position().set(cp.x + hx_off * cry.cos() + hz_off * cry.sin(), hy, cp.z - hx_off * cry.sin() + hz_off * cry.cos());
            hole.rotation().y += cry;
            g.world.add(&hole);
        }
    }
    // R389 if broken holes: 50% chance spawn 4-6 small pellets scattered
    // near crate — spilled contents through broken slats.
    if has_broken_holes && rnd() < 0.50 {
        let pellet_mat = lambert!({ color: 0x3a2818 });
        let pellet_count = 4 + (rnd() * 3.0).floor() as i32;
        for _p in 0..pellet_count {
            let pellet = mesh(pg("WoodenCrates__pellet", sphere_geo!(0.008 + rnd() * 0.004, 4, 3)), &pellet_mat);
            let p_ang = rnd() * PI * 2.0;
            let p_rad = cs * 0.5 + rnd() * 0.30;
            let cp = crate_.pos();
            pellet.position().set(cp.x + p_ang.cos() * p_rad, 0.008, cp.z + p_ang.sin() * p_rad);
            g.world.add(&pellet);
        }
    }
    // R387 25% chance: smaller crate stacked atop (freight pile).
    if rnd() < 0.25 {
        let cs2 = cs * (0.55 + rnd() * 0.20); // 55-75% of base
        let sib_crate = clone_obj(&crate_);
        sib_crate.scale().set(cs2 / cs, cs2 / cs, cs2 / cs);
        sib_crate.position().y = cs + cs2 / 2.0; // sit atop base
        sib_crate.position().x += (rnd() - 0.5) * cs * 0.15;
        sib_crate.position().z += (rnd() - 0.5) * cs * 0.15;
        sib_crate.rotation().y = rnd() * PI * 2.0;
        g.world.add(&sib_crate);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
