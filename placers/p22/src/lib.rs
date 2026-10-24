//! Prop placers ported from backrooms.js (chunk p22, lines 40193-41849; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod placers;
pub mod textures;

use placers::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeOverturnedTricycles", place_overturned_tricycles));
    r.push(("placeOldCRTTVs", place_old_crt_tvs));
    r.push(("placeOverflowingAshtrays", place_overflowing_ashtrays));
    r.push(("placePeelingWallpapers", place_peeling_wallpapers));
    r.push(("placeSpilledPillBottles", place_spilled_pill_bottles));
    r.push(("placeReligiousTriptychs", place_religious_triptychs));
    r.push(("placeShatteredVases", place_shattered_vases));
    r.push(("placeWornTeddyBears", place_worn_teddy_bears));
    r.push(("placeDeadMoths", place_dead_moths));
    r.push(("placeDollBodyParts", place_doll_body_parts));
    r.push(("placeSingleChildShoes", place_single_child_shoes));
    r.push(("placeDeadFlowerVases", place_dead_flower_vases));
    r.push(("placeBabyCribFragments", place_baby_crib_fragments));
    r.push(("placeCrystalChandeliers", place_crystal_chandeliers));
    r.push(("placeCrackedMirrors", place_cracked_mirrors));
    r.push(("placeRockingChairs", place_rocking_chairs));
    r.push(("placeScatteredPages", place_scattered_pages));
    r.push(("placeWallChalkboards", place_wall_chalkboards));
    r.push(("placeDollHeads", place_doll_heads));
    r.push(("placeCoatHooks", place_coat_hooks));
    r.push(("placeHangingCensers", place_hanging_censers));
    r.push(("placeBrokenWallClocks", place_broken_wall_clocks));
    r.push(("placeAncientBookStacks", place_ancient_book_stacks));
}

// ------------------------------------------------------------------ shared helpers
// (the JS repeats these loops verbatim in every placer; none of them draw randoms
// except where noted, so factoring them out keeps the draw sequence intact)

/// `{ x, z, dx, dz }`: an open tile and the side on which it touches a wall.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WallCand { pub x: i32, pub z: i32, pub dx: i32, pub dz: i32 }

/// `openTiles`: every interior tile with `tiles[...] === 0`, row-major.
pub(crate) fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// `candidates`: open tiles next to a wall (or the grid edge), one entry per wall side.
pub(crate) fn wall_candidates(g: &Game) -> Vec<WallCand> {
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

/// The wall-mount tail every wall prop shares: face the room, sit `inset` off
/// the wall at `center_y`, slide along the wall by `(rand - 0.5) * jitter`
/// (one draw).
pub(crate) fn mount_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64, jitter: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * jitter);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * jitter, center_y, wall_z);
    }
}

/// `o.position.set(wx(x) + (rand - 0.5) * spread, 0, wz(z) + (rand - 0.5) * spread);
///  o.rotation.y = rand * 2π` (three draws).
pub(crate) fn scatter_on_floor(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    o.position().set(g.wx(x) + (rnd() - 0.5) * spread, 0, g.wz(z) + (rnd() - 0.5) * spread);
    o.rotation().y = rnd() * PI * 2.0;
}

/// A BufferGeometry from Float32Array positions + an index, normals computed.
pub(crate) fn buffer_geo(verts: &[f64], index: Vec<u32>) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&a| a as f32).collect());
    geo.set_index(index);
    geo.compute_vertex_normals();
    geo
}

/// The four bars of a picture frame: `[barW, barH, bx, by]`.
pub(crate) fn frame_bars(w: f64, h: f64, fw: f64) -> [(f64, f64, f64, f64); 4] {
    [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ]
}

/// Store a number into a `Uint8ClampedArray` (round half to even, clamp 0..255).
pub(crate) fn u8_clamped(v: f64) -> u8 {
    if !(v > 0.0) { return 0; }
    if v >= 255.0 { return 255; }
    let f = v.floor();
    let d = v - f;
    let r = if d > 0.5 { f + 1.0 } else if d < 0.5 { f } else if f % 2.0 == 0.0 { f } else { f + 1.0 };
    r as u8
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }

    /// Every placer gets past its rare-chance early return at least once and
    /// adds something to `world` (so the smoke run really covers its body).
    #[test]
    fn every_placer_builds() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 23);
        for (name, f) in &list {
            let (mut g, dist) = game::Game::test_level(7, 25);
            game::rng::with_seed(12345, || { for _ in 0..400 { f(&mut g, &dist); } });
            assert!(g.world.child_count() > 0, "{name} never placed anything");
        }
    }

    #[test]
    fn textures_build() {
        use super::textures::*;
        for (t, w, h) in [
            (crt_static_texture(), 128, 96), (wallpaper_texture(), 192, 320), (triptych_texture(), 320, 256),
            (cracked_mirror_texture(), 192, 256), (scattered_page_texture(), 128, 160), (chalkboard_texture(), 256, 192),
            (broken_clock_texture(), 128, 128),
        ] {
            assert_eq!((t.width(), t.height()), (w, h));
        }
    }

    #[test]
    fn u8_clamped_matches_uint8clampedarray() {
        use super::u8_clamped;
        assert_eq!(u8_clamped(-3.0), 0);
        assert_eq!(u8_clamped(300.0), 255);
        assert_eq!(u8_clamped(40.4), 40);
        assert_eq!(u8_clamped(40.6), 41);
        assert_eq!(u8_clamped(40.5), 40); // ties to even
        assert_eq!(u8_clamped(41.5), 42);
    }
}
