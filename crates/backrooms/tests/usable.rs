//! Things a hand can use (`docs/design/PROPS.md` 3): each verb is where its prop
//! is, does what a hand does, once, and gives what the floor fixed for it;
//! and what a hand picks up (water, a battery, paper, a note) is gone once
//! taken (GitHub issue #12).
//!
//!   cargo test --release -p backrooms --test usable -- --nocapture
use engine::dmath::DMath;
use game::Game;

fn new_game() -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
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

/// `f` on every floor of `levels` of every seed, the floors side by side
/// (each on a thread of its own, built from nothing there: a floor does not
/// depend on another); what each gave, in order of seed, then level. A
/// failure in one is raised here, as the test's.
fn each_floor<T: Send>(levels: &[i32], f: impl Fn(u32, i32) -> T + Sync) -> Vec<T> {
    let f = &f;
    std::thread::scope(|sc| {
        let handles: Vec<_> = SEEDS
            .iter()
            .flat_map(|&seed| levels.iter().map(move |&l| (seed, l)))
            .map(|(seed, l)| std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || f(seed, l)).unwrap())
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    })
}

/// Every box stack, bin and book stack of a floor of the rooms has its verb,
/// and the floor's TV its switch; none on the Building, the Lobby or a
/// floor with rules of its own
#[test]
fn every_usable_prop_has_its_verb() {
    let levels: Vec<i32> = (1..=10).collect();
    let seen: usize = each_floor(&levels, |seed, l| {
        let mut seen = 0;
        let g = floor(seed, l);
        let plain = l > 4 && g.level_scp.is_none();
        for (kind, prompt) in
            [("cardboard_stack", "[E] OPEN THE BOX"), ("trash_can", "[E] LOOK INSIDE"), ("book_stack", "[E] TEAR OUT PAGES")]
        {
            let want = if plain { kinds(&g, kind) } else { 0 };
            assert_eq!(verbs(&g, prompt).len(), want, "seed {seed} L{l}: {prompt} on {} of {}", verbs(&g, prompt).len(), kinds(&g, kind));
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
        seen
    })
    .into_iter()
    .sum();
    assert!(seen > 30, "only {seen} verbs");
}

/// Each is used once, and gives what the floor fixed for it: the same on
/// the same floor built again, whenever it is used
#[test]
fn what_they_give_is_fixed_and_given_once() {
    each_floor(&[5, 6, 7, 9], |seed, l| {
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
    });
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
    let tried = each_floor(&[5, 6, 7], |seed, l| {
        let (mut loud, mut quiet) = (false, false);
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
                        (x - at.x).dhypot(z - at.z) < 4.5,
                        "seed {seed} L{l}: it went to look {:.1} m from the page",
                        (x - at.x).dhypot(z - at.z)
                    );
                    went = true;
                    break;
                }
            }
            assert!(went, "seed {seed} L{l}: a listening hound did not go to look where the page was torn");
            quiet = true;
        }
        (loud, quiet)
    });
    let (loud, quiet) = (tried.iter().any(|t| t.0), tried.iter().any(|t| t.1));
    assert!(loud && quiet, "no box or no book on the floors tried");
}

/// `secs` of play with the creatures held still (what is picked up is
/// picked up in play: `update_player`, the props' cull a few times a
/// second)
fn step(g: &mut Game, secs: f64) {
    for _ in 0..(secs * 60.0).round() as usize {
        for e in g.entities.iter_mut() {
            e.freeze = 1e9;
        }
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
    }
}

/// Whether anything of `o` is drawn: a mesh under it with itself and every
/// parent visible and not culled
fn drawn(o: &engine::scene::Obj) -> bool {
    let up = |m: &engine::scene::Obj| {
        let mut c = Some(m.clone());
        while let Some(x) = c {
            if !x.drawn() {
                return false;
            }
            c = x.parent();
        }
        true
    };
    let mut any = false;
    o.traverse(&mut |m| any |= m.is_mesh() && up(m));
    any
}

/// A pickup of the floor: its kind, its object, whether it is taken
fn pickups(g: &Game) -> Vec<(&'static str, engine::scene::Obj, bool)> {
    let mut out = vec![];
    out.extend(g.almonds.iter().map(|p| ("water", p.g.clone(), p.taken)));
    out.extend(g.battery_items.iter().map(|p| ("battery", p.g.clone(), p.taken)));
    out.extend(g.papers.iter().map(|p| ("paper", p.g.clone(), p.taken)));
    out.extend(g.lore_notes.iter().map(|n| ("note", n.g.clone(), n.taken)));
    out
}

/// What is picked up is gone (GitHub issue #12): stood on, every bottle,
/// battery, stack of paper and note of a floor (and what its drawers give)
/// is taken once, given once, and nothing of it is drawn again, nor its
/// contact shadow, nor her page or a hiding place's thing once taken,
/// however near you stay (the props' cull, which hides
/// what is far, showed what was taken again within a fifth of a second)
#[test]
fn what_is_picked_up_is_gone() {
    let levels: Vec<i32> = (1..=12).collect();
    let taken: (usize, usize) = each_floor(&levels, |seed, l| {
        let mut g = floor(seed, l);
        g.god_mode = true;
        step(&mut g, 0.3);
        // what the drawers give, out where a hand reaches it
        for i in verbs(&g, "[E] OPEN DRAWER") {
            use_it(&mut g, i);
        }
        let all = pickups(&g);
        assert!(all.iter().all(|p| !p.2), "seed {seed} L{l}: a pickup taken before you came");
        let had = (g.player.bottles, g.player.batteries, g.player.papers, g.lore_found.len());
        // stood on, each still lying there (a cabinet's drawers give theirs
        // in one place: one stand takes them all)
        for (k, (kind, o, _)) in all.iter().enumerate() {
            if pickups(&g)[k].2 {
                continue;
            }
            let at = o.world_position();
            g.player.pos.x = at.x;
            g.player.pos.z = at.z;
            g.player.pos.y = g.floor_y_at(at.x, at.z) + game::base::config::EYE;
            // long enough for the cull to look again (it does, every 0.2 s)
            step(&mut g, 0.7);
            assert!(pickups(&g)[k].2, "seed {seed} L{l}: {kind} at {:.2}, {:.2} not taken, stood on", at.x, at.z);
        }
        // her page at the camp and what the hiding places keep, taken by
        // their verbs (`take_page`, `take_secret`; a dish of sweets in
        // three), and stood beside past the cull
        let mut kept: Vec<(&str, engine::scene::Obj)> = vec![];
        if let Some(o) = g.story.camp.as_ref().filter(|c| !c.page_taken).and_then(|c| c.page.clone()) {
            g.take_page();
            kept.push(("her page", o));
        }
        for i in 0..g.secrets.list.len() {
            if let Some(o) = g.secrets.list[i].thing.clone().filter(|_| !g.secrets.list[i].emptied) {
                for _ in 0..3 {
                    g.take_secret(i);
                }
                kept.push(("a kept thing", o));
            }
        }
        for (what, o) in &kept {
            let at = o.world_position();
            g.player.pos.x = at.x;
            g.player.pos.z = at.z + 0.6;
            g.player.pos.y = g.floor_y_at(at.x, at.z + 0.6) + game::base::config::EYE;
            step(&mut g, 0.7);
            assert!(!drawn(o), "seed {seed} L{l}: {what} at {:.2}, {:.2} taken and still drawn", at.x, at.z);
        }
        // and stood on again, each: nothing more given, nothing back
        for (_, o, _) in &all {
            let at = o.world_position();
            g.player.pos.x = at.x;
            g.player.pos.z = at.z;
            step(&mut g, 0.25);
        }
        for (kind, o, _) in &all {
            let at = o.world_position();
            assert!(!drawn(o), "seed {seed} L{l}: {kind} at {:.2}, {:.2} taken and still drawn", at.x, at.z);
            let shadow = g.prop_cull.shadows.iter().flat_map(|c| c.slots.iter()).any(|s| s.owner.ptr_eq(o) && s.shown);
            assert!(!shadow, "seed {seed} L{l}: {kind} at {:.2}, {:.2} taken, its shadow left", at.x, at.z);
        }
        let of = |k: &str| all.iter().filter(|p| p.0 == k).count();
        let got = (g.player.bottles - had.0, g.player.batteries - had.1, g.player.papers - had.2, g.lore_found.len() - had.3);
        assert_eq!((got.0 as usize, got.1 as usize, got.3), (of("water"), of("battery"), of("note")), "seed {seed} L{l}: given");
        let stacks = of("paper") as i32;
        assert!((3 * stacks..=8 * stacks).contains(&got.2), "seed {seed} L{l}: {} sheets from {stacks} stacks", got.2);
        (all.len(), kept.len())
    })
    .into_iter()
    .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    assert!(taken.0 > 400 && taken.1 > 10, "only {taken:?} pickups stood on and pages or kept things taken");
}

/// The props' cull hides what is far and shows it again near, with its
/// contact shadow, on a flag of its own: what the game hid stays hidden
/// near, and what it shows again is drawn
#[test]
fn the_cull_hides_what_is_far_and_never_shows_what_was_hidden() {
    each_floor(&[5, 9], |seed, l| {
        let mut g = floor(seed, l);
        g.god_mode = true;
        step(&mut g, 0.3);
        let far = g.prop_cull.list.iter().map(|(o, _)| o.clone()).find(|o| o.culled()).expect("a prop out of reach");
        assert!(far.visible() && !drawn(&far), "seed {seed} L{l}: culled, as the cull's own");
        let at = far.world_position();
        let (px, pz) = (at.x, at.z + 0.01);
        g.player.pos.x = px;
        g.player.pos.z = pz;
        g.player.pos.y = g.floor_y_at(px, pz) + game::base::config::EYE;
        // (a tool's eye: nothing taken or fallen into while it stands there)
        g.tool_eye = true;
        step(&mut g, 0.5);
        assert!(!far.culled() && drawn(&far), "seed {seed} L{l}: near, not shown again");
        far.set_visible(false);
        step(&mut g, 0.5);
        assert!(!far.culled() && !drawn(&far), "seed {seed} L{l}: hidden by the game, shown by the cull");
        far.set_visible(true);
        step(&mut g, 0.3);
        assert!(drawn(&far), "seed {seed} L{l}: shown by the game, not drawn");
    });
}

/// What the placement rules place (`docs/design/ASSETS.md`, *Placement
/// rules*: the floor's own almond water and batteries) keeps its rule: a
/// purpose the rule names and none the asset never fits; on what the rule
/// says (a thing of a kind it names, or the floor, near a wall when it says
/// so); clear of the walls, apart from each other, as many as the rules
/// allow. And the rules name what is: every purpose a zone has, every kind a
/// floor places (each found on some floor)
#[test]
fn what_the_rules_place_keeps_its_rules() {
    use game::world::placement_rules::{self as pr, Near, On};
    let purposes = pr::purposes();
    for a in pr::all() {
        for r in &a.rules {
            assert!(r.purposes.iter().all(|p| p == "*" || purposes.contains(&p.as_str())), "{}: {:?}", a.name, r.purposes);
            assert!(r.why.trim().len() > 12, "{}: a rule without its why", a.name);
        }
        assert!(a.never.iter().all(|p| purposes.contains(&p.as_str())), "{}: never {:?}", a.name, a.never);
    }
    let levels: Vec<i32> = (4..=12).collect();
    let floors = each_floor(&levels, |seed, l| {
        let g = floor(seed, l);
        // the kinds of thing on the floor, and each one (what the rules
        // placed, apart)
        let mut things: Vec<(String, engine::scene::Obj)> = vec![];
        let mut ruled: Vec<engine::scene::Obj> = vec![];
        for o in g.world.children() {
            let kind = {
                let u = o.user();
                if u.s.get("by").is_some_and(|b| b == "rules") {
                    assert!(u.s.get("kind").and_then(|k| pr::of(k)).is_some(), "seed {seed} L{l}: marked by rules, with none: {:?}", u.s);
                    drop(u);
                    ruled.push(o.clone());
                    continue;
                }
                match (u.s.get("kind"), u.placer.as_deref()) {
                    (Some(k), _) => format!("core_{k}"),
                    (None, Some(p)) if !p.is_empty() => p.to_string(),
                    _ => continue,
                }
            };
            things.push((kind, o.clone()));
        }
        let mut at: Vec<(f64, f64, &str)> = vec![];
        let reached = g.bfs_open(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
        for a in pr::all() {
            let name = a.name.as_str();
            let mine: Vec<_> = ruled.iter().filter(|o| o.user().s.get("kind").is_some_and(|k| k == name)).collect();
            if g.building.is_some() || g.level_num < a.levels.0 {
                continue;
            }
            // as many as its rules allow: at least the least, where they allow
            // anywhere on the floor (the Showroom's, spotless, never)
            let least = if g.rule_spots(a).is_empty() { 0 } else { a.count.0 };
            assert!(mine.len() >= least && mine.len() <= a.count.1, "seed {seed} L{l} {:?}: {} {name}", g.level_scp, mine.len());
            for o in mine {
                let ri: usize = o
                    .user()
                    .s
                    .get("rule")
                    .and_then(|r| r.parse().ok())
                    .unwrap_or_else(|| panic!("seed {seed} L{l}: a {name} no rule placed"));
                let r = &a.rules[ri];
                let w = o.world_position();
                let label = g.purpose_label_at(w.x, w.z);
                let here = format!("seed {seed} L{l}: {name} at {:.2}, {:.2}, {:.2} ({label}, rule {ri})", w.x, w.y, w.z);
                assert!(r.purposes.iter().any(|q| q == "*" || q == label) && !a.never.iter().any(|q| q == label), "{here}: not its place");
                assert!(!g.circle_hits_wall(w.x, w.z, 0.05), "{here}: in a wall");
                // its own tile reached from where you arrive (a wall's face can lie
                // inside its wall tile: a thing slid to it was out of every walk)
                assert!(reached[g.t_idx(g.tx(w.x), g.tz(w.z))] >= 0, "{here}: cut off");
                assert!(!r.dark || g.dark_at(w.x, w.z), "{here}: in the light, by a rule for the dark");
                match r.on {
                    On::Surface => {
                        // resting on it: rays down on a grid under it meet
                        // nothing of what it stands on above its base, and
                        // that thing's level top at its base under at least
                        // half of it (not the top of the thing's box: a vase or
                        // a chair's back stands higher; a crate's slats leave gaps)
                        let grid: Vec<(f64, f64)> =
                            (0..25).map(|i| ((i % 5) as f64 * 0.015 - 0.03, (i / 5) as f64 * 0.015 - 0.03)).collect();
                        let on = things.iter().any(|(k, t)| {
                            if !r.of.contains(k) {
                                return false;
                            }
                            let faces = pr::level_faces(t);
                            let tops: Vec<Option<f64>> = grid.iter().map(|&(dx, dz)| pr::top_over(&faces, w.x + dx, w.z + dz)).collect();
                            let held = tops.iter().filter(|h| h.is_some_and(|h| (h - w.y).abs() < 0.005)).count();
                            tops.iter().all(|h| h.is_none_or(|h| h < w.y + 0.005)) && held * 2 >= grid.len()
                        });
                        // (what the rays met on those of its kinds under it)
                        let met: Vec<String> = things
                            .iter()
                            .filter(|(k, t)| {
                                let mut bb = engine::math::Box3::empty();
                                bb.set_from_object(t);
                                r.of.contains(k) && (bb.min.x..=bb.max.x).contains(&w.x) && (bb.min.z..=bb.max.z).contains(&w.z)
                            })
                            .map(|(k, t)| {
                                let faces = pr::level_faces(t);
                                let tops: Vec<String> = grid
                                    .iter()
                                    .map(|&(dx, dz)| {
                                        pr::top_over(&faces, w.x + dx, w.z + dz).map_or("-".into(), |h| format!("{:+.3}", h - w.y))
                                    })
                                    .collect();
                                format!("{k}: {}", tops.join(" "))
                            })
                            .collect();
                        assert!(on, "{here}: not resting on anything it may stand on; under it {met:?}");
                    }
                    On::Layout => panic!("{here}: placed by the rules pass with a rule of the layout's"),
                    On::Floor => {
                        assert!((w.y - g.floor_y_at(w.x, w.z)).abs() < 1e-6, "{here}: off the floor");
                        if r.near != Some(Near::Open) {
                            assert!(g.circle_hits_wall(w.x, w.z, 0.2), "{here}: not at a wall's foot");
                        }
                    }
                }
                at.push((w.x, w.z, name));
            }
        }
        for (i, a) in at.iter().enumerate() {
            for b in &at[i + 1..] {
                let apart = if a.2 == b.2 { pr::APART } else { pr::APART_KINDS };
                assert!((a.0 - b.0).dhypot(a.1 - b.1) > apart - 0.01, "seed {seed} L{l}: two at {a:?} and {b:?}");
            }
        }
        // and as far from those of their kinds that others made (her camp's)
        for (k, t) in &things {
            let Some(kind) = k.strip_prefix("core_").filter(|k| pr::of(k).is_some_and(|a| a.by == pr::By::Rules)) else { continue };
            let p = t.world_position();
            for a in &at {
                let apart = if a.2 == kind { pr::APART } else { pr::APART_KINDS };
                assert!((a.0 - p.x).dhypot(a.1 - p.z) > apart - 0.01, "seed {seed} L{l}: {a:?} by a {kind} at {:.2}, {:.2}", p.x, p.z);
            }
        }
        (at.len(), things.into_iter().map(|(k, _)| k).collect::<std::collections::BTreeSet<String>>())
    });
    let placed: usize = floors.iter().map(|f| f.0).sum();
    assert!(placed > 150, "only {placed} placed by the rules");
    for a in pr::all() {
        for r in a.rules.iter().filter(|r| r.on == On::Surface) {
            for k in &r.of {
                // a placer that is registered (some are rare); a core kind seen on a floor
                let exists = if k.starts_with("core_") {
                    floors.iter().any(|f| f.1.contains(k))
                } else {
                    game::world::tables::PLACER_THEME.iter().any(|(p, _)| p == k)
                };
                assert!(exists, "{}: stands on {k}, which nothing places", a.name);
            }
        }
    }
}

/// Rules the layout's makers do not ask yet (`docs/plan/DOORS.md`: each
/// milestone makes its own and takes it off this list)
const NOT_YET: [(&str, &str); 6] = [
    ("hermetic_door", "cell"),
    ("hermetic_door", "gallery"),
    ("lift_doors", "lift"),
    ("lift_doors", "broken lift"),
    ("interior_door", "wall"),
    ("interior_door", "barricade"),
];

/// What the floor's layout makes (the doors: `docs/plan/DOORS.md`) stands
/// where a rule of its asset allows it (`place_by_layout`): the rule names
/// its place in the layout, its purposes take the zone's and the asset
/// never fits none of it, on a floor in its levels, in its counts; every
/// floor of the rooms and of the Building has its one way down; and every
/// rule is one some floor's layout makes (a rule nothing makes is a lie)
#[test]
fn what_the_layout_places_keeps_its_rules() {
    use game::world::placement_rules::{self as pr, By};
    let levels: Vec<i32> = (1..=12).collect();
    let floors = each_floor(&levels, |seed, l| {
        let mut g = floor(seed, l);
        let here = format!("seed {seed} L{l} {:?}", g.level_scp);
        let ways = g.layout_things.iter().filter(|t| t.at == "way down").count();
        assert_eq!(ways, 1, "{here}: {ways} ways down");
        // through the way down where it leads to a place of its own: its doors
        let mode = if g.level_scp.as_deref() == Some("loop") {
            g.enter_loop();
            Some("loop")
        } else if g.layout_things.iter().any(|t| t.asset == "steel_door") {
            g.enter_stairwell();
            Some("stairwell")
        } else {
            None
        };
        if let Some(m) = mode {
            g.now_ms += 1000.0;
            g.run_scheduled();
            let n = g.layout_things.iter().filter(|t| t.at == m).count();
            assert_eq!(n, 2, "{here}: {n} doors in the {m}");
        }
        let mut made = std::collections::BTreeMap::<&str, usize>::new();
        let mut at = std::collections::BTreeSet::new();
        for t in &g.layout_things {
            let a = pr::of(t.asset).unwrap_or_else(|| panic!("{here}: {} has no rules", t.asset));
            assert_eq!(a.by, By::Layout, "{here}: {} is the rules pass's", t.asset);
            let r = &a.rules[t.rule];
            let what = format!("{here}: {} at {} ({:.2}, {:.2}; {}, rule {})", t.asset, t.at, t.x, t.z, t.purpose, t.rule);
            assert_eq!(r.at, t.at, "{what}: its rule is for {}", r.at);
            assert!(r.purposes.iter().any(|p| p == "*" || p == t.purpose), "{what}: not a purpose of its rule ({:?})", r.purposes);
            assert!(!a.never.iter().any(|p| p == t.purpose), "{what}: a purpose it never fits");
            assert!((a.levels.0..=a.levels.1).contains(&l), "{what}: not on its floors {:?}", a.levels);
            assert_eq!(g.purpose_label_at(t.x, t.z), t.purpose, "{what}: its purpose as recorded");
            *made.entry(t.asset).or_default() += 1;
            at.insert((t.asset.to_string(), t.at.to_string()));
        }
        for (asset, n) in &made {
            let a = pr::of(asset).unwrap();
            assert!(*n <= a.count.1, "{here}: {n} {asset}, past its most {}", a.count.1);
        }
        at
    });
    let made: std::collections::BTreeSet<(String, String)> = floors.into_iter().flatten().collect();
    for a in pr::all().iter().filter(|a| a.by == By::Layout) {
        for r in &a.rules {
            let key = (a.name.clone(), r.at.clone());
            let pending = NOT_YET.iter().any(|&(n, at)| n == a.name && at == r.at);
            assert!(made.contains(&key) || pending, "{}: no floor's layout makes one at {}", a.name, r.at);
            assert!(!(made.contains(&key) && pending), "{}: made at {} now: take it off NOT_YET", a.name, r.at);
        }
    }
}
