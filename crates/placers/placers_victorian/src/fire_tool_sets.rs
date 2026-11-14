//! `place_fire_tool_sets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Wrought-iron fire tool set on stand (floor): 3% chance per level.
pub(crate) fn place_fire_tool_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Wrought-iron matte black.
    let iron_mat = lambert!({ color: 0x181818 });
    // Optional brass-tipped highlight material (30% variant).
    let use_brass_tips = rnd() < 0.3;
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let tip_mat = if use_brass_tips { brass_mat.clone() } else { iron_mat.clone() };
    let fts = group();
    // Stand base — heavy round disc.
    let base_r = 0.075;
    let base = mesh(pg("FireToolSets__base", cyl_geo!(base_r, base_r + 0.008, 0.012, 12)), &iron_mat);
    base.position().y = 0.006;
    fts.add(&base);
    // Vertical stand rod (60 cm tall).
    let rod_h = 0.60;
    let rod = mesh(pg("FireToolSets__rod", cyl_geo!(0.006, 0.006, rod_h, 6)), &iron_mat);
    rod.position().y = 0.012 + rod_h / 2.0;
    fts.add(&rod);
    // Top ornamental finial.
    let finial = mesh(pg("FireToolSets__finial", sphere_geo!(0.012, 8, 6)), &tip_mat);
    finial.position().y = 0.012 + rod_h + 0.010;
    fts.add(&finial);
    // Cross-bar arms for hanging tools.
    let arm_y = 0.012 + rod_h - 0.030;
    let num_tools = 3 + (rnd() * 2.0).floor() as i32; // 3 or 4 tools
    for i in 0..num_tools {
        let arm_angle = (i as f64 / num_tools as f64) * PI * 2.0;
        let arm = mesh(pg("FireToolSets__arm", cyl_geo!(0.0025, 0.0025, 0.030, 4)), &iron_mat);
        arm.rotation().z = PI / 2.0;
        arm.position().set(arm_angle.cos() * 0.015, arm_y, arm_angle.sin() * 0.015);
        arm.rotation().y = -arm_angle;
        fts.add(&arm);
        // Small hook cap on arm end.
        let hook = mesh(pg("FireToolSets__hook", torus_geo!(0.004, 0.001, 4, 6, PI)), &iron_mat);
        hook.rotation().x = PI / 2.0;
        hook.rotation().z = -arm_angle;
        hook.position().set(arm_angle.cos() * 0.030, arm_y - 0.002, arm_angle.sin() * 0.030);
        fts.add(&hook);
        // Tool hanging from hook.
        let tool_cx = arm_angle.cos() * 0.030;
        let tool_cz = arm_angle.sin() * 0.030;
        let tool_types = ["poker", "tongs", "shovel", "brush"];
        let tool_type = tool_types[(i % 4) as usize];
        if tool_type == "poker" {
            let shaft_len = 0.36;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            let hook_tip = mesh(pg("FireToolSets__pokerTip", cyl_geo!(0.001, 0.005, 0.020, 4)), &iron_mat);
            hook_tip.position().set(tool_cx + 0.006, arm_y - shaft_len - 0.005, tool_cz);
            hook_tip.rotation().z = 0.5;
            fts.add(&hook_tip);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        } else if tool_type == "tongs" {
            let shaft_len = 0.32;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            // 2 pincer arms V-splayed at bottom.
            for pside in [-1.0f64, 1.0] {
                let pinc = mesh(pg("FireToolSets__pincer", cyl_geo!(0.0025, 0.0025, 0.055, 4)), &iron_mat);
                pinc.position().set(tool_cx + pside * 0.008, arm_y - shaft_len - 0.020, tool_cz);
                pinc.rotation().z = pside * 0.3;
                fts.add(&pinc);
            }
            // Loop grip at top: upright ring across the arm end.
            let grip_loop = mesh(pg("FireToolSets__loop", torus_geo!(0.010, 0.002, 4, 10)), &tip_mat);
            grip_loop.position().set(tool_cx, arm_y + 0.005, tool_cz);
            grip_loop.rotation().y = -arm_angle - PI / 2.0;
            fts.add(&grip_loop);
        } else if tool_type == "shovel" {
            let shaft_len = 0.30;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            // Small square scoop pan, hanging upright below the shaft.
            let scoop = mesh(pg("FireToolSets__scoop", box_geo!(0.035, 0.005, 0.030)), &iron_mat);
            {
                let mut r = scoop.rotation();
                r.set(PI / 2.0, PI / 2.0 - arm_angle, 0);
                r.order = EulerOrder::YXZ;
            }
            scoop.position().set(arm_angle.cos() * 0.022, arm_y - shaft_len - 0.0195, arm_angle.sin() * 0.022);
            fts.add(&scoop);
            // Scoop back wall (raised).
            let scoop_back = mesh(pg("FireToolSets__scoopBack", box_geo!(0.035, 0.020, 0.003)), &iron_mat);
            {
                let mut r = scoop_back.rotation();
                r.set(PI / 2.0, PI / 2.0 - arm_angle, 0);
                r.order = EulerOrder::YXZ;
            }
            scoop_back.position().set(tool_cx, arm_y - shaft_len - 0.0055, tool_cz);
            fts.add(&scoop_back);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        } else {
            // Brush (4th tool): shaft + horizontal bristle head.
            let shaft_len = 0.28;
            let shaft = mesh(pg("FireToolSets__shaft", cyl_geo!(0.003, 0.003, shaft_len, 5)), &iron_mat);
            shaft.position().set(tool_cx, arm_y - shaft_len / 2.0 - 0.005, tool_cz);
            fts.add(&shaft);
            let bristle_mat = lambert!({ color: 0x201810 });
            let bristles = mesh(pg("FireToolSets__bristles", cyl_geo!(0.014, 0.012, 0.028, 8)), &bristle_mat);
            bristles.rotation().z = PI / 2.0;
            bristles.position().set(tool_cx, arm_y - shaft_len - 0.010, tool_cz);
            fts.add(&bristles);
            let grip = mesh(pg("FireToolSets__grip", cyl_geo!(0.006, 0.006, 0.030, 6)), &tip_mat);
            grip.position().set(tool_cx, arm_y + 0.010, tool_cz);
            fts.add(&grip);
        }
    }
    // 30% chance: small ash pile beside the stand.
    if rnd() < 0.3 {
        let ash_mat = lambert!({ color: 0x606060 });
        let ash = mesh(circle_geo!(0.030, 8), &ash_mat);
        ash.rotation().x = -PI / 2.0;
        ash.position().set(base_r + 0.030, 0.001, 0);
        ash.rotation().z = rnd() * PI * 2.0;
        fts.add(&ash);
        // Tiny lumps in the ash.
        for _j in 0..3 {
            let lump = mesh(pg("FireToolSets__lump", sphere_geo!(0.004, 5, 4)), lambert!({ color: 0x1a1a1a }));
            let lx = base_r + 0.030 + (rnd() - 0.5) * 0.030;
            let lz = (rnd() - 0.5) * 0.030;
            lump.position().set(lx, 0.0016, lz); // resting in the ash
            lump.scale().y = 0.4;
            fts.add(&lump);
        }
    }
    fts.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    fts.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fts);
}
