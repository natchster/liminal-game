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

/// The places off the walks (`docs/MEGASTRUCTURE.md` G4): each room is
/// reached from the arrival, closed on its three other sides by building a
/// storey high, off the way to the door (its walk is not on the way: never a
/// room only the way passes), and holds something
#[test]
fn the_rooms_off_the_walks() {
    let floors: Vec<(i32, usize, Vec<String>)> = std::thread::scope(|sc| {
        let hs: Vec<_> = SEEDS
            .flat_map(|seed| (1..=3).map(move |level| (seed, level)))
            .map(|(seed, level)| std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || rooms_of(seed, level)).unwrap())
            .collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    });
    let mut wrong = vec![];
    for level in 1..=3 {
        let mut c: Vec<usize> = floors.iter().filter(|f| f.0 == level).map(|f| f.1).collect();
        c.sort();
        println!("L{level}: rooms a floor, median {} (least {}, most {})", c[c.len() / 2], c[0], c[c.len() - 1]);
        // the Stair Halls are flights between piers, and the Works' walks
        // run between machines: few walks there are floor beside building
        // closed on three sides (the Well's ring of building has many)
        let least = if level == 1 { 2 } else { 1 };
        let none = c.iter().filter(|&&n| n == 0).count();
        println!("L{level}: {none} of {} floors without a room", c.len());
        assert!(c[c.len() / 2] >= least, "L{level}: a floor has at least {least} rooms off its walks, mostly");
    }
    for f in &floors {
        wrong.extend(f.2.iter().cloned());
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A floor's rooms: how many, and what is wrong with each
fn rooms_of(seed: u32, level: i32) -> (i32, usize, Vec<String>) {
    use game::level::archgen::{DIRS, GROUND_MASS};
    use game::level::building::{walk_from, ROOM_HEADROOM};
    let g = floor(seed, level);
    let (l, p) = (g.layout.as_ref().unwrap(), g.building.as_ref().unwrap());
    let idx = |x: i32, z: i32| (z * l.w + x) as usize;
    let ds = walk_from(l, &[p.spawn]);
    let de = walk_from(l, &[p.exit]);
    let path = de[idx(p.spawn.0, p.spawn.1)];
    let at = |x: f64, z: f64| (g.tx(x), g.tz(z));
    let things: Vec<(i32, i32)> = g
        .almonds
        .iter()
        .chain(&g.battery_items)
        .map(|a| at(a.g.pos().x, a.g.pos().z))
        .chain(g.lore_notes.iter().map(|n| at(n.g.pos().x, n.g.pos().z)))
        .collect();
    let mut wrong = vec![];
    for r in &p.rooms {
        let (x, z) = r.tile;
        let tag = format!("seed {seed} L{level} room {x},{z}");
        if ds[idx(x, z)] < 0 {
            wrong.push(format!("{tag}: not reached from the arrival"));
        }
        // (the room is a dead end, so never on the way itself: its walk is
        // what may not be)
        let (wx, wz) = (x + DIRS[r.door][0], z + DIRS[r.door][1]);
        if ds[idx(wx, wz)] + de[idx(wx, wz)] == path {
            wrong.push(format!("{tag}: its walk {wx},{wz} is on the way"));
        }
        let floor_h = l.ground[idx(x, z)].h0;
        for (k, dd) in DIRS.iter().enumerate() {
            let n = l.ground[idx(x + dd[0], z + dd[1])];
            if k != r.door && n.kind != GROUND_MASS {
                wrong.push(format!("{tag}: open on side {k}"));
            }
            if k != r.door && n.h1 < floor_h + ROOM_HEADROOM {
                wrong.push(format!("{tag}: side {k} stops {:.1} m over its floor", n.h1 - floor_h));
            }
        }
        if !things.contains(&(x, z)) {
            wrong.push(format!("{tag}: holds nothing"));
        }
    }
    (level, p.rooms.len(), wrong)
}

/// A room's front stops a body (`level::building::cell_front_boxes`): it
/// cannot walk in through the wall either side of the doorway, and walks in
/// through the doorway (`ROOM_DOOR` wide) unhindered
#[test]
fn a_rooms_front_stops_a_body_beside_its_doorway() {
    let found: Vec<(usize, Vec<String>)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=8u32)
            .flat_map(|seed| (1..=3).map(move |level| (seed, level)))
            .map(|(seed, level)| std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || fronts_of(seed, level)).unwrap())
            .collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    });
    let rooms: usize = found.iter().map(|f| f.0).sum();
    assert!(rooms >= 8, "rooms enough to try ({rooms})");
    let wrong: Vec<String> = found.into_iter().flat_map(|f| f.1).collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A floor's rooms walked into from their walks, beside the doorway and
/// through it: how many, and where a body went through a wall or was stopped
/// in the doorway
fn fronts_of(seed: u32, level: i32) -> (usize, Vec<String>) {
    use game::base::config::PLAYER_R;
    use game::level::archgen::DIRS;
    use game::level::building::ROOM_DOOR;
    let g = floor(seed, level);
    let p = g.building.as_ref().unwrap();
    let cell = g.layout.as_ref().unwrap().cell;
    let mut wrong = vec![];
    for r in &p.rooms {
        let (x, z) = r.tile;
        let (dx, dz) = (DIRS[r.door][0] as f64, DIRS[r.door][1] as f64);
        // the front's face, and along it to the right seen from the walk
        let (fx, fz) = (g.wx(x) + dx * cell / 2.0, g.wz(z) + dz * cell / 2.0);
        let (rx, rz) = (-dz, dx);
        // from just out on the walk to the middle of the room, `along` the face
        let stopped = |along: f64| {
            (0..=52).any(|k| {
                let out = 0.6 - k as f64 * 0.05;
                g.circle_hits_wall(fx + dx * out + rx * along, fz + dz * out + rz * along, PLAYER_R)
            })
        };
        let tag = format!("seed {seed} L{level} room {x},{z}");
        // the wall either side: from the doorway's edge to the tile's
        let beside = ROOM_DOOR / 2.0 + (cell - ROOM_DOOR) / 4.0;
        for along in [-beside, beside] {
            if !stopped(along) {
                wrong.push(format!("{tag}: a body walks in through the front {along:+.1} m along it"));
            }
        }
        if stopped(0.0) {
            wrong.push(format!("{tag}: a body is stopped in the doorway"));
        }
    }
    (p.rooms.len(), wrong)
}

/// Two of the Building's small faults (`docs/MILESTONES.md` 11) kept mended:
/// on a floor whose lamps stand on posts (the Stair Halls, the Works) no two
/// lamp heads stand in one place (a flight's foot lamp was put a hand's
/// breadth from the lamp over a cell's door beside it, two heads pushed into
/// the corner of the building); and a room off a walk, a recess with a
/// storey of building over it, has less of the place's light than the walk
/// before its doorway (it had the same, and read lit from nowhere)
#[test]
fn the_lamps_stand_apart_and_the_rooms_are_darker_than_their_walks() {
    let found: Vec<(usize, Vec<String>)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=8u32)
            .flat_map(|seed| (1..=3).map(move |level| (seed, level)))
            .map(|(seed, level)| {
                std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || lamps_and_rooms(seed, level)).unwrap()
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    });
    let rooms: usize = found.iter().map(|f| f.0).sum();
    assert!(rooms >= 8, "rooms enough to try ({rooms})");
    let wrong: Vec<String> = found.into_iter().flat_map(|f| f.1).collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

fn lamps_and_rooms(seed: u32, level: i32) -> (usize, Vec<String>) {
    use game::level::archgen::DIRS;
    use game::level::building::Style;
    let g = floor(seed, level);
    let p = g.building.as_ref().unwrap();
    let mut wrong = vec![];
    if p.style != Style::Well {
        for (i, a) in g.panels.iter().enumerate() {
            for b in &g.panels[i + 1..] {
                let d = ((a.wx - b.wx).powi(2) + (a.wz - b.wz).powi(2) + (a.y - b.y).powi(2)).sqrt();
                if d < 0.6 {
                    wrong.push(format!("seed {seed} L{level}: two lamps {d:.2} m apart at ({:.2}, {:.2}, {:.2})", a.wx, a.y, a.wz));
                }
            }
        }
    }
    let cell = g.layout.as_ref().unwrap().cell;
    for r in &p.rooms {
        let (x, z) = r.tile;
        let (dx, dz) = (DIRS[r.door][0] as f64, DIRS[r.door][1] as f64);
        let room = g.light_map_at(g.wx(x), g.wz(z));
        let walk = g.light_map_at(g.wx(x) + dx * cell, g.wz(z) + dz * cell);
        if room >= walk * 0.9 {
            wrong.push(format!("seed {seed} L{level} room {x},{z}: lit {room:.3}, its walk {walk:.3}"));
        }
    }
    (p.rooms.len(), wrong)
}
