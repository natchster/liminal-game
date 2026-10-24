//! Prop placers: car keys, birth announcements, wallets, postcards, shoe pairs,
//! coat racks, dropped gloves, and laundry / cleaning props (laundry baskets,
//! ironing boards, vacuum cleaners, mop buckets, brooms and dustpans).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_car_keys", place_car_keys));
    r.push(("place_birth_announcements", place_birth_announcements));
    r.push(("place_wallets", place_wallets));
    r.push(("place_vintage_postcards", place_vintage_postcards));
    r.push(("place_shoe_pairs", place_shoe_pairs));
    r.push(("place_coat_racks", place_coat_racks));
    r.push(("place_dropped_gloves", place_dropped_gloves));
    r.push(("place_laundry_baskets", place_laundry_baskets));
    r.push(("place_ironing_boards", place_ironing_boards));
    r.push(("place_vacuum_cleaners", place_vacuum_cleaners));
    r.push(("place_cleaning_mop_buckets", place_cleaning_mop_buckets));
    r.push(("place_brooms_and_dustpans", place_brooms_and_dustpans));
}

// ------------------------------------------------------------------ helpers
/// The `open_tiles` scan every floor placer starts with (`tiles[t_idx(x, z)] == 0`).
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

/// The `wall_tiles` scan of the wall placers: open tiles with the first
/// neighbouring wall (`== 1`) in `[(1,0), (-1,0), (0,1), (0,-1)]` order.
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// Oriented cylinder segment between two points
/// (positioned at the midpoint, rotated so its +Y axis points from `prev` to `next`).
fn orient_seg(seg: &Obj, prev: &Vec3, next: &Vec3) {
    let mut mid = *prev;
    mid.add(next).multiply_scalar(0.5);
    seg.position().copy(&mid);
    let mut v = *next;
    v.sub(prev).normalize();
    seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
}

// R698 dropped car keys w/ keyring + tag (floor): 7% chance per level.
fn place_car_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let key_mat = if rnd() < 0.5 { metal_mat.clone() } else { brass_mat.clone() };
    // Fob color pool (car remote body).
    let fob_colors = [0x101010, 0x2060a0, 0x603030, 0x404040];
    let fob_mat = lambert!({ color: pick(&fob_colors) });
    let k = group();
    // Central keyring — flat torus.
    let ring = mesh(pg("CarKeys__ring", torus_geo!(0.012, 0.0012, 4, 12)), &metal_mat);
    ring.rotation().x = -PI / 2.0;
    ring.position().y = 0.002;
    k.add(&ring);
    // Main house key — flat body + notched teeth.
    let key = group();
    // Head — flat circle-ish disc.
    let key_head = mesh(pg("CarKeys__bow", cyl_geo!(0.008, 0.008, 0.0015, 8)), &key_mat);
    key_head.position().set(0, 0, 0);
    key.add(&key_head);
    // Shaft — thin rectangular box.
    let shaft = mesh(pg("CarKeys__blade", box_geo!(0.028, 0.0015, 0.004)), &key_mat);
    shaft.position().set(0.018, 0, 0);
    key.add(&shaft);
    // 3 tiny tooth notches on bottom edge.
    for i in 0..3 {
        let tooth = mesh(pg("CarKeys__tooth", box_geo!(0.003, 0.001, 0.003)), &key_mat);
        tooth.position().set(0.020 + i as f64 * 0.006, 0, 0.003);
        key.add(&tooth);
    }
    // Head hole (dark).
    let hole_mat = lambert!({ color: 0x0a0a0a });
    let hole = mesh(cyl_geo!(0.002, 0.002, 0.002, 6), &hole_mat);
    hole.position().set(-0.0042, 0.001, 0); // in the bow's hanging hole
    key.add(&hole);
    key.rotation().y = 0.5 + (rnd() - 0.5) * 0.5;
    key.rotation().z = (rnd() - 0.5) * 0.15;
    // threaded: the bow's hole (x = -0.0042) sits on the ring, key pointing out
    let kry = key.rot().y;
    key.position().set(kry.cos() * 0.0162, 0.002, -kry.sin() * 0.0162);
    k.add(&key);
    // 2nd smaller key — mailbox/luggage style (short, no teeth).
    let key2 = group();
    let key2_head = mesh(pg("CarKeys__bow", cyl_geo!(0.006, 0.006, 0.0015, 6)), &key_mat);
    key2.add(&key2_head);
    let shaft2 = mesh(pg("CarKeys__blade2", box_geo!(0.016, 0.0015, 0.003)), &key_mat);
    shaft2.position().set(0.011, 0, 0);
    key2.add(&shaft2);
    key2.rotation().y = 1.8;
    key2.position().set(1.8f64.cos() * 0.0152, 0.002, -1.8f64.sin() * 0.0152); // hole on the ring
    k.add(&key2);
    // Car fob — small rectangular remote w/ buttons.
    let fob = group();
    let fob_body = mesh(pg("CarKeys__fob", box_geo!(0.024, 0.006, 0.014)), &fob_mat);
    fob_body.position().y = 0.003;
    fob.add(&fob_body);
    // 2 tiny raised buttons.
    let btn_mat = lambert!({ color: 0x606060 });
    for i in 0..2 {
        let btn = mesh(pg("CarKeys__button", cyl_geo!(0.0025, 0.0025, 0.001, 5)), &btn_mat);
        btn.position().set(-0.005 + i as f64 * 0.010, 0.007, 0);
        fob.add(&btn);
    }
    // Fob attachment ring (small torus).
    let fob_ring = mesh(pg("CarKeys__smallring", torus_geo!(0.003, 0.0007, 3, 6)), &metal_mat);
    fob_ring.rotation().x = PI / 2.0;
    fob_ring.position().set(-0.014, 0.003, 0);
    fob.add(&fob_ring);
    fob.rotation().y = -0.6 + (rnd() - 0.5) * 0.3;
    // its small ring (local x = -0.014) links the key ring, body pointing out
    let fry = fob.rot().y;
    fob.position().set(fry.cos() * 0.0285, 0, -fry.sin() * 0.0285);
    k.add(&fob);
    // 60% chance: leather tag w/ initial letter.
    if rnd() < 0.6 {
        let tag_colors = [0x603020, 0x804020, 0x201010, 0x8b5a2b];
        let tag_mat = lambert!({ color: pick(&tag_colors) });
        let tag = group();
        // Tag body — small rectangular tag.
        let tag_body = mesh(pg("CarKeys__tag", box_geo!(0.016, 0.002, 0.024)), &tag_mat);
        tag_body.position().y = 0.001;
        tag.add(&tag_body);
        // Ring hole area (darker).
        let tag_ring_spot = mesh(circle_geo!(0.002, 5), &hole_mat);
        tag_ring_spot.rotation().x = -PI / 2.0;
        tag_ring_spot.position().set(0, 0.0021, -0.008);
        tag.add(&tag_ring_spot);
        // Metal attachment ring.
        let tag_ring = mesh(pg("CarKeys__smallring", torus_geo!(0.003, 0.0007, 3, 6)), &metal_mat);
        tag_ring.rotation().x = PI / 2.0;
        tag_ring.position().set(0, 0.002, -0.010);
        tag.add(&tag_ring);
        // Small pale initial letter (rectangle stamped indication — abstract).
        let stamp_mat = lambert!({ color: 0xd0c0a0 });
        let stamp = mesh(pg("CarKeys__tagplate", box_geo!(0.006, 0.002, 0.010)), &stamp_mat);
        stamp.position().set(0, 0.003, 0.003);
        tag.add(&stamp);
        // on the free side of the ring, its small ring (local z = -0.010) linking it
        tag.rotation().y = -1.4 + (rnd() - 0.5) * 0.3;
        let try_ = tag.rot().y;
        tag.position().set(try_.sin() * 0.0245, 0, try_.cos() * 0.0245);
        k.add(&tag);
    }
    k.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    k.rotation().y = rnd() * PI * 2.0;
    g.world.add(&k);
}

// R698 framed hospital birth announcement (wall): 2.5% chance per level.
fn place_birth_announcements(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let frame_colors = [0xf0e0e8, 0xe0e8f0, 0xf0e8d0, 0xd0a040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let card_mat = lambert!({ map: birth_announcement_texture() });
    let ba = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.20, 0.22);
    let frame = mesh(pg("BirthAnnouncements__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    ba.add(&frame);
    let card = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &card_mat);
    card.position().set(0, mount_y, 0.019);
    ba.add(&card);
    ba.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    ba.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    ba.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&ba);
}

// R700 wallet w/ ID card + spilled cash bills (floor): 6% chance per level.
fn place_wallets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Wallet leather color pool.
    let wallet_colors = [0x603020, 0x201010, 0x8b5a2b, 0x404040, 0x502010];
    let wallet_mat = lambert!({ color: pick(&wallet_colors) });
    // ID card color (pale card w/ photo).
    let id_base_mat = lambert!({ color: 0xf0e8d0 });
    let id_photo_mat = lambert!({ color: 0xa08060 });
    // Cash color (green US bill).
    let bill_mat = lambert!({ color: 0x60805a });
    let w = group();
    // Bi-fold wallet — open flat, so 2 halves + spine bump.
    let (half_w, half_d) = (0.045, 0.075);
    let left_half = mesh(pg("Wallets__half", box_geo!(half_w, 0.006, half_d)), &wallet_mat);
    left_half.position().set(-half_w / 2.0, 0.003, 0);
    w.add(&left_half);
    let right_half = mesh(pg("Wallets__half", box_geo!(half_w, 0.006, half_d)), &wallet_mat);
    right_half.position().set(half_w / 2.0, 0.003, 0);
    w.add(&right_half);
    // Spine bump — slightly raised center.
    let spine = mesh(pg("Wallets__fold", box_geo!(0.004, 0.008, half_d)), &wallet_mat);
    spine.position().set(0, 0.005, 0);
    w.add(&spine);
    // Card slot rim (thin box on left half interior).
    let slot_rim = mesh(pg("Wallets__pocket", box_geo!(half_w * 0.85, 0.002, 0.006)), &wallet_mat);
    slot_rim.position().set(-half_w / 2.0, 0.007, -half_d * 0.20);
    w.add(&slot_rim);
    // ID card protruding from left slot (partially).
    let id_card = group();
    let id_body = mesh(pg("Wallets__card", box_geo!(half_w * 0.85, 0.002, half_d * 0.55)), &id_base_mat);
    id_card.add(&id_body);
    // Photo square top-left of card.
    let id_photo = mesh(box_geo!(half_w * 0.30, 0.003, half_d * 0.20), &id_photo_mat);
    id_photo.scale().y = 0.5; // printed on the card: 0.25 mm proud, not 1.5 mm
    id_photo.position().set(-half_w * 0.24, 0.0005, -half_d * 0.15);
    id_card.add(&id_photo);
    // 3 text lines on card (thin dark boxes).
    let id_text_mat = lambert!({ color: 0x202020 });
    for i in 0..3 {
        let line = mesh(box_geo!(half_w * 0.35, 0.002, 0.002), &id_text_mat);
        line.scale().y = 0.5;
        line.position().set(half_w * 0.10, 0.0007, -half_d * 0.20 + i as f64 * 0.010);
        id_card.add(&line);
    }
    // Card is angled slightly out of slot.
    id_card.position().set(-half_w / 2.0, 0.007, half_d * 0.10);
    id_card.rotation().z = 0.15;
    id_card.rotation().y = -0.1;
    w.add(&id_card);
    // 2-3 spilled cash bills nearby.
    let bill_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..bill_count {
        let bill = mesh(pg("Wallets__bill", box_geo!(0.06, 0.001, 0.030)), &bill_mat);
        // Small darker denomination band on one edge.
        let band_mat = lambert!({ color: 0x304a2a });
        let band = mesh(pg("Wallets__band", box_geo!(0.06, 0.0012, 0.008)), &band_mat);
        band.position().set(0, 0.0002, 0.010);
        bill.add(&band);
        // Central presidential-portrait blob (darker oval).
        let portrait_mat = lambert!({ color: 0x304a2a });
        let portrait = mesh(circle_geo!(0.008, 6), &portrait_mat);
        portrait.rotation().x = -PI / 2.0;
        portrait.position().set(0, 0.0007, 0);
        bill.add(&portrait);
        let ang = (rnd() - 0.5) * PI * 1.5;
        let rr = 0.08 + rnd() * 0.06;
        bill.position().set(ang.cos() * rr, 0.0005 + i as f64 * 0.001, ang.sin() * rr);
        bill.rotation().y = rnd() * PI * 2.0;
        // Slight bend (rotate around random axis).
        bill.rotation().z = (rnd() - 0.5) * 0.2;
        w.add(&bill);
    }
    // Optional coin (30%) — small silver disc.
    if rnd() < 0.3 {
        let coin_mat = lambert!({ color: 0xa0a0a8 });
        let coin = mesh(pg("Wallets__coin", cyl_geo!(0.007, 0.007, 0.002, 8)), &coin_mat);
        coin.position().set((rnd() - 0.5) * 0.12, 0.001, (rnd() - 0.5) * 0.12);
        w.add(&coin);
    }
    w.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    w.rotation().y = rnd() * PI * 2.0;
    g.world.add(&w);
}

// R700 framed vintage postcard (wall): 3% chance per level.
fn place_vintage_postcards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xd0a040, 0x505050];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let card_mat = lambert!({ map: postcard_texture() });
    let vp = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.24, 0.17);
    let frame = mesh(pg("VintagePostcards__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    vp.add(&frame);
    let card = mesh(plane_geo!(frame_w - 0.018, frame_h - 0.018), &card_mat);
    card.position().set(0, mount_y, 0.017);
    vp.add(&card);
    vp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    vp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    vp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&vp);
}

// R702 shoe pair (floor): 6% chance per level. One upright, one tipped.
fn place_shoe_pairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let shoe_colors = [0x603020, 0x101010, 0x8b5a2b, 0x201010, 0xa02040, 0xf0e0d0, 0x2060a0];
    let shoe_color = pick(&shoe_colors);
    let shoe_mat = lambert!({ color: shoe_color });
    let sole_mat = lambert!({ color: 0x2a2a2a });
    let lace_mat = lambert!({ color: if shoe_color == 0x101010 { 0xffffff } else { 0x101010 } });
    // Shoe builder — sneaker/loafer silhouette.
    let make_shoe = |is_right: bool| -> Obj {
        let s = group();
        // Sole (flat rounded box).
        let sole = mesh(pg("ShoePairs__sole", box_geo!(0.09, 0.014, 0.24)), &sole_mat);
        sole.position().y = 0.007;
        s.add(&sole);
        // Upper body — main shoe upper (taller box in middle).
        let upper = mesh(pg("ShoePairs__upper", box_geo!(0.086, 0.040, 0.16)), &shoe_mat);
        upper.position().set(0, 0.030, -0.03);
        s.add(&upper);
        // Toe cap — slightly rounded box in front.
        let toe = mesh(pg("ShoePairs__toe", box_geo!(0.086, 0.030, 0.08)), &shoe_mat);
        toe.position().set(0, 0.023, 0.08);
        s.add(&toe);
        // Heel — taller box at back.
        let heel = mesh(pg("ShoePairs__heel", box_geo!(0.086, 0.050, 0.05)), &shoe_mat);
        heel.position().set(0, 0.033, -0.100);
        s.add(&heel);
        // Opening cavity — darker box on top (visual for shoe interior).
        let opening_mat = lambert!({ color: 0x1a1a1a });
        let opening = mesh(pg("ShoePairs__opening", box_geo!(0.070, 0.010, 0.100)), &opening_mat);
        opening.position().set(0, 0.055, -0.030);
        s.add(&opening);
        // Laces — 4 pairs of tiny cross cylinders down the vamp, on its sloping top.
        for i in 0..4 {
            let lace_z = 0.026 + i as f64 * 0.018;
            // Criss-cross pair lying flat, pitched with the vamp slope.
            for side in [-1.0, 1.0] {
                let lace = mesh(cyl_geo!(0.001, 0.001, 0.036, 3), &lace_mat);
                lace.rotation().z = PI / 2.0;
                lace.rotation().y = side * 0.7;
                lace.rotation().x = 0.15;
                lace.position().set(0, 0.0405 - (lace_z - 0.026) * 0.15, lace_z);
                s.add(&lace);
            }
        }
        // Small tongue.
        let tongue = mesh(pg("ShoePairs__tongue", box_geo!(0.048, 0.005, 0.030)), &shoe_mat);
        tongue.position().set(0, 0.063, -0.010);
        tongue.rotation().x = -0.3;
        s.add(&tongue);
        s
    };
    let pair = group();
    // Left shoe — upright, front-facing.
    let left_shoe = make_shoe(false);
    left_shoe.position().set(-0.06, 0, 0);
    left_shoe.rotation().y = 0.15;
    pair.add(&left_shoe);
    // Right shoe — tipped on side (Z-rotation 90°) at an angle.
    let right_shoe = make_shoe(true);
    let right_tilt = (rnd() - 0.5) * 0.3;
    right_shoe.rotation().z = PI / 2.0 + right_tilt;
    // Also flipped so sole visible.
    right_shoe.rotation().y = 0.8 + (rnd() - 0.5) * 0.4;
    // Rest the tipped shoe's lowest edge (sole or collar side) on the floor.
    right_shoe.position().set(0.10, 0.045 * right_tilt.cos() + 0f64.max(0.066 * right_tilt.sin()), 0.05);
    pair.add(&right_shoe);
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

// R703 wall coat rack w/ 2-3 hanging coats: 3% chance per level.
fn place_coat_racks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wood_mat = lambert!({ color: pick(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let rack = group();
    let mount_y = 1.75;
    // Horizontal rail — flat wide box.
    let rail_w = 0.60;
    let rail = mesh(pg("CoatRacks__rail", box_geo!(rail_w, 0.020, 0.030)), &wood_mat);
    rail.position().set(0, mount_y, 0.015);
    rack.add(&rail);
    // 5 brass peg hooks along rail.
    let mut peg_positions: Vec<f64> = vec![];
    for i in 0..5 {
        let px = -rail_w * 0.4 + i as f64 * (rail_w * 0.2);
        // Peg — small cylinder extending outward, w/ tiny knob.
        let peg = mesh(pg("CoatRacks__peg", cyl_geo!(0.004, 0.004, 0.025, 5)), &brass_mat);
        peg.rotation().x = PI / 2.0;
        peg.position().set(px, mount_y, 0.032);
        rack.add(&peg);
        let peg_knob = mesh(pg("CoatRacks__knob", sphere_geo!(0.006, 5, 4)), &brass_mat);
        peg_knob.position().set(px, mount_y - 0.003, 0.045);
        rack.add(&peg_knob);
        peg_positions.push(px);
    }
    // Coat colors — rich saturated pool.
    let coat_colors = [0x603030, 0x101020, 0x408060, 0x201010, 0xa02040, 0x203060, 0x604020, 0x2a2a2a];
    // Hang 2-3 coats on random pegs (deduplicated).
    let coat_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut used_pegs: Vec<usize> = vec![]; // a Set
    for c in 0..coat_count {
        let mut peg_idx;
        let mut attempts = 0;
        loop {
            peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            attempts += 1;
            if !(used_pegs.contains(&peg_idx) && attempts < 10) {
                break;
            }
        }
        if used_pegs.contains(&peg_idx) {
            break;
        }
        used_pegs.push(peg_idx);
        let px = peg_positions[peg_idx];
        let coat_mat = lambert!({ color: pick(&coat_colors) });
        // Coat silhouette — shoulders + body box + sleeves.
        let coat = group();
        // Shoulder wedge (small box at top).
        let shoulder = mesh(pg("CoatRacks__shoulders", box_geo!(0.13, 0.025, 0.06)), &coat_mat);
        shoulder.position().set(0, 0, 0);
        coat.add(&shoulder);
        // Body — tapered downward (taller box).
        let body = mesh(pg("CoatRacks__coat", box_geo!(0.15, 0.28, 0.05)), &coat_mat);
        body.position().set(0, -0.150, 0);
        coat.add(&body);
        // Sleeves — 2 thin box arms.
        for side in [-1.0, 1.0] {
            let sleeve = mesh(pg("CoatRacks__sleeve", box_geo!(0.035, 0.22, 0.045)), &coat_mat);
            sleeve.position().set(side * 0.080, -0.130, 0);
            sleeve.rotation().z = side * 0.15; // Slight outward tilt.
            coat.add(&sleeve);
        }
        // Optional collar strip.
        let collar_mat = lambert!({ color: 0x101010 });
        let collar = mesh(pg("CoatRacks__collar", box_geo!(0.09, 0.010, 0.045)), &collar_mat);
        collar.position().set(0, 0.012, 0.008);
        coat.add(&collar);
        coat.position().set(px, mount_y - 0.020, 0.055);
        rack.add(&coat);
        // R729 subtle sway animator — 35% chance per coat.
        if rnd() < 0.35 {
            let sway_phase = rnd() * PI * 2.0;
            let sway_phase2 = rnd() * PI * 2.0;
            let coat2 = coat.clone();
            g.animate(move |_g, _dt, t| {
                // Multi-freq sway: X-axis (front/back) + Z-axis (side/side).
                coat2.rotation().x = (t * 0.5 + sway_phase).sin() * 0.015;
                coat2.rotation().z = (t * 0.4 + sway_phase2).sin() * 0.010;
            });
        }
    }
    // Optional: scarf draped over one empty peg (40% if empty pegs exist).
    if used_pegs.len() < 5 && rnd() < 0.4 {
        let mut scarf_peg_idx;
        loop {
            scarf_peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            if !used_pegs.contains(&scarf_peg_idx) {
                break;
            }
        }
        let scarf_px = peg_positions[scarf_peg_idx];
        let scarf_colors = [0xa02040, 0x408060, 0xd0a040, 0x203060, 0xf0e0a0];
        let scarf_mat = lambert!({ color: pick(&scarf_colors) });
        // 2 hanging strips + short cross-band at top.
        for side in [-1.0, 1.0] {
            let strip = mesh(pg("CoatRacks__scarf", box_geo!(0.020, 0.24, 0.008)), &scarf_mat);
            strip.position().set(scarf_px + side * 0.010, mount_y - 0.12, 0.050);
            strip.rotation().z = side * 0.05;
            rack.add(&strip);
            // Small fringe tag at bottom of each strip.
            let fringe = mesh(pg("CoatRacks__fringe", box_geo!(0.020, 0.008, 0.006)), &scarf_mat);
            fringe.position().set(scarf_px + side * 0.010, mount_y - 0.244, 0.050);
            rack.add(&fringe);
        }
    }
    rack.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    rack.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&rack);
}

// R703 single dropped glove (floor): 8% chance per level.
fn place_dropped_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Glove color pool.
    let glove_colors = [0x101010, 0x603020, 0x201010, 0xa02040, 0x2a2a2a, 0xd0a060, 0xf0e8d0];
    let glove_mat = lambert!({ color: pick(&glove_colors) });
    let grp = group();
    // Palm/body — squashed box.
    let palm = mesh(pg("DroppedGloves__palm", box_geo!(0.070, 0.020, 0.070)), &glove_mat);
    palm.position().y = 0.010;
    grp.add(&palm);
    // 4 fingers — thin box protrusions from top edge of palm.
    let finger_lens = [0.045, 0.055, 0.052, 0.048]; // pinky, ring, middle, index
    for i in 0..4 {
        let finger = mesh(pg("DroppedGloves__finger", box_geo!(0.014, 0.014, finger_lens[i])), &glove_mat);
        // Position at top of palm, spread across width.
        finger.position().set(-0.024 + i as f64 * 0.016, 0.008, 0.035 + finger_lens[i] / 2.0);
        // Slight bend — random tilt for crumpled look.
        finger.rotation().x = -0.15 + (rnd() - 0.5) * 0.15;
        finger.rotation().y = (rnd() - 0.5) * 0.15;
        grp.add(&finger);
    }
    // Thumb — sticking off to side.
    let thumb = mesh(pg("DroppedGloves__thumb", box_geo!(0.014, 0.014, 0.040)), &glove_mat);
    thumb.position().set(0.038, 0.008, 0.005);
    thumb.rotation().y = 0.8;
    thumb.rotation().x = -0.2;
    grp.add(&thumb);
    // Cuff — slight raised band at wrist.
    let cuff = mesh(pg("DroppedGloves__cuff", box_geo!(0.076, 0.018, 0.024)), &glove_mat);
    cuff.position().set(0, 0.009, -0.043);
    grp.add(&cuff);
    // Optional decorative band on cuff (40% chance — stripes or trim).
    if rnd() < 0.4 {
        let trim_colors = [0xffffff, 0xd0a040, 0xa02040, 0x2060a0];
        let trim_mat = lambert!({ color: pick(&trim_colors) });
        let trim = mesh(pg("DroppedGloves__trim", box_geo!(0.078, 0.004, 0.008)), &trim_mat);
        trim.position().set(0, 0.017, -0.043);
        grp.add(&trim);
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    // Random Z-tilt for crumpled/tossed feel.
    grp.rotation().z = (rnd() - 0.5) * 0.3;
    g.world.add(&grp);
}

// R704 laundry basket w/ folded clothes (floor): 4% chance per level.
fn place_laundry_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Basket color pool — plastic or wicker.
    let basket_colors = [0xffffff, 0xa07040, 0xf0a0c0, 0x506080, 0xc0c0c8, 0x60a060];
    let basket_mat = lambert!({ color: pick(&basket_colors) });
    let lb = group();
    // Basket body — wide open cylinder.
    let (b_r, b_h) = (0.14, 0.16);
    let body = mesh(pg("LaundryBaskets__body", cyl_geo!(b_r, b_r * 0.85, b_h, 12, 1, true)), &basket_mat);
    body.position().y = b_h / 2.0 + 0.002;
    lb.add(&body);
    // Bottom disc.
    let bottom = mesh(circle_geo!(b_r * 0.85, 12), &basket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    lb.add(&bottom);
    // Rim torus.
    let rim = mesh(pg("LaundryBaskets__rim", torus_geo!(b_r, 0.008, 5, 14)), &basket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = b_h + 0.002;
    lb.add(&rim);
    // 2 side handles (partial torus arches on opposite sides).
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("LaundryBaskets__handle", torus_geo!(0.030, 0.006, 4, 6, PI)), &basket_mat);
        // Arch stands up on the rim in the wall's tangent plane (YZ), feet in the rim.
        handle.rotation().y = PI / 2.0;
        handle.position().set(side * b_r, b_h + 0.002, 0);
        lb.add(&handle);
    }
    // Woven texture hints — 8 vertical thin slats around body.
    let slat_mat = lambert!({ color: 0x0a0a0a, transparent: true, opacity: 0.15 });
    for i in 0..8 {
        let ang = (i as f64 / 8.0) * PI * 2.0;
        let slat = mesh(box_geo!(0.006, b_h * 0.85, 0.001), &slat_mat);
        slat.position().set(ang.cos() * b_r * 0.95, b_h / 2.0 + 0.002, ang.sin() * b_r * 0.95);
        slat.rotation().y = ang + PI / 2.0;
        lb.add(&slat);
    }
    // Stack of folded clothes inside — 3-5 layered rectangles in different colors.
    let cloth_colors = [0xffffff, 0xf0d0d0, 0xd0e0f0, 0xf0e0a0, 0xa0d0a0, 0xe0c0f0, 0x606080];
    let cloth_count = 3 + (rnd() * 3.0).floor() as i32;
    for i in 0..cloth_count {
        let cloth_mat = lambert!({ color: pick(&cloth_colors) });
        let cloth_h = 0.014 + rnd() * 0.006;
        let cw = b_r * 1.5 * (0.85 + rnd() * 0.15);
        let cd = b_r * 1.5 * (0.75 + rnd() * 0.20);
        let cloth = mesh(pg("LaundryBaskets__cloth", box_geo!(cw, cloth_h, cd)), &cloth_mat);
        cloth.position().set((rnd() - 0.5) * 0.01, b_h * 0.60 + i as f64 * (cloth_h + 0.001), (rnd() - 0.5) * 0.01);
        cloth.rotation().y = (rnd() - 0.5) * 0.15;
        lb.add(&cloth);
    }
    // Optional draped item hanging over rim — 40%.
    if rnd() < 0.4 {
        let drape_mat = lambert!({ color: pick(&cloth_colors) });
        // Simple L-shape: horizontal segment on rim + vertical segment hanging down.
        let drape_h = mesh(pg("LaundryBaskets__drapeTop", box_geo!(0.07, 0.006, 0.06)), &drape_mat);
        // Lies across the rim (its underside on the rim top at x = b_r).
        drape_h.position().set(b_r - 0.01, b_h + 0.015, 0);
        drape_h.rotation().z = -0.2;
        lb.add(&drape_h);
        let drape_v = mesh(pg("LaundryBaskets__drapeHang", box_geo!(0.06, 0.10, 0.005)), &drape_mat);
        // Hangs outside the rim (rim outer edge b_r + 0.008), top tucked under drape_h.
        drape_v.position().set(b_r + 0.011, b_h - 0.04, 0);
        drape_v.rotation().y = PI / 2.0; // face outward, thin side against the basket
        drape_v.rotation().z = 0.05;
        lb.add(&drape_v);
    }
    lb.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    lb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lb);
}

// R705 ironing board upright w/ iron (floor): 3% chance per level.
fn place_ironing_boards(g: &mut Game, _dist: &[i32]) {
    g.pp.hot_iron_positions = vec![]; // R712 reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Board cover pool — pastels + neutrals.
    let cover_colors = [0xf0d0d0, 0xd0e0f0, 0xf0e8d0, 0xa0c0a0, 0xc0c0c8];
    let cover_mat = lambert!({ color: pick(&cover_colors) });
    let leg_mat = lambert!({ color: 0x505050 });
    let iron_colors = [0x101010, 0xa02020, 0x2060a0, 0xc0c0c8];
    let iron_mat = lambert!({ color: pick(&iron_colors) });
    let cord_mat = lambert!({ color: 0x0a0a0a });
    let ib = group();
    // Board — 2 boxes + rounded ends via half cylinders.
    let board_h = 0.85;
    let body_main = mesh(pg("IroningBoards__board", box_geo!(0.32, 0.020, 0.16)), &cover_mat);
    body_main.position().set(0, board_h, 0);
    ib.add(&body_main);
    // Pointed nose end (extends beyond).
    let nose = mesh(pg("IroningBoards__nose", cyl_geo!(0.075, 0.075, 0.020, 8, 1, false, 0, PI)), &cover_mat);
    // Unrotated, the theta 0..PI half-disc already lies flat on the +X side.
    nose.position().set(0.16, board_h, 0);
    ib.add(&nose);
    // Rounded broad end.
    let back_end = mesh(pg("IroningBoards__backEnd", cyl_geo!(0.08, 0.08, 0.020, 8, 1, false, PI, PI)), &cover_mat);
    // Unrotated, the theta PI..2PI half-disc lies flat on the -X side.
    back_end.position().set(-0.16, board_h, 0);
    ib.add(&back_end);
    // 4 folding metal legs — 2 pairs forming X shape (seen from the side).
    for i in 0..2 {
        let mut side = -1;
        while side <= 1 {
            let leg = mesh(pg("IroningBoards__leg", cyl_geo!(0.006, 0.006, board_h * 1.02, 5)), &leg_mat);
            let off = if i == 0 { -0.06 } else { 0.06 };
            leg.position().set(side as f64 * 0.03, 0.42, off);
            leg.rotation().z = side as f64 * 0.22f64.atan2(0.84);
            ib.add(&leg);
            side += 2;
        }
    }
    // Pivot rod through the X crossing, linking the two rails.
    let brace = mesh(pg("IroningBoards__brace", cyl_geo!(0.004, 0.004, 0.30, 5)), &leg_mat);
    brace.rotation().x = PI / 2.0;
    brace.scale().y = 0.44;
    brace.position().set(0, 0.5345, 0);
    ib.add(&brace);
    // Rubber feet on floor (4 small dark caps).
    for ox in [-0.14, 0.14] {
        for oz in [-0.06, 0.06] {
            let foot = mesh(pg("IroningBoards__foot", cyl_geo!(0.008, 0.010, 0.006, 5)), &cord_mat);
            foot.position().set(ox, 0.003, oz);
            ib.add(&foot);
        }
    }
    // Iron resting on the broad end.
    let iron = group();
    // Iron body — flat base + upright body.
    let iron_base = mesh(pg("IroningBoards__ironBase", box_geo!(0.09, 0.010, 0.06)), &iron_mat);
    iron_base.position().y = 0.005;
    iron.add(&iron_base);
    // Nose taper (pointed tip), lying flat: axis along X, flat cut face down.
    let iron_nose = mesh(pg("IroningBoards__ironNose", cyl_geo!(0.032, 0.006, 0.030, 6, 1, false, 0, PI)), &iron_mat);
    iron_nose.rotation().z = PI / 2.0;
    iron_nose.position().set(0.040, 0, 0);
    iron.add(&iron_nose);
    // Upper body — smaller upper block.
    let iron_upper = mesh(pg("IroningBoards__ironUpper", box_geo!(0.07, 0.020, 0.05)), &iron_mat);
    iron_upper.position().set(-0.01, 0.020, 0);
    iron.add(&iron_upper);
    // Handle — curved arch (partial torus), feet on the housing top.
    let handle_arch = mesh(pg("IroningBoards__ironHandle", torus_geo!(0.024, 0.006, 4, 8, PI)), &iron_mat);
    handle_arch.position().set(-0.010, 0.030, 0);
    iron.add(&handle_arch);
    // Temperature dial (small cylinder on side).
    let dial_mat = lambert!({ color: 0xd0d0d0 });
    let dial = mesh(pg("IroningBoards__dial", cyl_geo!(0.008, 0.008, 0.003, 8)), &dial_mat);
    dial.rotation().x = PI / 2.0;
    dial.position().set(-0.030, 0.020, 0.028);
    iron.add(&dial);
    // Cord attachment point (small nub at back).
    let nub = mesh(pg("IroningBoards__cordBoot", cyl_geo!(0.005, 0.005, 0.010, 5)), &cord_mat);
    nub.rotation().z = PI / 2.0; // points out of the back (-X)
    nub.position().set(-0.045, 0.030, 0);
    iron.add(&nub);
    iron.position().set(-0.10, board_h + 0.010, 0);
    iron.rotation().y = PI * 0.25;
    ib.add(&iron);
    // R709 subtle steam-vent wisp — 40% chance (iron is "hot").
    // R712: track hot irons for steam-hiss audio pairing.
    let iron_is_hot = rnd() < 0.4;
    if iron_is_hot {
        let steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.15 });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
            base_x: f64,
            base_z: f64,
        }
        let mut wisps: Vec<Wisp> = vec![];
        // 2 small wisps rising from iron top.
        for i in 0..2 {
            let wisp = mesh(sphere_geo!(0.010, 6, 5), &steam_mat);
            wisp.scale().y = 2.0;
            let iry = iron.rot().y;
            let wisp_x = -0.10 + iry.cos() * (-0.02 + rnd() * 0.04);
            let wisp_z = iry.sin() * (-0.02 + rnd() * 0.04);
            wisp.position().set(wisp_x, board_h + 0.070 + i as f64 * 0.025, wisp_z);
            let p = wisp.pos();
            wisps.push(Wisp { mesh: wisp.clone(), phase: rnd() * PI * 2.0, base_y: p.y, base_x: p.x, base_z: p.z });
            ib.add(&wisp);
        }
        g.animate(move |_g, _dt, t| {
            for w in &wisps {
                {
                    let mut p = w.mesh.position();
                    p.y = w.base_y + (t * 0.5 + w.phase).sin() * 0.020 + 0.010;
                    p.x = w.base_x + (t * 0.4 + w.phase).sin() * 0.010;
                    p.z = w.base_z + (t * 0.6 + w.phase).cos() * 0.010;
                }
                let s = 0.85 + (t * 0.7 + w.phase).sin() * 0.18;
                w.mesh.scale().set(s, 2.0 * s, s);
            }
        });
    }
    // Cord snake — 8-segment curve from iron nub trailing to floor.
    // Starts at the cord boot's back end (iron-local (-0.05, 0.03, 0), yawed PI/4).
    let cord_start = v3(-0.135, board_h + 0.040, 0.035);
    let cord_end = v3(-0.35, 0.005, 0.15);
    let cord_segs = 8;
    let mut prev = cord_start;
    for i in 0..cord_segs {
        let t = (i + 1) as f64 / cord_segs as f64;
        let arc_amt = (t * PI).sin() * 0.03;
        let cx = cord_start.x + (cord_end.x - cord_start.x) * t + (t * PI * 2.0).sin() * arc_amt;
        let cy = cord_start.y + (cord_end.y - cord_start.y) * t + (t * PI * 1.5).sin().abs() * 0.01;
        let cz = cord_start.z + (cord_end.z - cord_start.z) * t + (t * PI * 2.0).cos() * arc_amt * 0.7;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("IroningBoards__cord", cyl_geo!(0.003, 0.003, dir_len, 4)), &cord_mat);
            orient_seg(&seg, &prev, &next);
            ib.add(&seg);
        }
        prev = next;
    }
    // 50% chance: folded garment draped over one end.
    if rnd() < 0.5 {
        let shirt_colors = [0xffffff, 0xd0e0f0, 0xf0d0d0, 0xf0e0a0, 0x606080];
        let shirt_mat = lambert!({ color: pick(&shirt_colors) });
        let shirt = mesh(pg("IroningBoards__shirt", box_geo!(0.14, 0.010, 0.10)), &shirt_mat);
        shirt.position().set(0.05, board_h + 0.015, 0);
        shirt.rotation().z = 0.05;
        ib.add(&shirt);
    }
    ib.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    ib.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ib);
    // R712 register position only for hot irons (with steam wisps).
    if iron_is_hot {
        let p = ib.pos();
        g.pp.hot_iron_positions.push(v3(p.x, 0, p.z));
    }
}

// R705 vacuum cleaner unattended (floor): 3% chance per level.
// R706: track positions for optional motor whine audio.
fn place_vacuum_cleaners(g: &mut Game, _dist: &[i32]) {
    g.pp.vacuum_positions = vec![]; // R706 reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let body_colors = [0xa02040, 0x2060a0, 0x606060, 0x101010, 0xd0a040, 0x40a060];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let gray_mat = lambert!({ color: 0x606060 });
    let dark_mat = lambert!({ color: 0x1a1a1a });
    let cord_mat = lambert!({ color: 0x0a0a0a });
    let dust_mat = lambert!({ color: 0xe0d8c0, transparent: true, opacity: 0.35 });
    let v = group();
    // Base — wide flat pad w/ brush roller line.
    let (base_w, base_h, base_d) = (0.32, 0.045, 0.20);
    let base = mesh(pg("VacuumCleaners__base", box_geo!(base_w, base_h, base_d)), &body_mat);
    base.position().y = base_h / 2.0 + 0.002;
    v.add(&base);
    // Brush roller (dark cylinder on underside front).
    let roller = mesh(pg("VacuumCleaners__roller", cyl_geo!(0.020, 0.020, base_w * 0.9, 6)), &dark_mat);
    roller.rotation().z = PI / 2.0;
    roller.position().set(0, 0.020, base_d * 0.40);
    v.add(&roller);
    // 2 rear wheels, just outside the base's side faces.
    for side in [-1.0, 1.0] {
        let wheel = mesh(pg("VacuumCleaners__wheel", cyl_geo!(0.024, 0.024, 0.010, 8)), &dark_mat);
        wheel.rotation().z = PI / 2.0;
        wheel.position().set(side * (base_w / 2.0 + 0.005), 0.024, -base_d * 0.30);
        v.add(&wheel);
    }
    // Vertical body — canister rising up.
    let can_h = 0.42;
    let canister = mesh(pg("VacuumCleaners__canister", box_geo!(0.16, can_h, 0.15)), &body_mat);
    canister.position().set(-0.02, base_h + can_h / 2.0 + 0.002, -base_d * 0.15);
    canister.rotation().x = -0.20; // Tilted back slightly.
    v.add(&canister);
    // Dust chamber (clear-ish window) — transparent box.
    let dust_cham = mesh(box_geo!(0.12, 0.16, 0.11), &dust_mat);
    dust_cham.position().set(-0.005, base_h + can_h * 0.30, -base_d * 0.12);
    dust_cham.rotation().x = -0.20;
    v.add(&dust_cham);
    // Handle — long thin cylinder rising up.
    let handle_len = 0.75;
    let handle = mesh(pg("VacuumCleaners__wand", cyl_geo!(0.014, 0.014, handle_len, 6)), &gray_mat);
    handle.rotation().x = -0.20;
    handle.position().set(-0.02, base_h + can_h + handle_len * 0.42, -base_d * 0.15 - 0.06);
    v.add(&handle);
    // Grip on handle — larger cylinder near top.
    let grip = mesh(pg("VacuumCleaners__grip", cyl_geo!(0.020, 0.020, 0.080, 6)), &dark_mat);
    grip.rotation().x = -0.20;
    grip.position().set(-0.02, base_h + can_h + handle_len * 0.90, -base_d * 0.15 - 0.14);
    v.add(&grip);
    // Power switch button (bright color on grip).
    let btn_mat = lambert!({ color: 0xa02020 });
    let btn = mesh(pg("VacuumCleaners__button", box_geo!(0.006, 0.010, 0.008)), &btn_mat);
    // On the wand's +X surface (axis x -0.02, r 0.014), following its tilt.
    btn.position().set(-0.004, base_h + can_h + handle_len * 0.75, -base_d * 0.15 - 0.110);
    btn.rotation().x = -0.20;
    v.add(&btn);
    // Cord — 10-segment curve trailing away to floor + plug.
    let cord_start = v3(-0.04, base_h + can_h + handle_len * 0.05, -base_d * 0.15 + 0.03);
    let cord_end = v3(-0.35 - rnd() * 0.15, 0.005, 0.15 + rnd() * 0.10);
    let cord_segs = 10;
    let mut prev = cord_start;
    for i in 0..cord_segs {
        let t = (i + 1) as f64 / cord_segs as f64;
        let arc_amt = (t * PI).sin() * 0.04;
        let cx = cord_start.x + (cord_end.x - cord_start.x) * t + (t * PI * 2.0).sin() * arc_amt;
        let cy = cord_start.y + (cord_end.y - cord_start.y) * t + (t * PI * 1.2).sin().abs() * 0.012;
        let cz = cord_start.z + (cord_end.z - cord_start.z) * t + (t * PI * 2.0).cos() * arc_amt * 0.6;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("IroningBoards__cord", cyl_geo!(0.003, 0.003, dir_len, 4)), &cord_mat);
            orient_seg(&seg, &prev, &next);
            v.add(&seg);
        }
        prev = next;
    }
    // Plug at cord end (dark box + prongs).
    let plug = mesh(pg("VacuumCleaners__plug", box_geo!(0.020, 0.010, 0.014)), &cord_mat);
    plug.position().copy(&cord_end);
    plug.position().y = 0.005;
    v.add(&plug);
    let prong_mat = lambert!({ color: 0x808080 });
    for side in [-1.0, 1.0] {
        let prong = mesh(pg("VacuumCleaners__prong", box_geo!(0.010, 0.002, 0.003)), &prong_mat);
        prong.position().set(cord_end.x + 0.010, 0.005, cord_end.z + side * 0.003);
        v.add(&prong);
    }
    v.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    v.rotation().y = rnd() * PI * 2.0;
    g.world.add(&v);
    // R706 register position for motor whine audio.
    let p = v.pos();
    g.pp.vacuum_positions.push(v3(p.x, 0, p.z));
}

// R706 mop + bucket w/ water puddle (floor): 4% chance per level.
fn place_cleaning_mop_buckets(g: &mut Game, _dist: &[i32]) {
    g.pp.mop_bucket_positions = vec![]; // R707 reset per level
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Bucket color pool.
    let bucket_colors = [0xa02020, 0x2060a0, 0xd0a040, 0x606060, 0x808080];
    let bucket_mat = lambert!({ color: pick(&bucket_colors) });
    // Mop handle color pool.
    let handle_colors = [0x8b5a2b, 0x604020, 0x505050, 0xa07040];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    // Mop head color — dirty/wet gray or off-white.
    let mop_head_colors = [0xc0b8a0, 0xa09888, 0xe0d8c8, 0x808070];
    let mop_head_mat = lambert!({ color: pick(&mop_head_colors) });
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let mb = group();
    // Bucket — trapezoidal (tapered cylinder).
    let bucket_h = 0.20;
    let (bucket_r_top, bucket_r_bot) = (0.13, 0.11);
    let bucket = mesh(pg("CleaningMopBuckets__bucket", cyl_geo!(bucket_r_top, bucket_r_bot, bucket_h, 12, 1, true)), &bucket_mat);
    bucket.position().y = bucket_h / 2.0 + 0.002;
    mb.add(&bucket);
    // Bottom disc.
    let bottom = mesh(circle_geo!(bucket_r_bot, 12), &bucket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    mb.add(&bottom);
    // Rim torus.
    let rim = mesh(pg("CleaningMopBuckets__rim", torus_geo!(bucket_r_top, 0.006, 4, 14)), &bucket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = bucket_h + 0.002;
    mb.add(&rim);
    // Metal handle arch — semi-torus arching over top, ends on the side nubs.
    let bucket_handle = mesh(pg("CleaningMopBuckets__bail", torus_geo!(bucket_r_top * 1.02, 0.004, 3, 10, PI)), &metal_mat);
    bucket_handle.rotation().z = 0.0;
    bucket_handle.position().y = bucket_h * 0.90;
    mb.add(&bucket_handle);
    // Handle-attach pivots (small nubs at bucket rim sides).
    for side in [-1.0, 1.0] {
        let nub = mesh(pg("CleaningMopBuckets__nub", sphere_geo!(0.005, 5, 4)), &metal_mat);
        nub.position().set(side * bucket_r_top, bucket_h * 0.90, 0);
        mb.add(&nub);
    }
    // Water inside bucket — dark surface disc slightly below rim.
    if rnd() < 0.7 {
        let water_mat = basic_mat!({ color: 0x304060, transparent: true, opacity: 0.75 });
        let water = mesh(circle_geo!(bucket_r_top * 0.92, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.position().y = bucket_h * 0.72;
        mb.add(&water);
    }
    // Mop — long handle leaning at an angle, foot in the head down in the bucket.
    let mop_handle_len = 1.10;
    let lean_angle: f64 = -0.35;
    let mop_handle = mesh(pg("CleaningMopBuckets__stick", cyl_geo!(0.010, 0.010, mop_handle_len, 6)), &handle_mat);
    // rotation.z = lean_angle leans the top toward +x, so the end points follow that axis.
    mop_handle.rotation().z = lean_angle;
    let mop_bottom_x = 0.0;
    let mop_bottom_y = bucket_h * 0.6;
    let mop_top_x = mop_bottom_x - lean_angle.sin() * mop_handle_len;
    let mop_top_y = mop_bottom_y + lean_angle.cos() * mop_handle_len;
    mop_handle.position().set((mop_bottom_x + mop_top_x) / 2.0, (mop_bottom_y + mop_top_y) / 2.0, 0);
    mb.add(&mop_handle);
    // Grip band at top of handle (small colored torus), ring axis along the stick.
    let grip_mat = lambert!({ color: 0xa02040 });
    let grip = mesh(pg("CleaningMopBuckets__grip", torus_geo!(0.011, 0.003, 3, 8)), &grip_mat);
    grip.rotation().y = -lean_angle;
    grip.rotation().x = -PI / 2.0;
    grip.position().set(mop_top_x + lean_angle.sin() * 0.05, mop_top_y - lean_angle.cos() * 0.05, 0);
    mb.add(&grip);
    // Mop head — bunch of thin strands radiating from bottom (inside bucket); the
    // clamp (centre mop_head_y + 0.02) takes the stick's foot.
    let mop_head_x = mop_bottom_x;
    let mop_head_y = mop_bottom_y - 0.025; // Inside bucket.
    let mop_head_z = 0.0;
    // 12 strands radiating out + down.
    let strand_count = 12;
    for i in 0..strand_count {
        let ang = (i as f64 / strand_count as f64) * PI * 2.0;
        let strand_len = 0.06 + rnd() * 0.04;
        let strand = mesh(pg("CleaningMopBuckets__strand", cyl_geo!(0.002, 0.001, strand_len, 3)), &mop_head_mat);
        // Angle downward + outward.
        strand.rotation().x = ang.sin() * 0.6;
        strand.rotation().z = ang.cos() * 0.6;
        // Position at mop head center.
        strand.position().set(mop_head_x, mop_head_y, mop_head_z);
        // Offset along direction.
        let dx = ang.cos() * strand_len * 0.35;
        let dz = ang.sin() * strand_len * 0.35;
        strand.position().set(mop_head_x + dx, mop_head_y - strand_len * 0.35, mop_head_z + dz);
        mb.add(&strand);
    }
    // Metal band clamping strands to handle.
    let clamp2 = mesh(pg("CleaningMopBuckets__clamp", cyl_geo!(0.014, 0.014, 0.020, 8)), &metal_mat);
    clamp2.position().set(mop_head_x, mop_head_y + 0.020, mop_head_z);
    mb.add(&clamp2);
    // 60% chance: water puddle on floor beside bucket.
    if rnd() < 0.6 {
        let puddle_mat = basic_mat!({ color: 0x304060, transparent: true, opacity: 0.55 });
        let puddle = mesh(circle_geo!(0.11, 14), &puddle_mat);
        puddle.rotation().x = -PI / 2.0;
        puddle.scale().set(1.4, 1.0, 0.9);
        puddle.position().set(-bucket_r_top - 0.05, 0.001, 0.04);
        puddle.rotation().z = rnd() * PI * 2.0;
        mb.add(&puddle);
    }
    mb.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    mb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mb);
    // R707 register mop bucket position for water-slosh audio.
    let p = mb.pos();
    g.pp.mop_bucket_positions.push(v3(p.x, 0, p.z));
}

// R708 broom + dustpan pair (floor): 5% chance per level.
fn place_brooms_and_dustpans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Broom handle color pool.
    let handle_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x505050];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    // Bristle color pool — natural straw or synthetic.
    let bristle_colors = [0xa07040, 0xc0a060, 0x805030, 0x808080, 0x1a1a1a];
    let bristle_mat = lambert!({ color: pick(&bristle_colors) });
    // Dustpan color.
    let pan_colors = [0xa02020, 0x2060a0, 0x606060, 0xd0a040, 0x101010];
    let pan_mat = lambert!({ color: pick(&pan_colors) });
    let bd = group();
    // Broom lying on floor — handle horizontal + bristle head at end.
    let broom_len = 0.90;
    let broom_handle = mesh(pg("BroomsAndDustpans__handle", cyl_geo!(0.010, 0.010, broom_len, 6)), &handle_mat);
    broom_handle.rotation().z = PI / 2.0;
    // Centred on the head's height (0..0.02) so it enters the socket boss.
    broom_handle.position().set(0, 0.010, 0);
    bd.add(&broom_handle);
    // Broom head — flat rectangular base + many bristles below.
    let head_base = mesh(pg("BroomsAndDustpans__head", box_geo!(0.14, 0.020, 0.05)), &handle_mat);
    head_base.position().set(broom_len * 0.5 - 0.02, 0.010, 0);
    bd.add(&head_base);
    // Bristle strands — 16 thin cylinders angling out radially.
    let strand_count = 16;
    for i in 0..strand_count {
        let t = i as f64 / (strand_count - 1) as f64;
        let bristle_len = 0.06 + rnd() * 0.02;
        let bristle = mesh(cyl_geo!(0.001, 0.0008, bristle_len, 3), &bristle_mat);
        // Position along head-base + slight angular spread.
        bristle.position().set(broom_len * 0.5 - 0.02 + (t - 0.5) * 0.12, 0.010 + bristle_len * 0.35, (rnd() - 0.5) * 0.03);
        bristle.rotation().x = (rnd() - 0.5) * 0.15;
        bristle.rotation().z = 0.15 + rnd() * 0.15;
        bd.add(&bristle);
    }
    // Small metal grip band where head meets handle.
    let grip_band = mesh(pg("BroomsAndDustpans__band", torus_geo!(0.012, 0.002, 3, 8)), lambert!({ color: 0xa0a0a8 }));
    grip_band.rotation().y = PI / 2.0;
    grip_band.position().set(broom_len * 0.42, 0.010, 0);
    bd.add(&grip_band);
    // Dustpan — lying near broom head end.
    let pan = group();
    // Base (flat rectangle).
    let pan_base = mesh(pg("BroomsAndDustpans__panFloor", box_geo!(0.16, 0.006, 0.11)), &pan_mat);
    pan_base.position().y = 0.003;
    pan.add(&pan_base);
    // Raised back edge (front is open for sweeping into it).
    let pan_back = mesh(pg("BroomsAndDustpans__panBack", box_geo!(0.16, 0.030, 0.008)), &pan_mat);
    pan_back.position().set(0, 0.018, -0.051);
    pan.add(&pan_back);
    // Side walls (short low boxes).
    for side in [-1.0, 1.0] {
        let wall = mesh(pg("BroomsAndDustpans__panSide", box_geo!(0.006, 0.024, 0.11)), &pan_mat);
        wall.position().set(side * 0.077, 0.015, 0);
        wall.rotation().z = 0.0;
        pan.add(&wall);
    }
    // Angled front-lip flap: the modelled lip is already a wedge, flat on the
    // floor, thinned so its back meets the pan floor top.
    let lip = mesh(pg("BroomsAndDustpans__panLip", box_geo!(0.14, 0.014, 0.03)), &pan_mat);
    lip.scale().y = 0.45;
    lip.position().set(0, 0.0032, 0.062);
    pan.add(&lip);
    // Handle from back edge angling up: its foot is in the back wall's boss,
    // the hook loop hangs off its top end.
    let pan_handle_len = 0.10;
    let pan_handle_dir = v3(0, 0.5f64.cos(), -0.5f64.sin());
    let pan_handle_foot = v3(0, 0.020, -0.053);
    let pan_handle = mesh(pg("BroomsAndDustpans__panHandle", cyl_geo!(0.006, 0.006, pan_handle_len, 5)), &pan_mat);
    pan_handle.rotation().x = -0.5;
    pan_handle.position().copy(&pan_handle_foot).add_scaled_vector(&pan_handle_dir, pan_handle_len / 2.0);
    pan.add(&pan_handle);
    // Hook loop at handle end.
    let hook_loop = mesh(pg("BroomsAndDustpans__hookLoop", torus_geo!(0.006, 0.001, 3, 6)), &pan_mat);
    hook_loop.rotation().x = -0.5;
    hook_loop.position().copy(&pan_handle_foot).add_scaled_vector(&pan_handle_dir, pan_handle_len + 0.005);
    pan.add(&hook_loop);
    // 65% chance: dust pile inside pan.
    if rnd() < 0.65 {
        let dust_mat = lambert!({ color: 0x605040 });
        // Small elongated pile in the middle of the pan.
        let dust = mesh(pg("BroomsAndDustpans__dust", sphere_geo!(0.020, 6, 4)), &dust_mat);
        dust.scale().set(1.4, 0.4, 1.0);
        dust.position().set(-0.010 + (rnd() - 0.5) * 0.020, 0.011, 0);
        pan.add(&dust);
        // A couple tiny dust chunks nearby.
        for i in 0..3 {
            let chunk = mesh(pg("BroomsAndDustpans__crumb", sphere_geo!(0.004, 4, 3)), &dust_mat);
            chunk.position().set(-0.02 + rnd() * 0.06, 0.008, -0.02 + rnd() * 0.04);
            pan.add(&chunk);
        }
    }
    pan.position().set(broom_len * 0.20, 0, 0.14);
    pan.rotation().y = -0.4;
    bd.add(&pan);
    bd.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    bd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bd);
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
