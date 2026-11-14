//! `place_laundry_baskets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Laundry basket w/ folded clothes (floor): 4% chance per level.
pub(crate) fn place_laundry_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Basket color pool — plastic or wicker.
    let basket_colors = [0xffffff, 0xa07040, 0xf0a0c0, 0x506080, 0xc0c0c8, 0x60a060];
    let basket_mat = lambert!({ color: pick(&basket_colors) });
    let lb = group();
    // Basket body — wide open cylinder.
    let (b_r, b_h) = (0.14, 0.16);
    let body = mesh(pg("LaundryBaskets__body", cyl_geo!(b_r, b_r * 0.85, b_h, 12, 1, true)), &basket_mat);
    body.position().y = b_h / 2.0 + 0.002;
    lb.add(&body);
    // Bottom disc.
    let bottom = mesh(circle_geo!(b_r * 0.85, 12), &basket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    lb.add(&bottom);
    // Rim torus.
    let rim = mesh(pg("LaundryBaskets__rim", torus_geo!(b_r, 0.008, 5, 14)), &basket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = b_h + 0.002;
    lb.add(&rim);
    // 2 side handles (partial torus arches on opposite sides).
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("LaundryBaskets__handle", torus_geo!(0.030, 0.006, 4, 6, PI)), &basket_mat);
        // Arch stands up on the rim in the wall's tangent plane (YZ), feet in the rim.
        handle.rotation().y = PI / 2.0;
        handle.position().set(side * b_r, b_h + 0.002, 0);
        lb.add(&handle);
    }
    // Woven texture hints — 8 vertical thin slats around body.
    let slat_mat = lambert!({ color: 0x0a0a0a, transparent: true, opacity: 0.15 });
    for i in 0..8 {
        let ang = (i as f64 / 8.0) * PI * 2.0;
        let slat = mesh(box_geo!(0.006, b_h * 0.85, 0.001), &slat_mat);
        slat.position().set(ang.dcos() * b_r * 0.95, b_h / 2.0 + 0.002, ang.dsin() * b_r * 0.95);
        slat.rotation().y = ang + PI / 2.0;
        lb.add(&slat);
    }
    // Stack of folded clothes inside — 3-5 layered rectangles in different colors.
    let cloth_colors = [0xffffff, 0xf0d0d0, 0xd0e0f0, 0xf0e0a0, 0xa0d0a0, 0xe0c0f0, 0x606080];
    let cloth_count = 3 + (rnd() * 3.0).floor() as i32;
    for i in 0..cloth_count {
        let cloth_mat = lambert!({ color: pick(&cloth_colors) });
        let cloth_h = 0.014 + rnd() * 0.006;
        let cw = b_r * 1.5 * (0.85 + rnd() * 0.15);
        let cd = b_r * 1.5 * (0.75 + rnd() * 0.20);
        let cloth = mesh(pg("LaundryBaskets__cloth", box_geo!(cw, cloth_h, cd)), &cloth_mat);
        cloth.position().set((rnd() - 0.5) * 0.01, b_h * 0.60 + i as f64 * (cloth_h + 0.001), (rnd() - 0.5) * 0.01);
        cloth.rotation().y = (rnd() - 0.5) * 0.15;
        lb.add(&cloth);
    }
    // Optional draped item hanging over rim — 40%.
    if rnd() < 0.4 {
        let drape_mat = lambert!({ color: pick(&cloth_colors) });
        // Simple L-shape: horizontal segment on rim + vertical segment hanging down.
        let drape_h = mesh(pg("LaundryBaskets__drapeTop", box_geo!(0.07, 0.006, 0.06)), &drape_mat);
        // Lies across the rim (its underside on the rim top at x = b_r).
        drape_h.position().set(b_r - 0.01, b_h + 0.015, 0);
        drape_h.rotation().z = -0.2;
        lb.add(&drape_h);
        let drape_v = mesh(pg("LaundryBaskets__drapeHang", box_geo!(0.06, 0.10, 0.005)), &drape_mat);
        // Hangs outside the rim (rim outer edge b_r + 0.008), top tucked under drape_h.
        drape_v.position().set(b_r + 0.011, b_h - 0.04, 0);
        drape_v.rotation().y = PI / 2.0; // face outward, thin side against the basket
        drape_v.rotation().z = 0.05;
        lb.add(&drape_v);
    }
    lb.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    lb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lb);
}
