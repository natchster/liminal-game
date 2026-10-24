//! p23, second half: ghost / cat / spider / broom / tombstone / bat /
//! skeleton hand / cauldron / pumpkin / witch hat / crystal ball / toy sword /
//! toy piano.
use crate::*;
use game::prelude::*;

// ------------------------------------------------------------------ line 42757
// R464 small ghost sheet: 5% chance per level. Draped cream
// sheet with 2 dark eye cutouts. Halloween family.
pub fn place_small_ghost_sheets(g: &mut Game, _dist: &[i32]) {
    g.pp.ghost_positions = vec![]; // R485 reset per level
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let sheet_mat = lambert!({ color: 0xf0ece0, side: DoubleSide });
    let eye_mat = basic_mat!({ color: 0x080604 });
    let ghost = group();
    // Draped body: rounded cone with wide draped skirt.
    let body_h = 0.24;
    let body = mesh(cone_geo!(0.070, body_h, 10, 4), &sheet_mat);
    body.position().y = body_h / 2.0;
    ghost.add(&body);
    // Head bulge: sphere on top.
    let head = mesh(sphere_geo!(0.050, 10, 8), &sheet_mat);
    head.position().y = body_h - 0.010;
    ghost.add(&head);
    // 2 dark eye holes (circles on front face).
    for s in [-1.0, 1.0] {
        let eye = mesh(circle_geo!(0.008, 5), &eye_mat);
        eye.position().set(s * 0.014, body_h - 0.005, 0.045);
        ghost.add(&eye);
    }
    // Rippled hem at bottom: 3 wave bumps at base.
    for i in 0..6 {
        let a = (i as f64 / 6.0) * PI * 2.0;
        let bump = mesh(sphere_geo!(0.020, 6, 4), &sheet_mat);
        bump.position().set(a.cos() * 0.055, 0, a.sin() * 0.055);
        bump.scale().set(1, 0.5, 1);
        ghost.add(&bump);
    }
    ghost.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    ghost.rotation().y = rnd() * PI * 2.0;
    // Very slow eerie sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let ghost2 = ghost.clone();
        g.animate(move |_g, _dt, t| {
            ghost2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.02;
        });
    }
    g.world.add(&ghost);
    // R485 track for whisper audio.
    let gp = ghost.pos();
    g.pp.ghost_positions.push(v3(gp.x, 0.20, gp.z));
    // R487 subtle white ghostly halo point light at head level.
    let spectral_light = point_light!(0xf0f0ff, 0.18, 0.9, 2);
    spectral_light.position().set(gp.x, 0.22, gp.z);
    g.world.add(&spectral_light);
    // Slow pulsing.
    let pulse_phase2 = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = (t * 0.6 + pulse_phase2).sin() * 0.5 + 0.5;
        spectral_light.set_intensity(0.10 + p * 0.15);
    });
}

// ------------------------------------------------------------------ line 42825
// R464 small black cat figurine: 6% chance per level. Dark cat
// silhouette (body + head + ears + tail). Halloween family.
pub fn place_small_black_cats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let cat_mat = lambert!({ color: 0x100808 });
    let eye_mat = basic_mat!({ color: 0xf0d020 });
    let cat = group();
    // Body: elongated ellipsoid (sitting).
    let body = mesh(pg("SmallBlackCats__body", sphere_geo!(0.030, 10, 8)), &cat_mat);
    body.scale().set(1, 1.5, 0.9);
    body.position().y = 0.045;
    cat.add(&body);
    // Head: smaller sphere.
    let head = mesh(pg("SmallBlackCats__head", sphere_geo!(0.024, 10, 8)), &cat_mat);
    head.position().y = 0.088;
    cat.add(&head);
    // 2 pointy ears (small triangles).
    for s in [-1.0, 1.0] {
        let ear = mesh(pg("SmallBlackCats__ear", cone_geo!(0.008, 0.014, 3)), &cat_mat);
        // Base set 2 mm into the head (r 0.024 at y 0.088), 0.45 rad off the
        // crown, leaning out 0.3 rad: the whole ear shows.
        ear.rotation().z = -s * 0.3;
        ear.position().set(s * 0.0117, 0.1144, 0);
        cat.add(&ear);
    }
    // 2 tiny yellow eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.003, 5, 4), &eye_mat);
        eye.position().set(s * 0.008, 0.092, 0.020);
        cat.add(&eye);
    }
    // Curling tail: torus arc.
    let tail = mesh(pg("SmallBlackCats__tail", torus_geo!(0.024, 0.006, 4, 12, PI * 1.3)), &cat_mat);
    tail.rotation().z = PI / 2.0;
    tail.rotation().y = PI / 4.0;
    tail.position().set(-0.028, 0.036, 0);
    cat.add(&tail);
    // Base/legs: small cylinder platform under sitting body.
    let legs = mesh(pg("SmallBlackCats__legs", cyl_geo!(0.020, 0.024, 0.020, 6)), &cat_mat);
    legs.position().y = 0.010;
    cat.add(&legs);
    cat.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cat.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cat);
}

// ------------------------------------------------------------------ line 42883
// R463 small spider: 5% chance per level. Dark body + 8 legs.
// Halloween family.
pub fn place_small_spiders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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

// ------------------------------------------------------------------ line 42947
// R463 small broom: 8% chance per level. Witch's broom with
// stick + twig bristles. Halloween family.
pub fn place_small_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let stick_mat = lambert!({ color: 0x6a3820 });
    let twig_mat = lambert!({ color: 0x8a5030 });
    let twine_mat = lambert!({ color: 0xa08050 });
    let broom = group();
    let stick_len = 0.55;
    // Stick.
    let stick = mesh(pg("SmallBrooms__stick", cyl_geo!(0.006, 0.006, stick_len, 6)), &stick_mat);
    // Lying on the floor at the handle end (y 0.006), raised to 0.020 at the
    // bristle end by the bundle.
    stick.rotation().z = PI / 2.0 + 0.014f64.atan2(stick_len);
    stick.position().set(stick_len / 2.0, 0.013, 0);
    broom.add(&stick);
    // Bristle bundle: 14 twigs radiating from end of stick.
    // Roots gathered in the twine binding, tips splayed flat on the floor in
    // a ±0.45 rad fan (thick end, +Y, at the root).
    let bristle_y = 0.020;
    for i in 0..14 {
        let angle = -0.45 + (i as f64 / 13.0) * 0.9;
        let twig_len = 0.11 + rnd() * 0.03;
        let twig = mesh(pg("SmallBrooms__twig", cyl_geo!(0.002, 0.001, twig_len, 3)), &twig_mat);
        let tz = (rnd() - 0.5) * 0.06;
        let root = v3(stick_len - 0.012, bristle_y + tz * 0.1, tz * 0.2);
        let drop = root.y - 0.0015;
        let reach = (twig_len * twig_len - drop * drop).sqrt();
        let tip = v3(root.x + angle.cos() * reach, 0.0015, root.z + angle.sin() * reach);
        let mut d = root;
        d.sub(&tip).normalize();
        twig.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
        let mut mid = root;
        mid.add(&tip).multiply_scalar(0.5);
        twig.position().copy(&mid);
        broom.add(&twig);
    }
    // Twine wrap on stick near bristles.
    let twine = mesh(pg("SmallBrooms__twine", cyl_geo!(0.008, 0.008, 0.020, 6)), &twine_mat);
    twine.rotation().z = PI / 2.0;
    twine.position().set(stick_len - 0.014, 0.020, 0);
    broom.add(&twine);
    broom.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    broom.rotation().y = rnd() * PI * 2.0;
    g.world.add(&broom);
}

// ------------------------------------------------------------------ line 43002
// R462 small tombstone: 6% chance per level. Gray stone slab
// with rounded top + engraved cross. Halloween family.
pub fn place_small_tombstones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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

// ------------------------------------------------------------------ line 43056
// R462 hanging bat: 5% chance per level. Small dark bat hanging
// upside down from ceiling with folded wings. Halloween family.
pub fn place_hanging_bats(g: &mut Game, _dist: &[i32]) {
    g.pp.bat_positions = vec![]; // R482 reset per level
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let bat_mat = lambert!({ color: 0x181008, side: DoubleSide });
    let eye_mat = basic_mat!({ color: 0xd82040 });
    let bat = group();
    let bat_y = WALL_H - 0.15;
    // Body (elongated capsule).
    let body = mesh(sphere_geo!(0.022, 8, 6), &bat_mat);
    body.scale().set(1, 1.5, 1);
    body.position().y = bat_y;
    bat.add(&body);
    // 2 folded wings (triangle wraps around body).
    for s in [-1.0, 1.0] {
        let wing_geo = tri_geo([0.0, 0.015, 0.0, s * 0.030, -0.020, 0.0, s * 0.020, 0.020, 0.0]);
        let wing = mesh(wing_geo, &bat_mat);
        wing.position().y = bat_y;
        bat.add(&wing);
    }
    // 2 pointy ears.
    for s in [-1.0, 1.0] {
        let ear = mesh(cone_geo!(0.004, 0.010, 3), &bat_mat);
        ear.position().set(s * 0.007, bat_y + 0.040, 0);
        ear.rotation().x = PI;
        bat.add(&ear);
    }
    // 2 tiny glowing red eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.002, 5, 4), &eye_mat);
        eye.position().set(s * 0.006, bat_y + 0.020, 0.014);
        bat.add(&eye);
    }
    // Hanging line/tether (invisible physically, just position at ceiling).
    bat.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bat.rotation().y = rnd() * PI * 2.0;
    // Slow sway animator.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let base_rot_z = bat.rot().z;
        let bat2 = bat.clone();
        g.animate(move |_g, _dt, t| {
            bat2.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.04;
        });
    }
    g.world.add(&bat);
    // R482 track for rare squeak audio.
    let bp = bat.pos();
    g.pp.bat_positions.push(v3(bp.x, bat_y, bp.z));
}

// ------------------------------------------------------------------ line 43127
// R461 small skeleton hand: 5% chance per level. Bone-white
// palm + 4 fingers + thumb. Halloween family.
pub fn place_small_skeleton_hands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let bone_mat = lambert!({ color: 0xe8dcc0 });
    let hand = group();
    // Palm: flat box.
    let palm = mesh(pg("SmallSkeletonHands__palm", box_geo!(0.055, 0.010, 0.045)), &bone_mat);
    palm.position().y = 0.005;
    hand.add(&palm);
    // 4 fingers (index/middle/ring/pinky), each 3 segments.
    let finger_x_offsets = [-0.020, -0.007, 0.007, 0.020];
    let finger_lengths = [0.05, 0.055, 0.05, 0.042];
    for f in 0..4 {
        let fx = finger_x_offsets[f];
        let flen = finger_lengths[f];
        // 3 segments per finger.
        for seg in 0..3 {
            let seg = seg as f64;
            let bone = mesh(pg("SmallSkeletonHands__bone", cyl_geo!(0.004, 0.004, flen / 3.0 * 0.9, 4)), &bone_mat);
            bone.rotation().x = PI / 2.0;
            bone.position().set(fx, 0.005, 0.023 + (seg + 0.5) * flen / 3.0);
            hand.add(&bone);
            // Joint sphere.
            let joint = mesh(pg("SmallSkeletonHands__joint", sphere_geo!(0.005, 5, 4)), &bone_mat);
            joint.position().set(fx, 0.005, 0.023 + (seg + 1.0) * flen / 3.0);
            hand.add(&joint);
        }
    }
    // Thumb (angled off palm).
    let thumb_len = 0.036;
    for seg in 0..2 {
        let seg = seg as f64;
        let bone = mesh(pg("SmallSkeletonHands__bone", cyl_geo!(0.004, 0.004, thumb_len / 2.0 * 0.9, 4)), &bone_mat);
        // Lying flat like the fingers, end to end along (-sin 0.9, 0, cos 0.9):
        // out from the palm's side towards the fingers' side.
        bone.rotation().x = PI / 2.0;
        bone.rotation().z = 0.9;
        bone.position().set(
            -0.024 - 0.9f64.sin() * (seg + 0.5) * thumb_len / 2.0,
            0.005,
            -0.012 + 0.9f64.cos() * (seg + 0.5) * thumb_len / 2.0,
        );
        hand.add(&bone);
    }
    // Wrist bones (2 small boxes at back).
    for s in [-1.0, 1.0] {
        let wrist = mesh(pg("SmallSkeletonHands__wrist", box_geo!(0.020, 0.008, 0.008)), &bone_mat);
        wrist.position().set(0, 0.005, -0.028 + s * 0.005);
        hand.add(&wrist);
    }
    hand.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hand);
}

// ------------------------------------------------------------------ line 43193
// R461 small cauldron: 6% chance per level. Dark iron pot with
// handle + 3 legs + optional inner green glow. Halloween family.
pub fn place_small_cauldrons(g: &mut Game, _dist: &[i32]) {
    g.pp.cauldron_positions = vec![]; // R479 reset per level
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let iron_mat = lambert!({ color: 0x181410 });
    let brew_colors: [u32; 3] = [0x30d040, 0xa020d0, 0xd82040];
    let cauldron = group();
    let is_glowing = rnd() < 0.50;
    // Body: cauldron shape (open cylinder + hemisphere bottom).
    // The pot stands 4 cm up on its legs: the round bottom (lowest 3 mm above
    // the floor) meets the leg tops at r 0.028 (y 0.013).
    let pot_lift = 0.040;
    let body = mesh(pg("SmallCauldrons__body", cyl_geo!(0.055, 0.045, 0.045, 12, 1, true)), &iron_mat);
    body.position().y = 0.030 + pot_lift;
    cauldron.add(&body);
    let bottom = mesh(pg("SmallCauldrons__bottom", sphere_geo!(0.045, 12, 6, 0, PI * 2.0, PI / 2.0, PI / 2.0)), &iron_mat);
    bottom.position().y = 0.008 + pot_lift;
    cauldron.add(&bottom);
    // Rim.
    let rim = mesh(pg("SmallCauldrons__rim", torus_geo!(0.055, 0.005, 5, 14)), &iron_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.052 + pot_lift;
    cauldron.add(&rim);
    // 3 legs.
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("SmallCauldrons__leg", cyl_geo!(0.006, 0.004, 0.014, 4)), &iron_mat);
        leg.position().set(a.cos() * 0.028, 0.007, a.sin() * 0.028);
        cauldron.add(&leg);
    }
    // Handle: arc above.
    let handle = mesh(pg("SmallCauldrons__handle", torus_geo!(0.055, 0.003, 4, 10, PI)), &iron_mat);
    // Upright bail (XY plane): eye ends in the rim at x ±0.055, arching over.
    handle.position().y = 0.052 + pot_lift;
    cauldron.add(&handle);
    if is_glowing {
        let brew_color = pick_u32(&brew_colors);
        let brew_mat = basic_mat!({ color: brew_color, transparent: true, opacity: 0.85 });
        // Liquid surface disc.
        let brew = mesh(circle_geo!(0.048, 12), &brew_mat);
        brew.rotation().x = -PI / 2.0;
        brew.position().y = 0.048 + pot_lift;
        cauldron.add(&brew);
        // Inner glow point light.
        let glow = point_light!(brew_color, 0.4, 0.6, 2);
        glow.position().y = 0.048 + pot_lift;
        cauldron.add(&glow);
        // Bubbling flicker.
        let flicker_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let wobble = (t * 3.0 + flicker_phase).sin() * 0.5 + (t * 7.0 + flicker_phase).sin() * 0.3;
            glow.set_intensity(0.4 + wobble * 0.10);
        });
    }
    cauldron.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cauldron.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cauldron);
    // R479 track glowing cauldrons for bubble audio.
    if is_glowing {
        let cp = cauldron.pos();
        g.pp.cauldron_positions.push(v3(cp.x, 0.05, cp.z));
    }
}

// ------------------------------------------------------------------ line 43272
// R460 small pumpkin: 5% chance per level. Orange pumpkin
// with carved face + inner point light. Halloween family.
pub fn place_small_pumpkins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let pump_mat = lambert!({ color: 0xd87020 });
    let stem_mat = lambert!({ color: 0x306018 });
    let face_mat = basic_mat!({ color: 0xff9020 });
    let is_lit = rnd() < 0.60;
    let pumpkin = group();
    // Body: flattened sphere.
    let body = mesh(pg("SmallPumpkins__body", sphere_geo!(0.055, 14, 10)), &pump_mat);
    body.scale().set(1, 0.8, 1);
    body.position().y = 0.044;
    pumpkin.add(&body);
    // Stem (small dark green cylinder).
    let stem = mesh(pg("SmallPumpkins__stem", cyl_geo!(0.008, 0.010, 0.020, 6)), &stem_mat);
    stem.position().y = 0.098;
    pumpkin.add(&stem);
    // Carved face: 2 triangle eyes + 1 mouth on front face.
    // Each decal sits 1 mm outside the body's (0.055, 0.044, 0.055) ellipsoid
    // (the lobed part stays inside it), turned to face along its normal.
    let on_body = |m: &Obj, px: f64, py: f64| {
        let zs = 0.055 * (1.0 - (px / 0.055).powi(2) - ((py - 0.044) / 0.044).powi(2)).sqrt();
        let mut n = v3(px / 0.003025, (py - 0.044) / 0.001936, zs / 0.003025);
        n.normalize();
        m.position().set(px, py, zs).add_scaled_vector(&n, 0.001);
        let mut tgt = m.pos();
        tgt.add(&n);
        m.look_at(tgt.x, tgt.y, tgt.z);
    };
    // Left eye.
    let eye_l = mesh(circle_geo!(0.008, 3), &face_mat);
    on_body(&eye_l, -0.018, 0.058);
    pumpkin.add(&eye_l);
    // Right eye.
    let eye_r = mesh(circle_geo!(0.008, 3), &face_mat);
    on_body(&eye_r, 0.018, 0.058);
    pumpkin.add(&eye_r);
    // Mouth: elongated crescent (rectangle w/ notch).
    let mouth = mesh(plane_geo!(0.036, 0.012), &face_mat);
    on_body(&mouth, 0.0, 0.030);
    pumpkin.add(&mouth);
    // Inner point light if lit.
    if is_lit {
        let glow = point_light!(0xff9020, 0.5, 0.7, 2);
        glow.position().set(0, 0.045, 0);
        pumpkin.add(&glow);
        // R484 warm halo sphere inside pumpkin body.
        let halo_mat = basic_mat!({ color: 0xff9020, transparent: true, opacity: 0.30, depthWrite: false });
        let halo = mesh(sphere_geo!(0.030, 6, 5), &halo_mat);
        halo.position().set(0, 0.045, 0);
        pumpkin.add(&halo);
        // Subtle flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 4.0 + rnd() * 3.0;
        g.animate(move |_g, _dt, t| {
            let wobble = (t * flicker_rate + flicker_phase).sin() * 0.5 + (t * flicker_rate * 1.7 + flicker_phase).sin() * 0.3;
            glow.set_intensity(0.5 + wobble * 0.10);
            halo.scale().set_scalar(1.0 + wobble * 0.20);
        });
    }
    pumpkin.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pumpkin.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pumpkin);
}

// ------------------------------------------------------------------ line 43350
// R460 small witch hat: 8% chance per level. Cone with brim on
// floor. Halloween family.
pub fn place_small_witch_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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

// ------------------------------------------------------------------ line 43401
// R459 small crystal ball: 5% chance per level. Translucent
// sphere on tripod stand. Fortune-teller family.
pub fn place_small_crystal_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick_tile(&open);
    let ball_colors: [u32; 4] = [0xc0e0f0, 0xe0d0f0, 0xd8e8d8, 0xf0e0d0];
    let ball_mat = basic_mat!({ color: pick_u32(&ball_colors), transparent: true, opacity: 0.45 });
    let stand_mat = lambert!({ color: 0x201a10 });
    let ball = group();
    let b_radius = 0.045;
    // Base plate.
    let base = mesh(pg("SmallCrystalBalls__base", cyl_geo!(0.035, 0.045, 0.008, 12)), &stand_mat);
    base.position().y = 0.004;
    ball.add(&base);
    // 3 support prongs.
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let prong = mesh(pg("SmallCrystalBalls__prong", cyl_geo!(0.003, 0.003, 0.018, 4)), &stand_mat);
        prong.position().set(a.cos() * 0.020, 0.018, a.sin() * 0.020);
        // Splay radially: tilt the top 0.4 rad toward local -X, then turn -X
        // onto this prong's outward direction (the claw's hook, +X, curls in).
        {
            let mut r = prong.rotation();
            r.set(0, PI - a, 0.4);
            r.order = EulerOrder::YXZ;
        }
        ball.add(&prong);
    }
    // Sphere on top.
    let sphere = mesh(sphere_geo!(b_radius, 16, 12), &ball_mat);
    sphere.position().y = 0.008 + 0.018 + b_radius - 0.008;
    ball.add(&sphere);
    // Tiny inner sparkle sphere.
    let spark = mesh(sphere_geo!(0.008, 6, 5), basic_mat!({ color: 0xffffff, transparent: true, opacity: 0.7 }));
    let spark_base_y = 0.008 + 0.018 + b_radius - 0.008 + 0.010;
    spark.position().set(-0.010, spark_base_y, -0.010);
    ball.add(&spark);
    // R485 subtle rotating inner mystical light + shimmer.
    let inner_light = point_light!(0xa0b0e0, 0.25, 0.5, 2);
    inner_light.position().y = 0.008 + 0.018 + b_radius - 0.008;
    ball.add(&inner_light);
    let shim_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let a = t * 0.6 + shim_phase;
        let orb_r = 0.015;
        {
            let mut p = spark.position();
            p.x = a.cos() * orb_r;
            p.z = a.sin() * orb_r;
            p.y = spark_base_y + (t * 1.2 + shim_phase).sin() * 0.006;
        }
        let pulse = 0.5 + (t * 1.7 + shim_phase).sin() * 0.5;
        inner_light.set_intensity(0.15 + pulse * 0.15);
        spark.scale().set_scalar(0.8 + pulse * 0.4);
    });
    ball.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}

// ------------------------------------------------------------------ line 43470
// R459 small toy sword: 8% chance per level. Kid's foam/plastic
// sword with blade + guard + hilt. Party-costume family.
pub fn place_small_toy_swords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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

// ------------------------------------------------------------------ line 43531
// R458 small toy piano: 6% chance per level. Rectangular piano
// with 7 white + 5 black keys. Child-music family.
pub fn place_small_toy_pianos(g: &mut Game, _dist: &[i32]) {
    g.pp.piano_positions = vec![]; // R488 reset per level
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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
    // R504 phantom key press animator — random key sinks briefly
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
    // R488 track for random chord audio.
    let pp = piano.pos();
    g.pp.piano_positions.push(v3(pp.x, 0.03, pp.z));
}
