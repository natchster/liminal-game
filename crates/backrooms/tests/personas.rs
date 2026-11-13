//! Bot personas and the analysis of their runs.
//!
//! Three players with different goals play the same levels with death on
//! (Holmgård's procedural personas): an **explorer** drawn to space it has not
//! seen (its coverage is written as a heatmap), a **rusher** who sprints for
//! the exit, and a **hider** who crouches in the darkest dead end near spawn.
//! They walk a 0.25 m grid of the places a body fits (`circle_hits_wall`),
//! so a stuck bot means a stuck player.
//!
//! Every run is held to the same oracles: never outside the world, never
//! stuck for long, the rusher reaches the exit, and no death in a level's
//! first seconds. Then the director's log of every run is analysed for the
//! shape of the fear (`docs/design/PILLARS.md` 5): how long a peak lasts, how long
//! the relief after it, whether the stalker's signs repeat, whether a sign
//! always leads to the same outcome, and how many stings a level plays.
//!
//!   cargo test --release -p backrooms --test personas -- --nocapture
//!
//! writes `target/personas/report.md` and the explorer heatmaps.
use game::base::config::PLAYER_R;
use game::entities::director::Phase;
use game::Game;
use std::collections::VecDeque;

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

/// (run seed, level): a maze level, a deeper one, an anomaly level.
// (rooms levels: the Building's floors above them have no creatures)
const LEVELS: &[(u32, i32)] = &[(4242, 6), (77, 8), (1999, 10)];
const SECONDS: f64 = 150.0;
const FINE: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Persona {
    Explorer,
    Rusher,
    Hider,
}

struct Run {
    persona: Persona,
    seed: u32,
    level: i32,
    died_at: Option<f64>,
    /// what killed you (cause, and the creature's kind)
    died_of: String,
    reached_exit: Option<f64>,
    stuck_for: f64,
    out_of_world: bool,
    coverage: f64,
    log: Vec<(f64, &'static str)>,
    end_t: f64,
}

/// Where a body fits, on a 0.25 m grid, and paths over it.
struct Grid {
    w: i32,
    h: i32,
    x0: f64,
    z0: f64,
    free: Vec<bool>,
}
impl Grid {
    fn new(g: &Game) -> Grid {
        let (w, h) = (g.w * FINE, g.h * FINE);
        let (x0, z0) = (g.wx(0) - 2.0, g.wz(0) - 2.0);
        let cs = 4.0 / FINE as f64;
        let mut free = vec![false; (w * h) as usize];
        for j in 0..h {
            for i in 0..w {
                let (x, z) = (x0 + (i as f64 + 0.5) * cs, z0 + (j as f64 + 0.5) * cs);
                free[(j * w + i) as usize] = !g.circle_hits_wall(x, z, PLAYER_R + 0.05);
            }
        }
        Grid { w, h, x0, z0, free }
    }
    fn cell(&self, x: f64, z: f64) -> (i32, i32) {
        (((x - self.x0) / 0.25).floor() as i32, ((z - self.z0) / 0.25).floor() as i32)
    }
    fn center(&self, c: (i32, i32)) -> (f64, f64) {
        (self.x0 + (c.0 as f64 + 0.5) * 0.25, self.z0 + (c.1 as f64 + 0.5) * 0.25)
    }
    fn ok(&self, c: (i32, i32)) -> bool {
        c.0 >= 0 && c.1 >= 0 && c.0 < self.w && c.1 < self.h && self.free[(c.1 * self.w + c.0) as usize]
    }
    /// The nearest free cell to `c`.
    fn snap(&self, c: (i32, i32)) -> Option<(i32, i32)> {
        for r in 0..12 {
            for dj in -r..=r {
                for di in -r..=r {
                    if self.ok((c.0 + di, c.1 + dj)) {
                        return Some((c.0 + di, c.1 + dj));
                    }
                }
            }
        }
        None
    }
    /// Breadth-first path from `a` to the first cell where `goal` holds.
    fn path(&self, a: (i32, i32), goal: impl Fn((i32, i32)) -> bool) -> Option<Vec<(i32, i32)>> {
        let a = self.snap(a)?;
        let mut prev = vec![u32::MAX; (self.w * self.h) as usize];
        let idx = |c: (i32, i32)| (c.1 * self.w + c.0) as usize;
        prev[idx(a)] = idx(a) as u32;
        let mut q = VecDeque::from([a]);
        while let Some(c) = q.pop_front() {
            if goal(c) {
                let mut out = vec![c];
                let mut k = idx(c);
                while prev[k] as usize != k {
                    k = prev[k] as usize;
                    out.push(((k as i32) % self.w, (k as i32) / self.w));
                }
                out.reverse();
                return Some(out);
            }
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + di, c.1 + dj);
                if self.ok(n) && prev[idx(n)] == u32::MAX {
                    prev[idx(n)] = idx(c) as u32;
                    q.push_back(n);
                }
            }
        }
        None
    }
}

fn press(g: &mut Game, code: &str, down: bool) {
    if down {
        g.keys.insert(code.to_string());
    } else {
        g.keys.remove(code);
    }
}

fn play(persona: Persona, seed: u32, level: i32) -> Run {
    game::base::rng::seed_native(u64::from(seed) * 31 + level as u64 + persona as u64 * 1_000_003);
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    g.state = "playing".into();
    g.hide_overlay();
    let grid = Grid::new(&g);
    let (tw, th) = (g.w, g.h);
    let mut visited = vec![false; (tw * th) as usize];
    let open_tiles = (0..th).flat_map(|z| (0..tw).map(move |x| (x, z))).filter(|&(x, z)| !g.is_wall_tile(x, z)).count().max(1);
    let exit = g.exit_data.as_ref().map(|e| (e.pos.x, e.pos.z));
    // the hider's hole: the darkest dead end within 40 m of spawn
    let hole = {
        let sp = (g.player.pos.x, g.player.pos.z);
        let mut best: Option<((i32, i32), f64)> = None;
        for z in 0..th {
            for x in 0..tw {
                if g.is_wall_tile(x, z) {
                    continue;
                }
                let n = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|(dx, dz)| !g.is_wall_tile(x + dx, z + dz)).count();
                let (wx, wz) = (g.wx(x), g.wz(z));
                let d = ((wx - sp.0).powi(2) + (wz - sp.1).powi(2)).sqrt();
                if n == 1 && d < 40.0 {
                    let score = d + 30.0 * g.region_light_at(x, z);
                    if best.is_none_or(|b| score < b.1) {
                        best = Some(((x, z), score));
                    }
                }
            }
        }
        best.map(|b| (g.wx(b.0 .0), g.wz(b.0 .1)))
    };
    let mut run = Run {
        persona,
        seed,
        level,
        died_at: None,
        died_of: String::new(),
        reached_exit: None,
        stuck_for: 0.0,
        out_of_world: false,
        coverage: 0.0,
        log: vec![],
        end_t: 0.0,
    };
    let (mut path, mut repath_t, mut t) = (Vec::<(i32, i32)>::new(), 0.0, 0.0);
    let (mut check_pos, mut check_t, mut stuck_now) = ((g.player.pos.x, g.player.pos.z), 0.0, 0.0);
    let mut settled = false;
    let mut lamp_t = 0.0;
    for _ in 0..(SECONDS * 60.0) as usize {
        let (px, pz) = (g.player.pos.x, g.player.pos.z);
        let (tx, tz) = (g.tx(px), g.tz(pz));
        for dz in -1..=1 {
            for dx in -1..=1 {
                if g.in_grid(tx + dx, tz + dz) {
                    visited[((tz + dz) * tw + tx + dx) as usize] = true;
                }
            }
        }
        if !px.is_finite() || !pz.is_finite() || !g.in_grid(tx, tz) || g.player.pos.y < -1.0 {
            run.out_of_world = true;
            break;
        }
        // where to go
        repath_t -= 1.0 / 60.0;
        if repath_t <= 0.0 || path.len() < 2 {
            repath_t = 1.0;
            let here = grid.cell(px, pz);
            path = match persona {
                Persona::Explorer => grid
                    .path(here, |c| {
                        let (x, z) = grid.center(c);
                        let (ttx, ttz) = (g.tx(x), g.tz(z));
                        g.in_grid(ttx, ttz) && !visited[(ttz * tw + ttx) as usize]
                    })
                    .unwrap_or_default(),
                Persona::Rusher => exit
                    .and_then(|e| {
                        let goal = grid.snap(grid.cell(e.0, e.1))?;
                        grid.path(here, |c| (c.0 - goal.0).abs() <= 2 && (c.1 - goal.1).abs() <= 2)
                    })
                    .unwrap_or_default(),
                Persona::Hider => hole
                    .and_then(|hpos| {
                        let goal = grid.snap(grid.cell(hpos.0, hpos.1))?;
                        grid.path(here, |c| (c.0 - goal.0).abs() <= 1 && (c.1 - goal.1).abs() <= 1)
                    })
                    .unwrap_or_default(),
            };
        }
        if persona == Persona::Rusher {
            if let Some(e) = exit.filter(|e| ((e.0 - px).powi(2) + (e.1 - pz).powi(2)).sqrt() < 2.5) {
                let _ = e;
                run.reached_exit = Some(t);
                g.try_interact();
                break;
            }
        }
        let arrived = path.len() < 2;
        if persona == Persona::Hider && arrived {
            settled = true;
        }
        // steer toward a point a metre down the path
        if let Some(&c) = path.get(4.min(path.len().saturating_sub(1))) {
            let (cx, cz) = grid.center(c);
            let (dx, dz) = (cx - px, cz - pz);
            if dx * dx + dz * dz > 1e-6 {
                g.player.yaw = (-dx).atan2(-dz);
            }
            if (dx * dx + dz * dz).sqrt() < 0.6 && path.len() > 4 {
                path.drain(..2);
            }
        }
        let walking = !(arrived || (persona == Persona::Hider && settled));
        press(&mut g, "KeyW", walking);
        let sprint = walking && persona == Persona::Rusher && g.player.stamina > 20.0;
        press(&mut g, "ShiftLeft", sprint);
        press(&mut g, "ControlLeft", persona == Persona::Hider && settled);
        // a lamp, as a player uses one: lit in the dark, out under the
        // panels (a second between switches); the hider puts it out once
        // settled
        lamp_t -= 1.0 / 60.0;
        if lamp_t <= 0.0 {
            let here = g.region_light_at(g.tx(g.player.pos.x), g.tz(g.player.pos.z));
            let want = match persona {
                Persona::Hider if settled => false,
                _ if here < 0.3 => true,
                _ if here > 0.5 => false,
                _ => g.player.flashlight_on,
            };
            if want != g.player.flashlight_on {
                g.toggle_flashlight();
                lamp_t = 1.0;
            }
        }
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
        if std::env::var("PERSONA_TRACE").map(|v| v == format!("{persona:?}:{seed}")).unwrap_or(false)
            && (t * 60.0).round() as i64 % 30 == 0
        {
            if let Some(si) = g.director.stalker {
                let p = g.perceive(si);
                let m = g.creature(si).map(|c| c.brain.mode.name()).unwrap_or("old");
                println!(
                    "  t {t:5.0} {} {m:8} d {:5.1} path {:5.1} line {} view {} you_lit {:.2} lamp {} phase {:?} push {} nerve {:.0}",
                    g.entities[si].kind,
                    p.dist,
                    p.path,
                    p.line,
                    p.in_view,
                    p.you_lit,
                    p.lamp,
                    g.director.phase,
                    g.director.final_push,
                    g.player.sanity
                );
            }
        }
        t += 1.0 / 60.0;
        // oracles
        check_t += 1.0 / 60.0;
        if check_t >= 3.0 {
            let moved = ((g.player.pos.x - check_pos.0).powi(2) + (g.player.pos.z - check_pos.1).powi(2)).sqrt();
            if walking && moved < 0.4 {
                stuck_now += check_t;
                run.stuck_for = run.stuck_for.max(stuck_now);
            } else {
                stuck_now = 0.0;
            }
            check_pos = (g.player.pos.x, g.player.pos.z);
            check_t = 0.0;
        }
        if g.state != "playing" {
            if g.state == "dead" || g.state == "gameover" || g.director.log.iter().any(|e| e.1 == "death") {
                run.died_at = Some(t);
                let killer = g.director.killer.map(|k| g.entities[k].kind.clone()).unwrap_or_default();
                run.died_of =
                    format!("{}{}{killer}", g.last_death_cause.clone().unwrap_or_default(), if killer.is_empty() { "" } else { ": " });
            } else if run.reached_exit.is_none() {
                run.reached_exit = Some(t);
            }
            break;
        }
        if g.level_num != level {
            run.reached_exit = Some(t);
            break;
        }
    }
    run.end_t = t;
    if std::env::var("PERSONA_DEBUG").is_ok() {
        println!("== {persona:?} {seed} L{level}");
        for (i, e) in g.entities.iter().enumerate() {
            let sent = if g.director.stalker == Some(i) { " (the stalker)" } else { "" };
            if let Some(c) = g.creature(i) {
                let tail: Vec<String> =
                    c.brain.log.iter().rev().take(14).rev().map(|x| format!("{:.0} {}({})", x.0, x.2.name(), x.3)).collect();
                println!("  {}{sent} #{i}: {}", e.kind, tail.join(", "));
            } else {
                println!("  {}{sent} #{i} (old AI) state {}", e.kind, e.state);
            }
        }
        let dl: Vec<String> = g.director.log.iter().map(|(t, w)| format!("{t:.0} {w}")).collect();
        println!("  director: {}", dl.join(", "));
    }
    run.coverage = visited.iter().filter(|v| **v).count() as f64 / open_tiles as f64;
    run.log = g.director.log.clone();
    if persona == Persona::Explorer {
        heatmap(&g, &visited, &format!("explorer_{seed}_L{level}"));
    }
    let _ = Phase::Calm;
    run
}

fn heatmap(g: &Game, visited: &[bool], name: &str) {
    let (w, h, s) = (g.w as usize, g.h as usize, 6usize);
    let mut px = vec![0u8; w * s * h * s * 4];
    for z in 0..h {
        for x in 0..w {
            let c = if g.is_wall_tile(x as i32, z as i32) {
                [24, 24, 24, 255]
            } else if visited[z * w + x] {
                [235, 200, 60, 255]
            } else {
                [90, 90, 90, 255]
            };
            for dy in 0..s {
                for dx in 0..s {
                    let i = ((z * s + dy) * w * s + x * s + dx) * 4;
                    px[i..i + 4].copy_from_slice(&c);
                }
            }
        }
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/personas");
    std::fs::create_dir_all(&dir).unwrap();
    engine::png::write_rgba(dir.join(format!("{name}.png")).to_str().unwrap(), (w * s) as u32, (h * s) as u32, &px).unwrap();
}

/// The shape of the fear in one run's director log.
#[derive(Debug, Default)]
struct Shape {
    peaks: usize,
    longest_peak: f64,
    shortest_relief: Option<f64>,
    cues: usize,
    cue_streak: usize,
    cues_then_peak: usize,
    stings: usize,
    notices: usize,
}

fn shape(log: &[(f64, &str)], end: f64) -> Shape {
    let mut s = Shape::default();
    let mut peak_at: Option<f64> = None;
    let mut relax_at: Option<f64> = None;
    let (mut last_cue, mut streak) = ("", 0usize);
    for (k, &(t, e)) in log.iter().enumerate() {
        match e {
            "peak" => {
                s.peaks += 1;
                peak_at = Some(t);
            }
            "relax" | "withdraw" | "death" => {
                if let Some(p) = peak_at.take() {
                    s.longest_peak = s.longest_peak.max(t - p);
                }
                if e != "death" {
                    relax_at = Some(t);
                }
            }
            "build" => {
                if let Some(r) = relax_at.take() {
                    s.shortest_relief = Some(s.shortest_relief.map_or(t - r, |x: f64| x.min(t - r)));
                }
            }
            "sting" => s.stings += 1,
            "notice" => s.notices += 1,
            _ => {}
        }
        if e.starts_with("cue:") {
            s.cues += 1;
            streak = if e == last_cue { streak + 1 } else { 1 };
            last_cue = e;
            s.cue_streak = s.cue_streak.max(streak);
            if log[k + 1..].iter().take_while(|x| x.0 - t < 20.0).any(|x| x.1 == "peak") {
                s.cues_then_peak += 1;
            }
        }
    }
    if let Some(p) = peak_at {
        s.longest_peak = s.longest_peak.max(end - p);
    }
    s
}

/// The longest a peak may last before the threat must back off.
const MAX_PEAK_S: f64 = 60.0;
/// The shortest relief (relax to the next build-up).
const MIN_RELIEF_S: f64 = 8.0;
/// The same sign of the stalker this many times in a row is monotony. The
/// director picks a sign by distance alone, so a stalker that stays far off
/// gives distant steps every time: up to 7 in a row in these runs (an open
/// design issue in docs/plan/QUALITY.md); this holds it from getting worse.
const MAX_CUE_STREAK: usize = 8;
/// Stings per level run (150 s): more and they stop meaning anything.
const MAX_STINGS: usize = 6;

#[test]
fn personas_play_and_the_fear_has_a_shape() {
    let mut runs = vec![];
    for &(seed, level) in LEVELS {
        for p in [Persona::Explorer, Persona::Rusher, Persona::Hider] {
            runs.push(play(p, seed, level));
        }
    }
    let mut problems = vec![];
    let mut report = String::from("# Persona runs\n\n| Run | Ended | Coverage | Stuck (max s) | Peaks | Longest peak | Shortest relief | Cues | Same cue in a row | Cues then a peak | Stings | Notices |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    let (mut all_cues, mut all_cue_peaks) = (0, 0);
    for r in &runs {
        let s = shape(&r.log, r.end_t);
        let name = format!("{:?} {} L{}", r.persona, r.seed, r.level);
        let ended = match (r.died_at, r.reached_exit) {
            (Some(t), _) => format!("died {t:.0} s ({})", r.died_of),
            (_, Some(t)) => format!("exit {t:.0} s"),
            _ => format!("{:.0} s", r.end_t),
        };
        report.push_str(&format!(
            "| {name} | {ended} | {:.0}% | {:.0} | {} | {:.0} s | {} | {} | {} | {} | {} | {} |\n",
            r.coverage * 100.0,
            r.stuck_for,
            s.peaks,
            s.longest_peak,
            s.shortest_relief.map(|x| format!("{x:.0} s")).unwrap_or("—".into()),
            s.cues,
            s.cue_streak,
            s.cues_then_peak,
            s.stings,
            s.notices
        ));
        all_cues += s.cues;
        all_cue_peaks += s.cues_then_peak;
        if r.out_of_world {
            problems.push(format!("{name}: left the world"));
        }
        if r.stuck_for > 12.0 {
            problems.push(format!("{name}: stuck for {:.0} s (a place a body cannot get out of?)", r.stuck_for));
        }
        if r.persona == Persona::Rusher && r.reached_exit.is_none() && r.died_at.is_none() {
            problems.push(format!("{name}: never reached the exit in {SECONDS} s"));
        }
        if r.died_at.is_some_and(|t| t < 20.0) {
            problems.push(format!("{name}: died {:.0} s into the level (no chance to react)", r.died_at.unwrap()));
        }
        if s.longest_peak > MAX_PEAK_S {
            problems.push(format!("{name}: a peak lasted {:.0} s (over {MAX_PEAK_S} s: the threat must back off)", s.longest_peak));
        }
        if let Some(x) = s.shortest_relief.filter(|x| *x < MIN_RELIEF_S) {
            problems.push(format!("{name}: a relief of only {x:.0} s"));
        }
        if s.cue_streak > MAX_CUE_STREAK {
            problems.push(format!("{name}: the same sign {} times in a row", s.cue_streak));
        }
        if s.stings > MAX_STINGS {
            problems.push(format!("{name}: {} stings in one level", s.stings));
        }
    }
    // a sign of the stalker must not always mean the same thing
    let share = all_cue_peaks as f64 / all_cues.max(1) as f64;
    report.push_str(&format!(
        "\nAcross all runs, {all_cue_peaks} of {all_cues} signs of the stalker were followed by a peak within 20 s ({:.0}%).\n",
        share * 100.0
    ));
    if all_cues >= 10 && !(0.05..=0.95).contains(&share) {
        problems.push(format!(
            "signs of the stalker lead to a peak {:.0}% of the time: a sign that always (or never) means a chase is a pattern, not dread",
            share * 100.0
        ));
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/personas");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("report.md"), &report).unwrap();
    println!("{report}");
    assert!(problems.is_empty(), "{problems:#?}");
}
