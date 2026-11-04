//! Prop placers: car keys, birth announcements, wallets, postcards, shoe pairs,
//! coat racks, dropped gloves, and laundry / cleaning props (laundry baskets,
//! ironing boards, vacuum cleaners, mop buckets, brooms and dustpans).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

mod t07;
use t07::{place_coat_racks, place_vacuum_cleaners};

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

// R702 shoe pair (floor): 6% chance per level. A pair of men's derby shoes
// kicked off and left: the left one standing, the right one fallen on its
// side (Blender batch r14a: welted, the quarters lapped over the vamp,
// creased where the foot bent it, a leather sole on a stacked heel).
fn place_shoe_pairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old colours stand for the leathers a pair of derbies comes in:
    // mid brown, black, tan, dark brown, oxblood, bone gone grey, navy;
    // (leather, lining, sole's edge, laces, eyelets)
    let shoe_colors = [0x603020, 0x101010, 0x8b5a2b, 0x201010, 0xa02040, 0xf0e0d0, 0x2060a0];
    let shoe_color = pick(&shoe_colors);
    let cols = match shoe_color {
        0x603020 => (0x5a3a26, 0x3e3026, 0x2c241e, 0x2e241c, 0x3a3630),
        0x101010 => (0x2a2826, 0x2e2a26, 0x221f1d, 0x1e1c1a, 0x3a3630),
        0x8b5a2b => (0x86603e, 0x5a4a3a, 0x3a2c22, 0x3a2c22, 0x5a4e3e),
        0x201010 => (0x3c2622, 0x30262a, 0x241e1c, 0x2a221e, 0x3a3630),
        0xa02040 => (0x5c2a2a, 0x3a2c28, 0x2a221e, 0x2a221e, 0x4a4036),
        0xf0e0d0 => (0xaaa08c, 0x6a6052, 0x3a342e, 0xa8a090, 0x6a6458),
        _ => (0x2e3646, 0x2c2c30, 0x262422, 0x22242a, 0x3a3832),
    };
    // the right one's old tilt now turns it a little more off the left
    let right_tilt = (rnd() - 0.5) * 0.3;
    let right_yaw = 0.8 + (rnd() - 0.5) * 0.4 + right_tilt * 0.5;
    let (px, pz) = (g.wx(x) + (rnd() - 0.5) * 1.8, g.wz(z) + (rnd() - 0.5) * 1.8);
    let yaw = rnd() * PI * 2.0;
    let vary = ((px * 7919.0).abs() as u32) ^ ((pz * 104_729.0).abs() as u32).rotate_left(9);
    let pair = group();
    // the left one standing, turned a little
    let left = r14a_footwear("shoe", "L", cols, vary);
    left.position().set(-0.14, 0, -0.01);
    left.rotation().y = 0.15;
    pair.add(&left);
    // the right one fallen on its outer side, as it lies (modelled so)
    let right = r14a_footwear("shoe", "Side", cols, vary ^ 0x1f3d);
    right.position().set(0.11, 0, 0.05);
    right.rotation().y = right_yaw;
    pair.add(&right);
    pair.position().set(px, 0, pz);
    pair.rotation().y = yaw;
    g.world.add(&pair);
}

// R703 wall coat rack: batch `t07` (t07.rs).

// R703 single dropped glove (floor): 8% chance per level. One glove dropped
// and left, lying limp (Blender batch r14a, the glove of
// `place_single_gloves` too): a lined leather dress glove, its three points
// stitched on the back, or a knitted one; 40% with a stripe knitted round
// the wrist.
fn place_dropped_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old colours stand for the gloves people drop: black, brown and
    // dark brown leather, maroon and charcoal knits, camel leather, an
    // oatmeal knit
    let glove_colors = [0x101010, 0x603020, 0x201010, 0xa02040, 0x2a2a2a, 0xd0a060, 0xf0e8d0];
    let col = pick(&glove_colors);
    // the old fingers' tilts (two draws each) now vary how it is worn
    let mut vary = 0u32;
    for _ in 0..4 {
        let a = rnd();
        let b = rnd();
        vary = vary.rotate_left(7) ^ ((a * 65535.0) as u32) ^ (((b * 65535.0) as u32) << 16);
    }
    // 40%: a stripe knitted round the wrist
    let stripe = if rnd() < 0.4 {
        let trim_colors = [0xffffff, 0xd0a040, 0xa02040, 0x2060a0];
        Some(match pick(&trim_colors) {
            0xffffff => 0xd2ccbe,
            0xd0a040 => 0xa88a48,
            0xa02040 => 0x7e3036,
            _ => 0x3a5274,
        })
    } else {
        None
    };
    let (px, pz) = (g.wx(x) + (rnd() - 0.5) * 2.4, g.wz(z) + (rnd() - 0.5) * 2.4);
    let yaw = rnd() * PI * 2.0;
    let _ = rnd(); // keeps the old tilt's draw: the glove lies flat
    let (cloth, base) = match col {
        0x101010 => (R14aCloth::Leather, 0x2c2a28),
        0x603020 => (R14aCloth::Leather, 0x5a3c2a),
        0x201010 => (R14aCloth::Leather, 0x3a2622),
        0xa02040 => (R14aCloth::Knit, 0x6a2c34),
        0x2a2a2a => (R14aCloth::Knit, 0x403e3c),
        0xd0a060 => (R14aCloth::Leather, 0xa0825c),
        _ => (R14aCloth::Knit, 0xb4ab9a),
    };
    let leather = cloth == R14aCloth::Leather;
    let (r, gg, b) = r14a_rgb(base);
    let darker = (((r * 0.92) as u32) << 16) | (((gg * 0.92) as u32) << 8) | ((b * 0.92) as u32);
    let spec = R14aGlove {
        back: (cloth, base),
        tips: None,
        // a leather glove's wrist hemmed, or (with a stripe) a knitted wristlet
        cuff: (if leather && stripe.is_none() { R14aCloth::Leather } else { R14aCloth::Knit }, darker),
        stripe,
        under: darker,
        points: leather,
    };
    let grp = r14a_glove(&spec, vary);
    grp.position().set(px, 0, pz);
    grp.rotation().y = yaw;
    g.world.add(&grp);
}

// ---- batch r14a: things worn and dropped (gloves, shoes, boots) ----------

/// The part `name` of a Blender batch, as modelled, added to `grp` in `mat`.
fn r14a_add(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

fn r14a_rgb(c: u32) -> (f64, f64, f64) {
    (((c >> 16) & 255) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64)
}

fn r14a_css(c: (f64, f64, f64), a: f64) -> String {
    let k = |v: f64| v.clamp(0.0, 255.0).round();
    format!("rgba({},{},{},{:.3})", k(c.0), k(c.1), k(c.2), a.clamp(0.0, 1.0))
}

fn r14a_mix(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
/// fading out from its middle (the context squashed, so no edge cuts it).
fn r14a_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &r14a_css(c, a));
    gr.add_color_stop(1, &r14a_css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A short wandering line from `at`, `len` px long at angle `ang`.
fn r14a_wander(ctx: &mut Canvas, at: (f64, f64), len: f64, ang: f64, c: (f64, f64, f64), a: f64, lw: f64) {
    ctx.set_stroke_style(&r14a_css(c, a));
    ctx.set_line_width(lw);
    ctx.begin_path();
    ctx.move_to(at.0, at.1);
    let (mut x, mut y, mut t) = (at.0, at.1, ang);
    for _ in 0..5 {
        t += (rnd() - 0.5) * 0.5;
        x += t.cos() * len / 5.0;
        y += t.sin() * len / 5.0;
        ctx.line_to(x, y);
    }
    ctx.stroke();
}

/// Leather after years, in its colour `base` (sRGB): mottled where it was
/// oiled and dried, its grain, creased, rubbed pale and scuffed (most low
/// on it, v < 0.35: the welt's edge all round, the toe's foot), a film of
/// dust. Painted on a stream of its own varied by `vary`.
fn r14a_leather_tex(base: u32, size: u32, vary: u32) -> Tex {
    canvas_tex(size, size, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || r14a_paint_leather(ctx, w, h, base))
    })
}

fn r14a_paint_leather(ctx: &mut Canvas, w: f64, h: f64, base: u32) {
    let c = r14a_rgb(base);
    let s = w / 256.0;
    ctx.set_fill_style(&r14a_css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.62, c.1 * 0.58, c.2 * 0.55);
    let pale = r14a_mix(c, (196.0, 184.0, 166.0), 0.42);
    for _ in 0..36 {
        let at = (rnd() * w, rnd() * h);
        let (col, a) = if rnd() < 0.55 { (dark, 0.10 + rnd() * 0.14) } else { (pale, 0.06 + rnd() * 0.10) };
        r14a_soft(ctx, at, (10.0 + rnd() * 34.0) * s, 0.5 + rnd() * 0.5, rnd() * 3.0, col, a);
    }
    // the grain: pores and a fine pebble
    for _ in 0..(2600.0 * s * s) as i32 {
        let col = if rnd() < 0.7 { dark } else { pale };
        ctx.set_fill_style(&r14a_css(col, 0.12 + rnd() * 0.16));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
    }
    // creases, each with its pale lip
    for _ in 0..(48.0 * s) as i32 {
        let at = (rnd() * w, rnd() * h);
        let len = (6.0 + rnd() * 24.0) * s;
        let ang = (rnd() - 0.5) * 1.0 + if rnd() < 0.25 { PI / 2.0 } else { 0.0 };
        r14a_wander(ctx, at, len, ang, (c.0 * 0.4, c.1 * 0.36, c.2 * 0.34), 0.45, 0.8 * s.max(0.7));
        r14a_wander(ctx, (at.0, at.1 + 1.0), len * 0.8, ang, pale, 0.18, 0.7);
    }
    // rubbed pale and scuffed, most of it low: the dye worn through to the
    // crust in ragged streaks along the rub, scratched
    for i in 0..30 {
        let y = if i < 22 { h * (0.66 + rnd() * 0.34) } else { rnd() * h };
        let at = (rnd() * w, y);
        let crust = r14a_mix(c, (190.0, 178.0, 160.0), 0.40 + rnd() * 0.2);
        let ang = (rnd() - 0.5) * 0.5;
        for _ in 0..(3 + (rnd() * 5.0) as i32) {
            let a2 = (at.0 + (rnd() - 0.5) * 10.0 * s, at.1 + (rnd() - 0.5) * 4.0 * s);
            r14a_wander(
                ctx,
                a2,
                (2.0 + rnd() * 9.0) * s,
                ang + (rnd() - 0.5) * 0.3,
                crust,
                0.18 + rnd() * 0.3,
                (0.5 + rnd() * 0.9) * s.max(0.6),
            );
        }
        r14a_soft(ctx, at, (4.0 + rnd() * 8.0) * s, 0.25, ang, crust, 0.10 + rnd() * 0.10);
    }
    // dust, and dried water's tide marks
    for _ in 0..12 {
        r14a_soft(ctx, (rnd() * w, rnd() * h), (18.0 + rnd() * 40.0) * s, 0.6, rnd() * 3.0, (150.0, 144.0, 132.0), 0.08 + rnd() * 0.08);
    }
    for _ in 0..3 {
        let (x, y, r) = (rnd() * w, rnd() * h, (10.0 + rnd() * 26.0) * s);
        ctx.set_stroke_style(&r14a_css(pale, 0.22));
        ctx.set_line_width(1.2 * s);
        ctx.begin_path();
        ctx.arc(x, y, r, rnd() * 3.0, 3.0 + rnd() * 3.0);
        ctx.stroke();
    }
}

/// What a glove is made of, piece by piece.
#[derive(Clone, Copy, PartialEq)]
enum R14aCloth {
    Leather,
    Knit,
    Canvas,
    Jersey,
}

/// A glove: its back's cloth and colour (sRGB), the leather tips, knuckle
/// strap and thumb of a leather palm work glove, its wrist's cloth and
/// colour, a stripe knitted round the wrist, the palm's colour, and a dress
/// glove's three stitched points.
struct R14aGlove {
    back: (R14aCloth, u32),
    tips: Option<u32>,
    cuff: (R14aCloth, u32),
    stripe: Option<u32>,
    under: u32,
    points: bool,
}

/// The glove of batch r14a: a right one lying back up, as modelled.
fn r14a_glove(s: &R14aGlove, vary: u32) -> Obj {
    let grp = group();
    let back = lambert!({ color: 0xffffff, map: &r14a_glove_back_tex(s.back.0, s.back.1, s.points, vary) });
    r14a_add(&grp, "Real__gloveBack", &back);
    r14a_add(&grp, "Real__gloveUnder", &lambert!({ color: s.under }));
    let cuff = lambert!({ color: 0xffffff, map: &r14a_glove_cuff_tex(s.cuff.0, s.cuff.1, s.stripe, vary ^ 0x5bd1) });
    r14a_add(&grp, "Real__gloveCuff", &cuff);
    if let Some(t) = s.tips {
        let tips = lambert!({ color: 0xffffff, map: &r14a_leather_tex(t, 128, vary ^ 0x2c1b) });
        r14a_add(&grp, "Real__gloveTips", &tips);
    }
    grp
}

/// The glove's back as modelled (r14a.py): its bounds in x and z (the top's
/// u runs with x, v against z), and each digit: base (x, z), length, flat
/// width, angle from +z toward +x, bow.
const R14A_GLOVE_XZ: (f64, f64, f64, f64) = (-0.0757, 0.0877, 0.0, 0.1955);
const R14A_DIGITS: [(f64, f64, f64, f64, f64, f64); 5] = [
    (0.0335, 0.094, 0.079, 0.0232, 0.13, 0.006),
    (0.0110, 0.098, 0.087, 0.0238, 0.03, -0.003),
    (-0.0120, 0.096, 0.080, 0.0228, -0.08, -0.007),
    (-0.0335, 0.088, 0.064, 0.0205, -0.23, -0.010),
    (0.045, 0.030, 0.068, 0.0265, 0.72, -0.0085),
];

/// A digit's axis at t (0 its base, 1 its tip): the point and its direction.
fn r14a_digit_at(d: &(f64, f64, f64, f64, f64, f64), t: f64) -> ((f64, f64), (f64, f64)) {
    let (bx, bz, l, _, ang, bow) = *d;
    let am = ang + bow / l * t;
    let a = ang + bow / l * 2.0 * t;
    ((bx + am.sin() * l * t, bz + am.cos() * l * t), (a.sin(), a.cos()))
}

/// A cloth's ground in colour `base`: knit wales, a canvas weave, a jersey's
/// fine knit, or leather; faded unevenly.
fn r14a_paint_cloth(ctx: &mut Canvas, w: f64, h: f64, cloth: R14aCloth, base: u32) {
    if cloth == R14aCloth::Leather {
        r14a_paint_leather(ctx, w, h, base);
        return;
    }
    let c = r14a_rgb(base);
    ctx.set_fill_style(&r14a_css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.66, c.1 * 0.64, c.2 * 0.62);
    let pale = r14a_mix(c, (200.0, 196.0, 186.0), 0.35);
    match cloth {
        R14aCloth::Canvas => {
            // the plain weave: warp and weft every 2 px, each thread uneven
            for i in (0..w as i32).step_by(2) {
                ctx.set_fill_style(&r14a_css(dark, 0.10 + rnd() * 0.10));
                ctx.fill_rect(i as f64, 0, 1, h);
            }
            for j in (0..h as i32).step_by(2) {
                ctx.set_fill_style(&r14a_css(pale, 0.08 + rnd() * 0.10));
                ctx.fill_rect(0, j as f64, w, 1);
            }
        }
        _ => {
            // knitted: columns of V stitches (a jersey's finer)
            let step = if cloth == R14aCloth::Knit { 4.0 } else { 3.0 };
            ctx.set_stroke_style(&r14a_css(dark, 0.22));
            ctx.set_line_width(0.8);
            let mut y = 0.0;
            while y < h {
                // a row of Vs, one path
                ctx.begin_path();
                let mut x = 0.0;
                ctx.move_to(x, y);
                while x < w {
                    ctx.line_to(x + step / 2.0, y + step * 0.8);
                    ctx.line_to(x + step, y);
                    x += step;
                }
                ctx.stroke();
                y += step * 0.8;
            }
            // fuzz and pills
            for _ in 0..500 {
                r14a_wander(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 4.0, rnd() * 6.3, pale, 0.25, 0.5);
            }
            for _ in 0..90 {
                ctx.set_fill_style(&r14a_css(pale, 0.4));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.5, 1.5);
            }
        }
    }
    // faded and soiled unevenly
    for _ in 0..30 {
        let (col, a) = if rnd() < 0.5 { ((c.0 * 0.6, c.1 * 0.56, c.2 * 0.5), 0.10 + rnd() * 0.12) } else { (pale, 0.08 + rnd() * 0.1) };
        r14a_soft(ctx, (rnd() * w, rnd() * h), 10.0 + rnd() * 34.0, 0.6 + rnd() * 0.4, rnd() * 3.0, col, a);
    }
}

/// The back of the glove: its cloth; the fingers' joints creased across, the
/// knuckles and fingertips rubbed and soiled where they took the work; a
/// dress glove's three stitched points; the dirt of the floor.
fn r14a_glove_back_tex(cloth: R14aCloth, base: u32, points: bool, vary: u32) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            r14a_paint_cloth(ctx, w, h, cloth, base);
            let (x0, x1, z0, z1) = R14A_GLOVE_XZ;
            let px = |x: f64, z: f64| ((x - x0) / (x1 - x0) * w, (z - z0) / (z1 - z0) * h);
            let c = r14a_rgb(base);
            let crease = (c.0 * 0.42, c.1 * 0.38, c.2 * 0.35);
            let dirt = (70.0, 62.0, 52.0);
            for (k, d) in R14A_DIGITS.iter().enumerate() {
                let hw = d.3 / 2.0;
                let joints: &[f64] = if k == 4 { &[0.45] } else { &[0.30, 0.62] };
                for &tj in joints {
                    for dt in [-0.03, 0.0, 0.03] {
                        let ((x, z), (dx, dz)) = r14a_digit_at(d, tj + dt);
                        let (nx, nz) = (-dz, dx);
                        let a = px(x - nx * hw, z - nz * hw);
                        let m = px(x + dx * 0.002, z + dz * 0.002);
                        let b = px(x + nx * hw, z + nz * hw);
                        ctx.set_stroke_style(&r14a_css(crease, if dt == 0.0 { 0.45 } else { 0.22 }));
                        ctx.set_line_width(1.1);
                        ctx.begin_path();
                        ctx.move_to(a.0, a.1);
                        ctx.quadratic_curve_to(m.0, m.1, b.0, b.1);
                        ctx.stroke();
                    }
                }
                // the tip soiled and rubbed
                let ((x, z), _) = r14a_digit_at(d, 0.93);
                let p = px(x, z);
                r14a_soft(ctx, p, 9.0, 1.0, 0.0, dirt, 0.35);
                if cloth == R14aCloth::Leather {
                    r14a_soft(ctx, p, 5.0, 0.7, rnd(), r14a_mix(c, (200.0, 190.0, 172.0), 0.5), 0.3);
                }
                // the knuckle
                if k < 4 {
                    let ((x, z), _) = r14a_digit_at(d, -0.05);
                    r14a_soft(ctx, px(x, z), 8.0, 0.6, 0.0, r14a_mix(c, (196.0, 188.0, 170.0), 0.5), 0.22);
                }
            }
            if points {
                // three lines of stitching from between the fingers' bases
                // toward the wrist, a little converging
                for (xa, xb) in [(0.022, 0.016), (-0.0005, -0.0005), (-0.023, -0.017)] {
                    let (a, b) = (px(xa, 0.088), px(xb, 0.036));
                    ctx.set_stroke_style(&r14a_css(crease, 0.6));
                    ctx.set_line_width(0.9);
                    ctx.set_line_dash(&[2.0, 1.6]);
                    for off in [-1.4, 1.4] {
                        ctx.begin_path();
                        ctx.move_to(a.0 + off, a.1);
                        ctx.line_to(b.0 + off, b.1);
                        ctx.stroke();
                    }
                    ctx.set_line_dash(&[]);
                }
            }
            // dirt of the floor, and where it was gripped
            for _ in 0..20 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 6.0 + rnd() * 22.0, 0.6 + rnd() * 0.4, rnd() * 3.0, dirt, 0.08 + rnd() * 0.12);
            }
        })
    })
}

/// The wrist: a ribbed knit (its ribs along the arm, its mouth at the
/// canvas' top), a stripe knitted round it, or the leather hemmed.
fn r14a_glove_cuff_tex(cloth: R14aCloth, base: u32, stripe: Option<u32>, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = r14a_rgb(base);
            if cloth == R14aCloth::Leather {
                r14a_paint_leather(ctx, w, h, base);
                // the hem's stitching by the mouth
                ctx.set_stroke_style(&r14a_css((c.0 * 0.45, c.1 * 0.4, c.2 * 0.38), 0.6));
                ctx.set_line_width(0.8);
                ctx.set_line_dash(&[2.0, 1.5]);
                ctx.begin_path();
                ctx.move_to(0, h * 0.10);
                ctx.line_to(w, h * 0.10);
                ctx.stroke();
                ctx.set_line_dash(&[]);
                return;
            }
            ctx.set_fill_style(&r14a_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            // the stripes, across it (the cuff runs z -0.066 at its mouth to 0)
            if let Some(sc) = stripe {
                let s = r14a_rgb(sc);
                for (a, b) in [(0.18, 0.26), (0.36, 0.44)] {
                    ctx.set_fill_style(&r14a_css(s, 1.0));
                    ctx.fill_rect(0, h * a, w, h * (b - a));
                }
            }
            // the ribs: a ridge and a valley every 5 px
            let mut x = 0.0;
            while x < w {
                ctx.set_fill_style("rgba(255,250,240,0.16)");
                ctx.fill_rect(x, 0, 2.0, h);
                ctx.set_fill_style("rgba(20,16,12,0.22)");
                ctx.fill_rect(x + 3.0, 0, 1.5, h);
                x += 5.0;
            }
            for _ in 0..200 {
                r14a_wander(
                    ctx,
                    (rnd() * w, rnd() * h),
                    2.0 + rnd() * 3.0,
                    rnd() * 6.3,
                    r14a_mix(c, (210.0, 204.0, 192.0), 0.4),
                    0.22,
                    0.5,
                );
            }
            // grubby where it was pulled on, the mouth stretched and darker
            r14a_soft(ctx, (w * 0.5, 0.0), w * 0.7, 0.18, 0.0, (60.0, 54.0, 46.0), 0.35);
            for _ in 0..10 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 16.0, 0.6, rnd() * 3.0, (70.0, 62.0, 52.0), 0.10 + rnd() * 0.1);
            }
        })
    })
}

/// The shoe or boot of batch r14a, `kind` "shoe" or "boot": `var` "R", "L",
/// "Side" (the right one fallen on its outer side, as it lies) or "" (the
/// boot standing). Colours sRGB: the leather, the lining, the sole's
/// rubber or leather, the laces, the eyelets' and hooks' metal.
fn r14a_footwear(kind: &str, var: &str, cols: (u32, u32, u32, u32, u32), vary: u32) -> Obj {
    let (leather, lining, sole, lace, metal) = cols;
    let grp = group();
    let name = |p: &str| format!("Real__{kind}{p}{var}");
    let boot = kind == "boot";
    let upper = lambert!({ color: 0xffffff, map: &r14a_leather_tex(leather, 256, vary) });
    r14a_add(&grp, &name("Upper"), &upper);
    r14a_add(&grp, &name("Lining"), &lambert!({ color: lining }));
    let edge = lambert!({ color: 0xffffff, map: &r14a_sole_tex(sole, false, vary ^ 0x91) });
    r14a_add(&grp, &name("Sole"), &edge);
    let heel =
        if boot || var == "Side" { edge.clone() } else { lambert!({ color: 0xffffff, map: &r14a_sole_tex(0x3e2c22, true, vary ^ 0x93) }) };
    r14a_add(&grp, &name("Heel"), &heel);
    // the tread shows only on one that has fallen over
    let tread = if var == "Side" {
        lambert!({ color: 0xffffff, map: &r14a_tread_tex(boot, sole, vary ^ 0x97) })
    } else {
        lambert!({ color: sole })
    };
    r14a_add(&grp, &name("Tread"), &tread);
    r14a_add(&grp, &name("Laces"), &lambert!({ color: lace }));
    let met = phong!({ color: metal, specular: 0x4a4a46, shininess: 40 });
    r14a_add(&grp, &name(if boot { "Hooks" } else { "Eyelets" }), &met);
    grp
}

/// A sole's edge and welt (or `stack`, a heel of leather lifts on a rubber
/// top piece, its layers across): inked and burnished, scuffed pale, dirt.
fn r14a_sole_tex(base: u32, stack: bool, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = r14a_rgb(base);
            ctx.set_fill_style(&r14a_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            if stack {
                // the lifts (each 4-5 mm of the heel's 22), the top piece below
                let mut y = 0.0;
                while y < h * 0.76 {
                    ctx.set_fill_style(&r14a_css((c.0 * 0.55, c.1 * 0.5, c.2 * 0.48), 0.7));
                    ctx.fill_rect(0, y, w, 1.0);
                    ctx.set_fill_style(&r14a_css(r14a_mix(c, (150.0, 120.0, 96.0), 0.3), 0.3));
                    ctx.fill_rect(0, y + 1.0, w, 1.0);
                    y += h * 0.19;
                }
                ctx.set_fill_style("rgb(40,38,36)");
                ctx.fill_rect(0, h * 0.77, w, h * 0.23);
            }
            for _ in 0..40 {
                ctx.set_fill_style(&r14a_css((c.0 * 0.7, c.1 * 0.68, c.2 * 0.66), 0.2));
                ctx.fill_rect(rnd() * w, rnd() * h, 4.0 + rnd() * 18.0, 1.0);
            }
            for _ in 0..22 {
                let crust = r14a_mix(c, (150.0, 140.0, 126.0), 0.5);
                r14a_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 6.0, 0.4, (rnd() - 0.5) * 0.5, crust, 0.35);
            }
            for _ in 0..10 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 20.0, 0.5, 0.0, (110.0, 102.0, 90.0), 0.12);
            }
        })
    })
}

/// The tread of one fallen on its side, as it lies (u from the toe, at u =
/// 0, back to the heel; v from the outer edge, on the floor, up to the inner
/// one): a shoe's leather sole worn dark at the ball and frayed at the toe,
/// paler in the waist where it never touched, the heel's rubber top piece
/// worn at its back; a boot's lugs, worn grey on their faces, grit in the
/// grooves.
fn r14a_tread_tex(boot: bool, base: u32, vary: u32) -> Tex {
    canvas_tex(256, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // the heel's breast, as modelled (r14a.py): the shoe's at 78% of
            // the tread from the toe, the boot's at 77%
            let br = if boot { 0.771 } else { 0.781 };
            if !boot {
                let leather = (122.0, 96.0, 70.0);
                ctx.set_fill_style(&r14a_css(leather, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // worn: the ball dark and smooth, the toe frayed paler
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.22, 0.6, 0.0, (64.0, 54.0, 44.0), 0.6);
                r14a_soft(ctx, (w * 0.04, h * 0.5), w * 0.07, 1.4, 0.0, (150.0, 132.0, 108.0), 0.5);
                // the waist, never on the floor: paler, the size stamped
                r14a_soft(ctx, (w * 0.66, h * 0.5), w * 0.12, 0.6, 0.0, (160.0, 132.0, 100.0), 0.45);
                ctx.save();
                ctx.translate(w * 0.66, h * 0.5);
                ctx.rotate(-PI / 2.0);
                ctx.set_fill_style("rgba(60,46,34,0.55)");
                ctx.set_font("bold 11px Helvetica");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("9", 0, -6);
                ctx.set_font("7px Helvetica");
                ctx.fill_text("LEATHER", 0, 7);
                ctx.restore();
                for _ in 0..140 {
                    r14a_wander(ctx, (rnd() * w * br, rnd() * h), 3.0 + rnd() * 9.0, (rnd() - 0.5) * 6.3, (70.0, 58.0, 46.0), 0.25, 0.6);
                }
                // the heel's top piece, black rubber, worn at the back
                ctx.set_fill_style("rgb(44,41,38)");
                ctx.fill_rect(w * br, 0, w * (1.0 - br), h);
                r14a_soft(ctx, (w * 1.0, h * 0.35), w * 0.06, 1.8, 0.0, (96.0, 88.0, 78.0), 0.6);
                for k in 0..5 {
                    ctx.set_fill_style("rgba(120,112,100,0.7)");
                    ctx.fill_rect(w * (br + 0.03 + 0.035 * (k as f64 % 2.0)), h * (0.2 + 0.15 * k as f64), 2.0, 2.0);
                }
            } else {
                let c = r14a_rgb(base);
                let groove = (c.0 * 0.55, c.1 * 0.55, c.2 * 0.55);
                ctx.set_fill_style(&r14a_css(groove, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // lugs in rows across the sole, chevroned; few in the waist
                let face = r14a_mix(c, (120.0, 114.0, 106.0), 0.35);
                let mut x = 6.0;
                let mut row = 0;
                while x < w - 4.0 {
                    let waist = x / w > 0.56 && x / w < br;
                    if x / w > br - 0.012 && x / w < br + 0.012 {
                        x += 6.0;
                        continue;
                    }
                    let n = if waist { 2 } else { 4 };
                    for k in 0..n {
                        let y = h * (0.12 + 0.76 * (k as f64 + 0.5 * (row % 2) as f64) / n as f64);
                        ctx.save();
                        ctx.translate(x, y);
                        ctx.rotate(if k * 2 < n { 0.5 } else { -0.5 });
                        ctx.set_fill_style(&r14a_css(face, 1.0));
                        ctx.fill_rect(-4.0, -9.0, 8.0, 18.0);
                        ctx.restore();
                    }
                    x += if waist { 16.0 } else { 11.0 };
                    row += 1;
                }
                // the faces worn grey, grit in the grooves
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.2, 0.6, 0.0, (130.0, 124.0, 114.0), 0.25);
                r14a_soft(ctx, (w * 0.92, h * 0.5), w * 0.1, 0.8, 0.0, (130.0, 124.0, 114.0), 0.25);
                for _ in 0..260 {
                    ctx.set_fill_style(&r14a_css((150.0, 140.0, 124.0), 0.35));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
                }
            }
        })
    })
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

// R705 vacuum cleaner: batch `t07` (t07.rs).

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
