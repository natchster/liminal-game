//! Barricades (`docs/design/PROPS.md` 2): each closes a real opening, as people
//! would, and the floor stays whole round it.
//!
//!   cargo test --release -p backrooms --test barricades -- --nocapture
use engine::dmath::DMath;
use game::Game;

fn new_game() -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = "nosound".into();
    g.boot();
    g
}

/// Every floor of the rooms these seeds build, and what the test reads
const SEEDS: [u32; 6] = [4242, 77, 202, 303, 7, 1999];
const LEVELS: [i32; 9] = [4, 5, 6, 7, 8, 9, 10, 11, 12];

fn each_floor(mut f: impl FnMut(u32, i32, &Game)) {
    for &seed in &SEEDS {
        let mut g = new_game();
        g.run_seed = seed;
        for &l in &LEVELS {
            g.init_level(l);
            g.state = "playing".into();
            f(seed, l, &g);
        }
    }
}

/// Each barricade seals its opening: a body walking up to it from either
/// side along the way, on a 0.1 m grid round the barricade's tile and its
/// two neighbours along the way, never reaches the other side; its slab
/// spans the opening wall to wall (no gap a body fits through); it never
/// stands over the ceiling; no tile has two.
#[test]
fn each_barricade_seals_its_opening() {
    let mut n = 0;
    let mut kinds = 0;
    each_floor(|seed, l, g| {
        let mut tiles = std::collections::HashSet::new();
        for b in &g.barricades {
            n += 1;
            assert!(tiles.insert(b.tile), "seed {seed} L{l}: two barricades on tile {}", b.tile);
            let (cx, cz) = ((b.x0 + b.x1) / 2.0, (b.z0 + b.z1) / 2.0);
            // its slab reaches the faces: a body (0.35 m) cannot pass beside it
            let (along, a0, a1) = if b.along_x { (cx, b.z0, b.z1) } else { (cz, b.x0, b.x1) };
            for off in [a0 - 0.06, a1 + 0.06] {
                let (x, z) = if b.along_x { (along, off) } else { (off, along) };
                let past = if b.along_x { g.circle_hits_wall(x + 0.0, z, 0.05) } else { g.circle_hits_wall(x, z + 0.0, 0.05) };
                assert!(past, "seed {seed} L{l}: barricade on tile {} leaves room beside it at ({x:.2}, {z:.2})", b.tile);
            }
            // a flood from one side along the way never reaches the other
            let (sx, sz, ex, ez) = if b.along_x { (cx - 2.5, cz, cx + 2.5, cz) } else { (cx, cz - 2.5, cx, cz + 2.5) };
            let (r, step) = (0.35, 0.1);
            let reach = |x: f64, z: f64| !g.circle_hits_wall(x, z, r);
            if reach(sx, sz) && reach(ex, ez) {
                let (ox, oz) = (cx - 6.0, cz - 6.0);
                let size = (12.0 / step) as i32;
                let mut seen = vec![false; (size * size) as usize];
                let cell = |x: f64, z: f64| (((x - ox) / step).round() as i32, ((z - oz) / step).round() as i32);
                let (i0, j0) = cell(sx, sz);
                let (i1, j1) = cell(ex, ez);
                let mut q = vec![(i0, j0)];
                seen[(j0 * size + i0) as usize] = true;
                let mut crossed = false;
                while let Some((i, j)) = q.pop() {
                    if (i - i1).abs() <= 1 && (j - j1).abs() <= 1 {
                        crossed = true;
                        break;
                    }
                    for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (ni, nj) = (i + di, j + dj);
                        if ni < 0 || nj < 0 || ni >= size || nj >= size || seen[(nj * size + ni) as usize] {
                            continue;
                        }
                        let (x, z) = (ox + ni as f64 * step, oz + nj as f64 * step);
                        // stay round the barricade: a way round by another door is not a gap in it
                        if (x - cx).abs() > 3.0 && (z - cz).abs() > 3.0 {
                            continue;
                        }
                        if if b.along_x { (z - cz).abs() > 2.2 } else { (x - cx).abs() > 2.2 } {
                            continue;
                        }
                        seen[(nj * size + ni) as usize] = true;
                        if reach(x, z) {
                            q.push((ni, nj));
                        }
                    }
                }
                assert!(!crossed, "seed {seed} L{l}: a body gets past the barricade on tile {}", b.tile);
            }
            // never over the ceiling (settling would raise the ceiling to fit it)
            assert!(
                b.top <= g.ceil_at_world(cx, cz) - 0.04,
                "seed {seed} L{l}: barricade on tile {} stands {:.2} m under a {:.2} m ceiling",
                b.tile,
                b.top,
                g.ceil_at_world(cx, cz)
            );
            let obj = g.world.children().into_iter().find(|o| {
                let p = o.pos();
                o.user().s.get("kind").is_some_and(|k| k == "barricades") && (p.x - cx).abs() < 0.3 && (p.z - cz).abs() < 0.3
            });
            let obj = obj.unwrap_or_else(|| panic!("seed {seed} L{l}: no barricade drawn on tile {}", b.tile));
            obj.update_world_matrix(true, true);
            let mut bb = engine::prelude::Box3::empty();
            bb.set_from_object(&obj);
            assert!(
                bb.max.y <= g.ceil_at_world(cx, cz) - 0.02,
                "seed {seed} L{l}: barricade on tile {} reaches {:.2} m under a {:.2} m ceiling",
                b.tile,
                bb.max.y,
                g.ceil_at_world(cx, cz)
            );
            kinds += 1;
        }
    });
    println!("{n} barricades on {} floors", SEEDS.len() * LEVELS.len());
    assert!(n > 20 && kinds == n, "only {n} barricades");
}

/// A floor stays whole round its barricades: the exit is reached from
/// where the floor begins along the ways the creatures walk (`bfs_open`),
/// every pickup is in reach, and her walk never crosses one.
#[test]
fn the_floor_stays_whole_round_them() {
    each_floor(|seed, l, g| {
        if g.barricades.is_empty() {
            return;
        }
        let (sx, sz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
        let d = g.bfs_open(sx, sz);
        let at = |p: engine::prelude::Vec3| d.get(g.t_idx(g.tx(p.x), g.tz(p.z))).copied().unwrap_or(-1);
        let e = g.exit_data.as_ref().expect("an exit").pos;
        assert!(at(e) >= 0, "seed {seed} L{l}: the exit is cut off");
        for p in g.almonds.iter().chain(g.battery_items.iter()) {
            assert!(at(p.g.pos()) >= 0, "seed {seed} L{l}: a pickup at ({:.1}, {:.1}) is cut off", p.g.pos().x, p.g.pos().z);
        }
        for t in &g.story.walk {
            assert!(!g.barricade_tiles.contains(&g.t_idx(t.0, t.1)), "seed {seed} L{l}: her walk crosses a barricade at {t:?}");
        }
    });
}

/// Who would block a way blocks it with what the place had: boards and
/// sheets in plain places, an office's furniture in an office wing; never
/// on a floor with rules of its own, never more than one on the Lobby.
#[test]
fn where_barricades_stand() {
    each_floor(|seed, l, g| {
        if g.level_scp.is_some() {
            assert!(g.barricades.is_empty(), "seed {seed} L{l}: a barricade on {:?}", g.level_scp);
        }
        if l == 4 {
            assert!(g.barricades.len() <= 1, "seed {seed}: {} barricades on the Lobby", g.barricades.len());
        }
        for b in &g.barricades {
            let (cx, cz) = ((b.x0 + b.x1) / 2.0, (b.z0 + b.z1) / 2.0);
            let w = if b.along_x { b.z1 - b.z0 } else { b.x1 - b.x0 };
            assert!((0.9..=4.1).contains(&w), "seed {seed} L{l}: an opening {w:.2} m wide");
            assert!(
                g.spawn_pos.x.dhypot(0.0) >= 0.0 && (cx - g.spawn_pos.x).dhypot(cz - g.spawn_pos.z) >= 6.0,
                "seed {seed} L{l}: a barricade by the spawn"
            );
        }
    });
}
