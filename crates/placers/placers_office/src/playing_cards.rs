//! `place_playing_cards`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Playing card: 9% chance per level. Single upturned card.
pub(crate) fn place_playing_cards(g: &mut Game, _dist: &[i32]) {
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

// `card_face_texture`: playing card with a random suit + rank.
game::lazy_static_tex!(
    card_face_texture,
    canvas_tex(64, 96, |ctx, w, h| {
        ctx.set_fill_style("#fafaf4");
        ctx.fill_rect(0, 0, w, h);
        // Border.
        ctx.set_stroke_style("#606060");
        ctx.set_line_width(1);
        ctx.stroke_rect(2, 2, w - 4.0, h - 4.0);
        // Random suit + rank.
        let suits = ["\u{2665}", "\u{2666}", "\u{2660}", "\u{2663}"];
        let ranks = ["A", "K", "Q", "J", "10", "7", "3"];
        let suit = suits[(rnd() * suits.len() as f64).floor() as usize];
        let rank = ranks[(rnd() * ranks.len() as f64).floor() as usize];
        let red = suit == "\u{2665}" || suit == "\u{2666}";
        ctx.set_fill_style(if red { "#c02020" } else { "#202020" });
        ctx.set_font("bold 14px sans-serif");
        ctx.set_text_align("left");
        // Top-left corner.
        ctx.fill_text(rank, 5, 16);
        ctx.set_font("14px sans-serif");
        ctx.fill_text(suit, 5, 32);
        // Center big suit.
        ctx.set_font("bold 36px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text(suit, w / 2.0, h / 2.0 + 12.0);
        // Bottom-right corner (rotated).
        ctx.save();
        ctx.translate(w - 5.0, h - 6.0);
        ctx.rotate(PI);
        ctx.set_font("bold 14px sans-serif");
        ctx.set_text_align("left");
        ctx.fill_text(rank, 0, 8);
        ctx.set_font("14px sans-serif");
        ctx.fill_text(suit, 0, 24);
        ctx.restore();
    })
);
