//! `place_teapots`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Teapot on trivet: 4% chance per level.
pub(crate) fn place_teapots(g: &mut Game, _dist: &[i32]) {
    g.pp.teapot_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pot_colors: [u32; 6] = [0xf0f0f0, 0x408080, 0xc02020, 0x203050, 0xd0a040, 0x606060];
    let pot_mat = lambert!({ color: pick(&pot_colors) });
    let trivet_mat = lambert!({ color: 0x505050 });
    let teapot = group();
    let trivet = mesh(pg("Teapots__trivet", cyl_geo!(0.075, 0.075, 0.006, 10)), &trivet_mat);
    trivet.position().y = 0.003;
    teapot.add(&trivet);
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let foot = mesh(pg("Teapots__foot", cyl_geo!(0.006, 0.006, 0.008, 4)), &trivet_mat);
        // Under the trivet (top 0.5 mm into its underside); the group is raised onto the feet.
        foot.position().set(a.cos() * 0.060, -0.0035, a.sin() * 0.060);
        teapot.add(&foot);
    }
    let body = mesh(pg("Teapots__body", sphere_geo!(0.060, 12, 10)), &pot_mat);
    body.scale().y = 0.75;
    body.position().y = 0.052;
    teapot.add(&body);
    let spout1 = mesh(pg("Teapots__spout", cyl_geo!(0.012, 0.014, 0.030, 6)), &pot_mat);
    // Wide end in the body, narrow end rising outward (-x) at 30 degrees.
    spout1.rotation().z = PI / 3.0;
    spout1.position().set(-0.058, 0.058, 0);
    teapot.add(&spout1);
    let spout2 = mesh(pg("Teapots__spoutTip", cyl_geo!(0.009, 0.012, 0.024, 6)), &pot_mat);
    // Tip continues from spout1's narrow end (-0.071, 0.0655), rising at 45 degrees.
    spout2.rotation().z = PI / 4.0;
    spout2.position().set(-0.0795, 0.074, 0);
    teapot.add(&spout2);
    let handle = mesh(pg("Teapots__handle", torus_geo!(0.030, 0.006, 4, 10, PI)), &pot_mat);
    // C looping out from the +x side, both ends on the belly.
    handle.rotation().z = -PI / 2.0;
    handle.position().set(0.044, 0.052, 0);
    teapot.add(&handle);
    let lid = mesh(pg("Teapots__lid", cyl_geo!(0.022, 0.026, 0.010, 10)), &pot_mat);
    lid.position().y = 0.101; // seated on the body top (0.097)
    teapot.add(&lid);
    let knob = mesh(pg("Teapots__knob", sphere_geo!(0.008, 6, 5)), &pot_mat);
    knob.position().y = 0.111;
    teapot.add(&knob);
    teapot.position().set(
        g.wx(x) + (rnd() - 0.5) * 2.0,
        0.0075, // standing on the trivet feet
        g.wz(z) + (rnd() - 0.5) * 2.0,
    );
    teapot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&teapot);
    // Subtle lid lift-and-settle: rare, brief Y bob when
    // sin(t*0.12 + phase) > 0.94 (~6% of time). Simulates steam
    // pressure lifting lid slightly.
    let phase = rnd() * PI * 2.0;
    let (lid2, knob2) = (lid.clone(), knob.clone());
    g.animate(move |_g, _dt, t| {
        let trig = (t * 0.12 + phase).sin();
        if trig > 0.94 {
            let local = (trig - 0.94) / 0.06; // 0..1
            let lift = (local * PI).sin() * 0.003;
            lid2.position().y = 0.101 + lift;
            knob2.position().y = 0.111 + lift;
        } else {
            lid2.position().y = 0.101;
            knob2.position().y = 0.111;
        }
    });
    let p = teapot.pos();
    g.pp.teapot_positions.push(v3(p.x, 0, p.z)); // track for whistle
}
