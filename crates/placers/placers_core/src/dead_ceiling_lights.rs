//! `place_dead_ceiling_lights`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Dead ceiling light fixture: 20% chance per level, 1-2 units.
// Small dark disc + tiny protruding bulb-remnant on ceiling.
pub(crate) fn place_dead_ceiling_lights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let disc_mat = lambert!({ color: 0x1a1810 });
    let bulb_mat = lambert!({ color: 0x2a2418 });
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        let light = group();
        // Base plate: small disc.
        let plate = mesh(pg("DeadCeilingLights__canopy", circle_geo!(0.11, 12)), &disc_mat);
        plate.rotation().x = PI / 2.0;
        plate.position().y = WALL_H - 0.0054; // the canopy's 5.4 mm back reaches the ceiling
        light.add(&plate);
        // Small "socket" cylinder protruding down.
        let socket = mesh(pg("DeadCeilingLights__socket", cyl_geo!(0.028, 0.028, 0.03, 6)), &bulb_mat);
        socket.position().y = WALL_H - 0.0204;
        light.add(&socket);
        // Tiny darker "broken bulb neck" sticking further down.
        if rnd() < 0.55 {
            let bulb = mesh(pg("DeadCeilingLights__bulb", cyl_geo!(0.015, 0.020, 0.025, 5)), &bulb_mat);
            bulb.position().y = WALL_H - 0.0454;
            light.add(&bulb);
        }
        light.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
        g.world.add(&light);
        // 30% chance: small draped cobweb over the dead-light socket.
        if rnd() < 0.30 {
            let dw_size = 0.12 + rnd() * 0.06;
            let dw_geo = tri_geo([0.0, 0.0, 0.0, dw_size, 0.0, 0.0, 0.0, -dw_size * 0.7, 0.0], [0, 1, 2]);
            let dw_color = cobweb_color_by_weather(g.weather_key());
            let dw_mat = basic_mat!({ color: dw_color, transparent: true, opacity: 0.35, depthWrite: false, side: DoubleSide });
            let dw = mesh(dw_geo, &dw_mat);
            // Position adjacent to the light socket, dangling.
            dw.rotation().y = rnd() * PI * 2.0;
            let lp = light.pos();
            dw.position().set(lp.x + (rnd() - 0.5) * 0.06, WALL_H - 0.03, lp.z + (rnd() - 0.5) * 0.06);
            g.world.add(&dw);
        }
        // 25% chance: matching hanging cable co-located 0.35-0.60 m
        // from the dead light — "wire ripped from fixture" narrative.
        if rnd() < 0.25 {
            let cable_col_mat = lambert!({ color: 0x181410 });
            let plug_col_mat = lambert!({ color: 0x080604 });
            let cable_col = group();
            let cable_len = 0.4 + rnd() * 0.5;
            let shaft = mesh(pg("CeilingCables__cable", cyl_geo!(0.005, 0.005, cable_len, 5)), &cable_col_mat);
            let c_tilt = (rnd() - 0.5) * 0.15;
            shaft.rotation().z = c_tilt;
            // tilted about its centre: top end 3 mm into the ceiling
            shaft.position().y = WALL_H + 0.003 - (cable_len / 2.0) * c_tilt.cos();
            cable_col.add(&shaft);
            let plug = mesh(pg("CeilingCables__plug", box_geo!(0.028, 0.038, 0.024)), &plug_col_mat);
            // on the tilted cable's lower end, the cable entering 4 mm into the plug's top
            plug.position().set(c_tilt.sin() * cable_len / 2.0, WALL_H + 0.003 - cable_len * c_tilt.cos() - 0.015, 0);
            cable_col.add(&plug);
            let c_ang = rnd() * PI * 2.0;
            let c_rad = 0.35 + rnd() * 0.25;
            let lp = light.pos();
            cable_col.position().set(lp.x + c_ang.cos() * c_rad, 0, lp.z + c_ang.sin() * c_rad);
            g.world.add(&cable_col);
        }
        i += 1;
    }
}

/// A one-triangle geometry with an index and vertex normals.
fn tri_geo(v: [f64; 9], idx: [u32; 3]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}
