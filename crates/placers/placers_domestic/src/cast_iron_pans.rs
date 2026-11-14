//! `place_cast_iron_pans`
use game::prelude::*;
use placers_kit::baking::fbm;
use placers_kit::baking::mix;
use placers_kit::baking::noise;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// A cast-iron skillet hung on the wall (Blender batch `r16`): a 10 1/4
// in skillet of the Lodge L8SK3 class, hung by the hole in its handle from
// a nail, its bottom against the wall and its cooking face to the room; its
// seasoning gone dull and flaking at the rim, rusted where it was put away
// wet: a tide of orange rust low in the pan where the water stood, rust run
// down from it and blooming at the pour lips and in the hole. 4% chance per
// level.
pub(crate) fn place_cast_iron_pans(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let pan = group();
    // its rust differs floor to floor: from where it hangs (no draw)
    let salt = ((x * 7919 + z * 104_729 + dx * 31 + dz * 17) as u32).wrapping_mul(0x9e37_79b9);
    // the nail's hole 1.93 m up, so the pan's middle is at 1.68 m (the old
    // pan's 1.80 m middle and its handle)
    let lift = PAN_NAIL_Y - PAN_HOLE_Y;
    if let Some(geo) = prop_part_as_modelled("Real__panBody") {
        let body = mesh(geo, phong!({ map: &pan_iron_tex(salt), color: 0xffffff, specular: 0x201e1c, shininess: 18 }));
        body.position().set(0, lift, -0.015);
        pan.add(&body);
    }
    if let Some(geo) = prop_part_as_modelled("Real__panNail") {
        let nail = mesh(geo, phong!({ color: 0x4a3c32, specular: 0x262422, shininess: 18 }));
        nail.position().set(0, lift, -0.015);
        pan.add(&nail);
    }
    pan.rotation().y = face_wall(dx, dz);
    pan.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&pan);
}

/// The skillet as hung (`Real__panBody`): its bounds (x, y; domestic/kitchen_and_hearth_left_behind.py), its
/// middle at y = 0, the hanging hole's centre, and where the nail is
const PAN_BOUNDS: [f64; 4] = [-0.1393, -0.1697, 0.1393, 0.2631];

const PAN_HOLE_Y: f64 = 0.2482;

const PAN_NAIL_Y: f64 = 1.93;

/// The cooking face's radius, and the inside of the rim (domestic/kitchen_and_hearth_left_behind.py `PAN_PROFILE`)
const PAN_FLOOR_R: f64 = 0.0915;

const PAN_RIM_R: f64 = 0.1284;

/// The iron, seen from the room (the box's texture coordinates: u across,
/// v up over the hung pan's bounds): black seasoning gone dull, worn grey
/// where it was scoured, flaking at the rim; rust where it was left wet: a
/// tide low in the pan where water stood when it lay flat, spots and
/// blooms, rust run down the walls from the lips and the rim, and in the
/// hanging hole; the handle polished dark by hands.
fn pan_iron_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 384, false, move |ctx, w, h| {
        let seed = salt ^ 0x1a0e;
        let [x0, y0, x1, y1] = PAN_BOUNDS;
        let mut r = SplitMix64::new(salt, 0xfe20);
        // where the water stood: a pool low in the pan (the pan lay flat;
        // it dried from its edges), its centre off the middle
        let pool = (r.range(-0.025, 0.025), r.range(-0.04, -0.015), r.range(0.055, 0.075));
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let py = y1 - (j as f64 + 0.5) / h * (y1 - y0);
            for i in 0..n {
                let px = x0 + (i as f64 + 0.5) / w * (x1 - x0);
                let rr = (px * px + py * py).sqrt();
                let f = fbm(seed, px * 40.0, py * 40.0);
                let grain = noise(seed ^ 3, px * 900.0, py * 900.0);
                // the seasoning: near-black, a little brown, its cast grain
                let mut c = [27.0 + 7.0 * f + 5.0 * grain, 25.0 + 6.0 * f + 4.0 * grain, 24.0 + 5.0 * f + 3.0 * grain];
                // dulled grey-brown where a spatula and steel wool went
                let inside = rr < PAN_FLOOR_R + 0.008;
                if inside {
                    let scour = (fbm(seed ^ 5, px * 25.0, py * 25.0) - 0.5).max(0.0) * 1.2;
                    mix(&mut c, [52.0, 48.0, 44.0], scour * 0.35);
                }
                // flaking at the rim: the seasoning lifted to grey iron
                let rim = (1.0 - ((rr - PAN_RIM_R) / 0.010).abs()).max(0.0);
                let flake = ((noise(seed ^ 7, px * 160.0, py * 160.0) - 0.55) * 3.0).max(0.0) * rim;
                mix(&mut c, [62.0, 56.0, 50.0], flake * 0.7);
                // rust where the water stood when it lay flat: a ragged pool,
                // uneven inside (orange powder, dark crust), blooms round it
                let (pxc, pyc, pr) = pool;
                let dpool = (((px - pxc) / 1.2).powi(2) + (py - pyc).powi(2)).sqrt() / pr;
                let ragged = dpool + 0.55 * (fbm(seed ^ 9, px * 22.0, py * 22.0) - 0.5);
                let in_pool = ((1.0 - ragged) * 4.0).clamp(0.0, 1.0) * (0.55 + 0.45 * fbm(seed ^ 17, px * 70.0, py * 70.0));
                let bloom = ((fbm(seed ^ 11, px * 34.0, py * 34.0) - 0.6).max(0.0) * 4.0).min(1.0);
                let spots = ((noise(seed ^ 19, px * 260.0, py * 260.0) - 0.8).max(0.0) * 5.0).min(1.0);
                let wet = if inside { 1.0 } else { 0.25 };
                // rust at the pour lips, where water ran out
                let lip = ((px.abs() - 0.118) / 0.012).clamp(0.0, 1.0) * (1.0 - (py.abs() / 0.02).min(1.0));
                let rust = ((in_pool + bloom * 0.8 + spots * 0.6) * wet + lip * 0.4 + rim * flake * 0.5).min(1.0);
                // rust's own colour: orange-brown powder, dark where it crusted
                let rk = noise(seed ^ 15, px * 300.0, py * 300.0);
                let crust = noise(seed ^ 21, px * 90.0, py * 90.0);
                let rc = [96.0 + 44.0 * rk - 30.0 * crust, 46.0 + 20.0 * rk - 14.0 * crust, 24.0 + 10.0 * rk - 6.0 * crust];
                mix(&mut c, rc, rust * (0.8 + 0.2 * grain));
                // the handle, polished dark by hands, the hole rusted
                if py > 0.14 {
                    let dh = ((py - PAN_HOLE_Y).powi(2) + px * px).sqrt();
                    let hole = (1.0 - (dh - 0.008) / 0.005).clamp(0.0, 1.0);
                    let polish = ((0.23 - py) / 0.06).clamp(0.0, 1.0) * 0.3;
                    mix(&mut c, [19.0, 18.0, 18.0], polish);
                    mix(&mut c, [104.0, 52.0, 26.0], hole * 0.7);
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
