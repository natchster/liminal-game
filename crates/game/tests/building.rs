//! The Building's floors (`level::building`, `docs/design/MEGASTRUCTURE.md` B2):
//! deterministic, every walk reachable, heights that change only along a
//! flight or across a mass or a void, the door the far low point, and the
//! navigation measures in their bands over many seeds. Map dumps of a dozen
//! seeds go to `target/building/` (text) to look at.
use game::base::rng::{seed_for_level, with_seed};
use game::level::archgen::{ground_breaks, mulberry32, GROUND_MASS};
use game::level::building::{ascii, generate_building, navigation, walk_from, Navigation, Style, CELL_REACH};

fn build(run_seed: u32, level: i32) -> (game::level::archgen::Layout, game::level::building::BuildingPlan) {
    let seed = seed_for_level(run_seed, level);
    with_seed(seed, || generate_building(level, &mut || game::base::rng::rnd()))
}

const SEEDS: std::ops::Range<u32> = 1..41;

#[test]
fn a_floor_is_a_function_of_its_seed() {
    for level in 1..=3 {
        let (a, pa) = build(4242, level);
        let (b, pb) = build(4242, level);
        assert_eq!(a, b, "level {level}: same seed, same floor");
        assert_eq!(pa, pb);
        let (c, _) = build(4243, level);
        assert_ne!(a.ground, c.ground, "level {level}: another seed, another floor");
        // and the stream it was given is all it reads
        let mut r1 = mulberry32(99);
        let mut r2 = mulberry32(99);
        assert_eq!(generate_building(level, &mut r1), generate_building(level, &mut r2));
    }
}

#[test]
fn every_floor_holds_together() {
    for level in 1..=3 {
        for seed in SEEDS {
            let (l, p) = build(seed, level);
            let tag = format!("seed {seed} level {level}");
            assert_eq!(l.ground.len(), (l.w * l.h) as usize);
            assert!(ground_breaks(l.w, l.h, &l.ground).is_empty(), "{tag}: heights touch at {:?}", ground_breaks(l.w, l.h, &l.ground));
            // the border is building
            for k in 0..l.w {
                for (x, z) in [(k, 0), (k, l.h - 1), (0, k), (l.w - 1, k)] {
                    assert_eq!(l.ground[(z * l.w + x) as usize].kind, GROUND_MASS, "{tag}: border at ({x}, {z})");
                }
            }
            // every walk reachable from the arrival, the door among them
            let d = walk_from(&l, &[p.spawn]);
            for i in 0..l.tiles.len() {
                assert_eq!(l.tiles[i] == 0, l.ground[i].walkable(), "{tag}: tiles and ground agree at {i}");
                assert!(l.tiles[i] != 0 || d[i] >= 0, "{tag}: tile {i} cannot be reached");
            }
            assert_eq!(l.tiles[(p.spawn.1 * l.w + p.spawn.0) as usize], 0, "{tag}: the arrival is floor");
            assert!(d[(p.exit.1 * l.w + p.exit.0) as usize] > 0, "{tag}: the door is reached");
            // the door is set in a mass
            let (mx, mz) = (p.exit.0 + game::level::archgen::DIRS[p.exit_side][0], p.exit.1 + game::level::archgen::DIRS[p.exit_side][1]);
            assert_eq!(l.ground[(mz * l.w + mx) as usize].kind, GROUND_MASS, "{tag}: the door's wall");
            // every cell is a recess with one way out, at the height of its walk
            for c in &p.cells {
                let (x, z) = c.tile;
                let open: Vec<usize> = (0..4)
                    .filter(|&k| {
                        let (nx, nz) = (x + game::level::archgen::DIRS[k][0], z + game::level::archgen::DIRS[k][1]);
                        l.tiles[(nz * l.w + nx) as usize] == 0
                    })
                    .collect();
                assert_eq!(open, vec![c.door], "{tag}: cell {:?} opens only through its door", c.tile);
            }
        }
    }
}

fn measures(level: i32) -> Vec<(u32, Navigation)> {
    SEEDS
        .map(|seed| {
            let (l, p) = build(seed, level);
            (seed, navigation(&l, &p))
        })
        .collect()
}

#[test]
fn the_way_down_is_found_not_walked() {
    let _ = std::fs::create_dir_all("../../target/building");
    for (level, style) in [(1, Style::Well), (2, Style::Halls), (3, Style::Works)] {
        let ms = measures(level);
        let mut seen = 0;
        for (seed, m) in &ms {
            let tag = format!("{style:?} seed {seed}: {m:?}");
            // a long way round to a door you could almost touch
            assert!(m.detour >= 2.0, "{tag}: the way is too straight");
            assert!(m.path >= 40, "{tag}: too short a floor");
            // down, mostly, by flights
            assert!(m.flights >= 3, "{tag}: too few flights");
            assert!(m.descent > m.climb, "{tag}: the way should go down");
            assert!(m.high - m.low >= 12.0, "{tag}: too flat");
            // choices on the way, and places that go nowhere
            assert!(m.decisions >= 1, "{tag}: no choice on the way");
            assert!(m.dead_ends >= 2, "{tag}: too few dead ends");
            // a cell within reach of every floor
            assert!(m.cell_reach <= CELL_REACH, "{tag}: a floor too far from a cell");
            seen += usize::from(m.door_seen);
        }
        // the door's light (the lamp over it, its tower's windows) is seen
        // from near the arrival on most floors
        let want = ms.len() * 7 / 10;
        assert!(seen >= want, "{style:?}: the door seen from the arrival on {seen} of {} floors", ms.len());
        // dumps to look at
        let mut out = String::new();
        for seed in 1..=4 {
            let (l, p) = build(seed, level);
            out.push_str(&format!("seed {seed}: {:?}\n{}\n", navigation(&l, &p), ascii(&l, &p)));
        }
        std::fs::write(format!("../../target/building/{}.txt", style.key()), out).unwrap();
        let mean = |f: &dyn Fn(&Navigation) -> f64| ms.iter().map(|(_, m)| f(m)).sum::<f64>() / ms.len() as f64;
        eprintln!(
            "{style:?}: detour {:.2}, path {:.0}, flights {:.1}, descent {:.1} m, climb {:.1} m, range {:.1} m, decisions {:.1}, dead ends {:.1}, cells {:.1}, door seen {seen}/{}",
            mean(&|m| m.detour),
            mean(&|m| m.path as f64),
            mean(&|m| m.flights as f64),
            mean(&|m| m.descent),
            mean(&|m| m.climb),
            mean(&|m| m.high - m.low),
            mean(&|m| m.decisions as f64),
            mean(&|m| m.dead_ends as f64),
            mean(&|m| m.cells as f64),
            ms.len()
        );
    }
}

/// `cargo test --release -p game --test building -- --ignored --nocapture census`
/// (`CENSUS_SEEDS=200` for more than the 40 the bands are held on)
#[test]
#[ignore]
fn census() {
    let n: u32 = std::env::var("CENSUS_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(SEEDS.end - 1);
    for level in 1..=3 {
        for seed in 1..=n {
            let (l, p) = build(seed, level);
            let m = navigation(&l, &p);
            println!(
                "L{level} seed {seed:>3}: detour {:.2} path {:>3} flights {:>2} climb {:>4.0} descent {:>4.0} range {:>4.0} decisions {:>2} dead {:>2} seen {} reach {:>2} cells {:>2} walk {}",
                m.detour, m.path, m.flights, m.climb, m.descent, m.high - m.low, m.decisions, m.dead_ends, m.door_seen as u8, m.cell_reach, m.cells, m.walkable
            );
        }
    }
}

/// `SEED=… LEVEL=… cargo test --release -p game --test building -- --ignored --nocapture show_floor`
#[test]
#[ignore]
fn show_floor() {
    let seed: u32 = std::env::var("SEED").unwrap().parse().unwrap();
    let level: i32 = std::env::var("LEVEL").map(|v| v.parse().unwrap()).unwrap_or(1);
    let (l, p) = build(seed, level);
    println!("{}", ascii(&l, &p));
}

#[test]
#[ignore]
fn print_plans() {
    for seed in [4242u32, 77, 9001, 1999, 31337, 7, 12345] {
        let p = game::level::scp::scp_plan_default(seed);
        println!("{seed}: {:?}", p.entries);
    }
}
