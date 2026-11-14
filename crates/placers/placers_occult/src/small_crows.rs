//! `place_small_crows`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::vn as crow_vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small crow: 5% chance per level. A moulded plastic Halloween crow
// (16 cm from beak to tail), standing on its wired legs: black polypropylene
// with its wings and tail moulded on, the long feathers free, a stout beak,
// two glass bead eyes that catch the light; dust in its feathers, its black
// rubbed grey on the beak and the wings' edges.
pub(crate) fn place_small_crows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = ((px * 7919.0).abs() as u32) ^ ((pz * 104_729.0).abs() as u32).rotate_left(13) ^ 0x7c0;
    let crow = group();
    // modelled with its beak toward +z: turned to face +x, as the old one
    let bird = group();
    bird.rotation().y = PI / 2.0;
    crow.add(&bird);
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            bird.add(&mesh(geo, mat.clone()));
        }
    };
    put("Real__crowBody", &phong!({ map: &crow_plastic_tex(salt), specular: 0x2c2c2c, shininess: 30 }));
    put("Real__crowBeak", &phong!({ color: 0x3a3836, specular: 0x262626, shininess: 20 }));
    put("Real__crowLegs", &lambert!({ color: 0x2c2a28 }));
    put("Real__crowEyes", &phong!({ color: 0x2e1c10, specular: 0x8a8a8a, shininess: 90 }));
    crow.position().set(px, 0, pz);
    crow.rotation().y = turn;
    g.world.add(&crow);
}

/// Black polypropylene (laid on the bird from every side): not quite even,
/// rubbed grey in places, dust caught in the moulded feathers
fn crow_plastic_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (u, v) = ((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                let n = crow_fbm(salt, u * 9.0, v * 9.0);
                let mut c = 0.145 + 0.05 * n;
                let rub = crow_fbm(salt ^ 3, u * 4.0, v * 4.0);
                if rub > 0.62 {
                    c += (rub - 0.62) * 0.35;
                }
                let dust = crow_vn(salt ^ 5, u * 170.0, v * 170.0);
                if dust > 0.8 {
                    c += (dust - 0.8) * 1.1;
                }
                let px = (c.clamp(0.0, 1.0) * 255.0).round() as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[px, px, (f64::from(px) * 0.97) as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn crow_fbm(seed: u32, x: f64, y: f64) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..4 {
        s += amp * crow_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
