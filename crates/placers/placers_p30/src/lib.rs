//! Prop placers: briefcases, keycard readers, wall phones,
//! mops, dead plants, ceiling speakers, handrails, first-aid kits, restroom
//! signs, cabinet doors, wet-floor signs and emergency lights.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_briefcases", place_briefcases));
    r.push(("place_keycard_readers", place_keycard_readers));
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

// R277 wall phone: 15% chance per level, 1 unit. The office wall set of the
// 2554 class (Blender batch `r12`): a moulded housing on its wall plate, the
// handset hung down its front on the chrome hanger, its coiled cord looping
// under it to the jack; off the hook the handset hangs on its drawn-out cord;
// broken, the housing cracked, the cord cut short, and the burnt board's smoke.
fn place_wall_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.phone_positions.clear(); // R278 reset per level
    g.pp.broken_phone_positions.clear(); // R325 reset per level
    let chance = rnd();
    if chance < 0.85 {
        return;
    } // 15% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    // the set's colour, from what is left of the chance draw (0.85..1): the
    // beige most offices had, the ash grey, black, now and then brown
    let u = (chance - 0.85) / 0.15;
    let shell = if u < 0.45 {
        0xb3a68a
    } else if u < 0.72 {
        0x9b978c
    } else if u < 0.9 {
        0x2e2c2a
    } else {
        0x5c4636
    };
    let phone = group();
    // the parts are modelled with the wall at z = 0: the housing's back on it
    let o = group();
    o.position().z = -0.03;
    phone.add(&o);
    let keys = mesh(wp_part("Real__wallPhoneKeys", 0.05, 0.07, 0.006), lambert!({ color: 0x3b3b3d }));
    o.add(&keys);
    let legends = mesh(
        plane_geo!(0.056, 0.074),
        lambert!({ map: &wp_keys_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    legends.position().set(-0.0125, 0.010, WP_D + 0.0015);
    o.add(&legends);
    let card = mesh(plane_geo!(0.040, 0.011), lambert!({ map: &wp_card_tex() }));
    card.position().set(-0.006, -0.047, WP_D - 0.0017);
    o.add(&card);
    let hook = mesh(wp_part("Real__wallPhoneHook", 0.032, 0.03, 0.02), phong!({ color: 0x575752, specular: 0x3a3a3a, shininess: 22 }));
    o.add(&hook);
    // R303 5% chance: broken phone with a cracked housing + no handset.
    let is_broken = rnd() < 0.05;
    let mut cracks = [(0.0, 0.0, 0.0, 0.0); 3];
    if is_broken {
        // the old crack planes' draws, now where the housing split: the
        // crack's length, its turn and its place on the face
        for ck in cracks.iter_mut() {
            let w = 0.001 + rnd() * 0.002;
            let turn = rnd() * PI;
            let x = (rnd() - 0.5) * 0.08;
            let y = -0.02 + (rnd() - 0.5) * 0.06;
            *ck = (w, turn, x, y);
        }
    }
    let housing_map = if is_broken { wp_broken_tex(cracks) } else { wp_plastic_tex() };
    let housing = mesh(wp_part("Real__wallPhoneHousing", WP_W, 0.232, WP_D), lambert!({ map: &housing_map, color: shell }));
    o.add(&housing);
    if is_broken {
        // R335 40% chance: smoke from the burnt board, leaking at the top.
        if rnd() < 0.40 {
            // R342 per-weather smoke color.
            let smoke_color = smoke_wisp_by_weather(g.weather_key());
            // lit by the room (lambert), so paler than its tint: smoke scatters the light
            let pale = |c: u32, k: u32| (((c >> k) & 0xff) + 0xd8) / 2 << k;
            let smoke_color = pale(smoke_color, 16) | pale(smoke_color, 8) | pale(smoke_color, 0);
            let smoke_mat = lambert!({ map: &wp_smoke_tex(), color: smoke_color, transparent: true, opacity: 0.6, depthWrite: false, side: DoubleSide });
            // two soft sheets crossed at 45 degrees to the wall, so neither is
            // ever seen edge on from the room
            let wisp = group();
            for turn in [PI / 4.0, -PI / 4.0] {
                let sheet = mesh(plane_geo!(0.09, 0.14), &smoke_mat);
                sheet.rotation().y = turn;
                sheet.position().y = 0.07;
                wisp.add(&sheet);
            }
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
    let cord_mat = lambert!({ map: &wp_handset_tex(), color: shell });
    if skip_handset {
        // no handset: the cord cut off a hand's width under the jack
        o.add(&mesh(wp_part("Real__wallPhoneCordCut", 0.02, 0.1, 0.02), &cord_mat));
    } else {
        let handset = mesh(wp_part("Real__wallPhoneHandset", 0.06, 0.21, 0.04), lambert!({ map: &wp_handset_tex(), color: shell }));
        if off_hook {
            // hanging upside down from its drawn-out cord, turned a little
            handset.position().set(0.030, -0.565, 0.012);
            handset.rotation().set(0, 0.30, PI);
            o.add(&mesh(wp_part("Real__wallPhoneCordOff", 0.02, 0.45, 0.02), &cord_mat));
        } else {
            // hung on the hook, the cord looping under the phone
            handset.position().set(-0.026, 0.001, WP_D - 0.004);
            o.add(&mesh(wp_part("Real__wallPhoneCordOn", 0.07, 0.2, 0.05), &cord_mat));
        }
        o.add(&handset);
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

// ------------------------------------------------------------ the wall phone (r12)

/// The housing's width and depth (Blender batch `r12`, `wallPhoneHousing`).
const WP_W: f64 = 0.123;
const WP_D: f64 = 0.058;

/// A Blender part of batch `r12` as modelled (a scale of one), or a box
/// standing in for it when the props are switched off.
fn wp_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// Moulded ABS after decades on an office wall, laid over the set's colour
/// by the housing's box projection (the whole canvas is its face): yellowed
/// from the top, grey with hands round the earpiece's seat and the keys, the
/// plate's edge darker where the wall's dust settled, scuffs and scratches.
fn paint_wp_plastic(ctx: &mut Canvas, w: f64, h: f64) {
    ctx.set_fill_style("#f4f1ea");
    ctx.fill_rect(0, 0, w, h);
    // the moulding's faint mottle
    for _ in 0..500 {
        let a = 0.025 + rnd() * 0.04;
        ctx.set_fill_style(&format!("rgba({},{},{},{a})", 90 + (rnd() * 60.0) as i32, 82, 70));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
    }
    // yellowed: the light and the smoke of the room, strongest at the top
    let gy = ctx.create_linear_gradient(0, 0, 0, h);
    gy.add_color_stop(0, "rgba(200,170,90,0.30)");
    gy.add_color_stop(0.6, "rgba(200,170,90,0.10)");
    gy.add_color_stop(1, "rgba(200,170,90,0.04)");
    ctx.set_fill_style(gy);
    ctx.fill_rect(0, 0, w, h);
    // hands: the earpiece's seat (top left), round the keys, the mouthpiece's rest
    for &(cx, cy, r, a) in &[(0.29, 0.16, 0.30, 0.30), (0.40, 0.45, 0.32, 0.20), (0.30, 0.83, 0.26, 0.22), (0.55, 0.30, 0.22, 0.14)] {
        ctx.save();
        ctx.translate(cx * w, cy * h);
        ctx.scale(1.0, 1.3);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r * w);
        gr.add_color_stop(0, &format!("rgba(60,48,36,{a})"));
        gr.add_color_stop(1, "rgba(60,48,36,0)");
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.arc(0, 0, r * w, 0, 7);
        ctx.fill();
        ctx.restore();
    }
    // dust on the top edge, grime along the foot
    let gd = ctx.create_linear_gradient(0, 0, 0, h);
    gd.add_color_stop(0, "rgba(120,110,95,0.30)");
    gd.add_color_stop(0.05, "rgba(120,110,95,0)");
    gd.add_color_stop(0.92, "rgba(50,42,34,0)");
    gd.add_color_stop(1, "rgba(50,42,34,0.30)");
    ctx.set_fill_style(gd);
    ctx.fill_rect(0, 0, w, h);
    // scuffs and fine scratches
    ctx.set_line_cap("round");
    for _ in 0..26 {
        let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 3.0 + rnd() * 16.0);
        ctx.set_line_width(0.5 + rnd() * 0.8);
        ctx.set_stroke_style(&format!("rgba(70,60,50,{})", 0.10 + rnd() * 0.22));
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
    for _ in 0..8 {
        ctx.set_fill_style(&format!("rgba(40,34,28,{})", 0.12 + rnd() * 0.15));
        ctx.begin_path();
        ctx.ellipse(rnd() * w, rnd() * h, 1.5 + rnd() * 4.0, 0.8 + rnd() * 1.5, rnd() * 3.0, 0, 7);
        ctx.fill();
    }
}

/// The housing's wear (shared by every set: its colour is the material's).
fn wp_plastic_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| c.get_or_init(|| canvas_tex_opt(128, 240, false, |ctx, w, h| paint_wp_plastic(ctx, w, h))).clone())
}

/// A broken set's housing: the same wear, and the three cracks where it was
/// struck (length, turn and place from the old crack planes' draws), each a
/// dark split with the plastic stress-whitened along it.
fn wp_broken_tex(cracks: [(f64, f64, f64, f64); 3]) -> Tex {
    canvas_tex_opt(128, 240, false, move |ctx, w, h| {
        paint_wp_plastic(ctx, w, h);
        // the face's metres to the canvas: x over the housing's 0.123 m, y over 0.232 m
        let to = |x: f64, y: f64| ((x / 0.123 + 0.5) * w, (0.5 - y / 0.232) * h);
        for &(cw, turn, x, y) in &cracks {
            // onto the flat of the face, right of the keys, where a blow lands
            let (px, py) = to(0.034 + x * 0.3, y * 1.7 + 0.02);
            let len = (0.040 + (cw - 0.001) / 0.002 * 0.035) / 0.123 * w;
            let (dx, dy) = (turn.cos(), turn.sin());
            let mut pts = vec![];
            let n = 9;
            for k in 0..=n {
                let t = k as f64 / n as f64 - 0.5;
                let j = if k == 0 || k == n { 0.0 } else { (rnd() - 0.5) * 3.0 };
                pts.push((px + dx * t * len - dy * j, py + dy * t * len + dx * j));
            }
            // the whitened plastic either side, then the split
            for (lw, col) in [(3.2, "rgba(255,250,235,0.35)"), (1.1, "rgba(28,22,18,0.85)")] {
                ctx.set_line_width(lw);
                ctx.set_line_join("round");
                ctx.set_stroke_style(col);
                ctx.begin_path();
                ctx.move_to(pts[0].0, pts[0].1);
                for p in &pts[1..] {
                    ctx.line_to(p.0, p.1);
                }
                ctx.stroke();
            }
            // a branch off its middle
            let (bx, by) = pts[n / 2];
            let bt = turn + 0.7 + rnd() * 0.6;
            ctx.set_line_width(0.8);
            ctx.begin_path();
            ctx.move_to(bx, by);
            ctx.line_to(bx + bt.cos() * len * 0.3, by + bt.sin() * len * 0.3);
            ctx.stroke();
        }
    })
}

/// The handset's and the cord's wear: grimy where it is held, rubbed shiny
/// and dark at the grip, the cups' rims scuffed.
fn wp_handset_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 160, false, |ctx, w, h| {
                ctx.set_fill_style("#f2eee6");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..240 {
                    ctx.set_fill_style(&format!("rgba(80,70,60,{})", 0.03 + rnd() * 0.05));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
                }
                let gr = ctx.create_linear_gradient(0, 0, 0, h);
                gr.add_color_stop(0, "rgba(60,48,36,0.12)");
                gr.add_color_stop(0.35, "rgba(60,48,36,0.32)");
                gr.add_color_stop(0.65, "rgba(60,48,36,0.32)");
                gr.add_color_stop(1, "rgba(60,48,36,0.14)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..12 {
                    let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 3.0 + rnd() * 10.0);
                    ctx.set_line_width(0.6);
                    ctx.set_stroke_style(&format!("rgba(60,50,40,{})", 0.15 + rnd() * 0.2));
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

/// What is printed on the keys, white on their grey (as the 2500 sets'):
/// the letters over the numeral, OPER over the 0, the star and the square;
/// the most used rubbed thin. The canvas spans the twelve keys, 2 px a mm.
fn wp_keys_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(112, 148, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let keys = [
                    ("1", ""),
                    ("2", "ABC"),
                    ("3", "DEF"),
                    ("4", "GHI"),
                    ("5", "JKL"),
                    ("6", "MNO"),
                    ("7", "PRS"),
                    ("8", "TUV"),
                    ("9", "WXY"),
                    ("*", ""),
                    ("0", "OPER"),
                    ("#", ""),
                ];
                ctx.set_text_align("center");
                ctx.set_text_baseline("alphabetic");
                for (i, (num, letters)) in keys.iter().enumerate() {
                    let (col, row) = ((i % 3) as f64, (i / 3) as f64);
                    // the key's middle: x -0.030 + col * 0.0175 over the canvas's -0.0405..0.0155
                    let cx = (0.0105 + col * 0.0175) * 2000.0;
                    let cy = (0.007 + row * 0.020) * 2000.0;
                    let worn = rnd();
                    ctx.set_fill_style(&format!("rgba(238,236,228,{})", 0.95 - worn * 0.45));
                    if !letters.is_empty() {
                        ctx.set_font("bold 5px Helvetica");
                        ctx.fill_text(letters, cx, cy - 2.0);
                    }
                    let big = if *num == "*" || *num == "#" { "bold 11px Helvetica" } else { "bold 9px Helvetica" };
                    ctx.set_font(big);
                    ctx.fill_text(num, cx, cy + if letters.is_empty() { 5.0 } else { 7.0 });
                }
            })
        })
        .clone()
    })
}

/// The number card behind its window: a printed rule and the extension
/// written in by hand, the card yellowed (555-01xx, the numbers kept for fiction).
fn wp_card_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(160, 44, false, |ctx, w, h| {
                ctx.set_fill_style("#d9d0b8");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_stroke_style("rgba(120,40,30,0.55)");
                ctx.set_line_width(1);
                ctx.stroke_rect(3.5, 3.5, w - 7.0, h - 7.0);
                ctx.set_fill_style("rgba(30,30,60,0.85)");
                ctx.set_font("22px Marker Felt");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("555-0147", w / 2.0, h / 2.0 + 1.0);
                let gr = ctx.create_linear_gradient(0, 0, w, 0);
                gr.add_color_stop(0, "rgba(90,70,40,0.25)");
                gr.add_color_stop(0.5, "rgba(90,70,40,0)");
                gr.add_color_stop(1, "rgba(90,70,40,0.2)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            })
        })
        .clone()
    })
}

/// A wisp of smoke: soft, thicker at its foot, drawn out and breaking up.
fn wp_smoke_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..40 {
                    let t = k as f64 / 39.0; // 0 at the foot, 1 at the top
                    let x = w / 2.0 + (t * 6.0 + rnd() * 0.8).sin() * w * 0.16 * t;
                    let y = h * (0.97 - t * 0.85);
                    let r = w * (0.14 + t * 0.26);
                    let a = 0.32 * (1.0 - t).powf(0.6);
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.scale(1.0, 1.4);
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(255,255,255,{a})"));
                    gr.add_color_stop(1, "rgba(255,255,255,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
            })
        })
        .clone()
    })
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
    // Zonal geraniums in a 31 cm terracotta pot (batch `r18`), not watered for
    // years: the stems woody and grey, knuckled at their nodes, a leaf or two
    // dried on them, the rest fallen; the soil shrunk from the wall and
    // cracked; salts bloomed white through the clay (r18.notes.md). One in
    // twelve is still alive, leggy, a few green leaves at its tips.
    use game::assets::props::prop_part_as_modelled;
    g.pp.living_plant_positions.clear(); // R291 reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    // R276 8% chance the plant is still ALIVE
    let is_alive = rnd() < 0.08;
    let plant = group();
    let salt = (pk.0 as u32).wrapping_mul(0x9e37_79b9) ^ (pk.1 as u32).wrapping_mul(0x85eb_ca6b);
    let put = |grp: &Obj, name: &str, mat: &Mat| -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?;
        let m = mesh(geo, mat);
        grp.add(&m);
        Some(m)
    };
    put(&plant, "Real__deadPlantPot", &lambert!({ map: &r18_plant::pot_tex(salt) }));
    put(&plant, "Real__deadPlantSoil", &lambert!({ map: &r18_plant::soil_tex(salt) }));
    let stem_mat = if is_alive { lambert!({ color: 0x5f6a3e }) } else { lambert!({ color: 0x6f665a }) };
    let leaf_mat = if is_alive {
        lambert!({ map: &r18_plant::leaf_tex(true, salt), side: DoubleSide })
    } else {
        lambert!({ map: &r18_plant::leaf_tex(false, salt), side: DoubleSide })
    };
    if !is_alive {
        put(&plant, "Real__deadPlantLitter", &leaf_mat);
    }
    // 3-4 stems, each its old draws: its way out, how far it leans, how deep
    // its foot is and the turn of the leaf at its top
    let stem_count = 3 + (rnd() * 2.0).floor() as i32;
    for s in 0..stem_count {
        let ang = (s as f64 / stem_count as f64) * PI * 2.0 + rnd() * 0.5;
        // living plants (R276) barely lean; the dead lean out further
        let droop = if is_alive { 0.05 + rnd() * 0.15 } else { 0.4 + rnd() * 0.4 };
        let lean = if is_alive { droop * 0.6 } else { 0.12 + (droop - 0.4) * 0.7 };
        let stem_dir = v3(ang.cos() * lean.sin(), lean.cos(), ang.sin() * lean.sin());
        // its foot 1-3 cm down in the soil (its surface 0.150), off the centre
        let stem_foot = v3(ang.cos() * 0.03, r18_plant::SOIL_Y - 0.03 + (0.172 + rnd() * 0.02 - 0.172), ang.sin() * 0.03);
        let spin = rnd() * PI;
        let k = s as usize % 3;
        let stem = group();
        stem.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &stem_dir);
        stem.position().copy(&stem_foot);
        // the stem's own lean turned outward
        let inner = group();
        inner.rotation().y = r18_plant::STEM_LEAN_AZ[k] - ang;
        stem.add(&inner);
        put(&inner, ["Real__deadPlantStemA", "Real__deadPlantStemB", "Real__deadPlantStemC"][k], &stem_mat);
        // a leaf at its top node, dried on it (or living)
        let (c, d) = r18_plant::STEM_TOP[k];
        let leaf = group();
        leaf.position().set(c[0], c[1], c[2]);
        leaf.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v3(d[0], d[1], d[2]));
        leaf.rotate_y(spin);
        put(&leaf, if is_alive { "Real__deadPlantLeafLive" } else { "Real__deadPlantLeafDry" }, &leaf_mat);
        inner.add(&leaf);
        plant.add(&stem);
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
        // the old glow's draws (R292, R293), kept: a plant gives no light
        if rnd() < 0.05 {
            if rnd() < 0.30 {
                let _ = rnd(); // keeps the old draw (its pulse's phase)
            }
        }
    }
    // R275/R276 40% chance: dry leaves fallen round the pot (dead plants only),
    // each its old draws: its size, its tilt, its turn, where it lies
    if !is_alive && rnd() < 0.40 {
        let debris_count = 3 + (rnd() * 3.0).floor() as i32;
        for d in 0..debris_count {
            let (sw, _sh) = (rnd(), rnd());
            let _tilt = rnd(); // keeps the old draw (it lies as it curled)
            let turn = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.20 + rnd() * 0.20;
            let leaf = group();
            put(&leaf, "Real__deadPlantFallen", &leaf_mat);
            leaf.scale().set_scalar(0.85 + 0.3 * sw);
            leaf.rotation().y = turn;
            leaf.position().set(pp.x + ang.cos() * rad, 0.0, pp.z + ang.sin() * rad);
            g.world.add(&leaf);
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

/// The dead geraniums of `place_dead_plants` (batch `r18`): where its parts
/// meet, and what is painted on them.
mod r18_plant {
    use super::*;

    /// The soil's surface in the pot
    pub(super) const SOIL_Y: f64 = 0.150;
    /// Each stem part's top node and its direction there, in the part's frame
    /// (printed by `r18.py`)
    pub(super) const STEM_TOP: [([f64; 3], [f64; 3]); 3] = [
        ([0.04409, 0.27403, 0.03280], [0.3067, 0.9472, 0.0939]),
        ([0.01691, 0.26503, -0.00006], [0.2041, 0.9717, -0.1187]),
        ([0.04742, 0.26873, -0.05838], [0.2606, 0.9368, -0.2335]),
    ];
    /// The way each stem part leans of itself (its top's azimuth from +x
    /// toward +z), turned outward in the pot
    pub(super) const STEM_LEAN_AZ: [f64; 3] = [0.6397, -0.0035, -0.8885];
    const POT_H: f64 = 0.193;

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
    fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// The two nearest distances to a jittered lattice (cells of 1)
    fn worley(seed: u32, x: f64, y: f64) -> (f64, f64) {
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dj in -1..=1 {
            for di in -1..=1 {
                let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                let fx = ii as f64 + hash(seed, ii, jj);
                let fy = jj as f64 + hash(seed ^ 77, ii, jj);
                let d = (fx - x).hypot(fy - y);
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d1, d2)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let (iw, _) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    fn rgb(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }
    fn css(c: [f64; 3], a: f64) -> String {
        format!("rgba({},{},{},{:.3})", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a)
    }

    /// The pot, round it and up it: terracotta faded, the salts bloomed white
    /// through it in a band under the rim and in patches, green-black where it
    /// stood damp at its foot, dust, a chip or two to the fresh clay
    pub(super) fn pot_tex(salt: u32) -> Tex {
        canvas_tex_opt(384, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0x7e11;
            let wrap = |sd: u32, u: f64, y: f64, fu: f64, fy: f64, o: u32| {
                fbm(sd, u * fu, y * fy, o) * (1.0 - u) + fbm(sd, (u - 1.0) * fu, y * fy, o) * u
            };
            fill(ctx, w, h, |u, v| {
                let y = v * POT_H;
                let clay = [0.63, 0.40, 0.29];
                let mut c = mix(clay, [0.55, 0.34, 0.25], wrap(seed, u, y, 30.0, 25.0, 4));
                // the white bloom: a band under the collar and patches
                let bloom = (wrap(seed ^ 1, u, y, 18.0, 14.0, 4) - 0.42) * 2.6 + sstep(0.09, 0.15, y) * 0.5 - sstep(0.165, 0.19, y) * 0.25;
                let lace = 0.75 + 0.25 * wrap(seed ^ 5, u, y, 160.0, 140.0, 2);
                c = mix(c, [0.8, 0.78, 0.72], bloom.clamp(0.0, 1.0) * 0.85 * lace);
                // the foot dark where it stood in its own water, green-black
                let foot = sstep(0.05, 0.0, y) * (0.6 + 0.4 * wrap(seed ^ 2, u, y, 50.0, 60.0, 3));
                c = mix(c, [0.2, 0.21, 0.15], foot * 0.85);
                // dust and grime over all
                c = mix(c, [0.45, 0.42, 0.37], 0.18 + 0.1 * wrap(seed ^ 3, u, y, 8.0, 6.0, 2));
                // pits in the clay
                let pit = (vn(seed ^ 4, u * 700.0, y * 1200.0) - 0.8).max(0.0) * 3.0;
                c = mix(c, [0.32, 0.22, 0.17], pit);
                rgb(c)
            });
        })
    }

    /// The soil from above: a dry grey-brown crust cracked into plates,
    /// paler where it dried last, perlite's white grains, a crumb or two
    pub(super) fn soil_tex(salt: u32) -> Tex {
        canvas_tex_opt(192, 192, false, move |ctx, w, h| {
            let seed = salt ^ 0x5011;
            fill(ctx, w, h, |u, v| {
                let (x, y) = (u * 0.26, v * 0.26);
                // plates of every size: the cracks meander (the lattice warped)
                let (wx, wy) =
                    (x + 0.012 * (fbm(seed ^ 6, x * 30.0, y * 30.0, 2) - 0.5), y + 0.012 * (fbm(seed ^ 7, x * 30.0, y * 30.0, 2) - 0.5));
                let (d1, d2) = worley(seed, wx / 0.034, wy / 0.034);
                let (e1, e2) = worley(seed ^ 9, wx / 0.012, wy / 0.012);
                let crack = sstep(0.045, 0.0, d2 - d1).max(sstep(0.03, 0.0, e2 - e1) * 0.5);
                let mut c = mix([0.27, 0.22, 0.17], [0.38, 0.33, 0.27], fbm(seed ^ 1, x * 90.0, y * 90.0, 4));
                c = mix(c, [0.44, 0.4, 0.34], (fbm(seed ^ 8, x * 25.0, y * 25.0, 3) - 0.45).max(0.0) * 1.2);
                c = mix(c, [0.09, 0.07, 0.05], crack * 0.85);
                let grain = (vn(seed ^ 2, x * 900.0, y * 900.0) - 0.88).max(0.0) * 9.0;
                c = mix(c, [0.82, 0.8, 0.75], grain.min(1.0));
                // the gap at its edge, dark (the edge of the map)
                let rho = ((u - 0.5).hypot(v - 0.5) * 2.0).min(1.0);
                c = mix(c, [0.08, 0.06, 0.05], sstep(0.9, 1.0, rho));
                rgb(c)
            });
        })
    }

    /// A leaf: dry, a papery tan-brown blotched darker and paler, its veins
    /// dark; or living, a dull green with the zonal geranium's dark horseshoe
    pub(super) fn leaf_tex(alive: bool, salt: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            let seed = salt ^ if alive { 0x1eaf } else { 0xd1ef };
            fill(ctx, w, h, |u, v| {
                let (x, y) = (u - 0.5, v - 0.5);
                let r = x.hypot(y);
                let vein = (((y.atan2(x) * 7.0 / PI).fract() - 0.5).abs() * 2.0 - 0.85).max(0.0) * 6.0;
                let n = fbm(seed, u * 6.0, v * 6.0, 4);
                let c = if alive {
                    let mut c = mix([0.2, 0.3, 0.13], [0.26, 0.36, 0.17], n);
                    c = mix(c, [0.15, 0.17, 0.08], sstep(0.05, 0.0, (r - 0.28).abs()) * 0.6);
                    mix(c, [0.36, 0.33, 0.18], sstep(0.4, 0.5, r) * 0.6)
                } else {
                    let mut c = mix([0.42, 0.33, 0.22], [0.55, 0.45, 0.31], n);
                    c = mix(c, [0.27, 0.2, 0.13], (n - 0.55).max(0.0) * 2.5);
                    mix(c, [0.3, 0.23, 0.15], vein.min(1.0) * 0.6)
                };
                rgb(c)
            });
        })
    }
}
