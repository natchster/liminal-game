//! `place_bird_nests`
use crate::shared::bark_tex;
use crate::shared::blob;
use crate::shared::css;
use crate::shared::thin_lit;
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

// Bird nest: 4% chance per level. A robin's nest fallen to the floor
// (Blender batch `t05`): a cup of dry grass and twigs over mud, lined with
// fine grass, loose stems trailing from it; now and then an egg or two left
// in it, one broken.
pub(crate) fn place_bird_nests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x604020, 0x503018, 0x705030];
    let nest = group();
    // the old outer ring's twelve twigs: each one's colour and tilt; the
    // first colour is the nest's (grey-brown, darker, paler)
    let mut tone = 0usize;
    for i in 0..12 {
        let c = choice(&twig_colors);
        if i == 0 {
            tone = twig_colors.iter().position(|&t| t == c).unwrap_or(0);
        }
        let _ = rnd();
    }
    // the inner ring's ten (kept for their draws)
    for _i in 0..10 {
        let _ = choice(&twig_colors);
        let _ = rnd();
    }
    const TONE: [u32; 3] = [0xffffff, 0xd8d2c8, 0xfff8ec];
    let grass = lambert!({ map: &nest_grass_tex(), color: TONE[tone] });
    for (name, mat) in [
        ("Real__nestCup", grass),
        ("Real__nestLining", lambert!({ map: &nest_lining_tex(), color: TONE[tone] })),
        ("Real__nestTwigs", lambert!({ map: &bark_tex(), color: 0xb0a490 })),
    ] {
        if let Some(geo) = part(name) {
            nest.add(&mesh(geo, &mat));
        }
    }
    // the loose stems wound round it and standing out of it: dry grass, thin
    for (name, c) in [("Real__nestStrands", 0x8c7c62), ("Real__nestStray", 0x9a8a6e)] {
        if let Some(geo) = thin_lit(name) {
            nest.add(&mesh(geo, lambert!({ color: c })));
        }
    }
    // 30%: one or two eggs left in it (the second broken)
    if rnd() < 0.30 {
        let egg_mat = lambert!({ map: &nest_egg_tex() });
        let egg_count = 1 + (rnd() * 2.0).floor() as i32;
        for e in 0..egg_count {
            let (ex, ez) = ((rnd() - 0.5) * 0.020, (rnd() - 0.5) * 0.020);
            let name = if e == 0 { "Real__nestEgg" } else { "Real__nestEggBroken" };
            let Some(geo) = part(name) else { continue };
            let egg = mesh(geo, &egg_mat);
            // lying in the cup's floor, turned by where it lies
            let (ox, oz) = if e == 0 { (ex - 0.012, ez) } else { (ex + 0.016, ez + 0.006) };
            egg.position().set(ox, NEST_FLOOR - 0.001 + 0.18 * (ox * ox + oz * oz).sqrt(), oz);
            egg.rotation().y = (ex * 300.0).sin() * PI;
            nest.add(&egg);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    nest.position().set(px, 0, pz);
    nest.rotation().y = rnd() * PI * 2.0;
    g.world.add(&nest);
}

// =================================================================== nest
/// The cup's floor (`nature/bits_of_the_outside_indoors.py` NEST_FLOOR): the eggs lie on it
const NEST_FLOOR: f64 = 0.026;

/// Dry grass and stems matted over mud, seen from any side (the nest's
/// faces take it by the box's three ways): straw and grey-brown stems every
/// way but mostly round it, mud showing between them, a few dark rootlets
fn nest_grass_tex() -> Tex {
    cached_tex!(512, 512, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let m = fbm(0x7e57, x / 30.0, y / 30.0, 4);
            let c = mix_unclamped([0.34, 0.28, 0.20], [0.24, 0.20, 0.15], sstep(0.4, 0.7, m));
            [c[0], c[1], c[2], 1.0]
        });
        ctx.set_line_cap("round");
        for i in 0..2600 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = if i % 4 == 0 { rnd() * PI } else { (rnd() - 0.5) * 0.7 };
            let l = 12.0 + rnd() * 50.0;
            let bend = (rnd() - 0.5) * 0.5;
            let tone = rnd();
            let c = if tone < 0.55 {
                mix_unclamped([0.62, 0.55, 0.40], [0.52, 0.45, 0.32], rnd())
            } else if tone < 0.85 {
                mix_unclamped([0.44, 0.38, 0.29], [0.36, 0.31, 0.24], rnd())
            } else {
                [0.22, 0.17, 0.12]
            };
            ctx.set_stroke_style(css(c, 0.55 + 0.4 * rnd()));
            ctx.set_line_width(0.8 + rnd() * 2.2);
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            ctx.begin_path();
            ctx.move_to(x - dx, y - dy);
            ctx.quadratic_curve_to(x - dy * bend, y + dx * bend, x + dx, y + dy);
            ctx.stroke();
        }
        // dust settled on it over the years
        for _ in 0..40 {
            blob(ctx, rnd() * w, rnd() * h, 10.0 + rnd() * 30.0, 0.6 + 0.4 * rnd(), rnd() * 3.0, [0.50, 0.47, 0.42], 0.10 + 0.12 * rnd());
        }
    })
}

/// The lining: fine dry grass laid round and round the cup (concentric seen
/// from above, where its floor takes it; round it on its walls)
fn nest_lining_tex() -> Tex {
    cached_tex!(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style(css([0.46, 0.40, 0.31], 1.0));
        ctx.fill_rect(0, 0, w, h);
        ctx.set_line_cap("round");
        for _ in 0..900 {
            let r = rnd().sqrt() * w * 0.5;
            let a0 = rnd() * PI * 2.0;
            let span = 0.4 + rnd() * 1.2;
            let c = mix_unclamped([0.66, 0.58, 0.44], [0.40, 0.34, 0.26], rnd());
            ctx.set_stroke_style(css(c, 0.5 + 0.45 * rnd()));
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.arc_dir(w / 2.0 + (rnd() - 0.5) * 6.0, h / 2.0 + (rnd() - 0.5) * 6.0, r, a0, a0 + span, false);
            ctx.stroke();
        }
        // the dust and droppings of an old nest, pale specks in its bottom
        for _ in 0..30 {
            let (a, r) = (rnd() * PI * 2.0, rnd() * w * 0.25);
            blob(ctx, w / 2.0 + a.cos() * r, h / 2.0 + a.sin() * r, 2.0 + rnd() * 4.0, 1.0, 0.0, [0.80, 0.78, 0.72], 0.5);
        }
        blob(ctx, w / 2.0, h / 2.0, w * 0.3, 1.0, 0.0, [0.48, 0.45, 0.40], 0.3);
    })
}

/// A robin's egg gone dull: its blue faded, dirt and a stain on it
fn nest_egg_tex() -> Tex {
    cached_tex!(64, 64, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let m = fbm(0xe66, x / 9.0, y / 9.0, 3);
            let c = mix_unclamped([0.62, 0.72, 0.68], [0.55, 0.60, 0.55], sstep(0.4, 0.8, m));
            [c[0], c[1], c[2], 1.0]
        });
        for _ in 0..24 {
            blob(ctx, rnd() * w, rnd() * h, 1.0 + rnd() * 2.5, 1.0, 0.0, [0.35, 0.30, 0.24], 0.4 + 0.4 * rnd());
        }
        blob(ctx, w * 0.3, h * 0.7, 14.0, 0.6, 0.4, [0.45, 0.40, 0.30], 0.35);
    })
}
