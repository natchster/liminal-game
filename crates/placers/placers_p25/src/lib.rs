//! Prop placers: dressing-table items (jewelry boxes, perfume, nail polish,
//! lipstick, compacts, hand mirrors, fans), costume pieces (paper crowns, opera
//! glasses, tiaras, feather boas), music (castanets, metronomes, sheet music,
//! batons, xylophones, kazoos, recorders, triangles, maracas, trumpets) and
//! party decorations (garlands, balloons, streamers, banners, candles, piñatas,
//! paper cups).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod dressing;
mod music;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_small_jewelry_boxes", dressing::place_small_jewelry_boxes));
    r.push(("place_small_perfumes", dressing::place_small_perfumes));
    r.push(("place_small_nail_polishes", dressing::place_small_nail_polishes));
    r.push(("place_small_lipsticks", dressing::place_small_lipsticks));
    r.push(("place_small_compacts", dressing::place_small_compacts));
    r.push(("place_small_hand_mirrors", dressing::place_small_hand_mirrors));
    r.push(("place_small_folding_fans", dressing::place_small_folding_fans));
    r.push(("place_small_paper_crowns", dressing::place_small_paper_crowns));
    r.push(("place_small_opera_glasses", dressing::place_small_opera_glasses));
    r.push(("place_small_tiaras", dressing::place_small_tiaras));
    r.push(("place_small_castanets", dressing::place_small_castanets));
    r.push(("place_small_metronomes", dressing::place_small_metronomes));
    r.push(("place_small_feather_boas", dressing::place_small_feather_boas));
    r.push(("place_music_sheets", music::place_music_sheets));
    r.push(("place_conductor_batons", music::place_conductor_batons));
    r.push(("place_small_xylophones", music::place_small_xylophones));
    r.push(("place_small_kazoos", music::place_small_kazoos));
    r.push(("place_small_recorders", music::place_small_recorders));
    r.push(("place_small_triangles", music::place_small_triangles));
    r.push(("place_paper_garlands", music::place_paper_garlands));
    r.push(("place_small_maracas", music::place_small_maracas));
    r.push(("place_small_trumpets", music::place_small_trumpets));
    r.push(("place_hanging_balloon_bundles", music::place_hanging_balloon_bundles));
    r.push(("place_hanging_streamers", music::place_hanging_streamers));
    r.push(("place_small_party_banners", music::place_small_party_banners));
    r.push(("place_candle_clusters", music::place_candle_clusters));
    r.push(("place_small_pinatas", music::place_small_pinatas));
    r.push(("place_small_paper_cups", music::place_small_paper_cups));
}

/// The `open_tiles` scan every placer starts with: open tiles, row-major.
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// A custom geometry: f32 positions + `set_index(idx)` + `compute_vertex_normals()`
pub(crate) fn buf_geo(verts: &[f64], idx: &[u32]) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&v| v as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

/// `o.rotation.set(x, y, z, order)`
pub(crate) fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    let mut r = o.rotation();
    r.set(x, y, z);
    r.order = order;
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
