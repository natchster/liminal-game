//! Dressing-table, party-costume and percussion props (R438-R444).
use crate::{buf_geo, open_tiles, pick, set_rot_order};
use game::prelude::*;

/// R444 small jewelry box: 6% chance per level. Small hinged box
/// with visible gems inside (60% open). Dressing-table family.
pub fn place_small_jewelry_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let box_colors = [0x6a3820, 0x8a5030, 0x201a10, 0xd8b040];
    let box_mat = lambert!({ color: pick(&box_colors) });
    let felt_mat = lambert!({ color: 0x8a2040 });
    let gem_colors = [0xd82060, 0x2080d8, 0x30c060, 0xd050a0, 0xd8b040];
    let jewel = group();
    let is_open = rnd() < 0.60;
    // Body: cube.
    let body = mesh(pg("SmallJewelryBoxes__body", box_geo!(0.10, 0.04, 0.07)), &box_mat);
    body.position().y = 0.020;
    jewel.add(&body);
    if is_open {
        // Felt inner floor.
        let felt = mesh(pg("SmallJewelryBoxes__felt", plane_geo!(0.09, 0.06)), &felt_mat);
        felt.rotation().x = -PI / 2.0;
        felt.position().y = 0.041;
        jewel.add(&felt);
        // 3-5 tiny gems scattered on felt.
        let n_gems = 3 + (rnd() * 3.0).floor() as i32;
        for _ in 0..n_gems {
            let gem_mat = basic_mat!({ color: pick(&gem_colors), transparent: true, opacity: 0.9 });
            let gem = mesh(sphere_geo!(0.006, 6, 5), &gem_mat);
            gem.position().set((rnd() - 0.5) * 0.07, 0.045, (rnd() - 0.5) * 0.05);
            jewel.add(&gem);
        }
        // Lid open behind (leaning back).
        let lid = mesh(pg("SmallJewelryBoxes__lidOpen", box_geo!(0.10, 0.03, 0.07)), &box_mat);
        lid.position().set(0, 0.078, -0.0305); // hinge edge on the box's back top edge
        lid.rotation().x = -PI / 3.0;
        jewel.add(&lid);
    } else {
        // Closed lid on top.
        let lid = mesh(pg("SmallJewelryBoxes__lid", box_geo!(0.10, 0.014, 0.07)), &box_mat);
        lid.position().y = 0.047;
        jewel.add(&lid);
        // Small brass hinge on back.
        let hinge = mesh(pg("SmallJewelryBoxes__hinge", cyl_geo!(0.003, 0.003, 0.02, 4)), lambert!({ color: 0xd8b040 }));
        hinge.rotation().z = PI / 2.0;
        hinge.position().set(0, 0.043, -0.033);
        jewel.add(&hinge);
    }
    jewel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    jewel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&jewel);
}

/// R443 small perfume bottle: batch `t03`'s (`t03.rs`)
#[path = "t03.rs"]
mod t03;
pub use t03::place_small_perfumes;

/// R443 small nail polish bottle: 10% chance per level. Small
/// square glass bottle with brush cap. Dressing-table family.
pub fn place_small_nail_polishes(g: &mut Game, _dist: &[i32]) {
    // A small bottle of nail polish (batch `r18`): heavy-bottomed square glass,
    // a black screw cap with its brush in the lacquer, the lacquer dried down
    // in it (pooled along its side when it fell over wet), a drip run down
    // the glass and dried, a crust round the neck; no maker's name
    // (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: red, plum, blue, gold, green, burgundy, near black
    let polish_colors = [0xd82040, 0x8a3060, 0x2050d8, 0xd8b040, 0x30a030, 0xa02040, 0x101820];
    let polish_color = pick(&polish_colors);
    let kind = polish_colors.iter().position(|&k| k == polish_color).unwrap_or(0);
    let lacquer = phong!({ color: r18_polish::LACQUER[kind], specular: 0x4a4a4a, shininess: 90 });
    let bottle = group();
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            bottle.add(&mesh(geo, mat));
        }
    };
    put("Real__nailPolishCap", &phong!({ color: 0x18171a, specular: 0x3a3a3a, shininess: 70 }));
    put("Real__nailPolishBrush", &lambert!({ color: 0xcfcab9 }));
    put("Real__nailPolishDrip", &lacquer);
    // 20% chance tipped on its side (it fell over while the lacquer was wet)
    let tipped = rnd() < 0.20;
    put(if tipped { "Real__nailPolishLacquerLying" } else { "Real__nailPolishLacquer" }, &lacquer);
    // the glass over what is in it; its thick bottom less clear (and solid
    // enough to stand on the floor: the lacquer is not what touches it)
    put("Real__nailPolishGlassBase", &phong!({ color: 0x7f8a82, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.62 }));
    put("Real__nailPolishGlass", &phong!({ color: 0xd2d6d0, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.42 }));
    if tipped {
        bottle.rotation().z = PI / 2.0;
        bottle.position().y = 0.0125;
    }
    bottle.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bottle.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
}

/// The nail polish of `place_small_nail_polishes` (batch `r18`)
mod r18_polish {
    /// The lacquers, dried dark in the glass (sRGB hex): red, plum, blue,
    /// gold, green, burgundy, near black
    pub(super) const LACQUER: [u32; 7] = [0x8a1c28, 0x5a2642, 0x23386a, 0x8a6a30, 0x2a5034, 0x561a26, 0x18191c];
}

/// R442 small lipstick tube: 10% chance per level. Two-tone tube
/// with colored bullet tip. Dressing-table family.
pub fn place_small_lipsticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tube_colors = [0x201a10, 0xd8b040, 0xa02040, 0x606870, 0xd050a0];
    let bullet_colors = [0xa02040, 0xd85090, 0x8a2030, 0xc02060, 0xd0405a];
    let tube_mat = lambert!({ color: pick(&tube_colors) });
    let bullet_mat = lambert!({ color: pick(&bullet_colors) });
    let stick = group();
    let is_open = rnd() < 0.50;
    // Body: main tube cylinder.
    let body = mesh(pg("SmallLipsticks__case", cyl_geo!(0.010, 0.010, 0.050, 8)), &tube_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.025, 0.010, 0);
    stick.add(&body);
    if is_open {
        // Cap laid separately, bullet extended.
        let cap = mesh(pg("SmallLipsticks__cap", cyl_geo!(0.012, 0.012, 0.035, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(-0.020, 0.010, 0.025);
        stick.add(&cap);
        // Bullet: sloped-tip cone.
        let bullet = mesh(pg("SmallLipsticks__bullet", cyl_geo!(0.008, 0.008, 0.020, 8)), &bullet_mat);
        bullet.rotation().z = PI / 2.0;
        bullet.position().set(0.060, 0.010, 0);
        stick.add(&bullet);
        let tip = mesh(pg("SmallLipsticks__tip", cone_geo!(0.008, 0.010, 8)), &bullet_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.075, 0.010, 0);
        stick.add(&tip);
    } else {
        // Closed: cap on top of body.
        let cap = mesh(pg("SmallLipsticks__cap", cyl_geo!(0.012, 0.012, 0.035, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.068, 0.010, 0);
        stick.add(&cap);
    }
    stick.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    stick.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stick);
}

/// R442 small makeup compact: 8% chance per level. Round clamshell
/// compact with mirror lid + powder pan. Dressing-table family.
pub fn place_small_compacts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd8b040, 0xf0d0d0, 0x201a10, 0xa02040, 0x606870];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let mirror_mat = basic_mat!({ color: 0xc0d0e0 });
    let powder_mat = lambert!({ color: 0xe8c0a0 });
    let compact = group();
    let is_open = rnd() < 0.60;
    let r = 0.038;
    // Hinge (two knuckles): on the back rim when closed, joining the two halves when open.
    let hinge = mesh(pg("SmallCompacts__hinge", sphere_geo!(0.003, 6, 4)), &body_mat);
    if is_open {
        // Bottom half (powder pan).
        let bot = mesh(pg("SmallCompacts__base", cyl_geo!(r, r, 0.010, 14)), &body_mat);
        bot.position().y = 0.005;
        compact.add(&bot);
        let powder = mesh(circle_geo!(r * 0.85, 14), &powder_mat);
        powder.rotation().x = -PI / 2.0;
        powder.position().y = 0.0105;
        compact.add(&powder);
        // Top half laid open to the side.
        let top = mesh(pg("SmallCompacts__lid", cyl_geo!(r, r, 0.008, 14)), &body_mat);
        top.position().set(r * 2.05, 0.004, 0);
        compact.add(&top);
        let mir = mesh(circle_geo!(r * 0.85, 14), &mirror_mat);
        mir.rotation().x = -PI / 2.0;
        mir.position().set(r * 2.05, 0.0085, 0);
        compact.add(&mir);
        // Hinge across the gap between the two rims, pin along Z.
        hinge.rotation().y = PI / 2.0;
        hinge.position().set(r * 1.025, 0.006, 0);
        compact.add(&hinge);
    } else {
        // Closed clamshell.
        let body = mesh(pg("SmallCompacts__body", cyl_geo!(r, r, 0.018, 14)), &body_mat);
        body.position().y = 0.009;
        compact.add(&body);
        // Small hinge dot on rim.
        hinge.position().set(0, 0.018, r);
        compact.add(&hinge);
    }
    compact.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    compact.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compact);
}

/// R441 small hand mirror: 8% chance per level. Round mirror on
/// handle. Dressing-table family.
pub fn place_small_hand_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_colors = [0xd8b040, 0xf0d0d0, 0x8a3820, 0x201a10];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let mirror_mat = basic_mat!({ color: 0xc0d0e0 });
    let mirror = group();
    // Frame back (thin disc).
    let back = mesh(pg("SmallHandMirrors__frame", cyl_geo!(0.055, 0.055, 0.008, 14)), &frame_mat);
    back.position().y = 0.004;
    mirror.add(&back);
    // Mirror face (slightly smaller lighter disc on top).
    let face = mesh(circle_geo!(0.048, 14), &mirror_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0085;
    mirror.add(&face);
    // Handle: cylinder extending outward.
    let handle_len = 0.12;
    let handle = mesh(pg("SmallHandMirrors__handle", cyl_geo!(0.008, 0.010, handle_len, 8)), &frame_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.055 + handle_len / 2.0, 0.010, 0); // thick end (r 0.010) resting on the floor
    mirror.add(&handle);
    // Handle end knob.
    let knob = mesh(pg("SmallHandMirrors__knob", sphere_geo!(0.012, 8, 6)), &frame_mat);
    knob.position().set(0.055 + handle_len + 0.008, 0.012, 0); // resting on the floor
    mirror.add(&knob);
    mirror.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mirror.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mirror);
}

/// R441 small folding fan: batch `t11`'s (`t11.rs`)
#[path = "t11.rs"]
mod t11;
pub use t11::place_small_folding_fans;

/// R440 small paper crown: 8% chance per level. Jagged strip of
/// colored paper on floor. Party-costume family.
pub fn place_small_paper_crowns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crown_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let crown_mat = lambert!({ color: pick(&crown_colors), side: DoubleSide });
    let crown = group();
    // Build zig-zag strip with alternating base + spike vertices.
    let n_points = 6;
    let arc_len = 0.42;
    let width = 0.035;
    let spike_h = 0.030;
    // 8-cylinder arc for ring band + jagged spike triangles above.
    let radius = arc_len / (2.0 * PI) * 1.4;
    for i in 0..n_points {
        let a = -PI / 2.0 + (i as f64 / (n_points - 1) as f64) * (PI * 1.25);
        let bx = a.cos() * radius;
        let bz = a.sin() * radius;
        // Base ring segment (as small box).
        if i < n_points - 1 {
            let a1 = -PI / 2.0 + ((i + 1) as f64 / (n_points - 1) as f64) * (PI * 1.25);
            let bx1 = a1.cos() * radius;
            let bz1 = a1.sin() * radius;
            let seg_len = hypot2(bx1 - bx, bz1 - bz);
            let seg = mesh(box_geo!(seg_len, 0.008, width), &crown_mat);
            seg.position().set((bx + bx1) / 2.0, 0.004, (bz + bz1) / 2.0);
            seg.rotation().y = -(bz1 - bz).atan2(bx1 - bx);
            crown.add(&seg);
        }
        // Spike triangle rising from top.
        let spike_geo = buf_geo(&[-width / 2.0 * 0.8, 0.008, 0.0, width / 2.0 * 0.8, 0.008, 0.0, 0.0, 0.008 + spike_h, 0.0], &[0, 1, 2]);
        let spike = mesh(spike_geo, &crown_mat);
        // Orient spike outward from ring center.
        spike.position().set(bx, 0, bz);
        spike.rotation().y = -bz.atan2(bx) + PI / 2.0;
        crown.add(&spike);
    }
    crown.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    crown.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crown);
}

/// R440 small opera glasses: 6% chance per level. Twin cylinders
/// on handle. Party-costume family (fancy accessory).
pub fn place_small_opera_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a3820, 0x201a10, 0xd8b040, 0x606870];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let lens_mat = basic_mat!({ color: 0x304040, transparent: true, opacity: 0.75 });
    let opera = group();
    // 2 lens cylinders side by side.
    for i in 0..2 {
        let s = if i == 0 { -1.0 } else { 1.0 };
        let barrel = mesh(pg("SmallOperaGlasses__barrel", cyl_geo!(0.018, 0.018, 0.045, 10)), &body_mat);
        barrel.rotation().z = PI / 2.0;
        // Side by side (parallel, either side of the bridge).
        barrel.position().set(0.030, 0.020, s * 0.023);
        opera.add(&barrel);
        // Lens disc on far end.
        let lens = mesh(circle_geo!(0.016, 8), &lens_mat);
        lens.rotation().y = PI / 2.0;
        lens.position().set(0.030 + 0.023, 0.020, s * 0.023);
        opera.add(&lens);
    }
    // Bridge connecting them.
    let bridge = mesh(pg("SmallOperaGlasses__bridge", box_geo!(0.030, 0.008, 0.010)), &body_mat);
    bridge.position().set(0.030, 0.020, 0);
    opera.add(&bridge);
    // Handle extending sideways, out from the side of the first barrel,
    // sloping down so the finial rests on the floor.
    let handle = mesh(pg("SmallOperaGlasses__handle", cyl_geo!(0.004, 0.004, 0.10, 6)), &body_mat);
    handle.rotation().x = -PI / 2.0 - 0.16;
    handle.position().set(0.030, 0.012, -0.039 - 0.05 * 0.16f64.cos());
    opera.add(&handle);
    opera.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    opera.rotation().y = rnd() * PI * 2.0;
    g.world.add(&opera);
}

/// R439 small tiara: 5% chance per level. Thin gold ring with 5
/// diamond spike accents. Party-costume family.
pub fn place_small_tiaras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let gold_mat = lambert!({ color: 0xd8b040 });
    let gem_colors = [0xd82060, 0x2080d8, 0x30c060, 0xd050a0];
    let gem_mat = basic_mat!({ color: pick(&gem_colors), transparent: true, opacity: 0.9 });
    let tiara = group();
    // Base: half-ring (torus arc approximated by 8 short cylinders on semicircle).
    let radius = 0.055;
    let n_arcs = 8;
    for i in 0..n_arcs {
        let a0 = -PI / 2.0 + (i as f64 / n_arcs as f64) * PI;
        let a1 = -PI / 2.0 + ((i + 1) as f64 / n_arcs as f64) * PI;
        let (x0, z0) = (a0.cos() * radius, a0.sin() * radius);
        let (x1, z1) = (a1.cos() * radius, a1.sin() * radius);
        let seg_len = hypot2(x1 - x0, z1 - z0);
        let seg = mesh(pg("SmallTiaras__band", cyl_geo!(0.004, 0.004, seg_len, 5)), &gold_mat);
        seg.position().set((x0 + x1) / 2.0, 0.006, (z0 + z1) / 2.0);
        seg.rotation().x = PI / 2.0;
        // With Euler XYZ this puts the rod axis along the arc tangent.
        seg.rotation().z = -(x1 - x0).atan2(z1 - z0);
        tiara.add(&seg);
    }
    // 5 spike points along the crown top.
    for i in 0..5 {
        let a = -PI / 4.0 + (i as f64 / 4.0) * (PI / 2.0);
        let ax = a.cos() * radius;
        let az = a.sin() * radius;
        let spike = mesh(pg("SmallTiaras__spike", cone_geo!(0.008, 0.030, 4)), &gold_mat);
        // Standing up on the band.
        spike.position().set(ax, 0.022, az);
        tiara.add(&spike);
        // Small gem at spike base.
        if i == 2 {
            let gem = mesh(sphere_geo!(0.006, 6, 5), &gem_mat);
            gem.position().set(ax, 0.010, az);
            tiara.add(&gem);
        }
    }
    tiara.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tiara.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tiara);
}

/// R439 small castanets: 8% chance per level. Pair of clamshell
/// castanets. Music family (percussion).
pub fn place_small_castanets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shell_colors = [0x8a3820, 0x4a2818, 0x201a10, 0xd8b820];
    let shell_mat = lambert!({ color: pick(&shell_colors) });
    let cord_mat = lambert!({ color: 0xf0e0c8 });
    let pair = group();
    for i in 0..2 {
        let cast = group();
        // 2 half-spheres (top + bottom shell) facing each other, hollows together.
        let top = mesh(pg("SmallCastanets__shell", sphere_geo!(0.030, 10, 4, 0.0, PI * 2.0, 0.0, PI / 2.0)), &shell_mat);
        top.scale().y = 0.6;
        top.position().y = 0.019; // dome up, hollow face down onto the bottom shell
        cast.add(&top);
        let bot = mesh(pg("SmallCastanets__shell", sphere_geo!(0.030, 10, 4, 0.0, PI * 2.0, 0.0, PI / 2.0)), &shell_mat);
        bot.rotation().x = PI; // dome down on the floor, hollow face up
        bot.scale().y = 0.6;
        bot.position().y = 0.018;
        cast.add(&bot);
        // Cord loop, wrapped round the two rims at the ear (+x).
        let lp = mesh(torus_geo!(0.006, 0.001, 4, 8), &cord_mat);
        lp.position().set(0.030, 0.0185, 0);
        cast.add(&lp);
        // Position side by side.
        cast.position().x = (if i == 0 { -0.040 } else { 0.040 }) + (rnd() - 0.5) * 0.02;
        cast.position().z = (rnd() - 0.5) * 0.02;
        cast.rotation().y = rnd() * PI * 2.0;
        pair.add(&cast);
    }
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

/// R438 small metronome: 6% chance per level. Pyramid body with
/// swinging pendulum arm animator. Music family.
pub fn place_small_metronomes(g: &mut Game, _dist: &[i32]) {
    g.pp.metronome_positions.clear(); // R479 reset per level
    g.pp.metronome_periods.clear();
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a5030, 0x6a3820, 0x502818, 0x4a2818];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let arm_mat = lambert!({ color: 0x181008 });
    let metronome = group();
    // Pyramid body: cone with 4 sides.
    let body = mesh(pg("SmallMetronomes__body", cone_geo!(0.055, 0.15, 4, 1, true)), &body_mat);
    body.position().y = 0.075;
    metronome.add(&body);
    // Base disc.
    let base = mesh(pg("SmallMetronomes__base", box_geo!(0.10, 0.008, 0.10)), &body_mat);
    base.position().y = 0.004;
    metronome.add(&base);
    // Front face slot: dark rectangle. The front face leans back 0.214 rad.
    let face_tilt = (0.028f64 / 0.129).atan();
    let slot = mesh(plane_geo!(0.020, 0.10), &arm_mat);
    slot.position().set(0, 0.075, 0.0287);
    slot.rotation().x = -face_tilt;
    metronome.add(&slot);
    // Swinging arm: pivots at its round foot and swings in the face's plane.
    let arm_pivot = group();
    arm_pivot.position().set(0, 0.018, 0.0405);
    arm_pivot.rotation().x = -face_tilt;
    metronome.add(&arm_pivot);
    let arm = mesh(pg("SmallMetronomes__arm", cyl_geo!(0.002, 0.002, 0.10, 4)), &arm_mat);
    arm.position().set(0, 0.048, 0.0045);
    // Small brass weight on arm.
    let weight = mesh(pg("SmallMetronomes__weight", box_geo!(0.010, 0.006, 0.008)), &brass_mat);
    weight.position().set(0, 0.030, 0);
    arm.add(&weight);
    arm_pivot.add(&arm);
    // Animate arm swing (period 1.2-1.8 s).
    let swing_phase = rnd() * PI * 2.0;
    let swing_period = 1.2 + rnd() * 0.6;
    let ap = arm_pivot.clone();
    g.animate(move |_g, _dt, t| {
        ap.rotation().z = (t * (PI * 2.0 / swing_period) + swing_phase).sin() * 0.35;
    });
    metronome.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    metronome.rotation().y = rnd() * PI * 2.0;
    g.world.add(&metronome);
    // R479 track metronome for tick audio (with its swing period, so ticks
    // sync with the pendulum: kept in the parallel `metronome_periods`).
    let p = metronome.pos();
    g.pp.metronome_positions.push(v3(p.x, 0.06, p.z));
    g.pp.metronome_periods.push(swing_period);
}

/// R438 small feather boa: 6% chance per level. Curved fluffy
/// tube on floor. Party-costume family.
pub fn place_small_feather_boas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let boa_colors = [0xd85090, 0xa020a0, 0xd8b820, 0x2050d8, 0xf0f0f0, 0x181008];
    let boa_color = pick(&boa_colors);
    let feath_mat = lambert!({ color: boa_color });
    let boa = group();
    // 12-segment curved rope of fluffy spheres.
    let n_segments = 12usize;
    let total_len = 0.7 + rnd() * 0.4;
    let curve_amp = 0.20;
    let curve_phase = rnd() * PI * 2.0;
    // Puffs go 4.5 cm apart by arc length along the curve, which is shrunk to
    // fit, so the clumps overlap into one rope.
    let curve = |t: f64| -> (f64, f64) { ((t - 0.5) * total_len, (t * PI + curve_phase).sin() * curve_amp) };
    let mut arc: Vec<f64> = vec![0.0];
    for k in 1..=64usize {
        let (ax, az) = curve((k - 1) as f64 / 64.0);
        let (bx, bz) = curve(k as f64 / 64.0);
        let v = arc[k - 1] + hypot2(bx - ax, bz - az);
        arc.push(v);
    }
    let shrink = 0.045 * (n_segments - 1) as f64 / arc[64];
    for i in 0..n_segments {
        let t = i as f64 / (n_segments - 1) as f64;
        let mut k = 1usize;
        while k < 64 && arc[k] < t * arc[64] {
            k += 1;
        }
        let (ca, cp) = curve(((k - 1) as f64 + (t * arc[64] - arc[k - 1]) / (arc[k] - arc[k - 1])) / 64.0);
        let along = ca * shrink;
        let perp = cp * shrink;
        let seg = mesh(pg("SmallFeatherBoas__puff", sphere_geo!(0.028, 6, 5)), &feath_mat);
        seg.scale().set(1.0 + rnd() * 0.3, 0.9 + rnd() * 0.4, 1.0 + rnd() * 0.3);
        seg.position().set(along, 0.025, perp);
        seg.rotation().y = rnd() * PI;
        boa.add(&seg);
    }
    boa.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    boa.rotation().y = rnd() * PI * 2.0;
    g.world.add(&boa);
}
