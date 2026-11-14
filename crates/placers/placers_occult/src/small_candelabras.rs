//! `place_small_candelabras`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage candelabra: 4% chance per level, 3 lit candles.
pub(crate) fn place_small_candelabras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let wax_mat = lambert!({ color: 0xf0e8d8 });
    let wick_mat = lambert!({ color: 0x2a2418 });
    let cand = group();
    // Base foot (wide disc).
    let base = mesh(pg("SmallCandelabras__base", cyl_geo!(0.045, 0.055, 0.014, 12)), &brass_mat);
    base.position().y = 0.007;
    cand.add(&base);
    // Central column.
    let col = mesh(pg("SmallCandelabras__column", cyl_geo!(0.010, 0.010, 0.14, 6)), &brass_mat);
    col.position().y = 0.084;
    cand.add(&col);
    // Bulges on column.
    for bulg_y in [0.040, 0.080, 0.120] {
        let bulge = mesh(pg("SmallCandelabras__knop", sphere_geo!(0.018, 8, 6)), &brass_mat);
        bulge.scale().set(1, 0.5, 1);
        bulge.position().y = bulg_y;
        cand.add(&bulge);
    }
    // 3 arms extending sideways + central up (the centre cup sits on the
    // column's top, 0.154).
    let arm_positions: [(f64, f64); 3] = [(-0.06, 0.130), (0.06, 0.130), (0.0, 0.157)];
    for &(ax, ay) in arm_positions.iter() {
        // Arm cylinder (thin diagonal from column to holder).
        if ax.abs() > 0.001 {
            let arm_len = hypot2(ax, ay - 0.120);
            let arm = mesh(pg("SmallCandelabras__arm", cyl_geo!(0.004, 0.004, arm_len, 4)), &brass_mat);
            arm.position().set(ax / 2.0, (0.120 + ay) / 2.0, 0);
            // Turn the arm's Y axis onto the column-to-cup direction.
            arm.rotation().z = (ay - 0.120).datan2(ax) - PI / 2.0;
            cand.add(&arm);
        }
        // Holder cup at top of arm.
        let cup = mesh(pg("SmallCandelabras__cup", cyl_geo!(0.012, 0.010, 0.006, 8)), &brass_mat);
        cup.position().set(ax, ay, 0);
        cand.add(&cup);
        // Candle: white wax cylinder.
        let candle_len = 0.05;
        let candle = mesh(pg("SmallCandelabras__candle", cyl_geo!(0.006, 0.006, candle_len, 6)), &wax_mat);
        candle.position().set(ax, ay + candle_len / 2.0 + 0.003, 0);
        cand.add(&candle);
        // Wick.
        let wick = mesh(cyl_geo!(0.001, 0.001, 0.006, 3), &wick_mat);
        wick.position().set(ax, ay + candle_len + 0.006, 0);
        cand.add(&wick);
        // Flame + point light.
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
        let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
        flame.position().set(ax, ay + candle_len + 0.012, 0);
        cand.add(&flame);
        let glow = point_light!(0xffb060, 0.3, 0.8, 2);
        glow.position().set(ax, ay + candle_len + 0.014, 0);
        cand.add(&glow);
        // Warm halo sphere per candle.
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.018, 6, 5), &halo_mat);
        halo.position().set(ax, ay + candle_len + 0.014, 0);
        cand.add(&halo);
        // Flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 6.0 + rnd() * 4.0;
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.dsin() * 0.5 + (p * 1.7).dsin() * 0.3;
            glow.set_intensity(0.3 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
            halo.scale().set_scalar(1.0 + wobble * 0.15);
        });
    }
    cand.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    cand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cand);
}
