//! Wear by use is the thing's own (`game::visuals::prop_look`,
//! `docs/PROPS.md` P4; code review 2): how much a prop is handled and
//! chipped, and the floor its low bands are measured from, are set on its
//! meshes (`Node::use_wear`), never kept on a material, since materials are
//! shared between props and some across levels (`SharedMats`: the exit
//! door's frame and the CRT's stand, the CRT's body and the payphone). So a
//! level looks the same whatever was built before it: level 5 reached from
//! level 4 wears as a fresh start at level 5 does.
//!
//!   cargo test --release -p backrooms --test prop_wear
mod common;

use engine::prelude::*;
use game::Game;

/// A New Game started at `first` with run seed `seed`, then each of `then`
/// in turn, as the stairs down bring them
fn run(seed: u32, first: i32, then: &[i32]) -> Game {
    let mut g = Game::new_bare();
    common::register_placers(&mut g.placers);
    g.launch_flags = format!("nosound&seed={seed}&level={first}");
    g.boot();
    g.start_new_game();
    assert_eq!((g.run_seed, g.level_num), (seed, first));
    for &l in then {
        g.load_level(l);
    }
    g
}

/// What the shader's wear by use reads, mesh by mesh (each instance of an
/// instanced one): where it is, and for each of its materials that take the
/// wear (`wear` > 0) the wear, whether it takes wear by use, how much the
/// thing is handled and chipped, and the height of its floor. Sorted.
fn wear_inputs(g: &Game) -> Vec<String> {
    let mut out = vec![];
    g.world.update_matrix_world(true);
    g.world.traverse(&mut |o: &Obj| {
        if !o.is_mesh() || !o.visible() {
            return;
        }
        let mats: Vec<(f64, f64)> = o.mats().iter().map(|m| (m.r().wear, m.r().wear_use)).filter(|(w, _)| *w > 0.0).collect();
        if mats.is_empty() {
            return;
        }
        let mw = o.matrix_world();
        let n = if o.is_instanced() { o.instance_count() } else { 1 };
        for i in 0..n {
            let (m, w) = if o.is_instanced() { (Mat4::mul(&mw, &o.get_matrix_at(i)), o.use_wear_at(i)) } else { (mw, o.use_wear()) };
            for &(wear, takes) in &mats {
                out.push(format!(
                    "{:.3} {:.3} {:.3} | wear {wear:.2} use {takes:.2} handled {:.2} chips {:.2} floor {:.3}",
                    m.e[12], m.e[13], m.e[14], w.handled, w.chips, w.floor
                ));
            }
        }
    });
    out.sort();
    out
}

/// Each of these levels wears the same built fresh as reached from the
/// level before. Before wear was the thing's own, each shared material
/// kept how much its first settled prop was handled, so these differed:
/// run 77's level 5 after level 4 (a small thing on the floor worn as
/// furniture), its level 6 after level 5 (a hung thing worn as furniture),
/// run 3's level 5 after level 4 (furniture worn as a hung thing).
#[test]
fn a_levels_wear_does_not_depend_on_the_level_before() {
    let mut failures = vec![];
    for (seed, before, level) in [(77, 4, 5), (77, 5, 6), (3, 4, 5)] {
        let fresh = wear_inputs(&run(seed, level, &[]));
        let after = wear_inputs(&run(seed, before, &[level]));
        assert!(fresh.len() > 500, "run {seed} L{level}: the props take the wear ({} inputs)", fresh.len());
        let only_fresh: Vec<&String> = fresh.iter().filter(|s| after.binary_search(s).is_err()).collect();
        let only_after: Vec<&String> = after.iter().filter(|s| fresh.binary_search(s).is_err()).collect();
        if !only_fresh.is_empty() || !only_after.is_empty() {
            failures.push(format!(
                "run {seed} L{level} after L{before}: {} inputs only fresh, {} only after; first:\n    fresh: {:?}\n    after: {:?}",
                only_fresh.len(),
                only_after.len(),
                only_fresh.iter().take(3).collect::<Vec<_>>(),
                only_after.iter().take(3).collect::<Vec<_>>()
            ));
        }
    }
    assert!(failures.is_empty(), "a level wears differently after the level before:\n  {}", failures.join("\n  "));
}

/// A thing's low wear (the floor's grime at its foot, the mop's tide line,
/// the scuffs) is measured from the floor under it: on the Building's
/// floors, whose storeys stand 3 m apart, a prop on another storey than the
/// ground's has its own storey's floor; on the rooms the floor is at 0.
#[test]
fn a_things_low_wear_is_measured_from_its_own_floor() {
    for level in [1, 2, 3, 5] {
        let g = run(4242, level, &[]);
        let (mut worn, mut off_ground) = (0, 0);
        g.world.update_matrix_world(true);
        g.world.traverse(&mut |o: &Obj| {
            if !o.is_mesh() || o.mats().iter().all(|m| m.r().wear_use == 0.0) {
                return;
            }
            let n = if o.is_instanced() { o.instance_count() } else { 1 };
            for i in 0..n {
                let (m, w) = if o.is_instanced() {
                    (Mat4::mul(&o.matrix_world(), &o.get_matrix_at(i)), o.use_wear_at(i))
                } else {
                    (o.matrix_world(), o.use_wear())
                };
                let under = g.floor_y_at(m.e[12], m.e[14]);
                assert!(
                    (w.floor - under).abs() < 0.3,
                    "L{level}: a thing at {:.2}, {:.2} worn from {:.2}, its floor at {under:.2}",
                    m.e[12],
                    m.e[14],
                    w.floor
                );
                worn += 1;
                if w.floor.abs() > 1.0 {
                    off_ground += 1;
                }
            }
        });
        println!("seed 4242 L{level}: {worn} worn meshes, {off_ground} on another storey than the ground's");
        assert!(worn > 0, "L{level}: something takes the wear");
        if level <= 3 {
            assert!(off_ground > 0, "L{level}: a worn thing on another storey");
        } else {
            assert_eq!(off_ground, 0, "L{level}: the rooms' floor is at 0");
        }
    }
}
