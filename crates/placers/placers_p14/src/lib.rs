//! Prop placers: astronomy (moon globes, constellation embroidery, telescopes,
//! mission patches, orreries), birdcages, aquariums, cuckoo clocks, fireplaces,
//! vintage ads and log baskets.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_moon_globes", place_moon_globes));
    r.push(("place_constellation_embroidery", place_constellation_embroidery));
    r.push(("place_tripod_telescopes", place_tripod_telescopes));
    r.push(("place_mission_patches", place_mission_patches));
    r.push(("place_orrery_models", place_orrery_models));
    r.push(("place_birdcages", place_birdcages));
    r.push(("place_aquariums", place_aquariums));
    r.push(("place_ornate_cuckoo_clocks", place_ornate_cuckoo_clocks));
    r.push(("place_fireplaces", place_fireplaces));
    r.push(("place_vintage_ads", place_vintage_ads));
    r.push(("place_log_baskets", place_log_baskets));
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

fn place_moon_globes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x8b5a2b, 0x2a1810, 0x505030, 0x101010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let mg = group();
    // Base cylinder.
    let base = mesh(pg("MoonGlobes__base", cyl_geo!(0.09, 0.11, 0.020, 12)), &wood_mat);
    base.position().y = 0.010;
    mg.add(&base);
    // Central column.
    let column = mesh(pg("MoonGlobes__column", cyl_geo!(0.010, 0.014, 0.32, 6)), &wood_mat);
    column.position().y = 0.18;
    mg.add(&column);
    // Yoke — full ring.
    let globe_r = 0.11;
    // The ring's lowest point (r 0.124) rests in the column's cup (0.34).
    let globe_center_y = 0.352 + globe_r;
    let yoke = mesh(pg("MoonGlobes__yoke", torus_geo!(globe_r + 0.010, 0.004, 4, 20)), &brass_mat);
    // Upright full meridian in the axis plane (XY).
    yoke.rotation().x = 0.0;
    yoke.rotation().z = 0.15; // Slight tilt.
    yoke.position().y = globe_center_y;
    mg.add(&yoke);
    // Axis rod.
    let axis_rod = mesh(pg("FloorGlobes__axis", cyl_geo!(0.002, 0.002, globe_r * 2.4, 4)), &brass_mat);
    axis_rod.rotation().z = 0.15;
    axis_rod.position().y = globe_center_y;
    mg.add(&axis_rod);
    // Moon sphere — light gray w/ dark maria patches + craters.
    let moon_group = group();
    let surface_mat = lambert!({ color: 0xa0a0a0 });
    let moon = mesh(pg("MoonGlobes__moon", sphere_geo!(globe_r, 20, 14)), &surface_mat);
    moon_group.add(&moon);
    // Dark "maria" patches — 4-6 larger dark ellipses.
    let maria_colors = [0x404050, 0x504040, 0x505050];
    let maria_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..maria_count {
        let maria_mat = lambert!({ color: choice(&maria_colors) });
        let u = rnd() * PI * 2.0;
        let v = rnd() * (PI * 0.75) + PI * 0.125;
        let rr = globe_r * 1.005;
        let mx = u.cos() * v.sin() * rr;
        let my = v.cos() * rr;
        let mz = u.sin() * v.sin() * rr;
        let maria = mesh(circle_geo!(0.015 + rnd() * 0.010, 8), &maria_mat);
        maria.position().set(mx, my, mz);
        maria.look_at(0, 0, 0);
        maria.rotation().y += PI;
        maria.rotation().z = rnd() * PI;
        maria.scale().set(1.2 + rnd() * 0.4, 1.0, 1.0);
        moon_group.add(&maria);
    }
    // Impact craters — 12-18 small dark circles.
    let crater_mat = lambert!({ color: 0x707078 });
    let crater_count = 12 + (rnd() * 7.0).floor() as i32;
    for i in 0..crater_count {
        let u = rnd() * PI * 2.0;
        let v = rnd() * PI + 0.05;
        let rr = globe_r * 1.003;
        let cx = u.cos() * v.sin() * rr;
        let cy = v.cos() * rr;
        let cz = u.sin() * v.sin() * rr;
        let crater = mesh(circle_geo!(0.004 + rnd() * 0.006, 6), &crater_mat);
        crater.position().set(cx, cy, cz);
        crater.look_at(0, 0, 0);
        crater.rotation().y += PI;
        moon_group.add(&crater);
    }
    moon_group.rotation().z = 0.15;
    moon_group.position().y = globe_center_y;
    mg.add(&moon_group);
    // 40% chance: slow rotation animator.
    if rnd() < 0.4 {
        let m2 = moon_group.clone();
        g.animate(move |_g, dt, _t| {
            m2.rotation().y += dt * 0.20; // Slower than earth ~3 rpm
        });
    }
    mg.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    mg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mg);
}

fn place_constellation_embroidery(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xc0a060, 0x2a1810];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let emb_mat = lambert!({ map: constellation_embroidery_texture() });
    let ce = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.26);
    let frame = mesh(pg("ConstellationEmbroidery__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    ce.add(&frame);
    let emb = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &emb_mat);
    emb.position().set(0, mount_y, 0.019);
    ce.add(&emb);
    ce.rotation().y = face_wall(dx, dz);
    ce.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    ce.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&ce);
}

fn place_tripod_telescopes(g: &mut Game, _dist: &[i32]) {
    g.pp.telescope_positions.clear(); // R738 reset per level
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_colors = [0x1a1a1a, 0x2a2a2a, 0x404050, 0x202030];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let wood_mat = lambert!({ color: 0x604020 });
    let chrome_mat = lambert!({ color: 0xc0c0c8 });
    let lens_mat = basic_mat!({ color: 0x4060a0, transparent: true, opacity: 0.5 });
    let tt = group();
    // Tripod — 3 splayed wooden legs meeting at top.
    let tripod_h = 0.85;
    for i in 0..3 {
        let ang = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("TripodTelescopes__leg", cyl_geo!(0.008, 0.010, tripod_h, 5)), &wood_mat);
        // Top leans in towards the axis; 0.245 rad from the foot at r 0.22 to the head's rim.
        leg.rotation().z = ang.cos() * 0.245;
        leg.rotation().x = -ang.sin() * 0.245;
        leg.scale().y = 1.02;
        leg.position().set(ang.cos() * 0.118 * (tripod_h / 2.0 / (tripod_h / 2.0)), tripod_h / 2.0, ang.sin() * 0.118);
        tt.add(&leg);
    }
    // Rubber foot caps at bottom of each leg.
    for i in 0..3 {
        let ang = (i as f64 / 3.0) * PI * 2.0;
        let foot = mesh(pg("TripodTelescopes__foot", cyl_geo!(0.012, 0.012, 0.008, 5)), lambert!({ color: 0x0a0a0a }));
        foot.position().set(ang.cos() * 0.22, 0.004, ang.sin() * 0.22);
        tt.add(&foot);
    }
    // Mount head — chunky block at top of tripod.
    let mount_head = mesh(pg("TripodTelescopes__head", box_geo!(0.060, 0.050, 0.060)), &body_mat);
    mount_head.position().y = tripod_h + 0.025;
    tt.add(&mount_head);
    // Right ascension knob on side of mount.
    let ra_knob = mesh(pg("TripodTelescopes__knob", cyl_geo!(0.014, 0.014, 0.010, 8)), &chrome_mat);
    ra_knob.rotation().z = PI / 2.0;
    ra_knob.position().set(0.035, tripod_h + 0.020, 0); // on the head's side face (x 0.030)
    tt.add(&ra_knob);
    // Declination knob on other axis.
    let dec_knob = mesh(pg("TripodTelescopes__knob", cyl_geo!(0.012, 0.012, 0.008, 8)), &chrome_mat);
    dec_knob.rotation().x = PI / 2.0;
    dec_knob.position().set(0, tripod_h + 0.020, 0.034); // on the head's front face (z 0.030)
    tt.add(&dec_knob);
    // Optical tube assembly.
    let tube_l = 0.70;
    let tube_r = 0.035;
    let tube_tilt = 0.25; // ~15° up: objective end (+X) raised
    let tube_group = group();
    let tube = mesh(pg("TripodTelescopes__tube", cyl_geo!(tube_r, tube_r, tube_l, 12)), &body_mat);
    tube.rotation().z = PI / 2.0;
    tube_group.add(&tube);
    // Objective lens at front end (chrome ring + tinted glass disc).
    let objective_ring = mesh(pg("TripodTelescopes__objRing", torus_geo!(tube_r, 0.005, 4, 12)), &chrome_mat);
    objective_ring.rotation().y = PI / 2.0;
    objective_ring.position().x = tube_l * 0.5;
    tube_group.add(&objective_ring);
    let objective_lens = mesh(circle_geo!(tube_r - 0.005, 12), &lens_mat);
    objective_lens.rotation().y = PI / 2.0;
    objective_lens.position().x = tube_l * 0.5 + 0.001;
    tube_group.add(&objective_lens);
    // Rear cap / eyepiece housing.
    let rear_cap = mesh(pg("TripodTelescopes__rearCap", cyl_geo!(tube_r + 0.005, tube_r + 0.005, 0.030, 12)), &body_mat);
    rear_cap.rotation().z = PI / 2.0;
    rear_cap.position().x = -tube_l * 0.5;
    tube_group.add(&rear_cap);
    // Eyepiece.
    let eyepiece = mesh(pg("TripodTelescopes__eyepiece", cyl_geo!(0.014, 0.010, 0.060, 8)), &chrome_mat);
    eyepiece.rotation().x = 0.4; // Angled up for viewing.
    eyepiece.position().set(-tube_l * 0.5, 0.030, 0.025);
    tube_group.add(&eyepiece);
    // Eyepiece lens: faces out along the eyepiece axis (0, cos .4, sin .4), on its top rim.
    let eyepiece_lens = mesh(circle_geo!(0.010, 8), &lens_mat);
    eyepiece_lens.rotation().x = 0.4 - PI / 2.0;
    eyepiece_lens.position().set(-tube_l * 0.5, 0.030 + 0.0305 * (0.4f64).cos(), 0.025 + 0.0305 * (0.4f64).sin());
    tube_group.add(&eyepiece_lens);
    // Finder scope — smaller parallel telescope on top.
    let (finder_l, finder_r) = (0.16, 0.010);
    let finder = mesh(pg("TripodTelescopes__finder", cyl_geo!(finder_r, finder_r, finder_l, 8)), &body_mat);
    finder.rotation().z = PI / 2.0;
    finder.position().set(tube_l * 0.05, tube_r + 0.020, 0);
    tube_group.add(&finder);
    // Finder support brackets.
    let mut side = -1;
    while side <= 1 {
        let bracket = mesh(pg("TripodTelescopes__bracket", cyl_geo!(0.002, 0.002, 0.020, 3)), &body_mat);
        bracket.position().set(tube_l * 0.05 + side as f64 * finder_l * 0.35, tube_r + 0.010, 0);
        tube_group.add(&bracket);
        side += 2;
    }
    // Focus knob on tube side.
    let focus_knob = mesh(pg("TripodTelescopes__knob", cyl_geo!(0.010, 0.010, 0.008, 8)), &chrome_mat);
    focus_knob.rotation().x = PI / 2.0; // axis out of the tube side (+Z), not along the tube
    focus_knob.position().set(-tube_l * 0.30, 0, tube_r + 0.004);
    tube_group.add(&focus_knob);
    // Tilt tube group upward for viewing.
    tube_group.rotation().z = tube_tilt;
    tube_group.position().y = tripod_h + 0.050 + tube_r - 0.004; // tube rests in the head's clamp
    tt.add(&tube_group);
    // R738 subtle pan animator — 40% chance ("tracking a star").
    let is_tracking = rnd() < 0.4;
    if is_tracking {
        let pan_phase = rnd() * PI * 2.0;
        let base_tube_z = tube_group.rot().z;
        let (mh, tg) = (mount_head.clone(), tube_group.clone());
        g.animate(move |_g, _dt, t| {
            // Pan the mount_head + tube_group subgroups (tt.rotation.y is the world orientation).
            mh.rotation().y = (t * 0.3 + pan_phase).sin() * 0.06;
            tg.rotation().y = (t * 0.3 + pan_phase).sin() * 0.06;
            // Slight elevation adjustment on tube itself.
            tg.rotation().z = base_tube_z + (t * 0.2 + pan_phase * 1.3).sin() * 0.020;
        });
    }
    tt.user().set("isTracking", is_tracking);
    tt.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    tt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tt);
    // R738 register only tracking telescopes for focus-knob click audio.
    if is_tracking {
        let p = tt.pos();
        g.pp.telescope_positions.push(v3(p.x, 0, p.z));
    }
}

fn place_mission_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xc0a060, 0x2a2a2a];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let patch_mat = lambert!({ map: mission_patch_texture() });
    let mp = group();
    let mount_y = 1.60;
    let frame_size = 0.22;
    let frame = mesh(pg("MissionPatches__frame", box_geo!(frame_size, frame_size, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    mp.add(&frame);
    let patch = mesh(plane_geo!(frame_size - 0.018, frame_size - 0.018), &patch_mat);
    patch.position().set(0, mount_y, 0.019);
    mp.add(&patch);
    mp.rotation().y = face_wall(dx, dz);
    mp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    mp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&mp);
}

fn place_orrery_models(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x604020 });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let sun_mat = basic_mat!({ color: 0xf0c040 });
    let orr = group();
    // Base — flat wooden disc.
    let base = mesh(pg("OrreryModels__base", cyl_geo!(0.10, 0.11, 0.014, 12)), &wood_mat);
    base.position().y = 0.007;
    orr.add(&base);
    // Central column w/ brass gear-hub feel.
    let column = mesh(pg("OrreryModels__column", cyl_geo!(0.010, 0.012, 0.20, 8)), &brass_mat);
    column.position().y = 0.115;
    orr.add(&column);
    // Hub disc at top of column.
    let hub = mesh(pg("OrreryModels__hub", cyl_geo!(0.024, 0.024, 0.010, 8)), &brass_mat);
    hub.position().y = 0.220;
    orr.add(&hub);
    // Central sun sphere (self-lit).
    let sun = mesh(sphere_geo!(0.030, 12, 10), &sun_mat);
    sun.position().y = 0.255; // resting on the hub gear (top 0.225)
    orr.add(&sun);
    // Small PointLight from sun.
    let sun_light = point_light!(0xf0c040, 0.35, 1.5, 2);
    sun_light.position().copy(&sun.pos());
    orr.add(&sun_light);
    // 5 planets — Mercury, Venus, Earth, Mars, Jupiter (simplified). (r, color, size)
    let planets: [(f64, u32, f64); 5] = [
        (0.045, 0x808080, 0.006), // Mercury
        (0.070, 0xd0a060, 0.010), // Venus
        (0.100, 0x2060a0, 0.011), // Earth
        (0.135, 0xa03020, 0.008), // Mars
        (0.180, 0xc09060, 0.020), // Jupiter
    ];
    let planet_group = group(); // Rotates around Y for animator.
    let mut planet_groups: Vec<(Obj, f64)> = vec![]; // Per-planet groups for independent rotation.
    for i in 0..planets.len() {
        let (pr, pcolor, psize) = planets[i];
        // Arm — thin brass wire extending from hub to planet.
        let arm = mesh(pg("OrreryModels__arm", cyl_geo!(0.001, 0.001, pr, 3)), &brass_mat);
        arm.rotation().z = PI / 2.0;
        arm.position().set(pr / 2.0, 0.220, 0); // arms radiate from the hub gear, not through the sun
                                                // Wrap arm+planet in group for individual planet spin.
        let pgrp = group();
        pgrp.add(&arm);
        let planet = mesh(pg("OrreryModels__planet", sphere_geo!(psize, 8, 6)), lambert!({ color: pcolor }));
        planet.position().set(pr, 0.220, 0);
        pgrp.add(&planet);
        // Optional ring for Jupiter (largest planet, i == 4).
        if i == 4 {
            let ring_mat = basic_mat!({ color: 0xc0a080, transparent: true, opacity: 0.5 });
            let ring = mesh(torus_geo!(psize * 1.6, psize * 0.15, 4, 12), &ring_mat);
            ring.rotation().x = PI / 2.0 - 0.15;
            ring.position().set(pr, 0.220, 0);
            pgrp.add(&ring);
        }
        // Initial angular offset — evenly distributed.
        pgrp.rotation().y = (i as f64 / planets.len() as f64) * PI * 2.0;
        planet_group.add(&pgrp);
        planet_groups.push((pgrp.clone(), 0.15 - i as f64 * 0.020)); // Inner planets faster.
    }
    orr.add(&planet_group);
    // 60% chance: slow revolution animator — each planet at own speed.
    if rnd() < 0.6 {
        g.animate(move |_g, dt, _t| {
            for (grp, speed) in &planet_groups {
                grp.rotation().y += dt * speed;
            }
        });
    }
    orr.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    orr.rotation().y = rnd() * PI * 2.0;
    g.world.add(&orr);
}

fn place_birdcages(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let iron_colors = [0x1a1a1a, 0x2a2a2a, 0xc0a060, 0x606060];
    let iron_mat = lambert!({ color: choice(&iron_colors) });
    let bc = group();
    // Floor stand — wide flat base + tall pole.
    let base = mesh(pg("Birdcages__base", cyl_geo!(0.10, 0.12, 0.014, 8)), &iron_mat);
    base.position().y = 0.007;
    bc.add(&base);
    let pole = mesh(pg("Birdcages__pole", cyl_geo!(0.008, 0.010, 1.10, 6)), &iron_mat);
    pole.position().y = 0.560;
    bc.add(&pole);
    // Cage bottom disc.
    let (cage_r, cage_h) = (0.11, 0.30);
    let cage_y = 1.112; // cage floor (±0.003) sits on the pole top (1.110)
    let cage_bottom = mesh(pg("Birdcages__floor", cyl_geo!(cage_r, cage_r, 0.006, 12)), &iron_mat);
    cage_bottom.position().y = cage_y;
    bc.add(&cage_bottom);
    // Cage top disc.
    let cage_top = mesh(pg("Birdcages__top", cyl_geo!(cage_r * 0.5, cage_r * 0.9, 0.006, 12)), &iron_mat);
    cage_top.position().y = cage_y + cage_h;
    bc.add(&cage_top);
    // Cage dome (small hemisphere on top).
    let cage_dome = mesh(pg("Birdcages__dome", sphere_geo!(cage_r * 0.5, 8, 5, 0, PI * 2.0, 0, PI / 2.0)), &iron_mat);
    cage_dome.position().y = cage_y + cage_h + 0.006;
    bc.add(&cage_dome);
    // Small top ring hook (already upright in the XY plane), its foot in the finial.
    let top_hook = mesh(pg("Birdcages__hook", torus_geo!(0.014, 0.002, 3, 8)), &iron_mat);
    top_hook.position().y = cage_y + cage_h + 0.072;
    bc.add(&top_hook);
    // 12 vertical bars around cage.
    for i in 0..12 {
        let ang = (i as f64 / 12.0) * PI * 2.0;
        let bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, cage_h, 3)), &iron_mat);
        // Slight taper — bars converge upward slightly.
        let bottom_x = ang.cos() * cage_r;
        let bottom_z = ang.sin() * cage_r;
        let top_x = ang.cos() * cage_r * 0.6;
        let top_z = ang.sin() * cage_r * 0.6;
        bar.position().set((bottom_x + top_x) / 2.0, cage_y + cage_h / 2.0, (bottom_z + top_z) / 2.0);
        // Compute bar tilt.
        let mut v = v3(top_x - bottom_x, cage_h, top_z - bottom_z);
        v.normalize();
        bar.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
        bc.add(&bar);
    }
    // 3 horizontal ring bands around cage.
    for y_frac in [0.2, 0.5, 0.8] {
        // Interpolate radius at height (converging upward).
        let r_at_y = cage_r - (cage_r - cage_r * 0.6) * y_frac;
        let ring = mesh(pg("Birdcages__ring", torus_geo!(r_at_y, 0.002, 3, 14)), &iron_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = cage_y + cage_h * y_frac;
        bc.add(&ring);
    }
    // Perch — horizontal wooden dowel across cage middle.
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let perch = mesh(pg("Birdcages__perch", cyl_geo!(0.005, 0.005, cage_r * 1.6, 5)), &wood_mat);
    perch.rotation().z = PI / 2.0;
    perch.position().y = cage_y + cage_h * 0.4;
    bc.add(&perch);
    // 70% chance: swing door open (an outward-tilted panel of bars).
    let door_open = rnd() < 0.7;
    if door_open {
        let door_panel = group();
        // 3 mini bars forming a door section.
        for j in 0..3 {
            let door_bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, cage_h * 0.5, 3)), &iron_mat);
            door_bar.position().set(-0.020 + j as f64 * 0.020, 0, 0);
            door_panel.add(&door_bar);
        }
        // Door frame.
        let door_frame = mesh(box_geo!(0.06, cage_h * 0.5, 0.002), &iron_mat);
        door_frame.position().set(0, 0, 0);
        door_frame.set_mat(&lambert!({ color: 0x1a1a1a, transparent: true, opacity: 0 }));
        door_panel.add(&door_frame);
        door_panel.position().set(cage_r - 0.008, cage_y + cage_h * 0.5, 0);
        door_panel.rotation().y = -0.7; // Swung open.
        bc.add(&door_panel);
    }
    // 60% chance: single feather on floor near base — implies departed bird.
    if rnd() < 0.6 {
        let feather_colors = [0xf0f0e8, 0xd0a060, 0x606060, 0xa04040, 0x2060a0];
        let feather_mat = lambert!({ color: choice(&feather_colors) });
        let feather = mesh(pg("Birdcages__feather", sphere_geo!(0.008, 5, 4)), &feather_mat);
        feather.scale().set(0.5, 0.15, 2.5);
        feather.position().set(0.14, 0.005, 0.08);
        feather.rotation().y = rnd() * PI * 2.0;
        bc.add(&feather);
    }
    // Small water dish + food dish at cage bottom.
    let dish_mat = lambert!({ color: 0xf0e0c0 });
    for (dx2, dz2, colored) in [(-0.05, 0.0, false), (0.05, 0.0, true)] {
        let dish = mesh(pg("Birdcages__dish", cyl_geo!(0.020, 0.018, 0.008, 8)), &dish_mat);
        dish.position().set(dx2, cage_y + 0.007, dz2);
        bc.add(&dish);
        if !colored {
            // Water in water dish (translucent blue).
            let water = mesh(circle_geo!(0.018, 8), basic_mat!({ color: 0x506080, transparent: true, opacity: 0.6 }));
            water.rotation().x = -PI / 2.0;
            water.position().set(dx2, cage_y + 0.011, dz2);
            bc.add(&water);
        } else {
            // Seed pile — small dark scatter.
            for i in 0..5 {
                let seed = mesh(pg("Birdcages__seed", sphere_geo!(0.002, 4, 3)), lambert!({ color: 0x604020 }));
                seed.position().set(dx2 + (rnd() - 0.5) * 0.020, cage_y + 0.012, dz2 + (rnd() - 0.5) * 0.020);
                bc.add(&seed);
            }
        }
    }
    bc.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    bc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bc);
}

fn place_aquariums(g: &mut Game, _dist: &[i32]) {
    g.pp.aquarium_positions.clear(); // R744 reset per level
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x604020 });
    let frame_mat = lambert!({ color: 0x101010 });
    let glass_mat = lambert!({ color: 0xd0e0f0, transparent: true, opacity: 0.20 });
    let water_mat = lambert!({ color: 0x2060a0, transparent: true, opacity: 0.55 });
    let aq = group();
    // Wooden stand.
    let (stand_w, stand_d, stand_h) = (0.55, 0.30, 0.60);
    let stand = mesh(pg("Aquariums__stand", box_geo!(stand_w, stand_h, stand_d)), &wood_mat);
    stand.position().y = stand_h / 2.0 + 0.002;
    aq.add(&stand);
    // 2 small drawer fronts on stand.
    for off in [-0.13, 0.13] {
        let drawer = mesh(pg("Aquariums__drawer", box_geo!(0.24, 0.16, 0.003)), &wood_mat);
        drawer.position().set(off, stand_h * 0.55, stand_d * 0.5 + 0.002);
        aq.add(&drawer);
        let knob = mesh(pg("Aquariums__knob", sphere_geo!(0.006, 5, 4)), &frame_mat);
        knob.position().set(off, stand_h * 0.55, stand_d * 0.5 + 0.006);
        aq.add(&knob);
    }
    // Tank on top.
    let (tank_w, tank_d, tank_h) = (0.50, 0.28, 0.36);
    // Front + back walls.
    for side in [-1.0, 1.0] {
        let wall = mesh(box_geo!(tank_w, tank_h, 0.006), &glass_mat);
        wall.position().set(0, stand_h + tank_h / 2.0 + 0.010, side * tank_d * 0.5);
        aq.add(&wall);
    }
    // Left + right walls.
    for side in [-1.0, 1.0] {
        let wall = mesh(box_geo!(0.006, tank_h, tank_d - 0.012), &glass_mat);
        wall.position().set(side * tank_w * 0.5, stand_h + tank_h / 2.0 + 0.010, 0);
        aq.add(&wall);
    }
    // Tank bottom + top frames.
    for y_off in [0.006, tank_h + 0.014] {
        let frame = mesh(pg("Aquariums__frame", box_geo!(tank_w + 0.006, 0.010, tank_d + 0.006)), &frame_mat);
        frame.position().set(0, stand_h + y_off, 0);
        aq.add(&frame);
    }
    // Water volume (fills tank interior, ~85% height).
    let water = mesh(box_geo!(tank_w - 0.014, tank_h * 0.85, tank_d - 0.014), &water_mat);
    water.position().set(0, stand_h + 0.014 + (tank_h * 0.85) / 2.0, 0);
    aq.add(&water);
    // Gravel bottom.
    let gravel_mat = lambert!({ color: 0x605040 });
    let gravel = mesh(pg("Aquariums__gravel", box_geo!(tank_w - 0.020, 0.028, tank_d - 0.020)), &gravel_mat);
    gravel.position().set(0, stand_h + 0.028, 0);
    aq.add(&gravel);
    // 2-3 aquatic plants (thin green vertical strips).
    let plant_colors = [0x408030, 0x60a040, 0x306020];
    let plant_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..plant_count {
        let plant_mat = lambert!({ color: choice(&plant_colors) });
        // 3 blades per plant cluster.
        for j in 0..3 {
            let blade = mesh(pg("Aquariums__blade", box_geo!(0.006, 0.14 + rnd() * 0.08, 0.002)), &plant_mat);
            let bh = blade.geo().params().height();
            blade.position().set(
                -tank_w * 0.4 + i as f64 * (tank_w * 0.8 / (plant_count as f64 - 0.5)) + (rnd() - 0.5) * 0.010,
                stand_h + 0.028 + bh / 2.0 + 0.014,
                (rnd() - 0.5) * (tank_d - 0.030),
            );
            blade.rotation().z = (rnd() - 0.5) * 0.3;
            aq.add(&blade);
        }
    }
    // 60% chance: 2-4 fish silhouettes.
    if rnd() < 0.6 {
        let fish_colors = [0xf0a020, 0xa02040, 0x2060a0, 0x60a0d0, 0xf0e040];
        let fish_count = 2 + (rnd() * 3.0).floor() as i32;
        struct Fish {
            body: Obj,
            tail: Obj,
            base_x: f64,
            base_z: f64,
            base_yaw: f64,
            phase: f64,
        }
        let mut fish_groups: Vec<Fish> = vec![];
        for i in 0..fish_count {
            let fish_mat = lambert!({ color: choice(&fish_colors) });
            // Simplified fish: 1 sphere w/ Y-scale (body) + tiny cone tail.
            let body = mesh(pg("Aquariums__fish", sphere_geo!(0.018, 6, 5)), &fish_mat);
            body.scale().set(1.4, 0.7, 0.5);
            body.position().set(
                (rnd() - 0.5) * (tank_w - 0.10),
                stand_h + 0.028 + 0.04 + rnd() * (tank_h * 0.7 - 0.04),
                (rnd() - 0.5) * (tank_d - 0.06),
            );
            body.rotation().y = rnd() * PI * 2.0;
            aq.add(&body);
            // Tail cone.
            let tail = mesh(pg("Aquariums__tail", cone_geo!(0.010, 0.014, 4)), &fish_mat);
            let byaw = body.rot().y;
            tail.rotation().z = -PI / 2.0;
            tail.rotation().y = byaw;
            tail.position().copy(&body.pos());
            // Behind the head: rotation.y sends local +X to (cos, 0, -sin).
            let dx = -byaw.cos() * 0.022;
            let dz = byaw.sin() * 0.022;
            tail.position().x += dx;
            tail.position().z += dz;
            aq.add(&tail);
            // R741 track for sway animator.
            let bp = body.pos();
            fish_groups.push(Fish {
                body: body.clone(),
                tail: tail.clone(),
                base_x: bp.x,
                base_z: bp.z,
                base_yaw: byaw,
                phase: rnd() * PI * 2.0,
            });
        }
        // R741 subtle fish sway animator — always fires when fish present.
        g.animate(move |_g, _dt, t| {
            for f in &fish_groups {
                // Slow X/Z drift in fish's facing direction.
                let fwd_x = f.base_yaw.cos();
                let fwd_z = -f.base_yaw.sin(); // local +X under rotation.y
                let drift = (t * 0.4 + f.phase).sin() * 0.020;
                f.body.position().x = f.base_x + fwd_x * drift;
                f.body.position().z = f.base_z + fwd_z * drift;
                f.tail.position().x = f.base_x + fwd_x * drift - fwd_x * 0.022;
                f.tail.position().z = f.base_z + fwd_z * drift - fwd_z * 0.022;
                // Tail wiggle — Y-rotation of tail relative to body.
                f.tail.rotation().y = f.base_yaw + (t * 3.5 + f.phase).sin() * 0.35;
            }
        });
    }
    // 4-6 bubble spheres rising from bottom.
    let bubble_mat = basic_mat!({ color: 0xf0f8ff, transparent: true, opacity: 0.55 });
    struct Bubble {
        mesh: Obj,
        base_y: f64,
        offset: f64,
        speed: f64,
        phase: f64,
    }
    let mut bubbles: Vec<Bubble> = vec![];
    let bubble_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..bubble_count {
        let bubble = mesh(sphere_geo!(0.005 + rnd() * 0.003, 5, 4), &bubble_mat);
        let bx = (rnd() - 0.5) * (tank_w - 0.06);
        let bz = (rnd() - 0.5) * (tank_d - 0.06);
        bubble.position().set(bx, stand_h + 0.028 + rnd() * (tank_h * 0.7), bz);
        aq.add(&bubble);
        let offset = rnd() * (tank_h * 0.7);
        let speed = 0.08 + rnd() * 0.05;
        let phase = rnd();
        bubbles.push(Bubble { mesh: bubble.clone(), base_y: stand_h + 0.028, offset, speed, phase });
    }
    // Bubble rise animator.
    g.animate(move |_g, dt, _t| {
        for b in bubbles.iter_mut() {
            b.offset = (b.offset + dt * b.speed) % (tank_h * 0.7);
            b.mesh.position().y = b.base_y + b.offset;
        }
    });
    // 60% chance: light on — PointLight in tank ceiling.
    if rnd() < 0.6 {
        let light = point_light!(0x60a0d0, 0.30, 1.8, 2);
        light.position().set(0, stand_h + tank_h, 0);
        aq.add(&light);
        // Also a small hood/light-canopy on top of tank.
        let canopy_mat = lambert!({ color: 0x1a1a1a });
        let canopy = mesh(pg("Aquariums__canopy", box_geo!(tank_w + 0.014, 0.020, tank_d + 0.014)), &canopy_mat);
        canopy.position().set(0, stand_h + tank_h + 0.030, 0);
        aq.add(&canopy);
    }
    aq.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    aq.rotation().y = rnd() * PI * 2.0;
    g.world.add(&aq);
    let p = aq.pos();
    g.pp.aquarium_positions.push(v3(p.x, 0, p.z)); // R744
}

fn place_ornate_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.cuckoo_clock_positions_r741.clear(); // R741 reset per level
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x604020 });
    let roof_mat = lambert!({ color: 0x2a1810 });
    let face_mat = lambert!({ color: 0xf0e8d0 });
    let hand_mat = lambert!({ color: 0x101010 });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let cc = group();
    let mount_y = 1.65;
    // Body — trapezoidal wooden chalet.
    let (body_w, body_h, body_d) = (0.22, 0.32, 0.10);
    let body = mesh(pg("OrnateCuckooClocks__body", box_geo!(body_w, body_h, body_d)), &wood_mat);
    body.position().set(0, mount_y, 0.05);
    cc.add(&body);
    // Sloped roof — 2 slanted planes forming pitched roof.
    for side in [-1.0, 1.0] {
        let roof = mesh(pg("OrnateCuckooClocks__roof", box_geo!(body_w * 0.9, 0.010, body_d * 1.2)), &roof_mat);
        // Outer end down; inner ends meet at the ridge (y top + 0.066), eaves on the case's top corners.
        roof.rotation().z = -side * 0.5;
        roof.position().set(side * body_w * 0.45 * (0.5f64).cos(), mount_y + body_h * 0.5 + 0.066 - body_w * 0.45 * (0.5f64).sin(), 0.05);
        cc.add(&roof);
    }
    // Roof ridge cap.
    let ridge = mesh(pg("OrnateCuckooClocks__ridge", box_geo!(0.010, 0.012, body_d * 1.2)), &roof_mat);
    ridge.position().set(0, mount_y + body_h * 0.5 + 0.070, 0.05); // caps the apex
    cc.add(&ridge);
    // 4 decorative scroll trim brackets under roof.
    for i in 0..4 {
        let ang = i as f64 * PI / 2.0;
        let trim = mesh(pg("OrnateCuckooClocks__trim", box_geo!(0.014, 0.010, 0.006)), &roof_mat);
        trim.position().set(-body_w * 0.4 + i as f64 * (body_w * 0.267), mount_y + body_h * 0.5 - 0.010, body_d * 0.5 + 0.053);
        cc.add(&trim);
    }
    // Face plate — circular disc on front of body.
    let face_r = 0.06;
    let face = mesh(circle_geo!(face_r, 16), &face_mat);
    // The case's front panel is at z 0.094 (the wreath/border stand proud to 0.100).
    face.position().set(0, mount_y + body_h * 0.15, body_d * 0.5 + 0.0445);
    cc.add(&face);
    // Gold face rim — the torus is already in the dial plane (XY).
    let face_rim = mesh(pg("OrnateCuckooClocks__rim", torus_geo!(face_r + 0.005, 0.004, 3, 16)), &brass_mat);
    face_rim.position().set(0, mount_y + body_h * 0.15, body_d * 0.5 + 0.048);
    cc.add(&face_rim);
    // 12 Roman-numeral tick marks.
    for i in 0..12 {
        let ang = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
        let is_major = i % 3 == 0;
        let tick = mesh(box_geo!(0.003, 0.001, if is_major { 0.008 } else { 0.005 }), &hand_mat);
        tick.position().set(ang.cos() * face_r * 0.85, mount_y + body_h * 0.15 + ang.sin() * face_r * 0.85, body_d * 0.5 + 0.0452);
        // Lie flat on the dial, long axis (Z) radial, thickness (Y) out of the dial.
        set_rotation_ordered(&tick, PI / 2.0, 0.0, ang + PI / 2.0, EulerOrder::ZYX);
        cc.add(&tick);
    }
    // Frozen hands.
    let hour_ang = rnd() * PI * 2.0 - PI / 2.0;
    let min_ang = rnd() * PI * 2.0 - PI / 2.0;
    let hour = mesh(pg("OrnateCuckooClocks__hourHand", box_geo!(0.005, 0.001, 0.028)), &hand_mat);
    hour.position().set(hour_ang.cos() * 0.014, mount_y + body_h * 0.15 + hour_ang.sin() * 0.014, body_d * 0.5 + 0.0458);
    set_rotation_ordered(&hour, PI / 2.0, 0.0, hour_ang + PI / 2.0, EulerOrder::ZYX); // flat on the dial, boss (-Z) at the centre
    cc.add(&hour);
    let minute = mesh(pg("OrnateCuckooClocks__minuteHand", box_geo!(0.003, 0.001, 0.042)), &hand_mat);
    minute.position().set(min_ang.cos() * 0.021, mount_y + body_h * 0.15 + min_ang.sin() * 0.021, body_d * 0.5 + 0.0468);
    set_rotation_ordered(&minute, PI / 2.0, 0.0, min_ang + PI / 2.0, EulerOrder::ZYX);
    cc.add(&minute);
    // Small cuckoo door above face.
    let door_panel = mesh(pg("OrnateCuckooClocks__door", box_geo!(0.032, 0.028, 0.003)), &roof_mat);
    door_panel.position().set(0, mount_y + body_h * 0.36, body_d * 0.5 + 0.046); // back on the case front (0.094)
    cc.add(&door_panel);
    // Door center line (horizontal groove).
    let door_line = mesh(box_geo!(0.032, 0.001, 0.001), &hand_mat);
    door_line.position().set(0, mount_y + body_h * 0.36, body_d * 0.5 + 0.048);
    cc.add(&door_line);
    // Pendulum below body — decorative pinecone-shape.
    let pendulum_group = group();
    let pendulum_rod = mesh(pg("OrnateCuckooClocks__rod", cyl_geo!(0.001, 0.001, 0.20, 3)), &brass_mat);
    pendulum_rod.position().y = -0.10;
    pendulum_group.add(&pendulum_rod);
    // Pinecone weight (small ellipsoid).
    let cone = mesh(pg("OrnateCuckooClocks__pinecone", cone_geo!(0.016, 0.030, 6)), &roof_mat);
    cone.rotation().x = PI;
    cone.position().y = -0.20;
    pendulum_group.add(&cone);
    pendulum_group.position().set(0, mount_y - body_h * 0.5, body_d * 0.5 + 0.02);
    cc.add(&pendulum_group);
    // 2 hanging weights on chains (pinecone shape).
    for side in [-1.0, 1.0] {
        let chain = mesh(pg("OrnateCuckooClocks__chain", cyl_geo!(0.001, 0.001, 0.18, 3)), &brass_mat);
        chain.position().set(side * body_w * 0.30, mount_y - body_h * 0.5 - 0.08, 0.055);
        cc.add(&chain);
        let weight = mesh(pg("OrnateCuckooClocks__pinecone", cone_geo!(0.012, 0.024, 5)), &roof_mat);
        weight.rotation().x = PI;
        weight.position().set(side * body_w * 0.30, mount_y - body_h * 0.5 - 0.18, 0.055);
        cc.add(&weight);
    }
    // 50% chance: pendulum swing animator (R741: tracked for cuckoo-call audio).
    let is_swinging = rnd() < 0.5;
    if is_swinging {
        let swing_phase = rnd() * PI * 2.0;
        let pg2 = pendulum_group.clone();
        g.animate(move |_g, _dt, t| {
            pg2.rotation().z = (t * PI * 1.4 + swing_phase).sin() * 0.20;
        });
    }
    cc.rotation().y = face_wall(dx, dz);
    cc.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&cc);
    if is_swinging {
        let p = cc.pos();
        g.pp.cuckoo_clock_positions_r741.push(v3(p.x, 0, p.z)); // R741
    }
}

/// `o.rotation.set(x, y, z, order)`
fn set_rotation_ordered(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    let mut r = o.rotation();
    r.set(x, y, z);
    r.order = order;
}

fn place_fireplaces(g: &mut Game, _dist: &[i32]) {
    g.pp.lit_fireplace_positions.clear(); // R744 reset per level
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let brick_mat = lambert!({ color: 0x804a30 });
    let stone_mat = lambert!({ color: 0x606060 });
    let dark_mat = lambert!({ color: 0x1a1a1a });
    let wood_mat = lambert!({ color: 0x604020 });
    let log_mat = lambert!({ color: 0x8b5a2b });
    let fp = group();
    // Fireplace body — tall wide box against wall.
    let (body_w, body_h, body_d) = (0.90, 1.10, 0.30);
    let body = mesh(pg("Fireplaces__body", box_geo!(body_w, body_h, body_d)), &brick_mat);
    body.position().set(0, body_h / 2.0 + 0.002, 0.15);
    fp.add(&body);
    // Opening — dark box cutout at center-lower (represents interior).
    let (open_w, open_h, open_d) = (0.50, 0.55, 0.20);
    let opening = mesh(pg("Fireplaces__opening", box_geo!(open_w, open_h, open_d)), &dark_mat);
    opening.position().set(0, open_h / 2.0 + 0.05, 0.15 + body_d / 2.0 - open_d / 2.0 + 0.001);
    fp.add(&opening);
    // Hearth stone slab in front — extends outward.
    let hearth = mesh(pg("Fireplaces__hearth", box_geo!(body_w + 0.06, 0.030, 0.14)), &stone_mat);
    hearth.position().set(0, 0.017, 0.15 + body_d / 2.0 + 0.06);
    fp.add(&hearth);
    // Mantel — horizontal ledge higher up.
    let mantel_h = open_h + 0.05 + 0.10;
    let mantel = mesh(pg("Fireplaces__mantel", box_geo!(body_w + 0.06, 0.026, 0.20)), &wood_mat);
    mantel.position().set(0, mantel_h, 0.15 + body_d / 2.0 - 0.02);
    fp.add(&mantel);
    // Andirons — 2 iron dog-legs at fireplace opening.
    for side in [-1.0, 1.0] {
        let post = mesh(pg("Fireplaces__post", cyl_geo!(0.010, 0.014, 0.18, 6)), &dark_mat);
        // Stands on the firebox floor (top y 0.070) at the mouth (z 0.30), under the log ends.
        post.position().set(side * open_w * 0.24, 0.070 + 0.09, 0.15 + body_d / 2.0 - 0.010);
        fp.add(&post);
        let orn = mesh(pg("Fireplaces__finial", sphere_geo!(0.014, 6, 5)), &dark_mat);
        orn.position().set(side * open_w * 0.24, 0.070 + 0.18 + 0.012, 0.15 + body_d / 2.0 - 0.010);
        fp.add(&orn);
        // Horizontal support bar going back from the post, lying on the firebox floor.
        let bar = mesh(pg("Fireplaces__bar", cyl_geo!(0.006, 0.006, open_d * 0.65, 4)), &dark_mat);
        bar.rotation().x = PI / 2.0;
        bar.position().set(side * open_w * 0.24, 0.070 + 0.007, 0.15 + body_d / 2.0 - 0.010 - open_d * 0.325);
        fp.add(&bar);
    }
    // 3-4 logs stacked between andirons, laid across the andiron bars (top y 0.083).
    let log_count = 3 + (rnd() * 2.0).floor() as i32;
    let log_slots: [(f64, f64); 4] = [(-0.024, 0.0), (0.024, 0.0), (0.0, 1.0), (-0.072, 0.0)];
    for i in 0..log_count as usize {
        let y_pos = 0.083 + 0.024 + log_slots[i].1 * 0.0416;
        let z_pos = 0.15 + body_d / 2.0 - 0.072 + log_slots[i].0;
        let log = mesh(pg("Fireplaces__log", cyl_geo!(0.024, 0.024, open_w * 0.55, 8)), &log_mat);
        log.rotation().z = PI / 2.0;
        log.position().set((rnd() - 0.5) * 0.02, y_pos, z_pos);
        fp.add(&log);
        // Bark end-cap circles (darker), on the log's sawn ends.
        let lx = log.pos().x;
        for end_side in [-1.0, 1.0] {
            let cap_mat = lambert!({ color: 0xa07040 });
            let cap = mesh(circle_geo!(0.024, 6), &cap_mat);
            cap.rotation().y = end_side * PI / 2.0;
            cap.position().set(lx + end_side * (open_w * 0.275 + 0.0005), y_pos, z_pos);
            fp.add(&cap);
        }
    }
    // Brick texture — 4 horizontal groove lines on body.
    for i in 1..5 {
        let groove = mesh(box_geo!(body_w + 0.005, 0.002, 0.002), &stone_mat);
        groove.position().set(0, mantel_h + 0.05 + i as f64 * 0.06, 0.15 + body_d / 2.0 + 0.001);
        fp.add(&groove);
    }
    // Chimney breast.
    let chimney_h = 1.0;
    let chimney = mesh(pg("Fireplaces__chimney", box_geo!(body_w * 0.6, chimney_h, body_d)), &brick_mat);
    chimney.position().set(0, body_h + chimney_h / 2.0, 0.15);
    fp.add(&chimney);
    // 55% chance: fire lit — glowing embers + flame + PointLight.
    let is_lit = rnd() < 0.55;
    if is_lit {
        let ember_mat = basic_mat!({ color: 0xff4020, transparent: true, opacity: 0.9 });
        let ember_disc = mesh(circle_geo!(open_w * 0.22, 10), &ember_mat);
        ember_disc.rotation().x = -PI / 2.0;
        // Ember bed on the firebox floor under the logs.
        ember_disc.position().set(0, 0.072, 0.15 + body_d / 2.0 - 0.110);
        fp.add(&ember_disc);
        // Multiple flame spheres — layered elongated w/ various colors.
        let flame_colors = [0xffe040, 0xffa020, 0xff6010];
        let flame_group = group();
        for i in 0..5 {
            let flame_mat = basic_mat!({ color: choice(&flame_colors), transparent: true, opacity: 0.75 });
            let flame = mesh(sphere_geo!(0.030, 6, 5), &flame_mat);
            flame.scale().y = 2.5;
            let flame_base_y = 0.05 + log_count as f64 * 0.03 + 0.020 + i as f64 * 0.020;
            flame.position().set(
                (rnd() - 0.5) * 0.06,
                flame_base_y,
                0.15 + body_d / 2.0 - open_d / 2.0 + 0.04 + open_d * 0.15 + (rnd() - 0.5) * 0.02,
            );
            flame_group.add(&flame);
        }
        fp.add(&flame_group);
        // PointLight — bright warm orange.
        let fire_light = point_light!(0xff8030, 0.85, 3.0, 2);
        fire_light.position().set(0, 0.05 + log_count as f64 * 0.03 + 0.10, 0.15 + body_d / 2.0 - open_d / 2.0 + 0.04 + open_d * 0.15);
        fp.add(&fire_light);
        // Flame flicker animator — multi-freq for organic feel.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_phase2 = rnd() * PI * 2.0;
        let base_intensity = 0.85;
        let (fl2, fg2) = (fire_light.clone(), flame_group.clone());
        g.animate(move |_g, _dt, t| {
            let f = 0.85 + (t * 7.0 + flicker_phase).sin() * 0.10 + (t * 13.0 + flicker_phase2).sin() * 0.05;
            fl2.set_intensity(base_intensity * f);
            // Individual flame Y-scale flicker.
            for (i, flame) in fg2.children().iter().enumerate() {
                let i = i as f64;
                let local_flick = 0.85 + (t * (8.0 + i) + flicker_phase * i).sin() * 0.15;
                flame.scale().set(local_flick * 0.9, 2.5 * local_flick, local_flick * 0.9);
            }
        });
    }
    // Back (local z 0) against the wall, front towards the room.
    fp.rotation().y = face_wall(dx, dz);
    fp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.005), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.005));
    g.world.add(&fp);
    // R744 register only lit fireplaces for crackle audio.
    if is_lit {
        let p = fp.pos();
        g.pp.lit_fireplace_positions.push(v3(p.x, 0, p.z));
    }
}

fn place_vintage_ads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x101010, 0xc0a060];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let ad_mat = lambert!({ map: vintage_ad_texture() });
    let va = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.20, 0.28);
    let frame = mesh(pg("VintageAds__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    va.add(&frame);
    let ad = mesh(plane_geo!(frame_w - 0.014, frame_h - 0.014), &ad_mat);
    ad.position().set(0, mount_y, 0.017);
    va.add(&ad);
    va.rotation().y = face_wall(dx, dz);
    va.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    va.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&va);
}

fn place_log_baskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Bimodal — metal wire basket or wicker basket.
    let is_wicker = rnd() < 0.5;
    let basket_color = if is_wicker {
        [0x8b5a2b, 0xa07040, 0x604020][(rnd() * 3.0).floor() as usize]
    } else {
        [0x1a1a1a, 0x606060, 0x2a2a2a][(rnd() * 3.0).floor() as usize]
    };
    let basket_mat = lambert!({ color: basket_color });
    let log_colors = [0x8b5a2b, 0xa07040, 0x805020, 0x604030];
    let bark_cap_mat = lambert!({ color: 0xd0a060 });
    let lb = group();
    let (b_r, b_h) = (0.14, 0.24);
    // Basket body — cylinder w/ open top.
    if is_wicker {
        let body = mesh(pg("LogBaskets__body", cyl_geo!(b_r, b_r * 0.9, b_h, 10, 1, true)), &basket_mat);
        body.position().y = b_h / 2.0 + 0.002;
        lb.add(&body);
        // Woven texture hints — 8 vertical thin lines.
        let slat_mat = lambert!({ color: 0x0a0a0a, transparent: true, opacity: 0.20 });
        for i in 0..8 {
            let ang = (i as f64 / 8.0) * PI * 2.0;
            let slat = mesh(box_geo!(0.006, b_h * 0.9, 0.001), &slat_mat);
            slat.position().set(ang.cos() * b_r * 0.98, b_h / 2.0 + 0.002, ang.sin() * b_r * 0.98);
            slat.rotation().y = ang + PI / 2.0;
            lb.add(&slat);
        }
    } else {
        // Metal wire basket — 12 vertical bars around perimeter.
        for i in 0..12 {
            let ang = (i as f64 / 12.0) * PI * 2.0;
            let bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, b_h, 4)), &basket_mat);
            bar.position().set(ang.cos() * b_r, b_h / 2.0 + 0.002, ang.sin() * b_r);
            lb.add(&bar);
        }
        // 3 horizontal ring bands.
        for y_frac in [0.15, 0.5, 0.85] {
            let ring = mesh(pg("LogBaskets__ring", torus_geo!(b_r, 0.003, 4, 14)), &basket_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().y = b_h * y_frac + 0.002;
            lb.add(&ring);
        }
    }
    // Bottom disc.
    let bottom = mesh(circle_geo!(b_r * 0.9, 10), &basket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    lb.add(&bottom);
    // 2 side handle loops (both variants): half rings bulging outwards on each side.
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("LogBaskets__handle", torus_geo!(0.026, 0.005, 4, 8, PI)), &basket_mat);
        handle.rotation().z = -side * PI / 2.0;
        handle.rotation().y = 0.0;
        handle.position().set(side * b_r, b_h + 0.002 - 0.032, 0);
        lb.add(&handle);
    }
    // Firewood logs stacked inside — 4-6 short logs.
    let log_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..log_count {
        let log_mat = lambert!({ color: choice(&log_colors) });
        let log_len = 0.22 + rnd() * 0.04;
        let log_r = 0.026 + rnd() * 0.010;
        let log = mesh(pg("Fireplaces__log", cyl_geo!(log_r, log_r, log_len, 8)), &log_mat);
        log.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        // Layers of two side by side, crossing the next layer at 60°; 72 % length to fit the basket.
        log.scale().y = 0.72;
        let half_l = log_len * 0.72 * 0.5;
        let layer = i / 2;
        let is_rotated = layer % 2 == 1;
        log.rotation().y = if is_rotated { PI / 3.0 } else { 0.0 };
        let across = (if i % 2 == 0 { -1.0 } else { 1.0 }) * 0.032; // offset perpendicular to the log
        let lr = log.rot();
        let layer_y = 0.002 + log_r + layer as f64 * 0.058 + half_l * lr.z.cos().abs();
        log.position().set(lr.y.sin() * across + (rnd() - 0.5) * 0.020, layer_y, lr.y.cos() * across + (rnd() - 0.5) * 0.030);
        lb.add(&log);
        // Bark end-cap circles, on the (tilted) log's two ends, facing out.
        let mut log_axis = v3(0, 1, 0);
        log_axis.apply_euler(&log.rot());
        let lp = log.pos();
        for end_side in [-1.0, 1.0] {
            let cap = mesh(circle_geo!(log_r, 6), &bark_cap_mat);
            let mut end_dir = log_axis;
            end_dir.multiply_scalar(end_side);
            cap.quaternion().set_from_unit_vectors(&v3(0, 0, 1), &end_dir);
            cap.position().copy(&lp).add_scaled_vector(&end_dir, half_l + 0.0005);
            lb.add(&cap);
        }
    }
    lb.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    lb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lb);
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
