//! `place_graduation_photos`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed graduation photo w/ tassel (wall): 2.5% chance per level.
pub(crate) fn place_graduation_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wall_tiles);
    // the frame (one draw, the old frame's colour): walnut, gilt, grey paint, oak
    let frame_colors = [0x604020, 0xd0a040, 0x808080, 0x8b5a2b];
    let finish = match pick(&frame_colors) {
        0x604020 => Finish::Wood([0x52, 0x36, 0x22]),
        0xd0a040 => Finish::Gilt,
        0x808080 => Finish::Paint([0x7a, 0x78, 0x72]),
        _ => Finish::Wood([0x86, 0x62, 0x3c]),
    };
    // the tassel's colours (one draw): old gold, maroon, navy, green, faded
    let tassel_color_pool = [0xd0a040, 0xa02040, 0x2060a0, 0x408060];
    let tassel = match pick(&tassel_color_pool) {
        0xd0a040 => 0x8e7444,
        0xa02040 => 0x66282e,
        0x2060a0 => 0x2c3a52,
        _ => 0x34483a,
    };
    let gp = group();
    let mount_y = 1.60;
    gp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    gp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.04;
    let p = gp.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    // the nail under the sawtooth, 16 mm below the top; the wall 15 mm behind the spot
    let nail = group();
    nail.position().set(0, mount_y + SIZE.1 / 2.0 - 0.016, -0.015);
    nail.rotation().z = tilt;
    gp.add(&nail);
    let body = group();
    body.position().y = -(SIZE.1 / 2.0 - 0.016);
    nail.add(&body);
    let frame_mat = match finish {
        Finish::Gilt => {
            phong!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1400.0, vary), specular: 0x3a3020, shininess: 22 })
        }
        _ => lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1400.0, vary) }),
    };
    let dirt = 0.22 + 0.3 * (vary % 83) as f32 / 83.0;
    let parts: [(&str, Mat); 6] = [
        ("Real__gradPhotoFrame", frame_mat),
        (
            "Real__gradPhotoMat",
            lambert!({ map: &pictures::mat_tex(MAT, Window::Rect(WIN.0, WIN.1, WIN.2), [0xdc, 0xd4, 0xbe], vary, None) }),
        ),
        ("Real__gradPhotoGlass", pictures::glass_mat(vary, dirt)),
        ("Real__gradPhotoBack", lambert!({ color: 0x4a3a2a })),
        ("Real__framedPhotoSawtooth", phong!({ color: 0x5c5c5a, specular: 0x404040, shininess: 40 })),
        ("Real__gradPhotoTassel", lambert!({ color: tassel })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            if name.ends_with("Sawtooth") {
                m.position().y = SIZE.1 / 2.0 - 0.016;
            }
            if name.ends_with("Glass") {
                m.set_render_order(2);
            }
            body.add(&m);
        }
    }
    // the portrait behind the mat
    let size = (0.098, 0.134);
    let print = mesh(plane_geo!(size.0, size.1), lambert!({ map: &pictures::print_tex("graduate", size, 2600.0, Age::photo(vary), vary) }));
    print.position().set(0, WIN.2, ZL - 0.0037);
    body.add(&print);
    g.world.add(&gp);
}

/// The frame's outside, the lip's underside, the mat (the rebate) and its
/// window (w, h, raised by) (`domestic/pictures_on_the_walls.py`: GR_*)
const SIZE: (f64, f64) = (0.172, 0.222);

const ZL: f64 = 0.0070;

const MAT: (f64, f64) = (0.143, 0.193);

const WIN: (f64, f64, f64) = (0.088, 0.124, 0.004);

/// The moulding's face, outer edge to sight (d, z), as `grad_frame`
static FACE: [(f64, f64); 12] = [
    (0.0, 0.0125),
    (0.0008, 0.0148),
    (0.0025, 0.0161),
    (0.0045, 0.0160),
    (0.0058, 0.0151),
    (0.0066, 0.0141),
    (0.0140, 0.0140),
    (0.0146, 0.0146),
    (0.0156, 0.0147),
    (0.0166, 0.0140),
    (0.0174, 0.0126),
    (0.018, 0.0110),
];
