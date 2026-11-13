//! Whole-level smoke test of `build_world` with every prop placer registered:
//! levels 1..12 for several run seeds, the seeded part of `init_level`
//! (`gen_layout_level` + `build_world` under `with_seed(seed_for_level(run_seed, n))`).
//! Asserts no panics and sane counts, and writes the per-level seeded draw
//! counts and every trace mark (one per `themed()` pass, one per lap) to
//! `$CARGO_TARGET_TMPDIR/world_marks.json` for inspection.
use game::base::rng::{seed_for_level, seeded_draws, with_seed};
use game::Game;

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
            let placed =
                g.world.children().iter().filter(|c| c.r().user.placer.as_deref().map(|p| p != "place_props").unwrap_or(false)).count();
            eprintln!("run {run_seed:#010x} level {n:2} scp {:9} weather {:8} {}x{}: {draws:6} draws, {:3} marks, themes {:3} ran / {:3} skipped, {kids:4} children ({meshes:5} meshes, {placed:4} themed), {:3} panels, {:3} anim, {:2} inter, settle {:?}, {ms:.0} ms",
                g.level_scp.as_deref().unwrap_or("-"), g.weather_key(), g.w, g.h, g.rand_trace.marks.len(), g.theme_stats_ran, g.theme_stats_skipped, g.panels.len(), g.animators.len(), g.interactables.len(),
                (g.settle_stats.get("attached"), g.settle_stats.get("moved"), g.settle_stats.get("pinned"), g.settle_stats.get("raised")));
            // sane counts
            assert_eq!(g.rand_trace.marks.len(), 2 + 7 + game::world::tables::THEMED_ORDER.len(), "marks");
            assert_eq!((g.theme_stats_ran + g.theme_stats_skipped) as usize, game::world::tables::THEMED_ORDER.len());
            assert!(g.theme_stats_ran > 0, "some placers ran");
            assert!(kids > 50 && meshes > 50, "a world was built");
            assert!(g.solids.as_ref().map(|s| s.count).unwrap_or(0) > 100, "solids");
            assert!(g.tiles.iter().all(|&t| t <= 1), "no masked tile left behind");
            assert!(g.world.parent().is_some(), "world is in the scene");
            let marks: Vec<String> = g.rand_trace.marks.iter().map(|(l, k)| format!("[\"{}\",{}]", l, k - d0)).collect();
            if !first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(&format!(
                "{{\"runSeed\":{run_seed},\"level\":{n},\"scp\":{},\"seed\":{seed},\"draws\":{draws},\"marks\":[{}]}}",
                g.level_scp.as_ref().map(|s| format!("\"{s}\"")).unwrap_or("null".into()),
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
