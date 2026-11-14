//! `place_flower_stems`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Single flower stem: 8% chance per level, gently drifting petals.
pub(crate) fn place_flower_stems(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let stem_mat = lambert!({ color: 0x408040 });
    let bloom_colors = [0xd82040, 0xf0d020, 0xd050a0, 0xe07020];
    let bloom_mat = lambert!({ color: pick(&bloom_colors), side: DoubleSide });
    let center_mat = lambert!({ color: 0xd8b040 });
    let flower = group();
    // Stem laid flat.
    let stem = mesh(pg("FlowerStems__stem", cyl_geo!(0.002, 0.002, 0.20, 4)), &stem_mat);
    stem.rotation().z = PI / 2.0;
    stem.position().set(-0.06, 0.002, 0);
    flower.add(&stem);
    // 2 leaves.
    for i in 0..2 {
        let leaf = mesh(pg("FlowerStems__leaf", plane_geo!(0.030, 0.014)), &stem_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.position().set(-0.08 + i as f64 * 0.06, 0.003, if i == 0 { 0.016 } else { -0.016 });
        leaf.rotation().z = if i == 0 { 0.5 } else { -0.5 };
        flower.add(&leaf);
    }
    // 5 petal circle around center.
    let mut petals: Vec<Obj> = vec![]; // track for drift animator
    for p in 0..5 {
        let a = (p as f64 / 5.0) * PI * 2.0;
        let petal = mesh(circle_geo!(0.014, 6), &bloom_mat);
        petal.rotation().x = -PI / 2.0;
        let base_x = 0.03 + a.dcos() * 0.010;
        let base_z = a.dsin() * 0.010;
        petal.position().set(base_x, 0.003, base_z);
        petal.user().base_x = base_x;
        petal.user().base_z = base_z;
        petal.user().phase = a + rnd() * 0.3;
        petals.push(petal.clone());
        flower.add(&petal);
    }
    // Center dot.
    let center = mesh(circle_geo!(0.006, 6), &center_mat);
    center.rotation().x = -PI / 2.0;
    center.position().set(0.03, 0.004, 0);
    flower.add(&center);
    flower.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    flower.rotation().y = rnd() * PI * 2.0;
    g.world.add(&flower);
    // Gentle petal drift (tiny X/Z oscillation, breathes-like).
    g.animate(move |_g, _dt, t| {
        for p in &petals {
            let (bx, bz, ph) = {
                let n = p.r();
                (n.user.base_x, n.user.base_z, n.user.phase)
            };
            let mut pos = p.position();
            pos.x = bx + (t * 0.4 + ph).dsin() * 0.0015;
            pos.z = bz + (t * 0.4 + ph).dcos() * 0.0015;
        }
    });
}
