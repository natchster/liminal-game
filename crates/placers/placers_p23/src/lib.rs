//! Prop placers: mystic props (reliquaries, stained glass, crystal skulls,
//! portraits, crucifixes, owls, candlesticks, skulls, potions) and Halloween
//! props (candy corn, treat pails, ghosts, cats, spiders, brooms, tombstones,
//! bats, skeleton hands, cauldrons, pumpkins, witch hats, crystal balls), plus
//! toy swords and toy pianos.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod halloween;
mod mystic;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_reliquary_boxes", mystic::place_reliquary_boxes));
    r.push(("place_stained_glass_windows", mystic::place_stained_glass_windows));
    r.push(("place_crystal_skulls", mystic::place_crystal_skulls));
    r.push(("place_ancient_portraits", mystic::place_ancient_portraits));
    r.push(("place_wall_crucifixes", mystic::place_wall_crucifixes));
    r.push(("place_small_owls", mystic::place_small_owls));
    r.push(("place_brass_candlesticks", mystic::place_brass_candlesticks));
    r.push(("place_skull_stacks", mystic::place_skull_stacks));
    r.push(("place_small_skulls", mystic::place_small_skulls));
    r.push(("place_small_potions", mystic::place_small_potions));
    r.push(("place_candy_corn_scatters", mystic::place_candy_corn_scatters));
    r.push(("place_small_treat_pails", mystic::place_small_treat_pails));
    r.push(("place_small_ghost_sheets", halloween::place_small_ghost_sheets));
    r.push(("place_small_black_cats", halloween::place_small_black_cats));
    r.push(("place_small_spiders", halloween::place_small_spiders));
    r.push(("place_small_brooms", halloween::place_small_brooms));
    r.push(("place_small_tombstones", halloween::place_small_tombstones));
    r.push(("place_hanging_bats", halloween::place_hanging_bats));
    r.push(("place_small_skeleton_hands", halloween::place_small_skeleton_hands));
    r.push(("place_small_cauldrons", halloween::place_small_cauldrons));
    r.push(("place_small_pumpkins", halloween::place_small_pumpkins));
    r.push(("place_small_witch_hats", halloween::place_small_witch_hats));
    r.push(("place_small_crystal_balls", halloween::place_small_crystal_balls));
    r.push(("place_small_toy_swords", halloween::place_small_toy_swords));
    r.push(("place_small_toy_pianos", halloween::place_small_toy_pianos));
}

// ------------------------------------------------------------------ shared local helpers

pub(crate) use placers_kit::place::open_tiles;

/// A uniformly random tile (one `rnd()` draw).
pub(crate) fn pick_tile(t: &[(i32, i32)]) -> (i32, i32) {
    t[(rnd() * t.len() as f64).floor() as usize]
}

/// A wall-mount candidate `{ x, z, dx, dz }`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WallCand {
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
}

/// The wall-candidate scan: open tile + each side whose neighbour is out of
/// grid or a wall (`!in_grid(nx, nz) || tiles[t_idx(nx, nz)] == 1`).
pub(crate) fn wall_cands(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
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

pub(crate) use placers_kit::place::pick_hex as pick_u32;

pub(crate) use placers_kit::geometry::tri_geo;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
