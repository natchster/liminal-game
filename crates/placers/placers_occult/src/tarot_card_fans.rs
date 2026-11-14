//! `place_tarot_card_fans`
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Tarot card fan: 5% chance per level. Five cards of a tarot deck of
// the Rider's size (70 x 120 mm) fanned on the floor as a reading left
// off: four face down, their backs the Rider's crackled brown, and one face
// up, a card of Pamela Colman Smith's (public domain); the cards' edges
// rubbed white, grimed and creased, one corner dog-eared.
pub(crate) fn place_tarot_card_fans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let vary = pictures::vary_of(px, pz, turn);
    let face = if vary.is_multiple_of(2) { "tarot_death" } else { "tarot_moon" };
    let mut age = Age::engraving(vary);
    age.fade = 0.06;
    age.spots = (vary % 7) as usize;
    age.stain = 0.0;
    age.grime = 0.06 + 0.1 * f64::from((vary >> 8) % 17) / 17.0;
    let face_mat = lambert!({ map: &pictures::print_tex(face, CARD, 2600.0, age, vary) });
    let back_mat = lambert!({ map: &tarot_back_tex(age, vary) });
    let bent = (vary >> 4) % 5;
    let cards = group();
    // fanned from a point 10 mm in from their feet (+z), each turned about
    // it; the face-up card the middle one, turned over and laid back on top
    let r = CARD.1 / 2.0 - 0.010;
    for i in 0..5 {
        let a = -0.48 + (f64::from(i) / 4.0) * 0.96;
        let name = if i as u32 == bent && i != 2 { "Real__tarotCardBent" } else { "Real__tarotCard" };
        let mat = if i == 2 { &face_mat } else { &back_mat };
        let layer = if i == 2 { 5.0 } else { f64::from(i) };
        if let Some(m) = put(&cards, name, mat) {
            m.position().set(a.sin() * r, layer * 0.0004, r - a.cos() * r);
            m.rotation().y = -a;
        }
    }
    cards.position().set(px, 0, pz);
    cards.rotation().y = turn;
    g.world.add(&cards);
}

/// A tarot card (m)
const CARD: (f64, f64) = (0.070, 0.120);

/// A card's back as the Rider deck's 1910 printing has it: a crackle of
/// dark lines over a mottled brown ("cracked mud") within a narrow cream
/// border; aged as a print
fn tarot_back_tex(age: Age, vary: u32) -> Tex {
    pictures::sheet_tex(CARD, 2600.0, age, vary, move |ctx, w, h| {
        let seed = vary ^ 0x7a40;
        fill(ctx, w, h, move |u, v| {
            let (x, y) = (u * CARD.0 * 1000.0, (1.0 - v) * CARD.1 * 1000.0);
            let edge = x.min(CARD.0 * 1000.0 - x).min(y).min(CARD.1 * 1000.0 - y);
            if edge < 3.2 {
                return [0.86, 0.82, 0.72, 1.0];
            }
            let (d, id) = crackle(seed, x, y, 3.6);
            let n = fbm(seed ^ 9, x * 0.08, y * 0.08, 3);
            let mut c = mix([0.50, 0.36, 0.24], [0.36, 0.25, 0.16], n * 0.8 + id * 0.3);
            c = mix(c, [0.14, 0.10, 0.07], 1.0 - sstep(0.12, 0.45, d));
            // the printed frame line inside the border
            if (edge - 4.0).abs() < 0.35 {
                c = mix(c, [0.16, 0.12, 0.09], 0.8);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The distance (in cells) to the nearest edge of a Voronoi crackle of
/// cells `size` across, and the cell's own value
fn crackle(seed: u32, x: f64, y: f64, size: f64) -> (f64, f64) {
    let (gx, gy) = (x / size, y / size);
    let (cx, cy) = (gx.floor() as i32, gy.floor() as i32);
    let (mut d1, mut d2, mut id) = (9.0f64, 9.0f64, 0.0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (ix, iy) = (cx + dx, cy + dy);
            let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
            let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
            let d = ((gx - px).powi(2) + (gy - py).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = hash(seed ^ 0x99, ix, iy);
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d2 - d1, id)
}
