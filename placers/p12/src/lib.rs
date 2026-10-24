//! Prop placers ported from backrooms.js (chunk p12; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeTallFloorLamps", place_tall_floor_lamps));
    r.push(("placeGentleRockers", place_gentle_rockers));
    r.push(("placeWideBookshelves", place_wide_bookshelves));
    r.push(("placeHallSideTables", place_hall_side_tables));
    r.push(("placeAbstractPaintings", place_abstract_paintings));
    r.push(("placeKitchenCalendars", place_kitchen_calendars));
    r.push(("placePetBowls", place_pet_bowls));
    r.push(("placePetToys", place_pet_toys));
    r.push(("placePetPortraits", place_pet_portraits));
    r.push(("placeDogLeashes", place_dog_leashes));
    r.push(("placeOrnateMusicBoxes", place_ornate_music_boxes));
    r.push(("placeTravelPosters", place_travel_posters));
}

// ------------------------------------------------------------------ helpers
/// `for z, x: if (tiles[tIdx(x, z)] === 0) openTiles.push({ x, z })`
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// Open tiles with their first wall neighbour (`[[1,0],[-1,0],[0,1],[0,-1]]` order).
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

/// `obj.rotation.y = Math.atan2(-dx, -dz); obj.position.set(wx(x) + dx * (CELL * 0.5 - inset), 0, wz(z) + dz * ...)`
fn mount_on_wall(g: &Game, o: &Obj, x: i32, z: i32, dx: i32, dz: i32, inset: f64) {
    o.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - inset);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - inset);
    o.position().set(px, 0, pz);
}

// ------------------------------------------------------------------ line 24059
// R716 tall floor lamp w/ shade + optional PointLight (floor): 3% chance per level.
fn place_tall_floor_lamps(g: &mut Game, _dist: &[i32]) {
    g.pp.tall_lamp_positions = vec![]; // R718 reset per level
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let stand_colors = [0x504030, 0x202020, 0xc0a060, 0x8b5a2b, 0x606060];
    let stand_mat = lambert!({ color: choice(&stand_colors) });
    let shade_colors = [0xf0e0c0, 0xd0a080, 0xf0e0a0, 0xc0d0e0, 0xe0c0e0, 0xf0f0e0];
    let shade_mat = lambert!({ color: choice(&shade_colors), transparent: true, opacity: 0.85 });
    let bulb_mat = basic_mat!({ color: 0xf0e0a0 });
    let fl = group();
    // Base — wide flat disc.
    let base = mesh(pg("TallFloorLamps__base", cyl_geo!(0.10, 0.11, 0.014, 12)), &stand_mat);
    base.position().y = 0.007;
    fl.add(&base);
    // Stand column — tall thin cylinder.
    let stand_h = 1.30;
    let stand = mesh(pg("TallFloorLamps__stand", cyl_geo!(0.010, 0.012, stand_h, 6)), &stand_mat);
    stand.position().y = 0.014 + stand_h / 2.0;
    fl.add(&stand);
    // Optional decorative middle band (torus on stand).
    let band = mesh(pg("TallFloorLamps__band", torus_geo!(0.014, 0.003, 3, 8)), &stand_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.014 + stand_h * 0.4;
    fl.add(&band);
    // Top harp — U-shape supporting shade (small ring).
    let harp_mat = lambert!({ color: 0x505050 });
    let harp_ring = mesh(pg("TallFloorLamps__harp", torus_geo!(0.018, 0.002, 3, 8)), &harp_mat);
    harp_ring.rotation().x = PI / 2.0;
    harp_ring.position().y = 0.014 + stand_h + 0.010;
    fl.add(&harp_ring);
    // Shade — wide tapered cylinder (top narrower).
    let (shade_top, shade_bottom, shade_h) = (0.14, 0.20, 0.24);
    let shade = mesh(cyl_geo!(shade_top, shade_bottom, shade_h, 12, 1, true), &shade_mat);
    shade.position().y = 0.014 + stand_h + shade_h / 2.0 + 0.030;
    fl.add(&shade);
    // Small bulb hint (self-lit sphere inside shade).
    let bulb = mesh(sphere_geo!(0.030, 8, 6), &bulb_mat);
    bulb.position().y = 0.014 + stand_h + shade_h / 2.0 + 0.020;
    fl.add(&bulb);
    // 55% chance: lamp is on — PointLight illumination + optional flicker.
    let lamp_is_on = rnd() < 0.55;
    if lamp_is_on {
        let light = point_light!(0xf0d090, 0.60, 3.5, 2);
        light.position().copy(&bulb.pos());
        fl.add(&light);
        // 40% chance: subtle flicker (dying bulb).
        if rnd() < 0.4 {
            let flicker_phase = rnd() * PI * 2.0;
            let light2 = light.clone();
            let bulb2 = bulb.clone();
            g.animate(move |_g, _dt, t| {
                let f = 0.85 + (t * 6.0 + flicker_phase).sin() * 0.10 + (t * 11.0 + flicker_phase * 1.4).sin() * 0.05;
                light2.set_intensity(0.60 * f);
                bulb2.scale().set(f, f, f);
            });
        }
    } else {
        // Bulb un-lit — swap to dim material.
        bulb.set_mat(lambert!({ color: 0x606050 }));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    fl.position().set(px, 0, pz);
    fl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fl);
    // R718 register on-lamp positions for filament buzz audio.
    if lamp_is_on {
        let p = fl.pos();
        g.pp.tall_lamp_positions.push(v3(p.x, 0, p.z));
    }
}

// ------------------------------------------------------------------ line 24145
// R716 rocking chair w/ optional gentle rocking animator (floor): 3% chance per level.
fn place_gentle_rockers(g: &mut Game, _dist: &[i32]) {
    g.pp.rocking_chair_positions = vec![]; // R718 reset per level
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    // Optional cushion color.
    let cushion_colors = [0xa02040, 0x408060, 0xd0a040, 0xf0e0c0, 0x2060a0, 0x804060];
    let cushion_mat = lambert!({ color: choice(&cushion_colors) });
    // Whole chair goes into a group that we rotate for the rocking animation.
    let rock_group = group();
    let chair = group();
    // Seat — flat rectangular box.
    let (seat_w, seat_d, seat_h) = (0.36, 0.32, 0.036);
    let seat = mesh(pg("GentleRockers__seat", box_geo!(seat_w, seat_h, seat_d)), &wood_mat);
    seat.position().y = 0.30;
    chair.add(&seat);
    // 4 legs — thin cylinders at corners.
    for cx in [-seat_w * 0.42, seat_w * 0.42] {
        for cz in [-seat_d * 0.42, seat_d * 0.42] {
            let leg = mesh(pg("GentleRockers__leg", cyl_geo!(0.010, 0.010, 0.28, 5)), &wood_mat);
            leg.scale().y = 0.94; // seat underside down to the runner's top (y ~0.02)
            leg.position().set(cx, 0.1525, cz);
            chair.add(&leg);
        }
    }
    // Back — tall rectangular panel behind seat.
    let (back_w, back_h) = (seat_w, 0.42);
    let back = mesh(pg("GentleRockers__back", box_geo!(back_w, back_h, 0.020)), &wood_mat);
    back.position().set(0, 0.30 + back_h / 2.0, -seat_d * 0.45);
    chair.add(&back);
    // 5 vertical back slats for classic rocker look.
    let slat_w = 0.010;
    for i in 0..5 {
        let sx = -seat_w * 0.35 + i as f64 * (seat_w * 0.7 / 4.0);
        let slat = mesh(pg("GentleRockers__slat", cyl_geo!(0.006, 0.006, back_h - 0.020, 5)), &wood_mat);
        slat.position().set(sx, 0.30 + back_h / 2.0, -seat_d * 0.45 + 0.005);
        chair.add(&slat);
    }
    // Top rail on back.
    let top_rail = mesh(pg("GentleRockers__topRail", box_geo!(seat_w + 0.020, 0.024, 0.020)), &wood_mat);
    top_rail.position().set(0, 0.30 + back_h + 0.010, -seat_d * 0.45);
    chair.add(&top_rail);
    // 2 armrests — horizontal boxes at seat height.
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("GentleRockers__arm", box_geo!(0.024, 0.020, seat_d * 0.7)), &wood_mat);
        arm.position().set(side * seat_w * 0.48, 0.30 + 0.10, -seat_d * 0.05);
        chair.add(&arm);
        // Vertical support for armrest (short cylinder).
        let arm_support = mesh(pg("GentleRockers__armSupport", cyl_geo!(0.008, 0.008, 0.11, 5)), &wood_mat);
        arm_support.position().set(side * seat_w * 0.48, 0.30 + 0.056, seat_d * 0.28);
        chair.add(&arm_support);
    }
    // 2 curved rocker runners — partial torus segments.
    for side in [-1.0f64, 1.0] {
        let runner = mesh(pg("GentleRockers__runner", torus_geo!(0.30, 0.008, 4, 12, PI * 0.75)), &wood_mat);
        // Arc centred on its lowest point, so it runs under the chair from
        // front to back; the rig flattens it into a long low rocker.
        runner.rotation().z = -PI * 0.875;
        runner.rotation().y = PI / 2.0;
        // Position runner so bottom of arc rests on floor plane.
        runner.position().set(0, 0.308, 0);
        let runner_rig = group();
        runner_rig.scale().set(1, 0.6, 1.15);
        runner_rig.position().set(side * seat_w * 0.42, 0, 0);
        runner_rig.add(&runner);
        chair.add(&runner_rig);
    }
    // 55% chance: cushion on seat.
    if rnd() < 0.55 {
        let cushion = mesh(pg("GentleRockers__cushion", box_geo!(seat_w - 0.020, 0.032, seat_d - 0.020)), &cushion_mat);
        cushion.position().set(0, 0.30 + seat_h / 2.0 + 0.016, 0);
        chair.add(&cushion);
        // 30% chance: back cushion too.
        if rnd() < 0.3 {
            let back_cushion = mesh(pg("GentleRockers__backCushion", box_geo!(seat_w - 0.030, back_h * 0.4, 0.024)), &cushion_mat);
            back_cushion.position().set(0, 0.30 + back_h * 0.30, -seat_d * 0.45 + 0.023); // in front of the spindles
            chair.add(&back_cushion);
        }
    }
    rock_group.add(&chair);
    // 50% chance: gentle rocking animator — very slow, subtle.
    let is_rocking = rnd() < 0.5;
    if is_rocking {
        let rock_phase = rnd() * PI * 2.0;
        let rg2 = rock_group.clone();
        g.animate(move |_g, _dt, t| {
            // Very slow gentle rocking — 0.7 Hz, ±3° amplitude.
            rg2.rotation().x = (t * 0.7 + rock_phase).sin() * 0.05;
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.2;
    rock_group.position().set(px, 0, pz);
    rock_group.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rock_group);
    // R718 register rocking chair positions for wood-creak audio.
    if is_rocking {
        let p = rock_group.pos();
        g.pp.rocking_chair_positions.push(v3(p.x, 0, p.z));
    }
}

// ------------------------------------------------------------------ line 24264
// R717 wall-mounted bookshelf w/ books (wall): 2.5% chance per level.
fn place_wide_bookshelves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030, 0x2a1a10];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let bs = group();
    let mount_y = 1.20; // Center at chest-height.
    // Shelf dimensions.
    let (shelf_w, shelf_d, shelf_t) = (0.60, 0.16, 0.014);
    // 3 shelves stacked vertically.
    let shelf_spacing = 0.32;
    for i in 0..3 {
        let shelf = mesh(pg("WideBookshelves__shelf", box_geo!(shelf_w, shelf_t, shelf_d)), &wood_mat);
        shelf.position().set(0, mount_y - shelf_spacing + i as f64 * shelf_spacing, 0.08);
        bs.add(&shelf);
    }
    // Back panel (thin) mounted against wall.
    let back = mesh(pg("WideBookshelves__back", box_geo!(shelf_w, shelf_spacing * 2.0 + shelf_t, 0.008)), &wood_mat);
    back.position().set(0, mount_y, 0.008);
    bs.add(&back);
    // 2 side panels.
    for side in [-1.0f64, 1.0] {
        let side_panel = mesh(pg("WideBookshelves__side", box_geo!(0.010, shelf_spacing * 2.0 + shelf_t, shelf_d)), &wood_mat);
        side_panel.position().set(side * shelf_w * 0.5, mount_y, 0.08);
        bs.add(&side_panel);
    }
    // Books on 2 shelves (top + middle only).
    let book_colors = [
        0x603030, 0x2a4a60, 0x408030, 0x604040, 0x201050, 0x8b5a2b,
        0x101010, 0xa02040, 0xd0a040, 0x606060, 0x804a20, 0x203060,
    ];
    for shelf_idx in 1..3 {
        let shelf_y = mount_y - shelf_spacing + shelf_idx as f64 * shelf_spacing;
        let mut book_x = -shelf_w * 0.42;
        let mut attempts = 0;
        while book_x < shelf_w * 0.42 && attempts < 20 {
            attempts += 1;
            let book_h = 0.14 + rnd() * 0.10;
            let book_w = 0.020 + rnd() * 0.014;
            let book_d = 0.10 + rnd() * 0.04;
            let is_leaning = rnd() < 0.20; // 20% books lean over.
            let book_mat = lambert!({ color: choice(&book_colors) });
            let book = mesh(pg("WideBookshelves__book", box_geo!(book_w, book_h, book_d)), &book_mat);
            book.position().set(book_x + book_w / 2.0, shelf_y + shelf_t / 2.0 + book_h / 2.0, 0.08);
            let mut base_z_rot = 0.0;
            if is_leaning {
                base_z_rot = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.15 + rnd() * 0.15);
                book.rotation().z = base_z_rot;
                // Leaning book needs slight height reduction.
                book.position().y -= 0.008;
            }
            bs.add(&book);
            // Optional dark title-line spine detail.
            let mut spine_line_ref: Option<Obj> = None;
            if rnd() < 0.5 {
                let spine_line_mat = lambert!({ color: 0x1a1a1a });
                let spine_line = mesh(box_geo!(book_w * 0.7, 0.002, 0.001), &spine_line_mat);
                let bx = book.pos().x;
                spine_line.position().set(bx, shelf_y + shelf_t / 2.0 + book_h * 0.7, 0.08 + book_d / 2.0 + 0.001);
                if is_leaning {
                    spine_line.rotation().z = base_z_rot;
                }
                bs.add(&spine_line);
                spine_line_ref = Some(spine_line);
            }
            // R726 subtle tilt animator on leaning books — 40% chance per leaning book.
            if is_leaning && rnd() < 0.4 {
                let tilt_phase = rnd() * PI * 2.0;
                let book2 = book.clone();
                let spine2 = spine_line_ref.clone();
                g.animate(move |_g, _dt, t| {
                    // Sinusoidal drift ±0.015 rad, 0.4 Hz — very slow settling.
                    let off = (t * 0.8 + tilt_phase).sin() * 0.015;
                    book2.rotation().z = base_z_rot + off;
                    if let Some(s) = &spine2 { s.rotation().z = base_z_rot + off; }
                });
            }
            book_x += book_w + 0.001;
        }
    }
    // Optional decorative object on top shelf — 60% chance.
    if rnd() < 0.6 {
        let top_shelf_y = mount_y + shelf_spacing;
        let obj_type = (rnd() * 3.0).floor() as i32;
        if obj_type == 0 {
            // Small vase.
            let vase_mat = lambert!({ color: [0xa02020, 0x2060a0, 0x408060, 0xd0a040][(rnd() * 4.0).floor() as usize] });
            let vase = mesh(pg("WideBookshelves__vase", cyl_geo!(0.030, 0.024, 0.070, 8)), &vase_mat);
            vase.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.035, 0.08);
            bs.add(&vase);
        } else if obj_type == 1 {
            // Framed photo (mini).
            let frame_mat = lambert!({ color: 0x604020 });
            let frame = mesh(pg("WideBookshelves__photoframe", box_geo!(0.05, 0.06, 0.004)), &frame_mat);
            frame.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.030, 0.08);
            bs.add(&frame);
            let photo = mesh(pg("WideBookshelves__photo", box_geo!(0.04, 0.05, 0.005)), lambert!({ color: 0xa08060 }));
            photo.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.030, 0.08 + 0.002);
            bs.add(&photo);
        } else {
            // Trophy (small cup on base).
            let trophy_mat = lambert!({ color: 0xc0a060 });
            let trophy_base = mesh(pg("WideBookshelves__trophybase", box_geo!(0.030, 0.010, 0.030)), &trophy_mat);
            trophy_base.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.005, 0.08);
            bs.add(&trophy_base);
            let trophy_stem = mesh(pg("WideBookshelves__trophystem", cyl_geo!(0.004, 0.004, 0.020, 4)), &trophy_mat);
            trophy_stem.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.020, 0.08);
            bs.add(&trophy_stem);
            let trophy_cup = mesh(pg("WideBookshelves__trophycup", cyl_geo!(0.014, 0.010, 0.020, 8)), &trophy_mat);
            trophy_cup.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.038, 0.08);
            bs.add(&trophy_cup);
        }
    }
    mount_on_wall(g, &bs, x, z, dx, dz, 0.015);
    g.world.add(&bs);
}

// ------------------------------------------------------------------ line 24399
// R719 hall side table w/ key tray + mail stack (floor near wall): 3% chance per level.
fn place_hall_side_tables(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let hst = group();
    // Tabletop.
    let (table_w, table_d, table_t) = (0.50, 0.24, 0.020);
    let table_y = 0.72;
    let top = mesh(pg("HallSideTables__top", box_geo!(table_w, table_t, table_d)), &wood_mat);
    top.position().set(0, table_y, 0);
    hst.add(&top);
    // Trim rim on top edges (decorative).
    let trim = mesh(pg("HallSideTables__trim", box_geo!(table_w + 0.008, 0.006, table_d + 0.008)), &wood_mat);
    trim.position().set(0, table_y + table_t / 2.0 + 0.003, 0);
    hst.add(&trim);
    // 4 tapered legs.
    for cx in [-table_w * 0.42, table_w * 0.42] {
        for cz in [-table_d * 0.35, table_d * 0.35] {
            let leg = mesh(pg("HallSideTables__leg", cyl_geo!(0.010, 0.014, table_y - table_t / 2.0, 6)), &wood_mat);
            leg.position().set(cx, (table_y - table_t / 2.0) / 2.0, cz);
            hst.add(&leg);
        }
    }
    // Small drawer front (thin panel + brass knob).
    let drawer = mesh(pg("HallSideTables__drawer", box_geo!(table_w - 0.020, 0.06, 0.003)), &wood_mat);
    // Apron-mounted: top edge under the tabletop, back on the front legs' faces.
    drawer.position().set(0, table_y - table_t / 2.0 - 0.030, table_d * 0.35 + 0.0115);
    drawer.scale().x = 0.44 / (table_w - 0.020);
    hst.add(&drawer);
    let knob_mat = lambert!({ color: 0xc0a060 });
    let knob = mesh(pg("HallSideTables__knob", sphere_geo!(0.008, 6, 5)), &knob_mat);
    knob.position().set(0, table_y - table_t / 2.0 - 0.030, table_d * 0.35 + 0.0145);
    hst.add(&knob);
    // Key tray on top (shallow dish).
    let tray_colors = [0xc0a060, 0x606060, 0x2060a0, 0xa02020];
    let tray_mat = lambert!({ color: choice(&tray_colors) });
    let tray = mesh(pg("HallSideTables__tray", cyl_geo!(0.05, 0.045, 0.008, 12)), &tray_mat);
    tray.position().set(-table_w * 0.25, table_y + table_t / 2.0 + 0.004, 0);
    hst.add(&tray);
    // 60% chance: key inside tray.
    if rnd() < 0.6 {
        let key_mat = lambert!({ color: 0xa0a0a8 });
        let key_body = mesh(pg("HallSideTables__keyBlade", box_geo!(0.020, 0.001, 0.003)), &key_mat);
        key_body.position().set(-table_w * 0.25 + 0.008, table_y + table_t / 2.0 + 0.0055, 0); // on the dish floor
        key_body.rotation().y = 0.3;
        hst.add(&key_body);
        let key_head = mesh(pg("HallSideTables__keyBow", cyl_geo!(0.006, 0.006, 0.001, 6)), &key_mat);
        // Bow lying flat on the blade's axis, just past its back end.
        key_head.position().set(-table_w * 0.25 + 0.008 - 0.015 * 0.3f64.cos(), table_y + table_t / 2.0 + 0.0055, 0.015 * 0.3f64.sin());
        key_head.rotation().y = 0.3;
        hst.add(&key_head);
    }
    // Mail stack on right side of table (2-4 stacked envelopes).
    let mail_count = 2 + (rnd() * 3.0).floor() as i32;
    let envelope_colors = [0xf0e8d0, 0xe8dcc8, 0xf0d8c8, 0xd8d0c0, 0xf0e0e0];
    for i in 0..mail_count {
        let env_mat = lambert!({ color: choice(&envelope_colors) });
        let env = mesh(pg("HallSideTables__envelope", box_geo!(0.10, 0.002, 0.06)), &env_mat);
        let ex = table_w * 0.25 + (rnd() - 0.5) * 0.010;
        let ey = table_y + table_t / 2.0 + 0.002 + i as f64 * 0.003;
        let ez = (rnd() - 0.5) * 0.020;
        env.position().set(ex, ey, ez);
        env.rotation().y = (rnd() - 0.5) * 0.20;
        hst.add(&env);
        // 40% envelope has address text lines.
        if rnd() < 0.4 {
            let text_mat = lambert!({ color: 0x202020 });
            let addr_line = mesh(box_geo!(0.05, 0.0015, 0.001), &text_mat);
            let ep = env.pos();
            let ery = env.rot().y;
            addr_line.position().set(ep.x - 0.01, ep.y + 0.001, ep.z - 0.008);
            addr_line.rotation().y = ery;
            hst.add(&addr_line);
            // 30% chance of stamp square top-right.
            if rnd() < 0.3 {
                let stamp_mat = lambert!({ color: 0xa02040 });
                let stamp = mesh(pg("HallSideTables__stamp", box_geo!(0.012, 0.002, 0.014)), &stamp_mat);
                stamp.position().set(ep.x + 0.035, ep.y + 0.001, ep.z - 0.018);
                stamp.rotation().y = ery;
                hst.add(&stamp);
            }
        }
    }
    // Optional small vase center-back (40%).
    if rnd() < 0.4 {
        let vase_colors = [0xa02020, 0x2060a0, 0x408060, 0xd0a040, 0xf0e8d0];
        let vase_mat = lambert!({ color: choice(&vase_colors) });
        let vase = mesh(pg("HallSideTables__vase", cyl_geo!(0.030, 0.024, 0.10, 8)), &vase_mat);
        vase.position().set(0, table_y + table_t / 2.0 + 0.050, -table_d * 0.3);
        hst.add(&vase);
    }
    // Position at wall — table backs against wall.
    mount_on_wall(g, &hst, x, z, dx, dz, 0.150);
    g.world.add(&hst);
}

// ------------------------------------------------------------------ line 24519
// R719 framed modernist abstract painting (wall).
game::lazy_static_tex!(abstract_modernist_texture, canvas_tex(220, 220, |ctx, w, h| {
    // Off-white canvas background.
    ctx.set_fill_style("#e8e0d0");
    ctx.fill_rect(0, 0, w, h);
    // Variant: 3 different modernist styles.
    let variant = (rnd() * 3.0).floor() as i32;
    if variant == 0 {
        // Kandinsky-inspired: circles + lines.
        ctx.set_fill_style("#f0c040");
        ctx.begin_path();
        ctx.arc(w * 0.35, h * 0.4, 45, 0, PI * 2.0);
        ctx.fill();
        // Red triangle.
        ctx.set_fill_style("#c03040");
        ctx.begin_path();
        ctx.move_to(w * 0.65, h * 0.3);
        ctx.line_to(w * 0.85, h * 0.7);
        ctx.line_to(w * 0.55, h * 0.7);
        ctx.close_path();
        ctx.fill();
        // Blue rectangle.
        ctx.set_fill_style("#3060a0");
        ctx.fill_rect(w * 0.2, h * 0.65, 100, 30);
        // Black line diagonals.
        ctx.set_stroke_style("#101010");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(0, h * 0.2);
        ctx.line_to(w, h * 0.8);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(w * 0.15, h);
        ctx.line_to(w * 0.6, 0);
        ctx.stroke();
        // Small green square.
        ctx.set_fill_style("#40a040");
        ctx.fill_rect(w * 0.05, h * 0.05, 30, 30);
    } else if variant == 1 {
        // Mondrian-inspired: black lines + primary color blocks.
        ctx.set_stroke_style("#101010");
        ctx.set_line_width(6);
        ctx.begin_path();
        ctx.move_to(w * 0.4, 0); ctx.line_to(w * 0.4, h);
        ctx.move_to(w * 0.75, 0); ctx.line_to(w * 0.75, h);
        ctx.move_to(0, h * 0.35); ctx.line_to(w, h * 0.35);
        ctx.move_to(0, h * 0.7); ctx.line_to(w, h * 0.7);
        ctx.stroke();
        // Color blocks.
        ctx.set_fill_style("#c02040");
        ctx.fill_rect(w * 0.4 + 3.0, 3, w * 0.35 - 6.0, h * 0.35 - 3.0);
        ctx.set_fill_style("#f0c040");
        ctx.fill_rect(3, h * 0.7 + 3.0, w * 0.4 - 3.0, h * 0.30 - 6.0);
        ctx.set_fill_style("#2060a0");
        ctx.fill_rect(w * 0.75 + 3.0, h * 0.35 + 3.0, w * 0.25 - 6.0, h * 0.35 - 6.0);
    } else {
        // Rothko-inspired: soft color fields.
        let top_grad = ctx.create_linear_gradient(0, 0, 0, h * 0.5);
        top_grad.add_color_stop(0, "#601820");
        top_grad.add_color_stop(1, "#803040");
        ctx.set_fill_style(&top_grad);
        ctx.fill_rect(0, 0, w, h * 0.45);
        // Bottom orange band.
        let bottom_grad = ctx.create_linear_gradient(0, h * 0.55, 0, h);
        bottom_grad.add_color_stop(0, "#d06030");
        bottom_grad.add_color_stop(1, "#a04020");
        ctx.set_fill_style(&bottom_grad);
        ctx.fill_rect(0, h * 0.55, w, h * 0.45);
        // Soft transition band in middle.
        ctx.set_fill_style("rgba(100,50,40,0.3)");
        ctx.fill_rect(0, h * 0.42, w, h * 0.16);
        // Add slight edge fuzz to simulate hand-painting.
        ctx.set_fill_style("rgba(0,0,0,0.10)");
        for _ in 0..120 {
            let fx = rnd() * w;
            let fy = rnd() * h;
            ctx.fill_rect(fx, fy, 2, 2);
        }
    }
    // Optional artist signature bottom-right (small).
    ctx.set_fill_style("#302020");
    ctx.set_font("italic 8px cursive");
    ctx.set_text_align("right");
    let signatures = ["M. Klein '62", "E. Vasquez", "J.M. '78", "A. Rodriguez", "S. Chen"];
    ctx.fill_text(choice(&signatures), w - 8.0, h - 6.0);
}));

// ------------------------------------------------------------------ line 24606
fn place_abstract_paintings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x101010, 0x606060, 0xd0d0d0, 0x604020];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let canvas_mat = lambert!({ map: abstract_modernist_texture() });
    let ap = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.28, 0.28);
    let frame = mesh(pg("AbstractPaintings__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    ap.add(&frame);
    let canvas = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &canvas_mat);
    canvas.position().set(0, mount_y, 0.017);
    ap.add(&canvas);
    mount_on_wall(g, &ap, x, z, dx, dz, 0.015);
    ap.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&ap);
}

// ------------------------------------------------------------------ line 24643
// R720 kitchen calendar w/ marked dates (wall).
game::lazy_static_tex!(kitchen_calendar_texture, canvas_tex(220, 260, |ctx, w, h| {
    // White paper background.
    ctx.set_fill_style("#f8f4e8");
    ctx.fill_rect(0, 0, w, h);
    // Top decorative image band (a simple gradient — pastoral scene).
    let top_band = h * 0.36;
    let grad = ctx.create_linear_gradient(0, 0, 0, top_band);
    grad.add_color_stop(0, "#a0d0e8");
    grad.add_color_stop(1, "#d0e0a0");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, top_band);
    // Simple silhouettes in top image — hills + tree.
    ctx.set_fill_style("#405030");
    ctx.begin_path();
    ctx.move_to(0, top_band);
    ctx.line_to(w * 0.20, top_band * 0.65);
    ctx.line_to(w * 0.45, top_band * 0.75);
    ctx.line_to(w * 0.70, top_band * 0.55);
    ctx.line_to(w, top_band * 0.70);
    ctx.line_to(w, top_band);
    ctx.close_path();
    ctx.fill();
    // Small tree.
    ctx.set_fill_style("#304020");
    ctx.begin_path();
    ctx.arc(w * 0.15, top_band * 0.60, 12, 0, PI * 2.0);
    ctx.fill();
    ctx.set_fill_style("#603020");
    ctx.fill_rect(w * 0.15 - 1.5, top_band * 0.60, 3, 12);
    // Month name.
    ctx.set_fill_style("#302020");
    ctx.set_font("bold 20px serif");
    ctx.set_text_align("center");
    let months = ["JANUARY", "MARCH", "JUNE", "SEPTEMBER", "NOVEMBER", "DECEMBER"];
    let month = choice(&months);
    ctx.fill_text(month, w / 2.0, top_band + 26.0);
    // Year.
    ctx.set_font("italic 14px serif");
    let year = 1985 + (rnd() * 12.0).floor() as i64;
    ctx.fill_text(&year.to_string(), w / 2.0, top_band + 46.0);
    // Days of week header.
    ctx.set_font("bold 10px sans-serif");
    let days = ["S", "M", "T", "W", "T", "F", "S"];
    let grid_top = top_band + 60.0;
    let cell_w = w / 8.0;
    for i in 0..7 {
        ctx.fill_text(days[i], cell_w * (i + 1) as f64, grid_top);
    }
    // Grid lines + date numbers (5 weeks, mostly-filled).
    ctx.set_font("11px sans-serif");
    ctx.set_stroke_style("rgba(100,80,60,0.25)");
    ctx.set_line_width(0.5);
    let cell_h = (h - grid_top - 20.0) / 5.0;
    let start_offset = (rnd() * 4.0).floor() as i32; // Start day of week for the 1st.
    let mut day_num = 1;
    for row in 0..5 {
        for col in 0..7 {
            let cell_x = cell_w * (col + 1) as f64;
            let cell_y = grid_top + 12.0 + row as f64 * cell_h;
            // Grid line to bottom of cell.
            ctx.begin_path();
            ctx.move_to(cell_x - cell_w / 2.0, cell_y + cell_h * 0.85);
            ctx.line_to(cell_x + cell_w / 2.0, cell_y + cell_h * 0.85);
            ctx.stroke();
            // Fill number if in valid range.
            let cell_idx = row * 7 + col;
            if cell_idx >= start_offset && day_num <= 30 {
                ctx.set_fill_style("#302020");
                ctx.fill_text(&day_num.to_string(), cell_x, cell_y + cell_h * 0.3);
                day_num += 1;
            }
        }
    }
    // 2-4 red-circled dates (implies marked events).
    ctx.set_stroke_style("#a02040");
    ctx.set_line_width(2);
    let mark_count = 2 + (rnd() * 3.0).floor() as i32;
    for _m in 0..mark_count {
        let row = (rnd() * 5.0).floor() as i32;
        let col = (rnd() * 7.0).floor() as i32;
        let cell_idx = row * 7 + col;
        if cell_idx >= start_offset && cell_idx - start_offset < 30 {
            let cx = cell_w * (col + 1) as f64;
            let cy = grid_top + 12.0 + row as f64 * cell_h + cell_h * 0.3;
            ctx.begin_path();
            ctx.arc(cx, cy, 8, 0, PI * 2.0);
            ctx.stroke();
        }
    }
    // Bottom company/brand text.
    ctx.set_font("italic 9px serif");
    ctx.set_fill_style("rgba(70,50,40,0.6)");
    ctx.set_text_align("center");
    let brands = ["Farmer's Almanac", "Country Living", "Home & Garden", "Nature's Beauty"];
    ctx.fill_text(choice(&brands), w / 2.0, h - 8.0);
}));

// ------------------------------------------------------------------ line 24739
fn place_kitchen_calendars(g: &mut Game, _dist: &[i32]) {
    g.pp.kitchen_calendar_positions = vec![]; // R735 reset per level
    if rnd() < 0.97 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let paper_mat = lambert!({ map: kitchen_calendar_texture() });
    let kc = group();
    let mount_y = 1.60;
    // Wall-hung calendar — no frame, just paper w/ small hanging nail.
    let (cal_w, cal_h) = (0.24, 0.28);
    let paper = mesh(plane_geo!(cal_w, cal_h), &paper_mat);
    paper.position().set(0, mount_y, 0.010);
    kc.add(&paper);
    // Small dark nail head visible at top center.
    let nail_mat = lambert!({ color: 0x606060 });
    let nail = mesh(pg("KitchenCalendars__nail", cyl_geo!(0.003, 0.003, 0.004, 5)), &nail_mat);
    nail.rotation().x = PI / 2.0;
    nail.position().set(0, mount_y + cal_h * 0.5 + 0.002, 0.012);
    kc.add(&nail);
    // Wire/ring loop just below nail (small torus).
    let wire_mat = lambert!({ color: 0x808080 });
    let wire = mesh(pg("KitchenCalendars__wire", torus_geo!(0.005, 0.0008, 3, 8)), &wire_mat);
    wire.rotation().x = PI / 2.0;
    wire.position().set(0, mount_y + cal_h * 0.5 - 0.005, 0.011);
    kc.add(&wire);
    mount_on_wall(g, &kc, x, z, dx, dz, 0.015);
    kc.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&kc);
    let p = kc.pos();
    g.pp.kitchen_calendar_positions.push(v3(p.x, 0, p.z)); // R735
}

// ------------------------------------------------------------------ line 24784
// R720 pet bowls — dog food + water bowl pair (floor): 4% chance per level.
fn place_pet_bowls(g: &mut Game, _dist: &[i32]) {
    g.pp.pet_bowl_positions = vec![]; // R723 reset per level
    if rnd() < 0.96 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bowl_colors = [0xa02020, 0x2060a0, 0x408060, 0xd0a040, 0x606060, 0xf0e8d0, 0xa060a0];
    // 2 bowls — different colors per pair.
    let food_bowl_mat = lambert!({ color: choice(&bowl_colors) });
    let water_bowl_mat = lambert!({ color: choice(&bowl_colors) });
    let pb = group();
    let (bowl_r, bowl_h) = (0.09, 0.045);
    // Food bowl (left).
    let food_bowl = mesh(pg("PetBowls__bowl", cyl_geo!(bowl_r, bowl_r * 0.85, bowl_h, 12, 1, true)), &food_bowl_mat);
    food_bowl.position().set(-0.13, bowl_h / 2.0 + 0.002, 0);
    pb.add(&food_bowl);
    // Food bowl bottom disc.
    let food_bottom = mesh(circle_geo!(bowl_r * 0.85, 12), &food_bowl_mat);
    food_bottom.rotation().x = -PI / 2.0;
    food_bottom.position().set(-0.13, 0.002, 0);
    pb.add(&food_bottom);
    // Food bowl rim torus.
    let food_rim = mesh(pg("PetBowls__rim", torus_geo!(bowl_r, 0.004, 3, 12)), &food_bowl_mat);
    food_rim.rotation().x = PI / 2.0;
    food_rim.position().set(-0.13, bowl_h + 0.002, 0);
    pb.add(&food_rim);
    // 70% chance: kibble inside — brown scattered spheres.
    if rnd() < 0.7 {
        let kibble_mat = lambert!({ color: 0x8b5020 });
        let kibble_count = 12 + (rnd() * 8.0).floor() as i32;
        for i in 0..kibble_count {
            let k_r = 0.008 + rnd() * 0.004;
            let kibble = mesh(pg("PetBowls__kibble", sphere_geo!(k_r, 5, 4)), &kibble_mat);
            let ang = rnd() * PI * 2.0;
            let rr = rnd() * bowl_r * 0.75;
            let ky = 0.015 + rnd() * 0.010;
            kibble.position().set(-0.13 + ang.cos() * rr, ky, ang.sin() * rr);
            pb.add(&kibble);
        }
    }
    // Water bowl (right).
    let water_bowl = mesh(pg("PetBowls__bowl", cyl_geo!(bowl_r, bowl_r * 0.85, bowl_h, 12, 1, true)), &water_bowl_mat);
    water_bowl.position().set(0.13, bowl_h / 2.0 + 0.002, 0);
    pb.add(&water_bowl);
    let water_bottom = mesh(circle_geo!(bowl_r * 0.85, 12), &water_bowl_mat);
    water_bottom.rotation().x = -PI / 2.0;
    water_bottom.position().set(0.13, 0.002, 0);
    pb.add(&water_bottom);
    let water_rim = mesh(pg("PetBowls__rim", torus_geo!(bowl_r, 0.004, 3, 12)), &water_bowl_mat);
    water_rim.rotation().x = PI / 2.0;
    water_rim.position().set(0.13, bowl_h + 0.002, 0);
    pb.add(&water_rim);
    // 80% chance: water surface disc inside.
    let has_water = rnd() < 0.8;
    if has_water {
        let water_mat = basic_mat!({ color: 0x506080, transparent: true, opacity: 0.65 });
        let water = mesh(circle_geo!(bowl_r * 0.82, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.position().set(0.13, bowl_h * 0.75, 0);
        pb.add(&water);
    }
    // 50% chance: tiny mat under both bowls.
    let mut mat_lift = 0.0;
    if rnd() < 0.5 {
        let mat_colors = [0x404040, 0x603020, 0x2060a0, 0x408060];
        let mat_mat = lambert!({ color: choice(&mat_colors) });
        let mat = mesh(pg("PetBowls__mat", box_geo!(0.35, 0.005, 0.16)), &mat_mat);
        mat.position().y = 0.0025;
        // Bowls (bottoms at 0.002) stand on the mat's top (0.005), not in it.
        mat_lift = 0.003;
        for c in pb.children() { c.position().y += mat_lift; }
        pb.add(&mat);
    }
    // Optional name label near bowls — 40% chance (tiny label plate).
    if rnd() < 0.4 {
        let label_mat = lambert!({ color: 0xc0a060 });
        let label = mesh(pg("PetBowls__label", box_geo!(0.04, 0.001, 0.010)), &label_mat);
        // Flat on the food bowl's flared outer wall (slope 0.3), facing out.
        label.position().set(-0.13, 0.032 + mat_lift, 0.0765 + 0.3 * 0.030);
        label.rotation().x = PI / 2.0 + 0.3f64.atan();
        pb.add(&label);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    pb.position().set(px, 0, pz);
    pb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pb);
    // R723 register position only if water bowl variant has water.
    if has_water {
        let p = pb.pos();
        g.pp.pet_bowl_positions.push(v3(p.x, 0, p.z));
    }
}

// ------------------------------------------------------------------ line 24892
// R722 pet toy — squeaky bone or ball (floor): 7% chance per level.
fn place_pet_toys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pt = group();
    let is_bone = rnd() < 0.55;
    if is_bone {
        // Squeaky bone — 2 rounded ends w/ narrow shaft between.
        let bone_colors = [0xf0e0c0, 0xd0c0a0, 0xf0d8b0, 0xe0d0a0];
        let bone_mat = lambert!({ color: choice(&bone_colors) });
        // Central shaft.
        let shaft_len = 0.14;
        let shaft = mesh(pg("PetToys__shaft", cyl_geo!(0.014, 0.014, shaft_len, 8)), &bone_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().y = 0.020;
        pt.add(&shaft);
        // 4 rounded "knob" ends (2 spheres per end, offset).
        for side in [-1.0f64, 1.0] {
            for off_z in [-0.014, 0.014] {
                let knob = mesh(pg("PetToys__knob", sphere_geo!(0.020, 6, 5)), &bone_mat);
                knob.position().set(side * shaft_len / 2.0, 0.020, off_z);
                pt.add(&knob);
            }
        }
        // Small tooth-mark divots — 2-3 dark spots on shaft.
        let mark_count = 2 + (rnd() * 2.0).floor() as i32;
        for i in 0..mark_count {
            let mark_mat = lambert!({ color: 0x604020 });
            let mark = mesh(circle_geo!(0.004, 5), &mark_mat);
            let mark_x = (rnd() - 0.5) * shaft_len * 0.7;
            let mz = (rnd() - 0.5) * 0.010;
            mark.position().set(mark_x, 0.020 + 0.015, mz);
            mark.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.5;
            pt.add(&mark);
        }
        // Slight tilt as if fallen over: a roll about its long axis, lifted so
        // the lower knobs rest on the floor. Y last keeps the random heading.
        let roll = (rnd() - 0.5) * 0.25;
        pt.set_rotation_order(EulerOrder::YXZ);
        pt.rotation().x = roll;
        pt.position().y = 0.020 * (1.0 - roll.cos()) + 0.014 * roll.sin().abs();
    } else {
        // Rubber ball — 1 vibrant sphere w/ tooth marks.
        let ball_colors = [0xa02040, 0x2060a0, 0xf0d040, 0x40a030, 0xf07020, 0xa040a0];
        let ball_mat = lambert!({ color: choice(&ball_colors) });
        let ball_r = 0.038;
        let ball = mesh(pg("PetToys__ball", sphere_geo!(ball_r, 10, 8)), &ball_mat);
        ball.position().y = ball_r;
        pt.add(&ball);
        // 2-4 dark dimple/bite marks scattered on ball.
        let mark_count = 2 + (rnd() * 3.0).floor() as i32;
        for i in 0..mark_count {
            let u = rnd() * PI * 2.0;
            let v = rnd() * (PI * 0.6) + PI * 0.2;
            let mark = mesh(circle_geo!(0.006, 5), lambert!({ color: 0x201010 }));
            let mx = u.cos() * v.sin() * ball_r * 1.005;
            let my = v.cos() * ball_r * 1.005;
            let mz = u.sin() * v.sin() * ball_r * 1.005;
            mark.position().set(mx, ball_r + my, mz);
            mark.look_at(0, ball_r, 0);
            // Flip so face-out.
            mark.rotation().y += PI;
            pt.add(&mark);
        }
        // Optional decorative white stripe across ball equator.
        if rnd() < 0.4 {
            let stripe_mat = lambert!({ color: 0xf0f0f0 });
            let stripe = mesh(pg("PetToys__stripe", torus_geo!(ball_r * 1.005, 0.003, 3, 12)), &stripe_mat);
            stripe.rotation().x = PI / 2.0;
            stripe.position().y = ball_r;
            pt.add(&stripe);
        }
    }
    let py = pt.pos().y; // the bone's roll lift (0 otherwise)
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    pt.position().set(px, py, pz);
    pt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pt);
}

// ------------------------------------------------------------------ line 24984
// R722 framed pet portrait (wall).
game::lazy_static_tex!(pet_portrait_texture, canvas_tex(200, 220, |ctx, w, h| {
    // Muted studio-portrait background — gradient.
    let is_dog = rnd() < 0.5;
    let grad = ctx.create_radial_gradient(w / 2.0, h * 0.4, 20, w / 2.0, h * 0.4, w * 0.7);
    grad.add_color_stop(0, "#8a9080");
    grad.add_color_stop(1, "#404838");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Vignette.
    let vg = ctx.create_radial_gradient(w / 2.0, h * 0.4, 30, w / 2.0, h * 0.4, w * 0.8);
    vg.add_color_stop(0, "rgba(0,0,0,0)");
    vg.add_color_stop(1, "rgba(0,0,0,0.4)");
    ctx.set_fill_style(&vg);
    ctx.fill_rect(0, 0, w, h);
    // Pet silhouette — soft-focus body.
    let (cx, cy) = (w / 2.0, h * 0.55);
    if is_dog {
        // Dog body — larger elongated ellipse w/ head above.
        let body_colors = ["#805030", "#4a3020", "#c0a070", "#e0c890", "#302015"];
        let body_color = choice(&body_colors);
        let c_aa = format!("{}aa", body_color);
        let c_00 = format!("{}00", body_color);
        // Body ellipse.
        let body_grad = ctx.create_radial_gradient(cx, cy + 20.0, 8, cx, cy + 20.0, 55);
        body_grad.add_color_stop(0, body_color);
        body_grad.add_color_stop(0.7, &c_aa);
        body_grad.add_color_stop(1, &c_00);
        ctx.set_fill_style(&body_grad);
        ctx.begin_path();
        ctx.ellipse(cx, cy + 20.0, 45, 55, 0, 0, PI * 2.0);
        ctx.fill();
        // Head — round + slightly forward.
        let head_grad = ctx.create_radial_gradient(cx, cy - 40.0, 5, cx, cy - 40.0, 32);
        head_grad.add_color_stop(0, body_color);
        head_grad.add_color_stop(0.7, &c_aa);
        head_grad.add_color_stop(1, &c_00);
        ctx.set_fill_style(&head_grad);
        ctx.begin_path();
        ctx.arc(cx, cy - 40.0, 32, 0, PI * 2.0);
        ctx.fill();
        // 2 ears — floppy or perky.
        ctx.set_fill_style(body_color);
        ctx.begin_path();
        ctx.ellipse(cx - 22.0, cy - 55.0, 8, 18, -0.3, 0, PI * 2.0);
        ctx.fill();
        ctx.begin_path();
        ctx.ellipse(cx + 22.0, cy - 55.0, 8, 18, 0.3, 0, PI * 2.0);
        ctx.fill();
        // Snout suggestion — small darker oval below head center.
        ctx.set_fill_style("rgba(0,0,0,0.35)");
        ctx.begin_path();
        ctx.ellipse(cx, cy - 30.0, 10, 6, 0, 0, PI * 2.0);
        ctx.fill();
        // Small black nose.
        ctx.set_fill_style("#101010");
        ctx.begin_path();
        ctx.arc(cx, cy - 32.0, 3, 0, PI * 2.0);
        ctx.fill();
        // 2 dark eye dots.
        ctx.begin_path();
        ctx.arc(cx - 8.0, cy - 44.0, 2, 0, PI * 2.0);
        ctx.arc(cx + 8.0, cy - 44.0, 2, 0, PI * 2.0);
        ctx.fill();
    } else {
        // Cat body — smaller, more compact ellipse.
        let cat_colors = ["#404040", "#a08050", "#f0e0d0", "#8b5a20", "#606060"];
        let cat_color = choice(&cat_colors);
        let c_aa = format!("{}aa", cat_color);
        let c_00 = format!("{}00", cat_color);
        // Body — squat oval.
        let body_grad = ctx.create_radial_gradient(cx, cy + 25.0, 8, cx, cy + 25.0, 45);
        body_grad.add_color_stop(0, cat_color);
        body_grad.add_color_stop(0.7, &c_aa);
        body_grad.add_color_stop(1, &c_00);
        ctx.set_fill_style(&body_grad);
        ctx.begin_path();
        ctx.ellipse(cx, cy + 25.0, 38, 45, 0, 0, PI * 2.0);
        ctx.fill();
        // Head — smaller round.
        let head_grad = ctx.create_radial_gradient(cx, cy - 25.0, 5, cx, cy - 25.0, 28);
        head_grad.add_color_stop(0, cat_color);
        head_grad.add_color_stop(0.7, &c_aa);
        head_grad.add_color_stop(1, &c_00);
        ctx.set_fill_style(&head_grad);
        ctx.begin_path();
        ctx.arc(cx, cy - 25.0, 28, 0, PI * 2.0);
        ctx.fill();
        // 2 pointed triangle ears.
        ctx.set_fill_style(cat_color);
        ctx.begin_path();
        ctx.move_to(cx - 24.0, cy - 45.0);
        ctx.line_to(cx - 12.0, cy - 55.0);
        ctx.line_to(cx - 8.0, cy - 40.0);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(cx + 24.0, cy - 45.0);
        ctx.line_to(cx + 12.0, cy - 55.0);
        ctx.line_to(cx + 8.0, cy - 40.0);
        ctx.close_path();
        ctx.fill();
        // Snout.
        ctx.set_fill_style("rgba(0,0,0,0.3)");
        ctx.begin_path();
        ctx.ellipse(cx, cy - 18.0, 8, 5, 0, 0, PI * 2.0);
        ctx.fill();
        // Pink nose triangle.
        ctx.set_fill_style("#c06070");
        ctx.begin_path();
        ctx.move_to(cx - 3.0, cy - 22.0);
        ctx.line_to(cx + 3.0, cy - 22.0);
        ctx.line_to(cx, cy - 18.0);
        ctx.close_path();
        ctx.fill();
        // 2 vertical-slit yellow eyes.
        ctx.set_fill_style("#f0e040");
        ctx.begin_path();
        ctx.ellipse(cx - 8.0, cy - 30.0, 3, 5, 0, 0, PI * 2.0);
        ctx.ellipse(cx + 8.0, cy - 30.0, 3, 5, 0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#101010");
        ctx.begin_path();
        ctx.ellipse(cx - 8.0, cy - 30.0, 1, 4, 0, 0, PI * 2.0);
        ctx.ellipse(cx + 8.0, cy - 30.0, 1, 4, 0, 0, PI * 2.0);
        ctx.fill();
        // Whiskers (thin lines).
        ctx.set_stroke_style("rgba(240,240,240,0.6)");
        ctx.set_line_width(0.8);
        for i in [-1.0f64, 1.0] {
            ctx.begin_path();
            ctx.move_to(cx + i * 8.0, cy - 20.0);
            let y1 = cy - 22.0 + rnd() * 4.0;
            ctx.line_to(cx + i * 22.0, y1);
            ctx.move_to(cx + i * 8.0, cy - 18.0);
            let y2 = cy - 16.0 + rnd() * 4.0;
            ctx.line_to(cx + i * 22.0, y2);
            ctx.stroke();
        }
    }
    // Optional collar suggestion — colored band around neck.
    if rnd() < 0.7 {
        let collar_colors = ["#a02040", "#2060a0", "#c0a040", "#a04080"];
        ctx.set_stroke_style(choice(&collar_colors));
        ctx.set_line_width(4);
        ctx.begin_path();
        if is_dog {
            ctx.move_to(cx - 22.0, cy - 15.0);
            ctx.quadratic_curve_to(cx, cy - 10.0, cx + 22.0, cy - 15.0);
        } else {
            ctx.move_to(cx - 18.0, cy - 5.0);
            ctx.quadratic_curve_to(cx, cy, cx + 18.0, cy - 5.0);
        }
        ctx.stroke();
        // Small tag on collar.
        ctx.set_fill_style("#c0a060");
        ctx.begin_path();
        ctx.arc(cx, if is_dog { cy - 5.0 } else { cy + 5.0 }, 4, 0, PI * 2.0);
        ctx.fill();
    }
    // Pet name at bottom.
    let dog_names = ["Buddy", "Max", "Rocky", "Charlie", "Duke", "Bailey"];
    let cat_names = ["Mittens", "Whiskers", "Luna", "Oliver", "Simba", "Nala"];
    let names = if is_dog { dog_names } else { cat_names };
    ctx.set_fill_style("#f0e0c0");
    ctx.set_font("italic bold 16px cursive");
    ctx.set_text_align("center");
    ctx.fill_text(choice(&names), w / 2.0, h - 20.0);
    // Years lived below.
    let y1 = 1970 + (rnd() * 20.0).floor() as i64;
    let y2 = y1 + 8 + (rnd() * 8.0).floor() as i64;
    ctx.set_font("italic 11px cursive");
    ctx.fill_text(&format!("{} - {}", y1, y2), w / 2.0, h - 6.0);
}));

// ------------------------------------------------------------------ line 25151
fn place_pet_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0xd0a040, 0x8b5a2b, 0x2a1a10];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let portrait_mat = lambert!({ map: pet_portrait_texture() });
    let pp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.24);
    let frame = mesh(pg("PetPortraits__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    pp.add(&frame);
    let portrait = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &portrait_mat);
    portrait.position().set(0, mount_y, 0.019);
    pp.add(&portrait);
    mount_on_wall(g, &pp, x, z, dx, dz, 0.015);
    pp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&pp);
}

// ------------------------------------------------------------------ line 25188
// R724 dog leash coiled + hanging harness (floor): 5% chance per level.
fn place_dog_leashes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leash_colors = [0xa02040, 0x2060a0, 0x101010, 0x604020, 0x408060, 0xd0a040, 0xa060a0];
    let leash_mat = lambert!({ color: choice(&leash_colors) });
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let dl = group();
    // Coiled leash — 3 concentric flattened torus rings stacked to look like a coil.
    for i in 0..3 {
        let coil = mesh(pg("DogLeashes__coil", torus_geo!(0.05 - i as f64 * 0.008, 0.006, 4, 12)), &leash_mat);
        coil.rotation().x = PI / 2.0;
        coil.position().y = 0.004 + i as f64 * 0.006;
        dl.add(&coil);
    }
    // Handle loop — larger torus offset to side.
    let handle = mesh(pg("DogLeashes__handle", torus_geo!(0.030, 0.006, 4, 10)), &leash_mat);
    handle.rotation().x = 0.3;
    handle.position().set(0.06, 0.018, 0.02);
    dl.add(&handle);
    // Metal clip at end (small D-ring).
    let clip = mesh(pg("DogLeashes__clip", torus_geo!(0.010, 0.002, 3, 8)), &metal_mat);
    clip.rotation().x = PI / 2.0;
    clip.position().set(-0.055, 0.010, 0.03);
    dl.add(&clip);
    // Metal snap-hook bar.
    let clip_bar = mesh(pg("DogLeashes__snap", box_geo!(0.014, 0.003, 0.006)), &metal_mat);
    clip_bar.position().set(-0.062, 0.010, 0.03);
    dl.add(&clip_bar);
    // 60% chance: dog harness nearby (thicker strap loop + 2 buckles).
    if rnd() < 0.6 {
        let harness_color = choice(&leash_colors);
        let harness_mat = lambert!({ color: harness_color });
        // Chest strap — larger rectangular loop.
        let chest_loop = mesh(pg("DogLeashes__chest", torus_geo!(0.06, 0.008, 4, 14)), &harness_mat);
        chest_loop.rotation().x = -PI / 2.0;
        chest_loop.rotation().z = 0.2;
        chest_loop.position().set(-0.12, 0.010, -0.02);
        dl.add(&chest_loop);
        // Neck strap — smaller loop.
        let neck_loop = mesh(pg("DogLeashes__neck", torus_geo!(0.038, 0.006, 4, 12)), &harness_mat);
        neck_loop.rotation().x = -PI / 2.0;
        neck_loop.rotation().z = -0.2;
        neck_loop.position().set(-0.16, 0.008, -0.05);
        dl.add(&neck_loop);
        // 2 metal buckles connecting straps.
        for off in [[-0.11, -0.06], [-0.16, 0.0]] {
            let buckle = mesh(pg("DogLeashes__buckle", box_geo!(0.014, 0.008, 0.010)), &metal_mat);
            buckle.position().set(off[0], 0.010, off[1]);
            dl.add(&buckle);
        }
        // Small metal D-ring on chest strap for leash attachment.
        let d_ring = mesh(pg("DogLeashes__dRing", torus_geo!(0.008, 0.0015, 3, 8)), &metal_mat);
        d_ring.rotation().x = 0.0;
        d_ring.position().set(-0.06, 0.010, -0.02);
        dl.add(&d_ring);
        // 30% chance: dog tag on chest strap (small circular disc).
        if rnd() < 0.3 {
            let tag_mat = lambert!({ color: 0xc0a060 });
            let tag = mesh(pg("DogLeashes__tag", cyl_geo!(0.010, 0.010, 0.002, 12)), &tag_mat);
            tag.position().set(-0.06, 0.006, -0.014);
            dl.add(&tag);
            // Small text-line indicator.
            let text_mat = lambert!({ color: 0x101010 });
            let text_line = mesh(box_geo!(0.008, 0.001, 0.001), &text_mat);
            text_line.position().set(-0.06, 0.008, -0.014);
            dl.add(&text_line);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    dl.position().set(px, 0, pz);
    dl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&dl);
}

// ------------------------------------------------------------------ line 25276
// R724 small ornate music box (floor): 4% chance per level.
fn place_ornate_music_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.music_box_positions = vec![]; // R725 reset per level
    if rnd() < 0.96 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let box_colors = [0x603020, 0x8b5a2b, 0x2a1810, 0x604040, 0x201010];
    let box_mat = lambert!({ color: choice(&box_colors) });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let velvet_colors = [0xa02020, 0x2060a0, 0x404060, 0x604060, 0x603030];
    let velvet_mat = lambert!({ color: choice(&velvet_colors) });
    let mb = group();
    let (box_w, box_d, box_h) = (0.14, 0.10, 0.055);
    // Bottom half.
    let bottom = mesh(pg("OrnateMusicBoxes__bottom", box_geo!(box_w, box_h, box_d)), &box_mat);
    bottom.position().y = box_h / 2.0 + 0.002;
    mb.add(&bottom);
    // Gold trim edges.
    let trim_top = mesh(pg("OrnateMusicBoxes__trim", box_geo!(box_w + 0.004, 0.003, box_d + 0.004)), &gold_mat);
    trim_top.position().y = box_h + 0.002;
    mb.add(&trim_top);
    let trim_bottom = mesh(pg("OrnateMusicBoxes__trim", box_geo!(box_w + 0.004, 0.003, box_d + 0.004)), &gold_mat);
    trim_bottom.position().y = 0.005;
    mb.add(&trim_bottom);
    // Top lid — tilted open at -PI/3 rad.
    let top = mesh(pg("OrnateMusicBoxes__lid", box_geo!(box_w, box_h * 0.35, box_d)), &box_mat);
    // Hinged on the casket's back top edge (on the trim), opened 60°.
    let lid_hinge = group();
    lid_hinge.position().set(0, box_h + 0.0035, -box_d * 0.5);
    lid_hinge.rotation().x = -PI / 3.0;
    mb.add(&lid_hinge);
    top.position().set(0, box_h * 0.175, box_d * 0.5);
    lid_hinge.add(&top);
    // Decorative gold pattern on outer top (4 tiny corner ornaments).
    for cx in [-box_w * 0.4, box_w * 0.4] {
        for cz in [-box_d * 0.35, box_d * 0.35] {
            let orn = mesh(pg("OrnateMusicBoxes__orn", sphere_geo!(0.003, 5, 4)), &gold_mat);
            orn.position().set(cx, box_h + box_d * 0.72, cz - box_d * 0.5 + (cz - box_d * 0.35).abs() * 0.2);
            // On the lid's outer border, in the hinge frame (so it rides the lid).
            orn.position().set(cx * 1.12, box_h * 0.35 - 0.001, box_d * 0.5 + cz * 1.2);
            lid_hinge.add(&orn);
        }
    }
    // Velvet lining inside bottom.
    let lining = mesh(pg("OrnateMusicBoxes__lining", box_geo!(box_w - 0.010, 0.003, box_d - 0.010)), &velvet_mat);
    lining.position().y = box_h + 0.002;
    mb.add(&lining);
    // Tiny mirror on interior top (reflective silver square).
    let mirror_mat = basic_mat!({ color: 0xe8f0f4 });
    let mirror = mesh(box_geo!(box_w * 0.6, 0.001, box_d * 0.55), &mirror_mat);
    // In the lid's well (hinge frame), just under the well ceiling.
    mirror.position().set(0, box_h * 0.175 + 0.0002, box_d * 0.5);
    lid_hinge.add(&mirror);
    // Tiny ballerina figure on spring.
    let ballerina_mat = lambert!({ color: 0xf0d0d8 });
    let skirt_mat = lambert!({ color: 0xf8f0f8 });
    let ballerina_group = group();
    // Post/spring.
    let post = mesh(pg("OrnateMusicBoxes__post", cyl_geo!(0.001, 0.001, 0.020, 3)), &gold_mat);
    post.position().y = 0.010;
    ballerina_group.add(&post);
    // Body — thin cylinder.
    let body = mesh(pg("OrnateMusicBoxes__body", cyl_geo!(0.003, 0.004, 0.014, 5)), &ballerina_mat);
    body.position().y = 0.028;
    ballerina_group.add(&body);
    // Head — small sphere.
    let head = mesh(pg("OrnateMusicBoxes__head", sphere_geo!(0.004, 5, 4)), &ballerina_mat);
    head.position().y = 0.037;
    ballerina_group.add(&head);
    // Skirt — flat wide cone.
    let skirt = mesh(pg("OrnateMusicBoxes__skirt", cone_geo!(0.010, 0.006, 8)), &skirt_mat);
    skirt.position().y = 0.022;
    ballerina_group.add(&skirt);
    // Arms — 2 tiny cylinders.
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("OrnateMusicBoxes__arm", cyl_geo!(0.001, 0.001, 0.010, 3)), &ballerina_mat);
        arm.rotation().z = side * 0.5;
        arm.position().set(side * 0.003, 0.033, 0);
        ballerina_group.add(&arm);
    }
    ballerina_group.position().set(0, box_h + 0.005, 0);
    mb.add(&ballerina_group);
    // 50% chance: ballerina spinning animator.
    let is_playing = rnd() < 0.5;
    if is_playing {
        let bg2 = ballerina_group.clone();
        g.animate(move |_g, dt, _t| {
            bg2.rotation().y += dt * 2.5; // ~24 RPM
        });
    }
    mb.user().set("isPlaying", is_playing);
    // Small winding key on side — cylinder + tiny cross bar.
    let key_shaft = mesh(pg("OrnateMusicBoxes__keyShaft", cyl_geo!(0.003, 0.003, 0.015, 5)), &gold_mat);
    key_shaft.rotation().z = PI / 2.0;
    key_shaft.position().set(box_w * 0.55, box_h * 0.5, 0);
    mb.add(&key_shaft);
    let key_bar = mesh(pg("OrnateMusicBoxes__keyBar", box_geo!(0.003, 0.014, 0.003)), &gold_mat);
    key_bar.position().set(box_w * 0.55 + 0.010, box_h * 0.5, 0);
    mb.add(&key_bar);
    let key_bar2 = mesh(pg("OrnateMusicBoxes__keyBar2", box_geo!(0.003, 0.003, 0.014)), &gold_mat);
    key_bar2.position().set(box_w * 0.55 + 0.010, box_h * 0.5, 0);
    mb.add(&key_bar2);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    mb.position().set(px, 0, pz);
    mb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mb);
    // R725 register position only for playing (spinning) music boxes.
    if is_playing {
        let p = mb.pos();
        g.pp.music_box_positions.push(v3(p.x, 0, p.z));
    }
}

// ------------------------------------------------------------------ line 25404
// R725 framed vintage travel poster (wall).
game::lazy_static_tex!(travel_poster_texture, canvas_tex(180, 260, |ctx, w, h| {
    // Base color varies per destination.
    struct V { bg: &'static str, accent: &'static str, name: &'static str, tag: &'static str, scene: &'static str }
    let variants = [
        V { bg: "#3080b0", accent: "#f0c040", name: "PARIS", tag: "City of Light", scene: "eiffel" },
        V { bg: "#e0a060", accent: "#a02040", name: "MOROCCO", tag: "Ancient Sands", scene: "desert" },
        V { bg: "#4090b0", accent: "#f0e0c0", name: "AMALFI", tag: "Coastal Beauty", scene: "sea" },
        V { bg: "#a04030", accent: "#e0d090", name: "GRAND CANYON", tag: "Timeless Wonder", scene: "canyon" },
        V { bg: "#204080", accent: "#f0e040", name: "NEW YORK", tag: "The Big Apple", scene: "skyline" },
        V { bg: "#606030", accent: "#c04030", name: "KYOTO", tag: "Cherry Blossoms", scene: "blossom" },
    ];
    let v = &variants[(rnd() * variants.len() as f64).floor() as usize];
    // Solid color background.
    ctx.set_fill_style(v.bg);
    ctx.fill_rect(0, 0, w, h);
    // Scene illustration in middle third.
    let mid_y = h * 0.35;
    let scene_h = h * 0.40;
    if v.scene == "eiffel" {
        // Eiffel tower silhouette.
        ctx.set_fill_style("#101020");
        ctx.begin_path();
        ctx.move_to(w / 2.0, mid_y);
        ctx.line_to(w / 2.0 - 30.0, mid_y + scene_h);
        ctx.line_to(w / 2.0 - 25.0, mid_y + scene_h);
        ctx.line_to(w / 2.0 - 10.0, mid_y + scene_h * 0.7);
        ctx.line_to(w / 2.0 + 10.0, mid_y + scene_h * 0.7);
        ctx.line_to(w / 2.0 + 25.0, mid_y + scene_h);
        ctx.line_to(w / 2.0 + 30.0, mid_y + scene_h);
        ctx.close_path();
        ctx.fill();
        // Cross-brace lines.
        ctx.set_stroke_style("#101020");
        ctx.set_line_width(2);
        for i in 0..4 {
            let y = mid_y + scene_h * (0.2 + i as f64 * 0.20);
            let spread = 10.0 + i as f64 * 5.0;
            ctx.begin_path();
            ctx.move_to(w / 2.0 - spread, y);
            ctx.line_to(w / 2.0 + spread, y);
            ctx.stroke();
        }
    } else if v.scene == "desert" {
        // Sand dunes + palm.
        ctx.set_fill_style("#d0a040");
        ctx.begin_path();
        ctx.move_to(0, mid_y + scene_h);
        ctx.line_to(0, mid_y + scene_h * 0.65);
        ctx.quadratic_curve_to(w * 0.3, mid_y + scene_h * 0.55, w * 0.55, mid_y + scene_h * 0.75);
        ctx.quadratic_curve_to(w * 0.8, mid_y + scene_h * 0.90, w, mid_y + scene_h * 0.7);
        ctx.line_to(w, mid_y + scene_h);
        ctx.close_path();
        ctx.fill();
        // Simple pyramid silhouettes.
        ctx.set_fill_style("#803020");
        ctx.begin_path();
        ctx.move_to(w * 0.35, mid_y + scene_h * 0.75);
        ctx.line_to(w * 0.45, mid_y + scene_h * 0.35);
        ctx.line_to(w * 0.55, mid_y + scene_h * 0.75);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(w * 0.55, mid_y + scene_h * 0.80);
        ctx.line_to(w * 0.62, mid_y + scene_h * 0.50);
        ctx.line_to(w * 0.68, mid_y + scene_h * 0.80);
        ctx.close_path();
        ctx.fill();
        // Sun.
        ctx.set_fill_style("#f0e040");
        ctx.begin_path();
        ctx.arc(w * 0.75, mid_y + scene_h * 0.25, 20, 0, PI * 2.0);
        ctx.fill();
    } else if v.scene == "sea" {
        // Sea + boat + cliff-side houses.
        ctx.set_fill_style("#204060");
        ctx.fill_rect(0, mid_y + scene_h * 0.5, w, scene_h * 0.5);
        // Wave lines.
        ctx.set_stroke_style("#f0e0c0");
        ctx.set_line_width(1);
        for i in 0..6 {
            let y = mid_y + scene_h * 0.55 + i as f64 * 6.0;
            ctx.begin_path();
            ctx.move_to(0, y);
            let mut x = 0.0;
            while x < w {
                ctx.line_to(x + 5.0, y - 2.0);
                ctx.line_to(x + 10.0, y);
                x += 10.0;
            }
            ctx.stroke();
        }
        // Cliff-side houses (stacked white cubes on left).
        ctx.set_fill_style("#f0e0c0");
        for i in 0..4 {
            let fi = i as f64;
            ctx.fill_rect(4.0 + fi * 10.0, mid_y + scene_h * (0.20 + fi * 0.10), 24, 20);
            // Small window.
            ctx.set_fill_style("#4090b0");
            ctx.fill_rect(12.0 + fi * 10.0, mid_y + scene_h * (0.20 + fi * 0.10) + 5.0, 6, 8);
            ctx.set_fill_style("#f0e0c0");
        }
        // Small sailboat.
        ctx.set_fill_style("#a02040");
        ctx.begin_path();
        ctx.move_to(w * 0.7, mid_y + scene_h * 0.75);
        ctx.line_to(w * 0.7, mid_y + scene_h * 0.50);
        ctx.line_to(w * 0.85, mid_y + scene_h * 0.75);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style("#101020");
        ctx.fill_rect(w * 0.68, mid_y + scene_h * 0.75, 20, 5);
    } else if v.scene == "canyon" {
        // Layered canyon walls.
        ctx.set_fill_style("#803020");
        ctx.begin_path();
        ctx.move_to(0, mid_y + scene_h * 0.4);
        ctx.line_to(w * 0.2, mid_y + scene_h * 0.55);
        ctx.line_to(w * 0.35, mid_y + scene_h * 0.42);
        ctx.line_to(w * 0.55, mid_y + scene_h * 0.60);
        ctx.line_to(w * 0.75, mid_y + scene_h * 0.48);
        ctx.line_to(w, mid_y + scene_h * 0.55);
        ctx.line_to(w, mid_y + scene_h);
        ctx.line_to(0, mid_y + scene_h);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style("#602010");
        ctx.begin_path();
        ctx.move_to(0, mid_y + scene_h * 0.7);
        ctx.line_to(w * 0.3, mid_y + scene_h * 0.85);
        ctx.line_to(w * 0.6, mid_y + scene_h * 0.7);
        ctx.line_to(w, mid_y + scene_h * 0.85);
        ctx.line_to(w, mid_y + scene_h);
        ctx.line_to(0, mid_y + scene_h);
        ctx.close_path();
        ctx.fill();
        // Sun rays.
        ctx.set_stroke_style("#f0e0a0");
        ctx.set_line_width(2);
        for i in 0..5 {
            let ang = (i as f64 / 4.0) * PI - PI / 2.0;
            ctx.begin_path();
            ctx.move_to(w * 0.7, mid_y);
            ctx.line_to(w * 0.7 + ang.cos() * 60.0, mid_y + ang.sin() * 60.0);
            ctx.stroke();
        }
    } else if v.scene == "skyline" {
        // Building silhouettes.
        ctx.set_fill_style("#101020");
        struct B { x: f64, w: f64, h: f64 }
        let buildings = [
            B { x: 10.0, w: 22.0, h: scene_h * 0.65 },
            B { x: 34.0, w: 18.0, h: scene_h * 0.80 },
            B { x: 54.0, w: 26.0, h: scene_h * 0.55 },
            B { x: 82.0, w: 20.0, h: scene_h * 0.90 },
            B { x: 104.0, w: 24.0, h: scene_h * 0.70 },
            B { x: 130.0, w: 20.0, h: scene_h * 0.85 },
            B { x: 152.0, w: 20.0, h: scene_h * 0.60 },
        ];
        for b in &buildings {
            ctx.fill_rect(b.x, mid_y + scene_h - b.h, b.w, b.h);
        }
        // Yellow window dots.
        ctx.set_fill_style("#f0e040");
        for _ in 0..40 {
            let b = &buildings[(rnd() * buildings.len() as f64).floor() as usize];
            let wy = mid_y + scene_h - b.h + 5.0 + rnd() * (b.h - 10.0);
            let wxp = b.x + 3.0 + rnd() * (b.w - 6.0);
            ctx.fill_rect(wxp, wy, 2, 3);
        }
    } else if v.scene == "blossom" {
        // Cherry blossom scene. Tree trunk.
        ctx.set_fill_style("#402010");
        ctx.fill_rect(w / 2.0 - 5.0, mid_y + scene_h * 0.65, 10, scene_h * 0.35);
        // 3 curved branches.
        ctx.set_stroke_style("#402010");
        ctx.set_line_width(4);
        for angle_d in [-1.0f64, 0.5, 1.0] {
            ctx.begin_path();
            ctx.move_to(w / 2.0, mid_y + scene_h * 0.65);
            ctx.quadratic_curve_to(w / 2.0 + angle_d * 30.0, mid_y + scene_h * 0.4, w / 2.0 + angle_d * 50.0, mid_y + scene_h * 0.3);
            ctx.stroke();
        }
        // Blossom clouds.
        ctx.set_fill_style("#f0c0d0");
        for (bx, by) in [(w / 2.0 - 40.0, mid_y + scene_h * 0.25), (w / 2.0 + 40.0, mid_y + scene_h * 0.25), (w / 2.0, mid_y + scene_h * 0.1)] {
            ctx.begin_path();
            ctx.arc(bx, by, 25, 0, PI * 2.0);
            ctx.fill();
        }
        // Tiny blossom dots.
        ctx.set_fill_style("#f0d0e0");
        for _ in 0..20 {
            let bx = w * 0.2 + rnd() * w * 0.6;
            let by = mid_y + rnd() * scene_h * 0.5;
            ctx.begin_path();
            ctx.arc(bx, by, 2, 0, PI * 2.0);
            ctx.fill();
        }
    }
    // Top banner — destination name in large accent.
    ctx.set_fill_style(v.accent);
    ctx.set_font("bold 26px serif");
    ctx.set_text_align("center");
    ctx.fill_text(v.name, w / 2.0, 40);
    // Tagline below name.
    ctx.set_font("italic 11px serif");
    ctx.set_fill_style(v.accent);
    ctx.fill_text(v.tag, w / 2.0, 60);
    // Bottom "Fly Now!" or similar tagline.
    ctx.set_font("bold 12px serif");
    ctx.set_fill_style(v.accent);
    let bottoms = ["VISIT TODAY", "FLY NOW", "BOOK YOUR TRIP", "DISCOVER", "YOUR JOURNEY AWAITS"];
    ctx.fill_text(choice(&bottoms), w / 2.0, h - 20.0);
    // Small airline tag bottom-right.
    ctx.set_font("bold 9px serif");
    ctx.set_text_align("right");
    let airlines = ["TWA", "PAN AM", "TAA", "BOAC"];
    ctx.fill_text(choice(&airlines), w - 8.0, h - 6.0);
}));

// ------------------------------------------------------------------ line 25618
fn place_travel_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let walls = wall_tiles(g);
    if walls.is_empty() { return; }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x101010, 0xc0a060, 0x8b5a2b];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let poster_mat = lambert!({ map: travel_poster_texture() });
    let tp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.20, 0.28);
    let frame = mesh(pg("TravelPosters__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    tp.add(&frame);
    let poster = mesh(plane_geo!(frame_w - 0.014, frame_h - 0.014), &poster_mat);
    poster.position().set(0, mount_y, 0.017);
    tp.add(&poster);
    mount_on_wall(g, &tp, x, z, dx, dz, 0.015);
    tp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&tp);
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
