//! What several victorian placers share.
use game::prelude::*;

/// `a.clone().sub(b)`
pub(crate) fn vsub(a: Vec3, b: Vec3) -> Vec3 {
    let mut v = a;
    v.sub(&b);
    v
}

/// Open tiles next to a wall (or the grid edge), one entry per facing wall:
/// `(x, z, dx, dz)` for each side where `!in_grid(nx, nz) || tiles[..] == 1`.
pub(crate) fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    candidates.push((x, z, dx, dz));
                }
            }
        }
    }
    candidates
}
