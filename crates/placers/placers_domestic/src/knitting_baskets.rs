//! `place_knitting_baskets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Knitting basket w/ yarn ball + needles: 4% chance
// per level. Twin variant of the sewing basket.
pub(crate) fn place_knitting_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wicker_colors: [u32; 4] = [0xa07040, 0x8b6030, 0xc09060, 0x704a20];
    let wicker_mat = lambert!({ color: pick(&wicker_colors) });
    let basket = group();
    // Basket body — squatter than sewing basket, no handle.
    let body = mesh(pg("KnittingBaskets__body", cyl_geo!(0.11, 0.090, 0.055, 12, 1, true)), &wicker_mat);
    body.position().y = 0.028;
    basket.add(&body);
    let bottom = mesh(circle_geo!(0.090, 12), &wicker_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    basket.add(&bottom);
    let rim = mesh(pg("KnittingBaskets__rim", torus_geo!(0.11, 0.006, 5, 14)), &wicker_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.055;
    basket.add(&rim);
    // Yarn ball inside — big sphere, saturated color.
    let yarn_colors: [u32; 7] = [0xa02020, 0x2060a0, 0x408060, 0xa060a0, 0xd0a040, 0xf0a0c0, 0x606060];
    let yarn_color = pick(&yarn_colors);
    let yarn_mat = lambert!({ color: yarn_color });
    let yarn = mesh(pg("KnittingBaskets__yarn", sphere_geo!(0.055, 8, 6)), &yarn_mat);
    yarn.position().set(0.020, 0.075, 0);
    basket.add(&yarn);
    // Subtle yarn ball roll animator — 40% chance.
    // Tiny rotation on Z + gentle micro-shift on X, mimics "just settled".
    if rnd() < 0.4 {
        let roll_phase = rnd() * PI * 2.0;
        let roll_phase2 = rnd() * PI * 2.0;
        let yarn_base_x = yarn.pos().x;
        let yarn2 = yarn.clone();
        g.animate(move |_g, _dt, t| {
            // Slow gentle rocking; keeps yarn ball visually alive.
            yarn2.rotation().z = (t * 0.6 + roll_phase).dsin() * 0.10;
            yarn2.rotation().x = (t * 0.5 + roll_phase2).dsin() * 0.08;
            yarn2.position().x = yarn_base_x + (t * 0.7 + roll_phase).dsin() * 0.001;
        });
    }
    // Wound-thread cross-lines on the yarn ball (2 thin torus rings).
    for _i in 0..2 {
        let wind = mesh(pg("KnittingBaskets__wind", torus_geo!(0.055, 0.002, 4, 12)), &yarn_mat);
        wind.position().copy(&yarn.pos());
        wind.rotation().x = (rnd() - 0.5) * PI;
        wind.rotation().z = (rnd() - 0.5) * PI;
        basket.add(&wind);
    }
    // Thread trail leaving the ball, arching out to floor — thin cylinder segments.
    let trail_segs = 5;
    let mut prev = v3(0.020, 0.075, 0);
    for i in 0..trail_segs {
        let nx = 0.020 + 0.055 + (i + 1) as f64 * 0.024;
        let ny = 0.075 - (i + 1) as f64 * 0.014;
        let nz = 0.010 + (i + 1) as f64 * 0.006;
        let next = v3(nx, 0.002f64.max(ny), nz);
        let mut mid = prev;
        mid.add(&next).multiply_scalar(0.5);
        let dir_len = next.distance_to(&prev);
        let seg = mesh(pg("KnittingBaskets__strand", cyl_geo!(0.0018, 0.0018, dir_len, 4)), &yarn_mat);
        seg.position().copy(&mid);
        let mut v = next;
        v.sub(&prev).normalize();
        // Rotate cylinder Y-axis to align w/ (prev→next).
        let y_axis = v3(0, 1, 0);
        let mut quat = Quat::IDENTITY;
        quat.set_from_unit_vectors(&y_axis, &v);
        seg.quaternion().copy(&quat);
        basket.add(&seg);
        prev = next;
    }
    // 2 knitting needles crossed inside — angled diagonally.
    let needle_mat = lambert!({ color: 0xc0a070 });
    for i in 0..2 {
        let needle = mesh(pg("KnittingBaskets__needle", cyl_geo!(0.002, 0.002, 0.22, 5)), &needle_mat);
        needle.rotation().z = PI / 2.0 + if i == 0 { -0.4 } else { 0.4 };
        needle.rotation().y = if i == 0 { 0.0 } else { PI / 8.0 };
        // Both low ends inside the basket; the second needle used to poke out through the wall.
        needle.position().set(if i == 0 { -0.030 } else { 0.030 }, 0.080 + i as f64 * 0.008, if i == 0 { -0.020 } else { -0.010 });
        basket.add(&needle);
        // Needle knob on the needle's upper end (tiny sphere), sticking out of the basket.
        let knob = mesh(pg("KnittingBaskets__knob", sphere_geo!(0.006, 5, 4)), &needle_mat);
        let mut needle_dir = v3(0, 1, 0);
        needle_dir.apply_euler(&needle.rot());
        if needle_dir.y < 0.0 {
            needle_dir.negate();
        }
        let mut kp = needle.pos();
        kp.add_scaled_vector(&needle_dir, 0.108);
        knob.position().copy(&kp);
        basket.add(&knob);
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}
