//! Prop placers ported from backrooms.js (chunk p23; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod halloween;
mod mystic;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeReliquaryBoxes", mystic::place_reliquary_boxes));
    r.push(("placeStainedGlassWindows", mystic::place_stained_glass_windows));
    r.push(("placeCrystalSkulls", mystic::place_crystal_skulls));
    r.push(("placeAncientPortraits", mystic::place_ancient_portraits));
    r.push(("placeWallCrucifixes", mystic::place_wall_crucifixes));
    r.push(("placeSmallOwls", mystic::place_small_owls));
    r.push(("placeBrassCandlesticks", mystic::place_brass_candlesticks));
    r.push(("placeSkullStacks", mystic::place_skull_stacks));
    r.push(("placeSmallSkulls", mystic::place_small_skulls));
    r.push(("placeSmallPotions", mystic::place_small_potions));
    r.push(("placeCandyCornScatters", mystic::place_candy_corn_scatters));
    r.push(("placeSmallTreatPails", mystic::place_small_treat_pails));
    r.push(("placeSmallGhostSheets", halloween::place_small_ghost_sheets));
    r.push(("placeSmallBlackCats", halloween::place_small_black_cats));
    r.push(("placeSmallSpiders", halloween::place_small_spiders));
    r.push(("placeSmallBrooms", halloween::place_small_brooms));
    r.push(("placeSmallTombstones", halloween::place_small_tombstones));
    r.push(("placeHangingBats", halloween::place_hanging_bats));
    r.push(("placeSmallSkeletonHands", halloween::place_small_skeleton_hands));
    r.push(("placeSmallCauldrons", halloween::place_small_cauldrons));
    r.push(("placeSmallPumpkins", halloween::place_small_pumpkins));
    r.push(("placeSmallWitchHats", halloween::place_small_witch_hats));
    r.push(("placeSmallCrystalBalls", halloween::place_small_crystal_balls));
    r.push(("placeSmallToySwords", halloween::place_small_toy_swords));
    r.push(("placeSmallToyPianos", halloween::place_small_toy_pianos));
}

// ------------------------------------------------------------------ shared local helpers

/// The `openTiles` scan every placer here repeats: open tiles of the interior,
/// z-major then x (no random draws).
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// `openTiles[Math.floor(Math.random() * openTiles.length)]`
pub(crate) fn pick_tile(t: &[(i32, i32)]) -> (i32, i32) { t[(rnd() * t.len() as f64).floor() as usize] }

/// A wall-mount candidate `{ x, z, dx, dz }`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WallCand { pub x: i32, pub z: i32, pub dx: i32, pub dz: i32 }

/// The wall-candidate scan: open tile + each side whose neighbour is out of
/// grid or a wall (`!inGrid(nx, nz) || tiles[tIdx(nx, nz)] === 1`).
pub(crate) fn wall_cands(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// `arr[Math.floor(Math.random() * arr.length)]` for colour tables.
pub(crate) fn pick_u32(a: &[u32]) -> u32 { a[(rnd() * a.len() as f64).floor() as usize] }

/// One-triangle BufferGeometry: `setAttribute('position', Float32Array(v))`,
/// `setIndex([0, 1, 2])`, `computeVertexNormals()`.
pub(crate) fn tri_geo(v: [f64; 9]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(vec![0, 1, 2]);
    geo.compute_vertex_normals();
    geo
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
