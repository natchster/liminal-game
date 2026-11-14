//! `place_soil_bags`
use crate::shared::blob;
use crate::shared::css;
use crate::shared::fbm_tile;
use crate::shared::over;
use game::prelude::*;
use placers_kit::garden::put_uv;
use placers_kit::garden::top;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// Bag of soil: 6% chance per level.
pub(crate) fn place_soil_bags(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // An 8 dry quart bag of potting soil lying on its back where it was put
    // down, slumped, faded and dusty, split at a corner where soil has run
    // out onto the floor (Blender batch `t06`)
    let bag = group();
    // where it lies (the old draws) and how it is turned (the old draw)
    bag.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bag.rotation().y = rnd() * PI * 2.0;
    let p = bag.pos();
    let salt = salt_of(p.x, p.z);
    let print = phong!({ map: &soil_print_tex(salt), specular: 0x1c1c1a, shininess: 18 });
    put_uv(&bag, "Real__soilBag", &print, top(-BAG_X, BAG_X, -BAG_Z, BAG_Z));
    put_uv(&bag, "Real__soilBagSeals", &print, top(-BAG_X, BAG_X, -BAG_Z, BAG_Z));
    // the soil run out of the split: peat is matte (no sheen to grey it
    // under a lamp), its map laid on from above over the heap
    put_uv(&bag, "Real__soilBagSpill", &phong!({ map: &soil_tex(salt), specular: 0x0c0c0c, shininess: 6 }), top(0.02, 0.22, 0.15, 0.315));
    // the fine soil and crumbs scattered round the heap
    let scatter = mesh(
        plane_geo!(0.34, 0.3),
        lambert!({ map: &soil_scatter_tex(salt), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    scatter.rotation().x = -PI / 2.0;
    scatter.position().set(0.175, 0.0012, 0.25);
    bag.add(&scatter);
    g.world.add(&bag);
}

// ============================================================ soil bags
/// The bag's film (`nature/garden_things.py` soil_bag): x across, z along, the seals' ends
const BAG_X: f64 = 0.1525;

const BAG_Z: f64 = 0.215;

/// The bag's print, laid on from above over the film (its top at -z):
/// BRIARWOOD (an invented brand) POTTING SOIL on a dark green film, a
/// photograph of flowering pot plants, the net contents; faded, scuffed,
/// dusty, creased, soil smeared out of the split at its +x, +z corner
fn soil_print_tex(salt: u32) -> Tex {
    canvas_tex_opt(320, 452, false, move |ctx, w, h| {
        let seed = salt ^ 0x5011;
        let film = rgb(0x27402a);
        ctx.set_fill_style(css(film, 1.0));
        ctx.fill_rect(0, 0, w, h);
        // the seals: crimped, a band of the print's green
        ctx.set_fill_style(css(rgb(0x1f3322), 1.0));
        ctx.fill_rect(0, 0, w, 24);
        ctx.fill_rect(0, h - 24.0, w, 24);
        ctx.set_stroke_style("rgba(255,255,255,0.07)");
        ctx.set_line_width(1);
        for k in 0..5 {
            for y0 in [4.0, h - 20.0] {
                ctx.begin_path();
                ctx.move_to(0, y0 + k as f64 * 3.5);
                ctx.line_to(w, y0 + k as f64 * 3.5);
                ctx.stroke();
            }
        }
        // the brand on a cream band
        ctx.set_fill_style(css(rgb(0xe6dec6), 1.0));
        ctx.fill_rect(0, 42, w, 50);
        ctx.set_fill_style(css(rgb(0x2a472c), 1.0));
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 36px Rockwell");
        ctx.fill_text_max("BRIARWOOD", w / 2.0, 80, w - 60.0);
        // a sprig each side of it
        for sx in [-1.0, 1.0] {
            let cx = w / 2.0 + sx * 136.0;
            ctx.set_stroke_style(css(rgb(0x2a472c), 1.0));
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.move_to(cx, 84);
            ctx.line_to(cx, 52);
            ctx.stroke();
            for k in 0..3 {
                blob(ctx, cx + sx * 5.0, 58.0 + k as f64 * 8.0, 6.0, 0.45, sx * 0.6, rgb(0x2a472c), 1.0);
                blob(ctx, cx - sx * 5.0, 62.0 + k as f64 * 8.0, 6.0, 0.45, -sx * 0.6, rgb(0x2a472c), 1.0);
            }
        }
        ctx.set_fill_style(css(rgb(0xf0e9d4), 1.0));
        ctx.set_font("bold 62px Futura");
        ctx.fill_text_max("POTTING", w / 2.0, 156, w - 22.0);
        ctx.fill_text_max("SOIL", w / 2.0, 214, w - 22.0);
        ctx.set_fill_style(css(rgb(0xc8bf62), 1.0));
        ctx.set_font("bold 15px Futura");
        ctx.fill_text_max("ALL PURPOSE  \u{b7}  INDOOR & OUTDOOR PLANTS", w / 2.0, 238, w - 34.0);
        // the photograph: pot plants in flower on a bench, in a greenhouse
        let (px, py, pw, ph) = (32.0, 250.0, w - 64.0, 132.0);
        ctx.save();
        ctx.begin_path();
        ctx.rect(px, py, pw, ph);
        ctx.clip();
        let gr = ctx.create_linear_gradient(0, py, 0, py + ph);
        gr.add_color_stop(0, &css(rgb(0xb7c49a), 1.0));
        gr.add_color_stop(0.6, &css(rgb(0x5d7a4a), 1.0));
        gr.add_color_stop(1, &css(rgb(0x6a5444), 1.0));
        ctx.set_fill_style(&gr);
        ctx.fill_rect(px, py, pw, ph);
        let mut r = SplitMix64::new(7, 0x50f0);
        // the greenhouse behind, out of focus
        for _ in 0..40 {
            let (x, y) = (px + r.next() * pw, py + r.next() * ph * 0.6);
            blob(ctx, x, y, r.range(8.0, 22.0), 1.0, 0.0, mix(rgb(0x6f8c56), rgb(0xd7dcb4), r.next()), 0.35);
        }
        // three pots on the bench, their plants and flowers
        for (k, cx) in [px + 52.0, px + 128.0, px + 202.0].into_iter().enumerate() {
            let base = py + ph - 8.0;
            let (tw, bw, pot_h) = (34.0, 24.0, 30.0);
            let pg = ctx.create_linear_gradient(cx - tw, 0, cx + tw, 0);
            pg.add_color_stop(0, &css(rgb(0x7a3e24), 1.0));
            pg.add_color_stop(0.4, &css(rgb(0xc0704a), 1.0));
            pg.add_color_stop(1, &css(rgb(0x6a3420), 1.0));
            ctx.set_fill_style(&pg);
            ctx.begin_path();
            ctx.move_to(cx - tw, base - pot_h);
            ctx.line_to(cx + tw, base - pot_h);
            ctx.line_to(cx + bw, base);
            ctx.line_to(cx - bw, base);
            ctx.close_path();
            ctx.fill();
            ctx.fill_rect(cx - tw - 3.0, base - pot_h - 6.0, (tw + 3.0) * 2.0, 7);
            // leaves
            for _ in 0..26 {
                let a = r.range(-2.9, -0.25);
                let d = r.range(6.0, 40.0);
                blob(
                    ctx,
                    cx + a.cos() * d,
                    base - pot_h - 6.0 + a.sin() * d * 0.75,
                    r.range(6.0, 11.0),
                    0.6,
                    a,
                    mix(rgb(0x2c4a22), rgb(0x6f9a46), r.next()),
                    0.85,
                );
            }
            // the flowers: geranium heads, petunias
            let fl = [rgb(0xc0283a), rgb(0xd8a0c0), rgb(0xe8e0d0)][k];
            for _ in 0..9 {
                let a = r.range(-2.7, -0.4);
                let d = r.range(14.0, 38.0);
                let (fx, fy) = (cx + a.cos() * d, base - pot_h - 10.0 + a.sin() * d * 0.8);
                for _ in 0..5 {
                    blob(
                        ctx,
                        fx + r.range(-5.0, 5.0),
                        fy + r.range(-4.0, 4.0),
                        r.range(3.0, 5.5),
                        1.0,
                        0.0,
                        mix(fl, [1.0, 1.0, 1.0], r.next() * 0.3),
                        0.9,
                    );
                }
            }
        }
        ctx.restore();
        // a thin white keyline round the photograph
        ctx.set_stroke_style("rgba(240,233,212,0.9)");
        ctx.set_line_width(2);
        ctx.stroke_rect(px, py, pw, ph);
        // the starburst
        ctx.set_fill_style(css(rgb(0xd9b640), 1.0));
        ctx.begin_path();
        let (sx, sy) = (w - 58.0, 262.0);
        for k in 0..32 {
            let rr = if k % 2 == 0 { 36.0 } else { 29.0 };
            let a = k as f64 / 32.0 * PI * 2.0;
            if k == 0 {
                ctx.move_to(sx + a.cos() * rr, sy + a.sin() * rr);
            } else {
                ctx.line_to(sx + a.cos() * rr, sy + a.sin() * rr);
            }
        }
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style(css(rgb(0x2a472c), 1.0));
        ctx.set_font("bold 12px Futura");
        ctx.fill_text_max("ENRICHED", sx, sy - 2.0, 56);
        ctx.set_font("10px Futura");
        ctx.fill_text_max("WITH PEAT", sx, sy + 11.0, 56);
        ctx.set_fill_style(css(rgb(0xf0e9d4), 1.0));
        ctx.set_font("bold 22px Futura");
        ctx.fill_text_max("8 DRY QT (8.8 L)", w / 2.0, 412, w - 50.0);
        // worn: scuffs and creases where the film folded and was dragged
        let mut s = SplitMix64::new(salt, 0x5cf);
        ctx.set_line_cap("round");
        for _ in 0..70 {
            let (x, y) = (s.next() * w, s.next() * h);
            let a = s.range(-0.6, 0.6) + if s.next() < 0.5 { 0.0 } else { PI / 2.0 };
            let l = s.range(6.0, 40.0);
            ctx.set_stroke_style(format!("rgba(220,215,200,{:.2})", s.range(0.05, 0.22)));
            ctx.set_line_width(s.range(0.6, 1.8));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..7 {
            let (x, y) = (s.next() * w, s.range(30.0, h - 30.0));
            let a = s.range(0.0, PI);
            let l = s.range(40.0, 140.0);
            for (st, off) in [("rgba(0,0,0,0.25)", 1.2), ("rgba(255,255,240,0.18)", -0.6)] {
                ctx.set_stroke_style(st);
                ctx.set_line_width(1.4);
                ctx.begin_path();
                ctx.move_to(x + off, y + off);
                ctx.quadratic_curve_to(
                    x + a.cos() * l * 0.5 + s.range(-8.0, 8.0),
                    y + a.sin() * l * 0.5,
                    x + a.cos() * l + off,
                    y + a.sin() * l + off,
                );
                ctx.stroke();
            }
        }
        // the split at the bottom right corner: soil smeared out of it
        for _ in 0..30 {
            blob(
                ctx,
                w - s.range(4.0, 60.0),
                h - s.range(4.0, 46.0),
                s.range(4.0, 14.0),
                s.range(0.4, 1.0),
                s.range(0.0, 3.0),
                rgb(0x2a1f17),
                s.range(0.3, 0.8),
            );
        }
        ctx.set_stroke_style("rgba(12,9,6,0.95)");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(w - 52.0, h - 26.0);
        ctx.line_to(w - 30.0, h - 30.0);
        ctx.line_to(w - 14.0, h - 22.0);
        ctx.line_to(w - 4.0, h - 34.0);
        ctx.stroke();
        // faded by years of light, dust settled in a film, darker toward the
        // floor at its edges
        over(ctx, w, h, |u, v, c| {
            let n = fbm(seed, u * 6.0, v * 8.0, 3);
            let mut o = mix(c, [0.48, 0.5, 0.44], 0.22 + 0.12 * n);
            o = mix(o, [0.56, 0.53, 0.47], 0.1 + 0.18 * fbm(seed ^ 9, u * 14.0, v * 18.0, 3));
            let speck = (vn(seed ^ 4, u * 260.0, v * 360.0) - 0.84).max(0.0) * 3.0;
            o = mix(o, [0.7, 0.67, 0.6], speck);
            let edge = sstep(0.06, 0.0, u.min(1.0 - u));
            mix(o, [0.16, 0.13, 0.1], edge * 0.4)
        });
    })
}

/// Soil out of a bag, from above over the heap (200 x 165 mm): dark peat,
/// crumbs and fibres of bark, the white grains of perlite; dried paler on
/// top in patches
fn soil_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x0e17;
        fill(ctx, w, h, |u, v| {
            // peat: near black-brown, dried a dusty grey-brown in patches
            let n = fbm_tile(seed, u * 8.0, v * 8.0, 8.0, 4);
            let mut c = mix([0.1, 0.08, 0.065], [0.27, 0.22, 0.18], n);
            let dry = fbm_tile(seed ^ 5, u * 3.0, v * 3.0, 3.0, 2);
            c = mix(c, [0.34, 0.3, 0.26], sstep(0.55, 0.75, dry) * 0.45);
            let bark = (vn(seed ^ 2, u * 40.0, v * 40.0) - 0.75).max(0.0) * 3.0;
            c = mix(c, [0.3, 0.18, 0.1], bark);
            let perl = (vn(seed ^ 3, u * 70.0, v * 70.0) - 0.92).max(0.0) * 9.0;
            c = mix(c, [0.74, 0.73, 0.68], perl.min(1.0));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Soil scattered on the floor round the spilled heap (its middle at the
/// split): fine soil in a soft drift, crumbs flung further, clear beyond
fn soil_scatter_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 224, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let mut r = SplitMix64::new(salt, 0x5ca7);
        // the drift: soft blobs thinning out from the split (at the top left)
        for _ in 0..60 {
            let a = r.range(-0.2, 1.7);
            let d = r.next().powf(0.7) * 0.75;
            let (x, y) = (w * 0.3 + a.cos() * d * w * 0.6, h * 0.25 + a.sin() * d * h * 0.7);
            blob(
                ctx,
                x,
                y,
                r.range(8.0, 26.0) * (1.0 - 0.6 * d),
                r.range(0.5, 1.0),
                r.range(0.0, 3.0),
                rgb(0x1e1914),
                r.range(0.15, 0.4) * (1.0 - d),
            );
        }
        // crumbs and bits of bark, perlite
        for _ in 0..140 {
            let a = r.range(-0.4, 1.9);
            let d = r.next().powf(0.6);
            let (x, y) = (w * 0.3 + a.cos() * d * w * 0.65, h * 0.25 + a.sin() * d * h * 0.75);
            // a grain of perlite now and then, dulled with the dust
            let c = if r.next() < 0.05 { [0.62, 0.6, 0.56] } else { mix(rgb(0x161210), rgb(0x3a2e24), r.next()) };
            ctx.set_fill_style(css(c, r.range(0.6, 0.95)));
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.6, 2.2), 0, 7);
            ctx.fill();
        }
    })
}
