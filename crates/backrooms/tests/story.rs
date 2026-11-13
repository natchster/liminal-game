//! The story layer, and the coherence it stands on.
//!
//! * A level is the same place however much of the run's lore has been read:
//!   a retry and a Continue rebuild it exactly (same draws at every build
//!   checkpoint, same objects), and only the notes lying on it change, to
//!   ones not yet read (`Game::fit_lore_notes`).
//!
//!   cargo test --release -p backrooms --test story
use game::Game;

fn new_game() -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
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

/// A level's frames are the same whatever the thread built before it: the
/// per-frame stream, a second into the level, is the same on a level built
/// first and on that level built after another. (Textures are painted once,
/// whenever first needed, each from a stream of its own,
/// `placers::kit::canvas_tex`: painted on the per-frame stream, they shifted
/// every frame after them, and only on the first level that needed them, so
/// a creature's choices on a level hung on the levels before it, E10.)
#[test]
fn a_levels_frames_are_the_same_whatever_came_before() {
    let stream = |before: Option<(u32, i32)>| {
        std::thread::spawn(move || {
            if let Some((seed, level)) = before {
                let mut g = new_game();
                g.run_seed = seed;
                build(&mut g, |g| g.init_level(level));
            }
            game::base::rng::seed_native(0xE10);
            let mut g = new_game();
            g.run_seed = 4242;
            build(&mut g, |g| g.init_level(6));
            for _ in 0..60 {
                g.frame(1.0 / 60.0);
            }
            (0..8).map(|_| game::base::rng::native_rnd()).collect::<Vec<f64>>()
        })
        .join()
        .unwrap()
    };
    let first = stream(None);
    for before in [(7, 4), (1999, 9), (4242, 2)] {
        assert_eq!(stream(Some(before)), first, "level 6 after seed {} level {}", before.0, before.1);
    }
}

/// A level's build takes nothing from the per-frame stream, whatever the
/// same game built before it: the surfaces painted the first time a level
/// needs them (drywall, concrete, brick, tile: `visuals::surfaces`, cached
/// for the game) paint from their painters' own streams. (They had drawn on
/// the per-frame stream, so the first level to need one moved every frame
/// after it, and in a run only that level did; rooms R5)
#[test]
fn a_levels_build_takes_nothing_from_the_frames_whatever_came_before() {
    let after_build = |before: Option<i32>| {
        std::thread::spawn(move || {
            let mut g = new_game();
            g.run_seed = 4242;
            if let Some(level) = before {
                build(&mut g, |g| g.init_level(level));
            }
            game::base::rng::seed_native(0xE11);
            build(&mut g, |g| g.init_level(6));
            (0..8).map(|_| game::base::rng::native_rnd()).collect::<Vec<f64>>()
        })
        .join()
        .unwrap()
    };
    let first = after_build(None);
    for before in [4, 9] {
        assert_eq!(after_build(Some(before)), first, "level 6 after level {before} in the same game");
    }
}

/// The Building's own torn notes (the residents' notices, the cleaners' log,
/// the surveys) lie on its floors and on no floor of the rooms; on its floors
/// lie no others.
#[test]
fn the_buildings_notes_lie_only_on_its_floors() {
    for seed in [4242u32, 7] {
        let mut g = new_game();
        g.run_seed = seed;
        for level in 1..=6 {
            g.init_level(level);
            let tags: Vec<Option<String>> = notes(&g).iter().map(|n| g.lore_entry_scp(n.1)).collect();
            let building = tags.iter().filter(|t| t.as_deref() == Some("building")).count();
            if level <= 3 {
                assert!(building >= 1 && building == tags.len(), "seed {seed} L{level}: {tags:?}");
            } else {
                assert_eq!(building, 0, "seed {seed} L{level}: the Building's notes in the rooms: {tags:?}");
            }
        }
    }
}

/// The arrival's tile and the door's.
fn ends(g: &Game) -> ((i32, i32), (i32, i32)) {
    let e = g.exit_data.as_ref().unwrap().pos;
    ((g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z)), (g.tx(e.x), g.tz(e.z)))
}

/// What must hold for her trail on a level built for (seed, level): a walk
/// from the arrival through her camp to the door over open floor; her camp
/// off the straight way when the level has room for that; the first mark
/// close to the arrival (a sheet: her signature) and the last close to the
/// door; each mark in sight of the one before, at most `spacing` tiles of
/// walk on and near enough to be seen at all at its kind's distance
/// (`MarkKind::reach`), except across a stretch the plan says lost its
/// marks; each kind only from the level she first used it, a turn only
/// where her way turns, and on that first level the first of a kind with a
/// sheet beside it.
fn check_trail(g: &Game) -> Result<String, String> {
    use game::story::MarkKind;
    let s = &g.story;
    let what = format!("seed {} L{}", g.run_seed, g.level_num);
    let camp = s.camp.as_ref().ok_or(format!("{what}: no camp"))?;
    let (spawn, exit) = ends(g);
    if s.walk.first() != Some(&spawn) || s.walk.last() != Some(&exit) || s.walk[s.camp_step] != camp.tile {
        return Err(format!("{what}: the walk does not go arrival → camp → door"));
    }
    for w in s.walk.windows(2) {
        let step = (w[0].0 - w[1].0).abs() + (w[0].1 - w[1].1).abs();
        if step != 1 || g.is_wall_tile(w[1].0, w[1].1) {
            return Err(format!("{what}: the walk jumps or goes through a wall at {:?} → {:?}", w[0], w[1]));
        }
    }
    let ds = g.story_walk_dist(spawn);
    let de = g.story_walk_dist(exit);
    let i = |p: (i32, i32)| (p.1 * g.w + p.0) as usize;
    let straight = ds[i(exit)];
    let detour = ds[i(camp.tile)] + de[i(camp.tile)] - straight;
    if s.trail.is_empty() {
        return Err(format!("{what}: no marks"));
    }
    if s.trail[0].kind != MarkKind::Sheet {
        return Err(format!("{what}: her first mark is a {:?}, not a sheet", s.trail[0].kind));
    }
    let first = s.trail[0].step;
    let last = s.walk.len() - 1 - s.trail.last().unwrap().step;
    if first > 3 || last > 3 {
        return Err(format!("{what}: the first mark is {first} steps from the arrival, the last {last} from the door"));
    }
    let k = game::story::spacing(g.depth());
    // the marks in walk order: hers and her camp (seen from where each is seen)
    let mut marks: Vec<(usize, f64, f64, MarkKind)> = s.trail.iter().map(|m| (m.step, m.seen_at().0, m.seen_at().1, m.kind)).collect();
    marks.push((s.camp_step, g.wx(camp.tile.0), g.wz(camp.tile.1), MarkKind::Sheet));
    marks.sort_by_key(|m| m.0);
    for w in marks.windows(2) {
        let gap = s.gaps.iter().any(|&(a, b)| a == w[0].0 && b == w[1].0);
        let apart = w[1].0 - w[0].0;
        let limit = if gap { k * 3 + 2 } else { k };
        if apart > limit {
            return Err(format!("{what}: {apart} steps between marks at steps {} and {} (at most {limit})", w[0].0, w[1].0));
        }
        if !gap && !g.los(w[0].1, w[0].2, w[1].1, w[1].2) {
            return Err(format!("{what}: the {:?} at step {} cannot be seen from the mark at step {}", w[1].3, w[1].0, w[0].0));
        }
        let d = (w[1].1 - w[0].1).hypot(w[1].2 - w[0].2);
        if !gap && w[1].0 != s.camp_step && d > w[1].3.reach() + 1e-6 {
            return Err(format!("{what}: the {:?} at step {} is {d:.1} m from the last, seen only {} m", w[1].3, w[1].0, w[1].3.reach()));
        }
    }
    let mut kinds: Vec<(MarkKind, usize)> = vec![];
    for m in &s.trail {
        if m.kind.from_level() > g.level_num || m.kind == MarkKind::Flat {
            return Err(format!("{what}: a {:?} before she used them (from L{})", m.kind, m.kind.from_level()));
        }
        if m.kind == MarkKind::Turn {
            let (a, b, c) = (s.walk[m.step - 1], s.walk[m.step], s.walk[m.step + 1]);
            if (b.0 - a.0, b.1 - a.1) == (c.0 - b.0, c.1 - b.1) {
                return Err(format!("{what}: a turn's mark at step {} where her way goes straight on", m.step));
            }
        }
        if m.kind.on_wall() {
            // on a wall's face: something solid just behind it, and open in front
            let (bx, bz) = (m.x - m.normal.0 * 0.03, m.z - m.normal.1 * 0.03);
            let (fx, fz) = (m.x + m.normal.0 * 0.1, m.z + m.normal.1 * 0.1);
            if !g.circle_hits_wall(bx, bz, 0.02) || g.circle_hits_wall(fx, fz, 0.02) {
                return Err(format!("{what}: the {:?} at step {} is not on a wall's face", m.kind, m.step));
            }
        }
        match kinds.iter_mut().find(|k| k.0 == m.kind) {
            Some(k) => k.1 += 1,
            None => kinds.push((m.kind, 1)),
        }
    }
    for &(kind, _) in &kinds {
        if kind != MarkKind::Sheet && kind.from_level() == g.level_num {
            let i = s.trail.iter().position(|m| m.kind == kind).unwrap();
            let paired = s.trail.iter().any(|m| m.kind == MarkKind::Sheet && m.step == s.trail[i].step);
            if !paired {
                return Err(format!("{what}: the first {:?} (first used here) has no sheet beside it", kind));
            }
        }
    }
    for m in &s.trail {
        let o = m.g.as_ref().ok_or(format!("{what}: a mark with no object"))?;
        if o.parent().is_none() || (o.pos().x - m.x).abs() > 1e-6 || (o.pos().z - m.z).abs() > 1e-6 {
            return Err(format!(
                "{what}: the {:?} at step {} is not where the plan put it ({:.3}, {:.3} for {:.3}, {:.3}; in the world: {})",
                m.kind,
                m.step,
                o.pos().x,
                o.pos().z,
                m.x,
                m.z,
                o.parent().is_some()
            ));
        }
    }
    let kinds: Vec<String> = kinds.iter().map(|(k, n)| format!("{} {n}", k.name())).collect();
    Ok(format!(
        "{what}: walk {} tiles (straight {straight}), camp {} steps in, {detour} off the straight way, {} marks ({}), gaps {:?}",
        s.walk.len() - 1,
        s.camp_step,
        s.trail.len(),
        kinds.join(", "),
        s.gaps
    ))
}

/// The level from above: walls dark, the walk pale, her sheets white, her
/// camp orange, the arrival green, the door red; trails that are not hers
/// blue, their dead ends violet. `target/story/`.
fn plan_png(g: &Game) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/story");
    std::fs::create_dir_all(&dir).unwrap();
    let k = 8i32;
    let (w, h) = (g.w * k, g.h * k);
    let mut px = vec![0u8; (w * h * 4) as usize];
    let put = |x: i32, z: i32, c: [u8; 4], r: i32, px: &mut Vec<u8>| {
        for dz in -r..=r {
            for dx in -r..=r {
                let (a, b) = (x + dx, z + dz);
                if a >= 0 && b >= 0 && a < w && b < h {
                    let i = ((b * w + a) * 4) as usize;
                    px[i..i + 4].copy_from_slice(&c);
                }
            }
        }
    };
    for z in 0..g.h {
        for x in 0..g.w {
            let c = if g.is_wall_tile(x, z) { [24, 24, 24, 255] } else { [92, 88, 70, 255] };
            put(x * k + k / 2, z * k + k / 2, c, k / 2, &mut px);
        }
    }
    let to_px = |x: f64, z: f64| (((x / 4.0) + g.w as f64 / 2.0) * k as f64, ((z / 4.0) + g.h as f64 / 2.0) * k as f64);
    for t in &g.story.walk {
        put(t.0 * k + k / 2, t.1 * k + k / 2, [150, 140, 105, 255], 1, &mut px);
    }
    for m in &g.story.trail {
        let (x, z) = to_px(m.x, m.z);
        put(x as i32, z as i32, [250, 248, 235, 255], 1, &mut px);
    }
    for f in &g.story.false_trails {
        for t in &f.walk[1..] {
            put(t.0 * k + k / 2, t.1 * k + k / 2, [90, 100, 140, 255], 1, &mut px);
        }
        for m in &f.trail {
            let (x, z) = to_px(m.x, m.z);
            put(x as i32, z as i32, [140, 170, 255, 255], 1, &mut px);
        }
        let (x, z) = to_px(f.end.0, f.end.1);
        put(x as i32, z as i32, [170, 60, 220, 255], 2, &mut px);
    }
    let (spawn, exit) = ends(g);
    put(spawn.0 * k + k / 2, spawn.1 * k + k / 2, [60, 200, 90, 255], 2, &mut px);
    put(exit.0 * k + k / 2, exit.1 * k + k / 2, [220, 50, 40, 255], 2, &mut px);
    if let Some(c) = &g.story.camp {
        let (x, z) = to_px(c.at.x, c.at.z);
        put(x as i32, z as i32, [240, 140, 30, 255], 2, &mut px);
    }
    let f = dir.join(format!("{}_L{}.png", g.run_seed, g.level_num));
    engine::png::write_rgba(f.to_str().unwrap(), w as u32, h as u32, &px).unwrap();
}

#[test]
fn her_trail_leads_through_her_camp_to_the_door() {
    let mut cases = vec![(4242u32, 1), (4242, 2), (4242, 3), (7, 4), (7, 5), (1999, 6), (1999, 8), (77, 10), (2026, 12)];
    // and every anomaly's floor of one run
    for n in 2..=12 {
        if let Some(k) = game::level::scp::scp_for_level(4242, n) {
            if !cases.contains(&(4242, n)) {
                cases.push((4242, n));
            }
            println!("run 4242: {k} on level {n}");
        }
    }
    let mut bad = vec![];
    for (seed, level) in cases {
        let mut g = new_game();
        g.run_seed = seed;
        g.init_level(level);
        plan_png(&g);
        match check_trail(&g) {
            Ok(line) => println!("{line}"),
            Err(e) => bad.push(e),
        }
        // the same plan from a fresh build
        let mut h = new_game();
        h.run_seed = seed;
        h.init_level(level);
        let (a, b) = (g.plan_story(), h.plan_story());
        if a != b || a.is_none() {
            bad.push(format!("seed {seed} L{level}: the plan is not a function of the level ({:?})", a.is_none()));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// She ran short of paper as she went down: on the Building's floors her
/// marks are mostly sheets, deep down mostly the others; every kind of hers
/// is met somewhere, and none glows (what lights a mark is the place, or
/// your lamp on tape).
#[test]
fn her_marks_change_as_she_goes_down() {
    use game::story::MarkKind;
    // (sheets, marks) on levels 1-3, 4-6, 7-10, 11-13
    let mut bands = [(0usize, 0usize); 4];
    let mut met: Vec<MarkKind> = vec![];
    for seed in [4242u32, 7, 1999] {
        for level in 1..=13 {
            let mut g = new_game();
            g.run_seed = seed;
            g.init_level(level);
            let b = match level {
                ..=3 => 0,
                4..=6 => 1,
                7..=10 => 2,
                _ => 3,
            };
            for m in &g.story.trail {
                bands[b].1 += 1;
                if m.kind == MarkKind::Sheet {
                    bands[b].0 += 1;
                }
                if !met.contains(&m.kind) {
                    met.push(m.kind);
                }
                let mut lit = false;
                m.g.as_ref().unwrap().traverse(&mut |o| {
                    if let Some(mat) = o.try_mat() {
                        lit |= mat.m().emissive.get_hex() != 0;
                    }
                });
                assert!(!lit, "seed {seed} L{level}: a {:?} glows", m.kind);
            }
        }
    }
    let share: Vec<f64> = bands.iter().map(|b| b.0 as f64 / b.1.max(1) as f64).collect();
    println!("sheets among her marks by depth (1-3, 4-6, 7-10, 11-13): {share:.2?} of {:?}", bands.map(|b| b.1));
    println!("kinds met: {met:?}");
    assert!(share[0] >= 0.6, "the Building's floors: sheets {:.2}", share[0]);
    assert!(share[3] <= 0.4, "deep down: sheets {:.2}", share[3]);
    assert!(share[0] > share[1] && share[1] > share[2] && share[2] >= share[3], "fewer sheets as she went down: {share:.2?}");
    for k in game::story::marks::HER_KINDS {
        assert!(met.contains(&k), "never met her {k:?}");
    }
}

/// Her notebook follows the descent on every run: thirteen pages, each once;
/// the floor pages on the run's ordinary floors in their order, an
/// anomaly's page on its floor, the last on level 13; nothing after it.
#[test]
fn her_notebook_follows_the_descent() {
    use game::story::notebook::{notebook, page_for_level, ANOMALY_PAGES, FLOOR_PAGES, LAST_LEVEL};
    for run in [1u32, 7, 77, 4242, 1999, 2026, 0xBEEF, 9_000_001] {
        let book = notebook(run);
        assert_eq!(book.len(), 13, "run {run}: {:?}", book.iter().map(|(n, p)| (n, p.id)).collect::<Vec<_>>());
        let ids: std::collections::BTreeSet<&str> = book.iter().map(|(_, p)| p.id).collect();
        assert_eq!(ids.len(), 13, "run {run}: a page twice");
        let floors: Vec<&str> = book.iter().filter(|(_, p)| p.id.starts_with('f')).map(|(_, p)| p.id).collect();
        assert_eq!(floors, FLOOR_PAGES.iter().map(|p| p.id).collect::<Vec<_>>(), "run {run}: floor pages out of order");
        for (n, p) in &book {
            match game::level::scp::scp_for_level(run, *n) {
                Some(k) => assert_eq!(p.id, k, "run {run} L{n}"),
                None if *n == LAST_LEVEL => assert_eq!(p.id, "last"),
                None => assert!(p.id.starts_with('f'), "run {run} L{n}: {}", p.id),
            }
        }
        assert!(ANOMALY_PAGES.iter().all(|a| ids.contains(a.id)));
        assert!(page_for_level(run, LAST_LEVEL + 1).is_none());
    }
}

/// Taking her page: it goes into the notebook and the journal (opened on
/// it), its last line stays at the top of the screen, it is gone from the
/// bedroll and the prompt with it; it stays taken on a retry and through a
/// save and a Continue, and the journal shows a torn edge for a floor passed
/// without taking hers.
#[test]
fn taking_her_page() {
    let mut g = new_game();
    g.run_seed = 4242;
    g.init_level(3);
    g.state = "playing".into();
    let id = g.story_page().expect("a page on level 3").id;
    let camp = g.story.camp.clone().expect("a camp");
    assert!(camp.page.as_ref().is_some_and(|o| o.visible()) && !camp.page_taken);
    // standing at the camp, the prompt is hers
    g.player.pos = engine::math::Vec3::new(camp.at.x, camp.at.y + game::prelude::EYE, camp.at.z);
    g.update_hud(1.0 / 60.0);
    assert_eq!(g.ui.prompt.text, "[E] TAKE HER PAGE");
    g.try_interact();
    assert!(g.notebook.contains(id), "the page is in the notebook");
    let p = game::story::notebook::page(id).unwrap();
    assert_eq!(g.thread, p.thread);
    assert!(g.ui.objective.text.contains(p.thread), "the thread shows: {:?}", g.ui.objective.text);
    assert!(!g.story.camp.as_ref().unwrap().page.as_ref().unwrap().visible(), "the page left the bedroll");
    g.update_hud(1.0 / 60.0);
    // (on the Building's floors her camp is in a cell: its wheel is still there to turn)
    assert_ne!(g.ui.prompt.text, "[E] TAKE HER PAGE", "nothing left to take");
    // the journal opens on it; levels 1 and 2 were passed without hers
    g.toggle_journal();
    assert!(g.ui.journal_entry_title.text.starts_with("HER NOTEBOOK"), "{}", g.ui.journal_entry_title.text);
    assert_eq!(g.ui.journal_entry_body.text, p.body);
    let ids = g.journal_entries_in_order();
    assert_eq!(&ids[1..4], &[-101, -102, -103], "{ids:?}");
    g.journal_cursor = 1.0;
    g.journal_render();
    assert_eq!(g.ui.journal_entry_body.text, "A torn edge, where a page was.");
    g.toggle_journal();
    // a retry: the page stays taken
    g.restart();
    let c = g.story.camp.as_ref().unwrap();
    assert!(c.page_taken && !c.page.as_ref().unwrap().visible(), "the retry put her page back");
    assert!(g.ui.objective.text.contains(p.thread));
    // a save and a Continue
    g.save_game();
    g.notebook.clear();
    g.thread.clear();
    g.continue_game();
    assert!(g.notebook.contains(id) && g.thread == p.thread, "Continue lost the notebook");
    assert!(g.story.camp.as_ref().unwrap().page_taken);
    // a new game starts an empty notebook
    g.launch_flags = "nosound&seed=4242".into();
    g.start_new_game();
    assert!(g.notebook.is_empty() && g.thread.is_empty() && g.ui.objective.text.is_empty());
}

/// `secs` of play at 60 fps.
fn play(g: &mut Game, secs: f64) {
    for _ in 0..(secs * 60.0).round() as usize {
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
    }
}

/// Taking her page wakes the place (PILLARS 5: you choose when the dread
/// starts): her lamp gutters out and its light leaves the light map, and the
/// quiet ends within `WAKE` seconds: the director's log reads the page, the
/// wake, then a build-up. On the first floor the first quiet waits for her
/// page (`FIRST_FLOOR_CALM`); on a retry her lamp is already out.
#[test]
fn her_page_wakes_the_place() {
    use game::entities::director::{Phase, FIRST_FLOOR_CALM, WAKE};
    // (rooms levels: what wakes on the Building's floors is not a creature)
    for (seed, level) in [(4242u32, 4), (4242, 6), (7, 8)] {
        let mut g = new_game();
        g.run_seed = seed;
        g.init_level(level);
        g.state = "playing".into();
        g.god_mode = true;
        if level == 1 {
            assert!(g.director.phase == Phase::Calm && g.director.t >= FIRST_FLOOR_CALM.0, "L1's first quiet: {:.0} s", g.director.t);
        }
        let camp = g.story.camp.clone().unwrap();
        let ti = (camp.tile.1 * g.w + camp.tile.0) as usize;
        let lit_before = g.light_map_cpu[ti];
        play(&mut g, 5.0);
        assert_eq!(g.director.phase, Phase::Calm, "seed {seed} L{level}: still quiet 5 s in");
        g.player.pos = engine::math::Vec3::new(camp.at.x, camp.at.y + game::prelude::EYE, camp.at.z);
        let t0 = g.perf_t;
        g.try_interact();
        assert!(!g.notebook.is_empty(), "seed {seed} L{level}: the page was not taken");
        play(&mut g, 2.5);
        let c = g.story.camp.as_ref().unwrap();
        assert!(!c.lit && !c.light.as_ref().unwrap().visible(), "seed {seed} L{level}: her lamp still burns");
        assert!(g.light_map_cpu[ti] <= lit_before, "seed {seed} L{level}: the camp is lighter after her lamp went out");
        play(&mut g, WAKE.1 + 0.5);
        let after: Vec<&str> = g.director.log.iter().filter(|e| e.0 >= t0 - 1e-9).map(|e| e.1).collect();
        let at = |what: &str| after.iter().position(|e| *e == what);
        println!("seed {seed} L{level}: after the page: {:?}", &after[..after.len().min(8)]);
        assert!(
            matches!((at("story:page"), at("wake"), at("build")), (Some(a), Some(b), Some(c)) if a < b && b < c),
            "seed {seed} L{level}: no build-up within {} s of the page: {after:?}",
            WAKE.1 + 3.0
        );
        // a retry: the page stays taken and her lamp out
        g.restart();
        let c = g.story.camp.as_ref().unwrap();
        assert!(c.page_taken && !c.lit && !c.light.as_ref().unwrap().visible(), "seed {seed} L{level}: the retry relit her lamp");
    }
}

/// Trails that are not hers: none above the floor of her page `f6`, one from
/// there, two from `f7`'s. Each leaves her walk, never goes back onto it,
/// and ends in a dead end well away from the door; its flat sheets can be
/// followed like hers; and at its end the place has you (`lure` in the
/// director's log, once).
#[test]
fn trails_that_are_not_hers_lead_nowhere() {
    let run = 4242u32;
    let book = game::story::notebook::notebook(run);
    let level_of = |id: &str| book.iter().find(|(_, p)| p.id == id).unwrap().0;
    let (l5, l7) = (level_of("f6"), level_of("f7"));
    for level in [l5 - 1, l5, l7] {
        let mut g = new_game();
        g.run_seed = run;
        g.init_level(level);
        g.state = "playing".into();
        g.god_mode = true;
        plan_png(&g);
        let due = if level >= l7 {
            2
        } else if level >= l5 {
            1
        } else {
            0
        };
        assert_eq!(g.false_trails_due(), due);
        assert_eq!(g.story.false_trails.len(), due, "L{level}: trails that are not hers");
        let hers: std::collections::HashSet<(i32, i32)> = g.story.walk.iter().copied().collect();
        let (_, exit) = ends(&g);
        let de = g.story_walk_dist(exit);
        let k = game::story::spacing(g.depth());
        for (n, f) in g.story.false_trails.iter().enumerate() {
            assert!(hers.contains(&f.walk[0]), "L{level} #{n}: does not leave her walk");
            assert!(f.walk[1..].iter().all(|t| !hers.contains(t)), "L{level} #{n}: goes back onto her walk");
            for w in f.walk.windows(2) {
                assert_eq!((w[0].0 - w[1].0).abs() + (w[0].1 - w[1].1).abs(), 1, "L{level} #{n}: jumps");
            }
            let e = *f.walk.last().unwrap();
            let walls = [(0, -1), (1, 0), (0, 1), (-1, 0)].iter().filter(|(dx, dz)| g.is_wall_tile(e.0 + dx, e.1 + dz)).count();
            assert!(walls >= 3, "L{level} #{n}: ends in the open ({walls} walls)");
            assert!(de[(e.1 * g.w + e.0) as usize] >= 8, "L{level} #{n}: ends by the door");
            assert!(!f.trail.is_empty() && f.trail[0].step <= 2, "L{level} #{n}: its first page is not at the branch");
            assert!(f.trail.iter().all(|m| m.kind == game::story::MarkKind::Flat), "L{level} #{n}: a mark of hers on it");
            for w in f.trail.windows(2) {
                assert!(w[1].step - w[0].step <= k && g.los(w[0].x, w[0].z, w[1].x, w[1].z), "L{level} #{n}: cannot be followed");
            }
            // one stamp, as no tired person walks: mid-floor, every two tiles where it can be seen
            let even = f.trail.windows(2).filter(|w| w[1].step - w[0].step == game::story::marks::FLAT_EVERY).count();
            assert!(even * 2 >= f.trail.len().saturating_sub(2), "L{level} #{n}: not even ({even} of {})", f.trail.len());
            for m in &f.trail {
                let t = f.walk[m.step];
                assert!(
                    (m.x - g.wx(t.0)).abs() < 1e-9 && (m.z - g.wz(t.1)).abs() < 1e-9,
                    "L{level} #{n}: a page off the middle of its tile"
                );
            }
            println!(
                "L{level}: a trail not hers leaves her walk at step {} and goes {} tiles to a dead end, {} sheets",
                g.story.walk.iter().position(|t| *t == f.walk[0]).unwrap(),
                f.walk.len() - 1,
                f.trail.len()
            );
        }
        // at the end of one, the place has you, once
        if let Some(f) = g.story.false_trails.first().cloned() {
            g.player.pos = engine::math::Vec3::new(f.end.0, g.floor_y_at(f.end.0, f.end.1) + game::prelude::EYE, f.end.1);
            play(&mut g, 0.5);
            play(&mut g, 0.5);
            let lures = g.director.log.iter().filter(|e| e.1 == "lure").count();
            assert_eq!(lures, 1, "L{level}: the place answered {lures} times");
            assert!(g.story.false_trails[0].sprung);
        }
    }
}

/// The bottom of her story. On level 13 her trail stops at her camp, her
/// lamp is cold and the last page is there; leaving the floor shows the
/// card, E goes on down with what is left of her paper, and below there is
/// no camp and no trail. The end is saved and comes only once.
#[test]
fn the_last_camp_ends_her_story() {
    use game::story::ending::HER_PAPER;
    use game::story::notebook::LAST_LEVEL;
    let mut g = new_game();
    g.run_seed = 4242;
    g.init_level(LAST_LEVEL);
    g.state = "playing".into();
    g.god_mode = true;
    let camp = g.story.camp.clone().expect("the last camp");
    assert!(!camp.lit && !camp.light.as_ref().unwrap().visible(), "her lamp should be cold at the last camp");
    assert_eq!(g.story_page().map(|p| p.id), Some("last"));
    assert!(!g.story.trail.is_empty(), "her trail to the last camp");
    assert!(g.story.trail.iter().all(|m| m.step < g.story.camp_step), "her paper goes on past the last camp");
    g.player.pos = engine::math::Vec3::new(camp.at.x, camp.at.y + game::prelude::EYE, camp.at.z);
    g.try_interact();
    assert!(g.notebook.contains("last"));
    let papers = g.player.papers;
    // the door (as `use_exit_door` calls it)
    g.next_level();
    assert_eq!(g.state, "ending");
    g.now_ms += 2000.0;
    g.run_scheduled();
    assert_eq!(g.live_menu(), Some(game::ui::menu::Screen::Ending), "no card at the end of her story");
    assert_eq!(g.menu.heading, "The last camp");
    assert!(g.menu.note.contains("1 of her 13 pages"), "{}", g.menu.note);
    println!("{}\n{}", g.menu.heading, g.menu.note);
    g.on_key_down("KeyE");
    g.now_ms += 500.0;
    g.run_scheduled();
    assert_eq!((g.state.as_str(), g.level_num), ("playing", LAST_LEVEL + 1));
    assert!(g.arc_done, "her story did not end");
    assert_eq!(g.player.papers, papers + HER_PAPER);
    assert!(g.live_menu().is_none(), "her card stayed up");
    assert!(g.level_card.is_some(), "no level card below her last camp");
    assert!(g.story.camp.is_none() && g.story.trail.is_empty(), "a camp of hers below the last");
    let saved = g.load_game().expect("saved");
    assert!(saved.arc_done && saved.level_num == LAST_LEVEL + 1);
    // the next door just leads down
    g.next_level();
    assert_eq!(g.state, "transition");
    // and the end comes once: back on the last camp's floor, with it done
    g.arc_done = true;
    g.state = "playing".into();
    g.init_level(LAST_LEVEL);
    g.next_level();
    assert_eq!(g.state, "transition", "her story ended twice");
}

/// The Reader reads the paper you drop and the paper lying about, and never
/// touches her folded sheets nor the flat ones (`docs/design/ENTITIES.md`, E6):
/// stood beside each of them on a floor that has both, it does not stop to
/// read, and every sheet ends as it does on the same floor without it.
#[test]
fn the_reader_leaves_her_sheets_alone() {
    use game::entities::brain::Mode;
    let run = 4242u32;
    let book = game::story::notebook::notebook(run);
    let level = book.iter().find(|(_, p)| p.id == "f7").unwrap().0;
    // the floor played the same way twice: the Reader beside each sheet in
    // turn, or kept far off; the sheets as they end
    let play_it = |beside_them: bool| {
        game::base::rng::seed_native(0x5EE7);
        let mut g = new_game();
        g.run_seed = run;
        g.init_level(level);
        g.state = "playing".into();
        g.god_mode = true;
        let mut sheets: Vec<(f64, f64, engine::scene::Obj)> =
            g.story.trail.iter().filter_map(|m| m.g.clone().map(|o| (m.x, m.z, o))).collect();
        for f in &g.story.false_trails {
            sheets.extend(f.trail.iter().filter_map(|m| m.g.clone().map(|o| (m.x, m.z, o))));
        }
        assert!(!sheets.is_empty() && !g.story.false_trails.is_empty(), "L{level}: her sheets and the flat ones");
        let ei = match g.entities.iter().position(|e| e.kind == "reader") {
            Some(i) => i,
            None => g.add_entity("reader", g.tx(sheets[0].0), g.tz(sheets[0].1), 0),
        };
        g.entities[ei].freeze = 0.0;
        let mut beside = 0;
        for (x, z, _) in sheets.clone() {
            let (tx, tz) = if beside_them { g.open_point(x + 1.0, z) } else { (-1e4, -1e4) };
            g.entities[ei].g.set_pos(tx, 0.0, tz);
            // (where no other paper lies in its reach, it must not stop at all)
            let alone = beside_them && game::entities::reader::paper_near(&g, ei).is_none();
            let n = g.creature(ei).unwrap().brain.log.len();
            play(&mut g, 2.0);
            if alone {
                beside += 1;
                let log = &g.creature(ei).unwrap().brain.log;
                assert!(log[n.min(log.len())..].iter().all(|c| c.2 != Mode::Read), "it stopped to read her sheet at ({x:.1}, {z:.1})");
            }
        }
        let state: Vec<_> = sheets.iter().map(|(_, _, o)| (o.visible(), o.parent().is_some(), o.pos().x, o.pos().y, o.pos().z)).collect();
        (state, beside, sheets.len())
    };
    let (with, beside, n) = play_it(true);
    let (without, _, _) = play_it(false);
    assert!(beside > 0, "it was never alone with a sheet of hers");
    assert_eq!(with, without, "her sheets did not end as they do without it");
    println!(
        "L{level}: stood beside {beside} of {n} sheets of hers and the flat ones; it never stopped for them, and they ended as without it"
    );
}
