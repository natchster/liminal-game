//! `place_shower_caps`
use crate::shared::salt;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::skip;

// Shower cap: 6% chance per level. A reusable PEVA shower cap (Blender
// family `washroom_shelf_and_dressing_table`) dropped on the floor opening down: the crown fallen in on
// itself in creases, spilled over its gathered elastic in two lobes, the
// frill flopped out round it; printed (small flowers, dots) or plain, or a
// hotel's clear disposable one, its old colour; yellowed, dust in it, mildew
// spotting where it stayed damp.
pub(crate) fn place_shower_caps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let cap_colors = [0xffffff, 0xf0d0e0, 0x8080e0, 0xf0f0a0, 0xe0e0e0];
    let col = pick(&cap_colors);
    let kind = cap_colors.iter().position(|&k| k == col).unwrap_or(0);
    let cap = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let salt = salt(px, pz);
    if let Some(geo) = prop_part_as_modelled("Real__showerCapFilm") {
        let tex = cap_print_tex(kind, salt);
        let mat = if kind == 4 {
            // clear polyethylene, gone milky
            phong!({ map: &tex, color: 0xdadcd6, specular: 0x404040, shininess: 50, transparent: true, opacity: 0.72, side: DoubleSide })
        } else {
            // PEVA's satin
            phong!({ map: &tex, color: 0xffffff, specular: 0x242424, shininess: 22, side: DoubleSide })
        };
        cap.add(&mesh(geo, &mat));
    }
    cap.position().set(px, 0, pz);
    cap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cap);
}

/// The cap's PEVA seen from above: its print (or plain; or the clear
/// film's crinkle), yellowed, dust in it, mildew spotting in clusters
fn cap_print_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        skip(salt);
        let base = ["#ddd8cc", "#d6b4bc", "#8e8cb0", "#ddd6a6", "#cfd2cc"][kind.min(4)];
        ctx.set_fill_style(base);
        ctx.fill_rect(0, 0, w, h);
        match kind {
            0 => {
                // small faded blue flowers, scattered
                for _ in 0..90 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    ctx.set_fill_style("rgba(92,112,150,0.75)");
                    for p in 0..5 {
                        let a = p as f64 * 1.2566 + rnd() * 0.2;
                        ctx.begin_path();
                        ctx.ellipse(x + a.dcos() * 2.6, y + a.dsin() * 2.6, 2.2, 1.4, a, 0, 7);
                        ctx.fill();
                    }
                    ctx.set_fill_style("rgba(190,160,80,0.8)");
                    ctx.begin_path();
                    ctx.arc(x, y, 1.1, 0, 7);
                    ctx.fill();
                }
            }
            1 => {
                // white polka dots on a half-drop grid
                ctx.set_fill_style("rgba(238,232,226,0.9)");
                let s = 18.0;
                let mut j = 0;
                while (j as f64) * s < h + s {
                    let mut i = 0;
                    while (i as f64) * s < w + s {
                        let x = i as f64 * s + if j % 2 == 1 { s / 2.0 } else { 0.0 };
                        ctx.begin_path();
                        ctx.arc(x, j as f64 * s, 3.4, 0, 7);
                        ctx.fill();
                        i += 1;
                    }
                    j += 1;
                }
            }
            3 => {
                // small daisies
                for _ in 0..70 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    ctx.set_fill_style("rgba(245,242,228,0.85)");
                    for p in 0..8 {
                        let a = p as f64 * 0.785;
                        ctx.begin_path();
                        ctx.ellipse(x + a.dcos() * 3.0, y + a.dsin() * 3.0, 2.4, 0.9, a, 0, 7);
                        ctx.fill();
                    }
                    ctx.set_fill_style("rgba(200,150,50,0.9)");
                    ctx.begin_path();
                    ctx.arc(x, y, 1.4, 0, 7);
                    ctx.fill();
                }
            }
            _ => {}
        }
        // the wear, pixel by pixel over the print
        let seed = salt ^ 0x5ca9;
        let mut img = ctx.get_image_data(0, 0, w, h);
        let iw = w as usize;
        let clear = kind == 4;
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (u, v) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                let n = fbm(seed, u * 8.0, v * 8.0, 4);
                // yellowed and dirty in blotches, dust settled
                let dirt = 0.06 + 0.18 * sstep(0.45, 0.8, n);
                // mildew: black specks in clusters
                let clus = sstep(0.62, 0.78, fbm(seed ^ 3, u * 5.0, v * 5.0, 2));
                let speck = sstep(0.80, 0.90, vn(seed ^ 4, u * 120.0, v * 120.0)) * clus;
                // the clear film's crinkle catches the light in lines
                let crink =
                    if clear { sstep(0.85, 1.0, 1.0 - (fbm(seed ^ 6, u * 14.0, v * 14.0, 3) - 0.5).abs() * 2.0) * 0.25 } else { 0.0 };
                for k in 0..3 {
                    let c = f64::from(out[i * 4 + k]) / 255.0;
                    let tint = [0.92, 0.86, 0.70][k];
                    let mut c2 = c * (1.0 - dirt) + c * tint * dirt * 0.85;
                    c2 = c2 * (1.0 - 0.85 * speck) + 0.05 * speck;
                    c2 += crink;
                    out[i * 4 + k] = (c2.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
