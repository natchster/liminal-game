//! `place_moon_globes`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_moon_globes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x8b5a2b, 0x2a1810, 0x505030, 0x101010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let mg = group();
    // Base cylinder.
    let base = mesh(pg("MoonGlobes__base", cyl_geo!(0.09, 0.11, 0.020, 12)), &wood_mat);
    base.position().y = 0.010;
    mg.add(&base);
    // Central column.
    let column = mesh(pg("MoonGlobes__column", cyl_geo!(0.010, 0.014, 0.32, 6)), &wood_mat);
    column.position().y = 0.18;
    mg.add(&column);
    // Yoke — full ring.
    let globe_r = 0.11;
    // The ring's lowest point (r 0.124) rests in the column's cup (0.34).
    let globe_center_y = 0.352 + globe_r;
    let yoke = mesh(pg("MoonGlobes__yoke", torus_geo!(globe_r + 0.010, 0.004, 4, 20)), &brass_mat);
    // Upright full meridian in the axis plane (XY).
    yoke.rotation().x = 0.0;
    yoke.rotation().z = 0.15; // Slight tilt.
    yoke.position().y = globe_center_y;
    mg.add(&yoke);
    // Axis rod.
    let axis_rod = mesh(pg("FloorGlobes__axis", cyl_geo!(0.002, 0.002, globe_r * 2.4, 4)), &brass_mat);
    axis_rod.rotation().z = 0.15;
    axis_rod.position().y = globe_center_y;
    mg.add(&axis_rod);
    // Moon sphere — light gray w/ dark maria patches + craters.
    let moon_group = group();
    let surface_mat = lambert!({ color: 0xa0a0a0 });
    let moon = mesh(pg("MoonGlobes__moon", sphere_geo!(globe_r, 20, 14)), &surface_mat);
    moon_group.add(&moon);
    // Dark "maria" patches — 4-6 larger dark ellipses.
    let maria_colors = [0x404050, 0x504040, 0x505050];
    let maria_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..maria_count {
        let maria_mat = lambert!({ color: choice(&maria_colors) });
        let u = rnd() * PI * 2.0;
        let v = rnd() * (PI * 0.75) + PI * 0.125;
        let rr = globe_r * 1.005;
        let mx = u.cos() * v.sin() * rr;
        let my = v.cos() * rr;
        let mz = u.sin() * v.sin() * rr;
        let maria = mesh(circle_geo!(0.015 + rnd() * 0.010, 8), &maria_mat);
        maria.position().set(mx, my, mz);
        maria.look_at(0, 0, 0);
        maria.rotation().y += PI;
        maria.rotation().z = rnd() * PI;
        maria.scale().set(1.2 + rnd() * 0.4, 1.0, 1.0);
        moon_group.add(&maria);
    }
    // Impact craters — 12-18 small dark circles.
    let crater_mat = lambert!({ color: 0x707078 });
    let crater_count = 12 + (rnd() * 7.0).floor() as i32;
    for i in 0..crater_count {
        let u = rnd() * PI * 2.0;
        let v = rnd() * PI + 0.05;
        let rr = globe_r * 1.003;
        let cx = u.cos() * v.sin() * rr;
        let cy = v.cos() * rr;
        let cz = u.sin() * v.sin() * rr;
        let crater = mesh(circle_geo!(0.004 + rnd() * 0.006, 6), &crater_mat);
        crater.position().set(cx, cy, cz);
        crater.look_at(0, 0, 0);
        crater.rotation().y += PI;
        moon_group.add(&crater);
    }
    moon_group.rotation().z = 0.15;
    moon_group.position().y = globe_center_y;
    mg.add(&moon_group);
    // 40% chance: slow rotation animator.
    if rnd() < 0.4 {
        let m2 = moon_group.clone();
        g.animate(move |_g, dt, _t| {
            m2.rotation().y += dt * 0.20; // Slower than earth ~3 rpm
        });
    }
    mg.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    mg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mg);
}
