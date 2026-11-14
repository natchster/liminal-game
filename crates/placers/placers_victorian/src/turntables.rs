//! `place_turntables`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_turntables(g: &mut Game, _dist: &[i32]) {
    g.pp.turntable_positions.clear(); // reset per level
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
            let is_torus = child.try_geo().is_some_and(|gg| gg.params().type_name() == "torus");
            if is_torus && (child.pos().x + 0.05).abs() < 0.001 {
                spin_targets.push(child);
            }
        }
        // Capture arm base Y-rotation for jitter animator.
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
            // Tonearm subtle jitter.
            arm2.rotation().y = arm_base_y_rot + (t * 0.7 + jitter_phase).dsin() * 0.012 + (t * 2.3 + jitter_phase2).dsin() * 0.004;
        });
    }
    tt.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    tt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tt);
    // Register only spinning turntables for vinyl-hiss audio.
    if tt_is_playing {
        let p = tt.pos();
        g.pp.turntable_positions.push(v3(p.x, 0, p.z));
    }
}
