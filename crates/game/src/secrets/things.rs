//! What is put away (`docs/SECRETS.md` 5): each a small, ordinary thing,
//! made as it is, its bottom at y = 0, centred, its front toward +z. Its
//! look says nothing of what it does; only what a real one carries is
//! printed on it (a card by a bowl, a sachet's word).
use super::Thing;
use crate::prelude::*;

/// A lathe from a profile of (radius, height) pairs
fn lathe(p: &[(f64, f64)], seg: f64) -> Geo {
    let pts: Vec<Vec2> = p.iter().map(|&(r, y)| Vec2::new(r, y)).collect();
    lathe_geometry(&pts, seg, 0.0, PI * 2.0)
}

impl Game {
    /// The thing, made (`seed` turns its bits: its own draws, none of the level's)
    pub(crate) fn make_thing(&mut self, t: Thing, seed: f64) -> Obj {
        match t {
            Thing::PaleMushrooms => mushrooms(false, seed),
            Thing::BrownMushrooms => mushrooms(true, seed),
            Thing::Sweets => sweets_bowl(seed),
            Thing::Seeds => seeds_saucer(seed),
            Thing::Bag => cloth_bag(),
            Thing::Salt => salt_sachet(),
            Thing::Jelly => jelly_jar(),
            Thing::Ration => ration(),
            Thing::Medicine => medicine_bottle(),
            Thing::Cap => tweed_cap(),
            Thing::Coin => holed_coin(),
            Thing::YellowWater => yellow_water(),
            Thing::Envelope => red_envelope(),
            _ => group(),
        }
    }
}

/// A cluster of mushrooms grown out of wet carpet: pale and soft-capped, or
/// brown and dry on rotten wood; five, one small, their gills under them
fn mushrooms(brown: bool, seed: f64) -> Obj {
    let g = group();
    // (half again a field mushroom's size: the place grows them so)
    let (cap_c, stem_c, gill_c) = if brown { (0x8a6440, 0xc9b495, 0x4a3420) } else { (0xddd6c3, 0xe9e3d3, 0xb7ae98) };
    let cap_mat = lambert!({ color: cap_c });
    let stem_mat = lambert!({ color: stem_c });
    let gill_mat = lambert!({ color: gill_c, side: DoubleSide });
    let spots = [(0.0, 0.0, 1.6), (0.055, 0.02, 1.3), (-0.05, 0.03, 1.45), (0.02, -0.055, 1.05), (-0.035, -0.04, 0.8)];
    for (k, &(x, z, s)) in spots.iter().enumerate() {
        let tilt = ((seed * 7.3 + k as f64 * 2.1).sin()) * 0.18;
        let m = group();
        let h = 0.05 * s;
        let stem = mesh(pg("Mushrooms__stem", cyl_geo!(0.006 * s, 0.009 * s, h, 8)), &stem_mat);
        stem.position().y = h / 2.0;
        m.add(&stem);
        // the cap: a low dome, its rim turned under
        let r = 0.022 * s;
        let cap = mesh(
            lathe(&[(0.0, h + r * 0.55), (r * 0.6, h + r * 0.48), (r * 0.95, h + r * 0.2), (r, h + 0.002), (r * 0.9, h - 0.002)], 12.0),
            &cap_mat,
        );
        m.add(&cap);
        let gills = mesh(circle_geo!(r * 0.92, 12), &gill_mat);
        gills.rotation().x = PI / 2.0;
        gills.position().y = h - 0.001;
        m.add(&gills);
        m.rotation().z = tilt;
        m.rotation().x = tilt * 0.6;
        m.position().set(x, 0, z);
        g.add(&m);
    }
    g
}

/// A shallow glass bowl of boiled sweets each in a twist of cellophane,
/// and a tent card beside it in a hand: PLEASE TAKE ONE
fn sweets_bowl(seed: f64) -> Obj {
    let g = group();
    // a plain white dish, crazed with age
    let dish = lambert!({ color: 0xd6d0c2 });
    let bowl = mesh(
        pg(
            "CeramicBowls__bowl",
            lathe(&[(0.0, 0.0), (0.045, 0.0), (0.075, 0.03), (0.088, 0.055), (0.084, 0.056), (0.07, 0.032), (0.0, 0.006)], 20.0),
        ),
        &dish,
    );
    g.add(&bowl);
    let colours = [0x8a1c1c, 0xc9a227, 0x2c6a3a, 0x8a1c1c, 0xd8d0c0, 0x6a2a6a, 0xc9a227];
    for k in 0..9 {
        let a = k as f64 * 2.4 + seed;
        let rr = if k == 0 { 0.0 } else { 0.03 + 0.025 * ((k % 3) as f64 / 2.0) };
        let s = sweet(colours[k % colours.len()]);
        s.position().set(a.cos() * rr, 0.018 + (k / 4) as f64 * 0.012, a.sin() * rr);
        s.rotation().y = a * 1.7;
        s.rotation().z = (a * 0.7).sin() * 0.3;
        g.add(&s);
    }
    // the card, folded into a tent, written by hand
    let tex = canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#e8e3d4");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(40,38,60,0.85)");
        ctx.set_font("italic 15px serif");
        ctx.set_text_align("center");
        ctx.fill_text("please take one", w / 2.0, h * 0.58);
    });
    let card_mat = lambert!({ map: tex, side: DoubleSide });
    for s in [-1.0, 1.0] {
        let c = mesh(plane_geo!(0.09, 0.045), &card_mat);
        c.position().set(0.14, 0.021, s * 0.011);
        c.rotation().y = if s > 0.0 { 0.0 } else { PI };
        c.rotation().x = -s * 0.25;
        g.add(&c);
    }
    g
}

/// A boiled sweet in a twist of cellophane, 4 cm end to end
fn sweet(color: u32) -> Obj {
    let g = group();
    let body = mesh(sphere_geo!(0.009, 8, 6), lambert!({ color: color }));
    body.scale().set(1.3, 0.8, 1.0);
    g.add(&body);
    let wrap = lambert!({ color: 0xe8e8e0, transparent: true, opacity: 0.55, side: DoubleSide });
    for s in [-1.0, 1.0] {
        let tw = mesh(cone_geo!(0.008, 0.016, 6), &wrap);
        tw.rotation().z = s * PI / 2.0;
        tw.position().x = s * 0.018;
        g.add(&tw);
    }
    g
}

/// A saucer, and on it a twist of paper opened out on a few red seeds,
/// wet, like a pomegranate's
fn seeds_saucer(seed: f64) -> Obj {
    let g = group();
    let saucer = mesh(
        pg(
            "PorcelainCupSaucers__saucer",
            lathe(&[(0.0, 0.0), (0.04, 0.0), (0.072, 0.012), (0.075, 0.016), (0.07, 0.016), (0.04, 0.006), (0.0, 0.006)], 20.0),
        ),
        lambert!({ color: 0xd9d4c8 }),
    );
    g.add(&saucer);
    let paper = mesh(plane_geo!(0.07, 0.07), lambert!({ color: 0xd6cdb6, side: DoubleSide }));
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = seed;
    paper.position().y = 0.0075;
    g.add(&paper);
    let aril = phong!({ color: 0x7a0f1a, specular: 0x6a5050, shininess: 60, transparent: true, opacity: 0.88 });
    for k in 0..7 {
        let a = k as f64 * 2.2 + seed;
        let r = 0.004 + (k % 3) as f64 * 0.007;
        let s = mesh(sphere_geo!(0.0045, 7, 5), &aril);
        s.scale().set(1.0, 0.8, 1.25);
        s.position().set(a.cos() * r, 0.011, a.sin() * r);
        s.rotation().y = a;
        g.add(&s);
    }
    g
}

/// A small cloth bag sewn shut, on a cord: a charm, worn; hung on a nail
fn cloth_bag() -> Obj {
    let g = group();
    let cloth = lambert!({ color: 0x5e4a3a });
    let bag = mesh(sphere_geo!(0.03, 10, 8), &cloth);
    bag.scale().set(1.0, 1.2, 0.55);
    bag.position().y = 0.036;
    g.add(&bag);
    // the seam across its top, gathered
    let neck = mesh(cyl_geo!(0.008, 0.016, 0.014, 8), &cloth);
    neck.position().y = 0.074;
    g.add(&neck);
    // the cord, a loop up to where it hangs
    let cord = mesh(torus_geo!(0.045, 0.0016, 4, 18), lambert!({ color: 0x2a2018 }));
    cord.position().y = 0.12;
    cord.scale().set(0.55, 1.0, 1.0);
    g.add(&cord);
    g
}

/// A paper sachet of salt as cafés give out: SALT on it
fn salt_sachet() -> Obj {
    let tex = canvas_tex_opt(96, 48, false, |ctx, w, h| {
        ctx.set_fill_style("#efede6");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(120,40,40,0.6)");
        ctx.set_line_width(2);
        ctx.stroke_rect(4, 4, w - 8.0, h - 8.0);
        ctx.set_fill_style("rgba(120,40,40,0.85)");
        ctx.set_font("bold 18px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("SALT", w / 2.0, h * 0.66);
        // the crimped ends
        ctx.set_fill_style("rgba(0,0,0,0.12)");
        for k in 0..12 {
            ctx.fill_rect(k as f64 * 8.0, 0, 4, 3);
            ctx.fill_rect(k as f64 * 8.0, h - 3.0, 4, 3);
        }
    });
    let g = group();
    let s = mesh(box_geo!(0.06, 0.004, 0.03), lambert!({ map: tex }));
    s.position().y = 0.002;
    g.add(&s);
    g
}

/// A small glass jar with no label, full of something set like jelly,
/// pale blue, its lid rusted
fn jelly_jar() -> Obj {
    let g = group();
    let glass = lambert!({ color: 0xc8d4d4, transparent: true, opacity: 0.35 });
    let body = mesh(lathe(&[(0.0, 0.0), (0.028, 0.0), (0.03, 0.006), (0.03, 0.06), (0.024, 0.07), (0.024, 0.076)], 16.0), &glass);
    g.add(&body);
    let jelly = mesh(
        cyl_geo!(0.027, 0.027, 0.052, 14),
        phong!({ color: 0x5a86a8, specular: 0x8aa0b0, shininess: 50, transparent: true, opacity: 0.8 }),
    );
    jelly.position().y = 0.029;
    g.add(&jelly);
    let lid = mesh(pg("MasonJars__lid", cyl_geo!(0.026, 0.026, 0.01, 14)), lambert!({ color: 0x6a5a48 }));
    lid.position().y = 0.08;
    g.add(&lid);
    g
}

/// A ration briquette in brown wrapping paper, its printing in Russian
/// (food concentrate, its weight)
fn ration() -> Obj {
    let tex = canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#8a6e4c");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..160 {
            ctx.set_fill_style(format!("rgba(40,28,16,{:.3})", 0.05 + rnd() * 0.1));
            ctx.fill_rect(rnd() * w, rnd() * h, 2.0, 1.0);
        }
        ctx.set_fill_style("rgba(30,24,20,0.8)");
        ctx.set_font("bold 13px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("КОНЦЕНТРАТ", w / 2.0, h * 0.42);
        ctx.set_font("11px sans-serif");
        ctx.fill_text("ПИЩЕВОЙ  300 г", w / 2.0, h * 0.72);
    });
    let g = group();
    let b = mesh(box_geo!(0.11, 0.03, 0.06), lambert!({ map: tex }));
    b.position().y = 0.015;
    g.add(&b);
    g
}

/// A small amber medicine bottle, its label soaked off but for a strip of
/// paper, a white cap
fn medicine_bottle() -> Obj {
    let g = group();
    let amber = phong!({ color: 0x5a2e10, specular: 0x6a5040, shininess: 70, transparent: true, opacity: 0.85 });
    let body = mesh(
        lathe(&[(0.0, 0.0), (0.019, 0.0), (0.021, 0.004), (0.021, 0.055), (0.016, 0.064), (0.011, 0.068), (0.011, 0.076)], 16.0),
        &amber,
    );
    g.add(&body);
    let strip = mesh(cyl_geo!(0.0215, 0.0215, 0.012, 12, 1, true, 0.3, 1.6), lambert!({ color: 0xcfc6ae, side: DoubleSide }));
    strip.position().y = 0.03;
    g.add(&strip);
    let cap = mesh(cyl_geo!(0.0125, 0.0125, 0.014, 12), lambert!({ color: 0xd8d4c8 }));
    cap.position().y = 0.082;
    g.add(&cap);
    g
}

/// A tweed flat cap, worn, hung by its brim
fn tweed_cap() -> Obj {
    let tex = canvas_tex_opt(64, 64, true, |ctx, w, h| {
        ctx.set_fill_style("#5b5245");
        ctx.fill_rect(0, 0, w, h);
        for y in 0..32 {
            for x in 0..32 {
                if (x + y) % 3 == 0 || rnd() < 0.15 {
                    ctx.set_fill_style(format!("rgba({},{},{},0.5)", 70 + (rnd() * 40.0) as i32, 62 + (rnd() * 30.0) as i32, 48,));
                    ctx.fill_rect(x as f64 * 2.0, y as f64 * 2.0, 2.0, 2.0);
                }
            }
        }
    });
    let wool = lambert!({ map: tex });
    let g = group();
    let crown = mesh(sphere_geo!(0.1, 14, 8, 0.0, PI * 2.0, 0.0, PI / 2.0), &wool);
    crown.scale().set(1.0, 0.35, 1.18);
    crown.position().set(0, 0, -0.02);
    g.add(&crown);
    let brim = mesh(cyl_geo!(0.07, 0.075, 0.006, 14, 1, false, -PI / 2.0, PI), &wool);
    brim.scale().set(1.0, 1.0, 0.75);
    brim.position().set(0, 0.002, 0.09);
    g.add(&brim);
    g
}

/// A foreign coin with a hole through it, on a loop of string
fn holed_coin() -> Obj {
    let g = group();
    let coin =
        mesh(pg("CoinScatters__coin", torus_geo!(0.0085, 0.0045, 6, 16)), phong!({ color: 0x8a7a5a, specular: 0x9a9070, shininess: 40 }));
    coin.scale().set(1.0, 1.0, 0.35);
    coin.position().y = 0.013;
    g.add(&coin);
    let string = mesh(torus_geo!(0.05, 0.0012, 4, 18), lambert!({ color: 0x3a3226 }));
    string.position().y = 0.06;
    string.scale().set(0.5, 1.0, 1.0);
    g.add(&string);
    g
}

/// An almond water bottle like any other, but faintly yellow, and its seal
/// broken (the cap a little off true)
fn yellow_water() -> Obj {
    let (body, cap) = crate::core_props::vending::goods::bottle();
    let g = group();
    g.add(&mesh(&body, lambert!({ color: 0xcfc89a, transparent: true, opacity: 0.62 })));
    let c = mesh(&cap, lambert!({ color: 0xc8c8c0 }));
    c.rotation().z = 0.08;
    c.position().set(0.002, 0.004, 0);
    g.add(&c);
    g
}

/// A red envelope, a little gold on it, a lock of dark hair caught at its flap
fn red_envelope() -> Obj {
    let tex = canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#9a1a18");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(200,160,70,0.85)");
        ctx.set_line_width(2);
        ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 9.0, 0.0, PI * 2.0);
        ctx.stroke();
        // the hair at the flap
        ctx.set_stroke_style("rgba(12,10,10,0.9)");
        ctx.set_line_width(1);
        for k in 0..9 {
            ctx.begin_path();
            ctx.move_to(w * 0.62 + k as f64, 0);
            ctx.quadratic_curve_to(w * 0.7 + k as f64 * 1.5, h * 0.2, w * 0.66 + k as f64 * 2.0, h * 0.42);
            ctx.stroke();
        }
    });
    let g = group();
    let e = mesh(pg("DroppedMailScatters__envelope", box_geo!(0.16, 0.003, 0.08)), lambert!({ map: tex }));
    e.position().y = 0.0015;
    g.add(&e);
    g
}
