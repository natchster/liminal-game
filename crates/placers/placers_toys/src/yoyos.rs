//! `place_yoyos`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Yo-yo: 7% chance per level. Two disc halves + string trailing.
pub(crate) fn place_yoyos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let yoyo_colors = [0xc02020, 0x2080e0, 0x40c060, 0xe0c040, 0xd050a0, 0x202020];
    let yoyo_mat = lambert!({ color: choice(&yoyo_colors) });
    let axle_mat = lambert!({ color: 0x606060 });
    let string_mat = lambert!({ color: 0xf0f0e8 });
    let yoyo = group();
    let r = 0.028;
    let half_l = mesh(pg("Yoyos__half", cyl_geo!(r, r, 0.008, 12)), &yoyo_mat);
    half_l.rotation().x = PI / 2.0;
    half_l.position().set(0, r, -0.006);
    yoyo.add(&half_l);
    let half_r = mesh(pg("Yoyos__half", cyl_geo!(r, r, 0.008, 12)), &yoyo_mat);
    half_r.rotation().x = PI / 2.0;
    half_r.position().set(0, r, 0.006);
    yoyo.add(&half_r);
    let axle = mesh(pg("Yoyos__axle", cyl_geo!(0.006, 0.006, 0.008, 6)), &axle_mat);
    axle.rotation().x = PI / 2.0;
    axle.position().set(0, r, 0);
    yoyo.add(&axle);
    // Trailing string: one joined polyline from the hub, out through the gap between the
    // halves, down to the floor and curving away along it.
    let mut str_pts: Vec<Vec3> = vec![v3(0.004, r - 0.004, 0), v3(r + 0.006, 0.0006, 0)];
    let mut hd = 0.0f64;
    for i in 1..6 {
        hd += 0.12;
        let q = str_pts[i];
        str_pts.push(v3(q.x + hd.cos() * 0.028, 0.0006, q.z + hd.sin() * 0.028));
    }
    for i in 0..6 {
        let seg = mesh(cyl_geo!(0.0006, 0.0006, 0.030, 3), &string_mat);
        let mut d = str_pts[i + 1];
        d.sub(&str_pts[i]);
        let len = d.length();
        d.divide_scalar(len);
        seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
        seg.scale().y = (len + 0.001) / 0.030; // overlap the joints a little
        seg.position().copy(&str_pts[i]).lerp(&str_pts[i + 1], 0.5);
        yoyo.add(&seg);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    yoyo.position().set(px, 0, pz);
    yoyo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&yoyo);
}
