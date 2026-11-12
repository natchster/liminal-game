//! The visual playtest (`docs/VISUAL_PLAYTEST.md`). The tour (VP1): every
//! stand is one a body can stand on, at an edge it cannot pass, facing it;
//! the tour is the same twice; it is spread over the level; it keeps to an
//! area and a budget; it looks up, down, along and crouched. Its renders and
//! pixel detectors (VP2, `backrooms --tour`): a hole and a fight planted are
//! found, the clean view and the void over a parapet are not.
//!
//!   cargo test --release -p backrooms --test visual_tour -- --nocapture   # stands and views per floor
mod common;

use game::debug::sight::FLAG_PX;
use game::debug::tour::{TourOpts, BODY};
use game::prelude::{CROUCH_EYE, EYE};
use game::Game;

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = common::new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g
}

/// A Building floor and a floor of the rooms
const FLOORS: [(u32, i32); 2] = [(4242, 1), (4242, 5)];

#[test]
fn every_stand_is_at_an_edge_a_body_can_reach() {
    for (seed, level) in FLOORS {
        let g = floor(seed, level);
        let stands = g.tour_stands(None);
        assert!(stands.len() > 40, "L{level}: only {} stands", stands.len());
        let mut kinds = std::collections::BTreeMap::new();
        for s in &stands {
            assert!(g.standable(s.x, s.z), "L{level}: a stand a body cannot stand on: {s:?}");
            // a step further toward the edge is not one a body can take,
            // and the edge is a body's radius away, not across the room
            let (fx, fz) = (-s.yaw.sin(), -s.yaw.cos());
            assert!(
                g.circle_hits_wall(s.x + fx * 0.05, s.z + fz * 0.05, BODY) || !g.standable(s.x + fx * 0.05, s.z + fz * 0.05),
                "L{level}: {s:?}"
            );
            let reach = if s.corner { 0.05 } else { 0.03 };
            assert!(!g.standable(s.x + fx * reach, s.z + fz * reach), "L{level}: a stand not at its edge: {s:?}");
            // the floor under it is the floor its eye is put over
            assert!((g.floor_y_at(s.x, s.z) - s.floor).abs() < 0.31, "L{level}: {s:?}");
            *kinds.entry(s.edge).or_insert(0) += 1;
        }
        println!("L{level}: {} stands {kinds:?}", stands.len());
        if level <= 3 {
            assert!(kinds.contains_key("void"), "L{level}: the Building has edges over the void: {kinds:?}");
        }
        assert!(kinds.contains_key("wall") || kinds.contains_key("solid"), "L{level}: {kinds:?}");
        assert!(stands.iter().any(|s| s.corner), "L{level}: no corners");
    }
}

#[test]
fn the_tour_is_the_same_twice_and_spread_over_the_level() {
    for (seed, level) in FLOORS {
        let o = TourOpts { budget: 120, area: None };
        let a = floor(seed, level).tour_views(&o);
        let b = floor(seed, level).tour_views(&o);
        assert_eq!(a, b, "L{level}: a tour must not depend on anything but the level");
        assert!(a.len() <= o.budget && a.len() > o.budget - 7, "L{level}: {} views for a budget of {}", a.len(), o.budget);
        // spread: the chosen stands reach most of the way across every stand's extent
        let g = floor(seed, level);
        let all = g.tour_stands(None);
        let span = |f: &dyn Fn(f64, f64) -> f64, xs: &mut dyn Iterator<Item = (f64, f64)>| {
            let v: Vec<f64> = xs.map(|(x, z)| f(x, z)).collect();
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        for (axis, f) in [("x", &(|x: f64, _z: f64| x) as &dyn Fn(f64, f64) -> f64), ("z", &|_x: f64, z: f64| z)] {
            let whole = span(f, &mut all.iter().map(|s| (s.x, s.z)));
            let toured = span(f, &mut a.iter().map(|v| (v.x, v.z)));
            assert!(toured > 0.8 * whole, "L{level}: the tour spans {toured:.1} m of {whole:.1} m in {axis}");
        }
        // and every look is there, crouched at a crouching eye
        for look in ["at", "left", "right", "up", "down", "crouch"] {
            assert!(a.iter().any(|v| v.look == look), "L{level}: no {look} view");
        }
        for v in &a {
            let floor = g.floor_y_at(v.x, v.z);
            let eye = if v.look == "crouch" { CROUCH_EYE } else { EYE };
            assert!((v.y - floor - eye).abs() < 0.31, "L{level}: {} at eye {:.2} over {floor:.2}", v.name, v.y);
            assert!(v.pitch.abs() < 1.2, "{}", v.name);
        }
        let names: std::collections::BTreeSet<&str> = a.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names.len(), a.len(), "L{level}: two views with one name");
        println!("L{level}: {} views; first {:?}", a.len(), a[0]);
    }
}

#[test]
fn an_area_keeps_the_tour_inside_it() {
    let (seed, level) = FLOORS[0];
    let g = floor(seed, level);
    // the middle quarter of the level's stands
    let all = g.tour_stands(None);
    let (mut xs, mut zs): (Vec<f64>, Vec<f64>) = all.iter().map(|s| (s.x, s.z)).unzip();
    xs.sort_by(f64::total_cmp);
    zs.sort_by(f64::total_cmp);
    let area = [xs[xs.len() / 4], zs[zs.len() / 4], xs[xs.len() * 3 / 4], zs[zs.len() * 3 / 4]];
    let views = g.tour_views(&TourOpts { budget: 60, area: Some(area) });
    assert!(!views.is_empty(), "no views inside {area:?}");
    for v in &views {
        assert!(v.x >= area[0] && v.x <= area[2] && v.z >= area[1] && v.z <= area[3], "{} outside {area:?}", v.name);
    }
    // nothing at all for an area off the level
    assert!(g.tour_views(&TourOpts { budget: 60, area: Some([1e4, 1e4, 1e4 + 8.0, 1e4 + 8.0]) }).is_empty());
}

/// A view's row of `views.tsv`
#[derive(Debug)]
struct Row {
    name: String,
    look: String,
    edge: String,
    background: usize,
    gaps: usize,
    flicker: usize,
}

/// `backrooms --tour` on one floor (the real renders), its rows
fn toured(name: &str, level: i32, extra: &[&str]) -> Vec<Row> {
    let dir = format!("{}/visual_tour/{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::remove_dir_all(&dir);
    let l = level.to_string();
    let mut args = vec!["--tour", &dir, "--seed", "4242", "--levels", &l, "--width", "480", "--height", "300"];
    args.extend_from_slice(extra);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_backrooms")).args(&args).output().expect("run backrooms --tour");
    assert!(out.status.success(), "{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    println!("{name}: {}", String::from_utf8_lossy(&out.stdout).trim());
    let tsv = std::fs::read_to_string(format!("{dir}/L{level}/views.tsv")).expect("views.tsv");
    let mut lines = tsv.lines();
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap_or_else(|| panic!("no column {n}"));
    lines
        .map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            let num = |n: &str| c[col(n)].parse::<usize>().unwrap();
            Row {
                name: c[col("name")].into(),
                look: c[col("look")].into(),
                edge: c[col("edge")].into(),
                background: num("background"),
                gaps: num("gaps"),
                flicker: num("flicker"),
            }
        })
        .collect()
}

/// The detectors' known answers (`docs/VISUAL_PLAYTEST.md` VP2): a hole
/// cut where the tour's first stand faces a wall is found in the view facing
/// it, and two squares fighting for one depth before its eye are found where
/// it sees them at a grazing angle (looking up past them: face on, their
/// fight is thin lines that move like an edge); the same views of the floor
/// as built are clean
#[test]
fn a_planted_hole_and_fight_are_found_and_the_clean_views_are_not() {
    let clean = toured("clean", 5, &["--tour-budget", "6"]);
    let hole = toured("hole", 5, &["--tour-budget", "6", "--tour-plant", "hole"]);
    let fight = toured("fight", 5, &["--tour-budget", "6", "--tour-plant", "fight"]);
    let view = |rows: &[Row], look: &str| rows.iter().find(|r| r.look == look).map(|r| (r.name.clone(), r.gaps, r.flicker)).unwrap();
    for look in ["at", "up"] {
        let (c, h, f) = (view(&clean, look), view(&hole, look), view(&fight, look));
        println!("{look}: clean {c:?} hole {h:?} fight {f:?}");
        assert!(c.1 < FLAG_PX && c.2 < FLAG_PX, "the clean view: {c:?}");
    }
    assert!(view(&hole, "at").1 >= FLAG_PX, "the hole is not found: {:?}", view(&hole, "at"));
    assert!(view(&fight, "up").2 >= FLAG_PX, "the fight is not found: {:?}", view(&fight, "up"));
}

/// Looking over a parapet into the void, the background shows (nothing is
/// drawn down there within the far plane) and it is no gap: on the Works,
/// every view over the void with the background in it is clean of gaps
#[test]
fn the_void_over_a_parapet_is_not_a_gap() {
    let rows = toured("void", 3, &["--tour-budget", "120"]);
    let open: Vec<&Row> = rows.iter().filter(|r| r.edge == "void" && r.background > 1000).collect();
    println!("{} views over the void with the background in them", open.len());
    assert!(open.len() >= 3, "{rows:?}");
    for r in &open {
        assert!(r.gaps < FLAG_PX, "the void is not a hole: {r:?}");
    }
}
