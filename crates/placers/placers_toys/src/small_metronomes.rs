//! `place_small_metronomes`: Small metronome: 6% chance per level. Pyramid body with swinging pendulum arm animator. Music family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small metronome: 6% chance per level. Pyramid body with
/// swinging pendulum arm animator. Music family.
pub(crate) fn place_small_metronomes(g: &mut Game, _dist: &[i32]) {
    g.pp.metronome_positions.clear(); // reset per level
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
    // Track metronome for tick audio (with its swing period, so ticks
    // sync with the pendulum: kept in the parallel `metronome_periods`).
    let p = metronome.pos();
    g.pp.metronome_positions.push(v3(p.x, 0.06, p.z));
    g.pp.metronome_periods.push(swing_period);
}
