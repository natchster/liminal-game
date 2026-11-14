//! Pacing: does the director give a level a rhythm? A bot that plays like a
//! cautious player (explores; runs from anything hunting it; crouches and
//! keeps still while something searches close by) plays whole levels with
//! death on, and the director's phases are logged:
//!
//! * the quiet is followed by a build-up within a couple of minutes;
//! * something reaches you, and a player who keeps away gets the creatures'
//!   withdrawal (a close call survived), not only death;
//! * the quiet and the relief between peaks stay within their lengths.
//!
//! On the Building's floors (levels 1–3) nothing walks: the director runs
//! the self-assembly (`game::world::assembly`). There a bot crosses each
//! floor to its door at the playthrough's pace (walking, sprinting while it
//! has the breath) and, when the siren sounds, goes to the nearest cell,
//! turns the wheel, waits for the all-clear and walks on: the phases are
//! logged, with how many sirens a floor meets on the way down.
//!
//!   cargo test --release -p backrooms --test pacing -- --nocapture
use game::Game;
use game::entities::director::Phase;
use game::world::assembly::{CREST, QUIET, Stage, WARNING};

#[derive(Default, Debug)]
struct Log {
    /// (phase, seconds spent in it)
    spans: Vec<(Phase, f64)>,
    first_build: Option<f64>,
    peaks: u32,
    /// peaks that ended with you alive (they withdrew, or lost you)
    survived: u32,
    withdrawals: u32,
    deaths: u32,
    causes: Vec<String>,
    /// Your asides said (what, and the director's phase then), and the
    /// ones let go (what and why)
    said: Vec<(String, Phase)>,
    dropped: Vec<String>,
}

fn key(g: &mut Game, code: &str, down: bool) {
    if down {
        g.keys.insert(code.to_string());
    } else {
        g.keys.remove(code);
    }
}

/// The cautious player's choice of keys and heading this frame.
fn steer(g: &mut Game, turn_t: &mut f64, frame: usize) {
    let p = g.player.pos;
    let near = |state: &str, g: &Game| {
        g.entities
            .iter()
            .filter(|e| e.state == state && e.kind != "watcher")
            .map(|e| {
                let q = e.pos();
                (((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt(), q)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    };
    let free = |g: &Game, yaw: f64, d: f64| {
        let (fx, fz) = (-yaw.sin(), -yaw.cos());
        !g.circle_hits_wall(p.x + fx * d, p.z + fz * d, 0.35) && !g.circle_hits_wall(p.x + fx * d * 0.5, p.z + fz * d * 0.5, 0.35)
    };
    for k in ["KeyW", "KeyS", "KeyA", "KeyD", "ShiftLeft", "KeyC"] {
        key(g, k, false);
    }
    let smiler_hunts = g.entities.iter().any(|e| e.state == "hunt" && e.kind == "smiler");
    // a Watcher close by and in sight: keep your eyes on it and back off
    let watcher = g
        .entities
        .iter()
        .filter(|e| e.kind == "watcher")
        .map(|e| e.pos())
        .find(|q| ((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt() < 12.0 && g.los(p.x, p.z, q.x, q.z));
    if let Some(q) = watcher {
        g.player.yaw = (-(q.x - p.x)).atan2(-(q.z - p.z));
        key(g, "KeyS", true);
    } else if let (true, Some((_, q))) = (smiler_hunts, near("hunt", g).filter(|h| h.0 < 30.0)) {
        // the Smiler: hold its gaze and back away, sidestepping walls
        let yaw = (-(q.x - p.x)).atan2(-(q.z - p.z));
        g.player.yaw = yaw;
        key(g, "KeyS", true);
        let back = yaw + std::f64::consts::PI;
        if !free(g, back, 1.5) {
            let left = free(g, back + 0.9, 1.5) || free(g, yaw + std::f64::consts::FRAC_PI_2, 1.5);
            key(g, if left { "KeyA" } else { "KeyD" }, true);
        }
    } else if let Some((d, q)) = near("hunt", g).filter(|h| h.0 < 30.0) {
        // run: the open heading that leads furthest from it
        let away = (-(p.x - q.x)).atan2(-(p.z - q.z));
        let mut best = (f64::MIN, g.player.yaw);
        for k in 0..16 {
            let yaw = away + (k as f64 - 7.5) * std::f64::consts::PI / 8.0;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let score = if free(g, yaw, 2.5) { 1.0 } else { -2.0 } + (fx * (p.x - q.x) + fz * (p.z - q.z)) / d.max(0.1);
            if score > best.0 {
                best = (score, yaw);
            }
        }
        g.player.yaw = best.1;
        key(g, "KeyW", true);
        key(g, "ShiftLeft", g.player.stamina > 15.0);
    } else if near("search", g).is_some_and(|s| s.0 < 14.0) {
        // something is looking: get low and keep still
        key(g, "KeyC", true);
    } else if frame % 600 < 50 {
        // now and then, a look behind
        if frame.is_multiple_of(600) {
            g.player.yaw += std::f64::consts::PI;
        } else if frame % 600 == 49 {
            g.player.yaw -= std::f64::consts::PI;
        }
    } else {
        // explore: walk, turn at walls and now and then
        *turn_t -= 1.0 / 60.0;
        if *turn_t <= 0.0 || !free(g, g.player.yaw, 1.2) {
            for k in 1..8 {
                let yaw = g.player.yaw + k as f64 * 0.9 * if frame.is_multiple_of(2) { 1.0 } else { -1.0 };
                if free(g, yaw, 2.0) {
                    g.player.yaw = yaw;
                    break;
                }
            }
            *turn_t = 4.0 + (frame % 5) as f64;
        }
        key(g, "KeyW", true);
    }
    // a lamp, as a player would carry one lit in the dark
    g.player.flashlight_on = true;
    g.player.battery = g.player.battery.max(50.0);
}

fn play_level(seed: u32, level: i32, secs: f64) -> Log {
    // the per-frame stream seeded too, so a run is the same each time
    // (`PACING_NATIVE` shifts it, to see the spread)
    let shift: u64 = std::env::var("PACING_NATIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    game::base::rng::seed_native(u64::from(seed) * 31 + level as u64 + shift * 1_000_003);
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    let mut log = Log::default();
    let (mut phase, mut since) = (g.director.phase, 0.0);
    let mut turn_t = 0.0;
    let mut t = 0.0;
    let mut calls = 0;
    let mut heard = 0;
    for frame in 0..(secs * 60.0) as usize {
        if g.director.close_calls != calls {
            log.survived += g.director.close_calls.saturating_sub(calls);
            calls = g.director.close_calls;
        }
        if g.state == "dead" {
            log.deaths += 1;
            let killer = g.director.killer.map(|k| g.entities[k].kind.clone()).unwrap_or_default();
            log.causes.push(format!("{}:{}@{:.0}s san{:.0}", g.last_death_cause.clone().unwrap_or_default(), killer, t, g.player.sanity));
            log.spans.push((phase, t - since));
            g.restart();
            calls = g.director.close_calls;
            phase = g.director.phase;
            since = t;
        }
        steer(&mut g, &mut turn_t, frame);
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
        t += 1.0 / 60.0;
        for (_, w) in &g.asides.log[heard..] {
            if let Some(what) = w.strip_prefix("say:") {
                log.said.push((what.to_string(), g.director.phase));
            } else if w.starts_with("drop:") {
                log.dropped.push(w.clone());
            }
        }
        heard = g.asides.log.len();
        let now = g.director.phase;
        if now != phase && std::env::var_os("PACING_DEBUG").is_some() {
            let st = g.director.stalker.map(|i| {
                let q = g.entities[i].pos();
                format!(
                    "{} {} {:.0}m",
                    g.entities[i].kind,
                    g.entities[i].state,
                    ((q.x - g.player.pos.x).powi(2) + (q.z - g.player.pos.z).powi(2)).sqrt()
                )
            });
            let ents: Vec<String> = g
                .entities
                .iter()
                .map(|e| {
                    let q = e.pos();
                    format!("{}:{}:{:.0}", e.kind, e.state, ((q.x - g.player.pos.x).powi(2) + (q.z - g.player.pos.z).powi(2)).sqrt())
                })
                .collect();
            println!("    t={t:.0} {} -> {} stalker {st:?} push {} ents {ents:?}", phase.name(), now.name(), g.director.final_push);
        }
        if now != phase {
            log.spans.push((phase, t - since));
            match now {
                Phase::Build if log.first_build.is_none() => log.first_build = Some(t),
                Phase::Peak => log.peaks += 1,
                Phase::Relax if phase == Phase::Peak && g.state == "playing" && g.entities.iter().any(|e| e.n("retreatT") > 0.0) => {
                    log.withdrawals += 1;
                }
                _ => {}
            }
            phase = now;
            since = t;
        }
    }
    if std::env::var("PACING_DEBUG").is_ok() {
        for (i, e) in g.entities.iter().enumerate() {
            if let Some(c) = g.creature(i) {
                let tail: Vec<String> =
                    c.brain.log.iter().rev().take(12).rev().map(|x| format!("{:.0} {}({})", x.0, x.2.name(), x.3)).collect();
                println!("  {} #{i}: {} changes; last: {}", e.kind, c.brain.log.len(), tail.join(", "));
            } else {
                println!("  {} #{i} (old AI)", e.kind);
            }
        }
        println!("  director: {:?}", g.director.log.iter().rev().take(10).collect::<Vec<_>>());
    }
    log
}

#[test]
fn the_director_paces_a_level() {
    // (each old scenario's depth on the rooms, the Building's three floors
    // over them: its level + 3, or the next ordinary floor after it when an
    // anomaly has that level)
    let runs = [(4242u32, 6), (77, 8), (9001, 11), (31337, 8)];
    let mut logs = vec![];
    for (seed, level) in runs {
        let log = play_level(seed, level, 300.0);
        let spans: Vec<String> = log.spans.iter().map(|(p, s)| format!("{} {s:.0}s", p.name())).collect();
        println!(
            "seed {seed} L{level}: first build at {:?} s, {} peaks, {} survived ({} by withdrawal), {} deaths\n  {}",
            log.first_build.map(|t| t.round()),
            log.peaks,
            log.survived,
            log.withdrawals,
            log.deaths,
            spans.join(" → ")
        );
        println!("  deaths: {:?}", log.causes);
        println!("  asides: {:?}", log.said.iter().map(|(w, p)| format!("{w} ({})", p.name())).collect::<Vec<_>>());
        println!("  let go: {:?}", log.dropped);
        logs.push(log);
    }
    // your asides keep the director's rhythm (docs/design/VOICES.md, rule 4):
    // nothing chosen is said in a peak (a blow's grunt is not chosen), and
    // the relief after a close call is said in the relief
    for (log, (seed, level)) in logs.iter().zip(runs) {
        for (w, phase) in &log.said {
            if !w.starts_with("knocked:") {
                assert!(*phase != Phase::Peak, "seed {seed} L{level}: \"{w}\" said in a peak");
            }
            if w.starts_with("aftermath:") {
                assert!(*phase == Phase::Relax, "seed {seed} L{level}: the relief said in {}", phase.name());
            }
        }
    }
    let said: Vec<&String> = logs.iter().flat_map(|l| l.said.iter().map(|(w, _)| w)).collect();
    assert!(said.iter().any(|w| w.starts_with("arrive:")), "a floor's arrival is said: {said:?}");
    assert!(
        said.iter().any(|w| w.starts_with("sighted:") || w.starts_with("aftermath:") || w.starts_with("knocked:")),
        "what comes is answered: {said:?}"
    );
    for (log, (seed, level)) in logs.iter().zip(runs) {
        let fb = log.first_build.unwrap_or(f64::INFINITY);
        assert!(fb < 150.0, "seed {seed} L{level}: the first build-up comes within 2.5 minutes ({fb:.0} s)");
        for (p, s) in &log.spans {
            match p {
                Phase::Calm => assert!(*s <= 95.0, "seed {seed} L{level}: a calm of {s:.0} s"),
                Phase::Relax => assert!(*s <= 50.0, "seed {seed} L{level}: a relief of {s:.0} s"),
                _ => {}
            }
        }
    }
    assert!(logs.iter().map(|l| l.peaks).sum::<u32>() >= 4, "something reaches you");
    assert!(logs.iter().map(|l| l.survived).sum::<u32>() >= 2, "keeping away is rewarded: close calls survived");
    assert!(logs.iter().map(|l| l.withdrawals).sum::<u32>() >= 1, "the director calls a hunt off");
}

/// A floor of the Building crossed: the phases, the sirens met before the
/// door, and how the bot fared. At the door in a quiet it stays on the
/// floor, waiting each siren out in a cell (the log goes on); at the door
/// with the siren sounding, it goes through.
#[derive(Default, Debug)]
struct Crossing {
    spans: Vec<(Phase, f64)>,
    /// seconds into the floor the door was reached (None: it was not)
    door_at: Option<f64>,
    /// seconds of it spent walking (not sealed in or waiting)
    walked: f64,
    /// when each siren began, and how long after it the bot was sealed in
    sirens: Vec<f64>,
    sealed_after: Vec<f64>,
    deaths: u32,
    /// Your asides said on the way (what, and whether you were sealed in)
    said: Vec<(String, bool)>,
    said_seen: usize,
}
impl Crossing {
    /// the sirens that began before the door was reached
    fn met(&self) -> usize {
        self.sirens.iter().filter(|&&s| self.door_at.is_none_or(|d| s < d)).count()
    }
}

/// The way over the floor's open tiles from (x, z) to `to` (tile centres)
fn tile_way(g: &Game, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    let (w, h) = (g.w, g.h);
    let mut prev = vec![None; (w * h) as usize];
    let idx = |x: i32, z: i32| (z * w + x) as usize;
    prev[idx(from.0, from.1)] = Some(from);
    let mut q = std::collections::VecDeque::from([from]);
    while let Some((x, z)) = q.pop_front() {
        if (x, z) == to {
            break;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if nx >= 0 && nz >= 0 && nx < w && nz < h && g.tiles[idx(nx, nz)] == 0 && prev[idx(nx, nz)].is_none() {
                prev[idx(nx, nz)] = Some((x, z));
                q.push_back((nx, nz));
            }
        }
    }
    prev[idx(to.0, to.1)]?;
    let mut way = vec![to];
    let mut c = to;
    while c != from {
        c = prev[idx(c.0, c.1)]?;
        way.push(c);
    }
    way.reverse();
    Some(way)
}

/// One step toward the centre of tile `t`: true once there
fn walk_toward(g: &mut Game, t: (i32, i32)) -> bool {
    let (tx, tz) = (g.wx(t.0), g.wz(t.1));
    let (dx, dz) = (tx - g.player.pos.x, tz - g.player.pos.z);
    if dx.hypot(dz) < 0.4 {
        return true;
    }
    g.player.yaw = (-dx).atan2(-dz);
    g.player.pitch = 0.0;
    key(g, "KeyW", true);
    // (as the playthrough's bot: a sprint while there is the breath for it)
    key(g, "ShiftLeft", g.player.stamina > 25.0);
    false
}

fn cross_floor(seed: u32, level: i32, secs: f64) -> Crossing {
    let shift: u64 = std::env::var("PACING_NATIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    game::base::rng::seed_native(u64::from(seed) * 31 + level as u64 + shift * 1_000_003);
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    assert!(g.building.is_some(), "seed {seed} L{level}: a floor of the Building");
    let plan = g.building.clone().unwrap();
    let mut c = Crossing::default();
    let (mut phase, mut since) = (g.director.phase, 0.0);
    let (mut t, mut stage) = (0.0, g.assembly.stage);
    // what the bot is doing: walking to the door, to a cell, sealed in
    let mut sealed: Option<usize> = None;
    let mut to_cell: Option<usize> = None;
    let mut way: Vec<(i32, i32)> = vec![];
    let mut at = 0usize;
    let mut stuck = 0usize;
    let here = |g: &Game| (g.tx(g.player.pos.x), g.tz(g.player.pos.z));
    for _ in 0..(secs * 60.0) as usize {
        for k in ["KeyW", "ShiftLeft"] {
            key(&mut g, k, false);
        }
        // the siren: to the nearest cell, unless the door is nearer (once
        // there, it waits each one out in a cell: the log goes on)
        if g.assembly.stage == Stage::Siren && stage == Stage::Quiet {
            c.sirens.push(t);
            let from = here(&g);
            let door = if c.door_at.is_some() { usize::MAX } else { tile_way(&g, from, plan.exit).map_or(usize::MAX, |w| w.len()) };
            let near = plan
                .cells
                .iter()
                .enumerate()
                .filter_map(|(i, cl)| tile_way(&g, from, cl.tile).map(|w| (w.len(), i, w)))
                .min_by_key(|(n, i, _)| (*n, *i));
            if let Some((n, i, w)) = near.filter(|n| n.0 < door) {
                let _ = n;
                to_cell = Some(i);
                way = w;
                at = 0;
            }
        }
        // the all-clear: out, and on to the door
        if let (Some(i), Stage::Quiet) = (sealed, g.assembly.stage) {
            g.turn_cell_wheel(i, false);
            sealed = None;
            way.clear();
        }
        stage = g.assembly.stage;
        if sealed.is_none() && (c.door_at.is_none() || to_cell.is_some()) {
            if way.is_empty() {
                way = tile_way(&g, here(&g), plan.exit).unwrap_or_default();
                at = 0;
            }
            while at < way.len() && walk_toward(&mut g, way[at]) {
                at += 1;
                stuck = 0;
            }
            if at < way.len() {
                if c.door_at.is_none() {
                    c.walked += 1.0 / 60.0;
                }
                stuck += 1;
                assert!(stuck < 60 * 8, "seed {seed} L{level}: stuck at {:?} on the way to {:?}", here(&g), way[at]);
            } else if let Some(i) = to_cell.take() {
                // in the cell: the wheel turned
                g.turn_cell_wheel(i, true);
                sealed = Some(i);
                c.sealed_after.push(t - c.sirens.last().copied().unwrap_or(t));
                way.clear();
            } else {
                c.door_at = Some(t);
                way.clear();
                // the siren sounding, through the door and down: the floor is left
                if g.assembly.up() {
                    break;
                }
            }
        }
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
        t += 1.0 / 60.0;
        for (_, w) in g.asides.log.iter().skip(c.said_seen) {
            if let Some(what) = w.strip_prefix("say:") {
                c.said.push((what.to_string(), g.assembly.sealed.is_some()));
            }
        }
        c.said_seen = g.asides.log.len();
        if g.state == "dead" {
            c.deaths += 1;
            break;
        }
        if g.director.phase != phase {
            c.spans.push((phase, t - since));
            phase = g.director.phase;
            since = t;
        }
    }
    c.spans.push((phase, t - since));
    c
}

#[test]
fn the_building_paces_its_floors() {
    // the floors of the rooms' runs above, crossed for five minutes (the
    // log goes on past the door: how the event comes again)
    let mut runs = vec![];
    for seed in [4242u32, 77, 9001, 31337] {
        for level in 1..=3 {
            let c = cross_floor(seed, level, 300.0);
            let spans: Vec<String> = c.spans.iter().map(|(p, s)| format!("{} {s:.0}s", p.name())).collect();
            println!(
                "seed {seed} L{level}: the door at {} s ({:.0} s walked), {} siren(s) met on the way (at {:?} s; sealed in {:?} s after it), {} deaths\n  {}",
                c.door_at.map_or("-".into(), |d| format!("{d:.0}")),
                c.walked,
                c.met(),
                c.sirens.iter().map(|s| s.round()).collect::<Vec<_>>(),
                c.sealed_after.iter().map(|s| (s * 10.0).round() / 10.0).collect::<Vec<_>>(),
                c.deaths,
                spans.join(" → ")
            );
            println!("  asides: {:?}", c.said);
            runs.push((seed, level, c));
        }
    }
    let met = runs.iter().filter(|r| r.2.met() > 0).count();
    println!("{met} of {} floors met a siren on the way to the door", runs.len());
    for (seed, level, c) in &runs {
        assert_eq!(c.deaths, 0, "seed {seed} L{level}: a cell is always in reach of the warning");
        assert!(c.door_at.is_some(), "seed {seed} L{level}: the door was not reached");
        for s in &c.sealed_after {
            assert!(*s < WARNING, "seed {seed} L{level}: sealed in {s:.1} s after the siren (the warning is {WARNING} s)");
        }
        // the quiets between two, the siren, and the fog's stay, in their lengths
        for (p, s) in &c.spans[1..c.spans.len().saturating_sub(1)] {
            match p {
                Phase::Calm => assert!(*s >= QUIET.0 - 1.0 && *s <= QUIET.1 + 1.0, "seed {seed} L{level}: a quiet of {s:.0} s"),
                Phase::Build => assert!((s - WARNING).abs() < 1.0, "seed {seed} L{level}: a siren of {s:.0} s"),
                Phase::Peak => assert!(*s >= CREST.0 && *s <= 60.0, "seed {seed} L{level}: the fog up {s:.0} s"),
                _ => {}
            }
        }
    }
    // at the bot's pace, most floors meet a siren before their door
    assert!(met * 4 >= runs.len() * 3, "{met} of {} floors met a siren on the way to the door", runs.len());
    // your asides on the Building (docs/design/VOICES.md): the siren has its word
    // on some floors, and a knock at your cell is answered inside it (the
    // all-clear's word may come as you step out: it is posted sealed in)
    let said: Vec<&(String, bool)> = runs.iter().flat_map(|r| r.2.said.iter()).collect();
    assert!(said.iter().any(|(w, _)| w.starts_with("siren:")), "no siren answered on twelve floors: {said:?}");
    for (w, sealed) in &said {
        if w.starts_with("held:knock") {
            assert!(*sealed, "\"{w}\" said outside a cell");
        }
    }
}

/// Your asides from run to run (`docs/design/VOICES.md`, *Your asides*, rule 3;
/// asides A10): eight runs' floors played by the cautious bot, creatures
/// live and deaths on. What you say differs from run to run, is never a line
/// twice in a run, is a few words a floor (neither nothing nor a stream),
/// and none of what is chosen falls in a peak.
#[test]
fn your_asides_differ_from_run_to_run() {
    let seeds = [4242u32, 77, 9001, 31337, 1999, 7, 202, 555];
    let level = 7;
    let runs: Vec<(u32, Log)> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds.iter().map(|&seed| s.spawn(move || (seed, play_level(seed, level, 240.0)))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut said: Vec<Vec<String>> = vec![];
    for (seed, log) in &runs {
        let ids: Vec<String> = log.said.iter().map(|(w, _)| w.rsplit(':').next().unwrap_or_default().to_string()).collect();
        println!(
            "seed {seed} L{level}: {} said: {:?}",
            ids.len(),
            log.said.iter().map(|(w, p)| format!("{w} ({})", p.name())).collect::<Vec<_>>()
        );
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "seed {seed}: a line said twice: {ids:?}");
        for (w, phase) in &log.said {
            assert!(w.starts_with("knocked:") || *phase != Phase::Peak, "seed {seed}: \"{w}\" said in a peak");
        }
        said.push(ids);
    }
    let counts: Vec<usize> = said.iter().map(|s| s.len()).collect();
    let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
    // how alike two runs' words are (shared lines over all their lines)
    let mut overlap = vec![];
    for i in 0..said.len() {
        for j in i + 1..said.len() {
            let (a, b) = (&said[i], &said[j]);
            let both = a.iter().filter(|x| b.contains(x)).count() as f64;
            let all = (a.len() + b.len()) as f64 - both;
            if all > 0.0 {
                overlap.push(both / all);
            }
        }
    }
    let mean_overlap = overlap.iter().sum::<f64>() / overlap.len().max(1) as f64;
    let mut distinct = said.clone();
    distinct.sort();
    distinct.dedup();
    println!(
        "asides a run: {counts:?} (mean {mean:.1}); {} of {} runs say a different sequence; mean overlap between two runs {mean_overlap:.2}",
        distinct.len(),
        said.len()
    );
    assert!((1.5..=8.0).contains(&mean), "a few words a floor, not nothing nor a stream: {counts:?}");
    assert!(counts.iter().all(|&c| c >= 1), "a run says something: {counts:?}");
    assert!(distinct.len() >= 7, "runs say different things: {said:?}");
    assert!(mean_overlap < 0.35, "two runs share most of their words ({mean_overlap:.2})");
}
