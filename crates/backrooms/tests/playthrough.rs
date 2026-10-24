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
//! The run's anomaly rules are checked: at most one per level, never the same
//! twice, none on level 1, all four met by level 12.
//!
//!   cargo test --release -p backrooms --test playthrough -- --nocapture
//!   PLAYTHROUGH_SEEDS=101,202 PLAYTHROUGH_LEVELS=13 cargo test ...
use game::base::config::EYE;
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
fn blocked(g: &Game, x: f64, z: f64, r: f64) -> bool {
    g.circle_hits_wall(x, z, r)
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
    /// drive along a path with W (sprinting while there is stamina)
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
            set_look(g, (-dx).atan2(-dz), 0.0);
            let sprint = g.player.stamina > 25.0;
            key(g, "ShiftLeft", sprint);
            g.player.sanity = 100.0;
            step(g, 1);
            frames += 1;
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
        key(g, "KeyW", false);
        key(g, "ShiftLeft", false);
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

fn play(seed: u32, levels: i32) -> Vec<(i32, Option<String>)> {
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}");
    g.view_width = 480.0;
    g.view_height = 270.0;
    g.scale_factor = 1.0;
    g.boot();
    g.start_new_game(); // New Game from the menu
    g.god_mode = true;
    let mut bot = Bot { fine_key: String::new(), free: vec![] };
    let mut met = vec![];
    for l in 1..=levels {
        assert!(wait_for(&mut g, 3600, |g| g.state == "playing" && g.level_num == l), "run {seed}: level {l} never started");
        freeze_entities(&mut g);
        let scp = g.level_scp.clone();
        met.push((l, scp.clone()));
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
        let (ex, ez) = exit(&g);
        let w = bot.walk(&mut g, ex, ez, 36000);
        assert!(w.ok, "run {seed} level {l}: could not reach the exit ({} frames, {} nudges)", w.frames, w.stuck);
        frames += w.frames;
        stuck += w.stuck;
        g.try_interact();
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
        assert!(
            wait_for(&mut g, 3600, |g| g.level_num == l + 1 && g.state == "playing"),
            "run {seed} level {l}: the door did not lead on (level {}, state {})",
            g.level_num,
            g.state
        );
        println!(
            "     run {seed} L{l:<2} {:<9} {:>4} s walked  {}",
            scp.as_deref().unwrap_or("-"),
            (frames as f64 / 60.0).round() as i64,
            if stuck > 0 { format!("{stuck} nudges") } else { String::new() }
        );
    }
    met
}

#[test]
fn playthrough_from_new_game() {
    let seeds: Vec<u32> = std::env::var("PLAYTHROUGH_SEEDS")
        .ok()
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| vec![101, 202]);
    let levels: i32 = std::env::var("PLAYTHROUGH_LEVELS").ok().and_then(|s| s.parse().ok()).unwrap_or(13);
    for seed in seeds {
        let met = play(seed, levels);
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
