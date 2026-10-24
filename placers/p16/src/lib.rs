//! Prop placers ported from backrooms.js (chunk p16; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeFireToolSets", place_fire_tool_sets));
    r.push(("placeBotanicalPrints", place_botanical_prints));
    r.push(("placeFloorKettles", place_floor_kettles));
    r.push(("placeSpecimenDomes", place_specimen_domes));
    r.push(("placeSecretaryDesks", place_secretary_desks));
    r.push(("placeTarnishedMirrors", place_tarnished_mirrors));
    r.push(("placeSheetCoveredFurniture", place_sheet_covered_furniture));
    r.push(("placeServantBellPulls", place_servant_bell_pulls));
    r.push(("placeVictorianHallStands", place_victorian_hall_stands));
}

/// `arr[Math.floor(Math.random() * arr.length)]` index
#[inline]
fn ri(len: usize) -> usize { (rnd() * len as f64).floor() as usize }

/// Every open (tile 0) interior tile, row-major like the JS loops.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// Open tiles next to a wall (or the grid edge), one entry per facing wall:
/// `candidates.push({ x, z, dx, dz })` for each `!inGrid(nx, nz) || tiles[...] === 1`.
fn wall_candidates(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    candidates.push((x, z, dx, dz));
                }
            }
        }
    }
    candidates
}

// ------------------------------------------------------------------ line 30363
// R752 wrought-iron fire tool set on stand (floor): 3% chance per level.
fn place_fire_tool_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
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
        for j in 0..3 {
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

// ------------------------------------------------------------------ line 30534
// R752 framed botanical specimen texture (pressed flower / plant).
game::lazy_static_tex!(botanical_texture_2, canvas_tex(200, 260, |ctx, w, h| {
    // Aged cream paper background.
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(0, 0, w, h);
    // Very faint radial vignette.
    let vig = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.25, w / 2.0, h / 2.0, w * 0.7);
    vig.add_color_stop(0, "rgba(240,232,208,0)");
    vig.add_color_stop(1, "rgba(120,100,60,0.25)");
    ctx.set_fill_style(&vig);
    ctx.fill_rect(0, 0, w, h);
    // Age spots.
    ctx.set_fill_style("rgba(140,100,50,0.18)");
    for _ in 0..8 {
        ctx.begin_path();
        let (ax, ay, ar) = (rnd() * w, rnd() * h, 2.0 + rnd() * 4.0);
        ctx.arc(ax, ay, ar, 0, PI * 2.0);
        ctx.fill();
    }
    // 4-variant scene generator.
    let variant = (rnd() * 4.0).floor() as usize;
    // (name, common, year)
    let specimens: [(&str, &str, i32); 4] = [
        ("Rosa gallica", "Rose", 1873),
        ("Pteridium aquilinum", "Bracken Fern", 1889),
        ("Cirsium vulgare", "Thistle", 1901),
        ("Anthemis arvensis", "Wildflower", 1885),
    ];
    let spec = specimens[variant];
    // Central specimen silhouette.
    ctx.set_stroke_style("rgba(40,55,30,0.85)");
    ctx.set_fill_style("rgba(70,90,40,0.55)");
    ctx.set_line_width(1.0);
    let (cx, cy) = (w / 2.0, h * 0.48);
    if variant == 0 {
        // Rose — main stem + flower cluster + 2 leaves.
        ctx.begin_path();
        ctx.move_to(cx, cy + 60.0);
        ctx.quadratic_curve_to(cx - 4.0, cy + 20.0, cx + 2.0, cy - 10.0);
        ctx.stroke();
        // Petals (5-pointed rose).
        ctx.begin_path();
        for i in 0..5 {
            let pa = (i as f64 / 5.0) * PI * 2.0 - PI / 2.0;
            let px = cx + 2.0 + pa.cos() * 16.0;
            let py = cy - 20.0 + pa.sin() * 16.0;
            if i == 0 { ctx.move_to(px, py); }
            else { ctx.quadratic_curve_to(cx + 2.0 + (pa - 0.3).cos() * 10.0, cy - 20.0 + (pa - 0.3).sin() * 10.0, px, py); }
        }
        ctx.close_path();
        ctx.fill();
        ctx.stroke();
        // Center of flower.
        ctx.set_fill_style("rgba(100,80,20,0.7)");
        ctx.begin_path();
        ctx.arc(cx + 2.0, cy - 20.0, 5, 0, PI * 2.0);
        ctx.fill();
        // 2 pointed leaves along stem.
        ctx.set_fill_style("rgba(70,90,40,0.55)");
        for (lx, ly, lr) in [(cx - 10.0, cy + 15.0, -0.4), (cx + 10.0, cy + 40.0, 0.4)] {
            ctx.save();
            ctx.translate(lx, ly);
            ctx.rotate(lr);
            ctx.begin_path();
            ctx.ellipse(0, 0, 6, 14, 0, 0, PI * 2.0);
            ctx.fill();
            ctx.stroke();
            ctx.restore();
        }
    } else if variant == 1 {
        // Fern — central stem + many small opposing leaflets.
        ctx.begin_path();
        ctx.move_to(cx, cy + 60.0);
        ctx.quadratic_curve_to(cx + 3.0, cy, cx - 2.0, cy - 60.0);
        ctx.stroke();
        for i in 0..12i32 {
            let y = cy + 55.0 - i as f64 * 10.0;
            let leaf_len = 6.0 + 0.max(8 - (i - 6).abs()) as f64 * 2.0;
            for side in [-1.0f64, 1.0] {
                ctx.begin_path();
                ctx.ellipse(cx + side * (leaf_len * 0.6), y, leaf_len * 0.7, 3, side * 0.35, 0, PI * 2.0);
                ctx.fill();
                ctx.stroke();
            }
        }
    } else if variant == 2 {
        // Thistle — stem + spiny bulb + 2 spiky leaves.
        ctx.begin_path();
        ctx.move_to(cx, cy + 60.0);
        ctx.line_to(cx, cy - 15.0);
        ctx.stroke();
        // Bulb (spherical head).
        ctx.begin_path();
        ctx.ellipse(cx, cy - 25.0, 10, 12, 0, 0, PI * 2.0);
        ctx.fill();
        ctx.stroke();
        // Spikes radiating from bulb top (purple bloom).
        ctx.set_stroke_style("rgba(100,50,120,0.8)");
        for i in 0..10 {
            let sa = (i as f64 / 10.0) * PI - PI / 2.0;
            ctx.begin_path();
            ctx.move_to(cx + sa.cos() * 10.0, cy - 25.0 + sa.sin() * 12.0);
            ctx.line_to(cx + sa.cos() * 18.0, cy - 30.0 + sa.sin() * 18.0);
            ctx.stroke();
        }
        ctx.set_stroke_style("rgba(40,55,30,0.85)");
        // Spiky leaves.
        for (lx, ly) in [(cx - 15.0, cy + 15.0), (cx + 15.0, cy + 30.0)] {
            ctx.begin_path();
            ctx.move_to(cx, ly - 5.0);
            ctx.line_to(lx, ly);
            ctx.line_to(cx, ly + 5.0);
            ctx.close_path();
            ctx.fill();
            ctx.stroke();
        }
    } else {
        // Wildflower — main stem + composite flower head + small leaves.
        ctx.begin_path();
        ctx.move_to(cx, cy + 60.0);
        ctx.quadratic_curve_to(cx + 5.0, cy + 10.0, cx, cy - 20.0);
        ctx.stroke();
        // Flower head — 8 white petals around yellow center.
        ctx.set_fill_style("rgba(240,220,180,0.8)");
        for i in 0..8 {
            let pa = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.ellipse(cx + pa.cos() * 8.0, cy - 25.0 + pa.sin() * 8.0, 4, 8, pa, 0, PI * 2.0);
            ctx.fill();
            ctx.stroke();
        }
        ctx.set_fill_style("rgba(180,140,40,0.9)");
        ctx.begin_path();
        ctx.arc(cx, cy - 25.0, 4, 0, PI * 2.0);
        ctx.fill();
        // 3 small leaves along stem.
        ctx.set_fill_style("rgba(70,90,40,0.55)");
        for i in 0..3 {
            let ly = cy - 5.0 + i as f64 * 20.0;
            ctx.begin_path();
            ctx.ellipse(cx + (if i % 2 == 0 { -8.0 } else { 8.0 }), ly, 5, 8, 0, 0, PI * 2.0);
            ctx.fill();
            ctx.stroke();
        }
    }
    // Latin binomial in cursive at bottom (hand-inked look).
    ctx.set_fill_style("#20180c");
    ctx.set_font("italic 12px serif");
    ctx.set_text_align("center");
    ctx.fill_text(spec.0, cx, h - 30.0);
    // Common name + date on separate line.
    ctx.set_font("10px serif");
    ctx.set_fill_style("#403020");
    ctx.fill_text(&format!("{} · {}", spec.1, spec.2), cx, h - 15.0);
    // Small collector's mark (top right corner).
    ctx.set_font("italic 8px serif");
    ctx.set_text_align("right");
    ctx.set_fill_style("#60401e");
    let collectors = ["H.M.", "E.S.", "A.L.", "W.T."];
    ctx.fill_text(&format!("coll: {}", collectors[ri(collectors.len())]), w - 8.0, 18);
}));

// ------------------------------------------------------------------ line 30693
fn place_botanical_prints(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    let frame_colors: [u32; 4] = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())] });
    let spec_mat = lambert!({ map: botanical_texture_2(), side: DoubleSide });
    let bp = group();
    let (bw, bh, bt, bfw) = (0.28, 0.36, 0.010, 0.014);
    for (bar_w, bar_h, bx, by) in [
        (bw + bfw * 2.0, bfw, 0.0, bh / 2.0 + bfw / 2.0),
        (bw + bfw * 2.0, bfw, 0.0, -bh / 2.0 - bfw / 2.0),
        (bfw, bh, -bw / 2.0 - bfw / 2.0, 0.0),
        (bfw, bh, bw / 2.0 + bfw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("BotanicalPrints__bar", box_geo!(bar_w, bar_h, bt)), &frame_mat);
        bar.position().set(bx, by, 0);
        bp.add(&bar);
    }
    // Inner cream mat behind specimen.
    let mat_mat = lambert!({ color: 0xe8d8b0 });
    let in_mat = mesh(plane_geo!(bw - 0.005, bh - 0.005), &mat_mat);
    in_mat.position().set(0, 0, bt / 2.0 - 0.001);
    bp.add(&in_mat);
    let canvas = mesh(plane_geo!(bw - 0.020, bh - 0.020), &spec_mat);
    canvas.position().set(0, 0, bt / 2.0 + 0.001);
    bp.add(&canvas);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.007);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.007);
    let center_y = 1.55;
    if cdx.abs() == 1 {
        bp.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        bp.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        bp.rotation().y = if cdz > 0 { PI } else { 0.0 };
        bp.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    bp.rotation().z = (rnd() - 0.5) * 0.015;
    g.world.add(&bp);
}

// ------------------------------------------------------------------ line 30756
// R753 cast-iron kettle on floor (floor): 4% chance per level.
fn place_floor_kettles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[ri(open.len())];
    // 3-color kettle pool: cast iron black, blackened copper, gunmetal.
    let kettle_colors: [u32; 3] = [0x181818, 0x2a1810, 0x404048];
    let kettle_mat = lambert!({ color: kettle_colors[ri(kettle_colors.len())] });
    // Optional brass knob highlight (40% variant).
    let use_brass_knob = rnd() < 0.4;
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let kt = group();
    // Round body — squat sphere w/ Y-scale.
    let body_r = 0.075;
    let body = mesh(pg("FloorKettles__body", sphere_geo!(body_r, 14, 10)), &kettle_mat);
    body.scale().y = 0.80;
    body.position().y = body_r * 0.80;
    kt.add(&body);
    // Bottom flattening disc.
    let bottom = mesh(pg("FloorKettles__foot", cyl_geo!(body_r * 0.85, body_r * 0.85, 0.008, 14)), &kettle_mat);
    bottom.position().y = 0.004;
    kt.add(&bottom);
    // Spout — angled cone extending from side.
    let spout_angle = rnd() * PI * 2.0;
    let spout = mesh(pg("FloorKettles__spout", cyl_geo!(0.010, 0.018, 0.060, 8)), &kettle_mat);
    spout.rotation().z = PI / 2.0 - 0.5;
    spout.position().set(spout_angle.cos() * (body_r + 0.020), body_r * 0.90, spout_angle.sin() * (body_r + 0.020));
    // Narrow end aims outward along (cos, sin).
    spout.rotation().y = PI - spout_angle;
    kt.add(&spout);
    // Spout tip cap, on the spout axis over its mouth.
    let spout_tip = mesh(pg("FloorKettles__spoutTip", cyl_geo!(0.010, 0.010, 0.006, 8)), &kettle_mat);
    spout_tip.rotation().z = PI / 2.0 - 0.5;
    spout_tip.position().set(
        spout_angle.cos() * (body_r + 0.020 + 0.031 * 0.5f64.cos()),
        body_r * 0.90 + 0.031 * 0.5f64.sin(),
        spout_angle.sin() * (body_r + 0.020 + 0.031 * 0.5f64.cos()),
    );
    spout_tip.rotation().y = PI - spout_angle;
    kt.add(&spout_tip);
    // Arched handle on top — half-torus bail standing upright over the lid.
    let handle = mesh(pg("FloorKettles__handle", torus_geo!(body_r * 0.7, 0.004, 6, 14, PI)), &kettle_mat);
    handle.rotation().y = -spout_angle;
    handle.position().set(0, body_r * 1.35, 0);
    kt.add(&handle);
    // Lid (flattened cylinder).
    let lid = mesh(pg("FloorKettles__lid", cyl_geo!(body_r * 0.42, body_r * 0.42, 0.008, 12)), &kettle_mat);
    lid.position().y = body_r * 1.60 - 0.0005;
    kt.add(&lid);
    // Lid knob — 40% brass.
    let knob = mesh(pg("FloorKettles__knob", sphere_geo!(0.006, 6, 5)), if use_brass_knob { &brass_mat } else { &kettle_mat });
    knob.position().y = body_r * 1.60 + 0.009;
    kt.add(&knob);
    // 60% chance: steam wisps rising from spout.
    if rnd() < 0.6 {
        let steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.18 });
        struct Wisp { mesh: Obj, phase: f64, base_y: f64, base_x: f64, base_z: f64 }
        let mut steam_wisps: Vec<Wisp> = vec![];
        let spout_tip_x = spout_angle.cos() * (body_r + 0.048);
        let spout_tip_z = spout_angle.sin() * (body_r + 0.048);
        for i in 0..3 {
            let wisp = mesh(sphere_geo!(0.012, 6, 5), &steam_mat);
            wisp.scale().y = 2.4;
            let px = spout_tip_x + (rnd() - 0.5) * 0.015;
            let py = body_r * 0.90 + 0.030 + i as f64 * 0.030;
            let pz = spout_tip_z + (rnd() - 0.5) * 0.015;
            wisp.position().set(px, py, pz);
            let phase = rnd() * PI * 2.0;
            let p = wisp.pos();
            steam_wisps.push(Wisp { mesh: wisp.clone(), phase, base_y: p.y, base_x: p.x, base_z: p.z });
            kt.add(&wisp);
        }
        // R755 pressure-breathing cycle: 20-30 s slow oscillation.
        let pressure_cycle = 20.0 + rnd() * 10.0;
        let pressure_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let pressure = 0.95 + ((t * (PI * 2.0) / pressure_cycle) + pressure_phase).sin() * 0.40;
            for w in &steam_wisps {
                {
                    let mut p = w.mesh.position();
                    p.y = w.base_y + (t * 0.7 + w.phase).sin() * 0.015 * pressure;
                    p.x = w.base_x + (t * 0.5 + w.phase).sin() * 0.015;
                    p.z = w.base_z + (t * 0.6 + w.phase * 1.3).cos() * 0.015;
                }
                let s = (0.88 + (t * 0.9 + w.phase).sin() * 0.16) * pressure;
                w.mesh.scale().set(s, 2.4 * s, s);
                // Opacity modulated by pressure (the wisps share one material).
                w.mesh.mat().set_opacity(0.10 + 0.14 * clamp(pressure * 0.7, 0.0, 1.0));
            }
        });
    }
    // 30% chance: small trivet mat underneath.
    if rnd() < 0.3 {
        let trivet_colors: [u32; 3] = [0x604020, 0x2a1810, 0x8b6f47];
        let trivet_mat = lambert!({ color: trivet_colors[ri(trivet_colors.len())] });
        let trivet = mesh(pg("FloorKettles__trivet", cyl_geo!(body_r + 0.020, body_r + 0.020, 0.003, 12)), &trivet_mat);
        trivet.position().y = 0.0015;
        kt.add(&trivet);
    }
    kt.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    kt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kt);
}

// ------------------------------------------------------------------ line 30891
// R753 Victorian glass-dome specimen display (floor): 3% chance per level.
fn place_specimen_domes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[ri(open.len())];
    // Base wood pool.
    let wood_colors: [u32; 4] = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let gold_mat = lambert!({ color: 0xc0a060, emissive: 0x201810 });
    // Very transparent glass for dome.
    let glass_mat = lambert!({ color: 0xd8e0e8, transparent: true, opacity: 0.20 });
    let sd = group();
    // Wooden base — round platform w/ small foot.
    let base_r = 0.075;
    let base = mesh(pg("SpecimenDomes__base", cyl_geo!(base_r, base_r + 0.006, 0.015, 16)), &wood_mat);
    base.position().y = 0.008;
    sd.add(&base);
    // Ornamental base ring (torus).
    let base_ring = mesh(pg("SpecimenDomes__baseRing", torus_geo!(base_r, 0.004, 5, 16)), &gold_mat);
    base_ring.rotation().x = PI / 2.0;
    base_ring.position().y = 0.016;
    sd.add(&base_ring);
    // Small gold plaque on the plinth's sloping face.
    let plaque_mat = gold_mat.clone();
    let plaque = mesh(pg("SpecimenDomes__plaque", box_geo!(0.036, 0.010, 0.002)), &plaque_mat);
    let plaque_cant = 0.4f64.atan();
    plaque.position().set(0, 0.008, base_r + 0.0040);
    plaque.rotation().x = -plaque_cant;
    sd.add(&plaque);
    // Tiny engraving lines on plaque.
    let eng_mat = lambert!({ color: 0x402020 });
    for i in 0..2 {
        let eng_line = mesh(box_geo!(0.024, 0.001, 0.0005), &eng_mat);
        let d = (i as f64 - 0.5) * 0.004;
        eng_line.position().set(0,
            0.008 + d * plaque_cant.cos() + 0.0011 * plaque_cant.sin(),
            base_r + 0.0040 - d * plaque_cant.sin() + 0.0011 * plaque_cant.cos());
        eng_line.rotation().x = -plaque_cant;
        sd.add(&eng_line);
    }
    // Glass hemisphere dome (top-half sphere).
    let dome_r = base_r - 0.008;
    let dome = mesh(sphere_geo!(dome_r, 16, 10, 0, PI * 2.0, 0, PI / 2.0), &glass_mat);
    dome.position().y = 0.016;
    sd.add(&dome);
    // Faint rim outline where dome meets base.
    let dome_rim = mesh(pg("SpecimenDomes__domeRim", torus_geo!(dome_r, 0.002, 4, 20)), &gold_mat);
    dome_rim.rotation().x = PI / 2.0;
    dome_rim.position().y = 0.017;
    sd.add(&dome_rim);
    // 3-variant specimen inside — bird / butterfly / beetle.
    let spec_variant = (rnd() * 3.0).floor() as i32;
    if spec_variant == 0 {
        // Taxidermy bird on perch.
        let bird_colors: [u32; 5] = [0x8b4513, 0x6b8060, 0x502010, 0x605040, 0x804020];
        let bird_color = bird_colors[ri(bird_colors.len())];
        let bird_mat = lambert!({ color: bird_color });
        // Small wooden perch lying on the plinth top, across the bird.
        let perch = mesh(pg("SpecimenDomes__perch", cyl_geo!(0.003, 0.003, 0.040, 5)), &wood_mat);
        perch.rotation().x = PI / 2.0;
        perch.position().set(0.005, 0.0185, 0);
        sd.add(&perch);
        // Bird body, long along X like the head-tail line.
        let bird_body = mesh(pg("SpecimenDomes__birdBody", sphere_geo!(0.014, 10, 7)), &bird_mat);
        bird_body.scale().set(1.3, 0.85, 1.0);
        bird_body.position().y = 0.0325;
        sd.add(&bird_body);
        // Head (smaller sphere).
        let bird_head = mesh(pg("SpecimenDomes__birdHead", sphere_geo!(0.009, 8, 6)), &bird_mat);
        bird_head.position().set(0.015, 0.0415, 0);
        sd.add(&bird_head);
        // Tiny beak (cone), base 1.5 mm into the face.
        let beak = mesh(pg("SpecimenDomes__beak", cone_geo!(0.002, 0.006, 5)), lambert!({ color: 0xa07030 }));
        beak.rotation().z = -PI / 2.0;
        beak.position().set(0.0255, 0.0415, 0);
        sd.add(&beak);
        // Tail (small flat cone) fanning out flat, root in the rump.
        let tail = mesh(pg("SpecimenDomes__tail", cone_geo!(0.007, 0.016, 4)), &bird_mat);
        tail.scale().z = 0.4;
        tail.rotation().x = -PI / 2.0;
        tail.rotation().z = PI / 2.0;
        tail.position().set(-0.023, 0.0305, 0);
        sd.add(&tail);
        // 2 wings.
        for side in [-1.0f64, 1.0] {
            let wing = mesh(pg("SpecimenDomes__wing", sphere_geo!(0.010, 8, 5)), &bird_mat);
            wing.scale().set(0.4, 0.5, 1.3);
            wing.rotation().y = PI / 2.0;
            wing.position().set(-0.002, 0.0345, side * 0.0125);
            sd.add(&wing);
        }
        // 2 legs, standing on the perch.
        for side in [-1.0f64, 1.0] {
            let leg = mesh(cyl_geo!(0.0008, 0.0008, 0.008, 3), lambert!({ color: 0xa07030 }));
            leg.position().set(0.005, 0.0235, side * 0.005);
            sd.add(&leg);
        }
        // Tiny eye dot.
        let eye = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 4, 4)), lambert!({ color: 0x080810 }));
        eye.position().set(0.020, 0.0435, 0.006);
        sd.add(&eye);
    } else if spec_variant == 1 {
        // Butterfly on pin.
        let wing_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let wing_color = wing_colors[ri(wing_colors.len())];
        let wing_mat = lambert!({ color: wing_color, side: DoubleSide });
        let body_mat = lambert!({ color: 0x181818 });
        // Vertical pin (silver needle).
        let pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0005, 0.0005, 0.030, 4)), lambert!({ color: 0xa0a0a8 }));
        pin.position().y = 0.030;
        sd.add(&pin);
        // Pin head.
        let pin_head = mesh(pg("SpecimenDomes__bead", sphere_geo!(0.0015, 5, 4)), lambert!({ color: 0xa0a0a8 }));
        pin_head.position().y = 0.046;
        sd.add(&pin_head);
        // Central body.
        let bf_body = mesh(pg("SpecimenDomes__bfBody", cyl_geo!(0.001, 0.001, 0.016, 4)), &body_mat);
        bf_body.rotation().z = PI / 2.0;
        bf_body.position().y = 0.036;
        sd.add(&bf_body);
        // Head.
        let bf_head = mesh(pg("SpecimenDomes__bfHead", sphere_geo!(0.002, 4, 4)), &body_mat);
        bf_head.position().set(0.010, 0.036, 0);
        sd.add(&bf_head);
        // 4 wing plates (2 upper, 2 lower, symmetric).
        for side in [-1.0f64, 1.0] {
            let upper_wing = mesh(circle_geo!(0.014, 5), &wing_mat);
            upper_wing.scale().set(1.0, 0.7, 1.0);
            upper_wing.rotation().x = -PI / 2.0;
            upper_wing.rotation().z = side * 0.3;
            upper_wing.position().set(0, 0.036, side * 0.014);
            sd.add(&upper_wing);
            let lower_wing = mesh(circle_geo!(0.010, 5), &wing_mat);
            lower_wing.scale().set(1.0, 0.5, 1.0);
            lower_wing.rotation().x = -PI / 2.0;
            lower_wing.rotation().z = side * -0.2;
            lower_wing.position().set(-0.003, 0.036, side * 0.012);
            sd.add(&lower_wing);
            let marking = mesh(circle_geo!(0.003, 5), lambert!({ color: 0x181818, side: DoubleSide }));
            marking.rotation().x = -PI / 2.0;
            marking.position().set(0, 0.0362, side * 0.014);
            sd.add(&marking);
        }
        // 2 antennae.
        for side in [-1.0f64, 1.0] {
            let ant = mesh(pg("SpecimenDomes__antenna", cyl_geo!(0.0004, 0.0004, 0.010, 3)), &body_mat);
            ant.position().set(0.014, 0.038, side * 0.002);
            ant.rotation().z = -0.6;
            sd.add(&ant);
        }
    } else {
        // Beetle display — 3-4 beetles pinned in a row.
        let beetle_count = 3 + (rnd() * 2.0).floor() as i32;
        let beetle_colors: [u32; 4] = [0x1a1a08, 0x2a3010, 0x502010, 0x102030];
        for i in 0..beetle_count {
            let beetle_color = beetle_colors[ri(beetle_colors.len())];
            let beetle_mat = lambert!({ color: beetle_color, emissive: 0x080810 });
            let beetle_group = group();
            // Oval body (squashed sphere).
            let beetle = mesh(pg("SpecimenDomes__beetle", sphere_geo!(0.006, 8, 6)), &beetle_mat);
            beetle.scale().set(1.0, 0.4, 1.6);
            beetle.position().y = 0.022;
            beetle_group.add(&beetle);
            // Small head bump.
            let b_head = mesh(pg("SpecimenDomes__beetleHead", sphere_geo!(0.003, 5, 4)), &beetle_mat);
            b_head.position().set(0, 0.024, 0.009);
            beetle_group.add(&b_head);
            // 6 tiny legs (3 per side).
            for side in [-1.0f64, 1.0] {
                for j in 0..3 {
                    let leg = mesh(cyl_geo!(0.0004, 0.0004, 0.006, 3), &beetle_mat);
                    leg.rotation().z = side * 0.5;
                    leg.position().set(side * 0.006, 0.021, -0.004 + j as f64 * 0.004);
                    beetle_group.add(&leg);
                }
            }
            // Tiny elytra split line down back.
            let split = mesh(box_geo!(0.0004, 0.001, 0.010), lambert!({ color: 0x080810 }));
            split.position().set(0, 0.024, 0);
            beetle_group.add(&split);
            // Pin from the plinth top up through the beetle.
            let b_pin = mesh(pg("SpecimenDomes__pin", cyl_geo!(0.0004, 0.0004, 0.014, 3)), lambert!({ color: 0xa0a0a8 }));
            b_pin.position().set(0, 0.0225, 0);
            beetle_group.add(&b_pin);
            // Position beetles in a row.
            beetle_group.position().x = (i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020;
            sd.add(&beetle_group);
        }
        // Small paper label in front of each beetle.
        for i in 0..beetle_count {
            let label = mesh(pg("SpecimenDomes__label", box_geo!(0.010, 0.0008, 0.004)), lambert!({ color: 0xf0e8c0 }));
            label.position().set((i as f64 - (beetle_count - 1) as f64 / 2.0) * 0.020, 0.0159, -0.020);
            sd.add(&label);
        }
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}

// ------------------------------------------------------------------ line 31124
// R754 vertical secretary desk with drop-front writing surface (floor): 2%.
fn place_secretary_desks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[ri(open.len())];
    // 4-color wood pool.
    let wood_colors: [u32; 4] = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let sec = group();
    // Main dimensions.
    let (body_w, body_d, body_h) = (0.55, 0.35, 0.60);
    let leg_h = 0.20;
    // Cabinet body.
    let cabinet = mesh(pg("SecretaryDesks__cabinet", box_geo!(body_w, body_h, body_d)), &wood_mat);
    cabinet.position().y = leg_h + body_h / 2.0;
    sec.add(&cabinet);
    // 4 tapered legs at corners.
    for cx in [-body_w * 0.42, body_w * 0.42] {
        for cz in [-body_d * 0.42, body_d * 0.42] {
            let leg = mesh(pg("SecretaryDesks__leg", cyl_geo!(0.014, 0.020, leg_h, 6)), &wood_mat);
            leg.position().set(cx, leg_h / 2.0, cz);
            sec.add(&leg);
        }
    }
    // 2 drawers on cabinet front + brass drawer pulls.
    let drawer_y0 = leg_h + body_h * 0.14;
    let drawer_h = body_h * 0.28;
    for i in 0..2 {
        let d_y = drawer_y0 + i as f64 * (drawer_h + 0.008);
        let drawer = mesh(pg("SecretaryDesks__drawer", box_geo!(body_w * 0.95, drawer_h, 0.005)), lambert!({ color: 0x4a3020 }));
        drawer.position().set(0, d_y, body_d / 2.0 + 0.003);
        sec.add(&drawer);
        for pside in [-1.0f64, 1.0] {
            let pull = mesh(pg("SecretaryDesks__pull", cyl_geo!(0.008, 0.008, 0.006, 8)), &brass_mat);
            pull.rotation().x = PI / 2.0;
            pull.position().set(pside * body_w * 0.20, d_y, body_d / 2.0 + 0.008);
            sec.add(&pull);
            let backplate = mesh(pg("SecretaryDesks__backplate", box_geo!(0.020, 0.010, 0.001)), &brass_mat);
            backplate.position().set(pside * body_w * 0.20, d_y, body_d / 2.0 + 0.006);
            sec.add(&backplate);
        }
    }
    // Top compartment — drop-front writing surface (currently open).
    let top_y = leg_h + body_h;
    let drop_w = body_w * 0.85;
    let drop_h = 0.30;
    // The group sits on the hinge line; local +Z runs out along the open fall.
    let drop_surf = group();
    drop_surf.position().set(0, top_y, body_d / 2.0);
    drop_surf.rotation().x = 0.05; // Almost fully open
    let drop_panel = mesh(pg("SecretaryDesks__dropPanel", box_geo!(drop_w, 0.010, drop_h)), &wood_mat);
    drop_panel.position().set(0, -0.005, drop_h / 2.0);
    drop_surf.add(&drop_panel);
    // Small brass hinges at the joint.
    for hside in [-1.0f64, 1.0] {
        let hinge = mesh(pg("SecretaryDesks__hinge", box_geo!(0.012, 0.012, 0.002)), &brass_mat);
        hinge.position().set(hside * drop_w * 0.30, -0.012, 0.001);
        drop_surf.add(&hinge);
    }
    sec.add(&drop_surf);
    // Support stays holding the drop-surface open.
    for cside in [-1.0f64, 1.0] {
        let arm = mesh(pg("SecretaryDesks__stay", cyl_geo!(0.001, 0.001, 0.14, 3)), &brass_mat);
        arm.position().set(cside * drop_w * 0.45, top_y - 0.066, body_d / 2.0 + 0.045);
        arm.rotation().x = 0.7;
        sec.add(&arm);
    }
    // Upper compartment (behind the dropped panel).
    let upper_comp_h = 0.30;
    let upper_comp = mesh(pg("SecretaryDesks__upper", box_geo!(body_w * 0.85, upper_comp_h, body_d * 0.75)), &wood_mat);
    upper_comp.position().set(0, top_y + upper_comp_h / 2.0, -body_d * 0.05);
    sec.add(&upper_comp);
    // Pigeonhole cubbies.
    let hole_count = 6;
    let hole_w = body_w * 0.85 / hole_count as f64 * 0.85;
    let hole_h = upper_comp_h * 0.55;
    for i in 0..hole_count {
        let hole_x = -body_w * 0.85 / 2.0 + (i as f64 + 0.5) * (body_w * 0.85 / hole_count as f64);
        let hole_back = mesh(box_geo!(hole_w, hole_h, 0.001), lambert!({ color: 0x0a0a0a }));
        hole_back.position().set(hole_x, top_y + upper_comp_h * 0.55, body_d * 0.32);
        sec.add(&hole_back);
        // 30% chance small rolled paper visible in cubby.
        if rnd() < 0.3 {
            let paper_mat = lambert!({ color: 0xf0e8d0 });
            let roll = mesh(pg("SecretaryDesks__roll", cyl_geo!(0.004, 0.004, hole_w * 0.7, 6)), &paper_mat);
            roll.rotation().z = PI / 2.0;
            roll.position().set(hole_x, top_y + upper_comp_h * 0.4, body_d * 0.32);
            sec.add(&roll);
        }
    }
    // 4 small drawers above pigeonholes.
    let small_drawer_y = top_y + upper_comp_h * 0.10;
    for i in 0..4 {
        let sd_x = -body_w * 0.85 / 2.0 + (i as f64 + 0.5) * (body_w * 0.85 / 4.0);
        let sdrawer = mesh(pg("SecretaryDesks__sdrawer", box_geo!(body_w * 0.85 / 4.0 * 0.85, 0.05, 0.005)), lambert!({ color: 0x4a3020 }));
        sdrawer.position().set(sd_x, small_drawer_y, body_d * 0.32 + 0.003);
        sec.add(&sdrawer);
        let knob = mesh(pg("SecretaryDesks__knob", sphere_geo!(0.003, 5, 4)), &brass_mat);
        knob.position().set(sd_x, small_drawer_y, body_d * 0.32 + 0.008);
        sec.add(&knob);
    }
    // 60% chance: papers on the dropped writing surface (children of dropSurf).
    if rnd() < 0.6 {
        let paper_mat = lambert!({ color: 0xf0e8d0 });
        let paper_count = 1 + (rnd() * 3.0).floor() as i32;
        for i in 0..paper_count {
            let paper = mesh(pg("SecretaryDesks__paper", box_geo!(0.10, 0.001, 0.14)), &paper_mat);
            paper.position().set((rnd() - 0.5) * drop_w * 0.5, 0.0005 + i as f64 * 0.001, drop_h * 0.5);
            paper.rotation().y = (rnd() - 0.5) * 0.3;
            drop_surf.add(&paper);
        }
        // 40% chance: small inkwell on surface.
        if rnd() < 0.4 {
            let ink_glass_mat = lambert!({ color: 0x505860, transparent: true, opacity: 0.55 });
            let inkwell_jar = mesh(cyl_geo!(0.012, 0.012, 0.014, 8), &ink_glass_mat);
            inkwell_jar.position().set(drop_w * 0.35, 0.007, drop_h * 0.3);
            drop_surf.add(&inkwell_jar);
            let ink_top = mesh(circle_geo!(0.010, 8), lambert!({ color: 0x080810 }));
            ink_top.rotation().x = -PI / 2.0;
            ink_top.position().set(drop_w * 0.35, 0.0125, drop_h * 0.3);
            drop_surf.add(&ink_top);
        }
    }
    // Top ornamental cornice.
    let cornice = mesh(pg("SecretaryDesks__cornice", box_geo!(body_w + 0.020, 0.020, body_d * 0.85 + 0.020)), &wood_mat);
    cornice.position().y = top_y + upper_comp_h + 0.010;
    sec.add(&cornice);
    sec.position().set(g.wx(x) + (rnd() - 0.5) * 0.6, 0, g.wz(z) + (rnd() - 0.5) * 0.6);
    sec.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sec);
}

// ------------------------------------------------------------------ line 31297
// R754 tarnished gilded ornate wall mirror texture.
game::lazy_static_tex!(tarnished_mirror_texture, canvas_tex(256, 320, |ctx, w, h| {
    // Base mirror gradient — dark silver reflection tone.
    let grad = ctx.create_linear_gradient(0, 0, w, h);
    grad.add_color_stop(0, "#3a3a45");
    grad.add_color_stop(0.5, "#5a5a68");
    grad.add_color_stop(1, "#2e2e38");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Tarnish patches — dark green/brown blotches at corners + edges.
    ctx.set_fill_style("rgba(30,40,20,0.45)");
    for _ in 0..6 {
        let tx = if rnd() < 0.6 { if rnd() < 0.5 { 0.0 } else { w } } else { rnd() * w };
        let ty = if rnd() < 0.6 { if rnd() < 0.5 { 0.0 } else { h } } else { rnd() * h };
        ctx.begin_path();
        let rx = 25.0 + rnd() * 30.0;
        let ry = 20.0 + rnd() * 25.0;
        let rot = rnd() * PI;
        ctx.ellipse(tx, ty, rx, ry, rot, 0, PI * 2.0);
        ctx.fill();
    }
    // Rust-red small spots scattered.
    ctx.set_fill_style("rgba(100,50,20,0.4)");
    for _ in 0..20 {
        ctx.begin_path();
        let (ax, ay, ar) = (rnd() * w, rnd() * h, 2.0 + rnd() * 4.0);
        ctx.arc(ax, ay, ar, 0, PI * 2.0);
        ctx.fill();
    }
    // 3-4 crack lines emanating from a starburst point.
    let ox = w * (0.3 + rnd() * 0.4);
    let oy = h * (0.3 + rnd() * 0.4);
    ctx.set_stroke_style("rgba(15,15,20,0.85)");
    ctx.set_line_width(1.5);
    let crack_count = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..crack_count {
        let angle = (i as f64 / crack_count as f64) * PI * 2.0 + rnd() * 0.5;
        // Zigzag crack line — up to 5 segments with random angle drift.
        let (mut cx, mut cy) = (ox, oy);
        let mut ang = angle;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        for _ in 0..5 {
            let seg_len = 25.0 + rnd() * 30.0;
            ang += (rnd() - 0.5) * 0.6;
            cx += ang.cos() * seg_len;
            cy += ang.sin() * seg_len;
            ctx.line_to(cx, cy);
            if cx < 0.0 || cx > w || cy < 0.0 || cy > h { break; }
        }
        ctx.stroke();
        // Secondary hairline branch (30% per crack).
        if rnd() < 0.3 {
            ctx.set_stroke_style("rgba(20,20,30,0.5)");
            ctx.set_line_width(0.7);
            ctx.begin_path();
            let mid_x = ox + angle.cos() * 40.0;
            let mid_y = oy + angle.sin() * 40.0;
            ctx.move_to(mid_x, mid_y);
            ctx.line_to(mid_x + (angle + 0.8).cos() * 30.0, mid_y + (angle + 0.8).sin() * 30.0);
            ctx.stroke();
            ctx.set_stroke_style("rgba(15,15,20,0.85)");
            ctx.set_line_width(1.5);
        }
    }
    // Small highlight dot at crack origin (impact point).
    ctx.set_fill_style("rgba(240,240,240,0.65)");
    ctx.begin_path();
    ctx.arc(ox, oy, 3, 0, PI * 2.0);
    ctx.fill();
    // Faint glass-warp reflection lines (2 subtle diagonals).
    ctx.set_stroke_style("rgba(200,210,220,0.10)");
    ctx.set_line_width(30);
    ctx.begin_path();
    ctx.move_to(0, h * 0.3); ctx.line_to(w, h * 0.2);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(0, h * 0.7); ctx.line_to(w, h * 0.8);
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 31371
fn place_tarnished_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // Ornate gilded frame — gold w/ tarnish tinge.
    let frame_colors: [u32; 4] = [0xa08040, 0x8b6f30, 0x907040, 0x604828];
    let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())], emissive: 0x1a1408 });
    let mirror_mat = lambert!({ map: tarnished_mirror_texture(), side: DoubleSide });
    let tm = group();
    // Frame outer bars.
    let (mw, mh, mt, mfw) = (0.40, 0.52, 0.020, 0.036);
    for (bar_w, bar_h, bx, by) in [
        (mw + mfw * 2.0, mfw, 0.0, mh / 2.0 + mfw / 2.0),
        (mw + mfw * 2.0, mfw, 0.0, -mh / 2.0 - mfw / 2.0),
        (mfw, mh, -mw / 2.0 - mfw / 2.0, 0.0),
        (mfw, mh, mw / 2.0 + mfw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("TarnishedMirrors__bar", box_geo!(bar_w, bar_h, mt)), &frame_mat);
        bar.position().set(bx, by, 0);
        tm.add(&bar);
    }
    // Ornate corner decorations.
    for (cx_or, cy_or) in [
        (-mw / 2.0 - mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (-mw / 2.0 - mfw / 2.0, -mh / 2.0 - mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, -mh / 2.0 - mfw / 2.0),
    ] {
        let corner_sphere = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.020, 8, 6)), &frame_mat);
        corner_sphere.position().set(cx_or, cy_or, mt / 2.0);
        tm.add(&corner_sphere);
    }
    // Ornate top pediment.
    let pediment = mesh(pg("TarnishedMirrors__pediment", box_geo!(mw * 0.6, 0.025, mt * 0.7)), &frame_mat);
    pediment.position().set(0, mh / 2.0 + mfw + 0.012, 0);
    tm.add(&pediment);
    // Central decorative rosette on pediment.
    let rosette = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.014, 8, 6)), &frame_mat);
    rosette.position().set(0, mh / 2.0 + mfw + 0.025, mt / 2.0);
    tm.add(&rosette);
    // Bottom pendant.
    let pendant = mesh(pg("TarnishedMirrors__pendant", sphere_geo!(0.012, 6, 5)), &frame_mat);
    pendant.position().set(0, -mh / 2.0 - mfw - 0.010, mt / 2.0);
    tm.add(&pendant);
    // Mirror surface w/ tarnish + crack texture.
    let surface = mesh(plane_geo!(mw, mh), &mirror_mat);
    surface.position().set(0, 0, mt / 2.0 + 0.001);
    tm.add(&surface);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.014);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.014);
    let center_y = 1.65;
    if cdx.abs() == 1 {
        tm.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tm.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        tm.rotation().y = if cdz > 0 { PI } else { 0.0 };
        tm.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    tm.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&tm);
}

// ------------------------------------------------------------------ line 31455
// R756 dust-sheet-covered furniture (floor): 3.5% chance per level.
fn place_sheet_covered_furniture(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.965 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[ri(open.len())];
    // Sheet color pool — aged whites.
    let sheet_colors: [u32; 4] = [0xe8e0d0, 0xd8d0c0, 0xf0e8d8, 0xc8c0b0];
    let sheet_mat = lambert!({ color: sheet_colors[ri(sheet_colors.len())] });
    let scf = group();
    let variant = (rnd() * 3.0).floor() as i32;
    if variant == 0 {
        // Chair-covered shape.
        let (body_w, body_d, body_h) = (0.48, 0.52, 0.85);
        let body = mesh(pg("SheetCoveredFurniture__chairBody", box_geo!(body_w, body_h, body_d)), &sheet_mat);
        body.position().y = body_h / 2.0;
        scf.add(&body);
        // Slight bulge for back rest.
        let back_bulge = mesh(box_geo!(body_w * 0.85, body_h * 0.60, 0.04), &sheet_mat);
        back_bulge.position().set(0, body_h * 0.55, -body_d * 0.5 - 0.012);
        scf.add(&back_bulge);
        // Seat corner bulges (2 armrest hints).
        for side in [-1.0f64, 1.0] {
            let arm = mesh(pg("SheetCoveredFurniture__arm", box_geo!(0.05, body_h * 0.35, body_d * 0.7)), &sheet_mat);
            arm.position().set(side * (body_w * 0.5 + 0.004), body_h * 0.35, 0);
            scf.add(&arm);
        }
        // Hanging sheet fold at bottom (skirt fringe).
        let skirt = mesh(pg("SheetCoveredFurniture__skirt", box_geo!(body_w + 0.05, 0.15, body_d + 0.05)), &sheet_mat);
        skirt.position().y = 0.075;
        scf.add(&skirt);
        // Draped fold ridges on top.
        for i in 0..3 {
            let fold = mesh(pg("SheetCoveredFurniture__fold", box_geo!(body_w * 0.30, 0.020, body_d * 0.20)), &sheet_mat);
            let fx = (rnd() - 0.5) * body_w * 0.7;
            let fz = (rnd() - 0.5) * body_d * 0.7;
            fold.position().set(fx, body_h + 0.005, fz);
            fold.rotation().y = rnd() * PI;
            scf.add(&fold);
        }
    } else if variant == 1 {
        // Table-covered shape.
        let (body_w, body_d, body_h) = (1.10, 0.70, 0.75);
        let top = mesh(pg("SheetCoveredFurniture__tableTop", box_geo!(body_w, 0.06, body_d)), &sheet_mat);
        top.position().y = body_h + 0.03;
        scf.add(&top);
        let skirt_h = 0.75;
        // 4 side panels of skirt (front + back + right + left).
        for (pw, pd, px, pz) in [
            (body_w + 0.05, 0.01, 0.0, body_d / 2.0 + 0.005),
            (body_w + 0.05, 0.01, 0.0, -body_d / 2.0 - 0.005),
            (0.01, body_d + 0.05, body_w / 2.0 + 0.005, 0.0),
            (0.01, body_d + 0.05, -body_w / 2.0 - 0.005, 0.0),
        ] {
            let skirt = mesh(box_geo!(pw, skirt_h, pd), &sheet_mat);
            skirt.position().set(px, body_h - skirt_h / 2.0 + 0.03, pz);
            scf.add(&skirt);
        }
        // Leg-shape bumps on the skirt's corner lines.
        for cx in [-body_w / 2.0 - 0.005, body_w / 2.0 + 0.005] {
            for cz in [-body_d / 2.0 - 0.005, body_d / 2.0 + 0.005] {
                let leg_bump = mesh(pg("SheetCoveredFurniture__legBump", box_geo!(0.05, skirt_h * 0.9, 0.05)), &sheet_mat);
                leg_bump.position().set(cx, skirt_h * 0.45, cz);
                scf.add(&leg_bump);
            }
        }
        // Central fold ridge on top.
        let center_fold = mesh(pg("SheetCoveredFurniture__centerFold", box_geo!(body_w * 0.7, 0.015, 0.04)), &sheet_mat);
        center_fold.position().set((rnd() - 0.5) * 0.1, body_h + 0.066, 0);
        scf.add(&center_fold);
        // Slight book/vase silhouette bulge on top (30%).
        if rnd() < 0.3 {
            let bump = mesh(pg("SheetCoveredFurniture__bump", sphere_geo!(0.08, 6, 5)), &sheet_mat);
            bump.scale().set(1.5, 0.5, 0.9);
            let bx = (rnd() - 0.5) * body_w * 0.6;
            let bz = (rnd() - 0.5) * body_d * 0.5;
            bump.position().set(bx, body_h + 0.10, bz);
            scf.add(&bump);
        }
    } else {
        // Tall wingback-armchair-covered shape.
        let (body_w, body_d, body_h) = (0.55, 0.50, 1.15);
        let body = mesh(pg("SheetCoveredFurniture__wingBody", box_geo!(body_w, body_h, body_d)), &sheet_mat);
        body.position().y = body_h / 2.0;
        scf.add(&body);
        // 2 wing bulges at top corners.
        for side in [-1.0f64, 1.0] {
            let wing = mesh(pg("SheetCoveredFurniture__wing", box_geo!(0.11, body_h * 0.4, 0.30)), &sheet_mat);
            wing.position().set(side * body_w * 0.42, body_h * 0.72, -body_d * 0.30);
            scf.add(&wing);
        }
        // Base skirt drape.
        let skirt = mesh(pg("SheetCoveredFurniture__skirt", box_geo!(body_w + 0.06, 0.20, body_d + 0.06)), &sheet_mat);
        skirt.position().y = 0.10;
        scf.add(&skirt);
        // Draped fold ridges on top.
        for i in 0..4 {
            let fold = mesh(pg("SheetCoveredFurniture__fold", box_geo!(0.15, 0.015, 0.10)), &sheet_mat);
            let fx = (rnd() - 0.5) * body_w * 0.7;
            let fz = (rnd() - 0.5) * body_d * 0.5;
            fold.position().set(fx, body_h + 0.005, fz);
            fold.rotation().y = rnd() * PI;
            scf.add(&fold);
        }
    }
    // 40% chance: thin dust layer on floor around base.
    if rnd() < 0.4 {
        let dust_mat = lambert!({ color: 0xa89880, transparent: true, opacity: 0.35 });
        let dust = mesh(circle_geo!(0.55, 12), &dust_mat);
        dust.rotation().x = -PI / 2.0;
        dust.position().y = 0.001;
        scf.add(&dust);
    }
    scf.position().set(g.wx(x) + (rnd() - 0.5) * 0.8, 0, g.wz(z) + (rnd() - 0.5) * 0.8);
    scf.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scf);
    // R757 subtle vertical breath animator on the top-most drape pieces.
    let midpoint_y = if variant == 1 { 0.85 } else { 1.1 };
    let top_drape_pieces: Vec<Obj> = scf.children().into_iter().filter(|c| {
        // Skip the floor dust circle.
        if let Some(geo) = c.try_geo() { if geo.params().type_name() == "CircleGeometry" { return false; } }
        c.pos().y > midpoint_y
    }).collect();
    if !top_drape_pieces.is_empty() {
        let breath_freq = (PI * 2.0) / (30.0 + rnd() * 15.0);
        let breath_phase = rnd() * PI * 2.0;
        let base_ys: Vec<f64> = top_drape_pieces.iter().map(|p| p.pos().y).collect();
        g.animate(move |_g, _dt, t| {
            let breath = (t * breath_freq + breath_phase).sin() * 0.0025;
            for i in 0..top_drape_pieces.len() {
                top_drape_pieces[i].position().y = base_ys[i] + breath;
            }
        });
    }
}

// ------------------------------------------------------------------ line 31620
// R756 Victorian servant's bell pull system (wall+ceiling): 2.5% chance per level.
fn place_servant_bell_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.bell_pull_positions.clear(); // R757 reset per level
    if rnd() < 0.975 { return; }
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
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
    // R756 subtle sway animator on the cord + tassel + strands + braids.
    let sway_group = group(); // unused in the JS too
    let sway_freq = 0.28 + rnd() * 0.20;
    let sway_phase = rnd() * PI * 2.0;
    let sway_center_y = cord_bot_y - 0.020;
    let mut swayers: Vec<Obj> = vec![cord.clone(), bulb.clone(), bulb_band.clone()];
    // Also all 8 strands (last 8 added to bp).
    let kids = bp.children();
    let strands_start = kids.len() - 8;
    for i in strands_start..kids.len() { swayers.push(kids[i].clone()); }
    // Also the 3 braid rings.
    for b in &braids { swayers.push(b.clone()); }
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
    g.pp.bell_pull_positions.push(v3(p.x, 0, p.z)); // R757
}

// ------------------------------------------------------------------ line 31774
// R758 Victorian floor hall stand (floor): 2.5% chance per level.
fn place_victorian_hall_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[ri(open.len())];
    // 4-color wood pool.
    let wood_colors: [u32; 4] = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: wood_colors[ri(wood_colors.len())] });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let hs = group();
    // Base tray for umbrellas.
    let tray_r = 0.16;
    let tray = mesh(pg("VictorianHallStands__tray", cyl_geo!(tray_r, tray_r, 0.020, 14)), &wood_mat);
    tray.position().y = 0.010;
    hs.add(&tray);
    // Tray rim (raised).
    let tray_rim = mesh(pg("VictorianHallStands__rim", torus_geo!(tray_r, 0.006, 5, 14)), &wood_mat);
    tray_rim.rotation().x = PI / 2.0;
    tray_rim.position().y = 0.020;
    hs.add(&tray_rim);
    // Central vertical post — 180 cm tall.
    let post_h = 1.80;
    let post = mesh(pg("VictorianHallStands__post", cyl_geo!(0.020, 0.030, post_h, 8)), &wood_mat);
    post.position().y = 0.020 + post_h / 2.0;
    hs.add(&post);
    // Ornamental turned rings on post at 3 heights.
    for ry in [0.35, 0.65, 1.10] {
        let ring = mesh(pg("VictorianHallStands__ring", torus_geo!(0.024, 0.005, 5, 12)), &wood_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = 0.020 + ry;
        hs.add(&ring);
    }
    // 6 hooks around post at top (3 upper + 3 middle).
    let hook_ys = [post_h - 0.020, post_h - 0.180];
    for level in 0..2 {
        for i in 0..3 {
            let angle = (i as f64 / 3.0) * PI * 2.0 + level as f64 * (PI / 3.0);
            let arm = mesh(pg("VictorianHallStands__arm", cyl_geo!(0.004, 0.004, 0.055, 4)), &wood_mat);
            arm.rotation().z = PI / 2.0;
            arm.rotation().y = -angle;
            arm.position().set(angle.cos() * 0.030, 0.020 + hook_ys[level], angle.sin() * 0.030);
            hs.add(&arm);
            // Hook cap — small brass ball.
            let cap = mesh(pg("VictorianHallStands__cap", sphere_geo!(0.008, 6, 5)), &brass_mat);
            cap.position().set(angle.cos() * 0.058, 0.020 + hook_ys[level] - 0.006, angle.sin() * 0.058);
            hs.add(&cap);
        }
    }
    // Central top ornamental finial.
    let finial = mesh(pg("VictorianHallStands__finial", sphere_geo!(0.024, 8, 6)), &wood_mat);
    finial.position().y = 0.020 + post_h + 0.015;
    hs.add(&finial);
    // Small oval mirror at mid-height, hanging flat against the post.
    let mirror_angle = rnd() * PI * 2.0;
    let mirror = group();
    let mirror_r = 0.09;
    let m_frame = mesh(pg("VictorianHallStands__mirror", torus_geo!(mirror_r, 0.008, 5, 16)), &wood_mat);
    mirror.add(&m_frame);
    let m_surf = mesh(circle_geo!(mirror_r - 0.005, 16), lambert!({ color: 0x707078, emissive: 0x101014 }));
    mirror.add(&m_surf);
    m_frame.scale().y = 1.30; // Slight oval elongation.
    m_surf.scale().y = 1.30;
    mirror.position().set(mirror_angle.cos() * 0.033, 0.85, mirror_angle.sin() * 0.033);
    mirror.rotation().y = -mirror_angle + PI / 2.0;
    hs.add(&mirror);
    // 2-3 umbrellas in tray base (60% chance).
    if rnd() < 0.6 {
        let umb_count = 2 + (rnd() * 2.0).floor() as i32;
        let umb_colors: [u32; 5] = [0x181818, 0x2a1010, 0x102040, 0x203010, 0x604020];
        for i in 0..umb_count {
            let umb_mat = lambert!({ color: umb_colors[ri(umb_colors.len())] });
            // Umbrella shaft (leaning outward).
            let u_angle = (i as f64 / umb_count as f64) * PI * 2.0 + rnd() * 0.5;
            let shaft = mesh(pg("VictorianHallStands__umbrella", cyl_geo!(0.004, 0.004, 0.75, 4)), &umb_mat);
            shaft.position().set(u_angle.cos() * tray_r * 0.5, 0.35, u_angle.sin() * tray_r * 0.5);
            shaft.rotation().x = u_angle.sin() * 0.12;
            shaft.rotation().z = -u_angle.cos() * 0.12;
            hs.add(&shaft);
            // J-hook handle at top: upright half-ring whose stick end sits on the shaft top.
            let handle_hook = mesh(pg("VictorianHallStands__crook", torus_geo!(0.018, 0.004, 4, 8, PI)), &umb_mat);
            let mut shaft_top = v3(0, 0.375, 0);
            shaft_top.apply_euler(&shaft.rot());
            shaft_top.add(&shaft.pos());
            handle_hook.rotation().y = PI - u_angle;
            handle_hook.position().set(
                shaft_top.x + u_angle.cos() * 0.018,
                shaft_top.y - 0.002,
                shaft_top.z + u_angle.sin() * 0.018,
            );
            hs.add(&handle_hook);
        }
    }
    // 50% chance: 1-2 hats on upper hooks.
    if rnd() < 0.5 {
        let hat_count = 1 + (rnd() * 2.0).floor() as i32;
        let hat_colors: [u32; 4] = [0x181818, 0x2a1810, 0x604020, 0x502010];
        for i in 0..hat_count {
            let hat_mat = lambert!({ color: hat_colors[ri(hat_colors.len())] });
            let hat_angle0 = (i as f64 / hat_count as f64) * PI * 2.0 + rnd() * 0.5;
            // Hang it on the nearest upper hook.
            let hat_angle = js_round(hat_angle0 / (PI * 2.0 / 3.0)) * (PI * 2.0 / 3.0);
            let brim_y = 0.020 + hook_ys[0] + 0.004;
            // Top hat / bowler variant.
            let is_top_hat = rnd() < 0.5;
            if is_top_hat {
                let crown = mesh(pg("VictorianHallStands__topHat", cyl_geo!(0.060, 0.058, 0.090, 10)), &hat_mat);
                crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.004 + 0.045, hat_angle.sin() * 0.10);
                hs.add(&crown);
                let brim = mesh(pg("VictorianHallStands__topHatBrim", cyl_geo!(0.080, 0.080, 0.008, 12)), &hat_mat);
                brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.004, hat_angle.sin() * 0.10);
                hs.add(&brim);
            } else {
                let bowler_crown = mesh(pg("VictorianHallStands__bowler", sphere_geo!(0.055, 10, 6, 0, PI * 2.0, 0, PI * 0.55)), &hat_mat);
                bowler_crown.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_crown);
                let bowler_brim = mesh(pg("VictorianHallStands__bowlerBrim", cyl_geo!(0.065, 0.065, 0.006, 12)), &hat_mat);
                bowler_brim.position().set(hat_angle.cos() * 0.10, brim_y + 0.003, hat_angle.sin() * 0.10);
                hs.add(&bowler_brim);
            }
        }
    }
    // 40% chance: single scarf hanging from middle hook.
    if rnd() < 0.4 {
        let scarf_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x804060];
        let scarf_mat = lambert!({ color: scarf_colors[ri(scarf_colors.len())] });
        let s_angle = rnd() * PI * 2.0;
        let scarf = mesh(pg("VictorianHallStands__scarf", box_geo!(0.005, 0.35, 0.045)), &scarf_mat);
        scarf.position().set(s_angle.cos() * 0.070, 0.020 + hook_ys[1] - 0.175, s_angle.sin() * 0.070);
        hs.add(&scarf);
        // Slight fringe bumps at bottom.
        for i in 0..3 {
            let fringe = mesh(pg("VictorianHallStands__fringe", box_geo!(0.005, 0.010, 0.008)), &scarf_mat);
            fringe.position().set(
                s_angle.cos() * 0.070,
                0.020 + hook_ys[1] - 0.360,
                s_angle.sin() * 0.070 + (i as f64 - 1.0) * 0.010,
            );
            hs.add(&fringe);
        }
    }
    hs.position().set(g.wx(x) + (rnd() - 0.5) * 0.6, 0, g.wz(z) + (rnd() - 0.5) * 0.6);
    hs.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hs);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
