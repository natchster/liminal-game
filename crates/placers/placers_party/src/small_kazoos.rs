//! `place_small_kazoos`: Small kazoo: 8% chance per level. Short cylinder with membrane disc. Child-music family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small kazoo: 8% chance per level. Short cylinder with
/// membrane disc. Child-music family.
pub(crate) fn place_small_kazoos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let kaz_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let body_mat = lambert!({ color: pick(&kaz_colors) });
    let membrane_mat = lambert!({ color: 0xe8dcc0 });
    let kazoo = group();
    // Body: horizontal tapered cylinder.
    let body = mesh(pg("SmallKazoos__body", cyl_geo!(0.014, 0.010, 0.09, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.045, 0.014, 0);
    kazoo.add(&body);
    // Membrane resonator: small cylinder rising from mid-top.
    let res_body = mesh(pg("SmallKazoos__resonator", cyl_geo!(0.010, 0.010, 0.012, 8)), &body_mat);
    res_body.position().set(0.045, 0.032, 0);
    kazoo.add(&res_body);
    // Membrane disc on top of resonator.
    let membrane = mesh(circle_geo!(0.008, 8), &membrane_mat);
    membrane.rotation().x = -PI / 2.0;
    membrane.position().set(0.045, 0.039, 0);
    kazoo.add(&membrane);
    kazoo.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    kazoo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kazoo);
}
