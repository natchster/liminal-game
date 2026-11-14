//! `place_wall_crucifixes`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::hang_on_wall;
use placers_kit::place::pick_hex;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Wall crucifix: 5% chance per level. A household wall crucifix of
// the common kind: a cross of two boards half-lapped, 28 x 16 cm, its face's
// edges chamfered, hung from a nail through a hole behind its head; a
// cast corpus nailed on (spelter, its gilding worn to the grey metal on
// what stands out), a titulus of painted tin above it. The wood's finish
// is the old colour's (a dark stain, oak, a walnut stain), rubbed pale on
// its edges by years of dusting, dust along the crossbar's top.
pub(crate) fn place_wall_crucifixes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let cross_colors: [u32; 3] = [0x603820, 0x8a5030, 0x2a1810];
    let stain = pick_hex(&cross_colors);
    let cross = group();
    // Wall placement.
    hang_on_wall(g, &cross, &c, 0.014, 1.70);
    // askew on its nail (the old tilt's draw), turned about the hole it hangs by
    let tilt = (rnd() - 0.5) * 0.08;
    let nail = group();
    nail.position().y = NAIL_Y;
    nail.rotation().z = tilt;
    cross.add(&nail);
    let body = group();
    body.position().y = -NAIL_Y;
    nail.add(&body);
    let p = cross.pos();
    let k = salt(p.x, p.z, 23);
    let wood = match stain {
        0x603820 => [0.30, 0.19, 0.11],
        0x8a5030 => [0.42, 0.28, 0.16],
        _ => [0.13, 0.08, 0.05],
    };
    put(&body, "Real__crucifixCross", &phong!({ map: &cross_tex(k, wood), specular: 0x161412, shininess: 14 }));
    // the corpus: gilt spelter, or one that was always grey
    let corpus = if k % 3 == 0 {
        phong!({ map: &spelter_tex(k, false), specular: 0x4a4a46, shininess: 30 })
    } else {
        phong!({ map: &spelter_tex(k, true), specular: 0x5a4a30, shininess: 30 })
    };
    put(&body, "Real__crucifixCorpus", &corpus);
    put(&body, "Real__crucifixTitle", &lambert!({ map: &title_tex(k) }));
    g.world.add(&cross);
}

/// The crucifix's hole for its nail above the middle (it tilts about it)
const NAIL_Y: f64 = 0.118;

/// The cross's face (u across its 16 cm, v up its 28 cm): the upright's
/// grain up and down, the crossbar's across, the joint between them; the
/// stain worn pale on the edges a duster rubs, dust along the crossbar's
/// top, a few knocks
fn cross_tex(salt: u32, stain: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 224, false, move |ctx, w, h| {
        let seed = salt ^ 0xc1;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * 0.160, (v - 0.5) * 0.280);
            let on_bar = (y - 0.050).abs() < 0.013 && x.abs() > 0.0135;
            let (along, across) = if on_bar { (x, y) } else { (y, x) };
            let g = fbm(seed ^ (on_bar as u32 * 7), along * 9.0, across * 160.0 + fbm(seed ^ 3, along * 4.0, across * 30.0, 2) * 3.0, 3);
            let mut c = mix(stain, stain.map(|k| k * 0.55), sstep(0.45, 0.75, g) * 0.8);
            // the edges: within 3 mm of the cross's outline, rubbed
            let edge = if on_bar {
                (0.080 - x.abs()).min(0.013 - (y - 0.050).abs())
            } else {
                let up = (0.014 - x.abs()).min(0.140 - y.abs());
                if (y - 0.050).abs() < 0.013 {
                    (0.080 - x.abs()).min(0.013 - (y - 0.050).abs()).max(up)
                } else {
                    up
                }
            };
            let rub = (1.0 - sstep(0.0, 0.0035, edge)) * sstep(0.3, 0.6, fbm(seed ^ 5, along * 40.0, across * 40.0, 2));
            c = mix(c, [0.52, 0.40, 0.27], rub * 0.6);
            // the joint's hairline
            if !on_bar && (y - 0.050).abs() < 0.013 && (x.abs() - 0.014).abs() < 0.0005 {
                c = c.map(|k| k * 0.6);
            }
            // dust on the crossbar's top edge and over all
            let top = if on_bar { 1.0 - sstep(0.0, 0.004, 0.063 - y) } else { 0.0 };
            let d = fbm(seed ^ 9, u * 30.0, v * 50.0, 2);
            c = mix(c, [0.50, 0.47, 0.42], (top * 0.45 + 0.06) * (0.6 + 0.8 * d));
            let knock = vn(seed ^ 11, u * 70.0, v * 120.0);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.95, 0.985, knock) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A cast corpus's metal: gilding (or a grey lacquer) over spelter, worn
/// through on what stands out to the grey metal, dark in the folds
fn spelter_tex(salt: u32, gilt: bool) -> Tex {
    canvas_tex_opt(64, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0xd7;
        let base = if gilt { [0.46, 0.37, 0.20] } else { [0.36, 0.35, 0.33] };
        fill(ctx, w, h, |u, v| {
            let mut c = base;
            c = mix(c, [0.40, 0.40, 0.38], sstep(0.5, 0.75, fbm(seed, u * 9.0, v * 12.0, 3)) * 0.7);
            c = mix(c, [0.10, 0.09, 0.08], sstep(0.55, 0.85, fbm(seed ^ 3, u * 16.0, v * 22.0, 3)) * 0.6);
            c = mix(c, [0.46, 0.44, 0.40], sstep(0.7, 0.95, vn(seed ^ 5, u * 40.0, v * 60.0)) * 0.3 + 0.05);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The titulus: tin painted cream, the four letters in black, rusted at
/// its curled ends
fn title_tex(salt: u32) -> Tex {
    canvas_tex_opt(96, 42, false, move |ctx, w, h| {
        let seed = salt ^ 0xe5;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.74, 0.68, 0.54];
            c = mix(c, [0.56, 0.48, 0.34], sstep(0.4, 0.8, fbm(seed, u * 5.0, v * 3.0, 3)) * 0.6);
            let end = u.min(1.0 - u);
            c = mix(c, [0.32, 0.20, 0.12], (1.0 - sstep(0.02, 0.12, end)) * 0.7);
            [c[0], c[1], c[2], 1.0]
        });
        ctx.set_fill_style("rgba(30,24,20,0.9)");
        ctx.set_font("bold 24px serif");
        ctx.set_text_align("center");
        ctx.fill_text("INRI", w / 2.0, h * 0.74);
    })
}
