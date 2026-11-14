//! `place_small_princess_wands`: Small princess wand: 6% chance per level. A party-shop fairy wand: a faceted plastic star (56 mm) glittered, on a white plastic rod 18 cm long wound with a foil stripe, three satin ribbons tied under the star; lying on the floor, the ribbons trailing back beside the rod. The glitter rubbed off in patches, the satin dirty, the white yellowed.
use crate::shared::wear_at;
use crate::shared::wear_tex;
use crate::shared::Wear;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::fbmp;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep as ss;
use placers_kit::paint::mul;
use placers_kit::paint::paint_tile;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::salt_at;
use placers_kit::place::scatter_pos;

/// Small princess wand: 6% chance per level. A party-shop fairy wand:
/// a faceted plastic star (56 mm) glittered, on a white plastic rod 18 cm
/// long wound with a foil stripe, three satin ribbons tied under the star;
/// lying on the floor, the ribbons trailing back beside the rod. The glitter
/// rubbed off in patches, the satin dirty, the white yellowed.
pub(crate) fn place_small_princess_wands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old star colours: the glitter's (gold, a pinkish silver, rose, blue)
    let star_colors = [0xd8b040, 0xf0d0d0, 0xd050a0, 0x2080d8];
    let ks = index_of(&star_colors, pick_color(&star_colors));
    let ribbon_colors = [0xd050a0, 0xd82040, 0xd8b040, 0xa020a0];
    // each ribbon's old colour and length: its satin, and which of the three
    // modelled lengths (8, 11, 14 cm) it is
    let mut ribbons = [(0usize, "S"); 3];
    for r in &mut ribbons {
        let kc = index_of(&ribbon_colors, pick_color(&ribbon_colors));
        let len = 0.08 + rnd() * 0.06;
        *r = (
            kc,
            if len < 0.10 {
                "S"
            } else if len < 0.12 {
                "M"
            } else {
                "L"
            },
        );
    }
    let wand = group();
    scatter_pos(g, &wand, x, z, 2.4);
    wand.rotation().y = rnd() * PI * 2.0;
    let p = wand.pos();
    let salt = salt_at(p.x, p.z);
    let glitter = rgb([0xa88c52, 0xbcaaa6, 0x9c5a76, 0x4e6a96][ks]);
    let star_tex = paint_tile(128, move |u, v| {
        let w =
            Wear { base: glitter, alt: mix(glitter, [0.75, 0.73, 0.7], 0.5), rub: 0.12, grime: 0.35, scuff: 0.2, mottle: 0.06, scales: 0 };
        let mut c = wear_at(salt ^ 31, u, v, &w);
        // the glitter: flakes catching the light, and dark ones
        let s = hash(salt ^ 33, (u * 64.0) as i32, (v * 64.0) as i32);
        if s > 0.9 {
            c = mix(c, [0.96, 0.94, 0.88], (s - 0.9) * 7.0);
        } else if s < 0.12 {
            c = mul(c, 0.7);
        }
        c
    });
    put_world(&wand, "Real__fairyWandStar", &phong!({ map: &star_tex, specular: 0x5a5a5a, shininess: 60 }), 0.03);
    let rod_tex = paint_tile(128, move |u, v| {
        let w = Wear { base: rgb(0xd4cdbd), alt: rgb(0xbfb39a), rub: 0.06, grime: 0.45, scuff: 0.3, mottle: 0.06, scales: 0 };
        let mut c = wear_at(salt ^ 35, u, v, &w);
        // the foil stripe wound round it, rubbed off in places
        let st = ((u * 3.0 + v * 3.0).fract() - 0.5).abs();
        let foil = (1.0 - ss(0.12, 0.15, st)) * ss(0.3, 0.45, fbmp(salt ^ 37, u, v, 4, 3));
        c = mix(c, mix(glitter, [0.8, 0.78, 0.74], 0.3), foil);
        c
    });
    put_world(&wand, "Real__fairyWandRod", &phong!({ map: &rod_tex, specular: 0x2a2a2a, shininess: 30 }), 0.04);
    let satins = [0xa0627e, 0x92404a, 0xa8925c, 0x744a76];
    for (i, &(kc, len)) in ribbons.iter().enumerate() {
        let satin = rgb(satins[kc]);
        let w = Wear { base: satin, alt: mix(satin, [0.85, 0.82, 0.78], 0.35), rub: 0.0, grime: 0.5, scuff: 0.15, mottle: 0.08, scales: 0 };
        let mat = phong!({ map: &wear_tex(salt ^ (40 + i as u32), 64, w), specular: 0x2c2c2c, shininess: 22 });
        put_world(&wand, &format!("Real__fairyWandRibbon{i}{len}"), &mat, 0.04);
        if i == 0 {
            put_world(&wand, "Real__fairyWandKnot", &mat, 0.04);
        }
    }
    g.world.add(&wand);
}
