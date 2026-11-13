//! The level sweep: properties every generated level must have, checked
//! through the real collision geometry over many seeds.
//!
//! For each `(run seed, level)`:
//!
//! * the spawn point is clear for a player-sized body (`PLAYER_R`);
//! * the exit can be walked to from spawn;
//! * open space is reachable: no tile with room for a body that the body
//!   cannot get to (a pocket sealed off by thick walls, bulges or jambs, which
//!   the tile-grid check inside archgen cannot see) — except behind a
//!   barricade, which by design may close off anything but the way from
//!   spawn to the exit;
//! * every pickup (almonds, batteries, paper piles, lore notes) can be walked
//!   to — a barricade must not lock one away;
//! * building the level again — in a fresh game, after another level — gives
//!   the same level.
//!
//! Reachability is a flood fill over a 0.25 m grid from spawn, a cell being
//! free when `circle_hits_wall` says a body fits there (walls, solids,
//! barricades: what the player actually collides with).
//!
//! The cases come from proptest; failing seeds are stored in
//! `tests/level_sweep.proptest-regressions` and replayed first (commit that
//! file). `SWEEP_CASES=500` runs a longer sweep (the nightly one).
use game::base::config::PLAYER_R;
use game::Game;
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};

/// Fine-grid cells per 4 m tile (0.25 m).
const FINE: i32 = 16;
/// A walk to the exit ends this close to it.
const EXIT_REACH: f64 = 2.5;
/// A pickup is in reach from this close.
const PICKUP_REACH: f64 = 1.5;
/// Found by this sweep (2026-10-25): barricades once kept only spawn → exit
/// solvable, so one could lock a pickup away (about 1 level in 20). Since
/// `barricades_settled` (`docs/design/PROPS.md` 2) keeps every pickup in reach, a
/// pickup lost behind one fails like any other.
const PICKUPS_BEHIND_BARRICADES: bool = false;

struct Report {
    /// the fine grid: 0 unknown, 1 free, 2 blocked; and which free cells were reached
    state: Vec<u8>,
    reached: Vec<bool>,
    spawn_clear: bool,
    exit_reached: Option<bool>,
    /// open tiles a body cannot reach: (tile x, tile z)
    pockets: Vec<(i32, i32)>,
    open_tiles: usize,
    /// pickups no body can get within `PICKUP_REACH` of: (what, x, z)
    lost_pickups: Vec<(&'static str, f64, f64)>,
    scp: Option<String>,
}

fn new_game() -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = "nosound".into();
    g.boot();
    g
}

fn build(g: &mut Game, run_seed: u32, level: i32) {
    g.run_seed = run_seed;
    g.init_level(level);
}

fn sweep(g: &Game) -> Report {
    let cs = 4.0 / FINE as f64;
    let (gw, gh) = (g.w * FINE, g.h * FINE);
    let x0 = g.wx(0) - 2.0;
    let z0 = g.wz(0) - 2.0;
    let center = |i: i32, j: i32| (x0 + (i as f64 + 0.5) * cs, z0 + (j as f64 + 0.5) * cs);
    let cell_of = |x: f64, z: f64| (((x - x0) / cs).floor() as i32, ((z - z0) / cs).floor() as i32);
    // 0 unknown, 1 free, 2 blocked
    let mut state = vec![0u8; (gw * gh) as usize];
    let free = |i: i32, j: i32, state: &mut Vec<u8>| -> bool {
        if i < 0 || j < 0 || i >= gw || j >= gh {
            return false;
        }
        let k = (j * gw + i) as usize;
        if state[k] == 0 {
            let (x, z) = center(i, j);
            state[k] = if g.circle_hits_wall(x, z, PLAYER_R) { 2 } else { 1 };
        }
        state[k] == 1
    };
    let (px, pz) = (g.player.pos.x, g.player.pos.z);
    let spawn_clear = !g.circle_hits_wall(px, pz, PLAYER_R);
    let mut reached = vec![false; (gw * gh) as usize];
    let (si, sj) = cell_of(px, pz);
    let mut queue = std::collections::VecDeque::new();
    // start from the free cell nearest the spawn point
    'find: for r in 0..6 {
        for dj in -r..=r {
            for di in -r..=r {
                if free(si + di, sj + dj, &mut state) {
                    queue.push_back((si + di, sj + dj));
                    reached[((sj + dj) * gw + si + di) as usize] = true;
                    break 'find;
                }
            }
        }
    }
    while let Some((i, j)) = queue.pop_front() {
        for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let (ni, nj) = (i + di, j + dj);
            if ni < 0 || nj < 0 || ni >= gw || nj >= gh || reached[(nj * gw + ni) as usize] {
                continue;
            }
            // no corner cutting
            if di != 0 && dj != 0 && !(free(i + di, j, &mut state) && free(i, j + dj, &mut state)) {
                continue;
            }
            if free(ni, nj, &mut state) {
                reached[(nj * gw + ni) as usize] = true;
                queue.push_back((ni, nj));
            }
        }
    }
    let near = |x: f64, z: f64, reach: f64| {
        let (ei, ej) = cell_of(x, z);
        let r = (reach / cs).ceil() as i32;
        (-r..=r).any(|dj| {
            (-r..=r).any(|di| {
                let (i, j) = (ei + di, ej + dj);
                i >= 0 && j >= 0 && i < gw && j < gh && reached[(j * gw + i) as usize] && ((di * di + dj * dj) as f64).sqrt() * cs <= reach
            })
        })
    };
    let exit_reached = g.exit_data.as_ref().map(|e| near(e.pos.x, e.pos.z, EXIT_REACH));
    let mut lost_pickups = vec![];
    let mut look = |what: &'static str, o: &engine::scene::Obj| {
        let p = o.world_position();
        if !near(p.x, p.z, PICKUP_REACH) {
            lost_pickups.push((what, (p.x * 10.0).round() / 10.0, (p.z * 10.0).round() / 10.0));
        }
    };
    g.almonds.iter().for_each(|p| look("almond water", &p.g));
    g.battery_items.iter().for_each(|p| look("battery", &p.g));
    g.papers.iter().for_each(|p| look("paper pile", &p.g));
    g.lore_notes.iter().for_each(|p| look("lore note", &p.g));
    // open tiles: any free cell inside; reachable: any reached cell inside
    let mut pockets = vec![];
    let mut open_tiles = 0;
    for tz in 0..g.h {
        for tx in 0..g.w {
            if g.is_wall_tile(tx, tz) {
                continue;
            }
            let (mut open, mut got) = (false, false);
            for j in tz * FINE..(tz + 1) * FINE {
                for i in tx * FINE..(tx + 1) * FINE {
                    if reached[(j * gw + i) as usize] {
                        got = true;
                    } else if free(i, j, &mut state) {
                        open = true;
                    }
                }
            }
            if open || got {
                open_tiles += 1;
                if !got {
                    pockets.push((tx, tz));
                }
            }
        }
    }
    Report { state, reached, spawn_clear, exit_reached, pockets, open_tiles, lost_pickups, scp: g.level_scp.clone() }
}

/// A connected region of open tiles no body can reach.
struct Pocket {
    tiles: Vec<(i32, i32)>,
    /// the floor tiles next to it that are filled solid by the walls: where
    /// the tile grid connects it and the geometry does not
    sealed_by: Vec<(i32, i32)>,
    /// sealed only by barricades: by design, they may close off anything but
    /// the way from spawn to the exit
    known: bool,
    /// how it is sealed, for the report
    cause: &'static str,
}

fn pockets(g: &Game, r: &Report) -> Vec<Pocket> {
    let gw = g.w * FINE;
    let roomy = |tx: i32, tz: i32| {
        (tz * FINE..(tz + 1) * FINE)
            .any(|j| (tx * FINE..(tx + 1) * FINE).any(|i| r.state[(j * gw + i) as usize] == 1 || r.reached[(j * gw + i) as usize]))
    };
    let set: std::collections::BTreeSet<(i32, i32)> = r.pockets.iter().copied().collect();
    let mut done = std::collections::BTreeSet::new();
    let mut out = vec![];
    for &start in &r.pockets {
        if !done.insert(start) {
            continue;
        }
        let (mut tiles, mut stack, mut sealed) = (vec![], vec![start], std::collections::BTreeSet::new());
        while let Some((x, z)) = stack.pop() {
            tiles.push((x, z));
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                if set.contains(&n) {
                    if done.insert(n) {
                        stack.push(n);
                    }
                } else if g.in_grid(n.0, n.1) && !g.is_wall_tile(n.0, n.1) && !roomy(n.0, n.1) {
                    sealed.insert(n);
                }
            }
        }
        let barricaded = |x: i32, z: i32| g.barricade_tiles.contains(&g.t_idx(x, z));
        // a barricade closes its opening with a slab, so its own tile is
        // half walked on: what lies behind it touches it
        let behind = tiles.iter().any(|&(x, z)| [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dz)| barricaded(x + dx, z + dz)));
        let cause = if sealed.iter().all(|&(x, z)| barricaded(x, z)) && behind {
            "behind a barricade (by design)"
        } else if sealed.is_empty() {
            "no way in on the tile grid"
        } else if sealed.iter().all(|&(x, z)| barricaded(x, z)) {
            "behind a barricade (by design)"
        } else {
            "sealed by geometry"
        };
        let known = cause.ends_with("(by design)");
        tiles.sort();
        out.push(Pocket { tiles, sealed_by: sealed.into_iter().collect(), known, cause });
    }
    out
}

/// The level's description without its random-draw trace (which counts draws).
fn fingerprint(g: &Game) -> String {
    let mut d = g.level_dump();
    if let Some(o) = d.as_object_mut() {
        o.remove("marks");
    }
    d.to_string()
}

fn check_level(run_seed: u32, level: i32) -> Result<String, String> {
    let mut g = new_game();
    if level > 1 {
        build(&mut g, run_seed, level - 1);
    }
    build(&mut g, run_seed, level);
    let r = sweep(&g);
    let after = fingerprint(&g);
    let mut fresh = new_game();
    build(&mut fresh, run_seed, level);
    let alone = fingerprint(&fresh);
    let mut problems = vec![];
    if !r.spawn_clear {
        problems.push("spawn is inside a wall".to_string());
    }
    if r.exit_reached == Some(false) {
        problems.push("the exit cannot be walked to".to_string());
    }
    let ps = pockets(&g, &r);
    // a pickup inside a barricaded pocket is the open issue `PICKUPS_BEHIND_BARRICADES`
    let in_barricaded = |x: f64, z: f64| ps.iter().any(|p| p.known && p.tiles.contains(&(g.tx(x), g.tz(z))));
    let (locked, lost): (Vec<&(&str, f64, f64)>, Vec<&(&str, f64, f64)>) =
        r.lost_pickups.iter().partition(|(_, x, z)| PICKUPS_BEHIND_BARRICADES && in_barricaded(*x, *z));
    if !lost.is_empty() {
        problems.push(format!("pickups no one can reach: {lost:?}"));
    }
    for p in ps.iter().filter(|p| !p.known) {
        problems.push(format!(
            "{} open tile(s) cannot be reached ({}), e.g. {:?}, sealed at {:?} — see `show_pocket`",
            p.tiles.len(),
            p.cause,
            &p.tiles[..p.tiles.len().min(4)],
            &p.sealed_by[..p.sealed_by.len().min(4)]
        ));
    }
    let known: Vec<String> = ps.iter().filter(|p| p.known).map(|p| format!("{} {}", p.tiles.len(), p.cause)).collect();
    if after != alone {
        problems.push("the level differs when another level was built before it".to_string());
    }
    let what = format!(
        "seed {run_seed} L{level} ({}, {} open tiles{}{})",
        r.scp.as_deref().unwrap_or("maze"),
        r.open_tiles,
        if known.is_empty() { String::new() } else { format!("; unreachable: {}", known.join(", ")) },
        if locked.is_empty() { String::new() } else { format!("; {} pickup(s) behind a barricade (open issue)", locked.len()) }
    );
    if problems.is_empty() {
        Ok(what)
    } else {
        Err(format!("{what}: {}", problems.join("; ")))
    }
}

/// Levels checked on every run before the random ones: each once showed
/// something the sweep must keep classifying right.
const ALWAYS: &[(u32, i32, &str)] = &[
    (1161854898, 7, "two rooms behind barricades, a battery locked in one"),
    (928196632, 4, "a loop level with three barricaded pockets, a pickup locked in one"),
];

#[test]
fn generated_levels_are_walkable_and_repeatable() {
    for &(seed, level, why) in ALWAYS {
        match check_level(seed, level) {
            Ok(what) => println!("ok    {what}  [{why}]"),
            Err(e) => panic!("{e}  [{why}]"),
        }
    }
    let cases = std::env::var("SWEEP_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    let mut runner = TestRunner::new(Config {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/level_sweep.proptest-regressions"))),
        // a smaller seed is not a simpler level: report the case as found
        max_shrink_iters: 0,
        ..Config::default()
    });
    let result = runner.run(&(any::<u32>(), 1i32..=9), |(seed, level)| match check_level(seed, level) {
        Ok(what) => {
            println!("ok    {what}");
            Ok(())
        }
        Err(why) => Err(TestCaseError::fail(why)),
    });
    if let Err(e) = result {
        panic!("{e}");
    }
}

/// `SEED=… LEVEL=… TX=… TZ=… cargo test --release -p backrooms --test level_sweep -- --ignored --nocapture show_pocket`
/// draws the fine grid around a tile: `#` blocked, `.` reached, `o` free but
/// unreachable, `S` spawn.
#[test]
#[ignore]
fn show_pocket() {
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<i64>().ok()).unwrap_or_else(|| panic!("set {k}"));
    let (seed, level, tx, tz) = (env("SEED") as u32, env("LEVEL") as i32, env("TX") as i32, env("TZ") as i32);
    let mut g = new_game();
    build(&mut g, seed, level);
    let r = sweep(&g);
    let gw = g.w * FINE;
    let (si, sj) = (((g.player.pos.x - g.wx(0) + 2.0) / 0.25) as i32, ((g.player.pos.z - g.wz(0) + 2.0) / 0.25) as i32);
    for j in ((tz - 2) * FINE).max(0)..((tz + 3) * FINE).min(g.h * FINE) {
        let row: String = (((tx - 3) * FINE).max(0)..((tx + 4) * FINE).min(gw))
            .map(|i| {
                let k = (j * gw + i) as usize;
                if (i, j) == (si, sj) {
                    'S'
                } else if r.reached[k] {
                    '.'
                } else if r.state[k] == 1 {
                    'o'
                } else if r.state[k] == 2 {
                    '#'
                } else {
                    ' '
                }
            })
            .collect();
        println!("{row}");
    }
    println!("tile ({tx},{tz}) at world ({:.1}, {:.1}); wall tile: {}", g.wx(tx), g.wz(tz), g.is_wall_tile(tx, tz));
}

/// `cargo test --release -p backrooms --test level_sweep -- --ignored --nocapture census`:
/// pockets, spawn and exit over `SWEEP_CASES` fixed seeds (default 20), one
/// build each — how common a problem is before deciding what to do about it.
/// `SEED=… LEVEL=…` looks at one level, with the zones that seal each pocket.
#[test]
#[ignore]
fn census() {
    let n: u32 = std::env::var("SWEEP_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let mut g = new_game();
    let (mut with_pockets, mut tiles) = (0, 0);
    let one =
        std::env::var("SEED").ok().and_then(|v| v.parse::<u32>().ok()).zip(std::env::var("LEVEL").ok().and_then(|v| v.parse::<i32>().ok()));
    for k in 0..if one.is_some() { 1 } else { n } {
        let (seed, level) = one.unwrap_or((k.wrapping_mul(2654435761) ^ 0x5EED, 1 + (k as i32 % 9)));
        build(&mut g, seed, level);
        let r = sweep(&g);
        if !r.pockets.is_empty() {
            with_pockets += 1;
            tiles += r.pockets.len();
        }
        let ps = pockets(&g, &r);
        println!(
            "seed {seed:>10} L{level} {:9}: spawn {} exit {:?} lost pickups {:?} pockets {:?}",
            r.scp.as_deref().unwrap_or("maze"),
            if r.spawn_clear { "clear" } else { "BLOCKED" },
            r.exit_reached,
            r.lost_pickups,
            ps.iter().map(|p| (p.tiles.len(), p.cause, p.sealed_by.first().copied())).collect::<Vec<_>>()
        );
        let zones = g.layout.as_ref().map(|l| l.zones.clone()).unwrap_or_default();
        for p in &ps {
            for &(x, z) in p.sealed_by.iter().take(3) {
                let zi = g.zone_at_tile(x, z);
                let zn = zi.and_then(|i| zones.get(i));
                let pz = g.zone_at_tile(p.tiles[0].0, p.tiles[0].1).and_then(|i| zones.get(i));
                println!(
                    "      sealed at ({x},{z}) by zone {:?} {:?} wall {:?} {:?} m, jamb {:?}, protrude {:?}; the pocket is in {:?} {:?} wall {:?} {:?}",
                    zi, zn.map(|z| z.arch), zn.map(|z| z.wall), zn.map(|z| z.wall_t), zn.map(|z| z.jamb), zn.map(|z| z.protrude),
                    pz.map(|z| z.id), pz.map(|z| z.arch), pz.map(|z| z.wall), pz.map(|z| z.wall_t)
                );
            }
        }
    }
    println!("{with_pockets} of {n} levels have unreachable open tiles ({tiles} tiles)");
}
