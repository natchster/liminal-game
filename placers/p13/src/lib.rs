//! Prop placers ported from backrooms.js (chunk p13; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeRecordStacks", place_record_stacks));
    r.push(("placePropagandaPosters", place_propaganda_posters));
    r.push(("placeTurntables", place_turntables));
    r.push(("placeFramedNewspapers", place_framed_newspapers));
    r.push(("placeFloorGlobes", place_floor_globes));
    r.push(("placeVintageTypewriters", place_vintage_typewriters));
    r.push(("placeAnatomyCharts", place_anatomy_charts));
    r.push(("placeEaselChalkboards", place_easel_chalkboards));
    r.push(("placeSchoolDesks", place_school_desks));
    r.push(("placePeriodicTables", place_periodic_tables));
    r.push(("placeSpilledBackpacks", place_spilled_backpacks));
    r.push(("placeFramedMaps", place_framed_maps));
}

/// Every open interior tile, row-major (`openTiles.push({ x, z })`).
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 { for x in 1..g.w - 1 { if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); } } }
    v
}
/// Open tiles next to a wall, with the first wall direction found
/// (`wallTiles.push({ x, z, dx, dz }); break;`).
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 { v.push((x, z, dx, dz)); break; }
                }
            }
        }
    }
    v
}
/// `Math.atan2(-dx, -dz)`
fn face_wall(dx: i32, dz: i32) -> f64 { (-(dx as f64)).atan2(-(dz as f64)) }

// ------------------------------------------------------------------ line 25655
fn place_record_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let rs = group();
    // 3-5 record sleeves — vertical square-ish cardboard.
    let sleeve_colors = [0xa02040, 0x2060a0, 0x408060, 0xd0a040, 0x101010, 0x804060, 0x6060a0, 0xa04020, 0x606060, 0xd0d040, 0x40a080];
    let record_count = 3 + (rnd() * 3.0).floor() as i32;
    let sleeve_w = 0.14;
    let mut stack_offset = 0.0;
    for i in 0..record_count {
        let sleeve_mat = lambert!({ color: choice(&sleeve_colors) });
        let sleeve = mesh(pg("RecordStacks__sleeve", box_geo!(sleeve_w, sleeve_w * 0.98, 0.005)), &sleeve_mat);
        // Lean slightly against back.
        let lean_amt = -0.15;
        sleeve.rotation().x = lean_amt;
        sleeve.position().set(0, sleeve_w * 0.49 - 0.0004, stack_offset); // leaned corner on the floor
        sleeve.position().x += (i as f64 - record_count as f64 / 2.0) * 0.008;
        rs.add(&sleeve);
        let sp = sleeve.pos();
        // Cover label — smaller square centered.
        let label_colors = [0xf0e0c0, 0xd0d0d0, 0xe0c0a0, 0xa0a0a0];
        let label_mat = lambert!({ color: choice(&label_colors) });
        let label = mesh(plane_geo!(sleeve_w * 0.6, sleeve_w * 0.15), &label_mat);
        label.rotation().x = lean_amt;
        label.position().set(sp.x, sp.y, stack_offset + 0.003);
        rs.add(&label);
        // Tiny center vinyl-hole hint (dark circle).
        let hole_mat = lambert!({ color: 0x0a0a0a });
        let hole = mesh(circle_geo!(sleeve_w * 0.20, 12), &hole_mat);
        hole.rotation().x = -PI / 2.0 + PI / 2.0 + lean_amt;
        hole.position().set(sp.x, sp.y, stack_offset + 0.004);
        // Actually, we want it face-out. Use rotation to face forward.
        hole.rotation().x = lean_amt;
        hole.rotation().y = 0.0;
        rs.add(&hole);
        // Center label dot on hole — spinning label color.
        let center_label_mat = lambert!({ color: choice(&[0xa02040, 0x2060a0, 0x408060, 0xd0a040]) });
        let center_label = mesh(circle_geo!(sleeve_w * 0.10, 10), &center_label_mat);
        center_label.rotation().x = lean_amt;
        center_label.position().set(sp.x, sp.y, stack_offset + 0.005);
        rs.add(&center_label);
        // Small center hole dot.
        let center_hole = mesh(circle_geo!(0.003, 6), &hole_mat);
        center_hole.rotation().x = lean_amt;
        center_hole.position().set(sp.x, sp.y, stack_offset + 0.006);
        rs.add(&center_hole);
        stack_offset += 0.0055; // 5 mm sleeves packed face to face
    }
    // 60% chance: one record out of sleeve, propped against pile.
    if rnd() < 0.6 {
        let bare_record_mat = lambert!({ color: 0x0a0a0a });
        let bare_record = mesh(pg("RecordStacks__record", cyl_geo!(sleeve_w * 0.48, sleeve_w * 0.48, 0.003, 24)), &bare_record_mat);
        bare_record.rotation().z = PI / 2.0;
        bare_record.rotation().y = 0.4;
        bare_record.position().set(sleeve_w * 0.7, sleeve_w * 0.48, 0.05);
        rs.add(&bare_record);
        // Vinyl grooves — 3 subtle concentric rings.
        let groove_mat = lambert!({ color: 0x1a1a1a });
        for i in 0..3 {
            let groove = mesh(torus_geo!(sleeve_w * (0.15 + i as f64 * 0.10), 0.0006, 3, 24), &groove_mat);
            // In the record's plane (its axis is (-cos .4, 0, sin .4)), just
            // proud of the grooved band on the +X face.
            groove.rotation().y = PI / 2.0 + 0.4;
            groove.position().set(sleeve_w * 0.7 + (0.4f64).cos() * 0.0010, sleeve_w * 0.48, 0.05 - (0.4f64).sin() * 0.0010);
            rs.add(&groove);
        }
        // Center label on bare record.
        let bare_label = mesh(circle_geo!(sleeve_w * 0.16, 10), lambert!({ color: 0xd04040 }));
        bare_label.rotation().y = PI / 2.0 + 0.4; // on the record's +X face
        bare_label.position().set(sleeve_w * 0.7 + (0.4f64).cos() * 0.0017, sleeve_w * 0.48, 0.05 - (0.4f64).sin() * 0.0017);
        rs.add(&bare_label);
    }
    rs.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    rs.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rs);
}

// ------------------------------------------------------------------ line 25853
fn place_propaganda_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x101010, 0x804a20, 0x2a2a2a];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let poster_mat = lambert!({ map: propaganda_poster_texture() });
    let pp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.20, 0.28);
    let frame = mesh(pg("PropagandaPosters__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    pp.add(&frame);
    let poster = mesh(plane_geo!(frame_w - 0.014, frame_h - 0.014), &poster_mat);
    poster.position().set(0, mount_y, 0.017);
    pp.add(&poster);
    pp.rotation().y = face_wall(dx, dz);
    pp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    pp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&pp);
}

// ------------------------------------------------------------------ line 25890
fn place_turntables(g: &mut Game, _dist: &[i32]) {
    g.pp.turntable_positions.clear(); // R729 reset per level
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x8b5a2b, 0x2a1810, 0x101010, 0x804040];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let platter_mat = lambert!({ color: 0x808088 });
    let vinyl_mat = lambert!({ color: 0x0a0a0a });
    let label_colors = [0xa02040, 0x2060a0, 0xd0a040, 0x408060, 0x606060];
    let label_mat = lambert!({ color: choice(&label_colors) });
    let chrome_mat = lambert!({ color: 0xc0c0c8 });
    let tt = group();
    // Base cabinet — rectangular wooden box.
    let (base_w, base_d, base_h) = (0.36, 0.28, 0.06);
    let base = mesh(pg("Turntables__base", box_geo!(base_w, base_h, base_d)), &wood_mat);
    base.position().y = base_h / 2.0 + 0.002;
    tt.add(&base);
    // Top platter — chrome/metal disc raised slightly.
    let platter_r = 0.12;
    let platter = mesh(pg("Turntables__platter", cyl_geo!(platter_r, platter_r, 0.008, 20)), &platter_mat);
    platter.position().set(-0.05, base_h + 0.006, 0);
    tt.add(&platter);
    // Vinyl on platter — slightly smaller dark disc.
    let vinyl = mesh(pg("Turntables__vinyl", cyl_geo!(platter_r * 0.95, platter_r * 0.95, 0.002, 24)), &vinyl_mat);
    vinyl.position().set(-0.05, base_h + 0.011, 0);
    tt.add(&vinyl);
    // Center label on vinyl.
    let center_label = mesh(pg("Turntables__label", cyl_geo!(platter_r * 0.30, platter_r * 0.30, 0.001, 12)), &label_mat);
    center_label.position().set(-0.05, base_h + 0.013, 0);
    tt.add(&center_label);
    // 3 subtle groove torus rings on vinyl.
    let groove_mat = lambert!({ color: 0x1a1a1a });
    for i in 0..3 {
        let groove = mesh(torus_geo!(platter_r * (0.45 + i as f64 * 0.14), 0.0004, 3, 32), &groove_mat);
        groove.rotation().x = PI / 2.0;
        groove.position().set(-0.05, base_h + 0.013, 0);
        tt.add(&groove);
    }
    // Spindle post (tiny cylinder in vinyl center).
    let spindle = mesh(pg("Turntables__spindle", cyl_geo!(0.002, 0.002, 0.010, 4)), &chrome_mat);
    spindle.position().set(-0.05, base_h + 0.016, 0);
    tt.add(&spindle);
    // Tonearm — pivot base + long arm w/ cartridge head.
    let tonearm_base = mesh(pg("Turntables__armBase", cyl_geo!(0.014, 0.014, 0.020, 8)), &chrome_mat);
    tonearm_base.position().set(0.12, base_h + 0.010, -base_d * 0.30);
    tt.add(&tonearm_base);
    // Arm shaft — slim horizontal cylinder pivoting from base.
    let arm_len = 0.16;
    let arm_group = group();
    let arm = mesh(pg("Turntables__arm", cyl_geo!(0.003, 0.003, arm_len, 6)), &chrome_mat);
    arm.rotation().z = PI / 2.0;
    arm.position().set(-arm_len / 2.0, 0, 0);
    arm_group.add(&arm);
    // Cartridge head at tip.
    let cartridge = mesh(pg("Turntables__cartridge", box_geo!(0.016, 0.006, 0.010)), &chrome_mat);
    cartridge.position().set(-arm_len, -0.003, 0);
    arm_group.add(&cartridge);
    // Needle (tiny cylinder pointing down).
    let needle = mesh(cyl_geo!(0.0005, 0.0005, 0.004, 3), &chrome_mat);
    needle.position().set(-arm_len, -0.008, 0);
    arm_group.add(&needle);
    // Position arm group at pivot base.
    arm_group.position().set(0.12, base_h + 0.020, -base_d * 0.30);
    // Rotate arm to point across platter (over vinyl).
    arm_group.rotation().y = 0.2;
    tt.add(&arm_group);
    // Small counterweight at arm-pivot end (rides the arm's jitter).
    let counterweight = mesh(pg("Turntables__weight", cyl_geo!(0.010, 0.010, 0.016, 8)), &chrome_mat);
    counterweight.rotation().z = PI / 2.0;
    counterweight.position().set(0.013, 0, 0);
    arm_group.add(&counterweight);
    // Speed selector + start/stop knobs on cabinet front.
    for i in 0..3 {
        let knob = mesh(pg("Turntables__knob", cyl_geo!(0.008, 0.008, 0.006, 8)), &chrome_mat);
        knob.rotation().x = PI / 2.0;
        knob.position().set(base_w * 0.30 - i as f64 * 0.030, base_h * 0.5, base_d * 0.5 + 0.003);
        tt.add(&knob);
    }
    // Speed selector labels (tiny dark rectangles above knobs).
    let label_dark_mat = lambert!({ color: 0x1a1a1a });
    for i in 0..3 {
        let speed_label = mesh(box_geo!(0.014, 0.003, 0.001), &label_dark_mat);
        speed_label.position().set(base_w * 0.30 - i as f64 * 0.030, base_h * 0.5 + 0.015, base_d * 0.5 + 0.003);
        tt.add(&speed_label);
    }
    // 40% chance: platter spinning animator ("record playing").
    let tt_is_playing = rnd() < 0.4;
    if tt_is_playing {
        // Rotate vinyl + label + grooves together — collect refs.
        let mut spin_targets: Vec<Obj> = vec![vinyl.clone(), center_label.clone()];
        for child in tt.children() {
            let is_torus = child.try_geo().map_or(false, |gg| gg.params().type_name() == "TorusGeometry");
            if is_torus && (child.pos().x + 0.05).abs() < 0.001 { spin_targets.push(child); }
        }
        // R732: capture arm base Y-rotation for jitter animator.
        let arm_base_y_rot = arm_group.rot().y;
        let jitter_phase = rnd() * PI * 2.0;
        let jitter_phase2 = rnd() * PI * 2.0;
        let (vinyl2, label2, arm2) = (vinyl.clone(), center_label.clone(), arm_group.clone());
        g.animate(move |_g, dt, t| {
            let rot = dt * 3.5; // ~33 RPM = ~3.5 rad/s
            // Grooves are tori turned flat (their z is the vertical axis); the
            // vinyl and label are Y-axis cylinders, so they spin about y.
            for target in &spin_targets {
                if target.ptr_eq(&vinyl2) || target.ptr_eq(&label2) { target.rotation().y -= rot; } else { target.rotation().z += rot; }
            }
            // R732 tonearm subtle jitter.
            arm2.rotation().y = arm_base_y_rot + (t * 0.7 + jitter_phase).sin() * 0.012 + (t * 2.3 + jitter_phase2).sin() * 0.004;
        });
    }
    tt.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    tt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tt);
    // R729 register only spinning turntables for vinyl-hiss audio.
    if tt_is_playing {
        let p = tt.pos();
        g.pp.turntable_positions.push(v3(p.x, 0, p.z));
    }
}

// ------------------------------------------------------------------ line 26034
fn place_framed_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x202020, 0xc0a060];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    // Reuse R692 NEWSPAPER_TEXTURE for the front page.
    let news_mat = lambert!({ map: newspaper_texture() });
    let fnw = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.28);
    let frame = mesh(pg("FramedNewspapers__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    fnw.add(&frame);
    // Gold-mat inner border (2-layer framing).
    let mat_border_mat = lambert!({ color: 0xd0c090 });
    let mat_border = mesh(pg("FramedNewspapers__mat", box_geo!(frame_w - 0.010, frame_h - 0.010, 0.002)), &mat_border_mat);
    mat_border.position().set(0, mount_y, 0.018);
    fnw.add(&mat_border);
    let news = mesh(plane_geo!(frame_w - 0.030, frame_h - 0.030), &news_mat);
    news.position().set(0, mount_y, 0.020);
    fnw.add(&news);
    // Small brass plaque below frame.
    let plaque_mat = lambert!({ color: 0xc0a060 });
    let plaque = mesh(pg("FramedNewspapers__plaque", box_geo!(frame_w * 0.6, 0.014, 0.004)), &plaque_mat);
    plaque.position().set(0, mount_y - frame_h * 0.5 - 0.015, 0.014);
    fnw.add(&plaque);
    // Engraved text line on plaque (thin dark rectangle).
    let text_line_mat = lambert!({ color: 0x2a1810 });
    let text_line = mesh(box_geo!(frame_w * 0.45, 0.001, 0.001), &text_line_mat);
    text_line.position().set(0, mount_y - frame_h * 0.5 - 0.015, 0.017);
    fnw.add(&text_line);
    fnw.rotation().y = face_wall(dx, dz);
    fnw.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    fnw.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&fnw);
}

// ------------------------------------------------------------------ line 26087
fn place_floor_globes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x8b5a2b, 0x2a1810, 0x8b6030, 0x505030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let fg = group();
    // Base — 3-leg tripod or 4-leg square base. Random 50/50.
    let is_tripod = rnd() < 0.5;
    if is_tripod {
        for i in 0..3 {
            let ang = (i as f64 / 3.0) * PI * 2.0;
            let leg = mesh(pg("FloorGlobes__leg", cyl_geo!(0.006, 0.008, 0.34, 5)), &wood_mat);
            // (tops lean in to meet under the globe: -x tilt for +z legs)
            leg.rotation().z = ang.cos() * 0.35;
            leg.rotation().x = -ang.sin() * 0.35;
            leg.position().set(ang.cos() * 0.06, 0.17, ang.sin() * 0.06);
            fg.add(&leg);
        }
        let base_connector = mesh(pg("FloorGlobes__tripodBase", cyl_geo!(0.13, 0.15, 0.010, 3)), &wood_mat);
        base_connector.position().y = 0.005;
        fg.add(&base_connector);
    } else {
        let base_pad = mesh(pg("FloorGlobes__squareBase", cyl_geo!(0.14, 0.15, 0.014, 4)), &wood_mat);
        base_pad.rotation().y = PI / 4.0;
        base_pad.position().y = 0.007;
        fg.add(&base_pad);
        let column = mesh(pg("FloorGlobes__column", cyl_geo!(0.012, 0.014, 0.32, 6)), &wood_mat);
        column.position().y = 0.17;
        fg.add(&column);
    }
    // Yoke/support arch — half torus arcing around globe (brass).
    let globe_r = 0.14;
    // The meridian's lowest point (r 0.157) rests on the column / leg tops (0.33).
    let globe_center_y = 0.345 + globe_r;
    let yoke = mesh(pg("FloorGlobes__yoke", torus_geo!(globe_r + 0.012, 0.005, 4, 16, PI * 1.15)), &brass_mat);
    yoke.rotation().x = 0.0;
    yoke.rotation().z = PI / 2.0 + 0.4 - PI * 0.075;
    yoke.position().y = globe_center_y;
    fg.add(&yoke);
    // Meridian degree markers on yoke — tiny dark boxes at 4 positions.
    for i in 0..4 {
        let ang = yoke.rot().z + (i as f64 + 0.5) * PI * 1.15 / 4.0; // along the meridian
        let mark = mesh(box_geo!(0.003, 0.002, 0.002), lambert!({ color: 0x1a1a1a }));
        mark.rotation().z = ang; // radial tick on the band's front face
        mark.position().set((globe_r + 0.012) * ang.cos(), globe_center_y + ang.sin() * (globe_r + 0.012), 0.0055);
        fg.add(&mark);
    }
    // Axis pole through globe (thin brass rod tilted).
    let axis_rod = mesh(pg("FloorGlobes__axis", cyl_geo!(0.002, 0.002, globe_r * 2.4, 4)), &brass_mat);
    axis_rod.rotation().z = 0.4;
    axis_rod.position().y = globe_center_y;
    fg.add(&axis_rod);
    // Globe sphere — earth tones (ocean blue + land patches painted-on).
    let globe_group = group();
    let ocean_mat = lambert!({ color: 0x2060a0 });
    let globe = mesh(pg("FloorGlobes__globe", sphere_geo!(globe_r, 20, 14)), &ocean_mat);
    globe_group.add(&globe);
    // 8-12 land patches — small green/brown ellipses stuck to sphere.
    let land_colors = [0x408030, 0x6a5030, 0x805030, 0xa0a040];
    let land_count = 8 + (rnd() * 5.0).floor() as i32;
    for i in 0..land_count {
        let land_mat = lambert!({ color: choice(&land_colors) });
        // Random point on sphere.
        let u = rnd() * PI * 2.0;
        let v = rnd() * (PI * 0.75) + PI * 0.125;
        let rr = globe_r * 1.005;
        let lx = u.cos() * v.sin() * rr;
        let ly = v.cos() * rr;
        let lz = u.sin() * v.sin() * rr;
        let land_patch = mesh(circle_geo!(0.014 + rnd() * 0.012, 6), &land_mat);
        land_patch.position().set(lx, ly, lz);
        land_patch.look_at(0, 0, 0);
        land_patch.rotation().y += PI; // Flip to face outward.
        land_patch.rotation().z = rnd() * PI;
        land_patch.scale().set(1.3 + rnd() * 0.5, 1.0, 1.0);
        globe_group.add(&land_patch);
    }
    // Position globe centered on axis rod, tilted.
    globe_group.rotation().z = 0.4;
    globe_group.position().y = globe_center_y;
    fg.add(&globe_group);
    // 40% chance: globe slowly spinning animator.
    if rnd() < 0.4 {
        let gg = globe_group.clone();
        g.animate(move |_g, dt, _t| {
            gg.rotation().y += dt * 0.35; // slow spin ~5 rpm
        });
    }
    fg.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    fg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fg);
}

// ------------------------------------------------------------------ line 26210
fn place_vintage_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_colors = [0x1a1a1a, 0x2a2a2a, 0x603030, 0x101020, 0x604040];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let key_mat = lambert!({ color: 0xf0f0e8 });
    let metal_mat = lambert!({ color: 0xc0c0c8 });
    let ink_mat = lambert!({ color: 0x1a1a1a });
    let tw = group();
    // Main body.
    let (body_w, body_d, body_h) = (0.30, 0.22, 0.10);
    let body = mesh(pg("VintageTypewriters__body", box_geo!(body_w, body_h, body_d)), &body_mat);
    body.position().y = body_h / 2.0 + 0.002;
    tw.add(&body);
    // Front slope (angled key deck).
    let deck = mesh(pg("VintageTypewriters__deck", box_geo!(body_w * 0.9, 0.024, body_d * 0.5)), &body_mat);
    deck.position().set(0, body_h * 0.4, body_d * 0.30);
    deck.rotation().x = 0.35;
    tw.add(&deck);
    // 4 rows × 10 keys (round white keys).
    for row in 0..4 {
        for col in 0..10 {
            let key_x = -body_w * 0.4 + col as f64 * (body_w * 0.08);
            let key_z = body_d * 0.20 + row as f64 * 0.020;
            let key_y = body_h * 0.6 - row as f64 * 0.010;
            let key = mesh(pg("VintageTypewriters__key", cyl_geo!(0.007, 0.007, 0.005, 8)), &key_mat);
            key.position().set(key_x, key_y, key_z);
            key.rotation().x = 0.35;
            tw.add(&key);
            // Tiny dark letter dot.
            let letter = mesh(circle_geo!(0.002, 5), &ink_mat);
            letter.rotation().x = -PI / 2.0 + 0.35;
            letter.position().set(key_x, key_y + 0.003, key_z);
            tw.add(&letter);
        }
    }
    // Space bar.
    let space_bar = mesh(pg("VintageTypewriters__spacebar", box_geo!(body_w * 0.6, 0.005, 0.018)), &key_mat);
    space_bar.rotation().x = 0.35;
    space_bar.position().set(0, 0.021, 0.118); // in front of (and below) key row 3
    tw.add(&space_bar);
    // Platen (roller) at back.
    let platen = mesh(pg("VintageTypewriters__platen", cyl_geo!(0.025, 0.025, body_w * 0.7, 12)), &metal_mat);
    platen.rotation().z = PI / 2.0;
    platen.position().set(0, body_h + 0.020, -body_d * 0.15);
    tw.add(&platen);
    // 2 platen end knobs.
    for side in [-1.0, 1.0] {
        let knob = mesh(pg("VintageTypewriters__knob", cyl_geo!(0.014, 0.014, 0.012, 8)), &metal_mat);
        knob.rotation().z = PI / 2.0;
        knob.position().set(side * body_w * 0.37, body_h + 0.020, -body_d * 0.15);
        tw.add(&knob);
    }
    // Paper sheet.
    let (paper_w, paper_h) = (body_w * 0.75, 0.18);
    let paper = mesh(plane_geo!(paper_w, paper_h), &paper_mat);
    paper.position().set(0, body_h + 0.020 + paper_h / 2.0, -body_d * 0.14);
    paper.rotation().x = -0.08; // Slight back-lean.
    tw.add(&paper);
    // Paper back face.
    let paper_back = mesh(plane_geo!(paper_w, paper_h), &paper_mat);
    paper_back.position().set(0, body_h + 0.020 + paper_h / 2.0, -body_d * 0.14 - 0.001);
    paper_back.rotation().x = -0.08;
    paper_back.rotation().y = PI;
    tw.add(&paper_back);
    // 5-7 rows of partial typed text.
    let line_count = 5 + (rnd() * 3.0).floor() as i32;
    for i in 0..line_count {
        let line_len = paper_w * (0.35 + rnd() * 0.55);
        let is_last = i == line_count - 1;
        // Last line is much shorter (interrupted mid-word).
        let actual_len = if is_last { paper_w * (0.15 + rnd() * 0.25) } else { line_len };
        let line = mesh(box_geo!(actual_len, 0.002, 0.001), &ink_mat);
        line.rotation().x = -0.08;
        let y_off = paper_h * 0.5 - 0.020 - i as f64 * 0.020;
        line.position().set(-paper_w / 2.0 + actual_len / 2.0 + 0.010,
            body_h + 0.020 + paper_h / 2.0 + y_off * (-0.08f64).cos(),
            -body_d * 0.14 - 0.002 + y_off * (-0.08f64).sin());
        tw.add(&line);
    }
    // Typebar hint.
    let typebar = mesh(pg("VintageTypewriters__typebar", box_geo!(0.003, 0.030, 0.003)), &metal_mat);
    typebar.rotation().x = 0.6;
    typebar.position().set(0, body_h + 0.010, -body_d * 0.05);
    tw.add(&typebar);
    // Carriage return lever on right.
    let lever = mesh(pg("VintageTypewriters__lever", cyl_geo!(0.002, 0.002, 0.030, 4)), &metal_mat);
    lever.rotation().z = 0.5;
    lever.position().set(body_w * 0.42, body_h + 0.030, -body_d * 0.15);
    tw.add(&lever);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
}

// ------------------------------------------------------------------ line 26433
fn place_anatomy_charts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x202020, 0x504030];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let chart_mat = lambert!({ map: anatomy_chart_texture() });
    let ac = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.32);
    let frame = mesh(pg("AnatomyCharts__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    ac.add(&frame);
    let chart = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &chart_mat);
    chart.position().set(0, mount_y, 0.019);
    ac.add(&chart);
    // Roller dowel top + bottom.
    let dowel_mat = lambert!({ color: 0x8b5a2b });
    for y_off in [frame_h * 0.5 + 0.005, -frame_h * 0.5 - 0.005] {
        let dowel = mesh(pg("AnatomyCharts__dowel", cyl_geo!(0.004, 0.004, frame_w + 0.010, 6)), &dowel_mat);
        dowel.rotation().z = PI / 2.0;
        dowel.position().set(0, mount_y + y_off, 0.019);
        ac.add(&dowel);
    }
    ac.rotation().y = face_wall(dx, dz);
    ac.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    ac.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&ac);
}

// ------------------------------------------------------------------ line 26566
fn place_easel_chalkboards(g: &mut Game, _dist: &[i32]) {
    g.pp.chalkboard_positions.clear(); // R732 reset per level
    if rnd() < 0.975 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let board_back_mat = lambert!({ color: 0x1a3020 });
    let chalk_face_mat = lambert!({ map: chalk_content_texture() });
    let ec = group();
    // Easel — 3 legs meeting at top (A-frame style w/ back support).
    let easel_h = 1.30;
    let leg_fl = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_fl.rotation().z = -0.12;
    leg_fl.rotation().x = -0.06;
    leg_fl.position().set(-0.12, easel_h / 2.0 - 0.005, 0.040);
    ec.add(&leg_fl);
    let leg_fr = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_fr.rotation().z = 0.12;
    leg_fr.rotation().x = -0.06;
    leg_fr.position().set(0.12, easel_h / 2.0 - 0.005, 0.040);
    ec.add(&leg_fr);
    let leg_back = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_back.rotation().x = 0.20;
    leg_back.position().set(0, easel_h / 2.0 - 0.012, -0.12); // top meets the front legs, foot on the floor
    ec.add(&leg_back);
    // Cross-brace.
    let brace = mesh(pg("EaselChalkboards__brace", cyl_geo!(0.005, 0.005, 0.30, 4)), &wood_mat);
    brace.rotation().z = PI / 2.0;
    brace.position().set(0, easel_h * 0.35, 0.04);
    ec.add(&brace);
    // Chalkboard — rectangular flat panel resting against front legs.
    let (board_w, board_h, board_t) = (0.44, 0.32, 0.014);
    let board_back = mesh(pg("EaselChalkboards__board", box_geo!(board_w, board_h, board_t)), &wood_mat);
    board_back.position().set(0, easel_h * 0.66, 0.05);
    board_back.rotation().x = -0.08;
    ec.add(&board_back);
    let board_face = mesh(plane_geo!(board_w - 0.020, board_h - 0.020), &chalk_face_mat);
    board_face.position().set(0, easel_h * 0.66, 0.05 + board_t / 2.0 + 0.001);
    board_face.rotation().x = -0.08;
    ec.add(&board_face);
    // Chalk tray at bottom of board.
    let chalk_tray = mesh(pg("EaselChalkboards__tray", box_geo!(board_w, 0.014, 0.030)), &wood_mat);
    chalk_tray.position().set(0, easel_h * 0.66 - board_h * 0.55, 0.05 + board_t / 2.0 + 0.008);
    chalk_tray.rotation().x = -0.08;
    ec.add(&chalk_tray);
    // 2-3 chalk sticks on tray + optional eraser block.
    let chalk_colors = [0xf0f0e8, 0xf0d0d0, 0xd0d0f0, 0xf0e0a0];
    let chalk_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..chalk_count {
        let chalk_mat = lambert!({ color: choice(&chalk_colors) });
        let chalk = mesh(pg("EaselChalkboards__chalk", cyl_geo!(0.005, 0.005, 0.055, 6)), &chalk_mat);
        chalk.rotation().z = PI / 2.0;
        let base_chalk_y = 0.05 + (rnd() - 0.5) * 0.1;
        chalk.rotation().y = base_chalk_y;
        chalk.position().set(
            -board_w * 0.3 + i as f64 * 0.06 + (rnd() - 0.5) * 0.010,
            easel_h * 0.66 - board_h * 0.55 + 0.014,
            0.05 + board_t / 2.0 + 0.010,
        );
        ec.add(&chalk);
        // R735 subtle chalk-stick roll animator — 30% chance per chalk.
        if rnd() < 0.3 {
            let roll_phase = rnd() * PI * 2.0;
            let c2 = chalk.clone();
            g.animate(move |_g, _dt, t| {
                c2.rotation().x = (t * 0.9 + roll_phase).sin() * 0.08;
            });
        }
    }
    // 60% chance: felt eraser block.
    if rnd() < 0.6 {
        let eraser_mat = lambert!({ color: 0x604020 });
        let eraser = mesh(pg("EaselChalkboards__eraser", box_geo!(0.05, 0.010, 0.020)), &eraser_mat);
        eraser.position().set(board_w * 0.30, easel_h * 0.66 - board_h * 0.55 + 0.014, 0.05 + board_t / 2.0 + 0.010);
        ec.add(&eraser);
        let felt_mat = lambert!({ color: 0x805030 });
        let felt = mesh(pg("EaselChalkboards__felt", box_geo!(0.048, 0.003, 0.018)), &felt_mat);
        felt.position().set(board_w * 0.30, easel_h * 0.66 - board_h * 0.55 + 0.020, 0.05 + board_t / 2.0 + 0.010);
        ec.add(&felt);
    }
    ec.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    ec.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ec);
    let p = ec.pos();
    g.pp.chalkboard_positions.push(v3(p.x, 0, p.z)); // R732
}

// ------------------------------------------------------------------ line 26674
fn place_school_desks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let metal_mat = lambert!({ color: 0x606060 });
    let sd = group();
    // Desk surface — flat wooden top.
    let (desk_w, desk_d, desk_t) = (0.38, 0.28, 0.020);
    let desk_y = 0.72;
    let desk_top = mesh(pg("SchoolDesks__top", box_geo!(desk_w, desk_t, desk_d)), &wood_mat);
    desk_top.position().set(0, desk_y, 0);
    sd.add(&desk_top);
    // Slight raised edge lip on front.
    let lip = mesh(pg("SchoolDesks__lip", box_geo!(desk_w, 0.010, 0.010)), &wood_mat);
    lip.position().set(0, desk_y + desk_t / 2.0 + 0.005, desk_d * 0.5 - 0.005);
    sd.add(&lip);
    // Metal side supports.
    let support = mesh(pg("SchoolDesks__upright", box_geo!(0.014, desk_y, 0.020)), &metal_mat);
    support.position().set(desk_w * 0.42, desk_y / 2.0, 0);
    sd.add(&support);
    // Metal foot at base.
    let foot_bar = mesh(pg("SchoolDesks__foot", box_geo!(0.014, 0.014, 0.16)), &metal_mat);
    foot_bar.position().set(desk_w * 0.42, 0.007, 0);
    sd.add(&foot_bar);
    // Chair seat — attached behind desk.
    let (chair_w, chair_d, chair_t) = (0.28, 0.24, 0.020);
    let chair_y = 0.44;
    let chair_seat = mesh(pg("SchoolDesks__seat", box_geo!(chair_w, chair_t, chair_d)), &wood_mat);
    chair_seat.position().set(0, chair_y, -desk_d * 0.5 - chair_d * 0.5 - 0.020);
    sd.add(&chair_seat);
    // Chair back.
    let chair_back = mesh(pg("SchoolDesks__back", box_geo!(chair_w, 0.28, 0.014)), &wood_mat);
    chair_back.position().set(0, chair_y + 0.14, -desk_d * 0.5 - chair_d - 0.024); // overlaps the seat's rear edge
    sd.add(&chair_back);
    // 4 metal legs on chair.
    for cx in [-chair_w * 0.42, chair_w * 0.42] {
        for cz in [-chair_d * 0.42, chair_d * 0.42] {
            let leg = mesh(pg("SchoolDesks__leg", cyl_geo!(0.006, 0.006, chair_y, 5)), &metal_mat);
            leg.position().set(cx, chair_y / 2.0, -desk_d * 0.5 - chair_d * 0.5 - 0.020 + cz);
            sd.add(&leg);
        }
    }
    // Wire book-storage compartment under desk.
    for i in 0..2 {
        let wire_bar = mesh(pg("SchoolDesks__wire", box_geo!(desk_w * 0.9, 0.005, 0.006)), &metal_mat);
        wire_bar.position().set(0, desk_y - 0.10 + i as f64 * 0.06, -desk_d * 0.15);
        sd.add(&wire_bar);
    }
    // Open textbook on desk surface.
    let book_colors = [0x603030, 0x2060a0, 0x408060, 0x804020, 0x604040];
    let book_mat = lambert!({ color: choice(&book_colors) });
    let page_mat = lambert!({ color: 0xf0e8d0 });
    let ink_mat = lambert!({ color: 0x1a1a1a });
    let (page_w, page_d) = (0.12, 0.16);
    let book_pages = mesh(pg("SchoolDesks__pages", box_geo!(page_w * 2.0, 0.006, page_d)), &page_mat);
    book_pages.position().set(0, desk_y + desk_t / 2.0 + 0.005, 0);
    book_pages.rotation().y = (rnd() - 0.5) * 0.3;
    sd.add(&book_pages);
    let br = book_pages.rot().y;
    let book_cover = mesh(pg("SchoolDesks__cover", box_geo!(page_w * 2.0 + 0.006, 0.003, page_d + 0.006)), &book_mat);
    book_cover.position().set(0, desk_y + desk_t / 2.0 + 0.002, 0);
    book_cover.rotation().y = br;
    sd.add(&book_cover);
    // Central spine hint.
    let spine = mesh(pg("SchoolDesks__spine", cyl_geo!(0.002, 0.002, page_d, 4)), &book_mat);
    spine.rotation().x = PI / 2.0;
    spine.rotation().y = br;
    spine.position().set(0, desk_y + desk_t / 2.0 + 0.009, 0);
    sd.add(&spine);
    // Text lines on both pages.
    for side in 0..2 {
        for line in 0..7 {
            let line_w = page_w * (0.5 + rnd() * 0.4);
            let text_line = mesh(box_geo!(line_w, 0.002, 0.001), &ink_mat);
            let base_x = if side == 0 { -page_w * 0.5 } else { page_w * 0.5 };
            // on the page surface (top 0.008), turned with the book
            let lx = base_x + (rnd() - 0.5) * 0.010;
            let lz = -page_d * 0.4 + line as f64 * (page_d * 0.11);
            text_line.position().set(lx * br.cos() + lz * br.sin(), desk_y + desk_t / 2.0 + 0.0089, -lx * br.sin() + lz * br.cos());
            text_line.rotation().y = br;
            sd.add(&text_line);
        }
    }
    // 70% chance: red apple on desk corner.
    if rnd() < 0.7 {
        let apple_colors = [0xa02020, 0xa04020, 0xc03020, 0x408030];
        let apple_mat = lambert!({ color: choice(&apple_colors) });
        let apple = mesh(pg("SchoolDesks__apple", sphere_geo!(0.028, 8, 6)), &apple_mat);
        apple.scale().y = 1.05;
        apple.position().set(-desk_w * 0.35, desk_y + desk_t / 2.0 + 0.028, -desk_d * 0.30);
        sd.add(&apple);
        let stem_mat = lambert!({ color: 0x604020 });
        let stem = mesh(pg("SchoolDesks__stem", cyl_geo!(0.001, 0.001, 0.008, 3)), &stem_mat);
        stem.position().set(-desk_w * 0.35, desk_y + desk_t / 2.0 + 0.062, -desk_d * 0.30);
        sd.add(&stem);
        let leaf_mat = lambert!({ color: 0x408040 });
        let leaf = mesh(pg("SchoolDesks__leaf", box_geo!(0.010, 0.001, 0.004)), &leaf_mat);
        leaf.position().set(-desk_w * 0.35 + 0.006, desk_y + desk_t / 2.0 + 0.062, -desk_d * 0.30);
        leaf.rotation().z = 0.3;
        sd.add(&leaf);
    }
    // 60% chance: yellow #2 pencil rolling on desk beside book.
    if rnd() < 0.6 {
        let pencil_mat = lambert!({ color: 0xf0c040 });
        let pencil = mesh(pg("SchoolDesks__pencil", cyl_geo!(0.003, 0.003, 0.10, 6)), &pencil_mat);
        pencil.rotation().z = PI / 2.0;
        pencil.rotation().y = (rnd() - 0.5) * 0.5;
        // lies on the open pages (top at +0.008), not buried in them
        let pencil_y = desk_y + desk_t / 2.0 + 0.011;
        let pry = pencil.rot().y;
        let (p_ax, p_az) = (pry.cos() * 0.054, -pry.sin() * 0.054); // half-length + cap, along the pencil
        pencil.position().set(page_w * 0.7, pencil_y, -page_d * 0.35);
        sd.add(&pencil);
        // Dark tip (apex pointing away from the pencil).
        let tip = mesh(pg("SchoolDesks__tip", cone_geo!(0.003, 0.008, 5)), lambert!({ color: 0x1a1a1a }));
        tip.rotation().z = PI / 2.0;
        tip.rotation().y = pry;
        tip.position().set(page_w * 0.7 - p_ax, pencil_y, -page_d * 0.35 - p_az);
        sd.add(&tip);
        // Pink eraser at back.
        let eraser_mat = lambert!({ color: 0xf0b0b0 });
        let eraser = mesh(pg("SchoolDesks__eraser", cyl_geo!(0.0035, 0.0035, 0.008, 6)), &eraser_mat);
        eraser.rotation().z = PI / 2.0;
        eraser.rotation().y = pry;
        eraser.position().set(page_w * 0.7 + p_ax, pencil_y, -page_d * 0.35 + p_az);
        sd.add(&eraser);
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}

// ------------------------------------------------------------------ line 26895
fn place_periodic_tables(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x404040, 0x2a2a2a];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let chart_mat = lambert!({ map: periodic_table_texture() });
    let pt = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.36, 0.26);
    let frame = mesh(pg("PeriodicTables__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    pt.add(&frame);
    let chart = mesh(plane_geo!(frame_w - 0.016, frame_h - 0.016), &chart_mat);
    chart.position().set(0, mount_y, 0.019);
    pt.add(&chart);
    pt.rotation().y = face_wall(dx, dz);
    pt.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    pt.rotation().z = (rnd() - 0.5) * 0.02;
    g.world.add(&pt);
}

// ------------------------------------------------------------------ line 26932
fn place_spilled_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pack_colors = [0xa02040, 0x2060a0, 0x408060, 0x101010, 0xd0a040, 0x604020, 0xa04080];
    let pack_mat = lambert!({ color: choice(&pack_colors) });
    let sp = group();
    // Main pack body — slumped rounded box.
    let (body_w, body_h, body_d) = (0.24, 0.18, 0.14);
    let body = mesh(pg("SpilledBackpacks__body", box_geo!(body_w, body_h, body_d)), &pack_mat);
    body.position().set(0, body_h / 2.0 + 0.002, 0);
    body.rotation().x = 0.15;
    sp.add(&body);
    // Main compartment zipper.
    let zipper_mat = lambert!({ color: 0x1a1a1a });
    let zipper = mesh(pg("SpilledBackpacks__zip", box_geo!(body_w * 0.85, 0.003, 0.006)), &zipper_mat);
    zipper.position().set(0, body_h + 0.003, -body_d * 0.05);
    zipper.rotation().x = 0.15;
    sp.add(&zipper);
    // Zipper pull tabs at ends.
    for side in [-1.0, 1.0] {
        let pull = mesh(pg("SpilledBackpacks__pull", box_geo!(0.006, 0.008, 0.003)), &zipper_mat);
        pull.position().set(side * body_w * 0.43, body_h + 0.002, 0);
        sp.add(&pull);
    }
    // Front pocket.
    let (pocket_w, pocket_h, pocket_d) = (body_w * 0.7, body_h * 0.5, 0.020);
    let pocket = mesh(pg("SpilledBackpacks__pocket", box_geo!(pocket_w, pocket_h, pocket_d)), &pack_mat);
    pocket.position().set(0, body_h * 0.35, body_d * 0.5 + pocket_d / 2.0);
    pocket.rotation().x = 0.15;
    sp.add(&pocket);
    // Front pocket zipper.
    let pocket_zip = mesh(pg("SpilledBackpacks__zip", box_geo!(pocket_w * 0.85, 0.003, 0.005)), &zipper_mat);
    pocket_zip.position().set(0, body_h * 0.35 + pocket_h / 2.0, body_d * 0.5 + pocket_d + 0.001);
    pocket_zip.rotation().x = 0.15;
    sp.add(&pocket_zip);
    // 2 shoulder straps on back.
    for side in [-1.0, 1.0] {
        let strap = mesh(pg("SpilledBackpacks__strap", box_geo!(0.030, body_h * 1.1, 0.010)), &pack_mat);
        // On the leaning back face (body-local z -0.074), spanning its height.
        strap.position().set(side * body_w * 0.30, body_h / 2.0 + 0.002 + 0.074 * (0.15f64).sin(), -0.074 * (0.15f64).cos());
        strap.scale().y = 0.9;
        strap.rotation().x = 0.15;
        sp.add(&strap);
    }
    // Small carry handle on top.
    let handle_mat = lambert!({ color: 0x0a0a0a });
    let handle = mesh(pg("SpilledBackpacks__handle", torus_geo!(0.024, 0.005, 3, 8, PI)), &handle_mat);
    handle.rotation().z = 0.0;
    handle.rotation().x = 0.15; // arch standing up from the top, leaning with the body
    handle.position().set(0, body_h + 0.004, -body_d * 0.10);
    sp.add(&handle);
    // 2-3 spilled books tumbling out from opening.
    let book_colors = [0x603030, 0x2060a0, 0x408060, 0x804020, 0x604040, 0x606060];
    let book_count = 2 + (rnd() * 2.0).floor() as i32;
    let page_color_mat = lambert!({ color: 0xf0e8d0 });
    for i in 0..book_count {
        let book_mat = lambert!({ color: choice(&book_colors) });
        let book = group();
        let bw = 0.11 + rnd() * 0.04;
        let bh = 0.014 + rnd() * 0.006;
        let bd = 0.08 + rnd() * 0.03;
        let book_body = mesh(pg("SpilledBackpacks__book", box_geo!(bw, bh, bd)), &book_mat);
        book_body.position().y = bh / 2.0;
        book.add(&book_body);
        // Page-edge trim.
        let page_edge = mesh(pg("SpilledBackpacks__pages", box_geo!(bw * 0.96, bh - 0.001, bd - 0.006)), &page_color_mat);
        page_edge.position().y = bh / 2.0;
        book.add(&page_edge);
        // Position scattered near backpack opening.
        let ang = -0.3 + rnd() * 1.2;
        let rr = 0.15 + rnd() * 0.10;
        book.position().set(ang.cos() * rr, 0.006 + i as f64 * 0.005, ang.sin() * rr * 0.7);
        book.rotation().y = rnd() * PI * 2.0;
        book.rotation().z = (rnd() - 0.5) * 0.3;
        sp.add(&book);
    }
    // 60% chance: pencil scattered nearby.
    if rnd() < 0.6 {
        let pencil_mat = lambert!({ color: 0xf0c040 });
        let pencil = mesh(pg("SpilledBackpacks__pencil", cyl_geo!(0.003, 0.003, 0.10, 6)), &pencil_mat);
        pencil.rotation().z = PI / 2.0;
        pencil.rotation().y = rnd() * PI * 2.0;
        pencil.position().set(0.20, 0.006, -0.05);
        sp.add(&pencil);
    }
    // 40% chance: crumpled paper.
    if rnd() < 0.4 {
        let paper_mat = lambert!({ color: 0xf0e8d0 });
        let paper_ball = mesh(pg("SpilledBackpacks__paper", sphere_geo!(0.020, 5, 4)), &paper_mat);
        paper_ball.position().set(0.24, 0.020, 0.08);
        paper_ball.scale().set(1.1, 0.85, 1.0);
        sp.add(&paper_ball);
    }
    sp.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    sp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sp);
}

// ------------------------------------------------------------------ line 27215
fn place_framed_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 { return; }
    let wt = wall_tiles(g);
    if wt.is_empty() { return; }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x2a2a2a, 0xd0a040];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let map_mat = lambert!({ map: framed_map_texture() });
    let fm = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.30, 0.22);
    let frame = mesh(pg("FramedMaps__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    fm.add(&frame);
    let map = mesh(plane_geo!(frame_w - 0.016, frame_h - 0.016), &map_mat);
    map.position().set(0, mount_y, 0.019);
    fm.add(&map);
    fm.rotation().y = face_wall(dx, dz);
    fm.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    fm.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&fm);
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
