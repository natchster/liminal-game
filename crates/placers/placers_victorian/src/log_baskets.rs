//! `place_log_baskets`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_log_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Bimodal — metal wire basket or wicker basket.
    let is_wicker = rnd() < 0.5;
    let basket_color = if is_wicker {
        [0x8b5a2b, 0xa07040, 0x604020][(rnd() * 3.0).floor() as usize]
    } else {
        [0x1a1a1a, 0x606060, 0x2a2a2a][(rnd() * 3.0).floor() as usize]
    };
    let basket_mat = lambert!({ color: basket_color });
    let log_colors = [0x8b5a2b, 0xa07040, 0x805020, 0x604030];
    let bark_cap_mat = lambert!({ color: 0xd0a060 });
    let lb = group();
    let (b_r, b_h) = (0.14, 0.24);
    // Basket body — cylinder w/ open top.
    if is_wicker {
        let body = mesh(pg("LogBaskets__body", cyl_geo!(b_r, b_r * 0.9, b_h, 10, 1, true)), &basket_mat);
        body.position().y = b_h / 2.0 + 0.002;
        lb.add(&body);
        // Woven texture hints — 8 vertical thin lines.
        let slat_mat = lambert!({ color: 0x0a0a0a, transparent: true, opacity: 0.20 });
        for i in 0..8 {
            let ang = (i as f64 / 8.0) * PI * 2.0;
            let slat = mesh(box_geo!(0.006, b_h * 0.9, 0.001), &slat_mat);
            slat.position().set(ang.cos() * b_r * 0.98, b_h / 2.0 + 0.002, ang.sin() * b_r * 0.98);
            slat.rotation().y = ang + PI / 2.0;
            lb.add(&slat);
        }
    } else {
        // Metal wire basket — 12 vertical bars around perimeter.
        for i in 0..12 {
            let ang = (i as f64 / 12.0) * PI * 2.0;
            let bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, b_h, 4)), &basket_mat);
            bar.position().set(ang.cos() * b_r, b_h / 2.0 + 0.002, ang.sin() * b_r);
            lb.add(&bar);
        }
        // 3 horizontal ring bands.
        for y_frac in [0.15, 0.5, 0.85] {
            let ring = mesh(pg("LogBaskets__ring", torus_geo!(b_r, 0.003, 4, 14)), &basket_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().y = b_h * y_frac + 0.002;
            lb.add(&ring);
        }
    }
    // Bottom disc.
    let bottom = mesh(circle_geo!(b_r * 0.9, 10), &basket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    lb.add(&bottom);
    // 2 side handle loops (both variants): half rings bulging outwards on each side.
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("LogBaskets__handle", torus_geo!(0.026, 0.005, 4, 8, PI)), &basket_mat);
        handle.rotation().z = -side * PI / 2.0;
        handle.rotation().y = 0.0;
        handle.position().set(side * b_r, b_h + 0.002 - 0.032, 0);
        lb.add(&handle);
    }
    // Firewood logs stacked inside — 4-6 short logs.
    let log_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..log_count {
        let log_mat = lambert!({ color: choice(&log_colors) });
        let log_len = 0.22 + rnd() * 0.04;
        let log_r = 0.026 + rnd() * 0.010;
        let log = mesh(pg("Fireplaces__log", cyl_geo!(log_r, log_r, log_len, 8)), &log_mat);
        log.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        // Layers of two side by side, crossing the next layer at 60°; 72 % length to fit the basket.
        log.scale().y = 0.72;
        let half_l = log_len * 0.72 * 0.5;
        let layer = i / 2;
        let is_rotated = layer % 2 == 1;
        log.rotation().y = if is_rotated { PI / 3.0 } else { 0.0 };
        let across = (if i % 2 == 0 { -1.0 } else { 1.0 }) * 0.032; // offset perpendicular to the log
        let lr = log.rot();
        let layer_y = 0.002 + log_r + layer as f64 * 0.058 + half_l * lr.z.cos().abs();
        log.position().set(lr.y.sin() * across + (rnd() - 0.5) * 0.020, layer_y, lr.y.cos() * across + (rnd() - 0.5) * 0.030);
        lb.add(&log);
        // Bark end-cap circles, on the (tilted) log's two ends, facing out.
        let mut log_axis = v3(0, 1, 0);
        log_axis.apply_euler(&log.rot());
        let lp = log.pos();
        for end_side in [-1.0, 1.0] {
            let cap = mesh(circle_geo!(log_r, 6), &bark_cap_mat);
            let mut end_dir = log_axis;
            end_dir.multiply_scalar(end_side);
            cap.quaternion().set_from_unit_vectors(&v3(0, 0, 1), &end_dir);
            cap.position().copy(&lp).add_scaled_vector(&end_dir, half_l + 0.0005);
            lb.add(&cap);
        }
    }
    lb.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    lb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lb);
}
