//! The Victorian family portrait (Blender batch `t01`): a landscape cabinet
//! photograph of a family (1887-1915, CC0, `visuals::pictures`) in a walnut
//! ogee frame with a gilt slip, its glass cracked from a knock long ago,
//! hung from a sawtooth on a nail, a little askew.
use crate::{mount_on_wall, wall_candidates};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// The frame's outside, the lip's underside (the glass beneath it), the
/// slip's outside and the glass (`t01.py`: FP_*)
const SIZE: (f64, f64) = (0.220, 0.166);
const ZL: f64 = 0.0110;
const SLIP: (f64, f64) = (0.175, 0.121);
/// The walnut moulding's face and the slip's, outer edge to sight (d, z),
/// as `family_frame` and `family_slip`
static FACE: [(f64, f64); 13] = [
    (0.0, 0.0160),
    (0.0010, 0.0195),
    (0.0030, 0.0225),
    (0.0060, 0.0240),
    (0.0095, 0.0238),
    (0.0125, 0.0225),
    (0.0150, 0.0200),
    (0.0165, 0.0180),
    (0.0185, 0.0172),
    (0.0210, 0.0168),
    (0.0235, 0.0155),
    (0.0250, 0.0138),
    (0.026, 0.0122),
];
static SLIP_FACE: [(f64, f64); 9] = [
    (0.0, 0.0089),
    (0.0045, 0.0089),
    (0.0062, 0.0082),
    (0.0080, 0.0068),
    (0.0094, 0.0058),
    (0.0104, 0.0054),
    (0.0112, 0.0050),
    (0.0118, 0.0043),
    (0.012, 0.0036),
];
const FAMILIES: [&str; 3] = ["family_parlour", "family_room", "family_door"];

// R511 family portrait: 5% chance per level. A family group in its frame.
pub(crate) fn place_family_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let portrait = group();
    // the nail under the sawtooth, 16 mm below the top; the wall 10 mm behind the spot
    let nail = group();
    nail.position().set(0, SIZE.1 / 2.0 - 0.016, -0.010);
    portrait.add(&nail);
    let body = group();
    body.position().y = -(SIZE.1 / 2.0 - 0.016);
    nail.add(&body);
    mount_on_wall(g, &portrait, &c, 0.010, 1.60, 0.6);
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.05;
    nail.rotation().z = tilt;
    let p = portrait.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let dirt = 0.3 + 0.3 * (vary % 79) as f32 / 79.0;
    let parts: [(&str, Mat); 5] = [
        (
            "Real__familyPortraitFrame",
            lambert!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, Finish::Wood([0x4c, 0x32, 0x20]), 1300.0, vary) }),
        ),
        (
            "Real__familyPortraitSlip",
            phong!({ map: &pictures::moulding_tex(SLIP, &SLIP_FACE, Outline::Rect, Finish::Gilt, 1500.0, vary ^ 0x51), specular: 0x3a3020, shininess: 24 }),
        ),
        ("Real__familyPortraitGlass", pictures::glass_mat(vary, dirt)),
        ("Real__familyPortraitBack", lambert!({ color: 0x4a3a2a })),
        ("Real__framedPhotoSawtooth", phong!({ color: 0x5c5c5a, specular: 0x404040, shininess: 40 })),
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
    // the card behind the slip
    let name = FAMILIES[(vary % FAMILIES.len() as u32) as usize];
    let size = (0.160, 0.106);
    let print = mesh(plane_geo!(size.0, size.1), lambert!({ map: &pictures::print_tex(name, size, 2400.0, Age::photo(vary), vary) }));
    print.position().z = 0.0024;
    body.add(&print);
    // the glass cracked from a knock at one corner
    let knock = [(0.12, 0.2), (0.85, 0.25), (0.2, 0.8), (0.82, 0.78)][(vary / 7 % 4) as usize];
    let cracks = pictures::cracked_glass_tex((SLIP.0, SLIP.1), knock, vary);
    let crack_mat = lambert!({
        map: &cracks, transparent: true, depthWrite: false,
        polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
    });
    let crack = mesh(plane_geo!(SLIP.0, SLIP.1), &crack_mat);
    crack.position().z = ZL - 0.0001;
    crack.set_render_order(3);
    body.add(&crack);
    g.world.add(&portrait);
}
