//! `place_dead_flower_vases`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Dead flowers in vase: 6% chance per level. Vase + dried stems + petals.
pub(crate) fn place_dead_flower_vases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let vase_colors = [0xe8dcc0, 0x8a5030, 0x4a2818, 0xa0a0a0];
    let vase_mat = lambert!({ color: pick_hex(&vase_colors) });
    let stem_mat = lambert!({ color: 0x604020 });
    let petal_mat = lambert!({ color: 0x8a5030, side: DoubleSide });
    let vase = group();
    // Vase: tapered cylinder + narrow neck.
    let body = mesh(pg("DeadFlowerVases__body", cyl_geo!(0.035, 0.028, 0.070, 12)), &vase_mat);
    body.position().y = 0.035;
    vase.add(&body);
    let neck = mesh(pg("DeadFlowerVases__neck", cyl_geo!(0.022, 0.028, 0.020, 10)), &vase_mat);
    neck.position().y = 0.080;
    vase.add(&neck);
    // 4-6 dried curled stems.
    let n_stems = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_stems {
        let a = (i as f64 / n_stems as f64) * PI * 2.0 + rnd() * 0.5;
        let stem_len = 0.08 + rnd() * 0.06;
        let tilt = rnd() * 0.4 + 0.2;
        let stem = mesh(pg("DeadFlowerVases__stem", cyl_geo!(0.0015, 0.0015, stem_len, 4)), &stem_mat);
        // Foot 1 cm down inside the neck (mouth at 0.090).
        stem.position().set(
            a.cos() * 0.010 + a.cos() * stem_len / 2.0 * tilt.sin(),
            0.080 + stem_len / 2.0 * tilt.cos(),
            a.sin() * 0.010 + a.sin() * stem_len / 2.0 * tilt.sin(),
        );
        stem.rotation().z = -a.cos() * tilt;
        stem.rotation().x = a.sin() * tilt;
        vase.add(&stem);
        // Drooping tip petal.
        let petal = mesh(circle_geo!(0.006, 5), &petal_mat);
        petal.position().set(
            a.cos() * 0.010 + a.cos() * stem_len * tilt.sin(),
            0.080 + stem_len * tilt.cos() - 0.005,
            a.sin() * 0.010 + a.sin() * stem_len * tilt.sin(),
        );
        petal.rotation().x = -PI / 3.0;
        vase.add(&petal);
    }
    // 3 fallen petals on floor.
    for i in 0..3 {
        let fallen = mesh(circle_geo!(0.006, 5), &petal_mat);
        fallen.rotation().x = -PI / 2.0;
        fallen.position().set((rnd() - 0.5) * 0.10, 0.002, (rnd() - 0.5) * 0.10);
        fallen.rotation().z = rnd() * PI * 2.0;
        vase.add(&fallen);
    }
    scatter_on_floor(g, &vase, x, z, 2.4);
    g.world.add(&vase);
}
