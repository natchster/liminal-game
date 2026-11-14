//! `place_victorian_hall_stands`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Victorian floor hall stand (floor): 2.5% chance per level.
pub(crate) fn place_victorian_hall_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // 4-color wood pool.
    let wood_colors: [u32; 4] = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let hs = group();
    // Base tray for umbrellas.
    let tray_r = 0.16;
    let tray = mesh(pg("VictorianHallStands__tray", cyl_geo!(tray_r, tray_r, 0.020, 14)), &wood_mat);
    tray.position().y = 0.010;
    hs.add(&tray);
    // Tray rim (raised).
    let tray_rim = mesh(pg("VictorianHallStands__rim", torus_geo!(tray_r, 0.006, 5, 14)), &wood_mat);
    tray_rim.rotation().x = PI / 2.0;
    tray_rim.position().y = 0.020;
    hs.add(&tray_rim);
    // Central vertical post — 180 cm tall.
    let post_h = 1.80;
    let post = mesh(pg("VictorianHallStands__post", cyl_geo!(0.020, 0.030, post_h, 8)), &wood_mat);
    post.position().y = 0.020 + post_h / 2.0;
    hs.add(&post);
    // Ornamental turned rings on post at 3 heights.
    for ry in [0.35, 0.65, 1.10] {
        let ring = mesh(pg("VictorianHallStands__ring", torus_geo!(0.024, 0.005, 5, 12)), &wood_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = 0.020 + ry;
        hs.add(&ring);
    }
    // 6 hooks around post at top (3 upper + 3 middle).
    let hook_ys = [post_h - 0.020, post_h - 0.180];
    for level in 0..2 {
        for i in 0..3 {
            let angle = (i as f64 / 3.0) * PI * 2.0 + level as f64 * (PI / 3.0);
            let arm = mesh(pg("VictorianHallStands__arm", cyl_geo!(0.004, 0.004, 0.055, 4)), &wood_mat);
            arm.rotation().z = PI / 2.0;
            arm.rotation().y = -angle;
            arm.position().set(angle.cos() * 0.030, 0.020 + hook_ys[level], angle.sin() * 0.030);
            hs.add(&arm);
            // Hook cap — small brass ball.
            let cap = mesh(pg("VictorianHallStands__cap", sphere_geo!(0.008, 6, 5)), &brass_mat);
            cap.position().set(angle.cos() * 0.058, 0.020 + hook_ys[level] - 0.006, angle.sin() * 0.058);
            hs.add(&cap);
        }
    }
    // Central top ornamental finial.
    let finial = mesh(pg("VictorianHallStands__finial", sphere_geo!(0.024, 8, 6)), &wood_mat);
    finial.position().y = 0.020 + post_h + 0.015;
    hs.add(&finial);
    // Small oval mirror at mid-height, hanging flat against the post.
    let mirror_angle = rnd() * PI * 2.0;
    let mirror = group();
    let mirror_r = 0.09;
    let m_frame = mesh(pg("VictorianHallStands__mirror", torus_geo!(mirror_r, 0.008, 5, 16)), &wood_mat);
    mirror.add(&m_frame);
    let m_surf = mesh(circle_geo!(mirror_r - 0.005, 16), lambert!({ color: 0x707078, emissive: 0x101014 }));
    mirror.add(&m_surf);
    m_frame.scale().y = 1.30; // Slight oval elongation.
    m_surf.scale().y = 1.30;
    mirror.position().set(mirror_angle.cos() * 0.033, 0.85, mirror_angle.sin() * 0.033);
    mirror.rotation().y = -mirror_angle + PI / 2.0;
    hs.add(&mirror);
    // 2-3 umbrellas in tray base (60% chance).
    if rnd() < 0.6 {
        let umb_count = 2 + (rnd() * 2.0).floor() as i32;
        let umb_colors: [u32; 5] = [0x181818, 0x2a1010, 0x102040, 0x203010, 0x604020];
        for i in 0..umb_count {
            let umb_mat = lambert!({ color: umb_colors[ri(umb_colors.len())] });
            // Umbrella shaft (leaning outward).
            let u_angle = (i as f64 / umb_count as f64) * PI * 2.0 + rnd() * 0.5;
            let shaft = mesh(pg("VictorianHallStands__umbrella", cyl_geo!(0.004, 0.004, 0.75, 4)), &umb_mat);
            shaft.position().set(u_angle.cos() * tray_r * 0.5, 0.35, u_angle.sin() * tray_r * 0.5);
            shaft.rotation().x = u_angle.sin() * 0.12;
            shaft.rotation().z = -u_angle.cos() * 0.12;
            hs.add(&shaft);
            // J-hook handle at top: upright half-ring whose stick end sits on the shaft top.
            let handle_hook = mesh(pg("VictorianHallStands__crook", torus_geo!(0.018, 0.004, 4, 8, PI)), &umb_mat);
            let mut shaft_top = v3(0, 0.375, 0);
            shaft_top.apply_euler(&shaft.rot());
            shaft_top.add(&shaft.pos());
            handle_hook.rotation().y = PI - u_angle;
            handle_hook.position().set(shaft_top.x + u_angle.cos() * 0.018, shaft_top.y - 0.002, shaft_top.z + u_angle.sin() * 0.018);
            hs.add(&handle_hook);
        }
    }
    // 50% chance: 1-2 hats on upper hooks.
    if rnd() < 0.5 {
        let hat_count = 1 + (rnd() * 2.0).floor() as i32;
        let hat_colors: [u32; 4] = [0x181818, 0x2a1810, 0x604020, 0x502010];
        for i in 0..hat_count {
            let hat_mat = lambert!({ color: hat_colors[ri(hat_colors.len())] });
            let hat_angle0 = (i as f64 / hat_count as f64) * PI * 2.0 + rnd() * 0.5;
            // Hang it on the nearest upper hook.
            let hat_angle = round_half_up(hat_angle0 / (PI * 2.0 / 3.0)) * (PI * 2.0 / 3.0);
            let brim_y = 0.020 + hook_ys[0] + 0.004;
            // Top hat / bowler variant.
            let is_top_hat = rnd() < 0.5;
            if is_top_hat {
                let crown = mesh(pg("VictorianHallStands__topHat", cyl_geo!(0.060, 0.058, 0.090, 10)), &hat_mat);
                crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.004 + 0.045, hat_angle.sin() * 0.10);
                hs.add(&crown);
                let brim = mesh(pg("VictorianHallStands__topHatBrim", cyl_geo!(0.080, 0.080, 0.008, 12)), &hat_mat);
                brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.004, hat_angle.sin() * 0.10);
                hs.add(&brim);
            } else {
                let bowler_crown = mesh(pg("VictorianHallStands__bowler", sphere_geo!(0.055, 10, 6, 0, PI * 2.0, 0, PI * 0.55)), &hat_mat);
                bowler_crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_crown);
                let bowler_brim = mesh(pg("VictorianHallStands__bowlerBrim", cyl_geo!(0.065, 0.065, 0.006, 12)), &hat_mat);
                bowler_brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_brim);
            }
        }
    }
    // 40% chance: single scarf hanging from middle hook.
    if rnd() < 0.4 {
        let scarf_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let scarf_mat = lambert!({ color: scarf_colors[ri(scarf_colors.len())] });
        let s_angle = rnd() * PI * 2.0;
        let scarf = mesh(pg("VictorianHallStands__scarf", box_geo!(0.005, 0.35, 0.045)), &scarf_mat);
        scarf.position().set(s_angle.cos() * 0.070, 0.020 + hook_ys[1] - 0.175, s_angle.sin() * 0.070);
        hs.add(&scarf);
        // Slight fringe bumps at bottom.
        for i in 0..3 {
            let fringe = mesh(pg("VictorianHallStands__fringe", box_geo!(0.005, 0.010, 0.008)), &scarf_mat);
            fringe.position().set(s_angle.cos() * 0.070, 0.020 + hook_ys[1] - 0.360, s_angle.sin() * 0.070 + (i as f64 - 1.0) * 0.010);
            hs.add(&fringe);
        }
    }
    hs.position().set(g.wx(x) + (rnd() - 0.5) * 0.6, 0, g.wz(z) + (rnd() - 0.5) * 0.6);
    hs.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hs);
}
