//! The cabin's framed map (Blender batch `t02`): a real etched map of the
//! country round a town (CC0, `visuals::pictures`: Paris and its villages,
//! the Hogenberg workshop, 1590; the Moselle round Trier, de Fer and
//! Mortier, 1702), printed at its own size (28 x 21 cm) and trimmed to a
//! plain pine frame, under glass, hung on its wire from a hook, leaning out
//! from the wall at the top, a little askew. The paper yellowed from its
//! edges, foxed; the pine's stain rubbed pale on its round.
use crate::{face_yaw, pick, wall_tiles};
use game::prelude::*;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// The sheet under the glass, the frame's outside and the wire's apex above
/// its middle (`t02.py`: VM_*, `vintage_wire`)
const PRINT: (f64, f64) = (0.276, 0.208);
const SIZE: (f64, f64) = (0.300, 0.232);
const APEX: f64 = 0.086;
/// The pine moulding's face, outer edge to sight (d, z), as `vintage_frame`
static FACE: [(f64, f64); 10] = [
    (0.0, 0.0140),
    (0.0007, 0.0161),
    (0.0020, 0.0177),
    (0.0040, 0.0186),
    (0.0075, 0.0189),
    (0.0110, 0.0186),
    (0.0132, 0.0178),
    (0.0144, 0.0166),
    (0.0155, 0.0130),
    (0.0160, 0.0122),
];

// Framed vintage map: 3% chance per level.
pub fn place_vintage_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let map = group();
    let mount_y = 1.65;
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.05;
    map.rotation().y = face_yaw(dx, dz);
    map.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = map.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = pictures::hang(&map, mount_y, 0.013, APEX, SIZE.1, 0.009, tilt);
    let pic = ["map_paris", "map_trier"][(vary % 2) as usize];
    // a pine stain: walnut-dark, or an old orange shellac gone brown
    let stain = if (vary >> 3) % 2 == 0 { [0x4e, 0x36, 0x22] } else { [0x6a, 0x46, 0x26] };
    let dirt = 0.3 + 0.3 * (vary % 71) as f32 / 71.0;
    pictures::add_parts(
        &body,
        &[
            (
                "Real__vintageMapFrame",
                lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, Finish::Wood(stain), 1100.0, vary) }),
            ),
            ("Real__vintageMapGlass", pictures::glass_mat(vary, dirt)),
            ("Real__vintageMapPrint", lambert!({ map: &pictures::print_tex(pic, PRINT, 1450.0, Age::engraving(vary), vary) })),
            ("Real__vintageMapBack", lambert!({ color: 0x4a3a2a })),
            ("Real__vintageMapWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&map);
}
