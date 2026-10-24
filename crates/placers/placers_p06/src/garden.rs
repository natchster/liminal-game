//! Garden shed props: soil, buckets, twine, framed botanicals, bird feeders,
//! gnome (R645–R652).
use crate::{face_yaw, open_tiles, pick, wall_tiles};
use game::prelude::*;

// R645 bag of soil: 6% chance per level. Bulging plastic sack w/ printed label.
game::lazy_static_tex!(
    soil_bag_texture,
    canvas_tex(96, 128, |ctx, w, h| {
        // Beige/tan bag.
        ctx.set_fill_style("#a08050");
        ctx.fill_rect(0, 0, w, h);
        // Title bar band.
        ctx.set_fill_style("#403020");
        ctx.fill_rect(0, h * 0.18, w, h * 0.10);
        // "POTTING SOIL" title placeholder (bold bar).
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(w * 0.15, h * 0.21, w * 0.70, h * 0.04);
        // Center soil illustration — dark clumps.
        ctx.set_fill_style("#2d1a0d");
        for _i in 0..12 {
            ctx.begin_path();
            let cx = w * (0.15 + rnd() * 0.7);
            let cy = h * (0.45 + rnd() * 0.3);
            let r = 6.0 + rnd() * 6.0;
            ctx.arc(cx, cy, r, 0, PI * 2.0);
            ctx.fill();
        }
        // Bottom weight label.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(w * 0.30, h * 0.85, w * 0.40, 4);
    })
);

pub fn place_soil_bags(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bag_mat = lambert!({ map: soil_bag_texture() });
    let bag = group();
    // Main bag body — bulging via slightly compressed box + rounded ends.
    let body = mesh(pg("SoilBags__body", box_geo!(0.12, 0.08, 0.20)), &bag_mat);
    body.position().y = 0.04;
    bag.add(&body);
    // 2 rounded end caps (spheres flattened).
    for side in [-1.0, 1.0] {
        let cap = mesh(pg("SoilBags__cap", sphere_geo!(0.055, 8, 6)), &bag_mat);
        cap.scale().set(1, 0.7, 0.6);
        cap.position().set(0, 0.04, side * 0.09);
        bag.add(&cap);
    }
    bag.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bag.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bag);
}

// R646 metal bucket: 5% chance per level. Tapered cylinder pail + curved handle.
pub fn place_metal_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bucket_colors: [u32; 5] = [0xa0a0a0, 0x808080, 0x606060, 0x2060c0, 0xc02020];
    let bucket_mat = lambert!({ color: pick(&bucket_colors) });
    let handle_mat = lambert!({ color: 0x404040 });
    let bucket = group();
    // Tapered body (narrower at bottom).
    let body = mesh(pg("MetalBuckets__body", cyl_geo!(0.075, 0.058, 0.11, 14)), &bucket_mat);
    body.position().y = 0.055;
    bucket.add(&body);
    // Rim (slight lip torus).
    let rim = mesh(pg("MetalBuckets__rim", torus_geo!(0.075, 0.005, 4, 16)), &bucket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.110;
    bucket.add(&rim);
    // Curved wire handle (half-torus over top).
    let handle = mesh(pg("MetalBuckets__handle", torus_geo!(0.082, 0.003, 4, 12, PI)), &handle_mat);
    handle.rotation().z = 0.0;
    handle.rotation().y = 0.0; // ends at ±x on the ears (was turned 90° away from them)
    handle.position().y = 0.106;
    bucket.add(&handle);
    // 2 handle attachment ears.
    for side in [-1.0, 1.0] {
        let ear = mesh(pg("MetalBuckets__ear", box_geo!(0.008, 0.010, 0.006)), &bucket_mat);
        ear.position().set(side * 0.076, 0.100, 0);
        bucket.add(&ear);
    }
    // 30% tipped on side (rolling): tilted so the rim and the base edge both
    // touch the floor (was pivoted on the base centre, half sunk), bail folded
    // down over the upper side, yaw applied last so it can lie any way.
    let tipped = rnd() < 0.30;
    if tipped {
        bucket.set_rotation_order(EulerOrder::YXZ);
        bucket.rotation().x = PI / 2.0 - 0.197;
        handle.rotation().x = -2.15;
    }
    bucket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, if tipped { 0.057 } else { 0.0 }, g.wz(z) + (rnd() - 0.5) * 2.0);
    bucket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bucket);
}

// R646 garden kneeling pad: 7% chance per level. Foam pad w/ handle cutout.
pub fn place_kneeling_pads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pad_colors: [u32; 5] = [0x40a040, 0x408040, 0x6080c0, 0xc0a040, 0x804020];
    let pad_mat = lambert!({ color: pick(&pad_colors) });
    let pad = group();
    // Main foam pad body.
    let body = mesh(pg("KneelingPads__body", box_geo!(0.35, 0.030, 0.20)), &pad_mat);
    body.position().y = 0.015;
    pad.add(&body);
    // Handle cutout — 2 side blocks + gap (visual only).
    let cutout_mat = lambert!({ color: 0x201818 });
    let cutout = mesh(pg("KneelingPads__cutout", box_geo!(0.080, 0.032, 0.020)), &cutout_mat);
    cutout.position().set(0.115, 0.014, 0);
    pad.add(&cutout);
    pad.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pad.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pad);
}

// R647 twine ball: 7% chance per level. Wound sphere w/ visible thread wraps +
// trailing end.
pub fn place_twine_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let twine_colors: [u32; 4] = [0xc8a870, 0xa08050, 0xe0d8b8, 0x806040];
    let twine_mat = lambert!({ color: pick(&twine_colors) });
    let twine = group();
    // Main ball sphere.
    let ball = mesh(pg("TwineBalls__ball", sphere_geo!(0.038, 12, 10)), &twine_mat);
    ball.position().y = 0.038;
    twine.add(&ball);
    // 6 thin wrap lines (rings around ball) for wound look.
    for _i in 0..6 {
        let wrap = mesh(pg("TwineBalls__wrap", torus_geo!(0.038, 0.001, 3, 16)), lambert!({ color: 0x806040 }));
        wrap.rotation().x = rnd() * PI;
        wrap.rotation().y = rnd() * PI;
        wrap.position().y = 0.038;
        twine.add(&wrap);
    }
    // Trailing end (short segment extending out).
    let end = mesh(pg("TwineBalls__end", cyl_geo!(0.001, 0.001, 0.070, 4)), &twine_mat);
    // Leaves the ball's lower flank (+Y end on its surface) and slopes down
    // to its frayed free end (-Y) lying on the floor.
    let end_a = v3(0.030, 0.016, 0.010);
    let end_b = v3(0.030 + 0.0684 * 0.4f64.cos(), 0.0012, 0.010 - 0.0684 * 0.4f64.sin());
    let mut dir = end_a;
    dir.sub(&end_b);
    dir.normalize();
    end.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
    {
        let mut p = end.position();
        p.copy(&end_a);
        p.add(&end_b);
        p.multiply_scalar(0.5);
    }
    twine.add(&end);
    twine.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    twine.rotation().y = rnd() * PI * 2.0;
    g.world.add(&twine);
}

// R647 framed vintage seed catalog: 3% chance per level. Wall art w/ botanical
// illustration.
game::lazy_static_tex!(
    seed_catalog_texture,
    canvas_tex(128, 168, |ctx, w, h| {
        // Cream paper.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(0, 0, w, h);
        // Vintage border (double lines).
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(1);
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        // Top title bar (dark).
        ctx.set_fill_style("#4a3020");
        ctx.fill_rect(14, 16, w - 28.0, 12);
        // Center vegetable illustration — tomato-ish shape.
        let (cx, cy) = (w / 2.0, h * 0.5);
        ctx.set_fill_style("#c02020");
        ctx.begin_path();
        ctx.arc(cx, cy, 22, 0, PI * 2.0);
        ctx.fill();
        // Green leaves on top.
        ctx.set_fill_style("#408040");
        for i in 0..5 {
            let a = (i as f64 / 5.0) * PI - PI / 2.0;
            ctx.begin_path();
            ctx.ellipse(cx + a.cos() * 12.0, cy - 20.0 + a.sin() * 6.0, 8, 4, a, 0, PI * 2.0);
            ctx.fill();
        }
        // Small stem.
        ctx.set_fill_style("#604020");
        ctx.fill_rect(cx - 1.0, cy - 30.0, 2, 8);
        // Bottom description bars.
        ctx.set_fill_style("#403020");
        for i in 0..4 {
            ctx.fill_rect(20, h - 40.0 + i as f64 * 6.0, w - 40.0 - rnd() * 20.0, 2);
        }
    })
);

pub fn place_seed_catalogs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0x604020 });
    let paper_mat = lambert!({ map: seed_catalog_texture() });
    let catalog = group();
    let mount_y = 1.65;
    // Frame.
    let frame = mesh(pg("SeedCatalogs__frame", box_geo!(0.22, 0.30, 0.012)), &frame_mat);
    frame.position().y = mount_y;
    catalog.add(&frame);
    // Catalog page.
    let paper = mesh(plane_geo!(0.18, 0.25), &paper_mat);
    // On the frame's recessed backing board (front z 0.002), edges under the
    // moulding's sight edge.
    paper.position().set(0, mount_y, 0.0025);
    catalog.add(&paper);
    catalog.rotation().z = (rnd() - 0.5) * 0.05;
    catalog.rotation().y = face_yaw(dx, dz);
    catalog.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&catalog);
}

// R648 framed pressed flower: 3% chance per level. Small shadow-box w/ pressed
// flower silhouette.
game::lazy_static_tex!(
    pressed_flower_texture,
    canvas_tex(96, 128, |ctx, w, h| {
        // Off-white paper.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // Stem line.
        ctx.set_stroke_style("#405020");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx, h - 10.0);
        ctx.line_to(cx + 2.0, cy);
        ctx.stroke();
        // 2 leaves along stem.
        ctx.set_fill_style("#506030");
        for i in 0..2 {
            ctx.begin_path();
            ctx.ellipse(
                cx + (if i == 0 { -8.0 } else { 10.0 }),
                h * 0.65 + i as f64 * 12.0,
                8,
                3,
                if i == 0 { -0.4 } else { 0.4 },
                0,
                PI * 2.0,
            );
            ctx.fill();
        }
        // Flower head — 6 pressed-flat petals.
        let petal_colors = ["#c04060", "#d060c0", "#f0c020", "#c08040", "#c060c0"];
        let petal_color = petal_colors[(rnd() * petal_colors.len() as f64).floor() as usize];
        ctx.set_fill_style(&format!("{}d0", petal_color)); // alpha for pressed look
        for p in 0..6 {
            let a = (p as f64 / 6.0) * PI * 2.0;
            ctx.begin_path();
            ctx.ellipse(cx + a.cos() * 12.0, cy + a.sin() * 12.0, 10, 6, a, 0, PI * 2.0);
            ctx.fill();
        }
        // Center dot.
        ctx.set_fill_style("#d0a020");
        ctx.begin_path();
        ctx.arc(cx, cy, 5, 0, PI * 2.0);
        ctx.fill();
    })
);

pub fn place_pressed_flowers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0xc0a0a0 });
    let glass_mat = basic_mat!({ color: 0xe8f0f8, transparent: true, opacity: 0.25 });
    let paper_mat = lambert!({ map: pressed_flower_texture() });
    let flower = group();
    let mount_y = 1.60;
    // Frame.
    let frame = mesh(pg("PressedFlowers__frame", box_geo!(0.18, 0.24, 0.018)), &frame_mat);
    frame.position().y = mount_y;
    flower.add(&frame);
    // Paper backing.
    let paper = mesh(plane_geo!(0.15, 0.21), &paper_mat);
    // On the mount at the bottom of the shadow box's well (front z 0.0015).
    paper.position().set(0, mount_y, 0.002);
    flower.add(&paper);
    // Glass front (thin translucent): inside the box, just behind the lip
    // of the sight edge (0.0062), not 4 mm in front of the frame.
    let glass = mesh(plane_geo!(0.15, 0.21), &glass_mat);
    glass.position().set(0, mount_y, 0.0058);
    flower.add(&glass);
    flower.rotation().z = (rnd() - 0.5) * 0.04;
    flower.rotation().y = face_yaw(dx, dz);
    flower.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&flower);
}

// R648 folded gardening apron: 5% chance per level. Rectangular fabric w/
// pocket band + tied strap.
pub fn place_garden_aprons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let apron_colors: [u32; 5] = [0x408040, 0x604040, 0x808030, 0xa07040, 0x506080];
    let apron_mat = lambert!({ color: pick(&apron_colors), side: DoubleSide });
    let pocket_mat = lambert!({ color: 0xc0a870 });
    let apron = group();
    // Main folded fabric block (2 folded layers).
    let layer1 = mesh(box_geo!(0.14, 0.008, 0.18), &apron_mat);
    layer1.position().y = 0.004;
    apron.add(&layer1);
    let layer2 = mesh(box_geo!(0.13, 0.008, 0.17), &apron_mat);
    layer2.position().set(0.006, 0.012, -0.008);
    apron.add(&layer2);
    // Pocket band across middle of top layer (contrasting color).
    let pocket = mesh(pg("GardenAprons__pocket", box_geo!(0.12, 0.002, 0.05)), &pocket_mat);
    pocket.position().set(0.006, 0.017, -0.008);
    apron.add(&pocket);
    // 2 tied strap loops (torus segments).
    for i in 0..2 {
        let strap = mesh(cyl_geo!(0.003, 0.003, 0.055, 4), &apron_mat);
        strap.rotation().z = PI / 2.0;
        strap.position().set(-0.020 + i as f64 * 0.030, 0.019, -0.075);
        apron.add(&strap);
    }
    apron.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    apron.rotation().y = rnd() * PI * 2.0;
    g.world.add(&apron);
}

// R649 garden tool tote bag: 4% chance per level. Fabric bag w/ tool handles
// poking out.
pub fn place_tote_bags(g: &mut Game, _dist: &[i32]) {
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

// R649 coat hooks pair: 6% chance per level. Small backboard w/ 2 J-hooks.
pub fn place_coat_hooks_pair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let board_mat = lambert!({ color: 0x8b5a2b });
    let hook_mat = lambert!({ color: 0xa08040 });
    let pair = group();
    let mount_y = 1.60;
    // Small board.
    let board = mesh(pg("CoatHooksPair__board", box_geo!(0.16, 0.05, 0.015)), &board_mat);
    board.position().y = mount_y;
    pair.add(&board);
    // 2 J-hooks.
    for i in 0..2 {
        let hook_x = -0.040 + i as f64 * 0.080;
        let base = mesh(pg("CoatHooksPair__rosette", cyl_geo!(0.008, 0.008, 0.006, 8)), &hook_mat);
        base.rotation().x = PI / 2.0;
        base.position().set(hook_x, mount_y, 0.010);
        pair.add(&base);
        let arm = mesh(pg("CoatHooksPair__arm", cyl_geo!(0.005, 0.005, 0.040, 6)), &hook_mat);
        arm.rotation().x = PI / 2.0;
        arm.position().set(hook_x, mount_y, 0.030);
        pair.add(&arm);
        let curve = mesh(pg("CoatHooksPair__curve", torus_geo!(0.012, 0.004, 3, 8, PI)), &hook_mat);
        // U open upward (a J): one end on the arm tip, free ball-tip end out in the room.
        curve.rotation().y = PI / 2.0;
        curve.rotation().z = PI;
        curve.position().set(hook_x, mount_y, 0.062);
        pair.add(&curve);
    }
    pair.rotation().y = face_yaw(dx, dz);
    pair.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.010), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.010));
    g.world.add(&pair);
}

// R650 hand tool wall pegboard: 3% chance per level. Board w/ hole grid +
// hanging tool silhouettes.
game::lazy_static_tex!(
    pegboard_texture,
    canvas_tex(160, 120, |ctx, w, h| {
        // Brown board.
        ctx.set_fill_style("#a08050");
        ctx.fill_rect(0, 0, w, h);
        // Dark hole grid.
        ctx.set_fill_style("#302010");
        let (cols, rows) = (12, 9);
        for r in 0..rows {
            for c in 0..cols {
                ctx.begin_path();
                ctx.arc(
                    6.0 + c as f64 * ((w - 12.0) / (cols - 1) as f64),
                    6.0 + r as f64 * ((h - 12.0) / (rows - 1) as f64),
                    2,
                    0,
                    PI * 2.0,
                );
                ctx.fill();
            }
        }
    })
);

pub fn place_tool_pegboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let board_mat = lambert!({ map: pegboard_texture() });
    let tool_mat = lambert!({ color: 0x505050 });
    let handle_mat = lambert!({ color: 0xc02020 });
    let peg = group();
    let mount_y = 1.65;
    // Main board.
    let board = mesh(pg("ToolPegboards__board", box_geo!(0.55, 0.42, 0.008)), &board_mat);
    board.position().y = mount_y;
    peg.add(&board);
    // Hanging tools: hammer + wrench + screwdriver silhouettes.
    // Hammer.
    let hammer_head = mesh(pg("ToolPegboards__hammerHead", box_geo!(0.008, 0.025, 0.055)), &tool_mat);
    // head sticks out of the board with its claw resting on the face (it went
    // 2 cm through the board); the handle hangs from the head's eye
    hammer_head.position().set(-0.20, mount_y + 0.10, 0.0315);
    peg.add(&hammer_head);
    let hammer_handle = mesh(pg("ToolPegboards__hammerHandle", cyl_geo!(0.005, 0.005, 0.11, 6)), &handle_mat);
    hammer_handle.position().set(-0.20, mount_y + 0.035, 0.0315);
    peg.add(&hammer_handle);
    // Wrench.
    let wrench = mesh(pg("ToolPegboards__wrench", box_geo!(0.006, 0.10, 0.010)), &tool_mat);
    wrench.position().set(-0.06, mount_y + 0.05, 0.008);
    peg.add(&wrench);
    let wrench_head = mesh(pg("ToolPegboards__wrenchHead", box_geo!(0.028, 0.020, 0.010)), &tool_mat);
    wrench_head.position().set(-0.06, mount_y + 0.11, 0.008);
    peg.add(&wrench_head);
    // Screwdriver.
    let sd_handle = mesh(pg("ToolPegboards__sdHandle", cyl_geo!(0.010, 0.008, 0.055, 8)), &handle_mat);
    sd_handle.position().set(0.06, mount_y + 0.09, 0.008);
    peg.add(&sd_handle);
    let sd_shaft = mesh(pg("ToolPegboards__sdShaft", cyl_geo!(0.003, 0.003, 0.060, 6)), &tool_mat);
    sd_shaft.position().set(0.06, mount_y + 0.035, 0.008); // enters the handle (was a 1.25 cm gap)
    peg.add(&sd_shaft);
    // Pliers.
    let pliers = mesh(pg("ToolPegboards__pliers", box_geo!(0.010, 0.10, 0.010)), &tool_mat);
    pliers.position().set(0.20, mount_y + 0.05, 0.008);
    peg.add(&pliers);
    let pliers_head = mesh(pg("ToolPegboards__pliersHead", box_geo!(0.020, 0.030, 0.010)), &tool_mat);
    pliers_head.position().set(0.20, mount_y + 0.115, 0.008);
    peg.add(&pliers_head);
    peg.rotation().y = face_yaw(dx, dz);
    peg.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&peg);
}

// R650 bird seed feeder: 4% chance per level. Tube feeder w/ perches +
// optional visible seeds.
pub fn place_bird_feeders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tube_mat = lambert!({ color: 0xc0d0d0, transparent: true, opacity: 0.6 });
    let cap_mat = lambert!({ color: 0x408040 });
    let seed_mat = lambert!({ color: 0x604020 });
    let perch_mat = lambert!({ color: 0x606060 });
    let feeder = group();
    // Clear plastic tube.
    let tube = mesh(cyl_geo!(0.030, 0.030, 0.18, 12), &tube_mat);
    tube.position().y = 0.090;
    feeder.add(&tube);
    // Interior seeds (visible through tube).
    let seeds = mesh(pg("BirdFeeders__seeds", cyl_geo!(0.025, 0.025, 0.10, 10)), &seed_mat);
    seeds.position().y = 0.055;
    feeder.add(&seeds);
    // Top cap.
    let top_cap = mesh(pg("BirdFeeders__cap", cyl_geo!(0.036, 0.036, 0.012, 12)), &cap_mat);
    top_cap.position().y = 0.186;
    feeder.add(&top_cap);
    // Bottom base disc.
    let base = mesh(pg("BirdFeeders__base", cyl_geo!(0.045, 0.045, 0.008, 12)), &cap_mat);
    base.position().y = 0.004;
    feeder.add(&base);
    // 4 perches sticking out at 90° each.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let perch = mesh(pg("BirdFeeders__perch", cyl_geo!(0.002, 0.002, 0.040, 4)), &perch_mat);
        perch.rotation().z = PI / 2.0;
        perch.position().set(a.cos() * 0.048, 0.030 + i as f64 * 0.010, a.sin() * 0.048);
        perch.rotation().y = -a;
        feeder.add(&perch);
    }
    // Hanging loop at top.
    let lp = mesh(pg("BirdFeeders__loop", torus_geo!(0.010, 0.002, 3, 8)), &perch_mat);
    // Standing upright (XY plane) with its twisted ends 4 mm down in the
    // cap's hanger boss (cap top 0.192).
    lp.position().y = 0.200;
    feeder.add(&lp);
    feeder.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    feeder.rotation().y = rnd() * PI * 2.0;
    g.world.add(&feeder);
}

// R651 bird bath: 3% chance per level. Pedestal + bowl + water disc.
pub fn place_bird_baths(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let stone_mat = lambert!({ color: 0x909090 });
    let water_mat = lambert!({ color: 0x60a0c0, transparent: true, opacity: 0.75 });
    let bath = group();
    // Base disc.
    let base = mesh(pg("BirdBaths__base", cyl_geo!(0.10, 0.12, 0.020, 12)), &stone_mat);
    base.position().y = 0.010;
    bath.add(&base);
    // Pedestal (tall column).
    let pedestal = mesh(pg("BirdBaths__pedestal", cyl_geo!(0.045, 0.055, 0.32, 10)), &stone_mat);
    pedestal.position().y = 0.18;
    bath.add(&pedestal);
    // Bowl top (wider, shallow dish).
    let bowl = mesh(pg("BirdBaths__bowl", cyl_geo!(0.14, 0.11, 0.030, 14)), &stone_mat);
    bowl.position().y = 0.355;
    bath.add(&bowl);
    // Water surface disc.
    let water = mesh(circle_geo!(0.128, 14), &water_mat);
    water.rotation().x = -PI / 2.0;
    water.position().y = 0.368;
    bath.add(&water);
    bath.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    bath.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bath);
    // R652 subtle water ripple animator (tiny Y bob + XZ scale breathing).
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        water.position().y = 0.368 + (t * 0.9 + phase).sin() * 0.0006;
        let s = 1.0 + (t * 0.7 + phase).sin() * 0.015;
        water.scale().set(s, 1, s);
    });
}

// R651 framed botanical illustration: 3% chance per level. Wall art w/
// detailed plant sketch.
game::lazy_static_tex!(
    botanical_texture,
    canvas_tex(112, 168, |ctx, w, h| {
        // Cream paper.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(0, 0, w, h);
        let cx = w / 2.0;
        // Vertical stem line.
        ctx.set_stroke_style("#405020");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx, h - 8.0);
        ctx.line_to(cx, 30);
        ctx.stroke();
        // 4 pairs of alternating leaves along stem.
        ctx.set_fill_style("#506030");
        for i in 0..4 {
            let y = h - 30.0 - i as f64 * 30.0;
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            ctx.begin_path();
            ctx.ellipse(cx + side * 14.0, y, 12, 4, side * 0.4, 0, PI * 2.0);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(cx - side * 14.0, y - 10.0, 10, 3, -side * 0.4, 0, PI * 2.0);
            ctx.fill();
        }
        // Flower cluster top.
        let petal_color = ["#c04060", "#f0c020", "#d060c0"][(rnd() * 3.0).floor() as usize];
        ctx.set_fill_style(petal_color);
        for p in 0..6 {
            let a = (p as f64 / 6.0) * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.cos() * 8.0, 20.0 + a.sin() * 8.0, 5, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style("#f0e020");
        ctx.begin_path();
        ctx.arc(cx, 20, 3, 0, PI * 2.0);
        ctx.fill();
        // Latin name (bar placeholder at bottom).
        ctx.set_fill_style("#403020");
        ctx.fill_rect(20, h - 18.0, w - 40.0, 3);
    })
);

pub fn place_botanicals(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0x604020 });
    let paper_mat = lambert!({ map: botanical_texture() });
    let bot = group();
    let mount_y = 1.65;
    let frame = mesh(pg("Botanicals__frame", box_geo!(0.20, 0.30, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    bot.add(&frame);
    let paper = mesh(plane_geo!(0.16, 0.26), &paper_mat);
    // On the backing board (front z 0.001) behind the reeded frame's sight edge.
    paper.position().set(0, mount_y, 0.0015);
    bot.add(&paper);
    bot.rotation().z = (rnd() - 0.5) * 0.05;
    bot.rotation().y = face_yaw(dx, dz);
    bot.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&bot);
}

// R652 garden gnome: 3% chance per level. Classic figurine w/ pointed red hat
// + beard + body + boots.
pub fn place_garden_gnomes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hat_mat = lambert!({ color: 0xc02020 });
    let body_colors: [u32; 4] = [0x2060a0, 0x408040, 0xa04040, 0x604020];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let skin_mat = lambert!({ color: 0xf0c898 });
    let beard_mat = lambert!({ color: 0xf0f0f0 });
    let boot_mat = lambert!({ color: 0x604020 });
    let gnome = group();
    // Boots (2 blocks).
    for side in [-1.0, 1.0] {
        let boot = mesh(pg("GardenGnomes__boot", box_geo!(0.014, 0.016, 0.020)), &boot_mat);
        // Toes (-Z, the face side) peek out from under the robe's hem.
        boot.position().set(side * 0.009, 0.008, -0.016);
        gnome.add(&boot);
    }
    // Body/robe (tapered cylinder).
    let body = mesh(pg("GardenGnomes__body", cyl_geo!(0.030, 0.024, 0.070, 8)), &body_mat);
    body.position().y = 0.050;
    gnome.add(&body);
    // Head (skin sphere).
    let head = mesh(pg("GardenGnomes__head", sphere_geo!(0.024, 8, 6)), &skin_mat);
    head.position().y = 0.098;
    gnome.add(&head);
    // White beard (triangular cone below face).
    let beard = mesh(pg("GardenGnomes__beard", cone_geo!(0.018, 0.030, 6)), &beard_mat);
    // On the face side (-Z, with the nose): wide top at the chin under the
    // nose, the point hanging down over the chest, tipped 0.2 rad forward.
    beard.rotation().x = PI + 0.2;
    beard.position().set(0, 0.074, -0.026);
    gnome.add(&beard);
    // Pointed red hat (tall cone).
    let hat = mesh(pg("GardenGnomes__hat", cone_geo!(0.026, 0.070, 8)), &hat_mat);
    hat.position().y = 0.140;
    gnome.add(&hat);
    // Hat brim (small torus).
    let brim = mesh(pg("GardenGnomes__brim", torus_geo!(0.022, 0.005, 4, 12)), &hat_mat);
    brim.rotation().x = PI / 2.0;
    brim.position().y = 0.115;
    gnome.add(&brim);
    // Small nose (skin sphere).
    let nose = mesh(pg("GardenGnomes__nose", sphere_geo!(0.005, 5, 4)), &skin_mat);
    nose.position().set(0, 0.098, -0.020);
    gnome.add(&nose);
    gnome.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    gnome.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gnome);
}
