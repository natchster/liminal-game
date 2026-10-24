//! Prop placers ported from backrooms.js (chunk p25; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod dressing;
mod music;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeSmallJewelryBoxes", dressing::place_small_jewelry_boxes));
    r.push(("placeSmallPerfumes", dressing::place_small_perfumes));
    r.push(("placeSmallNailPolishes", dressing::place_small_nail_polishes));
    r.push(("placeSmallLipsticks", dressing::place_small_lipsticks));
    r.push(("placeSmallCompacts", dressing::place_small_compacts));
    r.push(("placeSmallHandMirrors", dressing::place_small_hand_mirrors));
    r.push(("placeSmallFoldingFans", dressing::place_small_folding_fans));
    r.push(("placeSmallPaperCrowns", dressing::place_small_paper_crowns));
    r.push(("placeSmallOperaGlasses", dressing::place_small_opera_glasses));
    r.push(("placeSmallTiaras", dressing::place_small_tiaras));
    r.push(("placeSmallCastanets", dressing::place_small_castanets));
    r.push(("placeSmallMetronomes", dressing::place_small_metronomes));
    r.push(("placeSmallFeatherBoas", dressing::place_small_feather_boas));
    r.push(("placeMusicSheets", music::place_music_sheets));
    r.push(("placeConductorBatons", music::place_conductor_batons));
    r.push(("placeSmallXylophones", music::place_small_xylophones));
    r.push(("placeSmallKazoos", music::place_small_kazoos));
    r.push(("placeSmallRecorders", music::place_small_recorders));
    r.push(("placeSmallTriangles", music::place_small_triangles));
    r.push(("placePaperGarlands", music::place_paper_garlands));
    r.push(("placeSmallMaracas", music::place_small_maracas));
    r.push(("placeSmallTrumpets", music::place_small_trumpets));
    r.push(("placeHangingBalloonBundles", music::place_hanging_balloon_bundles));
    r.push(("placeHangingStreamers", music::place_hanging_streamers));
    r.push(("placeSmallPartyBanners", music::place_small_party_banners));
    r.push(("placeCandleClusters", music::place_candle_clusters));
    r.push(("placeSmallPinatas", music::place_small_pinatas));
    r.push(("placeSmallPaperCups", music::place_small_paper_cups));
}

/// The `openTiles` scan every placer starts with: open tiles, row-major.
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// `arr[Math.floor(Math.random() * arr.length)]`
#[inline]
pub(crate) fn pick<T: Copy>(a: &[T]) -> T { a[(rnd() * a.len() as f64).floor() as usize] }

/// `new THREE.BufferGeometry()` + `position` (Float32Array) + `setIndex(idx)` + `computeVertexNormals()`
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
        game::testkit::smoke_placers(&list);
    }
}
