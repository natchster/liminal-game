//! `place_small_toy_pianos`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;

// Small toy piano: 6% chance per level. Rectangular piano
// with 7 white + 5 black keys. Child-music family.
pub(crate) fn place_small_toy_pianos(g: &mut Game, _dist: &[i32]) {
    g.pp.piano_positions = vec![]; // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let body_colors: [u32; 5] = [0xd82040, 0x2050d8, 0xd8b040, 0x8a5030, 0x181008];
    let body_mat = lambert!({ color: pick_hex(&body_colors) });
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
    // Phantom key press animator — random key sinks briefly
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
    // Track for random chord audio.
    let pp = piano.pos();
    g.pp.piano_positions.push(v3(pp.x, 0.03, pp.z));
}
