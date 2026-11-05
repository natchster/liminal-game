//! The botanical print (Blender batch `t02`): a real plate of a flower
//! (CC0, `visuals::pictures`: Redouté's Rosa gallica, Ehret's Pulmonaria
//! and Magnolia, coloured by hand; van Spaendonck's poppy in stipple), as
//! such plates were reprinted, in a cream mat cut with a bevel (its foot the
//! wider), under glass, in a 16 mm half-round (rosewood, mahogany, walnut or
//! bird's-eye maple: the old frame colour's draw), hung on its wire from a
//! hook. The paper foxed and yellowed, the mat's bevel gone brown.
use super::*;
use game::visuals::pictures::{self, Age, Finish, Outline, Window};

/// The frame's outside, the mat (the rebate), its window (w, h, raised by),
/// the paper behind it and the wire's apex (`t02.py`: BP_*)
const SIZE: (f64, f64) = (0.306, 0.386);
const MAT: (f64, f64) = (0.281, 0.361);
const WIN: (f64, f64, f64) = (0.196, 0.280, 0.004);
const PAPER: (f64, f64) = (0.208, 0.292);
const APEX: f64 = 0.128;
/// The half-round's face, outer edge to sight (d, z), as `botanical_frame`
static FACE: [(f64, f64); 11] = [
    (0.0, 0.0140),
    (0.0008, 0.0168),
    (0.0026, 0.0189),
    (0.0055, 0.0200),
    (0.0085, 0.0197),
    (0.0110, 0.0183),
    (0.0126, 0.0166),
    (0.0136, 0.0161),
    (0.0146, 0.0156),
    (0.0154, 0.0145),
    (0.016, 0.0130),
];

pub(crate) fn place_botanical_prints(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // the moulding (the old frame colour's draw)
    let stain = match ri(4) {
        0 => [0x36, 0x1e, 0x14],
        1 => [0x4a, 0x24, 0x18],
        2 => [0x50, 0x34, 0x20],
        _ => [0xa8, 0x84, 0x52],
    };
    let bp = group();
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.007);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.007);
    let center_y = 1.55;
    if cdx.abs() == 1 {
        bp.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        bp.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        bp.rotation().y = if cdz > 0 { PI } else { 0.0 };
        bp.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.015;
    let p = bp.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = pictures::hang(&bp, 0.0, 0.007, APEX, SIZE.1, 0.010, tilt);
    let pic = ["bot_rose", "bot_pulmonaria", "bot_magnolia", "bot_poppy"][(vary % 4) as usize];
    let dirt = 0.25 + 0.3 * (vary % 61) as f32 / 61.0;
    // the sheet's edges lie under the mat: what yellowed most is hidden
    let mut age = Age::engraving(vary);
    age.edge *= 0.3;
    pictures::add_parts(
        &body,
        &[
            (
                "Real__botanicalPrintFrame",
                lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, Finish::Wood(stain), 1200.0, vary) }),
            ),
            (
                "Real__botanicalPrintMat",
                lambert!({ map: &pictures::mat_tex(MAT, Window::Rect(WIN.0, WIN.1, WIN.2), [0xdc, 0xd4, 0xbe], vary, None) }),
            ),
            ("Real__botanicalPrintGlass", pictures::glass_mat(vary, dirt)),
            ("Real__botanicalPrintPaper", lambert!({ map: &pictures::print_tex(pic, PAPER, 1370.0, age, vary) })),
            ("Real__botanicalPrintBack", lambert!({ color: 0x4a3a2a })),
            ("Real__botanicalPrintWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&bp);
}
