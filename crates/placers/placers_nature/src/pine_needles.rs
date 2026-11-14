//! `place_pine_needles`
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mark_mat;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;

// Pine needle cluster: 7% chance per level. Needles a Scots pine shed
// (Blender family `a_garden_and_a_wood_indoors`): pairs still in their papery sheaths and single
// needles broken from them, rust brown, grey-brown or dark with damp, on a
// drift of their own broken bits and dust.
pub(crate) fn place_pine_needles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let needle_colors = [0x304018, 0x405020, 0x203010];
    // shed: rust brown, grey-brown, dark brown with damp (the old greens)
    const SHED: [u32; 3] = [0x7a4a24, 0x6a5a44, 0x4a3420];
    let sheath = lambert!({ color: 0x6e645a });
    let mats: Vec<Mat> = SHED.iter().map(|&c| lambert!({ color: c })).collect();
    let cluster = group();
    let count = 10 + (rnd() * 10.0).floor() as i32;
    for i in 0..count {
        // the old needle's draws: its colour, its turn, where it lies
        let col = choice(&needle_colors);
        let k = needle_colors.iter().position(|&c| c == col).unwrap_or(0);
        let yaw = rnd() * PI * 2.0;
        let (nx, nz) = ((rnd() - 0.5) * 0.14, (rnd() - 0.5) * 0.14);
        // pairs mostly, a single broken from its pair every third
        let w = ["A", "B", "C"][(i % 3) as usize];
        let nd = group();
        if let Some(geo) = part(&format!("Real__pineNeedles{w}")) {
            nd.add(&mesh(geo, &mats[k]));
        }
        if w != "C"
            && let Some(geo) = part(&format!("Real__pineSheath{w}"))
        {
            nd.add(&mesh(geo, &sheath));
        }
        // one lying over another sits on it: a needle's thickness higher
        // for each laid before it near it (no draw)
        nd.position().set(nx, 0.00035 * (i % 4) as f64, nz);
        nd.rotation().y = yaw;
        cluster.add(&nd);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    let size = 0.30;
    let drift = mesh(plane_geo!(size, size), mark_mat(&needle_drift_tex(salt_of(px, pz), size)));
    drift.rotation().x = -PI / 2.0;
    drift.position().y = 0.0006;
    cluster.add(&drift);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

/// `rgba(…)` of an sRGB colour (0-1) at alpha `a`
fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &css(c, a));
    gr.add_color_stop(1, &css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A small deterministic stream of its own (for a painter's strokes, mixed
/// with where the thing lies): no level draw
struct Mini(u64);

impl Mini {
    fn new(seed: u32) -> Mini {
        Mini(u64::from(seed).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.next()
    }
}

// =========================================================== pine needles
/// A drift of shed needles on the floor, `size` m across: broken needles
/// and their bits lying every way, finer toward its edge, in a film of the
/// grit and dust that came in with them; soft and uneven (a mark, `r07`)
fn needle_drift_tex(salt: u32, size: f64) -> Tex {
    let px = 384.0;
    canvas_tex_opt(px, px, false, move |ctx, w, h| {
        let mut r = Mini::new(salt ^ 0x9ee);
        ctx.clear_rect(0, 0, w, h);
        let ppm = w / size;
        // the dust and grit under it, in soft patches
        for _ in 0..14 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.range(0.0, 0.32) * w);
            let (x, y) = (w / 2.0 + a.cos() * d, h / 2.0 + a.sin() * d);
            blob(ctx, x, y, r.range(0.05, 0.16) * w, r.range(0.5, 1.0), r.range(0.0, PI), [0.30, 0.26, 0.20], r.range(0.10, 0.22));
        }
        // the bits: needles broken short, every way, densest in the middle
        ctx.set_line_cap("round");
        for i in 0..240 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.next().powf(0.8) * 0.45 * w);
            let (x, y) = (w / 2.0 + a.cos() * d, h / 2.0 + a.sin() * d);
            // whole needles (a third) and the bits of broken ones
            let len = if i % 3 == 0 { r.range(0.036, 0.055) } else { r.range(0.005, 0.025) } * ppm * (1.0 - 0.4 * d / (0.45 * w));
            let t = r.range(0.0, 2.0 * PI);
            let tone = r.next();
            let c = mix_unclamped([0.36, 0.25, 0.15], [0.27, 0.22, 0.17], tone);
            let fade = 1.0 - sstep(0.25, 0.45, d / w);
            ctx.set_stroke_style(css(c, 0.8 * fade));
            ctx.set_line_width(if i % 3 == 0 { 2.0 } else { 1.4 });
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = r.range(-0.15, 0.15);
            ctx.quadratic_curve_to(
                x + (t + bend).cos() * len * 0.5,
                y + (t + bend).sin() * len * 0.5,
                x + t.cos() * len,
                y + t.sin() * len,
            );
            ctx.stroke();
        }
        // a few scale flakes and crumbs of bark
        for _ in 0..30 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.next() * 0.4 * w);
            blob(
                ctx,
                w / 2.0 + a.cos() * d,
                h / 2.0 + a.sin() * d,
                r.range(1.0, 2.6),
                r.range(0.4, 1.0),
                r.range(0.0, PI),
                [0.33, 0.22, 0.13],
                0.7,
            );
        }
    })
}
