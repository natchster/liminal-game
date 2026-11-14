//! `place_receipts`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dropped receipt: 9% chance per level. Thin white strip with printed "text" lines.
pub(crate) fn place_receipts(g: &mut Game, _dist: &[i32]) {
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

// `receipt_texture`: thin white strip with printed "text" lines + barcode.
game::lazy_static_tex!(
    receipt_texture,
    canvas_tex(48, 200, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Header dark bar.
        ctx.set_fill_style("#202020");
        ctx.fill_rect(6, 8, w - 12.0, 4);
        ctx.fill_rect(6, 16, w - 12.0, 2);
        // Text lines (uneven length).
        for i in 0..14 {
            ctx.set_fill_style("#404040");
            let y = 30.0 + i as f64 * 10.0;
            let line_w = 8.0 + rnd() * (w - 22.0);
            ctx.fill_rect(6, y, line_w, 2);
        }
        // Footer barcode-style bars.
        let bar_y = 180.0;
        ctx.set_fill_style("#202020");
        for i in 0..20 {
            ctx.fill_rect(6.0 + i as f64 * 1.8, bar_y, 1.0 + rnd() * 1.0, 12);
        }
    })
);
