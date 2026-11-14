//! `place_wallets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Wallet w/ ID card + spilled cash bills (floor): 6% chance per level.
pub(crate) fn place_wallets(g: &mut Game, _dist: &[i32]) {
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
        bill.position().set(ang.dcos() * rr, 0.0005 + i as f64 * 0.001, ang.dsin() * rr);
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
