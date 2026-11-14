//! `place_window_signs`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Window sign: 8% chance per level, 1 OPEN/CLOSED placard on a wall.
pub(crate) fn place_window_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = window_sign_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    let sign_mat = lambert!({ map: tex, transparent: true, side: DoubleSide });
    let sign = mesh(plane_geo!(0.22, 0.11), &sign_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let sign_height = 1.7 + rnd() * 0.15;
    if c.dx.abs() == 1 {
        sign.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        sign.position().set(wall_x, sign_height, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        sign.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        sign.position().set(wall_x + (rnd() - 0.5) * 1.0, sign_height, wall_z);
    }
    // Slight ±0.05 rad Z tilt for crooked-hang aesthetic.
    sign.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&sign);
    // 8% chance: cracked-glass overlay atop the sign.
    if rnd() < 0.08 {
        let crack_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let crack = mesh(plane_geo!(0.22, 0.11), &crack_mat);
        crack.position().copy(&sign.pos());
        crack.rotation().copy(&sign.rot());
        // Nudge outward 2 mm from wall so overlay renders in front.
        crack.position().x -= c.dx as f64 * 0.002;
        crack.position().z -= c.dz as f64 * 0.002;
        g.world.add(&crack);
        // 2-4 tiny floor shards beneath cracked sign.
        let shard_mat = basic_mat!({ color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let sh_count = 2 + (rnd() * 3.0).floor() as i32;
        let sp = sign.pos();
        for _sh in 0..sh_count {
            let shard = mesh(plane_geo!(0.012 + rnd() * 0.020, 0.016 + rnd() * 0.020), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.08 + rnd() * 0.25;
            shard.position().set(sp.x + sh_ang.dcos() * sh_rad, 0.003, sp.z + sh_ang.dsin() * sh_rad);
            g.world.add(&shard);
        }
    }
}
