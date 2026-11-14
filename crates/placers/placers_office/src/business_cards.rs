//! `place_business_cards`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dropped business card: 10% chance per level.
pub(crate) fn place_business_cards(g: &mut Game, _dist: &[i32]) {
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

// `card_texture`: business card (name line, subtitle, contact lines, logo dot).
game::lazy_static_tex!(
    card_texture,
    canvas_tex(96, 56, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Top name line (bold thick).
        ctx.set_fill_style("#202020");
        ctx.fill_rect(6, 12, w * 0.55, 3);
        // Subtitle.
        ctx.set_fill_style("#606060");
        ctx.fill_rect(6, 19, w * 0.35, 2);
        // Contact lines.
        for i in 0..3 {
            ctx.fill_rect(6, 30.0 + i as f64 * 6.0, w * (0.4 + rnd() * 0.3), 1);
        }
        // Small dark logo circle.
        ctx.set_fill_style("#802020");
        ctx.begin_path();
        ctx.arc(w - 12.0, 12, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
