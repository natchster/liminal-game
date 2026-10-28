//! Who lives where (`docs/ENTITIES.md` design 5, the ecology): on real
//! levels of several runs, the creatures a level is given by its places
//! (`Game::ecology`: one on the Lobby, two at depth 2, three deeper; the
//! Smiler only with dark regions, the Watcher only with furnished rooms,
//! the Reader only with paper lying about; each in its own kind of place);
//! where the Reader lives (`reader::place`: beside the paper, far from the
//! arrival and her camp); and where the one in the corner stands
//! (`corner::place`: in an inside corner off the main way, far from the
//! arrival, the door and her camp). Each level is built once for all three.
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

#[test]
fn who_lives_where() {
    let (mut levels, mut corners_placed, mut readers_placed, mut with_paper) = (0, 0, 0, 0);
    let mut seen_kinds = std::collections::BTreeMap::<&str, u32>::new();
    for seed in [4242u32, 77, 9001, 1999, 31337] {
        for level in 4..=12 {
            let mut g = Game::new_bare();
            register_placers(&mut g.placers);
            g.launch_flags = format!("seed={seed}&level={level}&nosound");
            g.boot();
            g.start_new_game();
            if g.building.is_some() || matches!(g.level_scp.as_deref(), Some("store" | "house")) {
                continue;
            }
            levels += 1;
            // ---- the ecology
            let want = g.ecology();
            let sent: Vec<&str> = g.entities.iter().map(|e| e.kind.as_str()).filter(|k| *k != "corner").collect();
            let depth = g.depth();
            let count = match depth {
                1 => 1,
                2 => 2,
                _ => 3,
            };
            assert_eq!(want.len(), count, "seed {seed} L{level}: {want:?} at depth {depth}");
            let mut placed = sent.clone();
            placed.sort();
            let mut wanted = want.clone();
            wanted.sort();
            // (the Reader only where it found a place beside the paper)
            if !sent.contains(&"reader") {
                wanted.retain(|k| *k != "reader");
            }
            assert_eq!(placed, wanted, "seed {seed} L{level}: placed {sent:?}, the places called for {want:?}");
            let (mut dark, mut furnished) = (0, 0);
            for z in 1..g.h - 1 {
                for x in 1..g.w - 1 {
                    if g.is_wall_tile(x, z) {
                        continue;
                    }
                    if g.region_light_at(x, z) < game::entities::DARK {
                        dark += 1;
                    }
                    if matches!(g.zone_purpose_at(x, z).map(|p| p.label), Some("living quarters" | "parlour")) {
                        furnished += 1;
                    }
                }
            }
            if want.contains(&"smiler") {
                assert!(dark >= game::entities::DARK_TILES, "seed {seed} L{level}: a Smiler with no dark region");
            }
            if want.contains(&"watcher") {
                assert!(furnished >= game::entities::FURNISHED_TILES, "seed {seed} L{level}: a Watcher with no furnished rooms");
            }
            if want.contains(&"reader") {
                assert!(g.papers.iter().any(|p| !p.taken), "seed {seed} L{level}: a Reader with no paper");
            }
            for e in &g.entities {
                let p = e.pos();
                let (x, z) = (g.tx(p.x), g.tz(p.z));
                match e.kind.as_str() {
                    "smiler" => assert!(g.region_light_at(x, z) < game::entities::DARK, "seed {seed} L{level}: a Smiler in the light"),
                    "watcher" => assert!(
                        matches!(g.zone_purpose_at(x, z).map(|p| p.label), Some("living quarters" | "parlour")),
                        "seed {seed} L{level}: a Watcher outside the furnished rooms"
                    ),
                    _ => {}
                }
                *seen_kinds
                    .entry(match e.kind.as_str() {
                        "smiler" => "smiler",
                        "hound" => "hound",
                        "watcher" => "watcher",
                        "reader" => "reader",
                        _ => "corner",
                    })
                    .or_insert(0) += 1;
            }
            println!("seed {seed} L{level} (depth {depth}): {sent:?}");
            // ---- the Reader
            let piles: Vec<(i32, i32)> = g.papers.iter().map(|p| (g.tx(p.g.pos().x), g.tz(p.g.pos().z))).collect();
            if !piles.is_empty() {
                with_paper += 1;
            }
            let readers: Vec<usize> = (0..g.entities.len()).filter(|&i| g.entities[i].kind == "reader").collect();
            assert!(readers.len() <= 1, "seed {seed} L{level}: {} of them", readers.len());
            if let Some(&ei) = readers.first() {
                readers_placed += 1;
                let p = g.entities[ei].pos();
                let (x, z) = (g.tx(p.x), g.tz(p.z));
                let i = g.t_idx(x, z);
                assert!(!g.is_wall_tile(x, z), "seed {seed} L{level}: in a wall");
                let near = piles.iter().map(|&(px, pz)| (px - x).abs().max((pz - z).abs())).min();
                assert!(near.map(|d| (1..=6).contains(&d)).unwrap_or(false), "seed {seed} L{level}: not beside paper ({near:?} tiles)");
                let from_start = g.bfs(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
                assert!(from_start[i] >= 18, "seed {seed} L{level}: {} tiles from the arrival", from_start[i]);
                if let Some(c) = g.story.camp.as_ref() {
                    let from_camp = g.bfs(c.tile.0, c.tile.1);
                    assert!(from_camp[i] < 0 || from_camp[i] >= 5, "seed {seed} L{level}: {} tiles from her camp", from_camp[i]);
                }
                println!(
                    "seed {seed} L{level}: the Reader at ({x}, {z}), {} tiles from the arrival, paper {} tiles off",
                    from_start[i],
                    near.unwrap()
                );
            }
            // ---- the one in the corner
            let corners: Vec<usize> = (0..g.entities.len()).filter(|&i| g.entities[i].kind == "corner").collect();
            assert!(corners.len() <= 1, "seed {seed} L{level}: {} of them", corners.len());
            let Some(&ei) = corners.first() else { continue };
            corners_placed += 1;
            let p = g.entities[ei].pos();
            let (x, z) = (g.tx(p.x), g.tz(p.z));
            let i = g.t_idx(x, z);
            let e = g.exit_data.as_ref().unwrap().pos;
            let from_exit = g.bfs(g.tx(e.x), g.tz(e.z));
            let from_start = g.bfs(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            assert!(from_start[i] >= 8, "seed {seed} L{level}: {} tiles from the arrival", from_start[i]);
            assert!(from_exit[i] >= 5, "seed {seed} L{level}: {} tiles from the door", from_exit[i]);
            if let Some(c) = g.story.camp.as_ref() {
                let from_camp = g.bfs(c.tile.0, c.tile.1);
                assert!(from_camp[i] < 0 || from_camp[i] >= 5, "seed {seed} L{level}: {} tiles from her camp", from_camp[i]);
            }
            // off the walk from the arrival to the door
            let (mut wx, mut wz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            loop {
                assert!((wx - x).abs().max((wz - z).abs()) > 2, "seed {seed} L{level}: on the way at ({wx}, {wz})");
                let here = from_exit[g.t_idx(wx, wz)];
                if here <= 0 {
                    break;
                }
                let next = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .map(|(dx, dz)| (wx + dx, wz + dz))
                    .find(|&(nx, nz)| g.in_grid(nx, nz) && from_exit[g.t_idx(nx, nz)] == here - 1);
                let Some((nx, nz)) = next else { break };
                wx = nx;
                wz = nz;
            }
            // in an inside corner, its face to the angle of the walls
            let yaw = g.entities[ei].yaw;
            let (cx, cz) = (yaw.sin().signum() as i32, yaw.cos().signum() as i32);
            assert!(g.is_wall_tile(x + cx, z) && g.is_wall_tile(x, z + cz), "seed {seed} L{level}: not in a corner");
        }
    }
    println!("levels {levels}: kinds {seen_kinds:?}; the Reader on {readers_placed} ({with_paper} with paper), the one in the corner on {corners_placed}");
    for kind in ["smiler", "hound", "watcher", "reader", "corner"] {
        assert!(seen_kinds.get(kind).copied().unwrap_or(0) > 0, "no {kind} on any level");
    }
    assert!(corners_placed > 0 && corners_placed < levels, "the one in the corner on {corners_placed} of {levels}");
}
