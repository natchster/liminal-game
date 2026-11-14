//! `place_framed_maps`
use crate::shared::on_nail;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_framed_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the frame (the old frame colour's draw): walnut, oak, black aluminium, brass-anodised aluminium
    let frame_colors = [0x604020, 0x8b5a2b, 0x2a2a2a, 0xd0a040];
    let color = choice(&frame_colors);
    let kind = frame_colors.iter().position(|&c| c == color).unwrap_or(0);
    let fm = group();
    let mount_y = 1.60;
    fm.rotation().y = face_wall(dx, dz);
    fm.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.03;
    let p = fm.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let size = if kind < 2 { OM_WOOD } else { OM_METAL };
    let body = on_nail(&fm, mount_y, 0.015, size.1, tilt);
    let frame = match kind {
        0 => (
            "Real__officeMapFrameWood",
            lambert!({ map: &pictures::moulding_tex(OM_WOOD, &OM_FACE, Outline::Rect, Finish::Wood([0x50, 0x34, 0x20]), 1200.0, vary) }),
        ),
        1 => (
            "Real__officeMapFrameWood",
            lambert!({ map: &pictures::moulding_tex(OM_WOOD, &OM_FACE, Outline::Rect, Finish::Wood([0x8a, 0x68, 0x40]), 1200.0, vary) }),
        ),
        2 => ("Real__officeMapFrameMetal", phong!({ color: 0x1c1c1e, specular: 0x3a3a3c, shininess: 44 })),
        _ => ("Real__officeMapFrameMetal", phong!({ color: 0x6a5634, specular: 0x6a5a3a, shininess: 44 })),
    };
    // a reprint on modern paper: faded by the office's light more than foxed
    let mut age = Age::engraving(vary);
    age.spots /= 3;
    age.fade += 0.03;
    let pic = ["map_cook", "map_antwerp"][(vary % 2) as usize];
    let dirt = 0.25 + 0.3 * (vary % 59) as f32 / 59.0;
    pictures::add_parts(
        &body,
        &[
            frame,
            ("Real__officeMapGlass", pictures::glass_mat(vary, dirt)),
            ("Real__officeMapPrint", lambert!({ map: &pictures::print_tex(pic, OM_PRINT, 1660.0, age, vary) })),
            ("Real__officeMapBack", lambert!({ color: 0x4a3a2a })),
        ],
    );
    g.world.add(&fm);
}

// ------------------------------------------------------- the framed map ---
/// The print and the two frames' outsides (`office/maps_charts_prints_and_posters.py`: OM_*)
const OM_PRINT: (f64, f64) = (0.274, 0.178);

const OM_WOOD: (f64, f64) = (0.300, 0.204);

const OM_METAL: (f64, f64) = (0.286, 0.190);

/// The gallery moulding's face, outer edge to sight (d, z), as `office_frame_wood`
static OM_FACE: [(f64, f64); 9] = [
    (0.0, 0.0182),
    (0.0005, 0.0196),
    (0.0016, 0.0200),
    (0.0122, 0.0200),
    (0.0130, 0.0196),
    (0.0134, 0.0184),
    (0.0142, 0.0180),
    (0.0158, 0.0176),
    (0.017, 0.0168),
];
