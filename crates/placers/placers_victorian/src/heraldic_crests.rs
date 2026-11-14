//! `place_heraldic_crests`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::mount_on_wall;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Framed heraldic coat of arms: 2.5% chance per level. A carved
// armorial plaque of oak (33.6 x 41.6 cm, the old frame's size): a heater
// shield, its edge moulded with a bead and a cove into the field; the
// field gessoed and painted, the ordinary carved proud of it and painted,
// its charges painted; the bead gilt over red bole. Invented arms, of no
// real family: azure a chevron between three acorns or; gules a bend
// argent between two mullets or; vert a cross argent between four
// annulets or. The paint crazed, chipped to the gesso at its knocks, the gold
// rubbed to the bole, dust on the top of the bead.
pub(crate) fn place_heraldic_crests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old frame's colour: the oak's stain
    let frame_colors = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let stain = match choice(&frame_colors) {
        0x2a1810 => [0.16, 0.10, 0.06],
        0x402015 => [0.24, 0.14, 0.08],
        0x604020 => [0.32, 0.21, 0.12],
        _ => [0.44, 0.32, 0.20],
    };
    let hc = group();
    mount_on_wall(g, &hc, &c, 0.007, 1.60, 0.4);
    hc.rotation().z = (rnd() - 0.5) * 0.020;
    let p = hc.pos();
    let k = salt(p.x, p.z, 31);
    let arms = (k / 7 % 3) as usize;
    put(&hc, "Real__crestBoard", &phong!({ map: &crest_board_tex(k, stain), specular: 0x1e1a12, shininess: 20 }));
    put(&hc, "Real__crestField", &lambert!({ map: &crest_field_tex(k, arms) }));
    let (ordinary, metal) = match arms {
        0 => ("Real__crestChevron", true),
        1 => ("Real__crestBend", false),
        _ => ("Real__crestCross", false),
    };
    let tex = crest_tincture_tex(k, metal);
    let mat = if metal { phong!({ map: &tex, specular: 0x4a3c22, shininess: 30 }) } else { lambert!({ map: &tex }) };
    put(&hc, ordinary, &mat);
    g.world.add(&hc);
}

/// The crest's outline (`victorian/things_on_a_wall_and_one_box.py` `crest_outline`): half its width, its top
/// and point, its top corners' radius; the arcs' radius and centre
const CREST_HW: f64 = 0.168;

const CREST_TOP: f64 = 0.208;

const CREST_RC: f64 = 0.022;

const CREST_R: f64 = 0.212_762;

const CREST_CX: f64 = CREST_HW - CREST_R;

/// The field part's bounds (x, y): its texture is laid across them
const FIELD_X: (f64, f64) = (-0.1490, 0.1490);

const FIELD_Y: (f64, f64) = (-0.1883, 0.1890);

/// How far in from the crest's outer edge a point of its face is (m)
fn crest_in(x: f64, y: f64) -> f64 {
    let ax = x.abs();
    let side = if y > 0.0 {
        if ax > CREST_HW - CREST_RC && y > CREST_TOP - CREST_RC {
            CREST_RC - ((ax - (CREST_HW - CREST_RC)).powi(2) + (y - (CREST_TOP - CREST_RC)).powi(2)).sqrt()
        } else {
            CREST_HW - ax
        }
    } else {
        CREST_R - ((ax - CREST_CX).powi(2) + y * y).sqrt()
    };
    side.min(CREST_TOP - y)
}

/// The board (u across its 33.6 cm, v up its 41.6 cm): oak under its old
/// stain on the edge and in the cove, the bead gilt over red bole, the
/// gold rubbed through on its crest and darkened in the hollow beside it,
/// dust along the top
fn crest_board_tex(salt: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(224, 278, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * 2.0 * CREST_HW, (v - 0.5) * 2.0 * CREST_TOP);
            let d = crest_in(x, y);
            let grain = fbm(seed, x * 6.0, y * 140.0 + fbm(seed ^ 3, x * 9.0, y * 3.0, 2) * 4.0, 3);
            let mut c = mix(stain, stain.map(|k| k * 0.6), sstep(0.45, 0.72, grain) * 0.7);
            let bead = sstep(0.0030, 0.0040, d) * (1.0 - sstep(0.0100, 0.0112, d));
            if bead > 0.0 {
                let crest = 1.0 - ((d - 0.0066) / 0.0035).abs().min(1.0);
                let mut gold = [0.50, 0.39, 0.19];
                gold = mix(gold, [0.36, 0.28, 0.14], fbm(seed ^ 5, x * 40.0, y * 40.0, 2) * 0.5);
                let worn = crest * sstep(0.45, 0.7, fbm(seed ^ 7, x * 90.0, y * 90.0, 2));
                gold = mix(gold, [0.40, 0.19, 0.11], worn * 0.8);
                c = mix(c, gold, bead);
            }
            // grime in the cove's hollow
            let cove = sstep(0.0105, 0.0125, d) * (1.0 - sstep(0.0150, 0.0175, d));
            c = c.map(|k| k * (1.0 - 0.3 * cove));
            // dust on what faces up: the top's bead and edge
            let top = 1.0 - sstep(0.0, 0.012, CREST_TOP - y);
            let dd = fbm(seed ^ 9, x * 50.0, y * 50.0, 2);
            c = mix(c, [0.52, 0.49, 0.44], (0.05 + 0.3 * top) * (0.6 + 0.8 * dd));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The field (u, v across the field part's bounds): its tincture over
/// gesso, its charges painted with a dark line round them, then aged: the
/// paint crazed into islands, chipped to the gesso, darkened, dust
fn crest_field_tex(salt: u32, arms: usize) -> Tex {
    let ppm = 700.0;
    let (fw, fh) = (FIELD_X.1 - FIELD_X.0, FIELD_Y.1 - FIELD_Y.0);
    canvas_tex_opt((fw * ppm).round(), (fh * ppm).round(), false, move |ctx, w, h| {
        let px = |x: f64| (x - FIELD_X.0) * ppm;
        let py = |y: f64| (FIELD_Y.1 - y) * ppm;
        let (field, charge) = match arms {
            0 => ("#24385c", "#a88a48"),
            1 => ("#7a2621", "#a88a48"),
            _ => ("#2f4a2c", "#a88a48"),
        };
        ctx.set_fill_style(field);
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style(charge);
        ctx.set_stroke_style("rgba(20,14,10,0.8)");
        ctx.set_line_width(1.2);
        match arms {
            0 => {
                // three acorns, slipped: two in chief, one in base
                for &(x, y) in &[(-0.080, 0.120), (0.080, 0.120), (0.0, -0.088)] {
                    acorn(ctx, px(x), py(y), 0.030 * ppm);
                }
            }
            1 => {
                // two mullets: in sinister chief and in dexter base
                for &(x, y) in &[(0.078, 0.110), (-0.074, -0.080)] {
                    mullet(ctx, px(x), py(y), 0.036 * ppm);
                }
            }
            _ => {
                // four annulets, one in each quarter
                for &(x, y) in &[(-0.088, 0.112), (0.088, 0.112), (-0.074, -0.058), (0.074, -0.058)] {
                    ctx.set_fill_style(charge);
                    ctx.begin_path();
                    ctx.arc(px(x), py(y), 0.020 * ppm, 0, PI * 2.0);
                    ctx.arc_dir(px(x), py(y), 0.0125 * ppm, PI * 2.0, 0, true);
                    ctx.fill();
                    ctx.stroke();
                }
            }
        }
        age_paint(ctx, w, h, salt ^ 0x71, 1.0);
    })
}

/// An acorn on its stalk, the cup hatched
fn acorn(ctx: &mut Canvas, x: f64, y: f64, s: f64) {
    ctx.begin_path();
    ctx.ellipse(x, y + s * 0.15, s * 0.42, s * 0.62, 0, 0, PI * 2.0);
    ctx.fill();
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(x - s * 0.55, y - s * 0.30);
    ctx.quadratic_curve_to(x, y - s * 0.95, x + s * 0.55, y - s * 0.30);
    ctx.quadratic_curve_to(x, y - s * 0.05, x - s * 0.55, y - s * 0.30);
    ctx.fill();
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(x, y - s * 0.72);
    ctx.quadratic_curve_to(x + s * 0.1, y - s * 1.05, x + s * 0.35, y - s * 1.15);
    ctx.set_line_width(s * 0.12);
    ctx.stroke();
    ctx.set_line_width(1.2);
    // the cup's scales
    ctx.set_stroke_style("rgba(40,28,14,0.55)");
    for k in 0..4 {
        let t = -0.4 + k as f64 * 0.27;
        ctx.begin_path();
        ctx.move_to(x + s * t, y - s * 0.62);
        ctx.line_to(x + s * (t + 0.22), y - s * 0.30);
        ctx.stroke();
    }
    ctx.set_stroke_style("rgba(20,14,10,0.8)");
}

/// A five-pointed mullet
fn mullet(ctx: &mut Canvas, x: f64, y: f64, r: f64) {
    ctx.begin_path();
    for k in 0..10 {
        let a = -PI / 2.0 + k as f64 * PI / 5.0;
        let rr = if k % 2 == 0 { r } else { r * 0.40 };
        if k == 0 {
            ctx.move_to(x + rr * a.cos(), y + rr * a.sin());
        } else {
            ctx.line_to(x + rr * a.cos(), y + rr * a.sin());
        }
    }
    ctx.close_path();
    ctx.fill();
    ctx.stroke();
}

/// The ordinary's paint: gold (an oil gilding, dulled) or silver (white
/// lead gone cream), crazed, chipped, dusty
fn crest_tincture_tex(salt: u32, metal: bool) -> Tex {
    canvas_tex_opt(128, 160, false, move |ctx, w, h| {
        ctx.set_fill_style(if metal { "#9c7e42" } else { "#c8c2b0" });
        ctx.fill_rect(0, 0, w, h);
        age_paint(ctx, w, h, salt ^ 0x83, 0.6);
    })
}

/// Paint on gesso over oak, aged: crazed into islands along fine dark
/// cracks, chipped to the white gesso (and through it to the wood) here and
/// there, darkened unevenly, dust; `scale` how many pixels a centimetre
fn age_paint(ctx: &mut Canvas, w: f64, h: f64, seed: u32, scale: f64) {
    let src = ctx.get_image_data(0, 0, w, h);
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let k = (j * n + i) * 4;
            let mut c = [0, 1, 2].map(|q| f64::from(src.data[k + q]) / 255.0);
            let (x, y) = (i as f64 / (7.0 * scale), j as f64 / (7.0 * scale));
            let crack = (vn(seed, x * 1.3, y * 1.3) - 0.5).abs().min((vn(seed ^ 3, x * 2.7, y * 2.7) - 0.5).abs() * 1.4);
            c = mix(c, [0.12, 0.09, 0.06], (1.0 - sstep(0.0, 0.025, crack)) * 0.55);
            let chip = fbm(seed ^ 5, x * 1.7, y * 1.7, 3) * (0.75 + 0.25 * fbm(seed ^ 6, x * 0.2, y * 0.2, 2));
            c = mix(c, [0.64, 0.60, 0.51], sstep(0.705, 0.715, chip));
            c = mix(c, [0.34, 0.24, 0.16], sstep(0.735, 0.745, chip));
            // the varnish over it gone amber, darker where it pooled
            let dark = fbm(seed ^ 7, x * 0.3, y * 0.3, 3);
            c = [c[0] * 0.93, c[1] * 0.86, c[2] * 0.72].map(|q| q * (0.72 + 0.32 * dark));
            let d = vn(seed ^ 9, x * 4.0, y * 4.0);
            c = mix(c, [0.52, 0.50, 0.46], sstep(0.75, 0.95, d) * 0.25 + 0.06);
            for q in 0..3 {
                out[i * 4 + q] = (c[q].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}
