//! The way down a floor of the Building, and what is off it
//! (`docs/MEGASTRUCTURE.md` G3, `docs/MILESTONES.md` 3): every floor, built
//! with its dressing, keeps what makes a way worth finding (choices, loops,
//! flights, the door seen, somewhere for the figure to stand) and what makes
//! leaving it worth it (a dead end holds something: water, a battery, a
//! note, a thing put away; her trail; notes, water and batteries enough).
//!
//!   cargo test --release -p backrooms --test building_floors -- --nocapture   # the audit's table
mod common;

use game::level::building::{navigation, Navigation, PLAT};
use game::Game;

const SEEDS: std::ops::RangeInclusive<u32> = 1..=24;

/// One floor, measured
#[derive(Debug, Clone)]
struct Audit {
    seed: u32,
    level: i32,
    nav: Navigation,
    /// dead ends, and those with something on them
    dead_ends: usize,
    holding: usize,
    water: usize,
    batteries: usize,
    notes: usize,
    secrets: usize,
    /// her trail's marks, and trails that are not hers
    trail: usize,
    false_trails: usize,
    /// where the figure across the void may stand
    perches: usize,
}

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = common::new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g
}

fn audit(seed: u32, level: i32) -> Audit {
    let g = floor(seed, level);
    let (l, p) = (g.layout.clone().expect("a layout"), g.building.clone().expect("a Building floor"));
    let nav = navigation(&l, &p);
    let at = |x: f64, z: f64| (g.tx(x), g.tz(z));
    let mut things: Vec<(i32, i32)> = vec![];
    things.extend(g.almonds.iter().map(|a| at(a.g.pos().x, a.g.pos().z)));
    things.extend(g.battery_items.iter().map(|a| at(a.g.pos().x, a.g.pos().z)));
    things.extend(g.lore_notes.iter().map(|n| at(n.g.pos().x, n.g.pos().z)));
    things.extend(g.secrets.list.iter().map(|s| s.plan.tile));
    let ends: Vec<_> = p.nodes.iter().filter(|n| n.degree == 1 && (n.x, n.z) != (1, 1)).collect();
    let holding =
        ends.iter().filter(|n| things.iter().any(|&(x, z)| (n.x..n.x + PLAT).contains(&x) && (n.z..n.z + PLAT).contains(&z))).count();
    Audit {
        seed,
        level,
        nav,
        dead_ends: ends.len(),
        holding,
        water: g.almonds.len(),
        batteries: g.battery_items.len(),
        notes: g.lore_notes.len(),
        secrets: g.secrets.list.len(),
        trail: g.story.trail.len(),
        false_trails: g.story.false_trails.len(),
        perches: g.figure_perches.len(),
    }
}

/// Every floor of levels 1-3 of `SEEDS`, built side by side
fn audits() -> Vec<Audit> {
    std::thread::scope(|sc| {
        let hs: Vec<_> = SEEDS
            .flat_map(|seed| (1..=3).map(move |level| (seed, level)))
            .map(|(seed, level)| std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || audit(seed, level)).unwrap())
            .collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    })
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

#[test]
fn the_way_and_what_is_off_it() {
    let all = audits();
    println!("level | way tiles | detour | flights | choices | dead ends (holding) | door seen | water | batteries | notes | secrets | her marks | false trails | perches");
    for level in 1..=3 {
        let f: Vec<&Audit> = all.iter().filter(|a| a.level == level).collect();
        let m = |g: &dyn Fn(&Audit) -> f64| median(f.iter().map(|a| g(a)).collect());
        let min = |g: &dyn Fn(&Audit) -> f64| f.iter().map(|a| g(a)).fold(f64::INFINITY, f64::min);
        println!(
            "L{level} | {} | {:.2} | {} | {} | {} ({}) | {}/{} | {} (min {}) | {} (min {}) | {} (min {}) | {} | {} (min {}) | {} | {} (min {})",
            m(&|a| a.nav.path as f64),
            m(&|a| a.nav.detour),
            m(&|a| a.nav.flights as f64),
            m(&|a| a.nav.decisions as f64),
            m(&|a| a.dead_ends as f64),
            m(&|a| a.holding as f64),
            f.iter().filter(|a| a.nav.door_seen).count(),
            f.len(),
            m(&|a| a.water as f64),
            min(&|a| a.water as f64),
            m(&|a| a.batteries as f64),
            min(&|a| a.batteries as f64),
            m(&|a| a.notes as f64),
            min(&|a| a.notes as f64),
            m(&|a| a.secrets as f64),
            m(&|a| a.trail as f64),
            min(&|a| a.trail as f64),
            m(&|a| a.false_trails as f64),
            m(&|a| a.perches as f64),
            min(&|a| a.perches as f64),
        );
    }
    let empty: Vec<String> = all
        .iter()
        .filter(|a| a.holding < a.dead_ends)
        .map(|a| format!("seed {} L{}: {} of {} dead ends hold nothing", a.seed, a.level, a.dead_ends - a.holding, a.dead_ends))
        .collect();
    println!("dead ends with nothing on them: {} floors of {}\n{}", empty.len(), all.len(), empty.join("\n"));
    // what is off the way: a dead end is worth its walk
    assert!(empty.is_empty(), "a dead end holds something: {empty:#?}");
    let mut short = vec![];
    for a in &all {
        let tag = format!("seed {} L{}", a.seed, a.level);
        // the way: a detour, flights, choices (G1's bands, held since)
        if a.nav.detour < 1.7 || a.nav.flights < 2 || a.nav.decisions < 1 {
            short.push(format!(
                "{tag}: the way is plain (detour {:.2}, {} flights, {} choices)",
                a.nav.detour, a.nav.flights, a.nav.decisions
            ));
        }
        // what lies about, her trail, somewhere for the figure
        if a.water < 2 || a.batteries < 1 || a.notes < 2 {
            short.push(format!("{tag}: {} water, {} batteries, {} notes", a.water, a.batteries, a.notes));
        }
        if a.trail < 15 {
            short.push(format!("{tag}: her trail has {} marks", a.trail));
        }
        if a.perches < 4 {
            short.push(format!("{tag}: {} places for the figure", a.perches));
        }
    }
    assert!(short.is_empty(), "{short:#?}");
    // the door is seen from near the arrival on nearly every floor
    let seen = all.iter().filter(|a| a.nav.door_seen).count();
    assert!(seen * 100 >= all.len() * 90, "the door seen from near the arrival on {seen} of {} floors", all.len());
}
