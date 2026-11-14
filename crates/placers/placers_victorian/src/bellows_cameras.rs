//! `place_bellows_cameras`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Victorian bellows camera on tripod (floor): 2.5% chance per level.
pub(crate) fn place_bellows_cameras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x2a1810, 0x8b5a2b, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let bellows_mat = lambert!({ color: 0x181818 });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let bc = group();
    // Tripod — 3 splayed wooden legs.
    let leg_h = 1.15; // Tripod ~1.15 m tall.
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("BellowsCameras__leg", cyl_geo!(0.008, 0.014, leg_h, 5)), &wood_mat);
        // Splay: foot on its rubber foot at r 0.20, top leaning 0.16 rad
        // inward so the three tops meet in the hub.
        let leg_dir = v3(-angle.cos() * 0.16f64.sin(), 0.16f64.cos(), -angle.sin() * 0.16f64.sin());
        leg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &leg_dir);
        leg.position().set(
            angle.cos() * 0.20 + leg_dir.x * leg_h / 2.0,
            0.008 + leg_dir.y * leg_h / 2.0,
            angle.sin() * 0.20 + leg_dir.z * leg_h / 2.0,
        );
        bc.add(&leg);
        // Foot at bottom.
        let foot = mesh(pg("BellowsCameras__foot", cyl_geo!(0.010, 0.012, 0.008, 6)), &bellows_mat);
        foot.position().set(angle.cos() * 0.20, 0.004, angle.sin() * 0.20);
        bc.add(&foot);
        // Metal brace between legs.
        if i < 2 {
            let brace = mesh(pg("BellowsCameras__brace", cyl_geo!(0.002, 0.002, 0.14, 3)), &brass_mat);
            let next_angle = ((i + 1) as f64 / 3.0) * PI * 2.0;
            // At the height where the legs' surfaces are 0.14 apart.
            let brace_r = (0.14 + 0.02) / 3f64.sqrt();
            let brace_y = 0.008 + (0.20 - brace_r) / 0.16f64.tan();
            brace.position().set(
                (angle.cos() + next_angle.cos()) * brace_r / 2.0,
                brace_y,
                (angle.sin() + next_angle.sin()) * brace_r / 2.0,
            );
            brace.look_at(next_angle.cos() * brace_r, brace_y, next_angle.sin() * brace_r);
            brace.rotate_x(PI / 2.0);
            bc.add(&brace);
        }
    }
    // Tripod head hub.
    let hub = mesh(pg("BellowsCameras__hub", sphere_geo!(0.020, 8, 6)), &brass_mat);
    hub.position().y = leg_h;
    bc.add(&hub);
    // Camera body — resting on tripod head.
    let cam_y = leg_h + 0.030;
    // Rear plate (film/plate holder).
    let rear_plate = mesh(pg("BellowsCameras__rearPlate", box_geo!(0.16, 0.14, 0.020)), &wood_mat);
    rear_plate.position().set(0, cam_y, -0.06);
    bc.add(&rear_plate);
    // Ground-glass viewing screen.
    let screen = mesh(plane_geo!(0.10, 0.08), lambert!({ color: 0x2a2a30 }));
    screen.rotation().y = PI;
    screen.position().set(0, cam_y, -0.070);
    bc.add(&screen);
    // Bellows (accordion).
    let bellows_len = 0.14;
    let bellows_body = mesh(pg("BellowsCameras__bellows", cyl_geo!(0.055, 0.055, bellows_len, 8)), &bellows_mat);
    bellows_body.rotation().x = PI / 2.0;
    bellows_body.position().set(0, cam_y, -0.06 + 0.010 + bellows_len / 2.0);
    bc.add(&bellows_body);
    // Accordion ridges — 5 tori, shrinking with the bellows' taper.
    for i in 0..5 {
        let t = i as f64 / 4.0;
        let ridge = mesh(pg("BellowsCameras__ridge", torus_geo!(0.056, 0.003, 3, 12)), &bellows_mat);
        let ridge_s = (0.055 - 0.008 * t) / 0.056;
        ridge.scale().set(ridge_s, ridge_s, 1);
        ridge.position().set(0, cam_y, -0.06 + 0.010 + t * bellows_len);
        bc.add(&ridge);
    }
    // Front lens board.
    let front_board = mesh(pg("BellowsCameras__frontBoard", box_geo!(0.14, 0.12, 0.014)), &wood_mat);
    front_board.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.007);
    bc.add(&front_board);
    // Lens barrel.
    let lens_barrel = mesh(pg("BellowsCameras__lens", cyl_geo!(0.028, 0.028, 0.045, 12)), &brass_mat);
    lens_barrel.rotation().x = PI / 2.0;
    lens_barrel.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.014 + 0.022);
    bc.add(&lens_barrel);
    // Lens glass.
    let lens_glass = mesh(circle_geo!(0.024, 12), lambert!({ color: 0x102030 }));
    lens_glass.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.014 + 0.045);
    bc.add(&lens_glass);
    // Focus knob on lens barrel side.
    let focus_knob = mesh(pg("BellowsCameras__knob", cyl_geo!(0.006, 0.006, 0.008, 6)), &brass_mat);
    focus_knob.position().set(0.028, cam_y, -0.06 + 0.010 + bellows_len + 0.022);
    bc.add(&focus_knob);
    // Top adjustment/leveling knob on camera body.
    let adjust_knob = mesh(pg("BellowsCameras__knob", cyl_geo!(0.006, 0.006, 0.006, 6)), &brass_mat);
    adjust_knob.position().set(0, cam_y + 0.073, -0.06); // on the rear plate top
    bc.add(&adjust_knob);
    // 60%: black hood cloth draped over rear plate.
    if rnd() < 0.6 {
        let cloth_mat = lambert!({ color: 0x0a0a0a });
        let cloth_top = mesh(pg("BellowsCameras__clothTop", box_geo!(0.20, 0.008, 0.12)), &cloth_mat);
        cloth_top.scale().x = 0.88;
        cloth_top.position().set(0, cam_y + 0.074, -0.06);
        bc.add(&cloth_top);
        let cloth_left = mesh(pg("BellowsCameras__clothSide", box_geo!(0.008, 0.20, 0.10)), &cloth_mat);
        cloth_left.position().set(-0.084, cam_y - 0.022, -0.06);
        bc.add(&cloth_left);
        let cloth_right = mesh(pg("BellowsCameras__clothSide", box_geo!(0.008, 0.20, 0.10)), &cloth_mat);
        cloth_right.position().set(0.084, cam_y - 0.022, -0.06);
        bc.add(&cloth_right);
        let cloth_back = mesh(pg("BellowsCameras__clothBack", box_geo!(0.20, 0.15, 0.008)), &cloth_mat);
        cloth_back.scale().x = 0.88;
        cloth_back.position().set(0, cam_y - 0.001, -0.116);
        bc.add(&cloth_back);
    }
    // 30%: magnesium flash tray beside camera (T-shaped rod + shallow pan).
    if rnd() < 0.3 {
        // Rod clamped to tripod leg 0 (the +X leg), rising beside the bellows.
        let flash_rod = mesh(pg("BellowsCameras__flashRod", cyl_geo!(0.003, 0.003, 0.25, 4)), &brass_mat);
        let rod_lo = v3(0.20 - (0.987 - 0.008) * 0.16f64.tan(), 0.987, 0);
        let rod_hi = v3(0.12, cam_y + 0.045, 0);
        {
            let mut p = flash_rod.position();
            p.copy(&rod_lo);
            p.add(&rod_hi);
            p.multiply_scalar(0.5);
        }
        let mut rod_d = rod_hi;
        rod_d.sub(&rod_lo);
        rod_d.normalize();
        flash_rod.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &rod_d);
        flash_rod.scale().y = rod_hi.distance_to(&rod_lo) / 0.25;
        bc.add(&flash_rod);
        // Horizontal T-bar at top.
        let t_bar = mesh(pg("BellowsCameras__tBar", cyl_geo!(0.003, 0.003, 0.08, 4)), &brass_mat);
        t_bar.rotation().z = PI / 2.0;
        t_bar.position().set(0.12, cam_y + 0.045, 0);
        bc.add(&t_bar);
        // Shallow flash pan at one end of T.
        let flash_pan = mesh(pg("BellowsCameras__flashPan", cyl_geo!(0.020, 0.015, 0.008, 8)), &brass_mat);
        flash_pan.position().set(0.16, cam_y + 0.045, 0);
        bc.add(&flash_pan);
        // Small pile of magnesium powder in pan.
        let powder = mesh(pg("BellowsCameras__powder", sphere_geo!(0.008, 6, 5)), lambert!({ color: 0xd0d0d0 }));
        powder.scale().set(1.0, 0.35, 1.0);
        powder.position().set(0.16, cam_y + 0.052, 0);
        bc.add(&powder);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.0;
    bc.position().set(px, 0, pz);
    bc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bc);
}
