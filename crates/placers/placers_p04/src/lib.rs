//! Prop placers: coins, nail files, emergency lanterns, wall pieces (mirrors,
//! clocks, frosted windows, framed photos, barometers, diplomas), paper bits
//! (envelopes, receipts, business cards, maps), games (cards, dice, poker chips),
//! old media and gadgets (cassettes, floppies, CDs, phones, handhelds, remotes,
//! pagers, calculators) and toys (yo-yos, Rubik's cubes, marbles, bricks, jump
//! ropes, teddy bears).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;
mod t01;
use t01::place_framed_photos;
mod t14;
use t14::place_teddy_bears;

// the jump rope: batch `t16`'s (`t16.rs`)
mod t16;
use t16::place_jump_ropes;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_coin_clusters", place_coin_clusters));
    r.push(("place_nail_files", place_nail_files));
    r.push(("place_emergency_lanterns", place_emergency_lanterns));
    r.push(("place_smudge_mirrors", place_smudge_mirrors));
    r.push(("place_analog_clocks", place_analog_clocks));
    r.push(("place_frosted_windows", place_frosted_windows));
    r.push(("place_envelopes", place_envelopes));
    r.push(("place_framed_photos", place_framed_photos));
    r.push(("place_receipts", place_receipts));
    r.push(("place_barometers", place_barometers));
    r.push(("place_business_cards", place_business_cards));
    r.push(("place_diplomas", place_diplomas));
    r.push(("place_playing_cards", place_playing_cards));
    r.push(("place_dice_clusters", place_dice_clusters));
    r.push(("place_poker_chips", place_poker_chips));
    r.push(("place_cassettes", place_cassettes));
    r.push(("place_folded_maps", place_folded_maps));
    r.push(("place_floppy_disks", place_floppy_disks));
    r.push(("place_c_ds", place_cds));
    r.push(("place_cell_phones", place_cell_phones));
    r.push(("place_gameboys", place_gameboys));
    r.push(("place_remotes", place_remotes));
    r.push(("place_pagers", place_pagers));
    r.push(("place_calculators", place_calculators));
    r.push(("place_yoyos", place_yoyos));
    r.push(("place_rubiks_cubes", place_rubiks_cubes));
    r.push(("place_marbles", place_marbles));
    r.push(("place_lego_bricks", place_lego_bricks));
    r.push(("place_jump_ropes", place_jump_ropes));
    r.push(("place_teddy_bears", place_teddy_bears));
}

use placers_kit::place::pick_open;

use placers_kit::place::pick_wall;

use placers_kit::place::jitter;

use placers_kit::place::wall_mount;

/// A custom geometry: positions + index, vertex normals computed.
fn tri_geo(verts: Vec<f32>, idx: Vec<u32>) -> Geo {
    let geom = Geo::new();
    geom.set_position(verts);
    geom.set_index(idx);
    geom.compute_vertex_normals();
    geom
}

// Dropped coin cluster: 8% chance per level. 3-6 coins, some face-up + some on edge.
fn place_coin_clusters(g: &mut Game, _dist: &[i32]) {
    g.pp.coin_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let coin_colors = [0xd0a040, 0xc0c0c0, 0xa06030, 0xb0b0b0, 0xc0a860];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let coin_mat = lambert!({ color: choice(&coin_colors) });
        // Coin: flat cylinder. Size varies.
        let r = 0.008 + rnd() * 0.005;
        let coin = mesh(pg("CoinScatters__coin", cyl_geo!(r, r, 0.0015, 12)), &coin_mat);
        // 20% chance on edge (rolled).
        if rnd() < 0.20 {
            coin.rotation().x = PI / 2.0;
            coin.rotation().z = rnd() * PI * 2.0;
            coin.position().set((rnd() - 0.5) * 0.14, r, (rnd() - 0.5) * 0.14);
        } else {
            coin.position().set((rnd() - 0.5) * 0.14, 0.001, (rnd() - 0.5) * 0.14);
        }
        cluster.add(&coin);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
    let p = cluster.pos();
    g.pp.coin_positions.push(v3(p.x, 0, p.z)); // track for tink audio
}

// Nail file (emery board): 7% chance per level.
fn place_nail_files(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let board_colors = [0xd050a0, 0xf0c060, 0x8060c0, 0xe0e0e0];
    let board_mat = lambert!({ color: choice(&board_colors) });
    let grain_mat = lambert!({ color: 0x606050 });
    let file = group();
    let board = mesh(pg("NailFiles__board", box_geo!(0.100, 0.003, 0.014)), &board_mat);
    board.position().y = 0.0015;
    file.add(&board);
    let grain = mesh(pg("NailFiles__grit", box_geo!(0.088, 0.001, 0.012)), &grain_mat);
    grain.position().y = 0.0035;
    file.add(&grain);
    let (px, pz) = jitter(g, x, z, 2.4);
    file.position().set(px, 0, pz);
    file.rotation().y = rnd() * PI * 2.0;
    g.world.add(&file);
}

// Hanging emergency lantern: 3% chance per level. Wall-mounted lantern w/ warm light.
fn place_emergency_lanterns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let bracket_mat = lambert!({ color: 0x404040 });
    let body_mat = lambert!({ color: 0x505050 });
    let lens_mat = basic_mat!({ color: 0xffe0a0 }); // self-lit warm.
    let lantern = group();
    let mount_y = 1.75;
    let bracket = mesh(pg("EmergencyLanterns__bracket", box_geo!(0.010, 0.02, 0.08)), &bracket_mat);
    bracket.position().set(0, mount_y + 0.05, 0.05);
    lantern.add(&bracket);
    // Chain link (2 small torus rings), alternate links at 90 degrees.
    for i in 0..2 {
        let link = mesh(pg("EmergencyLanterns__link", torus_geo!(0.006, 0.001, 3, 6)), &bracket_mat);
        link.rotation().y = if i % 2 == 0 { 0.0 } else { PI / 2.0 };
        link.position().set(0, mount_y + 0.034 - i as f64 * 0.012, 0.08);
        lantern.add(&link);
    }
    let body = mesh(pg("EmergencyLanterns__body", box_geo!(0.065, 0.09, 0.055)), &body_mat);
    body.position().set(0, mount_y - 0.03, 0.10);
    lantern.add(&body);
    // Warm lens on front (glowing), seated on the housing front.
    let lens = mesh(box_geo!(0.050, 0.055, 0.006), &lens_mat);
    lens.position().set(0, mount_y - 0.030, 0.129);
    lantern.add(&lens);
    let lamp_light = point_light!(0xffe4a0, 0.30, 2.2);
    lamp_light.position().set(0, mount_y - 0.03, 0.15);
    lantern.add(&lamp_light);
    // Subtle flicker animator: slow drift + very rare dip.
    let phase = rnd() * PI * 2.0;
    let lens_m = lens.mat();
    g.animate(move |_g, _dt, t| {
        let drift = 0.85 + 0.15 * (t * 0.7 + phase).sin();
        let dip = if (t * 0.3 + phase * 2.0).sin() > 0.94 { 0.55 } else { 1.0 };
        lamp_light.set_intensity(0.30 * drift * dip);
        let mut m = lens_m.m();
        m.opacity = drift * dip;
        m.transparent = true;
    });
    wall_mount(g, &lantern, x, z, dx, dz, 0.060);
    g.world.add(&lantern);
}

// Soap-smudged mirror, frosted window: batch `t04`'s (`t04.rs`)
mod t04;
use t04::{place_frosted_windows, place_smudge_mirrors};

// Analog wall clock: 3% chance per level. Round face + slowly moving hands.
fn place_analog_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.clock_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let rim_mat = lambert!({ color: 0x303028 });
    let face_mat = lambert!({ map: clock_face_texture() });
    let hand_mat = lambert!({ color: 0x101010 });
    let clock = group();
    let mount_y = 1.90;
    let rim = mesh(pg("AnalogClocks__rim", cyl_geo!(0.15, 0.15, 0.030, 20)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    clock.add(&rim);
    let face = mesh(circle_geo!(0.145, 20), &face_mat);
    face.position().set(0, mount_y, 0.016);
    clock.add(&face);
    // Hour hand (private geometry: pivot moved to its end).
    let hour_hand = mesh(pg_own("AnalogClocks__hourHand", box_geo!(0.008, 0.070, 0.004)), &hand_mat);
    hour_hand.position().set(0, mount_y, 0.019);
    hour_hand.geo().translate(0, 0.035, 0);
    clock.add(&hour_hand);
    // Minute hand (longer, thinner).
    let minute_hand = mesh(pg_own("AnalogClocks__minuteHand", box_geo!(0.005, 0.100, 0.004)), &hand_mat);
    minute_hand.position().set(0, mount_y, 0.021);
    minute_hand.geo().translate(0, 0.050, 0);
    clock.add(&minute_hand);
    // 60% chance: hands still move.
    let is_broken = rnd() < 0.4;
    if !is_broken {
        // minute hand: full revolution per 60 s; hour hand: per 720 s.
        let (mh, hh) = (minute_hand.clone(), hour_hand.clone());
        g.animate(move |_g, _dt, t| {
            mh.rotation().z = -t * (PI * 2.0 / 60.0);
            hh.rotation().z = -t * (PI * 2.0 / 720.0);
        });
    } else {
        // Frozen at random angle.
        minute_hand.rotation().z = rnd() * PI * 2.0;
        hour_hand.rotation().z = rnd() * PI * 2.0;
    }
    wall_mount(g, &clock, x, z, dx, dz, 0.015);
    g.world.add(&clock);
    if !is_broken {
        let p = clock.pos();
        g.pp.clock_positions.push(v3(p.x, 0, p.z)); // only ticking clocks
    }
}

// Dropped envelope: 7% chance per level. Optional letter poking out.
fn place_envelopes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let env_colors = [0xf0e8d0, 0xf8f4e8, 0xe8dcbc, 0xc0a898];
    let env_mat = lambert!({ color: choice(&env_colors), side: DoubleSide });
    let letter_mat = lambert!({ color: 0xffffff, side: DoubleSide });
    let stamp_mat = lambert!({ color: 0xc02040 });
    let env = group();
    let body = mesh(plane_geo!(0.14, 0.09), &env_mat);
    body.rotation().x = -PI / 2.0;
    body.position().y = 0.002;
    env.add(&body);
    // Diagonal seal flap crease (triangle overlay).
    let flap_geom = tri_geo(vec![-0.07, 0.0, -0.045, 0.07, 0.0, -0.045, 0.0, 0.0, 0.01], vec![0, 1, 2]);
    let flap = mesh(flap_geom, &env_mat);
    flap.position().y = 0.003;
    env.add(&flap);
    // Stamp corner.
    let stamp = mesh(plane_geo!(0.020, 0.024), &stamp_mat);
    stamp.rotation().x = -PI / 2.0;
    stamp.position().set(0.052, 0.004, -0.028);
    env.add(&stamp);
    // 40% chance letter poking out.
    if rnd() < 0.4 {
        let letter = mesh(plane_geo!(0.11, 0.075), &letter_mat);
        letter.rotation().x = -PI / 2.0;
        letter.position().set(0.04, 0.003, 0.020);
        letter.rotation().z = 0.15;
        env.add(&letter);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    env.position().set(px, 0, pz);
    env.rotation().y = rnd() * PI * 2.0;
    g.world.add(&env);
}

// Dropped receipt: 9% chance per level. Thin white strip with printed "text" lines.
fn place_receipts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let rec_mat = lambert!({ map: receipt_texture(), side: DoubleSide });
    let receipt = mesh(plane_geo!(0.048, 0.20), &rec_mat);
    receipt.rotation().x = -PI / 2.0;
    let (px, pz) = jitter(g, x, z, 2.4);
    receipt.position().set(px, 0.003, pz);
    receipt.rotation().z = rnd() * PI * 2.0;
    g.world.add(&receipt);
}

// Wall barometer gauge: 4% chance per level. Needle at random angle.
fn place_barometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let rim_mat = lambert!({ color: 0x503018 });
    let face_mat = lambert!({ map: barometer_texture() });
    let needle_mat = lambert!({ color: 0x201810 });
    let gauge = group();
    let mount_y = 1.85;
    let rim = mesh(pg("Barometers__rim", cyl_geo!(0.11, 0.11, 0.030, 16)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    gauge.add(&rim);
    let face = mesh(circle_geo!(0.100, 20), &face_mat);
    face.position().set(0, mount_y, 0.016);
    gauge.add(&face);
    // Needle (fixed at random angle within the 270 degree range).
    let needle = mesh(pg_own("Barometers__needle", box_geo!(0.005, 0.070, 0.003)), &needle_mat);
    needle.position().set(0, mount_y, 0.019);
    needle.geo().translate(0, 0.030, 0);
    needle.rotation().z = -PI * 1.25 + rnd() * PI * 1.5;
    gauge.add(&needle);
    let center = mesh(circle_geo!(0.008, 8), &needle_mat);
    center.position().set(0, mount_y, 0.020);
    gauge.add(&center);
    wall_mount(g, &gauge, x, z, dx, dz, 0.015);
    g.world.add(&gauge);
}

// Dropped business card: 10% chance per level.
fn place_business_cards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // (a white card, not a light: the lint's albedo ceiling)
    let card_mat = lambert!({ map: card_texture(), color: 0xeeeeee, side: DoubleSide });
    let card = mesh(plane_geo!(0.048, 0.028), &card_mat);
    card.rotation().x = -PI / 2.0;
    let (px, pz) = jitter(g, x, z, 2.5);
    card.position().set(px, 0.003, pz);
    card.rotation().z = rnd() * PI * 2.0;
    g.world.add(&card);
}

// Framed diploma: 3% chance per level. Ornate wall certificate.
fn place_diplomas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let frame_mat = lambert!({ color: 0xd0a040 });
    let paper_mat = lambert!({ map: diploma_texture() });
    let diploma = group();
    let mount_y = 1.65;
    let frame = mesh(pg("Diplomas__frame", box_geo!(0.26, 0.33, 0.020)), &frame_mat);
    frame.position().y = mount_y;
    diploma.add(&frame);
    let inner = mesh(pg("Diplomas__moulding", box_geo!(0.23, 0.30, 0.010)), lambert!({ color: 0xa07830 }));
    inner.position().set(0, mount_y, 0.013);
    diploma.add(&inner);
    let paper = mesh(plane_geo!(0.21, 0.28), &paper_mat);
    paper.position().set(0, mount_y, 0.019);
    diploma.add(&paper);
    diploma.rotation().z = (rnd() - 0.5) * 0.05; // slight tilt
    wall_mount(g, &diploma, x, z, dx, dz, 0.015);
    g.world.add(&diploma);
}

// Playing card: 9% chance per level. Single upturned card.
fn place_playing_cards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let card_mat = lambert!({ map: card_face_texture(), side: DoubleSide });
    let card = mesh(plane_geo!(0.043, 0.060), &card_mat);
    card.rotation().x = -PI / 2.0;
    let (px, pz) = jitter(g, x, z, 2.5);
    card.position().set(px, 0.003, pz);
    card.rotation().z = rnd() * PI * 2.0;
    g.world.add(&card);
}

// Dice cluster: 6% chance per level. 2-4 six-sided cubes w/ pip decals.
fn place_dice_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let die_colors = [0xf8f4e8, 0xc02020, 0x202020, 0x40a0e0];
    let die_color = choice(&die_colors);
    let cluster = group();
    let count = 2 + (rnd() * 3.0).floor() as i32;
    for _i in 0..count {
        let die_mat = lambert!({ color: die_color });
        let s = 0.020;
        let die = mesh(pg("DiceClusters__die", box_geo!(s, s, s)), &die_mat);
        die.position().set((rnd() - 0.5) * 0.10, s / 2.0, (rnd() - 0.5) * 0.10);
        // Rest flat on a face: x flips 0/pi and z quarter turns keep the cube axis-aligned.
        die.rotation().set(if rnd() < 0.5 { 0.0 } else { PI }, rnd() * PI, (rnd() * 4.0).floor() * PI / 2.0);
        cluster.add(&die);
        // Top pip decal, square with the die's top face (its yaw, mirrored when flipped over x).
        let pip_decal = mesh(plane_geo!(s * 0.9, s * 0.9), lambert!({ map: die_face_texture(), transparent: true }));
        let dr = die.rot();
        pip_decal.rotation().set(-PI / 2.0, 0, if dr.x != 0.0 { -dr.y } else { dr.y });
        pip_decal.position().copy(&die.pos());
        pip_decal.position().y += s / 2.0 + 0.001;
        cluster.add(&pip_decal);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// Poker chip cluster: 6% chance per level. 4-8 chips in 1-3 small stacks.
fn place_poker_chips(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // (body, ring)
    let chip_colors: [(u32, u32); 5] = [
        (0xc02020, 0xffffff), // red
        (0x202020, 0xffffff), // black
        (0x2060c0, 0xffffff), // blue
        (0x60c060, 0xffffff), // green
        (0xffffff, 0x202020), // white
    ];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    let stack_count = 1 + (rnd() * 3.0).floor() as i32;
    let mut idx = 0;
    for s in 0..stack_count {
        let stack_size = count / stack_count + if s == 0 { count % stack_count } else { 0 };
        let sx = (rnd() - 0.5) * 0.10;
        let sz = (rnd() - 0.5) * 0.10;
        for c in 0..stack_size {
            let cc = choice(&chip_colors);
            let chip_mat = lambert!({ color: cc.0 });
            let ring_mat = lambert!({ color: cc.1 });
            let chip = mesh(pg("PokerChips__chip", cyl_geo!(0.015, 0.015, 0.003, 12)), &chip_mat);
            chip.position().set(sx, 0.002 + c as f64 * 0.0035, sz);
            cluster.add(&chip);
            // Contrast stripe ring on edge.
            let ring = mesh(pg("PokerChips__stripe", torus_geo!(0.015, 0.0006, 3, 12)), &ring_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(sx, 0.002 + c as f64 * 0.0035, sz);
            cluster.add(&ring);
            idx += 1;
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// Cassette tape: 6% chance per level. Shell w/ 2 reels + label.
fn place_cassettes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0x202020, 0xf0e8d0, 0x804060, 0x408060, 0xc0a040];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let reel_mat = lambert!({ color: 0x101010 });
    let tape_mat = lambert!({ color: 0x402010 });
    let label_colors = [0xffffff, 0xf8d040, 0xd0a0e0, 0x80c0e0];
    let label_mat = lambert!({ color: label_colors[(rnd() * 4.0).floor() as usize] });
    let cassette = group();
    let shell = mesh(pg("Cassettes__shell", box_geo!(0.100, 0.006, 0.062)), &shell_mat);
    shell.position().y = 0.003;
    cassette.add(&shell);
    let label = mesh(plane_geo!(0.084, 0.030), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.007, 0.008);
    cassette.add(&label);
    // 2 reels.
    for side in [-1.0f64, 1.0] {
        let reel = mesh(pg("Cassettes__reel", cyl_geo!(0.017, 0.017, 0.001, 12)), &reel_mat);
        reel.position().set(side * 0.024, 0.007, -0.012);
        cassette.add(&reel);
        // Small brown tape strip between reels.
        if side == 1.0 {
            let tape = mesh(box_geo!(0.048, 0.001, 0.006), &tape_mat);
            tape.position().set(0, 0.007, -0.024);
            cassette.add(&tape);
        }
        // 6 sprocket teeth around each reel hub.
        for t in 0..6 {
            let a = (t as f64 / 6.0) * PI * 2.0;
            let tooth = mesh(box_geo!(0.002, 0.001, 0.002), &reel_mat);
            tooth.position().set(side * 0.024 + a.cos() * 0.006, 0.008, -0.012 + a.sin() * 0.006);
            cassette.add(&tooth);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cassette.position().set(px, 0, pz);
    cassette.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cassette);
}

// Folded map: 5% chance per level. Folded paper square w/ road texture.
fn place_folded_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let map_mat = lambert!({ map: map_texture(), side: DoubleSide });
    let map = group();
    let base = mesh(plane_geo!(0.11, 0.11), &map_mat);
    base.rotation().x = -PI / 2.0;
    base.position().y = 0.003;
    map.add(&base);
    // Small folded corner (raised triangle).
    let fold_geom = tri_geo(vec![0.04, 0.003, 0.03, 0.055, 0.003, 0.055, 0.055, 0.010, 0.03], vec![0, 1, 2]);
    let fold = mesh(fold_geom, &map_mat);
    map.add(&fold);
    let (px, pz) = jitter(g, x, z, 2.4);
    map.position().set(px, 0, pz);
    map.rotation().y = rnd() * PI * 2.0;
    g.world.add(&map);
}

// Floppy disk 3.5": 7% chance per level. Shell w/ metal shutter + label strip.
fn place_floppy_disks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0x202020, 0x4060c0, 0xc02020, 0x606060, 0xf0e8d0, 0x408060];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let shutter_mat = lambert!({ color: 0xa0a0a0 });
    let label_mat = lambert!({ color: 0xf8f4e8 });
    let floppy = group();
    let shell = mesh(pg("FloppyDisks__shell", box_geo!(0.085, 0.005, 0.088)), &shell_mat);
    shell.position().y = 0.0025;
    floppy.add(&shell);
    let shutter = mesh(pg("FloppyDisks__shutter", box_geo!(0.045, 0.002, 0.030)), &shutter_mat);
    shutter.position().set(0, 0.006, -0.028);
    floppy.add(&shutter);
    let notch = mesh(box_geo!(0.015, 0.001, 0.008), lambert!({ color: 0x201818 }));
    notch.position().set(0.015, 0.007, -0.028);
    floppy.add(&notch);
    let label = mesh(plane_geo!(0.055, 0.028), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.006, 0.012);
    floppy.add(&label);
    let tab_hole = mesh(box_geo!(0.005, 0.001, 0.005), lambert!({ color: 0x100808 }));
    tab_hole.position().set(-0.038, 0.006, 0.038);
    floppy.add(&tab_hole);
    let (px, pz) = jitter(g, x, z, 2.4);
    floppy.position().set(px, 0, pz);
    floppy.rotation().y = rnd() * PI * 2.0;
    g.world.add(&floppy);
}

// CD/DVD disc: 8% chance per level. Sometimes label side up.
fn place_cds(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let label_side_up = rnd() < 0.5;
    let data_mat = lambert!({ color: 0xd0d0d8 });
    let label_colors = [0xffffff, 0xd050a0, 0x2080c0, 0x40a040, 0xc0a040, 0x202020];
    let label_mat = lambert!({ color: choice(&label_colors) });
    let disc = group();
    let face = mesh(pg("CDs__disc", cyl_geo!(0.060, 0.060, 0.001, 16)), if label_side_up { &label_mat } else { &data_mat });
    face.position().y = 0.0015;
    disc.add(&face);
    let under = mesh(pg("CDs__disc", cyl_geo!(0.060, 0.060, 0.001, 16)), if label_side_up { &data_mat } else { &label_mat });
    under.position().y = 0.0005;
    under.rotation().x = PI;
    disc.add(&under);
    let hole = mesh(cyl_geo!(0.0075, 0.0075, 0.003, 8), lambert!({ color: 0x080808 }));
    hole.position().y = 0.0015;
    disc.add(&hole);
    let ring = mesh(pg("CDs__hubRing", torus_geo!(0.018, 0.001, 3, 12)), lambert!({ color: 0x808088 }));
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.0025;
    disc.add(&ring);
    let (px, pz) = jitter(g, x, z, 2.4);
    disc.position().set(px, 0, pz);
    disc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&disc);
}

// Cell phone (brick style): 6% chance per level.
fn place_cell_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x606060, 0x2040a0, 0xa02020, 0x408060];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x203020 });
    let key_mat = lambert!({ color: 0x808080 });
    let phone = group();
    let body = mesh(pg("CellPhones__body", box_geo!(0.045, 0.014, 0.100)), &body_mat);
    body.position().y = 0.007;
    phone.add(&body);
    let screen = mesh(box_geo!(0.032, 0.001, 0.024), &screen_mat);
    screen.position().set(0, 0.015, -0.028);
    phone.add(&screen);
    // 12 keypad buttons (3x4 grid).
    for r in 0..4 {
        for c in 0..3 {
            let key = mesh(pg("CellPhones__key", box_geo!(0.010, 0.002, 0.010)), &key_mat);
            key.position().set(-0.012 + c as f64 * 0.012, 0.015, -0.008 + r as f64 * 0.014);
            phone.add(&key);
        }
    }
    let antenna = mesh(pg("CellPhones__antenna", cyl_geo!(0.002, 0.002, 0.020, 4)), &body_mat);
    antenna.position().set(0.020, 0.020, -0.055);
    phone.add(&antenna);
    let (px, pz) = jitter(g, x, z, 2.4);
    phone.position().set(px, 0, pz);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}

// Gameboy-style handheld: 5% chance per level.
fn place_gameboys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0xe0e0e0, 0x40a0e0, 0xe08040, 0xa0d080];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_frame_mat = lambert!({ color: 0x404040 });
    let screen_mat = lambert!({ color: 0x506830 });
    let dpad_mat = lambert!({ color: 0x101010 });
    let ab_mat = lambert!({ color: 0x802020 });
    let gb = group();
    let body = mesh(pg("Gameboys__body", box_geo!(0.060, 0.014, 0.090)), &body_mat);
    body.position().y = 0.007;
    gb.add(&body);
    let screen_frame = mesh(pg("Gameboys__bezel", box_geo!(0.048, 0.002, 0.038)), &screen_frame_mat);
    screen_frame.position().set(0, 0.015, -0.020);
    gb.add(&screen_frame);
    let screen = mesh(pg("Gameboys__screen", box_geo!(0.036, 0.001, 0.028)), &screen_mat);
    screen.position().set(0, 0.0164, -0.020); // on the bezel (top 0.016)
    gb.add(&screen);
    // D-pad plus (2 cross bars), on the body (top 0.014).
    let dpad_h = mesh(pg("Gameboys__dpadH", box_geo!(0.014, 0.003, 0.005)), &dpad_mat);
    dpad_h.position().set(-0.016, 0.0153, 0.020);
    gb.add(&dpad_h);
    let dpad_v = mesh(pg("Gameboys__dpadV", box_geo!(0.005, 0.003, 0.014)), &dpad_mat);
    dpad_v.position().set(-0.016, 0.0153, 0.020);
    gb.add(&dpad_v);
    // A + B buttons.
    let btn_a = mesh(pg("Gameboys__button", cyl_geo!(0.005, 0.005, 0.003, 8)), &ab_mat);
    btn_a.position().set(0.020, 0.0153, 0.014);
    gb.add(&btn_a);
    let btn_b = mesh(pg("Gameboys__button", cyl_geo!(0.005, 0.005, 0.003, 8)), &ab_mat);
    btn_b.position().set(0.012, 0.0153, 0.024);
    gb.add(&btn_b);
    // Start + Select bars (2 small pills).
    for i in 0..2 {
        let bar = mesh(pg("Gameboys__pill", cyl_geo!(0.002, 0.002, 0.008, 5)), &screen_frame_mat);
        bar.rotation().z = PI / 2.0;
        bar.position().set(-0.006 + i as f64 * 0.012, 0.016, 0.036);
        gb.add(&bar);
    }
    // Speaker dots (5 tiny holes) on the body top.
    for d in 0..5 {
        let dot = mesh(cyl_geo!(0.001, 0.001, 0.001, 4), &screen_frame_mat);
        dot.position().set(0.014 + d as f64 * 0.003, 0.0143, 0.038);
        gb.add(&dot);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    gb.position().set(px, 0, pz);
    gb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gb);
}

// TV remote control: 6% chance per level.
fn place_remotes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x606060, 0x804020, 0xe0e0d0];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let btn_mat = lambert!({ color: 0x404040 });
    let big_btn_mat = lambert!({ color: 0x9c3a32 });
    let ir_mat = basic_mat!({ color: 0x5a1e1a });
    let remote = group();
    let body = mesh(pg("Remotes__body", box_geo!(0.045, 0.014, 0.18)), &body_mat);
    body.position().y = 0.007;
    remote.add(&body);
    // IR emitter dot (dark red, at one end).
    let ir = mesh(circle_geo!(0.004, 6), &ir_mat);
    ir.rotation().x = -PI / 2.0;
    ir.position().set(0, 0.015, -0.088);
    remote.add(&ir);
    let power = mesh(pg("Remotes__power", cyl_geo!(0.006, 0.006, 0.003, 8)), &big_btn_mat);
    power.position().set(0, 0.016, -0.070);
    remote.add(&power);
    // Number pad 3x4 grid.
    for r in 0..4 {
        for c in 0..3 {
            let key = mesh(pg("Remotes__key", box_geo!(0.008, 0.002, 0.008)), &btn_mat);
            key.position().set(-0.010 + c as f64 * 0.010, 0.015, -0.030 + r as f64 * 0.014);
            remote.add(&key);
        }
    }
    let vol = mesh(pg("Remotes__rocker", box_geo!(0.008, 0.003, 0.020)), &btn_mat);
    vol.position().set(-0.014, 0.016, 0.048);
    remote.add(&vol);
    let ch = mesh(pg("Remotes__rocker", box_geo!(0.008, 0.003, 0.020)), &btn_mat);
    ch.position().set(0.014, 0.016, 0.048);
    remote.add(&ch);
    let menu = mesh(pg("Remotes__menu", box_geo!(0.014, 0.002, 0.008)), &btn_mat);
    menu.position().set(0, 0.015, 0.076);
    remote.add(&menu);
    let (px, pz) = jitter(g, x, z, 2.4);
    remote.position().set(px, 0, pz);
    remote.rotation().y = rnd() * PI * 2.0;
    g.world.add(&remote);
}

// Pager: 8% chance per level. Chunky body w/ tiny LCD screen + clip.
fn place_pagers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x404040, 0x2040a0, 0x408060];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x506830 });
    let btn_mat = lambert!({ color: 0x707070 });
    let pager = group();
    let body = mesh(pg("Pagers__body", box_geo!(0.048, 0.020, 0.070)), &body_mat);
    body.position().y = 0.012; // lying on its back on the 2 mm belt clip
    pager.add(&body);
    let screen = mesh(box_geo!(0.036, 0.002, 0.020), &screen_mat);
    screen.position().set(0, 0.023, -0.020);
    pager.add(&screen);
    for i in 0..3 {
        let btn = mesh(pg("Pagers__button", cyl_geo!(0.004, 0.004, 0.002, 8)), &btn_mat);
        btn.position().set(-0.012 + i as f64 * 0.012, 0.024, 0.005);
        pager.add(&btn);
    }
    // Belt clip on back (bent metal strip).
    let clip_mat = lambert!({ color: 0x505050 });
    let clip = mesh(box_geo!(0.020, 0.002, 0.055), &clip_mat);
    clip.position().set(0, 0.001, 0);
    pager.add(&clip);
    let (px, pz) = jitter(g, x, z, 2.4);
    pager.position().set(px, 0, pz);
    pager.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pager);
}

// Pocket calculator: 6% chance per level. Solar strip + display + button grid.
fn place_calculators(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x404060, 0x604040, 0xe0e0e0];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x506830 });
    let solar_mat = lambert!({ color: 0x201830 });
    let btn_mat = lambert!({ color: 0x808080 });
    let op_mat = lambert!({ color: 0xc0a020 });
    let calc = group();
    let body = mesh(pg("Calculators__body", box_geo!(0.055, 0.010, 0.085)), &body_mat);
    body.position().y = 0.005;
    calc.add(&body);
    let screen = mesh(box_geo!(0.042, 0.001, 0.018), &screen_mat);
    screen.position().set(0, 0.011, -0.028);
    calc.add(&screen);
    let solar = mesh(box_geo!(0.020, 0.001, 0.004), &solar_mat);
    solar.position().set(-0.014, 0.011, -0.0395); // clear of the LCD so the two don't z-fight
    calc.add(&solar);
    // 4x4 button grid.
    for r in 0..4 {
        for c in 0..4 {
            let is_op = c == 3;
            let key = mesh(pg("Calculators__key", box_geo!(0.009, 0.002, 0.009)), if is_op { &op_mat } else { &btn_mat });
            key.position().set(-0.018 + c as f64 * 0.012, 0.011, -0.010 + r as f64 * 0.014);
            calc.add(&key);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    calc.position().set(px, 0, pz);
    calc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&calc);
}

// Yo-yo: 7% chance per level. Two disc halves + string trailing.
fn place_yoyos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let yoyo_colors = [0xc02020, 0x2080e0, 0x40c060, 0xe0c040, 0xd050a0, 0x202020];
    let yoyo_mat = lambert!({ color: choice(&yoyo_colors) });
    let axle_mat = lambert!({ color: 0x606060 });
    let string_mat = lambert!({ color: 0xf0f0e8 });
    let yoyo = group();
    let r = 0.028;
    let half_l = mesh(pg("Yoyos__half", cyl_geo!(r, r, 0.008, 12)), &yoyo_mat);
    half_l.rotation().x = PI / 2.0;
    half_l.position().set(0, r, -0.006);
    yoyo.add(&half_l);
    let half_r = mesh(pg("Yoyos__half", cyl_geo!(r, r, 0.008, 12)), &yoyo_mat);
    half_r.rotation().x = PI / 2.0;
    half_r.position().set(0, r, 0.006);
    yoyo.add(&half_r);
    let axle = mesh(pg("Yoyos__axle", cyl_geo!(0.006, 0.006, 0.008, 6)), &axle_mat);
    axle.rotation().x = PI / 2.0;
    axle.position().set(0, r, 0);
    yoyo.add(&axle);
    // Trailing string: one joined polyline from the hub, out through the gap between the
    // halves, down to the floor and curving away along it.
    let mut str_pts: Vec<Vec3> = vec![v3(0.004, r - 0.004, 0), v3(r + 0.006, 0.0006, 0)];
    let mut hd = 0.0f64;
    for i in 1..6 {
        hd += 0.12;
        let q = str_pts[i];
        str_pts.push(v3(q.x + hd.cos() * 0.028, 0.0006, q.z + hd.sin() * 0.028));
    }
    for i in 0..6 {
        let seg = mesh(cyl_geo!(0.0006, 0.0006, 0.030, 3), &string_mat);
        let mut d = str_pts[i + 1];
        d.sub(&str_pts[i]);
        let len = d.length();
        d.divide_scalar(len);
        seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
        seg.scale().y = (len + 0.001) / 0.030; // overlap the joints a little
        seg.position().copy(&str_pts[i]).lerp(&str_pts[i + 1], 0.5);
        yoyo.add(&seg);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    yoyo.position().set(px, 0, pz);
    yoyo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&yoyo);
}

// Twisty puzzle cube: batch `t22`'s (`t22.rs`)
mod t22;
use t22::place_rubiks_cubes;

// Marbles cluster: 8% chance per level. 5-10 small translucent glass spheres.
fn place_marbles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let marble_colors = [0xd8e8f0, 0xf0d8a0, 0xd0c0e0, 0xa0e0c0, 0xe0a0a0, 0xc0e0f0];
    let cluster = group();
    let count = 5 + (rnd() * 6.0).floor() as i32;
    for _i in 0..count {
        let m_mat = lambert!({ color: choice(&marble_colors), transparent: true, opacity: 0.7 });
        let r = 0.008 + rnd() * 0.003;
        let marble = mesh(sphere_geo!(r, 8, 6), &m_mat);
        marble.position().set((rnd() - 0.5) * 0.14, r, (rnd() - 0.5) * 0.14);
        cluster.add(&marble);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// LEGO brick scatter: 6% chance per level. 5-9 small blocks w/ studs, some stacked.
fn place_lego_bricks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let brick_colors = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xe0e0e0, 0xd050a0, 0x202020];
    let cluster = group();
    let count = 5 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let brick_mat = lambert!({ color: choice(&brick_colors) });
        let brick = group();
        // Random brick size: 2x2, 2x4, or 2x3.
        let w = 0.020;
        let l = 0.020 + (rnd() * 3.0).floor() * 0.020;
        let stud_cols = 2.0;
        let stud_rows = round_half_up(l / 0.020);
        let body = mesh(pg("LegoBricks__body", box_geo!(w, 0.010, l)), &brick_mat);
        body.position().y = 0.005;
        brick.add(&body);
        // Studs on top (on the body top, 0.010).
        for r in 0..stud_rows as i32 {
            for c in 0..stud_cols as i32 {
                let stud = mesh(pg("LegoBricks__stud", cyl_geo!(0.004, 0.004, 0.003, 8)), &brick_mat);
                stud.position().set(-w / 2.0 + (c as f64 + 0.5) * (w / stud_cols), 0.0114, -l / 2.0 + (r as f64 + 0.5) * (l / stud_rows));
                brick.add(&stud);
            }
        }
        let px = (rnd() - 0.5) * 0.16;
        let stacked = rnd() < 0.3; // some stacked
        let pz = (rnd() - 0.5) * 0.16;
        let ry = rnd() * PI * 2.0;
        let below = if stacked && cluster.child_count() > 0 { cluster.children().last().cloned() } else { None };
        if let Some(below) = below {
            // On the previous brick, square to its stud grid and shifted along it by up to one stud.
            let off = round_half_up(px / 0.08) * 0.010;
            let br = below.rot();
            let bp = below.pos();
            brick.rotation().y = br.y + if ry < PI { 0.0 } else { PI / 2.0 };
            brick.position().set(bp.x + br.y.sin() * off, bp.y + 0.010, bp.z + br.y.cos() * off);
        } else {
            brick.position().set(px, 0, pz);
            brick.rotation().y = ry;
        }
        cluster.add(&brick);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
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
