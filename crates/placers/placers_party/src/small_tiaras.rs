//! `place_small_tiaras`: Small tiara: 5% chance per level. Thin gold ring with 5 diamond spike accents. Party-costume family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small tiara: 5% chance per level. Thin gold ring with 5
/// diamond spike accents. Party-costume family.
pub(crate) fn place_small_tiaras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let gold_mat = lambert!({ color: 0xd8b040 });
    let gem_colors = [0xd82060, 0x2080d8, 0x30c060, 0xd050a0];
    let gem_mat = basic_mat!({ color: pick(&gem_colors), transparent: true, opacity: 0.9 });
    let tiara = group();
    // Base: half-ring (torus arc approximated by 8 short cylinders on semicircle).
    let radius = 0.055;
    let n_arcs = 8;
    for i in 0..n_arcs {
        let a0 = -PI / 2.0 + (i as f64 / n_arcs as f64) * PI;
        let a1 = -PI / 2.0 + ((i + 1) as f64 / n_arcs as f64) * PI;
        let (x0, z0) = (a0.cos() * radius, a0.sin() * radius);
        let (x1, z1) = (a1.cos() * radius, a1.sin() * radius);
        let seg_len = hypot2(x1 - x0, z1 - z0);
        let seg = mesh(pg("SmallTiaras__band", cyl_geo!(0.004, 0.004, seg_len, 5)), &gold_mat);
        seg.position().set((x0 + x1) / 2.0, 0.006, (z0 + z1) / 2.0);
        seg.rotation().x = PI / 2.0;
        // With Euler XYZ this puts the rod axis along the arc tangent.
        seg.rotation().z = -(x1 - x0).atan2(z1 - z0);
        tiara.add(&seg);
    }
    // 5 spike points along the crown top.
    for i in 0..5 {
        let a = -PI / 4.0 + (i as f64 / 4.0) * (PI / 2.0);
        let ax = a.cos() * radius;
        let az = a.sin() * radius;
        let spike = mesh(pg("SmallTiaras__spike", cone_geo!(0.008, 0.030, 4)), &gold_mat);
        // Standing up on the band.
        spike.position().set(ax, 0.022, az);
        tiara.add(&spike);
        // Small gem at spike base.
        if i == 2 {
            let gem = mesh(sphere_geo!(0.006, 6, 5), &gem_mat);
            gem.position().set(ax, 0.010, az);
            tiara.add(&gem);
        }
    }
    tiara.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tiara.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tiara);
}
