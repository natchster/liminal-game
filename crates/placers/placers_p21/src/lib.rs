//! Prop placers: unsettling household remnants — hospital crosses, tea sets,
//! coins, rings, puddles, cups and saucers, alarm bells, rust streaks,
//! wristwatches, claw marks, nails, biohazard signs, caution tape, broken
//! bottles and glasses, chalk arrows, mouse traps, graffiti, family portraits,
//! tapes, mop buckets, dust bunnies, books, records, frames, suitcases, cans and
//! deer heads.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod placers;
pub mod textures;

use placers::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_hospital_crosses", place_hospital_crosses));
    r.push(("place_childrens_tea_sets", place_childrens_tea_sets));
    r.push(("place_coin_scatters", place_coin_scatters));
    r.push(("place_wedding_rings", place_wedding_rings));
    r.push(("place_water_puddles", place_water_puddles));
    r.push(("place_porcelain_cup_saucers", place_porcelain_cup_saucers));
    r.push(("place_alarm_bells", place_alarm_bells));
    r.push(("place_rust_stain_streaks", place_rust_stain_streaks));
    r.push(("place_forgotten_wristwatches", place_forgotten_wristwatches));
    r.push(("place_claw_marks", place_claw_marks));
    r.push(("place_rusted_nails", place_rusted_nails));
    r.push(("place_biohazard_signs", place_biohazard_signs));
    r.push(("place_teddy_paws", place_teddy_paws));
    r.push(("place_caution_tapes", place_caution_tapes));
    r.push(("place_broken_bottles", place_broken_bottles));
    r.push(("place_chalk_arrows", place_chalk_arrows));
    r.push(("place_shattered_eyeglasses", place_shattered_eyeglasses));
    r.push(("place_mouse_traps", place_mouse_traps));
    r.push(("place_crayon_graffiti", place_crayon_graffiti));
    r.push(("place_family_portraits", place_family_portraits));
    r.push(("place_cassette_tapes", place_cassette_tapes));
    r.push(("place_mop_buckets", place_mop_buckets));
    r.push(("place_dust_bunnies", place_dust_bunnies));
    r.push(("place_coloring_books", place_coloring_books));
    r.push(("place_vinyl_record_discs", place_vinyl_record_discs));
    r.push(("place_broken_picture_frames", place_broken_picture_frames));
    r.push(("place_opened_suitcases", place_opened_suitcases));
    r.push(("place_rusted_tin_cans", place_rusted_tin_cans));
    r.push(("place_deer_head_mounts", place_deer_head_mounts));
}

// ------------------------------------------------------------------ shared helpers
// (every placer runs these same loops; none of them draw randoms except where
// noted, so sharing them keeps each placer's draw sequence intact)

/// `{ x, z, dx, dz }`: an open tile and the side on which it touches a wall.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WallCand {
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
}

pub(crate) use placers_kit::place::open_tiles;

/// `candidates`: open tiles next to a wall (or the grid edge), one entry per wall side.
pub(crate) fn wall_candidates(g: &Game) -> Vec<WallCand> {
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

pub(crate) use placers_kit::place::scatter_on_floor;

pub(crate) use placers_kit::geometry::tri_geo;

pub(crate) use placers_kit::geometry::frame_bars;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
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
            game::base::rng::with_seed(12345, || {
                for _ in 0..400 {
                    f(&mut g, &dist);
                }
            });
            assert!(g.world.child_count() > 0, "{name} never placed anything");
        }
    }

    #[test]
    fn textures_build() {
        use super::textures::*;
        for (t, w, h) in [
            (rust_stain_texture(), 128, 320),
            (claw_marks_texture(), 256, 320),
            (biohazard_texture(), 160, 160),
            (caution_tape_texture(), 512, 48),
            (chalk_arrow_texture(), 256, 128),
            (crayon_graffiti_texture(), 384, 320),
            (family_portrait_texture(), 160, 128),
            (coloring_texture(), 320, 200),
            (exit_sign_texture(), 128, 64),
            (calendar_texture(), 192, 256),
        ] {
            assert_eq!((t.width(), t.height()), (w, h));
        }
    }
}
