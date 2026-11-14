//! `place_small_phonographs`
use crate::shared::vsub;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage phonograph: 4% chance per level.
pub(crate) fn place_small_phonographs(g: &mut Game, _dist: &[i32]) {
    g.pp.phonograph_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let wood_mat = lambert!({ color: 0x6a3820 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let record_mat = lambert!({ color: 0x201a10 });
    let label_mat = lambert!({ color: 0xa02040 });
    let phono = group();
    // Wood base cube.
    let base = mesh(pg("SmallPhonographs__base", box_geo!(0.16, 0.06, 0.14)), &wood_mat);
    base.position().y = 0.03;
    phono.add(&base);
    // Turntable disc.
    let turntable = mesh(pg("SmallPhonographs__turntable", cyl_geo!(0.055, 0.055, 0.006, 12)), &record_mat);
    turntable.position().y = 0.063;
    phono.add(&turntable);
    // Record on turntable (slightly smaller).
    let record = mesh(pg("SmallPhonographs__record", cyl_geo!(0.052, 0.052, 0.002, 14)), &record_mat);
    record.position().y = 0.067;
    phono.add(&record);
    // Record label.
    let label = mesh(circle_geo!(0.015, 8), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0685;
    phono.add(&label);
    // Tone arm: from its pivot (collar end, -Y) on the cabinet's back-right
    // corner to the sound box (+Y end) on the record.
    let arm_pivot = v3(0.076, 0.064, -0.068);
    let arm_tip = v3(0.032, 0.0745, 0);
    let arm = mesh(pg("SmallPhonographs__arm", cyl_geo!(0.003, 0.003, 0.08, 4)), &brass_mat);
    arm.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &vsub(arm_tip, arm_pivot).normalized());
    arm.scale().y = arm_tip.distance_to(&arm_pivot) / 0.08;
    {
        let mut p = arm.position();
        p.copy(&arm_pivot).add(&arm_tip).multiply_scalar(0.5);
    }
    phono.add(&arm);
    let arm_head = mesh(pg("SmallPhonographs__head", box_geo!(0.010, 0.006, 0.008)), &brass_mat);
    arm_head.position().set(0.032, 0.070, 0);
    phono.add(&arm_head);
    // Large brass horn: throat (apex, +Y) at the arm pivot, the flared mouth up and back.
    let horn = mesh(pg("SmallPhonographs__horn", cone_geo!(0.08, 0.20, 12, 1, true)), &brass_mat);
    let horn_dir = v3(-0.30, 0.90, -0.30).normalized();
    horn.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &(-horn_dir));
    {
        let mut p = horn.position();
        p.copy(&arm_pivot).add_scaled_vector(&horn_dir, 0.10);
    }
    phono.add(&horn);
    // Turntable slow rotation animator.
    let rot_rate = 0.5 + rnd() * 0.5;
    {
        let (turntable, record, label) = (turntable.clone(), record.clone(), label.clone());
        g.animate(move |_g, _dt, t| {
            turntable.rotation().y = t * rot_rate;
            record.rotation().y = t * rot_rate;
            label.rotation().z = t * rot_rate;
        });
    }
    phono.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    phono.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phono);
    // Track for vinyl crackle audio.
    let pp = phono.pos();
    g.pp.phonograph_positions.push(v3(pp.x, 0.10, pp.z));
}
