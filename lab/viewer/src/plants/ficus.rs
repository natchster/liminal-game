//! The weeping fig (*Ficus benjamina*), the office's tree: three stems
//! braided as the nursery trained them, a crown of branches whose tips weep,
//! small glossy leaves with a drip tip in two rows along each twig. Its
//! answer to any stress is to drop its leaves, so the dying fig stands
//! thinned over a ring of yellow and brown leaves on the carpet, and the
//! dead one is bare twigs with a few curled leaves still hanging on.
use super::kit::*;
use super::*;

fn leaf(u: f64, w: f64) -> f64 {
    w * (PI * u.dpowf(0.8)).dsin().max(0.0).dpowf(0.75) * (1.0 - smooth(0.8, 1.0, u) * 0.55)
}

struct Twig {
    pts: Vec<V>,
    radii: Vec<f64>,
    depth: usize,
}

fn branch(r: &mut Rng, start: V, dir: V, length: f64, radius: f64, depth: usize, out: &mut Vec<Twig>) {
    let step = 0.045;
    let steps = ((length / step) as usize).max(2);
    let mut d = dir.norm();
    let mut p = start;
    let (mut pts, mut radii) = (vec![p], vec![radius]);
    let mut kids = vec![];
    let n_kids = if depth >= 4 { 0 } else { [4.0, 4.0, 3.0, 3.0][depth] as usize + (r.f() * 1.5) as usize };
    let golden = PI * (3.0 - 5.0f64.sqrt());
    for i in 0..steps {
        let u = (i + 1) as f64 / steps as f64;
        // up near the trunk, weeping toward the tips
        let up = 0.06 * (1.0 - depth as f64 * 0.3).max(0.0);
        let weep = 0.035 * depth as f64 * u;
        let wander = V(r.normal(), r.normal() * 0.5, r.normal()) * 0.07;
        d = (d + UP * (up - weep) + wander).norm();
        p += d * step;
        pts.push(p);
        radii.push(radius * (1.0 - 0.65 * u));
        if u > 0.3 && kids.len() < n_kids && r.chance(n_kids as f64 / (steps as f64 * 0.7)) {
            kids.push((p, d, u, kids.len()));
        }
    }
    for (p, d, u, k) in kids {
        let mut kr = r.fork(k as u64);
        let around = d.perp().rot(d, k as f64 * golden + kr.normal() * 0.3);
        let cd = d.rot(around, kr.range(0.45, 0.9));
        let cl = length * kr.range(0.5, 0.72) * (1.15 - u * 0.4);
        branch(&mut kr, p, cd, cl, radius * (1.0 - 0.65 * u) * 0.72, depth + 1, out);
    }
    out.push(Twig { pts, radii, depth });
}

pub fn grow(seed: u64, health: f64) -> Plant {
    let mut r = Rng::new(seed ^ 0x4649_4355);
    let pot = Pot { terracotta: r.chance(0.3), r_top: r.range(0.19, 0.22), r_bottom: r.range(0.15, 0.17), height: r.range(0.3, 0.34) };
    let mut parts = pot.build(seed, health);
    let soil = pot.soil_y();
    let trunk_h = r.range(0.55, 0.8);
    let lean_side = r.range(0.0, 2.0 * PI);
    let mut twigs = vec![];
    // three stems braided round each other
    for s in 0..3 {
        let mut sr = r.fork(s);
        let a0 = s as f64 * 2.0 * PI / 3.0;
        let turns = 1.6;
        let rad = 0.017;
        let n = 24;
        let (mut pts, mut radii) = (vec![], vec![]);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let a = a0 + t * turns * 2.0 * PI;
            pts.push(V(a.dcos() * rad * (1.0 - t * 0.3), soil - 0.02 + t * trunk_h, a.dsin() * rad * (1.0 - t * 0.3)));
            radii.push(0.012 * (1.0 - t * 0.25));
        }
        let top = *pts.last().unwrap();
        twigs.push(Twig { pts, radii, depth: 0 });
        let out = V(a0.dcos(), 0.0, a0.dsin());
        let dir = (UP * 2.2 + out).norm();
        let len = sr.range(0.55, 0.75);
        branch(&mut sr, top, dir, len, 0.009, 1, &mut twigs);
    }
    let (mut wood, mut green, mut litter) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
    let bark = [0.47, 0.42, 0.35];
    for (i, t) in twigs.iter().enumerate() {
        let sides = if t.depth <= 1 { 7 } else { 5 };
        let s = i as u64;
        tube(&mut wood, &t.pts, &t.radii, sides, true, &|u, a| {
            let n = noise1(s, u * 7.0 + a * 3.0) * 0.04;
            let c = mix(bark, [0.36, 0.40, 0.26], smooth(0.5, 1.0, u) * (t.depth as f64 / 3.0));
            [c[0] + n, c[1] + n, c[2] + n]
        });
    }
    // the leaves: two rows along each outer twig
    let mut count = 0u64;
    for t in twigs.iter().filter(|t| t.depth >= 2) {
        let n = t.pts.len();
        let mut side_sign = 1.0;
        // the outermost twigs carry two nodes a step, the inner ones one
        let per = if t.depth >= 3 { 2 } else { 1 };
        for k in per..n * per {
            let i = k / per;
            let u = k as f64 / ((n - 1) * per) as f64;
            if u < 0.25 || i >= n {
                continue;
            }
            count += 1;
            let mut lr = r.fork(count);
            let p = t.pts[i - 1].lerp(t.pts[i], ((k % per) + 1) as f64 / per as f64);
            let d = (t.pts[i] - t.pts[i - 1]).norm();
            side_sign = -side_sign;
            let side = (d.cross(UP).norm() * side_sign).rot(d, lr.normal() * 0.35);
            // inner leaves (old, shaded) go first
            let lh = (health + lr.normal() * 0.13 - (1.0 - u) * (1.0 - health) * 0.5).clamp(0.0, 1.0);
            let young = [0.33, 0.48, 0.15];
            let mature = [0.11, 0.23, 0.08];
            let living = mix(mature, young, smooth(0.8, 1.0, u) * lr.f());
            let length = lr.range(0.055, 0.085);
            let w = length * lr.range(0.22, 0.28);
            if lr.chance((1.0 - lh).dpowf(0.9) * 0.97) {
                // dropped: on the floor round the pot, most of them yellow
                if lr.chance(0.8) {
                    let at = scatter(&mut lr, pot.r_top + 0.03, pot.r_top + 0.6, lean_side);
                    let fall = aged(living, lr.range(0.0, 0.7) * lh + 0.15);
                    litter.extend(&fallen_leaf(&mut lr, at, 0.0, length, w, leaf, fall, 1.0 - lh));
                }
                continue;
            }
            let dir = (side * 0.75 + d * 0.45 + V(0.0, -0.55, 0.0)).norm();
            let face = (UP - dir * dir.dot(UP)).norm();
            let mut b = Blade::new(p + side * 0.006, dir, face, length, w, leaf);
            b.segs_u = 5;
            b.segs_v = 1;
            b.droop = 2.5 + (1.0 - lh) * 4.0;
            b.fold = 0.2 + (1.0 - lh) * 0.5;
            b.curl = w * 0.15 + (1.0 - lh) * w * 0.6;
            b.twist = lr.normal() * 0.3;
            let c = mix(living, aged(living, lh), (1.0 - lh) * 0.95);
            b.build(&mut green, &|uu, v| mix(c, [c[0] * 0.8, c[1] * 0.85, c[2] * 0.8], smooth(0.4, 0.0, v.abs()) * (1.0 - uu) * 0.5));
        }
    }
    parts.push((Surface::Wood, wood));
    parts.push((Surface::Leaf, green));
    parts.push((Surface::Leaf, litter));
    Plant { name: format!("weeping fig {seed} at {health:.2}"), parts }
}
