//! `place_tall_floor_lamps`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Tall floor lamp w/ shade + optional PointLight (floor): 3% chance per level.
pub(crate) fn place_tall_floor_lamps(g: &mut Game, _dist: &[i32]) {
    g.pp.tall_lamp_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let stand_colors = [0x504030, 0x202020, 0xc0a060, 0x8b5a2b, 0x606060];
    let stand_mat = lambert!({ color: choice(&stand_colors) });
    let shade_colors = [0xf0e0c0, 0xd0a080, 0xf0e0a0, 0xc0d0e0, 0xe0c0e0, 0xf0f0e0];
    let shade_mat = lambert!({ color: choice(&shade_colors), transparent: true, opacity: 0.85 });
    let bulb_mat = basic_mat!({ color: 0xf0e0a0 });
    let fl = group();
    // Base — wide flat disc.
    let base = mesh(pg("TallFloorLamps__base", cyl_geo!(0.10, 0.11, 0.014, 12)), &stand_mat);
    base.position().y = 0.007;
    fl.add(&base);
    // Stand column — tall thin cylinder.
    let stand_h = 1.30;
    let stand = mesh(pg("TallFloorLamps__stand", cyl_geo!(0.010, 0.012, stand_h, 6)), &stand_mat);
    stand.position().y = 0.014 + stand_h / 2.0;
    fl.add(&stand);
    // Optional decorative middle band (torus on stand).
    let band = mesh(pg("TallFloorLamps__band", torus_geo!(0.014, 0.003, 3, 8)), &stand_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.014 + stand_h * 0.4;
    fl.add(&band);
    // Top harp — U-shape supporting shade (small ring).
    let harp_mat = lambert!({ color: 0x505050 });
    let harp_ring = mesh(pg("TallFloorLamps__harp", torus_geo!(0.018, 0.002, 3, 8)), &harp_mat);
    harp_ring.rotation().x = PI / 2.0;
    harp_ring.position().y = 0.014 + stand_h + 0.010;
    fl.add(&harp_ring);
    // Shade — wide tapered cylinder (top narrower).
    let (shade_top, shade_bottom, shade_h) = (0.14, 0.20, 0.24);
    let shade = mesh(cyl_geo!(shade_top, shade_bottom, shade_h, 12, 1, true), &shade_mat);
    shade.position().y = 0.014 + stand_h + shade_h / 2.0 + 0.030;
    fl.add(&shade);
    // Small bulb hint (self-lit sphere inside shade).
    let bulb = mesh(sphere_geo!(0.030, 8, 6), &bulb_mat);
    bulb.position().y = 0.014 + stand_h + shade_h / 2.0 + 0.020;
    fl.add(&bulb);
    // 55% chance: lamp is on — PointLight illumination + optional flicker.
    let lamp_is_on = rnd() < 0.55;
    if lamp_is_on {
        let light = point_light!(0xf0d090, 0.60, 3.5, 2);
        light.position().copy(&bulb.pos());
        fl.add(&light);
        // 40% chance: subtle flicker (dying bulb).
        if rnd() < 0.4 {
            let flicker_phase = rnd() * PI * 2.0;
            let light2 = light.clone();
            let bulb2 = bulb.clone();
            g.animate(move |_g, _dt, t| {
                let f = 0.85 + (t * 6.0 + flicker_phase).dsin() * 0.10 + (t * 11.0 + flicker_phase * 1.4).dsin() * 0.05;
                light2.set_intensity(0.60 * f);
                bulb2.scale().set(f, f, f);
            });
        }
    } else {
        // Bulb un-lit — swap to dim material.
        bulb.set_mat(lambert!({ color: 0x606050 }));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    fl.position().set(px, 0, pz);
    fl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fl);
    // Register on-lamp positions for filament buzz audio.
    if lamp_is_on {
        let p = fl.pos();
        g.pp.tall_lamp_positions.push(v3(p.x, 0, p.z));
    }
}
