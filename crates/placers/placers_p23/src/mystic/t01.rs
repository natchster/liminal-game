//! The old portrait (Blender batch `t01`): a 17th-century Dutch portrait in
//! oil (an anonymous sitter, CC0, `visuals::pictures`) on a "figure" No. 6
//! canvas (33 x 41 cm), its varnish gone amber, cracked into islands,
//! grimed, in an ebonised ogee frame rubbed brown on its crests; hung on its
//! wire from a brass picture hook, leaning out from the wall at the top as
//! a wire-hung picture does, a little askew.
use super::hang_on_wall;
use crate::wall_cands;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// The canvas, the frame's outside and the wire's apex above the picture's
/// middle (`t01.py`: OP_*, `old_wire`)
const CANVAS: (f64, f64) = (0.330, 0.410);
const SIZE: (f64, f64) = (0.412, 0.492);
const APEX: f64 = 0.156;
/// How far the frame's top stands off the wall, its wire pulling it
const LEAN: f64 = 0.012;
/// The moulding's face, outer edge to sight (d, z), as `old_frame`
static FACE: [(f64, f64); 22] = [
    (0.0, 0.0240),
    (0.0012, 0.0262),
    (0.0035, 0.0270),
    (0.0045, 0.0285),
    (0.0060, 0.0305),
    (0.0085, 0.0328),
    (0.0115, 0.0347),
    (0.0150, 0.0358),
    (0.0185, 0.0360),
    (0.0215, 0.0356),
    (0.0245, 0.0343),
    (0.0270, 0.0325),
    (0.0295, 0.0312),
    (0.0320, 0.0306),
    (0.0360, 0.0305),
    (0.0370, 0.0312),
    (0.0385, 0.0316),
    (0.0400, 0.0312),
    (0.0412, 0.0300),
    (0.0425, 0.0286),
    (0.0438, 0.0270),
    (0.045, 0.0258),
];
const SITTERS: [&str; 3] = ["oil_man", "oil_woman2", "oil_woman"];

// R490 ancient portrait: 4% chance per level. An old portrait in oil.
pub fn place_ancient_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_cands(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let portrait = group();
    // the hook on the wall (14 mm behind the spot) at the wire's apex; the
    // frame swings about it, and leans out from its foot
    let nail = group();
    nail.position().set(0, APEX, -0.014);
    portrait.add(&nail);
    let foot = group();
    foot.position().y = -APEX - SIZE.1 / 2.0;
    foot.rotation().x = (LEAN / SIZE.1).atan();
    nail.add(&foot);
    let body = group();
    body.position().y = SIZE.1 / 2.0;
    foot.add(&body);
    // Wall placement.
    hang_on_wall(g, &portrait, &c, 0.014, 1.65);
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.05;
    nail.rotation().z = tilt;
    let p = portrait.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let finish = if (vary >> 7) % 3 == 0 { Finish::Wood([0x3c, 0x29, 0x1c]) } else { Finish::Ebony };
    let parts: [(&str, Mat); 4] = [
        (
            "Real__oldPortraitFrame",
            phong!({ map: &pictures::moulding_tex(SIZE, &FACE, Outline::Rect, finish, 1000.0, vary), specular: 0x141210, shininess: 18 }),
        ),
        (
            "Real__oldPortraitCanvas",
            lambert!({ map: &pictures::print_tex(SITTERS[(vary % 3) as usize], CANVAS, 1000.0, Age::oil(vary), vary) }),
        ),
        ("Real__oldPortraitStretcher", lambert!({ color: 0x86705a })),
        ("Real__oldPortraitWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, mat));
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__oldPortraitHook") {
        nail.add(&mesh(geo, phong!({ color: 0x6a5630, specular: 0x5a4a2a, shininess: 36 })));
    }
    g.world.add(&portrait);
}
