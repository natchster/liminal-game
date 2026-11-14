//! `place_first_aid_kits`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// First aid kit: 20% chance per level, 1 wall-mounted unit.
pub(crate) fn place_first_aid_kits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let white_mat = lambert!({ color: 0xf0e8d8 });
    let red_mat = lambert!({ color: 0xa02010 });
    let trim_mat = lambert!({ color: 0x7a6a54 });
    let kit = group();
    let body = mesh(pg("FirstAidKits__body", box_geo!(0.20, 0.18, 0.10)), &white_mat);
    kit.add(&body);
    // Red cross on front (2 overlapping thin rectangles).
    let cross_v = mesh(plane_geo!(0.04, 0.12), &red_mat);
    cross_v.position().set(0, 0, 0.051);
    kit.add(&cross_v);
    let cross_h = mesh(plane_geo!(0.12, 0.04), &red_mat);
    cross_h.position().set(0, 0, 0.051);
    kit.add(&cross_h);
    let trim = mesh(plane_geo!(0.20, 0.02), &trim_mat);
    trim.position().set(0, 0.08, 0.051); // top strip of the front face (body top y 0.09)
    kit.add(&trim);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.05); // body back (z −0.05) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.05);
    let kit_height = 1.5 + rnd() * 0.3; // 1.5-1.8 m (chest-to-eye)
    if c.dx.abs() == 1 {
        kit.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        kit.position().set(wall_x, kit_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        kit.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        kit.position().set(wall_x + (rnd() - 0.5) * 1.2, kit_height, wall_z);
    }
    g.world.add(&kit);
    let kp = kit.pos();
    let k_rot_y = kit.rot().y;
    // 20% chance small orange pill bottle on floor near the kit.
    if rnd() < 0.20 {
        let bottle_mat = lambert!({ color: 0xd8621a });
        let cap_mat = lambert!({ color: 0xf0f0e8 });
        let bottle = group();
        let body = mesh(pg("FirstAidKits__bottle", cyl_geo!(0.02, 0.02, 0.06, 8)), &bottle_mat);
        body.position().y = 0.03;
        bottle.add(&body);
        let cap = mesh(pg("FirstAidKits__cap", cyl_geo!(0.022, 0.022, 0.015, 8)), &cap_mat);
        cap.position().y = 0.068;
        bottle.add(&cap);
        let is_tipped_bottle = rnd() < 0.35;
        if is_tipped_bottle {
            bottle.rotation().z = PI / 2.0;
            bottle.position().y = 0.022;
        }
        let (fx, fz) = (k_rot_y.dsin(), k_rot_y.dcos()); // into the room (kit front +Z)
        let bdist = 0.30 + rnd() * 0.40;
        let by = bottle.pos().y;
        bottle.position().set(kp.x + fx * bdist + (rnd() - 0.5) * 0.4, by, kp.z + fz * bdist + (rnd() - 0.5) * 0.4);
        bottle.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bottle);
        // Tipped bottles 60% chance spill 3-5 small white pills nearby.
        if is_tipped_bottle && rnd() < 0.60 {
            let pill_mat = lambert!({ color: 0xf0eee4 });
            let pill_count = 3 + (rnd() * 3.0).floor() as i32;
            let bp = bottle.pos();
            for _p in 0..pill_count {
                let pill = mesh(pg("FirstAidKits__pill", cyl_geo!(0.006, 0.006, 0.004, 6)), &pill_mat);
                let p_ang = rnd() * PI * 2.0;
                let p_rad = 0.03 + rnd() * 0.10;
                pill.position().set(bp.x + p_ang.dcos() * p_rad, 0.003, bp.z + p_ang.dsin() * p_rad);
                g.world.add(&pill);
            }
        }
    }
    // 8% chance small syringe on floor near the kit.
    if rnd() < 0.08 {
        let barrel_mat = lambert!({ color: 0xd8d0c8 });
        let needle_mat = lambert!({ color: 0x808080 });
        let syringe = group();
        let barrel = mesh(pg("FirstAidKits__barrel", cyl_geo!(0.007, 0.007, 0.06, 6)), &barrel_mat);
        barrel.rotation().z = PI / 2.0;
        syringe.add(&barrel);
        let needle = mesh(pg("FirstAidKits__needle", cyl_geo!(0.001, 0.001, 0.04, 4)), &needle_mat);
        needle.rotation().z = PI / 2.0;
        needle.position().x = 0.05;
        syringe.add(&needle);
        let plunger = mesh(pg("FirstAidKits__plunger", cyl_geo!(0.008, 0.008, 0.008, 6)), &barrel_mat);
        plunger.rotation().z = PI / 2.0;
        plunger.position().x = -0.034;
        syringe.add(&plunger);
        let (fx, fz) = (k_rot_y.dsin(), k_rot_y.dcos()); // into the room (kit front +Z)
        let sdist = 0.20 + rnd() * 0.30;
        syringe.position().set(kp.x + fx * sdist + (rnd() - 0.5) * 0.3, 0.008, kp.z + fz * sdist + (rnd() - 0.5) * 0.3);
        syringe.rotation().y = rnd() * PI * 2.0;
        g.world.add(&syringe);
    }
    // 25% chance a clipboard is dropped on the floor near the kit.
    if rnd() < 0.25 {
        let board_mat = lambert!({ color: 0x9a7048 });
        let paper_mat = lambert!({ color: 0xece0d0 });
        let clip_mat = lambert!({ color: 0x606060 });
        let clipboard = group();
        let board = mesh(pg("Clipboards__board", box_geo!(0.20, 0.008, 0.28)), &board_mat);
        board.position().y = 0.004;
        clipboard.add(&board);
        let paper = mesh(pg("Clipboards__paper", plane_geo!(0.18, 0.24)), &paper_mat);
        paper.rotation().x = -PI / 2.0;
        paper.position().y = 0.009;
        clipboard.add(&paper);
        let clip = mesh(pg("Clipboards__clip", box_geo!(0.06, 0.012, 0.02)), &clip_mat);
        clip.position().set(0, 0.014, -0.11);
        clipboard.add(&clip);
        let (fx, fz) = (k_rot_y.dsin(), k_rot_y.dcos()); // into the room (kit front +Z)
        let dist = 0.3 + rnd() * 0.4;
        clipboard.position().set(kp.x + fx * dist + (rnd() - 0.5) * 0.4, 0, kp.z + fz * dist + (rnd() - 0.5) * 0.4);
        clipboard.rotation().y = rnd() * PI * 2.0;
        g.world.add(&clipboard);
    }
    // 25% chance safety goggles dropped near the kit.
    if rnd() < 0.25 {
        let strap_mat = lambert!({ color: 0x181818 });
        // Per-weather lens tint.
        let lens_color = goggle_lens_by_weather(g.weather_key());
        let lens_mat = lambert!({ color: lens_color, transparent: true, opacity: 0.55 });
        let goggles = group();
        for off in [-0.028, 0.028] {
            let lens = mesh(circle_geo!(0.024, 12), &lens_mat);
            lens.position().set(off, 0.005, 0);
            lens.rotation().x = -PI / 2.0;
            goggles.add(&lens);
            // lie flat round the flat lens (torus is built upright in XY)
            let rim = mesh(pg("FirstAidKits__rim", torus_geo!(0.024, 0.004, 4, 12)), &strap_mat);
            rim.rotation().x = -PI / 2.0;
            rim.position().set(off, 0.005, 0);
            goggles.add(&rim);
        }
        let bridge = mesh(pg("FirstAidKits__bridge", box_geo!(0.012, 0.005, 0.008)), &strap_mat);
        bridge.position().set(0, 0.005, 0);
        goggles.add(&bridge);
        let strap = mesh(pg("FirstAidKits__strap", cyl_geo!(0.002, 0.002, 0.14, 4)), &strap_mat);
        strap.rotation().z = PI / 2.0;
        strap.position().set(0, 0.006, -0.030); // behind the rims (outer z −0.028)
        goggles.add(&strap);
        // kit front is local +Z: (sin, cos) of its yaw points into the room
        let (gfx, gfz) = (k_rot_y.dsin(), k_rot_y.dcos());
        let g_dist = 0.35 + rnd() * 0.35;
        goggles.position().set(kp.x + gfx * g_dist + (rnd() - 0.5) * 0.4, 0, kp.z + gfz * g_dist + (rnd() - 0.5) * 0.4);
        goggles.rotation().y = rnd() * PI * 2.0;
        g.world.add(&goggles);
    }
}

/// Goggle lens per weather
fn goggle_lens_by_weather(key: &str) -> u32 {
    by_weather(key, 0x6890a8, 0xd8c880, 0xa87070, 0x907050, 0x8ac0d8, 0x8ac0d8)
}
