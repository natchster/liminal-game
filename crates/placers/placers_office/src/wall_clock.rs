//! `place_wall_clock`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Stopped wall clock: 20% chance per level — circular wall prop with a
// frozen cryptic time (1.9-2.4 m).
pub(crate) fn place_wall_clock(g: &mut Game, _dist: &[i32]) {
    g.pp.wall_clock_positions.clear(); // reset per level
    g.pp.cracked_clock_positions.clear(); // reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let geo = circle_geo!(0.20, 24);
    // Pick one of 6 clock face variants; the cracked variant is the last.
    let clock_textures = clock_textures();
    let clock_tex_idx = (rnd() * clock_textures.len() as f64).floor() as usize;
    let clock_tex = clock_textures[clock_tex_idx].clone();
    let is_cracked = clock_tex_idx == clock_textures.len() - 1;
    let mat = lambert!({ map: clock_tex, transparent: true, side: DoubleSide });
    let clock = mesh(geo, &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let clock_height = 1.9 + rnd() * 0.5;
    if c.dx.abs() == 1 {
        clock.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        clock.position().set(wall_x, clock_height, wall_z + (rnd() - 0.5) * 1.4);
    } else {
        clock.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        clock.position().set(wall_x + (rnd() - 0.5) * 1.4, clock_height, wall_z);
    }
    // Slight ±0.03 rad Z tilt for crooked-hang aesthetic.
    clock.rotation().z = (rnd() - 0.5) * 0.06;
    g.world.add(&clock);
    // Track for tick audio events.
    let cp = clock.pos();
    g.pp.wall_clock_positions.push(v3(cp.x, clock_height, cp.z));
    // Cracked variant sways slightly (0.10 Hz, ±0.015 rad).
    if is_cracked {
        let base_sway_z = clock.rot().z;
        let sway_phase = rnd() * PI * 2.0;
        let clock2 = clock.clone();
        g.animate(move |_g, _dt, t| {
            clock2.rotation().z = base_sway_z + (t * 0.2 * PI + sway_phase).sin() * 0.015;
        });
        // Track cracked clock position for periodic creak audio.
        g.pp.cracked_clock_positions.push(v3(cp.x, clock_height, cp.z));
    }
    // Cracked-clock floor shards.
    if is_cracked {
        let shard_mat = basic_mat!({
            color: shard_color_by_weather(g.weather_key()), transparent: true, opacity: 0.8,
            depthWrite: false, side: DoubleSide,
        });
        let sh_count = 2 + (rnd() * 3.0).floor() as i32;
        for sh in 0..sh_count {
            let sw = 0.012 + rnd() * 0.020;
            let shh = 0.016 + rnd() * 0.022;
            let shard = mesh(plane_geo!(sw, shh), &shard_mat);
            shard.rotation().x = -PI / 2.0;
            shard.rotation().z = rnd() * PI * 2.0;
            let sh_ang = rnd() * PI * 2.0;
            let sh_rad = 0.10 + rnd() * 0.30;
            shard.position().set(cp.x + sh_ang.cos() * sh_rad, 0.003, cp.z + sh_ang.sin() * sh_rad);
            g.world.add(&shard);
        }
    }
}
