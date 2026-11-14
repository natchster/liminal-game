//! `place_floor_globes`
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use placers_kit::geometry::put;
use placers_kit::noise::crack;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::vary;

// The floor globe: 3% chance per level. A 12 in terrestrial globe on its
// floor stand (Blender family `a_studys_things`): a library stand of three turned legs
// joined by stretchers to a hub, the wooden horizon ring papered with its
// calendar and zodiac, or a turned pedestal on a cross of four feet; the
// brass meridian, the axis pins, the hour circle at the north pole. The
// ball tilted in its meridian, stopped where it was last turned.
pub(crate) fn place_floor_globes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old draws, call for call: the wood, the stand, the land patches
    // (their colour, place, size, turn and stretch), the old spin's, where it
    // stands and how it is turned
    let wood_colors = [0x604020, 0x8b5a2b, 0x2a1810, 0x8b6030, 0x505030];
    let wood = choice(&wood_colors);
    let is_tripod = rnd() < 0.5;
    let land_count = 8 + (rnd() * 5.0).floor() as i32;
    let land_colors = [0x408030, 0x6a5030, 0x805030, 0xa0a040];
    let mut stop = 0.0;
    for i in 0..land_count {
        let _ = choice(&land_colors); // keeps the old patch colour's draw
        let u = rnd();
        let _v = rnd();
        let _size = rnd();
        let _turn = rnd();
        let _stretch = rnd();
        if i == 0 {
            // where the ball was stopped: the first patch's longitude
            stop = u * PI * 2.0;
        }
    }
    let _ = rnd(); // keeps the old draw of its spin (40%): nothing turns it now
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    let turn = rnd() * PI * 2.0;
    let fg = group();
    fg.position().set(px, 0, pz);
    fg.rotation().y = turn;
    let vary = vary(px, pz, turn);
    // the stand, stained as the old wood's draw
    let stain = gl_stain(wood);
    let stand = phong!({ map: &gl_wood_tex(vary, stain), specular: 0x161412, shininess: 14 });
    if is_tripod {
        put(&fg, "Real__globeTripod", &stand);
        put(&fg, "Real__globeHorizon", &stand);
        put(&fg, "Real__globeHorizonPaper", &lambert!({ map: &gl_horizon_tex(vary) }));
    } else {
        put(&fg, "Real__globePedestal", &stand);
    }
    // the meridian and the axis, tilted; the ball turns in them
    let tilt = group();
    tilt.position().y = GL_CY;
    tilt.rotation().z = GL_TILT;
    fg.add(&tilt);
    let brass = phong!({ map: &gl_meridian_tex(vary), specular: 0x4a3c22, shininess: 30 });
    put(&tilt, "Real__globeMeridian", &brass);
    put(&tilt, "Real__globePins", &phong!({ color: 0x6a5631, specular: 0x4a3c22, shininess: 30 }));
    let ball = group();
    ball.rotation().y = stop;
    tilt.add(&ball);
    put(&ball, "Real__globeBall", &phong!({ map: &gl_map_tex(vary), specular: 0x1c1a16, shininess: 22 }));
    g.world.add(&fg);
}

/// The ball's middle (y) and the axis's tilt in the meridian (23.5 degrees)
const GL_CY: f64 = 0.720;

const GL_TILT: f64 = 0.41;

/// Where the Blaeu map lies in its cell of the atlas (px of the 480 x 400
/// cell): its neatlines west and east (360 degrees), the equator's row,
/// px a unit of Mercator's y, the map's top and foot rows (80.5 N, 70.7 S);
/// measured on the atlas (the polar circles at rows 139.5 and 325, the
/// tropic of Capricorn at 257)
const BL_X0: f64 = 53.0;

const BL_K: f64 = 59.05;

const BL_EQ: f64 = 232.25;

const BL_TOP: f64 = 85.5;

const BL_FOOT: f64 = 336.5;

/// The sheet's two polar insets (centre x, y, radius, px), laid over the
/// South Sea: a globe has none
const BL_INSETS: [(f64, f64, f64); 2] = [(88.8, 298.8, 37.5), (385.0, 299.5, 37.5)];

/// The old wood colours' stains (sRGB): mahogany, walnut, ebonised,
/// golden oak, fumed oak
fn gl_stain(c: u32) -> [f64; 3] {
    match c {
        0x604020 => [0.30, 0.15, 0.08],
        0x8b5a2b => [0.36, 0.24, 0.14],
        0x2a1810 => [0.07, 0.055, 0.045],
        0x8b6030 => [0.46, 0.32, 0.17],
        _ => [0.27, 0.22, 0.15],
    }
}

/// The stand's wood as its parts are mapped (the grain up the legs and the
/// column, v): stained, its polish dulled, rubbed lighter on what sticks
/// out, dust in the grain
fn gl_wood_tex(vary: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x6e0;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, v, u, stain, 6.0, 40.0);
            let rub = sstep(0.62, 0.8, fbm(seed ^ 3, u * 7.0, v * 22.0, 3));
            o = mix(o, stain.map(|c| (c * 1.6 + 0.06).min(0.6)), rub * 0.35);
            let dust = 0.07 + 0.12 * fbm(seed ^ 5, u * 6.0, v * 12.0, 3);
            opaque(mix(o, [0.42, 0.39, 0.34], dust))
        });
    })
}

/// The brass meridian's face as it is mapped (x, y of its plane): dull
/// brass, its degrees engraved on the band, rubbed bright where hands took
/// it at the top, dark in the engraving
fn gl_meridian_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x3e1;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * 2.0 * 0.159, (v - 0.5) * 2.0 * 0.159);
            let r = (x * x + y * y).sqrt();
            let a = y.datan2(x).to_degrees().rem_euclid(360.0);
            let k = tarnish(seed, u, v, 7.0);
            let mut o = [0.42, 0.33, 0.17].map(|c| c * k);
            // the degrees: a tick each, longer each 5 and 10, on the outer half
            let d = (a - a.round()).abs();
            let long = if (a.round() as i64) % 10 == 0 {
                0.0021
            } else if (a.round() as i64) % 5 == 0 {
                0.0015
            } else {
                0.0009
            };
            let rel = 0.1590 - r;
            if (0.0003..long).contains(&rel) && d < 0.14 {
                o = o.map(|c| c * 0.35);
            }
            // rubbed bright at the top where it is held
            o = mix(o, [0.58, 0.47, 0.26], sstep(0.55, 0.95, v) * 0.25 * fbm(seed ^ 1, u * 9.0, v * 9.0, 2));
            opaque(o)
        });
    })
}

/// The horizon's printed paper as its top is mapped: rings of a library
/// globe's horizon, from its outer edge in: the degrees of amplitude, the
/// points of the compass, the signs of the zodiac with their degrees, the
/// calendar's months and days; varnished, gone brown, rubbed through and
/// stained where it was leant on
fn gl_horizon_tex(vary: u32) -> Tex {
    canvas_tex_opt(768, 768, false, move |ctx, w, h| {
        let seed = vary ^ 0x402;
        let half = 0.214;
        let px = w / (2.0 * half);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // rings (m from the middle): the paper's outer edge to its inner
        let rings = [0.2140, 0.2085, 0.2050, 0.1985, 0.1935, 0.1865, 0.1815, 0.1745];
        fill(ctx, w, h, |u, v| {
            let (x, z) = ((u - 0.5) * 2.0 * half, (0.5 - v) * 2.0 * half);
            let r = (x * x + z * z).sqrt();
            let mut o = [0.80, 0.72, 0.55];
            // hand colour washed in the zodiac's ring and the calendar's
            let a = z.datan2(x).to_degrees().rem_euclid(360.0);
            let sector = (a / 30.0).floor() as i32;
            if r > 0.1865 && r < 0.1935 {
                let tints = [[0.80, 0.68, 0.52], [0.70, 0.74, 0.60], [0.78, 0.66, 0.58]];
                o = tints[sector.rem_euclid(3) as usize];
            }
            opaque(o)
        });
        ctx.set_stroke_style("rgba(40,30,20,0.85)");
        for (k, rr) in rings.iter().enumerate() {
            ctx.set_line_width(if k == 0 || k == rings.len() - 1 { 1.6 } else { 0.9 });
            ctx.begin_path();
            ctx.arc(cx, cy, rr * px, 0, PI * 2.0);
            ctx.stroke();
        }
        // ticks: degrees on the outer ring, days on the calendar's
        ctx.set_line_width(0.7);
        for k in 0..360 {
            let a = (k as f64).to_radians();
            let (c, s) = (a.dcos(), a.dsin());
            let len = if k % 10 == 0 {
                0.0055
            } else if k % 5 == 0 {
                0.0040
            } else {
                0.0025
            };
            ctx.begin_path();
            ctx.move_to(cx + c * 0.2085 * px, cy + s * 0.2085 * px);
            ctx.line_to(cx + c * (0.2085 - len) * px, cy + s * (0.2085 - len) * px);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(cx + c * 0.1815 * px, cy + s * 0.1815 * px);
            ctx.line_to(
                cx + c * (0.1815 - if k % 30 == 0 { 0.007 } else { 0.0022 }) * px,
                cy + s * (0.1815 - if k % 30 == 0 { 0.007 } else { 0.0022 }) * px,
            );
            ctx.stroke();
        }
        ctx.set_fill_style("rgba(34,26,18,0.9)");
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
        let compass = ["E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW", "N", "NNE", "NE", "ENE"];
        words(ctx, &compass, 0.2018, 22.5, 8.0, 0.0);
        let signs =
            ["ARIES", "TAURUS", "GEMINI", "CANCER", "LEO", "VIRGO", "LIBRA", "SCORPIO", "SAGITTARIUS", "CAPRICORN", "AQUARIUS", "PISCES"];
        words(ctx, &signs, 0.1900, 30.0, 7.0, 15.0);
        let months =
            ["MARCH", "APRIL", "MAY", "JUNE", "JULY", "AUGUST", "SEPTEMBER", "OCTOBER", "NOVEMBER", "DECEMBER", "JANUARY", "FEBRUARY"];
        words(ctx, &months, 0.1780, 30.0, 7.0, 25.0);
        // the varnish gone amber, rubbed through where it was leant on, a
        // tide mark, dust; computed over what was printed
        let src = ctx.get_image_data(0, 0, w, h);
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let mut rr = SplitMix64::new(vary, 0x40e);
        let lean = rr.next() * PI * 2.0;
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let k = (j * n + i) * 4;
                let (u, v) = (i as f64 / w, j as f64 / h);
                let (x, z) = ((u - 0.5) * 2.0, (v - 0.5) * 2.0);
                let a = z.datan2(x);
                let mut o = [0, 1, 2].map(|c| f64::from(src.data[k + c]) / 255.0);
                // amber varnish, deeper in patches
                let am = 0.8 + 0.2 * fbm(seed, u * 8.0, v * 8.0, 3);
                o = [o[0] * 0.92 * am, o[1] * 0.80 * am, o[2] * 0.58 * am];
                // rubbed through to the pale paper where hands leant on it
                let da = ((a - lean + PI).rem_euclid(PI * 2.0) - PI).abs();
                let rub = sstep(0.7, 0.2, da) * sstep(0.55, 0.75, fbm(seed ^ 3, u * 30.0, v * 30.0, 3));
                o = mix(o, [0.70, 0.64, 0.50], rub * 0.6);
                // foxing and grime
                let fox = sstep(0.80, 0.9, fbm(seed ^ 7, u * 40.0, v * 40.0, 2));
                o = o.map(|c| c * (1.0 - 0.3 * fox));
                let dust = 0.10 + 0.14 * fbm(seed ^ 9, u * 5.0, v * 5.0, 3);
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

/// The ball's gores as the sphere is mapped (u the longitude eastward, v
/// the latitude from the south pole): the Blaeu map sampled back through
/// Mercator's projection, its polar insets taken out (the sea's paper and
/// the graticule carried through), the polar caps beyond the map's edges
/// printed with their meridians and circles, the twelve gores' joins; the
/// varnish gone amber, crazed, the paper rubbed off where hands turned it
/// round its middle, foxed, dust on its top
fn gl_map_tex(vary: u32) -> Tex {
    let none = Age {
        fade: 0.0,
        tone: [1.0, 1.0, 1.0],
        edge: 0.0,
        silver: 0.0,
        spots: 0,
        crease: false,
        stain: 0.0,
        crack: (0.0, 0.0),
        grime: 0.0,
        losses: 0,
        weave: 0.0,
        border: 0.0,
    };
    let sheet = pictures::print_tex("map_world", (0.48, 0.40), 1000.0, none, vary);
    let (sw, sh, data) = {
        let t = sheet.0.borrow();
        let img = t.image.borrow();
        (img.width as usize, img.height as usize, img.data.clone())
    };
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x9e0;
        let at = |x: f64, y: f64| -> [f64; 3] {
            let fx = x.clamp(0.0, (sw - 1) as f64);
            let fy = y.clamp(0.0, (sh - 1) as f64);
            let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(sw - 1), (y0 + 1).min(sh - 1));
            let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
            let p = |x: usize, y: usize, c: usize| f64::from(data[(y * sw + x) * 4 + c]) / 255.0;
            [0, 1, 2].map(|c| {
                let a = p(x0, y0, c) + (p(x1, y0, c) - p(x0, y0, c)) * tx;
                let b = p(x0, y1, c) + (p(x1, y1, c) - p(x0, y1, c)) * tx;
                a + (b - a) * ty
            })
        };
        // the sea's paper, the mean of a band of open South Sea
        let mut sea = [0.0; 3];
        let mut nsea = 0.0;
        for k in 0..40 {
            let c = at(150.0 + k as f64 * 2.0, 300.0);
            sea = [sea[0] + c[0], sea[1] + c[1], sea[2] + c[2]];
            nsea += 1.0;
        }
        let sea = sea.map(|c| c / nsea);
        let ink = [0.22, 0.17, 0.12];
        fill(ctx, w, h, |u, v| {
            let lon = u * PI * 2.0;
            let lat = (v - 0.5) * PI;
            let mx = BL_X0 + lon * BL_K;
            let my = BL_EQ - (PI / 4.0 + lat / 2.0).dtan().dln() * BL_K;
            let grat = |lon_deg: f64, lat_deg: f64, every_lon: f64, every_lat: f64| -> f64 {
                let a = ((lon_deg / every_lon) - (lon_deg / every_lon).round()).abs() * every_lon * lat.dcos().max(0.05);
                let b = ((lat_deg / every_lat) - (lat_deg / every_lat).round()).abs() * every_lat;
                1.0 - sstep(0.0, 0.45, a.min(b))
            };
            let (lon_d, lat_d) = (lon.to_degrees(), lat.to_degrees());
            let mut o = if !(BL_TOP + 1.0..=BL_FOOT - 1.0).contains(&my) {
                // a polar cap: the paper, its meridians every 30 degrees,
                // circles every 10, the pole's little ring
                let mut c = sea;
                let line = grat(lon_d, lat_d, 30.0, 10.0);
                c = mix(c, ink, line * 0.55);
                let pole = (90.0 - lat_d.abs()).abs();
                if (pole - 2.5).abs() < 0.25 {
                    c = mix(c, ink, 0.6);
                }
                c
            } else {
                at(mx, my)
            };
            // the insets: out, the sea's paper and the graticule through it
            for &(ix, iy, ir) in &BL_INSETS {
                let d = ((mx - ix).dpowi(2) + (my - iy).dpowi(2)).sqrt();
                if d < ir + 2.0 {
                    let k = sstep(ir + 2.0, ir - 1.0, d);
                    let mut c = mix(sea, [0.0; 3], 0.04 * fbm(seed ^ 1, u * 60.0, v * 40.0, 2));
                    c = mix(c, ink, grat(lon_d, lat_d, 10.0, 10.0) * 0.35);
                    o = mix(o, c, k);
                }
            }
            // a soft join between the map and the caps
            for edge in [BL_TOP + 1.0, BL_FOOT - 1.0] {
                let d = (my - edge).abs();
                if d < 2.0 {
                    o = mix(o, ink, (1.0 - d / 2.0) * 0.5);
                }
            }
            // the gores' joins every 30 degrees, a hair out of true
            let g = ((lon_d + 0.6 * (lat * 3.0).dsin()) / 30.0).fract();
            let gd = g.min(1.0 - g) * 30.0 * lat.dcos();
            if lat_d.abs() < 72.0 {
                o = o.map(|c| c * (1.0 - 0.18 * (1.0 - sstep(0.0, 0.25, gd))));
            }
            // varnish gone amber, deeper in its runs
            let am = 0.78 + 0.18 * fbm(seed ^ 2, u * 6.0, v * 9.0, 3);
            o = [o[0] * 0.93 * am, o[1] * 0.80 * am, o[2] * 0.55 * am];
            // crazed: the varnish's fine cracks
            let d = crack(seed ^ 4, u * 140.0, v * 70.0);
            o = o.map(|c| c * (1.0 - 0.18 * (1.0 - sstep(0.0, 0.05, d))));
            // rubbed round its middle by the hands that turned it: the
            // varnish and the print worn to the pale paper, grime round it
            let band = 1.0 - sstep(0.0, 0.45, lat.abs());
            let rub = band * sstep(0.62, 0.8, fbm(seed ^ 6, u * 24.0, v * 14.0, 3));
            o = mix(o, [0.72, 0.66, 0.52], rub * 0.45);
            o = o.map(|c| c * (1.0 - 0.15 * band * fbm(seed ^ 8, u * 10.0, v * 10.0, 2)));
            // foxing
            let fox = sstep(0.82, 0.92, fbm(seed ^ 10, u * 50.0, v * 25.0, 2));
            o = [o[0] * (1.0 - 0.2 * fox), o[1] * (1.0 - 0.3 * fox), o[2] * (1.0 - 0.4 * fox)];
            // dust settled on its top
            let dust = sstep(0.6, 0.97, v) * (0.10 + 0.20 * fbm(seed ^ 12, u * 12.0, v * 8.0, 3)) + 0.03;
            opaque(mix(o, [0.45, 0.42, 0.37], dust))
        });
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
