//! Partitions that lean (rooms R4, `world::angled`), over levels of the
//! rooms: each leaning run stays inside its own wall tiles, its free end no
//! further than `DEV_MAX` off its line; its staircase lies inside the wall
//! as drawn and closes it to a look all along; the turn the walls are
//! drawn with is the turn the staircase follows; and no prop stands inside
//! one.
mod common;
use engine::dmath::DMath;
use game::base::config::CELL;
use game::level::archgen::{BOX_ANGLED, segment_clear};
use game::world::angled::{DEV_MAX, Wall};

/// The rooms' levels looked at: (run seed, level)
fn levels() -> Vec<(u32, i32)> {
    let mut out = vec![];
    for k in 0..6u32 {
        for level in [4, 5, 6, 8] {
            out.push((k.wrapping_mul(2654435761) ^ 0x1EA2, level));
        }
    }
    out
}

/// The across coordinate of a point, and its along one
fn al_ac(w: &Wall, u: f64, v: f64) -> (f64, f64) {
    if w.along_x { (u, v) } else { (v, u) }
}

#[test]
fn leaning_walls_stay_in_their_tiles_and_close_to_a_look() {
    let mut g = common::new_game();
    let (mut walls, mut levels_with, mut degrees) = (0, 0, vec![]);
    for (seed, level) in levels() {
        g.run_seed = seed;
        g.init_level(level);
        let s = g.solids.as_ref().expect("solids");
        if !g.angled.is_empty() {
            levels_with += 1;
        }
        for w in &g.angled {
            walls += 1;
            degrees.push(w.theta.to_degrees().abs());
            let what = format!("seed {seed} L{level} wall along {} at {:.3}", if w.along_x { "x" } else { "z" }, w.line);
            // the turn the walls are drawn with: the pivot stays put, a point on the line stays on it
            let (pu, pv) = if w.along_x { (w.pivot, w.line) } else { (w.line, w.pivot) };
            let (qu, qv) = w.turn(pu, pv);
            assert!((qu - pu).abs() < 1e-9 && (qv - pv).abs() < 1e-9, "{what}: the pivot moves");
            for &(t, p) in &w.parts {
                for (u, v) in [(p[0], p[1]), (p[2], p[1]), (p[0], p[3]), (p[2], p[3])] {
                    let (nu, nv) = w.turn(u, v);
                    // inside the run's own tiles (turning draws a part a little toward
                    // the pivot, into the next of them; and its end a hair into the
                    // wall it ends against: as far as the turn would open a sliver
                    // there, and its corner turns past that)
                    let m = 2.0 * w.half * w.theta.dtan().abs() + 0.004;
                    let near = |q: usize| {
                        let (qx, qz) = ((q as i32 % g.w) as f64, (q as i32 / g.w) as f64);
                        nu > qx - m && nu < qx + 1.0 + m && nv > qz - m && nv < qz + 1.0 + m
                    };
                    assert!(
                        w.parts.iter().any(|&(q, _)| near(q)),
                        "{what}: a corner of tile {t}'s part leaves the run's tiles: {p:?} ({u:.4}, {v:.4}) → ({nu:.4}, {nv:.4}); pivot {:.4} lean {:.2}°",
                        w.pivot,
                        w.theta.to_degrees()
                    );
                    let (_, ac0) = al_ac(w, u, v);
                    let (_, ac1) = al_ac(w, nu, nv);
                    assert!(
                        ((ac1 - ac0).abs() - w.half * w.theta.dsin().abs()) * CELL <= DEV_MAX + 1e-6,
                        "{what}: an end goes off its line"
                    );
                }
            }
            // closed to a look, all along: across it from one side's room to the
            // other's (off the boxes' shared edges, where a look exactly along
            // one passes between any two boxes, straight walls' too)
            let (a0, a1) = w.parts.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &(_, p)| {
                let (b0, b1) = (al_ac(w, p[0], p[1]).0, al_ac(w, p[2], p[3]).0);
                (lo.min(b0), hi.max(b1))
            });
            let n = ((a1 - a0) * 40.0) as usize;
            for k in 0..n {
                let al = w.pivot + (a0 + (a1 - a0) * (k as f64 + 0.37) / n as f64 - w.pivot) * w.theta.dcos();
                let (c0, c1) = (w.face(al, -1.0) - 0.2, w.face(al, 1.0) + 0.2);
                let ((u0, v0), (u1, v1)) = if w.along_x { ((al, c0), (al, c1)) } else { ((c0, al), (c1, al)) };
                assert!(!segment_clear(s, u0, v0, u1, v1, 1.2), "{what}: a look passes through it at {al:.3}");
            }
        }
        // nothing stands inside one as drawn: a prop's origin (each instance's,
        // where props were merged) is never within its faces. (A probe along a
        // run that leans once slid a lamp 3 m along the wall's side, into it.)
        let mut at: Vec<(f64, f64)> = vec![];
        let mut all = vec![];
        g.world.traverse(&mut |o: &engine::scene::Obj| all.push(o.clone()));
        for o in all {
            if o.r().user.arch || o.child_count() > 0 || o.is_light() {
                continue;
            }
            if o.is_instanced() {
                at.extend((0..o.instance_count()).map(|i| {
                    let e = o.get_matrix_at(i).e;
                    (e[12], e[14])
                }));
            } else {
                let p = o.world_position();
                at.push((p.x, p.z));
            }
        }
        for &(x, z) in &at {
            let (u, v) = (g.to_u(x), g.to_v(z));
            for w in &g.angled {
                let (al, ac) = al_ac(w, u, v);
                let on = w.parts.iter().any(|&(_, p)| al >= al_ac(w, p[0], p[1]).0 && al <= al_ac(w, p[2], p[3]).0);
                let (lo, hi) = (w.face(al, -1.0), w.face(al, 1.0));
                assert!(
                    !(on && ac > lo + 0.002 && ac < hi - 0.002),
                    "seed {seed} L{level}: a prop at ({x:.2}, {z:.2}) stands inside a leaning wall (along {} at {:.3})",
                    if w.along_x { "x" } else { "z" },
                    w.line
                );
            }
        }
        // every step inside the wall it stands in for
        for b in 0..s.count {
            let bx = s.box_at(b);
            if bx[7] as i32 != BOX_ANGLED {
                continue;
            }
            let w = g.angled.iter().find(|w| w.owns(bx[6] as usize)).expect("a step of no wall");
            for (u, v) in [(bx[0], bx[1]), (bx[2], bx[1]), (bx[0], bx[3]), (bx[2], bx[3])] {
                let (al, ac) = al_ac(w, u as f64, v as f64);
                let (lo, hi) = (w.face(al, -1.0), w.face(al, 1.0));
                assert!(
                    ac >= lo - 1e-5 && ac <= hi + 1e-5,
                    "seed {seed} L{level}: a step stands out of its wall ({ac:.4} not in {lo:.4}..{hi:.4})"
                );
            }
        }
    }
    degrees.sort_by(f64::total_cmp);
    let n = levels().len();
    println!(
        "{walls} leaning walls on {levels_with} of {n} levels; lean {:.1}°..{:.1}°, median {:.1}°",
        degrees.first().unwrap_or(&0.0),
        degrees.last().unwrap_or(&0.0),
        degrees.get(degrees.len() / 2).unwrap_or(&0.0)
    );
    assert!(levels_with * 2 >= n, "walls lean on only {levels_with} of {n} levels of the rooms");
}
