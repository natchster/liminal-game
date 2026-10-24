//! Prop placers ported from backrooms.js (chunk p31; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod floor;
mod textures;
mod wall;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeClipboards", floor::place_clipboards));
    r.push(("placeFireAlarms", wall::place_fire_alarms));
    r.push(("placeCardboardBoxes", floor::place_cardboard_boxes));
    r.push(("placeCoatHookRows", wall::place_coat_hook_rows));
    r.push(("placeMissingPhotoHooks", wall::place_missing_photo_hooks));
    r.push(("placeCoffeeCups", floor::place_coffee_cups));
    r.push(("placeWaterFountains", wall::place_water_fountains));
    r.push(("placeJunctionBoxes", wall::place_junction_boxes));
    r.push(("placeBulletinBoards", wall::place_bulletin_boards));
    r.push(("placeTrashCans", floor::place_trash_cans));
    r.push(("placeMotionSensors", floor::place_motion_sensors));
    r.push(("placeExitSigns", wall::place_exit_signs));
    r.push(("placeThermostats", wall::place_thermostats));
    r.push(("placeRadiators", wall::place_radiators));
    r.push(("placeFireExtinguishers", wall::place_fire_extinguishers));
    r.push(("placeRustedPipes", wall::place_rusted_pipes));
    r.push(("placeCeilingStains", floor::place_ceiling_stains));
    r.push(("placeWallClock", wall::place_wall_clock));
    r.push(("placeCrackedWallHoles", wall::place_cracked_wall_holes));
    r.push(("placeOverturnedChair", floor::place_overturned_chair));
    r.push(("placeCigaretteButts", floor::place_cigarette_butts));
    r.push(("placeWeatheredPosters", wall::place_weathered_posters));
}

// ------------------------------------------------------------------ shared helpers

/// A wall facing: open tile (x, z) with a wall (or the grid edge) at (x+dx, z+dz).
#[derive(Clone, Copy)]
pub(crate) struct Cand { pub x: i32, pub z: i32, pub dx: i32, pub dz: i32 }

/// The wall-facing candidate scan every wall placer repeats:
/// open tiles × 4 directions where `!inGrid(nx, nz) || tiles[tIdx(nx, nz)] === 1`.
pub(crate) fn wall_candidates(g: &Game) -> Vec<Cand> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) { candidates.push(Cand { x, z, dx, dz }); }
            }
        }
    }
    candidates
}

/// Open interior tiles in row-major order (`tiles[tIdx(x, z)] === 0`).
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// In-place Fisher-Yates: `for (i = n-1; i > 0; i--) j = floor(random() * (i+1))`.
pub(crate) fn shuffle<T>(v: &mut [T]) {
    if v.len() < 2 { return; }
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// `arr[Math.floor(Math.random() * arr.length)]` for a slice of Copy values.
pub(crate) fn pick<T: Copy>(a: &[T]) -> T { a[(rnd() * a.len() as f64).floor() as usize] }

/// `TRASH_BAG_COLOR_BY_WEATHER[key] ?? .normal` (backrooms.js line 5787)
pub(crate) fn trash_bag_color_by_weather(key: &str) -> u32 { by_weather(key, 0x0a0f14, 0x281a0a, 0x1a0808, 0x2a2418, 0x0a0a10, 0x1a1a1e) }
/// `POSTER_TINT_BY_WEATHER[key] ?? .normal` (backrooms.js line 5647)
pub(crate) fn poster_tint_by_weather(key: &str) -> u32 { by_weather(key, 0xa0b0c0, 0xe0c890, 0xb08080, 0xc0b090, 0xd8ccb0, 0xffffff) }

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
