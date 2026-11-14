//! `place_crystal_chandeliers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_on_floor;

// Crystal chandelier: 3% chance per level. 6 arms + dangles + inner light.
pub(crate) fn place_crystal_chandeliers(g: &mut Game, _dist: &[i32]) {
    g.pp.chandelier_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let brass_mat = lambert!({ color: 0xc89040 });
    let crystal_mat = basic_mat!({ color: 0xf0f0ff, transparent: true, opacity: 0.55 });
    let chain_mat = lambert!({ color: 0x8a7050 });
    let chandelier = group();
    let chain_len = 0.35 + rnd() * 0.15;
    let center_y = WALL_H - chain_len - 0.05;
    // Hanging chain, from the ceiling down into the cap.
    let chain = mesh(pg("CrystalChandeliers__chain", cyl_geo!(0.003, 0.003, chain_len, 4)), &chain_mat);
    chain.position().y = WALL_H - chain_len / 2.0;
    chandelier.add(&chain);
    // Top brass cap.
    let top_cap = mesh(pg("CrystalChandeliers__cap", cyl_geo!(0.030, 0.020, 0.010, 8)), &brass_mat);
    top_cap.position().y = center_y + 0.055;
    chandelier.add(&top_cap);
    // Central body sphere.
    let body = mesh(pg("CrystalChandeliers__body", sphere_geo!(0.030, 12, 10)), &brass_mat);
    body.position().y = center_y + 0.020;
    chandelier.add(&body);
    // 6 arms extending outward.
    let n_arms = 6;
    for i in 0..n_arms {
        let a = (i as f64 / n_arms as f64) * PI * 2.0;
        let arm_x = a.cos() * 0.09;
        let arm_z = a.sin() * 0.09;
        // Arm cylinder; tip socket under the cup's foot.
        let arm = mesh(pg("CrystalChandeliers__arm", cyl_geo!(0.003, 0.003, 0.10, 4)), &brass_mat);
        arm.position().set(arm_x / 2.0, center_y + 0.013, arm_z / 2.0);
        arm.rotation().z = -(0.0f64).atan2(0.09);
        // Just orient toward outward direction.
        let arm_dir = v3(arm_x, 0.005, arm_z).normalized();
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&v3(0, 1, 0), &arm_dir);
        arm.quaternion().copy(&q);
        chandelier.add(&arm);
        // Small candle cup at end of arm.
        let cup = mesh(pg("CrystalChandeliers__cup", cyl_geo!(0.008, 0.006, 0.006, 6)), &brass_mat);
        cup.position().set(arm_x, center_y + 0.020, arm_z);
        chandelier.add(&cup);
        // Small candle.
        let candle = mesh(pg("CrystalChandeliers__candle", cyl_geo!(0.005, 0.005, 0.024, 5)), lambert!({ color: 0xf0e8d8 }));
        candle.position().set(arm_x, center_y + 0.034, arm_z);
        chandelier.add(&candle);
        // Small flame + halo (all lit).
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.9 });
        let flame = mesh(cone_geo!(0.003, 0.008, 5), &flame_mat);
        flame.position().set(arm_x, center_y + 0.050, arm_z);
        chandelier.add(&flame);
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.016, 6, 5), &halo_mat);
        halo.position().set(arm_x, center_y + 0.050, arm_z);
        chandelier.add(&halo);
    }
    // 8 crystal dangles below body.
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        let dr = 0.07;
        let dangle = mesh(cone_geo!(0.005, 0.030, 4), &crystal_mat);
        dangle.position().set(a.cos() * dr, center_y - 0.030, a.sin() * dr);
        dangle.rotation().x = PI;
        chandelier.add(&dangle);
    }
    // Single central light.
    let glow = point_light!(0xffb060, 0.9, 2.0, 2);
    glow.position().y = center_y + 0.020;
    chandelier.add(&glow);
    let flicker_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = t * 5.0 + flicker_phase;
        glow.set_intensity(0.9 + p.sin() * 0.06 + (p * 1.7).sin() * 0.04);
    });
    scatter_on_floor(g, &chandelier, x, z, 1.5);
    // Track for chime audio.
    let p = chandelier.pos();
    g.pp.chandelier_positions.push(v3(p.x, center_y, p.z));
    // Slow sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 12.0 + rnd() * 6.0;
        // swing about the chain's ceiling point (local 0, WALL_H, 0), not the floor
        // under it: shift the group by the pivot's displacement each frame
        let mut sway_off = [0.0f64; 3];
        let ch = chandelier.clone();
        g.animate(move |_g, _dt, t| {
            let rz = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.015;
            ch.rotation().z = rz;
            let s = rz.sin() * WALL_H;
            let ry = ch.rot().y;
            let o = [s * ry.cos(), (1.0 - rz.cos()) * WALL_H, -s * ry.sin()];
            {
                let mut pp = ch.position();
                pp.x += o[0] - sway_off[0];
                pp.y += o[1] - sway_off[1];
                pp.z += o[2] - sway_off[2];
            }
            sway_off = o;
        });
    }
    g.world.add(&chandelier);
}
