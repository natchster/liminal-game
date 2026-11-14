//! The windows of the rooms (`docs/design/WINDOWS.md`): where they stand, the
//! openings cut for them, the light they throw in, what they look out on,
//! and that they are the same on every build.
mod common;
use game::Game;
use game::core_props::window::{Kind, SLIVER, model, view};
use game::prelude::*;

/// Every floor of the rooms these seeds build, with what it is
fn floors(seeds: u32, levels: std::ops::RangeInclusive<i32>, mut f: impl FnMut(&mut Game, u32, i32)) {
    let mut g = common::new_game();
    for k in 0..seeds {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x51_D0;
        for level in levels.clone() {
            g.init_level(level);
            let seed = g.run_seed;
            f(&mut g, seed, level);
        }
    }
}

/// A level with a window of `kind`: (game, its index)
fn level_with(kind: Kind) -> (Game, usize) {
    let mut g = common::new_game();
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x51_D0;
        for level in [5, 6, 7, 9] {
            g.init_level(level);
            if let Some(i) = g.windows.iter().position(|w| w.kind == kind) {
                g.state = "playing".into();
                return (g, i);
            }
        }
    }
    panic!("no level with a {kind:?} window");
}

/// The tile (x, z) of a world point
fn tile(g: &Game, p: &Vec3) -> (i32, i32) {
    (g.tx(p.x), g.tz(p.z))
}

#[test]
fn windows_stand_where_a_building_would_have_them() {
    let (mut floors_with, mut eligible, mut rows, mut lights, mut planned) = (0, 0, 0, 0, 0);
    floors(14, 4..=11, |g, seed, level| {
        let scp = g.level_scp.clone();
        if level == 4 || matches!(scp.as_deref(), Some("store" | "house")) {
            assert!(g.windows.is_empty(), "seed {seed} level {level} ({scp:?}): {} windows where none belong", g.windows.len());
            return;
        }
        eligible += 1;
        planned += g.window_plans.len();
        floors_with += (!g.windows.is_empty()) as i32;
        rows += g.windows.len();
        assert!(g.windows.len() <= 3, "seed {seed} level {level}: {} rows", g.windows.len());
        let l = g.layout.as_ref().unwrap();
        for w in &g.windows {
            lights += w.lights.len();
            // the zone of the first open tile before it is one a building
            // would have windows in (its wall's tile may be its neighbour's)
            let mut out = w.depth + 0.1;
            let mut before = w.world(0.0, 1.0, out);
            while g.is_wall_tile(g.tx(before.x), g.tz(before.z)) && out < w.depth + 4.5 {
                out += 0.25;
                before = w.world(0.0, 1.0, out);
            }
            let (tx, tz) = tile(g, &before);
            let zi = l.zone_of[g.t_idx(tx, tz)] as usize;
            let (arch, p) = (&l.zones[zi].arch, &g.zone_purposes[zi]);
            let home = ["office", "hall"].contains(arch) || p.has_any(&["office", "domestic", "victorian", "party"]);
            assert!(home, "seed {seed} level {level}: a window in a {arch} zone ({:?})", p.primary);
            assert!(p.primary.is_some(), "seed {seed} level {level}: a window in a zone left bare");
            // office strip windows in offices, sash windows elsewhere
            let office = *arch == "office" || p.has_any(&["office"]);
            assert_eq!(w.kind == Kind::Strip, office, "seed {seed} level {level}: a {:?} window in a {arch} zone", w.kind);
            // under the ceiling
            assert!(w.head + 0.1 < g.ceil_at_world(before.x, before.z), "seed {seed} level {level}: the head over the ceiling");
        }
    });
    println!("windows on {floors_with} of {eligible} floors: {rows} rows of {planned} chosen, {lights} lights");
    assert!(floors_with * 2 >= eligible, "windows on only {floors_with} of {eligible} floors that may have them");
}

#[test]
fn a_window_is_cut_into_its_wall_and_the_wall_stays_solid() {
    for kind in [Kind::DoubleHung, Kind::Strip] {
        let (g, i) = level_with(kind);
        let w = &g.windows[i];
        // the walls as drawn: every wall box
        let mut walls: Vec<[f64; 6]> = vec![];
        g.world.traverse(&mut |o: &Obj| {
            if !o.is_instanced() || !matches!(o.mat().r().patch, Patch::ArchUV { .. }) {
                return;
            }
            for k in 0..o.instance_count() {
                let m = o.get_matrix_at(k);
                let (p, _, s) = m.decompose();
                walls.push([p.x - s.x / 2.0, p.y - s.y / 2.0, p.z - s.z / 2.0, p.x + s.x / 2.0, p.y + s.y / 2.0, p.z + s.z / 2.0]);
            }
        });
        let inside = |p: &Vec3| walls.iter().any(|b| p.x > b[0] && p.x < b[3] && p.y > b[1] && p.y < b[4] && p.z > b[2] && p.z < b[5]);
        for l in &w.lights {
            let (cx, cy) = ((l.rect[0] + l.rect[1]) / 2.0, (l.rect[2] + l.rect[3]) / 2.0);
            // open through the reveal, from its face to its back
            for z in [w.depth * 0.9, w.depth * 0.5, 0.004] {
                let p = w.world(cx, cy, z);
                assert!(!inside(&p), "{kind:?}: the wall is drawn in its opening at depth {z:.3}");
                // ... but what you walk into is whole: the glass is solid
                assert!(g.circle_hits_wall(p.x, p.z, 0.01), "{kind:?}: the opening can be walked into at depth {z:.3}");
            }
            // the wall's back is still there (the far room sees a wall)
            let back = w.world(cx, cy, -SLIVER / 2.0);
            assert!(inside(&back), "{kind:?}: the cut goes through the wall's back");
            // and the wall round the opening is drawn
            let above = w.world(cx, w.head + 0.15, w.depth * 0.5);
            assert!(inside(&above) || above.y > g.ceil_at_world(above.x, above.z) - 0.05, "{kind:?}: no wall over the opening");
        }
    }
}

#[test]
fn windows_are_the_same_on_every_build() {
    let sig = |g: &Game| -> Vec<String> {
        g.windows
            .iter()
            .map(|w| {
                let o = w.root.pos();
                let blinds: Vec<String> = w
                    .lights
                    .iter()
                    .map(|l| format!("{:?}{}{}", l.blind.map(|b| (b.down * 100.0).round()), l.papered, l.patch.len()))
                    .collect();
                format!("{:?} {:.3} {:.3} {:.3} {} {} {}", w.kind, o.x, o.z, w.depth, w.lights.len(), w.seed, blinds.join(","))
            })
            .collect()
    };
    let mut g = common::new_game();
    g.run_seed = 4242;
    g.init_level(7);
    let a = sig(&g);
    g.init_level(9);
    g.init_level(5);
    g.init_level(7);
    let b = sig(&g);
    assert!(!a.is_empty(), "seed 4242 level 7 has windows");
    assert_eq!(a, b, "the same floor built twice");
}

#[test]
fn its_light_falls_on_the_floor_of_its_own_room() {
    let mut patches = 0;
    floors(8, 5..=9, |g, seed, level| {
        for w in &g.windows {
            let face = w.world(0.0, 0.0, w.depth);
            for (k, l) in w.lights.iter().enumerate() {
                if l.papered {
                    assert!(l.patch.is_empty(), "seed {seed} level {level}: paper lets the lamp through");
                    continue;
                }
                if l.patch.is_empty() {
                    continue;
                }
                patches += 1;
                for q in &l.patch {
                    for &(x, z) in q {
                        // out from the wall, into the room, as far as its lamp throws it
                        let out = (x - face.x) * w.normal.x + (z - face.z) * w.normal.z;
                        assert!((0.02..3.4).contains(&out), "seed {seed} level {level} light {k}: its light {out:.2} m out from its wall");
                    }
                    // on the floor of its own room: each cell's middle in sight
                    // of the opening, or under a wall that hides it
                    let (mx, mz) = ((q[0].0 + q[2].0) / 2.0, (q[0].1 + q[2].1) / 2.0);
                    let from = w.world((l.rect[0] + l.rect[1]) / 2.0, 1.0, w.depth + 0.05);
                    assert!(
                        g.los(from.x, from.z, mx, mz) || g.circle_hits_wall(mx, mz, 0.01),
                        "seed {seed} level {level} light {k}: its light lies behind a wall at {mx:.2},{mz:.2}"
                    );
                    assert!(w.lit_at(mx, mz));
                }
            }
        }
    });
    assert!(patches > 10, "only {patches} lights threw any light");
}

#[test]
fn every_window_looks_out_on_the_same_place_later_the_deeper_it_is() {
    let mut seen: Vec<(i32, f32)> = vec![];
    floors(10, 5..=12, |g, seed, level| {
        for w in &g.windows {
            let f = w.view.r().patch_uniforms.borrow().f;
            // the way ahead is straight out from the glass, whichever way it faces
            assert!(
                (f[4] as f64 + w.normal.x).abs() < 1e-6 && (f[5] as f64 + w.normal.z).abs() < 1e-6,
                "seed {seed} level {level}: the view turned"
            );
            assert!(matches!(w.view.r().patch, Patch::WindowView));
            // the point it is seen from is just outside the glass, over ground under the floor
            let o = Vec3::new(f[0] as f64, f[1] as f64, f[2] as f64);
            let local = w.local(&o);
            assert!(
                local.z.abs() < 0.01 && (local.y - (view::GROUND + view::CAMERA_H)).abs() < 0.01,
                "seed {seed} level {level}: seen from {local:?}"
            );
            seen.push((g.depth(), f[7]));
        }
    });
    seen.sort_by_key(|a| a.0);
    for p in seen.windows(2) {
        assert!(p[1].1 <= p[0].1 + 1e-6, "deeper is later: {p:?}");
    }
    let (first, last) = (seen.first().unwrap(), seen.last().unwrap());
    assert!(last.1 < first.1 * 0.6, "the light left: {first:?} to {last:?}");
}

#[test]
fn its_lamp_is_heard_through_the_glass_only_near_it() {
    let (mut g, i) = level_with(Kind::DoubleHung);
    let w = &g.windows[i];
    let far = w.world(0.0, 0.0, w.depth + 30.0);
    let near = w.world(0.0, 0.0, w.depth + 1.5);
    g.player.pos = Vec3::new(far.x, EYE, far.z);
    for _ in 0..30 {
        g.windows_frame(1.0 / 60.0);
    }
    assert!(g.windows.iter().all(|w| w.hum.is_none() || w.hum.as_ref().unwrap().level() < 0.01), "heard from 30 m");
    g.player.pos = Vec3::new(near.x, EYE, near.z);
    for _ in 0..120 {
        g.windows_frame(1.0 / 60.0);
    }
    if g.ac.is_some() {
        assert!(g.windows[i].hum.as_ref().map(|h| h.level() > 0.5).unwrap_or(false), "not heard at 1.5 m");
    }
    let _ = model::DH_W;
}
