//! `place_abstract_landscapes`
use crate::shared::add_parts;
use crate::shared::hang;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed landscape: 4% chance per level.
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

/// The landscape's panel, its frame's outside and the wire's apex above its
/// middle (`domestic/pictures_on_the_walls.py`: LP_*, `land_wire`)
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
