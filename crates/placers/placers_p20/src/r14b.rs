//! Batch `r14b`'s props in this crate, made real (`docs/design/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Keychain w/ keys: 8% chance per level. A bunch of 3-5 keys on a
// 25 mm split ring, dropped (Blender batch r14b): house keys of the KW1 and
// SC1 kinds, a padlock key, a small cabinet key, each cut brass, some
// nickel-plated, lying fanned over one another, their bows bunched on the
// ring; a metal-rimmed paper tag on it, written on once.
pub(super) fn place_keychain_keys(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.keychain_positions = vec![]; // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // how many keys, and for each whether it is bare brass or plated (the
    // old draws, call for call)
    let n_keys = 3 + (rnd() * 3.0).floor() as i32;
    let brass: Vec<bool> = (0..n_keys).map(|_| rnd() < 0.5).collect();
    let keychain = group();
    scatter(g, &keychain, x, z, 2.4);
    let p = keychain.pos();
    let seed = r14b_kit::seed_of(&[p.x, p.z]);
    let brass_mat = phong!({ color: 0xffffff, specular: 0x8a6e3e, shininess: 44 });
    brass_mat.m().map = Some(r14b_kit::tarnish_tex(true, seed));
    let nickel_mat = phong!({ color: 0xffffff, specular: 0x8a8a86, shininess: 56 });
    nickel_mat.m().map = Some(r14b_kit::tarnish_tex(false, seed));
    for (i, &b) in brass.iter().enumerate() {
        if let Some(geo) = prop_part_as_modelled(&format!("Real__keysKey{i}")) {
            keychain.add(&mesh(geo, if b { &brass_mat } else { &nickel_mat }));
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__keysRing") {
        keychain.add(&mesh(geo, phong!({ color: 0x3a3a38, specular: 0x6a6a68, shininess: 50 })));
    }
    if let Some(geo) = prop_part_as_modelled("Real__keysTagPaper") {
        keychain.add(&mesh(geo, lambert!({ map: &r14b_kit::tag_tex(seed) })));
    }
    g.world.add(&keychain);
    // Track for jangle audio.
    let p = keychain.pos();
    g.pp.keychain_positions.push(v3(p.x, 0.003, p.z));
}

/// Batch r14b's kit (the keys): a seed from where they lie, and what is
/// painted: brass and nickel gone dull, the tag's paper and what was
/// written on it.
mod r14b_kit {
    use game::prelude::*;

    /// A seed from values already fixed (a position): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    /// Brass (or nickel over it) as years in a pocket and on a floor leave
    /// it: dull, darker in blotches, rubbed bright in patches, green or rust
    /// in specks (the metal's albedo: dark, its colour in its highlight)
    pub fn tarnish_tex(brass: bool, seed: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            let (base, dark, bright, speck) = if brass {
                ("#857040", "rgba(46,34,16,0.45)", "rgba(172,142,84,0.5)", "rgba(70,96,70,0.6)")
            } else {
                ("#6e6e6a", "rgba(34,32,30,0.45)", "rgba(150,150,146,0.5)", "rgba(96,64,40,0.6)")
            };
            ctx.set_fill_style(base);
            ctx.fill_rect(0, 0, w, h);
            let mut r = SplitMix64::new(seed, if brass { 0xb7a5 } else { 0x41c3 });
            for _ in 0..24 {
                let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(3.0, 12.0));
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, dark);
                gr.add_color_stop(1, "rgba(0,0,0,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
            }
            for _ in 0..14 {
                let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(2.0, 7.0));
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, bright);
                gr.add_color_stop(1, "rgba(0,0,0,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
            }
            for _ in 0..20 {
                ctx.set_fill_style(speck);
                ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), r.range(0.6, 1.6), r.range(0.6, 1.6));
            }
        })
    }

    /// What the tag says, in felt pen
    const TAGS: [&str; 10] = ["SPARE", "BOILER", "STORE 2", "B-14", "BACK DOOR", "OFFICE", "PLANT RM", "SHED", "LOCKER 31", "MAIN"];

    /// The tag's paper, seen from above (the eyelet toward the ring): card
    /// gone yellow and grey, a printed rule round it, the word written in
    /// felt pen and faded, a damp stain
    pub fn tag_tex(seed: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(seed, 0x7a95);
            ctx.set_fill_style("#c9bea0");
            ctx.fill_rect(0, 0, w, h);
            let gr = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.2, w / 2.0, h / 2.0, w * 0.55);
            gr.add_color_stop(0, "rgba(120,100,60,0)");
            gr.add_color_stop(1, "rgba(110,90,52,0.35)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h);
            ctx.set_stroke_style("rgba(60,90,120,0.45)");
            ctx.set_line_width(1.2);
            ctx.begin_path();
            ctx.arc(w / 2.0, h / 2.0, w * 0.40, 0, 7);
            ctx.stroke();
            let word = TAGS[((r.next() * TAGS.len() as f64) as usize).min(TAGS.len() - 1)];
            ctx.save();
            ctx.translate(w / 2.0, h / 2.0 + 4.0);
            ctx.rotate(r.range(-0.25, 0.15));
            ctx.set_fill_style("rgba(28,30,44,0.72)");
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_font("22px Marker Felt");
            ctx.fill_text_max(word, 0, 0, w * 0.74);
            ctx.restore();
            // faded and rubbed where a thumb held it
            for _ in 0..30 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                ctx.set_fill_style(&format!("rgba(206,196,170,{:.2})", r.range(0.1, 0.3)));
                ctx.begin_path();
                ctx.ellipse(x, y, r.range(2.0, 8.0), r.range(1.0, 4.0), r.range(0.0, 3.0), 0, 7);
                ctx.fill();
            }
            let (x, y, rr) = (r.range(20.0, 108.0), r.range(20.0, 108.0), r.range(14.0, 30.0));
            let st = ctx.create_radial_gradient(x, y, rr * 0.7, x, y, rr);
            st.add_color_stop(0, "rgba(120,90,50,0.0)");
            st.add_color_stop(0.85, "rgba(120,90,50,0.25)");
            st.add_color_stop(1, "rgba(120,90,50,0)");
            ctx.set_fill_style(st);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        })
    }
}
