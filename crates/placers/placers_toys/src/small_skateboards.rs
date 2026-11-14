//! `place_small_skateboards`: Small skateboard: 8% chance per level. A plastic cruiser board (Blender family `a_play_areas_toys`): a 22 x 6 in moulded polypropylene deck, its top textured for grip and its tail kicked, on 3 in cast aluminium trucks and 59 mm urethane wheels; one in seven lies upside down, resting on its tail and nose (built so: the game turns it only about y). The plastic faded and scuffed white at its edges, the wheels yellowed, grime.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;
use placers_kit::place::scatter_pos;

/// Small skateboard: 8% chance per level. A plastic cruiser board
/// (Blender family `a_play_areas_toys`): a 22 x 6 in moulded polypropylene deck, its top
/// textured for grip and its tail kicked, on 3 in cast aluminium trucks
/// and 59 mm urethane wheels; one in seven lies upside down, resting on its
/// tail and nose (built so: the game turns it only about y). The plastic
/// faded and scuffed white at its edges, the wheels yellowed, grime.
pub(crate) fn place_small_skateboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    // the old deck colour: its plastic's, faded
    let deck_colors = [0xd82040, 0x2050d8, 0x30a030, 0xa020a0, 0xd8b040, 0x201a10];
    let old = pick_color(&deck_colors);
    let k = deck_colors.iter().position(|&c| c == old).unwrap_or(0);
    let plastic =
        [[0.55, 0.22, 0.20], [0.24, 0.33, 0.52], [0.30, 0.46, 0.32], [0.42, 0.30, 0.45], [0.70, 0.60, 0.30], [0.17, 0.16, 0.15]][k];
    let skate = group();
    scatter_pos(g, &skate, x, z, 2.0);
    skate.rotation().y = rnd() * PI * 2.0;
    // the old 15% upside down
    let flip = if rnd() < 0.15 { "Flip" } else { "" };
    let p = skate.pos();
    let salt = salt_of(p.x, p.z);
    put(&skate, &format!("Real__cruiserDeck{flip}"), &phong!({ map: &deck_tex(plastic, salt), specular: 0x161616, shininess: 20 }));
    put(&skate, &format!("Real__cruiserTrucks{flip}"), &phong!({ color: 0x74787c, specular: 0x2c2c2c, shininess: 28 }));
    put(&skate, &format!("Real__cruiserBushings{flip}"), &lambert!({ color: 0x8e6434 }));
    put(&skate, &format!("Real__cruiserWheels{flip}"), &lambert!({ map: &wheel_tex(salt) }));
    g.world.add(&skate);
}

/// The deck's plastic, laid on its bounds (u along it, v across): the fine
/// moulded grip of its top, faded, scuffed white where it scraped (its
/// nose, its tail, its rails), dark where feet wore it, grime
fn deck_tex(plastic: [f64; 3], salt: u32) -> Tex {
    paint(512, 128, move |u, v| {
        let mut c = mix(plastic, [0.60, 0.58, 0.54], 0.22 * sstep(0.4, 0.8, fbm(salt, u * 6.0, v * 2.0, 3)));
        // the moulded grip: a fine raised lattice
        let (gx, gy) = (u * 180.0, v * 50.0);
        let lat = ((gx + gy).dsin() * (gx - gy).dsin()).abs();
        c = mix(c, [c[0] * 0.82, c[1] * 0.82, c[2] * 0.82], 0.5 * (1.0 - sstep(0.05, 0.35, lat)));
        // scraped white at the ends and the rails, in streaks along it
        let ends = sstep(0.14, 0.02, u).max(sstep(0.88, 0.99, u)).max(sstep(0.12, 0.0, v)).max(sstep(0.88, 1.0, v));
        let streak = sstep(0.55, 0.75, fbm(salt ^ 0x2d, u * 60.0, v * 6.0, 3));
        c = mix(c, [0.74, 0.72, 0.68], 0.75 * ends * streak);
        // where feet stood: darker, the grip worn smooth
        let feet = (-(((u - 0.30) / 0.10).dpowi(2) + ((v - 0.5) / 0.35).dpowi(2)))
            .dexp()
            .max((-(((u - 0.72) / 0.10).dpowi(2) + ((v - 0.5) / 0.35).dpowi(2))).dexp());
        c = mix(c, [c[0] * 0.70, c[1] * 0.70, c[2] * 0.70], 0.5 * feet);
        let dirt = sstep(0.5, 0.85, fbm(salt ^ 0x4d, u * 10.0, v * 3.0, 3));
        mix(c, [0.28, 0.26, 0.23], 0.45 * dirt)
    })
}

/// The urethane of the wheels: off-white gone yellow, the tread grey with
/// grime
fn wheel_tex(salt: u32) -> Tex {
    paint(64, 64, move |u, v| {
        let c = mix([0.76, 0.71, 0.56], [0.66, 0.60, 0.44], fbm(salt ^ 0x6d, u * 5.0, v * 5.0, 3));
        mix(c, [0.36, 0.33, 0.29], 0.55 * sstep(0.45, 0.8, fbm(salt ^ 0x7d, u * 9.0, v * 9.0, 2)))
    })
}

/// The part `name` of family `a_play_areas_toys` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// A texture painted pixel by pixel (sRGB) from `f(u, v)` (v down the
/// image), on every core: no canvas call and no draw inside
fn paint(w: usize, h: usize, f: impl Fn(f64, f64) -> [f64; 3] + Sync + Send + 'static) -> Tex {
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let c = f((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
