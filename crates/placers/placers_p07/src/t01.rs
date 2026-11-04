//! The small paintings of a parlour (Blender batch `t01`), each a real one
//! (CC0, `visuals::pictures`), hung on its wire from a brass picture hook,
//! leaning out from the wall at the top, a little askew:
//!
//! - the landscape: a grey Hague School oil sketch on a mahogany panel
//!   (French "paysage" No. 3, 27 x 19 cm) in a gilt cove frame;
//! - the still life: fruit in oil on a "figure" No. 3 canvas (27 x 22 cm)
//!   in a dark walnut frame with a gilt slip.
//!
//! Both aged as they would be: the varnish gone amber, cracked, grimed.
use crate::{face_wall, pick, wall_tiles};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// The landscape's panel, its frame's outside and the wire's apex above its
/// middle (`t01.py`: LP_*, `land_wire`)
const LP_PANEL: (f64, f64) = (0.270, 0.190);
const LP_SIZE: (f64, f64) = (0.320, 0.240);
const LP_APEX: f64 = 0.075;
/// The gilt frame's face, outer edge to sight (d, z), as `land_frame`
static LP_FACE: [(f64, f64); 19] = [
    (0.0, 0.0200),
    (0.0008, 0.0225),
    (0.0022, 0.0250),
    (0.0040, 0.0262),
    (0.0058, 0.0258),
    (0.0068, 0.0247),
    (0.0078, 0.0268),
    (0.0095, 0.0290),
    (0.0115, 0.0300),
    (0.0135, 0.0296),
    (0.0160, 0.0278),
    (0.0185, 0.0252),
    (0.0210, 0.0226),
    (0.0232, 0.0206),
    (0.0250, 0.0196),
    (0.0262, 0.0198),
    (0.0274, 0.0192),
    (0.0283, 0.0178),
    (0.029, 0.0160),
];
/// The still life's canvas, frame, slip and wire (`t01.py`: SL_*, `still_wire`)
const SL_CANVAS: (f64, f64) = (0.270, 0.220);
const SL_SIZE: (f64, f64) = (0.340, 0.290);
const SL_SLIP: (f64, f64) = (0.279, 0.229);
const SL_APEX: f64 = 0.085;
/// The walnut frame's face and the slip's, as `still_frame` and `still_slip`
static SL_FACE: [(f64, f64); 17] = [
    (0.0, 0.0210),
    (0.0012, 0.0238),
    (0.0030, 0.0262),
    (0.0055, 0.0282),
    (0.0085, 0.0295),
    (0.0115, 0.0300),
    (0.0145, 0.0297),
    (0.0170, 0.0284),
    (0.0190, 0.0272),
    (0.0210, 0.0268),
    (0.0230, 0.0272),
    (0.0252, 0.0270),
    (0.0275, 0.0258),
    (0.0298, 0.0238),
    (0.0318, 0.0212),
    (0.0332, 0.0186),
    (0.034, 0.0168),
];
static SL_SLIP_FACE: [(f64, f64); 9] = [
    (0.0, 0.0149),
    (0.0045, 0.0149),
    (0.0062, 0.0141),
    (0.0080, 0.0121),
    (0.0096, 0.0105),
    (0.0108, 0.0099),
    (0.0118, 0.0091),
    (0.0125, 0.0081),
    (0.013, 0.0069),
];

/// A painting's hanging: the hook on the wall at the wire's apex (`apex`
/// above the picture's middle at `mount_y`, the wall `wall` behind the
/// spot), the frame swinging about it by `tilt` and leaning out `lean` at
/// its top from its foot (`h` its height); returns the body the parts go in
fn hang(o: &Obj, mount_y: f64, wall: f64, apex: f64, h: f64, lean: f64, tilt: f64) -> Obj {
    let nail = group();
    nail.position().set(0, mount_y + apex, -wall);
    nail.rotation().z = tilt;
    o.add(&nail);
    if let Some(geo) = prop_part_as_modelled("Real__oldPortraitHook") {
        nail.add(&mesh(geo, phong!({ color: 0x6a5630, specular: 0x5a4a2a, shininess: 36 })));
    }
    let foot = group();
    foot.position().y = -apex - h / 2.0;
    foot.rotation().x = (lean / h).atan();
    nail.add(&foot);
    let body = group();
    body.position().y = h / 2.0;
    foot.add(&body);
    body
}

fn add_parts(body: &Obj, parts: &[(&str, Mat)]) {
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, mat));
        }
    }
}

// R671 framed landscape: 4% chance per level.
pub(crate) fn place_abstract_landscapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let painting = group();
    let mount_y = 1.65;
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.04;
    painting.rotation().y = face_wall(dx, dz);
    painting.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = painting.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = hang(&painting, mount_y, 0.013, LP_APEX, LP_SIZE.1, 0.006, tilt);
    let mut age = Age::oil(vary);
    // a panel of the late 19th century: no weave, hardly a crack
    age.weave = 0.0;
    age.crack = (age.crack.0 * 0.8, age.crack.1 * 0.35);
    let pic = ["land_pond", "land_road"][(vary % 2) as usize];
    add_parts(
        &body,
        &[
            (
                "Real__landscapePaintingFrame",
                phong!({ map: &pictures::moulding_tex(LP_SIZE, &LP_FACE, Outline::Rect, Finish::Gilt, 1100.0, vary), specular: 0x3a3020, shininess: 22 }),
            ),
            ("Real__landscapePaintingPanel", lambert!({ map: &pictures::print_tex(pic, LP_PANEL, 1300.0, age, vary) })),
            ("Real__landscapePaintingWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&painting);
}

// R673 framed still life: 3% chance per level.
pub(crate) fn place_still_lifes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let still = group();
    let mount_y = 1.65;
    // askew on its hook (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.04;
    still.rotation().y = face_wall(dx, dz);
    still.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.017), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.017));
    let p = still.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = hang(&still, mount_y, 0.017, SL_APEX, SL_SIZE.1, 0.010, tilt);
    let pic = ["still_peaches", "still_dark"][(vary % 2) as usize];
    add_parts(
        &body,
        &[
            (
                "Real__stillLifeFrame",
                lambert!({ map: &pictures::moulding_tex(SL_SIZE, &SL_FACE, Outline::Rect, Finish::Wood([0x3e, 0x28, 0x1a]), 1100.0, vary) }),
            ),
            (
                "Real__stillLifeSlip",
                phong!({ map: &pictures::moulding_tex(SL_SLIP, &SL_SLIP_FACE, Outline::Rect, Finish::Gilt, 1300.0, vary ^ 0x5a), specular: 0x3a3020, shininess: 22 }),
            ),
            ("Real__stillLifeCanvas", lambert!({ map: &pictures::print_tex(pic, SL_CANVAS, 1300.0, Age::oil(vary), vary) })),
            ("Real__stillLifeStretcher", lambert!({ color: 0x86705a })),
            ("Real__stillLifeWire", phong!({ color: 0x56544e, specular: 0x50504c, shininess: 40 })),
        ],
    );
    g.world.add(&still);
}
