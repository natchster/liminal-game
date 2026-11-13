//! Prop placers: office-corridor wall fixtures (fire alarms and extinguishers,
//! coat-hook rows, missing-photo hooks, water fountains, junction boxes,
//! bulletin boards, motion sensors, exit signs, thermostats, radiators, rusted
//! pipes, a wall clock, wall holes, posters) and floor / ceiling props
//! (clipboards, boxes, coffee cups, trash cans, ceiling stains, an overturned
//! chair, cigarette butts).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod floor;
mod textures;
mod wall;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_clipboards", floor::place_clipboards));
    r.push(("place_fire_alarms", wall::place_fire_alarms));
    r.push(("place_cardboard_boxes", floor::place_cardboard_boxes));
    r.push(("place_coat_hook_rows", wall::place_coat_hook_rows));
    r.push(("place_missing_photo_hooks", wall::place_missing_photo_hooks));
    r.push(("place_coffee_cups", floor::place_coffee_cups));
    r.push(("place_water_fountains", wall::place_water_fountains));
    r.push(("place_junction_boxes", wall::place_junction_boxes));
    r.push(("place_bulletin_boards", wall::place_bulletin_boards));
    r.push(("place_trash_cans", floor::place_trash_cans));
    r.push(("place_motion_sensors", floor::place_motion_sensors));
    r.push(("place_exit_signs", wall::place_exit_signs));
    r.push(("place_thermostats", wall::place_thermostats));
    r.push(("place_radiators", wall::place_radiators));
    r.push(("place_fire_extinguishers", wall::place_fire_extinguishers));
    r.push(("place_rusted_pipes", wall::place_rusted_pipes));
    r.push(("place_ceiling_stains", floor::place_ceiling_stains));
    r.push(("place_wall_clock", wall::place_wall_clock));
    r.push(("place_cracked_wall_holes", wall::place_cracked_wall_holes));
    r.push(("place_overturned_chair", floor::place_overturned_chair));
    r.push(("place_cigarette_butts", floor::place_cigarette_butts));
    r.push(("place_weathered_posters", wall::place_weathered_posters));
}

// ------------------------------------------------------------------ shared helpers

/// A wall facing: open tile (x, z) with a wall (or the grid edge) at (x+dx, z+dz).
#[derive(Clone, Copy)]
pub(crate) struct Cand {
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
}

/// The wall-facing candidate scan every wall placer repeats:
/// open tiles × 4 directions where `!in_grid(nx, nz) || tiles[t_idx(nx, nz)] == 1`.
pub(crate) fn wall_candidates(g: &Game) -> Vec<Cand> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) {
                    candidates.push(Cand { x, z, dx, dz });
                }
            }
        }
    }
    candidates
}

pub(crate) use placers_kit::place::open_tiles;

/// In-place Fisher-Yates: for i from n-1 down to 1, swap with j = floor(rnd() * (i+1)).
pub(crate) fn shuffle<T>(v: &mut [T]) {
    if v.len() < 2 {
        return;
    }
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

pub(crate) use placers_kit::place::pick;

/// Trash bag colour per weather (else the normal entry)
pub(crate) fn trash_bag_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0f14, 0x281a0a, 0x1a0808, 0x2a2418, 0x0a0a10, 0x1a1a1e)
}
/// Poster tint per weather (else the normal entry)
pub(crate) fn poster_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa0b0c0, 0xe0c890, 0xb08080, 0xc0b090, 0xd8ccb0, 0xffffff)
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
