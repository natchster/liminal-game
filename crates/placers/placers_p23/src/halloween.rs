//! p23, second half: ghost / cat / spider / broom / tombstone / bat /
//! skeleton hand / cauldron / pumpkin / witch hat / crystal ball / toy sword /
//! toy piano.
use crate::*;
use game::prelude::*;

// the ghost, the cat, the broom, the bat, the skeleton hand, the cauldron
// and the pumpkin: batch `t08`'s (`halloween/t08.rs`)
mod t08;
pub use t08::{
    place_hanging_bats, place_small_black_cats, place_small_brooms, place_small_cauldrons, place_small_ghost_sheets, place_small_pumpkins,
    place_small_skeleton_hands,
};

// the crystal ball: batch `t18`'s (`halloween/t18.rs`)
mod t18;
pub use t18::place_small_crystal_balls;

// Small spider: 5% chance per level. Dark body + 8 legs.
// Halloween family.
pub fn place_small_spiders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let spider_mat = lambert!({ color: 0x100808 });
    let eye_mat = basic_mat!({ color: 0xd82040 });
    let spider = group();
    let body_r = 0.014;
    // Body (abdomen sphere).
    let body = mesh(pg("SmallSpiders__body", sphere_geo!(body_r, 8, 6)), &spider_mat);
    body.position().y = body_r;
    spider.add(&body);
    // Head (smaller sphere).
    let head = mesh(pg("SmallSpiders__head", sphere_geo!(body_r * 0.7, 6, 5)), &spider_mat);
    head.position().set(body_r * 1.2, body_r, 0);
    spider.add(&head);
    // 8 legs (bent lines using 2 cylinders each).
    // Each leg: hip under the body/head joint, the first segment rising 20°
    // outward to a knee, the second dropping 63° to a foot on the floor.
    // Four legs a side (±Z; the head is +X), fanned from forward to back.
    let leg_span = |m: &Obj, p: Vec3, q: Vec3| {
        let mut mid = p;
        mid.add(&q).multiply_scalar(0.5);
        m.position().copy(&mid);
        let mut d = q;
        d.sub(&p).normalize();
        m.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
    };
    for i in 0..8 {
        let side = if i < 4 { -1.0 } else { 1.0 };
        let leg_i = i % 4;
        let a = -PI / 3.0 + (leg_i as f64 / 3.0) * (PI * 2.0 / 3.0);
        let h = v3(-a.sin(), 0, side * a.cos());
        let hip = v3(0.010 + h.x * 0.008, 0.006, h.z * 0.008);
        let mut knee = hip;
        knee.add_scaled_vector(&h, 0.022 * 0.35f64.cos()).set_y(0.006 + 0.022 * 0.35f64.sin());
        let mut foot = knee;
        foot.add_scaled_vector(&h, (0.014f64.powi(2) - (knee.y - 0.001).powi(2)).sqrt()).set_y(0.001);
        // First segment (out from body).
        let seg1 = mesh(pg("SmallSpiders__leg", cyl_geo!(0.001, 0.001, 0.022, 3)), &spider_mat);
        leg_span(&seg1, hip, knee);
        spider.add(&seg1);
        // Second segment (down to floor).
        let seg2 = mesh(pg("SmallSpiders__leg", cyl_geo!(0.001, 0.001, 0.014, 3)), &spider_mat);
        leg_span(&seg2, knee, foot);
        spider.add(&seg2);
    }
    // 2 tiny red eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.0015, 4, 3), &eye_mat);
        eye.position().set(body_r * 1.4, body_r + 0.004, s * 0.004);
        spider.add(&eye);
    }
    spider.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    spider.rotation().y = rnd() * PI * 2.0;
    g.world.add(&spider);
}

// Small tombstone: 6% chance per level. Gray stone slab
// with rounded top + engraved cross. Halloween family.
pub fn place_small_tombstones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let stone_colors: [u32; 4] = [0x808080, 0x707070, 0x606060, 0x585858];
    let stone_mat = lambert!({ color: pick_u32(&stone_colors) });
    let engrave_mat = lambert!({ color: 0x2a2418 });
    let stone = group();
    // Slab body (rectangular).
    let body = mesh(pg("SmallTombstones__body", box_geo!(0.12, 0.16, 0.024)), &stone_mat);
    body.position().y = 0.084;
    stone.add(&body);
    // Rounded top (half cylinder).
    let top = mesh(pg("SmallTombstones__top", cyl_geo!(0.060, 0.060, 0.024, 12, 1, false, 0, PI)), &stone_mat);
    // Turn the half-disc (local x >= 0) about its own axis first, so the
    // arch spans the slab's full width across the top instead of a D on its
    // right half.
    top.rotation().y = PI / 2.0;
    top.rotation().x = PI / 2.0;
    top.position().set(0, 0.164, 0);
    stone.add(&top);
    // Base slab.
    let base = mesh(pg("SmallTombstones__base", box_geo!(0.14, 0.014, 0.040)), &stone_mat);
    base.position().y = 0.007;
    stone.add(&base);
    // Engraved cross on front (2 bars).
    let v_bar = mesh(pg("SmallTombstones__vBar", box_geo!(0.010, 0.070, 0.003)), &engrave_mat);
    v_bar.position().set(0, 0.120, 0.013);
    stone.add(&v_bar);
    let h_bar = mesh(pg("SmallTombstones__hBar", box_geo!(0.035, 0.010, 0.003)), &engrave_mat);
    h_bar.position().set(0, 0.130, 0.013);
    stone.add(&h_bar);
    // Slight tilt for aged look.
    stone.rotation().z = (rnd() - 0.5) * 0.15;
    stone.rotation().x = (rnd() - 0.5) * 0.10;
    stone.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    stone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stone);
}

// Small witch hat: 8% chance per level. Cone with brim on
// floor. Halloween family.
pub fn place_small_witch_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let hat_colors: [u32; 3] = [0x101010, 0x201820, 0x8a2040];
    let hat_mat = lambert!({ color: pick_u32(&hat_colors) });
    let band_mat = lambert!({ color: 0xd8b040 });
    let buckle_mat = lambert!({ color: 0xd8b040 });
    let hat = group();
    // Brim: flat disc.
    let brim = mesh(pg("SmallWitchHats__brim", cyl_geo!(0.080, 0.080, 0.006, 16)), &hat_mat);
    brim.position().y = 0.003;
    hat.add(&brim);
    // Cone (tall).
    let cone = mesh(pg("SmallWitchHats__cone", cone_geo!(0.045, 0.14, 12, 1, true)), &hat_mat);
    cone.position().y = 0.076;
    hat.add(&cone);
    // Slight bent tip: nothing needed (bendable via slight rotation).
    cone.rotation().z = (rnd() - 0.5) * 0.2;
    // Tilt about the crown's base (centred on the brim, y 0.006), not its middle.
    let rz = cone.rot().z;
    cone.position().x = -rz.sin() * 0.07;
    cone.position().y = 0.006 + rz.cos() * 0.07;
    // Band around cone base.
    let band = mesh(pg("SmallWitchHats__band", cyl_geo!(0.045, 0.045, 0.010, 12)), &band_mat);
    band.position().y = 0.014;
    hat.add(&band);
    // Buckle.
    let buckle = mesh(pg("SmallWitchHats__buckle", box_geo!(0.014, 0.010, 0.001)), &buckle_mat);
    buckle.position().set(0, 0.014, 0.046);
    hat.add(&buckle);
    // 25% chance tipped/knocked over.
    if rnd() < 0.25 {
        hat.rotation().z = PI / 2.0;
        hat.position().y = 0.04;
    }
    hat.position().x = g.wx(x) + (rnd() - 0.5) * 2.0;
    hat.position().z = g.wz(z) + (rnd() - 0.5) * 2.0;
    hat.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hat);
}

// Small toy sword: 8% chance per level. Kid's foam/plastic
// sword with blade + guard + hilt. Party-costume family.
pub fn place_small_toy_swords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let blade_colors: [u32; 3] = [0xc0c8d0, 0xd8b040, 0xa8b0b8];
    let blade_mat = lambert!({ color: pick_u32(&blade_colors) });
    let grip_colors: [u32; 4] = [0xd82040, 0x2050d8, 0x8a5030, 0x30a030];
    let grip_mat = lambert!({ color: pick_u32(&grip_colors) });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let sword = group();
    // Blade: long thin flattened box.
    let blade_len = 0.28;
    let blade = mesh(pg("SmallToySwords__blade", box_geo!(blade_len, 0.020, 0.005)), &blade_mat);
    blade.position().set(blade_len / 2.0 + 0.030, 0.010, 0);
    sword.add(&blade);
    // Blade tip point.
    let tip = mesh(pg("SmallToySwords__tip", cone_geo!(0.010, 0.020, 4)), &blade_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(blade_len + 0.040, 0.010, 0);
    sword.add(&tip);
    // Cross guard (horizontal bar).
    let guard = mesh(pg("SmallToySwords__guard", box_geo!(0.008, 0.036, 0.010)), &gold_mat);
    guard.position().set(0.030, 0.010, 0);
    sword.add(&guard);
    // Grip.
    let grip = mesh(pg("SmallToySwords__grip", cyl_geo!(0.008, 0.008, 0.05, 8)), &grip_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(0.005, 0.010, 0);
    sword.add(&grip);
    // Pommel (gold ball at grip end).
    let pommel = mesh(pg("SmallToySwords__pommel", sphere_geo!(0.010, 8, 6)), &gold_mat);
    pommel.position().set(-0.020, 0.010, 0);
    sword.add(&pommel);
    sword.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    sword.rotation().y = rnd() * PI * 2.0;
    // Lie it flat (it stood on the blade's edge with the 3.6 cm guard through
    // the floor): roll 90° about the blade axis, then dip the point 0.02 rad so
    // it rests on the pommel and the blade end. 'YZX' = roll, dip, then yaw.
    sword.set_rotation_order(EulerOrder::YZX);
    sword.rotation().x = PI / 2.0;
    sword.rotation().z = -0.02;
    sword.position().y = 0.010;
    g.world.add(&sword);
}

// Small toy piano: 6% chance per level. Rectangular piano
// with 7 white + 5 black keys. Child-music family.
pub fn place_small_toy_pianos(g: &mut Game, _dist: &[i32]) {
    g.pp.piano_positions = vec![]; // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let body_colors: [u32; 5] = [0xd82040, 0x2050d8, 0xd8b040, 0x8a5030, 0x181008];
    let body_mat = lambert!({ color: pick_u32(&body_colors) });
    let white_key = lambert!({ color: 0xf0ece0 });
    let black_key = lambert!({ color: 0x181008 });
    let piano = group();
    let keyboard_w = 0.20;
    let keyboard_d = 0.06;
    // Body/case.
    let body = mesh(pg("SmallToyPianos__case", box_geo!(keyboard_w + 0.02, 0.024, keyboard_d + 0.02)), &body_mat);
    body.position().y = 0.012;
    piano.add(&body);
    // Angled back panel.
    let back = mesh(pg("SmallToyPianos__back", box_geo!(keyboard_w + 0.02, 0.05, 0.014)), &body_mat);
    back.position().set(0, 0.038, -(keyboard_d / 2.0 + 0.008));
    piano.add(&back);
    // 7 white keys.
    let n_white = 7;
    let w_key_w = keyboard_w / n_white as f64;
    let mut white_keys: Vec<Obj> = vec![];
    for i in 0..n_white {
        let key = mesh(pg("SmallToyPianos__whitekey", box_geo!(w_key_w - 0.0015, 0.008, keyboard_d * 0.9)), &white_key);
        key.position().set(-keyboard_w / 2.0 + w_key_w / 2.0 + i as f64 * w_key_w, 0.028, 0);
        piano.add(&key);
        white_keys.push(key);
    }
    // Phantom key press animator — random key sinks briefly
    // every 20-40 s. Deeply unsettling ("who pressed it?").
    let mut next_t = 20.0 + rnd() * 20.0;
    let mut active_key: i32 = -1;
    let mut active_until = 0.0;
    g.animate(move |_g, _dt, t| {
        if t >= next_t && active_key < 0 {
            active_key = (rnd() * white_keys.len() as f64).floor() as i32;
            active_until = t + 0.5;
            next_t = t + 20.0 + rnd() * 20.0;
        }
        if active_key >= 0 {
            let k = &white_keys[active_key as usize];
            let remaining = active_until - t;
            if remaining <= 0.0 {
                k.position().y = 0.028;
                active_key = -1;
            } else {
                // Sink 3mm then rise back.
                let frac = 1.0 - (remaining - 0.25).abs() / 0.25;
                k.position().y = 0.028 - 0.0f64.max(frac) * 0.003;
            }
        }
    });
    // 5 black keys interleaved (position between white keys 0-1, 1-2, 3-4, 4-5, 5-6).
    let black_positions = [0.5, 1.5, 3.5, 4.5, 5.5];
    for bp in black_positions {
        let bkey = mesh(pg("SmallToyPianos__blackkey", box_geo!(w_key_w * 0.6, 0.010, keyboard_d * 0.55)), &black_key);
        bkey.position().set(-keyboard_w / 2.0 + w_key_w / 2.0 + bp * w_key_w, 0.031, -keyboard_d / 2.0 + keyboard_d * 0.275 + 0.005);
        piano.add(&bkey);
    }
    piano.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    piano.rotation().y = rnd() * PI * 2.0;
    g.world.add(&piano);
    // Track for random chord audio.
    let pp = piano.pos();
    g.pp.piano_positions.push(v3(pp.x, 0.03, pp.z));
}
