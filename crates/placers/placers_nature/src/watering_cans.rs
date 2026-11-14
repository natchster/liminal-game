//! `place_watering_cans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Mini watering can: 5% chance per level. Body + spout + handle.
pub(crate) fn place_watering_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let can_colors: [u32; 5] = [0x408060, 0x606060, 0x2060c0, 0xc02020, 0xc0a040];
    let can_mat = lambert!({ color: pick(&can_colors) });
    let rose_mat = lambert!({ color: 0x505050 });
    let can = group();
    // Main body (tapered cylinder).
    let body = mesh(pg("WateringCans__body", cyl_geo!(0.045, 0.050, 0.080, 12)), &can_mat);
    body.position().y = 0.040;
    can.add(&body);
    // Spout (angled cylinder extending out).
    // The spout's narrow end (+Y) points out and up along spout_dir; its wide,
    // flanged end is set into the body low on the -X side.
    let spout = mesh(pg("WateringCans__spout", cyl_geo!(0.008, 0.010, 0.095, 6)), &can_mat);
    let spout_dir = v3(-(PI / 3.0).dsin(), (PI / 3.0).dcos(), 0);
    spout.rotation().z = PI / 3.0;
    spout.position().set(-0.056, 0.070, 0);
    can.add(&spout);
    let mut spout_end = spout.pos();
    spout_end.add_scaled_vector(&spout_dir, 0.0475);
    // Rose (dispenser end) on spout — small perforated disc: conical back
    // on the spout's lip, face (+Y) out along the spout.
    let rose = mesh(pg("WateringCans__rose", cyl_geo!(0.012, 0.014, 0.006, 8)), &rose_mat);
    rose.rotation().z = PI / 3.0;
    {
        let mut p = rose.position();
        p.copy(&spout_end);
        p.add_scaled_vector(&spout_dir, 0.0025);
    }
    can.add(&rose);
    let mut rose_face = rose.pos();
    rose_face.add_scaled_vector(&spout_dir, 0.0035);
    let rose_up = v3((PI / 3.0).dcos(), (PI / 3.0).dsin(), 0);
    // 5 tiny hole dots on rose.
    for hh in 0..5 {
        let hole = mesh(circle_geo!(0.0012, 4), lambert!({ color: 0x101010 }));
        let a = (hh as f64 / 5.0) * PI * 2.0;
        hole.quaternion().set_from_unit_vectors(&v3(0, 0, 1), &spout_dir);
        {
            let mut p = hole.position();
            p.copy(&rose_face);
            p.add_scaled_vector(&rose_up, a.dcos() * 0.005);
            p.add(&v3(0, 0, a.dsin() * 0.005));
        }
        can.add(&hole);
    }
    // Handle (curved arc over top): an upright strap in the spout's plane
    // (XY), both ends set into the can's top.
    let handle = mesh(pg("WateringCans__handle", torus_geo!(0.038, 0.005, 4, 8, PI)), &can_mat);
    handle.position().set(0.003, 0.081, 0);
    can.add(&handle);
    can.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}
