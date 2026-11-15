//! Whole-level smoke test of `build_world` with every prop placer registered:
//! levels 1..12 for several run seeds, the seeded part of `init_level`
//! (`gen_layout_level` + `build_world` under `with_seed(seed_for_level(run_seed, n))`).
//! Asserts no panics and sane counts, and writes the per-level seeded draw
//! counts and every trace mark (one per `themed()` pass, one per lap) to
//! `$CARGO_TARGET_TMPDIR/world_marks.json` for inspection.
use game::Game;
use game::base::rng::{seed_for_level, seeded_draws, with_seed};

/// Held by this binary's tests that paint: CoreGraphics rasterizes
/// differently when two threads draw at once, so a test comparing pixels
/// paints alone (the check's pool runs each test in a process of its own)
static PAINTING: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn every_themed_placer_is_registered() {
    let mut r = vec![];
    placers::register_all(&mut r);
    let names: std::collections::HashSet<&str> = r.iter().map(|(n, _)| *n).collect();
    let missing: Vec<&str> = game::world::tables::THEMED_ORDER.iter().copied().filter(|n| !names.contains(n)).collect();
    assert!(missing.is_empty(), "themed placers missing from the registry: {missing:?}");
}

#[test]
fn levels_build_with_all_placers() {
    let _alone = PAINTING.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut out = String::from("{\"levels\":[\n");
    let mut first = true;
    for run_seed in [0x1234_5678u32, 0x0BAD_F00D, 2026] {
        // one game per run, levels in order (as a playthrough builds them)
        let mut g = Game::new_bare();
        placers::register_all(&mut g.placers);
        g.init_world();
        g.run_seed = run_seed;
        g.rand_trace.on = true;
        for n in 1..=12 {
            let t0 = std::time::Instant::now();
            g.level_num = n;
            g.level_scp = game::level::scp::scp_for_level(run_seed, n).map(|s| s.to_string());
            g.current_seed = seed_for_level(run_seed, n);
            g.pick_level_weather();
            g.rand_trace.marks.clear();
            let d0 = seeded_draws();
            let seed = g.current_seed;
            with_seed(seed, || {
                g.gen_layout_level(n, (31 + n * 4).min(53));
                g.trace_mark("layout");
                g.build_world();
                g.trace_mark("world");
            });
            let draws = seeded_draws() - d0;
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let kids = g.world.child_count();
            let mut meshes = 0usize;
            g.world.traverse(&mut |o| {
                if o.is_mesh() {
                    meshes += 1;
                }
            });
            let placed = g.world.children().iter().filter(|c| c.r().user.placer.as_deref().is_some_and(|p| p != "place_props")).count();
            eprintln!(
                "run {run_seed:#010x} level {n:2} scp {:9} weather {:8} {}x{}: {draws:6} draws, {:3} marks, themes {:3} ran / {:3} skipped, {kids:4} children ({meshes:5} meshes, {placed:4} themed), {:3} panels, {:3} anim, {:2} inter, settle {:?}, {ms:.0} ms",
                g.level_scp.as_deref().unwrap_or("-"),
                g.weather_key(),
                g.w,
                g.h,
                g.rand_trace.marks.len(),
                g.theme_stats_ran,
                g.theme_stats_skipped,
                g.panels.len(),
                g.animators.len(),
                g.interactables.len(),
                (g.settle_stats.get("attached"), g.settle_stats.get("moved"), g.settle_stats.get("pinned"), g.settle_stats.get("raised"))
            );
            // sane counts
            assert_eq!(g.rand_trace.marks.len(), 2 + 7 + game::world::tables::THEMED_ORDER.len(), "marks");
            assert_eq!((g.theme_stats_ran + g.theme_stats_skipped) as usize, game::world::tables::THEMED_ORDER.len());
            assert!(g.theme_stats_ran > 0, "some placers ran");
            assert!(kids > 50 && meshes > 50, "a world was built");
            assert!(g.solids.as_ref().map_or(0, |s| s.count) > 100, "solids");
            assert!(g.tiles.iter().all(|&t| t <= 1), "no masked tile left behind");
            assert!(g.world.parent().is_some(), "world is in the scene");
            let marks: Vec<String> = g.rand_trace.marks.iter().map(|(l, k)| format!("[\"{}\",{}]", l, k - d0)).collect();
            if !first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(&format!(
                "{{\"runSeed\":{run_seed},\"level\":{n},\"scp\":{},\"seed\":{seed},\"draws\":{draws},\"marks\":[{}]}}",
                g.level_scp.as_ref().map_or("null".into(), |s| format!("\"{s}\"")),
                marks.join(",")
            ));
            // the next level frees what this one made
            g.dispose_unused();
        }
    }
    out.push_str("\n]}\n");
    let p = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("world_marks.json");
    std::fs::write(&p, out).unwrap();
    eprintln!("marks written to {}", p.display());
}

/// Every texture of a floor as it was built on this thread, in the order the
/// scene holds them: its name and a hash of its pixels (each read through
/// `Tex::image`, which paints it first if its painting waits)
fn floor_textures(seed: u32, level: i32, on_demand: bool) -> (Vec<(String, u64)>, usize) {
    // (a fresh thread: nothing of what the thread made before, as a run's
    // process starts)
    std::thread::spawn(move || floor_textures_here(seed, level, on_demand)).join().expect("the floor's thread")
}
fn floor_textures_here(seed: u32, level: i32, on_demand: bool) -> (Vec<(String, u64)>, usize) {
    engine::texture::set_paint_on_demand(on_demand);
    // (the boot's textures draw on the per-frame stream: the same for both,
    // as `--fresh` makes it)
    game::base::rng::seed_native(u64::from(seed) << 8 | level as u64);
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    let waiting = engine::texture::unpainted();
    let mut out = vec![];
    let mut seen = std::collections::HashSet::new();
    g.scene.traverse(&mut |o| {
        for m in o.mats() {
            let mm = m.m();
            let detail = match &mm.patch {
                engine::material::Patch::ArchUV { detail: Some(d), .. } => Some(d.map.clone()),
                _ => None,
            };
            for t in [&mm.map, &mm.alpha_map, &mm.bump_map, &mm.emissive_map, &detail].into_iter().flatten() {
                if !seen.insert(t.id()) {
                    continue;
                }
                let img = t.image();
                let im = img.borrow();
                // FNV-1a over its size and pixels
                let mut h: u64 = 0xcbf2_9ce4_8422_2325;
                for b in im.width.to_le_bytes().into_iter().chain(im.height.to_le_bytes()).chain(im.data.iter().copied()) {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0100_0000_01b3);
                }
                let placer = o.user().placer.clone().unwrap_or_default();
                let name = o.name();
                let who = format!("{placer}|{name}|{}x{}", im.width, im.height);
                out.push((who, h));
            }
        }
    });
    (out, waiting)
}

/// Painted on demand (`engine::texture::paint_on_demand`: the tests' and
/// the headless tools' way, where most of a floor's textures are never
/// looked at), a floor's textures have the very pixels of one painted at
/// once (the game's window's way): each painter keeps the seed it was given
/// as its texture was made, whenever it paints. Nearly every texture of a
/// floor waits until it is read.
#[test]
fn a_floor_painted_on_demand_has_the_pixels_of_one_painted_at_once() {
    let _alone = PAINTING.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    for (seed, level) in [(4242u32, 5), (7, 9), (77, 2)] {
        let (at_once, none) = floor_textures(seed, level, false);
        let (on_demand, waiting) = floor_textures(seed, level, true);
        assert_eq!(none, 0, "{seed}/{level}: painted at once, nothing waits");
        assert!(waiting > at_once.len() / 2, "{seed}/{level}: only {waiting} of {} textures waited", at_once.len());
        assert_eq!(at_once.len(), on_demand.len(), "{seed}/{level}: the floor holds as many textures");
        let differ: Vec<&String> = at_once.iter().zip(&on_demand).filter(|(a, b)| a != b).map(|(a, _)| &a.0).collect();
        assert!(differ.is_empty(), "{seed}/{level}: painted on demand, these differ: {differ:?}");
    }
}
