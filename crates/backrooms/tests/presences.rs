//! What is seen across the Building's void (`docs/design/MEGASTRUCTURE.md`, B7;
//! `game::world::far_figures`): a figure far off where no walk reaches, who
//! turns its back when looked at and is gone when you look away, and never
//! lets you near; the door's light seen from the arrival; the Well's lift
//! cage over the void; the Works' last door, lined with the rooms' yellow,
//! opening on the Lobby.
mod common;

use engine::math::Vec3;
use game::level::archgen::{segment_clear_3d, GROUND_VOID};
use game::level::building::{door_lights, STOREY};
use game::world::far_figures::{FigureState, AWAY, FAR, NEAR, TOO_NEAR};
use game::Game;

const DT: f64 = 1.0 / 60.0;

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native(0xF16E_0000 ^ u64::from(seed) ^ ((level as u64) << 32));
    let mut g = common::new_game();
    // a new run picks its own run seed (from the native stream): the floor
    // asked for is built after it, with the seed asked for
    g.start_new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g.hide_overlay();
    g.pointer_locked = true;
    g
}

fn step(g: &mut Game, secs: f64) {
    for _ in 0..(secs / DT).round() as usize {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
    }
}

/// Turn the eye toward `p` (a figure's chest), or straight away from it
fn face(g: &mut Game, p: Vec3, toward: bool) {
    let c = g.player.pos;
    let (dx, dy, dz) = (p.x - c.x, p.y + 1.4 - c.y, p.z - c.z);
    let s = if toward { 1.0 } else { -1.0 };
    g.player.yaw = (-dx * s).atan2(-dz * s);
    g.player.pitch = if toward { dy.atan2((dx * dx + dz * dz).sqrt()) } else { 0.0 };
}

/// Stand where a figure could be seen (the debug screen's viewpoint), the
/// eye turned away, and let it choose where to stand
fn stand_where_it_is_seen(g: &mut Game) {
    g.god_mode = true;
    g.show_far_figure();
    let at = g.far_figure.as_ref().unwrap().at;
    let f = g.far_figure.as_mut().unwrap();
    f.state = FigureState::Gone;
    f.t = 0.05;
    f.body.set_visible(false);
    face(g, at, false);
    step(g, 0.5);
    assert_eq!(g.far_figure.as_ref().unwrap().state, FigureState::Standing, "it stands somewhere");
}

fn level_dist(a: Vec3, b: Vec3) -> f64 {
    ((a.x - b.x).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

#[test]
fn the_figure_stands_only_where_no_walk_reaches() {
    for level in 1..=3 {
        for seed in [4242u32, 7, 77] {
            let g = floor(seed, level);
            assert!(g.far_figure.is_some(), "seed {seed} L{level}: no figure");
            assert!(!g.figure_perches.is_empty());
            // every perch a storey off every walk tile round it: measured from
            // the walk tiles' own floors, not by the code's own filter
            for p in &g.figure_perches {
                let (u, v) = (g.to_u(p.x).floor() as i32, g.to_v(p.z).floor() as i32);
                for z in (v - 2).max(0)..(v + 3).min(g.h) {
                    for x in (u - 2).max(0)..(u + 3).min(g.w) {
                        if !g.ground[g.t_idx(x, z)].walkable() {
                            continue;
                        }
                        let (wx, wz) = (g.wx(x), g.wz(z));
                        let dy = (g.floor_y_of(x, z) - p.y).abs();
                        let dh = ((wx - p.x).powi(2) + (wz - p.z).powi(2)).sqrt();
                        assert!(
                            dy >= STOREY - 0.25 || dh > 6.0,
                            "seed {seed} L{level}: a walk at ({x},{z}) {dh:.1} m from a perch at {p:?}, {dy:.1} m under it"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_figure_keeps_far_and_turns_its_back_when_looked_at() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        stand_where_it_is_seen(&mut g);
        let f = g.far_figure.clone().unwrap();
        let d = level_dist(f.at, g.player.pos);
        assert!((NEAR..FAR).contains(&d), "L{level}: it stands {d:.0} m off");
        assert!(f.body.visible());
        // not looked at, it stays, turned toward you
        face(&mut g, f.at, false);
        step(&mut g, 10.0);
        assert_eq!(g.far_figure.as_ref().unwrap().state, FigureState::Standing, "L{level}: unlooked at, it waits");
        // looked at, it turns its back
        face(&mut g, f.at, true);
        step(&mut g, 2.5);
        let f2 = g.far_figure.clone().unwrap();
        assert_eq!(f2.state, FigureState::Leaving, "L{level}: looked at, it turns away");
        // your asides were told: looked at until it turned (weighed or owed)
        let k = "uncanny:figure";
        assert!(g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k), "L{level}: {:?}", g.asides.memory);
        assert!(f2.body.visible(), "L{level}: watched, it does not vanish");
        let toward = (g.player.pos.x - f.at.x).atan2(g.player.pos.z - f.at.z);
        let off = ((f2.body.rotation().y - toward).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI).abs();
        assert!(off < 0.2, "L{level}: its back to you ({off:.2} rad off)");
        // you look away: it is gone, for a long while
        face(&mut g, f.at, false);
        step(&mut g, 0.2);
        let f3 = g.far_figure.clone().unwrap();
        assert_eq!(f3.state, FigureState::Gone);
        assert!(!f3.body.visible());
        assert!(f3.t >= AWAY.0 - 0.3 && f3.t <= AWAY.1, "L{level}: gone for {:.0} s", f3.t);
        assert_eq!(g.state, "playing");
    }
}

#[test]
fn the_figure_never_lets_you_near() {
    for level in 1..=3 {
        let mut g = floor(77, level);
        stand_where_it_is_seen(&mut g);
        let at = g.far_figure.as_ref().unwrap().at;
        // straight at it, through anything, at a run, looking at it
        let mut nearest_seen = f64::INFINITY;
        for _ in 0..900 {
            let p = g.player.pos;
            let d = level_dist(at, p);
            if d < 1.0 {
                break;
            }
            let k = (4.0 * DT / d).min(1.0);
            g.player.pos = Vec3::new(p.x + (at.x - p.x) * k, p.y, p.z + (at.z - p.z) * k);
            face(&mut g, at, true);
            step(&mut g, DT);
            if g.far_figure.as_ref().unwrap().body.visible() {
                nearest_seen = nearest_seen.min(level_dist(at, g.player.pos));
            }
        }
        assert!(nearest_seen >= TOO_NEAR * 0.6 - 0.5, "L{level}: it let you within {nearest_seen:.1} m");
        assert_eq!(g.far_figure.as_ref().unwrap().state, FigureState::Gone);
    }
}

#[test]
fn the_figure_does_not_stand_while_the_fog_comes() {
    let mut g = floor(4242, 1);
    g.god_mode = true;
    let c = g.building.as_ref().unwrap().cells[0].tile;
    let (wx, wz) = (g.wx(c.0), g.wz(c.1));
    g.player.pos = Vec3::new(wx, g.floor_y_at(wx, wz) + game::base::config::EYE, wz);
    g.turn_cell_wheel(0, true);
    g.assembly.t = 0.5;
    step(&mut g, 2.0);
    assert_ne!(g.assembly.stage, game::world::assembly::Stage::Quiet);
    g.far_figure.as_mut().unwrap().t = 0.05;
    step(&mut g, 4.0);
    assert_eq!(g.far_figure.as_ref().unwrap().state, FigureState::Gone, "it waits out the fog");
}

/// Every part of the figure within this of its middle, level (it fits behind
/// the Well's parapet, 0.32 m out from where it stands, and on a gantry's
/// 0.7 m)
const REACH: f64 = 0.3;

/// Its body (G7): a person, MakeHuman's man in a long coat (`bodies.bin`
/// "far_figure"), a little taller than anyone, facing +Z (its rule turns +Z
/// toward you), its soles on the floor and every part of it near its
/// middle; standing, it stands on its perch's floor, lit by the place's
/// light (not glowing)
#[test]
fn the_figure_is_a_person_standing_on_its_perch() {
    use engine::material::MatKind;
    let geo = game::world::far_figures::far_figure_geometry().expect("bodies.bin: no far_figure");
    let (mut lo, mut hi, mut reach) = (f64::MAX, f64::MIN, 0.0f64);
    let (mut toe, mut heel) = (f64::MIN, f64::MAX);
    let tris = geo.r().index.as_ref().map_or(0, |i| i.len() / 3);
    for v in geo.r().position.chunks_exact(3) {
        let (x, y, z) = (v[0] as f64, v[1] as f64, v[2] as f64);
        lo = lo.min(y);
        hi = hi.max(y);
        reach = reach.max((x * x + z * z).sqrt());
        if y < 0.05 {
            toe = toe.max(z);
            heel = heel.min(z);
        }
    }
    println!("the figure: {tris} triangles, its soles at {lo:.3} m, {hi:.2} m tall, within {reach:.2} m of its middle, toes {toe:.2}, heels {heel:.2}");
    assert!(tris <= 18_000, "{tris} triangles (a body's budget is 18k)");
    assert!((-0.005..0.02).contains(&lo), "its soles {lo} m off the floor");
    assert!((1.85..2.0).contains(&hi), "{hi:.2} m tall: a little taller than anyone");
    assert!(reach < REACH, "a part of it {reach:.2} m from its middle");
    assert!(toe > 0.12 && toe > -heel + 0.04, "it faces +Z: its toes at {toe:.2}, its heels at {heel:.2}");
    for level in 1..=3 {
        let mut g = floor(4242, level);
        stand_where_it_is_seen(&mut g);
        let f = g.far_figure.clone().unwrap();
        let mut bb = engine::math::Box3::empty();
        bb.set_from_object(&f.body);
        assert!((bb.min.y - f.at.y).abs() < 0.02, "L{level}: its soles at {:.2}, its perch's floor at {:.2}", bb.min.y, f.at.y);
        assert!((bb.max.y - bb.min.y - hi).abs() < 0.02, "L{level}: drawn at its own height");
        let mut meshes = 0;
        f.body.traverse(&mut |o| {
            if !o.is_mesh() {
                return;
            }
            meshes += 1;
            for m in o.mats() {
                let m = m.r();
                assert!(matches!(m.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard), "L{level}: lit by the place");
                assert!(m.emissive_intensity * m.emissive.r.max(m.emissive.g).max(m.emissive.b) < 0.01, "L{level}: it glows");
            }
        });
        assert_eq!(meshes, 1, "L{level}: one piece");
    }
}

/// A figure standing when the siren starts turns its back and goes
#[test]
fn the_figure_goes_when_the_siren_starts() {
    let mut g = floor(7, 2);
    stand_where_it_is_seen(&mut g);
    let at = g.far_figure.as_ref().unwrap().at;
    face(&mut g, at, false);
    g.assembly.t = 0.05;
    step(&mut g, 1.0);
    assert_ne!(g.assembly.stage, game::world::assembly::Stage::Quiet, "the siren");
    step(&mut g, 2.0);
    assert_eq!(g.far_figure.as_ref().unwrap().state, FigureState::Gone, "it went at the siren");
}

/// From where you arrive (the arrival's tile and the tiles round it), a
/// lamp of the door's tower is in sight on most floors, with every solid of
/// the built floor (cell fronts too) in the way
#[test]
fn the_doors_light_is_seen_from_the_arrival() {
    let (mut seen, mut n) = (0, 0);
    let mut log = vec![];
    for level in 1..=3 {
        for seed in [4242u32, 7, 77, 1999] {
            let g = floor(seed, level);
            let b = g.building.clone().unwrap();
            let s = g.solids.as_ref().unwrap();
            let (ex, ez) = (b.exit.0 as f64 + 0.5, b.exit.1 as f64 + 0.5);
            let high = g.ground.iter().filter(|t| t.walkable()).map(|t| t.h0.max(t.h1) as f64).fold(f64::NEG_INFINITY, f64::max);
            let lights = door_lights(g.floor_y_of(b.exit.0, b.exit.1), g.floor_y_of(b.spawn.0, b.spawn.1).min(high));
            let mut ok = false;
            for dz in -2..=2 {
                for dx in -2..=2 {
                    let (x, z) = (b.spawn.0 + dx, b.spawn.1 + dz);
                    if x < 0 || z < 0 || x >= g.w || z >= g.h || !g.ground[g.t_idx(x, z)].walkable() {
                        continue;
                    }
                    let y = g.floor_y_of(x, z) + game::base::config::EYE;
                    ok |= lights.iter().any(|&ly| segment_clear_3d(s, x as f64 + 0.5, z as f64 + 0.5, y, ex, ez, ly));
                }
            }
            n += 1;
            seen += ok as usize;
            log.push(format!("s{seed} L{level} {}", if ok { "seen" } else { "-" }));
        }
    }
    println!("{}", log.join(", "));
    assert!(seen * 100 >= n * 70, "the door's light seen from the arrival on {seen} of {n} floors");
}

/// The Well's lift cage hangs in open void (on most floors), and the Works'
/// last door is lined with the rooms' yellow and opens on the Lobby
#[test]
fn the_lift_and_the_last_door() {
    let mut lifts = 0;
    for seed in [4242u32, 7, 77, 1999] {
        let g = floor(seed, 1);
        if let Some(at) = g.lift_at {
            lifts += 1;
            let (u, v) = (g.to_u(at.x).floor() as i32, g.to_v(at.z).floor() as i32);
            for z in v - 1..=v + 1 {
                for x in u - 1..=u + 1 {
                    assert_eq!(g.ground[g.t_idx(x, z)].kind, GROUND_VOID, "seed {seed}: the cage over ({x},{z})");
                }
            }
        }
    }
    assert!(lifts >= 3, "a lift cage on {lifts} of 4 Well floors");
    for seed in [4242u32, 7] {
        let mut g = floor(seed, 3);
        assert!(g.world.get_object_by_name("lobby-paper").is_some(), "seed {seed}: the yellow round the Works' door");
        g.next_level();
        g.now_ms += 1000.0;
        g.run_scheduled();
        assert_eq!(g.level_num, 4);
        assert!(g.building.is_none(), "through it: the rooms");
        assert_eq!(g.depth(), 1, "the first floor of the rooms, the Lobby");
        assert!(g.world.get_object_by_name("lobby-paper").is_none());
    }
}
