//! `place_small_cupcake_wrappers`: Small cupcake wrapper: 10% chance per level. Empty pleated paper wrapper ("who ate the cupcake?").
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small cupcake wrapper: 10% chance per level. Empty pleated
/// paper wrapper ("who ate the cupcake?").
pub(crate) fn place_small_cupcake_wrappers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wrap_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let wrap_mat = lambert!({ color: pick(&wrap_colors), side: DoubleSide });
    let wrap = group();
    // Pleated body: 12-sided cylinder gives visible pleats.
    let body = mesh(cyl_geo!(0.030, 0.022, 0.028, 12, 1, true), &wrap_mat);
    body.position().y = 0.014;
    wrap.add(&body);
    // Bottom.
    let bottom = mesh(circle_geo!(0.022, 12), &wrap_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    wrap.add(&bottom);
    // 40% flattened / crushed variant.
    if rnd() < 0.40 {
        wrap.scale().y = 0.35;
    }
    wrap.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    wrap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wrap);
}
