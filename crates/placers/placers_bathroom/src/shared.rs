//! What several bathroom placers share.
use game::prelude::*;

/// A placed thing's own number, from where it stands (no draw): its
/// painters start their strokes from it, so no two floors share them
pub(crate) fn salt(px: f64, pz: f64) -> u32 {
    ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32
}

/// Wall-mount candidates `{ x, z, dx, dz }`: one per (open tile, side facing a
/// wall or the grid edge), in scan order.
pub(crate) fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push((x, z, dx, dz));
                }
            }
        }
    }
    v
}
