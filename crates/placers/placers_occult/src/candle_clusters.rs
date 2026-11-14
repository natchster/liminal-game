//! `place_candle_clusters`: Candle cluster: 8% chance per level. 3-5 birthday candles arranged in a small ring (cake without cake).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Candle cluster: 8% chance per level. 3-5
/// birthday candles arranged in a small ring (cake without cake).
pub(crate) fn place_candle_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wax_colors = [0xf0e8d8, 0xf0c0d0, 0xd8f0d0, 0xd0d8f0, 0xf0d8a0, 0xe0b0e0];
    let wick_mat = lambert!({ color: 0x2a2418 });
    let n_candles = 3 + (rnd() * 3.0).floor() as i32;
    let ring_radius = 0.04 + rnd() * 0.03;
    let cluster = group();
    for i in 0..n_candles {
        let angle = (i as f64 / n_candles as f64) * PI * 2.0 + rnd() * 0.3;
        let wax_mat = lambert!({ color: pick(&wax_colors) });
        let candle = group();
        let body = mesh(pg("CandleClusters__body", cyl_geo!(0.008, 0.008, 0.06, 8)), &wax_mat);
        body.position().y = 0.03;
        candle.add(&body);
        let wick = mesh(pg("CandleClusters__wick", cyl_geo!(0.001, 0.001, 0.008, 4)), &wick_mat);
        wick.position().y = 0.064;
        candle.add(&wick);
        // 70% lit for cluster (more festive than lone candle).
        if rnd() < 0.70 {
            let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
            let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
            flame.position().y = 0.074;
            candle.add(&flame);
            let flame_light = point_light!(0xffb060, 0.28, 0.7, 2);
            flame_light.position().y = 0.075;
            candle.add(&flame_light);
            // Warm halo sphere.
            let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
            let halo = mesh(sphere_geo!(0.018, 6, 5), &halo_mat);
            halo.position().y = 0.076;
            candle.add(&halo);
            let flicker_phase = rnd() * PI * 2.0;
            let flicker_rate = 6.0 + rnd() * 4.0;
            g.animate(move |_g, _dt, t| {
                let p = t * flicker_rate + flicker_phase;
                let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
                flame_light.set_intensity(0.28 + wobble * 0.07);
                flame.scale().y = 1.0 + wobble * 0.14;
                halo.scale().set_scalar(1.0 + wobble * 0.15);
            });
        }
        candle.position().set(angle.cos() * ring_radius, 0, angle.sin() * ring_radius);
        cluster.add(&candle);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    cluster.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cluster);
}
