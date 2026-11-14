//! `place_servant_bell_pulls`
use crate::shared::wall_candidates;
use game::prelude::*;
use placers_kit::place::ri;

// Victorian servant's bell pull system (wall+ceiling): 2.5% chance per level.
pub(crate) fn place_servant_bell_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.bell_pull_positions.clear(); // reset per level
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // 4-color cord pool: burgundy, forest, navy, dark gold.
    let cord_colors: [u32; 4] = [0x501020, 0x203010, 0x102040, 0x604020];
    let cord_mat = lambert!({ color: cord_colors[ri(cord_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let gold_mat = lambert!({ color: 0xc0a060, emissive: 0x201810 });
    let bp = group();
    // Wall-mounted brass bell bracket at top of wall.
    let bracket_y = 2.30;
    let bracket = mesh(pg("ServantBellPulls__bracket", box_geo!(0.08, 0.020, 0.030)), &brass_mat);
    bracket.position().set(0, bracket_y, 0.015);
    bp.add(&bracket);
    // Brass bell body — open bell shape.
    let bell = mesh(pg("ServantBellPulls__bell", cyl_geo!(0.045, 0.030, 0.075, 10, 1, true)), &brass_mat);
    bell.position().set(0, bracket_y - 0.045, 0.045);
    bp.add(&bell);
    // Bell top cap.
    let bell_top = mesh(pg("ServantBellPulls__bellTop", cyl_geo!(0.030, 0.030, 0.008, 10)), &brass_mat);
    bell_top.position().set(0, bracket_y - 0.008, 0.045);
    bp.add(&bell_top);
    // Clapper.
    let clapper = mesh(pg("ServantBellPulls__clapper", sphere_geo!(0.008, 5, 4)), &brass_mat);
    clapper.position().set(0, bracket_y - 0.055, 0.045);
    bp.add(&clapper);
    // Small mounting screws visible on bracket.
    for side in [-1.0f64, 1.0] {
        let screw = mesh(pg("ServantBellPulls__screw", cyl_geo!(0.003, 0.003, 0.003, 5)), &brass_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(side * 0.028, bracket_y, 0.003);
        bp.add(&screw);
    }
    // Spring coil connecting bracket to top of pull cord.
    let spring = mesh(pg("ServantBellPulls__spring", torus_geo!(0.008, 0.001, 4, 6)), &brass_mat);
    spring.rotation().x = PI / 2.0;
    spring.position().set(0, bracket_y - 0.020, 0.020);
    bp.add(&spring);
    // Long fabric pull cord.
    let cord_top_y = bracket_y - 0.020;
    let cord_bot_y = 1.20;
    let cord_len = cord_top_y - cord_bot_y;
    let cord = mesh(pg("ServantBellPulls__cord", cyl_geo!(0.005, 0.005, cord_len, 6)), &cord_mat);
    cord.position().set(0, (cord_top_y + cord_bot_y) / 2.0, 0.020);
    bp.add(&cord);
    // Twisted-braid ridges on cord.
    let mut braids: Vec<Obj> = vec![];
    for i in 0..3 {
        let braid = mesh(pg("ServantBellPulls__braid", torus_geo!(0.006, 0.001, 4, 8)), &cord_mat);
        braid.rotation().x = PI / 2.0;
        braid.position().set(0, cord_bot_y + (i as f64 + 1.0) * (cord_len / 4.0), 0.020);
        bp.add(&braid);
        braids.push(braid);
    }
    // Ornate tassel at bottom — bulb.
    let bulb = mesh(pg("ServantBellPulls__bulb", sphere_geo!(0.025, 8, 6)), &cord_mat);
    bulb.scale().y = 1.3;
    bulb.position().set(0, cord_bot_y - 0.020, 0.020);
    bp.add(&bulb);
    // Gold band around bulb top.
    let bulb_band = mesh(pg("ServantBellPulls__band", torus_geo!(0.021, 0.003, 4, 10)), &gold_mat);
    bulb_band.rotation().x = PI / 2.0;
    bulb_band.position().set(0, cord_bot_y - 0.005, 0.020);
    bp.add(&bulb_band);
    // 8 tassel fringe strands hanging from bulb bottom.
    for i in 0..8 {
        let angle = (i as f64 / 8.0) * PI * 2.0;
        let strand = mesh(pg("ServantBellPulls__strand", cyl_geo!(0.001, 0.001, 0.055, 3)), &cord_mat);
        strand.position().set(angle.cos() * 0.010, cord_bot_y - 0.065, 0.020 + angle.sin() * 0.010);
        strand.rotation().x = angle.sin() * 0.15;
        strand.rotation().z = -angle.cos() * 0.15;
        bp.add(&strand);
    }
    // Subtle sway animator on the cord + tassel + strands + braids.
    let _sway_group = group(); // unused
    let sway_freq = 0.28 + rnd() * 0.20;
    let sway_phase = rnd() * PI * 2.0;
    let _sway_center_y = cord_bot_y - 0.020;
    let mut swayers: Vec<Obj> = vec![cord.clone(), bulb.clone(), bulb_band.clone()];
    // Also all 8 strands (last 8 added to bp).
    let kids = bp.children();
    let strands_start = kids.len() - 8;
    for i in strands_start..kids.len() {
        swayers.push(kids[i].clone());
    }
    // Also the 3 braid rings.
    for b in &braids {
        swayers.push(b.clone());
    }
    let sway_base_x: Vec<f64> = swayers.iter().map(|s| s.pos().x).collect();
    g.animate(move |_g, _dt, t| {
        let sway_angle = (t * sway_freq + sway_phase).sin() * 0.020;
        for i in 0..swayers.len() {
            let s = &swayers[i];
            let y_delta = s.pos().y - cord_top_y; // negative
            s.position().x = sway_base_x[i] + sway_angle.sin() * y_delta * -1.0;
        }
        // The cord pivots at its top instead of sliding sideways off the spring.
        cord.rotation().z = sway_angle;
    });
    // Mount at wall position.
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.005);
    if cdx.abs() == 1 {
        bp.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        bp.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        bp.rotation().y = if cdz > 0 { PI } else { 0.0 };
        bp.position().set(wall_x + (rnd() - 0.5) * 0.4, 0, wall_z);
    }
    g.world.add(&bp);
    let p = bp.pos();
    g.pp.bell_pull_positions.push(v3(p.x, 0, p.z));
}
