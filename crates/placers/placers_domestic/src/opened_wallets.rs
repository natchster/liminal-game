//! `place_opened_wallets`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Opened wallet w/ credit cards fanned (floor): 5% chance per level.
pub(crate) fn place_opened_wallets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x603020, 0x201010, 0x8b5a2b, 0x404040, 0x502010, 0x101010];
    let leather_mat = lambert!({ color: choice(&leather_colors) });
    let wg = group();
    // Bi-fold layout — laid flat open with visible interior card slots.
    let (half_w, half_d) = (0.055, 0.09);
    let left_half = mesh(pg("OpenedWallets__leftHalf", box_geo!(half_w, 0.005, half_d)), &leather_mat);
    left_half.position().set(-half_w / 2.0, 0.0025, 0);
    wg.add(&left_half);
    let right_half = mesh(pg("OpenedWallets__rightHalf", box_geo!(half_w, 0.005, half_d)), &leather_mat);
    right_half.position().set(half_w / 2.0, 0.0025, 0);
    wg.add(&right_half);
    // Spine bump.
    let spine = mesh(pg("OpenedWallets__spine", box_geo!(0.004, 0.007, half_d)), &leather_mat);
    spine.position().set(0, 0.004, 0);
    wg.add(&spine);
    // Card slot dividers visible on left half — 3 horizontal thin ridges.
    for i in 0..3 {
        let slot_rim = mesh(pg("OpenedWallets__slotLip", box_geo!(half_w * 0.85, 0.0015, 0.003)), &leather_mat);
        slot_rim.position().set(-half_w / 2.0, 0.006, -half_d * 0.30 + i as f64 * half_d * 0.20);
        wg.add(&slot_rim);
    }
    // Cash slot fold on right half (horizontal seam).
    let cash_slot = mesh(pg("OpenedWallets__cashPocket", box_geo!(half_w * 0.85, 0.0015, half_d * 0.85)), &leather_mat);
    cash_slot.position().set(half_w / 2.0, 0.006, 0);
    wg.add(&cash_slot);
    // 3-5 credit cards fanned out from left slots.
    let card_colors = [
        0x2060a0, // blue
        0x606060, // silver
        0xa08040, // gold
        0x101010, // black
        0xa02040, // red
        0x408060, // green
        0xc0a060, // rose gold
    ];
    let card_count = 3 + (rnd() * 3.0).floor() as i32;
    // Fan pivot near left half's edge.
    let fan_pivot_x = -half_w / 2.0 + half_w * 0.35;
    let fan_pivot_z = 0.0;
    let fan_spread = 1.3; // Total angular spread across all cards.
    for i in 0..card_count {
        let card_mat = lambert!({ color: choice(&card_colors) });
        let card = group();
        // Card body.
        let card_body = mesh(pg("OpenedWallets__card", box_geo!(0.085, 0.001, 0.055)), &card_mat);
        card_body.position().y = 0.0005;
        card.add(&card_body);
        // Chip square top-left of card (gold rectangle).
        let chip_mat = lambert!({ color: 0xd8b060 });
        let chip = mesh(pg("OpenedWallets__chip", box_geo!(0.010, 0.0015, 0.008)), &chip_mat);
        chip.position().set(-0.025, 0.0012, -0.015);
        card.add(&chip);
        // Bank/network logo indicator (small colored circle).
        let logo_colors = [0xa02040, 0x2060a0, 0xd0a040, 0x606060];
        let logo_mat = lambert!({ color: choice(&logo_colors) });
        let logo = mesh(circle_geo!(0.005, 6), &logo_mat);
        logo.rotation().x = -PI / 2.0;
        logo.position().set(0.030, 0.0013, -0.015);
        card.add(&logo);
        // Card number embossed (thin light bar).
        let emboss_mat = lambert!({ color: 0xf0e8d0 });
        let emboss = mesh(pg("OpenedWallets__emboss", box_geo!(0.060, 0.0012, 0.003)), &emboss_mat);
        emboss.position().set(0, 0.0013, 0.010);
        card.add(&emboss);
        // Position fan-style — angular spread + slight radial offset.
        let ang_t = (i as f64 / (card_count - 1) as f64) - 0.5; // -0.5..0.5
        let card_angle = ang_t * fan_spread;
        let card_radius = 0.04 + i as f64 * 0.005; // Progressive outward stack.
        card.position().set(
            fan_pivot_x + (card_angle - PI / 2.0).dcos() * card_radius,
            0.006 + i as f64 * 0.0012,
            fan_pivot_z + (card_angle - PI / 2.0).dsin() * card_radius,
        );
        card.rotation().y = card_angle;
        wg.add(&card);
    }
    // 60% chance: single cash bill peeking from right slot.
    if rnd() < 0.6 {
        let bill_mat = lambert!({ color: 0x60805a });
        let bill = mesh(pg("OpenedWallets__bill", box_geo!(0.045, 0.001, 0.025)), &bill_mat);
        bill.position().set(half_w / 2.0 + 0.02, 0.008, half_d * 0.20);
        bill.rotation().y = -0.3;
        wg.add(&bill);
        // Portrait blob.
        let portrait = mesh(circle_geo!(0.006, 5), lambert!({ color: 0x304a2a }));
        portrait.rotation().x = -PI / 2.0;
        portrait.position().set(half_w / 2.0 + 0.02, 0.0086, half_d * 0.20);
        portrait.rotation().z = -0.3;
        wg.add(&portrait);
    }
    // Photo slot on inside w/ tiny blur figure — 40% chance.
    if rnd() < 0.4 {
        let photo_frame_mat = lambert!({ color: 0xf0e8c0 });
        let photo_frame = mesh(box_geo!(half_w * 0.6, 0.001, half_d * 0.4), &photo_frame_mat);
        photo_frame.position().set(-half_w / 2.0, 0.008, -half_d * 0.05);
        wg.add(&photo_frame);
        // Tiny sepia figure blob.
        let fig_mat = lambert!({ color: 0x604030 });
        let fig = mesh(circle_geo!(half_w * 0.15, 5), &fig_mat);
        fig.rotation().x = -PI / 2.0;
        fig.position().set(-half_w / 2.0, 0.0085, -half_d * 0.05);
        wg.add(&fig);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    wg.position().set(px, 0, pz);
    wg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wg);
}
