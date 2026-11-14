//! `place_makeup_sponges`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::fbm as t03_fbm;
use placers_kit::noise::hash as t03_hash;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Makeup sponge: 9% chance per level. A teardrop blending sponge of
// polyurethane foam (Blender batch `t03`), dropped and lying on its side as
// it rests, its old colour faded and greyed, its pores dark with dust; the
// old 55% used: foundation dried into its point and down one side (the old
// smudge colour its shade), a tide at its edge.
pub(crate) fn place_makeup_sponges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Sponge color pool — pastel.
    let sponge_colors: [u32; 6] = [0xf0a0a0, 0xa0d0f0, 0xf0c0e0, 0xd0e0a0, 0xf8c088, 0xe0c0f0];
    let col = pick(&sponge_colors);
    let kind = sponge_colors.iter().position(|&k| k == col).unwrap_or(0);
    // the old 55% smudge: it was used, and its colour the foundation's shade
    let shade = if rnd() < 0.55 {
        let smudge_colors: [u32; 4] = [0x704030, 0x8b6040, 0x604020, 0x503020];
        let c = pick(&smudge_colors);
        Some(smudge_colors.iter().position(|&k| k == c).unwrap_or(0))
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let yaw = rnd() * PI * 2.0;
    // the old tilt draw: how it rolled about its own length as it came to rest
    let roll = (rnd() - 0.5) * 0.5 * 4.0;
    let salt = ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32;
    let sp = group();
    if let Some(geo) = prop_part_as_modelled("Real__makeupSpongeFoam") {
        // modelled standing on its base: rolled, laid on its side as it
        // rests (bathroom/washroom_shelf_and_dressing_table.py), its lowest point on the floor; the turn is in the
        // geometry, so its box is its own and it sits on the floor
        let geo = geo.duplicate();
        geo.rotate_y(roll);
        geo.rotate_z(T03_SPONGE_LAY);
        let bb = geo.compute_bounding_box();
        geo.translate(-(bb.min.x + bb.max.x) / 2.0, -bb.min.y, -(bb.min.z + bb.max.z) / 2.0);
        sp.add(&mesh(geo, lambert!({ map: &t03_sponge_tex(kind, shade, salt) })));
    }
    sp.position().set(px, 0, pz);
    sp.rotation().y = yaw;
    g.world.add(&sp);
}

/// The turn about z that lays the standing sponge on its side at rest (its
/// volume's centre lowest), printed by bathroom/washroom_shelf_and_dressing_table.py (`t03 sponge: lay …`)
const T03_SPONGE_LAY: f64 = -1.87623;

/// The foam (u round it, v from its base to its point, as a cylinder's part
/// lays it): its colour faded and greyed, pores dark with dust, dirtier on
/// the side it lies on; foundation (`shade`) dried into its point and down
/// the side that faces up when it lies unrolled (u = 0.75), darkest at its
/// tide
fn t03_sponge_tex(kind: usize, shade: Option<usize>, salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b09;
        let base = [[0.78, 0.52, 0.55], [0.55, 0.67, 0.76], [0.80, 0.63, 0.71], [0.66, 0.72, 0.50], [0.84, 0.66, 0.52], [0.71, 0.62, 0.78]]
            [kind.min(5)];
        let found = shade.map(|s| [[0.76, 0.58, 0.46], [0.82, 0.66, 0.53], [0.68, 0.51, 0.39], [0.58, 0.43, 0.32]][s.min(3)]);
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (u, v) = ((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                // the wrap: noise that meets itself across u = 0 / 1
                let (cu, su) = ((u * std::f64::consts::TAU).cos(), (u * std::f64::consts::TAU).sin());
                let n = t03_fbm(seed, cu * 2.0 + 5.0, su * 2.0 + v * 3.0, 3);
                // pores: dark pits, dust settled in them
                let pore = t03_sstep(0.70, 0.86, t03_vn_wrap(seed ^ 2, u * 96.0, v * 44.0, 96));
                // the side it lies on (u = 0.25), and the grey of dust
                let under = 0.5 + 0.5 * ((u - 0.25) * std::f64::consts::TAU).cos();
                let grey = 0.10 + 0.10 * n + 0.14 * under;
                let mut c = [0.0; 3];
                for k in 0..3 {
                    let faded = base[k] * (1.0 - grey) + 0.52 * grey;
                    c[k] = faded * (0.92 + 0.12 * n) * (1.0 - 0.30 * pore);
                }
                if let Some(f) = found {
                    // round the side that faces up, from the point down
                    let du = {
                        let d = (u - 0.75).abs();
                        d.min(1.0 - d)
                    };
                    let edge = 0.56 + 0.12 * (t03_fbm(seed ^ 5, cu * 3.0, su * 3.0 + v * 2.0, 2) - 0.5) * 2.0;
                    // how far into the stain: up the point, and more on the up side
                    let s = v + (0.18 - du) * 0.8;
                    let a = (t03_sstep(edge, edge + 0.12, s) + 0.8 * t03_sstep(0.86, 0.97, v)).min(1.0);
                    let tide = (-((s - edge - 0.05) / 0.025).powi(2)).exp() * a;
                    let soak = a * (0.55 + 0.35 * n);
                    for k in 0..3 {
                        let fc = f[k] * (1.0 - 0.25 * tide) * (1.0 - 0.2 * pore);
                        c[k] = c[k] * (1.0 - soak) + fc * soak;
                    }
                }
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// Value noise whose lattice wraps every `period` cells in x (round the
/// sponge: its pores meet themselves across the seam, round, not streaked)
fn t03_vn_wrap(seed: u32, x: f64, y: f64, period: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (i0, i1) = (i.rem_euclid(period), (i + 1).rem_euclid(period));
    let (a, b) = (t03_hash(seed, i0, j), t03_hash(seed, i1, j));
    let (c, d) = (t03_hash(seed, i0, j + 1), t03_hash(seed, i1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn t03_sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
