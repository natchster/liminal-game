//! `place_umbrella_stands`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::salt;
use placers_kit::place::wall_tiles;

// Umbrella stand w/ 2-3 umbrellas (floor near wall): 3% chance per
// level. A rolled steel umbrella stand (batch `t07`: 200 mm across, 490 mm
// tall, its seam down the back), painted, chipped at its rim, rusting at its
// foot; two or three furled stick umbrellas in it (r09's), stood on their
// ferrules and leant out against its rim, their crooks turned to the room.
pub(crate) fn place_umbrella_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    // the old colour draw: hammered brown paint, charcoal, black enamel,
    // brass, galvanised steel
    let stand_colors = [0x8b5a2b, 0x404040, 0x201010, 0xc0a060, 0x808080];
    let sc = choice(&stand_colors);
    let si = stand_colors.iter().position(|&c| c == sc).unwrap_or(0);
    let salt = salt(g.wx(x), g.wz(z), si as u32);
    let us = group();
    let (spec, shin) = [(0x262626, 20.0), (0x2a2a2a, 24.0), (0x3a3a3a, 50.0), (0x6a5a34, 40.0), (0x4a4a48, 30.0)][si];
    put(&us, "Real__umbrellaStandBody", &phong!({ map: &stand_tex(si, salt), specular: spec, shininess: shin }));
    // 2-3 furled umbrellas standing in it.
    let umb_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut crooks: Vec<(Obj, f64, f64)> = vec![];
    let fabric_colors = [0x101010, 0x201060, 0x603030, 0x204030, 0xa02040, 0x606060, 0x304060];
    let handle_colors = [0x603020, 0x8b5a2b, 0x101010, 0xa0a0a8];
    for i in 0..umb_count {
        let fc = choice(&fabric_colors);
        let hc = choice(&handle_colors);
        let tilt_angle = (rnd() - 0.5) * 0.30;
        let y_rot = (i as f64 / umb_count as f64) * PI * 2.0 + rnd() * 0.4;
        // the cloth as dyed nylon after years: faded toward a dusty grey
        let fade = |c: u32, sh: u32| ((c >> sh) & 255) * 7 / 20 + 0x2a;
        let cloth = (fade(fc, 16) << 16) | (fade(fc, 8) << 8) | fade(fc, 0);
        let handle = match hc {
            0x603020 => phong!({ color: 0x4a3020, specular: 0x2a2018, shininess: 24 }),
            0x8b5a2b => phong!({ color: 0x6e4c30, specular: 0x2a2018, shininess: 24 }),
            0x101010 => phong!({ color: 0x1e1c1a, specular: 0x262626, shininess: 30 }),
            _ => phong!({ color: 0x5e5e5c, specular: 0x7a7a76, shininess: 60 }),
        };
        let umbrella = group();
        {
            let cloth = lambert!({ color: cloth });
            for (name, mat) in [
                ("Real__umbrellaCanopy", cloth.clone()),
                ("Real__umbrellaStrap", cloth),
                ("Real__umbrellaShaft", phong!({ color: 0x1e1d1c, specular: 0x3a3a38, shininess: 40 })),
                ("Real__umbrellaHandle", handle),
                ("Real__umbrellaMetal", phong!({ color: 0x5e5c58, specular: 0x8a8884, shininess: 60 })),
            ] {
                put(&umbrella, name, &mat);
            }
        }
        // leant out from its ferrule on the bottom till its canopy meets
        // the rim, toward the old turn (less toward the wall behind); its
        // crook turned once they all stand (below)
        let wallward = (-y_rot.sin()).max(0.0);
        let lean = (0.08 + tilt_angle.abs() / 0.15 * 0.05) * (1.0 - 0.45 * wallward);
        let foot = STAND_REACH - STAND_RIM * lean.tan();
        let lean_g = group();
        lean_g.position().set(foot * y_rot.cos(), STAND_FLOOR, foot * y_rot.sin());
        lean_g.rotation().y = -y_rot;
        let tip = group();
        tip.rotation().z = -lean;
        tip.add(&umbrella);
        lean_g.add(&tip);
        us.add(&lean_g);
        crooks.push((umbrella, y_rot, tilt_angle));
    }
    // the crooks turned apart: the leftmost out to the left, the rightmost
    // to the right, one between them to the room, each a little round
    // toward the room and either way by its old tilt; so no crook reaches
    // over another's, seen from the room or from the side (turned all one
    // way by the tilt alone, two touched: M8). A crook's way is
    // (cos a, sin a) in the stand's x and z (+z the room, -z its wall)
    let mut by_x: Vec<usize> = (0..crooks.len()).collect();
    by_x.sort_by(|&a, &b| crooks[a].1.cos().total_cmp(&crooks[b].1.cos()));
    for (rank, &i) in by_x.iter().enumerate() {
        let (umbrella, y_rot, tilt) = &crooks[i];
        let way = if rank == 0 {
            PI - 0.40
        } else if rank + 1 == by_x.len() {
            0.40
        } else {
            PI / 2.0
        };
        let crook = way + tilt * 0.8;
        umbrella.rotation().y = y_rot - crook;
    }
    us.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    // Position at wall-adjacent floor spot.
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.140);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.140);
    us.position().set(px, 0, pz);
    g.world.add(&us);
}

/// The umbrella stand's inside (batch `t07`): its bottom's top, the canopy's
/// clearance under its rim (the bead's inside radius less a furled canopy's
/// 29 mm), its rim's height over that bottom
const STAND_FLOOR: f64 = 0.0125;

const STAND_REACH: f64 = 0.0985 - 0.029;

const STAND_RIM: f64 = 0.490 - 0.0125;

/// The stand's finish laid round it (u round, v up; inside and out): its
/// paint (or brass, or zinc's spangle), dust on its bead, scratched inside
/// its rim where umbrellas went in, chipped to the steel at the rim and the
/// foot, rust where the wet stood at its foot, a run of dried drips
fn stand_tex(si: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, true, move |ctx, w, h| {
        let seed = salt ^ 0x57a;
        let base = [[0.27, 0.19, 0.13], [0.19, 0.19, 0.185], [0.12, 0.085, 0.08], [0.42, 0.32, 0.15], [0.40, 0.41, 0.40]][si];
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 30.0, v * 14.0, 4);
            let mut c = mix(base, [base[0] * 0.75, base[1] * 0.75, base[2] * 0.75], n);
            match si {
                0 => {
                    // hammered: dimples of light and dark
                    let (d1, d2) = (vn(seed ^ 2, u * 140.0, v * 70.0), vn(seed ^ 4, u * 140.0 + 0.5, v * 70.0 + 0.5));
                    c = mix(c, [base[0] * 1.4, base[1] * 1.35, base[2] * 1.3], (d1 - d2).max(0.0) * 0.8);
                }
                3 => {
                    // the lacquer gone, brass tarnished in clouds
                    c = mix(c, [0.24, 0.20, 0.12], sstep(0.45, 0.75, fbm(seed ^ 6, u * 6.0, v * 4.0, 4)) * 0.7);
                }
                4 => {
                    let sp = vn(seed ^ 8, u * 40.0, v * 20.0);
                    c = mix(c, [0.50, 0.51, 0.50], (sp - 0.5).abs() * 0.5);
                    c = mix(c, [0.62, 0.62, 0.58], sstep(0.6, 0.8, fbm(seed ^ 10, u * 5.0, v * 3.0, 3)) * 0.4);
                }
                _ => {}
            }
            // chips at the rim and the foot (not the brass)
            let chip = vn(seed ^ 12, u * 90.0, v * 40.0);
            let band = sstep(0.93, 0.99, v) + sstep(0.06, 0.0, v);
            if si != 3 && chip > 0.8 - 0.2 * band.min(1.0) && band > 0.05 {
                c = mix(c, [0.22, 0.21, 0.20], 0.8);
            }
            // rust from the foot up, where the wet stood
            let rust = sstep(0.12, 0.0, v) * sstep(0.45, 0.7, fbm(seed ^ 14, u * 12.0, v * 4.0, 3));
            c = mix(c, [0.30, 0.17, 0.09], if si == 3 { rust * 0.2 } else { rust * 0.8 });
            // dried drips run down from the rim
            let drip = (vn(seed ^ 16, u * 70.0, 0.5) - 0.72).max(0.0) * 4.0 * sstep(0.4, 0.95, v);
            c = mix(c, [0.36, 0.34, 0.30], drip * 0.5);
            // grime toward the foot, dust on the bead
            c = mix(c, [0.14, 0.13, 0.11], sstep(0.25, 0.0, v) * 0.35);
            c = mix(c, [0.48, 0.46, 0.42], sstep(0.97, 1.0, v) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
