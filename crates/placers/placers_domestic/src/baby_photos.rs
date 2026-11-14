//! `place_baby_photos`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_baby_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // the frame's paint (one draw, the old frame's colour): pink, blue,
    // cream or rose, as they are after years
    let frame_colors: [u32; 4] = [0xf0d8e8, 0xd8e8f0, 0xf0f0d0, 0xf0d0d0];
    let paint = match pick(&frame_colors) {
        0xf0d8e8 => [0xc8, 0xa8, 0xae],
        0xd8e8f0 => [0xa6, 0xb4, 0xba],
        0xf0f0d0 => [0xcc, 0xc2, 0x9a],
        _ => [0xc6, 0x9e, 0x94],
    };
    let bp = group();
    let mount_y = 1.55;
    bp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    bp.position().set(px, 0, pz);
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.04;
    let vary = pictures::vary_of(px, pz, tilt);
    // the nail under the sawtooth, 16 mm below the top; the wall 15 mm behind the spot
    let nail = group();
    nail.position().set(0, mount_y + SIZE / 2.0 - 0.016, -0.015);
    nail.rotation().z = tilt;
    bp.add(&nail);
    let body = group();
    body.position().y = -(SIZE / 2.0 - 0.016);
    nail.add(&body);
    let note = if vary % 5 < 2 { Some(NOTES[(vary / 5 % NOTES.len() as u32) as usize].to_string()) } else { None };
    let dirt = 0.25 + 0.3 * (vary % 89) as f32 / 89.0;
    let parts: [(&str, Mat); 5] = [
        (
            "Real__babyPhotoFrame",
            lambert!({ map: &pictures::moulding_tex((SIZE, SIZE), &FACE, Outline::Rect, Finish::Paint(paint), 1300.0, vary) }),
        ),
        (
            "Real__babyPhotoMat",
            lambert!({ map: &pictures::mat_tex((MAT, MAT), Window::Oval(OVAL.0, OVAL.1, OVAL.2), [0xe2, 0xd8, 0xc0], vary, note) }),
        ),
        ("Real__babyPhotoGlass", pictures::glass_mat(vary, dirt)),
        ("Real__babyPhotoBack", lambert!({ color: 0x4a3a2a })),
        ("Real__framedPhotoSawtooth", phong!({ color: 0x5c5c5a, specular: 0x404040, shininess: 40 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            if name.ends_with("Sawtooth") {
                m.position().y = SIZE / 2.0 - 0.016;
            }
            if name.ends_with("Glass") {
                m.set_render_order(2);
            }
            body.add(&m);
        }
    }
    // the print behind the mat
    let name = BABIES[(vary % BABIES.len() as u32) as usize];
    let size = (0.100, 0.130);
    let print = mesh(plane_geo!(size.0, size.1), lambert!({ map: &pictures::print_tex(name, size, 2600.0, Age::photo(vary), vary) }));
    print.position().set(0, OVAL.2, ZL - 0.0037);
    body.add(&print);
    g.world.add(&bp);
}

/// The frame's outside (square), the lip's underside, the rebate (the mat,
/// glass and back inside it) and the mat's oval window (`t01.py`: BB_*)
const SIZE: f64 = 0.200;

const ZL: f64 = 0.0070;

const MAT: f64 = 0.143;

const OVAL: (f64, f64, f64) = (0.0430, 0.0560, 0.0030);

/// The moulding's face, outer edge to sight (d, z), as `baby_frame`
static FACE: [(f64, f64); 13] = [
    (0.0, 0.0120),
    (0.0015, 0.0168),
    (0.0045, 0.0200),
    (0.0090, 0.0218),
    (0.0140, 0.0221),
    (0.0190, 0.0212),
    (0.0232, 0.0190),
    (0.0262, 0.0160),
    (0.0278, 0.0150),
    (0.0294, 0.0148),
    (0.0308, 0.0138),
    (0.0316, 0.0122),
    (0.032, 0.0104),
];

const BABIES: [&str; 2] = ["baby_basket", "baby_chair"];

/// What is pencilled under the window, when anything is
const NOTES: [&str; 5] = ["Baby Anna, 3 mo.", "Little Sam", "Baby Grace", "Our Lily, 1909", "Noah, newborn"];
