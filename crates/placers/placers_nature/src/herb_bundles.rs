//! `place_herb_bundles`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging herbs bundle: 5% chance per level.
pub(crate) fn place_herb_bundles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // Herbs hung head down from a brass cup hook to dry and left there for
    // years (Blender family `garden_things`): five to eight stems tied at their cut ends
    // with cotton string, fanned, dry and dusty. The old colour's draw
    // chooses the herb: garden sage, rosemary, lavender, an older rosemary,
    // a sage gone yellow
    let herb_colors: [u32; 5] = [0x506030, 0x405020, 0x607040, 0x304018, 0x807040];
    let bundle = group();
    let body = group();
    // the wall is 0.008 behind the old group's origin
    body.position().z = -0.008;
    bundle.add(&body);
    put(&body, "Real__herbHook", &phong!({ color: 0x86703e, specular: 0x403420, shininess: 30 }));
    put(&body, "Real__herbTwine", &lambert!({ color: 0xb2a890 }));
    let stem_count = 5 + (rnd() * 4.0).floor() as i32;
    let herb = match pick(&herb_colors) {
        0x506030 => Herb::Sage(0x5e6650),
        0x405020 => Herb::Rosemary(0x5e4c3a, 0x55593c),
        0x607040 => Herb::Lavender(0x6c6a4e, 0x5e5668),
        0x304018 => Herb::Rosemary(0x564434, 0x4a4c34),
        _ => Herb::Sage(0x6e6c52),
    };
    let n = stem_count as f64;
    // each stem dried a little differently: every other one darker
    let shade = |c: u32, k: f64| {
        let [r, gg, b] = [(c >> 16) & 255, (c >> 8) & 255, c & 255].map(|v| (f64::from(v) * k).round().min(255.0) as u32);
        (r << 16) | (gg << 8) | b
    };
    // each tinted by the dried surface (mottled, veined, a bloom of hairs);
    // the colours a shade lighter to keep their tone under the tint
    let dried = dried_tex();
    let mat = |c: u32, k: f64| lambert!({ map: &dried, color: shade(c, k * 1.08), side: DoubleSide });
    let mats_for = |k: f64| -> (Mat, Mat) {
        match herb {
            Herb::Sage(c) => (mat(c, k), mat(c, k)),
            Herb::Rosemary(w, l) | Herb::Lavender(w, l) => (mat(w, k), mat(l, k)),
        }
    };
    let both = [mats_for(1.0), mats_for(0.86)];
    for i in 0..stem_count {
        // the old leaves' three draws a stem: how it fans out, how far its
        // head leans out from the wall, how it is turned on itself
        let (u_fan, u_tilt, u_turn) = (rnd(), rnd(), rnd());
        let k = i as f64 - (n - 1.0) / 2.0;
        let stem = group();
        stem.set_rotation_order(EulerOrder::XZY);
        stem.position().set(HB_TIE.0 + k * 0.0028, HB_TIE.1, HB_TIE.2 + 0.003 * f64::from(i % 2) - 0.001);
        stem.rotation().set(-(0.06 + 0.16 * u_tilt), (u_turn - 0.5) * 1.2, k / n * 0.3 + (u_fan - 0.5) * 0.1);
        let v = ["A", "B", "C"][i as usize % 3];
        let mats = &both[i as usize % 2];
        match herb {
            Herb::Sage(_) => {
                put_world(&stem, &format!("Real__herbSage{v}"), &mats.0, 0.015);
            }
            Herb::Rosemary(..) => {
                put_world(&stem, &format!("Real__herbRosemaryWood{v}"), &mats.0, 0.015);
                put_world(&stem, &format!("Real__herbRosemaryNeedles{v}"), &mats.1, 0.015);
            }
            Herb::Lavender(..) => {
                put_world(&stem, &format!("Real__herbLavenderStem{v}"), &mats.0, 0.015);
                put_world(&stem, &format!("Real__herbLavenderSpike{v}"), &mats.1, 0.015);
            }
        }
        body.add(&stem);
    }
    bundle.rotation().y = face_wall(dx, dz);
    bundle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&bundle);
}

/// The binding's middle on the wall frame (`nature/garden_things.py` HB_TIE)
const HB_TIE: (f64, f64, f64) = (0.0, 1.874, 0.026);

fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// Part `name` with its texture coordinates laid on at their real scale
/// (`size` m a repeat) from the side each vertex faces most
fn put_world(grp: &Obj, name: &str, mat: &Mat, size: f64) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let (ax, ay, az) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let (u, v) = if ay >= ax && ay >= az {
                (x, z)
            } else if ax >= az {
                (z, y)
            } else {
                (x, y)
            };
            uv.push((u / size) as f32);
            uv.push((v / size) as f32);
        }
        gg.uv = uv;
        gg.touch();
    }
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// Value noise that repeats every `p` cells
fn vn(seed: u32, x: f64, y: f64, p: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let w = |a: i32| a.rem_euclid(p);
    let (a, b) = (hash(seed, w(i), w(j)), hash(seed, w(i + 1), w(j)));
    let (c, d) = (hash(seed, w(i), w(j + 1)), hash(seed, w(i + 1), w(j + 1)));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// A dried leaf's surface as a tint over its colour (a repeat every 15 mm):
/// mottled where it dried unevenly, a network of darker veins sunk in the
/// crinkled blade, a pale bloom of fine hairs; the same on every floor (one
/// texture, made once a thread)
fn dried_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|t| {
        t.get_or_init(|| {
            canvas_tex(96, 96, |ctx, w, h| {
                let iw = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, iw * 4, |j, out| {
                    for i in 0..iw {
                        let (x, y) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                        let m = 0.6 * vn(0x5a6e, x * 6.0, y * 6.0, 6) + 0.4 * vn(0x5a6f, x * 12.0, y * 12.0, 12);
                        let vein = 1.0 - ((vn(0x5a70, x * 9.0, y * 9.0, 9) - 0.5).abs() * 14.0).min(1.0);
                        let hair = (vn(0x5a71, x * 48.0, y * 48.0, 48) - 0.7).max(0.0) * 1.6;
                        let k = (0.84 + 0.2 * m - 0.16 * vein + 0.12 * hair).clamp(0.0, 1.0);
                        let c = (k * 255.0).round() as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[c, c, (f64::from(c) * 0.97) as u8, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

/// The herbs: sage, rosemary or lavender; each its parts (`nature/garden_things.py`) and its
/// colours dried
#[derive(Clone, Copy)]
enum Herb {
    Sage(u32),
    Rosemary(u32, u32),
    Lavender(u32, u32),
}
