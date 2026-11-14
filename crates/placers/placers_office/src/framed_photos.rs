//! `place_framed_photos`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Small framed photo (personal): 4% chance per level.
pub(crate) fn place_framed_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    // the old frame's colour (one draw): walnut, black, brass, aluminium
    let frame_colors = [0x604020, 0x202020, 0xd0b070, 0x606060];
    let colour: u32 = choice(&frame_colors);
    let frame = group();
    let mount_y = 1.60;
    // Slight tilt (photos never perfectly straight): about its nail
    let tilt = (rnd() - 0.5) * 0.06;
    wall_mount(g, &frame, x, z, dx, dz, 0.006);
    let p = frame.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let metal = colour == 0xd0b070 || colour == 0x606060;
    let outer = if metal { METAL_OUTER } else { WOOD_OUTER };
    // the nail under the sawtooth, 14 mm below the frame's top; the wall's
    // face 6 mm behind the spot
    let nail = group();
    nail.position().set(0, mount_y + outer.1 / 2.0 - 0.014, -0.006);
    nail.rotation().z = tilt;
    frame.add(&nail);
    let body = group();
    body.position().y = -(outer.1 / 2.0 - 0.014);
    nail.add(&body);
    let frame_mat = match colour {
        0x604020 => {
            lambert!({ map: &pictures::moulding_tex(outer, &WOOD_FACE, Outline::Rect, Finish::Wood([0x5c, 0x3e, 0x28]), 1500.0, vary) })
        }
        0x202020 => {
            phong!({ map: &pictures::moulding_tex(outer, &WOOD_FACE, Outline::Rect, Finish::Ebony, 1500.0, vary), specular: 0x161412, shininess: 30 })
        }
        0xd0b070 => phong!({ color: 0x6a5834, specular: 0x5a4a2c, shininess: 48 }),
        _ => phong!({ color: 0x6e6e6c, specular: 0x6a6a68, shininess: 56 }),
    };
    let dirt = 0.22 + 0.25 * (vary % 97) as f32 / 97.0;
    let parts: [(&str, Mat); 4] = [
        (if metal { "Real__framedPhotoFrameMetal" } else { "Real__framedPhotoFrameWood" }, frame_mat),
        ("Real__framedPhotoGlass", pictures::glass_mat(vary, dirt)),
        ("Real__framedPhotoBack", lambert!({ color: 0x4a3a2a })),
        ("Real__framedPhotoSawtooth", phong!({ color: 0x5c5c5a, specular: 0x404040, shininess: 40 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            if name.ends_with("Sawtooth") {
                m.position().y = outer.1 / 2.0 - 0.014;
            }
            if name.ends_with("Glass") {
                m.set_render_order(2);
            }
            body.add(&m);
        }
    }
    // the print under the glass: which snapshot, how it has aged
    let name = PHOTOS[(vary % PHOTOS.len() as u32) as usize];
    let mut age = Age::photo(vary);
    age.border = if (vary >> 5) % 3 == 0 { 0.0035 } else { 0.0 };
    let print = mesh(plane_geo!(PRINT.0, PRINT.1), lambert!({ map: &pictures::print_tex(name, PRINT, 2400.0, age, vary) }));
    print.position().z = ZL - 0.0023;
    body.add(&print);
    g.world.add(&frame);
}

/// The print (5 x 7 in), the lip's underside (the glass under it) and the
/// two frames' outsides (`t01.py`: PH_*)
const PRINT: (f64, f64) = (0.127, 0.178);

const ZL: f64 = 0.0060;

const WOOD_OUTER: (f64, f64) = (0.151, 0.202);

const METAL_OUTER: (f64, f64) = (0.132, 0.183);

/// The wooden moulding's face, outer edge to sight (d, z), as `framed_photo_wood`
static WOOD_FACE: [(f64, f64); 9] = [
    (0.0, 0.0130),
    (0.0006, 0.0152),
    (0.0020, 0.0168),
    (0.0040, 0.0176),
    (0.0075, 0.0178),
    (0.0105, 0.0172),
    (0.0122, 0.0160),
    (0.0158, 0.0100),
    (0.016, 0.0094),
];

/// The snapshots a desk keeps (portrait prints)
const PHOTOS: [&str; 3] = ["mother_children", "couple_reading", "garden_family"];
