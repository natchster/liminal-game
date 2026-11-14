//! `place_small_triangles`: Small percussion triangle: 6% chance per level. Bent metal triangle + beater rod. Child-music family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small percussion triangle: 6% chance per level. Bent metal
/// triangle + beater rod. Child-music family.
pub(crate) fn place_small_triangles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let metal_mat = lambert!({ color: 0xc0c8d0 });
    let wood_mat = lambert!({ color: 0x8a6540 });
    let tri = group();
    // Triangle is 3 metal rod segments forming triangle silhouette laid flat.
    let side = 0.10;
    let half_h = side * 3f64.sqrt() / 4.0;
    // (x, z) of each corner (y is 0)
    let vertices = [(-side / 2.0, -half_h), (side / 2.0, -half_h), (0.0, half_h)];
    for i in 0..3 {
        let (x0, z0) = vertices[i];
        let (x1, z1) = vertices[(i + 1) % 3];
        // Skip one side to leave the open-triangle gap.
        if i == 2 {
            continue;
        }
        let seg_len = hypot2(x1 - x0, z1 - z0);
        let seg = mesh(pg("SmallTriangles__rod", cyl_geo!(0.003, 0.003, seg_len, 5)), &metal_mat);
        seg.position().set((x0 + x1) / 2.0, 0.003, (z0 + z1) / 2.0);
        seg.rotation().x = PI / 2.0;
        seg.rotation().z = (-(x1 - x0)).datan2(z1 - z0); // rod axis along (dx, 0, dz)
        tri.add(&seg);
    }
    // Third side with a small gap
    let (x2a, z2a) = vertices[2];
    let (vx0, vz0) = vertices[0];
    let (dx, dz) = (vx0 - x2a, vz0 - z2a);
    let seg_len = hypot2(dx, dz) * 0.90;
    let seg3 = mesh(pg("SmallTriangles__rod", cyl_geo!(0.003, 0.003, seg_len, 5)), &metal_mat);
    seg3.position().set(x2a + dx * 0.45, 0.003, z2a + dz * 0.45);
    seg3.rotation().x = PI / 2.0;
    seg3.rotation().z = (-dx).datan2(dz); // rod axis along (dx, 0, dz)
    tri.add(&seg3);
    // Beater rod: thin wood cylinder laid nearby.
    let beater = mesh(pg("SmallTriangles__beater", cyl_geo!(0.003, 0.003, 0.08, 5)), &wood_mat);
    beater.rotation().z = PI / 2.0;
    beater.position().set(0.09, 0.003, 0.02);
    beater.rotation().y = rnd() * 0.6 - 0.3;
    tri.add(&beater);
    tri.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tri.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tri);
}
