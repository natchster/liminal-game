//! Prop placers: extension cords, walkie-talkies, rolled carpets, abandoned
//! laptops, umbrellas, window signs, pinned photos, handprints and wall mirrors.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod tex;
use tex::*;

mod r11a;
use r11a::place_wall_handprints;
mod r13a;
use r13a::place_wall_mirrors;
mod r13b;
use r13b::place_rolled_carpets;

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

use placers_kit::place::pick;

/// Extension cord pool per weather; other keys get `[0x181410, 0x201818, 0x202020, 0x141014]`
fn extension_cord_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x0a0f18, 0x101820],
        "electric" => &[0x201808, 0x282010],
        "haunted" => &[0x1a0808, 0x201014],
        "derelict" => &[0x181410, 0x201818],
        // still / normal have no entry → the baseline
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

// Extension cord: 20% chance per level, 1 coiled cord on floor.
fn place_extension_cords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
      // 30% chance: near a tracked socket; 25%: near a tracked laptop.
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
    // Per-weather cord palette biases the color pool.
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

// Walkie talkie: 12% chance per level, 1 handheld radio on floor.
fn place_walkie_talkies(g: &mut Game, _dist: &[i32]) {
    g.pp.walkie_talkie_positions.clear(); // reset per level
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
    // Track for static-crackle audio events.
    let p = wt.pos();
    g.pp.walkie_talkie_positions.push(v3(p.x, 0.10, p.z));
}

// Abandoned laptop: 12% chance per level, 1 laptop on the floor.
fn place_abandoned_laptops(g: &mut Game, _dist: &[i32]) {
    g.pp.laptop_positions.clear(); // reset per level
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
    // 8% chance the laptop is open with a faint blue screen glow.
    let is_open = rnd() < 0.08;
    if is_open {
        // Keyboard overlay on top face of base.
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
        // 50% chance a garbled-text canvas variant, else faint blue.
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
        // 30% of garbled screens: brief periodic tint flicker.
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
        // 30% chance screen flicker — subtle intensity dip pattern.
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
        // Closed variant: the stacked lid.
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
    // Track laptop position for extension cord co-loc.
    let lp = laptop.pos();
    g.pp.laptop_positions.push(v3(lp.x, 0.02, lp.z));
    // 20% chance a briefcase spawns adjacent to the laptop.
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
    // 25% chance a coffee cup on floor near laptop.
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

// Leaning umbrella: 15% chance per level, 1 closed umbrella against a
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
    // Bias primary color to per-weather palette, 20% grab-bag.
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
    // 30% chance: the wet it dripped, dried to a mark round its ferrule
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

// Window sign: 8% chance per level, 1 OPEN/CLOSED placard on a wall.
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
    // Slight ±0.05 rad Z tilt for crooked-hang aesthetic.
    sign.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&sign);
    // 8% chance: cracked-glass overlay atop the sign.
    if rnd() < 0.08 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.22, 0.11), &crack_mat);
        crack.position().copy(&sign.pos());
        crack.rotation().copy(&sign.rot());
        // Nudge outward 2 mm from wall so overlay renders in front.
        crack.position().x -= c.dx as f64 * 0.002;
        crack.position().z -= c.dz as f64 * 0.002;
        g.world.add(&crack);
        // 2-4 tiny floor shards beneath cracked sign.
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

// Pinned photos: 12% chance per level, 1 polaroid pinned to a wall.
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
    // Per-weather color tint via material multiply.
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
    // 5% chance: cracked-glass overlay atop the photo.
    if rnd() < 0.05 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.14, 0.14), &crack_mat);
        crack.position().copy(&photo.pos());
        crack.rotation().copy(&photo.rot());
        crack.position().x -= c.dx as f64 * 0.001;
        crack.position().z -= c.dz as f64 * 0.001;
        g.world.add(&crack);
        // 2-3 tiny floor shards beneath cracked photo.
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

use placers_kit::noise::hash as hash01;

use placers_kit::noise::vn as vnoise;

use placers_kit::noise::fbm;

use placers_kit::noise::sstep;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
