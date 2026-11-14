//! `place_orrery_models`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::vary;

// The orrery: 2.5% chance per level. A planetary orrery (Blender batch
// `t20`, after Newton's of London, about 1820): a turned mahogany drum on
// three feet, its printed paper dial of the zodiac and the calendar, a brass
// pillar carrying the brass sun, five brass arms on their own sleeves with
// their posts and the ivory planets, Mercury to Jupiter, the Earth a little
// globe with its moon; a crank in its side. The mahogany dull and dusty,
// the dial browned under its varnish, the brass gone brown, the ivory
// yellowed.
pub(crate) fn place_orrery_models(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old draws, call for call: whether its planets turned, where it
    // stands and how it is turned
    let _ = rnd(); // keeps the old draw of its planets' turning (60%): they stand still now
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    let turn = rnd() * PI * 2.0;
    let orr = group();
    orr.position().set(px, 0, pz);
    orr.rotation().y = turn;
    let vary = vary(px, pz, turn);
    put(&orr, "Real__orreryBase", &phong!({ map: &or_wood_tex(vary), specular: 0x161412, shininess: 16 }));
    put(&orr, "Real__orreryDial", &lambert!({ map: &or_dial_tex(vary) }));
    let brass = phong!({ map: &or_brass_tex(vary), specular: 0x4a3c22, shininess: 34 });
    put(&orr, "Real__orreryPillar", &brass);
    put(&orr, "Real__orrerySun", &brass);
    put(&orr, "Real__orreryCrank", &brass);
    let ivory = lambert!({ color: 0xbfae8c });
    put(&orr, "Real__orreryCrankKnob", &ivory);
    // the planets on their arms, each its own group about the pillar, where
    // the crank last left them
    for i in 0..5 {
        let pgrp = group();
        put(&pgrp, &format!("Real__orreryArm{i}"), &brass);
        let ball = group();
        ball.position().set(OR_ORBIT[i], OR_PLANET_Y, 0);
        pgrp.add(&ball);
        put(&ball, &format!("Real__orreryPlanet{i}"), &lambert!({ map: &or_planet_tex(vary, i) }));
        if i == 2 {
            put(&ball, "Real__orreryMoon", &lambert!({ color: 0x8e8a80 }));
        }
        pgrp.rotation().y = (i as f64 / 5.0) * PI * 2.0;
        orr.add(&pgrp);
    }
    g.world.add(&orr);
}

/// The orrery (victorian/a_studys_things.py `OR_*`): each planet's orbit (m from the pillar), the
/// planets' height, the Earth's moon's offset from the Earth
const OR_ORBIT: [f64; 5] = [0.050, 0.075, 0.104, 0.135, 0.178];

const OR_PLANET_Y: f64 = 0.236;

/// Turned mahogany as the drum and feet are mapped: its figure, the polish
/// gone dull and blooming grey, dust on what faces up
fn or_wood_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x0a1;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, u, v, [0.32, 0.15, 0.08], 10.0, 30.0);
            let bloom = sstep(0.5, 0.8, fbm(seed ^ 3, u * 6.0, v * 6.0, 3));
            o = mix(o, [0.40, 0.33, 0.28], bloom * 0.25);
            let dust = 0.08 + 0.12 * fbm(seed ^ 5, u * 8.0, v * 8.0, 3);
            opaque(mix(o, [0.43, 0.40, 0.35], dust))
        });
    })
}

/// Brass gone brown and dull, mottled, rubbed brighter in places
fn or_brass_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0xb7;
        fill(ctx, w, h, |u, v| {
            let k = tarnish(seed, u, v, 5.0);
            let rub = sstep(0.7, 0.85, fbm(seed ^ 2, u * 8.0, v * 8.0, 2));
            opaque(mix([0.40, 0.31, 0.16].map(|c| c * k), [0.55, 0.44, 0.24], rub * 0.4))
        });
    })
}

/// An ivory planet as a sphere is mapped (u round it, v up it): Mercury
/// grey, Venus cream, the Earth a little globe (seas, coasts of no exact
/// shape), Mars red-brown, Jupiter banded; the ivory yellowed and grimed
fn or_planet_tex(vary: u32, i: usize) -> Tex {
    canvas_tex_opt(64, 32, false, move |ctx, w, h| {
        let seed = vary ^ (0x91 + i as u32 * 31);
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 3.0, 3);
            let ivory = [0.74, 0.66, 0.50];
            let o = match i {
                0 => mix(ivory, [0.52, 0.50, 0.46], 0.6 + 0.3 * n),
                1 => mix(ivory, [0.80, 0.70, 0.50], 0.5),
                2 => {
                    let land = sstep(0.52, 0.56, fbm(seed ^ 4, u * 5.0, v * 3.0, 4));
                    let sea = [0.36, 0.44, 0.46];
                    let ice = sstep(0.12, 0.05, v.min(1.0 - v));
                    mix(mix(sea, [0.56, 0.50, 0.33], land), ivory, ice * 0.8)
                }
                3 => mix([0.52, 0.30, 0.20], [0.45, 0.25, 0.16], n),
                _ => {
                    let band = ((v * 9.0 + 0.3 * n).dsin() * 0.5 + 0.5).dpowf(1.5);
                    mix(ivory, [0.55, 0.40, 0.26], band * 0.7)
                }
            };
            let grime = 0.85 + 0.15 * fbm(seed ^ 9, u * 10.0, v * 5.0, 2);
            opaque(o.map(|c| c * grime))
        });
    })
}

/// The printed paper dial (r 142 mm) as its face is mapped: the sun's face
/// at its middle, the planets' orbits, the zodiac's signs and degrees, the
/// calendar's months and days, a ring of the compass's points; coloured by
/// hand, browned under its varnish, foxed, a ring of dust where the drum's
/// edge kept the cloth that covered it
fn or_dial_tex(vary: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = vary ^ 0xd1a;
        let half = 0.142;
        let px = w / (2.0 * half);
        let (cx, cy) = (w / 2.0, h / 2.0);
        fill(ctx, w, h, |u, v| {
            let (x, z) = ((u - 0.5) * 2.0 * half, (0.5 - v) * 2.0 * half);
            let r = (x * x + z * z).sqrt();
            let a = z.datan2(x).to_degrees().rem_euclid(360.0);
            let mut o = [0.82, 0.76, 0.60];
            // the zodiac's ring washed in three colours by hand, the
            // calendar's in a pale blue
            if (0.110..0.126).contains(&r) {
                let tints = [[0.84, 0.70, 0.52], [0.70, 0.76, 0.62], [0.82, 0.66, 0.60]];
                o = tints[((a / 30.0).floor() as usize) % 3];
            } else if (0.126..0.140).contains(&r) {
                o = [0.70, 0.75, 0.76];
            }
            opaque(o)
        });
        let ink = "rgba(34,26,18,0.9)";
        ctx.set_stroke_style(ink);
        for (k, rr) in [0.140, 0.137, 0.126, 0.123, 0.110, 0.107, 0.050, 0.075, 0.104].iter().enumerate() {
            ctx.set_line_width(if k < 6 { 1.0 } else { 0.6 });
            ctx.begin_path();
            ctx.arc(cx, cy, rr * px, 0, PI * 2.0);
            ctx.stroke();
        }
        ctx.set_line_width(0.6);
        for k in 0..360 {
            let a = (k as f64).to_radians();
            let (c, s) = (a.dcos(), a.dsin());
            let l: f64 = if k % 30 == 0 {
                0.016
            } else if k % 10 == 0 {
                0.0045
            } else if k % 5 == 0 {
                0.0032
            } else {
                0.002
            };
            ctx.begin_path();
            ctx.move_to(cx + c * 0.110 * px, cy + s * 0.110 * px);
            ctx.line_to(cx + c * (0.110 + l) * px, cy + s * (0.110 + l) * px);
            ctx.stroke();
        }
        // the sun's face at its middle: rays round a disc
        for k in 0..24 {
            let a = k as f64 * PI / 12.0;
            let (r0, r1) = (0.013 * px, if k % 2 == 0 { 0.026 } else { 0.020 } * px);
            ctx.begin_path();
            ctx.move_to(cx + (a - 0.06).dcos() * r0, cy + (a - 0.06).dsin() * r0);
            ctx.line_to(cx + a.dcos() * r1, cy + a.dsin() * r1);
            ctx.line_to(cx + (a + 0.06).dcos() * r0, cy + (a + 0.06).dsin() * r0);
            ctx.set_fill_style("rgba(170,120,40,0.85)");
            ctx.fill();
        }
        ctx.set_fill_style("rgba(196,150,60,0.9)");
        ctx.begin_path();
        ctx.arc(cx, cy, 0.013 * px, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let words = |ctx: &mut Canvas, list: &[&str], r: f64, step: f64, size: f64, off: f64| {
            ctx.set_font(&format!("{size:.0}px Times New Roman"));
            for (k, t) in list.iter().enumerate() {
                let a = (off + k as f64 * step).to_radians();
                ctx.save();
                ctx.translate(cx + a.dcos() * r * px, cy + a.dsin() * r * px);
                ctx.rotate(a + PI / 2.0);
                ctx.fill_text(t, 0, 0);
                ctx.restore();
            }
        };
        let signs =
            ["ARIES", "TAURUS", "GEMINI", "CANCER", "LEO", "VIRGO", "LIBRA", "SCORPIO", "SAGITTAR.", "CAPRICORN", "AQUARIUS", "PISCES"];
        words(ctx, &signs, 0.1175, 30.0, 10.0, 15.0);
        let months =
            ["MARCH", "APRIL", "MAY", "JUNE", "JULY", "AUGUST", "SEPTEMBER", "OCTOBER", "NOVEMBER", "DECEMBER", "JANUARY", "FEBRUARY"];
        words(ctx, &months, 0.1315, 30.0, 9.0, 25.0);
        let planets = ["MERCURY", "VENUS", "THE EARTH"];
        for (k, (t, r)) in planets.iter().zip([0.050, 0.075, 0.104]).enumerate() {
            words(ctx, &[t], r - 0.004, 0.0, 8.0, 200.0 + k as f64 * 40.0);
        }
        // browned under its varnish, foxed, the drum's dust ring
        let src = ctx.get_image_data(0, 0, w, h);
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let k = (j * n + i) * 4;
                let (u, v) = (i as f64 / w, j as f64 / h);
                let r = ((u - 0.5).dpowi(2) + (v - 0.5).dpowi(2)).sqrt() * 2.0;
                let mut o = [0, 1, 2].map(|c| f64::from(src.data[k + c]) / 255.0);
                let am = 0.82 + 0.16 * fbm(seed, u * 7.0, v * 7.0, 3);
                o = [o[0] * 0.92 * am, o[1] * 0.80 * am, o[2] * 0.60 * am];
                let fox = sstep(0.80, 0.9, fbm(seed ^ 3, u * 40.0, v * 40.0, 2));
                o = o.map(|c| c * (1.0 - 0.3 * fox));
                let dust = 0.10 + 0.12 * fbm(seed ^ 5, u * 6.0, v * 6.0, 3) + 0.12 * sstep(0.85, 0.98, r);
                o = mix(o, [0.45, 0.42, 0.37], dust);
                for c in 0..3 {
                    out[i * 4 + c] = (o[c].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Wood seen along its grain (u along it): the growth rings' streaks and
/// the figure, `base` its colour (sRGB)
fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
    let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
    let ring = ((v * across + warp * 6.0).dsin() * 0.5 + 0.5).dpowf(3.0);
    let streak = fbm(seed, u * along * 0.2, v * across, 3);
    let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
    base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
}

/// Tarnished metal's mottle (a multiplier near 1)
fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
    0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * sstep(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
}
