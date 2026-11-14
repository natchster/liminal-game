//! `place_glass_shards`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Broken glass shards: 5% chance per level. Small angular triangles on floor.
pub(crate) fn place_glass_shards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let glass_mat = lambert!({ color: 0xc0d0d8, transparent: true, opacity: 0.55, side: DoubleSide });
    let cluster = group();
    let count = 4 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        // Small custom triangle.
        let s = 0.015 + rnd() * 0.015;
        let vx = (rnd() - 0.3) * s;
        let vz = s * (0.7 + rnd() * 0.4);
        let verts: Vec<f32> = vec![0.0, 0.0, 0.0, s as f32, 0.0, 0.0, vx as f32, 0.0, vz as f32];
        let geom = tri_geo(verts, vec![0, 1, 2]);
        let shard = mesh(geom, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.13, 0.002 + rnd() * 0.003, (rnd() - 0.5) * 0.13);
        shard.rotation().y = rnd() * PI * 2.0;
        shard.rotation().z = (rnd() - 0.5) * 0.4;
        cluster.add(&shard);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

/// A custom geometry: positions + index, vertex normals computed.
fn tri_geo(verts: Vec<f32>, idx: Vec<u32>) -> Geo {
    let geom = Geo::new();
    geom.set_position(verts);
    geom.set_index(idx);
    geom.compute_vertex_normals();
    geom
}
