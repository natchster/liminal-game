//! Prop placers: briefcases, keycard readers, dead vending machines, wall phones,
//! mops, dead plants, ceiling speakers, handrails, first-aid kits, restroom
//! signs, cabinet doors, wet-floor signs and emergency lights.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_briefcases", place_briefcases));
    r.push(("place_keycard_readers", place_keycard_readers));
    r.push(("place_dead_vending_machines", place_dead_vending_machines));
    r.push(("place_wall_phones", place_wall_phones));
    r.push(("place_mops", place_mops));
    r.push(("place_dead_plants", place_dead_plants));
    r.push(("place_ceiling_speakers", place_ceiling_speakers));
    r.push(("place_handrails", place_handrails));
    r.push(("place_first_aid_kits", place_first_aid_kits));
    r.push(("place_restroom_signs", place_restroom_signs));
    r.push(("place_cabinet_doors", place_cabinet_doors));
    r.push(("place_wet_floor_signs", place_wet_floor_signs));
    r.push(("place_emergency_lights", place_emergency_lights));
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

/// In-place Fisher-Yates, `for (i = n-1; i > 0; i--) { j = floor(rnd * (i+1)); swap }`
fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// Smoke wisp per weather
fn smoke_wisp_by_weather(key: &str) -> u32 {
    by_weather(key, 0x505860, 0x707068, 0x704040, 0x604838, 0x606058, 0x606058)
}
/// Plant glow per weather
fn plant_glow_by_weather(key: &str) -> u32 {
    by_weather(key, 0x40a080, 0x80c040, 0x9040c0, 0xa08040, 0xd0e0d0, 0x60a040)
}
/// Goggle lens per weather
fn goggle_lens_by_weather(key: &str) -> u32 {
    by_weather(key, 0x6890a8, 0xd8c880, 0xa87070, 0x907050, 0x8ac0d8, 0x8ac0d8)
}
/// Oil stain per weather
fn oil_stain_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0c10, 0x0a0805, 0x1a0605, 0x100a08, 0x0a0806, 0x0a0806)
}

// R282 abandoned briefcase: 15% chance per level.
fn place_briefcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let leather_mat = lambert!({ color: 0x1e1610 });
    let clip_mat = lambert!({ color: 0x8a7a48 });
    let handle_mat = lambert!({ color: 0x281a12 });
    let briefcase = group();
    // Body: 0.36 wide × 0.10 thick × 0.28 deep.
    let body = mesh(pg("Briefcases__body", box_geo!(0.36, 0.10, 0.28)), &leather_mat);
    body.position().y = 0.05;
    briefcase.add(&body);
    // Top seam (thin darker line along top edge).
    let seam = mesh(pg("Briefcases__rim", box_geo!(0.36, 0.008, 0.28)), &handle_mat);
    seam.position().y = 0.10;
    briefcase.add(&seam);
    // 2 clasps on top-front.
    for cs in [-1.0, 1.0] {
        let clasp = mesh(pg("Briefcases__clasp", box_geo!(0.03, 0.01, 0.02)), &clip_mat);
        clasp.position().set(cs * 0.10, 0.106, 0.14);
        briefcase.add(&clasp);
    }
    // Handle: arch on top.
    let handle_arch = mesh(pg("Briefcases__handle", torus_geo!(0.05, 0.005, 4, 12, PI)), &handle_mat);
    handle_arch.rotation().x = -PI / 2.0;
    handle_arch.position().set(0, 0.109, 0); // lying on the rim's top (y 0.104), tube r 0.005
    briefcase.add(&handle_arch);
    // 15% chance briefcase tipped on side.
    let is_tipped = rnd() < 0.15;
    if is_tipped {
        briefcase.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
        briefcase.position().y = 0.18; // local x (±0.18, the width) becomes height
    }
    // R283 10% chance briefcase is OPEN with scrap papers spilling out (not when tipped).
    let is_open = !is_tipped && rnd() < 0.10;
    if is_open {
        let lid = mesh(pg("Briefcases__lid", box_geo!(0.36, 0.02, 0.28)), &leather_mat);
        let lid_group = group();
        // the lid reaches forward from the hinge (z 0…0.28) so it swings up behind the case
        lid.position().set(0, 0.01, 0.14);
        lid_group.add(&lid);
        // the carry handle rides on the lid's outer face
        lid_group.add(&handle_arch);
        handle_arch.position().set(0, 0.025, 0.14);
        lid_group.position().set(0, 0.10, -0.14); // pivot at back top edge
        lid_group.rotation().x = -PI / 1.8; // ~100° open
        briefcase.add(&lid_group);
    }
    // Wall-corner bias for realistic "set down against wall" placement.
    let corner_bias = rnd() < 0.70;
    let ox = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.1 + rnd() * 0.4) } else { (rnd() - 0.5) * 2.2 };
    let oz = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.1 + rnd() * 0.4) } else { (rnd() - 0.5) * 2.2 };
    briefcase.position().x = g.wx(pk.0) + ox;
    briefcase.position().z = g.wz(pk.1) + oz;
    briefcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&briefcase);
    // R283 open briefcase spills 2-3 scrap papers around it.
    if is_open {
        let paper_mat = lambert!({ color: 0xece0d0 });
        let scrap_count = 2 + (rnd() * 2.0).floor() as i32;
        let bp = briefcase.pos();
        for _sp in 0..scrap_count {
            let scrap = mesh(pg("Briefcases__scrap", plane_geo!(0.10 + rnd() * 0.04, 0.13 + rnd() * 0.05)), &paper_mat);
            scrap.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.2;
            scrap.rotation().z = rnd() * PI * 2.0;
            let sc_ang = rnd() * PI * 2.0;
            let sc_rad = 0.15 + rnd() * 0.25;
            scrap.position().set(bp.x + sc_ang.cos() * sc_rad, 0.003, bp.z + sc_ang.sin() * sc_rad);
            g.world.add(&scrap);
        }
    }
}

// R282 keycard reader: 25% chance per level, 1-2 units.
fn place_keycard_readers(g: &mut Game, _dist: &[i32]) {
    g.pp.keycard_reader_positions.clear(); // R283 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut cands = wall_cands(g);
    shuffle(&mut cands);
    let body_mat = lambert!({ color: 0x2a2828 });
    let slot_mat = lambert!({ color: 0x8a8880 });
    // R284 per-weather LED color for readers (shares R259 palette).
    let led_color = led_color_by_weather(g.weather_key());
    let red_led_mat = basic_mat!({ color: led_color });
    let mut i = 0;
    while i < count && i < cands.len() {
        let c = cands[i];
        let reader = group();
        let body = mesh(pg("KeycardReaders__body", box_geo!(0.10, 0.14, 0.03)), &body_mat);
        reader.add(&body);
        // Card slot (horizontal thin band, upper portion).
        let slot = mesh(plane_geo!(0.08, 0.008), &slot_mat);
        slot.position().set(0, 0.03, 0.016);
        reader.add(&slot);
        // Tiny red LED near bottom.
        let led = mesh(box_geo!(0.008, 0.008, 0.008), &red_led_mat);
        led.position().set(0, -0.04, 0.017);
        reader.add(&led);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        let reader_height = 1.2 + rnd() * 0.2;
        if c.dx.abs() == 1 {
            reader.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            reader.position().set(wall_x, reader_height, wall_z + (rnd() - 0.5) * 1.2);
        } else {
            reader.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            reader.position().set(wall_x + (rnd() - 0.5) * 1.2, reader_height, wall_z);
        }
        g.world.add(&reader);
        // R283 track for beep audio events.
        let rp = reader.pos();
        g.pp.keycard_reader_positions.push(v3(rp.x, reader_height, rp.z));
        i += 1;
    }
}

// R281 cracked-glass overlay texture for vending displays.
game::lazy_static_tex!(
    vending_crack_texture,
    canvas_tex_opt(128, 192, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let cx = 40.0 + rnd() * (w - 80.0);
        let cy = 60.0 + rnd() * (h - 120.0);
        ctx.set_stroke_style("rgba(240, 240, 240, 0.55)");
        ctx.set_line_width(1);
        // 8 radial cracks with slight jitter.
        for r in 0..8 {
            let a = (r as f64 / 8.0) * PI * 2.0 + rnd() * 0.5;
            let len = 30.0 + rnd() * 60.0;
            ctx.begin_path();
            ctx.move_to(cx, cy);
            let (mut x, mut y) = (cx, cy);
            for _step in 0..8 {
                x += (a + (rnd() - 0.5) * 0.3).cos() * (len / 8.0);
                y += (a + (rnd() - 0.5) * 0.3).sin() * (len / 8.0);
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        // Small concentric ring near impact.
        ctx.set_stroke_style("rgba(240, 240, 240, 0.35)");
        ctx.begin_path();
        ctx.arc(cx, cy, 5, 0, PI * 2.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(cx, cy, 10, 0, PI * 2.0);
        ctx.stroke();
    })
);

// R279 dead vending machine: 12% chance per level, 1 unit against a wall.
fn place_dead_vending_machines(g: &mut Game, _dist: &[i32]) {
    g.pp.vending_positions.clear(); // R280 reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let body_mat = lambert!({ color: 0x3a3838 });
    let display_mat = lambert!({ color: 0x0a0a0a });
    let trim_mat = lambert!({ color: 0x606058 });
    let button_mat = lambert!({ color: 0x2a2828 });
    let machine = group();
    // Main body: 0.90 wide × 1.85 tall × 0.55 deep.
    let body = mesh(pg("DeadVendingMachines__cabinet", box_geo!(0.90, 1.85, 0.55)), &body_mat);
    body.position().y = 0.925;
    machine.add(&body);
    // Top display window (dark glass, most of upper front).
    let display = mesh(plane_geo!(0.78, 1.0), &display_mat);
    display.position().set(0, 1.20, 0.276);
    machine.add(&display);
    // R281 20% chance cracked glass overlay on display.
    if rnd() < 0.20 {
        let crack_mat = basic_mat!({ map: vending_crack_texture(), transparent: true, opacity: 0.85, depthWrite: false });
        let crack = mesh(plane_geo!(0.78, 1.0), &crack_mat);
        crack.position().set(0, 1.20, 0.278);
        machine.add(&crack);
    }
    // Faint scanline pattern in display via lighter horizontal strip.
    let scan = mesh(plane_geo!(0.78, 0.008), &trim_mat);
    scan.position().set(0, 1.35, 0.277);
    machine.add(&scan);
    // Coin slot (small vertical dark rectangle).
    let coin_slot = mesh(plane_geo!(0.02, 0.05), &display_mat);
    coin_slot.position().set(0.32, 0.90, 0.276);
    machine.add(&coin_slot);
    // Button pad (3x3 grid of small dark squares below display).
    for bx in 0..3 {
        for by in 0..3 {
            let btn = mesh(pg("DeadVendingMachines__button", box_geo!(0.06, 0.06, 0.008)), &button_mat);
            btn.position().set(-0.22 + bx as f64 * 0.08, 0.66 + by as f64 * 0.08, 0.278);
            machine.add(&btn);
        }
    }
    // Delivery tray (dark opening near bottom).
    let tray = mesh(plane_geo!(0.36, 0.14), &display_mat);
    tray.position().set(0, 0.28, 0.276);
    machine.add(&tray);
    // Bottom trim.
    let bottom_trim = mesh(pg("DeadVendingMachines__plinth", box_geo!(0.90, 0.06, 0.55)), &trim_mat);
    bottom_trim.position().y = 0.03;
    machine.add(&bottom_trim);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.28);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.28);
    if c.dx.abs() == 1 {
        machine.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        machine.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        machine.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        machine.position().set(wall_x + (rnd() - 0.5) * 0.6, 0, wall_z);
    }
    g.world.add(&machine);
    let mp = machine.pos();
    let m_rot_y = machine.rot().y;
    // R280 20% chance this machine is "still powered" — hum audio + R769 interactive.
    if rnd() < 0.20 {
        g.pp.vending_positions.push(v3(mp.x, 0.9, mp.z));
        // R769 interactable — E dispenses a random item with 4 outcomes.
        let used = Rc::new(Cell::new(false));
        let mut use_pos = mp;
        use_pos.y = 1.0;
        let used_e = used.clone();
        g.interact(
            use_pos,
            1.5,
            "[E] BUY",
            move |_g| !used_e.get(),
            move |g| {
                used.set(true);
                let roll = rnd();
                if g.ac.is_some() {
                    // Coin drop + button click.
                    g.noise_burst(0.03, 3000.0, 0.6, 0.08, "bandpass", None, 0.0, 0.0);
                    g.blip(1400.0, 0.05, "square", 0.08, None, 0.05, 0.0);
                }
                if roll < 0.40 {
                    // Almond water — the player picks it up automatically.
                    g.player.bottles = g.player.bottles + 1;
                    g.player.sanity = clamp(g.player.sanity + 20.0, 0.0, SAN_MAX);
                    if g.ac.is_some() {
                        g.blip(600.0, 0.10, "sine", 0.10, None, 0.35, 0.0); // dispensing thunk
                        g.blip(400.0, 0.15, "sine", 0.08, None, 0.55, 0.0);
                    }
                    g.show_msg("ALMOND WATER — +1 BOTTLE", 2.0);
                } else if roll < 0.65 {
                    // Battery.
                    g.player.batteries = g.player.batteries + 1;
                    if g.ac.is_some() {
                        g.blip(800.0, 0.10, "square", 0.10, None, 0.35, 0.0);
                        g.noise_burst(0.05, 200.0, 0.5, 0.06, "lowpass", Some(100.0), 0.40, 0.0);
                    }
                    g.show_msg("BATTERY — +1", 2.0);
                } else if roll < 0.90 {
                    // Nothing — stuck.
                    if g.ac.is_some() {
                        g.noise_burst(0.15, 400.0, 0.8, 0.10, "lowpass", Some(200.0), 0.30, 0.0);
                        g.blip(220.0, 0.30, "sawtooth", 0.08, Some(160.0), 0.35, 0.0);
                    }
                    g.show_msg("STUCK. IT KEEPS YOUR COIN.", 2.2);
                    // Small sanity ping (frustration).
                    g.player.sanity = clamp(g.player.sanity - 4.0, 0.0, SAN_MAX);
                    // Not "used" up — allow retry after cooldown.
                    let used_t = used.clone();
                    g.schedule(8000.0, move |_g| {
                        used_t.set(false);
                    });
                } else {
                    // Mercury-color soda — 20s speed boost + brief warble sfx.
                    g.player.speed_boost_until = g.perf_t + 20.0;
                    if g.ac.is_some() {
                        g.blip(1000.0, 0.18, "sine", 0.10, Some(1400.0), 0.35, 0.0);
                        g.blip(1400.0, 0.15, "sine", 0.09, Some(1800.0), 0.55, 0.0);
                        g.blip(1800.0, 0.12, "sine", 0.07, Some(2200.0), 0.72, 0.0);
                    }
                    g.show_msg("MERCURY SODA — YOU FEEL FAST", 2.6);
                }
            },
        );
    }
    // R280 30% chance 1-2 coffee cups near the machine.
    if rnd() < 0.30 {
        let cup_count = 1 + (rnd() * 2.0).floor() as i32;
        for _cn in 0..cup_count {
            let vc_col = [0xece0d0u32, 0xa03018, 0xf0eee4][(rnd() * 3.0).floor() as usize];
            let vc_mat = lambert!({ color: vc_col });
            let vcup = group();
            let vcup_body = mesh(pg("CoffeeCups__cup", cyl_geo!(0.038, 0.028, 0.09, 10)), &vc_mat);
            vcup_body.position().y = 0.045;
            vcup.add(&vcup_body);
            let vcoffee = mesh(circle_geo!(0.032, 10), lambert!({ color: 0x2a1810 }));
            vcoffee.rotation().x = -PI / 2.0;
            vcoffee.position().y = 0.091;
            vcup.add(&vcoffee);
            // Position 0.4-0.9 m in front of machine.
            let (fx, fz) = (-m_rot_y.sin(), -m_rot_y.cos());
            let cdist = 0.4 + rnd() * 0.5;
            vcup.position().set(mp.x + fx * cdist + (rnd() - 0.5) * 0.5, 0, mp.z + fz * cdist + (rnd() - 0.5) * 0.5);
            vcup.rotation().y = rnd() * PI * 2.0;
            g.world.add(&vcup);
        }
    }
    // R383 30% chance: 3-6 tiny coins scattered on the floor near the machine.
    if rnd() < 0.30 {
        let coin_count = 3 + (rnd() * 4.0).floor() as i32;
        for _cn in 0..coin_count {
            let coin_tones: [u32; 3] = [0xb8a840, 0xa08040, 0xc8b850]; // brass/copper/gold
            let coin_mat = lambert!({ color: pick(&coin_tones) });
            let coin = mesh(pg("DeadVendingMachines__coin", cyl_geo!(0.011, 0.011, 0.002, 8)), &coin_mat);
            let c_ang = rnd() * PI * 2.0;
            let c_rad = 0.20 + rnd() * 0.50;
            let (cfx, cfz) = (-m_rot_y.sin(), -m_rot_y.cos());
            coin.position().set(
                mp.x + cfx * c_rad + c_ang.cos() * (rnd() * 0.3),
                0.001 + rnd() * 0.001,
                mp.z + cfz * c_rad + c_ang.sin() * (rnd() * 0.3),
            );
            coin.rotation().x = (rnd() - 0.5) * 0.05;
            coin.rotation().z = (rnd() - 0.5) * 0.05;
            g.world.add(&coin);
        }
    }
}

// R277 wall phone: 15% chance per level, 1 unit.
fn place_wall_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.phone_positions.clear(); // R278 reset per level
    g.pp.broken_phone_positions.clear(); // R325 reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let body_mat = lambert!({ color: 0x2a2828 });
    let handset_mat = lambert!({ color: 0x1a1a1a });
    let cord_mat = lambert!({ color: 0x3a3838 });
    let phone = group();
    // Base body: 0.14 wide × 0.20 tall × 0.06 deep.
    let body = mesh(pg("WallPhones__body", box_geo!(0.14, 0.20, 0.06)), &body_mat);
    phone.add(&body);
    // Small keypad face (lighter rectangle).
    let keypad = mesh(plane_geo!(0.10, 0.08), lambert!({ color: 0x8a8878 }));
    keypad.position().set(0, -0.02, 0.031);
    phone.add(&keypad);
    // R303 5% chance: broken phone with cracked keypad + no handset.
    let is_broken = rnd() < 0.05;
    if is_broken {
        let crack_mat = basic_mat!({ color: 0x1a1010, transparent: true, opacity: 0.9, depthWrite: false });
        for _ck in 0..3 {
            let crack = mesh(plane_geo!(0.001 + rnd() * 0.002, 0.06), &crack_mat);
            crack.rotation().z = rnd() * PI;
            crack.position().set((rnd() - 0.5) * 0.08, -0.02 + (rnd() - 0.5) * 0.06, 0.032);
            phone.add(&crack);
        }
        // R335 40% chance: tiny gray smoke wisp above the damaged keypad.
        if rnd() < 0.40 {
            // R342 per-weather smoke color.
            let smoke_color = smoke_wisp_by_weather(g.weather_key());
            let smoke_mat = basic_mat!({ color: smoke_color, transparent: true, opacity: 0.35, depthWrite: false });
            let wisp = mesh(cyl_geo!(0.008, 0.018, 0.09, 6), &smoke_mat);
            let base_y = 0.10 + rnd() * 0.03;
            wisp.position().set((rnd() - 0.5) * 0.04, base_y, 0.02);
            wisp.rotation().z = (rnd() - 0.5) * 0.3;
            phone.add(&wisp);
            // R336 rise animator — wisp scales up + fades over a 6-10 s cycle.
            let wisp_phase = rnd() * PI * 2.0;
            let wisp_period = 6.0 + rnd() * 4.0; // 6-10 s per rise cycle
            let wisp_base_op = smoke_mat.opacity_v();
            let w2 = wisp.clone();
            let sm = smoke_mat.clone();
            g.animate(move |_g, _dt, t| {
                // Sawtooth: 0 → 1 → 0 → 1 ... over wisp_period.
                let frac = ((t + wisp_phase) / wisp_period) % 1.0;
                let s = 1.0 + frac * 0.6;
                w2.scale().set(s, 1.0 + frac * 0.4, s);
                w2.position().y = base_y + frac * 0.04;
                sm.m().opacity = wisp_base_op * (1.0 - frac);
            });
        }
    }
    // 25% chance handset off-hook; R303 broken phones have no handset at all.
    let off_hook = !is_broken && rnd() < 0.25;
    let skip_handset = is_broken;
    if skip_handset {
        // No handset rendered — broken phone with visible cracks.
    } else if off_hook {
        // hangs from the end of the coiled cord, under the cord jack on the base's underside
        let handset = mesh(pg("WallPhones__handset", cyl_geo!(0.015, 0.015, 0.16, 6)), &handset_mat);
        handset.position().set(0.02, -0.215, 0.012);
        phone.add(&handset);
        // one continuous coiled cord from the jack down to the handset
        for cd in 0..3 {
            let coil = mesh(pg("WallPhones__coil", torus_geo!(0.015, 0.003, 4, 8)), &cord_mat);
            coil.rotation().set(PI / 2.0, 0, cd as f64 * PI);
            coil.scale().z = 3.0;
            coil.position().set(0.02, -0.1086 - cd as f64 * 0.0102, 0.012);
            phone.add(&coil);
        }
    } else {
        // Handset on top (on-hook position).
        let handset = mesh(pg("WallPhones__handsetrest", cyl_geo!(0.02, 0.015, 0.13, 6)), &handset_mat);
        handset.rotation().z = PI / 2.0;
        handset.position().set(0, 0.10, 0.04);
        phone.add(&handset);
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.03); // body back (z −0.03) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.03);
    let phone_height = 1.3 + rnd() * 0.2;
    if c.dx.abs() == 1 {
        phone.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        phone.position().set(wall_x, phone_height, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        phone.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        phone.position().set(wall_x + (rnd() - 0.5) * 1.0, phone_height, wall_z);
    }
    g.world.add(&phone);
    let pp = phone.pos();
    // R278 track for ring audio events — only on-hook phones can ring.
    if !off_hook {
        g.pp.phone_positions.push(v3(pp.x, phone_height, pp.z));
    }
    // R325 track broken phones separately for spark-crackle audio.
    if is_broken {
        g.pp.broken_phone_positions.push(v3(pp.x, phone_height, pp.z));
    }
}

// R277 mops: 30% chance per level, 1 mop leaning against a wall.
fn place_mops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let stick_mat = lambert!({ color: 0x6a4a28 });
    let head_mat = lambert!({ color: 0x8a8878 });
    let mop = group();
    let stick = mesh(pg("Mops__stick", cyl_geo!(0.015, 0.015, 1.4, 6)), &stick_mat);
    stick.position().y = 0.70;
    mop.add(&stick);
    let head = mesh(pg("Mops__head", cyl_geo!(0.08, 0.06, 0.14, 8)), &head_mat);
    head.position().y = 0.07;
    mop.add(&head);
    // Lean stick ~10° toward wall (subtle).
    mop.rotation().z = -PI / 18.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.15);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.15);
    if c.dx.abs() == 1 {
        mop.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mop.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        mop.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mop.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&mop);
    // R278 40% chance mop bucket beside — janitorial co-location.
    if rnd() < 0.40 {
        let bucket_mat = lambert!({ color: 0xd8b820 });
        let bucket = group();
        let body = mesh(pg("Mops__bucket", cyl_geo!(0.14, 0.11, 0.22, 12)), &bucket_mat);
        body.position().y = 0.11;
        bucket.add(&body);
        let rim = mesh(pg("Mops__rim", torus_geo!(0.14, 0.006, 4, 12)), &bucket_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().y = 0.22;
        bucket.add(&rim);
        // Wire handle (thin arch — 2 small cylinders).
        let handle_mat = lambert!({ color: 0x3a3830 });
        for hs in [-1.0, 1.0] {
            let h_strand = mesh(pg("Mops__wire", cyl_geo!(0.003, 0.003, 0.10, 4)), &handle_mat);
            h_strand.position().set(hs * 0.12, 0.28, 0);
            bucket.add(&h_strand);
        }
        let handle_top = mesh(pg("Mops__handleTop", cyl_geo!(0.003, 0.003, 0.24, 4)), &handle_mat);
        handle_top.rotation().z = PI / 2.0;
        handle_top.position().y = 0.33;
        bucket.add(&handle_top);
        // R279 30% chance bucket contains dark water surface.
        if rnd() < 0.30 {
            let water_mat = basic_mat!({ color: 0x1a1a20, transparent: true, opacity: 0.75, depthWrite: false });
            let water = mesh(circle_geo!(0.12, 12), &water_mat);
            water.rotation().x = -PI / 2.0;
            water.position().y = 0.18; // ~80% full
            bucket.add(&water);
        }
        // Position 0.3-0.6 m from mop.
        let dist = 0.3 + rnd() * 0.3;
        let ang = rnd() * PI * 2.0;
        let mp = mop.pos();
        bucket.position().set(mp.x + ang.cos() * dist, 0, mp.z + ang.sin() * dist);
        g.world.add(&bucket);
    }
}

// R274 dead potted plants: 0-1 per level (25% chance).
fn place_dead_plants(g: &mut Game, _dist: &[i32]) {
    g.pp.living_plant_positions.clear(); // R291 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let pot_mat = lambert!({ color: 0x7a5028 });
    let soil_mat = lambert!({ color: 0x2a1810 });
    // R276 8% chance the plant is still ALIVE — greener stems + leaves.
    let is_alive = rnd() < 0.08;
    let stem_mat = lambert!({ color: if is_alive { 0x2a4a1a } else { 0x50402a } });
    let leaf_mat = lambert!({ color: if is_alive { 0x2a5028 } else { 0x3a2a18 } });
    let plant = group();
    let pot = mesh(pg("DeadPlants__pot", cyl_geo!(0.16, 0.12, 0.20, 12)), &pot_mat);
    pot.position().y = 0.10;
    plant.add(&pot);
    let soil = mesh(circle_geo!(0.14, 12), &soil_mat);
    soil.rotation().x = -PI / 2.0;
    soil.position().y = 0.202;
    plant.add(&soil);
    // 3-4 drooping stems.
    let stem_count = 3 + (rnd() * 2.0).floor() as i32;
    for s in 0..stem_count {
        let stem = mesh(pg("DeadPlants__stem", cyl_geo!(0.006, 0.008, 0.32, 5)), &stem_mat);
        let ang = (s as f64 / stem_count as f64) * PI * 2.0 + rnd() * 0.5;
        // Living plants (R276) barely droop.
        let droop = if is_alive { 0.05 + rnd() * 0.15 } else { 0.4 + rnd() * 0.4 };
        // Tilt radially outward (towards ang), not tangentially.
        let stem_dir = v3(ang.cos() * droop.sin(), droop.cos(), ang.sin() * droop.sin());
        stem.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &stem_dir);
        // Foot 1-3 cm down in the soil (surface 0.202), slightly off-centre.
        let stem_foot = v3(ang.cos() * 0.03, 0.172 + rnd() * 0.02, ang.sin() * 0.03);
        stem.position().copy(&stem_foot).add_scaled_vector(&stem_dir, 0.16);
        plant.add(&stem);
        // Tiny leaf on stem tip.
        let leaf = mesh(pg("DeadPlants__leaf", box_geo!(0.04, 0.005, 0.025)), &leaf_mat);
        leaf.position().copy(&stem_foot).add_scaled_vector(&stem_dir, 0.32);
        let sq = stem.quat();
        leaf.quaternion().copy(&sq);
        leaf.rotate_y(rnd() * PI); // turned about the stem
        plant.add(&leaf);
    }
    // Position near wall (bias toward corners).
    let wall_bias = rnd() < 0.75;
    let ox = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.3 + rnd() * 0.3) } else { (rnd() - 0.5) * 2.4 };
    let oz = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.3 + rnd() * 0.3) } else { (rnd() - 0.5) * 2.4 };
    plant.position().set(g.wx(pk.0) + ox, 0, g.wz(pk.1) + oz);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
    let pp = plant.pos();
    // R291 track living plants for rustle audio events.
    if is_alive {
        g.pp.living_plant_positions.push(v3(pp.x, 0.4, pp.z));
        // R292 5% chance a living plant emits a faint green glow; R293 30% of those pulse.
        if rnd() < 0.05 {
            let base_glow_int = 0.30;
            let glow_color = plant_glow_by_weather(g.weather_key());
            let glow = point_light!(glow_color, base_glow_int, 1.2, 2);
            glow.position().set(pp.x, 0.5, pp.z);
            g.world.add(&glow);
            if rnd() < 0.30 {
                let pulse_phase = rnd() * PI * 2.0;
                let gl = glow.clone();
                g.animate(move |_g, _dt, t| {
                    gl.set_intensity(base_glow_int * (0.85 + 0.15 * (t * 0.5 * PI + pulse_phase).sin()));
                });
            }
        }
    }
    // R275/R276 40% chance fallen leaves scattered around pot base (dead plants only).
    if !is_alive && rnd() < 0.40 {
        let debris_count = 3 + (rnd() * 3.0).floor() as i32;
        for _d in 0..debris_count {
            let leaf_debris = mesh(pg("DeadPlants__fallenLeaf", plane_geo!(0.04 + rnd() * 0.02, 0.025 + rnd() * 0.02)), &leaf_mat);
            leaf_debris.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.3;
            leaf_debris.rotation().z = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.20 + rnd() * 0.20;
            leaf_debris.position().set(pp.x + ang.cos() * rad, 0.003, pp.z + ang.sin() * rad);
            g.world.add(&leaf_debris);
        }
    }
}

// R274 ceiling speakers: 25% chance per level, 1-2 units flush on the ceiling.
fn place_ceiling_speakers(g: &mut Game, _dist: &[i32]) {
    g.pp.speaker_positions.clear(); // R275 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let grille_mat = lambert!({ color: 0xd8d0c0 });
    let rim_mat = lambert!({ color: 0x7a6a54 });
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        let speaker = group();
        let base = mesh(pg("CeilingSpeakers__trim", cyl_geo!(0.11, 0.11, 0.02, 12)), &rim_mat);
        speaker.add(&base);
        let grille = mesh(pg("CeilingSpeakers__grille", cyl_geo!(0.09, 0.09, 0.005, 12)), &grille_mat);
        grille.position().y = -0.012;
        speaker.add(&grille);
        // Small grille perforations suggested by concentric ring.
        let ring = mesh(ring_geo!(0.045, 0.055, 12), &rim_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = -0.0155; // 1 mm under the grille's face (bottom −0.0145)
        speaker.add(&ring);
        let ox = (rnd() - 0.5) * 1.4;
        let oz = (rnd() - 0.5) * 1.4;
        speaker.position().set(g.wx(x) + ox, WALL_H - 0.01, g.wz(z) + oz); // trim top on the ceiling
        g.world.add(&speaker);
        // R275 track for PA-announcement audio events.
        let sp = speaker.pos();
        g.pp.speaker_positions.push(v3(sp.x, sp.y, sp.z));
        i += 1;
    }
}

// R270 handrail: 0-2 wall-mounted horizontal metal bars per level.
fn place_handrails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.60 {
        return;
    } // 40% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut cands = wall_cands(g);
    shuffle(&mut cands);
    let rail_mat = lambert!({ color: 0x8a8880 });
    let bracket_mat = lambert!({ color: 0x5a5850 });
    let mut i = 0;
    while i < count && i < cands.len() {
        let c = cands[i];
        // Rail: 2.8-3.4 m long along wall.
        let rail_len = 2.8 + rnd() * 0.6;
        let rail = mesh(pg("Handrails__rail", cyl_geo!(0.02, 0.02, rail_len, 8)), &rail_mat);
        if c.dx.abs() == 1 {
            rail.rotation().x = PI / 2.0; // Y-axis cylinder → Z-axis
        } else {
            rail.rotation().z = PI / 2.0; // Y-axis cylinder → X-axis
        }
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.08);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.08);
        let rail_height = 0.9;
        if c.dx.abs() == 1 {
            rail.position().set(wall_x, rail_height, g.wz(c.z));
        } else {
            rail.position().set(g.wx(c.x), rail_height, wall_z);
        }
        g.world.add(&rail);
        // 2 brackets attaching rail to wall.
        for bs in [-1.0, 1.0] {
            let bracket = mesh(pg("Handrails__bracket", box_geo!(0.03, 0.06, 0.06)), &bracket_mat);
            // block runs from the wall to 6 mm into the rail's back
            let b_off = CELL * 0.5 - 0.033;
            bracket.scale().z = 1.1;
            if c.dx.abs() == 1 {
                bracket.rotation().y = PI / 2.0;
                bracket.position().set(g.wx(c.x) + c.dx as f64 * b_off, rail_height, g.wz(c.z) + bs * (rail_len / 2.0 - 0.15));
            } else {
                bracket.position().set(g.wx(c.x) + bs * (rail_len / 2.0 - 0.15), rail_height, g.wz(c.z) + c.dz as f64 * b_off);
            }
            g.world.add(&bracket);
        }
        // R271 worn-spot variant: 15% chance a darker band at the rail midpoint.
        if rnd() < 0.15 {
            let worn_mat = lambert!({ color: 0x4a4640 });
            let worn = mesh(pg("Handrails__worn", cyl_geo!(0.022, 0.022, 0.30, 8)), &worn_mat);
            if c.dx.abs() == 1 {
                worn.rotation().x = PI / 2.0;
            } else {
                worn.rotation().z = PI / 2.0;
            }
            worn.position().copy(&rail.pos());
            g.world.add(&worn);
        }
        i += 1;
    }
}

// R268 first aid kit: 20% chance per level, 1 wall-mounted unit.
fn place_first_aid_kits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let white_mat = lambert!({ color: 0xf0e8d8 });
    let red_mat = lambert!({ color: 0xa02010 });
    let trim_mat = lambert!({ color: 0x7a6a54 });
    let kit = group();
    let body = mesh(pg("FirstAidKits__body", box_geo!(0.20, 0.18, 0.10)), &white_mat);
    kit.add(&body);
    // Red cross on front (2 overlapping thin rectangles).
    let cross_v = mesh(plane_geo!(0.04, 0.12), &red_mat);
    cross_v.position().set(0, 0, 0.051);
    kit.add(&cross_v);
    let cross_h = mesh(plane_geo!(0.12, 0.04), &red_mat);
    cross_h.position().set(0, 0, 0.051);
    kit.add(&cross_h);
    let trim = mesh(plane_geo!(0.20, 0.02), &trim_mat);
    trim.position().set(0, 0.08, 0.051); // top strip of the front face (body top y 0.09)
    kit.add(&trim);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.05); // body back (z −0.05) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.05);
    let kit_height = 1.5 + rnd() * 0.3; // 1.5-1.8 m (chest-to-eye)
    if c.dx.abs() == 1 {
        kit.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        kit.position().set(wall_x, kit_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        kit.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        kit.position().set(wall_x + (rnd() - 0.5) * 1.2, kit_height, wall_z);
    }
    g.world.add(&kit);
    let kp = kit.pos();
    let k_rot_y = kit.rot().y;
    // R295 20% chance small orange pill bottle on floor near the kit.
    if rnd() < 0.20 {
        let bottle_mat = lambert!({ color: 0xd8621a });
        let cap_mat = lambert!({ color: 0xf0f0e8 });
        let bottle = group();
        let body = mesh(pg("FirstAidKits__bottle", cyl_geo!(0.02, 0.02, 0.06, 8)), &bottle_mat);
        body.position().y = 0.03;
        bottle.add(&body);
        let cap = mesh(pg("FirstAidKits__cap", cyl_geo!(0.022, 0.022, 0.015, 8)), &cap_mat);
        cap.position().y = 0.068;
        bottle.add(&cap);
        let is_tipped_bottle = rnd() < 0.35;
        if is_tipped_bottle {
            bottle.rotation().z = PI / 2.0;
            bottle.position().y = 0.022;
        }
        let (fx, fz) = (k_rot_y.sin(), k_rot_y.cos()); // into the room (kit front +Z)
        let bdist = 0.30 + rnd() * 0.40;
        let by = bottle.pos().y;
        bottle.position().set(kp.x + fx * bdist + (rnd() - 0.5) * 0.4, by, kp.z + fz * bdist + (rnd() - 0.5) * 0.4);
        bottle.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bottle);
        // R296 tipped bottles 60% chance spill 3-5 small white pills nearby.
        if is_tipped_bottle && rnd() < 0.60 {
            let pill_mat = lambert!({ color: 0xf0eee4 });
            let pill_count = 3 + (rnd() * 3.0).floor() as i32;
            let bp = bottle.pos();
            for _p in 0..pill_count {
                let pill = mesh(pg("FirstAidKits__pill", cyl_geo!(0.006, 0.006, 0.004, 6)), &pill_mat);
                let p_ang = rnd() * PI * 2.0;
                let p_rad = 0.03 + rnd() * 0.10;
                pill.position().set(bp.x + p_ang.cos() * p_rad, 0.003, bp.z + p_ang.sin() * p_rad);
                g.world.add(&pill);
            }
        }
    }
    // R297 8% chance small syringe on floor near the kit.
    if rnd() < 0.08 {
        let barrel_mat = lambert!({ color: 0xd8d0c8 });
        let needle_mat = lambert!({ color: 0x808080 });
        let syringe = group();
        let barrel = mesh(pg("FirstAidKits__barrel", cyl_geo!(0.007, 0.007, 0.06, 6)), &barrel_mat);
        barrel.rotation().z = PI / 2.0;
        syringe.add(&barrel);
        let needle = mesh(pg("FirstAidKits__needle", cyl_geo!(0.001, 0.001, 0.04, 4)), &needle_mat);
        needle.rotation().z = PI / 2.0;
        needle.position().x = 0.05;
        syringe.add(&needle);
        let plunger = mesh(pg("FirstAidKits__plunger", cyl_geo!(0.008, 0.008, 0.008, 6)), &barrel_mat);
        plunger.rotation().z = PI / 2.0;
        plunger.position().x = -0.034;
        syringe.add(&plunger);
        let (fx, fz) = (k_rot_y.sin(), k_rot_y.cos()); // into the room (kit front +Z)
        let sdist = 0.20 + rnd() * 0.30;
        syringe.position().set(kp.x + fx * sdist + (rnd() - 0.5) * 0.3, 0.008, kp.z + fz * sdist + (rnd() - 0.5) * 0.3);
        syringe.rotation().y = rnd() * PI * 2.0;
        g.world.add(&syringe);
    }
    // R269 25% chance a clipboard is dropped on the floor near the kit.
    if rnd() < 0.25 {
        let board_mat = lambert!({ color: 0x9a7048 });
        let paper_mat = lambert!({ color: 0xece0d0 });
        let clip_mat = lambert!({ color: 0x606060 });
        let clipboard = group();
        let board = mesh(pg("Clipboards__board", box_geo!(0.20, 0.008, 0.28)), &board_mat);
        board.position().y = 0.004;
        clipboard.add(&board);
        let paper = mesh(pg("Clipboards__paper", plane_geo!(0.18, 0.24)), &paper_mat);
        paper.rotation().x = -PI / 2.0;
        paper.position().y = 0.009;
        clipboard.add(&paper);
        let clip = mesh(pg("Clipboards__clip", box_geo!(0.06, 0.012, 0.02)), &clip_mat);
        clip.position().set(0, 0.014, -0.11);
        clipboard.add(&clip);
        let (fx, fz) = (k_rot_y.sin(), k_rot_y.cos()); // into the room (kit front +Z)
        let dist = 0.3 + rnd() * 0.4;
        clipboard.position().set(kp.x + fx * dist + (rnd() - 0.5) * 0.4, 0, kp.z + fz * dist + (rnd() - 0.5) * 0.4);
        clipboard.rotation().y = rnd() * PI * 2.0;
        g.world.add(&clipboard);
    }
    // R307 25% chance safety goggles dropped near the kit.
    if rnd() < 0.25 {
        let strap_mat = lambert!({ color: 0x181818 });
        // R323 per-weather lens tint.
        let lens_color = goggle_lens_by_weather(g.weather_key());
        let lens_mat = lambert!({ color: lens_color, transparent: true, opacity: 0.55 });
        let goggles = group();
        for off in [-0.028, 0.028] {
            let lens = mesh(circle_geo!(0.024, 12), &lens_mat);
            lens.position().set(off, 0.005, 0);
            lens.rotation().x = -PI / 2.0;
            goggles.add(&lens);
            // lie flat round the flat lens (torus is built upright in XY)
            let rim = mesh(pg("FirstAidKits__rim", torus_geo!(0.024, 0.004, 4, 12)), &strap_mat);
            rim.rotation().x = -PI / 2.0;
            rim.position().set(off, 0.005, 0);
            goggles.add(&rim);
        }
        let bridge = mesh(pg("FirstAidKits__bridge", box_geo!(0.012, 0.005, 0.008)), &strap_mat);
        bridge.position().set(0, 0.005, 0);
        goggles.add(&bridge);
        let strap = mesh(pg("FirstAidKits__strap", cyl_geo!(0.002, 0.002, 0.14, 4)), &strap_mat);
        strap.rotation().z = PI / 2.0;
        strap.position().set(0, 0.006, -0.030); // behind the rims (outer z −0.028)
        goggles.add(&strap);
        // kit front is local +Z: (sin, cos) of its yaw points into the room
        let (gfx, gfz) = (k_rot_y.sin(), k_rot_y.cos());
        let g_dist = 0.35 + rnd() * 0.35;
        goggles.position().set(kp.x + gfx * g_dist + (rnd() - 0.5) * 0.4, 0, kp.z + gfz * g_dist + (rnd() - 0.5) * 0.4);
        goggles.rotation().y = rnd() * PI * 2.0;
        g.world.add(&goggles);
    }
}

/// Restroom sign textures (M / W pictograms)
pub fn restroom_sign_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                vec![
                    canvas_tex_opt(64, 96, false, |ctx, w, h| {
                        ctx.set_fill_style("#484842");
                        ctx.fill_rect(0, 0, w, h);
                        ctx.set_fill_style("#f0e8d8");
                        // Simplified figure: circle head + trapezoidal body.
                        ctx.begin_path();
                        ctx.arc(w / 2.0, h / 2.0 - 22.0, 8, 0, PI * 2.0);
                        ctx.fill();
                        ctx.begin_path();
                        ctx.move_to(w / 2.0 - 16.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 16.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 12.0, h / 2.0 + 24.0);
                        ctx.line_to(w / 2.0 - 12.0, h / 2.0 + 24.0);
                        ctx.close_path();
                        ctx.fill();
                    }),
                    canvas_tex_opt(64, 96, false, |ctx, w, h| {
                        ctx.set_fill_style("#484842");
                        ctx.fill_rect(0, 0, w, h);
                        ctx.set_fill_style("#f0e8d8");
                        ctx.begin_path();
                        ctx.arc(w / 2.0, h / 2.0 - 22.0, 8, 0, PI * 2.0);
                        ctx.fill();
                        // Skirted figure: triangle body.
                        ctx.begin_path();
                        ctx.move_to(w / 2.0 - 14.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 14.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 20.0, h / 2.0 + 24.0);
                        ctx.line_to(w / 2.0 - 20.0, h / 2.0 + 24.0);
                        ctx.close_path();
                        ctx.fill();
                    }),
                ]
            })
        })
        .clone()
    })
}

// R268 restroom signs: 20% chance per level, 1 wall-mounted pictogram.
fn place_restroom_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = restroom_sign_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    let mat = lambert!({ map: tex, transparent: true, side: DoubleSide });
    let sign = mesh(plane_geo!(0.14, 0.20), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.012);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.012);
    let sign_height = 1.85 + rnd() * 0.2; // 1.85-2.05 m
    if c.dx.abs() == 1 {
        sign.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        sign.position().set(wall_x, sign_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        sign.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        sign.position().set(wall_x + (rnd() - 0.5) * 1.2, sign_height, wall_z);
    }
    g.world.add(&sign);
}

// R267 supply cabinet doors: 20% chance per level, 1 two-door unit flush on a wall.
fn place_cabinet_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let door_mat = lambert!({ color: 0xa89880 });
    let handle_mat = lambert!({ color: 0x2a2018 });
    let trim_mat = lambert!({ color: 0x7a6a54 });
    let cabinet = group();
    // Overall cabinet width 1.4 m, split into 2 doors of 0.68 m each.
    let door_w = 0.68;
    let door_h = 2.0;
    let gap = 0.04;
    // R271 10% chance one door slightly ajar showing dark interior.
    let ajar_side: i32 = if rnd() < 0.10 {
        if rnd() < 0.5 {
            -1
        } else {
            1
        }
    } else {
        0
    };
    let ajar_f = ajar_side as f64;
    if ajar_side != 0 {
        let interior_mat = basic_mat!({ color: 0x0a0806 });
        let interior = mesh(plane_geo!(0.4, door_h * 0.9), &interior_mat);
        interior.position().set(ajar_f * (door_w / 2.0 + gap / 2.0), 0, -0.02);
        cabinet.add(&interior);
        // R305 60% chance: small interior contents peek through the ajar gap.
        if rnd() < 0.60 {
            let shelf_mat = lambert!({ color: 0x3a2a20 });
            let shelf_y = -0.15 + rnd() * 0.3; // shelf between -0.15 and 0.15 m
            let shelf = mesh(pg("CabinetDoors__shelf", box_geo!(0.38, 0.006, 0.14)), &shelf_mat);
            shelf.position().set(ajar_f * (door_w / 2.0 + gap / 2.0), shelf_y, -0.01);
            cabinet.add(&shelf);
            let item_colors: [u32; 4] = [0xa08060, 0x606060, 0x807040, 0x704028];
            let item_count = 1 + (rnd() * 2.0).floor() as i32;
            for _i in 0..item_count {
                let is_can = rnd() < 0.5;
                let item_mat = lambert!({ color: pick(&item_colors) });
                let item = if is_can {
                    mesh(pg("CabinetDoors__can", cyl_geo!(0.024, 0.024, 0.08, 6)), &item_mat)
                } else {
                    mesh(pg("CabinetDoors__carton", box_geo!(0.05, 0.07, 0.04)), &item_mat)
                };
                let shelf_center = ajar_f * (door_w / 2.0 + gap / 2.0);
                item.position().set(shelf_center + (rnd() - 0.5) * 0.28, shelf_y + 0.043, -0.02 + 0.04);
                item.rotation().y = (rnd() - 0.5) * 0.4;
                cabinet.add(&item);
            }
        }
    }
    for side in [-1i32, 1] {
        let sf = side as f64;
        let door = mesh(pg("CabinetDoors__door", plane_geo!(door_w, door_h)), &door_mat);
        door.position().set(sf * (door_w / 2.0 + gap / 2.0), 0, 0.005);
        // R271 if this side is ajar, rotate the door outward slightly.
        if side == ajar_side {
            door.rotation().y = sf * -0.35; // ~20° swing
            door.position().z = 0.03; // pull forward as it swings open
        }
        cabinet.add(&door);
        // Frame trim around each door.
        for (ty, sx, sy) in [(door_h / 2.0, door_w, 0.03), (-door_h / 2.0, door_w, 0.03)] {
            let tr = mesh(pg("CabinetDoors__trim", plane_geo!(sx, sy)), &trim_mat);
            tr.position().set(sf * (door_w / 2.0 + gap / 2.0), ty, 0.006);
            cabinet.add(&tr);
        }
        // Handle: small dark box near center-adjoining edge.
        let handle = mesh(pg("CabinetDoors__handle", box_geo!(0.015, 0.10, 0.015)), &handle_mat);
        handle.position().set(sf * (gap / 2.0 + 0.05), 0, 0.014);
        cabinet.add(&handle);
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let center_h = 1.1; // door center at 1.1 m (bottom at 0.1, top at 2.1)
    if c.dx.abs() == 1 {
        cabinet.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        cabinet.position().set(wall_x, center_h, wall_z);
    } else {
        cabinet.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        cabinet.position().set(wall_x, center_h, wall_z);
    }
    g.world.add(&cabinet);
    // R309 30% chance: toolbox on floor 0.35-0.60 m in front of cabinet.
    if rnd() < 0.30 {
        let body_tool_mat = lambert!({ color: 0x8a2020 });
        let trim_tool_mat = lambert!({ color: 0x2a1010 });
        let handle_tool_mat = lambert!({ color: 0x181410 });
        let toolbox = group();
        let tbody = mesh(pg("CabinetDoors__toolbox", box_geo!(0.24, 0.10, 0.12)), &body_tool_mat);
        tbody.position().y = 0.05;
        toolbox.add(&tbody);
        // R313 20% chance: hinged-open lid variant showing 2-3 tiny tools inside.
        let is_open = rnd() < 0.20;
        if is_open {
            let interior_mat = basic_mat!({ color: 0x0a0605 });
            let interior = mesh(plane_geo!(0.22, 0.10), &interior_mat);
            interior.rotation().x = -PI / 2.0;
            interior.position().y = 0.101;
            toolbox.add(&interior);
            // Lid rotated up on back hinge.
            let open_lid = mesh(pg("CabinetDoors__lidopen", box_geo!(0.245, 0.010, 0.125)), &body_tool_mat);
            open_lid.position().set(0, 0.10, -0.062); // rotate around back edge
            open_lid.rotation().x = -1.10; // ~63° open
            open_lid.position().y += 0.055; // lift up so hinge stays at back
            open_lid.position().z += 0.017;
            toolbox.add(&open_lid);
            let tool_colors: [u32; 3] = [0x707878, 0x8a6040, 0x5a5850];
            let tool_count = 2 + (rnd() * 2.0).floor() as i32;
            for _ti in 0..tool_count {
                let tool_mat = lambert!({ color: pick(&tool_colors) });
                let is_screw = rnd() < 0.4;
                let tool = if is_screw {
                    mesh(pg("CabinetDoors__screwdriver", cyl_geo!(0.005, 0.005, 0.06, 5)), &tool_mat)
                } else {
                    mesh(pg("CabinetDoors__spanner", box_geo!(0.04, 0.008, 0.014)), &tool_mat)
                };
                if is_screw {
                    tool.rotation().z = PI / 2.0;
                }
                tool.position().set((rnd() - 0.5) * 0.18, 0.106, (rnd() - 0.5) * 0.08);
                tool.rotation().y = rnd() * PI * 2.0;
                toolbox.add(&tool);
            }
        } else {
            // Closed variant: flat lid ridge.
            let ridge = mesh(pg("CabinetDoors__lidtop", box_geo!(0.245, 0.004, 0.125)), &trim_tool_mat);
            ridge.position().y = 0.10;
            toolbox.add(&ridge);
        }
        // Carry handle: folded flat on the closed lid's top, or on the open lid's outer face.
        let thandle = mesh(pg("CabinetDoors__carryhandle", cyl_geo!(0.006, 0.006, 0.14, 5)), &handle_tool_mat);
        thandle.rotation().z = PI / 2.0;
        if is_open {
            thandle.position().set(0, 0.160, -0.055);
        } else {
            thandle.position().y = 0.108;
        }
        toolbox.add(&thandle);
        // Latch: tiny box at front-center (closed variant only).
        if !is_open {
            let latch = mesh(pg("CabinetDoors__latch", box_geo!(0.020, 0.015, 0.006)), &handle_tool_mat);
            latch.position().set(0, 0.055, 0.063);
            toolbox.add(&latch);
        }
        // Position 0.35-0.60 m out from cabinet along wall-normal.
        let fx = (-c.dx) as f64;
        let fz = (-c.dz) as f64;
        let tdist = 0.35 + rnd() * 0.25;
        toolbox.position().set(wall_x + fx * tdist + (rnd() - 0.5) * 0.5, 0, wall_z + fz * tdist + (rnd() - 0.5) * 0.5);
        toolbox.rotation().y = rnd() * PI * 2.0;
        g.world.add(&toolbox);
        // R312 30% chance: small wrench dropped on floor 0.10-0.25 m from toolbox.
        if rnd() < 0.30 {
            let wrench_mat = lambert!({ color: 0x707878 });
            let wrench = group();
            let w_shaft = mesh(pg("CabinetDoors__wrench", cyl_geo!(0.006, 0.006, 0.12, 5)), &wrench_mat);
            w_shaft.rotation().z = PI / 2.0;
            wrench.add(&w_shaft);
            // Head jaws: 2 small parallel boxes side by side in the plane the wrench lies in.
            for jyoff in [-0.010, 0.010] {
                let jaw = mesh(pg("CabinetDoors__jaw", box_geo!(0.020, 0.005, 0.008)), &wrench_mat);
                jaw.position().set(0.055, 0, jyoff);
                wrench.add(&jaw);
            }
            let knurl = mesh(pg("CabinetDoors__knurl", cyl_geo!(0.010, 0.010, 0.008, 6)), &wrench_mat);
            knurl.rotation().z = PI / 2.0;
            knurl.position().set(0.038, 0, 0);
            wrench.add(&knurl);
            let w_ang = rnd() * PI * 2.0;
            let w_dist = 0.10 + rnd() * 0.15;
            let tp = toolbox.pos();
            wrench.position().set(tp.x + w_ang.cos() * w_dist, 0.010, tp.z + w_ang.sin() * w_dist); // resting on the knurl ring
            wrench.rotation().y = rnd() * PI * 2.0;
            g.world.add(&wrench);
            // R314 40% chance: small dark oil-stain circle beneath wrench.
            if rnd() < 0.40 {
                // R324 per-weather oil color.
                let oil_color = oil_stain_by_weather(g.weather_key());
                let oil_mat = basic_mat!({ color: oil_color, transparent: true, opacity: 0.7, depthWrite: false });
                let oil = mesh(circle_geo!(0.04 + rnd() * 0.03, 10), &oil_mat);
                oil.rotation().x = -PI / 2.0;
                oil.rotation().z = rnd() * PI * 2.0;
                oil.scale().set(1.3, 0.9, 1); // slight stretch — organic pool
                let wp = wrench.pos();
                oil.position().set(wp.x + (rnd() - 0.5) * 0.05, 0.003, wp.z + (rnd() - 0.5) * 0.05);
                g.world.add(&oil);
            }
        }
    }
}

// R267 wet floor sign: 20% chance per level, 1 yellow A-frame sign.
fn place_wet_floor_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let yellow_mat = lambert!({ color: 0xe8b820 });
    let sign = group();
    // Two panels forming A-frame.
    for side in [-1.0f64, 1.0] {
        let panel = mesh(pg("WetFloorSigns__panel", plane_geo!(0.28, 0.42)), &yellow_mat);
        panel.rotation().y = side * (PI / 12.0); // 15° tilt from vertical
        panel.rotation().z = 0.0;
        // Lean each leaf 15° about X: tops meet at z = 0, feet 22 cm apart, fronts facing out.
        panel.rotation().set(-side * (PI / 12.0), if side < 0.0 { PI } else { 0.0 }, 0);
        panel.position().set(0, 0.21 * (PI / 12.0).cos() + 0.002, side * 0.21 * (PI / 12.0).sin());
        sign.add(&panel);
    }
    sign.position().set(g.wx(pk.0) + (rnd() - 0.5) * 1.8, 0, g.wz(pk.1) + (rnd() - 0.5) * 1.8);
    sign.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sign);
}

// R265 emergency light: 20% chance per level, 1 wall-mounted 2-bulb fixture.
fn place_emergency_lights(g: &mut Game, _dist: &[i32]) {
    g.pp.glow_light_positions.clear(); // R273 reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let housing_mat = lambert!({ color: 0xf0e8d8 });
    let rim_mat = lambert!({ color: 0x3a3028 });
    let light = group();
    let housing = mesh(pg("EmergencyLights__housing", box_geo!(0.30, 0.10, 0.10)), &housing_mat);
    light.add(&housing);
    // R266 20% chance the bulbs flicker via animator.
    let flickering = rnd() < 0.20;
    // Per-light material so animator can pulse individually.
    let bulb_mat = lambert!({ color: 0xe8e4c0, transparent: flickering, opacity: 1 });
    for side in [-1.0, 1.0] {
        let bulb = mesh(pg("EmergencyLights__head", cyl_geo!(0.04, 0.045, 0.05, 10)), &bulb_mat);
        bulb.rotation().x = PI / 2.0;
        bulb.position().set(side * 0.09, 0, 0.075);
        light.add(&bulb);
        let rim = mesh(pg("EmergencyLights__bezel", cyl_geo!(0.048, 0.048, 0.008, 10)), &rim_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(side * 0.09, 0, 0.100);
        light.add(&rim);
    }
    if flickering {
        // Random-interval brief opacity dip (5-15 s between dips, 60 ms dip).
        let mut flicker_t = 5.0 + rnd() * 10.0;
        let mut dip_t = 0.0;
        let bm = bulb_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                bm.m().opacity = if dip_t > 0.0 { 0.20 } else { 1.0 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.06;
                    flicker_t = 5.0 + rnd() * 10.0;
                }
            }
        });
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.05); // housing back (z −0.05) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.05);
    let light_height = 2.5 + rnd() * 0.3; // 2.5-2.8 m (above door lintel)
    if c.dx.abs() == 1 {
        light.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        light.position().set(wall_x, light_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        light.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        light.position().set(wall_x + (rnd() - 0.5) * 1.2, light_height, wall_z);
    }
    g.world.add(&light);
    // R269/R270 "on" variant: 10% chance the emergency light emits a real dim point light.
    if rnd() < 0.10 {
        let base_int = 0.6;
        let point_l = point_light!(0xf5e090, base_int, 3.0, 2);
        // 0.15 m forward of housing (in front of bulbs; local +Z).
        let ly = light.rot().y;
        let (fx, fz) = (ly.sin(), ly.cos());
        let lp = light.pos();
        point_l.position().set(lp.x + fx * 0.15, lp.y - 0.02, lp.z + fz * 0.15);
        g.world.add(&point_l);
        bulb_mat.m().color.set_hex(0xfff8c0); // brighter "on" bulb color
                                              // R270 subtle sin-wave flicker: ±10% at 0.7 Hz with random phase.
        let flicker_phase = rnd() * PI * 2.0;
        let pl = point_l.clone();
        g.animate(move |_g, _dt, t| {
            pl.set_intensity(base_int * (0.90 + 0.10 * (t * 1.4 * PI + flicker_phase).sin()));
        });
        // R273 track for spatial hum audio events.
        let pp = point_l.pos();
        g.pp.glow_light_positions.push(v3(pp.x, pp.y, pp.z));
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
