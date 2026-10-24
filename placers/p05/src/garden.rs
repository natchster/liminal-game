//! Wall crafts, hand mirror, plants and garden tools (backrooms.js R635–R645).
use crate::{face_yaw, open_tiles, pick, set_rot_order, wall_tiles};
use game::prelude::*;

// ------------------------------------------------------------------ line 13426
// R635 framed cross-stitch/embroidery: 4% chance per level. Round wooden hoop
// w/ fabric + simple pattern.
game::lazy_static_tex!(crossstitch_texture, canvas_tex(128, 128, |ctx, w, h| {
    // Cream fabric base.
    ctx.set_fill_style("#f8f0d8");
    ctx.fill_rect(0, 0, w, h);
    // Small heart or flower pattern in center.
    ctx.set_fill_style("#c02020");
    // Little cross-stitch "X"s in heart shape (11 pixel dots).
    let heart_dots: [(f64, f64); 9] = [
        (0.5, 0.35), (0.4, 0.4), (0.6, 0.4),
        (0.35, 0.5), (0.65, 0.5), (0.5, 0.5),
        (0.4, 0.6), (0.6, 0.6),
        (0.5, 0.7),
    ];
    for (px, py) in heart_dots {
        ctx.fill_rect(w * px - 4.0, h * py - 4.0, 8, 8);
    }
    // Green stitches along bottom (leaves/vines).
    ctx.set_fill_style("#408040");
    for i in 0..6 {
        ctx.fill_rect(20.0 + i as f64 * 16.0, h * 0.85, 6, 6);
    }
}));

// ------------------------------------------------------------------ line 13448
pub fn place_cross_stitches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = pick(&walls);
    let hoop_mat = lambert!({ color: 0x8b5a2b });
    let fab_mat = lambert!({ map: crossstitch_texture() });
    let stitch = group();
    let mount_y = 1.55;
    // Outer wooden hoop ring.
    let hoop = mesh(pg("CrossStitches__hoop", torus_geo!(0.10, 0.012, 5, 20)), &hoop_mat);
    hoop.position().y = mount_y;
    // Torus is already in the XY plane, framing the fabric disc (no turn).
    stitch.add(&hoop);
    // Fabric disc inside hoop.
    let fabric = mesh(circle_geo!(0.096, 16), &fab_mat);
    fabric.position().set(0, mount_y, 0.001);
    stitch.add(&fabric);
    // Small screw/clamp at top.
    let clamp = mesh(pg("CrossStitches__clamp", cyl_geo!(0.008, 0.008, 0.010, 6)), lambert!({ color: 0xa08050 }));
    clamp.position().set(0, mount_y + 0.108, 0.005);
    stitch.add(&clamp);
    stitch.rotation().z = (rnd() - 0.5) * 0.06;
    stitch.rotation().y = face_yaw(dx, dz);
    stitch.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&stitch);
}

// ------------------------------------------------------------------ line 13496
// R637 hand mirror: 6% chance per level. Round mirror w/ long handle laid flat
// on floor.
pub fn place_hand_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let frame_colors: [u32; 5] = [0xd0a040, 0x606060, 0x804060, 0xc02020, 0x101010];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let glass_mat = lambert!({ color: 0xe0e8f0 });
    let mirror = group();
    // Round frame disc.
    let frame = mesh(pg("HandMirrors__frame", cyl_geo!(0.030, 0.030, 0.005, 12)), &frame_mat);
    frame.position().y = 0.003;
    mirror.add(&frame);
    // Glass surface (slightly smaller silver disc on top).
    let glass = mesh(circle_geo!(0.026, 12), &glass_mat);
    glass.rotation().x = -PI / 2.0;
    glass.position().y = 0.006;
    mirror.add(&glass);
    // Handle (long thin rectangle extending).
    let handle = mesh(pg("HandMirrors__handle", box_geo!(0.008, 0.006, 0.070)), &frame_mat);
    handle.position().set(0, 0.003, 0.060);
    mirror.add(&handle);
    // Small grip knob at end.
    let knob = mesh(pg("HandMirrors__knob", sphere_geo!(0.008, 6, 5)), &frame_mat);
    knob.scale().set_scalar(0.625); // r 5 mm: a finial just wider than the 8 mm handle, resting on the floor
    knob.position().set(0, 0.005, 0.098);
    mirror.add(&knob);
    mirror.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mirror.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mirror);
}

// ------------------------------------------------------------------ line 13541
// R637 framed watercolor: 4% chance per level. Ornate frame + pastel abstract
// wash inside.
game::lazy_static_tex!(watercolor_texture, canvas_tex(128, 96, |ctx, w, h| {
    // Soft off-white paper.
    ctx.set_fill_style("#f0ecdc");
    ctx.fill_rect(0, 0, w, h);
    // 5 large soft blobs of pastel color.
    let pastels = ["#a0c0e8", "#e0b0c0", "#c0d8a8", "#e8d090", "#c0a0d0"];
    for i in 0..8 {
        ctx.set_fill_style(&format!("{}a0", pastels[i % pastels.len()])); // alpha ~62%.
        ctx.begin_path();
        let cx = rnd() * w;
        let cy = rnd() * h;
        let r = 8.0 + rnd() * 24.0;
        ctx.arc(cx, cy, r, 0, PI * 2.0);
        ctx.fill();
    }
    // Thin dark accent lines (brush strokes).
    ctx.set_stroke_style("#403020");
    ctx.set_line_width(1);
    for _i in 0..3 {
        ctx.begin_path();
        let (mx, my) = (rnd() * w, rnd() * h);
        ctx.move_to(mx, my);
        let (c1x, c1y) = (rnd() * w, rnd() * h);
        let (c2x, c2y) = (rnd() * w, rnd() * h);
        let (ex, ey) = (rnd() * w, rnd() * h);
        ctx.bezier_curve_to(c1x, c1y, c2x, c2y, ex, ey);
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 13570
pub fn place_watercolors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0xd0a040 });
    let matting_mat = lambert!({ color: 0xf0eddb });
    let paint_mat = lambert!({ map: watercolor_texture() });
    let painting = group();
    let mount_y = 1.65;
    // Gold outer frame.
    let frame = mesh(pg("Watercolors__frame", box_geo!(0.34, 0.26, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    painting.add(&frame);
    // White matting inset.
    let matting = mesh(pg("Watercolors__mat", box_geo!(0.28, 0.20, 0.010)), &matting_mat);
    matting.position().set(0, mount_y, 0.008);
    painting.add(&matting);
    // Watercolor paper surface.
    let art = mesh(plane_geo!(0.22, 0.16), &paint_mat);
    art.position().set(0, mount_y, 0.014);
    painting.add(&art);
    painting.rotation().z = (rnd() - 0.5) * 0.05; // slight tilt
    painting.rotation().y = face_yaw(dx, dz);
    painting.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&painting);
}

// ------------------------------------------------------------------ line 13617
// R638 hanging plant (vine hanger): 3% chance per level. Wall-mount macramé
// cord + pot + trailing vines.
pub fn place_hanging_plants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = pick(&walls);
    let bracket_mat = lambert!({ color: 0x404040 });
    let pot_colors: [u32; 4] = [0x8b5a2b, 0x606060, 0xa0603c, 0xd0a070];
    let pot_mat = lambert!({ color: pick(&pot_colors) });
    let leaf_colors: [u32; 4] = [0x408040, 0x306028, 0x507838, 0x60a058];
    let leaf_mat = lambert!({ color: pick(&leaf_colors), side: DoubleSide });
    let dirt_mat = lambert!({ color: 0x2d1a0d });
    let plant = group();
    let mount_y = 1.90;
    // Wall bracket arm out from wall.
    let arm = mesh(pg("HangingPlants__arm", box_geo!(0.010, 0.014, 0.14)), &bracket_mat);
    arm.position().set(0, mount_y, 0.07);
    plant.add(&arm);
    // Hanging chain link.
    let chain = mesh(pg("HangingPlants__chain", cyl_geo!(0.0015, 0.0015, 0.06, 3)), &bracket_mat);
    // Stretched from the bracket's eye down into the pot (top mountY - 0.066).
    chain.scale().y = 0.070 / 0.06;
    chain.position().set(0, mount_y - 0.035, 0.13);
    plant.add(&chain);
    // Pot (bowl below chain).
    let pot = mesh(pg("HangingPlants__pot", cyl_geo!(0.055, 0.045, 0.048, 12)), &pot_mat);
    pot.position().set(0, mount_y - 0.09, 0.13);
    plant.add(&pot);
    // Dirt disc top of pot.
    let dirt = mesh(circle_geo!(0.052, 12), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().set(0, mount_y - 0.066, 0.13);
    plant.add(&dirt);
    // 5-8 trailing vine leaves cascading below pot.
    let vine_count = 5 + (rnd() * 4.0).floor() as i32;
    // R639 track for sway animator: (leaf, userData.baseZ, userData.phase, userData.depth)
    let mut vine_leaves: Vec<(Obj, f64, f64, f64)> = vec![];
    for i in 0..vine_count {
        let t = i as f64 / (vine_count - 1) as f64;
        let leaf = mesh(circle_geo!(0.018 * (1.0 - t * 0.3), 6), &leaf_mat);
        leaf.scale().set(0.6, 1.4, 1);
        let swing = (rnd() - 0.5) * 0.06;
        leaf.position().set(swing, mount_y - 0.09 - i as f64 * 0.032, 0.13 + (rnd() - 0.5) * 0.03);
        let base_z = (rnd() - 0.5) * 0.6;
        leaf.rotation().z = base_z;
        let phase = rnd() * PI * 2.0 + i as f64 * 0.3;
        {
            let mut u = leaf.user();
            u.base_z = base_z;
            u.phase = phase;
            u.set("depth", t);
        }
        vine_leaves.push((leaf.clone(), base_z, phase, t));
        plant.add(&leaf);
    }
    plant.rotation().y = face_yaw(dx, dz);
    plant.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.080), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.080));
    g.world.add(&plant);
    // R639 gentle sway animator (lower leaves sway more — pendulum effect).
    g.animate(move |_g, _dt, t| {
        for (l, base_z, phase, depth) in &vine_leaves {
            let amp = 0.04 + depth * 0.06; // lower leaves swing wider
            l.rotation().z = base_z + (t * 0.32 + phase).sin() * amp;
        }
    });
}

// ------------------------------------------------------------------ line 13706
// R638 potted seedling: 7% chance per level. Small pot w/ tiny green sprout.
pub fn place_seedlings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 { return; } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let pot_mat = lambert!({ color: 0xa0603c });
    let dirt_mat = lambert!({ color: 0x2d1a0d });
    let stem_mat = lambert!({ color: 0x60a040 });
    let leaf_mat = lambert!({ color: 0x408040, side: DoubleSide });
    let seedling = group();
    // Small pot.
    let pot = mesh(pg("Seedlings__pot", cyl_geo!(0.028, 0.024, 0.030, 10)), &pot_mat);
    pot.position().y = 0.015;
    seedling.add(&pot);
    // Dirt top.
    let dirt = mesh(pg("Seedlings__dirt", circle_geo!(0.026, 10)), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().y = 0.031;
    seedling.add(&dirt);
    // Tiny stem stub.
    let stem = mesh(pg("Seedlings__stem", cyl_geo!(0.0015, 0.0015, 0.024, 4)), &stem_mat);
    stem.position().y = 0.043;
    seedling.add(&stem);
    // 2 tiny cotyledons (seed leaves).
    for side in [-1.0, 1.0] {
        let leaf = mesh(circle_geo!(0.006, 5), &leaf_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.rotation().z = side * 0.6;
        leaf.position().set(side * 0.005, 0.054, 0);
        seedling.add(&leaf);
    }
    seedling.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    seedling.rotation().y = rnd() * PI * 2.0;
    g.world.add(&seedling);
}

// ------------------------------------------------------------------ line 13756
// R640 gardening trowel: 6% chance per level. Metal scoop blade + handle.
pub fn place_trowels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0x8b5a2b, 0x404040, 0xc02020, 0x2060c0];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let trowel = group();
    // Scoop blade (curved via 2 rotated planes + a cylinder half).
    // Simpler: use a slightly tapered cylinder half via ConeGeometry inverted.
    let blade = mesh(pg("Trowels__blade", cyl_geo!(0.020, 0.014, 0.080, 6, 1, false, 0, PI)), &blade_mat);
    // Scoop lying hollow-up: the half-cylinder's round back (+X) down, its
    // width across (Z), the tip end (+Y) toward -X; the belly rests on the
    // floor and the rim is at y 0.020.
    blade.rotation().x = PI;
    blade.rotation().z = PI / 2.0;
    blade.position().set(-0.030, 0.020, 0);
    trowel.add(&blade);
    // Blade tip cone (pointed end): flattened (0.35) to a point continuing
    // the scoop's floor, lying on the floor.
    let tip = mesh(pg("Trowels__tip", cone_geo!(0.015, 0.020, 6)), &blade_mat);
    tip.rotation().z = -PI / 2.0;
    tip.scale().x = 0.35;
    tip.position().set(-0.080, 0.0055, 0);
    trowel.add(&tip);
    // Shaft (thin cylinder from blade to handle).
    let shaft = mesh(pg("Trowels__shaft", cyl_geo!(0.005, 0.005, 0.040, 6)), &blade_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.008, 0.010, 0);
    trowel.add(&shaft);
    // Wooden/plastic handle grip.
    let handle = mesh(pg("Trowels__handle", cyl_geo!(0.012, 0.014, 0.060, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.056, 0.012, 0);
    trowel.add(&handle);
    // Small end cap.
    let cap = mesh(pg("Trowels__cap", sphere_geo!(0.014, 6, 5)), &handle_mat);
    cap.position().set(0.090, 0.012, 0);
    trowel.add(&cap);
    trowel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trowel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&trowel);
}

// ------------------------------------------------------------------ line 13814
// R640 seed packet: 8% chance per level. Colored paper envelope w/ printed
// flower image.
game::lazy_static_tex!(seed_packet_texture, canvas_tex(80, 120, |ctx, w, h| {
    // Random pastel background.
    let bgs = ["#f0e0a0", "#e0b0a0", "#a0c0e0", "#c0e0a0"];
    ctx.set_fill_style(bgs[(rnd() * bgs.len() as f64).floor() as usize]);
    ctx.fill_rect(0, 0, w, h);
    // Border stripe at top.
    ctx.set_fill_style("#404020");
    ctx.fill_rect(0, 0, w, 10);
    // "Seed name" title bar (dark bar).
    ctx.set_fill_style("#202010");
    ctx.fill_rect(6, 16, w - 12.0, 6);
    // Center flower illustration (5 petal circle + center).
    let (cx, cy) = (w / 2.0, h * 0.55);
    let flower_colors = ["#c02020", "#f0c020", "#d060c0", "#c04060", "#e08040"];
    ctx.set_fill_style(flower_colors[(rnd() * flower_colors.len() as f64).floor() as usize]);
    for p in 0..5 {
        let a = (p as f64 / 5.0) * PI * 2.0;
        ctx.begin_path();
        ctx.arc(cx + a.cos() * 12.0, cy + a.sin() * 12.0, 8, 0, PI * 2.0);
        ctx.fill();
    }
    // Center yellow dot.
    ctx.set_fill_style("#f0e020");
    ctx.begin_path();
    ctx.arc(cx, cy, 5, 0, PI * 2.0);
    ctx.fill();
    // Bottom info bar.
    ctx.set_fill_style("#404020");
    for i in 0..3 {
        ctx.fill_rect(6, h - 24.0 + i as f64 * 6.0, w - 12.0 - rnd() * 20.0, 2);
    }
}));

// ------------------------------------------------------------------ line 13846
pub fn place_seed_packets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let packet_mat = lambert!({ map: seed_packet_texture(), side: DoubleSide });
    let packet = mesh(plane_geo!(0.055, 0.080), &packet_mat);
    packet.rotation().x = -PI / 2.0;
    packet.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0.003, g.wz(z) + (rnd() - 0.5) * 2.5);
    packet.rotation().z = rnd() * PI * 2.0;
    g.world.add(&packet);
}

// ------------------------------------------------------------------ line 13873
// R641 mini watering can: 5% chance per level. Body + spout + handle.
pub fn place_watering_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
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
    // The spout's narrow end (+Y) points out and up along spoutDir; its wide,
    // flanged end is set into the body low on the -X side.
    let spout = mesh(pg("WateringCans__spout", cyl_geo!(0.008, 0.010, 0.095, 6)), &can_mat);
    let spout_dir = v3(-(PI / 3.0).sin(), (PI / 3.0).cos(), 0);
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
    let rose_up = v3((PI / 3.0).cos(), (PI / 3.0).sin(), 0);
    // 5 tiny hole dots on rose.
    for hh in 0..5 {
        let hole = mesh(circle_geo!(0.0012, 4), lambert!({ color: 0x101010 }));
        let a = (hh as f64 / 5.0) * PI * 2.0;
        hole.quaternion().set_from_unit_vectors(&v3(0, 0, 1), &spout_dir);
        {
            let mut p = hole.position();
            p.copy(&rose_face);
            p.add_scaled_vector(&rose_up, a.cos() * 0.005);
            p.add(&v3(0, 0, a.sin() * 0.005));
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

// ------------------------------------------------------------------ line 13937
// R641 gardening gloves (pair): 7% chance per level. Two glove shapes lying on
// floor.
pub fn place_gardening_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 { return; } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let glove_colors: [u32; 5] = [0x408040, 0xa04020, 0x808030, 0x604020, 0xc0a060];
    let glove_mat = lambert!({ color: pick(&glove_colors) });
    let cuff_mat = lambert!({ color: 0x604020 });
    let pair = group();
    // Draw 2 gloves side-by-side.
    for gi in 0..2 {
        let glove = group();
        // Palm block.
        let palm = mesh(pg("GardeningGloves__palm", box_geo!(0.045, 0.010, 0.055)), &glove_mat);
        palm.position().y = 0.005;
        glove.add(&palm);
        // Thumb bump.
        let thumb = mesh(pg("GardeningGloves__thumb", sphere_geo!(0.010, 6, 5)), &glove_mat);
        thumb.scale().set(1, 0.5, 1.4);
        thumb.position().set(-0.028, 0.005, -0.010);
        glove.add(&thumb);
        // 4 finger stubs.
        for f in 0..4 {
            let finger = mesh(pg("GardeningGloves__finger", box_geo!(0.008, 0.010, 0.020)), &glove_mat);
            finger.position().set(-0.014 + f as f64 * 0.010, 0.005, 0.035);
            glove.add(&finger);
        }
        // Cuff (darker band at wrist).
        let cuff = mesh(pg("GardeningGloves__cuff", box_geo!(0.048, 0.012, 0.012)), &cuff_mat);
        cuff.position().set(0, 0.006, -0.030);
        glove.add(&cuff);
        glove.position().set(if gi == 0 { -0.030 } else { 0.030 }, 0, 0);
        glove.rotation().y = if gi == 0 { 0.15 } else { -0.15 };
        pair.add(&glove);
    }
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

// ------------------------------------------------------------------ line 13990
// R643 pruning shears: 6% chance per level. 2 curved blades + 2 handles + pivot.
pub fn place_pruning_shears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0xc02020, 0xd0a040, 0x408040, 0x202020];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let shears = group();
    // Upper blade (curved via short cylinder + cone tip).
    // Lying flat, jaws and handles sprung open in the floor plane about a
    // vertical pivot bolt at (0.005, 0): the blade (-Z side, edge +Z) and
    // the hook (+Z side, edge -Z) open ±0.15 rad from their roots at the bolt,
    // their tips on their ends; the handles open ±0.2 rad from the bolt.
    let upper_blade = mesh(pg("PruningShears__blade", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    upper_blade.rotation().y = -0.15;
    upper_blade.position().set(0.005 - 0.020 * 0.15f64.cos(), 0.0025, -0.004 - 0.020 * 0.15f64.sin());
    shears.add(&upper_blade);
    // Upper blade tip cone (point outward, flattened to the blade's thickness).
    let upper_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    upper_tip.rotation().set(0, -0.15, -PI / 2.0);
    upper_tip.scale().x = 0.45;
    upper_tip.position().set(0.005 - 0.0485 * 0.15f64.cos(), 0.0025, -0.004 - 0.0485 * 0.15f64.sin());
    shears.add(&upper_tip);
    // Lower blade (matches, angled other way).
    let lower_blade = mesh(pg("PruningShears__hook", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    lower_blade.rotation().y = 0.15;
    lower_blade.position().set(0.005 - 0.020 * 0.15f64.cos(), 0.0025, 0.004 + 0.020 * 0.15f64.sin());
    shears.add(&lower_blade);
    let lower_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    lower_tip.rotation().set(0, 0.15, -PI / 2.0);
    lower_tip.scale().x = 0.45;
    lower_tip.position().set(0.005 - 0.0485 * 0.15f64.cos(), 0.0025, 0.004 + 0.0485 * 0.15f64.sin());
    shears.add(&lower_tip);
    // Pivot (rivet in middle): vertical bolt through blades and handles.
    let pivot = mesh(pg("PruningShears__pivot", cyl_geo!(0.004, 0.004, 0.020, 6)), &blade_mat);
    pivot.scale().y = 0.75;
    pivot.position().set(0.005, 0.0075, 0);
    shears.add(&pivot);
    // 2 handles extending back.
    let handle_upper = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_upper.rotation().y = 0.2;
    handle_upper.position().set(0.005 + 0.0245 * 0.2f64.cos(), 0.007, -0.006 - 0.0245 * 0.2f64.sin());
    shears.add(&handle_upper);
    let handle_lower = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_lower.rotation().y = -0.2;
    handle_lower.position().set(0.005 + 0.0245 * 0.2f64.cos(), 0.007, 0.006 + 0.0245 * 0.2f64.sin());
    shears.add(&handle_lower);
    shears.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    shears.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shears);
}

// ------------------------------------------------------------------ line 14056
// R643 garden hoe: 4% chance per level. Long wooden handle + metal flat blade.
pub fn place_garden_hoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0xa07040 });
    let head_mat = lambert!({ color: 0x505050 });
    let hoe = group();
    // Long handle (laid flat).
    let handle = mesh(pg("GardenHoes__handle", cyl_geo!(0.012, 0.012, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.012;
    hoe.add(&handle);
    // Metal collar at head end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.014, 0.014, 0.020, 8)), &head_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.012, 0);
    hoe.add(&collar);
    // Flat blade head (perpendicular to handle).
    let head = mesh(pg("GardenHoes__head", box_geo!(0.016, 0.006, 0.070)), &head_mat);
    // Socket side 1 mm into the collar's end (-0.275), lying on the floor.
    head.position().set(-0.282, 0.003, 0);
    hoe.add(&head);
    // Small end cap on handle other end.
    let cap = mesh(pg("GardenHoes__cap", sphere_geo!(0.014, 6, 5)), &wood_mat);
    cap.position().set(0.278, 0.012, 0);
    hoe.add(&cap);
    hoe.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    hoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hoe);
}

// ------------------------------------------------------------------ line 14099
// R644 garden rake: 4% chance per level. Long handle + crossbar + tines fan.
pub fn place_garden_rakes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let head_mat = lambert!({ color: 0x505050 });
    let rake = group();
    // Long handle.
    let handle = mesh(pg("GardenHoes__handle", cyl_geo!(0.011, 0.011, 0.60, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.014;
    rake.add(&handle);
    // Collar at head.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.013, 0.013, 0.020, 8)), &head_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.295, 0.014, 0);
    rake.add(&collar);
    // Crossbar (short horizontal).
    let bar = mesh(pg("GardenRakes__bar", box_geo!(0.005, 0.008, 0.100)), &head_mat);
    bar.position().set(-0.307, 0.014, 0); // against the collar's end (-0.305)
    rake.add(&bar);
    // 8 tine spikes descending from crossbar.
    // The rake lies on its back: each tine's thick root (+Y) is set 1 mm
    // into the bar's top (y 0.018), the point rising 0.3 rad forward (-X).
    for i in 0..8 {
        let tine = mesh(pg("GardenRakes__tine", cyl_geo!(0.001, 0.0018, 0.040, 4)), &head_mat);
        tine.position().set(-0.307 - 0.020 * 0.3f64.sin(), 0.017 + 0.020 * 0.3f64.cos(), -0.045 + i as f64 * 0.013);
        tine.rotation().z = 0.3 - PI; // angle tines forward slightly
        rake.add(&tine);
    }
    // End cap.
    let cap = mesh(pg("GardenRakes__cap", sphere_geo!(0.013, 6, 5)), &wood_mat);
    cap.position().set(0.302, 0.014, 0);
    rake.add(&cap);
    rake.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    rake.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rake);
}

// ------------------------------------------------------------------ line 14150
// R644 garden hose coil: 5% chance per level. Concentric tori forming coiled hose.
pub fn place_garden_hoses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let hose_colors: [u32; 4] = [0x408040, 0x2060c0, 0x606060, 0x804040];
    let hose_mat = lambert!({ color: pick(&hose_colors) });
    let nozzle_mat = lambert!({ color: 0xd0a040 });
    let hose = group();
    // 4 concentric torus loops for coiled read.
    // Lying flat on the floor, each turn pressed against the next (1.4 cm
    // pitch for a 1.6 cm hose), so they read as one flat coil.
    for i in 0..4 {
        let r = 0.090 - i as f64 * 0.014;
        let lp = mesh(pg("GardenHoses__loop", torus_geo!(r, 0.008, 5, 16)), &hose_mat);
        lp.rotation().x = PI / 2.0;
        lp.rotation().z = i as f64 * 1.3; // turn each loop's stripe/bump round
        lp.position().y = 0.008;
        hose.add(&lp);
    }
    // Brass spray nozzle at end.
    // The hose end leaves the outer turn tangentially at (0.09, 0): the
    // coupling (+Y) sits on the hose there, the front points along +Z.
    let nozzle = mesh(pg("GardenHoses__nozzle", cyl_geo!(0.010, 0.012, 0.060, 8)), &nozzle_mat);
    nozzle.rotation().x = -PI / 2.0;
    nozzle.position().set(0.094, 0.011, 0.028);
    hose.add(&nozzle);
    // Nozzle tip cone.
    let tip = mesh(pg("GardenHoses__tip", cone_geo!(0.008, 0.014, 6)), &nozzle_mat);
    tip.rotation().x = PI / 2.0;
    tip.position().set(0.094, 0.011, 0.064);
    hose.add(&tip);
    hose.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    hose.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hose);
}

// ------------------------------------------------------------------ line 14200
// R645 garden shovel: 3% chance per level. Long handle + D-grip + spade blade.
pub fn place_garden_shovels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let blade_mat = lambert!({ color: 0x505050 });
    let shovel = group();
    // Long handle.
    let handle = mesh(pg("GardenShovels__handle", cyl_geo!(0.014, 0.014, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.014;
    shovel.add(&handle);
    // Collar at blade end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.016, 0.016, 0.020, 8)), &blade_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.014, 0);
    shovel.add(&collar);
    // Spade blade (rectangular w/ tapered tip via cone).
    let blade = mesh(pg("GardenShovels__blade", box_geo!(0.070, 0.008, 0.090)), &blade_mat);
    // Socket strap 2 mm into the collar (end -0.275); the blade lies on the floor.
    blade.position().set(-0.308, 0.004, 0);
    shovel.add(&blade);
    // Blade tip taper.
    // A flat triangular point in the blade's plane: apex out (-X), the
    // pyramid squashed (0.12) to the blade's thickness instead of standing
    // 9 cm tall with its lowest corner 3 cm under the floor.
    let tip = mesh(cone_geo!(0.045, 0.030, 3), &blade_mat);
    tip.rotation().z = PI / 2.0;
    tip.rotation().x = PI / 2.0;
    tip.scale().z = 0.12;
    tip.position().set(-0.308 - 0.035 - 0.014, 0.0054, 0);
    shovel.add(&tip);
    // D-grip at handle end (torus + 2 side bars).
    // The D lies in the blade's plane (XZ): chord across the handle's end
    // (narrowed 0.6 so its ends meet the shaft), crown bulging out along +X.
    let d_grip = mesh(pg("GardenShovels__grip", torus_geo!(0.024, 0.006, 5, 12, PI)), &wood_mat);
    set_rot_order(&d_grip, -PI / 2.0, -PI / 2.0, 0.0, EulerOrder::YXZ);
    d_grip.scale().x = 0.6;
    d_grip.position().set(0.273, 0.014, 0);
    shovel.add(&d_grip);
    shovel.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    shovel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shovel);
}
