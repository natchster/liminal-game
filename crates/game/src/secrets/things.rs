//! What is put away (`docs/design/SECRETS.md` 5): each a small, ordinary thing,
//! made as it is, its bottom at y = 0, centred, its front toward +z. Its
//! look says nothing of what it does; only what a real one carries is
//! printed on it (a card by a dish, a sachet's word).
use super::Thing;
use crate::assets::props::prop_part_as_modelled;
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
            Thing::Sweets => sweets_dish(seed),
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

/// A white dish of boiled sweets, each in a twist of cellophane, and a
/// tent card beside it in a hand: please take one
fn sweets_dish(seed: f64) -> Obj {
    let g = group();
    // a plain white dish, crazed with age
    let dish = lambert!({ color: 0xd6d0c2 });
    let dish = mesh(
        pg(
            "CeramicBowls__bowl",
            lathe(&[(0.0, 0.0), (0.045, 0.0), (0.075, 0.03), (0.088, 0.055), (0.084, 0.056), (0.07, 0.032), (0.0, 0.006)], 20.0),
        ),
        &dish,
    );
    g.add(&dish);
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
/// a pomegranate's
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

/// A small cloth bag sewn shut, on a cord: a charm, worn (a ladanka, an
/// omamori), lying where it was put: a pouch of coarse linen 6 by 7.5 cm,
/// stuffed, its mouth gathered, bound and sewn; the cord wound three times
/// round its neck and knotted, its loop lying loose beside it (Blender batch
/// `r09`)
fn cloth_bag() -> Obj {
    let g = group();
    let weave = canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#6a5a48");
        ctx.fill_rect(0, 0, w, h);
        // a plain weave of thick, uneven thread, darker in the hollows
        for k in 0..32 {
            let v = rnd();
            ctx.set_fill_style(format!("rgba({},{},{},0.5)", 120 + (v * 40.0) as i32, 104 + (v * 34.0) as i32, 82 + (v * 24.0) as i32));
            ctx.fill_rect(0, k as f64 * 2.0, w, 1.2);
            ctx.set_fill_style(format!("rgba(40,32,24,{:.3})", 0.2 + rnd() * 0.2));
            ctx.fill_rect(k as f64 * 2.0, 0, 0.8, h);
        }
        // handled: dark where the fingers held it, a pale stain
        for _ in 0..6 {
            let (x, y, r) = (rnd() * w, rnd() * h, 6.0 + rnd() * 14.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, &format!("rgba(30,24,18,{:.3})", 0.15 + rnd() * 0.2));
            gr.add_color_stop(1, "rgba(30,24,18,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, r, 0, 7);
            ctx.fill();
        }
    });
    weave.t().repeat.set(3, 3);
    let parts: [(&str, Mat); 2] =
        [("Real__secretBag", lambert!({ color: 0xffffff, map: &weave })), ("Real__secretBagCord", lambert!({ color: 0x2e251c }))];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            g.add(&mesh(geo, &mat));
        }
    }
    if g.child_count() == 0 {
        let bag = mesh(box_geo!(0.06, 0.024, 0.075), lambert!({ color: 0x5e4a3a }));
        bag.position().y = 0.012;
        g.add(&bag);
    }
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

/// A ration of concentrate as the Building gives it out: a grey
/// briquette in a paper wrapper printed as its coupon is (ONE PORTION.
/// CONCENTRATE, GREY.)
fn ration() -> Obj {
    let tex = canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#b9b4a6");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..160 {
            ctx.set_fill_style(format!("rgba(60,58,52,{:.3})", 0.05 + rnd() * 0.1));
            ctx.fill_rect(rnd() * w, rnd() * h, 2.0, 1.0);
        }
        ctx.set_fill_style("rgba(30,30,28,0.8)");
        ctx.set_font("bold 13px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("ONE PORTION", w / 2.0, h * 0.42);
        ctx.set_font("11px sans-serif");
        ctx.fill_text("CONCENTRATE, GREY", w / 2.0, h * 0.72);
    });
    let g = group();
    let b = mesh(box_geo!(0.11, 0.03, 0.06), lambert!({ map: tex }));
    b.position().y = 0.015;
    g.add(&b);
    // a corner of the wrapper torn back on the grey of it
    let c = mesh(box_geo!(0.03, 0.031, 0.02), lambert!({ color: 0x77756e }));
    c.position().set(0.045, 0.0155, 0.022);
    g.add(&c);
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

/// A tweed flat cap, worn, hung on its coat hook by the inside of its
/// crown: its opening to the wall, the crown out, the peak hanging at its
/// foot (Blender family `crate_sash_umbrella_and_things_put_away`: modelled as it hangs, in the frame the hook's
/// pose turns, `dress.rs`). Brown herringbone with flecks of colour in it,
/// rubbed and faded at the roll
fn tweed_cap() -> Obj {
    let tex = canvas_tex_opt(64, 64, true, |ctx, w, h| {
        ctx.set_fill_style("#5d5548");
        ctx.fill_rect(0, 0, w, h);
        // herringbone: columns of twill, each turned against the last
        for col in 0..8 {
            let x0 = col as f64 * 8.0;
            let dir = if col % 2 == 0 { 1.0 } else { -1.0 };
            for k in -8..24 {
                let v = rnd();
                let (light, a) = (k % 2 == 0, 0.35 + v * 0.3);
                ctx.set_stroke_style(if light {
                    format!("rgba({},{},{},{a:.3})", 150 + (v * 30.0) as i32, 136 + (v * 24.0) as i32, 110 + (v * 16.0) as i32)
                } else {
                    format!("rgba(40,34,28,{a:.3})")
                });
                ctx.set_line_width(1.4);
                let y = k as f64 * 3.0;
                ctx.begin_path();
                ctx.move_to(x0, y);
                ctx.line_to(x0 + 8.0, y + dir * 8.0);
                ctx.stroke();
            }
        }
        // the flecks of a Donegal yarn: rust, moss, cream
        for _ in 0..70 {
            let c =
                ["rgba(138,74,42,0.8)", "rgba(92,104,58,0.7)", "rgba(196,184,150,0.8)", "rgba(60,70,92,0.6)"][(rnd() * 4.0) as usize % 4];
            ctx.set_fill_style(c);
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd(), 1);
        }
    });
    tex.t().repeat.set(6, 6);
    let wool = lambert!({ color: 0xffffff, map: &tex });
    let g = group();
    for name in ["Real__secretCap", "Real__secretCapPeak"] {
        if let Some(geo) = prop_part_as_modelled(name) {
            g.add(&mesh(geo, &wool));
        }
    }
    if g.child_count() == 0 {
        let c = mesh(box_geo!(0.2, 0.06, 0.26), &wool);
        c.position().y = 0.03;
        g.add(&c);
    }
    g
}

/// A foreign coin of brass, 22 mm, a hole through it, worn smooth, lying
/// on the loop of string through its hole, knotted (Blender family `crate_sash_umbrella_and_things_put_away`)
fn holed_coin() -> Obj {
    let g = group();
    let parts: [(&str, Mat); 2] = [
        ("Real__secretCoin", phong!({ color: 0x7a6a48, specular: 0x8a7a50, shininess: 40 })),
        ("Real__secretCoinString", lambert!({ color: 0x3a3226 })),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            g.add(&mesh(geo, &mat));
        }
    }
    if g.child_count() == 0 {
        let c = mesh(cyl_geo!(0.011, 0.011, 0.0016, 16), lambert!({ color: 0x8a7a5a }));
        c.position().y = 0.0008;
        g.add(&c);
    }
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

/// A red envelope, a hongbao (9 by 17 cm), lying on a floor: its gold
/// print (a border and the character for luck) rubbed, the red faded and
/// dusty, creased; a lock of dark hair caught at its flap, out over its end
/// onto the floor (Blender family `crate_sash_umbrella_and_things_put_away`)
fn red_envelope() -> Obj {
    let tex = canvas_tex_opt(256, 128, false, |ctx, w, h| {
        // the paper, red gone dull, faded in streaks where light fell on it
        ctx.set_fill_style("#7c2621");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(160,96,80,{:.3})", 0.05 + rnd() * 0.10));
            ctx.fill_rect(rnd() * w, 0, 6.0 + rnd() * 30.0, h);
        }
        // the gold: a double border, the character upright along it (its top
        // to the flap's end), in a ring
        ctx.set_stroke_style("rgba(166,138,82,0.85)");
        ctx.set_line_width(2);
        ctx.stroke_rect(9, 9, w - 18.0, h - 18.0);
        ctx.set_line_width(1);
        ctx.stroke_rect(14, 14, w - 28.0, h - 28.0);
        ctx.save();
        ctx.translate(w * 0.55, h / 2.0);
        ctx.rotate(PI / 2.0);
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(0, 0, 40, 0, 7);
        ctx.stroke();
        ctx.set_fill_style("rgba(170,142,84,0.9)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 58px serif");
        ctx.fill_text("\u{798F}", 0, 2);
        ctx.restore();
        // rubbed: the gold worn through to the red, dust settled, a crease
        for _ in 0..260 {
            ctx.set_fill_style(format!("rgba(120,34,28,{:.3})", 0.3 + rnd() * 0.5));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        // dust settled on it, thicker away from where it was touched
        for _ in 0..1400 {
            ctx.set_fill_style(format!("rgba(140,132,118,{:.3})", 0.10 + rnd() * 0.25));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
        for _ in 0..12 {
            let (x, y, r) = (rnd() * w, rnd() * h, 10.0 + rnd() * 30.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, &format!("rgba(130,122,108,{:.3})", 0.10 + rnd() * 0.15));
            gr.add_color_stop(1, "rgba(130,122,108,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, r, 0, 7);
            ctx.fill();
        }
        ctx.set_stroke_style("rgba(40,10,8,0.35)");
        ctx.set_line_width(1.2);
        ctx.begin_path();
        ctx.move_to(w * 0.18, 0);
        ctx.line_to(w * 0.30, h);
        ctx.stroke();
        ctx.set_stroke_style("rgba(200,120,100,0.25)");
        ctx.begin_path();
        ctx.move_to(w * 0.18 + 1.5, 0);
        ctx.line_to(w * 0.30 + 1.5, h);
        ctx.stroke();
    });
    let g = group();
    let parts: [(&str, Mat); 2] = [
        ("Real__secretEnvelope", lambert!({ color: 0xffffff, map: &tex })),
        ("Real__secretEnvelopeHair", lambert!({ color: 0x1c1714, side: DoubleSide })),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            g.add(&mesh(geo, &mat));
        }
    }
    if g.child_count() == 0 {
        let e = mesh(box_geo!(0.17, 0.003, 0.09), lambert!({ map: &tex }));
        e.position().y = 0.0015;
        g.add(&e);
    }
    g
}
