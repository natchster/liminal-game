//! `place_peacock_feather_vases`
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::salt;

// Tall ceramic floor vase with 3-5 peacock feathers (floor): 3% chance.
// A late Victorian earthenware floor vase (45 cm, scaled by the old height
// draw), a baluster on a bare turned foot ring, the narrow neck and the
// trumpet mouth, one glaze (the old colour pick) run thick and dark down
// its body and thin and pale at its edges, crazed, gilt bands rubbed half
// away (the old trim draw); peacock eye feathers stood in it, their herl
// gone bronze and sparse, the eyes still bright, leaning out and dipping.
pub(crate) fn place_peacock_feather_vases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old glaze colour pool: the glaze
    let vase_colors = [0x2a4060, 0x603030, 0x304020, 0x604040, 0x505060];
    let vase_color = choice(&vase_colors);
    let glaze = vase_colors.iter().position(|&c| c == vase_color).unwrap_or(0);
    // the old gold trim: the gilt bands
    let use_gold_trim = rnd() < 0.6;
    let pf = group();
    // the old height: the whole vase scaled to it (40-50 cm)
    let vase_h = 0.40 + rnd() * 0.10;
    let k = vase_h / VASE_H;
    // 3-5 peacock feathers
    let feather_count = 3 + (rnd() * 3.0).floor() as i32;
    let mut feathers = vec![];
    for i in 0..feather_count {
        let angle = (i as f64 / feather_count as f64) * PI * 2.0 + rnd() * 0.3;
        let shaft_len = 0.35 + rnd() * 0.15;
        feathers.push((angle, shaft_len));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    let s = salt(px, pz, 17);
    if let Some(v) = put(&pf, "Real__peacockVase", &phong!({ map: &glaze_tex(s, glaze, use_gold_trim), specular: 0x3a3a3a, shininess: 55 }))
    {
        v.scale().set(k, k, k);
    }
    let vane = lambert!({ map: &feather_tex(s), alphaTest: 0.5, side: DoubleSide });
    let quill = lambert!({ color: 0x6e5c40 });
    for &(angle, shaft_len) in &feathers {
        // its length scales it (88-112 %), the longer leaning out further
        let t = (shaft_len - 0.35) / 0.15;
        let fs = 0.88 + 0.24 * t;
        let pivot = group();
        pivot.position().set(angle.cos() * 0.010 * k, VASE_H * k - FEATHER_FOOT, angle.sin() * 0.010 * k);
        pivot.rotation().y = PI / 2.0 - angle;
        let feather = group();
        feather.rotation().x = 0.16 + 0.12 * t;
        feather.scale().set(fs, fs, fs);
        put(&feather, "Real__peacockFeather", &vane);
        put(&feather, "Real__peacockFeatherShaft", &quill);
        pivot.add(&feather);
        pf.add(&pivot);
    }
    pf.position().set(px, 0, pz);
    pf.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pf);
}

/// The vase as modelled (`victorian/lodge_trophies_and_parlour_ornaments.py` `VASE_H`): the game scales it by the old
/// height draw
const VASE_H: f64 = 0.450;

/// How far down the neck a feather's modelled foot stands
const FEATHER_FOOT: f64 = 0.05;

/// The old glaze colours (straight sRGB)
const GLAZES: [[f64; 3]; 5] = [[0.17, 0.25, 0.38], [0.38, 0.19, 0.19], [0.19, 0.25, 0.13], [0.38, 0.25, 0.25], [0.31, 0.31, 0.38]];

/// Value noise round the vase (u wraps): sampled on a circle in u, so its
/// two edges meet
fn wrap_vn(seed: u32, u: f64, v: f64, fu: f64, fv: f64) -> f64 {
    let a = u * std::f64::consts::TAU;
    let r = fu / std::f64::consts::TAU;
    0.5 * vn(seed, 40.0 + r * a.cos(), v * fv) + 0.5 * vn(seed ^ 0x77, 40.0 + r * a.sin(), v * fv + 13.0)
}

fn wrap_fbm(seed: u32, u: f64, v: f64, fu: f64, fv: f64) -> f64 {
    let mut s = 0.0;
    let mut amp = 0.5;
    let mut n = 0.0;
    let mut f = 1.0;
    for k in 0..3u32 {
        s += amp * wrap_vn(seed.wrapping_add(k * 7919), u, v, fu * f, fv * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// The vase's glaze (u round it, v up it): the glaze run thick and dark
/// in streaks down the body and pooled above the foot, thin and pale where
/// it breaks over the rim, the foot ring's turned shoulder and the
/// shoulder's swell; the bare buff body of the foot ring; crazing stained
/// with dirt; gilt bands rubbed half away; a chip off the rim; dust lying
/// on the shoulder
fn glaze_tex(salt: u32, glaze: usize, gilt: bool) -> Tex {
    let base = GLAZES[glaze.min(4)];
    canvas_tex_opt(256, 192, false, move |ctx, w, h| {
        let seed = salt ^ 0x2a1;
        fill(ctx, w, h, |u, v| {
            let mut c = base;
            // runs: streaks down the body, darker where thick
            let run = wrap_fbm(seed, u, v, 26.0, 3.0);
            c = mix(c, [c[0] * 0.55, c[1] * 0.55, c[2] * 0.6], sstep(0.45, 0.75, run) * 0.8);
            // pooled thick above the foot
            c = mix(c, [base[0] * 0.5, base[1] * 0.5, base[2] * 0.55], sstep(0.12, 0.05, v) * 0.7);
            // breaking pale over the edges: the rim, the shoulder, the foot's turn
            let brk =
                sstep(0.95, 0.99, v) + (1.0 - sstep(0.0, 0.03, (v - 0.73).abs())) * 0.35 + (1.0 - sstep(0.0, 0.02, (v - 0.05).abs())) * 0.5;
            c = mix(c, [0.62, 0.55, 0.44], brk.min(1.0) * 0.45);
            // speckle in the glaze
            let sp = wrap_vn(seed ^ 3, u, v, 300.0, 220.0);
            c = mix(c, [c[0] * 0.6, c[1] * 0.6, c[2] * 0.6], sstep(0.82, 0.92, sp) * 0.6);
            // crazing, dirt in the cracks
            let cz = (wrap_vn(seed ^ 5, u, v, 90.0, 70.0) - 0.5).abs().min((wrap_vn(seed ^ 6, u, v, 60.0, 50.0) - 0.5).abs());
            c = mix(c, [0.14, 0.12, 0.10], (1.0 - sstep(0.0, 0.018, cz)) * 0.45);
            if gilt {
                // three gilt bands and the rim's line, rubbed half away
                let mut b = 0.0f64;
                for y in [0.25, 0.55, 0.85] {
                    b = b.max(1.0 - sstep(0.008, 0.011, (v - y).abs()));
                }
                b = b.max(sstep(0.985, 0.99, v));
                let rub = sstep(0.38, 0.62, wrap_fbm(seed ^ 7, u, v, 30.0, 40.0));
                c = mix(c, [0.58, 0.47, 0.27], b * rub);
                c = mix(c, [0.40, 0.33, 0.21], b * (1.0 - rub) * 0.4);
            }
            // the foot ring left bare: buff earthenware, grimy
            let bare = 1.0 - sstep(0.022, 0.03, v);
            c = mix(c, mix([0.60, 0.50, 0.38], [0.30, 0.26, 0.21], wrap_vn(seed ^ 9, u, v, 40.0, 10.0)), bare);
            // a chip off the rim: the white body
            let chip = (1.0 - sstep(0.0, 0.02, (u - 0.37).abs())) * sstep(0.975, 0.99, v);
            c = mix(c, [0.78, 0.74, 0.66], chip);
            // dust: lying on the shoulder, a film over all of it
            let d = wrap_fbm(seed ^ 11, u, v, 50.0, 40.0);
            let shoulder = sstep(0.64, 0.7, v) * (1.0 - sstep(0.82, 0.86, v));
            c = mix(c, [0.56, 0.53, 0.48], shoulder * (0.25 + 0.3 * sstep(0.4, 0.7, d)) + 0.06);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// One feather's vane (u across it, v up its 0.455 m; the modelled part's
/// bounds): nothing at the quill; the herl, long barbs off the rachis,
/// sparse and bronze-green, longer up toward the eye; the eye at the tip
/// (a kidney of deep blue, rings of teal, bronze and green-gold, a dark
/// edge, the loose barbs round it); the rachis. Its alpha is the barbs.
fn feather_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0x3b5;
        // the vane's size (m): across, up
        let (fw, fh) = (0.162, 0.455);
        let ve = 0.395 / fh;
        // the eye, painted pixel by pixel
        fill(ctx, w, h, |u, v| {
            let dx = (u - 0.5) * fw;
            let dy = (v - ve) * fh;
            // the rings are ellipses taller than wide, the core a kidney
            // dented at its foot
            let r = (dx * dx + (dy / 1.3) * (dy / 1.3)).sqrt();
            let rk = ((dx * 1.1).powi(2) + ((dy - 0.004) / 1.2).powi(2)).sqrt()
                - 0.004 * (1.0 - sstep(0.0, 0.01, dx.abs())) * sstep(0.0, 0.01, -dy);
            let a = dy.atan2(dx);
            let fib = 0.5 + 0.5 * (a * 70.0).sin();
            if r > 0.040 {
                return [0.0, 0.0, 0.0, 0.0];
            }
            let mut c = [0.34, 0.34, 0.16];
            c = mix(c, [0.10, 0.08, 0.05], 1.0 - sstep(0.031, 0.034, r));
            c = mix(c, [0.34, 0.40, 0.18], 1.0 - sstep(0.026, 0.030, r));
            c = mix(c, [0.48, 0.32, 0.16], 1.0 - sstep(0.019, 0.022, r));
            c = mix(c, [0.16, 0.42, 0.42], 1.0 - sstep(0.0145, 0.017, r));
            c = mix(c, [0.08, 0.10, 0.26], 1.0 - sstep(0.0095, 0.0115, rk));
            c = mix(c, [c[0] * 0.7, c[1] * 0.7, c[2] * 0.7], fib * 0.3);
            // the eye's edge frays into loose barbs
            let alpha = if r > 0.034 { if fib > 0.55 { 1.0 } else { 0.0 } } else { 1.0 };
            [c[0], c[1], c[2], alpha]
        });
        // the herl: barbs off the rachis, out and up and dipping at their
        // ends, sparse and short low down, longer toward the eye (each from
        // the salt: every vase's feathers their own)
        ctx.set_line_cap("round");
        let n = 130;
        for (si, side) in [-1.0f64, 1.0].into_iter().enumerate() {
            for i in 0..n {
                let r = |k: i32| hash(seed ^ (k as u32).wrapping_mul(0x9e37), i, si as i32);
                let t = 0.12 + 0.74 * (i as f64 + r(1) * 0.9) / n as f64;
                let reach = (0.010 + 0.052 * sstep(0.12, 0.8, t)) * (0.55 + 0.6 * r(2));
                let lift = reach * (0.35 + 0.35 * r(3));
                let (x0, y0) = (0.5 * w, (1.0 - t) * h);
                let (cx, cy) = (x0 + side * reach * 0.45 / fw * w, y0 - lift * 1.1 / fh * h);
                let x1 = x0 + side * reach / fw * w;
                let y1 = y0 - lift * (0.55 - 0.5 * r(5)) / fh * h;
                // green-bronze, a few gone copper
                let g = 0.75 + 0.35 * r(4);
                let (cr, cg, cb) = if r(6) < 0.25 { (100.0, 78.0, 40.0) } else { (78.0, 86.0, 40.0) };
                let col = format!("rgb({},{},{})", (cr * g) as u8, (cg * g) as u8, (cb * g) as u8);
                ctx.set_stroke_style(col.as_str());
                ctx.set_line_width(1.6);
                ctx.begin_path();
                ctx.move_to(x0, y0);
                ctx.quadratic_curve_to(cx, cy, x1, y1);
                ctx.stroke();
            }
        }
        // the rachis
        ctx.set_stroke_style("rgb(104,90,64)");
        ctx.set_line_width(1.6);
        ctx.begin_path();
        ctx.move_to(0.5 * w, h);
        ctx.line_to(0.5 * w, (1.0 - ve) * h);
        ctx.stroke();
    })
}

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
