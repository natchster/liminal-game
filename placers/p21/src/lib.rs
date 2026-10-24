//! Prop placers ported from backrooms.js (chunk p21, lines 38348-40110; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod placers;
pub mod textures;

use placers::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeHospitalCrosses", place_hospital_crosses));
    r.push(("placeChildrensTeaSets", place_childrens_tea_sets));
    r.push(("placeCoinScatters", place_coin_scatters));
    r.push(("placeWeddingRings", place_wedding_rings));
    r.push(("placeWaterPuddles", place_water_puddles));
    r.push(("placePorcelainCupSaucers", place_porcelain_cup_saucers));
    r.push(("placeAlarmBells", place_alarm_bells));
    r.push(("placeRustStainStreaks", place_rust_stain_streaks));
    r.push(("placeForgottenWristwatches", place_forgotten_wristwatches));
    r.push(("placeClawMarks", place_claw_marks));
    r.push(("placeRustedNails", place_rusted_nails));
    r.push(("placeBiohazardSigns", place_biohazard_signs));
    r.push(("placeTeddyPaws", place_teddy_paws));
    r.push(("placeCautionTapes", place_caution_tapes));
    r.push(("placeBrokenBottles", place_broken_bottles));
    r.push(("placeChalkArrows", place_chalk_arrows));
    r.push(("placeShatteredEyeglasses", place_shattered_eyeglasses));
    r.push(("placeMouseTraps", place_mouse_traps));
    r.push(("placeCrayonGraffiti", place_crayon_graffiti));
    r.push(("placeFamilyPortraits", place_family_portraits));
    r.push(("placeCassetteTapes", place_cassette_tapes));
    r.push(("placeMopBuckets", place_mop_buckets));
    r.push(("placeDustBunnies", place_dust_bunnies));
    r.push(("placeColoringBooks", place_coloring_books));
    r.push(("placeVinylRecordDiscs", place_vinyl_record_discs));
    r.push(("placeBrokenPictureFrames", place_broken_picture_frames));
    r.push(("placeOpenedSuitcases", place_opened_suitcases));
    r.push(("placeRustedTinCans", place_rusted_tin_cans));
    r.push(("placeDeerHeadMounts", place_deer_head_mounts));
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

/// A one-triangle BufferGeometry (`setIndex([0, 1, 2])`, normals computed).
pub(crate) fn tri_geo(v: [f64; 9]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(vec![0, 1, 2]);
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
        assert_eq!(list.len(), 29);
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
            (rust_stain_texture(), 128, 320), (claw_marks_texture(), 256, 320), (biohazard_texture(), 160, 160),
            (caution_tape_texture(), 512, 48), (chalk_arrow_texture(), 256, 128), (crayon_graffiti_texture(), 384, 320),
            (family_portrait_texture(), 160, 128), (coloring_texture(), 320, 200), (exit_sign_texture(), 128, 64),
            (calendar_texture(), 192, 256),
        ] {
            assert_eq!((t.width(), t.height()), (w, h));
        }
    }
}
