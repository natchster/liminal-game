//! `place_toy_drums`: Small toy drum: 8% chance per level. Cylinder body + top + bottom membrane discs + loose strap.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small toy drum: 8% chance per level. Cylinder body + top +
/// bottom membrane discs + loose strap.
pub(crate) fn place_toy_drums(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let mem_mat = lambert!({ color: 0xd8ccb0 });
    let strap_mat = lambert!({ color: 0x4a3018 });
    let drum = group();
    // Body: vertical cylinder.
    let body = mesh(pg("ToyDrums__body", cyl_geo!(0.06, 0.06, 0.09, 12)), &body_mat);
    body.position().y = 0.045;
    drum.add(&body);
    // Top membrane disc.
    let top_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    top_m.rotation().x = -PI / 2.0;
    top_m.position().y = 0.091;
    drum.add(&top_m);
    // Bottom membrane disc.
    let bot_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    bot_m.rotation().x = PI / 2.0;
    bot_m.position().y = -0.001;
    drum.add(&bot_m);
    // Loose carry strap lying face-up on the floor, one end tucked under the
    // drum's rim at +X, trailing off 0.3 rad from the tangent.
    let strap = mesh(plane_geo!(0.015, 0.20), &strap_mat);
    strap.rotation().x = -PI / 2.0;
    strap.position().set(0.06 + 0.3f64.dsin() * 0.095, 0.0015, 0.3f64.dcos() * 0.095);
    strap.rotation().z = 0.3;
    drum.add(&strap);
    drum.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    drum.rotation().y = rnd() * PI * 2.0;
    g.world.add(&drum);
}
