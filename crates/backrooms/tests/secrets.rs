//! Secrets (`docs/design/SECRETS.md` 4–5): where things are put away, and what
//! taking them does.
//!
//!   cargo test --release -p backrooms --test secrets -- --nocapture
use game::secrets::{run_things, Thing, RUN_FLOORS, SPECIAL};
use game::Game;

fn new_game() -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = "nosound".into();
    g.boot();
    g
}

fn level(seed: u32, level: i32) -> Game {
    let mut g = new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g
}

/// The run lays its things over its floors from its seed alone: each on as
/// many floors of the first descent as it says (fewer only where its floors
/// ran out), never on an anomaly's floor, at most two a floor; the hole on
/// three floors; the same twice.
#[test]
fn the_run_lays_its_things_over_its_floors() {
    for seed in [4242u32, 7, 1999, 77, 2026, 101, 202, 303] {
        let mut count = std::collections::BTreeMap::new();
        let mut holes = 0;
        for l in 1..=RUN_FLOORS {
            let (things, hole) = run_things(seed, l);
            assert_eq!(run_things(seed, l), (things.clone(), hole), "seed {seed} L{l}: not a function of the seed");
            assert!(things.len() <= 2, "seed {seed} L{l}: {things:?}");
            if game::secrets::whole_floor_anomaly(game::level::scp::scp_for_level(seed, l)) {
                assert!(things.is_empty() && !hole, "seed {seed} L{l}: something in the Showroom or the Borrowed House");
            }
            if l == 4 {
                assert!(things.len() <= 1, "seed {seed}: the Lobby holds {things:?}");
            }
            for t in things {
                let (lo, hi, _) = t.floors();
                assert!((lo..=hi).contains(&l), "seed {seed}: {t:?} on L{l}, outside {lo}-{hi}");
                *count.entry(t).or_insert(0) += 1;
            }
            holes += hole as usize;
        }
        for t in SPECIAL {
            let n = count.get(&t).copied().unwrap_or(0);
            assert!(n >= 1 && n <= t.floors().2, "seed {seed}: {t:?} on {n} floors");
        }
        assert_eq!(holes, 3, "seed {seed}: the hole");
        println!("seed {seed}: {count:?}");
    }
}

/// Every hiding place lies off the way (two tiles or more from her walk and
/// from the trails that are not hers, reached from the arrival, away from
/// the arrival, the door and her camp), apart from the others; a sign is
/// near its place and seen from it; a floor's count in its band; the plan a
/// function of the level.
#[test]
fn secrets_lie_off_the_way() {
    let mut kinds = std::collections::BTreeMap::new();
    let mut things = 0;
    for (seed, l) in [(4242u32, 1), (4242, 2), (4242, 3), (4242, 4), (4242, 6), (7, 5), (7, 6), (1999, 8), (77, 10), (2026, 12), (101, 14)]
    {
        let g = level(seed, l);
        let plan = g.plan_secrets();
        assert_eq!(plan, level(seed, l).plan_secrets(), "seed {seed} L{l}: the plan is not a function of the level");
        let (want, hole) = run_things(seed, l);
        let n = plan.len();
        if l == 4 {
            assert!(plan.iter().all(|p| p.thing == Thing::PaleMushrooms), "seed {seed}: the Lobby holds more than what grows");
        }
        let band = match l {
            _ if game::secrets::whole_floor_anomaly(g.level_scp.as_deref()) => 0..=0,
            ..=3 => 2..=4,
            4 => 0..=2,
            _ => 2..=5,
        };
        assert!(band.contains(&n), "seed {seed} L{l}: {n} places");
        let spawn = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
        let ds = g.story_walk_dist(spawn);
        for p in &plan {
            let t = p.tile;
            let near = |w: &(i32, i32)| (w.0 - t.0).abs() <= 1 && (w.1 - t.1).abs() <= 1;
            assert!(!g.story.walk.iter().any(near), "seed {seed} L{l}: a {:?} by her walk", p.kind);
            assert!(!g.story.false_trails.iter().any(|f| f.walk.iter().any(near)), "seed {seed} L{l}: a {:?} on a trail not hers", p.kind);
            assert!(ds[g.t_idx(t.0, t.1)] >= 0, "seed {seed} L{l}: a {:?} out of reach", p.kind);
            for q in &plan {
                if q != p {
                    assert!((q.tile.0 - t.0).abs() + (q.tile.1 - t.1).abs() >= 4, "seed {seed} L{l}: two places together");
                }
            }
            if let Some((x, z, _, n)) = p.sign {
                let (cx, cz) = (g.wx(t.0), g.wz(t.1));
                assert!((x - cx).hypot(z - cz) < 2.5 * game::prelude::CELL, "seed {seed} L{l}: a sign far from its place");
                assert!(g.los(cx, cz, x + n.0 * 0.3, z + n.1 * 0.3), "seed {seed} L{l}: a sign its place cannot see");
            }
            *kinds.entry(p.kind.name()).or_insert(0) += 1;
            things += p.thing.special() as usize;
        }
        let placed: Vec<Thing> = plan.iter().map(|p| p.thing).filter(|t| t.special()).collect();
        println!(
            "seed {seed} L{l}: {n} places {:?}; things {placed:?} of {want:?}, hole {hole}",
            plan.iter().map(|p| p.kind.name()).collect::<Vec<_>>()
        );
    }
    println!("{kinds:?}; {things} things");
    for k in ["tile", "vent", "hole", "radiator", "cell", "corner"] {
        assert!(kinds.contains_key(k), "no {k} placed anywhere");
    }
}

/// What is taken stays taken: a thing eaten is gone from its place, the
/// notebook keeps it (what and where), and a retry or a Continue finds the
/// place opened and empty; the run's list and the places' state are saved.
#[test]
fn what_is_taken_stays_taken() {
    // a floor with a thing put away
    let (seed, l, i) = (4242u32..4300)
        .flat_map(|s| (5..=12).map(move |l| (s, l)))
        .find_map(|(s, l)| {
            let (things, _) = run_things(s, l);
            (!things.is_empty()).then_some((s, l))
        })
        .map(|(s, l)| {
            let g = level(s, l);
            let i = g.secrets.list.iter().position(|x| x.plan.thing.special());
            (s, l, i)
        })
        .unwrap();
    let Some(i) = i else { panic!("seed {seed} L{l}: its thing found no place") };
    let mut g = level(seed, l);
    let kind = g.secrets.list[i].plan.kind;
    let thing = g.secrets.list[i].plan.thing;
    if kind.closed() {
        g.open_secret(i);
    }
    assert!(g.secrets.list[i].thing.is_some(), "{kind:?}: the {thing:?} does not show");
    g.take_secret(i);
    assert!(g.secrets.list[i].emptied || thing == Thing::Sweets);
    assert_eq!(g.took.len(), 1);
    assert_eq!(g.took[0].level, l);
    let page = g.what_i_took_page();
    assert!(page.contains(&format!("Level {l}")), "{page}");
    // a retry: the same level, fitted to the run
    let key = g.secrets.list[i].key.clone();
    g.init_level(l);
    g.fit_secrets();
    let s = g.secrets.list.iter().find(|s| s.key == key).unwrap();
    assert!(s.opened && (s.emptied || thing == Thing::Sweets), "a retry put the {thing:?} back");
    // saved and loaded
    g.save_game();
    let mut h = new_game();
    h.storage = g.storage.clone();
    let d = h.load_game().expect("a save");
    h.apply_saved(&d);
    assert_eq!(h.took, g.took);
    assert_eq!(h.secrets_taken, g.secrets_taken);
}

/// A floor of `seed`'s run whose plan has a place of `kind` (holding `thing`, if given)
fn find(kind: game::secrets::PlaceKind, thing: Option<Thing>) -> (Game, usize) {
    for seed in [4242u32, 7, 1999, 77, 2026, 101, 202, 303, 5, 6, 8, 9] {
        for l in 1..=13 {
            let (things, hole) = run_things(seed, l);
            let wanted = match (kind, thing) {
                (game::secrets::PlaceKind::Hole, _) => hole,
                (_, Some(t)) => things.contains(&t),
                _ => true,
            };
            if !wanted {
                continue;
            }
            let g = level(seed, l);
            if let Some(i) = g.secrets.list.iter().position(|s| s.plan.kind == kind && thing.is_none_or(|t| s.plan.thing == t)) {
                return (g, i);
            }
        }
    }
    panic!("no {kind:?} with {thing:?} on the floors tried");
}

/// The hole in the wall gives back what you last used up and takes one
/// thing you carry; with nothing on you it takes from you (the stamina
/// ceiling, lower for the run); once a hole.
#[test]
fn the_hole_gives_back_and_takes() {
    let (mut g, i) = find(game::secrets::PlaceKind::Hole, None);
    g.player.bottles = 0;
    g.player.batteries = 1;
    g.player.papers = 0;
    g.player.coins = 0;
    g.last_used = Some(Thing::Almond);
    g.open_secret(i);
    assert_eq!((g.player.bottles, g.player.batteries), (1, 0), "it gave back the bottle and took the battery");
    assert!(g.secrets.list[i].emptied, "once a hole");
    assert_eq!(g.took.last().map(|t| t.what.as_str()), Some("hole"));
    let max = g.body.stam_max;
    // the next floor's hole, with nothing on you (the bottle you drank given
    // back all the same): it takes from you, and you feel it
    let (mut h, j) = find(game::secrets::PlaceKind::Hole, None);
    (h.player.bottles, h.player.batteries, h.player.papers, h.player.coins) = (0, 0, 0, 0);
    h.last_used = Some(Thing::Almond);
    h.open_secret(j);
    assert_eq!(h.took.last().map(|t| t.what.as_str()), Some("hole_took"));
    assert!(h.body.stam_max < max, "it took nothing from you: ceiling {}", h.body.stam_max);
    assert_eq!(h.player.bottles, 1, "it gave back the bottle");
    assert!(h.msg_text.contains("TOUCHES YOUR WRIST"), "not felt: {:?}", h.msg_text);
    // your asides were told (weighed or owed)
    assert!(
        h.asides.memory.weighed.iter().any(|w| w == "uncanny:hole") || h.asides.memory.owed.contains_key("uncanny:hole"),
        "{:?}",
        h.asides.memory
    );
}

/// The dish: a sweet is taken and eaten, the dish stays; the third is the
/// last it gives, and the run knows it was a third.
#[test]
fn the_dish_and_its_third_sweet() {
    let (mut g, i) = find(game::secrets::PlaceKind::Table, Some(Thing::Sweets));
    for k in 1..=3 {
        g.take_secret(i);
        assert_eq!(g.secrets.list[i].uses, k);
        assert_eq!(g.secrets.list[i].emptied, k == 3, "after {k}");
    }
    let what: Vec<&str> = g.took.iter().map(|t| t.what.as_str()).collect();
    assert_eq!(what, ["sweet", "sweet", "third_sweet"]);
    assert!(g.body.shake);
    // and a retry: the dish empty
    let key = g.secrets.list[i].key.clone();
    let l = g.level_num;
    g.init_level(l);
    g.fit_secrets();
    assert!(g.secrets.list.iter().find(|s| s.key == key).unwrap().emptied);
}

/// The sign is taught safely: on the run's first floor, the first place
/// that is opened is found open and empty, its sign crossed out.
#[test]
fn the_first_sign_is_crossed_out() {
    for seed in [4242u32, 7, 1999, 77, 2026] {
        let g = level(seed, 1);
        let crossed: Vec<_> = g.secrets.list.iter().filter(|s| s.plan.crossed).collect();
        assert!(crossed.len() <= 1, "seed {seed}: {} crossed out", crossed.len());
        for s in crossed {
            assert!(s.opened && s.emptied && s.sign.is_some(), "seed {seed}: crossed but not opened and empty");
            assert_eq!(s.plan.thing, Thing::Nothing);
        }
        println!(
            "seed {seed} L1: {:?}",
            g.secrets.list.iter().map(|s| (s.plan.kind.name(), s.plan.crossed, s.plan.sign.is_some())).collect::<Vec<_>>()
        );
    }
}

fn took(ids: &[&str]) -> Vec<game::secrets::keeps::Took> {
    ids.iter().map(|w| game::secrets::keeps::Took { what: w.to_string(), level: 6, place: "corner".into() }).collect()
}

/// Each thing moves what it says it does, by its step, and nothing else
/// (`docs/design/SECRETS.md` 5): read from the run's list alone.
#[test]
fn each_thing_does_what_it_says() {
    use game::secrets::keeps::{body_of, Body, SIZE_EYE};
    let base = Body::default();
    let one = |id: &str| body_of(&took(&[id]));
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
    let pale = one("pale_mushrooms");
    assert!(pale.size == -1 && close(pale.eye, -SIZE_EYE) && close(pale.steps, 0.88) && close(pale.pace, 0.96));
    let brown = one("brown_mushrooms");
    assert!(brown.size == 1 && close(brown.eye, SIZE_EYE) && close(brown.steps, 1.12) && close(brown.pace, 1.04));
    assert_eq!(body_of(&took(&["pale_mushrooms", "brown_mushrooms"])), base, "one of each: your own size");
    assert_eq!(body_of(&took(&["pale_mushrooms"; 3])).size, -2, "two steps at most");
    assert!(close(one("sweet").nerve, 1.1) && close(body_of(&took(&["sweet", "sweet"])).nerve, 1.2));
    let third = body_of(&took(&["sweet", "sweet", "third_sweet"]));
    assert!(third.shake && close(third.lamp_seen, 1.2) && close(third.nerve, 1.2));
    let seeds = one("seeds");
    assert!(close(seeds.dark, 0.8) && close(seeds.lamp_seen, 0.85) && close(seeds.door, 0.8));
    let bag = one("bag");
    assert!(close(bag.dark, 0.75) && bag.dark_floor == game::player::DARK_FLOOR + 5.0);
    assert!(close(one("salt").touch, 0.8));
    assert!(close(one("jelly").steps, 0.85));
    let ration = one("ration");
    assert!(ration.stam_max == game::prelude::STAM_MAX + 12.0 && close(ration.drink, 0.8));
    let medicine = one("medicine");
    assert!(close(medicine.nerve, 1.2) && close(medicine.wind, 0.9));
    let cap = one("cap");
    assert!(close(cap.still, 1.25) && close(cap.cry_mul, 0.5));
    let coin = one("coin");
    assert!(close(coin.cry_add, 0.15) && coin.no_coin_return);
    assert!(close(one("yellow_water").still, 1.0 / 1.2));
    assert!(close(one("envelope").look, 1.25));
    assert!(one("hole_took").stam_max == game::prelude::STAM_MAX - 15.0);
    assert_eq!(one("hole"), base, "the hole's exchange itself leaves you as you were");
    // and each thing changes only its own (every field but those, as it was)
    for t in SPECIAL {
        let b = one(t.id());
        assert_ne!(b, base, "{t:?} does nothing");
    }
}

/// Whatever is taken, and however often, it stays within its bounds: each
/// system two steps at most, and never what the creatures' fairness rests
/// on (the dark leaves you at least what it did, a touch takes at most what
/// it did: E10), and no thing kills.
#[test]
fn what_stays_with_you_is_bounded() {
    use game::secrets::keeps::body_of;
    let ids: Vec<&str> = SPECIAL.iter().map(|t| t.id()).chain(["third_sweet", "hole_took"]).collect();
    // every thing three times over, in turn, and all of them together
    let mut lists: Vec<Vec<&str>> = ids.iter().map(|&i| vec![i; 3]).collect();
    lists.push(ids.iter().flat_map(|&i| [i, i, i]).collect());
    lists.push(ids.clone());
    for l in lists {
        let b = body_of(&took(&l));
        let w = format!("{l:?}");
        assert!((-2..=2).contains(&b.size), "{w}: size {}", b.size);
        assert!(b.dark_floor >= game::player::DARK_FLOOR, "{w}: the dark leaves {}", b.dark_floor);
        assert!(b.touch <= 1.0 && b.touch >= 0.6, "{w}: a touch takes ×{}", b.touch);
        // each system two steps at most, a step the smallest a thing moves it
        assert!(b.dark >= 0.6 && b.dark <= 1.0, "{w}: the dark ×{}", b.dark);
        assert!(b.stam_max >= 85.0 && b.stam_max <= 112.0, "{w}: stamina {}", b.stam_max);
        assert!((0.76..=1.24).contains(&b.steps), "{w}: steps ×{}", b.steps);
        assert!((0.92..=1.08).contains(&b.pace), "{w}: pace ×{}", b.pace);
        assert!(b.nerve >= 1.0 && b.nerve <= 1.2, "{w}: nerve ×{}", b.nerve);
        assert!((0.7..=1.2).contains(&b.lamp_seen), "{w}: a Smiler sees your lamp ×{}", b.lamp_seen);
        assert!(b.look <= 1.5 && b.door >= 0.6, "{w}");
        // (the chance itself is capped at 60% where a cry is answered: `cry_for_help`)
        assert!(b.cry_mul <= 1.0 && b.cry_add <= 0.15, "{w}");
    }
}

/// On the Building's floors nothing walks: a cry there is never answered
/// by footprints (the lights, or the door's own call).
#[test]
fn on_the_buildings_floors_nothing_walks() {
    for seed in [4242u32, 7, 1999] {
        for l in 1..=3 {
            let g = level(seed, l);
            let a = g.way_answer_here();
            assert!(a.is_some(), "seed {seed} L{l}: no way to the door");
            assert_ne!(a, Some(game::base::records::WayAnswer::Prints), "seed {seed} L{l}: footprints on the Building's floors");
        }
    }
}
