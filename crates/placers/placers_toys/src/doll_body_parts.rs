//! `place_doll_body_parts`
use crate::shared::vinyl_tex;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::place::open_tiles;
use placers_kit::place::salt;
use placers_kit::place::scatter;

// A doll's arms and legs: 4% chance per level. Two or three of a vinyl
// baby doll's limbs (family `rubber_duck_baby_doll_and_hole`'s doll: an arm bent at the elbow, its hand's
// fingers moulded together; a leg bent at the knee, its foot with toes),
// come off its cloth body: each limb's top a groove the cloth was tied in
// and a lip round its open end, the hollow dark inside. Lying where they
// fell, apart.
pub(crate) fn place_doll_body_parts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let k = salt(g.wx(x), g.wz(z), 499);
    // one doll's: the same vinyl, yellowed alike
    let vinyl = lambert!({ map: &vinyl_tex(k, 0xd3ae97) });
    let dark = lambert!({ color: 0x1c1716 });
    let grp = group();
    let n_parts = 2 + (rnd() * 2.0).floor() as i32;
    // each limb lying so far: its middle, its turn and whether an arm
    let mut placed: Vec<(f64, f64, f64, bool)> = vec![];
    for _ in 0..n_parts {
        let is_arm = rnd() < 0.5;
        let part = group();
        let limb = group();
        let (name, hole, half) = if is_arm {
            ("Real__dollLimbArm", "Real__dollLimbArmHole", 0.052)
        } else {
            ("Real__dollLimbLeg", "Real__dollLimbLegHole", 0.056)
        };
        put(&limb, name, &vinyl);
        put(&limb, hole, &dark);
        // turned about its middle, as the old limb was
        limb.position().x = -half;
        part.add(&limb);
        let (mut ox, mut oz) = ((rnd() - 0.5) * 0.30, (rnd() - 0.5) * 0.20);
        let yaw = rnd() * PI * 2.0;
        // (the old tilt's draw: a little more turn)
        let tilt = (rnd() - 0.5) * 0.4;
        // apart: a limb never lies through another nor touches it (no draw).
        // Each is a long, bent thing, so they are kept apart along their
        // lengths, not only at their middles (two limbs 105 mm apart at the
        // middle still crossed end over end: M8)
        let ang = yaw + tilt;
        for _ in 0..16 {
            let mut moved = false;
            for &(qx, qz, qa, q_arm) in &placed {
                let (a, b) = closest_between(&limb_axis(ox, oz, ang, is_arm), &limb_axis(qx, qz, qa, q_arm));
                let (dx, dz) = (a.0 - b.0, a.1 - b.1);
                let d = (dx * dx + dz * dz).sqrt();
                if d < LIMB_GAP {
                    let (ux, uz) = if d > 1e-6 { (dx, dz) } else { (ox - qx + 1e-3, oz - qz) };
                    let l = (ux * ux + uz * uz).sqrt().max(1e-6);
                    ox += ux / l * (LIMB_GAP - d + 0.002);
                    oz += uz / l * (LIMB_GAP - d + 0.002);
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        placed.push((ox, oz, ang, is_arm));
        part.position().set(ox, 0, oz);
        part.rotation().y = yaw + tilt;
        grp.add(&part);
    }
    scatter(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

/// The least room between two limbs' axes: their thickness (a fat thigh, a
/// foot or a hand 30-40 mm across) and a hand's breadth of floor between
/// (at 40 mm, seen low along them, they still read as one jointed limb)
const LIMB_GAP: f64 = 0.062;

/// A limb's axis on the floor (x, z), as `toys/soft_toys_and_dolls.py` bends it: the top, the
/// elbow or the knee, the wrist or the ankle, the hand's or the foot's end;
/// in its group (its middle at x, z, turned by `a`, the limb set back by
/// its half length as the placer sets it)
fn limb_axis(x: f64, z: f64, a: f64, arm: bool) -> [(f64, f64); 4] {
    let (half, pts) = if arm {
        (0.052, [(0.004, 0.0), (0.046, 0.006), (0.062, 0.033), (0.080, 0.064)])
    } else {
        (0.056, [(0.004, 0.0), (0.054, 0.008), (0.094, -0.010), (0.112, 0.026)])
    };
    let (s, c) = a.sin_cos();
    pts.map(|(px, pz)| {
        let px = px - half;
        (x + px * c + pz * s, z - px * s + pz * c)
    })
}

/// The closest points of two limbs' axes (polylines)
fn closest_between(p: &[(f64, f64); 4], q: &[(f64, f64); 4]) -> ((f64, f64), (f64, f64)) {
    let closest_on = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        let (ex, ez) = (b.0 - a.0, b.1 - a.1);
        let t = (((c.0 - a.0) * ex + (c.1 - a.1) * ez) / (ex * ex + ez * ez).max(1e-12)).clamp(0.0, 1.0);
        (a.0 + ex * t, a.1 + ez * t)
    };
    let d2 = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2);
    // two segments that cross meet at the crossing
    let cross = |a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)| {
        let den = (b.0 - a.0) * (d.1 - c.1) - (b.1 - a.1) * (d.0 - c.0);
        if den.abs() < 1e-12 {
            return None;
        }
        let t = ((c.0 - a.0) * (d.1 - c.1) - (c.1 - a.1) * (d.0 - c.0)) / den;
        let u = ((c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)) / den;
        ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t))
    };
    let mut best = (p[0], q[0]);
    for i in 0..3 {
        for j in 0..3 {
            let (p0, p1, q0, q1) = (p[i], p[i + 1], q[j], q[j + 1]);
            if let Some(x) = cross(p0, p1, q0, q1) {
                return (x, x);
            }
            for c in
                [(p0, closest_on(q0, q1, p0)), (p1, closest_on(q0, q1, p1)), (closest_on(p0, p1, q0), q0), (closest_on(p0, p1, q1), q1)]
            {
                if d2(c.0, c.1) < d2(best.0, best.1) {
                    best = c;
                }
            }
        }
    }
    best
}
