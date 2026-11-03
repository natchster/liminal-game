//! Prop placers: extension cords, walkie-talkies, rolled carpets, abandoned
//! laptops, umbrellas, window signs, pinned photos, handprints and wall mirrors.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod tex;
use tex::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_extension_cords", place_extension_cords));
    r.push(("place_walkie_talkies", place_walkie_talkies));
    r.push(("place_rolled_carpets", place_rolled_carpets));
    r.push(("place_abandoned_laptops", place_abandoned_laptops));
    r.push(("place_umbrellas", place_umbrellas));
    r.push(("place_window_signs", place_window_signs));
    r.push(("place_pinned_photos", place_pinned_photos));
    r.push(("place_wall_handprints", place_wall_handprints));
    r.push(("place_wall_mirrors", place_wall_mirrors));
}

// ------------------------------------------------------------------ helpers

/// `{ x, z, dx, dz }`: an open tile and the direction of a wall next to it.
#[derive(Clone, Copy, Debug)]
struct Cand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The shared "open tile next to a wall (or the grid edge)" candidate scan.
fn wall_cands(g: &Game) -> Vec<Cand> {
    let mut out = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    out.push(Cand { x, z, dx, dz });
                }
            }
        }
    }
    out
}

/// Every open interior tile `{ x, z }`.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut out = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                out.push((x, z));
            }
        }
    }
    out
}

/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// `c.dx > 0 ? -PI/2 : PI/2` / `c.dz > 0 ? PI : 0`
fn wall_yaw(c: &Cand) -> f64 {
    if c.dx.abs() == 1 {
        if c.dx > 0 {
            -PI / 2.0
        } else {
            PI / 2.0
        }
    } else if c.dz > 0 {
        PI
    } else {
        0.0
    }
}

/// Extension cord pool per weather; other keys get `[0x181410, 0x201818, 0x202020, 0x141014]`
fn extension_cord_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x0a0f18, 0x101820],
        "electric" => &[0x201808, 0x282010],
        "haunted" => &[0x1a0808, 0x201014],
        "derelict" => &[0x181410, 0x201818],
        // still / normal have no entry → R343 baseline
        _ => &[0x181410, 0x201818, 0x202020, 0x141014],
    }
}
/// Umbrella colour per weather
fn umbrella_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a1218, 0x282010, 0x2a0a0a, 0x3a3020, 0x1a1a2a, 0x1a1a1a)
}
/// Photo tint per weather
fn photo_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8ea0b4, 0xd0b070, 0xa06060, 0xa89870, 0xc4b898, 0xffffff)
}
/// Mirror frame per weather
fn mirror_frame_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a3038, 0x2a1a0a, 0x2a1010, 0x2a2418, 0x3a2a18, 0x3a2a18)
}
/// Mirror glass per weather
fn mirror_glass_by_weather(key: &str) -> u32 {
    by_weather(key, 0x3a5060, 0x605020, 0x603030, 0x484038, 0x505860, 0x505860)
}
/// Soap bar colour pool per weather; other keys get `[0xf0e8d0, 0xd8c0d0, 0xa8d0d8, 0xe8d0a0]`
fn soap_bar_color_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0xa8d0d8, 0xf0e8d0],
        "electric" => &[0xe8d0a0, 0xf0e8d0],
        "haunted" => &[0x8a3040, 0x603030],
        "derelict" => &[0x8a7050, 0xa08060],
        "still" => &[0xf0e8d0, 0xd8c0d0],
        _ => &[0xf0e8d0, 0xd8c0d0, 0xa8d0d8, 0xe8d0a0],
    }
}
/// Toothbrush colour pool per weather; other keys get `[0x40a0d8, 0xf0f0f0, 0xd0d060, 0xd06060]`
fn toothbrush_color_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x40a0d8, 0xf0f0f0],
        "electric" => &[0xd0d060, 0xf0f0f0],
        "haunted" => &[0xd06060, 0x8a3030],
        "derelict" => &[0x8a7040, 0x6a5030],
        "still" => &[0xf0f0f0, 0x40a0d8],
        _ => &[0x40a0d8, 0xf0f0f0, 0xd0d060, 0xd06060],
    }
}

// R343 extension cord: 20% chance per level, 1 coiled cord on floor.
fn place_extension_cords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
      // R348 30% chance: near a tracked R249 socket; R354 25%: near a tracked R287 laptop.
    let (mut x, mut z) = (0i32, 0i32);
    let mut near_socket = false;
    let mut socket_pos: Option<Vec3> = None;
    let mut near_laptop = false;
    let mut laptop_pos: Option<Vec3> = None;
    let roll = rnd();
    if !g.pp.socket_positions.is_empty() && roll < 0.30 {
        let sp = g.pp.socket_positions[(rnd() * g.pp.socket_positions.len() as f64).floor() as usize];
        socket_pos = Some(sp);
        x = g.tx(sp.x);
        z = g.tz(sp.z);
        if !g.in_grid(x, z) || g.tiles[g.t_idx(x, z)] != 0 {
            socket_pos = None;
        } else {
            near_socket = true;
        }
    } else if !g.pp.laptop_positions.is_empty() && roll < 0.55 {
        let lp = g.pp.laptop_positions[(rnd() * g.pp.laptop_positions.len() as f64).floor() as usize];
        laptop_pos = Some(lp);
        x = g.tx(lp.x);
        z = g.tz(lp.z);
        if !g.in_grid(x, z) || g.tiles[g.t_idx(x, z)] != 0 {
            laptop_pos = None;
        } else {
            near_laptop = true;
        }
    }
    if !near_socket && !near_laptop {
        let open = open_tiles(g);
        if open.is_empty() {
            return;
        }
        let p = pick(&open);
        x = p.0;
        z = p.1;
    }
    // R349 per-weather cord palette biases the color pool.
    let cord_pool = extension_cord_by_weather(g.weather_key());
    let cord_mat = lambert!({ color: pick(cord_pool) });
    let plug_mat = lambert!({ color: 0x080604 });
    let cord = group();
    // 3 short cylinder segments zig-zagging.
    let seg_len = 0.6;
    let (mut cur_x, mut cur_z, mut cur_ang) = (0.0f64, 0.0f64, rnd() * PI * 2.0);
    for _s in 0..3 {
        let seg = mesh(pg("ExtensionCords__seg", cyl_geo!(0.012, 0.012, seg_len, 5)), &cord_mat);
        seg.rotation().z = PI / 2.0;
        seg.rotation().y = -cur_ang; // axis along (cos a, 0, sin a), the way the path advances
        seg.position().set(cur_x + cur_ang.cos() * seg_len * 0.5, 0.012, cur_z + cur_ang.sin() * seg_len * 0.5);
        cord.add(&seg);
        // Advance path with angular jitter.
        cur_x += cur_ang.cos() * seg_len;
        cur_z += cur_ang.sin() * seg_len;
        cur_ang += (rnd() - 0.5) * 1.4;
    }
    // Small plug head at cord end, turned to the cord heading, boot (-x) on the cord end.
    let plug = mesh(pg("ExtensionCords__plug", box_geo!(0.05, 0.03, 0.035)), &plug_mat);
    plug.rotation().y = -cur_ang;
    plug.position().set(cur_x + cur_ang.cos() * 0.025, 0.015, cur_z + cur_ang.sin() * 0.025);
    cord.add(&plug);
    // 2 tiny prongs sticking out from plug.
    for yoff in [-0.008, 0.008] {
        let prong = mesh(pg("ExtensionCords__prong", box_geo!(0.02, 0.003, 0.003)), &plug_mat);
        prong.rotation().y = -cur_ang;
        prong.position().set(cur_x + cur_ang.cos() * 0.059, 0.015 + yoff, cur_z + cur_ang.sin() * 0.059);
        cord.add(&prong);
    }
    if let (true, Some(sp)) = (near_socket, socket_pos) {
        cord.position().set(sp.x + (rnd() - 0.5) * 0.4, 0, sp.z + (rnd() - 0.5) * 0.4);
    } else if let (true, Some(lp)) = (near_laptop, laptop_pos) {
        cord.position().set(lp.x + (rnd() - 0.5) * 0.5, 0, lp.z + (rnd() - 0.5) * 0.5);
    } else {
        cord.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    }
    cord.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cord);
}

// R306 walkie talkie: 12% chance per level, 1 handheld radio on floor.
fn place_walkie_talkies(g: &mut Game, _dist: &[i32]) {
    g.pp.walkie_talkie_positions.clear(); // R307 reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0x18201a });
    let grille_mat = lambert!({ color: 0x080a08 });
    let antenna_mat = lambert!({ color: 0x0a0a0a });
    let led_mat = basic_mat!({ color: 0xe04030 });
    let wt = group();
    let body = mesh(pg("WalkieTalkies__body", box_geo!(0.06, 0.14, 0.03)), &body_mat);
    body.position().y = 0.015;
    wt.add(&body);
    let grille = mesh(plane_geo!(0.04, 0.03), &grille_mat);
    grille.position().set(0, 0.05, 0.016);
    wt.add(&grille);
    let led = mesh(circle_geo!(0.004, 6), &led_mat);
    led.position().set(0.02, 0.005, 0.016);
    wt.add(&led);
    let antenna = mesh(pg("WalkieTalkies__antenna", cyl_geo!(0.003, 0.003, 0.11, 5)), &antenna_mat);
    antenna.position().set(-0.02, 0.11, 0);
    wt.add(&antenna);
    // Lying on its side — rotate 90° around Z.
    wt.rotation().z = PI / 2.0;
    wt.rotation().y = rnd() * PI * 2.0;
    let ox = (rnd() - 0.5) * 2.4;
    let oz = (rnd() - 0.5) * 2.4;
    wt.position().set(g.wx(x) + ox, 0.03, g.wz(z) + oz); // on its side: half-width 0.03 is now the height
    g.world.add(&wt);
    // R307 track for static-crackle audio events.
    let p = wt.pos();
    g.pp.walkie_talkie_positions.push(v3(p.x, 0.10, p.z));
}

// R288 rolled carpet: 15% chance per level, 1 rolled carpet against a wall.
fn place_rolled_carpets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let carpet_colors: [u32; 4] = [0x4a2818, 0x38281a, 0x503820, 0x2a1a18];
    let carpet_hex = pick(&carpet_colors);
    let carpet_mat = lambert!({ color: carpet_hex });
    let roll = mesh(pg("RolledCarpets__roll", cyl_geo!(0.12, 0.12, 2.4, 8)), &carpet_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.15);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.15);
    if c.dx.abs() == 1 {
        roll.rotation().x = PI / 2.0; // align with Z axis
        roll.position().set(wall_x, 0.12, g.wz(c.z));
    } else {
        roll.rotation().z = PI / 2.0; // align with X axis
        roll.position().set(g.wx(c.x), 0.12, wall_z);
    }
    g.world.add(&roll);
    // R310 25% chance: second smaller rolled carpet stacked on top of the first.
    if rnd() < 0.25 {
        // the carpet material's hex is the picked hex (exact sRGB round trip)
        let second_colors: Vec<u32> = carpet_colors.iter().copied().filter(|&cc| cc != carpet_hex).collect();
        let second_hex = pick(&second_colors);
        let second_mat = lambert!({ color: second_hex });
        let roll2 = mesh(pg("RolledCarpets__roll", cyl_geo!(0.09, 0.09, 1.9, 8)), &second_mat);
        let tilt = (rnd() - 0.5) * 0.35;
        if c.dx.abs() == 1 {
            // Y→X then yaw by π/2 + tilt: lies along Z with a real yaw
            roll2.rotation().z = PI / 2.0;
            roll2.rotation().y = PI / 2.0 + tilt;
            let rp = roll.pos();
            roll2.position().set(rp.x + c.dx as f64 * 0.02, 0.12 + 0.21, rp.z + (rnd() - 0.5) * 0.3);
        } else {
            roll2.rotation().z = PI / 2.0;
            roll2.rotation().y = tilt;
            let rp = roll.pos();
            roll2.position().set(rp.x + (rnd() - 0.5) * 0.3, 0.12 + 0.21, rp.z + c.dz as f64 * 0.02);
        }
        g.world.add(&roll2);
        // R322 5% chance: 3rd smallest roll perched atop the 2-stack.
        if rnd() < 0.05 {
            let third_colors: Vec<u32> = carpet_colors.iter().copied().filter(|&cc| cc != carpet_hex && cc != second_hex).collect();
            let third_pool: Vec<u32> = if !third_colors.is_empty() { third_colors } else { carpet_colors.to_vec() };
            let third_mat = lambert!({ color: pick(&third_pool) });
            let roll3 = mesh(pg("RolledCarpets__roll", cyl_geo!(0.07, 0.07, 1.5, 8)), &third_mat);
            let tilt3 = (rnd() - 0.5) * 0.5;
            let r2 = roll2.pos();
            if c.dx.abs() == 1 {
                roll3.rotation().z = PI / 2.0;
                roll3.rotation().y = PI / 2.0 + tilt3;
                roll3.position().set(r2.x + (rnd() - 0.5) * 0.05, r2.y + 0.16, r2.z + (rnd() - 0.5) * 0.3);
            } else {
                roll3.rotation().z = PI / 2.0;
                roll3.rotation().y = tilt3;
                roll3.position().set(r2.x + (rnd() - 0.5) * 0.3, r2.y + 0.16, r2.z + (rnd() - 0.5) * 0.05);
            }
            g.world.add(&roll3);
        }
    }
}

// R287 abandoned laptop: 12% chance per level, 1 laptop on the floor.
fn place_abandoned_laptops(g: &mut Game, _dist: &[i32]) {
    g.pp.laptop_positions.clear(); // R354 reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let body_mat = lambert!({ color: 0x1a1a1e });
    let logo_mat = basic_mat!({ color: 0x4a4a52 });
    let laptop = group();
    let base = mesh(pg("AbandonedLaptops__base", box_geo!(0.30, 0.02, 0.22)), &body_mat);
    base.position().y = 0.01;
    laptop.add(&base);
    // R288 8% chance the laptop is open with a faint blue screen glow.
    let is_open = rnd() < 0.08;
    if is_open {
        // R291 keyboard overlay on top face of base.
        let keyboard_mat = lambert!({ map: laptop_keyboard_texture() });
        let keyboard = mesh(plane_geo!(0.28, 0.20), &keyboard_mat);
        keyboard.rotation().x = -PI / 2.0;
        keyboard.position().set(0, 0.021, 0);
        laptop.add(&keyboard);
        // Lid rotated up ~105° from closed position; hinged at back.
        let lid_group = group();
        let lid_open = mesh(pg("AbandonedLaptops__lid", box_geo!(0.30, 0.015, 0.22)), &body_mat);
        // Lid modelled closed, lying flat forward of the hinge (z 0…0.22).
        lid_open.position().set(0, 0.0075, 0.11);
        lid_group.add(&lid_open);
        // R309: 50% chance a garbled-text canvas variant, else faint blue.
        let use_garbled = rnd() < 0.50;
        let screen_mat = if use_garbled {
            let texs = laptop_screen_textures();
            let g_tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
            basic_mat!({ map: g_tex })
        } else {
            basic_mat!({ color: 0x2a4890 })
        };
        let screen = mesh(plane_geo!(0.26, 0.18), &screen_mat);
        // on the lid's inner face (its underside while closed), parallel to the slab
        screen.rotation().x = PI / 2.0;
        screen.position().set(0, -0.001, 0.11);
        lid_group.add(&screen);
        // R312 30% of garbled screens: brief periodic tint flicker.
        if use_garbled && rnd() < 0.30 {
            let flicker_phase = rnd() * PI * 2.0;
            let flicker_period = 1.2 + rnd() * 1.8; // 1.2–3.0 s
            let sm = screen_mat.clone();
            g.animate(move |_g, _dt, t| {
                let phase = (t / flicker_period) * PI * 2.0 + flicker_phase;
                let raw = phase.cos();
                let dip = if raw > 0.85 { (raw - 0.85) / 0.15 } else { 0.0 };
                let bright = 1.0 - dip * 0.55; // 100% down to 45% during dips
                sm.m().color.set_rgb(bright, bright, bright);
            });
        }
        lid_group.position().set(0, 0.02, -0.10); // hinge at back edge
        lid_group.rotation().x = -PI / 1.7; // ~105° open
        laptop.add(&lid_group);
        // Point light with slight blue tint from screen forward.
        let base_int = 0.35;
        let screen_light = point_light!(0x4060b0, base_int, 1.5, 2);
        screen_light.position().set(0, 0.15, 0.10);
        laptop.add(&screen_light);
        // R289 30% chance screen flicker — subtle intensity dip pattern.
        if rnd() < 0.30 {
            let mut flicker_t = 2.0 + rnd() * 4.0;
            let mut dip_t = 0.0;
            let sl = screen_light.clone();
            g.animate(move |_g, dt, _t| {
                if dip_t > 0.0 {
                    dip_t -= dt;
                    sl.set_intensity(if dip_t > 0.0 { base_int * 0.30 } else { base_int });
                } else {
                    flicker_t -= dt;
                    if flicker_t <= 0.0 {
                        dip_t = 0.05; // 50 ms flicker dip
                        flicker_t = 2.0 + rnd() * 4.0;
                    }
                }
            });
        }
    } else {
        // Closed variant: the R287 stacked lid.
        let lid = mesh(pg("AbandonedLaptops__lid", box_geo!(0.30, 0.015, 0.22)), &body_mat);
        lid.position().y = 0.028;
        laptop.add(&lid);
        let logo = mesh(plane_geo!(0.04, 0.04), &logo_mat);
        logo.rotation().x = -PI / 2.0;
        logo.position().set(0, 0.036, 0);
        laptop.add(&logo);
    }
    // Corner bias (near walls) for realistic "set down against wall" placement.
    let wall_bias = rnd() < 0.60;
    let ox = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.0 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.2 };
    let oz = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.0 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.2 };
    laptop.position().x = g.wx(pk.0) + ox;
    laptop.position().z = g.wz(pk.1) + oz;
    laptop.rotation().y = rnd() * PI * 2.0;
    g.world.add(&laptop);
    // R354 track laptop position for extension cord co-loc.
    let lp = laptop.pos();
    g.pp.laptop_positions.push(v3(lp.x, 0.02, lp.z));
    // R289 20% chance a briefcase spawns adjacent to the laptop.
    if rnd() < 0.20 {
        let leather_mat = lambert!({ color: 0x1e1610 });
        let clip_mat = lambert!({ color: 0x8a7a48 });
        let handle_mat = lambert!({ color: 0x281a12 });
        let bc = group();
        let bc_body = mesh(pg("Briefcases__body", box_geo!(0.36, 0.10, 0.28)), &leather_mat);
        bc_body.position().y = 0.05;
        bc.add(&bc_body);
        let bc_seam = mesh(pg("Briefcases__rim", box_geo!(0.36, 0.008, 0.28)), &handle_mat);
        bc_seam.position().y = 0.10;
        bc.add(&bc_seam);
        for cs in [-1.0, 1.0] {
            let clasp = mesh(pg("Briefcases__clasp", box_geo!(0.03, 0.01, 0.02)), &clip_mat);
            clasp.position().set(cs * 0.10, 0.106, 0.14);
            bc.add(&clasp);
        }
        let bc_handle_arch = mesh(pg("Briefcases__handle", torus_geo!(0.05, 0.005, 4, 12, PI)), &handle_mat);
        bc_handle_arch.rotation().x = -PI / 2.0;
        bc_handle_arch.position().set(0, 0.109, 0); // lying on the rim's top (y 0.104)
        bc.add(&bc_handle_arch);
        // Position 0.35-0.55 m from laptop.
        let bc_ang = rnd() * PI * 2.0;
        let bc_dist = 0.35 + rnd() * 0.20;
        bc.position().set(lp.x + bc_ang.cos() * bc_dist, 0, lp.z + bc_ang.sin() * bc_dist);
        bc.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bc);
    }
    // R290 25% chance a coffee cup on floor near laptop.
    if rnd() < 0.25 {
        let vc_mat = lambert!({ color: 0xece0d0 });
        let vcup = group();
        let vcup_body = mesh(pg("AbandonedLaptops__cup", cyl_geo!(0.038, 0.028, 0.09, 10)), &vc_mat);
        vcup_body.position().y = 0.045;
        vcup.add(&vcup_body);
        let vcoffee = mesh(circle_geo!(0.032, 10), lambert!({ color: 0x2a1810 }));
        vcoffee.rotation().x = -PI / 2.0;
        vcoffee.position().y = 0.091;
        vcup.add(&vcoffee);
        let cup_ang = rnd() * PI * 2.0;
        let cup_dist = 0.30 + rnd() * 0.25;
        vcup.position().set(lp.x + cup_ang.cos() * cup_dist, 0, lp.z + cup_ang.sin() * cup_dist);
        vcup.rotation().y = rnd() * PI * 2.0;
        g.world.add(&vcup);
    }
}

// R285 leaning umbrella: 15% chance per level, 1 closed umbrella against a
// wall: a classic stick umbrella (Blender batch `r09`), 0.89 m, furled and
// strapped, stood on its ferrule with its crook against the wall.
fn place_umbrellas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    // R304: bias primary color to per-weather palette, 20% R285 grab-bag.
    let fabric_colors: [u32; 4] = [0x1a1a2a, 0x2a1a1a, 0x1a2a1a, 0x0a0a0a];
    let weather_color = umbrella_color_by_weather(g.weather_key());
    let fabric_color = if rnd() < 0.8 { weather_color } else { pick(&fabric_colors) };
    // (the cloth as dyed nylon is after years: those dark colours, faded
    // toward grey, as dark as black cloth really is)
    let fade = |c: u32, sh: u32| ((c >> sh) & 255) * 3 / 4 + 0x24;
    let cloth = (fade(fabric_color, 16) << 16) | (fade(fabric_color, 8) << 8) | fade(fabric_color, 0);
    let umbrella = group();
    {
        use game::assets::props::prop_part_as_modelled as part;
        let cloth = lambert!({ color: cloth });
        for (name, mat) in [
            ("Real__umbrellaCanopy", cloth.clone()),
            ("Real__umbrellaStrap", cloth),
            ("Real__umbrellaShaft", phong!({ color: 0x1e1d1c, specular: 0x3a3a38, shininess: 40 })),
            ("Real__umbrellaHandle", phong!({ color: 0x5a3c26, specular: 0x2a2018, shininess: 24 })),
            ("Real__umbrellaMetal", phong!({ color: 0x5e5c58, specular: 0x8a8884, shininess: 60 })),
        ] {
            if let Some(geo) = part(name) {
                umbrella.add(&mesh(geo, mat));
            }
        }
    }
    // Stood on its ferrule, leaning toward the wall (local -z) till its crook
    // touches it, a little sideways sway; yaw applied last.
    umbrella.set_rotation_order(EulerOrder::YXZ);
    umbrella.rotation().x = -UMBRELLA_LEAN;
    umbrella.rotation().z = -0.05 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.232);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.232);
    if c.dx.abs() == 1 {
        umbrella.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        umbrella.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        umbrella.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        umbrella.position().set(wall_x + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&umbrella);
    // R286 30% chance: the wet it dripped, dried to a mark round its ferrule
    if rnd() < 0.30 {
        let r = 0.08 + rnd() * 0.05;
        let stain = mesh(
            plane_geo!(r * 2.8, r * 1.6),
            lambert!({ color: 0xffffff, map: &umbrella_drip_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        let uy = umbrella.rot().y;
        let fx = -uy.sin();
        let fz = -uy.cos();
        let up = umbrella.pos();
        stain.position().set(up.x + fx * 0.03, 0.003, up.z + fz * 0.03);
        g.world.add(&stain);
    }
}

/// How far the umbrella leans back, radians: its crook's top (0.887 m up,
/// 11.5 mm round) just touches a wall 0.232 m from its ferrule
const UMBRELLA_LEAN: f64 = 0.2489;

/// What an umbrella stood dripping leaves: a soft, uneven dark mark, its
/// rim darker where it dried (transparent round it)
fn umbrella_drip_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for k in 0..6 {
            let (x, y) = (w / 2.0 + (rnd() - 0.5) * w * 0.3, h / 2.0 + (rnd() - 0.5) * h * 0.3);
            let rr = w * (0.16 + rnd() * 0.16) * if k == 0 { 1.5 } else { 1.0 };
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(14,22,26,0.30)");
            gr.add_color_stop(0.8, "rgba(14,22,26,0.36)");
            gr.add_color_stop(1, "rgba(14,22,26,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, 7);
            ctx.fill();
        }
    })
}

// R304 window sign: 8% chance per level, 1 OPEN/CLOSED placard on a wall.
fn place_window_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = window_sign_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    let sign_mat = lambert!({ map: tex, transparent: true, side: DoubleSide });
    let sign = mesh(plane_geo!(0.22, 0.11), &sign_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let sign_height = 1.7 + rnd() * 0.15;
    if c.dx.abs() == 1 {
        sign.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        sign.position().set(wall_x, sign_height, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        sign.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        sign.position().set(wall_x + (rnd() - 0.5) * 1.0, sign_height, wall_z);
    }
    // R333 slight ±0.05 rad Z tilt for crooked-hang aesthetic.
    sign.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&sign);
    // R317 8% chance: cracked-glass overlay atop the sign.
    if rnd() < 0.08 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.22, 0.11), &crack_mat);
        crack.position().copy(&sign.pos());
        crack.rotation().copy(&sign.rot());
        // Nudge outward 2 mm from wall so overlay renders in front.
        crack.position().x -= c.dx as f64 * 0.002;
        crack.position().z -= c.dz as f64 * 0.002;
        g.world.add(&crack);
        // R327 2-4 tiny floor shards beneath cracked sign.
        let shard_mat = basic_mat!({ color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let sh_count = 2 + (rnd() * 3.0).floor() as i32;
        let sp = sign.pos();
        for _sh in 0..sh_count {
            let shard = mesh(plane_geo!(0.012 + rnd() * 0.020, 0.016 + rnd() * 0.020), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.08 + rnd() * 0.25;
            shard.position().set(sp.x + sh_ang.cos() * sh_rad, 0.003, sp.z + sh_ang.sin() * sh_rad);
            g.world.add(&shard);
        }
    }
}

// R303 pinned photos: 12% chance per level, 1 polaroid pinned to a wall.
fn place_pinned_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = photo_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    // R311 per-weather color tint via material multiply.
    let photo_tint = photo_tint_by_weather(g.weather_key());
    let photo_mat = lambert!({ map: tex, color: photo_tint, transparent: true, side: DoubleSide });
    let photo = mesh(plane_geo!(0.14, 0.14), &photo_mat);
    let pin_mat = lambert!({ color: 0x2a1a10 });
    let pin = mesh(pg("PinnedPhotos__pin", box_geo!(0.008, 0.008, 0.006)), &pin_mat);
    // photo 2 mm off the wall; the pin's flange (back, 6 mm deep) on its front
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.002);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.002);
    let photo_height = 1.5 + rnd() * 0.35;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        photo.rotation().y = rot;
        pin.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        photo.position().set(wall_x, photo_height, wall_z + jit_z);
        pin.position().set(wall_x - c.dx as f64 * 0.003, photo_height + 0.056, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        photo.rotation().y = rot;
        pin.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        photo.position().set(wall_x + jit_x, photo_height, wall_z);
        pin.position().set(wall_x + jit_x, photo_height + 0.056, wall_z - c.dz as f64 * 0.003);
    }
    // Slight tilt as if pinned crookedly.
    photo.rotation().z = (rnd() - 0.5) * 0.15;
    g.world.add(&photo);
    g.world.add(&pin);
    // R318 5% chance: cracked-glass overlay atop the photo.
    if rnd() < 0.05 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.14, 0.14), &crack_mat);
        crack.position().copy(&photo.pos());
        crack.rotation().copy(&photo.rot());
        crack.position().x -= c.dx as f64 * 0.001;
        crack.position().z -= c.dz as f64 * 0.001;
        g.world.add(&crack);
        // R330 2-3 tiny floor shards beneath cracked photo.
        let shard_mat = basic_mat!({ color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let sh_count = 2 + (rnd() * 2.0).floor() as i32;
        let pp = photo.pos();
        for _sh in 0..sh_count {
            let shard = mesh(plane_geo!(0.010 + rnd() * 0.015, 0.014 + rnd() * 0.016), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.06 + rnd() * 0.20;
            shard.position().set(pp.x + sh_ang.cos() * sh_rad, 0.003, pp.z + sh_ang.sin() * sh_rad);
            g.world.add(&shard);
        }
    }
}

// R298 wall handprints: very rare (5% chance per level). The grime a hand
// left on a wall (`assets/src/blender/props/r11a.notes.md`): someone leaned
// on it at a hand's height, or put a hand out going down a corridor in the
// dark; dirt, soot and the skin's oil came off where the hand pressed (the
// finger pads, the pads under the fingers, the heel of the hand), not where
// it arched (the hollow of the palm, the creases), and smeared as it slid
// away. Soft, partial, uneven; a dragged hand leaves its fingers' tracks
// thinning out along the wall. Painted on transparent planes the size of the
// old ones.
fn place_wall_handprints(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let grime = grime_by_weather(g.weather_key());
    let vary = hand_vary(c.x * 4 + c.dx, c.z * 4 + c.dz);
    // which hand it was (the second print, beside it, the other one)
    let right = vary & 1 == 0;
    // R300 12% chance the print is actually an extreme drag streak.
    let is_drag = rnd() < 0.12;
    if is_drag {
        let drag_mat = hand_mat(hand_drag_tex(right, grime, vary));
        let drag = mesh(plane_geo!(0.60, 0.30), &drag_mat);
        let wall_xd = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_zd = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let drag_height = 1.0 + rnd() * 0.5;
        let wall_j = (rnd() - 0.5) * 0.8;
        let (drag_pos_x, drag_pos_z);
        if c.dx.abs() == 1 {
            drag.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            drag_pos_x = wall_xd;
            drag_pos_z = wall_zd + wall_j;
            drag.position().set(drag_pos_x, drag_height, drag_pos_z);
        } else {
            drag.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            drag_pos_x = wall_xd + wall_j;
            drag_pos_z = wall_zd;
            drag.position().set(drag_pos_x, drag_height, drag_pos_z);
        }
        drag.rotation().z = (rnd() - 0.5) * 0.15;
        g.world.add(&drag);
        // R301 20% chance a full handprint at the drag's start edge.
        if rnd() < 0.20 {
            let start_hand = mesh(plane_geo!(0.15, 0.22), &hand_mat(hand_print_tex(right, grime, vary ^ 0x5a)));
            let dr = drag.rot();
            start_hand.rotation().y = dr.y;
            start_hand.rotation().z = dr.z;
            // Offset toward the drag's left (leftmost = start of drag texture).
            let right_vec_x = (dr.y + PI / 2.0).sin();
            let right_vec_z = (dr.y + PI / 2.0).cos();
            start_hand.position().set(drag_pos_x - right_vec_x * 0.28, drag_height, drag_pos_z - right_vec_z * 0.28);
            g.world.add(&start_hand);
        }
        return;
    }
    // R299 20% chance downward-facing print (someone leaned) vs default upward.
    let is_downward = rnd() < 0.20;
    let make_hand = |tilt_z: f64, right: bool, k: u32| -> Obj {
        let h = mesh(plane_geo!(0.15, 0.22), &hand_mat(hand_print_tex(right, grime, vary ^ k)));
        // Downward: turned a half turn, so the fingers point down.
        if is_downward {
            h.rotation().z = PI + tilt_z;
        } else {
            h.rotation().z = tilt_z;
        }
        h
    };
    // R769 handprint wall-stick fix: 15 mm off the wall face; jitter only where the wall continues.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let hand_height = if is_downward { 0.7 + rnd() * 0.4 } else { 1.1 + rnd() * 0.6 };
    let wall_y = if c.dz > 0 { PI } else { 0.0 };
    let rot_y = if c.dx.abs() == 1 {
        if c.dx > 0 {
            -PI / 2.0
        } else {
            PI / 2.0
        }
    } else {
        wall_y
    };
    let (wall_tx, wall_tz) = (c.x + c.dx, c.z + c.dz);
    let mut min_j = -CELL * 0.42;
    let mut max_j = CELL * 0.42;
    if c.dx.abs() == 1 {
        if !g.in_grid(wall_tx, wall_tz - 1) || g.tiles[g.t_idx(wall_tx, wall_tz - 1)] != 1 {
            min_j = -0.45;
        }
        if !g.in_grid(wall_tx, wall_tz + 1) || g.tiles[g.t_idx(wall_tx, wall_tz + 1)] != 1 {
            max_j = 0.45;
        }
    } else {
        if !g.in_grid(wall_tx - 1, wall_tz) || g.tiles[g.t_idx(wall_tx - 1, wall_tz)] != 1 {
            min_j = -0.45;
        }
        if !g.in_grid(wall_tx + 1, wall_tz) || g.tiles[g.t_idx(wall_tx + 1, wall_tz)] != 1 {
            max_j = 0.45;
        }
    }
    let jitter = min_j + rnd() * (max_j - min_j);
    let pos_x = if c.dx.abs() == 1 { wall_x } else { wall_x + jitter };
    let pos_z = if c.dx.abs() == 1 { wall_z + jitter } else { wall_z };
    let tilt_z = (rnd() - 0.5) * 0.4;
    let hand = make_hand(tilt_z, right, 0);
    hand.rotation().y = rot_y;
    hand.position().set(pos_x, hand_height, pos_z);
    g.world.add(&hand);
    // R299 15% chance second handprint 0.3-0.5 m along the wall: the other hand.
    if rnd() < 0.15 {
        let spacing = 0.3 + rnd() * 0.2;
        let dir = if rnd() < 0.5 { 1.0 } else { -1.0 };
        // R769 clamp second-print offset so combined jitter+spacing stays on wall.
        let combined = clamp(jitter + dir * spacing, min_j, max_j);
        let tilt_z2 = (rnd() - 0.5) * 0.4;
        let hand2 = make_hand(tilt_z2, !right, 1);
        hand2.rotation().y = rot_y;
        if c.dx.abs() == 1 {
            hand2.position().set(pos_x, hand_height, wall_z + combined);
        } else {
            hand2.position().set(wall_x + combined, hand_height, pos_z);
        }
        g.world.add(&hand2);
    } else if rnd() < 0.05 && !is_downward {
        // R300 5% vertical pair variant — one high (~1.7 m), one low (~0.9 m).
        let tilt_z3 = (rnd() - 0.5) * 0.4;
        let high_hand = make_hand(tilt_z3, !right, 2);
        high_hand.rotation().y = rot_y;
        high_hand.position().set(pos_x, 1.7, pos_z);
        g.world.add(&high_hand);
        // Original hand also at 0.9 m for pair — override position.
        hand.position().y = 0.9;
    }
    // R302 30% chance a stranger's chalk or a balled-up sheet near the handprint.
    if rnd() < 0.30 {
        if rnd() < 0.5 {
            // a date chalked beside it, someone's note of when they saw it
            // (laid in upright strips under 8 cm wide, each where the old
            // stroke was: settling finds the same wall face for them)
            let chalk_spacing = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.30 + rnd() * 0.20);
            let chalk_h = hand_height + (rnd() - 0.5) * 0.3;
            let tex = chalk_date_tex(vary);
            let chalk_mat = lambert!({
                map: &tex,
                color: 0xdcd8ce,
                transparent: true,
                depthWrite: false,
                polygonOffset: true,
                polygonOffsetFactor: -2,
                polygonOffsetUnits: -2
            });
            let (cx, cz) = if c.dx.abs() == 1 { (pos_x, pos_z + chalk_spacing) } else { (pos_x + chalk_spacing, pos_z) };
            let mark = group();
            let n = (CHALK_DATE_W / 0.07).ceil() as usize;
            let sw = CHALK_DATE_W / n as f64;
            for k in 0..n {
                let geo = plane_geo!(sw, CHALK_DATE_H);
                let (u0, u1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
                geo.set_uv(vec![u0, 1.0, u1, 1.0, u0, 0.0, u1, 0.0]);
                geo.translate((k as f64 + 0.5) * sw - CHALK_DATE_W / 2.0, 0, 0);
                let strip = mesh(geo, &chalk_mat);
                strip.rotation().y = rot_y;
                strip.position().set(cx, chalk_h, cz);
                mark.add(&strip);
            }
            g.world.add(&mark);
        } else {
            // a sheet balled up and dropped (batch r08's), its old size picking
            // a tight ball or a loose wad
            let sz = 0.06 + rnd() * 0.05;
            let u = (sz - 0.06) / 0.05;
            let name = if u < 0.5 { "Real__paperBall" } else { "Real__paperWad" };
            let geo = game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(sz, 0.04, sz));
            let scrap = mesh(geo, &scrap_ball_mat());
            scrap.scale().set_scalar(0.9 + 0.2 * (u * 2.0).fract());
            scrap.position().set(pos_x + (rnd() - 0.5) * 0.3, 0.0, pos_z + (rnd() - 0.5) * 0.3);
            // Nudge slightly away from wall to avoid clipping.
            if c.dx.abs() == 1 {
                scrap.position().x -= c.dx as f64 * 0.2;
            } else {
                scrap.position().z -= c.dz as f64 * 0.2;
            }
            scrap.rotation().y = rnd() * PI * 2.0;
            g.world.add(&scrap);
        }
    }
    // R302 5% chance the handprint FADES cyclically.
    if rnd() < 0.05 {
        let fade_mat = hand.mat();
        let base_opacity = fade_mat.opacity_v();
        let fade_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // 30 s period: opacity oscillates base_opacity ↔ 0.15.
            let frac = ((t * (PI / 15.0) + fade_phase).sin() + 1.0) * 0.5;
            fade_mat.m().opacity = 0.15 + (base_opacity - 0.15) * frac;
        });
    }
}

/// The chalked date's plane (m): as tall as the old stroke, so its middle is
/// where the old one's was
const CHALK_DATE_H: f64 = 0.12;
const CHALK_DATE_W: f64 = 0.13;

/// The grime a hand leaves on a wall in each weather (sRGB): dirt and the
/// skin's oil, mud where it is wet, soot where things burnt or sparked, dust
/// where everything is dust, and a rust-dark brown where the floor is wrong
fn grime_by_weather(key: &str) -> [f64; 3] {
    by_weather(key, [0.26, 0.21, 0.15], [0.15, 0.14, 0.13], [0.27, 0.15, 0.12], [0.2, 0.18, 0.16], [0.36, 0.34, 0.31], [0.24, 0.2, 0.16])
}

/// A number that differs from one placing to the next (from where it was
/// put), to vary what the painters draw: a painter's own stream starts the
/// same on every floor
fn hand_vary(a: i32, b: i32) -> u32 {
    (hash01(0x2981_6c35, a, b) * 4.0e9) as u32
}

/// The decal material of a mark painted in `tex`, lit as the wall under it
fn hand_mat(tex: Tex) -> Mat {
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// The shape of a hand pressed flat, fingers up (m, x across, y up, its
/// middle at the plane's middle; a right hand's thumb on the left, as its
/// print is seen): the signed distance to it, and how hard each part pressed
/// (the finger pads and the pads under them, the heel of the hand; not the
/// hollow of the palm or the creases), 0..1
fn hand_press(x: f64, y: f64, right: bool, spread: f64) -> (f64, f64) {
    let x = if right { x } else { -x };
    // a capsule from a to b, radius r: (distance, how far along 0..1)
    let cap = |a: (f64, f64), b: (f64, f64), r: f64| -> (f64, f64) {
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let t = (((x - a.0) * ex + (y - a.1) * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        ((x - a.0 - ex * t).hypot(y - a.1 - ey * t) - r, t)
    };
    // the palm: a rounded block narrowing to the wrist
    let (pcx, pcy) = (0.002, -0.042);
    let half_w = 0.04 - 0.008 * ((pcy - y) / 0.05).clamp(0.0, 1.0);
    let qx = (x - pcx).abs() - half_w + 0.014;
    let qy = (y - pcy).abs() - 0.047 + 0.014;
    let palm = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - 0.014;
    let mut d = palm;
    let mut press = 0.0f64;
    let fingers = [
        ((-0.028, 0.0), 0.069, 0.0094, -0.14),
        ((-0.009, 0.004), 0.078, 0.0098, -0.03),
        ((0.011, 0.002), 0.072, 0.0092, 0.07),
        ((0.029, -0.006), 0.057, 0.0079, 0.18),
    ];
    let blob = |cx: f64, cy: f64, sx: f64, sy: f64| (-(((x - cx) / sx).powi(2) + ((y - cy) / sy).powi(2))).exp();
    if palm < 0.0 {
        // the heel of the hand (the ball of the thumb, and the little
        // finger's side), the pads under the fingers; the hollow between
        // them touched lightly; the creases lighter still
        let heel = blob(-0.017, -0.068, 0.017, 0.021).max(blob(0.022, -0.07, 0.012, 0.022));
        let pads = fingers.iter().map(|&((bx, by), ..)| blob(bx, by - 0.008, 0.009, 0.0075)).fold(0.0f64, f64::max);
        let hollow = 1.0 - 0.6 * blob(0.004, -0.035, 0.018, 0.016);
        let crease = |a: (f64, f64), b: (f64, f64)| 0.35 + 0.65 * sstep(0.0005, 0.0018, cap(a, b, 0.0).0);
        let creases =
            crease((-0.036, -0.058), (0.004, -0.02)) * crease((-0.04, -0.03), (0.035, -0.018)) * crease((-0.02, -0.012), (0.04, -0.008));
        press = (0.32 * hollow).max(0.85 * heel.max(pads)) * creases;
    }
    // the fingers: (base, length, radius, lean); the thumb from the palm's side
    for (k, &((bx, by), len, r, lean)) in fingers.iter().enumerate() {
        let a = lean * (0.6 + 0.8 * spread) + (k as f64 - 1.5) * 0.02 * spread;
        let tip = (bx + a.sin() * len, by + a.cos() * len);
        let (fd, t) = cap((bx, by), tip, r);
        d = smin_hand(d, fd, 0.004);
        if fd < 0.0 {
            // the pad at the tip pressed hardest, the joints' creases lighter
            let joint = |at: f64| 0.55 + 0.45 * sstep(0.0015, 0.005, ((t - at) * len).abs());
            let p = (0.6 + 0.4 * sstep(0.62, 0.8, t)) * joint(0.36) * joint(0.66);
            press = press.max(p * sstep(0.0, -0.002, fd));
        }
    }
    // the thumb: short and broad, its root in the ball of the thumb
    let (td, t) = cap((-0.027, -0.066), (-0.06 - 0.008 * spread, -0.03 + 0.004 * spread), 0.0122);
    d = smin_hand(d, td, 0.008);
    if td < 0.0 {
        press = press.max(
            (0.5 + 0.4 * sstep(0.5, 0.85, t)) * (0.6 + 0.4 * sstep(0.0015, 0.005, ((t - 0.55) * 0.05).abs())) * sstep(0.0, -0.002, td),
        );
    }
    (d, press)
}

/// The smooth minimum of two distances, blended over `k` (a finger runs
/// into the palm)
fn smin_hand(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// A hand's print in grime on a plane 0.15 × 0.22 m: pressed unevenly
/// (heavier on one side, sometimes only the fingers or only the heel of the
/// hand), mottled as dirt transfers, slid a little as it came away
fn hand_print_tex(right: bool, grime: [f64; 3], vary: u32) -> Tex {
    hand_tex(120.0, 176.0, grime, vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = 0.15 / w;
        let spread = rnd();
        // how it was pressed: across it (a lean), and which part touched
        let (tx, ty) = ((rnd() - 0.5) * 2.0, (rnd() - 0.5) * 2.0);
        let part = rnd();
        let load = 0.55 + 0.35 * rnd();
        // the slide as it came away: down and to one side, a centimetre or two
        let slide = (0.004 + 0.016 * rnd(), (rnd() - 0.5) * 0.6);
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = 0.11 - (j as f64 + 0.5) * s;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - 0.075;
                let mut a = 0.0f64;
                for k in 0..7 {
                    // back along the slide: the print, fainter as it smeared
                    let f = k as f64 / 6.0;
                    let (sx, sy) = (x - slide.1 * slide.0 * f, y + slide.0 * f);
                    let (d, p) = hand_press(sx, sy, right, spread);
                    if d > 0.002 {
                        continue;
                    }
                    let lean = 1.0 + 0.6 * (tx * sx / 0.06 + ty * sy / 0.09);
                    let which = if part < 0.6 {
                        1.0
                    } else if part < 0.85 {
                        // the fingers and the pads under them only
                        sstep(-0.03, -0.01, sy)
                    } else {
                        // the heel of the hand and the fingertips
                        sstep(-0.035, -0.055, sy).max(sstep(0.045, 0.06, sy))
                    };
                    let edge = sstep(0.003, -0.003, d);
                    a = a.max(p * lean.clamp(0.2, 1.4) * which * edge * (1.0 - 0.8 * f) * if k == 0 { 1.0 } else { 0.55 });
                }
                // dirt comes off in specks and patches, not evenly
                let mottle = (0.4 + 0.6 * sstep(0.28, 0.68, fbm(seed ^ 0x9, x * 60.0, y * 60.0, 3)))
                    * (0.65 + 0.35 * fbm(seed, x * 260.0, y * 260.0, 3));
                let speck = hash01(seed ^ 0x77, (x / 0.0011) as i32, (y / 0.0011) as i32);
                let a = (a * load * mottle).min(1.0);
                let a = a * (0.55 + 0.45 * sstep(speck - 0.4, speck + 0.2, a * 1.3));
                let px = [grime[0], grime[1], grime[2], (a * 0.85).min(0.75)];
                for (k, v) in px.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A hand dragged along a wall, on a plane 0.60 × 0.30 m: from where it was
/// pressed (its left), the palm's smear and its fingers' four tracks running
/// on, wavering, thinning and breaking up as the grime ran out
fn hand_drag_tex(right: bool, grime: [f64; 3], vary: u32) -> Tex {
    hand_tex(384.0, 192.0, grime, vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = 0.6 / w;
        // the tracks: fingers 2 cm apart, each its height, width, reach and wobble
        let mid = (rnd() - 0.5) * 0.03;
        let tracks: Vec<(f64, f64, f64, f64, u32)> = (0..4)
            .map(|k| {
                let y = mid + (k as f64 - 1.5) * 0.02 * if right { 1.0 } else { -1.0 };
                (y, 0.006 + 0.002 * rnd(), 0.45 + 0.4 * rnd(), 0.85 + 0.15 * rnd(), seed.wrapping_add(k * 17))
            })
            .collect();
        let start = -0.24;
        let sag = 0.01 + 0.02 * rnd();
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = 0.15 - (j as f64 + 0.5) * s;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - 0.3;
                let run = ((x - start) / 0.5).max(0.0);
                let mut a = 0.0f64;
                // the palm's smear where it began
                let palm = (-(((x - start - 0.03) / 0.065).powi(2) + ((y - mid + 0.03) / 0.04).powi(2))).exp();
                a = a.max(palm * 0.85 * (0.4 + 0.6 * fbm(seed ^ 0x3, x * 60.0, y * 60.0, 3)));
                for &(ty, tw, reach, press, ts) in &tracks {
                    if x < start {
                        continue;
                    }
                    // the track sags as the arm went down, and wavers
                    let yc = ty - sag * run * run + (fbm(ts, run * 6.0, 0.5, 2) - 0.5) * 0.006;
                    let fade = (1.0 - run / reach).max(0.0);
                    let wd = tw * (0.6 + 0.4 * fade);
                    let across = sstep(wd, wd * 0.3, (y - yc).abs());
                    // broken where the finger lifted and the grime gave out
                    let broken = sstep(0.05 + 0.5 * (1.0 - fade), 0.45, vnoise(ts ^ 0x9, run * 30.0, (y - yc) * 300.0));
                    // streaked along by the skin's ridges and grit
                    let streak = 0.6 + 0.4 * vnoise(ts ^ 0x5, x * 25.0, y * 1200.0);
                    a = a.max(across * fade.powf(0.5) * press * broken * streak);
                }
                let speck = hash01(seed ^ 0x77, (x / 0.0011) as i32, (y / 0.0011) as i32);
                let a = a * (0.55 + 0.45 * sstep(speck - 0.4, speck + 0.2, a * 1.3));
                // nothing at the plane's edges
                let a = a * sstep(0.0, 0.01, (0.3 - x.abs()).min(0.15 - y.abs()));
                let px = [grime[0], grime[1], grime[2], (a * 0.95).min(0.85)];
                for (k, v) in px.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A transparent mark `w × h` px painted by `paint` on a stream of its own
/// varied by `vary`; where it is clear its colour is `bleed` (sRGB), so its
/// smaller mips keep its colour at its soft edges
fn hand_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], vary: u32, paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, move |ctx: &mut Canvas, cw: f64, ch: f64| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || paint(ctx, cw, ch))
    });
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// A date chalked quickly on a wall (day and month, underlined), on a plane
/// `CHALK_DATE_W × CHALK_DATE_H`: blackboard chalk caught on the wall's tooth,
/// broken where it skipped, its dust under it. White (the material greys it:
/// white chalk on a wall, years on)
fn chalk_date_tex(vary: u32) -> Tex {
    let (w, h) = (CHALK_DATE_W / 0.001, CHALK_DATE_H / 0.001);
    hand_tex(w.round(), h.round(), [1.0, 1.0, 1.0], vary ^ 0x302, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let day = 1 + (rnd() * 28.0) as u64;
        let month = 1 + (rnd() * 12.0) as u64;
        let text = format!("{day}.{month}");
        // digits 3.5 cm tall (1 mm a px), a quick slant, an underline
        let (dh, dw) = (34.0 + rnd() * 6.0, 20.0 + rnd() * 3.0);
        let slant = 0.08 + 0.1 * rnd();
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        let mut lines: Vec<Vec<(f64, f64)>> = vec![];
        let total: f64 = text.chars().map(|c| if c == '.' { dw * 0.45 } else { dw * 1.28 }).sum::<f64>() - dw * 0.28;
        let mut x0 = (w - total) / 2.0;
        let y0 = h * 0.22;
        for ch in text.chars() {
            if ch == '.' {
                let p = (x0 + dw * 0.05, y0 + dh * 0.9);
                lines.push(vec![p, (p.0 + 2.5, p.1 + 3.0), (p.0 + 4.0, p.1 + 5.5)]);
                x0 += dw * 0.45;
                continue;
            }
            for st in hand_chalk_digit(ch.to_digit(10).unwrap_or(0)) {
                lines.push(st.iter().map(|&(u, v)| jig((x0 + u * dw + (1.0 - v) * dh * slant, y0 + v * dh), 1.6)).collect());
            }
            x0 += dw * 1.28;
        }
        let ul = y0 + dh + 9.0;
        lines.push(vec![
            jig(((w - total) / 2.0 - 4.0, ul + 2.0), 1.5),
            jig((w / 2.0, ul), 1.5),
            jig(((w + total) / 2.0 + 6.0, ul - 3.0), 1.5),
        ]);
        let alpha = hand_chalk_raster(&lines, w, h, 6.0, seed);
        let mut img = ctx.create_image_data(w, h);
        for (k, a) in alpha.iter().enumerate() {
            img.data[k * 4..k * 4 + 4].copy_from_slice(&[255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8]);
        }
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The strokes of a digit in a unit box (u across, v down), as a quick hand
/// draws it
fn hand_chalk_digit(d: u32) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
    };
    match d {
        0 => vec![arc(0.5, 0.5, 0.48, 0.5, -1.4, 4.95)],
        1 => vec![vec![(0.22, 0.2), (0.55, 0.0), (0.55, 1.0)]],
        2 => {
            let mut s = arc(0.5, 0.28, 0.45, 0.28, 3.4, 6.6);
            s.extend([(0.05, 1.0), (1.0, 0.98)]);
            vec![s]
        }
        3 => {
            let mut s = arc(0.45, 0.25, 0.45, 0.25, 3.6, 7.6);
            s.extend(arc(0.45, 0.73, 0.5, 0.27, -1.4, 2.6));
            vec![s]
        }
        4 => vec![vec![(0.7, 1.0), (0.7, 0.0), (0.0, 0.68), (1.0, 0.68)]],
        5 => {
            let mut s = vec![(0.9, 0.0), (0.18, 0.0), (0.12, 0.43)];
            s.extend(arc(0.45, 0.68, 0.5, 0.31, -2.4, 2.7));
            vec![s]
        }
        6 => {
            let mut s = vec![(0.85, 0.05), (0.45, 0.2), (0.08, 0.62)];
            s.extend(arc(0.5, 0.72, 0.43, 0.27, 3.1, 9.4));
            vec![s]
        }
        7 => vec![vec![(0.0, 0.02), (0.95, 0.0), (0.35, 1.0)]],
        8 => {
            let mut s = arc(0.5, 0.25, 0.38, 0.25, 1.6, 7.6);
            s.extend(arc(0.5, 0.74, 0.46, 0.26, -1.6, 4.7));
            vec![s]
        }
        _ => {
            let mut s = arc(0.48, 0.3, 0.44, 0.3, 0.2, 6.6);
            s.extend([(0.9, 0.4), (0.62, 1.0)]);
            vec![s]
        }
    }
}

/// The chalk left on a wall by `lines` (px, each smoothed and pressed
/// heavier in its middle), a stick `width` px wide: how much each pixel of a
/// `w × h` canvas holds (0..1), caught on the wall's tooth, gone where it
/// skipped, its dust fallen below. No draws.
fn hand_chalk_raster(lines: &[Vec<(f64, f64)>], w: f64, h: f64, width: f64, seed: u32) -> Vec<f64> {
    let (n, m) = (w as usize, h as usize);
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        // smoothed (Chaikin, twice), points every 3 px
        let mut p = l.clone();
        for _ in 0..2 {
            if p.len() < 3 {
                break;
            }
            let mut q = vec![p[0]];
            for k in 0..p.len() - 1 {
                let (a, b) = (p[k], p[k + 1]);
                q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
                q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
            }
            q.push(p[p.len() - 1]);
            p = q;
        }
        let mut d = vec![];
        for k in 0..p.len().saturating_sub(1) {
            let (a, b) = (p[k], p[k + 1]);
            let c = ((a.0 - b.0).hypot(a.1 - b.1) / 3.0).ceil().max(1.0) as usize;
            for i in 0..c {
                let t = i as f64 / c as f64;
                d.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        if let Some(&e) = p.last() {
            d.push(e);
        }
        let nd = d.len().max(2) - 1;
        // a dab (a full stop) pressed all along; a stroke lifting at its ends
        let dab = nd < 4;
        let press = |k: usize| {
            let t = k as f64 / nd as f64;
            if dab {
                return 0.95;
            }
            (0.8 + 0.2 * (t * 7.0 + li as f64).sin()) * (t / 0.08).clamp(0.35, 1.0) * ((1.0 - t) / 0.12).clamp(0.25, 1.0)
        };
        for k in 0..d.len().saturating_sub(1) {
            segs.push((d[k], d[k + 1], press(k), press(k + 1), seed.wrapping_add(li as u32 * 97)));
        }
    }
    let r = width / 2.0;
    // each 16 px square's segments, so a pixel looks only at those near it
    const CELL: usize = 16;
    let (cw, chh) = (n.div_ceil(CELL), m.div_ceil(CELL));
    let mut cells: Vec<Vec<u32>> = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - r - 1.0, cw), cell(a.0.max(b.0) + r + 1.0, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - r - 1.0, chh), cell(a.1.max(b.1) + r + 1.0, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CELL) * cw + i / CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                let rr = r * (0.85 + 0.3 * vnoise(ls ^ 0x5a, x * 0.35, y * 0.35));
                let cover = sstep(rr + 0.8, rr - 0.8, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = sstep(0.2, 0.5, vnoise(ls ^ 0x77, (a.0 + ex * t) * 0.11, (a.1 + ey * t) * 0.11));
                let press = (pa + (pb - pa) * t) * (0.3 + 0.7 * skip) * (1.0 - 0.45 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            let grain = hash01(seed ^ 0x1d, (x / 1.3) as i32, (y / 1.3) as i32);
            let patch = vnoise(seed ^ 0x2e, x * 0.12, y * 0.12);
            let tooth = 0.7 * grain + 0.3 * patch;
            // chalk on a wall holds only on the tooth's peaks
            let a = sstep(tooth - 0.06, tooth + 0.05, keep * 0.82);
            out[i * 4] = (a * 255.0).round() as u8;
            out[i * 4 + 1] = (keep * 255.0).round() as u8;
        }
    });
    let mut out: Vec<f64> = (0..n * m).map(|k| f64::from(bytes[k * 4]) / 255.0).collect();
    // the dust: what fell from each stroke, down the wall under it, fading
    for i in 0..n {
        let mut acc = 0.0f64;
        for j in 0..m {
            let k = j * n + i;
            let held = f64::from(bytes[k * 4 + 1]) / 255.0;
            acc = acc * 0.93 + held * 0.1;
            let speck = hash01(seed ^ 0x3f, i as i32, j as i32);
            let dust = acc * (0.12 + 0.5 * (speck > 0.93) as u8 as f64);
            out[k] = 1.0 - (1.0 - out[k]) * (1.0 - dust.min(0.5));
        }
    }
    out
}

/// Office paper balled up: off-white, fragments of its type turned every way,
/// grime in its creases (laid on the ball by its box faces); its print made
/// once, shared
fn scrap_ball_mat() -> Mat {
    lambert!({ map: &scrap_ball_tex() })
}

game::lazy_static_tex!(
    scrap_ball_tex,
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#ddd8cb");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..90 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 200.0 + rnd() * 34.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.2)", v - 4.0, v - 14.0));
            ctx.fill_rect(x, y, 2.0 + rnd() * 7.0, 2.0 + rnd() * 7.0);
        }
        // patches of type, each a facet turned its own way
        for _ in 0..10 {
            ctx.save();
            ctx.translate(rnd() * w, rnd() * h);
            ctx.rotate(rnd() * PI * 2.0);
            ctx.set_fill_style(format!("rgba(40,38,36,{:.2})", 0.25 + rnd() * 0.25));
            for l in 0..4 {
                let mut xx = -18.0;
                while xx < 18.0 {
                    let ww = 2.0 + rnd() * 6.0;
                    ctx.fill_rect(xx, l as f64 * 4.0 - 8.0, ww, 1.6);
                    xx += ww + 1.5;
                }
            }
            ctx.restore();
        }
        ctx.set_line_width(0.7);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 8.0 + rnd() * 30.0;
            ctx.set_stroke_style(format!("rgba(70,64,52,{:.2})", 0.1 + rnd() * 0.12));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
);

/// A hash of (seed, i, j) in [0, 1)
fn hash01(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Smooth value noise in [0, 1]
fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash01(seed, i, j), hash01(seed, i + 1, j));
    let (c, d) = (hash01(seed, i, j + 1), hash01(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// Smoothstep from `e0` to `e1` (either way round)
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// R285 wall mirror: 20% chance per level, 1 framed mirror on a wall (batch r13a).
fn place_wall_mirrors(g: &mut Game, _dist: &[i32]) {
    g.pp.bathroom_bundle_positions.clear(); // R319 reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let key = g.weather_key();
    // A domestic wall mirror, 42 x 60 cm (Blender batch `r13a`): 4 mm
    // silvered glass in a moulded frame, a stained hardwood one or a gilt
    // one, hung flat on four brass mirror plates. The glass gives back the
    // room's light (`Patch::Glass`), its silver going black from the edges;
    // dust on it, now and then hands' smears, now and then struck and
    // cracked, its splinters on the floor below (docs/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    // R297 30% chance smudged; R317 10% chance cracked (the old overlays' draws)
    let smudged = rnd() < 0.30;
    let mirror_cracked = rnd() < 0.10;
    let mirror = group();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let height_draw = rnd();
    let mirror_height = 1.55 + height_draw * 0.25; // eye-level range
    if c.dx.abs() == 1 {
        mirror.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mirror.position().set(wall_x, mirror_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        mirror.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mirror.position().set(wall_x + (rnd() - 0.5) * 1.2, mirror_height, wall_z);
    }
    {
        let p = mirror.pos();
        let vary = wm_vary(p.x, p.z, height_draw);
        // which frame: a gilt one about one in three, from its height's draw
        let gilt = (height_draw * 7919.0).fract() < 0.35;
        // the parts are modelled with the wall's face at z = 0: 8 mm behind the spot
        let body = group();
        body.position().z = -0.008;
        mirror.add(&body);
        let frame_name = if gilt { "Real__wallMirrorFrameGilt" } else { "Real__wallMirrorFrameWood" };
        if let Some(geo) = prop_part_as_modelled(frame_name) {
            let mat = if gilt {
                phong!({ map: &wm_gilt_tex(vary), specular: 0x3a3020, shininess: 24 })
            } else {
                lambert!({ map: &moulding_grain_tex(WM_SIZE[0], WM_SIZE[1], WM_MOULD, [0x56, 0x3a, 0x26], vary, 900.0) })
            };
            body.add(&mesh(geo, &mat));
        }
        if let Some(geo) = prop_part_as_modelled("Real__wallMirrorPlates") {
            body.add(&mesh(geo, phong!({ color: 0x5e4c2c, specular: 0x4a3c24, shininess: 36 })));
        }
        let brk = if mirror_cracked { WmBreak::struck(WM_GLASS[0], WM_GLASS[1], u64::from(vary) << 9 | 0x3b) } else { WmBreak::default() };
        // the black paint on the glass's back, seen where the silver has gone
        let back = mesh(plane_geo!(WM_GLASS[0] + 0.01, WM_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
        back.position().z = 0.0050;
        body.add(&back);
        let glass = mesh(plane_geo!(WM_GLASS[0] + WM_UNDER, WM_GLASS[1] + WM_UNDER), wm_glass_mat(wm_silver_tex(&brk, vary)));
        glass.position().z = WM_GLASS[2] - 0.0001;
        glass.set_render_order(2);
        body.add(&glass);
        let film_mat = lambert!({
            map: &wm_film_tex(&brk, smudged, gilt, vary), transparent: true, depthWrite: false,
            polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
        });
        let film = mesh(plane_geo!(WM_GLASS[0], WM_GLASS[1]), &film_mat);
        film.position().z = WM_GLASS[2] + 0.0003;
        film.set_render_order(3);
        body.add(&film);
    }
    g.world.add(&mirror);
    // R286 2% chance: mirror imperceptibly sways.
    if rnd() < 0.02 {
        let base_rot_z = mirror.rot().z;
        let sway_phase = rnd() * PI * 2.0;
        let m2 = mirror.clone();
        g.animate(move |_g, _dt, t| {
            m2.rotation().z = base_rot_z + (t * 0.3 * PI + sway_phase).sin() * 0.02;
        });
    }
    let mp = mirror.pos();
    let m_rot_y = mirror.rot().y;
    // R318 if mirror is cracked, 3-6 splinters of it on the floor below: the
    // silvered face up or its painted back, in front of the wall
    if mirror_cracked {
        let face_up = phong!({ color: 0x1c1e1e, specular: 0x8a8c8c, shininess: 300 });
        let face_down = lambert!({ color: 0x2c2a28 });
        let (along, out) = ((m_rot_y.cos(), -m_rot_y.sin()), (m_rot_y.sin(), m_rot_y.cos()));
        let shard_count = 3 + (rnd() * 4.0).floor() as i32;
        for _sh in 0..shard_count {
            let sw = 0.015 + rnd() * 0.02;
            let shh = 0.020 + rnd() * 0.02;
            let turn = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.10 + rnd() * 0.35;
            let up = (turn * 7.0).fract() < 0.6;
            let Some(geo) = prop_part_as_modelled("Real__glassShard") else { continue };
            let shard = mesh(geo, if up { &face_up } else { &face_down });
            let k = ((sw + shh) / 0.05).clamp(0.85, 1.15);
            shard.scale().set(k, k, k);
            shard.rotation().y = turn;
            let (a, n) = (sh_ang.cos() * sh_rad, 0.03 + sh_ang.sin().abs() * sh_rad);
            shard.position().set(mp.x + along.0 * a + out.0 * n, 0.001, mp.z + along.1 * a + out.1 * n);
            g.world.add(&shard);
        }
    }
    // R316 25% chance: small wall clock adjacent to the mirror — bathroom-vestibule bundle.
    if rnd() < 0.25 {
        let clocks = clock_textures();
        let clock_tex = clocks[(rnd() * clocks.len() as f64).floor() as usize].clone();
        let clock_mat = lambert!({ map: clock_tex, transparent: true, side: DoubleSide });
        // the school clock's black steel case and domed glass (batch r04)
        // round its printed dial, a little smaller (32 cm), its back on the wall
        let adj_clock = group();
        {
            const K: f64 = 0.944;
            let body = group();
            body.scale().set(K, K, K);
            body.position().z = -0.008;
            adj_clock.add(&body);
            if let Some(geo) = prop_part_as_modelled("Real__clockCase") {
                body.add(&mesh(geo, phong!({ color: 0x1b1b1c, specular: 0x242424, shininess: 30 })));
            }
            let dial = mesh(circle_geo!(0.1499, 48), &clock_mat);
            dial.position().z = 0.0301;
            body.add(&dial);
            if let Some(geo) = prop_part_as_modelled("Real__clockGlass") {
                body.add(&mesh(
                    geo,
                    phong!({ color: 0x9a9a96, transparent: true, opacity: 0.18, depthWrite: false, specular: 0x4a4a4a, shininess: 80 }),
                ));
            }
        }
        let adj_offset = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.60 + rnd() * 0.40);
        let adj_height = 1.90 + rnd() * 0.30;
        adj_clock.rotation().y = m_rot_y;
        if c.dx.abs() == 1 {
            adj_clock.position().set(wall_x, adj_height, mp.z + adj_offset);
        } else {
            adj_clock.position().set(mp.x + adj_offset, adj_height, wall_z);
        }
        // R337 slight ±0.03 rad Z tilt for bundle clock too.
        adj_clock.rotation().z = (rnd() - 0.5) * 0.06;
        g.world.add(&adj_clock);
        let cp = adj_clock.pos();
        // Track for R308 tick audio events (same as R253 clocks).
        g.pp.wall_clock_positions.push(v3(cp.x, adj_height, cp.z));
        // R319 track bathroom-bundle center for rare faucet-drip audio.
        let bundle_cx = (mp.x + cp.x) * 0.5;
        let bundle_cz = (mp.z + cp.z) * 0.5;
        g.pp.bathroom_bundle_positions.push(v3(bundle_cx, 1.0, bundle_cz));
        // The basin under the mirror (batch r13a: a wall-hung vitreous china
        // basin, its rim 0.895 m up, where the soap and the toothbrush lie;
        // its chrome waste and bottle trap). R320's draw (the old dark bar
        // of a "sink outline", 60%) now says whether its taps are still on
        // it at all; R331's (50% of those) whether they are: else the deck
        // shows the two holes they were taken from
        let sink_placed = rnd() < 0.60;
        let basin = group();
        basin.rotation().y = m_rot_y;
        if c.dx.abs() == 1 {
            basin.position().set(wall_x, 0.0, mp.z);
        } else {
            basin.position().set(mp.x, 0.0, wall_z);
        }
        let glaze = phong!({ map: &wm_basin_tex(wm_vary(mp.x, mp.z, 0.75), false), specular: 0x3c3c3a, shininess: 60 });
        let bowl = phong!({ map: &wm_basin_tex(wm_vary(mp.x, mp.z, 0.5), true), specular: 0x3c3c3a, shininess: 60 });
        let chrome = phong!({ color: 0x55585a, specular: 0x8a8c8e, shininess: 70 });
        for (name, mat) in [("Real__wallMirrorBasin", &glaze), ("Real__wallMirrorBasinBowl", &bowl), ("Real__wallMirrorBasinTrap", &chrome)]
        {
            if let Some(geo) = prop_part_as_modelled(name) {
                let m = mesh(geo, mat);
                m.position().z = -0.008;
                basin.add(&m);
            }
        }
        if sink_placed && rnd() < 0.50 {
            if let Some(geo) = prop_part_as_modelled("Real__wallMirrorBasinTaps") {
                // dull with limescale
                let m = mesh(geo, phong!({ color: 0x626462, specular: 0x6a6c6c, shininess: 50 }));
                m.position().z = -0.008;
                basin.add(&m);
            }
        }
        g.world.add(&basin);
        // R321 30% chance: small soap dispenser on the opposite side from the clock.
        let mut dispenser_placed = false;
        let mut dispenser_offset = 0.0;
        if rnd() < 0.30 {
            let dispenser_mat = lambert!({ color: 0x4a4038 });
            let nozzle_mat = lambert!({ color: 0x181410 });
            let dispenser = group();
            let d_body = mesh(pg("WallMirrors__dispenser", cyl_geo!(0.025, 0.025, 0.16, 6)), &dispenser_mat);
            dispenser.add(&d_body);
            let nozzle = mesh(pg("WallMirrors__nozzle", cyl_geo!(0.006, 0.006, 0.02, 4)), &nozzle_mat);
            nozzle.rotation().z = PI / 2.0;
            nozzle.position().set(0.020, -0.075, 0);
            dispenser.add(&nozzle);
            let d_offset = -sign_of(adj_offset) * (0.30 + rnd() * 0.10);
            let d_height = 1.15 + rnd() * 0.10;
            dispenser.rotation().y = m_rot_y;
            if c.dx.abs() == 1 {
                dispenser.position().set(wall_x - c.dx as f64 * 0.028, d_height, mp.z + d_offset);
            } else {
                dispenser.position().set(mp.x + d_offset, d_height, wall_z - c.dz as f64 * 0.028);
            }
            g.world.add(&dispenser);
            dispenser_placed = true;
            dispenser_offset = d_offset;
        }
        // R335 30% chance (if dispenser present): small soap bar on the sink shelf.
        if dispenser_placed && rnd() < 0.30 {
            let bar_pool = soap_bar_color_by_weather(key);
            let bar_mat = lambert!({ color: pick(bar_pool) });
            let soap_bar = mesh(pg("WallMirrors__soap", box_geo!(0.06, 0.020, 0.035)), &bar_mat);
            soap_bar.rotation().y = m_rot_y + (rnd() - 0.5) * 0.4;
            // on the basin's deck, at its end under the dispenser
            let bar_pos_offset = (dispenser_offset + (rnd() - 0.5) * 0.08).clamp(-0.205, 0.205);
            if c.dx.abs() == 1 {
                soap_bar.position().set(wall_x - c.dx as f64 * 0.05, 0.905, mp.z + bar_pos_offset);
            } else {
                soap_bar.position().set(mp.x + bar_pos_offset, 0.905, wall_z - c.dz as f64 * 0.05);
            }
            g.world.add(&soap_bar);
        }
        // R320 40% chance: small dark drip stain on the floor at bundle center.
        if rnd() < 0.40 {
            // (lit, soft and ragged: an unlit disc had glowed in the dark)
            let stain_mat = lambert!({
                map: &wm_stain_tex(wm_vary(bundle_cx, bundle_cz, 0.25)), transparent: true, depthWrite: false,
                polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
            });
            let stain = mesh(circle_geo!(0.08 + rnd() * 0.07, 24), &stain_mat);
            stain.rotation().x = -PI / 2.0;
            stain.rotation().z = rnd() * PI * 2.0;
            stain.scale().set(1.3, 0.9, 1);
            let nudge_x = -(c.dx as f64) * 0.20;
            let nudge_z = -(c.dz as f64) * 0.20;
            stain.position().set(bundle_cx + nudge_x, 0.004, bundle_cz + nudge_z);
            g.world.add(&stain);
        }
        // R328 40% chance: RESTROOM wayfinding sign above the mirror (window sign #3).
        if rnd() < 0.40 {
            let sign_mat = lambert!({ map: window_sign_textures()[3].clone(), transparent: true, side: DoubleSide });
            let restroom_sign = mesh(plane_geo!(0.22, 0.11), &sign_mat);
            restroom_sign.rotation().y = m_rot_y;
            let sign_height = 2.20 + rnd() * 0.15; // above adj_clock's height
            let sign_offset = -sign_of(adj_offset) * (0.20 + rnd() * 0.25);
            if c.dx.abs() == 1 {
                restroom_sign.position().set(wall_x, sign_height, mp.z + sign_offset);
            } else {
                restroom_sign.position().set(mp.x + sign_offset, sign_height, wall_z);
            }
            g.world.add(&restroom_sign);
        }
        // R328 25% chance: tiny toothbrush laid on the sink shelf area.
        if rnd() < 0.25 {
            let brush_pool = toothbrush_color_by_weather(key);
            let brush_mat = lambert!({ color: pick(brush_pool) });
            let bristle_mat = lambert!({ color: 0xe8e0d0 });
            let brush = group();
            let handle = mesh(pg("WallMirrors__brush", cyl_geo!(0.004, 0.004, 0.10, 5)), &brush_mat);
            handle.rotation().z = PI / 2.0;
            brush.add(&handle);
            let bristles = mesh(pg("WallMirrors__bristles", box_geo!(0.012, 0.006, 0.008)), &bristle_mat);
            bristles.position().set(0.056, 0, 0);
            brush.add(&bristles);
            brush.rotation().y = m_rot_y + (rnd() - 0.5) * 0.4;
            let br_offset = (rnd() - 0.5) * 0.30;
            if c.dx.abs() == 1 {
                brush.position().set(wall_x - c.dx as f64 * 0.05, 0.905, mp.z + br_offset);
            } else {
                brush.position().set(mp.x + br_offset, 0.905, wall_z - c.dz as f64 * 0.05);
            }
            g.world.add(&brush);
        }
    }
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn wm_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

/// A mirror's breakage, in metres on its glass (x right and y up from its
/// middle): each crack's points and weight (1 a fine one), where it was
/// struck and how wide the crushed spot is. From a seed (no level draws), so
/// the silver and the film agree
#[derive(Clone, Default)]
struct WmBreak {
    lines: Vec<(Vec<[f64; 2]>, f64)>,
    hits: Vec<([f64; 2], f64)>,
    /// the radial cracks in turn round the blow
    radials: Vec<Vec<[f64; 2]>>,
}

impl WmBreak {
    /// struck once: radial cracks from the blow to the edges (some stopping
    /// short, a few branching), rings between them near it
    fn struck(gw: f64, gh: f64, seed: u64) -> WmBreak {
        let (hw, hh) = (gw / 2.0, gh / 2.0);
        let mut r = SplitMix64::new(0, seed);
        let mut b = WmBreak::default();
        let inside = |p: [f64; 2]| p[0].abs() < hw + 0.003 && p[1].abs() < hh + 0.003;
        let run = |r: &mut SplitMix64, p: [f64; 2], a: f64, reach: f64| -> Vec<[f64; 2]> {
            let mut pts = vec![p];
            let (mut q, mut a, mut l) = (p, a, 0.0);
            while l < reach && inside(q) {
                let step = r.range(0.008, 0.022);
                a += r.range(-0.12, 0.12);
                q = [q[0] + a.cos() * step, q[1] + a.sin() * step];
                l += step;
                pts.push(q);
            }
            pts
        };
        // the blow: chest high, a little to one side
        let p = [r.range(-0.28, 0.28) * gw, r.range(-0.1, 0.3) * gh];
        let crush = r.range(0.005, 0.01);
        b.hits.push((p, crush));
        let n = 7 + (r.next() * 5.0) as usize;
        let a0 = r.range(0.0, PI * 2.0);
        let mut dirs: Vec<Vec<[f64; 2]>> = vec![];
        for i in 0..n {
            let a = a0 + PI * 2.0 * i as f64 / n as f64 + r.range(-0.25, 0.25);
            let start = [p[0] + a.cos() * crush, p[1] + a.sin() * crush];
            let reach = if r.next() < 0.25 { r.range(0.05, 0.16) } else { 9.0 };
            let pts = run(&mut r, start, a, reach);
            if r.next() < 0.4 && pts.len() > 5 {
                let k = pts.len() * (3 + (r.next() * 3.0) as usize) / 10;
                let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
                let turn = a + side * r.range(0.3, 0.6);
                let reach = if r.next() < 0.4 { r.range(0.04, 0.12) } else { 9.0 };
                let br = run(&mut r, pts[k], turn, reach);
                b.lines.push((br, 0.8));
            }
            dirs.push(pts.clone());
            b.lines.push((pts, 1.25));
        }
        let at = |pts: &Vec<[f64; 2]>, rad: f64| pts.iter().find(|q| (q[0] - p[0]).hypot(q[1] - p[1]) >= rad).copied();
        let rings = [r.range(0.018, 0.035), r.range(0.05, 0.09)];
        for (ri, &rad) in rings.iter().enumerate() {
            let chance = if ri == 0 { 0.55 } else { 0.3 };
            for i in 0..n {
                let j = (i + 1) % n;
                if r.next() > chance {
                    continue;
                }
                let r0 = rad * r.range(0.75, 1.3);
                let r1 = r0 * r.range(0.8, 1.2);
                let (Some(q0), Some(q1)) = (at(&dirs[i], r0), at(&dirs[j], r1)) else { continue };
                let m = [(q0[0] + q1[0]) / 2.0 + r.range(-0.002, 0.002), (q0[1] + q1[1]) / 2.0 + r.range(-0.002, 0.002)];
                b.lines.push((vec![q0, m, q1], 0.75));
            }
        }
        b.radials = dirs;
        b
    }
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn wm_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The glass: silver behind it (`Patch::Glass`, nearly all of the light
/// back), its map where the silver has gone to the black paint behind
fn wm_glass_mat(silver: Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 4000, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(silver);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off what it faces), nothing behind it
        f[0] = 0.13;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the map's grey is the silver gone, to the black paint behind (linear)
        f[8] = 1.0;
        f[9] = 0.016;
        f[10] = 0.015;
        f[11] = 0.013;
        f[12] = 0.0;
        // silvered: most of the light comes back at any angle
        f[13] = 0.86;
    }
    m
}

/// Where the silver has gone (grey: how much, to the black paint behind):
/// a black edge creeping in from the frame in stretches with a wandering,
/// feathered front (deepest along the foot and in the corners), spots of it
/// ahead, pinholes; the silver flaked along the cracks and round the blow
fn wm_silver_tex(b: &WmBreak, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(256, 384, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (WM_GLASS[0] + WM_UNDER, WM_GLASS[1] + WM_UNDER);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.set_fill_style("#000");
            ctx.fill_rect(0, 0, w, h);
            // the room in it, darker low down
            let gr = ctx.create_linear_gradient(0, 0, 0, h);
            gr.add_color_stop(0, "rgba(255,255,255,0)");
            gr.add_color_stop(0.4, "rgba(255,255,255,0.05)");
            gr.add_color_stop(1, "rgba(255,255,255,0.26)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, w, h);
            let spot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, a: f64| {
                let stops = [
                    (0.0, format!("rgba(255,255,255,{a:.3})")),
                    (0.5, format!("rgba(255,255,255,{:.3})", a * 0.75)),
                    (1.0, "rgba(255,255,255,0)".into()),
                ];
                wm_blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
            };
            // the black edge along each side (how deep at most in px, the side's length)
            let sides: [(f64, f64); 4] = [(13.0, w), (6.0, h), (6.0, h), (4.0, w)]; // foot, left, right, top
            for (si, &(deep, len)) in sides.iter().enumerate() {
                let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
                let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
                let depth = |t: f64| -> f64 {
                    let wave = 0.5
                        + 0.22 * (fr[0] * t * 6.28 + ph[0]).sin()
                        + 0.16 * (fr[1] * t * 6.28 + ph[1]).sin()
                        + 0.1 * (fr[2] * t * 6.28 + ph[2]).sin();
                    let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                    let corner = 1.0 + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).exp();
                    deep * (on * on * (3.0 - 2.0 * on)) * wave * corner
                };
                let at = |t: f64, d: f64| -> (f64, f64) {
                    match si {
                        0 => (t * w, h - d),
                        1 => (d, h - t * h),
                        2 => (w - d, h - t * h),
                        _ => (t * w, d),
                    }
                };
                ctx.set_fill_style("rgba(255,255,255,0.85)");
                ctx.begin_path();
                let (x0, y0) = at(0.0, 0.0);
                ctx.move_to(x0, y0);
                let n = (len / 3.0) as usize;
                for k in 0..=n {
                    let t = k as f64 / n as f64;
                    let (x, y) = at(t, depth(t));
                    ctx.line_to(x, y);
                }
                let (x1, y1) = at(1.0, 0.0);
                ctx.line_to(x1, y1);
                ctx.close_path();
                ctx.fill();
                for _ in 0..(len / 2.5) as usize {
                    let t = rnd();
                    let d = depth(t);
                    if d < 0.5 {
                        continue;
                    }
                    let (x, y) = at(t, d * (0.8 + 0.4 * rnd()));
                    spot(ctx, x, y, 1.0 + rnd() * d.min(10.0) * 0.45, 0.3 + 0.4 * rnd());
                }
                for _ in 0..(len / 6.0) as usize {
                    let t = rnd();
                    let d = depth(t).max(1.5) * (1.3 + 2.5 * rnd().powi(2));
                    let (x, y) = at(t, d);
                    spot(ctx, x, y, 0.6 + rnd() * rnd() * 3.0, 0.4 + 0.5 * rnd());
                }
            }
            // pinholes, all over but most low down
            for _ in 0..60 {
                let (x, y) = (rnd() * w, h * rnd().powf(0.6));
                ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.5 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.arc(x, y, 0.35 + rnd() * rnd() * 1.1, 0, 7);
                ctx.fill();
            }
            // along the cracks the silver flaked; round the blow more of it
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(255,255,255,0.55)");
                ctx.set_line_width(0.9 * k);
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.stroke();
            }
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                for _ in 0..10 {
                    let (a, d) = (rnd() * 7.0, rr * (1.0 + 2.5 * rnd()));
                    spot(ctx, x + a.cos() * d, y + a.sin() * d, 1.0 + rnd() * rr * 0.8, 0.4 + 0.4 * rnd());
                }
            }
        })
    })
}

/// The mirror's face: a film of dust, a wiped smear and finger marks, the
/// frame's wood seen again round its edge, and the breakage (each crack a
/// dark line where the silver broke and a bright one where its edge catches
/// the light, the crushed spot of the blow); clear elsewhere
fn wm_film_tex(b: &WmBreak, smudged: bool, gilt: bool, vary: u32) -> Tex {
    let b = b.clone();
    canvas_tex_opt(320, 484, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (gw, gh) = (WM_GLASS[0], WM_GLASS[1]);
            let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
            ctx.clear_rect(0, 0, w, h);
            // the moulding seen again in the glass, the glass's thickness a dark line between
            ctx.set_fill_style(if gilt { "rgba(128,104,58,0.5)" } else { "rgba(74,50,34,0.5)" });
            for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
                ctx.fill_rect(x, y, rw, rh);
            }
            ctx.set_stroke_style("rgba(18,16,14,0.45)");
            ctx.set_line_width(1.1);
            ctx.stroke_rect(3.5, 3.5, w - 7.0, h - 7.0);
            // dust, unevenly, and more of it lower down
            for _ in 0..14 {
                let (x, y, r) = (rnd() * w, h * rnd().powf(0.7), w * (0.12 + 0.3 * rnd()));
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(176,172,162,{:.3})", 0.05 + 0.06 * rnd()));
                g.add_color_stop(1, "rgba(176,172,162,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            // the last cleaning's swirls, faint; smudged: hands' smears
            // over them, a palm where someone leaned, fingers down its side
            ctx.set_line_cap("round");
            for _ in 0..if smudged { 5 } else { 2 } {
                let (x, y, r) = (w * (0.2 + 0.6 * rnd()), h * (0.3 + 0.5 * rnd()), w * (0.1 + 0.2 * rnd()));
                ctx.set_stroke_style(format!("rgba(186,182,172,{:.3})", if smudged { 0.04 + 0.04 * rnd() } else { 0.015 + 0.02 * rnd() }));
                ctx.set_line_width(8.0 + 12.0 * rnd());
                ctx.begin_path();
                let a = rnd() * 6.3;
                ctx.arc(x, y, r, a, a + 0.8 + rnd() * 1.4);
                ctx.stroke();
            }
            if smudged {
                for _ in 0..2 {
                    let side = rnd() < 0.5;
                    let x0 = if side { w * (0.05 + 0.08 * rnd()) } else { w * (0.87 - 0.08 * rnd()) };
                    let y0 = h * (0.25 + 0.4 * rnd());
                    let tilt = (rnd() - 0.5) * 0.6;
                    for f in 0..4 {
                        let stops = [
                            (0.0, "rgba(198,192,180,0.2)".to_string()),
                            (0.7, "rgba(198,192,180,0.1)".into()),
                            (1.0, "rgba(198,192,180,0)".into()),
                        ];
                        let (fx, fy) = (x0 + tilt * f as f64 * 11.0 + (rnd() - 0.5) * 3.0, y0 + f as f64 * 11.0);
                        wm_blob(ctx, fx, fy, 5.0 + rnd() * 2.0, 0.65, 1.4 + tilt, &stops);
                    }
                }
                let (px0, py0) = (w * (0.3 + 0.4 * rnd()), h * (0.45 + 0.25 * rnd()));
                let stops = [
                    (0.0, "rgba(198,192,180,0.12)".to_string()),
                    (0.6, "rgba(198,192,180,0.07)".into()),
                    (1.0, "rgba(198,192,180,0)".into()),
                ];
                wm_blob(ctx, px0, py0, 26.0 + 8.0 * rnd(), 0.8, rnd() * 3.0, &stops);
            }
            // the cracks: dark where the silver broke, bright where the edge catches the light
            ctx.set_line_join("round");
            let path = |ctx: &mut Canvas, pts: &[[f64; 2]], dx: f64, dy: f64| {
                ctx.begin_path();
                for (i, p) in pts.iter().enumerate() {
                    let (x, y) = px(*p);
                    if i == 0 {
                        ctx.move_to(x + dx, y + dy);
                    } else {
                        ctx.line_to(x + dx, y + dy);
                    }
                }
                ctx.stroke();
            };
            // each piece between the radials sits a hair off the others and
            // gives back the room a little differently
            let nr = b.radials.len();
            for i in 0..nr {
                let (ra, rb) = (&b.radials[i], &b.radials[(i + 1) % nr]);
                let v = if rnd() < 0.5 { "rgba(220,218,210," } else { "rgba(10,10,9," };
                ctx.set_fill_style(format!("{v}{:.3})", 0.02 + 0.035 * rnd()));
                ctx.begin_path();
                for (k, q) in ra.iter().chain(rb.iter().rev()).enumerate() {
                    let (x, y) = px(*q);
                    if k == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
            for (pts, k) in &b.lines {
                ctx.set_stroke_style("rgba(12,11,10,0.7)");
                ctx.set_line_width(1.2 * k);
                path(ctx, pts, 0.0, 0.0);
                ctx.set_stroke_style(format!("rgba(226,224,216,{:.2})", 0.28 + 0.22 * rnd()));
                ctx.set_line_width(0.6 * k);
                path(ctx, pts, 0.6, -0.5);
            }
            // the crushed spot: glass ground to white, splinters out of it
            for (p, cr) in &b.hits {
                let (x, y) = px(*p);
                let rr = cr / gw * w;
                ctx.set_fill_style("rgba(214,212,204,0.85)");
                ctx.begin_path();
                for i in 0..14 {
                    let a = i as f64 / 14.0 * PI * 2.0;
                    let q = rr * (0.6 + 0.6 * rnd());
                    if i == 0 {
                        ctx.move_to(x + a.cos() * q, y + a.sin() * q);
                    } else {
                        ctx.line_to(x + a.cos() * q, y + a.sin() * q);
                    }
                }
                ctx.close_path();
                ctx.fill();
                for _ in 0..28 {
                    let a = rnd() * 7.0;
                    let (r0, r1) = (rr * rnd(), rr * (1.0 + 1.6 * rnd()));
                    let v = if rnd() < 0.5 { 22 } else { 226 };
                    ctx.set_stroke_style(format!("rgba({v},{v},{},0.7)", v - 6));
                    ctx.set_line_width(0.6 + 0.4 * rnd());
                    ctx.begin_path();
                    ctx.move_to(x + a.cos() * r0, y + a.sin() * r0);
                    ctx.line_to(x + a.cos() * r1, y + a.sin() * r1);
                    ctx.stroke();
                }
            }
        })
    })
}

/// A moulded frame's wood as its front is mapped (the frame's bounds, `w` x
/// `h` m, its rails `mould` m wide, `ppm` px a metre): each rail's grain
/// along it, mitred at the corners, the joints opened a hair; stained `base`
/// (sRGB), its figure darker, the stain rubbed thin along its outer edge and
/// its sight edge, knocks and scratches, dust along the top rail. `vary`
/// mixes the frame's place into its stream
fn moulding_grain_tex(w: f64, h: f64, mould: f64, base: [u8; 3], vary: u32, ppm: f64) -> Tex {
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let m = mould * ppm;
            let [r0, g0, b0] = base.map(f64::from);
            let col = |k: f64, a: f64| format!("rgba({:.0},{:.0},{:.0},{a:.3})", r0 * k, g0 * k, b0 * k);
            ctx.set_fill_style(col(1.0, 1.0));
            ctx.fill_rect(0, 0, cw, ch);
            // the four rails: (outline, along x?)
            let rails: [([(f64, f64); 4], bool); 4] = [
                ([(0.0, 0.0), (cw, 0.0), (cw - m, m), (m, m)], true),
                ([(0.0, ch), (cw, ch), (cw - m, ch - m), (m, ch - m)], true),
                ([(0.0, 0.0), (m, m), (m, ch - m), (0.0, ch)], false),
                ([(cw, 0.0), (cw - m, m), (cw - m, ch - m), (cw, ch)], false),
            ];
            for (poly, along_x) in rails.iter() {
                ctx.save();
                ctx.begin_path();
                ctx.move_to(poly[0].0, poly[0].1);
                for q in &poly[1..] {
                    ctx.line_to(q.0, q.1);
                }
                ctx.close_path();
                ctx.clip();
                // each rail its own board: a shade of its own
                ctx.set_fill_style(col(0.88 + 0.24 * rnd(), 0.5));
                ctx.fill_rect(0, 0, cw, ch);
                let len = if *along_x { cw } else { ch };
                let (lo, hi) = match (poly[0], along_x) {
                    ((_, y), true) if y < 1.0 => (0.0, m),
                    (_, true) => (ch - m, ch),
                    ((x, _), false) if x < 1.0 => (0.0, m),
                    _ => (cw - m, cw),
                };
                // the grain: long wavering lines along the rail, darker figure among them
                let lines = (m * 2.2) as usize;
                for _ in 0..lines {
                    let across = lo + rnd() * (hi - lo);
                    let (amp, fr, ph) = (0.4 + rnd() * 1.6, 1.0 + rnd() * 3.0, rnd() * 6.3);
                    let dark = rnd() < 0.3;
                    ctx.set_stroke_style(col(if dark { 0.55 } else { 0.72 + 0.2 * rnd() }, if dark { 0.35 } else { 0.18 + 0.2 * rnd() }));
                    ctx.set_line_width(if dark { 0.8 + rnd() * 1.4 } else { 0.4 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = (len / 8.0) as usize + 2;
                    for k in 0..=n {
                        let t = k as f64 / n as f64 * len;
                        let a = across + amp * (t / len * fr * 6.28 + ph).sin();
                        let (x, y) = if *along_x { (t, a) } else { (a, t) };
                        if k == 0 {
                            ctx.move_to(x, y);
                        } else {
                            ctx.line_to(x, y);
                        }
                    }
                    ctx.stroke();
                }
                // pores, short dark flecks along the grain
                for _ in 0..(len * m / 60.0) as usize {
                    let (t, a) = (rnd() * len, lo + rnd() * (hi - lo));
                    let l = 1.5 + rnd() * 4.0;
                    ctx.set_stroke_style(col(0.45, 0.25 + 0.2 * rnd()));
                    ctx.set_line_width(0.6);
                    ctx.begin_path();
                    let (x, y, dx, dy) = if *along_x { (t, a, l, 0.0) } else { (a, t, 0.0, l) };
                    ctx.move_to(x, y);
                    ctx.line_to(x + dx, y + dy);
                    ctx.stroke();
                }
                ctx.restore();
            }
            // the mitres: each joint opened a hair, grime in it
            ctx.set_stroke_style("rgba(20,14,10,0.55)");
            ctx.set_line_width(0.9);
            for (a, b) in [((0.0, 0.0), (m, m)), ((cw, 0.0), (cw - m, m)), ((0.0, ch), (m, ch - m)), ((cw, ch), (cw - m, ch - m))] {
                ctx.begin_path();
                ctx.move_to(a.0, a.1);
                ctx.line_to(b.0, b.1);
                ctx.stroke();
            }
            // the stain rubbed thin along the outer edge and the sight edge
            ctx.set_line_width(2.4);
            for inset in [1.2, m - 1.4] {
                ctx.set_stroke_style("rgba(222,196,160,0.14)");
                ctx.stroke_rect(inset, inset, cw - 2.0 * inset, ch - 2.0 * inset);
            }
            // knocks and scratches
            for _ in 0..(cw + ch) as usize / 14 {
                let edge = rnd() < 0.7;
                let (x, y) = if rnd() < 0.5 {
                    (
                        rnd() * cw,
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            ch - 1.0
                        } else {
                            ch - rnd() * m
                        },
                    )
                } else {
                    (
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            cw - 1.0
                        } else {
                            cw - rnd() * m
                        },
                        rnd() * ch,
                    )
                };
                if rnd() < 0.5 {
                    let stops = [(0.0, "rgba(214,182,140,0.45)".to_string()), (1.0, "rgba(214,182,140,0)".into())];
                    wm_blob(ctx, x, y, 0.8 + rnd() * 2.2, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
                } else {
                    let a = rnd() * 6.3;
                    let l = 3.0 + rnd() * 12.0;
                    ctx.set_stroke_style(format!("rgba(200,170,130,{:.2})", 0.12 + 0.18 * rnd()));
                    ctx.set_line_width(0.5);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            }
            // dust along the top rail, and grime where hands set it straight
            let gd = ctx.create_linear_gradient(0, 0, 0, m);
            gd.add_color_stop(0, "rgba(168,162,150,0.32)");
            gd.add_color_stop(1, "rgba(168,162,150,0.08)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(m * 0.4, 0, cw - m * 0.8, m);
            for x in [m * 0.6, cw - m * 0.6] {
                let stops = [(0.0, "rgba(28,20,14,0.22)".to_string()), (1.0, "rgba(28,20,14,0)".into())];
                wm_blob(ctx, x, ch * (0.55 + 0.2 * rnd()), m * 1.4, 0.6, PI / 2.0, &stops);
            }
        })
    })
}

/// The wall mirror: its size, its moulding's width, its glass as seen in
/// the frame (w, h) and its face's distance from the wall
/// (`Real__wallMirrorFrameWood` / `Gilt`, r13a.py), and how much more glass
/// runs under the lip (both sides)
const WM_SIZE: [f64; 2] = [0.420, 0.600];
const WM_MOULD: f64 = 0.034;
const WM_GLASS: [f64; 3] = [0.352, 0.532, 0.0090];
const WM_UNDER: f64 = 0.012;

/// The gilt frame as its front is mapped (its bounds): old gold leaf on red
/// bole over gesso, dulled, burnished on the beads and the cushion's crest
/// and rubbed through there to the bole, chipped to the white gesso along
/// its outer edge, grime in its quirks, dust on its top. Each pixel from
/// where on the moulding's profile it lies (its distance in from the outer
/// edge), no canvas call in the loop
fn wm_gilt_tex(vary: u32) -> Tex {
    let ppm = 900.0;
    canvas_tex_opt((WM_SIZE[0] * ppm).round(), (WM_SIZE[1] * ppm).round(), false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let (n, m) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) / ppm;
            for i in 0..n {
                let x = (i as f64 + 0.5) / ppm;
                let d = x.min(WM_SIZE[0] - x).min(y).min(WM_SIZE[1] - y);
                let bump = |c: f64, s: f64| (-((d - c) / s).powi(2)).exp();
                // the beads and the cushion's crest, the quirks between
                let crest = bump(0.0057, 0.0016).max(bump(0.0200, 0.0042)).max(bump(0.0303, 0.0016));
                let recess = bump(0.0086, 0.0009).max(bump(0.0277, 0.0009)).max(bump(0.0340, 0.0008));
                let (u, v) = (x * 900.0, y * 900.0);
                let mot = fbm(seed ^ 0x11, u * 0.05, v * 0.05, 3);
                let gold = [146.0, 116.0, 62.0];
                let mut c = [0.0; 3];
                for k in 0..3 {
                    c[k] = gold[k] * (0.82 + 0.3 * crest + 0.16 * (mot - 0.5));
                }
                // rubbed through to the red bole on the high places
                let rub = crest * sstep(0.56, 0.7, fbm(seed ^ 0x12, u * 0.09, v * 0.09, 3));
                let bole = [118.0, 62.0, 40.0];
                // chipped to the gesso along the outer edge, a knock here and there
                let chip = sstep(0.0035, 0.0015, d) * sstep(0.62, 0.72, fbm(seed ^ 0x13, u * 0.12, v * 0.12, 2))
                    + sstep(0.86, 0.9, vnoise(seed ^ 0x14, u * 0.06, v * 0.06)) * crest;
                let gesso = [204.0, 194.0, 168.0];
                // grime in the quirks and over it all in patches; dust on the top rail
                let grime = (recess * 0.75 + 0.25 * sstep(0.5, 0.8, fbm(seed ^ 0x15, u * 0.02, v * 0.02, 2))).min(0.9);
                let dust = if y < WM_MOULD { 0.22 + 0.1 * (1.0 - crest) } else { 0.04 } * (0.6 + 0.4 * mot);
                for k in 0..3 {
                    let mut t = c[k] + (bole[k] - c[k]) * rub;
                    t += (gesso[k] - t) * chip.min(1.0);
                    t *= 1.0 - 0.55 * grime;
                    t += ([150.0, 146.0, 136.0][k] - t) * dust;
                    out[i * 4 + k] = t.round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        let _ = m;
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The basin's glaze as its top is mapped (x across, z out from the wall:
/// the body 0.5 x 0.4 m, or the bowl's inside, 0.38 x 0.27 m from 0.105 m
/// out): off-white vitreous china gone grey, its glaze crazed, grime in
/// patches; in the bowl the rust-brown runs where the taps dripped for years
/// down to the plug, and scale round the plug; dust on the deck
fn wm_basin_tex(vary: u32, bowl: bool) -> Tex {
    let (cw, chh) = if bowl { (196.0, 140.0) } else { (256.0, 205.0) };
    canvas_tex_opt(cw, chh, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let px =
                |x: f64, z: f64| if bowl { ((x + 0.19) / 0.38 * w, (z - 0.105) / 0.27 * h) } else { ((x + 0.25) / 0.5 * w, z / 0.4 * h) };
            ctx.set_fill_style("#d6d1c4");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..14 {
                let (x, y, r) = (rnd() * w, rnd() * h, 20.0 + rnd() * 60.0);
                let stops = [(0.0, format!("rgba(120,112,96,{:.3})", 0.05 + 0.08 * rnd())), (1.0, "rgba(120,112,96,0)".into())];
                wm_blob(ctx, x, y, r, 0.6 + 0.4 * rnd(), rnd() * 3.0, &stops);
            }
            // crazing: a fine network in the glaze
            ctx.set_line_width(0.5);
            for _ in 0..60 {
                let (mut x, mut y) = (rnd() * w, rnd() * h);
                ctx.set_stroke_style(format!("rgba(90,84,74,{:.2})", 0.1 + 0.15 * rnd()));
                ctx.begin_path();
                ctx.move_to(x, y);
                let mut a = rnd() * 6.3;
                for _ in 0..4 {
                    a += (rnd() - 0.5) * 1.6;
                    x += a.cos() * (4.0 + rnd() * 8.0);
                    y += a.sin() * (4.0 + rnd() * 8.0);
                    ctx.line_to(x, y);
                }
                ctx.stroke();
            }
            if !bowl {
                let gd = ctx.create_linear_gradient(0, 0, 0, h * 0.3);
                gd.add_color_stop(0, "rgba(150,146,134,0.35)");
                gd.add_color_stop(1, "rgba(150,146,134,0)");
                ctx.set_fill_style(&gd);
                ctx.fill_rect(0, 0, w, h * 0.3);
                return;
            }
            // the brown the dripping left, pooled toward the plug; short runs under the taps
            let (pxp, pyp) = px(0.0, 0.25);
            for _ in 0..6 {
                let (x, y) = (pxp + (rnd() - 0.5) * w * 0.25, pyp + (rnd() - 0.5) * h * 0.3);
                let stops = [(0.0, format!("rgba(122,82,46,{:.3})", 0.1 + 0.08 * rnd())), (1.0, "rgba(122,82,46,0)".into())];
                wm_blob(ctx, x, y, w * (0.1 + 0.12 * rnd()), 0.7 + 0.3 * rnd(), rnd() * 3.0, &stops);
            }
            ctx.set_line_cap("round");
            for sx in [-1.0, 1.0] {
                let (x0, _) = px(sx * 0.08, 0.105);
                let g = ctx.create_linear_gradient(0, 0, 0, h * 0.3);
                g.add_color_stop(0, "rgba(118,72,38,0.32)");
                g.add_color_stop(1, "rgba(118,72,38,0)");
                ctx.set_stroke_style(&g);
                ctx.set_line_width(5.0 + 3.0 * rnd());
                ctx.begin_path();
                ctx.move_to(x0, 0);
                ctx.line_to(x0 - sx * 4.0, h * 0.3);
                ctx.stroke();
            }
            let stops =
                [(0.0, "rgba(176,164,138,0.0)".to_string()), (0.5, "rgba(176,164,138,0.45)".into()), (1.0, "rgba(176,164,138,0)".into())];
            wm_blob(ctx, pxp, pyp, 18.0, 1.0, 0.0, &stops);
        })
    })
}

/// A drip's stain dried on the floor: a soft, ragged, darker patch, its
/// tide line a little darker, clear round it (transparent)
fn wm_stain_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.clear_rect(0, 0, w, h);
            for _ in 0..9 {
                let (x, y) = (w * (0.35 + 0.3 * rnd()), h * (0.35 + 0.3 * rnd()));
                let r = w * (0.12 + 0.16 * rnd());
                let a = 0.12 + 0.12 * rnd();
                let stops = [
                    (0.0, format!("rgba(26,28,30,{a:.3})")),
                    (0.75, format!("rgba(26,28,30,{:.3})", a * 0.8)),
                    (1.0, "rgba(26,28,30,0)".into()),
                ];
                wm_blob(ctx, x, y, r, 0.6 + 0.4 * rnd(), rnd() * 3.0, &stops);
            }
            ctx.set_stroke_style("rgba(22,22,22,0.12)");
            ctx.set_line_width(1.6);
            ctx.begin_path();
            let r0 = w * 0.34;
            for k in 0..=40 {
                let a = k as f64 / 40.0 * PI * 2.0;
                let rr = r0 * (0.85 + 0.15 * (a * 3.0 + rnd() * 0.4).sin());
                let (x, y) = (w / 2.0 + a.cos() * rr, h / 2.0 + a.sin() * rr * 0.8);
                if k == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();
        })
    })
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
