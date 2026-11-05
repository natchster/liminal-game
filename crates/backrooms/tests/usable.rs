//! Things a hand can use (`docs/PROPS.md` 3): each verb is where its prop
//! is, does what a hand does, once, and gives what the floor fixed for it.
//!
//!   cargo test --release -p backrooms --test usable -- --nocapture
use game::Game;

fn register_placers(r: &mut Vec<(&'static str, game::base::state::PlacerFn)>) {
    placers_p00::register(r);
    placers_p01::register(r);
    placers_p02::register(r);
    placers_p03::register(r);
    placers_p04::register(r);
    placers_p05::register(r);
    placers_p06::register(r);
    placers_p07::register(r);
    placers_p08::register(r);
    placers_p09::register(r);
    placers_p10::register(r);
    placers_p11::register(r);
    placers_p12::register(r);
    placers_p13::register(r);
    placers_p14::register(r);
    placers_p15::register(r);
    placers_p16::register(r);
    placers_p17::register(r);
    placers_p18::register(r);
    placers_p19::register(r);
    placers_p20::register(r);
    placers_p21::register(r);
    placers_p22::register(r);
    placers_p23::register(r);
    placers_p24::register(r);
    placers_p25::register(r);
    placers_p26::register(r);
    placers_p27::register(r);
    placers_p28::register(r);
    placers_p29::register(r);
    placers_p30::register(r);
    placers_p31::register(r);
    placers_p32::register(r);
}

fn new_game() -> Game {
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = "nosound".into();
    g.boot();
    g
}

fn floor(seed: u32, l: i32) -> Game {
    // the per-frame stream from the floor, not the clock: a creature's rough
    // guess at a sound (an angle and a metre or three) repeats run to run
    game::base::rng::seed_native((u64::from(seed) << 8) | l as u64);
    let mut g = new_game();
    g.run_seed = seed;
    g.init_level(l);
    g.state = "playing".into();
    g
}

fn kinds(g: &Game, kind: &str) -> usize {
    g.world.children().iter().filter(|o| o.user().s.get("kind").is_some_and(|k| k == kind)).count()
}

fn verbs(g: &Game, prompt: &str) -> Vec<usize> {
    (0..g.interactables.len()).filter(|&i| g.interactables[i].prompt == prompt).collect()
}

/// Use interactable `i` once
fn use_it(g: &mut Game, i: usize) {
    let mut f = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
    f(g);
    g.interactables[i].on_use = f;
}

const SEEDS: [u32; 4] = [4242, 77, 202, 7];

/// Every box stack, bin and book stack of a floor of the rooms has its verb,
/// and the floor's TV its switch; none on the Building, the Lobby or a
/// floor with rules of its own
#[test]
fn every_usable_prop_has_its_verb() {
    let mut seen = 0;
    for &seed in &SEEDS {
        for l in 1..=10 {
            let g = floor(seed, l);
            let plain = l > 4 && g.level_scp.is_none();
            for (kind, prompt) in
                [("cardboard_stack", "[E] OPEN THE BOX"), ("trash_can", "[E] LOOK INSIDE"), ("book_stack", "[E] TEAR OUT PAGES")]
            {
                let want = if plain { kinds(&g, kind) } else { 0 };
                assert_eq!(
                    verbs(&g, prompt).len(),
                    want,
                    "seed {seed} L{l}: {prompt} on {} of {}",
                    verbs(&g, prompt).len(),
                    kinds(&g, kind)
                );
                seen += want;
            }
            let tv = if plain && g.tv_data.is_some() { 1 } else { 0 };
            assert_eq!(verbs(&g, "[E] SWITCH IT OFF").len(), tv, "seed {seed} L{l}: the TV's switch");
            // every dead lift's button; a locker only where a unit is shut
            let dead = g.world.children().iter().filter(|o| o.user().s.contains_key("dead_lift")).count();
            assert_eq!(verbs(&g, "[E] PRESS THE BUTTON").len(), if plain { dead } else { 0 }, "seed {seed} L{l}: the dead lifts' buttons");
            let shut: usize = g
                .world
                .children()
                .iter()
                .filter(|o| o.user().s.get("kind").is_some_and(|k| k == "locker"))
                .map(|o| o.user().o.keys().filter(|k| k.starts_with("door")).count())
                .sum();
            let lockers = verbs(&g, "[E] OPEN THE LOCKER").len();
            assert!(lockers <= shut && (plain || lockers == 0), "seed {seed} L{l}: {lockers} lockers to open of {shut} shut");
            seen += lockers + dead;
        }
    }
    assert!(seen > 30, "only {seen} verbs");
}

/// Each is used once, and gives what the floor fixed for it: the same on
/// the same floor built again, whenever it is used
#[test]
fn what_they_give_is_fixed_and_given_once() {
    for &seed in &SEEDS {
        for l in [5, 6, 7, 9] {
            let mut gains = vec![];
            for _ in 0..2 {
                let mut g = floor(seed, l);
                let mut got = vec![];
                for prompt in ["[E] OPEN THE BOX", "[E] LOOK INSIDE", "[E] TEAR OUT PAGES", "[E] OPEN THE LOCKER"] {
                    for i in verbs(&g, prompt) {
                        let before = g.player.papers + g.player.bottles;
                        assert!((g.interactables[i].enabled)(&g), "seed {seed} L{l}: {prompt} not offered");
                        use_it(&mut g, i);
                        got.push(g.player.papers + g.player.bottles - before);
                        assert!(!(g.interactables[i].enabled)(&g), "seed {seed} L{l}: {prompt} offered again");
                    }
                }
                gains.push(got);
            }
            assert_eq!(gains[0], gains[1], "seed {seed} L{l}: what the props give changed between two builds");
            // a book always gives pages; a box or a bin now and then
            let g = floor(seed, l);
            let books = verbs(&g, "[E] TEAR OUT PAGES").len();
            let least = if l >= 9 { 1 } else { 3 };
            assert!(gains[0].iter().filter(|&&n| n >= least).count() >= books, "seed {seed} L{l}: a book without pages");
            // a locker opened swings its door: its leaf turned once the animators run
            let mut g = floor(seed, l);
            if let Some(&i) = verbs(&g, "[E] OPEN THE LOCKER").first() {
                use_it(&mut g, i);
                for _ in 0..60 {
                    g.run_animators(1.0 / 60.0, 0.0);
                }
                let turned = g
                    .world
                    .children()
                    .iter()
                    .filter(|o| o.user().s.get("kind").is_some_and(|k| k == "locker"))
                    .any(|o| o.children().iter().any(|c| c.rot().y < -1.5));
                assert!(turned, "seed {seed} L{l}: the locker's door did not swing open");
            }
        }
    }
}

/// The TV switched off goes dark (its light out, its static stopped) and
/// comes back when switched on
#[test]
fn the_tv_switches_off_and_on() {
    let mut found = false;
    'out: for &seed in &SEEDS {
        for l in 5..=12 {
            let mut g = floor(seed, l);
            let (Some(off), Some(on)) = (verbs(&g, "[E] SWITCH IT OFF").first().copied(), verbs(&g, "[E] SWITCH IT ON").first().copied())
            else {
                continue;
            };
            assert!((g.interactables[off].enabled)(&g) && !(g.interactables[on].enabled)(&g));
            use_it(&mut g, off);
            let tv = g.tv_data.clone().unwrap();
            assert!(!tv.on && tv.light.intensity() == 0.0, "seed {seed} L{l}: the TV still lit");
            assert!(!(g.interactables[off].enabled)(&g) && (g.interactables[on].enabled)(&g));
            use_it(&mut g, on);
            assert!(g.tv_data.as_ref().unwrap().on);
            found = true;
            break 'out;
        }
    }
    assert!(found, "no TV on any floor tried");
}

/// A Hound standing near interactable `i` (an open point `d` m off it),
/// roaming, or listening with `listening`
fn hound_near(g: &mut Game, i: usize, d: f64, listening: bool) -> usize {
    use game::entities::brain::Mode;
    let p = g.interactables[i].pos;
    let (x, z) = [(d, 0.0), (-d, 0.0), (0.0, d), (0.0, -d)]
        .into_iter()
        .map(|(dx, dz)| g.open_point(p.x + dx, p.z + dz))
        .find(|&(x, z)| g.los(x, z, p.x, p.z))
        .unwrap_or_else(|| g.open_point(p.x + d, p.z));
    let ei = g.add_entity("hound", g.tx(x), g.tz(z), 0);
    g.entities[ei].g.set_pos(x, 0.0, z);
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.brain.set(if listening { Mode::Alert } else { Mode::Roam }, "the test sets it", now);
    c.brain.mem.awareness = 0.0;
    c.brain.mem.heard = None;
    ei
}

/// The tape and a locker's door stop a creature near them, as a machine's
/// bang does; a page torn out stops nothing, but a Hound crouched to listen
/// goes to look near it (told roughly where, never sent)
#[test]
fn a_loud_verb_stops_what_is_near_and_a_quiet_one_does_not() {
    use game::entities::brain::Mode;
    let (mut loud, mut quiet) = (false, false);
    for &seed in &SEEDS {
        for l in [5, 6, 7] {
            let g0 = floor(seed, l);
            if let Some(&i) = verbs(&g0, "[E] OPEN THE BOX").first() {
                let mut g = floor(seed, l);
                let ei = hound_near(&mut g, i, 2.0, false);
                use_it(&mut g, i);
                g.update_creature(ei, 1.0 / 60.0);
                assert_eq!(g.creature(ei).unwrap().brain.mode, Mode::Alert, "seed {seed} L{l}: the tape did not stop a hound 2 m off");
                loud = true;
            }
            if let Some(&i) = verbs(&g0, "[E] TEAR OUT PAGES").first() {
                let mut g = floor(seed, l);
                let ei = hound_near(&mut g, i, 2.0, false);
                use_it(&mut g, i);
                g.update_creature(ei, 1.0 / 60.0);
                assert_ne!(g.creature(ei).unwrap().brain.mode, Mode::Alert, "seed {seed} L{l}: a page torn out stopped a roaming hound");
                // one crouched to listen goes to look near it once all is still
                let mut g = floor(seed, l);
                let ei = hound_near(&mut g, i, 2.0, true);
                let at = g.interactables[i].pos;
                use_it(&mut g, i);
                let mut went = false;
                for _ in 0..360 {
                    g.perf_t += 1.0 / 60.0;
                    g.update_creature(ei, 1.0 / 60.0);
                    let c = g.creature(ei).unwrap();
                    if c.brain.mode == Mode::Search {
                        let (x, z, _) = c.brain.mem.heard.expect("where it heard");
                        assert!(
                            (x - at.x).hypot(z - at.z) < 4.5,
                            "seed {seed} L{l}: it went to look {:.1} m from the page",
                            (x - at.x).hypot(z - at.z)
                        );
                        went = true;
                        break;
                    }
                }
                assert!(went, "seed {seed} L{l}: a listening hound did not go to look where the page was torn");
                quiet = true;
            }
        }
    }
    assert!(loud && quiet, "no box or no book on the floors tried");
}
