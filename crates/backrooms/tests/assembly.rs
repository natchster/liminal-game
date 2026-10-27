//! The self-assembly on the Building's floors (`docs/MEGASTRUCTURE.md`,
//! B5; `game::world::assembly`): the siren before the fog, the fog rising
//! out of the void and sinking again in the director's phases, a sealed
//! cell keeps it out, out in it you are gone, her page sounds the siren,
//! and every floor tile has a cell within reach of the warning at a walk.
mod common;

use game::base::config::{CELL, EYE, WALK};
use game::entities::director::Phase;
use game::level::building::CELL_REACH;
use game::world::assembly::{Stage, WARNING};
use game::Game;

const DT: f64 = 1.0 / 60.0;

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native(0xA55E_0000 ^ u64::from(seed) ^ ((level as u64) << 32));
    let mut g = common::new_game();
    g.run_seed = seed;
    g.start_new_game();
    g.init_level(level);
    g.state = "playing".into();
    g.hide_overlay();
    g.pointer_locked = true;
    g
}

fn step(g: &mut Game, secs: f64) {
    for _ in 0..(secs / DT).round() as usize {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
    }
}

/// Stand the body on tile (x, z), on its floor
fn stand(g: &mut Game, x: i32, z: i32) {
    let (wx, wz) = (g.wx(x), g.wz(z));
    g.player.pos = engine::math::Vec3::new(wx, g.floor_y_at(wx, wz) + EYE, wz);
}

#[test]
fn the_siren_comes_first_then_the_fog_rises_and_sinks() {
    let mut g = floor(4242, 1);
    g.god_mode = true;
    // stand in a cell, sealed, out of the way
    let c = g.building.as_ref().unwrap().cells[0].tile;
    stand(&mut g, c.0, c.1);
    g.turn_cell_wheel(0, true);
    g.assembly.t = 1.0;
    let (mut stages, mut last_y) = (vec![], f64::NEG_INFINITY);
    let mut rose_while = (true, true);
    for _ in 0..(400.0 / 0.25) as usize {
        step(&mut g, 0.25);
        let a = &g.assembly;
        if stages.last() != Some(&a.stage) {
            stages.push(a.stage);
        }
        match a.stage {
            Stage::Siren | Stage::Rising => rose_while.0 &= a.y >= last_y - 1e-9,
            Stage::Falling => rose_while.1 &= a.y <= last_y + 1e-9,
            _ => {}
        }
        last_y = a.y;
        if stages.len() >= 6 && a.stage == Stage::Quiet {
            break;
        }
    }
    assert_eq!(
        stages,
        vec![Stage::Quiet, Stage::Siren, Stage::Rising, Stage::Crest, Stage::Falling, Stage::Quiet],
        "the order: {stages:?}"
    );
    assert!(rose_while.0 && rose_while.1, "it rises, then it sinks");
    // the director's log reads as on the rooms: build, peak, relax
    let log: Vec<&str> = g.director.log.iter().map(|e| e.1).filter(|w| ["build", "peak", "relax"].contains(w)).collect();
    assert_eq!(log, vec!["build", "peak", "relax"]);
    assert_eq!(g.director.phase, Phase::Calm);
    assert_eq!(g.state, "playing");
}

#[test]
fn a_sealed_cell_keeps_it_out_and_out_in_it_you_are_gone() {
    for sealed in [true, false] {
        let mut g = floor(4242, 1);
        let c = g.building.as_ref().unwrap().cells[0].tile;
        stand(&mut g, c.0, c.1);
        if sealed {
            g.turn_cell_wheel(0, true);
        }
        // the fog stands over the cell's floor
        let h = g.floor_y_of(c.0, c.1);
        g.assembly.stage = Stage::Crest;
        g.assembly.y = h + 6.0;
        g.assembly.crest = h + 6.0;
        g.assembly.t = 30.0;
        let s0 = g.player.sanity;
        step(&mut g, 14.0);
        if sealed {
            assert_eq!(g.state, "playing", "sealed in, it does not come in");
            assert!(g.player.sanity >= s0 - 1e-9, "sealed in, the mind holds: {s0} → {}", g.player.sanity);
        } else {
            assert_eq!(g.state, "dead", "out in it");
            assert_eq!(g.last_death_cause.as_deref(), Some("assembly"));
        }
    }
}

#[test]
fn a_shut_door_stops_a_body_and_an_open_one_lets_it_by() {
    let mut g = floor(4242, 2);
    let d = g.cell_doors[0].clone();
    let (mx, mz) = ((d.span[0] + d.span[2]) / 2.0, (d.span[1] + d.span[3]) / 2.0);
    assert!(!g.hits_cell_door(mx, mz, 0.35), "open, the doorway is free");
    g.turn_cell_wheel(0, true);
    assert!(g.hits_cell_door(mx, mz, 0.35), "shut, it stops a body");
    assert!(g.circle_hits_wall(mx, mz, 0.35));
    g.turn_cell_wheel(0, false);
    assert!(!g.hits_cell_door(mx, mz, 0.35));
}

#[test]
fn her_page_sounds_the_siren() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        assert_eq!(g.assembly.stage, Stage::Quiet);
        assert!(g.assembly.t > 60.0, "level {level}: a long quiet first ({:.0} s)", g.assembly.t);
        g.take_page();
        step(&mut g, 9.5);
        assert!(g.assembly.stage == Stage::Siren, "level {level}: the siren within seconds of her page ({:?})", g.assembly.stage);
    }
}

#[test]
fn a_cell_is_in_reach_of_the_warning() {
    // the farthest any floor tile is from a cell, at a walk, inside the
    // siren's warning (before the fog reaches even the lowest walk)
    let walk_s = CELL_REACH as f64 * CELL / WALK;
    assert!(walk_s * 1.5 < WARNING, "{walk_s:.0} s to the farthest cell at a walk, {WARNING:.0} s of warning");
    for level in 1..=3 {
        for seed in [4242u32, 7, 77, 1999] {
            let (l, p) = game::base::rng::with_seed(game::base::rng::seed_for_level(seed, level), || {
                game::level::building::generate_building(level, &mut || game::base::rng::rnd())
            });
            let m = game::level::building::navigation(&l, &p);
            assert!(m.cell_reach <= CELL_REACH, "seed {seed} level {level}: a floor {} tiles from a cell", m.cell_reach);
        }
    }
}

/// The Building's floors paced: on each, twelve minutes standing sealed in
/// the first cell. The first siren comes within two and a half minutes (the
/// first floor waits longer for her camp to be found), the quiet between
/// two is long, and the fog comes again. A log, like `tests/pacing.rs`'s.
#[test]
fn the_building_paces_a_floor() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        g.god_mode = true;
        let c = g.building.as_ref().unwrap().cells[0].tile;
        stand(&mut g, c.0, c.1);
        g.turn_cell_wheel(0, true);
        let mut spans: Vec<(Phase, f64)> = vec![(g.director.phase, 0.0)];
        let (mut first_siren, mut t) = (None, 0.0);
        for _ in 0..(720.0 / 0.5) as usize {
            step(&mut g, 0.5);
            t += 0.5;
            if g.assembly.stage == Stage::Siren && first_siren.is_none() {
                first_siren = Some(t);
            }
            match spans.last_mut() {
                Some(s) if s.0 == g.director.phase => s.1 += 0.5,
                _ => spans.push((g.director.phase, 0.5)),
            }
        }
        let text: Vec<String> = spans.iter().map(|(p, s)| format!("{} {s:.0}s", p.name())).collect();
        println!("seed 4242 L{level}: {} assemblies, first siren at {first_siren:?} s\n  {}", g.assembly.count, text.join(" → "));
        let fs = first_siren.unwrap_or(f64::INFINITY);
        let most = if level == 1 { 165.0 } else { 135.0 };
        assert!(fs <= most, "L{level}: the first siren within {most} s ({fs:.0} s)");
        assert!(g.assembly.count >= 3, "L{level}: it comes again ({} in 12 minutes)", g.assembly.count);
        for (p, s) in &spans[1..spans.len() - 1] {
            if *p == Phase::Calm {
                assert!(*s >= 60.0, "L{level}: a quiet between two of {s:.0} s");
            }
        }
        assert_eq!(g.state, "playing");
    }
}
