//! `place_hanging_plants`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::fbm as fbm1;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::rgb as hex;
use placers_kit::place::face_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging plant: 3% chance per level.
pub(crate) fn place_hanging_plants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // A golden pothos left for years in a moulded plastic hanging pot on a
    // three-wire hanger, from a wrought iron bracket screwed to the wall
    // (Blender batch `t06`): gone leggy, bare vines half a metre long, a
    // leaf every few nodes, some yellow, some dried brown. The old pot
    // colour's draw chooses the pot's plastic; the old leaf colour's, how
    // the plant has fared (a dull green, a dark green, the golden pothos's
    // marbling, gone pale and yellow)
    let pot_colors: [u32; 4] = [0x8b5a2b, 0x606060, 0xa0603c, 0xd0a070];
    let pot_c = match pick(&pot_colors) {
        0x8b5a2b => 0x8e5638,
        0x606060 => 0x4a4a48,
        0xa0603c => 0xa46a4c,
        _ => 0xb4a486,
    };
    let leaf_colors: [u32; 4] = [0x408040, 0x306028, 0x507838, 0x60a058];
    let (green, gold, yellow_more) = match pick(&leaf_colors) {
        0x408040 => (0x4a6436, 0.35, 0.0),
        0x306028 => (0x34502c, 0.2, 0.0),
        0x507838 => (0x56703a, 0.8, 0.0),
        _ => (0x6a7a40, 0.5, 0.3),
    };
    let plant = group();
    // the wall is 0.080 behind the old group's origin
    let body = group();
    body.position().z = -0.080;
    plant.add(&body);
    plant.rotation().y = face_yaw(dx, dz);
    plant.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.080), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.080));
    let salt = {
        let p = plant.pos();
        let (a, b) = ((p.x * 1000.0).round() as i64 as u64, (p.z * 1000.0).round() as i64 as u64);
        let h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        (h ^ (h >> 31)) as u32
    };
    put(&body, "Real__hangPlantBracket", &phong!({ color: 0x2a2724, specular: 0x1c1a18, shininess: 14 }));
    put(&body, "Real__hangPlantScrews", &lambert!({ color: 0x5c5248 }));
    put(&body, "Real__hangPlantHanger", &phong!({ color: 0x86847e, specular: 0x3a3a38, shininess: 30 }));
    put(&body, "Real__hangPlantPot", &phong!({ map: &pot_tex(hex(pot_c), salt), specular: 0x161616, shininess: 20 }));
    if let Some(geo) = prop_part_as_modelled("Real__hangPlantSoil") {
        let geo = geo.duplicate();
        {
            let mut gg = geo.g();
            let n = gg.position.len() / 3;
            let mut uv = Vec::with_capacity(n * 2);
            for i in 0..n {
                let (px, pz) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
                uv.push((0.5 + px / (2.0 * HP_SOIL_R)) as f32);
                uv.push((0.5 - (pz - HP_POT_Z) / (2.0 * HP_SOIL_R)) as f32);
            }
            gg.uv = uv;
            gg.touch();
        }
        body.add(&mesh(geo, &lambert!({ map: &soil_tex(salt) })));
    }
    let stem = lambert!({ color: 0x5a6438 });
    let leaf = phong!({ map: &leaf_tex(hex(green), gold, 0.3, salt), specular: 0x161a12, shininess: 14, side: DoubleSide });
    let yellow = lambert!({ map: &leaf_tex(mix(hex(0x9a9246), hex(0xa89a50), yellow_more), 0.2, 0.6, salt ^ 9), side: DoubleSide });
    let dead = lambert!({ map: &leaf_tex(hex(0x7a6444), 0.0, 0.8, salt ^ 17), side: DoubleSide });
    put(&body, "Real__hangPlantCrownStem", &stem);
    put_world(&body, "Real__hangPlantCrownGreen", &leaf, 0.12);
    put_world(&body, "Real__hangPlantCrownYellow", &yellow, 0.12);
    put_world(&body, "Real__hangPlantCrownDead", &dead, 0.12);
    // the vines: over the rim at their places round the front of the pot,
    // each hanging from where it crosses the rim
    let vine_count = 5 + (rnd() * 4.0).floor() as i32;
    let mut sway: Vec<(Obj, f64, f64, f64)> = vec![];
    for i in 0..vine_count {
        // its place round the rim (the old swing's draw), its own lean
        // sideways (the old turn's draw) and its phase (the old phase's);
        // the old depth's draw is kept for its place
        let u_place = rnd();
        let _ = rnd(); // keeps the old draw (the leaf's depth off the pot)
        let lean = (rnd() - 0.5) * 0.06;
        let phase = rnd() * PI * 2.0 + i as f64 * 0.3;
        let t = (i as f64 + 0.5) / vine_count as f64;
        let theta = -1.85 + 3.7 * t + (u_place - 0.5) * 0.35;
        let around = group();
        around.position().set(0, HP_RIM_Y, HP_POT_Z);
        around.rotation().y = theta;
        let pivot = group();
        pivot.position().z = HP_RIM_R;
        pivot.rotation().z = lean;
        around.add(&pivot);
        let v = VINES[i as usize % VINES.len()];
        // the second time round a vine is turned about its own hang
        if i as usize >= VINES.len() {
            pivot.rotation().y = 0.6;
        }
        put(&pivot, &format!("Real__hangPlantVine{v}Stem"), &stem);
        put_world(&pivot, &format!("Real__hangPlantVine{v}Green"), &leaf, 0.12);
        put_world(&pivot, &format!("Real__hangPlantVine{v}Yellow"), &yellow, 0.12);
        put_world(&pivot, &format!("Real__hangPlantVine{v}Dead"), &dead, 0.12);
        body.add(&around);
        sway.push((pivot, lean, phase, 0.006 + 0.01 * t));
    }
    g.world.add(&plant);
    // a draught stirs the vines, barely
    g.animate(move |_g, _dt, t| {
        for (p, lean, phase, amp) in &sway {
            p.rotation().z = lean + (t * 0.32 + phase).sin() * amp;
        }
    });
}

/// The pot (`t06.py` HP_*): its axis out from the wall, its rim's top and
/// radius; where the vines cross the rim is on it
const HP_POT_Z: f64 = 0.190;

const HP_RIM_Y: f64 = 1.645;

const HP_RIM_R: f64 = 0.0825;

const HP_SOIL_R: f64 = 0.071;

/// The vines (`t06.py` VINES)
const VINES: [&str; 4] = ["A", "B", "C", "D"];

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

/// fbm that repeats every `p` in x and y (blended across its seams)
fn fbm(seed: u32, x: f64, y: f64, p: f64, oct: u32) -> f64 {
    let (u, v) = ((x / p).rem_euclid(1.0), (y / p).rem_euclid(1.0));
    let (x, y) = (u * p, v * p);
    let a = fbm1(seed, x, y, oct);
    let b = fbm1(seed, x - p, y, oct);
    let c = fbm1(seed, x, y - p, oct);
    let d = fbm1(seed, x - p, y - p, oct);
    (a * (1.0 - u) + b * u) * (1.0 - v) + (c * (1.0 - u) + d * u) * v
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
/// up, 0..1) gives straight sRGB
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
    let iw = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..3 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A pothos leaf's surface, repeating every 12 cm: its green (`base`),
/// marbled with the golden pothos's cream-yellow streaks (`gold`: how
/// much), brown spots where it was hurt, a film of dust
fn leaf_tex(base: [f64; 3], gold: f64, brown: f64, seed: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 6.0, v * 6.0);
            let n = fbm(seed, x, y, 6.0, 3);
            let mut c = base.map(|k| k * (0.85 + 0.3 * n));
            // the streaks: long, feathered, running one way
            let st = fbm(seed ^ 3, x * 0.5 + y, y * 0.5, 6.0, 4);
            c = mix(c, [0.72, 0.66, 0.4], sstep(0.62 - gold * 0.12, 0.7 - gold * 0.1, st) * gold);
            let sp = fbm(seed ^ 5, x * 1.5, y * 1.5, 9.0, 3);
            c = mix(c, [0.33, 0.24, 0.13], sstep(0.66, 0.72, sp) * brown);
            mix(c, [0.55, 0.54, 0.5], 0.1 + 0.08 * fbm(seed ^ 7, x * 2.0, y * 2.0, 12.0, 2))
        });
    })
}

/// The moulded pot round and up (u round, v up): its plastic faded paler
/// where the light fell, dust on its rim, the white crust of hard water
/// run down from its rim and round its saucer
fn pot_tex(base: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let wrap = |sd: u32, fu: f64, fv: f64| {
                let a = vn(sd, u * fu, v * fv);
                let b = vn(sd, (u - 1.0) * fu, v * fv);
                a * (1.0 - u) + b * u
            };
            let mut c = base.map(|k| k * (0.92 + 0.12 * wrap(salt, 20.0, 6.0)));
            c = mix(c, [0.7, 0.68, 0.64], 0.15 * wrap(salt ^ 1, 3.0, 2.0));
            // runs of hard water down from the rim
            let run = wrap(salt ^ 2, 40.0, 1.2);
            c = mix(c, [0.8, 0.78, 0.72], sstep(0.62, 0.75, run) * sstep(0.3, 0.9, v) * 0.5);
            // the saucer's crust and the dust over the rim
            c = mix(c, [0.78, 0.76, 0.7], sstep(0.14, 0.05, v) * 0.35);
            mix(c, [0.6, 0.58, 0.54], sstep(0.88, 1.0, v) * 0.4)
        });
    })
}

/// The soil from above (its 14 cm): dried grey-brown, shrunk from the wall
/// and cracked, perlite's white grains, the dark gap at its edge
fn soil_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = vn(salt, u * 30.0, v * 30.0) * 0.6 + vn(salt ^ 1, u * 90.0, v * 90.0) * 0.4;
            let mut c = mix([0.2, 0.16, 0.12], [0.36, 0.31, 0.25], n);
            let crack = sstep(0.04, 0.0, (vn(salt ^ 3, u * 9.0, v * 9.0) - 0.5).abs());
            c = mix(c, [0.08, 0.06, 0.05], crack * 0.8);
            let grain = (vn(salt ^ 2, u * 160.0, v * 160.0) - 0.86).max(0.0) * 7.0;
            c = mix(c, [0.8, 0.79, 0.74], grain.min(1.0));
            let rho = ((u - 0.5).hypot(v - 0.5) * 2.0).min(1.0);
            mix(c, [0.06, 0.05, 0.04], sstep(0.88, 1.0, rho))
        });
    })
}
