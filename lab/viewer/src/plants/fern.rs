//! The Boston fern (*Nephrolepis exaltata*): a crown of arching fronds, each
//! a rachis carrying alternate pinnae that shorten toward both ends. The
//! office fern dies of dry air: the old outer fronds first, the pinnae
//! browning from the rachis out and dropping, until what is left is a heap
//! of bare brown rachises over the rim and a ring of pinnae on the floor.
use super::kit::*;
use super::*;

fn rachis(u: f64, w: f64) -> f64 {
    w * (1.0 - u * 0.7)
}

fn pinna(u: f64, w: f64) -> f64 {
    // oblong, a little eared at the base, blunt at the tip
    w * (0.55 + 0.45 * smooth(0.0, 0.2, u)) * (1.0 - smooth(0.7, 1.0, u) * 0.8)
}

pub fn grow(seed: u64, health: f64) -> Plant {
    let mut r = Rng::new(seed ^ 0x4645_524e);
    let pot = Pot { terracotta: r.chance(0.5), r_top: r.range(0.13, 0.155), r_bottom: r.range(0.095, 0.11), height: r.range(0.18, 0.22) };
    let mut parts = pot.build(seed, health);
    let crown = V(0.0, pot.soil_y() + 0.01, 0.0);
    let n = r.range(26.0, 38.0) as usize;
    let size = r.range(0.85, 1.15);
    let lean_side = r.range(0.0, 2.0 * PI);
    let golden = PI * (3.0 - 5.0f64.sqrt());
    let (mut green, mut stems, mut litter) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
    for k in 0..n {
        let mut fr = r.fork(k as u64);
        // young fronds stand in the middle, the old arch outside
        let age = k as f64 / n as f64;
        let fh = (health + fr.normal() * 0.1 - age * (1.0 - health) * 0.7).clamp(0.0, 1.0);
        let az = k as f64 * golden + fr.normal() * 0.2;
        let out = V(az.dcos(), 0.0, az.dsin());
        let elev = (1.4 - age * 0.9 + fr.normal() * 0.08) - (1.0 - fh) * 0.35;
        let dir = (out * elev.dcos() + UP * elev.dsin()).norm();
        let length = size * fr.range(0.38, 0.6) * (0.7 + 0.5 * age);
        let base = crown + out * fr.range(0.0, 0.03);
        let face = UP.rot(out, fr.normal() * 0.3);
        let mut rib = Blade::new(base, dir, face, length, 0.0022, rachis);
        rib.segs_u = 18;
        rib.droop = 4.5 + age * 3.5 + (1.0 - fh) * 3.5;
        rib.twist = fr.normal() * 0.4;
        let bare = fh < 0.2;
        let rib_col = if bare { [0.40, 0.31, 0.19] } else { mix([0.30, 0.38, 0.16], [0.45, 0.36, 0.20], 1.0 - fh) };
        rib.build(&mut stems, &|_, _| rib_col);
        let points = rib.midrib();
        let pairs = fr.range(26.0, 38.0) as usize;
        let lp = size * fr.range(0.034, 0.048);
        for i in 0..pairs * 2 {
            let mut pr = fr.fork(i as u64);
            let u = 0.06 + 0.92 * (i as f64 / (pairs * 2) as f64);
            // shorter at both ends of the frond
            let len = lp * (PI * (0.12 + 0.88 * u)).dsin().dpowf(0.6);
            // a pinna browns from the rachis out and from the old fronds first
            let ph = (fh + pr.normal() * 0.12 - (1.0 - u) * (1.0 - fh) * 0.3).clamp(0.0, 1.0);
            let gone = pr.chance((1.0 - ph).dpowf(1.6) * 0.85);
            let at = (u * (points.len() - 1) as f64) as usize;
            let (p, d, side) = points[at.min(points.len() - 1)];
            let s = if i % 2 == 0 { 1.0 } else { -1.0 };
            let dir_p = d.rot(d.cross(side).norm(), s * pr.range(1.05, 1.35));
            let young = [0.42, 0.56, 0.22];
            let mature = [0.27, 0.43, 0.15];
            let living = mix(mature, young, smooth(0.6, 1.0, u) * (1.0 - age));
            let c = mix(living, aged(living, ph), (1.0 - ph) * 0.9);
            if gone {
                if pr.chance(0.35) {
                    let spot = scatter(&mut pr, 0.05, pot.r_top + 0.4, lean_side);
                    let y = if spot.len() < pot.soil_r() { pot.soil_y() } else { 0.0 };
                    litter.extend(&fallen_leaf(&mut pr, spot, y, len, len * 0.22, pinna, aged(living, ph * 0.4), 1.0));
                }
                continue;
            }
            let mut b = Blade::new(p, dir_p, d.cross(side).norm(), len, len * 0.24, pinna);
            b.segs_u = 3;
            b.segs_v = 1;
            b.droop = 1.2 + (1.0 - ph) * 3.0;
            b.fold = 0.15 + (1.0 - ph) * 0.4;
            b.curl = len * 0.08 * (1.0 - ph);
            let s_c = pr.next_u64();
            b.build(&mut green, &|uu, v| {
                let n = noise1(s_c, uu * 3.0 + v) * 0.04;
                [c[0] + n, c[1] + n, c[2] + n * 0.5]
            });
        }
    }
    parts.push((Surface::Leaf, stems));
    parts.push((Surface::Leaf, green));
    parts.push((Surface::Leaf, litter));
    Plant { name: format!("Boston fern {seed} at {health:.2}"), parts }
}
