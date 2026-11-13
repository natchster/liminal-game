//! Playthrough: is the game beatable? A bot plays whole runs from New Game: on
//! every level it walks to the exit with the real movement and collision (a
//! fine-grid path steered with the movement keys; a small counted nudge only
//! when truly stuck), deals with the level's anomaly the way a player must,
//! and goes through the door:
//!
//!   Showroom       walk to the exit
//!   Borrowed House put out every lamp in the nest, then leave by the front door
//!   Stairwell      go through the steel door, walk down every flight, out the bottom
//!   Loop           go through the plain door, walk back lap by lap to lap 0, out
//!
//! Entities are held still and deaths are counted, not suffered (god mode).
//! Live (`the_whole_run_live`), the creatures walk the rooms and deaths are
//! suffered: the bot plays the rule-follower's counters as it walks
//! (`tests/rule_follower`), and a death is a retry (Wake up) of the level, as
//! a player's is; the Building's floors and the anomalies' own modes are
//! played as above (the self-assembly and the anomalies' residents have
//! tests of their own; E9 rebuilds the residents).
//! The run's anomaly rules are checked: at most one per level, never the same
//! twice, none on level 1, all four met by level 12. Leaving level 13 ends
//! M.'s story (the card; E goes on).
//!
//!   cargo test --release -p backrooms --test playthrough -- --nocapture
//!   PLAYTHROUGH_SEEDS=101,202 PLAYTHROUGH_LEVELS=13 cargo test ...
use game::base::config::EYE;
use game::Game;
mod rule_follower;

// ---- the game hooks the bot uses -----------------------------------------
fn step(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
    }
}
fn key(g: &mut Game, code: &str, down: bool) {
    if down {
        g.keys.insert(code.to_string());
    } else {
        g.keys.remove(code);
    }
}
fn set_look(g: &mut Game, yaw: f64, pitch: f64) {
    g.player.yaw = yaw;
    g.player.pitch = pitch;
}
/// A wall, or a hole in the floor: a player walks round one as round the
/// other (stepping in, you fall a level)
fn blocked(g: &Game, x: f64, z: f64, r: f64) -> bool {
    g.circle_hits_wall(x, z, r) || g.floor_holes.iter().any(|h| (h.x - x).hypot(h.z - z) < h.r + r)
}
fn freeze_entities(g: &mut Game) {
    for e in g.entities.iter_mut() {
        e.freeze = 1e9;
    }
}
fn exit(g: &Game) -> (f64, f64) {
    let e = g.exit_data.as_ref().expect("level has an exit");
    (e.pos.x, e.pos.z)
}
struct Stair {
    seg: i32,
    x: f64,
    z: f64,
    flights: i32,
    near_exit: bool,
}
fn stair(g: &Game) -> Option<Stair> {
    if !g.in_stairwell {
        return None;
    }
    let s = g.stairwell.as_ref()?;
    let st = s.state();
    Some(Stair { seg: st.seg, x: st.x, z: st.z, flights: s.flights(), near_exit: g.stair_near_exit })
}
struct LoopView {
    lap: i32,
    room: i32,
    x: f64,
    z: f64,
    near_exit: bool,
}
fn loop_view(g: &Game) -> Option<LoopView> {
    if !g.in_loop {
        return None;
    }
    let l = g.loop_rooms.as_ref()?;
    Some(LoopView { lap: l.lap(), room: l.room(), x: l.state().x, z: l.state().z, near_exit: g.loop_near_exit })
}
/// unlit lamps of the nest
fn lamp_pos(g: &Game) -> Vec<(f64, f64)> {
    g.house_state
        .lamps
        .iter()
        .filter(|l| !l.out)
        .map(|l| {
            let v = l.g.world_position();
            (v.x, v.z)
        })
        .collect()
}

// ---- the bot -----------------------------------------------------------------
/// tile path (4-connected, open tiles) from where the player stands to (x, z)
fn tile_path(g: &Game, x: f64, z: f64) -> Option<Vec<(f64, f64)>> {
    let (w, h) = (g.w as i64, g.h as i64);
    let barred: std::collections::HashSet<usize> = g.barricade_tiles.iter().copied().collect();
    let open = |i: i64| i >= 0 && (i as usize) < g.tiles.len() && g.tiles[i as usize] == 0 && !barred.contains(&(i as usize));
    let near = |tx: i64, tz: i64| -> (i64, i64) {
        if open(tz * w + tx) {
            return (tx, tz);
        }
        for r in 1..3 {
            for dz in -r..=r {
                for dx in -r..=r {
                    if open((tz + dz) * w + tx + dx) {
                        return (tx + dx, tz + dz);
                    }
                }
            }
        }
        (tx, tz)
    };
    let (sx, sz) = near(g.tx(g.player.pos.x) as i64, g.tz(g.player.pos.z) as i64);
    let (ex, ez) = near(g.tx(x) as i64, g.tz(z) as i64);
    let n = (w * h) as usize;
    let mut prev = vec![-2i64; n];
    let s0 = sz * w + sx;
    if s0 < 0 || s0 as usize >= n {
        return None;
    }
    prev[s0 as usize] = -1;
    let mut q = vec![s0];
    let goal = ez * w + ex;
    let mut k = 0;
    while k < q.len() {
        let c = q[k];
        k += 1;
        if c == goal {
            break;
        }
        for d in [1, -1, w, -w] {
            let nb = c + d;
            if nb >= 0 && (nb as usize) < n && prev[nb as usize] == -2 && open(nb) {
                prev[nb as usize] = c;
                q.push(nb);
            }
        }
    }
    if goal < 0 || goal as usize >= n || prev[goal as usize] == -2 {
        return None;
    }
    let mut out = vec![];
    let mut c = goal;
    while c != -1 {
        out.push((g.wx((c % w) as i32), g.wz((c / w) as i32)));
        c = prev[c as usize];
    }
    out.reverse();
    out.push((x, z));
    Some(out)
}

/// a fine path: 8 cells per tile, each cell free if a player-sized circle (plus
/// a hand's width) fits there by the game's own collision, searched 8-ways,
/// then thinned to waypoints three cells apart
fn fine_path(g: &Game, free: &[u8], x: f64, z: f64) -> Option<Vec<(f64, f64)>> {
    const R: i64 = 8;
    let (gw, gh) = (g.w as i64 * R, g.h as i64 * R);
    let cs = 4.0 / R as f64;
    let cell = |wx: f64, wz: f64| (((wx + g.w as f64 * 2.0) / cs).floor() as i64, ((wz + g.h as f64 * 2.0) / cs).floor() as i64);
    let near = |(i, j): (i64, i64)| -> i64 {
        for r in 0..6 {
            for dj in -r..=r {
                for di in -r..=r {
                    let c = (j + dj) * gw + i + di;
                    if c >= 0 && (c as usize) < free.len() && free[c as usize] != 0 {
                        return c;
                    }
                }
            }
        }
        j * gw + i
    };
    let s0 = near(cell(g.player.pos.x, g.player.pos.z));
    let e0 = near(cell(x, z));
    let n = (gw * gh) as usize;
    if s0 < 0 || e0 < 0 || s0 as usize >= n || e0 as usize >= n {
        return None;
    }
    let mut prev = vec![-2i64; n];
    prev[s0 as usize] = -1;
    let mut q = vec![s0];
    let mut k = 0;
    const DIRS8: [(i64, i64); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    while k < q.len() {
        let c = q[k];
        k += 1;
        if c == e0 {
            break;
        }
        let (ci, cj) = (c % gw, c / gw);
        for (di, dj) in DIRS8 {
            let (ni, nj) = (ci + di, cj + dj);
            if ni < 0 || nj < 0 || ni >= gw || nj >= gh {
                continue;
            }
            let nb = nj * gw + ni;
            if prev[nb as usize] != -2 || free[nb as usize] == 0 {
                continue;
            }
            // a diagonal step needs both sides free: no cutting past a corner
            if di != 0 && dj != 0 && (free[(cj * gw + ni) as usize] == 0 || free[(nj * gw + ci) as usize] == 0) {
                continue;
            }
            prev[nb as usize] = c;
            q.push(nb);
        }
    }
    if prev[e0 as usize] == -2 {
        return None;
    }
    let mut cells = vec![];
    let mut c = e0;
    while c != -1 {
        cells.push(c);
        c = prev[c as usize];
    }
    cells.reverse();
    let mut out: Vec<(f64, f64)> = cells
        .iter()
        .step_by(3)
        .map(|&c| (((c % gw) as f64 + 0.5) * cs - g.w as f64 * 2.0, (((c / gw) as f64) + 0.5) * cs - g.h as f64 * 2.0))
        .collect();
    out.push((x, z));
    Some(out)
}

struct Walk {
    ok: bool,
    frames: usize,
    stuck: usize,
}
struct Bot {
    fine_key: String,
    free: Vec<u8>,
    /// the creatures walk and kill (on the rooms' own floors)
    live: bool,
    /// the time and the last sheet dropped (the rule-follower's)
    t: f64,
    last_drop: f64,
    /// deaths suffered
    deaths: Vec<String>,
}
impl Bot {
    fn free_grid(&mut self, g: &Game) -> &[u8] {
        let (ex, ez) = exit(g);
        let key = format!("{}:{}:{}", g.level_num, ex, ez);
        if self.fine_key != key {
            const R: i64 = 8;
            let (gw, gh) = (g.w as i64 * R, g.h as i64 * R);
            let cs = 4.0 / R as f64;
            let mut free = vec![0u8; (gw * gh) as usize];
            for j in 0..gh {
                for i in 0..gw {
                    let (wx, wz) = ((i as f64 + 0.5) * cs - g.w as f64 * 2.0, (j as f64 + 0.5) * cs - g.h as f64 * 2.0);
                    free[(j * gw + i) as usize] = if blocked(g, wx, wz, 0.42) { 0 } else { 1 };
                }
            }
            self.free = free;
            self.fine_key = key;
        }
        &self.free
    }
    /// A level: live, her camp and her page first (as the rule-follower of
    /// `beatable.rs` plays it); then the door.
    fn level(&mut self, g: &mut Game) -> Walk {
        let camp = g.story.camp.as_ref().filter(|c| c.page_id.is_some() && !c.page_taken).map(|c| (c.at.x, c.at.z));
        let mut frames = 0;
        if let (true, Some((cx, cz))) = (self.live, camp) {
            let w = self.walk(g, cx, cz, 36000);
            frames += w.frames;
            if g.state == "dead" {
                return w;
            }
            if (g.player.pos.x - cx).hypot(g.player.pos.z - cz) < 1.6 {
                g.take_page();
            }
        }
        let (ex, ez) = exit(g);
        let mut w = self.walk(g, ex, ez, 36000);
        w.frames += frames;
        w
    }

    /// drive along a path with W (sprinting while there is stamina; live,
    /// as the rule-follower's counters say)
    fn walk(&mut self, g: &mut Game, x: f64, z: f64, max_frames: usize) -> Walk {
        let free = self.free_grid(g).to_vec();
        let Some(p) = fine_path(g, &free, x, z).or_else(|| tile_path(g, x, z)) else { return Walk { ok: false, frames: 0, stuck: 0 } };
        let (mut i, mut frames, mut stuck, mut best, mut still, mut tries) = (1usize, 0usize, 0usize, 1e9f64, 0usize, 0usize);
        key(g, "KeyW", true);
        while i < p.len() && frames < max_frames {
            let (wx, wz) = p[i];
            let (dx, dz) = (wx - g.player.pos.x, wz - g.player.pos.z);
            let d = dx.hypot(dz);
            if d < if i == p.len() - 1 { 0.45 } else { 0.4 } {
                i += 1;
                best = 1e9;
                still = 0;
                tries = 0;
                continue;
            }
            if self.live {
                // the rule-follower: what the creatures it perceives call for
                let c = rule_follower::counter(g, self.last_drop, self.t);
                if c.paper {
                    g.drop_paper_marker();
                    self.last_drop = self.t;
                }
                if let Some(on) = c.lamp {
                    if on != g.player.flashlight_on {
                        g.toggle_flashlight();
                    }
                }
                let to = match c.away_from {
                    Some((ax, az)) => {
                        let (ux, uz) = (g.player.pos.x - ax, g.player.pos.z - az);
                        let l = ux.hypot(uz).max(1e-6);
                        Some((g.player.pos.x + ux / l * 3.0, g.player.pos.z + uz / l * 3.0))
                    }
                    None => Some((wx, wz)),
                };
                rule_follower::drive(g, to, c.face, c.gait);
                step(g, 1);
                self.t += 1.0 / 60.0;
                frames += 1;
                if g.state == "dead" {
                    break;
                }
                if c.gait == rule_follower::Gait::Still || c.away_from.is_some() {
                    // (held or turned aside: no progress expected)
                    best = 1e9;
                    still = 0;
                    continue;
                }
            } else {
                set_look(g, (-dx).atan2(-dz), 0.0);
                let sprint = g.player.stamina > 25.0;
                key(g, "ShiftLeft", sprint);
                g.player.sanity = 100.0;
                step(g, 1);
                frames += 1;
            }
            if d < best - 0.02 {
                best = d;
                still = 0;
            } else if i == p.len() - 1 && d < 1.0 && still > 30 {
                i += 1;
                continue;
            }
            // the goal sits against a wall
            else {
                still += 1;
                if still > 60 {
                    // caught on a jamb or a partition end: side-step first, like a player
                    tries += 1;
                    still = 0;
                    if tries <= 2 {
                        let side = if tries == 1 { "KeyA" } else { "KeyD" };
                        key(g, "KeyW", false);
                        key(g, side, true);
                        step(g, 25);
                        key(g, side, false);
                        key(g, "KeyW", true);
                        frames += 25;
                    } else {
                        // truly stuck: a small nudge (never into a wall), counted
                        let (nx, nz) = (g.player.pos.x + dx / d * 0.3, g.player.pos.z + dz / d * 0.3);
                        if !blocked(g, nx, nz, game::base::config::PLAYER_R) {
                            g.player.pos.x = nx;
                            g.player.pos.y = EYE;
                            g.player.pos.z = nz;
                        }
                        stuck += 1;
                        tries = 0;
                    }
                }
            }
            if g.state != "playing" {
                break;
            }
        }
        for k in ["KeyW", "KeyS", "KeyA", "KeyD", "ShiftLeft", "ControlLeft"] {
            key(g, k, false);
        }
        let (ex, ez) = p[p.len() - 1];
        Walk { ok: (ex - g.player.pos.x).hypot(ez - g.player.pos.z) < 1.2, frames, stuck }
    }
}

/// step until `cond` holds (the frame loop keeps running while the test waits)
fn wait_for(g: &mut Game, max_frames: usize, cond: impl Fn(&Game) -> bool) -> bool {
    for _ in 0..max_frames {
        if cond(g) {
            return true;
        }
        step(g, 1);
    }
    cond(g)
}

/// The peaks and the contacts of the level so far (the director's log).
fn tally(g: &Game) -> (usize, usize) {
    let n = |what: &str| g.director.log.iter().filter(|(_, w)| *w == what).count();
    (n("peak"), n("contact"))
}

fn play(seed: u32, levels: i32, live: bool) -> (Vec<(i32, Option<String>)>, Vec<String>) {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}");
    g.view_width = 480.0;
    g.view_height = 270.0;
    g.scale_factor = 1.0;
    g.boot();
    g.start_new_game(); // New Game from the menu
    g.god_mode = true;
    let mut bot = Bot { fine_key: String::new(), free: vec![], live: false, t: 0.0, last_drop: -10.0, deaths: vec![] };
    let mut met = vec![];
    for l in 1..=levels {
        assert!(wait_for(&mut g, 3600, |g| g.state == "playing" && g.level_num == l), "run {seed}: level {l} never started");
        let scp = g.level_scp.clone();
        // live on the rooms' own floors: the creatures walk, deaths are suffered
        bot.live = live && g.building.is_none() && !matches!(scp.as_deref(), Some("store" | "house"));
        g.god_mode = !bot.live;
        if !bot.live {
            freeze_entities(&mut g);
        }
        met.push((l, scp.clone()));
        let creatures = (0..g.entities.len()).filter(|&i| g.creature(i).is_some()).count();
        let (mut frames, mut stuck) = (0, 0);
        if scp.as_deref() == Some("house") {
            // the nest first: walk to each lamp and put it out
            for _ in 0..20 {
                let lamps = lamp_pos(&g);
                if lamps.is_empty() {
                    break;
                }
                let (x, z) = lamps[0];
                let w = bot.walk(&mut g, x + 0.3, z + 0.3, 36000);
                frames += w.frames;
                stuck += w.stuck;
                g.try_interact();
                step(&mut g, 1);
                assert!(w.ok || w.frames > 0, "run {seed} level {l}: no path to a lamp");
            }
            assert!(g.house_state.out, "run {seed} level {l}: could not put out the nest");
            step(&mut g, 150);
        }
        // live, her story as a player follows it: her camp and her page
        // (which wakes the place), then the door
        let (mut peaks, mut contacts) = (0, 0);
        let mut w = bot.level(&mut g);
        // a death: Wake up, and the level again (as many times as a player would)
        let mut tries = 0;
        while bot.live && g.state == "dead" && tries < 5 {
            let cause = g.last_death_cause.clone().unwrap_or_default();
            let killer = g.director.killer.filter(|&k| k < g.entities.len()).map(|k| g.entities[k].kind.clone()).unwrap_or_default();
            bot.deaths.push(format!("L{l}: {cause} {killer}"));
            let (p, c) = tally(&g);
            peaks += p;
            contacts += c;
            g.restart();
            assert!(
                wait_for(&mut g, 3600, |g| g.state == "playing" && g.level_num == l),
                "run {seed}: level {l} never came back after a death"
            );
            w = bot.level(&mut g);
            tries += 1;
        }
        let (p, c) = tally(&g);
        peaks += p;
        contacts += c;
        assert!(w.ok, "run {seed} level {l}: could not reach the exit ({} frames, {} nudges; deaths {:?})", w.frames, w.stuck, bot.deaths);
        frames += w.frames;
        stuck += w.stuck;
        g.try_interact();
        // the modes beyond the doors (the Stairwell, the Loop) are played as before
        g.god_mode = true;
        if scp.as_deref() == Some("stairwell") {
            assert!(wait_for(&mut g, 900, |g| stair(g).is_some()), "run {seed} level {l}: the steel door did not open");
            let fl = 13.0 * 0.31;
            let wp = |seg: i32| -> (f64, f64) {
                if seg % 2 != 0 {
                    let k = (seg - 1) / 2;
                    if k % 2 == 0 {
                        (0.8, fl - 0.4)
                    } else {
                        (-0.8, 0.4)
                    }
                } else {
                    let k = seg / 2;
                    if k % 2 != 0 {
                        (0.8, -0.8)
                    } else {
                        (-0.8, fl + 0.8)
                    }
                }
            };
            key(&mut g, "KeyW", true);
            let mut f = 0;
            while f < 60 * 400 {
                let Some(s) = stair(&g) else { break };
                if s.near_exit {
                    break;
                }
                if s.seg > 2 * (s.flights - 1) {
                    let far = (s.flights - 1) % 2 == 0;
                    let tz = if far { fl + 1.4 } else { -1.4 };
                    set_look(&mut g, (-(0.0 - s.x)).atan2(-(tz - s.z)), 0.0);
                } else {
                    let (tx, tz) = wp(s.seg);
                    set_look(&mut g, (-(tx - s.x)).atan2(-(tz - s.z)), -0.25);
                }
                g.player.sanity = 100.0;
                step(&mut g, 1);
                f += 1;
            }
            key(&mut g, "KeyW", false);
            let near = stair(&g).is_some_and(|s| s.near_exit);
            g.try_interact();
            assert!(near, "run {seed} level {l}: never reached the bottom of the Stairwell");
            frames += f;
        } else if scp.as_deref() == Some("loop") {
            assert!(wait_for(&mut g, 900, |g| loop_view(g).is_some()), "run {seed} level {l}: the door did not lead into the Loop");
            let p = 5.2;
            key(&mut g, "KeyW", true);
            let mut f = 0;
            while f < 60 * 240 {
                let Some(s) = loop_view(&g) else { break };
                if s.lap == 0 && s.room == 0 && s.z < p / 2.0 + 1.0 {
                    break;
                }
                set_look(&mut g, s.x.atan2(1.0), 0.0);
                g.player.sanity = 100.0;
                step(&mut g, 1);
                f += 1;
            }
            let (tx, tz) = (-1.75, p / 2.0 + 1.2);
            for _ in 0..400 {
                let Some(q) = loop_view(&g) else { break };
                if q.near_exit {
                    break;
                }
                set_look(&mut g, (-(tx - q.x)).atan2(-(tz - q.z)), 0.0);
                step(&mut g, 1);
                f += 1;
            }
            key(&mut g, "KeyW", false);
            let near = loop_view(&g).is_some_and(|s| s.near_exit);
            g.try_interact();
            assert!(near, "run {seed} level {l}: could not walk back out of the Loop");
            frames += f;
        }
        if l == game::story::notebook::LAST_LEVEL {
            // the end of her story: the card, then on down (E)
            assert!(wait_for(&mut g, 600, |g| g.state == "ending"), "run {seed}: her story did not end on level {l}");
            assert!(wait_for(&mut g, 300, |g| g.live_menu().is_some()), "run {seed}: no card at the end of her story");
            g.key_down("KeyE", false);
            g.key_up("KeyE");
        }
        assert!(
            wait_for(&mut g, 3600, |g| g.level_num == l + 1 && g.state == "playing"),
            "run {seed} level {l}: the door did not lead on (level {}, state {})",
            g.level_num,
            g.state
        );
        println!(
            "     run {seed} L{l:<2} {:<9} {:>4} s walked  {}{}",
            scp.as_deref().unwrap_or("-"),
            (frames as f64 / 60.0).round() as i64,
            if bot.live { format!("{creatures} creatures, {peaks} peaks, {contacts} contacts  ") } else { String::new() },
            if stuck > 0 { format!("{stuck} nudges") } else { String::new() }
        );
    }
    (met, bot.deaths)
}

#[test]
fn playthrough_from_new_game() {
    let seeds: Vec<u32> = std::env::var("PLAYTHROUGH_SEEDS")
        .ok()
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| vec![101, 202]);
    let levels: i32 = std::env::var("PLAYTHROUGH_LEVELS").ok().and_then(|s| s.parse().ok()).unwrap_or(13);
    for seed in seeds {
        let (met, _) = play(seed, levels, false);
        // the run's anomaly rules
        let keys: Vec<&str> = met.iter().filter_map(|(_, k)| k.as_deref()).collect();
        let uniq: std::collections::HashSet<&&str> = keys.iter().collect();
        assert_eq!(uniq.len(), keys.len(), "run {seed}: an anomaly repeated: {met:?}");
        assert!(met.iter().all(|(l, k)| !(*l == 1 && k.is_some())), "run {seed}: an anomaly on level 1");
        if levels >= 12 {
            for k in ["stairwell", "store", "house", "loop"] {
                assert!(keys.contains(&k), "run {seed}: {k} not met by level 12: {met:?}");
            }
        }
    }
}

/// The whole run, live (`docs/design/ENTITIES.md` design 7): from New Game past
/// level 13, the creatures walking the rooms and deaths suffered, the bot
/// playing the rule-follower's counters; the run ends (the game is beatable
/// as shipped, not only in god mode), with few deaths on the way.
#[test]
fn the_whole_run_live() {
    let seeds: Vec<u32> = std::env::var("PLAYTHROUGH_SEEDS")
        .ok()
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| vec![101, 202, 303]);
    for seed in seeds {
        // the frames' randomness from the run's seed, not the clock: a run
        // that fails can be played again as it was (it had run out of time
        // on one level in a few runs and passed in the next)
        game::base::rng::seed_native(seed as u64);
        let (met, deaths) = play(seed, 14, true);
        println!("run {seed}, live: {} levels, {} deaths {deaths:?}", met.len(), deaths.len());
        assert!(deaths.len() <= 4, "run {seed}: {} deaths on the way: {deaths:?}", deaths.len());
    }
}
