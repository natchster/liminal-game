//! `place_potted_plants`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Potted plant: 5% chance per level.
pub(crate) fn place_potted_plants(g: &mut Game, _dist: &[i32]) {
    g.pp.plant_positions.clear(); // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pot_colors = [0xa05030, 0x604020, 0xe0e0d0, 0x30302a];
    let pot_mat = lambert!({ color: choice(&pot_colors) });
    let dirt_mat = lambert!({ color: 0x3a2818 });
    let leaf_colors = [0x408040, 0x306030, 0x506050, 0x8a5030]; // last = withered
    let leaf_mat = lambert!({ color: choice(&leaf_colors), side: DoubleSide });
    let plant = group();
    // Pot (tapered cylinder).
    let pot = mesh(pg("PottedPlants__pot", cyl_geo!(0.055, 0.045, 0.08, 10)), &pot_mat);
    pot.position().y = 0.04;
    plant.add(&pot);
    // Dirt disc.
    let dirt = mesh(circle_geo!(0.048, 10), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().y = 0.081;
    plant.add(&dirt);
    // 5-8 leaves radiating up + out.
    let n_leaves = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_leaves {
        let a = (i as f64 / n_leaves as f64) * PI * 2.0 + rnd() * 0.2;
        let leaf_h = 0.10 + rnd() * 0.08;
        // Custom triangle leaf.
        let leaf_geo = Geo::new();
        let verts: Vec<f32> =
            vec![0.0, 0.0, 0.0, 0.015, (leaf_h * 0.4) as f32, 0.0, 0.0, leaf_h as f32, 0.0, -0.015, (leaf_h * 0.4) as f32, 0.0];
        leaf_geo.set_position(verts);
        leaf_geo.set_index(vec![0, 1, 2, 0, 2, 3]);
        leaf_geo.compute_vertex_normals();
        let leaf = mesh(leaf_geo, &leaf_mat);
        leaf.position().set(a.cos() * 0.02, 0.082, a.sin() * 0.02);
        leaf.rotation().y = -a + PI / 2.0;
        leaf.rotation().z = 0.3 + rnd() * 0.4;
        plant.add(&leaf);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    plant.position().set(px, 0, pz);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
    // Track for rustle audio.
    let p = plant.pos();
    g.pp.plant_positions.push(v3(p.x, 0.15, p.z));
}
