//! Batch `t17`'s botanical, made real (`docs/design/PROPS.md` P3, "its picture a
//! picture"; `assets/src/blender/props/t17.py`, `t17.notes.md`): a small
//! hand-coloured botanical plate (CC0, `visuals::pictures`: Redouté's Rosa
//! gallica, Ehret's Pulmonaria and Magnolia, van Spaendonck's poppy) in a
//! cream mat under glass, in a 20 x 30 cm frame of a 16 mm flat moulding,
//! hung on its wire from a hook. Its placer keeps the old one's draws, call
//! for call.
use crate::{face_yaw, pick, wall_tiles};
use game::prelude::*;
use game::visuals::pictures::{self, Age, Finish, Outline, Window};

/// The frame's outside, the mat (the rebate less a hair), its window (w, h,
/// raised by), the paper behind it and the wire's apex above the middle
/// (`t17.py`: BT_*)
const SIZE: (f64, f64) = (0.200, 0.300);
const MAT: (f64, f64) = (0.175, 0.275);
const WIN: (f64, f64, f64) = (0.118, 0.188, 0.004);
const PAPER: (f64, f64) = (0.132, 0.202);
const APEX: f64 = 0.102;
/// The moulding's face, outer edge to sight (d, z), as `t17.py` BT_FACE
static FACE: [(f64, f64); 9] = [
    (0.0, 0.0150),
    (0.0006, 0.0166),
    (0.0018, 0.0177),
    (0.0035, 0.0181),
    (0.0110, 0.0181),
    (0.0128, 0.0177),
    (0.0140, 0.0168),
    (0.0150, 0.0156),
    (0.016, 0.0146),
];

// Framed botanical illustration: 3% chance per level. A botanical
// plate framed small (Blender batch `t17`): the print foxed and browned
// under its glass, the mat's bevel gone brown, the moulding's stain rubbed
// on its edges; askew on its wire.
pub fn place_botanicals(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let bot = group();
    let mount_y = 1.65;
    // askew on its hook (the old tilt's draw: the old frame turned about the
    // floor under it, the new swings about its hook)
    let tilt = (rnd() - 0.5) * 0.05;
    bot.rotation().y = face_yaw(dx, dz);
    bot.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = bot.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = pictures::hang(&bot, mount_y, 0.013, APEX, SIZE.1, 0.008, tilt);
    let pic = ["bot_pulmonaria", "bot_rose", "bot_poppy", "bot_magnolia"][(vary % 4) as usize];
    // the moulding: stained oak, walnut, mahogany, or ebonised (where it hangs)
    let finish = match (vary >> 3) % 4 {
        0 => Finish::Wood([0x8a, 0x66, 0x40]),
        1 => Finish::Wood([0x4e, 0x34, 0x22]),
        2 => Finish::Wood([0x5a, 0x2c, 0x1c]),
        _ => Finish::Ebony,
    };
    let dirt = 0.3 + 0.3 * (vary % 53) as f32 / 53.0;
    // the sheet's edges lie under the mat: what yellowed most is hidden
    let mut age = Age::engraving(vary);
    age.edge *= 0.3;
    pictures::add_parts(
        &body,
        &[
            ("Real__botFrame", lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1200.0, vary) })),
            ("Real__botMat", lambert!({ map: &pictures::mat_tex(MAT, Window::Rect(WIN.0, WIN.1, WIN.2), [0xd8, 0xcf, 0xb6], vary, None) })),
            ("Real__botGlass", pictures::glass_mat(vary, dirt)),
            ("Real__botPaper", lambert!({ map: &pictures::print_tex(pic, PAPER, 1400.0, age, vary) })),
            ("Real__botBack", lambert!({ color: 0x4e3e2c })),
            ("Real__botWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&bot);
}
