//! Prop placers ported from backrooms.js (chunk p08, lines 17607–19116; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeKnittingBaskets", place_knitting_baskets));
    r.push(("placeHangingColanders", place_hanging_colanders));
    r.push(("placeHerbSieves", place_herb_sieves));
    r.push(("placeDonuts", place_donuts));
    r.push(("placeMakeupSponges", place_makeup_sponges));
    r.push(("placeEmbroideryHoops", place_embroidery_hoops));
    r.push(("placeLipsticks", place_lipsticks));
    r.push(("placeCrossStitchSamplers", place_cross_stitch_samplers));
    r.push(("placeNailPolishes", place_nail_polishes));
    r.push(("placeChildDrawings", place_child_drawings));
    r.push(("placePerfumes", place_perfumes));
    r.push(("placeFamilyPhotos", place_family_photos));
    r.push(("placeHairbrushes", place_hairbrushes));
    r.push(("placeBabyPhotos", place_baby_photos));
    r.push(("placeStandingMirrors", place_standing_mirrors));
    r.push(("placeWeddingPhotos", place_wedding_photos));
    r.push(("placeBittenDonuts", place_bitten_donuts));
}

// ------------------------------------------------------------------ helpers

/// The `openTiles` scan every floor placer repeats: open interior tiles, row-major.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// The `wallTiles` scan: open interior tiles with the first adjacent wall
/// (`tiles[...] === 1`) in [[1,0],[-1,0],[0,1],[0,-1]] order.
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// `arr[Math.floor(Math.random() * arr.length)]`
fn pick<T: Copy>(a: &[T]) -> T { a[(rnd() * a.len() as f64).floor() as usize] }

/// `Math.atan2(-dx, -dz)` (keeps JS's -0 for a zero component)
fn face_wall(dx: i32, dz: i32) -> f64 { (-(dx as f64)).atan2(-(dz as f64)) }

/// Wall-mount position `wx(x) + dx * (CELL * 0.5 - inset)`, `wz(z) + dz * (...)`.
fn wall_pos(g: &Game, x: i32, z: i32, dx: i32, dz: i32, inset: f64) -> (f64, f64) {
    (g.wx(x) + dx as f64 * (CELL * 0.5 - inset), g.wz(z) + dz as f64 * (CELL * 0.5 - inset))
}

// ------------------------------------------------------------------ line 17607
// R682 knitting basket w/ yarn ball + needles: 4% chance
// per level. Twin variant of R680 sewing basket.
fn place_knitting_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let wicker_colors: [u32; 4] = [0xa07040, 0x8b6030, 0xc09060, 0x704a20];
    let wicker_mat = lambert!({ color: pick(&wicker_colors) });
    let basket = group();
    // Basket body — squatter than sewing basket, no handle.
    let body = mesh(pg("KnittingBaskets__body", cyl_geo!(0.11, 0.090, 0.055, 12, 1, true)), &wicker_mat);
    body.position().y = 0.028;
    basket.add(&body);
    let bottom = mesh(circle_geo!(0.090, 12), &wicker_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    basket.add(&bottom);
    let rim = mesh(pg("KnittingBaskets__rim", torus_geo!(0.11, 0.006, 5, 14)), &wicker_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.055;
    basket.add(&rim);
    // Yarn ball inside — big sphere, saturated color.
    let yarn_colors: [u32; 7] = [0xa02020, 0x2060a0, 0x408060, 0xa060a0, 0xd0a040, 0xf0a0c0, 0x606060];
    let yarn_color = pick(&yarn_colors);
    let yarn_mat = lambert!({ color: yarn_color });
    let yarn = mesh(pg("KnittingBaskets__yarn", sphere_geo!(0.055, 8, 6)), &yarn_mat);
    yarn.position().set(0.020, 0.075, 0);
    basket.add(&yarn);
    // R701 subtle yarn ball roll animator — 40% chance.
    // Tiny rotation on Z + gentle micro-shift on X, mimics "just settled".
    if rnd() < 0.4 {
        let roll_phase = rnd() * PI * 2.0;
        let roll_phase2 = rnd() * PI * 2.0;
        let yarn_base_x = yarn.pos().x;
        let yarn2 = yarn.clone();
        g.animate(move |_g, _dt, t| {
            // Slow gentle rocking; keeps yarn ball visually alive.
            yarn2.rotation().z = (t * 0.6 + roll_phase).sin() * 0.10;
            yarn2.rotation().x = (t * 0.5 + roll_phase2).sin() * 0.08;
            yarn2.position().x = yarn_base_x + (t * 0.7 + roll_phase).sin() * 0.001;
        });
    }
    // Wound-thread cross-lines on the yarn ball (2 thin torus rings).
    for i in 0..2 {
        let wind = mesh(pg("KnittingBaskets__wind", torus_geo!(0.055, 0.002, 4, 12)), &yarn_mat);
        wind.position().copy(&yarn.pos());
        wind.rotation().x = (rnd() - 0.5) * PI;
        wind.rotation().z = (rnd() - 0.5) * PI;
        basket.add(&wind);
    }
    // Thread trail leaving the ball, arching out to floor — thin cylinder segments.
    let trail_segs = 5;
    let mut prev = v3(0.020, 0.075, 0);
    for i in 0..trail_segs {
        let nx = 0.020 + 0.055 + (i + 1) as f64 * 0.024;
        let ny = 0.075 - (i + 1) as f64 * 0.014;
        let nz = 0.010 + (i + 1) as f64 * 0.006;
        let next = v3(nx, 0.002f64.max(ny), nz);
        let mut mid = prev;
        mid.add(&next).multiply_scalar(0.5);
        let dir_len = next.distance_to(&prev);
        let seg = mesh(pg("KnittingBaskets__strand", cyl_geo!(0.0018, 0.0018, dir_len, 4)), &yarn_mat);
        seg.position().copy(&mid);
        let mut v = next;
        v.sub(&prev).normalize();
        // Rotate cylinder Y-axis to align w/ (prev→next).
        let y_axis = v3(0, 1, 0);
        let mut quat = Quat::IDENTITY;
        quat.set_from_unit_vectors(&y_axis, &v);
        seg.quaternion().copy(&quat);
        basket.add(&seg);
        prev = next;
    }
    // 2 knitting needles crossed inside — angled diagonally.
    let needle_mat = lambert!({ color: 0xc0a070 });
    for i in 0..2 {
        let needle = mesh(pg("KnittingBaskets__needle", cyl_geo!(0.002, 0.002, 0.22, 5)), &needle_mat);
        needle.rotation().z = PI / 2.0 + if i == 0 { -0.4 } else { 0.4 };
        needle.rotation().y = if i == 0 { 0.0 } else { PI / 8.0 };
        // Both low ends inside the basket; the second needle used to poke out through the wall.
        needle.position().set(if i == 0 { -0.030 } else { 0.030 }, 0.080 + i as f64 * 0.008, if i == 0 { -0.020 } else { -0.010 });
        basket.add(&needle);
        // Needle knob on the needle's upper end (tiny sphere), sticking out of the basket.
        let knob = mesh(pg("KnittingBaskets__knob", sphere_geo!(0.006, 5, 4)), &needle_mat);
        let mut needle_dir = v3(0, 1, 0);
        needle_dir.apply_euler(&needle.rot());
        if needle_dir.y < 0.0 { needle_dir.negate(); }
        let mut kp = needle.pos();
        kp.add_scaled_vector(&needle_dir, 0.108);
        knob.position().copy(&kp);
        basket.add(&knob);
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}

// ------------------------------------------------------------------ line 17707
// R682 hanging colander: 4% chance per level. Wall-mounted
// perforated metal bowl.
fn place_hanging_colanders(g: &mut Game, _dist: &[i32]) {
    g.pp.colander_positions.clear(); // R721 reset per level
    if rnd() < 0.96 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    // 3-color metal pool: brushed steel, copper, dark iron.
    let metal_colors: [u32; 3] = [0xa0a0a8, 0xc08050, 0x505050];
    let metal_mat = lambert!({ color: pick(&metal_colors) });
    let hole_mat = lambert!({ color: 0x0a0a0a });
    let col = group();
    let mount_y = 1.75;
    // Bowl body — inverted half-sphere (bottom half only, opening faces +Z away from wall).
    let bowl = mesh(pg("HangingColanders__bowl", sphere_geo!(0.12, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &metal_mat);
    bowl.rotation().x = PI; // Flip so cavity faces down/out.
    // Centre 0.115 off the mount: the rim's back (0.126) clears the wall face
    // (z -0.015) instead of passing 3 cm into it.
    bowl.position().set(0, mount_y, 0.115);
    col.add(&bowl);
    // Rim torus at the wide (rim) end.
    let rim = mesh(pg("HangingColanders__rim", torus_geo!(0.12, 0.006, 4, 16)), &metal_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().set(0, mount_y, 0.115);
    col.add(&rim);
    // Perforation dots on the outer surface (scattered).
    for i in 0..14 {
        let u = rnd() * PI * 2.0;
        let v = rnd() * (PI / 2.0 - 0.2) + 0.1;
        let rr = 0.121;
        let hx = u.cos() * v.sin() * rr;
        let hy = -v.cos() * rr;
        let hz = u.sin() * v.sin() * rr;
        let hole = mesh(circle_geo!(0.007, 5), &hole_mat);
        hole.position().set(hx, mount_y + hy, 0.115 + hz);
        hole.look_at(0, mount_y, 0.115);
        hole.rotate_y(PI); // face out of the bowl (lookAt pointed the front face inward: culled)
        col.add(&hole);
    }
    // Handle: short bar sticking out from the rim.
    let handle = mesh(pg("HangingColanders__handle", box_geo!(0.08, 0.010, 0.020)), &metal_mat);
    handle.position().set(0.14, mount_y, 0.115);
    col.add(&handle);
    // Hanging cord (short vertical line from top of rim to peg on wall).
    let cord_mat = lambert!({ color: 0x604020 });
    let cord = mesh(pg("HangingColanders__cord", cyl_geo!(0.002, 0.002, 0.10, 4)), &cord_mat);
    // From the peg (0.16) down to the rim's back point (y 0, z -0.005); it ended 6 cm short.
    cord.scale().y = 1.6;
    cord.position().set(0, mount_y + 0.08, 0.003);
    col.add(&cord);
    let peg = mesh(pg("HangingColanders__peg", cyl_geo!(0.006, 0.006, 0.010, 5)), &metal_mat);
    peg.rotation().x = PI / 2.0;
    peg.position().set(0, mount_y + 0.16, 0.005);
    col.add(&peg);
    col.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    col.position().set(px, 0, pz);
    g.world.add(&col);
    let p = col.pos();
    g.pp.colander_positions.push(v3(p.x, 0, p.z)); // R721
}

// ------------------------------------------------------------------ line 17782
// R683 hanging herb sieve: 4% chance per level. Wall-
// mounted round mesh strainer w/ handle.
fn place_herb_sieves(g: &mut Game, _dist: &[i32]) {
    g.pp.herb_sieve_positions.clear(); // R701 reset per level
    if rnd() < 0.96 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let metal_colors: [u32; 3] = [0xb0b0b8, 0xa08040, 0x606060];
    let metal_mat = lambert!({ color: pick(&metal_colors) });
    let mesh_mat = lambert!({ color: 0x282828, transparent: true, opacity: 0.7 });
    let sieve = group();
    let mount_y = 1.65;
    // Ring frame — flat torus facing wall.
    let ring = mesh(pg("HerbSieves__ring", torus_geo!(0.09, 0.006, 5, 20)), &metal_mat);
    ring.position().set(0, mount_y, 0.02);
    sieve.add(&ring);
    // Dark mesh disc (semi-transparent).
    let net = mesh(circle_geo!(0.088, 20), &mesh_mat);
    net.position().set(0, mount_y, 0.018);
    sieve.add(&net);
    // 4 crossing bars simulating woven mesh (2 horizontal + 2 vertical bar hints).
    for i in 0..4 {
        let bar = mesh(box_geo!(if i < 2 { 0.176 } else { 0.002 }, if i < 2 { 0.002 } else { 0.176 }, 0.002), &metal_mat);
        let off = ((i % 2) as f64 - 0.5) * 0.045;
        bar.position().set(if i < 2 { 0.0 } else { off }, mount_y + if i < 2 { off } else { 0.0 }, 0.019);
        sieve.add(&bar);
    }
    // Handle — flat bar extending to the side.
    let handle = mesh(pg("HerbSieves__handle", box_geo!(0.15, 0.014, 0.006)), &metal_mat);
    handle.position().set(0.165, mount_y, 0.02);
    sieve.add(&handle);
    let handle_hole_mat = lambert!({ color: 0x0a0a0a });
    let hole = mesh(circle_geo!(0.005, 6), &handle_hole_mat);
    hole.position().set(0.23, mount_y, 0.024);
    sieve.add(&hole);
    // Hanging cord + peg.
    let cord_mat = lambert!({ color: 0x604020 });
    let cord = mesh(pg("HangingColanders__cord", cyl_geo!(0.002, 0.002, 0.08, 4)), &cord_mat);
    cord.position().set(0, mount_y + 0.13, 0.01);
    sieve.add(&cord);
    let peg = mesh(pg("HangingColanders__peg", cyl_geo!(0.006, 0.006, 0.010, 5)), &metal_mat);
    peg.rotation().x = PI / 2.0;
    peg.position().set(0, mount_y + 0.17, 0.005);
    sieve.add(&peg);
    sieve.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    sieve.position().set(px, 0, pz);
    g.world.add(&sieve);
    let p = sieve.pos();
    g.pp.herb_sieve_positions.push(v3(p.x, 0, p.z)); // R701
}

// ------------------------------------------------------------------ line 17844
// R683 donut on napkin: 8% chance per level. Glazed torus
// donut resting on a folded paper napkin square.
fn place_donuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Napkin — pastel color pool.
    let napkin_colors: [u32; 5] = [0xf8f8f0, 0xf0e8d8, 0xf8e0e0, 0xe0f0e8, 0xf0e0f0];
    let napkin_mat = lambert!({ color: pick(&napkin_colors) });
    // Donut base — golden fried.
    let dough_mat = lambert!({ color: 0xc89050 });
    // Glaze — 5-color pool (glazed, chocolate, strawberry, matcha, maple).
    let glaze_colors: [u32; 5] = [0xf0e8c0, 0x603020, 0xd0a0a8, 0xa0c090, 0xa07030];
    let glaze_mat = lambert!({ color: pick(&glaze_colors) });
    let donut = group();
    // Napkin square (thin box).
    let napkin = mesh(pg("Donuts__napkin", box_geo!(0.20, 0.003, 0.20)), &napkin_mat);
    napkin.position().y = 0.0015;
    napkin.rotation().y = (rnd() - 0.5) * 0.3;
    donut.add(&napkin);
    // Folded corner detail (small triangle box tilted).
    let fold_geom = pg("Donuts__fold", box_geo!(0.06, 0.005, 0.06));
    let fold = mesh(fold_geom, &napkin_mat);
    // Lies on the napkin with its +X+Z tip on the napkin's corner, turned
    // with the napkin; the modelled corner curls up by itself (the old 0.4 rad
    // tilt sank one side 7 mm into the floor and it overhung the corner 1 cm).
    let nry = napkin.rot().y;
    let (nc, ns) = (nry.cos(), nry.sin());
    fold.position().set(0.07 * (nc + ns), 0.0053, 0.07 * (nc - ns));
    fold.rotation().y = nry;
    donut.add(&fold);
    // Donut dough (torus).
    let dough = mesh(pg("Donuts__dough", torus_geo!(0.045, 0.020, 6, 14)), &dough_mat);
    dough.rotation().x = PI / 2.0;
    dough.position().y = 0.023;
    donut.add(&dough);
    // Glaze top (slightly larger torus, thinner, glaze color).
    let glaze = mesh(pg("Donuts__glaze", torus_geo!(0.045, 0.018, 6, 14)), &glaze_mat);
    glaze.rotation().x = PI / 2.0;
    glaze.position().y = 0.033;
    donut.add(&glaze);
    // Optional sprinkles — 55% chance: 6-10 tiny colored boxes.
    if rnd() < 0.55 {
        let sprinkle_colors: [u32; 6] = [0xff4040, 0x40a0ff, 0xffe040, 0x40e080, 0xff80c0, 0xffffff];
        let count = 6 + (rnd() * 5.0).floor() as i32;
        for i in 0..count {
            let s_mat = lambert!({ color: pick(&sprinkle_colors) });
            let s = mesh(pg("Donuts__sprinkle", box_geo!(0.008, 0.002, 0.002)), &s_mat);
            let ang = rnd() * PI * 2.0;
            let rr = 0.045 + (rnd() - 0.5) * 0.010;
            // On the glaze's top surface at this radius (at 0.041 they were 8-10 mm
            // inside the icing).
            s.position().set(ang.cos() * rr, 0.0334 + (0.018 * 0.018 - (rr - 0.045).powi(2)).sqrt(), ang.sin() * rr);
            s.rotation().y = rnd() * PI * 2.0;
            donut.add(&s);
        }
    }
    donut.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    donut.rotation().y = rnd() * PI * 2.0;
    g.world.add(&donut);
}

// ------------------------------------------------------------------ line 17919
// R684 makeup sponge: 9% chance per level. Small teardrop
// beauty blender sponge — pastel color.
fn place_makeup_sponges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Sponge color pool — pastel.
    let sponge_colors: [u32; 6] = [0xf0a0a0, 0xa0d0f0, 0xf0c0e0, 0xd0e0a0, 0xf8c088, 0xe0c0f0];
    let sponge_mat = lambert!({ color: pick(&sponge_colors) });
    let sp = group();
    // Body — sphere w/ Y-scale for teardrop shape.
    let body = mesh(pg("MakeupSponges__body", sphere_geo!(0.030, 8, 6)), &sponge_mat);
    body.scale().y = 1.3;
    body.position().y = 0.035;
    sp.add(&body);
    // Pointed tip — small cone.
    let tip = mesh(pg("MakeupSponges__tip", cone_geo!(0.014, 0.024, 6)), &sponge_mat);
    tip.position().y = 0.078;
    sp.add(&tip);
    // Optional darker smudge patch — 55% chance: signifies "used".
    if rnd() < 0.55 {
        let smudge_colors: [u32; 4] = [0x704030, 0x8b6040, 0x604020, 0x503020];
        let smudge_mat = lambert!({ color: pick(&smudge_colors) });
        let smudge = mesh(circle_geo!(0.010, 6), &smudge_mat);
        smudge.position().set(0.020, 0.045, 0.020);
        smudge.look_at(0.10, 0.045, 0.10);
        sp.add(&smudge);
    }
    sp.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    sp.rotation().y = rnd() * PI * 2.0;
    // Random tilt as if fell over.
    sp.rotation().z = (rnd() - 0.5) * 0.5;
    g.world.add(&sp);
}

// ------------------------------------------------------------------ line 17966
// R684 framed embroidery hoop (wall): 4% chance per level.
// Wooden hoop stretching fabric w/ visible embroidered pattern.
game::lazy_static_tex!(embroidery_texture, canvas_tex(192, 192, |ctx, w, h| {
    // Fabric background — off-white/cream.
    ctx.set_fill_style("#f4ecd8");
    ctx.fill_rect(0, 0, w, h);
    // Faint weave texture — subtle vertical + horizontal light lines.
    ctx.set_stroke_style("rgba(200, 190, 170, 0.4)");
    ctx.set_line_width(1);
    let mut i = 0.0;
    while i < w {
        ctx.begin_path();
        ctx.move_to(i, 0); ctx.line_to(i, h);
        ctx.stroke();
        i += 3.0;
    }
    let mut j = 0.0;
    while j < h {
        ctx.begin_path();
        ctx.move_to(0, j); ctx.line_to(w, j);
        ctx.stroke();
        j += 3.0;
    }
    // Central embroidered flower — random color palette.
    let petal_palette: [[&str; 3]; 4] = [
        ["#a03060", "#f0c080", "#80a040"],
        ["#4060a0", "#f0d0a0", "#608040"],
        ["#804080", "#e0a0c0", "#508050"],
        ["#a04040", "#c0a060", "#406030"],
    ];
    let p = petal_palette[(rnd() * petal_palette.len() as f64).floor() as usize];
    let (cx, cy) = (w / 2.0, h / 2.0);
    // 6 petals — small ellipses arranged around center.
    ctx.set_fill_style(p[0]);
    for i in 0..6 {
        let ang = (i as f64 / 6.0) * PI * 2.0;
        ctx.save();
        ctx.translate(cx + ang.cos() * 26.0, cy + ang.sin() * 26.0);
        ctx.rotate(ang);
        ctx.begin_path();
        ctx.ellipse(0, 0, 18, 10, 0, 0, PI * 2.0);
        ctx.fill();
        ctx.restore();
    }
    // Center disc.
    ctx.set_fill_style(p[1]);
    ctx.begin_path();
    ctx.arc(cx, cy, 12, 0, PI * 2.0);
    ctx.fill();
    // Stem — curved leaf below.
    ctx.set_stroke_style(p[2]);
    ctx.set_line_width(4);
    ctx.begin_path();
    ctx.move_to(cx, cy + 12.0);
    ctx.quadratic_curve_to(cx - 20.0, cy + 60.0, cx - 8.0, cy + 84.0);
    ctx.stroke();
    // Small leaf.
    ctx.set_fill_style(p[2]);
    ctx.begin_path();
    ctx.ellipse(cx - 22.0, cy + 50.0, 12, 6, -0.6, 0, PI * 2.0);
    ctx.fill();
    // Frame border — visible stitching around edge.
    ctx.set_stroke_style("rgba(150, 130, 100, 0.6)");
    ctx.set_line_width(1);
    let mut a = 0.0;
    while a < PI * 2.0 {
        let r0 = w.min(h) * 0.44;
        let r1 = r0 - 4.0;
        ctx.begin_path();
        ctx.move_to(cx + a.cos() * r0, cy + a.sin() * r0);
        ctx.line_to(cx + a.cos() * r1, cy + a.sin() * r1);
        ctx.stroke();
        a += PI / 24.0;
    }
}));

// ------------------------------------------------------------------ line 18033
fn place_embroidery_hoops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let wood_colors: [u32; 4] = [0x8b5a2b, 0xa07040, 0x604020, 0xb08050];
    let wood_mat = lambert!({ color: pick(&wood_colors) });
    let fabric_mat = lambert!({ map: embroidery_texture() });
    let hoop = group();
    let mount_y = 1.60;
    // Outer wooden ring — flat torus.
    let outer_ring = mesh(pg("EmbroideryHoops__outer", torus_geo!(0.135, 0.010, 5, 24)), &wood_mat);
    outer_ring.position().set(0, mount_y, 0.014);
    hoop.add(&outer_ring);
    // Inner ring (very close to outer, simulates the two-ring clamp).
    let inner_ring = mesh(pg("EmbroideryHoops__inner", torus_geo!(0.125, 0.005, 4, 22)), &wood_mat);
    inner_ring.position().set(0, mount_y, 0.010);
    hoop.add(&inner_ring);
    // Fabric disc w/ embroidery pattern.
    let fabric = mesh(circle_geo!(0.128, 22), &fabric_mat);
    fabric.position().set(0, mount_y, 0.006);
    hoop.add(&fabric);
    // Small tightening screw at top.
    let screw = mesh(pg("EmbroideryHoops__screw", box_geo!(0.020, 0.014, 0.014)), &wood_mat);
    screw.position().set(0, mount_y + 0.14, 0.014);
    hoop.add(&screw);
    // Tiny hanging string above.
    let string_mat = lambert!({ color: 0x604020 });
    let st = mesh(pg("HangingColanders__cord", cyl_geo!(0.001, 0.001, 0.04, 3)), &string_mat);
    st.position().set(0, mount_y + 0.17, 0.008);
    hoop.add(&st);
    hoop.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    hoop.position().set(px, 0, pz);
    // Slight tilt.
    hoop.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&hoop);
}

// ------------------------------------------------------------------ line 18084
// R685 lipstick tube: 9% chance per level. Slim cylinder w/
// exposed lipstick nub in optional variants.
fn place_lipsticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Tube color pool — metallics.
    let tube_colors: [u32; 5] = [0x101010, 0xc09040, 0xa02040, 0x606060, 0xf0d0e0];
    let tube_mat = lambert!({ color: pick(&tube_colors) });
    // Lipstick color pool — vivid.
    let stick_colors: [u32; 5] = [0xa02020, 0x801020, 0xc04070, 0x603040, 0xd06040];
    let stick_mat = lambert!({ color: pick(&stick_colors) });
    let lip = group();
    // Main tube body.
    let body = mesh(pg("Lipsticks__body", cyl_geo!(0.012, 0.012, 0.055, 10)), &tube_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.012, 0);
    lip.add(&body);
    // Decorative band ring at midpoint.
    let band_mat = lambert!({ color: 0xd0b060 });
    let band = mesh(pg("Lipsticks__band", torus_geo!(0.0125, 0.0025, 4, 10)), &band_mat);
    band.rotation().y = PI / 2.0;
    band.position().set(0, 0.012, 0);
    lip.add(&band);
    // 60% chance: cap is off + lipstick extended.
    if rnd() < 0.6 {
        // Base tube (shorter, cap removed).
        // Twist ring — small torus at exposed end.
        let twist_ring = mesh(pg("Lipsticks__twist", torus_geo!(0.012, 0.003, 4, 10)), &tube_mat);
        twist_ring.rotation().y = PI / 2.0;
        twist_ring.position().set(0.028, 0.012, 0);
        lip.add(&twist_ring);
        // Extended lipstick nub — slanted cone.
        let stick = mesh(pg("Lipsticks__stick", cyl_geo!(0.008, 0.010, 0.024, 8)), &stick_mat);
        stick.rotation().z = PI / 2.0;
        stick.position().set(0.045, 0.012, 0);
        lip.add(&stick);
        // Angled cut at tip (small dark oval).
        let tip = mesh(circle_geo!(0.009, 8), &stick_mat);
        tip.rotation().y = PI / 2.0;
        tip.position().set(0.057, 0.012, 0);
        tip.rotation().x = 0.5;
        lip.add(&tip);
        // Loose cap lying beside — cylinder tilted.
        let cap = mesh(pg("Lipsticks__cap", cyl_geo!(0.014, 0.014, 0.030, 10)), &tube_mat);
        // Lying on its side on the floor, turned 0.4 rad (it stood tilted 0.4 rad, rim 5 mm into the floor).
        cap.rotation().set(0, 0.4, PI / 2.0);
        cap.position().set(-0.05, 0.014, 0.03);
        lip.add(&cap);
    } else {
        // Cap still on — extend body length.
        let cap = mesh(pg("Lipsticks__cap", cyl_geo!(0.014, 0.014, 0.030, 10)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.043, 0.012, 0);
        lip.add(&cap);
        let cap_top = mesh(circle_geo!(0.014, 10), &tube_mat);
        cap_top.rotation().y = PI / 2.0;
        cap_top.position().set(0.058, 0.012, 0);
        lip.add(&cap_top);
    }
    lip.position().set(g.wx(x) + (rnd() - 0.5) * 2.6, 0, g.wz(z) + (rnd() - 0.5) * 2.6);
    lip.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lip);
}

// ------------------------------------------------------------------ line 18162
// R685 framed cross-stitch sampler w/ alphabet (wall):
// 3% chance per level. Traditional embroidered alphabet
// on aida cloth in wooden frame.
game::lazy_static_tex!(cross_stitch_texture, canvas_tex(256, 192, |ctx, w, h| {
    // Aida cloth base — cream/tan.
    ctx.set_fill_style("#ecdfc0");
    ctx.fill_rect(0, 0, w, h);
    // Aida cloth grid — 8×8px cells w/ subtle grid lines.
    ctx.set_stroke_style("rgba(180, 160, 130, 0.35)");
    ctx.set_line_width(1);
    let mut i = 0.0;
    while i <= w {
        ctx.begin_path();
        ctx.move_to(i, 0); ctx.line_to(i, h);
        ctx.stroke();
        i += 8.0;
    }
    let mut j = 0.0;
    while j <= h {
        ctx.begin_path();
        ctx.move_to(0, j); ctx.line_to(w, j);
        ctx.stroke();
        j += 8.0;
    }
    // Alphabet — 2 rows of 13 chars in cross-stitch style.
    // Uses filled small squares (2×2 blocks) to simulate stitches.
    let row_colors = ["#a03050", "#40608a"]; // top row + bottom row
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.set_font("bold 18px monospace");
    for row in 0..2usize {
        ctx.set_fill_style(row_colors[row]);
        for i in 0..13usize {
            let k = row * 13 + i;
            let ch = &alphabet[k..k + 1];
            let cx = 20.0 + i as f64 * 17.0;
            let cy = 34.0 + row as f64 * 26.0;
            ctx.fill_text(ch, cx, cy);
        }
    }
    // Simple house motif center-bottom.
    let (cx, cy) = (w / 2.0, h * 0.75);
    // House body.
    ctx.set_fill_style("#a06040");
    ctx.fill_rect(cx - 22.0, cy - 4.0, 44, 24);
    // Roof triangle.
    ctx.set_fill_style("#603020");
    ctx.begin_path();
    ctx.move_to(cx - 26.0, cy - 4.0);
    ctx.line_to(cx, cy - 24.0);
    ctx.line_to(cx + 26.0, cy - 4.0);
    ctx.close_path();
    ctx.fill();
    // Door.
    ctx.set_fill_style("#402020");
    ctx.fill_rect(cx - 6.0, cy + 4.0, 12, 16);
    // 2 flanking flowers (small crosses + circle centers).
    for side in [-1.0, 1.0] {
        ctx.set_fill_style("#60a040");
        ctx.fill_rect(cx + side * 44.0 - 2.0, cy + 4.0, 4, 16);
        ctx.set_fill_style("#e04040");
        ctx.begin_path();
        ctx.arc(cx + side * 44.0, cy - 2.0, 6, 0, PI * 2.0);
        ctx.fill();
    }
    // Border stitch line.
    ctx.set_stroke_style("#603020");
    ctx.set_line_width(3);
    ctx.set_line_dash(&[6.0, 4.0]);
    ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
    ctx.set_line_dash(&[]);
    // Signed initials + year in tiny bottom-right cross-stitch.
    ctx.set_fill_style("#603030");
    ctx.set_font("10px monospace");
    ctx.set_text_align("right");
    ctx.fill_text("E.M. '87", w - 12.0, h - 10.0);
}));

// ------------------------------------------------------------------ line 18232
fn place_cross_stitch_samplers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let wood_colors: [u32; 4] = [0x8b5a2b, 0x604020, 0xa07040, 0x704a20];
    let frame_mat = lambert!({ color: pick(&wood_colors) });
    let cloth_mat = lambert!({ map: cross_stitch_texture() });
    let sampler = group();
    let mount_y = 1.55;
    // Frame — outer box.
    let (frame_w, frame_h) = (0.28, 0.22);
    let frame = mesh(pg("CrossStitchSamplers__frame", box_geo!(frame_w, frame_h, 0.016)), &frame_mat);
    frame.position().set(0, mount_y, 0.012);
    sampler.add(&frame);
    // Cloth face — slightly inset.
    let cloth = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &cloth_mat);
    cloth.position().set(0, mount_y, 0.021);
    sampler.add(&cloth);
    sampler.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    sampler.position().set(px, 0, pz);
    // Slight tilt.
    sampler.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&sampler);
}

// ------------------------------------------------------------------ line 18271
// R686 nail polish bottle: 8% chance per level. Small
// squat glass bottle w/ colored polish + black cap.
fn place_nail_polishes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Polish color pool — vivid + pastel mix.
    let polish_colors: [u32; 7] = [0xa02020, 0xf0a0c0, 0x203060, 0x804080, 0xd0a040, 0xf0e0e8, 0x101010];
    let polish_mat = lambert!({ color: pick(&polish_colors), transparent: true, opacity: 0.85 });
    // Glass (translucent tint over polish).
    let glass_mat = lambert!({ color: 0xd0d0e8, transparent: true, opacity: 0.35 });
    // Cap — usually black.
    let cap_mat = lambert!({ color: 0x0a0a0a });
    let bot = group();
    // Squat body — box (feels like classic OPI-style bottle).
    let body = mesh(box_geo!(0.040, 0.038, 0.040), &polish_mat);
    body.position().y = 0.019;
    bot.add(&body);
    // Glass overlay slightly larger.
    let glass = mesh(box_geo!(0.044, 0.040, 0.044), &glass_mat);
    glass.position().y = 0.020;
    bot.add(&glass);
    // Neck (short cylinder).
    let neck = mesh(pg("NailPolishes__neck", cyl_geo!(0.008, 0.010, 0.008, 8)), &cap_mat);
    neck.position().y = 0.044;
    bot.add(&neck);
    // Cap — tall cylinder.
    let cap = mesh(pg("NailPolishes__cap", cyl_geo!(0.012, 0.012, 0.036, 10)), &cap_mat);
    cap.position().y = 0.066;
    bot.add(&cap);
    // Tiny cap-top disc.
    let cap_top = mesh(circle_geo!(0.012, 10), &cap_mat);
    cap_top.rotation().x = -PI / 2.0;
    cap_top.position().y = 0.084;
    bot.add(&cap_top);
    // 30% chance: bottle tipped over.
    if rnd() < 0.3 {
        bot.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        bot.position().y = 0.022;
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.6;
    let py = bot.pos().y;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.6;
    bot.position().set(px, py, pz);
    bot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bot);
}

// ------------------------------------------------------------------ line 18329
// R686 framed child's crayon drawing (wall): 3% chance per level.
// Simple stick-figure family + house on manila paper.
game::lazy_static_tex!(child_drawing_texture, canvas_tex(240, 192, |ctx, w, h| {
    // Manila/cream paper background.
    ctx.set_fill_style("#f4e8c8");
    ctx.fill_rect(0, 0, w, h);
    // Subtle paper grain — scattered lighter dots.
    ctx.set_fill_style("rgba(220, 200, 160, 0.3)");
    for _ in 0..60 {
        ctx.begin_path();
        let (ax, ay, ar) = (rnd() * w, rnd() * h, 1.0 + rnd() * 1.5);
        ctx.arc(ax, ay, ar, 0, PI * 2.0);
        ctx.fill();
    }
    // Blue sky band top-third — crayon scribbles.
    ctx.set_stroke_style("#5090d0");
    ctx.set_line_width(3);
    for _ in 0..25 {
        ctx.begin_path();
        let sx = rnd() * w;
        let sy = rnd() * h * 0.28;
        ctx.move_to(sx, sy);
        let (lx, ly) = (sx + 20.0 + rnd() * 20.0, sy + (rnd() - 0.5) * 6.0);
        ctx.line_to(lx, ly);
        ctx.stroke();
    }
    // Sun (top-right yellow scribbled circle).
    ctx.set_stroke_style("#f0c040");
    ctx.set_line_width(4);
    let (sun_x, sun_y, sun_r) = (w * 0.82, h * 0.18, 20.0);
    ctx.begin_path();
    ctx.arc(sun_x, sun_y, sun_r, 0, PI * 2.0);
    ctx.stroke();
    // Sun rays.
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        ctx.begin_path();
        ctx.move_to(sun_x + a.cos() * (sun_r + 3.0), sun_y + a.sin() * (sun_r + 3.0));
        ctx.line_to(sun_x + a.cos() * (sun_r + 15.0), sun_y + a.sin() * (sun_r + 15.0));
        ctx.stroke();
    }
    // Green grass ground line.
    ctx.set_stroke_style("#60a040");
    ctx.set_line_width(5);
    for i in 0..30 {
        let gx = i as f64 * 8.0;
        ctx.begin_path();
        ctx.move_to(gx, h * 0.75);
        ctx.line_to(gx + 4.0, h * 0.68 + rnd() * 8.0);
        ctx.stroke();
    }
    ctx.begin_path();
    ctx.move_to(0, h * 0.75);
    ctx.line_to(w, h * 0.75);
    ctx.stroke();
    // House (center-left) — crayon square + triangle.
    let (hx, hy) = (w * 0.30, h * 0.55);
    ctx.set_stroke_style("#b04020");
    ctx.set_line_width(4);
    ctx.stroke_rect(hx - 30.0, hy, 60, 40);
    ctx.begin_path();
    ctx.move_to(hx - 34.0, hy);
    ctx.line_to(hx, hy - 30.0);
    ctx.line_to(hx + 34.0, hy);
    ctx.stroke();
    // Door.
    ctx.set_stroke_style("#602010");
    ctx.stroke_rect(hx - 8.0, hy + 18.0, 16, 22);
    // Window.
    ctx.set_stroke_style("#4080b0");
    ctx.stroke_rect(hx + 10.0, hy + 8.0, 12, 12);
    // Stick figure family — 3 figures right side.
    struct Fig { x: f64, head_r: f64, body_h: f64, color: &'static str, leg_y: f64 }
    let figs = [
        Fig { x: w * 0.62, head_r: 12.0, body_h: 30.0, color: "#a02040", leg_y: h * 0.75 }, // parent
        Fig { x: w * 0.72, head_r: 12.0, body_h: 30.0, color: "#4060a0", leg_y: h * 0.75 }, // parent
        Fig { x: w * 0.82, head_r: 8.0, body_h: 18.0, color: "#40a040", leg_y: h * 0.75 },  // child
    ];
    for f in &figs {
        ctx.set_stroke_style(f.color);
        ctx.set_line_width(3);
        // Head.
        ctx.begin_path();
        ctx.arc(f.x, f.leg_y - f.body_h - f.head_r, f.head_r, 0, PI * 2.0);
        ctx.stroke();
        // Smile.
        ctx.begin_path();
        ctx.arc(f.x, f.leg_y - f.body_h - f.head_r + 2.0, f.head_r * 0.55, 0.2, PI - 0.2);
        ctx.stroke();
        // Two dots for eyes.
        ctx.begin_path();
        ctx.arc(f.x - f.head_r * 0.4, f.leg_y - f.body_h - f.head_r - 2.0, 1.5, 0, PI * 2.0);
        ctx.arc(f.x + f.head_r * 0.4, f.leg_y - f.body_h - f.head_r - 2.0, 1.5, 0, PI * 2.0);
        ctx.set_fill_style(f.color);
        ctx.fill();
        // Body line.
        ctx.begin_path();
        ctx.move_to(f.x, f.leg_y - f.body_h);
        ctx.line_to(f.x, f.leg_y - 4.0);
        ctx.stroke();
        // Arms — out-and-down.
        ctx.begin_path();
        ctx.move_to(f.x - f.body_h * 0.5, f.leg_y - f.body_h * 0.6);
        ctx.line_to(f.x + f.body_h * 0.5, f.leg_y - f.body_h * 0.6);
        ctx.stroke();
        // Legs.
        ctx.begin_path();
        ctx.move_to(f.x, f.leg_y - 4.0);
        ctx.line_to(f.x - f.body_h * 0.35, f.leg_y + 8.0);
        ctx.move_to(f.x, f.leg_y - 4.0);
        ctx.line_to(f.x + f.body_h * 0.35, f.leg_y + 8.0);
        ctx.stroke();
    }
    // Child's signature — bottom left, wobbly.
    ctx.set_fill_style("#603030");
    ctx.set_font("bold 16px cursive");
    ctx.fill_text("by Emma, age 6", 10, h - 8.0);
}));

// ------------------------------------------------------------------ line 18442
fn place_child_drawings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    // Frame — plain wood or thin metal.
    let frame_colors: [u32; 4] = [0x8b5a2b, 0x604020, 0x505050, 0xa07040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let paper_mat = lambert!({ map: child_drawing_texture() });
    let dw = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.30, 0.24);
    let frame = mesh(pg("ChildDrawings__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    dw.add(&frame);
    let paper = mesh(plane_geo!(frame_w - 0.024, frame_h - 0.024), &paper_mat);
    paper.position().set(0, mount_y, 0.017);
    dw.add(&paper);
    dw.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    dw.position().set(px, 0, pz);
    dw.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&dw);
}

// ------------------------------------------------------------------ line 18479
// R687 perfume bottle w/ atomizer bulb: 6% chance per level.
// Squat glass bottle w/ neck + rubber squeeze bulb.
fn place_perfumes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Perfume liquid pool — amber/gold/rose tones.
    let liquid_colors: [u32; 5] = [0xd0a040, 0xf0d090, 0xf0c0a0, 0xa08040, 0xf0a080];
    let liquid_mat = lambert!({ color: pick(&liquid_colors), transparent: true, opacity: 0.75 });
    let glass_mat = lambert!({ color: 0xd8d0e8, transparent: true, opacity: 0.30 });
    // Bulb color pool — vintage rubber tones.
    let bulb_colors: [u32; 4] = [0xa02040, 0x603020, 0x201020, 0x4a3020];
    let bulb_mat = lambert!({ color: pick(&bulb_colors) });
    let metal_mat = lambert!({ color: 0xc0a060 });
    let p = group();
    // Bottle body — sphere flattened slightly for classic
    // Art Deco perfume silhouette.
    let body_geom = sphere_geo!(0.035, 12, 8);
    let body = mesh(&body_geom, &liquid_mat);
    body.scale().y = 0.85;
    body.position().y = 0.030;
    p.add(&body);
    // Glass overlay (slightly larger).
    let glass = mesh(&body_geom, &glass_mat);
    glass.scale().set(1.08, 0.90, 1.08);
    glass.position().y = 0.030;
    p.add(&glass);
    // Neck (short thin cylinder).
    let neck = mesh(pg("Perfumes__neck", cyl_geo!(0.008, 0.010, 0.016, 8)), &metal_mat);
    neck.position().y = 0.066;
    p.add(&neck);
    // Atomizer connector (horizontal metal ring).
    let connector = mesh(pg("Perfumes__ring", torus_geo!(0.010, 0.002, 4, 8)), &metal_mat);
    connector.rotation().x = PI / 2.0;
    connector.position().y = 0.077;
    p.add(&connector);
    // Squeeze bulb — larger squashed sphere off to the side.
    let bulb = mesh(pg("Perfumes__bulb", sphere_geo!(0.020, 8, 7)), &bulb_mat);
    bulb.scale().set(1.2, 1.0, 1.2);
    bulb.position().set(0.048, 0.075, 0);
    p.add(&bulb);
    // Thin flexible connector tube (short cylinder aligned).
    let tube_mat = lambert!({ color: 0x503020 });
    let tube = mesh(pg("Perfumes__tube", cyl_geo!(0.003, 0.003, 0.030, 5)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().set(0.025, 0.075, 0);
    p.add(&tube);
    // Small tassel hanging from bulb — 3-segment tiny cord.
    let tassel_mat = lambert!({ color: 0xd0a040 });
    for i in 0..3 {
        let seg = mesh(pg("Perfumes__cord", cyl_geo!(0.001, 0.001, 0.006, 3)), &tassel_mat);
        seg.position().set(0.048, 0.053 - i as f64 * 0.006, 0); // top segment 1 mm into the bulb (bottom y 0.055)
        p.add(&seg);
    }
    let tassel_knot = mesh(pg("Perfumes__tassel", sphere_geo!(0.003, 4, 4)), &tassel_mat);
    tassel_knot.position().set(0.048, 0.035, 0);
    p.add(&tassel_knot);
    p.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    p.rotation().y = rnd() * PI * 2.0;
    g.world.add(&p);
}

// ------------------------------------------------------------------ line 18557
// R687 framed family photograph w/ blurred figures (wall):
// 3% chance per level. Sepia-toned photo w/ soft indistinct
// human silhouettes — deeply liminal.
game::lazy_static_tex!(family_photo_texture, canvas_tex(224, 180, |ctx, w, h| {
    // Sepia background gradient.
    let grad = ctx.create_linear_gradient(0, 0, 0, h);
    grad.add_color_stop(0, "#c8a878");
    grad.add_color_stop(1, "#8a6848");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Faint radial vignette darkening corners.
    let rg = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.2, w / 2.0, h / 2.0, w * 0.7);
    rg.add_color_stop(0, "rgba(0,0,0,0)");
    rg.add_color_stop(1, "rgba(20,10,0,0.4)");
    ctx.set_fill_style(&rg);
    ctx.fill_rect(0, 0, w, h);
    // Subtle film grain — many tiny dark dots.
    ctx.set_fill_style("rgba(20,10,0,0.15)");
    for _ in 0..240 {
        ctx.begin_path();
        let (ax, ay) = (rnd() * w, rnd() * h);
        ctx.arc(ax, ay, 1, 0, PI * 2.0);
        ctx.fill();
    }
    // Blurred wall/backdrop hint — horizontal darker band mid-height.
    ctx.set_fill_style("rgba(80,50,30,0.25)");
    ctx.fill_rect(0, h * 0.55, w, h * 0.20);
    // 3-4 blurred figures — soft radial gradients w/ elongated form.
    let fig_count = 3 + (rnd() * 2.0).floor() as i32;
    let center_y = h * 0.55;
    for i in 0..fig_count {
        let cx = w * (0.20 + i as f64 / (fig_count as f64 - 0.4) * 0.6);
        let is_child = i == fig_count - 1 && rnd() < 0.6;
        let size = if is_child { 25.0 } else { 40.0 };
        // Head (softer, above shoulders).
        let head_grad = ctx.create_radial_gradient(cx, center_y - size * 0.9, 3, cx, center_y - size * 0.9, size * 0.35);
        head_grad.add_color_stop(0, "rgba(70,45,25,0.85)");
        head_grad.add_color_stop(0.6, "rgba(80,55,35,0.5)");
        head_grad.add_color_stop(1, "rgba(80,55,35,0)");
        ctx.set_fill_style(&head_grad);
        ctx.begin_path();
        ctx.arc(cx, center_y - size * 0.9, size * 0.35, 0, PI * 2.0);
        ctx.fill();
        // Body (larger, tapered — elongated ellipse).
        let body_grad = ctx.create_radial_gradient(cx, center_y - 4.0, 4, cx, center_y - 4.0, size * 0.7);
        body_grad.add_color_stop(0, "rgba(60,35,20,0.85)");
        body_grad.add_color_stop(0.7, "rgba(80,55,35,0.35)");
        body_grad.add_color_stop(1, "rgba(80,55,35,0)");
        ctx.set_fill_style(&body_grad);
        ctx.begin_path();
        ctx.ellipse(cx, center_y - 4.0, size * 0.5, size * 0.85, 0, 0, PI * 2.0);
        ctx.fill();
    }
    // Bottom margin — white/cream border of old photo.
    ctx.set_fill_style("#f0e0c0");
    ctx.fill_rect(0, h - 20.0, w, 20);
    ctx.fill_rect(0, 0, w, 8);
    ctx.fill_rect(0, 0, 8, h);
    ctx.fill_rect(w - 8.0, 0, 8, h);
    // Handwritten caption on bottom margin.
    ctx.set_fill_style("#402010");
    ctx.set_font("11px cursive");
    ctx.set_text_align("center");
    let captions = ["Summer 1976", "The family, 1982", "Home, spring '79", "Us, before."];
    let cap = captions[(rnd() * captions.len() as f64).floor() as usize];
    ctx.fill_text(cap, w / 2.0, h - 6.0);
    // Slight fold/crease diagonal — thin brown line.
    ctx.set_stroke_style("rgba(40,20,0,0.2)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(w * 0.15, h * 0.10);
    ctx.line_to(w * 0.85, h * 0.90);
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 18627
fn place_family_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let frame_colors: [u32; 4] = [0x604020, 0x8b5a2b, 0x505050, 0xd0a040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: family_photo_texture() });
    let fp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.24, 0.19);
    let frame = mesh(pg("FamilyPhotos__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    fp.add(&frame);
    let photo = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &photo_mat);
    photo.position().set(0, mount_y, 0.019);
    fp.add(&photo);
    fp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    fp.position().set(px, 0, pz);
    fp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&fp);
}

// ------------------------------------------------------------------ line 18663
// R688 hairbrush w/ tangled hair strands: 7% chance per level.
// Oval brush w/ bristles + optional wisp of dark hair.
fn place_hairbrushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Brush body color pool.
    let body_colors: [u32; 6] = [0x101010, 0xa02040, 0x6040a0, 0xd0a040, 0xe0c0e0, 0x606060];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let bristle_mat = lambert!({ color: 0x0a0a0a });
    // Hair strand color pool.
    let hair_colors: [u32; 5] = [0x1a1010, 0x402010, 0x604020, 0xa07040, 0xd0a060];
    let hair_mat = lambert!({ color: pick(&hair_colors) });
    let brush = group();
    // Oval brush pad (paddle) — flat ellipse via scaled cylinder.
    let pad = mesh(pg("Hairbrushes__pad", cyl_geo!(0.045, 0.045, 0.014, 16)), &body_mat);
    // Lies flat, cushion up (y 0.007-0.021); oval along Z like the bristle grid.
    pad.scale().z = 1.4; // Elongate to oval.
    pad.position().set(0, 0.014, 0);
    brush.add(&pad);
    // Handle — cylinder extending out.
    let handle = mesh(pg("Hairbrushes__handle", cyl_geo!(0.012, 0.015, 0.10, 8)), &body_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.093, 0.014, 0); // near end 2 mm into the pad rim (x -0.045)
    brush.add(&handle);
    // Handle knob at end.
    let knob = mesh(pg("Hairbrushes__knob", sphere_geo!(0.016, 6, 5)), &body_mat);
    knob.scale().set_scalar(0.875); // r 14 mm: rests on the floor, on the handle axis
    knob.position().set(-0.155, 0.014, 0);
    brush.add(&knob);
    // Bristles — grid of tiny cylinders on top of pad (5x7 grid = 35).
    for iy in -3..=3 {
        for ix in -2..=2 {
            let bx = ix as f64 * 0.014;
            let by = iy as f64 * 0.014;
            // Reject corners for oval mask.
            if (bx * bx * 1.4 + by * by * 0.7) > 0.045 * 0.045 * 1.15 { continue; }
            let bristle = mesh(pg("Hairbrushes__bristle", cyl_geo!(0.001, 0.001, 0.012, 3)), &bristle_mat);
            bristle.position().set(bx, 0.026, by);
            brush.add(&bristle);
        }
    }
    // 60% chance: tangled hair strands caught in bristles.
    if rnd() < 0.6 {
        // Use oriented-segment-chain pattern (from R682) to draw curly hair.
        let strand_count = 3 + (rnd() * 3.0).floor() as i32;
        for s in 0..strand_count {
            // Start point near random spot on pad top.
            let ang_start = rnd() * PI * 2.0;
            let r_start = rnd() * 0.035;
            let mut prev = v3(ang_start.cos() * r_start, 0.030, ang_start.sin() * r_start * 1.3);
            // Chain 6 segments arcing outward from pad.
            let mut dir_ang = ang_start + (rnd() - 0.5) * 1.5;
            for i in 0..6 {
                let step = 0.008 + rnd() * 0.008;
                dir_ang += (rnd() - 0.5) * 1.2; // Wander.
                let mut next = prev;
                next.add(&v3(dir_ang.cos() * step, -0.001 * i as f64, dir_ang.sin() * step));
                let dir_len = next.distance_to(&prev);
                let mut mid = prev;
                mid.add(&next).multiply_scalar(0.5);
                let seg = mesh(cyl_geo!(0.0008, 0.0008, dir_len, 3), &hair_mat);
                seg.position().copy(&mid);
                let mut v = next;
                v.sub(&prev).normalize();
                seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
                brush.add(&seg);
                prev = next;
            }
        }
    }
    brush.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}

// ------------------------------------------------------------------ line 18755
// R688 framed baby photo w/ pastel border (wall): 2.5% chance
// per level. Small square photo w/ soft focus baby silhouette
// + pastel decorated frame.
game::lazy_static_tex!(baby_photo_texture, canvas_tex(180, 180, |ctx, w, h| {
    // Soft cream background gradient (very light).
    let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 20, w / 2.0, h / 2.0, w * 0.7);
    grad.add_color_stop(0, "#fff0e0");
    grad.add_color_stop(1, "#f0d8c0");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Subtle grain.
    ctx.set_fill_style("rgba(200,170,140,0.15)");
    for _ in 0..120 {
        ctx.begin_path();
        let (ax, ay) = (rnd() * w, rnd() * h);
        ctx.arc(ax, ay, 1, 0, PI * 2.0);
        ctx.fill();
    }
    // Central baby silhouette — soft round head w/ hint of body.
    let (cx, cy) = (w / 2.0, h * 0.55);
    // Head — large soft radial (skin tone).
    let head_grad = ctx.create_radial_gradient(cx, cy - 25.0, 5, cx, cy - 25.0, 40);
    head_grad.add_color_stop(0, "rgba(230,180,140,0.95)");
    head_grad.add_color_stop(0.55, "rgba(210,160,120,0.6)");
    head_grad.add_color_stop(1, "rgba(210,160,120,0)");
    ctx.set_fill_style(&head_grad);
    ctx.begin_path();
    ctx.arc(cx, cy - 25.0, 40, 0, PI * 2.0);
    ctx.fill();
    // Body — softer blanket.
    let body_grad = ctx.create_radial_gradient(cx, cy + 20.0, 5, cx, cy + 20.0, 55);
    body_grad.add_color_stop(0, "rgba(240,220,240,0.85)");
    body_grad.add_color_stop(0.6, "rgba(230,210,230,0.5)");
    body_grad.add_color_stop(1, "rgba(230,210,230,0)");
    ctx.set_fill_style(&body_grad);
    ctx.begin_path();
    ctx.ellipse(cx, cy + 20.0, 50, 40, 0, 0, PI * 2.0);
    ctx.fill();
    // Tiny eye smudges + smile.
    ctx.set_fill_style("rgba(60,40,30,0.5)");
    ctx.begin_path();
    ctx.arc(cx - 10.0, cy - 32.0, 2, 0, PI * 2.0);
    ctx.arc(cx + 10.0, cy - 32.0, 2, 0, PI * 2.0);
    ctx.fill();
    ctx.set_stroke_style("rgba(160,80,60,0.4)");
    ctx.set_line_width(2);
    ctx.begin_path();
    ctx.arc(cx, cy - 22.0, 6, 0.2, PI - 0.2);
    ctx.stroke();
    // Pastel decorative border — hearts + circles.
    let border_palettes: [[&str; 3]; 3] = [
        ["#f0a0c0", "#c0d0f0", "#e0c0f0"],
        ["#a0d0f0", "#c0f0d0", "#f0e0a0"],
        ["#f0c0a0", "#c0e0e0", "#e0a0c0"],
    ];
    let p = border_palettes[(rnd() * border_palettes.len() as f64).floor() as usize];
    // Solid pastel border.
    ctx.set_fill_style(p[0]);
    ctx.fill_rect(0, 0, w, 12);
    ctx.fill_rect(0, h - 12.0, w, 12);
    ctx.fill_rect(0, 0, 12, h);
    ctx.fill_rect(w - 12.0, 0, 12, h);
    // Decorative motifs — small hearts + circles alternating on border.
    let motifs = 8;
    for side in 0..4 {
        for i in 0..motifs {
            let t = (i as f64 + 0.5) / motifs as f64;
            let (mx, my);
            if side == 0 { mx = t * w; my = 6.0; }
            else if side == 1 { mx = w - 6.0; my = t * h; }
            else if side == 2 { mx = t * w; my = h - 6.0; }
            else { mx = 6.0; my = t * h; }
            ctx.set_fill_style(p[1 + (i % 2) as usize]);
            if i % 2 == 0 {
                // Heart-ish shape (simplified — two small circles).
                ctx.begin_path();
                ctx.arc(mx - 2.0, my, 2, 0, PI * 2.0);
                ctx.arc(mx + 2.0, my, 2, 0, PI * 2.0);
                ctx.fill();
            } else {
                ctx.begin_path();
                ctx.arc(mx, my, 2.5, 0, PI * 2.0);
                ctx.fill();
            }
        }
    }
    // Baby name + date caption on cream area bottom.
    let names = ["Baby Anna", "Little Sam", "Baby Grace", "Our Lily", "Baby Noah"];
    let dates = ["2 mo.", "3 mo.", "Christmas '84", "April 1991", "newborn"];
    let nm = names[(rnd() * names.len() as f64).floor() as usize];
    let dt = dates[(rnd() * dates.len() as f64).floor() as usize];
    ctx.set_fill_style("#603030");
    ctx.set_font("italic 12px cursive");
    ctx.set_text_align("center");
    ctx.fill_text(&format!("{}, {}", nm, dt), w / 2.0, h - 22.0);
}));

// ------------------------------------------------------------------ line 18847
fn place_baby_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let frame_colors: [u32; 4] = [0xf0d8e8, 0xd8e8f0, 0xf0f0d0, 0xf0d0d0];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: baby_photo_texture() });
    let bp = group();
    let mount_y = 1.55;
    let frame_size = 0.20;
    let frame = mesh(pg("BabyPhotos__frame", box_geo!(frame_size, frame_size, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    bp.add(&frame);
    let photo = mesh(plane_geo!(frame_size - 0.018, frame_size - 0.018), &photo_mat);
    photo.position().set(0, mount_y, 0.019);
    bp.add(&photo);
    bp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    bp.position().set(px, 0, pz);
    bp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&bp);
}

// ------------------------------------------------------------------ line 18884
// R689 small mirror on stand w/ optional smudge (floor):
// 6% chance per level. Vanity/table mirror on hinged frame
// w/ 50% chance of visible fingerprint/lipstick smudge.
fn place_standing_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    // Frame color pool.
    let frame_colors: [u32; 5] = [0xd0a040, 0x606060, 0xf0e0d0, 0xa04060, 0x8b5a2b];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    // Mirror glass — bluish-tinted reflective look.
    let mirror_mat = basic_mat!({ color: 0xd8e8f0 });
    let back_mat = lambert!({ color: 0x302030 });
    let mir = group();
    // Base — wide flat oval disc.
    let base = mesh(pg("StandingMirrors__base", cyl_geo!(0.055, 0.065, 0.010, 16)), &frame_mat);
    base.position().y = 0.005;
    mir.add(&base);
    // Vertical stem (thin cylinder up from base).
    let stem = mesh(pg("StandingMirrors__stem", cyl_geo!(0.005, 0.005, 0.070, 6)), &frame_mat);
    // y 0.008-0.058: from inside the base up into the tilted frame's bottom
    // rim (tube bottom y 0.055, z 0.008), below the glass and back plate.
    stem.scale().y = 0.714;
    stem.position().set(0, 0.033, 0.006);
    mir.add(&stem);
    // Mirror oval frame — thin torus tilted slightly.
    let tilt = -0.15;
    let frame_outer = mesh(pg("StandingMirrors__frame", torus_geo!(0.055, 0.006, 5, 20)), &frame_mat);
    frame_outer.rotation().x = tilt;
    frame_outer.position().y = 0.115;
    mir.add(&frame_outer);
    // Mirror face — thin disc, oval-shape via Y-scale.
    let mirror = mesh(circle_geo!(0.053, 20), &mirror_mat);
    mirror.rotation().x = tilt;
    mirror.position().y = 0.115;
    mirror.position().z = 0.002;
    mir.add(&mirror);
    // Back plate — dark oval behind.
    let back = mesh(pg("StandingMirrors__back", circle_geo!(0.055, 20)), &back_mat);
    back.rotation().x = tilt;
    back.rotation().y = PI;
    back.position().y = 0.115;
    back.position().z = -0.002;
    mir.add(&back);
    // 2 hinge pegs (small dots on frame sides).
    for side in [-1.0, 1.0] {
        let peg = mesh(pg("StandingMirrors__peg", cyl_geo!(0.005, 0.005, 0.010, 5)), &frame_mat);
        peg.rotation().z = PI / 2.0;
        peg.position().set(side * 0.062, 0.115, 0);
        mir.add(&peg);
    }
    // 50% chance: smudge on mirror surface.
    if rnd() < 0.5 {
        // Smudge color pool: lipstick (red), fingerprint (dark), foundation (tan).
        let smudge_colors: [u32; 4] = [0xa03040, 0x604020, 0xc0a080, 0x805050];
        let smudge_mat = basic_mat!({ color: pick(&smudge_colors), transparent: true, opacity: 0.55 });
        // 3-6 elliptical smudge blobs scattered on mirror face.
        let blob_count = 3 + (rnd() * 4.0).floor() as i32;
        for i in 0..blob_count {
            let rr = 0.010 + rnd() * 0.010;
            let blob = mesh(circle_geo!(rr, 6), &smudge_mat);
            let ang = rnd() * PI * 2.0;
            let dd = rnd() * 0.030;
            blob.position().set(ang.cos() * dd, 0, ang.sin() * dd);
            blob.rotation().x = tilt;
            // Offset slightly above mirror surface.
            let mirror_pos = v3(ang.cos() * dd, 0.115, 0.004 + ang.sin() * dd);
            blob.position().copy(&mirror_pos);
            blob.rotation().x = tilt;
            // Random rotation for organic look.
            blob.rotation().z = rnd() * PI * 2.0;
            mir.add(&blob);
        }
    }
    mir.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    mir.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mir);
}

// ------------------------------------------------------------------ line 18976
// R689 framed black-and-white wedding photo (wall):
// 2.5% chance per level. Vintage formal wedding portrait
// w/ soft blurred figures.
game::lazy_static_tex!(wedding_photo_texture, canvas_tex(200, 240, |ctx, w, h| {
    // B&W gradient background — light gray on top, darker below.
    let grad = ctx.create_linear_gradient(0, 0, 0, h);
    grad.add_color_stop(0, "#e0e0e0");
    grad.add_color_stop(1, "#909090");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Radial vignette darkening corners.
    let rg = ctx.create_radial_gradient(w / 2.0, h * 0.4, 20, w / 2.0, h * 0.4, w * 0.8);
    rg.add_color_stop(0, "rgba(0,0,0,0)");
    rg.add_color_stop(1, "rgba(0,0,0,0.4)");
    ctx.set_fill_style(&rg);
    ctx.fill_rect(0, 0, w, h);
    // Fine film grain.
    ctx.set_fill_style("rgba(0,0,0,0.15)");
    for _ in 0..300 {
        ctx.begin_path();
        let (ax, ay) = (rnd() * w, rnd() * h);
        ctx.arc(ax, ay, 1, 0, PI * 2.0);
        ctx.fill();
    }
    // 2 formal figures center — bride (left) + groom (right).
    // Bride: wider silhouette with veil trailing down.
    let (bride_x, bride_y) = (w * 0.36, h * 0.55);
    // Veil area (large soft aura above and around bride's head/shoulders).
    let veil_grad = ctx.create_radial_gradient(bride_x, bride_y - 50.0, 5, bride_x, bride_y - 50.0, 60);
    veil_grad.add_color_stop(0, "rgba(230,230,230,0.7)");
    veil_grad.add_color_stop(1, "rgba(230,230,230,0)");
    ctx.set_fill_style(&veil_grad);
    ctx.begin_path();
    ctx.arc(bride_x, bride_y - 50.0, 60, 0, PI * 2.0);
    ctx.fill();
    // Bride head.
    let bride_head = ctx.create_radial_gradient(bride_x, bride_y - 55.0, 3, bride_x, bride_y - 55.0, 22);
    bride_head.add_color_stop(0, "rgba(60,55,50,0.9)");
    bride_head.add_color_stop(0.55, "rgba(90,80,70,0.5)");
    bride_head.add_color_stop(1, "rgba(90,80,70,0)");
    ctx.set_fill_style(&bride_head);
    ctx.begin_path();
    ctx.arc(bride_x, bride_y - 55.0, 22, 0, PI * 2.0);
    ctx.fill();
    // Bride dress (wide flowing white).
    let bride_body = ctx.create_radial_gradient(bride_x, bride_y + 25.0, 8, bride_x, bride_y + 25.0, 60);
    bride_body.add_color_stop(0, "rgba(210,210,210,0.85)");
    bride_body.add_color_stop(0.55, "rgba(180,180,180,0.5)");
    bride_body.add_color_stop(1, "rgba(180,180,180,0)");
    ctx.set_fill_style(&bride_body);
    ctx.begin_path();
    ctx.ellipse(bride_x, bride_y + 25.0, 44, 65, 0, 0, PI * 2.0);
    ctx.fill();
    // Groom (right) — dark suit.
    let (groom_x, groom_y) = (w * 0.62, h * 0.55);
    // Groom head.
    let groom_head = ctx.create_radial_gradient(groom_x, groom_y - 50.0, 3, groom_x, groom_y - 50.0, 20);
    groom_head.add_color_stop(0, "rgba(60,50,45,0.9)");
    groom_head.add_color_stop(0.6, "rgba(80,65,55,0.5)");
    groom_head.add_color_stop(1, "rgba(80,65,55,0)");
    ctx.set_fill_style(&groom_head);
    ctx.begin_path();
    ctx.arc(groom_x, groom_y - 50.0, 20, 0, PI * 2.0);
    ctx.fill();
    // Groom suit (dark suit — narrower).
    let groom_body = ctx.create_radial_gradient(groom_x, groom_y + 15.0, 5, groom_x, groom_y + 15.0, 45);
    groom_body.add_color_stop(0, "rgba(30,25,25,0.9)");
    groom_body.add_color_stop(0.6, "rgba(50,45,40,0.55)");
    groom_body.add_color_stop(1, "rgba(50,45,40,0)");
    ctx.set_fill_style(&groom_body);
    ctx.begin_path();
    ctx.ellipse(groom_x, groom_y + 15.0, 26, 55, 0, 0, PI * 2.0);
    ctx.fill();
    // Bouquet — subtle lighter blob at bride's waist.
    let bouquet = ctx.create_radial_gradient(bride_x + 10.0, bride_y + 5.0, 3, bride_x + 10.0, bride_y + 5.0, 14);
    bouquet.add_color_stop(0, "rgba(200,200,200,0.7)");
    bouquet.add_color_stop(1, "rgba(200,200,200,0)");
    ctx.set_fill_style(&bouquet);
    ctx.begin_path();
    ctx.arc(bride_x + 10.0, bride_y + 5.0, 14, 0, PI * 2.0);
    ctx.fill();
    // White border margins (matte).
    ctx.set_fill_style("#f0ece0");
    ctx.fill_rect(0, 0, w, 12);
    ctx.fill_rect(0, h - 28.0, w, 28);
    ctx.fill_rect(0, 0, 12, h);
    ctx.fill_rect(w - 12.0, 0, 12, h);
    // Handwritten caption bottom — date/dedication.
    let captions = ["June 12, 1958", "Our Wedding Day, '62", "Charles & Marie, 1971", "Forever, Sept. '49"];
    let cap = captions[(rnd() * captions.len() as f64).floor() as usize];
    ctx.set_fill_style("#402020");
    ctx.set_font("italic 12px cursive");
    ctx.set_text_align("center");
    ctx.fill_text(cap, w / 2.0, h - 10.0);
    // Small age crease at one corner.
    ctx.set_stroke_style("rgba(60,50,40,0.15)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(w * 0.85, h * 0.05);
    ctx.line_to(w, h * 0.15);
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 19079
fn place_wedding_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = pick(&wt);
    let frame_colors: [u32; 4] = [0xd0a040, 0x604020, 0x808080, 0xf0e0c0];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: wedding_photo_texture() });
    let wp = group();
    let mount_y = 1.62;
    let (frame_w, frame_h) = (0.20, 0.24);
    let frame = mesh(pg("WeddingPhotos__frame", box_geo!(frame_w, frame_h, 0.016)), &frame_mat);
    frame.position().set(0, mount_y, 0.012);
    wp.add(&frame);
    let photo = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &photo_mat);
    photo.position().set(0, mount_y, 0.021);
    wp.add(&photo);
    wp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    wp.position().set(px, 0, pz);
    wp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&wp);
}

// ------------------------------------------------------------------ line 19116
// R690 donut w/ single bite missing (floor): 5% chance per level.
// Twin bimodal variant of R683 donut — partial ring w/ bite gap +
// crumbs. Uses TorusGeometry theta gap.
fn place_bitten_donuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = pick(&open);
    let napkin_colors: [u32; 5] = [0xf8f8f0, 0xf0e8d8, 0xf8e0e0, 0xe0f0e8, 0xf0e0f0];
    let napkin_mat = lambert!({ color: pick(&napkin_colors) });
    let dough_mat = lambert!({ color: 0xc89050 });
    let glaze_colors: [u32; 5] = [0xf0e8c0, 0x603020, 0xd0a0a8, 0xa0c090, 0xa07030];
    let glaze_mat = lambert!({ color: pick(&glaze_colors) });
    let bitten_donut = group();
    // Napkin square (like R683 donut, more crumpled).
    let napkin = mesh(pg("BittenDonuts__napkin", box_geo!(0.20, 0.003, 0.20)), &napkin_mat);
    napkin.position().y = 0.0015;
    napkin.rotation().y = (rnd() - 0.5) * 0.4;
    bitten_donut.add(&napkin);
    // Multiple folded corners — 2 wrinkles for "used" look.
    for i in 0..2 {
        let fold = mesh(pg("BittenDonuts__wrinkle", box_geo!(0.05, 0.005, 0.05)), &napkin_mat);
        fold.position().set((rnd() - 0.5) * 0.15, 0.005, (rnd() - 0.5) * 0.15);
        // Lies flat on the napkin; the modelled ridge is the wrinkle, and the
        // old 0.3-0.6 rad tilt draw now sets its height (the tilt sank one side
        // up to 1 cm into the floor).
        let pinch = 0.3 + rnd() * 0.3;
        fold.scale().y = 0.55 + pinch;
        let sy = fold.scl().y;
        fold.position().y = 0.0028 + 0.0025 * sy;
        let nry = napkin.rot().y;
        fold.rotation().y = nry + (rnd() - 0.5) * 0.4;
        bitten_donut.add(&fold);
    }
    // Donut dough — partial torus (bite gap = ~PI/3.5 = ~50°).
    let bite_arc = PI * 2.0 - PI / 3.5;
    let dough = mesh(pg("BittenDonuts__dough", torus_geo!(0.045, 0.020, 6, 14, bite_arc)), &dough_mat);
    dough.rotation().x = PI / 2.0;
    dough.rotation().z = rnd() * PI * 2.0;
    dough.position().y = 0.023;
    bitten_donut.add(&dough);
    // Glaze — matching partial torus above.
    let glaze = mesh(pg("BittenDonuts__glaze", torus_geo!(0.045, 0.018, 6, 14, bite_arc)), &glaze_mat);
    glaze.rotation().x = PI / 2.0;
    glaze.rotation().z = dough.rot().z;
    glaze.position().y = 0.033;
    bitten_donut.add(&glaze);
    // Bite-mark inner surface — small darker semi-sphere at torn end.
    let bite_inner_mat = lambert!({ color: 0xa07040 });
    // Compute bite gap position.
    let bite_ang = dough.rot().z + bite_arc + PI / 7.0;
    let bite_inner = mesh(pg("BittenDonuts__bite", sphere_geo!(0.014, 6, 4)), &bite_inner_mat);
    bite_inner.position().set(bite_ang.cos() * 0.045, 0.028, bite_ang.sin() * 0.045);
    bitten_donut.add(&bite_inner);
    // Scattered crumbs — 5-8 tiny boxes on napkin.
    let crumb_count = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..crumb_count {
        let crumb_size = 0.004 + rnd() * 0.005;
        let crumb = mesh(pg("BittenDonuts__crumb", box_geo!(crumb_size, crumb_size, crumb_size)), &dough_mat);
        crumb.position().set((rnd() - 0.5) * 0.15, 0.004 + rnd() * 0.002, (rnd() - 0.5) * 0.15);
        crumb.rotation().y = rnd() * PI * 2.0;
        bitten_donut.add(&crumb);
    }
    // Maybe 1-2 sprinkle crumbs too.
    if rnd() < 0.5 {
        let sprinkle_colors: [u32; 6] = [0xff4040, 0x40a0ff, 0xffe040, 0x40e080, 0xff80c0, 0xffffff];
        for i in 0..3 {
            let s_mat = lambert!({ color: pick(&sprinkle_colors) });
            let s = mesh(pg("Donuts__sprinkle", box_geo!(0.008, 0.002, 0.002)), &s_mat);
            s.position().set((rnd() - 0.5) * 0.15, 0.004, (rnd() - 0.5) * 0.15);
            s.rotation().y = rnd() * PI * 2.0;
            bitten_donut.add(&s);
        }
    }
    bitten_donut.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bitten_donut.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bitten_donut);
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
