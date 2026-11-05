//! Prop placers: record stacks and turntables, posters, framed newspapers and
//! maps, floor globes, typewriters, and classroom props (anatomy charts,
//! chalkboard easels, school desks, periodic tables, spilled backpacks).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_record_stacks", place_record_stacks));
    r.push(("place_propaganda_posters", place_propaganda_posters));
    r.push(("place_turntables", place_turntables));
    r.push(("place_framed_newspapers", place_framed_newspapers));
    r.push(("place_floor_globes", place_floor_globes));
    r.push(("place_vintage_typewriters", place_vintage_typewriters));
    r.push(("place_anatomy_charts", place_anatomy_charts));
    r.push(("place_easel_chalkboards", place_easel_chalkboards));
    r.push(("place_school_desks", place_school_desks));
    r.push(("place_periodic_tables", place_periodic_tables));
    r.push(("place_spilled_backpacks", place_spilled_backpacks));
    r.push(("place_framed_maps", place_framed_maps));
}

/// Every open interior tile, row-major.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}
/// Open tiles next to a wall, with the first wall direction found
/// (later directions are not checked).
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
/// `atan2(-dx, -dz)`
fn face_wall(dx: i32, dz: i32) -> f64 {
    (-(dx as f64)).atan2(-(dz as f64))
}

// the record stack: batch `t10`'s (`t10.rs`)
mod t10;
use t10::place_record_stacks;

fn place_propaganda_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
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

fn place_turntables(g: &mut Game, _dist: &[i32]) {
    g.pp.turntable_positions.clear(); // R729 reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
            let is_torus = child.try_geo().map_or(false, |gg| gg.params().type_name() == "torus");
            if is_torus && (child.pos().x + 0.05).abs() < 0.001 {
                spin_targets.push(child);
            }
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
                if target.ptr_eq(&vinyl2) || target.ptr_eq(&label2) {
                    target.rotation().y -= rot;
                } else {
                    target.rotation().z += rot;
                }
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

fn place_framed_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x202020, 0xc0a060];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    // Reuse the R692 `newspaper_texture` for the front page.
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

fn place_floor_globes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

fn place_vintage_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        line.position().set(
            -paper_w / 2.0 + actual_len / 2.0 + 0.010,
            body_h + 0.020 + paper_h / 2.0 + y_off * (-0.08f64).cos(),
            -body_d * 0.14 - 0.002 + y_off * (-0.08f64).sin(),
        );
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

fn place_anatomy_charts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
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

fn place_easel_chalkboards(g: &mut Game, _dist: &[i32]) {
    g.pp.chalkboard_positions.clear(); // R732 reset per level
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

fn place_school_desks(g: &mut Game, _dist: &[i32]) {
    // A combination chair desk (batch `r18`): a laminate top on particleboard
    // in its edge moulding, carved and written on, a corner knocked off; the
    // seat and back of moulded plywood worn to the wood; a chromed steel
    // frame on two runners gone to rust at its feet; the book rack under the
    // seat. On it a textbook left open, sometimes a pencil, sometimes an apple
    // dried to a husk (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let sd = group();
    let put = |grp: &Obj, name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            grp.add(&mesh(geo, mat));
        }
    };
    // the old book's draws: its cloth's colour, its turn on the desk, the 14
    // lines of print on its pages (what is on the pages)
    let book_colors = [0x603030, 0x2060a0, 0x408060, 0x804020, 0x604040];
    let book_kind = {
        let c = choice(&book_colors);
        book_colors.iter().position(|&k| k == c).unwrap_or(0)
    };
    let br = (rnd() - 0.5) * 0.3;
    let mut text_salt = 0x5c40_0d35u32;
    for _side in 0..2 {
        for _line in 0..7 {
            let lw = rnd();
            let lx = rnd();
            text_salt = text_salt.rotate_left(7) ^ ((lw * 65535.0) as u32) ^ ((lx * 65535.0) as u32).rotate_left(16);
        }
    }
    let salt = text_salt ^ (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b);
    put(&sd, "Real__schoolDeskTop", &lambert!({ map: &r18_desk::top_tex(salt) }));
    put(&sd, "Real__schoolDeskEdge", &phong!({ color: 0x3b2c22, specular: 0x1a1816, shininess: 20 }));
    put(&sd, "Real__schoolDeskCore", &lambert!({ color: 0x9a8462 }));
    let chrome = phong!({ map: &r18_desk::chrome_tex(salt), specular: 0x6c6c68, shininess: 50 });
    put(&sd, "Real__schoolDeskFrame", &chrome);
    put(&sd, "Real__schoolDeskRack", &chrome);
    put(&sd, "Real__schoolDeskScrews", &phong!({ color: 0x6a6964, specular: 0x6a6a66, shininess: 40 }));
    put(&sd, "Real__schoolDeskGlides", &phong!({ color: 0x1c1b1a, specular: 0x121212, shininess: 12 }));
    put(&sd, "Real__schoolDeskSeat", &phong!({ map: &r18_desk::ply_tex(false, salt), specular: 0x1e1a14, shininess: 18 }));
    put(&sd, "Real__schoolDeskBack", &phong!({ map: &r18_desk::ply_tex(true, salt), specular: 0x1e1a14, shininess: 18 }));
    // the book, open, turned as it was left (the pencil on its pages)
    let book = group();
    book.rotation().y = br;
    put(&book, "Real__schoolDeskBookCover", &lambert!({ color: r18_desk::CLOTH[book_kind] }));
    put(&book, "Real__schoolDeskBookPages", &lambert!({ map: &r18_desk::pages_tex(salt) }));
    sd.add(&book);
    // 70% chance: an apple left on the desk, dried to a brown husk
    if rnd() < 0.7 {
        let apple_colors = [0xa02020, 0xa04020, 0xc03020, 0x408030];
        let ak = {
            let c = choice(&apple_colors);
            apple_colors.iter().position(|&k| k == c).unwrap_or(0)
        };
        let apple = group();
        put(&apple, "Real__schoolDeskApple", &lambert!({ map: &r18_desk::apple_tex(ak, salt) }));
        put(&apple, "Real__schoolDeskAppleStem", &lambert!({ color: 0x3a2c1e }));
        apple.position().set(-0.235, r18_desk::TOP_Y, -0.150);
        apple.rotation().y = ak as f64 * 1.7 + 0.4;
        sd.add(&apple);
    }
    // 60% chance: a yellow HB pencil, sharpened short, on the pages
    if rnd() < 0.6 {
        let turn = (rnd() - 0.5) * 0.5;
        let pencil = group();
        put(&pencil, "Real__schoolDeskPencil", &phong!({ map: &r18_desk::pencil_tex(salt), specular: 0x2a2620, shininess: 30 }));
        put(&pencil, "Real__schoolDeskPencilWood", &lambert!({ color: 0xb3916a }));
        put(&pencil, "Real__schoolDeskPencilLead", &phong!({ color: 0x2b2b2d, specular: 0x5a5a5c, shininess: 50 }));
        put(&pencil, "Real__schoolDeskPencilFerrule", &phong!({ color: 0x8e8670, specular: 0x6a6658, shininess: 40 }));
        put(&pencil, "Real__schoolDeskPencilEraser", &lambert!({ color: 0x9c6c62 }));
        pencil.rotation().y = turn;
        pencil.position().set(0.084, r18_desk::TOP_Y + r18_desk::page_top(0.084), -0.06);
        book.add(&pencil);
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}

fn place_periodic_tables(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
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

fn place_spilled_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

fn place_framed_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
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
        game::debug::testkit::smoke_placers(&list);
    }
}

/// The school desk of `place_school_desks` (batch `r18`): where its parts
/// meet, and what is painted on them.
mod r18_desk {
    use super::*;

    /// Its own stream starts the same on every floor: a placed thing's salt
    /// moves where it starts, so the strokes differ from floor to floor
    fn skip(salt: u32) {
        for _ in 0..(salt % 61) {
            let _ = rnd();
        }
    }

    /// The desk top's surface
    pub(super) const TOP_Y: f64 = 0.745;
    /// The book's cloth: maroon, blue, green, brown, plum, faded (sRGB hex)
    pub(super) const CLOTH: [u32; 5] = [0x5a2e2a, 0x2c4560, 0x34503e, 0x6a4428, 0x50363a];
    const BK_W: f64 = 0.168;
    const BK_T: f64 = 0.011;

    /// The height of the open pages over the desk at x across the spread (as
    /// `r18.py` models them)
    pub(super) fn page_top(x: f64) -> f64 {
        let ax = x.abs();
        0.0035 + BK_T * (1.0 - (-ax / 0.022).exp()) - 0.0018 * (ax / BK_W).powi(3) + 0.0012
    }

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
    fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// The two nearest distances to a jittered lattice (cells of 1)
    fn worley(seed: u32, x: f64, y: f64) -> (f64, f64) {
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dj in -1..=1 {
            for di in -1..=1 {
                let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                let fx = ii as f64 + hash(seed, ii, jj);
                let fy = jj as f64 + hash(seed ^ 77, ii, jj);
                let d = (fx - x).hypot(fy - y);
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d1, d2)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let (iw, _) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    fn rgb(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }
    fn css(c: [f64; 3], a: f64) -> String {
        format!("rgba({},{},{},{:.3})", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a)
    }

    /// The top's laminate (laid from above: u across, v from the back edge to
    /// the front): a medium oak print worn pale where arms rested, carved
    /// with initials and a heart cut through to the paper under it, written
    /// on in ballpoint and pencil, ringed where cans stood, dusty
    pub(super) fn top_tex(salt: u32) -> Tex {
        canvas_tex_opt(512, 384, false, move |ctx, w, h| {
            skip(salt);
            let seed = salt ^ 0x70b5;
            // laid from above, the texture's top is at the desk's back (the
            // student's side) and its left at the student's right: turned
            // half round, so what is carved reads from the seat
            fill(ctx, w, h, |u, v| {
                let (u, v) = (1.0 - u, 1.0 - v);
                let (x, y) = (u * 0.61, v * 0.457);
                let wob = fbm(seed ^ 9, x * 6.0, y * 6.0, 2) * 0.02;
                let grain = fbm(seed, x * 4.0, (y + wob) * 160.0, 3);
                let figure = fbm(seed ^ 1, x * 1.5, (y + wob) * 25.0, 3);
                let mut c = mix([0.5, 0.36, 0.22], [0.62, 0.47, 0.31], figure);
                c = mix(c, [0.38, 0.26, 0.15], (grain - 0.5).max(0.0) * 1.6);
                let pore = (vn(seed ^ 2, x * 900.0, y * 300.0) - 0.82).max(0.0) * 4.0;
                c = mix(c, [0.3, 0.2, 0.12], pore);
                // worn where forearms lay: paler, matte
                let wear = (-(((x - 0.305) / 0.18).powi(2) + ((y - 0.08) / 0.09).powi(2))).exp() * 0.5;
                c = mix(c, [0.7, 0.6, 0.47], wear * (0.6 + 0.4 * fbm(seed ^ 3, x * 40.0, y * 40.0, 2)));
                // dust and grime
                c = mix(c, [0.48, 0.45, 0.4], 0.12 + 0.12 * fbm(seed ^ 4, x * 10.0, y * 10.0, 3));
                rgb(c)
            });
            ctx.translate(w, h);
            ctx.rotate(PI);
            // rings where cans and cups stood
            for _ in 0..3 {
                let (cx, cy, r) = (w * (0.15 + 0.7 * rnd()), h * (0.2 + 0.6 * rnd()), 26.0 + rnd() * 8.0);
                ctx.set_stroke_style(css([0.3, 0.22, 0.14], 0.22 + 0.15 * rnd()).as_str());
                ctx.set_line_width(2.0 + rnd() * 2.0);
                ctx.begin_path();
                ctx.arc(cx, cy, r, rnd() * 2.0, 4.0 + rnd() * 2.3);
                ctx.stroke();
            }
            // carved: cut through the print to the pale paper under it,
            // its edges dark with dirt
            let carve = |ctx: &mut Canvas, text: &str, x: f64, y: f64, size: f64, a: f64| {
                ctx.save();
                ctx.translate(x, y);
                ctx.rotate(a);
                ctx.set_font(format!("bold {size:.0}px Helvetica Neue").as_str());
                ctx.set_line_join("round");
                ctx.set_line_width(size * 0.12);
                ctx.set_stroke_style(css([0.2, 0.14, 0.09], 0.6).as_str());
                ctx.stroke_text(text, 0.8, 0.8);
                ctx.set_line_width(size * 0.06);
                ctx.set_stroke_style(css([0.68, 0.6, 0.48], 0.75).as_str());
                ctx.stroke_text(text, 0.0, 0.0);
                ctx.restore();
            };
            let pick = |k: usize, list: &[&'static str]| list[(hash(seed ^ 0x51, k as i32, 3) * list.len() as f64) as usize % list.len()];
            let initials = ["JR", "T.M.", "KL", "D+S", "R.W.", "AJ"];
            carve(ctx, pick(0, &initials), w * 0.74, h * 0.3, 30.0, -0.2);
            carve(ctx, pick(1, &initials), w * 0.12, h * 0.82, 24.0, 0.15);
            // a heart, gouged
            ctx.save();
            ctx.translate(w * 0.86, h * 0.72);
            ctx.rotate(0.3);
            ctx.set_line_width(4.0);
            ctx.set_stroke_style(css([0.22, 0.15, 0.1], 0.7).as_str());
            ctx.begin_path();
            ctx.move_to(0.0, 14.0);
            ctx.bezier_curve_to(-22.0, -2.0, -12.0, -20.0, 0.0, -8.0);
            ctx.bezier_curve_to(12.0, -20.0, 22.0, -2.0, 0.0, 14.0);
            ctx.stroke();
            ctx.set_line_width(2.0);
            ctx.set_stroke_style(css([0.78, 0.7, 0.58], 0.9).as_str());
            ctx.stroke();
            ctx.restore();
            // ballpoint: a doodled box, a name, a game of noughts and crosses
            ctx.set_stroke_style(css([0.16, 0.2, 0.42], 0.55).as_str());
            ctx.set_line_width(1.3);
            ctx.begin_path();
            let (gx, gy) = (w * 0.2, h * 0.28);
            for k in 1..3 {
                ctx.move_to(gx + k as f64 * 14.0, gy);
                ctx.line_to(gx + k as f64 * 14.0 + 2.0, gy + 42.0);
                ctx.move_to(gx - 2.0, gy + k as f64 * 14.0);
                ctx.line_to(gx + 42.0, gy + k as f64 * 14.0 - 2.0);
            }
            ctx.stroke();
            ctx.set_font("16px Marker Felt");
            ctx.set_fill_style(css([0.16, 0.2, 0.42], 0.5).as_str());
            ctx.fill_text(pick(2, &["Lisa", "Danny", "Mike W.", "Tina", "Rob"]), w * 0.45, h * 0.9);
            ctx.set_fill_style(css([0.3, 0.3, 0.3], 0.35).as_str());
            ctx.fill_text(pick(3, &["1987", "1991", "1994", "'89"]), w * 0.6, h * 0.62);
            // scratches all over
            for _ in 0..160 {
                let (x, y) = (rnd() * w, rnd() * h);
                let a = (rnd() - 0.5) * 0.6 + if rnd() < 0.3 { PI / 2.0 } else { 0.0 };
                let l = 5.0 + rnd() * 40.0;
                ctx.set_stroke_style(css([0.74, 0.66, 0.54], 0.12 + 0.2 * rnd()).as_str());
                ctx.set_line_width(0.6 + rnd() * 0.8);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                ctx.stroke();
            }
        })
    }

    /// The chrome on the steel (laid as a box): its shine gone to grey, rust
    /// coming through it in pits, thickest at the feet and the runners
    pub(super) fn chrome_tex(salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let seed = salt ^ 0xc407;
            fill(ctx, w, h, |u, v| {
                let mut c = mix([0.36, 0.36, 0.35], [0.44, 0.44, 0.42], fbm(seed, u * 20.0, v * 20.0, 3));
                // the plating dulled to a grey film, blotched
                c = mix(c, [0.3, 0.29, 0.27], (fbm(seed ^ 3, u * 9.0, v * 9.0, 3) - 0.4).max(0.0) * 1.2);
                let low = sstep(0.3, 0.0, v);
                let pit = (fbm(seed ^ 1, u * 90.0, v * 90.0, 3) - 0.6 + 0.3 * low).max(0.0) * 4.0;
                c = mix(c, [0.33, 0.2, 0.12], pit.min(1.0));
                c = mix(c, [0.24, 0.21, 0.17], low * 0.5);
                rgb(c)
            });
        })
    }

    /// Moulded plywood under its varnish (the seat's grain front to back,
    /// the back's across), worn through to the paler wood where it was sat
    /// on and leant on, its edges dark with hands
    pub(super) fn ply_tex(back: bool, salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            skip(salt);
            let seed = salt ^ if back { 0xbac4 } else { 0x5ea7 };
            fill(ctx, w, h, |u, v| {
                let (a, b) = if back { (v, u) } else { (u, v) };
                let grain = fbm(seed, a * 60.0, b * 3.0, 3);
                let mut c = mix([0.46, 0.3, 0.16], [0.56, 0.38, 0.21], fbm(seed ^ 1, a * 8.0, b * 2.0, 3));
                c = mix(c, [0.36, 0.22, 0.11], (grain - 0.5).max(0.0) * 1.5);
                let worn =
                    (-(((u - 0.5) / 0.28).powi(2) + ((v - 0.5) / 0.3).powi(2))).exp() * (0.5 + 0.5 * fbm(seed ^ 2, u * 30.0, v * 30.0, 2));
                c = mix(c, [0.66, 0.53, 0.38], worn * 0.55);
                let edge = sstep(0.4, 0.5, (u - 0.5).abs().max((v - 0.5).abs()));
                c = mix(c, [0.22, 0.16, 0.1], edge * 0.5);
                rgb(c)
            });
            for _ in 0..60 {
                let (x, y) = (rnd() * w, rnd() * h);
                let a = rnd() * PI;
                let l = 4.0 + rnd() * 20.0;
                ctx.set_stroke_style(css([0.7, 0.58, 0.42], 0.2 + 0.2 * rnd()).as_str());
                ctx.set_line_width(0.7);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                ctx.stroke();
            }
        })
    }

    /// The open spread (from above: u across both pages, v from the foot of
    /// the pages to the head): a geography textbook's chapter on the water
    /// cycle in two columns, its figure, running heads and page numbers; the
    /// paper gone yellow, foxed, a tide mark where damp got in at the foot
    pub(super) fn pages_tex(salt: u32) -> Tex {
        canvas_tex_opt(512, 368, false, move |ctx, w, h| {
            let seed = salt ^ 0x9a6e;
            // turned half round to read from the seat (as the desk top's)
            fill(ctx, w, h, |u, v| {
                let (x, y) = (1.0 - u, 1.0 - v);
                let mut c = mix([0.8, 0.76, 0.64], [0.86, 0.83, 0.72], fbm(seed, x * 8.0, y * 8.0, 3));
                // the gutter darker, the fore-edges browned
                c = mix(c, [0.62, 0.56, 0.44], sstep(0.04, 0.0, (x - 0.5).abs()) * 0.6);
                c = mix(c, [0.66, 0.58, 0.42], sstep(0.44, 0.5, (x - 0.5).abs()) * 0.5);
                // foxing
                let fox = (vn(seed ^ 1, x * 60.0, y * 45.0) - 0.84).max(0.0) * 5.0;
                c = mix(c, [0.62, 0.48, 0.32], fox * 0.6);
                // the tide mark at the foot
                let tide = 0.12 + 0.05 * fbm(seed ^ 2, x * 9.0, 0.5, 2);
                c = mix(c, [0.7, 0.62, 0.48], sstep(tide + 0.01, tide - 0.03, y) * 0.35);
                c = mix(c, [0.52, 0.44, 0.32], (-((y - tide) / 0.006).powi(2)).exp() * 0.45);
                rgb(c)
            });
            ctx.translate(w, h);
            ctx.rotate(PI);
            let ink = css([0.12, 0.11, 0.1], 0.86);
            let page = |ctx: &mut Canvas, x0: f64, right: bool, n: u32| {
                ctx.set_fill_style(ink.as_str());
                ctx.set_text_align("left");
                ctx.set_font("italic 8px serif");
                let head = if right { "THE WATER CYCLE" } else { "Unit 3  Weather and Climate" };
                ctx.fill_text(head, x0 + 14.0, 18.0);
                ctx.set_text_align(if right { "right" } else { "left" });
                ctx.fill_text(&format!("{n}"), if right { x0 + 242.0 } else { x0 + 14.0 }, 352.0);
                ctx.set_text_align("left");
                let body = [
                    "Water is always on the move. It falls as rain",
                    "and snow, runs off the land in rivers and soaks",
                    "into the ground. The sun warms the surface of",
                    "lakes and the sea, and some of the water turns",
                    "to vapour and rises into the air. This is called",
                    "evaporation. Plants give off water vapour from",
                    "their leaves too: this is transpiration.",
                    "As the vapour rises it cools, and condenses into",
                    "tiny drops of water that form clouds. When the",
                    "drops join together and grow too heavy to stay",
                    "up, they fall as precipitation: rain, sleet, hail",
                    "or snow. Most of it falls back into the sea.",
                    "The rest falls on the land, where it collects in",
                    "lakes, flows to the sea in rivers, or sinks into",
                    "the ground to become groundwater.",
                ];
                if right {
                    ctx.set_font("bold 13px Helvetica Neue");
                    ctx.fill_text("3.4  Where does the rain go?", x0 + 14.0, 44.0);
                    ctx.set_font("9px serif");
                    for (k, l) in body.iter().enumerate().take(9) {
                        ctx.fill_text(l, x0 + 14.0, 64.0 + k as f64 * 11.5);
                    }
                    // the figure: sea, land, a cloud, the sun, the arrows round
                    ctx.set_stroke_style(ink.as_str());
                    ctx.set_line_width(1.2);
                    ctx.stroke_rect(x0 + 16.0, 176.0, 226.0, 128.0);
                    ctx.begin_path();
                    ctx.move_to(x0 + 16.0, 280.0);
                    ctx.bezier_curve_to(x0 + 90.0, 276.0, x0 + 120.0, 250.0, x0 + 170.0, 238.0);
                    ctx.line_to(x0 + 242.0, 228.0);
                    ctx.stroke();
                    ctx.begin_path();
                    ctx.arc(x0 + 40.0, 200.0, 10.0, 0.0, 6.3);
                    ctx.stroke();
                    ctx.begin_path();
                    ctx.arc(x0 + 150.0, 200.0, 12.0, 3.1, 6.3);
                    ctx.arc(x0 + 168.0, 196.0, 14.0, 3.4, 6.3);
                    ctx.arc(x0 + 186.0, 202.0, 10.0, 3.6, 6.4);
                    ctx.stroke();
                    for k in 0..3 {
                        let xx = x0 + 70.0 + k as f64 * 18.0;
                        ctx.begin_path();
                        ctx.move_to(xx, 274.0);
                        ctx.line_to(xx + 14.0, 218.0);
                        ctx.stroke();
                        let xr = x0 + 180.0 + k as f64 * 8.0;
                        ctx.begin_path();
                        ctx.move_to(xr, 214.0);
                        ctx.line_to(xr - 4.0, 236.0);
                        ctx.stroke();
                    }
                    ctx.set_font("italic 8px serif");
                    ctx.fill_text("Fig. 3.7  The water cycle.", x0 + 16.0, 318.0);
                } else {
                    ctx.set_font("9px serif");
                    for (k, l) in body.iter().enumerate() {
                        ctx.fill_text(l, x0 + 14.0, 40.0 + k as f64 * 11.5);
                    }
                    ctx.set_font("bold 10px Helvetica Neue");
                    ctx.fill_text("Questions", x0 + 14.0, 232.0);
                    ctx.set_font("9px serif");
                    let q = [
                        "1  What is evaporation?",
                        "2  Name two kinds of precipitation.",
                        "3  Where does most rain fall?",
                        "4  Draw and label the water cycle.",
                    ];
                    for (k, l) in q.iter().enumerate() {
                        ctx.fill_text(l, x0 + 14.0, 250.0 + k as f64 * 13.0);
                    }
                }
            };
            let n = 40 + 2 * (hash(seed ^ 3, 1, 1) * 60.0) as u32;
            page(ctx, 0.0, false, n);
            page(ctx, 256.0, true, n + 1);
            // pencil underlinings and a note in the margin
            ctx.set_stroke_style(css([0.3, 0.3, 0.3], 0.5).as_str());
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(14.0, 98.0);
            ctx.line_to(190.0, 97.0);
            ctx.stroke();
        })
    }

    /// The pencil's paint: chrome yellow worn through to the wood at its
    /// edges and bitten near its end, its stamp gone grey
    pub(super) fn pencil_tex(salt: u32) -> Tex {
        canvas_tex_opt(64, 16, false, move |ctx, w, h| {
            let seed = salt ^ 0x9e11;
            fill(ctx, w, h, |u, v| {
                let mut c = [0.78, 0.6, 0.18];
                c = mix(c, [0.6, 0.48, 0.3], (fbm(seed, u * 30.0, v * 6.0, 2) - 0.55).max(0.0) * 2.0);
                c = mix(c, [0.62, 0.5, 0.36], sstep(0.1, 0.0, u) * (0.5 + 0.5 * vn(seed ^ 1, u * 80.0, v * 10.0)));
                rgb(c)
            });
        })
    }

    /// The apple dried: a dark brown husk, its old colour still in patches,
    /// a white bloom of mould dried in its folds
    pub(super) fn apple_tex(kind: usize, salt: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            let seed = salt ^ 0xa991 ^ kind as u32;
            let old = [[0.3, 0.12, 0.08], [0.32, 0.17, 0.08], [0.34, 0.12, 0.07], [0.24, 0.24, 0.1]][kind.min(3)];
            fill(ctx, w, h, |u, v| {
                let mut c = mix([0.2, 0.13, 0.08], [0.3, 0.2, 0.12], fbm(seed, u * 10.0, v * 10.0, 3));
                c = mix(c, old, (fbm(seed ^ 1, u * 4.0, v * 4.0, 2) - 0.5).max(0.0) * 1.6);
                c = mix(c, [0.62, 0.6, 0.55], (vn(seed ^ 2, u * 40.0, v * 40.0) - 0.82).max(0.0) * 3.0);
                rgb(c)
            });
        })
    }
}
