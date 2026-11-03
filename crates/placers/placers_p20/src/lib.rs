//! Prop placers: abandoned-office litter and fixtures — sunglasses, fire alarm
//! pulls, bottles, dispensers, balls, pay phones, matchbooks, radios, cigarette
//! packs, bookshelves, keys, safety vests, hard hats, hose reels, gloves, mail,
//! newspapers, lockers, pizza boxes, cans, hammers, switches, wastebaskets,
//! elevator buttons, mold, umbrellas, boots, overturned chairs and torn photos.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_broken_sunglasses", place_broken_sunglasses));
    r.push(("place_fire_alarm_pulls", place_fire_alarm_pulls));
    r.push(("place_crushed_water_bottles", place_crushed_water_bottles));
    r.push(("place_spray_bottles", place_spray_bottles));
    r.push(("place_soap_dispensers", place_soap_dispensers));
    r.push(("place_deflated_balls", place_deflated_balls));
    r.push(("place_wall_pay_phones", place_wall_pay_phones));
    r.push(("place_matchbooks", place_matchbooks));
    r.push(("place_small_radios", place_small_radios));
    r.push(("place_cigarette_packs", place_cigarette_packs));
    r.push(("place_bookshelves", place_bookshelves));
    r.push(("place_keychain_keys", place_keychain_keys));
    r.push(("place_safety_vests", place_safety_vests));
    r.push(("place_hard_hats", place_hard_hats));
    r.push(("place_fire_hose_reels", place_fire_hose_reels));
    r.push(("place_single_gloves", place_single_gloves));
    r.push(("place_dropped_mail_scatters", place_dropped_mail_scatters));
    r.push(("place_newspaper_stacks", place_newspaper_stacks));
    r.push(("place_metal_lockers", place_metal_lockers));
    r.push(("place_pizza_boxes", place_pizza_boxes));
    r.push(("place_crushed_soda_cans", place_crushed_soda_cans));
    r.push(("place_rusted_hammers", place_rusted_hammers));
    r.push(("place_broken_light_switches", place_broken_light_switches));
    r.push(("place_overturned_wastebaskets", place_overturned_wastebaskets));
    r.push(("place_elevator_buttons", place_elevator_buttons));
    r.push(("place_mold_patches", place_mold_patches));
    r.push(("place_folded_umbrellas", place_folded_umbrellas));
    r.push(("place_single_boots", place_single_boots));
    r.push(("place_overturned_chairs", place_overturned_chairs));
    r.push(("place_torn_photographs", place_torn_photographs));
}

// ------------------------------------------------------------------ shared scaffolding
// Every placer in this crate repeats the same two tile scans and the same
// wall-mount tail; they are factored here with identical draws.

/// `{ x, z, dx, dz }`: an open tile with a wall on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallC {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The wall-adjacent candidate scan: one entry per open tile and wall (or grid-edge) side.
fn wall_candidates(g: &Game) -> Vec<WallC> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallC { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// The open-tile scan, row-major.
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

/// The wall-face point: `wx(c.x) + c.dx * (CELL * 0.5 - inset)`, same for z.
fn wall_xz(g: &Game, c: WallC, inset: f64) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset), g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset))
}

/// The wall-mount tail: face the room, then position with a
/// `(`rnd()` - 0.5) * jitter` slide along the wall.
fn mount(o: &Obj, c: WallC, wall_x: f64, wall_z: f64, center_y: f64, jitter: f64) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let zz = wall_z + (rnd() - 0.5) * jitter;
        o.position().set(wall_x, center_y, zz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let xx = wall_x + (rnd() - 0.5) * jitter;
        o.position().set(xx, center_y, wall_z);
    }
}

/// `o.position.set(wx(x) + (r - 0.5) * s, y, wz(z) + (r - 0.5) * s); o.rotation.y = r * 2π`
fn scatter_y(g: &Game, o: &Obj, x: i32, z: i32, spread: f64, y: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * spread;
    let pz = g.wz(z) + (rnd() - 0.5) * spread;
    o.position().set(px, y, pz);
    o.rotation().y = rnd() * PI * 2.0;
}
fn scatter(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    scatter_y(g, o, x, z, spread, 0.0)
}

/// `o.position.x = wx(x) + ...; o.position.z = wz(z) + ...; o.rotation.y = ...` (y kept)
fn scatter_xz(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    o.position().x = g.wx(x) + (rnd() - 0.5) * spread;
    o.position().z = g.wz(z) + (rnd() - 0.5) * spread;
    o.rotation().y = rnd() * PI * 2.0;
}

// R544 broken sunglasses: 7% chance per level. Frame + 2 lenses
// (one cracked, one intact) + snapped temple.
fn place_broken_sunglasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let frame_colors = [0x181008, 0x603820, 0x2050a0, 0xd82040];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let lens_mat = basic_mat!({ color: 0x2a2818, transparent: true, opacity: 0.7 });
    let glasses = group();
    // 2 lens frames.
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("BrokenSunglasses__rim", torus_geo!(0.024, 0.003, 4, 12)), &frame_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(s * 0.028, 0.003, 0);
        glasses.add(&rim);
        // Lens (dark tinted).
        let lens = mesh(circle_geo!(0.022, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(s * 0.028, 0.004, 0);
        glasses.add(&lens);
    }
    // Bridge.
    let bridge = mesh(pg("BrokenSunglasses__bridge", cyl_geo!(0.002, 0.002, 0.014, 4)), &frame_mat);
    bridge.rotation().z = PI / 2.0;
    bridge.position().y = 0.003;
    glasses.add(&bridge);
    // 1 intact temple + 1 broken temple.
    let attached_side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    let temple = mesh(pg("BrokenSunglasses__temple", cyl_geo!(0.002, 0.002, 0.10, 4)), &frame_mat);
    temple.rotation().x = PI / 2.0;
    temple.rotation().z = 0.1;
    temple.position().set(attached_side * 0.052, 0.003, -0.05);
    glasses.add(&temple);
    // Broken temple lying separately.
    let broken = mesh(pg("BrokenSunglasses__broken", cyl_geo!(0.002, 0.002, 0.06, 4)), &frame_mat);
    // Lying flat on the floor (axis horizontal), turned 45 degrees, not stuck half into it.
    broken.set_rotation_order(EulerOrder::YXZ);
    broken.rotation().set(PI / 2.0, PI / 4.0, 0);
    broken.position().set(-attached_side * 0.08, 0.003, 0.06);
    glasses.add(&broken);
    scatter(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}

// R540 fire alarm pull: 4% chance per level. A single-action pull station
// flush on its box in the wall (batch `r06`'s, the fire alarms'): FIRE raised
// over the lever's well, PULL DOWN on the lever, the glass rod across it, and
// an addressable station's LED blinking as the panel polls it.
fn place_fire_alarm_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.alarm_pull_positions = vec![]; // R541 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    // the pull station of batch `r06` (the fire alarms'), here flush on a box
    // in the wall: its cover's back (z 0.035 in its parts' frame) on the wall,
    // and the LED of an addressable station (batch `r12`)
    let alarm = group();
    let o = group();
    o.position().z = -0.008 - 0.035;
    alarm.add(&o);
    let red = lambert!({ map: &pull_paint_tex(), color: 0xffffff });
    let white = lambert!({ color: 0xc4bfb1 });
    let steel = phong!({ color: 0x6a6964, specular: 0x2c2c2c, shininess: 34 });
    let add = |name: &str, mat: &Mat, x: f64, y: f64, z: f64| {
        let m = mesh(pull_part(name, 0.1, 0.14, 0.03), mat);
        m.position().set(x, y, z);
        o.add(&m);
    };
    add("Real__pullBody", &red, 0.0, 0.0, 0.0);
    add("Real__pullLetters", &white, 0.0, 0.0, 0.0);
    add("Real__pullScrews", &steel, 0.0, 0.0, 0.0);
    add("Real__pullLock", &steel, 0.030, -0.050, 0.035 + 0.030 + 0.0035);
    // the lever up in its well, the glass rod across it in its lugs
    let pivot = group();
    pivot.position().set(0, 0.020, 0.035 + 0.030);
    o.add(&pivot);
    pivot.add(&mesh(pull_part("Real__pullHandle", 0.07, 0.03, 0.016), &red));
    pivot.add(&mesh(pull_part("Real__pullHandleText", 0.05, 0.008, 0.001), &white));
    add("Real__pullRod", &lambert!({ color: 0xc8d6d0, transparent: true, opacity: 0.5 }), 0.0, 0.002, 0.035 + 0.030 + 0.0045);
    // R541 the station's LED, dark between the panel's polls.
    let led_mat = basic_mat!({ color: 0x3a0e0a });
    let led = mesh(pull_part("Real__fireAlarmPullLed", 0.007, 0.007, 0.002), &led_mat);
    led.position().set(0.036, 0.052, 0.035 + 0.030);
    o.add(&led);
    // Its blink: a short red flash every four seconds (the old throb's phase).
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let k = (t * 0.25 + pulse_phase / (PI * 2.0)) % 1.0;
        led_mat.m().color.set_hex(if k < 0.035 { 0xd8402c } else { 0x3a0e0a });
    });
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&alarm, c, wall_x, wall_z, 1.50, 0.6);
    g.world.add(&alarm);
    // R541 track for beep audio.
    let p = alarm.pos();
    g.pp.alarm_pull_positions.push(v3(p.x, 1.50, p.z));
}

// R540 crushed water bottle: 10% chance per level. Deformed
// clear plastic w/ label.
fn place_crushed_water_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let bottle_mat = basic_mat!({ color: 0xc0d8e0, transparent: true, opacity: 0.55 });
    let cap_mat = lambert!({ color: 0x2050c0 });
    let label_mat = lambert!({ color: 0xf0ece0 });
    let bottle = group();
    // Body (partially crushed).
    let body = mesh(cyl_geo!(0.020, 0.024, 0.06, 8), &bottle_mat);
    body.scale().set(1, 1, 0.5);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.020, 0);
    bottle.add(&body);
    // Cap.
    let cap = mesh(pg("CrushedWaterBottles__cap", cyl_geo!(0.010, 0.010, 0.008, 8)), &cap_mat);
    // on the neck: the narrow (r 0.020) end is at −X; closed top (local −Y) faces out
    cap.rotation().z = -PI / 2.0;
    cap.position().set(-0.034, 0.020, 0);
    bottle.add(&cap);
    // Label band.
    let label = mesh(pg("CrushedWaterBottles__label", cyl_geo!(0.021, 0.021, 0.030, 8, 1, true)), &label_mat);
    label.rotation().z = PI / 2.0;
    label.position().set(-0.006, 0.020, 0);
    // wrap the outside of the tapered body (r up to 0.0226 under the label), not inside it
    label.scale().set(1.08, 1, 0.5 * 1.08);
    bottle.add(&label);
    // the wide end (r 0.024 about y 0.020) rests on the floor
    scatter_y(g, &bottle, x, z, 2.4, 0.004);
    g.world.add(&bottle);
}

// R539 spray bottle: 8% chance per level. Trigger-spray
// cleaning bottle w/ colored liquid.
fn place_spray_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let liquid_colors = [0x30d040, 0x2050d8, 0xf0d020, 0xd82040];
    let liquid_color = choice(&liquid_colors);
    let liquid_mat = basic_mat!({ color: liquid_color, transparent: true, opacity: 0.75 });
    let trigger_mat = lambert!({ color: 0xe0e0e0 });
    let bottle = group();
    // Body.
    let body = mesh(cyl_geo!(0.028, 0.030, 0.14, 10), &liquid_mat);
    body.position().y = 0.070;
    bottle.add(&body);
    // Trigger head.
    let trigger_head = mesh(pg("SprayBottles__head", box_geo!(0.035, 0.030, 0.020)), &trigger_mat);
    trigger_head.position().set(0.005, 0.155, 0);
    bottle.add(&trigger_head);
    // Trigger lever.
    let lever = mesh(pg("SprayBottles__lever", box_geo!(0.024, 0.006, 0.010)), &trigger_mat);
    // Hangs under the nozzle at the front (+X), finger scallops forward (was sticking out the back).
    lever.rotation().z = PI / 2.0 + 0.4;
    lever.position().set(0.027, 0.146, 0);
    bottle.add(&lever);
    // Nozzle.
    let nozzle = mesh(pg("SprayBottles__nozzle", cyl_geo!(0.003, 0.003, 0.014, 5)), &trigger_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(0.028, 0.158, 0);
    bottle.add(&nozzle);
    // 30% chance tipped.
    if rnd() < 0.30 {
        bottle.rotation().z = PI / 2.0;
        bottle.position().y = 0.028;
    }
    scatter_xz(g, &bottle, x, z, 2.0);
    g.world.add(&bottle);
}

// R539 wall soap dispenser: 6% chance per level. Push-plunger
// wall unit w/ pink soap visible.
fn place_soap_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let body_mat = basic_mat!({ color: 0xf0e0e0, transparent: true, opacity: 0.75 });
    let soap_mat = basic_mat!({ color: 0xf0a080 });
    let nozzle_mat = lambert!({ color: 0x606060 });
    let dispenser = group();
    // Body (translucent).
    let body = mesh(box_geo!(0.12, 0.20, 0.08), &body_mat);
    body.position().set(0, 0, -0.04);
    dispenser.add(&body);
    // Soap inside (partial fill).
    let soap = mesh(box_geo!(0.10, 0.12, 0.06), &soap_mat);
    soap.position().set(0, -0.020, -0.04);
    dispenser.add(&soap);
    // Nozzle at bottom.
    let nozzle = mesh(pg("SoapDispensers__nozzle", cyl_geo!(0.010, 0.012, 0.020, 6)), &nozzle_mat);
    nozzle.position().set(0, -0.11, -0.010);
    dispenser.add(&nozzle);
    // Push plunger on top.
    let plunger = mesh(pg("SoapDispensers__plunger", box_geo!(0.08, 0.014, 0.06)), &nozzle_mat);
    plunger.position().set(0, 0.11, -0.04);
    dispenser.add(&plunger);
    // Origin 8 cm off the wall: the 8 cm deep body (local z -0.08..0) backs onto
    // the wall face (at 4 cm its back half was inside the wall).
    let (wall_x, wall_z) = wall_xz(g, c, 0.080);
    mount(&dispenser, c, wall_x, wall_z, 1.30, 0.4);
    g.world.add(&dispenser);
}

// R538 deflated sport ball: 7% chance per level. Flattened
// sphere w/ hex/pentagon panels.
fn place_deflated_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let is_soccer = rnd() < 0.5;
    let ball_color = if is_soccer { 0xe8dcc0 } else { 0xd87020 };
    let ball_mat = lambert!({ color: ball_color });
    let patch_mat = lambert!({ color: if is_soccer { 0x181410 } else { 0x181410 } });
    let ball = group();
    // Flattened body.
    let body = mesh(pg("DeflatedBalls__body", sphere_geo!(0.075, 12, 8)), &ball_mat);
    body.scale().set(1, 0.55, 1);
    body.position().y = 0.042;
    ball.add(&body);
    // 3-4 patches (pentagons or lines).
    let n_patches = 3 + (rnd() * 2.0).floor() as i32;
    let npf = n_patches as f64;
    for i in 0..n_patches {
        let fi = i as f64;
        let a = (fi / npf) * PI * 2.0;
        if is_soccer {
            // Dark pentagon patches.
            let patch = mesh(circle_geo!(0.014, 5), &patch_mat);
            // on the flattish top, spread over the arc clear of the buckled side
            // (+X), tilted with the crown's slope and resting on it
            let pa = 1.1 + ((fi + 0.5) / npf) * 4.3;
            {
                let mut r = patch.rotation();
                r.set(-PI / 2.0 + 0.16, PI / 2.0 - pa, 0);
                r.order = EulerOrder::YXZ;
            }
            patch.position().set(pa.cos() * 0.028, 0.0814, pa.sin() * 0.028);
            ball.add(&patch);
        } else {
            // Basket lines.
            let line = mesh(box_geo!(0.10, 0.001, 0.002), &patch_mat);
            line.rotation().y = a;
            line.scale().x = 0.45; // only across the crown, where a straight line can lie on it
            line.position().y = 0.0831;
            ball.add(&line);
        }
    }
    scatter(g, &ball, x, z, 2.0);
    g.world.add(&ball);
}

// R537 wall pay phone: 4% chance per level. A single-slot payphone of the Western
// kind (Blender batch `r12`): the stainless upper housing with its instruction
// card, coin entry, keypad and coin return, the black vault under it, the
// handset hung on the switchhook at its side on its armoured cord.
fn place_wall_pay_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.pay_phone_positions = vec![]; // R542 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let phone = group();
    // the parts are modelled with the wall at z = 0: the housing's back on it
    let o = group();
    o.position().z = -0.05;
    phone.add(&o);
    let steel = phong!({ map: &pay_steel_tex(), color: 0xa6a6a0, specular: 0x383836, shininess: 22 });
    o.add(&mesh(pay_part("Real__wallPayPhoneUpper", 0.2, 0.3, 0.135), &steel));
    o.add(&mesh(pay_part("Real__wallPayPhoneVault", 0.196, 0.23, 0.124), lambert!({ map: &pay_vault_tex(), color: 0x2e2e2b })));
    let chrome = phong!({ color: 0x8c8c88, specular: 0x5a5a58, shininess: 60 });
    o.add(&mesh(pay_part("Real__wallPayPhoneKeys", 0.07, 0.095, 0.009), &chrome));
    o.add(&mesh(pay_part("Real__wallPayPhoneTrim", 0.26, 0.3, 0.14), &chrome));
    let decal = |tex: &Tex| lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let legends = mesh(plane_geo!(0.075, 0.100), decal(&pay_keys_tex()));
    legends.position().set(0.0, 0.357, PP_D + 0.0058);
    o.add(&legends);
    let card = mesh(plane_geo!(0.110, 0.072), lambert!({ map: &pay_card_tex() }));
    card.position().set(PP_CARD.0, PP_CARD.1, PP_D - 0.0027);
    o.add(&card);
    let window = mesh(plane_geo!(0.110, 0.072), decal(&pay_window_tex()));
    window.position().set(PP_CARD.0, PP_CARD.1, PP_D - 0.0006);
    o.add(&window);
    let handset =
        mesh(pay_part("Real__wallPayPhoneHandset", 0.065, 0.23, 0.045), phong!({ color: 0x1e1d1c, specular: 0x262626, shininess: 18 }));
    handset.position().set(-0.1025, 0.358, 0.070);
    handset.rotation().y = -PI / 2.0;
    o.add(&handset);
    o.add(&mesh(pay_part("Real__wallPayPhoneCord", 0.03, 0.16, 0.02), phong!({ color: 0x7a7a75, specular: 0x404040, shininess: 40 })));
    let (wall_x, wall_z) = wall_xz(g, c, 0.050);
    mount(&phone, c, wall_x, wall_z, 0.7, 0.4);
    g.world.add(&phone);
    // R542 track for ring audio.
    let p = phone.pos();
    g.pp.pay_phone_positions.push(v3(p.x, 1.0, p.z));
}

// R537 matchbook: 8% chance per level. Small booklet + 4-6
// scattered matches.
fn place_matchbooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let cover_colors = [0xa02040, 0x2050a0, 0x30a030, 0xd8b040];
    let cover_mat = lambert!({ color: choice(&cover_colors) });
    let match_stick_mat = lambert!({ color: 0xe8dcc0 });
    let match_head_mat = lambert!({ color: 0x8a3020 });
    let grp = group();
    // Booklet body.
    let booklet = mesh(pg("Matchbooks__booklet", box_geo!(0.030, 0.005, 0.040)), &cover_mat);
    booklet.position().y = 0.003;
    grp.add(&booklet);
    // 5 scattered matches.
    let n_matches = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_matches {
        let m = group();
        let stick = mesh(pg("Matchbooks__stick", cyl_geo!(0.0008, 0.0008, 0.024, 4)), &match_stick_mat);
        stick.rotation().z = PI / 2.0;
        stick.position().x = 0.012;
        m.add(&stick);
        let head = mesh(pg("Matchbooks__head", sphere_geo!(0.0015, 5, 4)), &match_head_mat);
        head.position().x = 0.024;
        m.add(&head);
        let px = (rnd() - 0.5) * 0.12;
        let pz = (rnd() - 0.5) * 0.08;
        m.position().set(px, 0.0012, pz);
        m.rotation().y = rnd() * PI * 2.0;
        grp.add(&m);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R536 small transistor radio: 6% chance per level. Small box
// + dial + speaker grille + antenna.
fn place_small_radios(g: &mut Game, _dist: &[i32]) {
    g.pp.radio_positions = vec![]; // R542 reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let case_colors = [0x8a5030, 0x604020, 0x181410, 0xa02040];
    let case_mat = lambert!({ color: choice(&case_colors) });
    let grille_mat = lambert!({ color: 0x2a2418 });
    let dial_mat = lambert!({ color: 0xe8dcc0 });
    let ant_mat = lambert!({ color: 0xc0c0c0 });
    let radio = group();
    // Body.
    let body = mesh(pg("SmallRadios__body", box_geo!(0.12, 0.07, 0.04)), &case_mat);
    body.position().y = 0.035;
    radio.add(&body);
    // Speaker grille (dark disc on front-left).
    let grille = mesh(circle_geo!(0.024, 12), &grille_mat);
    grille.position().set(-0.030, 0.035, 0.021);
    radio.add(&grille);
    // Dial (small cream rectangle on front-right).
    let dial = mesh(plane_geo!(0.036, 0.020), &dial_mat);
    dial.position().set(0.030, 0.045, 0.021);
    radio.add(&dial);
    // Tuning knob.
    let knob = mesh(pg("SmallRadios__knob", cyl_geo!(0.006, 0.006, 0.005, 8)), &grille_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0.045, 0.020, 0.020);
    radio.add(&knob);
    // Antenna (extended from top).
    let ant = mesh(pg("SmallRadios__antenna", cyl_geo!(0.001, 0.001, 0.12, 4)), &ant_mat);
    ant.rotation().z = 0.4;
    // lower end (centre + (0.06 sin 0.4, −0.06 cos 0.4)) 2 mm into the case top
    // near its right end at (0.05, 0.068); it floated ~5 mm off the corner
    ant.position().set(0.05 - 0.06 * 0.4f64.sin(), 0.068 + 0.06 * 0.4f64.cos(), 0);
    radio.add(&ant);
    scatter(g, &radio, x, z, 2.4);
    g.world.add(&radio);
    // R542 track for AM static audio.
    let p = radio.pos();
    g.pp.radio_positions.push(v3(p.x, 0.035, p.z));
}

// R536 cigarette pack: 10% chance per level. Small flip-top
// box in red/blue/green colors.
fn place_cigarette_packs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let pack_colors = [0xa02020, 0x2050c0, 0x30a030, 0xd8b040, 0xe8dcc0];
    let pack_mat = lambert!({ color: choice(&pack_colors) });
    let foil_mat = basic_mat!({ color: 0xd0d0d0 });
    let pack = group();
    // Body.
    let body = mesh(pg("CigarettePacks__body", box_geo!(0.05, 0.08, 0.020)), &pack_mat);
    body.position().y = 0.040;
    pack.add(&body);
    // 50% chance flipped open — silver foil visible.
    if rnd() < 0.50 {
        let foil = mesh(plane_geo!(0.048, 0.018), &foil_mat);
        foil.rotation().x = -PI / 2.0;
        foil.position().set(0, 0.081, 0);
        pack.add(&foil);
        // Lid tilted back.
        let lid = mesh(pg("CigarettePacks__lid", box_geo!(0.05, 0.020, 0.004)), &pack_mat);
        lid.position().set(0, 0.086, -0.0187); // lower (hinge) end on the top back edge, z -0.010
        lid.rotation().x = -PI / 3.0;
        pack.add(&lid);
    }
    // 30% tipped on side.
    if rnd() < 0.30 {
        pack.rotation().z = PI / 2.0;
        pack.position().y = 0.025;
    }
    scatter_xz(g, &pack, x, z, 2.4);
    g.world.add(&pack);
}

// R535 wall bookshelf: 3% chance per level. Wooden shelf w/
// multi-color book spines.
fn place_bookshelves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let wood_mat = lambert!({ color: 0x603820 });
    let book_colors = [0xa02040, 0x2050a0, 0x30a030, 0xd8b040, 0x604020, 0x181410, 0xd8a080];
    let shelf = group();
    let (bw, bh, bd) = (0.60, 0.16, 0.16);
    // Shelf back panel.
    let back = mesh(pg("Bookshelves__back", box_geo!(bw, bh, 0.014)), &wood_mat);
    back.position().set(0, bh / 2.0, -bd / 2.0 + 0.007);
    shelf.add(&back);
    // Top + bottom shelves.
    for yy in [0.0, bh] {
        let board = mesh(pg("Bookshelves__board", box_geo!(bw, 0.014, bd)), &wood_mat);
        board.position().set(0, yy, 0);
        shelf.add(&board);
    }
    // 2 side panels.
    for s in [-1.0, 1.0] {
        let side = mesh(pg("Bookshelves__side", box_geo!(0.014, bh, bd)), &wood_mat);
        side.position().set(s * (bw / 2.0 - 0.007), bh / 2.0, 0);
        shelf.add(&side);
    }
    // 12-18 book spines standing.
    let n_books = 12 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_books {
        let book_mat = lambert!({ color: choice(&book_colors) });
        let spine_w = 0.014 + rnd() * 0.014;
        let spine_h = 0.11 + rnd() * 0.03;
        let book = mesh(pg("Bookshelves__book", box_geo!(spine_w, spine_h, 0.08)), &book_mat);
        book.position().set(-bw / 2.0 + 0.014 + (i as f64 + 0.5) * ((bw - 0.028) / n_books as f64), spine_h / 2.0 + 0.008, -0.020);
        // 20% chance leaning.
        if rnd() < 0.20 {
            book.rotation().z = (rnd() - 0.5) * 0.3;
        }
        shelf.add(&book);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.080);
    mount(&shelf, c, wall_x, wall_z, 1.4, 0.4);
    g.world.add(&shelf);
}

// R535 keychain w/ keys: 8% chance per level. Metal ring + 3-5
// keys dangling.
fn place_keychain_keys(g: &mut Game, _dist: &[i32]) {
    g.pp.keychain_positions = vec![]; // R543 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let metal_mat = lambert!({ color: 0xc0c0c0 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let keychain = group();
    // Ring.
    let ring = mesh(pg("KeychainKeys__ring", torus_geo!(0.014, 0.002, 4, 10)), &metal_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.003;
    keychain.add(&ring);
    // 3-5 keys fanned.
    let n_keys = 3 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_keys {
        let use_brass = rnd() < 0.5;
        let key_mat = if use_brass { &brass_mat } else { &metal_mat };
        let key = group();
        // Shaft.
        let shaft = mesh(pg("KeychainKeys__blade", cyl_geo!(0.002, 0.002, 0.05, 4)), key_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().x = 0.030;
        key.add(&shaft);
        // Bow (ring end).
        let bow = mesh(pg("KeychainKeys__bow", torus_geo!(0.008, 0.002, 4, 8)), key_mat);
        bow.rotation().x = PI / 2.0; // flat like the ring and blade (turned about Y it stood up, half in the floor)
        bow.position().x = 0.006;
        key.add(&bow);
        // Tooth notch.
        let tooth = mesh(pg("KeychainKeys__tooth", box_geo!(0.004, 0.005, 0.002)), key_mat);
        // lying flat, sticking out of the blade's edge near the tip (not on top of it)
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.052, 0, 0.0035);
        key.add(&tooth);
        // Position fanned from ring.
        let a = -PI / 4.0 + (i as f64 / (n_keys - 1) as f64) * (PI / 2.0);
        key.position().y = 0.003;
        key.rotation().y = a;
        keychain.add(&key);
    }
    scatter(g, &keychain, x, z, 2.4);
    g.world.add(&keychain);
    // R543 track for jangle audio.
    let p = keychain.pos();
    g.pp.keychain_positions.push(v3(p.x, 0.003, p.z));
}

// R534 safety vest: 8% chance per level. Folded neon vest w/
// reflective silver stripes.
fn place_safety_vests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let vest_colors = [0xe0f020, 0xe07020, 0xe02020];
    let vest_mat = lambert!({ color: choice(&vest_colors) });
    let stripe_mat = basic_mat!({ color: 0xc0c0c0 });
    let vest = group();
    // Body (folded rectangle).
    let body = mesh(pg("SafetyVests__body", box_geo!(0.20, 0.020, 0.16)), &vest_mat);
    body.position().y = 0.010;
    vest.add(&body);
    // 2 reflective horizontal stripes.
    for sz in [-0.02, 0.02] {
        let stripe = mesh(plane_geo!(0.20, 0.020), &stripe_mat);
        stripe.rotation().x = -PI / 2.0;
        stripe.position().set(0, 0.021, sz);
        vest.add(&stripe);
    }
    scatter(g, &vest, x, z, 2.0);
    g.world.add(&vest);
}

// R533 bulletin board: 5% chance per level. Cork board with
// scattered notes + pushpins.
// (Referenced by no placer; kept for completeness. Lazily built, so it
// costs nothing unless used.)
game::lazy_static_tex!(
    bulletin_texture_2,
    canvas_tex(384, 320, |ctx, w, h| {
        // Cork background.
        ctx.set_fill_style("#8a6540");
        ctx.fill_rect(0, 0, w, h);
        // Cork texture speckles.
        for _ in 0..300 {
            ctx.set_fill_style(&format!("rgba(60,40,20,{})", 0.15 + rnd() * 0.20));
            ctx.begin_path();
            let (ax, ay, ar) = (rnd() * w, rnd() * h, 1.0 + rnd() * 2.0);
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // 6-8 pinned notes.
        let note_colors = ["#f0ece0", "#f0e0a0", "#e0d0f0", "#d0e8f0"];
        for _ in 0..7 {
            let nx = 20.0 + rnd() * (w - 100.0);
            let ny = 20.0 + rnd() * (h - 80.0);
            let nw = 60.0 + rnd() * 40.0;
            let nh = 50.0 + rnd() * 30.0;
            ctx.save();
            ctx.translate(nx + nw / 2.0, ny + nh / 2.0);
            ctx.rotate((rnd() - 0.5) * 0.3);
            ctx.set_fill_style(note_colors[(rnd() * note_colors.len() as f64).floor() as usize]);
            ctx.fill_rect(-nw / 2.0, -nh / 2.0, nw, nh);
            // Text lines.
            ctx.set_stroke_style("rgba(40,20,10,0.7)");
            ctx.set_line_width(1.5);
            for l in 0..3 {
                let lf = l as f64;
                ctx.begin_path();
                ctx.move_to(-nw / 2.0 + 6.0, -nh / 2.0 + 12.0 + lf * 10.0);
                ctx.line_to(-nw / 2.0 + 6.0 + nw * 0.7, -nh / 2.0 + 12.0 + lf * 10.0);
                ctx.stroke();
            }
            // Pushpin (red).
            ctx.set_fill_style("#d82040");
            ctx.begin_path();
            ctx.arc(0, -nh / 2.0 + 6.0, 4, 0, PI * 2.0);
            ctx.fill();
            ctx.restore();
        }
    })
);

// R533 safety hard hat: 8% chance per level. Yellow domed hat
// w/ brim + chin strap.
fn place_hard_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let hat_colors = [0xd8b040, 0xa02020, 0x2050c0, 0xf0e8d0];
    let hat_mat = lambert!({ color: choice(&hat_colors) });
    let strap_mat = lambert!({ color: 0x181008 });
    let hat = group();
    // Dome.
    let dome = mesh(pg("HardHats__dome", sphere_geo!(0.070, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &hat_mat);
    dome.position().y = 0.020;
    hat.add(&dome);
    // Brim.
    let brim = mesh(pg("HardHats__brim", cyl_geo!(0.070, 0.080, 0.010, 12)), &hat_mat);
    brim.position().y = 0.020;
    hat.add(&brim);
    // Front brim extension.
    let front = mesh(pg("HardHats__peak", box_geo!(0.030, 0.008, 0.030)), &hat_mat);
    front.position().set(0.075, 0.024, 0);
    hat.add(&front);
    // 2 side strap dots.
    for s in [-1.0, 1.0] {
        let strap = mesh(pg("HardHats__strapDot", cyl_geo!(0.006, 0.006, 0.004, 6)), &strap_mat);
        strap.rotation().z = PI / 2.0;
        strap.position().set(0, 0.025, s * 0.070);
        hat.add(&strap);
    }
    scatter(g, &hat, x, z, 2.4);
    g.world.add(&hat);
}

// R532 fire hose reel: 4% chance per level. Red drum reel +
// coiled hose + nozzle valve.
fn place_fire_hose_reels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let red_mat = lambert!({ color: 0xa02020 });
    let hose_mat = lambert!({ color: 0xf0e8d8 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let reel = group();
    // Mounting plate.
    let plate = mesh(pg("FireHoseReels__plate", box_geo!(0.42, 0.42, 0.010)), &red_mat);
    plate.position().z = -0.005;
    reel.add(&plate);
    // Drum cylinder (thick).
    let drum = mesh(pg("FireHoseReels__drum", cyl_geo!(0.10, 0.10, 0.08, 12)), &red_mat);
    drum.rotation().x = PI / 2.0;
    drum.scale().set(1.2, 1, 1.2); // r 0.12: the hose coils (inner r 0.12) now wrap it (was a 2 cm gap)
    drum.position().set(0, 0, 0.040);
    reel.add(&drum);
    // Coiled hose (thick torus around drum).
    let hose = mesh(pg("FireHoseReels__hose", torus_geo!(0.14, 0.020, 6, 16)), &hose_mat);
    hose.position().set(0, 0, 0.030);
    reel.add(&hose);
    // Second wrap for depth.
    let hose2 = mesh(pg("FireHoseReels__hose", torus_geo!(0.14, 0.018, 6, 16)), &hose_mat);
    hose2.position().set(0, 0, 0.058);
    reel.add(&hose2);
    // Brass nozzle valve on side of drum.
    let nozzle = mesh(pg("FireHoseReels__nozzle", cyl_geo!(0.014, 0.010, 0.030, 6)), &brass_mat);
    nozzle.position().set(0.14, -0.14, 0.055);
    nozzle.rotation().z = 0.4;
    reel.add(&nozzle);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&reel, c, wall_x, wall_z, 1.50, 0.4);
    g.world.add(&reel);
}

// R532 single glove: 8% chance per level. One work glove
// (leather/knit) on floor.
fn place_single_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let glove_colors = [0x6a3820, 0x2a2418, 0x8a3020, 0x304050, 0xa08050];
    let glove_mat = lambert!({ color: choice(&glove_colors) });
    let glove = group();
    // Palm block.
    let palm = mesh(pg("SingleGloves__palm", box_geo!(0.055, 0.014, 0.045)), &glove_mat);
    palm.position().y = 0.007;
    glove.add(&palm);
    // 4 fingers.
    for f in 0..4 {
        let finger = mesh(pg("SingleGloves__finger", box_geo!(0.010, 0.010, 0.035)), &glove_mat);
        finger.position().set(-0.018 + f as f64 * 0.012, 0.005, 0.040);
        glove.add(&finger);
    }
    // Thumb (angled).
    let thumb = mesh(pg("SingleGloves__finger", box_geo!(0.010, 0.010, 0.028)), &glove_mat);
    thumb.rotation().z = -0.5;
    thumb.position().set(-0.030, 0.008, 0.020);
    glove.add(&thumb);
    // Cuff.
    let cuff = mesh(pg("SingleGloves__cuff", cyl_geo!(0.024, 0.024, 0.020, 8, 1, true)), &glove_mat);
    cuff.rotation().x = PI / 2.0;
    cuff.scale().z = 0.4; // flattened like the lying palm: 1.9 cm tall, not a 4.8 cm tube
    cuff.position().set(0, 0.0096, -0.030);
    glove.add(&cuff);
    scatter(g, &glove, x, z, 2.4);
    g.world.add(&glove);
}

// R531 dropped mail scatter: 8% chance per level. 5-9
// envelopes fanned on floor.
fn place_dropped_mail_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let env_colors = [0xf0ece0, 0xe8d8b8, 0xf0d8c8, 0xd8e8d0];
    let ink_mat = lambert!({ color: 0x201810 });
    let grp = group();
    let n_env = 5 + (rnd() * 5.0).floor() as i32;
    for i in 0..n_env {
        let env_mat = lambert!({ color: choice(&env_colors) });
        let env = mesh(pg("DroppedMailScatters__envelope", box_geo!(0.12, 0.001, 0.06)), &env_mat);
        let px = (rnd() - 0.5) * 0.35;
        let py = 0.0005 + i as f64 * 0.0006; // bottom one on the floor (1 mm thick), the rest stacked
        let pz = (rnd() - 0.5) * 0.30;
        env.position().set(px, py, pz);
        env.rotation().y = rnd() * PI * 2.0;
        grp.add(&env);
        // Small address rectangle.
        if rnd() < 0.5 {
            let addr = mesh(plane_geo!(0.030, 0.008), &ink_mat);
            addr.rotation().x = -PI / 2.0;
            // offset 2 cm along the envelope's own long axis (its yaw), just over its top
            let (ep, ery) = (env.pos(), env.rot().y);
            addr.position().set(ep.x + 0.02 * ery.cos(), ep.y + 0.0006, ep.z - 0.02 * ery.sin());
            addr.rotation().z = ery;
            grp.add(&addr);
        }
    }
    scatter(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R530 newspaper stack: 8% chance per level. 3-6 folded
// newspapers stacked with yellowed pages.
fn place_newspaper_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let paper_colors = [0xe8d8b0, 0xd8c8a0, 0xe8dcc0];
    let ink_mat = lambert!({ color: 0x201810 });
    let grp = group();
    let n_papers = 3 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_papers {
        let paper_mat = lambert!({ color: choice(&paper_colors) });
        let paper = mesh(pg("NewspaperStacks__paper", box_geo!(0.20, 0.008, 0.14)), &paper_mat);
        paper.position().y = 0.004 + i as f64 * 0.008;
        paper.position().x = (rnd() - 0.5) * 0.010;
        paper.position().z = (rnd() - 0.5) * 0.010;
        paper.rotation().y = (rnd() - 0.5) * 0.15;
        grp.add(&paper);
        // Ink title bar on each paper's top.
        if i == n_papers - 1 {
            let title = mesh(box_geo!(0.12, 0.001, 0.020), &ink_mat);
            let pp = paper.pos();
            title.position().set(pp.x, pp.y + 0.005, pp.z - 0.04);
            title.rotation().y = paper.rot().y;
            grp.add(&title);
        }
    }
    scatter(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R530 metal locker: 4% chance per level. Wall-adjacent tall
// gray/green locker w/ vent slats + handle.
fn place_metal_lockers(g: &mut Game, _dist: &[i32]) {
    g.pp.locker_positions = vec![]; // R531 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let locker_colors = [0x606870, 0x506050, 0x403830, 0xd8b060];
    let body_mat = lambert!({ color: choice(&locker_colors) });
    let dark_mat = lambert!({ color: 0x101410 });
    let locker = group();
    // Body (tall box).
    let body = mesh(pg("MetalLockers__body", box_geo!(0.30, 1.60, 0.30)), &body_mat);
    body.position().set(0, 0.80, -0.15);
    locker.add(&body);
    // 4 vent slats on top of door.
    for i in 0..4 {
        let vent = mesh(box_geo!(0.14, 0.004, 0.002), &dark_mat);
        vent.position().set(0, 1.44 + i as f64 * 0.010, 0);
        locker.add(&vent);
    }
    // Handle.
    let handle = mesh(pg("MetalLockers__handle", box_geo!(0.010, 0.030, 0.010)), &dark_mat);
    handle.position().set(0.10, 0.90, 0);
    locker.add(&handle);
    // Number plate.
    let plate = mesh(plane_geo!(0.05, 0.03), &dark_mat);
    plate.position().set(-0.06, 1.35, 0.001);
    locker.add(&plate);
    // origin = the door face; the 0.30-deep body (local z −0.30…0) backs onto the wall
    let (wall_x, wall_z) = wall_xz(g, c, 0.300);
    mount(&locker, c, wall_x, wall_z, 0.0, 0.4);
    g.world.add(&locker);
    // R531 track for slam audio.
    let p = locker.pos();
    g.pp.locker_positions.push(v3(p.x, 0.8, p.z));
}

// R529 pizza box: 7% chance per level. Empty square box w/
// grease stain interior.
fn place_pizza_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let box_mat = lambert!({ color: 0xd8b070 });
    let grease_mat = lambert!({ color: 0x8a5030 });
    let bx = group();
    let bs = 0.28;
    // Body.
    let body = mesh(pg("PizzaBoxes__body", box_geo!(bs, 0.024, bs)), &box_mat);
    body.position().y = 0.012;
    bx.add(&body);
    // Grease stain on top.
    let grease = mesh(circle_geo!(0.10, 12), &grease_mat);
    grease.rotation().x = -PI / 2.0;
    grease.position().y = 0.025;
    bx.add(&grease);
    // 40% chance open (lid tilted back).
    if rnd() < 0.40 {
        let lid = mesh(pg("PizzaBoxes__lid", box_geo!(bs, 0.008, bs)), &box_mat);
        // Hinged on the body's top back edge (y 0.024, z -bs/2), flopped back 120 degrees.
        lid.position().set(0, 0.143, -bs / 2.0 - 0.0735);
        lid.rotation().x = -2.0 * PI / 3.0;
        bx.add(&lid);
    }
    scatter(g, &bx, x, z, 1.8);
    g.world.add(&bx);
}

// R529 crushed soda can: 10% chance per level: a 355 ml aluminium can
// crushed underfoot (Blender batch r08): stamped flat on its end, its printed
// wall folded into rings and the lid with its tab slid off centre, or stepped
// on lying down, pressed flat between its ends; its label a plain one of no
// real brand, faded, scratched and cracked along its folds to the metal.
fn place_crushed_soda_cans(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for five plain labels (cola, club soda,
    // lemon-lime, orange, sparkling water)
    let can_colors = [0xa02040, 0x2050c0, 0x30a030, 0xd8b040, 0xc0c0c0];
    let col = choice(&can_colors);
    let k = can_colors.iter().position(|&c| c == col).unwrap_or(0);
    let can = group();
    scatter_y(g, &can, x, z, 2.4, 0.0);
    // how it was crushed, from where it lies (no new draw)
    let p = can.pos();
    let flat = ((p.x * 7.31 + p.z * 3.17).abs().fract()) < 0.5;
    let label = phong!({ map: &crushed_can_label(k, flat), color: 0xffffff, specular: 0x4a4a4a, shininess: 40 });
    let alu = phong!({ color: 0x8e9092, specular: 0x6a6a6a, shininess: 52 });
    let parts = if flat { ["Real__canFlatBody", "Real__canFlatEnds"] } else { ["Real__canCrushedBody", "Real__canCrushedEnds"] };
    for (name, mat) in parts.iter().zip([&label, &alu]) {
        if let Some(geo) = prop_part_as_modelled(name) {
            can.add(&mesh(geo, mat));
        }
    }
    g.world.add(&can);
}
/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
fn r08_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The crushed can's label, unrolled round its wall: the name of what was in
/// it (no real brand), its band, 12 FL OZ (355 mL), a nutrition panel and a
/// barcode; faded, scuffed, scratched to the aluminium and cracked bare
/// along its folds. Stamped on its end, the whole label round it (u round
/// it, v up the 93 mm of its wall, folded into the crushed part's height);
/// pressed flat, the half of it on top (u along it, the lid at u = 1; v
/// across, its folded edges at v = 0 and 1).
fn crushed_can_label(k: usize, flat: bool) -> Tex {
    // ground, band, ink (sRGB), the name, its face
    const LABELS: [(u32, u32, u32, &str, &str); 5] = [
        (0x8a2428, 0xe2dccc, 0xf0ebe0, "COLA", "bold italic 92px Futura"),
        (0x24365e, 0xc8ccd0, 0xeeeeea, "CLUB SODA", "bold 64px Gill Sans"),
        (0x2c5a34, 0xd8cc6a, 0xf2efe2, "LEMON-LIME", "bold 58px Futura"),
        (0xc0782a, 0xf0e2c0, 0xfaf4e6, "ORANGE", "bold 80px Rockwell"),
        (0xb4b6b6, 0x3a5a86, 0x2c4a74, "SPARKLING WATER", "bold 44px Avenir Next Condensed"),
    ];
    // the crush, as modelled (r08.py `CrushedCan`, `CAN_PH`)
    const PH: [f64; 5] = [1.3, 4.1, 0.6, 2.9, 5.2];
    let (ground, band, ink, name, face) = LABELS[k];
    let (cw, ch) = if flat { (256, 256) } else { (512, 256) };
    canvas_tex(cw, ch, move |ctx, cw, ch| {
        // the label is laid out unrolled, 512 round by 256 up; pressed flat
        // its first half lies along the canvas (its top at the canvas' right)
        let (w, h) = (512.0, 256.0);
        if flat {
            ctx.set_transform(0, 1, -1, 0, cw, 0);
        }
        let hex = |c: u32| format!("#{c:06x}");
        ctx.set_fill_style(hex(ground).as_str());
        ctx.fill_rect(0, 0, w, h);
        // a band round it, waving
        ctx.set_fill_style(hex(band).as_str());
        ctx.begin_path();
        ctx.move_to(0, h * 0.70);
        for i in 0..=32 {
            let x = i as f64 / 32.0 * w;
            ctx.line_to(x, h * 0.70 + (x / w * PI * 4.0).sin() * 10.0);
        }
        ctx.line_to(w, h * 0.82);
        for i in (0..=32).rev() {
            let x = i as f64 / 32.0 * w;
            ctx.line_to(x, h * 0.82 + (x / w * PI * 4.0 + 0.8).sin() * 8.0);
        }
        ctx.close_path();
        ctx.fill();
        // the name, twice round, along the can
        ctx.set_fill_style(hex(ink).as_str());
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font(face);
        for c in [0.25, 0.75] {
            ctx.fill_text_max(name, w * c, h * 0.42, w * 0.44);
        }
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text("12 FL OZ (355 mL)", w * 0.25, h * 0.92);
        ctx.fill_text("12 FL OZ (355 mL)", w * 0.75, h * 0.92);
        // the nutrition panel and the barcode, on the back
        let (px, py) = (w * 0.47, h * 0.12);
        ctx.set_fill_style("rgba(236,232,222,0.92)");
        ctx.fill_rect(px, py, 44, 74);
        ctx.set_fill_style("rgba(30,28,26,0.9)");
        ctx.set_font("bold 7px Helvetica");
        ctx.set_text_align("left");
        ctx.fill_text("Nutrition Facts", px + 3.0, py + 6.0);
        ctx.set_font("5px Helvetica");
        for (i, l) in ["Serving size 1 can", "Calories 140", "Total Fat 0g", "Sodium 45mg", "Total Carb. 39g", "Sugars 39g", "Protein 0g"]
            .iter()
            .enumerate()
        {
            ctx.fill_text(l, px + 3.0, py + 15.0 + i as f64 * 7.5);
        }
        ctx.set_fill_style("rgba(236,232,222,0.92)");
        ctx.fill_rect(w * 0.025, h * 0.14, 26, 40);
        ctx.set_fill_style("rgba(20,20,20,0.9)");
        let mut bx = w * 0.025 + 3.0;
        while bx < w * 0.025 + 23.0 {
            let bw = 0.6 + rnd() * 1.6;
            ctx.fill_rect(bx, h * 0.14 + 3.0, bw, 30);
            bx += bw + 0.6 + rnd() * 1.2;
        }
        ctx.set_font("5px Helvetica");
        ctx.fill_text("PLEASE RECYCLE", w * 0.025, h * 0.14 + 48.0);
        // faded: years under the light and the dust
        ctx.set_fill_style("rgba(196,192,182,0.16)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let at = (rnd() * w, rnd() * h);
            r08_soft(ctx, at, 14.0 + rnd() * 40.0, 0.5 + rnd() * 0.3, rnd() * 3.0, (110, 100, 84, 0.08 + rnd() * 0.16));
        }
        ctx.reset_transform();
        let (w, h) = (cw, ch);
        if flat {
            // cracked bare along its two folded edges and where it creased
            // going flat by its ends
            ctx.set_stroke_style("rgba(192,194,196,0.7)");
            ctx.set_line_width(2.0);
            for y in [3.0, h - 3.0] {
                ctx.begin_path();
                ctx.move_to(w * 0.18, y + (rnd() - 0.5) * 3.0);
                for i in 1..=12 {
                    ctx.line_to(w * (0.18 + 0.64 * i as f64 / 12.0), y + (rnd() - 0.5) * 3.0);
                }
                ctx.stroke();
            }
            ctx.set_line_width(1.4);
            for _ in 0..10 {
                let x = if rnd() < 0.5 { w * (0.08 + rnd() * 0.2) } else { w * (0.72 + rnd() * 0.2) };
                let y = rnd() * h;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + (rnd() - 0.5) * 30.0, y + 20.0 + rnd() * 50.0);
                ctx.stroke();
            }
        }
        // cracked bare where the folds' crests bent it (3 folds, out of step round it)
        ctx.set_line_width(2.2);
        for f in (1..=3).filter(|_| !flat) {
            ctx.set_stroke_style("rgba(192,194,196,0.7)");
            ctx.begin_path();
            for i in 0..=96 {
                let u = i as f64 / 96.0;
                let a = u * PI * 2.0;
                let lag = 0.18 * (a + PH[0]).sin() + 0.08 * (2.0 * a + PH[1]).sin();
                let t = ((f as f64 - lag) / 3.0).clamp(0.0, 1.0);
                let s = ((t - 0.3) / 0.7).clamp(0.0, 1.0);
                let tilt = 0.157 * s * s * (3.0 - 2.0 * s);
                let y = 0.011 * t - 0.033 * a.cos() * tilt.sin() + 0.0012 * (3.0 * a + PH[3]).sin() * (PI * t).sin();
                let v = (y / 0.0162).clamp(0.0, 1.0);
                let (x, yy) = (u * w, (1.0 - v) * h + (rnd() - 0.5) * 2.0);
                if i == 0 {
                    ctx.move_to(x, yy);
                } else {
                    ctx.line_to(x, yy);
                }
            }
            ctx.stroke();
        }
        // scratched to the aluminium, scraped along the floor
        ctx.set_line_width(1.0);
        for _ in 0..110 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = (rnd() - 0.5) * 0.6;
            let l = 4.0 + rnd() * 50.0;
            ctx.set_stroke_style(format!("rgba(196,198,200,{:.2})", 0.25 + rnd() * 0.5));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

// R528 rusted hammer: 8% chance per level. Metal head + wood
// handle on floor.
fn place_rusted_hammers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let rust_mat = lambert!({ color: 0x6a4028 });
    let handle_mat = lambert!({ color: 0x8a5030 });
    let hammer = group();
    // Handle.
    let handle = mesh(pg("RustedHammers__handle", cyl_geo!(0.008, 0.010, 0.20, 6)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0, 0.010, 0);
    hammer.add(&handle);
    // Head (rectangular block).
    let head = mesh(pg("RustedHammers__head", box_geo!(0.030, 0.020, 0.028)), &rust_mat);
    head.position().set(0.10, 0.014, 0);
    hammer.add(&head);
    // Claw prongs (2 curved forward extensions).
    for i in 0..2 {
        let claw = mesh(pg("RustedHammers__claw", box_geo!(0.008, 0.020, 0.008)), &rust_mat);
        // on the back of the head (+z, opposite the face), prongs pointing out
        // and curling back toward the handle; split vertically (hammer on its cheek)
        claw.position().set(0.10, 0.014 + (if i == 0 { -1.0 } else { 1.0 }) * 0.004, 0.022);
        claw.rotation().x = -PI / 2.0;
        claw.rotation().z = PI;
        hammer.add(&claw);
    }
    scatter(g, &hammer, x, z, 2.4);
    g.world.add(&hammer);
}

// R528 broken wall light switch: 6% chance per level. Switch
// plate w/ hanging switch bat + exposed wires.
fn place_broken_light_switches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let plate_mat = lambert!({ color: 0xe8e0c8 });
    let bat_mat = lambert!({ color: 0xf0ece0 });
    let wire_colors = [0xd82040, 0x2050d8, 0xd8b040];
    let sw = group();
    // Plate.
    let plate = mesh(pg("BrokenLightSwitches__plate", box_geo!(0.060, 0.10, 0.006)), &plate_mat);
    plate.position().set(0, 0, -0.003);
    sw.add(&plate);
    // Switch bat (tilted hanging - broken).
    let bat = mesh(pg("BrokenLightSwitches__toggle", box_geo!(0.014, 0.028, 0.010)), &bat_mat);
    bat.position().set(0, 0.010, 0.005);
    bat.rotation().z = 0.6; // hanging tilted
    sw.add(&bat);
    // 2 exposed wires poking out from side.
    for i in 0..2 {
        let fi = i as f64;
        let wire_mat = lambert!({ color: choice(&wire_colors) });
        let wire = mesh(pg("BrokenLightSwitches__wire", cyl_geo!(0.001, 0.001, 0.05, 3)), &wire_mat);
        wire.position().set(-0.030, -0.020 + fi * 0.008, 0.005);
        wire.rotation().z = -0.5 + fi * 0.3;
        sw.add(&wire);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.003);
    mount(&sw, c, wall_x, wall_z, 1.30, 0.6);
    g.world.add(&sw);
}

// R527 overturned wastebasket: 6% chance per level. Tipped
// metal bin + 4-7 pieces of scattered trash.
fn place_overturned_wastebaskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let bin_colors = [0x404040, 0x8a5030, 0x304060, 0x202020];
    let bin_mat = lambert!({ color: choice(&bin_colors) });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let can_colors = [0xa02040, 0x2050c0, 0x30a030];
    let grp = group();
    // Bin body (tipped on side).
    let bin = mesh(pg("OverturnedWastebaskets__bin", cyl_geo!(0.11, 0.09, 0.25, 12, 1, true)), &bin_mat);
    bin.rotation().z = -PI / 2.0; // wide mouth (top, r 0.11) at +X toward the trash, narrow base at −X
    bin.position().set(-0.05, 0.11, 0);
    grp.add(&bin);
    // Bin bottom.
    let bin_bottom = mesh(pg("OverturnedWastebaskets__base", circle_geo!(0.09, 12)), &bin_mat);
    bin_bottom.rotation().y = -PI / 2.0;
    bin_bottom.position().set(-0.175, 0.11, 0);
    grp.add(&bin_bottom);
    // Trash pieces (crumpled paper + can).
    let n_trash = 4 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_trash {
        let is_paper = rnd() < 0.7;
        let piece;
        if is_paper {
            piece = mesh(pg("OverturnedWastebaskets__paper", sphere_geo!(0.014 + rnd() * 0.010, 6, 5)), &paper_mat);
            piece.scale().set(1, 0.7, 1.2);
        } else {
            let can_mat = lambert!({ color: choice(&can_colors) });
            piece = mesh(pg("CrushedSodaCans__can", cyl_geo!(0.014, 0.014, 0.030, 8)), &can_mat);
            piece.rotation().z = PI / 2.0;
        }
        let px = 0.05 + rnd() * 0.20;
        let py = if is_paper { 0.015 } else { 0.014 };
        let pz = (rnd() - 0.5) * 0.24;
        piece.position().set(px, py, pz);
        piece.rotation().y = rnd() * PI * 2.0;
        grp.add(&piece);
    }
    scatter(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

// R527 elevator call button: 4% chance per level. Small panel
// with up/down arrows + LED.
fn place_elevator_buttons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let panel_mat = lambert!({ color: 0x808088 });
    let button_mat = lambert!({ color: 0x181410 });
    let led_mat = basic_mat!({ color: 0xd82040 });
    let panel = group();
    // Panel plate.
    let plate = mesh(pg("ElevatorButtons__plate", box_geo!(0.06, 0.12, 0.012)), &panel_mat);
    plate.position().set(0, 0, -0.006);
    panel.add(&plate);
    // 2 buttons (up + down triangles).
    for (by, ay) in [(0.025, PI), (-0.025, 0.0)] {
        let btn = mesh(pg("ElevatorButtons__button", cyl_geo!(0.016, 0.016, 0.005, 8)), &button_mat);
        btn.rotation().x = PI / 2.0;
        btn.position().set(0, by, 0.002);
        panel.add(&btn);
        // Triangle glyph.
        let glyph_geo = Geo::new();
        let gs: f64 = 0.008;
        let verts: Vec<f32> = vec![0.0, gs as f32, 0.0, -gs as f32, -gs as f32, 0.0, gs as f32, -gs as f32, 0.0];
        glyph_geo.set_position(verts);
        glyph_geo.set_index(vec![0, 1, 2]);
        glyph_geo.compute_vertex_normals();
        let glyph = mesh(glyph_geo, basic_mat!({ color: 0xe0e0e0 }));
        glyph.position().set(0, by, 0.005);
        glyph.rotation().z = ay;
        panel.add(&glyph);
    }
    // Small red LED.
    let led = mesh(sphere_geo!(0.003, 5, 4), &led_mat);
    led.position().set(0.020, 0, 0.004);
    panel.add(&led);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&panel, c, wall_x, wall_z, 1.30, 0.6);
    g.world.add(&panel);
}

// R526 mold patches: 6% chance per level. Green fuzzy patches
// wall decal.
game::lazy_static_tex!(
    mold_texture,
    canvas_tex(256, 320, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // 8-12 blobs of moldy discoloration.
        let n_blobs = 8 + (rnd() * 5.0).floor() as i32;
        for _ in 0..n_blobs {
            let cx = rnd() * w;
            let cy = rnd() * h;
            let r = 20.0 + rnd() * 30.0;
            // Layered blobs (dark to light).
            for layer in 0..3 {
                let lf = layer as f64;
                let alpha = 0.45 - lf * 0.10;
                let shade = if layer == 0 {
                    "#2a3818"
                } else if layer == 1 {
                    "#405a20"
                } else {
                    "#607040"
                };
                let hx = |a: usize, b: usize| u8::from_str_radix(&shade[a..b], 16).unwrap();
                ctx.set_fill_style(&format!("rgba({},{},{},{})", hx(1, 3), hx(3, 5), hx(5, 7), alpha));
                ctx.begin_path();
                ctx.ellipse(cx, cy, r * (1.0 - lf * 0.2), r * (0.7 - lf * 0.15), rnd() * PI, 0, PI * 2.0);
                ctx.fill();
            }
        }
        // Speckles.
        for _ in 0..60 {
            ctx.set_fill_style(&format!("rgba(60,80,30,{})", 0.4 + rnd() * 0.3));
            ctx.begin_path();
            let (ax, ay, ar) = (rnd() * w, rnd() * h, 1.0 + rnd() * 2.0);
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
    })
);

fn place_mold_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let mat = basic_mat!({ map: mold_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(0.60, 0.80), &mat);
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    let center_y = 1.2 + rnd() * 0.6;
    mount(&decal, c, wall_x, wall_z, center_y, 0.5);
    g.world.add(&decal);
}

// R526 folded umbrella: 8% chance per level. Compact folded
// umbrella w/ handle and strap.
fn place_folded_umbrellas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let cloth_colors = [0x181410, 0x2050a0, 0xa02040, 0x304050, 0x603820];
    let cloth_mat = lambert!({ color: choice(&cloth_colors) });
    let handle_mat = lambert!({ color: 0x2a1810 });
    let strap_mat = lambert!({ color: 0x8a7050 });
    let umb = group();
    // Main body (folded cylinder).
    let body = mesh(pg("FoldedUmbrellas__body", cyl_geo!(0.020, 0.020, 0.32, 8)), &cloth_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.020, 0);
    umb.add(&body);
    // Ridged folds (small dark stripes).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let stripe = mesh(pg("FoldedUmbrellas__seam", cyl_geo!(0.021, 0.021, 0.32, 3)), &handle_mat);
        stripe.rotation().z = PI / 2.0;
        stripe.rotation().x = a;
        stripe.scale().set(1, 1, 0.03);
        stripe.position().set(0, 0.020, 0);
        umb.add(&stripe);
    }
    // Tip cap (pointy).
    let tip = mesh(pg("FoldedUmbrellas__tip", cone_geo!(0.014, 0.020, 6)), &cloth_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(0.170, 0.020, 0);
    umb.add(&tip);
    // Curved handle.
    let handle = mesh(pg("FoldedUmbrellas__handle", torus_geo!(0.020, 0.005, 4, 8, PI)), &handle_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().set(-0.175, 0.020, 0);
    umb.add(&handle);
    // Strap loop.
    let strap = mesh(torus_geo!(0.008, 0.001, 3, 6), &strap_mat);
    // A band round the furled canopy (YZ plane, R ~0.021), not buried inside it.
    strap.rotation().y = PI / 2.0;
    strap.scale().set_scalar(2.6);
    strap.position().set(0, 0.020, 0);
    umb.add(&strap);
    scatter(g, &umb, x, z, 2.0);
    g.world.add(&umb);
}

// R523 single worn boot: 8% chance per level. Work boot w/
// sole, upper, and laces.
fn place_single_boots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let boot_colors = [0x201a10, 0x503020, 0x6a3820, 0x2a1810];
    let boot_mat = lambert!({ color: choice(&boot_colors) });
    let sole_mat = lambert!({ color: 0x100804 });
    let lace_mat = lambert!({ color: 0xf0ece0 });
    let boot = group();
    // Sole.
    let sole = mesh(pg("SingleBoots__sole", box_geo!(0.13, 0.014, 0.055)), &sole_mat);
    sole.position().y = 0.007;
    boot.add(&sole);
    // Toe upper (rounded).
    let toe = mesh(pg("SingleBoots__toe", sphere_geo!(0.035, 8, 6)), &boot_mat);
    toe.scale().set(1, 0.6, 0.9);
    toe.position().set(0.025, 0.028, 0);
    boot.add(&toe);
    // Heel upper (taller).
    let heel = mesh(pg("SingleBoots__heel", box_geo!(0.045, 0.050, 0.055)), &boot_mat);
    heel.position().set(-0.030, 0.040, 0);
    boot.add(&heel);
    // Ankle rise.
    let ankle = mesh(pg("SingleBoots__shaft", cyl_geo!(0.028, 0.030, 0.06, 8)), &boot_mat);
    ankle.position().set(-0.030, 0.090, 0);
    boot.add(&ankle);
    // Laces (3 crossed X patterns).
    for i in 0..3 {
        for j in 0..2 {
            let lace = mesh(box_geo!(0.028, 0.001, 0.002), &lace_mat);
            // On the shaft's front (+x) face, crossing in its plane (was on the axis, hidden).
            lace.scale().x = 0.7;
            lace.position().set(0.0004, 0.070 + i as f64 * 0.008, 0);
            lace.rotation().set(if j == 0 { 0.4 } else { -0.4 }, PI / 2.0, 0);
            boot.add(&lace);
        }
    }
    // 25% tipped.
    if rnd() < 0.25 {
        // Fallen onto its side: roll about the boot's length axis, then the yaw below.
        boot.set_rotation_order(EulerOrder::YXZ);
        boot.rotation().x = PI / 2.0;
        boot.position().y = 0.032;
    }
    scatter_xz(g, &boot, x, z, 2.4);
    g.world.add(&boot);
}

// R522 overturned wooden chair: 5% chance per level. Basic
// 4-leg chair silhouette on side.
fn place_overturned_chairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let wood_colors = [0x603820, 0x8a5030, 0x201a10, 0xa07050];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let chair = group();
    // Seat.
    let seat = mesh(pg("OverturnedChairs__seat", box_geo!(0.34, 0.024, 0.30)), &wood_mat);
    seat.position().y = 0.40;
    chair.add(&seat);
    // 4 legs.
    for (lx, lz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let leg = mesh(pg("OverturnedChairs__leg", cyl_geo!(0.012, 0.012, 0.40, 5)), &wood_mat);
        leg.position().set(lx * 0.14, 0.20, lz * 0.12);
        chair.add(&leg);
    }
    // Back rest bars.
    for i in 0..4 {
        let bar = mesh(pg("OverturnedChairs__spindle", cyl_geo!(0.008, 0.008, 0.28, 4)), &wood_mat);
        bar.position().set(-0.14 + i as f64 * 0.093, 0.56, -0.12);
        chair.add(&bar);
    }
    // Top rail.
    let rail = mesh(pg("OverturnedChairs__rail", box_geo!(0.30, 0.024, 0.020)), &wood_mat);
    rail.position().set(0, 0.70, -0.12);
    chair.add(&rail);
    // Tip on side: local x becomes height, so lift by the seat's half-width
    // (0.17) to rest its edge on the floor instead of half under it.
    chair.rotation().z = PI / 2.0;
    scatter_y(g, &chair, x, z, 1.6, 0.17);
    g.world.add(&chair);
}

// R522 torn photograph: 8% chance per level. Half-photo w/
// jagged tear edge (canvas texture).
game::lazy_static_tex!(
    torn_photo_texture,
    canvas_tex_opt(128, 160, false, |ctx, w, h| {
        // Sepia photo.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 20, w / 2.0, h / 2.0, 100);
        grad.add_color_stop(0, "#c0a080");
        grad.add_color_stop(1, "#503020");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Silhouette figure.
        ctx.set_fill_style("#201810");
        ctx.begin_path();
        ctx.arc(w / 2.0, h * 0.4, 20, 0, PI * 2.0);
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(w / 2.0 - 25.0, h);
        ctx.line_to(w / 2.0 + 25.0, h);
        ctx.line_to(w / 2.0 + 15.0, h * 0.55);
        ctx.line_to(w / 2.0 - 15.0, h * 0.55);
        ctx.close_path();
        ctx.fill();
        // Tear edge (right side): make right portion transparent.
        ctx.set_global_composite_operation("destination-out");
        ctx.set_fill_style("#000");
        ctx.begin_path();
        ctx.move_to(w * 0.6, 0);
        // Jagged tear.
        let mut y = 0.0;
        while y < h {
            ctx.line_to(w * (0.55 + rnd() * 0.10), y);
            y += 8.0;
        }
        ctx.line_to(w, h);
        ctx.line_to(w, 0);
        ctx.close_path();
        ctx.fill();
        ctx.set_global_composite_operation("source-over");
    })
);

fn place_torn_photographs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let photo_mat = lambert!({ map: torn_photo_texture(), side: DoubleSide, transparent: true, alphaTest: 0.1 });
    let photo = mesh(plane_geo!(0.10, 0.13), &photo_mat);
    photo.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    photo.position().set(px, 0.003, pz);
    photo.rotation().z = rnd() * PI * 2.0;
    g.world.add(&photo);
}

// ------------------------------------------------------------ batch r12
// The parts of Blender batch `r12` (assets/src/blender/props/r12.py) and the
// pull station of `r06` are placed as modelled, each prop's parts sharing one
// frame: x across its front, y up, z out of the wall, the wall at z = 0.

/// A Blender part as modelled (a scale of one), or a box standing in for it
/// when the props are switched off.
fn pay_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}
fn pull_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    pay_part(name, w, h, d)
}

/// The payphone's depth and its card's window (x, y, w, h) (`r12.py`, PP_*).
const PP_D: f64 = 0.135;
const PP_CARD: (f64, f64, f64, f64) = (-0.026, 0.474, 0.112, 0.074);

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! r12_tex {
    ($w:expr, $h:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, false, $paint)).clone())
    }};
}

/// A soft round blob (squashed by the context, never the ellipse: no rim).
fn pay_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, rgba: (u8, u8, u8, f64)) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({},{},{},{})", rgba.0, rgba.1, rgba.2, rgba.3));
    gr.add_color_stop(1, &format!("rgba({},{},{},0)", rgba.0, rgba.1, rgba.2));
    ctx.set_fill_style(gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Short straight scratches.
fn pay_scratches(ctx: &mut Canvas, w: f64, h: f64, n: usize, rgb: (u8, u8, u8), a0: f64, len: f64) {
    ctx.set_line_cap("round");
    for _ in 0..n {
        let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, len * (0.2 + rnd()));
        ctx.set_line_width(0.5 + rnd() * 0.7);
        ctx.set_stroke_style(&format!("rgba({},{},{},{})", rgb.0, rgb.1, rgb.2, a0 + rnd() * a0));
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
}

/// The upper housing's stainless (the canvas is its face, 0.204 x 0.294 m):
/// the grain down it, keyed scratches, the ghosts of stickers peeled off,
/// grime round the keys and the coin return, COIN RETURN stamped over it.
fn pay_steel_tex() -> Tex {
    r12_tex!(204, 294, |ctx, w, h| {
        ctx.set_fill_style("#e8e8e4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..800 {
            ctx.set_fill_style(&format!("rgba({},{},{},{})", if rnd() < 0.5 { 255 } else { 80 }, 250, 245, 0.05 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 0.6, 5.0 + rnd() * 30.0);
        }
        // metres on the face to the canvas
        let to = |x: f64, y: f64| ((x + 0.102) / 0.204 * w, (0.530 - y) / 0.294 * h);
        // sticker ghosts: glue gone grey with dirt, a torn corner left
        for &(x, y, sw, sh) in &[(0.040, 0.300, 34.0, 22.0), (-0.085, 0.300, 26.0, 40.0)] {
            let (px, py) = to(x, y);
            ctx.set_fill_style("rgba(120,112,96,0.30)");
            ctx.fill_rect(px, py, sw, sh);
            ctx.set_fill_style("rgba(220,210,180,0.75)");
            ctx.fill_rect(px, py, 6, 5);
        }
        // grime where hands go
        let (kx, ky) = to(0.0, 0.357);
        pay_blob(ctx, kx, ky, 50.0, 1.3, (60, 54, 46, 0.30));
        let (rx, ry) = to(0.060, 0.266);
        pay_blob(ctx, rx, ry, 26.0, 1.0, (50, 44, 36, 0.35));
        let (cx, cy) = to(0.064, 0.480);
        pay_blob(ctx, cx, cy, 20.0, 1.0, (50, 44, 36, 0.25));
        // stamped labels
        ctx.set_fill_style("rgba(40,40,38,0.75)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 7px Helvetica");
        let (lx, ly) = to(0.060, 0.290);
        ctx.fill_text("COIN RETURN", lx, ly);
        let (dx, dy) = to(0.064, 0.448);
        ctx.fill_text("25¢ 10¢ 5¢", dx, dy);
        // keyed scratches: long, wandering, through the grain
        ctx.set_line_cap("round");
        for _ in 0..7 {
            let (mut x, mut y) = (rnd() * w, rnd() * h);
            ctx.set_line_width(0.8);
            ctx.set_stroke_style(&format!("rgba(70,68,64,{})", 0.25 + rnd() * 0.25));
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..6 {
                x += (rnd() - 0.5) * 30.0;
                y += (rnd() - 0.5) * 18.0;
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        pay_scratches(ctx, w, h, 40, (60, 58, 54), 0.12, 16.0);
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(130,120,100,0.3)");
        gd.add_color_stop(0.06, "rgba(130,120,100,0)");
        gd.add_color_stop(0.9, "rgba(50,44,36,0)");
        gd.add_color_stop(1, "rgba(50,44,36,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// The vault's black enamel: chipped to grey primer and the steel at its
/// edges and round the lock, rust weeping from the chips, kicked at the foot.
fn pay_vault_tex() -> Tex {
    r12_tex!(160, 190, |ctx, w, h| {
        ctx.set_fill_style("#f0f0ec");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            ctx.set_fill_style(&format!("rgba(60,60,58,{})", 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        // chips: pale primer and bare steel, at the edges and the lock
        for k in 0..30 {
            let (x, y) = if k < 8 {
                (w * 0.5 + (rnd() - 0.5) * 30.0, h * 0.2 + (rnd() - 0.5) * 30.0)
            } else if k % 2 == 0 {
                (rnd() * w, if rnd() < 0.5 { 3.0 } else { h - 3.0 })
            } else {
                (if rnd() < 0.5 { 3.0 } else { w - 3.0 }, rnd() * h)
            };
            let r = 1.5 + rnd() * 4.0;
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(255,255,255,0.9)" } else { "rgba(190,150,120,0.9)" });
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.4 + rnd() * 0.6), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // scuffed from kicks at the foot
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(255,255,255,0.0)");
        gd.add_color_stop(0.7, "rgba(255,255,255,0.0)");
        gd.add_color_stop(1, "rgba(255,255,255,0.35)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 40, (255, 255, 255), 0.25, 18.0);
    })
}

/// The instruction card (0.112 x 0.074 m): how to call, what it costs, the
/// emergency and the operator, this phone's number (555-01xx, kept for
/// fiction), sun-faded and foxed.
fn pay_card_tex() -> Tex {
    r12_tex!(336, 222, |ctx, w, h| {
        ctx.set_fill_style("#e3dfd2");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#2c4a6a");
        ctx.fill_rect(0, 0, w, 34);
        ctx.set_fill_style("#efebe0");
        ctx.set_text_align("left");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 19px Helvetica");
        ctx.fill_text("LOCAL CALLS 35¢", 12, 18);
        ctx.set_fill_style("#22201c");
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text("1  LIFT RECEIVER", 12, 52);
        ctx.fill_text("2  DEPOSIT COINS — LISTEN FOR DIAL TONE", 12, 70);
        ctx.fill_text("3  DIAL NUMBER", 12, 88);
        ctx.set_font("11px Helvetica");
        ctx.fill_text("COINS RETURNED IF CALL IS NOT COMPLETED", 12, 108);
        ctx.fill_text("LONG DISTANCE: DIAL 1 + AREA CODE + NUMBER", 12, 124);
        ctx.set_fill_style("#8a2a1e");
        ctx.fill_rect(10, 136, w - 20.0, 30);
        ctx.set_fill_style("#efebe0");
        ctx.set_font("bold 15px Helvetica");
        ctx.fill_text("EMERGENCY  911  NO COINS NEEDED", 18, 151);
        ctx.set_fill_style("#22201c");
        ctx.set_font("12px Helvetica");
        ctx.fill_text("OPERATOR  0          REPAIR  611", 12, 182);
        ctx.set_font("bold 12px Helvetica");
        ctx.fill_text("THIS PHONE  (555) 555-0182", 12, 204);
        // faded from the top, foxed, a water stain from the corner
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(200,190,160,0.35)");
        gy.add_color_stop(1, "rgba(120,100,60,0.18)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..24 {
            pay_blob(ctx, rnd() * w, rnd() * h, 3.0 + rnd() * 7.0, 1.0, (130, 90, 40, 0.08 + rnd() * 0.12));
        }
        pay_blob(ctx, w, h, 80.0, 1.0, (120, 90, 50, 0.28));
    })
}

/// What is printed on the keys, black in their chrome: the numeral and its
/// letters (the canvas spans the keypad, 0.075 x 0.100 m, 3 px a mm).
fn pay_keys_tex() -> Tex {
    r12_tex!(225, 300, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let keys = [
            ("1", ""),
            ("2", "ABC"),
            ("3", "DEF"),
            ("4", "GHI"),
            ("5", "JKL"),
            ("6", "MNO"),
            ("7", "PQRS"),
            ("8", "TUV"),
            ("9", "WXYZ"),
            ("*", ""),
            ("0", "OPER"),
            ("#", ""),
        ];
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        for (i, (num, letters)) in keys.iter().enumerate() {
            let (col, row) = ((i % 3) as f64, (i / 3) as f64);
            let cx = (0.0375 - 0.026 + col * 0.026) * 3000.0;
            let cy = (0.050 - (0.396 - 0.357) + row * 0.026) * 3000.0;
            ctx.set_fill_style(&format!("rgba(26,25,24,{})", 0.92 - rnd() * 0.4));
            if !letters.is_empty() {
                ctx.set_font("bold 8px Helvetica");
                ctx.fill_text(letters, cx, cy - 6.0);
            }
            ctx.set_font("bold 19px Helvetica");
            ctx.fill_text(num, cx, cy + if letters.is_empty() { 8.0 } else { 14.0 });
        }
    })
}

/// The card's clear window: scratched and clouded, a crack from a corner.
fn pay_window_tex() -> Tex {
    r12_tex!(168, 111, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(200,195,180,0.18)");
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 50, (235, 235, 230), 0.18, 26.0);
        ctx.set_stroke_style("rgba(240,240,235,0.6)");
        ctx.set_line_width(0.8);
        ctx.begin_path();
        ctx.move_to(w, 0);
        ctx.line_to(w * 0.82, h * 0.18);
        ctx.line_to(w * 0.74, h * 0.22);
        ctx.line_to(w * 0.6, h * 0.41);
        ctx.stroke();
        for &(x, y) in &[(0.0, 0.0), (w, h), (0.0, h)] {
            pay_blob(ctx, x, y, 30.0, 1.0, (150, 140, 120, 0.3));
        }
    })
}

/// The pull station's cover: red enamel over die-cast, rubbed to the metal at
/// its edges and round the lever's well where hands go, grimy, a few chips
/// (as batch `r06`'s fire alarms; laid on its front by the part's box
/// projection: the whole texture is the face).
fn pull_paint_tex() -> Tex {
    r12_tex!(128, 170, |ctx, w, h| {
        ctx.set_fill_style("#8a1c14");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            ctx.set_fill_style(&format!("rgba({},0,0,{})", if rnd() < 0.5 { 40 } else { 200 }, 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
        }
        ctx.set_stroke_style("rgba(150,120,110,0.35)");
        ctx.set_line_width(3);
        ctx.stroke_rect(1.5, 1.5, w - 3.0, h - 3.0);
        for &(x, y) in &[(3.0, 3.0), (w - 4.0, 3.0), (3.0, h - 4.0), (w - 4.0, h - 4.0), (w * 0.5, h - 3.0), (3.0, h * 0.4)] {
            ctx.set_fill_style("rgba(120,116,110,0.85)");
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 4.0, 1.5 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        pay_blob(ctx, w * 0.5, h * 0.55, w * 0.6, 1.0, (30, 20, 16, 0.30));
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(160,150,130,0.22)");
        gd.add_color_stop(0.08, "rgba(160,150,130,0)");
        gd.add_color_stop(0.8, "rgba(40,30,24,0)");
        gd.add_color_stop(1, "rgba(40,30,24,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 14, (170, 140, 130), 0.2, 16.0);
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
