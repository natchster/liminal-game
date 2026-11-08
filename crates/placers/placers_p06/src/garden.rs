//! Garden shed props: soil, buckets, twine, framed botanicals, bird feeders,
//! gnome.
use crate::{face_yaw, open_tiles, pick, wall_tiles};
use game::prelude::*;

// Soil bag, seed catalogue, apron, bird bath, gnome:
// made real by batch `t06` (`t06.rs`)
pub use crate::t06::{place_bird_baths, place_garden_aprons, place_garden_gnomes, place_seed_catalogs, place_soil_bags};

// Framed botanical: made real by batch `t17` (`garden/t17.rs`)
mod t17;
pub use t17::place_botanicals;

// Metal bucket: 5% chance per level. Tapered cylinder pail + curved handle.
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

// Garden kneeling pad: 7% chance per level. Foam pad w/ handle cutout.
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

// Twine ball: 7% chance per level. Wound sphere w/ visible thread wraps +
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

// Framed pressed flower: 3% chance per level. Small shadow-box w/ pressed
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

// Garden tool tote bag: 4% chance per level. Fabric bag w/ tool handles
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

// Coat hooks pair: 6% chance per level. Small backboard w/ 2 J-hooks.
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

// Hand tool wall pegboard: 3% chance per level. Board w/ hole grid +
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

// Bird seed feeder: 4% chance per level. Tube feeder w/ perches +
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
