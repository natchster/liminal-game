//! `place_wall_doors`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// A door to nowhere (`LORE.md`, *Places*; the asset pipeline's interior door,
// `assets/src/props/interior_door/`, `core_props::doors`): on one floor in
// about twenty-five, an ordinary door shut in its frame against a wall, its
// knob on it. Its draws as they were (the chance, the wall, its paint);
// held to its rules after them (`place_by_layout`: the purposes that had
// rooms with doors, never the Lobby), and never on a floor with rules of
// its own.
pub(crate) fn place_wall_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let paint = choice(&[0usize, 1, 2, 3]);
    let (wall_x, wall_z) = wall_xz(g, c, 0.0);
    // where the layout records it: half a metre in from its tile's edge
    // toward its wall, in the open tile still (its purpose is the tile's) and
    // within reach of the face it settles on (a protruding wall's too: what
    // keeps a haunch and a caged lamp off it measures from here, review D)
    let (at_x, at_z) = wall_xz(g, c, 0.5);
    if g.level_scp.is_some() || g.place_by_layout("interior_door", "wall", at_x, at_z).is_none() {
        return;
    }
    let door = game::core_props::doors::interior_door_in_frame(paint);
    mount(&door, c, wall_x, wall_z, 0.0, None);
    g.world.add(&door);
}
