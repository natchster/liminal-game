//! `place_small_toy_planes`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_pos;

// Small toy plane: 8% chance per level. Toy airplane with
// fuselage + wings + tail + propeller. Child-toy family.
pub(crate) fn place_small_toy_planes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let body_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0x606870, 0xf0ece0];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let prop_mat = lambert!({ color: 0x201a10 });
    let plane = group();
    // Fuselage: elongated capsule.
    let fus = mesh(pg("SmallToyPlanes__fuselage", cyl_geo!(0.014, 0.008, 0.12, 8)), &body_mat);
    fus.rotation().z = PI / 2.0;
    fus.position().set(0.06, 0.020, 0);
    plane.add(&fus);
    // Nose cap (rounded).
    let nose = mesh(pg("SmallToyPlanes__nose", sphere_geo!(0.014, 8, 6)), &body_mat);
    nose.position().set(0, 0.020, 0);
    plane.add(&nose);
    // Tail cap.
    let tail = mesh(pg("SmallToyPlanes__tail", sphere_geo!(0.008, 6, 5)), &body_mat);
    tail.position().set(0.12, 0.020, 0);
    plane.add(&tail);
    // Wings: long thin box across.
    let wings = mesh(pg("SmallToyPlanes__wing", box_geo!(0.025, 0.005, 0.14)), &body_mat);
    wings.position().set(0.050, 0.020, 0);
    plane.add(&wings);
    // Tail fin (vertical).
    let fin = mesh(pg("SmallToyPlanes__fin", box_geo!(0.020, 0.024, 0.004)), &body_mat);
    fin.position().set(0.110, 0.028, 0);
    plane.add(&fin);
    // Tail horizontal stabilizer.
    let stab = mesh(pg("SmallToyPlanes__stab", box_geo!(0.020, 0.003, 0.045)), &body_mat);
    stab.position().set(0.110, 0.020, 0);
    plane.add(&stab);
    // Propeller (crossed sticks at nose).
    let prop_pivot = group();
    prop_pivot.position().set(-0.015, 0.020, 0); // just ahead of the nose (front at x -0.014)
    for i in 0..2 {
        let blade = mesh(pg("SmallToyPlanes__blade", box_geo!(0.002, 0.030, 0.005)), &prop_mat);
        blade.rotation().x = i as f64 * PI / 2.0;
        prop_pivot.add(&blade);
    }
    plane.add(&prop_pivot);
    // Propeller slow spin animator.
    let spin_rate = 3.0 + rnd() * 4.0;
    let pp = prop_pivot.clone();
    g.animate(move |_g, _dt, t| {
        pp.rotation().x = t * spin_rate;
    });
    scatter_pos(g, &plane, x, z, 2.4);
    plane.rotation().y = rnd() * PI * 2.0;
    // 30% chance tipped on side: rolled about the fuselage (x), resting on a
    // wing tip (YXZ so the roll is about the plane's own axis, then the yaw).
    if rnd() < 0.30 {
        plane.set_rotation_order(EulerOrder::YXZ);
        plane.rotation().x = PI / 2.0;
        plane.position().y = 0.070;
    }
    g.world.add(&plane);
}
