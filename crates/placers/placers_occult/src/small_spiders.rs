//! `place_small_spiders`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;

// Small spider: 5% chance per level. Dark body + 8 legs.
// Halloween family.
pub(crate) fn place_small_spiders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let spider_mat = lambert!({ color: 0x100808 });
    let eye_mat = basic_mat!({ color: 0xd82040 });
    let spider = group();
    let body_r = 0.014;
    // Body (abdomen sphere).
    let body = mesh(pg("SmallSpiders__body", sphere_geo!(body_r, 8, 6)), &spider_mat);
    body.position().y = body_r;
    spider.add(&body);
    // Head (smaller sphere).
    let head = mesh(pg("SmallSpiders__head", sphere_geo!(body_r * 0.7, 6, 5)), &spider_mat);
    head.position().set(body_r * 1.2, body_r, 0);
    spider.add(&head);
    // 8 legs (bent lines using 2 cylinders each).
    // Each leg: hip under the body/head joint, the first segment rising 20°
    // outward to a knee, the second dropping 63° to a foot on the floor.
    // Four legs a side (±Z; the head is +X), fanned from forward to back.
    let leg_span = |m: &Obj, p: Vec3, q: Vec3| {
        let mut mid = p;
        mid.add(&q).multiply_scalar(0.5);
        m.position().copy(&mid);
        let mut d = q;
        d.sub(&p).normalize();
        m.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
    };
    for i in 0..8 {
        let side = if i < 4 { -1.0 } else { 1.0 };
        let leg_i = i % 4;
        let a = -PI / 3.0 + (leg_i as f64 / 3.0) * (PI * 2.0 / 3.0);
        let h = v3(-a.sin(), 0, side * a.cos());
        let hip = v3(0.010 + h.x * 0.008, 0.006, h.z * 0.008);
        let mut knee = hip;
        knee.add_scaled_vector(&h, 0.022 * 0.35f64.cos()).set_y(0.006 + 0.022 * 0.35f64.sin());
        let mut foot = knee;
        foot.add_scaled_vector(&h, (0.014f64.powi(2) - (knee.y - 0.001).powi(2)).sqrt()).set_y(0.001);
        // First segment (out from body).
        let seg1 = mesh(pg("SmallSpiders__leg", cyl_geo!(0.001, 0.001, 0.022, 3)), &spider_mat);
        leg_span(&seg1, hip, knee);
        spider.add(&seg1);
        // Second segment (down to floor).
        let seg2 = mesh(pg("SmallSpiders__leg", cyl_geo!(0.001, 0.001, 0.014, 3)), &spider_mat);
        leg_span(&seg2, knee, foot);
        spider.add(&seg2);
    }
    // 2 tiny red eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.0015, 4, 3), &eye_mat);
        eye.position().set(body_r * 1.4, body_r + 0.004, s * 0.004);
        spider.add(&eye);
    }
    spider.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    spider.rotation().y = rnd() * PI * 2.0;
    g.world.add(&spider);
}
