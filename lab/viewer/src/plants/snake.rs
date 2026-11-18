//! The snake plant (*Dracaena trifasciata*): a rosette of stiff strap
//! leaves from a rhizome, banded dark and grey-green, some with a yellow
//! margin. Kept alive by almost nothing; left long enough, the outer leaves
//! go first: they lean out, fold over at a soft base and lie across the rim,
//! their tips brown and papery, and the rot works up from the soil.
use super::kit::*;
use super::*;

fn strap(u: f64, w: f64) -> f64 {
    w * (0.45 + 0.55 * smooth(0.0, 0.25, u)) * (1.0 - u * u * u).max(0.0).sqrt()
}

pub fn grow(seed: u64, health: f64) -> Plant {
    let mut r = Rng::new(seed ^ 0x534e_414b);
    let pot = Pot { terracotta: r.chance(0.35), r_top: r.range(0.13, 0.16), r_bottom: r.range(0.10, 0.12), height: r.range(0.22, 0.27) };
    let mut parts = pot.build(seed, health);
    let soil_y = pot.soil_y();
    let laurentii = r.chance(0.5);
    let tall = r.range(0.8, 1.15);
    // the rhizome's offsets: two or three clumps
    let clumps: Vec<V> = (0..r.range(2.0, 3.99) as usize)
        .map(|_| {
            let a = r.range(0.0, 2.0 * PI);
            let d = r.range(0.0, pot.soil_r() * 0.45);
            V(a.dcos() * d, soil_y, a.dsin() * d)
        })
        .collect();
    let n = r.range(15.0, 23.0) as usize;
    let lean_side = r.range(0.0, 2.0 * PI);
    let mut leaves = MeshBuf::default();
    let mut litter = MeshBuf::default();
    for k in 0..n {
        let mut lr = r.fork(k as u64);
        let c = clumps[k % clumps.len()];
        // older leaves are outer and die first
        let age = lr.f();
        let lh = (health + lr.normal() * 0.12 - age * (1.0 - health) * 0.6).clamp(0.0, 1.0);
        let az = lr.range(0.0, 2.0 * PI);
        let out = V(az.dcos(), 0.0, az.dsin());
        // upright when tended; leaning out, then down, as the base softens
        let lean = lr.range(0.04, 0.22) + age * 0.15 + (1.0 - lh) * lr.range(0.2, 0.9);
        let dir = UP.rot(UP.cross(out).norm(), lean);
        let length = tall * lr.range(0.42, 0.82) * (0.75 + 0.25 * (1.0 - age));
        let w = lr.range(0.03, 0.042);
        let base = c + out * lr.range(0.0, 0.02) - V(0.0, 0.02, 0.0);
        let face = out.rot(UP, lr.range(-0.6, 0.6));
        // a collapsed leaf's own stream decides whether it is still on the plant
        let collapsed = lh < 0.4 && lr.chance(0.75 - lh);
        let dropped = lh < 0.15 && lr.chance(0.4);
        let mut b = Blade::new(base, dir, face, length, w, strap);
        b.segs_u = 30;
        b.segs_v = 2;
        b.droop = 0.25 + (1.0 - lh) * 1.6;
        b.fold = lr.range(0.3, 0.55) + (1.0 - lh) * 0.5;
        b.twist = lr.normal() * 0.5;
        b.wave = (0.0015, lr.range(1.0, 2.5));
        if collapsed {
            b.kink = Some((lr.range(0.08, 0.3), lr.range(1.1, 1.9)));
        }
        let s_band = lr.next_u64();
        let kink_u = b.kink.map(|(u, _)| u);
        let tip_burn = (1.0 - lh) * 0.75 + lr.range(0.0, 0.04);
        let colour = move |u: f64, v: f64| -> [f64; 3] {
            let a = v.abs();
            let dark = [0.15, 0.24, 0.12];
            let light = [0.36, 0.44, 0.30];
            // the bands: zig-zags across the leaf, uneven
            let x = u * length * 24.0 + a * 1.6 + noise1(s_band, u * 6.0) * 0.9;
            let band = 0.5 + 0.5 * (2.0 * PI * x).dsin();
            let mut c = mix(dark, light, smooth(0.55, 0.92, band) * 0.75);
            if laurentii {
                c = mix(c, [0.76, 0.68, 0.30], smooth(0.78, 0.9, a));
            }
            // pale where it comes out of the soil
            c = mix(c, [0.58, 0.55, 0.40], smooth(0.05, 0.0, u));
            c = mix(c, aged(c, lh), (1.0 - lh) * 0.55);
            // the tip dries first: a yellow line, then papery brown
            let edge = 1.0 - tip_burn + noise1(s_band ^ 5, a * 4.0) * 0.04;
            c = mix(c, [0.62, 0.56, 0.24], smooth(edge - 0.1, edge, u) * 0.8);
            c = mix(c, [0.52, 0.42, 0.27], smooth(edge, edge + 0.04, u));
            // a soft, rotting base where it folded
            if let Some(ku) = kink_u {
                c = mix(c, [0.27, 0.25, 0.12], smooth(0.08, 0.0, (u - ku).abs()));
            }
            c
        };
        if dropped {
            // broken off and lying on the floor beside the pot
            let at = scatter(&mut lr, pot.r_top + 0.05, pot.r_top + 0.45, lean_side);
            let dry = 1.0 - lh;
            litter.extend(&fallen_leaf(&mut lr, at, 0.0, length * 0.9, w, strap, aged([0.3, 0.36, 0.2], lh * 0.5), dry));
        } else {
            b.build(&mut leaves, &colour);
        }
    }
    parts.push((Surface::Leaf, leaves));
    parts.push((Surface::Leaf, litter));
    Plant { name: format!("snake plant {seed} at {health:.2}"), parts }
}
