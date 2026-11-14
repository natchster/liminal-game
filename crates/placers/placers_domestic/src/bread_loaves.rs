//! `place_bread_loaves`
use game::prelude::*;
use placers_kit::baking::fbm as loaf_fbm;
use placers_kit::baking::mould as loaf_mould;
use placers_kit::baking::noise as loaf_noise;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// A loaf left on its board (Blender batch `r16`): a small white bloomer
// on a beech bread board, its end cut off long ago; stale, shrunk and
// cracked, its crust gone dull, mould grown over it in blue-green and white
// colonies, thickest on the cut face; the board scored by knives, greyed,
// a hanging hole at its end, crumbs by the loaf. 5% chance per level.
pub(crate) fn place_bread_loaves(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let loaf = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let yaw = rnd() * PI * 2.0;
    // how far the mould has got differs floor to floor: from where it lies
    let salt = ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32;
    let parts = [
        ("Real__breadBoard", lambert!({ map: &loaf_board_tex(salt) })),
        ("Real__breadCrust", lambert!({ map: &loaf_crust_tex(salt) })),
        ("Real__breadCrumb", lambert!({ map: &loaf_crumb_tex(salt) })),
        ("Real__breadCrumbs", lambert!({ color: 0x9a8058 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            loaf.add(&mesh(geo, mat));
        }
    }
    loaf.position().set(px, 0, pz);
    loaf.rotation().y = yaw;
    g.world.add(&loaf);
}

/// The loaf as modelled (domestic/kitchen_and_hearth_left_behind.py): the crust part's bounds in x and z, the
/// slashes (their middles along x from the loaf's own middle, the angle of
/// the line across them), the board's size
const LOAF_CRUST_X: (f64, f64) = (-0.1415, 0.0330);

const LOAF_CRUST_Z: (f64, f64) = (-0.0550, 0.0550);

const LOAF_X0: f64 = -0.035;

const LOAF_SLASHES: [(f64, f64); 5] = [(-0.074, 0.55), (-0.040, 0.58), (-0.006, 0.54), (0.028, 0.57), (0.062, 0.55)];

const LOAF_BOARD: (f64, f64) = (0.320, 0.220);

fn loaf_mix(c: &mut [f64; 3], to: [f64; 3], k: f64) {
    let k = k.clamp(0.0, 1.0);
    for i in 0..3 {
        c[i] += (to[i] - c[i]) * k;
    }
}

/// The crust (the box's texture coordinates over the crust's bounds: on
/// top, u along the loaf and v across it): a bake's browns, darker on the
/// ridges, paler down the opened slashes, gone dull and grey with age;
/// cracks; mould colonies, most toward the cut end and low on the sides
fn loaf_crust_tex(salt: u32) -> Tex {
    canvas_tex_opt(384, 192, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a1;
        let mut r = SplitMix64::new(salt, 0x10af);
        // mould colonies (px): a few big, many small, more at the cut end (u = 1)
        let cols: Vec<(f64, f64, f64)> = (0..10)
            .map(|k| {
                let u = 1.0 - r.next().powf(2.2) * 0.8;
                let rad = if k < 2 { r.range(14.0, 26.0) } else { r.range(3.0, 9.0) };
                (u * w, r.next() * h, rad)
            })
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fv = (j as f64 + 0.5) / h;
            let zz = LOAF_CRUST_Z.1 - fv * (LOAF_CRUST_Z.1 - LOAF_CRUST_Z.0);
            for i in 0..n {
                let fu = (i as f64 + 0.5) / w;
                let xx = LOAF_CRUST_X.0 + fu * (LOAF_CRUST_X.1 - LOAF_CRUST_X.0);
                let f = loaf_fbm(seed, xx * 60.0, zz * 60.0);
                // the bake: golden brown, darker across the middle of the top
                let mid = 1.0 - (zz.abs() / 0.055).min(1.0);
                let mut c =
                    [150.0 - 30.0 * mid + 18.0 * (f - 0.5), 108.0 - 26.0 * mid + 14.0 * (f - 0.5), 64.0 - 18.0 * mid + 10.0 * (f - 0.5)];
                // the slashes opened pale down their middles (the grooves
                // themselves are modelled; their paint only near the crown,
                // where the top's mapping is the crust's own)
                let crown = (1.0 - (zz.abs() / 0.03).min(1.0)).powi(2);
                for &(xs, ang) in &LOAF_SLASHES {
                    let (dx, dz) = (xx - (xs + LOAF_X0), zz);
                    let along = dx * ang.cos() + dz * ang.sin();
                    let pale = (-(along / 0.004).powi(2)).exp() * crown;
                    loaf_mix(&mut c, [176.0, 150.0, 108.0], pale * 0.45);
                }
                // stale: dull, greyed
                loaf_mix(&mut c, [128.0, 118.0, 100.0], 0.25);
                // cracks in the dried crust
                let cr = (1.0 - ((loaf_noise(seed ^ 4, xx * 140.0, zz * 140.0) - 0.5).abs() * 30.0).min(1.0)) * (f > 0.5) as i32 as f64;
                loaf_mix(&mut c, [70.0, 52.0, 34.0], cr * 0.7);
                // the mould
                let (s, wr, st) = loaf_mould(&cols, i as f64, j as f64, seed ^ 7);
                loaf_mix(&mut c, [140.0, 128.0, 92.0], st * 0.3);
                loaf_mix(&mut c, [204.0, 204.0, 194.0], wr * 0.7);
                let sp = loaf_noise(seed ^ 9, i as f64 / 1.5, j as f64 / 1.5);
                loaf_mix(&mut c, [92.0 + 24.0 * sp, 106.0 + 18.0 * sp, 90.0 + 12.0 * sp], s * 0.9);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // black pin mould here and there
        for _ in 0..30 {
            let (x, y) = (w * (1.0 - r.next().powf(1.4)), r.next() * h);
            ctx.set_fill_style("rgba(30,30,28,0.8)");
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.5, 1.4), 0, PI * 2.0);
            ctx.fill();
        }
    })
}

/// The cut face (u across it, v up it): dry crumb, its holes, the crust's
/// edge round it, and the mould grown over it from its rim: blue-green
/// colonies with white rims, a yellow stain round them, black pin mould
fn loaf_crumb_tex(salt: u32) -> Tex {
    canvas_tex_opt(192, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0xc4b3;
        let mut r = SplitMix64::new(salt, 0x2c8d);
        let cols: Vec<(f64, f64, f64)> = (0..9)
            .map(|k| {
                let a = r.range(0.0, PI * 2.0);
                let d = r.range(0.25, 0.48);
                (w * (0.5 + a.cos() * d), h * (0.55 + a.sin() * d * 0.9), if k < 3 { r.range(18.0, 34.0) } else { r.range(5.0, 12.0) })
            })
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let holes = loaf_noise(seed, x / 2.2, y / 2.6);
                let big = loaf_noise(seed ^ 1, x / 6.0, y / 7.0);
                let mut c = [186.0, 172.0, 140.0];
                let k = 0.92 + 0.12 * holes - 0.35 * (0.3 - holes).max(0.0) - 0.4 * (0.22 - big).max(0.0);
                for q in 0..3 {
                    c[q] *= k;
                }
                // dried and greyed toward the crust
                let (dx, dy) = ((x / w - 0.5) * 2.0, (y / h - 0.55) * 2.1);
                let rim = (dx * dx + dy * dy).sqrt();
                loaf_mix(&mut c, [150.0, 136.0, 108.0], (rim - 0.6).max(0.0) * 1.5);
                let (s, wr, st) = loaf_mould(&cols, x, y, seed ^ 7);
                loaf_mix(&mut c, [160.0, 144.0, 96.0], st * 0.4);
                loaf_mix(&mut c, [210.0, 210.0, 200.0], wr * 0.75);
                let sp = loaf_noise(seed ^ 9, x / 1.4, y / 1.4);
                loaf_mix(&mut c, [88.0 + 26.0 * sp, 104.0 + 20.0 * sp, 88.0 + 14.0 * sp], s * 0.9);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        for _ in 0..24 {
            let a = r.range(0.0, PI * 2.0);
            let d = r.range(0.1, 0.5);
            let (x, y) = (w * (0.5 + a.cos() * d), h * (0.55 + a.sin() * d));
            ctx.set_fill_style("rgba(28,28,26,0.85)");
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.5, 1.3), 0, PI * 2.0);
            ctx.fill();
        }
    })
}

/// The board's top (u along it, v across): beech, its grain along it, oiled
/// once and long dry, greyed; scored by the bread knife across and along,
/// darker where the loaf lay and round its foot; a water ring
fn loaf_board_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 352, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a2;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let grain = loaf_noise(seed, x / 60.0, y / 1.6) * 0.6 + loaf_noise(seed ^ 1, x / 14.0, y / 0.8) * 0.4;
                let fleck = (loaf_noise(seed ^ 2, x / 1.5, y / 3.0) - 0.8).max(0.0) * 3.0;
                let mut c = [152.0 + 22.0 * (grain - 0.5), 124.0 + 18.0 * (grain - 0.5), 92.0 + 14.0 * (grain - 0.5)];
                loaf_mix(&mut c, [120.0, 92.0, 64.0], fleck);
                // greyed with years and dust
                let dust = loaf_fbm(seed ^ 3, x / 50.0, y / 50.0);
                loaf_mix(&mut c, [126.0, 118.0, 106.0], 0.35 + 0.3 * dust);
                // darker where the loaf has lain on it (its middle at x_0)
                let (lu, lv) =
                    (((x / w) * LOAF_BOARD.0 - LOAF_BOARD.0 / 2.0 - LOAF_X0 + 0.035) / 0.12, (y / h - 0.5) * LOAF_BOARD.1 / 0.06);
                let under = (1.0 - (lu * lu + lv * lv).sqrt()).max(0.0);
                loaf_mix(&mut c, [104.0, 84.0, 60.0], under * 0.5);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(salt, 0x60a4);
        // knife scores: fine dark lines, most across the board
        for _ in 0..90 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = PI / 2.0 + r.range(-0.6, 0.6) + if r.next() < 0.25 { PI / 2.0 } else { 0.0 };
            let l = r.range(10.0, 60.0);
            ctx.set_stroke_style(&format!("rgba(70,54,40,{:.2})", r.range(0.15, 0.45)));
            ctx.set_line_width(r.range(0.5, 1.1));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // a water ring from a mug, long ago
        let (cx, cy) = (w * r.range(0.7, 0.85), h * r.range(0.25, 0.75));
        ctx.set_stroke_style("rgba(84,66,48,0.18)");
        ctx.set_line_width(1.6);
        ctx.begin_path();
        ctx.arc(cx, cy, 26.0, 0.3, 5.6);
        ctx.stroke();
    })
}
