//! The parlour's antique map (Blender batch `t02`): a hand-coloured map of
//! the 17th or 18th century (CC0, `visuals::pictures`: Blaeu's world map in
//! its border of planets, seasons and wonders; van Keulen's sea chart of the
//! Dvina; le Clerc's planisphere of the constellations; Homann's Gulf of
//! Mexico), as the 19th century reprinted them, reduced to a 40 x 34 cm
//! sheet, under a gilt fillet and glass in a 25 mm cushion moulding (walnut,
//! ebonised, oak or mahogany: the old frame colour's draw), hung on its wire
//! from a hook, leaning out at the top. Its paper yellowed and foxed.
use super::*;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// The sheet, the fillet's and the frame's outsides, the wire's apex
/// (`t02.py`: AM_*, `antique_wire`)
const MAP: (f64, f64) = (0.404, 0.336);
const FILLET: (f64, f64) = (0.418, 0.350);
const SIZE: (f64, f64) = (0.460, 0.392);
const APEX: f64 = 0.126;
/// The cushion moulding's face, outer edge to sight (d, z), as `antique_frame`
static FACE: [(f64, f64); 15] = [
    (0.0, 0.0195),
    (0.0008, 0.0222),
    (0.0024, 0.0246),
    (0.0048, 0.0266),
    (0.0080, 0.0277),
    (0.0110, 0.0280),
    (0.0140, 0.0274),
    (0.0165, 0.0258),
    (0.0180, 0.0240),
    (0.0192, 0.0232),
    (0.0204, 0.0233),
    (0.0216, 0.0238),
    (0.0228, 0.0232),
    (0.0238, 0.0218),
    (0.025, 0.0196),
];
/// The gilt fillet's face, as `antique_fillet`
static FILLET_FACE: [(f64, f64); 9] = [
    (0.0, 0.0104),
    (0.0045, 0.0104),
    (0.0060, 0.0098),
    (0.0076, 0.0084),
    (0.0092, 0.0074),
    (0.0104, 0.0073),
    (0.0116, 0.0068),
    (0.0124, 0.0060),
    (0.013, 0.0052),
];

pub(crate) fn place_antique_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let mut candidates: Vec<(i32, i32, i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    candidates.push((x, z, dx, dz));
                }
            }
        }
    }
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    let (cx, cz, cdx, cdz) = c;
    // the moulding (the old frame colour's draw): walnut, ebonised, oak, mahogany
    let finish = match ri(4) {
        0 => Finish::Wood([0x50, 0x34, 0x20]),
        1 => Finish::Ebony,
        2 => Finish::Wood([0x86, 0x66, 0x40]),
        _ => Finish::Wood([0x4a, 0x24, 0x18]),
    };
    let map = group();
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.007);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.007);
    let center_y = 1.55;
    if cdx.abs() == 1 {
        map.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        map.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        map.rotation().y = if cdz > 0 { PI } else { 0.0 };
        map.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.020;
    let p = map.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = pictures::hang(&map, 0.0, 0.007, APEX, SIZE.1, 0.010, tilt);
    let pic = ["map_world", "map_dvina", "map_sky", "map_mexico"][(vary % 4) as usize];
    let frame_mat = match finish {
        Finish::Ebony => {
            phong!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1000.0, vary), specular: 0x1a1a1a, shininess: 30 })
        }
        _ => lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1000.0, vary) }),
    };
    let dirt = 0.3 + 0.3 * (vary % 67) as f32 / 67.0;
    pictures::add_parts(
        &body,
        &[
            ("Real__antiqueMapFrame", frame_mat),
            (
                "Real__antiqueMapFillet",
                phong!({ map: &pictures::moulding_tex(FILLET, &FILLET_FACE, Outline::Rect, Finish::Gilt, 1300.0, vary ^ 0x5a), specular: 0x3a3020, shininess: 22 }),
            ),
            ("Real__antiqueMapGlass", pictures::glass_mat(vary, dirt)),
            ("Real__antiqueMapPrint", lambert!({ map: &pictures::print_tex(pic, MAP, 1180.0, Age::engraving(vary), vary) })),
            ("Real__antiqueMapBack", lambert!({ color: 0x4a3a2a })),
            ("Real__antiqueMapWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&map);
}
