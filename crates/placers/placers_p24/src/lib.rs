//! Prop placers: small toys (wands, robots, microphones, dinosaurs, phones, piggy
//! banks, boats, planes, music boxes, skateboards, hula hoops, spinning tops),
//! magnifying glasses, magnetic letters, remotes, kids' drawings, games (jacks,
//! chalk and hopscotch, darts, pool cues and balls, dominoes, chess pieces,
//! poker chips, dice, cards) and hairbrushes.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod games;
mod toys;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_small_princess_wands", toys::place_small_princess_wands));
    r.push(("place_small_toy_robots", toys::place_small_toy_robots));
    r.push(("place_small_toy_microphones", toys::place_small_toy_microphones));
    r.push(("place_small_toy_dinos", toys::place_small_toy_dinos));
    r.push(("place_small_toy_phones", toys::place_small_toy_phones));
    r.push(("place_small_piggy_banks", toys::place_small_piggy_banks));
    r.push(("place_small_magnifying_glasses", toys::place_small_magnifying_glasses));
    r.push(("place_small_jacks", toys::place_small_jacks));
    r.push(("place_magnetic_letters", toys::place_magnetic_letters));
    r.push(("place_small_remote_controls", toys::place_small_remote_controls));
    r.push(("place_kids_drawings", toys::place_kids_drawings));
    r.push(("place_small_toy_boats", toys::place_small_toy_boats));
    r.push(("place_small_music_boxes", toys::place_small_music_boxes));
    r.push(("place_small_toy_planes", toys::place_small_toy_planes));
    r.push(("place_small_skateboards", games::place_small_skateboards));
    r.push(("place_small_hula_hoops", games::place_small_hula_hoops));
    r.push(("place_small_chalk_sticks", games::place_small_chalk_sticks));
    r.push(("place_small_spinning_tops", games::place_small_spinning_tops));
    r.push(("place_hopscotch_chalks", games::place_hopscotch_chalks));
    r.push(("place_small_darts", games::place_small_darts));
    r.push(("place_small_pool_cues", games::place_small_pool_cues));
    r.push(("place_small_dominoes", games::place_small_dominoes));
    r.push(("place_small_pool_balls", games::place_small_pool_balls));
    r.push(("place_small_chess_pieces", games::place_small_chess_pieces));
    r.push(("place_small_poker_chips", games::place_small_poker_chips));
    r.push(("place_small_dice_pairs", games::place_small_dice_pairs));
    r.push(("place_small_card_stacks", games::place_small_card_stacks));
    r.push(("place_small_hairbrushes", games::place_small_hairbrushes));
}

// ------------------------------------------------------------------ shared idioms

/// The open-tile scan plus one random pick that every placer here starts with.
/// `None` when there are no open tiles (no draw happens then).
pub(crate) fn pick_open_tile(g: &Game) -> Option<(i32, i32)> {
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    if open_tiles.is_empty() {
        return None;
    }
    Some(open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize])
}

/// Position `o` at `(wx(x) + (rnd() - 0.5) * s, 0, wz(z) + (rnd() - 0.5) * s)`, x drawn first.
pub(crate) fn scatter_pos(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().set(px, 0, pz);
}

/// Set only x and z: `wx(x) + (rnd() - 0.5) * s`, then `wz(z) + (rnd() - 0.5) * s`.
pub(crate) fn scatter_xz(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    o.position().x = px;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().z = pz;
}

/// A uniformly random colour (one `rnd()` draw).
#[inline]
pub(crate) fn pick_color(c: &[u32]) -> u32 {
    c[(rnd() * c.len() as f64).floor() as usize]
}

/// A custom geometry from a flat position list (f64 → f32) and an index.
pub(crate) fn tri_geo(verts: &[f64], idx: Vec<u32>) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&v| v as f32).collect());
    geo.set_index(idx);
    geo.compute_vertex_normals();
    geo
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 28);
        game::debug::testkit::smoke_placers(&list);
    }
}
