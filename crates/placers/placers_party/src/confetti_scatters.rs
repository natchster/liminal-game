//! `place_confetti_scatters`: Confetti scatter: 10% chance per level. 10-15 tiny colored planes scattered on floor in ~0.35 m radius circle.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Confetti scatter: 10% chance per level. 10-15 tiny colored
/// planes scattered on floor in ~0.35 m radius circle.
pub(crate) fn place_confetti_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let conf_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0, 0xa020a0, 0xf0a0a0];
    let cx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let cz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let count = 10 + (rnd() * 6.0).floor() as i32;
    for _ in 0..count {
        let c_col = pick(&conf_colors);
        let conf_mat = basic_mat!({ color: c_col, transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
        let conf = mesh(plane_geo!(0.008, 0.014), &conf_mat);
        conf.rotation().x = -PI / 2.0;
        conf.rotation().z = rnd() * PI * 2.0;
        let c_ang = rnd() * PI * 2.0;
        let c_rad = rnd() * 0.35;
        conf.position().set(cx + c_ang.dcos() * c_rad, 0.003, cz + c_ang.dsin() * c_rad);
        g.world.add(&conf);
    }
}
