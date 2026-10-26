//! The story layer, and the coherence it stands on.
//!
//! * A level is the same place however much of the run's lore has been read:
//!   a retry and a Continue rebuild it exactly (same draws at every build
//!   checkpoint, same objects), and only the notes lying on it change, to
//!   ones not yet read (`Game::fit_lore_notes`).
//!
//!   cargo test --release -p backrooms --test story
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

/// Build `level` of run `seed` (or rebuild it through `f`), tracing its draws.
fn build(g: &mut Game, f: impl FnOnce(&mut Game)) {
    g.rand_trace(true);
    f(g);
}

/// The level's description, draws at every checkpoint included; less what
/// `fit_lore_notes` changes on purpose: the notes' own size (a note that
/// continues a story gets a wider glow and a beacon) and the animator count
/// (the beacon turns to face you).
fn dump(g: &Game) -> serde_json::Value {
    let mut d = g.level_dump();
    if let Some(c) = d.get_mut("counts").and_then(|c| c.as_object_mut()) {
        c.remove("animators");
    }
    let at = |x: f64| (x * 1e6).round() / 1e6;
    let notes: Vec<(f64, f64)> = g.lore_notes.iter().map(|n| (at(n.g.pos().x), at(n.g.pos().z))).collect();
    for o in d.get_mut("objects").and_then(|o| o.as_array_mut()).into_iter().flatten() {
        let p = (o["pos"][0].as_f64().unwrap_or(f64::NAN), o["pos"][2].as_f64().unwrap_or(f64::NAN));
        if notes.iter().any(|n| (n.0 - p.0).abs() < 1e-5 && (n.1 - p.1).abs() < 1e-5) {
            if let Some(m) = o.as_object_mut() {
                for k in ["min", "max", "meshes"] {
                    m.remove(k);
                }
            }
        }
    }
    d
}

/// The first place two descriptions differ, as a path and the two values.
fn first_diff(a: &serde_json::Value, b: &serde_json::Value, at: &str) -> Option<String> {
    use serde_json::Value::*;
    match (a, b) {
        (Object(x), Object(y)) => {
            for (k, v) in x {
                match y.get(k) {
                    Some(w) => {
                        if let Some(d) = first_diff(v, w, &format!("{at}.{k}")) {
                            return Some(d);
                        }
                    }
                    None => return Some(format!("{at}.{k}: only in the first")),
                }
            }
            y.keys().find(|k| !x.contains_key(*k)).map(|k| format!("{at}.{k}: only in the second"))
        }
        (Array(x), Array(y)) => {
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                if let Some(d) = first_diff(v, w, &format!("{at}[{i}]")) {
                    return Some(d);
                }
            }
            (x.len() != y.len()).then(|| format!("{at}: {} vs {} items", x.len(), y.len()))
        }
        _ => (a != b).then(|| format!("{at}: {a} vs {b}")),
    }
}

fn same(a: &serde_json::Value, b: &serde_json::Value, what: &str) {
    if let Some(d) = first_diff(a, b, "") {
        panic!("{what}: {d}");
    }
}

/// Where each note lies, and which it is.
fn notes(g: &Game) -> Vec<((i64, i64), usize)> {
    g.lore_notes.iter().map(|n| (((n.g.pos().x * 1000.0) as i64, (n.g.pos().z * 1000.0) as i64), n.id)).collect()
}

#[test]
fn a_level_is_the_same_place_whatever_was_read() {
    for (seed, level) in [(4242u32, 2), (7, 4), (1999, 6)] {
        let mut fresh = new_game();
        fresh.run_seed = seed;
        build(&mut fresh, |g| g.init_level(level));
        let (want, want_notes) = (dump(&fresh), notes(&fresh));
        assert!(!want_notes.is_empty(), "seed {seed} L{level}: no notes to test with");

        // the notes on it, and a dozen more, already read
        let mut read: std::collections::BTreeSet<usize> = want_notes.iter().map(|n| n.1).collect();
        read.extend(0..12);
        let mut g = new_game();
        g.run_seed = seed;
        g.lore_found = read.clone();
        build(&mut g, |g| g.init_level(level));
        same(&dump(&g), &want, &format!("seed {seed} L{level}: the build read the save"));
        let got = notes(&g);
        assert_eq!(got.iter().map(|n| n.0).collect::<Vec<_>>(), want_notes.iter().map(|n| n.0).collect::<Vec<_>>());
        for (pos, id) in &got {
            let scp = g.lore_entry_scp(*id).is_some();
            assert!(scp || !read.contains(id), "seed {seed} L{level}: note at {pos:?} is {id}, already read");
        }
        let ids: std::collections::BTreeSet<usize> = got.iter().map(|n| n.1).collect();
        assert_eq!(ids.len(), got.len(), "seed {seed} L{level}: a note lies twice");
        assert_eq!(g.lore_found, read, "fitting the notes changed what was read");

        // a retry after reading one more
        g.lore_found.insert(got[0].1);
        build(&mut g, |g| g.restart());
        same(&dump(&g), &want, &format!("seed {seed} L{level}: the retry is another place"));
        assert!(!notes(&g).iter().any(|n| n.1 == got[0].1 && g.lore_entry_scp(n.1).is_none()), "the note just read lies there again");

        // Continue from the save
        g.save_game();
        let found = g.lore_found.clone();
        g.lore_found.clear();
        build(&mut g, |g| g.continue_game());
        same(&dump(&g), &want, &format!("seed {seed} L{level}: Continue built another place"));
        assert_eq!(g.lore_found, found);
        assert!(notes(&g).iter().all(|n| g.lore_entry_scp(n.1).is_some() || !found.contains(&n.1)));
    }
}
