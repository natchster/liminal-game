//! Beatable (`docs/ENTITIES.md` design 7). A **rule-follower**, a bot that
//! knows every creature's counter and plays it from what a player perceives
//! (as the arenas do), plays levels 4–13 of several runs as a player follows
//! her story (to her camp, her page taken, which wakes the place; then to
//! the door) with the creatures live and deaths on: it must finish at least
//! 90% of them. The explorer (it wanders two minutes, then makes for the
//! door) and the rusher (straight for the door), who know no counter, must
//! get through some (they finish nearly as often: a contact mostly knocks
//! you down; E8 measured it). Each run's levels are played in order, and
//! what the run remembers of the creatures goes on from level to level (who
//! you have met: a first meeting does not kill, later ones may; what they
//! have learned of your habits).
//! In every run no death may break the **hard rules**: a death with fewer
//! than two tells you could perceive, at a first meeting, in a level's first
//! 30 s, or after a chase faster than the cap. And the dark-zone tension is
//! measured: how long a nerve sat spent in the dark, and what came of it
//! (the dark itself takes it no lower than `player::DARK_FLOOR`).
//!
//!   cargo test --release -p backrooms --test beatable -- --nocapture
//!   BEATABLE_TRACE=seed:level   (the rule-follower's choices on one level;
//!                                `one_level` plays it as the sweep did)
//!   BEATABLE_FROM=s             (with it: from s on, the creatures near and
//!                                where the bot steers)
//!
//! The Showroom and the Borrowed House keep their residents' own rules until
//! E9 (the playthrough leaves them in god mode); the levels with the
//! Stairwell's or the Loop's door are walked to their own door.
use game::base::config::{PLAYER_R, SPRINT};
use game::entities::creature::RunMemory;
use game::Game;
mod rule_follower;
use rule_follower::*;
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

const SEEDS: &[u32] = &[4242, 77, 9001, 1999];
const LEVELS: std::ops::RangeInclusive<i32> = 4..=13;
const SECONDS: f64 = 480.0;
const FINE: i32 = 16;
const DT: f64 = 1.0 / 60.0;

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
    /// Whether a body walks straight from `a` to `b` (every cell on the way free).
    fn clear(&self, a: (f64, f64), b: (f64, f64)) -> bool {
        let n = (((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() / 0.1).ceil().max(1.0) as i32;
        (0..=n).all(|k| {
            let f = k as f64 / n as f64;
            self.ok(self.cell(a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f))
        })
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Bot {
    /// knows every counter; her camp, then the door
    RuleFollower,
    /// knows nothing: wanders two minutes, then makes for the door
    Explorer,
    /// knows nothing: runs for the door
    Rusher,
}

#[derive(Default, Debug)]
struct Outcome {
    finished: bool,
    died: Option<(f64, String)>,
    /// broken hard rules, as text
    fails: Vec<String>,
    /// seconds with the nerve under 25 in the dark with the lamp out
    dark_spent: f64,
    /// a death on a spent nerve after time in the dark (under 25, 5 s of
    /// dark in the last 30)
    dark_death: bool,
    /// of `dark_spent`: the lamp out by a counter, by a dead battery
    dark_counter: f64,
    dark_battery: f64,
    /// why a level was not finished (no death)
    unfinished: String,
    /// the director's peaks, and the creatures' contacts
    peaks: usize,
    contacts: usize,
}

/// Plays one level; `memory` is what the run remembers of the creatures
/// (who you have met, your habits, what they have learned), carried from
/// the levels before as a run carries it, and on to the next.
fn play(bot: Bot, seed: u32, level: i32, memory: &mut RunMemory) -> Option<Outcome> {
    game::base::rng::seed_native(u64::from(seed) * 7919 + level as u64 * 31 + bot as u64);
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    if g.building.is_some() || matches!(g.level_scp.as_deref(), Some("store" | "house")) {
        return None;
    }
    g.creature_run = memory.clone();
    g.creature_learn_pending();
    g.state = "playing".into();
    g.hide_overlay();
    g.god_mode = false;
    let trace = std::env::var("BEATABLE_TRACE").map(|v| v == format!("{seed}:{level}")).unwrap_or(false) && bot == Bot::RuleFollower;
    // (BEATABLE_FROM=s: from then on, more of the trace)
    let from = std::env::var("BEATABLE_FROM").ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(f64::MAX);
    let grid = Grid::new(&g);
    let exit = g.exit_data.as_ref().map(|e| (e.pos.x, e.pos.z))?;
    let mut out = Outcome::default();
    let (mut path, mut repath_t, mut t) = (Vec::<(i32, i32)>::new(), 0.0, 0.0);
    let (mut lamp_t, mut last_drop, mut why) = (0.0, -10.0, "");
    let mut dark_recent: VecDeque<(f64, bool)> = VecDeque::new();
    let mut visited = vec![false; (g.w * g.h) as usize];
    let mut detour: Option<(f64, f64, f64)> = None;
    let mut given_up: Vec<(f64, f64)> = vec![];
    let camp = g.story.camp.as_ref().map(|c| (c.at.x, c.at.z));
    // how long each creature has been over the chase cap
    let mut over = vec![0.0; 64];
    let cap = game::entities::creature::CHASE_CAP + 0.15;
    for _ in 0..(SECONDS * 60.0) as usize {
        let (px, pz) = (g.player.pos.x, g.player.pos.z);
        if ((exit.0 - px).powi(2) + (exit.1 - pz).powi(2)).sqrt() < 2.5 {
            g.try_interact();
        }
        // the lamp's battery, as a player keeps it: a spare in when it runs
        // low, and one picked up on the way when there is no spare
        if bot == Bot::RuleFollower && g.player.battery < 10.0 && g.player.batteries > 0 {
            g.swap_battery();
        }
        // a detour for what it lacks: paper (with a Reader on the level) from
        // a pile within 30 m, a battery within 25 m when it has no spare; the
        // nearest, and once chosen kept until taken, or given up after 30 s
        // (taking the first in reach, it went to and fro between two)
        let reader_here = g.entities.iter().any(|e| e.kind == "reader");
        let mut wanted: Vec<(f64, f64, f64)> = vec![];
        if bot == Bot::RuleFollower && reader_here && g.player.papers < 4 {
            wanted.extend(g.papers.iter().filter(|p| !p.taken).map(|p| (p.g.pos().x, p.g.pos().z, 30.0)));
        }
        if bot == Bot::RuleFollower && g.player.batteries == 0 && g.player.battery < 40.0 {
            wanted.extend(g.battery_items.iter().filter(|b| !b.taken).map(|b| (b.g.pos().x, b.g.pos().z, 25.0)));
        }
        if let Some((dx, dz, since)) = detour {
            let still_there = wanted.iter().any(|q| (q.0 - dx).abs() < 0.01 && (q.1 - dz).abs() < 0.01);
            if !still_there || t - since > 30.0 {
                if still_there {
                    given_up.push((dx, dz));
                }
                detour = None;
            }
        }
        if detour.is_none() {
            let dist = |q: &(f64, f64, f64)| ((q.0 - px).powi(2) + (q.1 - pz).powi(2)).sqrt();
            detour = wanted
                .iter()
                .filter(|q| dist(q) < q.2 && !given_up.iter().any(|u| (u.0 - q.0).abs() < 0.01 && (u.1 - q.1).abs() < 0.01))
                .min_by(|a, b| dist(a).total_cmp(&dist(b)))
                .map(|q| (q.0, q.1, t));
        }
        let pile = detour.map(|(x, z, _)| (x, z));
        let (tx0, tz0) = (g.tx(px), g.tz(pz));
        for dz in -1..=1 {
            for dx in -1..=1 {
                if g.in_grid(tx0 + dx, tz0 + dz) {
                    visited[g.t_idx(tx0 + dx, tz0 + dz)] = true;
                }
            }
        }
        // her page, as she meant it to be found
        let page_left = g.story.camp.as_ref().is_some_and(|c| c.page_id.is_some() && !c.page_taken);
        if bot == Bot::RuleFollower && page_left {
            if let Some((cx, cz)) = camp {
                if ((cx - px).powi(2) + (cz - pz).powi(2)).sqrt() < 1.6 {
                    g.take_page();
                }
            }
        }
        let exploring = bot == Bot::Explorer && t < 120.0;
        repath_t -= DT;
        let c = if bot == Bot::RuleFollower {
            counter(&g, last_drop, t)
        } else {
            Counter {
                gait: if bot == Bot::Rusher { Gait::Run } else { Gait::Walk },
                face: None,
                away_from: None,
                lamp: None,
                paper: false,
                why: "",
            }
        };
        if repath_t <= 0.0 || path.len() < 2 || c.away_from.is_some() {
            repath_t = 1.0;
            let here = grid.cell(px, pz);
            let goal_pos = if let Some((ax, az)) = c.away_from {
                // away from it, toward the door where that is away too
                let (dx, dz) = (px - ax, pz - az);
                let l = (dx * dx + dz * dz).sqrt().max(1e-6);
                Some((px + dx / l * 6.0, pz + dz / l * 6.0))
            } else if exploring {
                None
            } else if bot == Bot::RuleFollower && page_left {
                camp
            } else {
                Some(pile.unwrap_or(exit))
            };
            path = match goal_pos {
                Some((gx, gz)) => grid
                    .snap(grid.cell(gx, gz))
                    .and_then(|goal| grid.path(here, |q| (q.0 - goal.0).abs() <= 2 && (q.1 - goal.1).abs() <= 2))
                    .unwrap_or_default(),
                // the explorer: the nearest place it has not been
                None => grid
                    .path(here, |q| {
                        let (x, z) = grid.center(q);
                        let (ttx, ttz) = (g.tx(x), g.tz(z));
                        g.in_grid(ttx, ttz) && !visited[g.t_idx(ttx, ttz)]
                    })
                    .unwrap_or_default(),
            };
        }
        if let Some(&q) = path.get(4.min(path.len().saturating_sub(1))) {
            let (cx, cz) = grid.center(q);
            if ((cx - px).powi(2) + (cz - pz).powi(2)).sqrt() < 0.6 && path.len() > 4 {
                path.drain(..2);
            }
        }
        // the farthest of the next four points a body walks to straight (the
        // line to one round a corner cuts it, and a body pushed into an
        // inside corner moves on neither axis: one bot stood there 400 s)
        let next = (1..=4.min(path.len().saturating_sub(1)))
            .rev()
            .map(|k| grid.center(path[k]))
            .find(|&(qx, qz)| grid.clear((px, pz), (qx, qz)))
            .or_else(|| path.get(1).map(|&q| grid.center(q)));
        let gait = if bot == Bot::Rusher { Gait::Run } else { c.gait };
        if trace && t >= from && t < from + 2.0 && (t * 4.0).floor() != ((t - DT) * 4.0).floor() {
            let head: Vec<(f64, f64)> = path.iter().take(6).map(|&q| grid.center(q)).collect();
            println!("    {t:6.2} steer at ({px:.2}, {pz:.2}) to {next:?}; here ok {} path {head:.2?}", grid.ok(grid.cell(px, pz)));
        }
        drive(&mut g, next, c.face, gait);
        if c.paper {
            g.drop_paper_marker();
            last_drop = t;
        }
        // the lamp: lit in the dark, out under the panels, or as its counter says
        lamp_t -= DT;
        if lamp_t <= 0.0 {
            let here = g.region_light_at(g.tx(g.player.pos.x), g.tz(g.player.pos.z));
            let want = c.lamp.unwrap_or(if here < 0.3 {
                true
            } else if here > 0.5 {
                false
            } else {
                g.player.flashlight_on
            });
            if want != g.player.flashlight_on {
                g.toggle_flashlight();
                lamp_t = 0.5;
            }
        }
        // (BEATABLE_FROM: each creature within 30 m twice a second)
        if trace && t >= from && (t * 2.0).floor() != ((t - DT) * 2.0).floor() {
            for (i, e) in g.entities.iter().enumerate() {
                let Some(cr) = g.creature(i) else { continue };
                let p = e.pos();
                let d = ((p.x - px).powi(2) + (p.z - pz).powi(2)).sqrt();
                if d < 30.0 {
                    let pc = g.perceive(i);
                    println!(
                        "    {t:6.1}   {} {:?} d {d:.1} line {} you_lit {:.2} lamp {} nerve {:.0} light here {:.2} gait {:?}",
                        e.kind,
                        cr.brain.mode,
                        pc.line,
                        pc.you_lit,
                        g.player.flashlight_on,
                        g.player.sanity,
                        g.region_light_at(g.tx(px), g.tz(pz)),
                        c.gait
                    );
                }
            }
        }
        if trace && (t / 10.0).floor() != ((t - DT) / 10.0).floor() {
            let goal = if page_left {
                "her camp"
            } else if pile.is_some() {
                "a pile or a battery"
            } else {
                "the door"
            };
            println!(
                "    {t:6.1} at ({px:.1}, {pz:.1}) for {goal}, path {}, nerve {:.0}, battery {:.0}",
                path.len(),
                g.player.sanity,
                g.player.battery
            );
        }
        if trace && c.why != why {
            why = c.why;
            println!("    {t:6.1} {why} (nerve {:.0}, papers {})", g.player.sanity, g.player.papers);
        }
        let nerve_before = g.player.sanity;
        let met_before = g.creature_run.met.clone();
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
        t += DT;
        // the dark's toll
        let (tx, tz) = (g.tx(g.player.pos.x), g.tz(g.player.pos.z));
        let dark = g.region_light_at(tx, tz) <= 0.0 && !g.player.flashlight_on;
        if dark && g.player.sanity < 25.0 {
            out.dark_spent += DT;
            if g.player.battery <= 0.0 {
                out.dark_battery += DT;
            } else if c.lamp == Some(false) {
                out.dark_counter += DT;
            }
        }
        dark_recent.push_back((t, dark));
        while dark_recent.front().is_some_and(|(s, _)| t - s > 30.0) {
            dark_recent.pop_front();
        }
        // the chase cap, for every creature
        for (i, e) in g.entities.iter().enumerate().take(64) {
            let v = (e.vx * e.vx + e.vz * e.vz).sqrt();
            over[i] = if v > cap { over[i] + DT } else { 0.0 };
            if over[i] > 0.8 {
                out.fails.push(format!("{} ran at {v:.1} m/s for {:.1} s (the cap is {:.1})", e.kind, over[i], cap - 0.15));
                over[i] = -1e9;
            }
        }
        if g.state != "playing" || g.level_num != level {
            let dead = g.state == "dead" || g.state == "gameover" || g.director.log.iter().any(|e| e.1 == "death");
            if dead {
                let cause = g.last_death_cause.clone().unwrap_or_default();
                let killer = g.director.killer.filter(|&k| k < g.entities.len());
                let kind = killer.map(|k| g.entities[k].kind.clone()).unwrap_or_default();
                out.died = Some((t, format!("{cause}{}{kind}", if kind.is_empty() { "" } else { ": " })));
                if t < 30.0 {
                    out.fails.push(format!("a death {t:.0} s into the level"));
                }
                if cause == "entity" {
                    if let Some(k) = killer {
                        let tells = g.creature(k).map(|c| c.tells.iter().filter(|x| g.perf_t - **x < 20.0).count()).unwrap_or(0);
                        if tells < 2 {
                            out.fails.push(format!("{kind} killed with {tells} tells"));
                        }
                    }
                    if !met_before.contains(&kind) {
                        out.fails.push(format!("{kind} killed at a first meeting"));
                    }
                }
                let dark_s = dark_recent.iter().filter(|(_, d)| *d).count() as f64 * DT;
                out.dark_death = nerve_before < 25.0 && dark_s > 5.0;
            } else {
                out.finished = true;
            }
            break;
        }
    }
    out.peaks = g.director.log.iter().filter(|e| e.1 == "peak").count();
    out.contacts = g.director.log.iter().filter(|e| e.1 == "contact").count();
    if !out.finished && out.died.is_none() {
        let (px, pz) = (g.player.pos.x, g.player.pos.z);
        out.unfinished =
            format!("{:.0} m from the door after {SECONDS} s, path {}", ((exit.0 - px).powi(2) + (exit.1 - pz).powi(2)).sqrt(), path.len());
    }
    for k in ["KeyW", "KeyS", "KeyA", "KeyD", "ShiftLeft", "ControlLeft"] {
        press(&mut g, k, false);
    }
    let _ = SPRINT;
    *memory = g.creature_run.clone();
    Some(out)
}

/// One level, traced, after this run's levels before it, so it plays as it
/// did in the sweep, where each run plays on a thread of its own: what is
/// made once a thread (the creatures' cloth, any canvas texture) draws on the
/// per-frame stream the first time only, so a level's frames depend on what
/// that thread played before.
///
///   BEATABLE_TRACE=seed:level cargo test --release -p backrooms --test beatable one_level -- --ignored --nocapture
#[test]
#[ignore]
fn one_level() {
    let v = std::env::var("BEATABLE_TRACE").expect("BEATABLE_TRACE=seed:level");
    let (seed, level): (u32, i32) = v.split_once(':').map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap())).unwrap();
    let mut memory = RunMemory::default();
    for l in *LEVELS.start()..level {
        play(Bot::RuleFollower, seed, l, &mut memory);
    }
    println!("{:?}", play(Bot::RuleFollower, seed, level, &mut memory));
    println!("the run's memory: {memory:?}");
}

/// One run of `bot`: every level of `seed` in order, the run's memory carried
/// from each to the next; each level's outcome, and what the run learned
fn run(bot: Bot, seed: u32) -> (Vec<(i32, Outcome)>, String) {
    let mut memory = RunMemory::default();
    let mut out = vec![];
    for level in LEVELS {
        if let Some(o) = play(bot, seed, level, &mut memory) {
            out.push((level, o));
        }
    }
    (out, format!("{seed}: {:?} (habits {:?})", memory.learned, memory.habits))
}

/// Plays `bot` on every level of every run, the runs side by side (each on a
/// thread of its own, so a run plays as it would alone); its share finished,
/// and the hard rules any death of it broke.
fn sweep(bot: Bot) -> (f64, Vec<String>) {
    let runs: Vec<(u32, (Vec<(i32, Outcome)>, String))> = std::thread::scope(|sc| {
        let handles: Vec<_> = SEEDS
            .iter()
            .map(|&seed| std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(sc, move || (seed, run(bot, seed))).unwrap())
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    });
    let mut fails = vec![];
    let (mut levels, mut finished, mut deaths, mut dark_deaths, mut dark_spent) = (0, 0, 0, 0, 0.0);
    let (mut dark_counter, mut dark_battery) = (0.0, 0.0);
    let (mut peaks, mut contacts, mut quiet) = (0, 0, 0);
    let mut died = vec![];
    let mut unfinished = vec![];
    let mut learned = vec![];
    for (seed, (outcomes, run_learned)) in runs {
        for (level, o) in outcomes {
            levels += 1;
            if o.finished {
                finished += 1;
            }
            if let Some((t, of)) = &o.died {
                deaths += 1;
                died.push(format!("{seed} L{level} at {t:.0} s ({of})"));
            }
            if o.dark_death {
                dark_deaths += 1;
            }
            peaks += o.peaks;
            contacts += o.contacts;
            if o.peaks == 0 {
                quiet += 1;
            }
            dark_spent += o.dark_spent;
            dark_counter += o.dark_counter;
            dark_battery += o.dark_battery;
            if !o.unfinished.is_empty() {
                unfinished.push(format!("{seed} L{level}: {}", o.unfinished));
            }
            for f in o.fails {
                fails.push(format!("{bot:?} {seed} L{level}: {f}"));
            }
        }
        learned.push(run_learned);
    }
    let share = finished as f64 / levels.max(1) as f64;
    println!(
        "{bot:?}: finished {finished} of {levels} levels ({:.0}%), {deaths} deaths {died:?}; {peaks} peaks, {contacts} contacts, {quiet} levels with no peak; {dark_deaths} of them on a spent nerve after time in the dark; {dark_spent:.0} s in the dark on a spent nerve in all ({dark_counter:.0} s the lamp out for a creature, {dark_battery:.0} s its battery dead); unfinished {unfinished:?}; learned by the end of each run {learned:?}",
        100.0 * share
    );
    (share, fails)
}

#[test]
fn the_rule_follower_gets_through_and_no_death_breaks_the_rules() {
    let (share, fails) = sweep(Bot::RuleFollower);
    assert!(fails.is_empty(), "hard rules broken:\n{}", fails.join("\n"));
    assert!(share >= 0.9, "the rule-follower finished {:.0}% of the levels", share * 100.0);
}

/// Those who know no counter get through less, but some, and no death of
/// theirs breaks a hard rule either.
#[test]
fn the_explorer_gets_through_some() {
    let (share, fails) = sweep(Bot::Explorer);
    assert!(fails.is_empty(), "hard rules broken:\n{}", fails.join("\n"));
    assert!(share > 0.0, "the explorer never got through");
}

#[test]
fn the_rusher_gets_through_some() {
    let (share, fails) = sweep(Bot::Rusher);
    assert!(fails.is_empty(), "hard rules broken:\n{}", fails.join("\n"));
    assert!(share > 0.0, "the rusher never got through");
}
