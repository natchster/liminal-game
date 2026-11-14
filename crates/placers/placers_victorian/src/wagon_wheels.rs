//! `place_wagon_wheels`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::face_yaw;
use placers_kit::place::pick;
use placers_kit::place::salt;
use placers_kit::place::wall_tiles;

// Wagon wheel (wall): 3% chance per level. A 12 in wheel (a hand
// cart's or a goat cart's) hung on a nail.
pub(crate) fn place_wagon_wheels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let (wx, wz) = (g.wx(x) + dx as f64 * (CELL * 0.5 - 0.018), g.wz(z) + dz as f64 * (CELL * 0.5 - 0.018));
    let k = salt(wx, wz, 41);
    // what is left of the cart's paint: none, a red, a green
    let paint = match k % 3 {
        0 => None,
        1 => Some([0.38, 0.17, 0.12]),
        _ => Some([0.20, 0.27, 0.18]),
    };
    let wheel = group();
    let w = group();
    w.position().set(WHEEL[0], WHEEL[1], WHEEL[2]);
    put(&w, "Real__wagonWheelSpokes", &lambert!({ map: &spokes_tex(k, paint) }));
    put(&w, "Real__wagonWheelFelloes", &lambert!({ map: &felloes_tex(k, paint) }));
    put(&w, "Real__wagonWheelHub", &lambert!({ map: &hub_tex(k) }));
    put(&w, "Real__wagonWheelTyre", &phong!({ map: &rust_tex(k, true), specular: 0x1a1816, shininess: 12 }));
    put(&w, "Real__wagonWheelIron", &phong!({ map: &rust_tex(k ^ 0x55, false), specular: 0x1a1816, shininess: 12 }));
    wheel.add(&w);
    wheel.rotation().y = face_yaw(dx, dz);
    wheel.position().set(wx, 0, wz);
    g.world.add(&wheel);
}

/// Where the wheel's centre is in the mount (`victorian/things_on_a_wall_and_one_box.py`: the hub's back on
/// the wall, 18 mm behind the spot), its height (the old wheel's)
const WHEEL: [f64; 3] = [0.0, 1.75, 0.028];

/// The first spoke's angle, the first felloe joint's (`WH_SPOKE0`, `WH_JOINT0`)
const SPOKE0: f64 = PI / 2.0 + PI / 10.0;

const JOINT0: f64 = SPOKE0 + PI / 10.0;

/// The parts' half extents across the wheel's face (their textures are laid across them)
const SPOKES: (f64, f64) = (0.1211, 0.1168);

const FELLOES: f64 = 0.1450;

const TYRE: f64 = 0.1500;

const HUB: f64 = 0.0360;

/// Oak gone grey in the weather and then indoors: its grain along `along`
/// (m) and across it, open checks along the grain, the old paint left in
/// patches where it was sheltered (`shelter` 0..1)
fn weathered(seed: u32, along: f64, across: f64, shelter: f64, paint: Option<[f64; 3]>) -> [f64; 3] {
    let g = fbm(seed, along * 18.0, across * 900.0 + fbm(seed ^ 3, along * 6.0, across * 60.0, 2) * 5.0, 3);
    let mut c = mix([0.42, 0.37, 0.30], [0.25, 0.20, 0.15], sstep(0.42, 0.72, g) * 0.8);
    c = mix(c, [0.48, 0.45, 0.40], sstep(0.5, 0.8, fbm(seed ^ 5, along * 9.0, across * 40.0, 2)) * 0.3);
    if let Some(p) = paint {
        let keep = sstep(0.60, 0.66, fbm(seed ^ 7, along * 40.0, across * 120.0, 3) + 0.22 * shelter);
        c = mix(c, p, keep * 0.75);
    }
    let check = (fbm(seed ^ 9, along * 4.0, across * 300.0, 2) - 0.5).abs();
    c = mix(c, [0.08, 0.06, 0.05], (1.0 - sstep(0.0, 0.012, check)) * sstep(0.5, 0.7, vn(seed ^ 11, along * 10.0, across * 30.0)) * 0.8);
    c
}

/// The spokes (u, v across the wheel's face): each spoke's grain along its
/// own length; the paint kept near the hub and the felloe
fn spokes_tex(salt: u32, paint: Option<[f64; 3]>) -> Tex {
    canvas_tex_opt(192, 186, false, move |ctx, w, h| {
        let seed = salt ^ 0x31;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u * 2.0 - 1.0) * SPOKES.0, (v * 2.0 - 1.0) * SPOKES.1);
            let r = (x * x + y * y).sqrt();
            let step = PI / 5.0;
            let a = (y.atan2(x) - SPOKE0).rem_euclid(step);
            let k = ((y.atan2(x) - SPOKE0).rem_euclid(PI * 2.0) / step).round() as u32 % 10;
            let off = if a > step / 2.0 { a - step } else { a };
            let shelter = 1.0 - sstep(0.04, 0.07, r) + sstep(0.09, 0.115, r);
            let c = weathered(seed ^ k.wrapping_mul(97), r, r * off.sin(), shelter, paint);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The felloes (u, v across the wheel's face): the grain round the rim, a
/// dark joint between each two, the paint kept along the tyre's edge
fn felloes_tex(salt: u32, paint: Option<[f64; 3]>) -> Tex {
    canvas_tex_opt(208, 208, false, move |ctx, w, h| {
        let seed = salt ^ 0x43;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u * 2.0 - 1.0) * FELLOES, (v * 2.0 - 1.0) * FELLOES);
            let r = (x * x + y * y).sqrt();
            let th = y.atan2(x);
            let rel = (th - JOINT0).rem_euclid(PI * 2.0 / 5.0);
            let k = ((th - JOINT0).rem_euclid(PI * 2.0) / (PI * 2.0 / 5.0)).floor() as u32;
            let shelter = sstep(0.136, 0.144, r);
            let mut c = weathered(seed ^ k.wrapping_mul(131), rel * 0.13, r, shelter, paint);
            let joint = (rel.min(PI * 2.0 / 5.0 - rel)) * r;
            c = mix(c, [0.07, 0.05, 0.04], (1.0 - sstep(0.0004, 0.0016, joint)) * 0.8);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The hub's face: elm's end grain, its rings round the axle box, checks
/// running out from it, grease gone black round the box
fn hub_tex(salt: u32) -> Tex {
    canvas_tex_opt(96, 96, false, move |ctx, w, h| {
        let seed = salt ^ 0x57;
        fill(ctx, w, h, |u, v| {
            let (x, y) = ((u * 2.0 - 1.0) * HUB, (v * 2.0 - 1.0) * HUB);
            let r = (x * x + y * y).sqrt();
            let th = y.atan2(x);
            let ring = ((r * 900.0 + fbm(seed, th * 3.0, r * 40.0, 2) * 4.0).sin() * 0.5 + 0.5).powf(3.0);
            let mut c = mix([0.36, 0.30, 0.23], [0.20, 0.15, 0.11], ring * 0.6);
            c = mix(c, [0.44, 0.40, 0.34], sstep(0.5, 0.8, fbm(seed ^ 3, x * 60.0, y * 60.0, 2)) * 0.3);
            let check = (fbm(seed ^ 5, th * 6.0, r * 8.0, 2) - 0.5).abs();
            c = mix(c, [0.06, 0.05, 0.04], (1.0 - sstep(0.0, 0.02, check)) * sstep(0.014, 0.020, r) * 0.8);
            c = mix(c, [0.07, 0.06, 0.05], (1.0 - sstep(0.012, 0.019, r)) * 0.85);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Wrought iron rusted: dark scale with brown-orange rust in blooms, worn
/// to the bare metal on the tyre's tread; `tyre`: laid across the tyre's
/// face (its tread the outer ring)
fn rust_tex(salt: u32, tyre: bool) -> Tex {
    canvas_tex_opt(160, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0x69;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.14, 0.12, 0.11];
            let r1 = fbm(seed, u * 14.0, v * 14.0, 3);
            c = mix(c, [0.32, 0.18, 0.09], sstep(0.45, 0.7, r1) * 0.85);
            c = mix(c, [0.42, 0.24, 0.11], sstep(0.65, 0.8, fbm(seed ^ 3, u * 40.0, v * 40.0, 2)) * 0.5);
            if tyre {
                let (x, y) = ((u * 2.0 - 1.0) * TYRE, (v * 2.0 - 1.0) * TYRE);
                let r = (x * x + y * y).sqrt();
                c = mix(c, [0.24, 0.23, 0.22], sstep(0.1485, 0.1498, r) * 0.6);
            }
            c = mix(c, [0.46, 0.43, 0.38], sstep(0.75, 0.95, vn(seed ^ 5, u * 60.0, v * 60.0)) * 0.25);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
