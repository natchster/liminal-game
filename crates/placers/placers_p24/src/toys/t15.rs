//! Batch `t15`'s toys in this crate, made real (`docs/design/PROPS.md` P3, "toys
//! stay toys"; `assets/src/blender/props/t15.py`, `t15.notes.md`): the
//! fairy wand, the toy microphone and the small tyrannosaurus. Their
//! placers and what only they use; each keeps the old placer's draws, call
//! for call (the comments at each say what a draw now chooses).
use crate::{pick_color, pick_open_tile, scatter_pos};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

use placers_kit::place::index_of;

/// Small princess wand: 6% chance per level. A party-shop fairy wand:
/// a faceted plastic star (56 mm) glittered, on a white plastic rod 18 cm
/// long wound with a foil stripe, three satin ribbons tied under the star;
/// lying on the floor, the ribbons trailing back beside the rod. The glitter
/// rubbed off in patches, the satin dirty, the white yellowed.
pub fn place_small_princess_wands(g: &mut Game, _dist: &[i32]) {
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

/// Small toy microphone: 8% chance per level. A child's toy
/// microphone, 14 cm: a round grille moulded as a mesh in grey plastic and
/// painted silver, a dark band round it, a coloured handle with grip rings
/// and a slide switch; lying on its grille and its butt. The silver rubbed
/// through to the grey, the handle scuffed and grimed.
pub fn place_small_toy_microphones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old handle colours, as moulded plastic faded
    let handle_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0x181008];
    let k = index_of(&handle_colors, pick_color(&handle_colors));
    let plastic = rgb([0x8a3636, 0x3a527a, 0xa88c48, 0x4e6c44, 0x6a406a, 0x2e2a26][k]);
    let mic = group();
    scatter_pos(g, &mic, x, z, 2.4);
    mic.rotation().y = rnd() * PI * 2.0;
    let p = mic.pos();
    let salt = salt_at(p.x, p.z);
    let grille_tex = paint_tile(128, move |u, v| {
        let w = Wear { base: rgb(0x9c9c98), alt: rgb(0x46484a), rub: 0.15, grime: 0.5, scuff: 0.2, mottle: 0.08, scales: 0 };
        let c = wear_at(salt ^ 51, u, v, &w);
        // the mesh's holes, about 2 mm apart, each a little off its place
        // (so the sides the texture is laid from meet without a seam)
        let (gx, gy) = (u * 10.0, v * 10.0);
        let (ci, cj) = (gx.floor() as i32, gy.floor() as i32);
        let jx = (hash(salt ^ 55, ci.rem_euclid(10), cj.rem_euclid(10)) - 0.5) * 0.3;
        let jy = (hash(salt ^ 57, ci.rem_euclid(10), cj.rem_euclid(10)) - 0.5) * 0.3;
        let (fx, fy) = (gx.fract() - 0.5 - jx, gy.fract() - 0.5 - jy);
        let hole = 1.0 - ss(0.24, 0.32, (fx * fx + fy * fy).sqrt());
        mix(c, [0.05, 0.05, 0.05], hole * 0.92)
    });
    put_world(&mic, "Real__toyMicGrille", &phong!({ map: &grille_tex, specular: 0x4a4a4a, shininess: 40 }), 0.02);
    put(&mic, "Real__toyMicBand", &phong!({ color: 0x2a2826, specular: 0x202020, shininess: 30 }));
    let w = Wear { base: plastic, alt: mix(plastic, [0.82, 0.8, 0.76], 0.35), rub: 0.0, grime: 0.5, scuff: 0.5, mottle: 0.06, scales: 0 };
    put_world(&mic, "Real__toyMicHandle", &phong!({ map: &wear_tex(salt ^ 52, 128, w), specular: 0x222222, shininess: 26 }), 0.04);
    put(&mic, "Real__toyMicKnob", &phong!({ color: 0x24221f, specular: 0x202020, shininess: 20 }));
    g.world.add(&mic);
}

/// Small toy dinosaur: 6% chance per level. A solid PVC tyrannosaurus
/// of the 1980s kind, 18 cm: its body level, the tail's tip on the floor,
/// the mouth open on painted teeth, the eyes dots of paint; moulded in one
/// colour, its skin sculpted in scales. Faded, scuffed, grime in its scales.
pub fn place_small_toy_dinos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old colours, as PVC faded
    let dino_colors = [0x408040, 0x306030, 0x504020, 0x805040, 0x2050d8];
    let k = index_of(&dino_colors, pick_color(&dino_colors));
    let pvc = rgb([0x5c6c42, 0x46583a, 0x5e4e38, 0x7e5e46, 0x445470][k]);
    let dino = group();
    scatter_pos(g, &dino, x, z, 2.4);
    dino.rotation().y = rnd() * PI * 2.0;
    let p = dino.pos();
    let salt = salt_at(p.x, p.z);
    let w = Wear { base: pvc, alt: mix(pvc, [0.85, 0.82, 0.75], 0.3), rub: 0.0, grime: 0.45, scuff: 0.4, mottle: 0.07, scales: 22 };
    put_world(&dino, "Real__smallTrexBody", &phong!({ map: &wear_tex(salt ^ 61, 256, w), specular: 0x141414, shininess: 14 }), 0.05);
    put(&dino, "Real__smallTrexTeeth", &lambert!({ color: 0xbdb39a }));
    put(&dino, "Real__smallTrexEyes", &lambert!({ color: 0x1c1915 }));
    g.world.add(&dino);
}
// ------------------------------------------------------------- the paint ---

/// Part `name` of batch `t15` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

use placers_kit::geometry::put_world;

use placers_kit::place::salt_at;

use placers_kit::paint::rgb;

use placers_kit::noise::sstep as ss;

use placers_kit::noise::mix;

use placers_kit::paint::mul;

use placers_kit::noise::hash;

use placers_kit::noise::vnp;

use placers_kit::noise::fbmp;

use placers_kit::noise::cell_edge;

/// How a moulded or painted thing has worn on a floor for years
#[derive(Clone, Copy)]
struct Wear {
    /// its colour, and what shows where it is rubbed through (the plastic
    /// under paint, or plastic whitened by stress)
    base: [f64; 3],
    alt: [f64; 3],
    /// how much of it is rubbed through, grimed, scuffed, mottled (faded
    /// unevenly); its sculpted scales (cells across a tile; 0 none)
    rub: f64,
    grime: f64,
    scuff: f64,
    mottle: f64,
    scales: i32,
}

/// The worn surface at (u, v) of a repeating tile
fn wear_at(seed: u32, u: f64, v: f64, w: &Wear) -> [f64; 3] {
    let m = fbmp(seed, u, v, 3, 4);
    let mut c = mul(w.base, 1.0 + w.mottle * (m - 0.5) * 2.4);
    if w.scales > 0 {
        let e = cell_edge(seed ^ 0x51ab, u, v, w.scales);
        c = mul(c, 1.0 - 0.2 * (1.0 - ss(0.0, 0.14, e)));
    }
    if w.rub > 0.0 {
        // rubbed through in small patches: `rub` is about the share of the
        // surface (this fractal noise sits near 0.49, its spread about 0.15)
        let r = fbmp(seed ^ 0x2b3c, u, v, 12, 3);
        let t = 0.8 - 0.9 * w.rub;
        c = mix(c, w.alt, ss(t, t + 0.025, r));
    }
    // grime settled in soft patches
    let gm = fbmp(seed ^ 0x77e1, u, v, 3, 4);
    c = mix(c, [0.15, 0.13, 0.10], w.grime * 0.7 * ss(0.5, 0.8, gm));
    if w.scuff > 0.0 {
        // fine scratches in patches: thin lines along a stretched noise
        let patch = ss(0.5, 0.7, fbmp(seed ^ 0x9137, u, v, 3, 2));
        let l1 = (vnp(seed ^ 0x4421, u * 40.0, v * 5.0, 40, 5) - 0.5).abs();
        let l2 = (vnp(seed ^ 0x1d77, u * 6.0, v * 34.0, 6, 34) - 0.5).abs();
        let line = (1.0 - ss(0.0, 0.025, l1)).max(1.0 - ss(0.0, 0.025, l2));
        c = mix(c, mix(w.base, [0.86, 0.84, 0.8], 0.45), w.scuff * patch * line * 0.8);
    }
    // dust in the grain, and grit
    let d = vnp(seed ^ 0xd057, u * 90.0, v * 90.0, 90, 90);
    if d > 0.8 {
        c = mix(c, [0.55, 0.52, 0.47], (d - 0.8) * 2.2);
    }
    c
}

use placers_kit::paint::paint_tile;

/// A worn surface's tile
fn wear_tex(seed: u32, px: u32, w: Wear) -> Tex {
    paint_tile(px, move |u, v| wear_at(seed, u, v, &w))
}
