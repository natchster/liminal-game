//! `place_tripod_telescopes`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_tripod_telescopes(g: &mut Game, _dist: &[i32]) {
    g.pp.telescope_positions.clear(); // reset per level
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
        leg.rotation().z = ang.dcos() * 0.245;
        leg.rotation().x = -ang.dsin() * 0.245;
        leg.scale().y = 1.02;
        leg.position().set(ang.dcos() * 0.118, tripod_h / 2.0, ang.dsin() * 0.118);
        tt.add(&leg);
    }
    // Rubber foot caps at bottom of each leg.
    for i in 0..3 {
        let ang = (i as f64 / 3.0) * PI * 2.0;
        let foot = mesh(pg("TripodTelescopes__foot", cyl_geo!(0.012, 0.012, 0.008, 5)), lambert!({ color: 0x0a0a0a }));
        foot.position().set(ang.dcos() * 0.22, 0.004, ang.dsin() * 0.22);
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
    eyepiece_lens.position().set(-tube_l * 0.5, 0.030 + 0.0305 * (0.4f64).dcos(), 0.025 + 0.0305 * (0.4f64).dsin());
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
    // Subtle pan animator — 40% chance ("tracking a star").
    let is_tracking = rnd() < 0.4;
    if is_tracking {
        let pan_phase = rnd() * PI * 2.0;
        let base_tube_z = tube_group.rot().z;
        let (mh, tg) = (mount_head.clone(), tube_group.clone());
        g.animate(move |_g, _dt, t| {
            // Pan the mount_head + tube_group subgroups (tt.rotation.y is the world orientation).
            mh.rotation().y = (t * 0.3 + pan_phase).dsin() * 0.06;
            tg.rotation().y = (t * 0.3 + pan_phase).dsin() * 0.06;
            // Slight elevation adjustment on tube itself.
            tg.rotation().z = base_tube_z + (t * 0.2 + pan_phase * 1.3).dsin() * 0.020;
        });
    }
    tt.user().set("isTracking", is_tracking);
    tt.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    tt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tt);
    // Register only tracking telescopes for focus-knob click audio.
    if is_tracking {
        let p = tt.pos();
        g.pp.telescope_positions.push(v3(p.x, 0, p.z));
    }
}
