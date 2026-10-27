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
//!   cargo test --release -p backrooms --test pacing -- --nocapture
use game::entities::director::Phase;
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
        if frame % 600 == 0 {
            g.player.yaw += std::f64::consts::PI;
        } else if frame % 600 == 49 {
            g.player.yaw -= std::f64::consts::PI;
        }
    } else {
        // explore: walk, turn at walls and now and then
        *turn_t -= 1.0 / 60.0;
        if *turn_t <= 0.0 || !free(g, g.player.yaw, 1.2) {
            for k in 1..8 {
                let yaw = g.player.yaw + k as f64 * 0.9 * if frame % 2 == 0 { 1.0 } else { -1.0 };
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
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    let mut log = Log::default();
    let (mut phase, mut since) = (g.director.phase, 0.0);
    let mut turn_t = 0.0;
    let mut t = 0.0;
    let mut calls = 0;
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
        logs.push(log);
    }
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
