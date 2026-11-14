//! `place_cuckoo_clocks`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Cuckoo clock: 3% chance per level. Chalet body w/ peaked roof + face +
// pinecone weights.
pub(crate) fn place_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.cuckoo_clock_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let chalet_mat = lambert!({ color: 0x8b5a2b });
    let roof_mat = lambert!({ color: 0x604020 });
    let face_mat = lambert!({ color: 0xf0e8c8 });
    let hand_mat = lambert!({ color: 0x101010 });
    let cone_mat = lambert!({ color: 0x504020 });
    let cuckoo = group();
    let mount_y = 1.75;
    // Chalet body (main rectangle).
    let body = mesh(pg("CuckooClocks__body", box_geo!(0.22, 0.20, 0.06)), &chalet_mat);
    body.position().y = mount_y;
    cuckoo.add(&body);
    // Peaked roof (triangular prism via cone with 3 sides).
    let roof = mesh(pg("CuckooClocks__roof", cone_geo!(0.14, 0.10, 3)), &roof_mat);
    roof.rotation().y = PI / 2.0;
    roof.position().y = mount_y + 0.158; // gable wall's foot (part y −0.058) on the case top (+0.10)
    cuckoo.add(&roof);
    // Round clock face.
    let face = mesh(circle_geo!(0.055, 12), &face_mat);
    face.position().set(0, mount_y + 0.020, 0.032);
    cuckoo.add(&face);
    // Face rim.
    let rim = mesh(pg("CuckooClocks__rim", torus_geo!(0.055, 0.004, 4, 12)), &roof_mat);
    rim.rotation().x = 0.0;
    rim.position().set(0, mount_y + 0.020, 0.033);
    cuckoo.add(&rim);
    // Hour + minute hands.
    let hour_h = mesh(pg_own("CuckooClocks__hour", box_geo!(0.004, 0.030, 0.003)), &hand_mat);
    hour_h.position().set(0, mount_y + 0.035, 0.036);
    hour_h.geo().translate(0, -0.005, 0);
    hour_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&hour_h);
    let minute_h = mesh(pg_own("CuckooClocks__minute", box_geo!(0.003, 0.040, 0.003)), &hand_mat);
    minute_h.position().set(0, mount_y + 0.035, 0.037);
    minute_h.geo().translate(0, -0.010, 0);
    minute_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&minute_h);
    // 2 pinecone weights hanging below body (chain + cone).
    for side in [-1.0, 1.0] {
        let chain = mesh(pg("CuckooClocks__chain", cyl_geo!(0.001, 0.001, 0.12, 3)), &roof_mat);
        chain.position().set(side * 0.070, mount_y - 0.16, 0.020);
        cuckoo.add(&chain);
        let cone = mesh(pg("CuckooClocks__cone", cone_geo!(0.010, 0.024, 6)), &cone_mat);
        cone.rotation().x = PI;
        cone.position().set(side * 0.070, mount_y - 0.231, 0.020); // eye (top) on the chain's end (−0.22)
        cuckoo.add(&cone);
    }
    cuckoo.rotation().y = face_wall(dx, dz);
    cuckoo.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.030), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.030));
    g.world.add(&cuckoo);
    let p = cuckoo.pos();
    g.pp.cuckoo_clock_positions.push(v3(p.x, 0, p.z)); // track for cuckoo call
}
