//! Height in the world (`docs/MEGASTRUCTURE.md`, design 1): floors at
//! heights, a flight of stairs between them, voids you cannot enter but see
//! across, slabs you see no further through, on a small hand-made floor of
//! the Building (`Game::test_ground_level`).
use game::level::archgen::{build_ground_solids, ground_breaks, segment_clear, segment_clear_3d, Ground, BOX_VOID};
use game::prelude::*;

const DT: f64 = 1.0 / 60.0;

fn playing() -> Game {
    let mut g = Game::test_ground_level();
    g.init_player();
    g.state = "playing".into();
    g.pointer_locked = true;
    g
}

/// Walk with `key` held for `secs`, facing `yaw`
fn walk(g: &mut Game, yaw: f64, key: &str, secs: f64) {
    g.player.yaw = yaw;
    g.key_down(key, false);
    for _ in 0..(secs / DT) as usize {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.update_player(DT);
    }
    g.key_up(key);
}

/// Facing +x, +z (forward is (−sin yaw, −cos yaw))
const EAST: f64 = -PI / 2.0;
const SOUTH: f64 = PI;

#[test]
fn the_floor_has_a_height_and_a_stair_a_slope() {
    let g = Game::test_ground_level();
    assert_eq!(g.floor_y_of(2, 1), 0.0);
    assert_eq!(g.floor_y_of(6, 1), -3.0);
    // the flight: from 0 at its back edge to −3 at its front edge
    let (u0, um, u1) = (g.wx(4) - CELL * 0.5, g.wx(4), g.wx(4) + CELL * 0.5);
    let z = g.wz(1);
    assert!((g.floor_y_at(u0 + 1e-6, z) - 0.0).abs() < 1e-3);
    assert!((g.floor_y_at(um, z) + 1.5).abs() < 1e-9);
    assert!((g.floor_y_at(u1 - 1e-6, z) + 3.0).abs() < 1e-3);
    // heights change only along the flight or across the wall and the void
    assert!(ground_breaks(g.w, g.h, &g.ground).is_empty(), "{:?}", ground_breaks(g.w, g.h, &g.ground));
    // the rooms have no ground: every floor at 0
    let rooms = Game::new_bare();
    assert_eq!(rooms.floor_y_at(3.0, -7.0), 0.0);
}

#[test]
fn the_rule_finds_two_heights_that_touch() {
    let mut ground = vec![Ground::floor(0.0); 9];
    ground[4] = Ground::floor(3.0);
    assert_eq!(ground_breaks(3, 3, &ground).len(), 4, "the raised middle touches four neighbours");
    // a stair meets its ends, not its sides
    let mut ground = vec![Ground::void(); 9];
    ground[3] = Ground::floor(0.0);
    ground[4] = Ground::stair(0.0, 3.0, 0);
    ground[5] = Ground::floor(3.0);
    assert!(ground_breaks(3, 3, &ground).is_empty());
    ground[1] = Ground::floor(1.5);
    assert_eq!(ground_breaks(3, 3, &ground), vec![((1, 0), 2)], "a floor beside a flight's side");
    // two flights side by side, the same run, are one wide flight
    let mut ground = vec![Ground::void(); 9];
    for z in 0..2 {
        ground[z * 3] = Ground::floor(0.0);
        ground[z * 3 + 1] = Ground::stair(0.0, 3.0, 0);
        ground[z * 3 + 2] = Ground::floor(3.0);
    }
    assert!(ground_breaks(3, 3, &ground).is_empty());
    ground[4] = Ground::stair(0.0, 1.5, 0);
    assert!(!ground_breaks(3, 3, &ground).is_empty(), "a different run beside it");
}

#[test]
fn down_the_flight_and_back_up() {
    let mut g = playing();
    walk(&mut g, EAST, "KeyW", 3.5);
    let (x, z) = (g.tx(g.player.pos.x), g.tz(g.player.pos.z));
    assert_eq!(z, 1);
    assert!(x >= 5, "walked down the flight to the lower gallery (tile x {x})");
    assert!((g.player.pos.y - (-3.0 + EYE)).abs() < 1e-9, "stands on the lower gallery: y {}", g.player.pos.y);
    let cam = g.camera.pos().y;
    assert!(cam > -3.0 + 1.3 && cam < -3.0 + 1.9, "the eye is over the lower floor: {cam}");
    // halfway back up the flight the body is halfway up
    g.player.pos.x = g.wx(5);
    walk(&mut g, -EAST, "KeyW", 0.8);
    let (px, pz) = (g.player.pos.x, g.player.pos.z);
    assert_eq!(g.tx(px), 4, "on the flight");
    assert!((g.player.pos.y - EYE - g.floor_y_at(px, pz)).abs() < 1e-9);
    assert!(g.player.pos.y - EYE > -3.0 && g.player.pos.y - EYE < 0.0);
    walk(&mut g, -EAST, "KeyW", 3.0);
    assert!((g.player.pos.y - EYE).abs() < 1e-9, "back on the upper gallery");
}

#[test]
fn a_void_and_a_cliff_stop_you() {
    let mut g = playing();
    // into the void from the upper gallery
    g.player.pos.x = g.wx(3);
    walk(&mut g, SOUTH, "KeyW", 2.0);
    assert_eq!((g.tx(g.player.pos.x), g.tz(g.player.pos.z)), (3, 1), "the void stopped the body");
    // into the wall between the heights on the bottom row
    g.player.pos.x = g.wx(3);
    g.player.pos.z = g.wz(3);
    walk(&mut g, EAST, "KeyW", 2.0);
    assert_eq!((g.tx(g.player.pos.x), g.tz(g.player.pos.z)), (3, 3), "the wall stopped the body");
    // off the side of the flight
    g.player.pos.x = g.wx(4);
    g.player.pos.z = g.wz(1);
    walk(&mut g, SOUTH, "KeyW", 2.0);
    assert_eq!(g.tz(g.player.pos.z), 1, "the flight's side is the void");
}

#[test]
fn seen_across_a_void_not_through_a_floor_or_a_wall() {
    let g = Game::test_ground_level();
    let s = g.solids.as_ref().unwrap();
    // across the void between the two galleries
    assert!(g.los(g.wx(2), g.wz(1), g.wx(2), g.wz(3)));
    // from the upper gallery down to the lower one, over the void
    assert!(g.los(g.wx(2), g.wz(3), g.wx(6), g.wz(1)));
    // through the wall between the heights
    assert!(!g.los(g.wx(3), g.wz(3), g.wx(5), g.wz(3)));
    // from under the upper gallery up through its slab
    assert!(!segment_clear_3d(s, 2.2, 2.5, -2.0, 2.2, 1.2, 1.0), "the slab stops the look");
    assert!(segment_clear_3d(s, 2.2, 2.5, -2.0, 2.2, 2.9, 1.0), "over the void only");
    // the 3D line at one height is the flat one
    for &(u0, v0, u1, v1) in &[(1.5, 1.5, 7.5, 3.5), (2.5, 3.5, 6.5, 3.5), (1.5, 3.5, 7.5, 1.5)] {
        assert_eq!(segment_clear_3d(s, u0, v0, 1.5, u1, v1, 1.5), segment_clear(s, u0, v0, u1, v1, 1.5));
    }
    // a void is one box that stops a body
    let void_boxes = (0..s.count).filter(|&b| s.kind(b) == BOX_VOID).count();
    assert_eq!(void_boxes, 5);
    let again = build_ground_solids(g.w, g.h, CELL, &g.ground);
    assert_eq!(again.boxes, s.boxes, "the solids are a function of the ground");
}
