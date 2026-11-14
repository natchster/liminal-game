//! `place_briefcases`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Abandoned briefcase: 15% chance per level.
pub(crate) fn place_briefcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let leather_mat = lambert!({ color: 0x1e1610 });
    let clip_mat = lambert!({ color: 0x8a7a48 });
    let handle_mat = lambert!({ color: 0x281a12 });
    let briefcase = group();
    // Body: 0.36 wide × 0.10 thick × 0.28 deep.
    let body = mesh(pg("Briefcases__body", box_geo!(0.36, 0.10, 0.28)), &leather_mat);
    body.position().y = 0.05;
    briefcase.add(&body);
    // Top seam (thin darker line along top edge).
    let seam = mesh(pg("Briefcases__rim", box_geo!(0.36, 0.008, 0.28)), &handle_mat);
    seam.position().y = 0.10;
    briefcase.add(&seam);
    // 2 clasps on top-front.
    for cs in [-1.0, 1.0] {
        let clasp = mesh(pg("Briefcases__clasp", box_geo!(0.03, 0.01, 0.02)), &clip_mat);
        clasp.position().set(cs * 0.10, 0.106, 0.14);
        briefcase.add(&clasp);
    }
    // Handle: arch on top.
    let handle_arch = mesh(pg("Briefcases__handle", torus_geo!(0.05, 0.005, 4, 12, PI)), &handle_mat);
    handle_arch.rotation().x = -PI / 2.0;
    handle_arch.position().set(0, 0.109, 0); // lying on the rim's top (y 0.104), tube r 0.005
    briefcase.add(&handle_arch);
    // 15% chance briefcase tipped on side.
    let is_tipped = rnd() < 0.15;
    if is_tipped {
        briefcase.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
        briefcase.position().y = 0.18; // local x (±0.18, the width) becomes height
    }
    // 10% chance briefcase is OPEN with scrap papers spilling out (not when tipped).
    let is_open = !is_tipped && rnd() < 0.10;
    if is_open {
        let lid = mesh(pg("Briefcases__lid", box_geo!(0.36, 0.02, 0.28)), &leather_mat);
        let lid_group = group();
        // the lid reaches forward from the hinge (z 0…0.28) so it swings up behind the case
        lid.position().set(0, 0.01, 0.14);
        lid_group.add(&lid);
        // the carry handle rides on the lid's outer face
        lid_group.add(&handle_arch);
        handle_arch.position().set(0, 0.025, 0.14);
        lid_group.position().set(0, 0.10, -0.14); // pivot at back top edge
        lid_group.rotation().x = -PI / 1.8; // ~100° open
        briefcase.add(&lid_group);
    }
    // Wall-corner bias for realistic "set down against wall" placement.
    let corner_bias = rnd() < 0.70;
    let ox = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.1 + rnd() * 0.4) } else { (rnd() - 0.5) * 2.2 };
    let oz = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.1 + rnd() * 0.4) } else { (rnd() - 0.5) * 2.2 };
    briefcase.position().x = g.wx(pk.0) + ox;
    briefcase.position().z = g.wz(pk.1) + oz;
    briefcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&briefcase);
    // Open briefcase spills 2-3 scrap papers around it.
    if is_open {
        let paper_mat = lambert!({ color: 0xece0d0 });
        let scrap_count = 2 + (rnd() * 2.0).floor() as i32;
        let bp = briefcase.pos();
        for _sp in 0..scrap_count {
            let scrap = mesh(pg("Briefcases__scrap", plane_geo!(0.10 + rnd() * 0.04, 0.13 + rnd() * 0.05)), &paper_mat);
            scrap.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.2;
            scrap.rotation().z = rnd() * PI * 2.0;
            let sc_ang = rnd() * PI * 2.0;
            let sc_rad = 0.15 + rnd() * 0.25;
            scrap.position().set(bp.x + sc_ang.dcos() * sc_rad, 0.003, bp.z + sc_ang.dsin() * sc_rad);
            g.world.add(&scrap);
        }
    }
}
