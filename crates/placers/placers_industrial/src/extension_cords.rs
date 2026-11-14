//! `place_extension_cords`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Extension cord: 20% chance per level, 1 coiled cord on floor.
pub(crate) fn place_extension_cords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
      // 30% chance: near a tracked socket; 25%: near a tracked laptop.
    let (mut x, mut z) = (0i32, 0i32);
    let mut near_socket = false;
    let mut socket_pos: Option<Vec3> = None;
    let mut near_laptop = false;
    let mut laptop_pos: Option<Vec3> = None;
    let roll = rnd();
    if !g.pp.socket_positions.is_empty() && roll < 0.30 {
        let sp = g.pp.socket_positions[(rnd() * g.pp.socket_positions.len() as f64).floor() as usize];
        socket_pos = Some(sp);
        x = g.tx(sp.x);
        z = g.tz(sp.z);
        if !g.in_grid(x, z) || g.tiles[g.t_idx(x, z)] != 0 {
            socket_pos = None;
        } else {
            near_socket = true;
        }
    } else if !g.pp.laptop_positions.is_empty() && roll < 0.55 {
        let lp = g.pp.laptop_positions[(rnd() * g.pp.laptop_positions.len() as f64).floor() as usize];
        laptop_pos = Some(lp);
        x = g.tx(lp.x);
        z = g.tz(lp.z);
        if !g.in_grid(x, z) || g.tiles[g.t_idx(x, z)] != 0 {
            laptop_pos = None;
        } else {
            near_laptop = true;
        }
    }
    if !near_socket && !near_laptop {
        let open = open_tiles(g);
        if open.is_empty() {
            return;
        }
        let p = pick(&open);
        x = p.0;
        z = p.1;
    }
    // Per-weather cord palette biases the color pool.
    let cord_pool = extension_cord_by_weather(g.weather_key());
    let cord_mat = lambert!({ color: pick(cord_pool) });
    let plug_mat = lambert!({ color: 0x080604 });
    let cord = group();
    // 3 short cylinder segments zig-zagging.
    let seg_len = 0.6;
    let (mut cur_x, mut cur_z, mut cur_ang) = (0.0f64, 0.0f64, rnd() * PI * 2.0);
    for _s in 0..3 {
        let seg = mesh(pg("ExtensionCords__seg", cyl_geo!(0.012, 0.012, seg_len, 5)), &cord_mat);
        seg.rotation().z = PI / 2.0;
        seg.rotation().y = -cur_ang; // axis along (cos a, 0, sin a), the way the path advances
        seg.position().set(cur_x + cur_ang.cos() * seg_len * 0.5, 0.012, cur_z + cur_ang.sin() * seg_len * 0.5);
        cord.add(&seg);
        // Advance path with angular jitter.
        cur_x += cur_ang.cos() * seg_len;
        cur_z += cur_ang.sin() * seg_len;
        cur_ang += (rnd() - 0.5) * 1.4;
    }
    // Small plug head at cord end, turned to the cord heading, boot (-x) on the cord end.
    let plug = mesh(pg("ExtensionCords__plug", box_geo!(0.05, 0.03, 0.035)), &plug_mat);
    plug.rotation().y = -cur_ang;
    plug.position().set(cur_x + cur_ang.cos() * 0.025, 0.015, cur_z + cur_ang.sin() * 0.025);
    cord.add(&plug);
    // 2 tiny prongs sticking out from plug.
    for yoff in [-0.008, 0.008] {
        let prong = mesh(pg("ExtensionCords__prong", box_geo!(0.02, 0.003, 0.003)), &plug_mat);
        prong.rotation().y = -cur_ang;
        prong.position().set(cur_x + cur_ang.cos() * 0.059, 0.015 + yoff, cur_z + cur_ang.sin() * 0.059);
        cord.add(&prong);
    }
    if let (true, Some(sp)) = (near_socket, socket_pos) {
        cord.position().set(sp.x + (rnd() - 0.5) * 0.4, 0, sp.z + (rnd() - 0.5) * 0.4);
    } else if let (true, Some(lp)) = (near_laptop, laptop_pos) {
        cord.position().set(lp.x + (rnd() - 0.5) * 0.5, 0, lp.z + (rnd() - 0.5) * 0.5);
    } else {
        cord.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    }
    cord.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cord);
}

/// Extension cord pool per weather; other keys get `[0x181410, 0x201818, 0x202020, 0x141014]`
fn extension_cord_by_weather(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x0a0f18, 0x101820],
        "electric" => &[0x201808, 0x282010],
        "haunted" => &[0x1a0808, 0x201014],
        "derelict" => &[0x181410, 0x201818],
        // still / normal have no entry → the baseline
        _ => &[0x181410, 0x201818, 0x202020, 0x141014],
    }
}
