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

use placers_kit::place::open_tiles;

use placers_kit::geometry::clone_obj;

use placers_kit::place::ri as pick;

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

// Wall emergency flashlight: 12% chance per level. Small dark
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

// Vintage typewriter: 10% chance per level. Box + slanted key
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
    // 30% chance: 3-4 typed paper sheets scattered on floor.
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

// Twin-bell alarm clock: 12% chance per level. A wind-up twin-bell
// alarm clock (Blender batch r14b: a 104 mm drum of painted steel on two
// splayed feet, two bells and their hammer, a carrying handle, the wind keys
// at its back), its printed dial yellowed behind a domed glass, its hands
// stopped where it ran down; the paint chipped to rust.
fn place_alarm_clocks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.alarm_clock_positions = vec![]; // reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let clock = group();
    clock.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    clock.rotation().y = rnd() * PI * 2.0;
    // which clock, and when it stopped, from where it stands (no draw)
    let p = clock.pos();
    let seed = r14b_kit::seed_of(&[p.x, p.z]);
    let paint = (seed % 5) as usize;
    let (h, m) = (f64::from((seed >> 4) % 12), f64::from((seed >> 8) % 60));
    let alarm = f64::from((seed >> 14) % 48) / 4.0;
    const Y: f64 = 0.0600;
    const DIAL: f64 = 0.0145;
    let col = [0xd2c8b0, 0x6e2a24, 0x1c1b1a, 0x3e5a48, 0x9aa0a2][paint];
    let case_mat = phong!({ color: col, specular: 0x2a2a2a, shininess: 24 });
    case_mat.m().map = Some(r14b_kit::enamel_tex(seed));
    if let Some(geo) = prop_part_as_modelled("Real__alarmCase") {
        clock.add(&mesh(geo, &case_mat));
    }
    let chrome = phong!({ color: 0x5c5c5a, specular: 0xa4a4a0, shininess: 64 });
    chrome.m().map = Some(r14b_kit::pitted_tex(seed));
    if let Some(geo) = prop_part_as_modelled("Real__alarmChrome") {
        clock.add(&mesh(geo, &chrome));
    }
    let dial = mesh(circle_geo!(0.0466, 64), lambert!({ map: &r14b_kit::dial_tex(seed) }));
    dial.position().set(0, Y, DIAL);
    clock.add(&dial);
    let black = lambert!({ color: 0x1a1816 });
    let hand = |name: &str, z: f64, turn: f64, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let pivot = group();
            pivot.add(&mesh(geo, mat));
            pivot.position().set(0, Y, z);
            pivot.rotation().z = -turn * PI * 2.0;
            clock.add(&pivot);
        }
    };
    hand("Real__alarmSet", DIAL + 0.0004, alarm / 12.0, &lambert!({ color: 0x7a2a1e }));
    hand("Real__alarmHour", DIAL + 0.0010, (h + m / 60.0) / 12.0, &black);
    hand("Real__alarmMinute", DIAL + 0.0017, m / 60.0, &black);
    if let Some(geo) = prop_part_as_modelled("Real__alarmGlass") {
        let glass = mesh(geo, r14b_kit::glass_mat(&r14b_kit::glass_dirt_tex(seed)));
        glass.set_render_order(2);
        clock.add(&glass);
    }
    g.world.add(&clock);
    // Track alarm clock position for rare ring audio events.
    let cp = clock.pos();
    g.pp.alarm_clock_positions.push(v3(cp.x, 0.08, cp.z));
}

// Small clock radio: 12% chance per level. Block with tuning
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

// Wall barometer: 10% chance per level. Tall dark-wood case
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

// Small wall watch: 12% chance per level. Tiny 8-cm circle
// on wall — differs from the 20-cm analog clock and the digital one.
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
    // 10% chance: cracked-glass overlay atop watch face.
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

// Digital wall clock: 15% chance per level. Small black
// rectangle with red LED time display. Distinct from the analog
// wall clock.
fn place_digital_wall_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.digital_clock_positions = vec![]; // reset per level
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
    // Track digital clock position for periodic mains-hum audio.
    let dp = dc.pos();
    g.pp.digital_clock_positions.push(v3(dp.x, dc_height, dp.z));
    // 40% chance: brief LED flicker animator (opacity/tint dip
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
    // 25% chance: 1-2 sibling digital clocks 0.28-0.36 m in row.
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
    // 50% chance: small power cord from clock bottom running
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

// Wall no-smoking sign: 15% chance per level. Small round sign
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

// Small ceramic bowl: 10% chance per level. Shallow wide
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
    // 40% chance liquid disc replaces empty interior.
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

// Ceramic mug: 12% chance per level. Straight-walled cylinder
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
    // 25% chance: sit mug on a matching saucer plate.
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
    // 40% chance liquid disc replaces dark empty interior.
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
    // 25% chance: small tea/coffee stain on floor nearby.
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
    // 20% chance: 1-2 sibling mugs nearby (break room scene).
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

// Small trophy: 10% chance per level. Gold cup + base on floor.
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
    // Small gold plaque on front face with tiny dark engraving lines.
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
    // 30% chance: trophy lies on side (Z-rotated 90°).
    if rnd() < 0.30 {
        trophy.rotation().z = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        trophy.position().y = 0.10; // roughly cup diameter above floor
    }
    g.world.add(&trophy);
}

// Wall analog gauge: 15% chance per level. Small circular
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
    // 30% chance: 1-2 sibling gauges 0.24-0.30 m apart in row.
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

// Wall calendar: 20% chance per level, 1 unit. Small
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
    // 30% chance: X marks across some day cells + torn bottom-
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

// Wall thermostat: 20% chance per level, 1 unit. Small rectangular
// beige control panel with tiny dial.
fn place_wall_thermostats(g: &mut Game, _dist: &[i32]) {
    g.pp.thermostat_positions = vec![]; // reset per level
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
    // Track thermostat position for click audio events.
    let tp = thermo.pos();
    g.pp.thermostat_positions.push(v3(tp.x, thermo_height, tp.z));
    // 30% chance: subtle readout flicker animator — brief dim
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

// Empty picture frame leaning against wall: 15% chance per
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

// Broken window frame: 15% chance per level. The lower sash of an old
// timber double-hung window (Blender batch `r09`), taken out and left
// leaning on a wall, two over two lights: its white paint gone cream and
// peeling to the grey wood, the putty cracked, a few shards of its glass
// still in the putty, the stub of its cord; the rest of the glass on the
// floor in front of it. Now and then a damp streak down the wall behind it,
// and the wet at its foot.
fn place_broken_window_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let shard_mat_col = shard_color_by_weather(g.weather_key());
    let glass = sash_glass_mat(shard_mat_col);
    let sash = group();
    {
        use game::assets::props::prop_part_as_modelled as part;
        let frame = part("Real__sashFrame").unwrap_or_else(|| {
            let g = box_geo!(SASH_W, SASH_H, SASH_T);
            g.translate(0, SASH_H / 2.0, -SASH_T / 2.0);
            g
        });
        sash.add(&mesh(frame, lambert!({ color: 0xffffff, map: &sash_paint_tex() })));
        if let Some(geo) = part("Real__sashPutty") {
            sash.add(&mesh(geo, lambert!({ color: 0x9a927e })));
        }
        if let Some(geo) = part("Real__sashCord") {
            sash.add(&mesh(geo, lambert!({ color: 0x80766a })));
        }
    }
    // 3-5 shards still held in the putty, each at an edge of a light (the
    // edge, where along it, which way it leans and how big: its four draws)
    let shard_count = 3 + (rnd() * 3.0).floor() as i32;
    for _s in 0..shard_count {
        let s_size = 0.04 + rnd() * 0.05;
        let edge = (rnd() * 4.0).floor() as i32;
        let along = rnd();
        let lean = rnd() - 0.5;
        sash.add(&mesh(sash_shard_geo(s_size, edge, along, lean), &glass));
    }
    // leaning on the wall: its foot out from the wall, its top against it
    let lean_a = SASH_LEAN;
    let out = SASH_H * lean_a.sin() + SASH_T * lean_a.cos();
    sash.set_rotation_order(EulerOrder::YXZ);
    sash.rotation().x = -lean_a;
    // (the wall point it was hung at, along the wall by the old draw)
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.010);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.010);
    let (mx, mz) = if c.dx.abs() == 1 {
        sash.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        (wall_x, wall_z + (rnd() - 0.5) * 1.0)
    } else {
        sash.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        (wall_x + (rnd() - 0.5) * 1.0, wall_z)
    };
    let back = out - 0.010;
    sash.position().set(mx - c.dx as f64 * back, SASH_T * lean_a.sin(), mz - c.dz as f64 * back);
    g.world.add(&sash);
    // 30% chance: a damp streak down the wall behind it, from above
    // to the floor, soft and uneven, a tide line at its foot
    let mut has_streak_under_window = false;
    if rnd() < 0.30 {
        let ws_color = by_weather(g.weather_key(), 0x1a2028, 0x1a1408, 0x180a08, 0x1a1610, 0x181410, 0x181410);
        let ws_w = 0.07 + rnd() * 0.05;
        let ws_h = 1.0 + rnd() * 0.2;
        let ws_mat = lambert!({ color: 0xffffff, map: &sash_streak_tex(ws_color), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
        let ws = mesh(plane_geo!(ws_w * 2.2, ws_h), &ws_mat);
        ws.rotation().y = sash.rot().y;
        ws.position().set(mx + c.dx as f64 * 0.006, 1.4 - 0.3 - ws_h / 2.0 + 0.05, mz + c.dz as f64 * 0.006);
        g.world.add(&ws);
        has_streak_under_window = true;
    }
    // 40% chance (if streak present): the wet at its foot, soaked into the floor
    if has_streak_under_window && rnd() < 0.40 {
        let (wp_color, _) = by_weather(
            g.weather_key(),
            (0x1a2830, 0x08101a),
            (0x201808, 0x100804),
            (0x281010, 0x100404),
            (0x181410, 0x080604),
            (0x101820, 0x040810),
            (0x101820, 0x040810),
        );
        let r = 0.13 + rnd() * 0.08;
        let wet = mesh(
            plane_geo!(r * 2.6, r * 1.8),
            lambert!({ color: 0xffffff, map: &sash_wet_tex(wp_color), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        wet.rotation().x = -PI / 2.0;
        wet.rotation().z = rnd() * PI * 2.0;
        wet.position().set(mx - c.dx as f64 * 0.16, 0.002, mz - c.dz as f64 * 0.16);
        g.world.add(&wet);
    }
    // 4-7 shards of its glass on the floor in front of it
    let ws_count = 4 + (rnd() * 4.0).floor() as i32;
    for _sh in 0..ws_count {
        let (w, h) = (0.014 + rnd() * 0.026, 0.018 + rnd() * 0.028);
        let wshard = mesh(floor_shard_geo(w, h), &glass);
        wshard.rotation().y = rnd() * PI * 2.0;
        // on the room's side of the wall: half a turn about the way out
        let sh_ang = (rnd() - 0.5) * PI;
        let sh_rad = 0.12 + rnd() * 0.45;
        let (ox, oz) = (-c.dx as f64, -c.dz as f64);
        let (ca, sa) = (sh_ang.cos(), sh_ang.sin());
        wshard.position().set(mx + (ox * ca - oz * sa) * sh_rad, 0.0, mz + (ox * sa + oz * ca) * sh_rad);
        g.world.add(&wshard);
    }
}

/// The sash (`r09`): its width, height and thickness, m
const SASH_W: f64 = 0.60;
const SASH_H: f64 = 0.70;
const SASH_T: f64 = 0.044;
/// How far it leans back against the wall, radians
const SASH_LEAN: f64 = 0.21;

/// Window glass: thin, a little green at its edges, dusty; it catches the light
fn sash_glass_mat(tint: u32) -> Mat {
    // (the weather's tint of the shards, darkened: glass shows what is behind it)
    let ch = |sh: u32| ((tint >> sh) & 255) * 7 / 10;
    let color = (ch(16) << 16) | (ch(8) << 8) | ch(0);
    phong!({ color: color, specular: 0x6a7472, shininess: 90, transparent: true, opacity: 0.24, depthWrite: false, side: DoubleSide })
}

/// A shard still in the putty at `edge` (0 top, 1 bottom, 2 left, 3 right)
/// of a light: its foot buried in the rebate, its point into the light,
/// jagged; 3 mm of glass on the glass's plane
fn sash_shard_geo(size: f64, edge: i32, along: f64, lean: f64) -> Geo {
    // the four lights' sight openings (`r09`'s `sash_lights`)
    let (x0, x1, y0, y1) = (-SASH_W / 2.0 + 0.050, SASH_W / 2.0 - 0.050, 0.080, SASH_H - 0.050);
    let (xm, ym, hb) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.011);
    let half = (along >= 0.5) as usize;
    // the outer edge of a light, or (leaning far) its edge on the glazing bar
    let bar = lean.abs() > 0.3;
    // (the edge as a line: its start, its direction along it, inward)
    let (sx, sy, tx, ty, nx, ny, len) = match (edge, bar) {
        (0, false) => (if half == 0 { x0 } else { xm + hb }, y1, 1.0, 0.0, 0.0, -1.0, xm - hb - x0),
        (0, true) => (if half == 0 { x0 } else { xm + hb }, ym - hb, 1.0, 0.0, 0.0, -1.0, xm - hb - x0),
        (1, false) => (if half == 0 { x0 } else { xm + hb }, y0, 1.0, 0.0, 0.0, 1.0, xm - hb - x0),
        (1, true) => (if half == 0 { x0 } else { xm + hb }, ym + hb, 1.0, 0.0, 0.0, 1.0, xm - hb - x0),
        (2, false) => (x0, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, 1.0, 0.0, ym - hb - y0),
        (2, true) => (xm + hb, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, 1.0, 0.0, ym - hb - y0),
        (_, false) => (x1, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, -1.0, 0.0, ym - hb - y0),
        (_, true) => (xm - hb, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, -1.0, 0.0, ym - hb - y0),
    };
    let f = 0.12 + 0.76 * ((along * 2.0).fract());
    let (bw, depth) = (size * (0.7 + 0.8 * lean.abs()), size * (1.0 + 0.6 * (0.5 - lean.abs())));
    // its outline in (along the edge, into the light), from under the putty
    let reb = 0.009;
    let a0 = (f * len - bw / 2.0).max(0.004);
    let a1 = (a0 + bw).min(len - 0.004);
    let tip = (a0 + a1) / 2.0 + lean * size * 1.2;
    let pts = [
        (a0, -reb),
        (a1, -reb),
        (a1 - bw * 0.08, depth * 0.35),
        (a1 - bw * (0.25 + lean * 0.3), depth * 0.62),
        (tip, depth),
        (a0 + bw * 0.12, depth * 0.48),
    ];
    let z0 = -0.0125;
    let to_sash = |a: f64, d: f64| (sx + tx * a + nx * d, sy + ty * a + ny * d);
    let p2: Vec<(f64, f64)> = pts.iter().map(|&(a, d)| to_sash(a, d)).collect();
    slab_geo(&p2, z0, z0 + 0.003)
}

/// A shard on the floor, w by h, lying flat: an irregular five-sided piece
fn floor_shard_geo(w: f64, h: f64) -> Geo {
    let k = (w * 977.0 + h * 613.0).fract();
    let pts = [
        (-w / 2.0, -h / 2.0 + h * 0.1 * k),
        (w * (0.2 + 0.25 * k), -h / 2.0),
        (w / 2.0, h * (0.1 - 0.2 * k)),
        (w * 0.1, h / 2.0),
        (-w / 2.0 + w * 0.15 * k, h * 0.2),
    ];
    // (x across, its other side along z, from the floor up 2.5 mm)
    let g = slab_geo(&pts, 0.0, 0.0025);
    g.rotate_x(-PI / 2.0);
    g.translate(0, 0, 0);
    g
}

/// A flat convex piece: the outline (x, y) between z0 and z1, its edges
fn slab_geo(pts: &[(f64, f64)], z0: f64, z1: f64) -> Geo {
    let n = pts.len();
    let mut pos = vec![];
    let mut idx = vec![];
    for &z in &[z0, z1] {
        for &(x, y) in pts {
            pos.extend_from_slice(&[x as f32, y as f32, z as f32]);
        }
    }
    for i in 1..n - 1 {
        idx.extend_from_slice(&[0, (i + 1) as u32, i as u32]);
        idx.extend_from_slice(&[n as u32, (n + i) as u32, (n + i + 1) as u32]);
    }
    let base = pos.len() as u32 / 3;
    for i in 0..n {
        let j = (i + 1) % n;
        let (a, b) = (pts[i], pts[j]);
        for &(x, y, z) in &[(a.0, a.1, z0), (b.0, b.1, z0), (b.0, b.1, z1), (a.0, a.1, z1)] {
            pos.extend_from_slice(&[x as f32, y as f32, z as f32]);
        }
        let q = base + i as u32 * 4;
        idx.extend_from_slice(&[q, q + 1, q + 2, q, q + 2, q + 3]);
    }
    let g = Geo::new();
    g.set_position(pos);
    g.set_index(idx);
    g.compute_vertex_normals();
    g
}

/// The sash's paint (u across it, v up it): white gloss gone cream and
/// grey, crazed, peeled in flakes to the grey wood, most of all on the
/// bottom rail where the water stood; dirt in the corners of the lights
fn sash_paint_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        // the paint: white gloss gone cream and grey, uneven
        ctx.set_fill_style("#bfb7a1");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let (x, y, r) = (rnd() * w, rnd() * h, 10.0 + rnd() * 40.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, &format!("rgba(150,140,118,{:.3})", 0.1 + rnd() * 0.2));
            gr.add_color_stop(1, "rgba(150,140,118,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, r, 0, 7);
            ctx.fill();
        }
        // crazed: a net of fine cracks
        ctx.set_stroke_style("rgba(70,62,52,0.4)");
        ctx.set_line_width(0.7);
        for _ in 0..260 {
            let (mut x, mut y) = (rnd() * w, rnd() * h);
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..3 {
                let vert = rnd() < 0.6;
                x += if vert { (rnd() - 0.5) * 3.0 } else { (rnd() - 0.5) * 14.0 };
                y += if vert { (rnd() - 0.5) * 14.0 } else { (rnd() - 0.5) * 3.0 };
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        // the bottom rail, where the water stood: the paint mostly gone, the
        // grey wood bare and checked along its grain, islands of paint left
        let (stile, bot, top) = (0.050 / SASH_W, 0.080 / SASH_H, 0.050 / SASH_H);
        let rail_y = (1.0 - bot * 1.15) * h;
        ctx.set_fill_style("#8c8576");
        ctx.fill_rect(0, rail_y, w, h - rail_y);
        for _ in 0..80 {
            ctx.set_fill_style(format!("rgba(64,58,50,{:.3})", 0.15 + rnd() * 0.3));
            ctx.fill_rect(rnd() * w, rail_y + rnd() * (h - rail_y), 10.0 + rnd() * 50.0, 0.7);
        }
        for _ in 0..26 {
            let (x, y) = (rnd() * w, rail_y + rnd() * (h - rail_y));
            let (rx, ry) = (3.0 + rnd() * 12.0, 1.0 + rnd() * 3.0);
            let mut pts = vec![];
            for i in 0..10 {
                let a = i as f64 / 10.0 * PI * 2.0;
                let k = 0.6 + rnd() * 0.5;
                pts.push((x + a.cos() * rx * k, y + a.sin() * ry * k));
            }
            for (col, dy) in [("rgba(40,34,28,0.45)", 0.8), ("#c3bba5", 0.0)] {
                ctx.set_fill_style(col);
                ctx.begin_path();
                for (i, &(px, py)) in pts.iter().enumerate() {
                    if i == 0 {
                        ctx.move_to(px, py + dy);
                    } else {
                        ctx.line_to(px, py + dy);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // elsewhere flakes gone along each member's grain (up the stiles and
        // the bar, across the rails), a shadow under the lifted edge
        for _ in 0..70 {
            let (u, v) = (rnd(), bot * 1.15 + rnd() * (1.0 - bot * 1.15));
            let across = v > 1.0 - top || (v - (bot + (1.0 - top)) / 2.0).abs() < 0.02;
            let along_u = across && !(u < stile || u > 1.0 - stile || (u - 0.5).abs() < 0.02);
            let (x, y) = (u * w, (1.0 - v) * h);
            let r = 1.2 + rnd() * 3.5;
            let (ex, ey) = if along_u { (2.8, 0.6) } else { (0.6, 2.8) };
            let mut pts = vec![];
            for i in 0..8 {
                let a = i as f64 / 8.0 * PI * 2.0;
                let rr = r * (0.5 + rnd() * 0.7);
                pts.push((x + a.cos() * rr * ex, y + a.sin() * rr * ey));
            }
            for (col, d) in [("rgba(60,52,44,0.35)", 0.8), ("#8f887a", 0.0)] {
                ctx.set_fill_style(col);
                ctx.begin_path();
                for (i, &(px, py)) in pts.iter().enumerate() {
                    if i == 0 {
                        ctx.move_to(px + d, py + d);
                    } else {
                        ctx.line_to(px + d, py + d);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // grime: settled along the bottom rail, run down from the corners
        let gr = ctx.create_linear_gradient(0, h, 0, h * 0.75);
        gr.add_color_stop(0, "rgba(54,46,36,0.5)");
        gr.add_color_stop(1, "rgba(54,46,36,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, h * 0.75, w, h * 0.25);
        // mould in the corners of the lights (their sight corners, in the texture)
        let (sx0, sx1) = ((0.050 / SASH_W) * w, (1.0 - 0.050 / SASH_W) * w);
        let (sy0, sy1) = (h - (0.080 / SASH_H) * h, (0.050 / SASH_H) * h);
        let (smx, smy) = ((sx0 + sx1) / 2.0, (sy0 + sy1) / 2.0);
        for (cx, cy) in [(sx0, sy0), (smx, sy0), (sx1, sy0), (sx0, smy), (smx, smy), (sx1, smy), (sx0, sy1), (smx, sy1), (sx1, sy1)] {
            for _ in 0..14 {
                let (x, y, r) = (cx + (rnd() - 0.5) * 18.0, cy + (rnd() - 0.5) * 18.0, 0.6 + rnd() * 2.2);
                ctx.set_fill_style(format!("rgba(36,34,26,{:.3})", 0.2 + rnd() * 0.4));
                ctx.begin_path();
                ctx.arc(x, y, r, 0, 7);
                ctx.fill();
            }
        }
    })
}

/// A damp streak run down a wall: drips from above, soft-edged, darker
/// where it dried at the foot (the tide line); transparent round it
fn sash_streak_tex(col: u32) -> Tex {
    let (r, gg, b) = ((col >> 16) & 255, (col >> 8) & 255, col & 255);
    canvas_tex_opt(64, 256, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for _ in 0..9 {
            let x = w * (0.25 + rnd() * 0.5);
            let len = h * (0.5 + rnd() * 0.5);
            let wid = 2.0 + rnd() * 7.0;
            let gr = ctx.create_linear_gradient(0, 0, 0, len);
            gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0)"));
            gr.add_color_stop(0.2, &format!("rgba({r},{gg},{b},{:.3})", 0.15 + rnd() * 0.15));
            gr.add_color_stop(1, &format!("rgba({r},{gg},{b},{:.3})", 0.25 + rnd() * 0.2));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.move_to(x - wid * 0.3, 0);
            ctx.quadratic_curve_to(x - wid, len * 0.5, x - wid * 0.4, len);
            ctx.arc(x, len, wid * 0.45, PI, 0);
            ctx.quadratic_curve_to(x + wid, len * 0.5, x + wid * 0.3, 0);
            ctx.close_path();
            ctx.fill();
        }
        // the tide line where it stood and dried
        ctx.save();
        ctx.translate(w / 2.0, h - 8.0);
        ctx.scale(1, 0.25);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, w * 0.5);
        gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0.35)"));
        gr.add_color_stop(1, &format!("rgba({r},{gg},{b},0)"));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, w * 0.5, 0, 7);
        ctx.fill();
        ctx.restore();
    })
}

/// The wet soaked into the floor: a soft, uneven dark patch, a darker rim
/// where it is drying
fn sash_wet_tex(col: u32) -> Tex {
    let (r, gg, b) = ((col >> 16) & 255, (col >> 8) & 255, col & 255);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for k in 0..7 {
            let (x, y) = (w / 2.0 + (rnd() - 0.5) * w * 0.35, h / 2.0 + (rnd() - 0.5) * h * 0.35);
            let rr = w * (0.18 + rnd() * 0.18) * if k == 0 { 1.4 } else { 1.0 };
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0.4)"));
            gr.add_color_stop(0.75, &format!("rgba({r},{gg},{b},0.32)"));
            gr.add_color_stop(1, &format!("rgba({r},{gg},{b},0)"));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, 7);
            ctx.fill();
        }
    })
}

// Wooden broom: 12% chance per level. Thin wooden pole +
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
    // 25% chance: small dark dust pile beside bristle head.
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

// Small wooden ladder: 12% chance per level. 2 vertical rails
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
    // 25% chance: small paint bucket at ladder base.
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
        // 25% chance: colored paint spill stain 0.2-0.4 m from bucket.
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

// Small handbell: 10% chance per level. Brass dome + wooden
// handle + tiny inner clapper.
fn place_small_handbells(g: &mut Game, _dist: &[i32]) {
    g.pp.handbell_positions = vec![]; // reset per level
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
    // Track handbell position for rare chime audio events.
    let bp = bell.pos();
    g.pp.handbell_positions.push(v3(bp.x, 0.05, bp.z));
}

// Wooden crate: 15% chance per level. A nailed slatted crate of rough
// sawn pine gone grey (Blender batch `r09`): its boards with gaps between
// them nailed to corner posts, a slatted lid, wood wool packed inside;
// stencilled on its long sides. Now and then a board stove in, splinters and
// wood wool on the floor by it; a smaller crate stacked on it.
fn place_wooden_crates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // the pine as it weathered: grey, grey-brown, a dull honey, darkened by damp
    let tones = [0x8c8476, 0x7e7262, 0x9a8566, 0x6e665a];
    let tone = tones[pick(tones.len())];
    let cs = 0.35 + rnd() * 0.10; // 35-45 cm: the 0.40 m crate, a little smaller or larger
    let s = cs / CRATE_W;
    // (what is painted on it differs from crate to crate by its size: no draw of its own)
    let look = CrateLook::new(tone, (cs * 7919.0).fract());
    let (crate_, sides) = crate_group("", &look);
    crate_.scale().set(s, s, s);
    crate_.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    crate_.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crate_);
    // 20% chance: one or two boards stove in (a face, a board), wood
    // wool pulled out through the hole
    let has_broken_holes = rnd() < 0.20;
    if has_broken_holes {
        let hole_count = 1 + (rnd() * 2.0).floor() as i32;
        // (face -> its boards broken: +z, -z, +x, -x as the face draw counts them)
        let mut broken: [Vec<usize>; 4] = Default::default();
        for _hh in 0..hole_count {
            let face = (rnd() * 4.0).floor() as usize;
            let hy = cs * 0.3 + rnd() * cs * 0.4;
            let (_, n, _, gap, _, _, bw) = crate_dims("");
            let k = ((hy / s / (bw + gap)).floor() as usize).clamp(1, n - 2);
            if !broken[face].contains(&k) {
                broken[face].push(k);
            }
            // the wood wool pulled out through it, hanging from the gap
            let (hx_off, hz_off) = [(0.0, 1.0), (0.0, -1.0), (1.0, 0.0), (-1.0, 0.0)][face];
            let cp = crate_.pos();
            let cry = crate_.rot().y;
            let yk = (k as f64 * (bw + gap) + bw * 0.35) * s;
            let tuft = group();
            if let Some(gm) = game::assets::props::prop_part_as_modelled("Real__crateWoolTuft") {
                let m = mesh(gm, &look.wool);
                m.rotation().x = -1.2;
                m.position().set(0.0, -0.012, -0.012);
                tuft.add(&m);
            }
            let r = cs / 2.0 + 0.006;
            let (lx, lz) = (hx_off * r, hz_off * r);
            tuft.position().set(cp.x + lx * cry.cos() + lz * cry.sin(), yk, cp.z - lx * cry.sin() + lz * cry.cos());
            tuft.rotation().y = cry + [0.0, PI, PI / 2.0, -PI / 2.0][face];
            tuft.scale().set(s, s, s);
            g.world.add(&tuft);
        }
        // each stove-in face: its side made again with those boards snapped
        // (slot: +z 0, +x 1, -z 2, -x 3)
        for (face, ks) in broken.iter().enumerate() {
            if ks.is_empty() {
                continue;
            }
            let slot = [0, 2, 1, 3][face];
            let tag = if ks.len() > 1 {
                "12"
            } else if ks[0] == 1 {
                "1"
            } else {
                "2"
            };
            let ab = if slot % 2 == 0 { "A" } else { "B" };
            if let Some(gm) = game::assets::props::prop_part_as_modelled(&format!("Real__crateSide{ab}Broken{tag}")) {
                sides[slot].set_geo(gm);
            }
        }
    }
    // If broken holes: 50% chance 4-6 splinters of the board and tufts
    // of wood wool on the floor by it
    if has_broken_holes && rnd() < 0.50 {
        let pellet_count = 4 + (rnd() * 3.0).floor() as i32;
        for p in 0..pellet_count {
            let size = 0.008 + rnd() * 0.004;
            let p_ang = rnd() * PI * 2.0;
            let p_rad = cs * 0.5 + rnd() * 0.30;
            let cp = crate_.pos();
            // clear of the crate's own footprint (it is square, and turned)
            let cry = crate_.rot().y;
            let (dx, dz) = (p_ang.cos(), p_ang.sin());
            let (lx, lz) = (dx * cry.cos() - dz * cry.sin(), dx * cry.sin() + dz * cry.cos());
            let foot = cs / 2.0 / lx.abs().max(lz.abs()).max(1e-3) + 0.03;
            let rad = p_rad.max(foot);
            let name = if p % 2 == 0 { "Real__crateSplinter" } else { "Real__crateWoolTuft" };
            let bit = group();
            if let Some(gm) = game::assets::props::prop_part_as_modelled(name) {
                let m = mesh(gm, if p % 2 == 0 { &look.split } else { &look.wool });
                bit.add(&m);
            }
            let k = 0.9 + (size - 0.008) / 0.004 * 0.2;
            bit.scale().set(k, k, k);
            bit.rotation().y = p_ang * 2.7 + p as f64;
            bit.position().set(cp.x + dx * rad, 0.0, cp.z + dz * rad);
            g.world.add(&bit);
        }
    }
    // 25% chance: a smaller crate stacked on its lid (freight pile)
    if rnd() < 0.25 {
        let cs2 = cs * (0.55 + rnd() * 0.20); // 55-75% of base
                                              // the smaller crates are their own (0.30 or 0.225 m, their boards
                                              // thinner), scaled within 15% of that
        let key = if cs2 < 0.26 { "S" } else { "M" };
        let (w2, ..) = crate_dims(key);
        let (sib_crate, _) = crate_group(key, &look);
        let s2 = cs2 / w2;
        sib_crate.scale().set(s2, s2, s2);
        let cp = crate_.pos();
        sib_crate.position().set(cp.x, cs, cp.z); // on the lid
        sib_crate.position().x += (rnd() - 0.5) * cs * 0.15;
        sib_crate.position().z += (rnd() - 0.5) * cs * 0.15;
        sib_crate.rotation().y = rnd() * PI * 2.0;
        g.world.add(&sib_crate);
    }
}

/// The big crate's side, m (`r09`'s `CRATES`)
const CRATE_W: f64 = 0.40;

/// A crate's sizes (`r09`'s `crate_dims`): W, boards a side, board
/// thickness, gap, post, the sides' height, a board's width
fn crate_dims(key: &str) -> (f64, usize, f64, f64, f64, f64, f64) {
    let (w, n, t, gap, post) = match key {
        "M" => (0.30, 3, 0.010, 0.018, 0.026),
        "S" => (0.225, 3, 0.009, 0.014, 0.022),
        _ => (CRATE_W, 4, 0.012, 0.020, 0.030),
    };
    let hs = w - t;
    (w, n, t, gap, post, hs, (hs - (n - 1) as f64 * gap) / n as f64)
}

/// What a crate is made of: its boards (grain, and the long sides'
/// stencils), the posts and bottom, the nails, the wood wool, the raw wood
/// of a splinter
struct CrateLook {
    side_a: Mat,
    side_b: Mat,
    frame: Mat,
    nails: Mat,
    wool: Mat,
    split: Mat,
}

impl CrateLook {
    fn new(tone: u32, v: f64) -> CrateLook {
        let wool_tex = crate_wool_tex();
        wool_tex.t().repeat.set(3, 3);
        CrateLook {
            side_a: lambert!({ color: tone, map: &crate_board_tex(true, v) }),
            side_b: lambert!({ color: tone, map: &crate_board_tex(false, v) }),
            frame: lambert!({ color: tone }),
            nails: lambert!({ color: 0x3a3029 }),
            wool: lambert!({ color: 0xb09a74, map: &wool_tex }),
            split: lambert!({ color: 0xa88f6c }),
        }
    }
}

/// A crate built of its parts as modelled (`r09`), its bottom at y = 0:
/// the group, and its four sides (+z, +x, -z, -x) so a stove-in one can be
/// swapped in
fn crate_group(key: &str, look: &CrateLook) -> (Obj, Vec<Obj>) {
    use game::assets::props::prop_part_as_modelled as part;
    let c = group();
    let (w, ..) = crate_dims(key);
    let mut sides = vec![];
    for slot in 0..4 {
        let ab = if slot % 2 == 0 { "A" } else { "B" };
        let geo = part(&format!("Real__crateSide{ab}{key}")).unwrap_or_else(|| {
            let g = box_geo!(w, w, 0.012);
            g.translate(0, w / 2.0, w / 2.0 - 0.006);
            g
        });
        let m = mesh(geo, if slot % 2 == 0 { &look.side_a } else { &look.side_b });
        m.rotation().y = slot as f64 * PI / 2.0;
        c.add(&m);
        sides.push(m);
    }
    for (name, mat) in [("Frame", &look.frame), ("Lid", &look.side_b), ("Nails", &look.nails), ("Wool", &look.wool)] {
        if let Some(geo) = part(&format!("Real__crate{name}{key}")) {
            c.add(&mesh(geo, mat));
        }
    }
    (c, sides)
}

/// A side's boards (u along them, v up the side): the grain of rough sawn
/// pine gone grey, its knots, each board a shade of its own, darker where
/// it stood in the wet; on the long sides a shipper's stencil, worn
fn crate_board_tex(stencil: bool, v: f64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        // (the painter's own stream, moved on by this crate's own value)
        for _ in 0..(v * 61.0) as usize {
            rnd();
        }
        ctx.set_fill_style("#e4ded2");
        ctx.fill_rect(0, 0, w, h);
        // each board its own shade (4 boards and 3 gaps up the side)
        let (bw, gap) = (0.082 / 0.388 * h, 0.020 / 0.388 * h);
        for k in 0..4 {
            let y0 = h - (k as f64 * (bw + gap) + bw);
            let v = rnd();
            ctx.set_fill_style(format!("rgba({},{},{},{:.3})", 70 + (v * 40.0) as i32, 62 + (v * 30.0) as i32, 50, 0.06 + rnd() * 0.12));
            ctx.fill_rect(0, y0, w, bw);
        }
        // the grain: long wavy lines along the boards, and the fibres torn up by the saw
        for _ in 0..170 {
            let y = rnd() * h;
            let (a, f, ph) = (0.6 + rnd() * 2.0, 0.01 + rnd() * 0.03, rnd() * 6.3);
            ctx.set_stroke_style(format!("rgba(70,58,44,{:.3})", 0.05 + rnd() * 0.16));
            ctx.set_line_width(0.6 + rnd() * 1.4);
            ctx.begin_path();
            let x0 = rnd() * w * 0.5 - 20.0;
            let len = 60.0 + rnd() * 240.0;
            let mut x = x0;
            ctx.move_to(x, y);
            while x < x0 + len {
                x += 8.0;
                ctx.line_to(x, y + (x * f + ph).sin() * a);
            }
            ctx.stroke();
        }
        for _ in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(format!("rgba(255,250,240,{:.3})", 0.05 + rnd() * 0.10));
            ctx.fill_rect(x, y, 2.0 + rnd() * 6.0, 1);
            ctx.set_fill_style(format!("rgba(50,42,34,{:.3})", 0.05 + rnd() * 0.10));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 4.0, 1);
        }
        // knots: a dark heart, its rings, the grain swept round it
        for _ in 0..(2 + (rnd() * 3.0) as i32) {
            let (x, y, r) = (rnd() * w, rnd() * h, 2.5 + rnd() * 4.5);
            for ring in (0..4).rev() {
                ctx.set_fill_style(format!("rgba(60,44,30,{:.3})", 0.10 + 0.18 * (4 - ring) as f64 / 4.0));
                ctx.begin_path();
                ctx.ellipse(x, y, r * (1.0 + ring as f64 * 0.9) * 1.7, r * (1.0 + ring as f64 * 0.6), 0, 0, 7);
                ctx.fill();
            }
        }
        // the weather: grey bleaching in streaks, and the damp it stood in at the foot
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(200,200,196,{:.3})", 0.04 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 30.0 + rnd() * 120.0, 2.0 + rnd() * 6.0);
        }
        let damp = ctx.create_linear_gradient(0, h, 0, h * 0.6);
        damp.add_color_stop(0, "rgba(40,32,24,0.45)");
        damp.add_color_stop(1, "rgba(40,32,24,0)");
        ctx.set_fill_style(&damp);
        ctx.fill_rect(0, h * 0.6, w, h * 0.4);
        if stencil {
            // a shipper's stencil, sprayed black through a card: the spray's
            // soft edge and its overspray, the bridges of the letters left
            // open; then rubbed and weathered thin
            let marks: [(&str, &str); 6] = [
                ("FRAGILE", "No 47"),
                ("HANDLE WITH CARE", "LOT 3"),
                ("THIS SIDE UP", "No 112"),
                ("KEEP DRY", "No 8"),
                ("GLASS", "No 21"),
                ("DO NOT DROP", "LOT 9"),
            ];
            let (big, small) = marks[((v * 6.0) as usize).min(5)];
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            // the third board from the floor, and the second
            let row = |k: f64| h - (k * (bw + gap) + bw / 2.0);
            let (dx, dy) = ((rnd() - 0.5) * 16.0, (rnd() - 0.5) * 4.0);
            for (a, o) in [(0.2, 1.2), (0.2, -1.0), (0.5, 0.0)] {
                ctx.set_fill_style(format!("rgba(26,24,22,{a})"));
                ctx.set_font("bold 44px Avenir Next Condensed");
                ctx.fill_text_max(big, w / 2.0 + dx + o, row(2.0) + 2.0 + dy + o * 0.5, w * 0.84);
                ctx.set_font("bold 30px Avenir Next Condensed");
                ctx.fill_text_max(small, w * 0.5 + dx * 0.5 + o, row(1.0) + 2.0 - o * 0.4, w * 0.5);
            }
            for _ in 0..500 {
                let (x, y) = (w / 2.0 + dx + (rnd() - 0.5) * w * 0.9, row(2.0) + (rnd() - 0.5) * bw * 1.1);
                ctx.set_fill_style(format!("rgba(26,24,22,{:.3})", 0.1 + rnd() * 0.25));
                ctx.fill_rect(x, y, 1, 1);
            }
            ctx.set_global_composite_operation("destination-out");
            ctx.set_fill_style("rgba(0,0,0,1)");
            // the bridges: a thin bar of card across each letter's middle
            for k in 0..44 {
                let x = 8.0 + k as f64 * (w - 16.0) / 44.0 + rnd() * 2.0;
                ctx.fill_rect(x, row(2.0) - 1.0 + dy, 1.4, 3.0 + rnd() * 2.0);
            }
            // weathered: whole patches gone thin, flakes gone
            for _ in 0..60 {
                let (x, y, r) = (rnd() * w, row(1.0 + rnd() * 1.2), 3.0 + rnd() * 14.0);
                ctx.save();
                ctx.translate(x, y);
                ctx.scale(1, 0.45);
                let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                let a = 0.2 + rnd() * 0.5;
                gr.add_color_stop(0, &format!("rgba(0,0,0,{a:.3})"));
                gr.add_color_stop(1, "rgba(0,0,0,0)");
                ctx.set_fill_style(&gr);
                ctx.begin_path();
                ctx.arc(0, 0, r, 0, 7);
                ctx.fill();
                ctx.restore();
            }
            ctx.set_global_composite_operation("source-over");
        }
        // worn: rubbed through to the wood here and there, and scuffed
        ctx.set_global_composite_operation("source-atop");
        for _ in 0..140 {
            ctx.set_fill_style(format!("rgba(214,206,192,{:.3})", 0.2 + rnd() * 0.5));
            ctx.begin_path();
            ctx.ellipse(rnd() * w, rnd() * h, 1.0 + rnd() * 6.0, 0.6 + rnd() * 2.0, 0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_composite_operation("source-over");
    })
}

/// Wood wool: fine curled shavings, pale straw with darker ones among them
fn crate_wool_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#8a7656");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..420 {
            let v = rnd();
            ctx.set_stroke_style(format!(
                "rgba({},{},{},{:.3})",
                190 + (v * 50.0) as i32,
                168 + (v * 50.0) as i32,
                120 + (v * 40.0) as i32,
                0.35 + rnd() * 0.5
            ));
            ctx.set_line_width(0.6 + rnd() * 1.0);
            let (x, y) = (rnd() * w, rnd() * h);
            let (a, l) = (rnd() * 6.3, 6.0 + rnd() * 26.0);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (a + 1.2).cos() * l, y + (a + 1.2).sin() * l, x + a.cos() * l * 1.4, y + a.sin() * l * 1.4);
            ctx.stroke();
        }
    })
}

/// Batch r14b's kit (the alarm clock): a seed from where it stands,
/// hashed noise, its printed dial, the paint and the plating worn, the
/// dust on its glass.
mod r14b_kit {
    use game::prelude::*;

    pub use placers_kit::place::seed_of;

    use placers_kit::noise::hash as h01;

    use placers_kit::noise::vn as vnoise;

    /// The dial (12 at the top): printed on cream-painted tin, the minute
    /// ring of dots, the hours in a bold geometric face, a luminous dot at
    /// each hour gone brown, the alarm ring's fine track, the maker's name
    /// (none real), UNADJUSTED and the one jewel; yellowed toward the rim,
    /// a tide line where damp stood, fly specks, the paint crazed
    pub fn dial_tex(seed: u32) -> Tex {
        canvas_tex_opt(512, 512, false, move |ctx, w, h| {
            let (cx, cy, r) = (w / 2.0, h / 2.0, w / 2.0);
            ctx.set_fill_style("#d9cfb4");
            ctx.fill_rect(0, 0, w, h);
            let g0 = ctx.create_radial_gradient(cx, cy, r * 0.5, cx, cy, r);
            g0.add_color_stop(0, "rgba(170,140,90,0)");
            g0.add_color_stop(1, "rgba(150,120,72,0.34)");
            ctx.set_fill_style(g0);
            ctx.fill_rect(0, 0, w, h);
            let ink = "rgba(26,22,18,0.92)";
            ctx.set_stroke_style(ink);
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.arc(cx, cy, r * 0.94, 0, 7);
            ctx.stroke();
            ctx.set_line_width(1.2);
            ctx.begin_path();
            ctx.arc(cx, cy, r * 0.87, 0, 7);
            ctx.stroke();
            ctx.set_fill_style(ink);
            for k in 0..60 {
                let a = k as f64 / 60.0 * PI * 2.0;
                let rr = if k % 5 == 0 { 4.6 } else { 2.2 };
                ctx.begin_path();
                ctx.arc(cx + a.sin() * r * 0.905, cy - a.cos() * r * 0.905, rr, 0, 7);
                ctx.fill();
            }
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_font("bold 70px Futura");
            for k in 1..=12 {
                let a = k as f64 / 12.0 * PI * 2.0;
                ctx.fill_text(&k.to_string(), cx + a.sin() * r * 0.70, cy - a.cos() * r * 0.70 + 3.0);
            }
            // the luminous dots inside the numerals, gone brown
            ctx.set_fill_style("rgba(150,128,80,0.9)");
            for k in 0..12 {
                let a = k as f64 / 12.0 * PI * 2.0;
                ctx.begin_path();
                ctx.arc(cx + a.sin() * r * 0.52, cy - a.cos() * r * 0.52, 6.0, 0, 7);
                ctx.fill();
            }
            ctx.set_fill_style(ink);
            ctx.set_font("bold 24px Futura");
            ctx.fill_text("HALDEN", cx, cy - r * 0.30);
            ctx.set_font("15px Helvetica Neue");
            ctx.fill_text("UNADJUSTED", cx, cy + r * 0.26);
            ctx.fill_text("ONE JEWEL", cx, cy + r * 0.34);
            // the paint crazed in fine cracks, the yellowing, a tide line
            let mut rr = SplitMix64::new(seed, 0xa1a7);
            ctx.set_line_width(0.8);
            for _ in 0..70 {
                let (mut x, mut y) = (rr.range(0.0, w), rr.range(0.0, h));
                ctx.set_stroke_style(&format!("rgba(90,70,40,{:.2})", rr.range(0.15, 0.35)));
                ctx.begin_path();
                ctx.move_to(x, y);
                for _ in 0..4 {
                    x += rr.range(-14.0, 14.0);
                    y += rr.range(-14.0, 14.0);
                    ctx.line_to(x, y);
                }
                ctx.stroke();
            }
            let ty = cy + r * rr.range(0.45, 0.7);
            let gr = ctx.create_linear_gradient(0, ty - 30.0, 0, h);
            gr.add_color_stop(0, "rgba(130,96,50,0)");
            gr.add_color_stop(0.12, "rgba(130,96,50,0.3)");
            gr.add_color_stop(0.2, "rgba(130,96,50,0.12)");
            gr.add_color_stop(1, "rgba(120,90,50,0.18)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, ty - 30.0, w, h - ty + 30.0);
            for _ in 0..90 {
                ctx.set_fill_style(&format!("rgba(50,38,25,{:.2})", rr.range(0.3, 0.8)));
                ctx.begin_path();
                ctx.arc(rr.range(0.0, w), rr.range(0.0, h), rr.range(0.8, 2.6), 0, 7);
                ctx.fill();
            }
        })
    }

    /// The enamel worn (multiplied over the paint): dulled in a mottle,
    /// chipped to the dark steel and rusted in the chips, dust settled
    pub fn enamel_tex(seed: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64 / w, j as f64 / h);
                    let m = vnoise(seed, x * 8.0, y * 8.0) * 0.6 + vnoise(seed ^ 0x6d, x * 30.0, y * 30.0) * 0.4;
                    let v = (205.0 + 50.0 * m).min(255.0) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, (f64::from(v) * 0.97) as u8, 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
            let mut r = SplitMix64::new(seed, 0xe7a3);
            // chips knocked off to the steel, small and ragged, rusted
            for _ in 0..14 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                let rad = r.range(0.8, 2.6);
                ctx.set_fill_style("rgba(70,56,44,0.85)");
                ctx.begin_path();
                for k in 0..7 {
                    let a = k as f64 / 7.0 * PI * 2.0;
                    let rr = rad * r.range(0.5, 1.3);
                    if k == 0 {
                        ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                    } else {
                        ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                    }
                }
                ctx.close_path();
                ctx.fill();
                ctx.set_fill_style("rgba(128,70,34,0.55)");
                ctx.begin_path();
                ctx.arc(x, y, rad * 0.45, 0, 7);
                ctx.fill();
            }
            // rust bled from the chips in faint runs
            for _ in 0..6 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                let gr = ctx.create_linear_gradient(x, y, x, y + 14.0);
                gr.add_color_stop(0, "rgba(120,70,36,0.35)");
                gr.add_color_stop(1, "rgba(120,70,36,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - 1.0, y, 2.0, 14);
            }
        })
    }

    /// The plating pitted and dulled (multiplied over the chrome)
    pub fn pitted_tex(seed: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            ctx.set_fill_style("#ffffff");
            ctx.fill_rect(0, 0, w, h);
            let mut r = SplitMix64::new(seed, 0x9e11);
            for _ in 0..18 {
                let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(3.0, 10.0));
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, "rgba(90,80,64,0.5)");
                gr.add_color_stop(1, "rgba(90,80,64,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
            }
            for _ in 0..60 {
                ctx.set_fill_style(&format!("rgba(96,56,30,{:.2})", r.range(0.3, 0.8)));
                ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), r.range(0.6, 1.5), r.range(0.6, 1.5));
            }
        })
    }

    /// Dust on the glass (grey: how much), heavier toward its top
    pub fn glass_dirt_tex(seed: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64 / w, j as f64 / h);
                    let d = vnoise(seed, x * 6.0, y * 6.0) * 0.6 + vnoise(seed ^ 0x19, x * 24.0, y * 24.0) * 0.4;
                    let top = (1.0 - y) * 0.25;
                    let v = ((0.2 + 0.35 * d + top) * 255.0).min(255.0) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
        })
    }

    /// The glass (`Patch::Glass`): the room seen in it by Fresnel, its dust
    /// from the map's grey
    pub fn glass_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 200, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "alarm_glass".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.45;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.2;
            f[9] = 0.2;
            f[10] = 0.19;
            f[11] = 0.17;
            f[12] = 0.0;
            f[13] = 0.07;
        }
        m
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
