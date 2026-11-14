//! `place_still_lifes`
use crate::shared::add_parts;
use crate::shared::hang;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed still life: 3% chance per level.
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

/// The still life's canvas, frame, slip and wire (`domestic/pictures_on_the_walls.py`: SL_*, `still_wire`)
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
