//! `place_marble_scatters`: Marble scatter: 10% chance per level. 4-7 tiny colored spheres scattered on floor — "spilled marbles".
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Marble scatter: 10% chance per level. 4-7 tiny colored
/// spheres scattered on floor — "spilled marbles".
pub(crate) fn place_marble_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let marble_colors = [0x2050d8, 0xd82020, 0xd8b820, 0x30a030, 0xa0a0a0, 0x8020d8];
    let cx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let cz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let count = 4 + (rnd() * 4.0).floor() as i32;
    for _ in 0..count {
        let m_col = pick(&marble_colors);
        let marble_mat = lambert!({ color: m_col });
        let marble = mesh(pg("MarbleScatters__marble", sphere_geo!(0.010, 6, 4)), &marble_mat);
        let m_ang = rnd() * PI * 2.0;
        let m_rad = rnd() * 0.30;
        marble.position().set(cx + m_ang.dcos() * m_rad, 0.010, cz + m_ang.dsin() * m_rad);
        g.world.add(&marble);
    }
}
