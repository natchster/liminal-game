//! `place_tote_bags`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Garden tool tote bag: 4% chance per level. Fabric bag w/ tool handles
// poking out.
pub(crate) fn place_tote_bags(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bag_colors: [u32; 4] = [0x604020, 0x804030, 0x506040, 0xa08050];
    let bag_mat = lambert!({ color: pick(&bag_colors) });
    let handle_mat = lambert!({ color: 0x8b5a2b });
    let tool_mat = lambert!({ color: 0x505050 });
    let tote = group();
    // Main bag body (cube-ish).
    let body = mesh(pg("ToteBags__body", box_geo!(0.16, 0.16, 0.10)), &bag_mat);
    body.position().y = 0.08;
    tote.add(&body);
    // 2 tote strap handles (arcs over top).
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("ToteBags__strap", torus_geo!(0.030, 0.005, 4, 8, PI)), &handle_mat);
        // Arc in the XY plane on the front/back wall, ends tucked 3 mm under the rim.
        handle.position().set(0, 0.157, side * 0.047);
        tote.add(&handle);
    }
    // 2-3 tool handles poking out of bag top.
    let tool_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..tool_count {
        let t_handle = mesh(pg("ToteBags__tool", cyl_geo!(0.006, 0.006, 0.11, 6)), &tool_mat);
        t_handle.position().set(-0.03 + i as f64 * 0.024, 0.20, 0);
        t_handle.rotation().z = (rnd() - 0.5) * 0.4;
        tote.add(&t_handle);
    }
    tote.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    tote.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tote);
}
