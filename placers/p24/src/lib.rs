//! Prop placers ported from backrooms.js (chunk p24; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod games;
mod toys;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeSmallPrincessWands", toys::place_small_princess_wands));
    r.push(("placeSmallToyRobots", toys::place_small_toy_robots));
    r.push(("placeSmallToyMicrophones", toys::place_small_toy_microphones));
    r.push(("placeSmallToyDinos", toys::place_small_toy_dinos));
    r.push(("placeSmallToyPhones", toys::place_small_toy_phones));
    r.push(("placeSmallPiggyBanks", toys::place_small_piggy_banks));
    r.push(("placeSmallMagnifyingGlasses", toys::place_small_magnifying_glasses));
    r.push(("placeSmallJacks", toys::place_small_jacks));
    r.push(("placeMagneticLetters", toys::place_magnetic_letters));
    r.push(("placeSmallRemoteControls", toys::place_small_remote_controls));
    r.push(("placeKidsDrawings", toys::place_kids_drawings));
    r.push(("placeSmallToyBoats", toys::place_small_toy_boats));
    r.push(("placeSmallMusicBoxes", toys::place_small_music_boxes));
    r.push(("placeSmallToyPlanes", toys::place_small_toy_planes));
    r.push(("placeSmallSkateboards", games::place_small_skateboards));
    r.push(("placeSmallHulaHoops", games::place_small_hula_hoops));
    r.push(("placeSmallChalkSticks", games::place_small_chalk_sticks));
    r.push(("placeSmallSpinningTops", games::place_small_spinning_tops));
    r.push(("placeHopscotchChalks", games::place_hopscotch_chalks));
    r.push(("placeSmallDarts", games::place_small_darts));
    r.push(("placeSmallPoolCues", games::place_small_pool_cues));
    r.push(("placeSmallDominoes", games::place_small_dominoes));
    r.push(("placeSmallPoolBalls", games::place_small_pool_balls));
    r.push(("placeSmallChessPieces", games::place_small_chess_pieces));
    r.push(("placeSmallPokerChips", games::place_small_poker_chips));
    r.push(("placeSmallDicePairs", games::place_small_dice_pairs));
    r.push(("placeSmallCardStacks", games::place_small_card_stacks));
    r.push(("placeSmallHairbrushes", games::place_small_hairbrushes));
}

// ------------------------------------------------------------------ shared idioms

/// The open-tile scan + `openTiles[Math.floor(Math.random() * openTiles.length)]`
/// every placer here starts with. `None` = `if (openTiles.length === 0) return;`
/// (no draw happens then, exactly like the JS).
pub(crate) fn pick_open_tile(g: &Game) -> Option<(i32, i32)> {
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { open_tiles.push((x, z)); }
        }
    }
    if open_tiles.is_empty() { return None; }
    Some(open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize])
}

/// `o.position.set(wx(x) + (Math.random() - 0.5) * s, 0, wz(z) + (Math.random() - 0.5) * s)`
pub(crate) fn scatter_pos(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().set(px, 0, pz);
}

/// `o.position.x = wx(x) + (Math.random() - 0.5) * s; o.position.z = wz(z) + (Math.random() - 0.5) * s;`
pub(crate) fn scatter_xz(g: &Game, o: &Obj, x: i32, z: i32, s: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    o.position().x = px;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    o.position().z = pz;
}

/// `colors[Math.floor(Math.random() * colors.length)]`
#[inline]
pub(crate) fn pick_color(c: &[u32]) -> u32 { c[(rnd() * c.len() as f64).floor() as usize] }

/// A BufferGeometry from a flat position list (f64 → Float32Array) and an index.
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
        game::testkit::smoke_placers(&list);
    }
}
