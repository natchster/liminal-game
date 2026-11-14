//! `place_dress_forms`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Dress form / tailor's mannequin on tripod stand (floor): 2.5% chance.
pub(crate) fn place_dress_forms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Torso fabric colour pool — muslin/canvas naturals.
    let torso_colors = [0xe0d8c0, 0xd0c8b0, 0xc8bfa0, 0xb8a898];
    let torso_mat = lambert!({ color: choice(&torso_colors) });
    let wood_mat = lambert!({ color: 0x604020 });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let df = group();
    // Tripod stand — 3 splayed wooden legs.
    let leg_h = 0.30;
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("DressForms__leg", cyl_geo!(0.014, 0.020, leg_h, 6)), &wood_mat);
        // Splay: foot on its pad at r 0.19, the top leaning 0.548 rad inward
        // into the hub (stretched 10% to reach it).
        let leg_dir = v3(-angle.dcos() * 0.548f64.dsin(), 0.548f64.dcos(), -angle.dsin() * 0.548f64.dsin());
        leg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &leg_dir);
        leg.scale().y = 1.1;
        leg.position().set(
            angle.dcos() * 0.19 + leg_dir.x * leg_h * 0.55,
            0.008 + leg_dir.y * leg_h * 0.55,
            angle.dsin() * 0.19 + leg_dir.z * leg_h * 0.55,
        );
        df.add(&leg);
        // Foot pad at base.
        let foot = mesh(pg("DressForms__foot", cyl_geo!(0.020, 0.024, 0.008, 8)), &wood_mat);
        foot.position().set(angle.dcos() * 0.19, 0.004, angle.dsin() * 0.19);
        df.add(&foot);
    }
    // Central tripod hub cap.
    let hub = mesh(pg("DressForms__hub", sphere_geo!(0.024, 8, 6)), &wood_mat);
    hub.position().y = leg_h;
    df.add(&hub);
    // Central vertical pole (up to torso bottom).
    let pole_h = 0.60;
    let pole = mesh(pg("DressForms__pole", cyl_geo!(0.012, 0.012, pole_h, 6)), &wood_mat);
    pole.position().y = leg_h + pole_h / 2.0;
    df.add(&pole);
    // Brass adjustment wheel on pole.
    let wheel = mesh(pg("DressForms__wheel", torus_geo!(0.016, 0.004, 4, 12)), &brass_mat);
    wheel.rotation().x = PI / 2.0; // horizontal round the pole, like the neck wheel
    wheel.position().y = leg_h + pole_h * 0.4;
    df.add(&wheel);
    // Torso form — 3 stacked ellipsoids for bust / waist / hip.
    let torso_base_y = leg_h + pole_h;
    // Hip (widest).
    let hip = mesh(pg("DressForms__hip", sphere_geo!(0.14, 12, 8)), &torso_mat);
    hip.scale().set(1.0, 0.45, 0.85);
    hip.position().y = torso_base_y + 0.06;
    df.add(&hip);
    // Waist (narrowest), stretched from the hip's top to the bust's bottom.
    let waist = mesh(pg("DressForms__waist", sphere_geo!(0.10, 12, 8)), &torso_mat);
    waist.scale().set(1.0, 0.72, 0.85);
    waist.position().y = torso_base_y + 0.199;
    df.add(&waist);
    // Bust (medium, higher).
    let bust = mesh(pg("DressForms__bust", sphere_geo!(0.13, 12, 8)), &torso_mat);
    bust.scale().set(1.0, 0.577, 0.85);
    bust.position().y = torso_base_y + 0.35;
    df.add(&bust);
    // Shoulder narrow top.
    let shoulder = mesh(pg("DressForms__shoulder", cyl_geo!(0.08, 0.11, 0.06, 8)), &torso_mat);
    shoulder.position().y = torso_base_y + 0.46;
    df.add(&shoulder);
    // Neck/collar top (no head).
    let collar = mesh(pg("DressForms__collar", cyl_geo!(0.030, 0.040, 0.030, 8)), &torso_mat);
    collar.position().y = torso_base_y + 0.51;
    df.add(&collar);
    // Brass adjustment wheel at neck top.
    let neck_wheel = mesh(pg("DressForms__wheel", torus_geo!(0.014, 0.003, 4, 12)), &brass_mat);
    neck_wheel.rotation().x = PI / 2.0;
    neck_wheel.position().y = torso_base_y + 0.530;
    df.add(&neck_wheel);
    // Visible seam lines running vertically along torso.
    let seam_mat = lambert!({ color: 0x6a5a48 });
    for seam_angle in [0.0, PI] {
        let seam = mesh(box_geo!(0.002, 0.44, 0.001), &seam_mat);
        seam.position().set(seam_angle.dcos() * 0.115, torso_base_y + 0.22, seam_angle.dsin() * 0.115);
        df.add(&seam);
    }
    // 40%: pincushion on hip area.
    if rnd() < 0.4 {
        let cushion_mat = lambert!({ color: 0x8b1010 });
        let cushion = mesh(pg("DressForms__cushion", sphere_geo!(0.018, 8, 6)), &cushion_mat);
        cushion.scale().y = 0.75;
        cushion.position().set(0.10, torso_base_y + 0.06, 0.08);
        df.add(&cushion);
        // 4-6 tiny pin heads.
        let pin_count = 4 + (rnd() * 3.0).floor() as i32;
        for _i in 0..pin_count {
            let pin_angle = rnd() * PI;
            let pin_shaft = mesh(cyl_geo!(0.0006, 0.0006, 0.015, 3), lambert!({ color: 0xa0a0a8 }));
            let pin_angle_z = pin_angle - PI / 2.0;
            pin_shaft.rotation().z = pin_angle_z;
            pin_shaft.position().set(0.10 + pin_angle.dcos() * 0.020, torso_base_y + 0.06 + pin_angle.dsin() * 0.015, 0.08);
            df.add(&pin_shaft);
            // Head — coloured bead.
            let bead_colors = [0xf0d040, 0xa02040, 0x2060a0, 0x408060];
            let bead_mat = lambert!({ color: choice(&bead_colors) });
            let bead = mesh(sphere_geo!(0.0018, 5, 4), &bead_mat);
            bead.position().set(0.10 + pin_angle.dcos() * 0.028, torso_base_y + 0.06 + pin_angle.dsin() * 0.023, 0.08);
            df.add(&bead);
        }
    }
    // 30%: partially draped fabric on torso (unfinished dress).
    if rnd() < 0.3 {
        let fabric_colors = [0x603030, 0x2a4060, 0x604020, 0x503060, 0x604040];
        let fabric_mat = lambert!({ color: choice(&fabric_colors) });
        // Stood up against the bust's front, narrowed to 18 cm, tilted 0.15.
        let drape = mesh(pg("DressForms__drape", box_geo!(0.24, 0.005, 0.15)), &fabric_mat);
        drape.scale().x = 0.75;
        {
            let mut r = drape.rotation();
            r.set(PI / 2.0, 0, 0.15);
            r.order = EulerOrder::ZXY;
        }
        drape.position().set(0, torso_base_y + 0.33, 0.113);
        df.add(&drape);
        // Fabric flowing down side.
        let flow = mesh(pg("DressForms__flow", box_geo!(0.008, 0.22, 0.10)), &fabric_mat);
        flow.position().set(0.095, torso_base_y + 0.14, 0.10);
        flow.rotation().z = 0.12;
        df.add(&flow);
        // 3 pin markings holding fabric in place.
        for i in 0..3 {
            let pin_mark = mesh(sphere_geo!(0.0018, 4, 4), lambert!({ color: 0xa0a0a8 }));
            let pz = 0.1162 + rnd() * 0.001; // heads on the drape's face
            pin_mark.position().set(-0.06 + i as f64 * 0.06, torso_base_y + 0.30, pz);
            df.add(&pin_mark);
        }
        // Slow breeze-sway animator on the loose flowing strip.
        let sway_freq = 0.30 + rnd() * 0.25;
        let sway_phase = rnd() * PI * 2.0;
        let flow_base_rot_z = flow.rot().z;
        g.animate(move |_g, _dt, t| {
            flow.rotation().z = flow_base_rot_z + (t * sway_freq + sway_phase).dsin() * 0.028;
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.0;
    df.position().set(px, 0, pz);
    df.rotation().y = rnd() * PI * 2.0;
    g.world.add(&df);
}
