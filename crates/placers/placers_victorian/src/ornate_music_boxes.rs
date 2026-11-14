//! `place_ornate_music_boxes`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Small ornate music box (floor): 4% chance per level.
pub(crate) fn place_ornate_music_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.music_box_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let box_colors = [0x603020, 0x8b5a2b, 0x2a1810, 0x604040, 0x201010];
    let box_mat = lambert!({ color: choice(&box_colors) });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let velvet_colors = [0xa02020, 0x2060a0, 0x404060, 0x604060, 0x603030];
    let velvet_mat = lambert!({ color: choice(&velvet_colors) });
    let mb = group();
    let (box_w, box_d, box_h) = (0.14, 0.10, 0.055);
    // Bottom half.
    let bottom = mesh(pg("OrnateMusicBoxes__bottom", box_geo!(box_w, box_h, box_d)), &box_mat);
    bottom.position().y = box_h / 2.0 + 0.002;
    mb.add(&bottom);
    // Gold trim edges.
    let trim_top = mesh(pg("OrnateMusicBoxes__trim", box_geo!(box_w + 0.004, 0.003, box_d + 0.004)), &gold_mat);
    trim_top.position().y = box_h + 0.002;
    mb.add(&trim_top);
    let trim_bottom = mesh(pg("OrnateMusicBoxes__trim", box_geo!(box_w + 0.004, 0.003, box_d + 0.004)), &gold_mat);
    trim_bottom.position().y = 0.005;
    mb.add(&trim_bottom);
    // Top lid — tilted open at -PI/3 rad.
    let top = mesh(pg("OrnateMusicBoxes__lid", box_geo!(box_w, box_h * 0.35, box_d)), &box_mat);
    // Hinged on the casket's back top edge (on the trim), opened 60°.
    let lid_hinge = group();
    lid_hinge.position().set(0, box_h + 0.0035, -box_d * 0.5);
    lid_hinge.rotation().x = -PI / 3.0;
    mb.add(&lid_hinge);
    top.position().set(0, box_h * 0.175, box_d * 0.5);
    lid_hinge.add(&top);
    // Decorative gold pattern on outer top (4 tiny corner ornaments).
    for cx in [-box_w * 0.4, box_w * 0.4] {
        for cz in [-box_d * 0.35, box_d * 0.35] {
            let orn = mesh(pg("OrnateMusicBoxes__orn", sphere_geo!(0.003, 5, 4)), &gold_mat);
            orn.position().set(cx, box_h + box_d * 0.72, cz - box_d * 0.5 + (cz - box_d * 0.35).abs() * 0.2);
            // On the lid's outer border, in the hinge frame (so it rides the lid).
            orn.position().set(cx * 1.12, box_h * 0.35 - 0.001, box_d * 0.5 + cz * 1.2);
            lid_hinge.add(&orn);
        }
    }
    // Velvet lining inside bottom.
    let lining = mesh(pg("OrnateMusicBoxes__lining", box_geo!(box_w - 0.010, 0.003, box_d - 0.010)), &velvet_mat);
    lining.position().y = box_h + 0.002;
    mb.add(&lining);
    // Tiny mirror on interior top (reflective silver square).
    let mirror_mat = basic_mat!({ color: 0xe8f0f4 });
    let mirror = mesh(box_geo!(box_w * 0.6, 0.001, box_d * 0.55), &mirror_mat);
    // In the lid's well (hinge frame), just under the well ceiling.
    mirror.position().set(0, box_h * 0.175 + 0.0002, box_d * 0.5);
    lid_hinge.add(&mirror);
    // Tiny ballerina figure on spring.
    let ballerina_mat = lambert!({ color: 0xf0d0d8 });
    let skirt_mat = lambert!({ color: 0xf8f0f8 });
    let ballerina_group = group();
    // Post/spring.
    let post = mesh(pg("OrnateMusicBoxes__post", cyl_geo!(0.001, 0.001, 0.020, 3)), &gold_mat);
    post.position().y = 0.010;
    ballerina_group.add(&post);
    // Body — thin cylinder.
    let body = mesh(pg("OrnateMusicBoxes__body", cyl_geo!(0.003, 0.004, 0.014, 5)), &ballerina_mat);
    body.position().y = 0.028;
    ballerina_group.add(&body);
    // Head — small sphere.
    let head = mesh(pg("OrnateMusicBoxes__head", sphere_geo!(0.004, 5, 4)), &ballerina_mat);
    head.position().y = 0.037;
    ballerina_group.add(&head);
    // Skirt — flat wide cone.
    let skirt = mesh(pg("OrnateMusicBoxes__skirt", cone_geo!(0.010, 0.006, 8)), &skirt_mat);
    skirt.position().y = 0.022;
    ballerina_group.add(&skirt);
    // Arms — 2 tiny cylinders.
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("OrnateMusicBoxes__arm", cyl_geo!(0.001, 0.001, 0.010, 3)), &ballerina_mat);
        arm.rotation().z = side * 0.5;
        arm.position().set(side * 0.003, 0.033, 0);
        ballerina_group.add(&arm);
    }
    ballerina_group.position().set(0, box_h + 0.005, 0);
    mb.add(&ballerina_group);
    // 50% chance: ballerina spinning animator.
    let is_playing = rnd() < 0.5;
    if is_playing {
        let bg2 = ballerina_group.clone();
        g.animate(move |_g, dt, _t| {
            bg2.rotation().y += dt * 2.5; // ~24 RPM
        });
    }
    mb.user().set("isPlaying", is_playing);
    // Small winding key on side — cylinder + tiny cross bar.
    let key_shaft = mesh(pg("OrnateMusicBoxes__keyShaft", cyl_geo!(0.003, 0.003, 0.015, 5)), &gold_mat);
    key_shaft.rotation().z = PI / 2.0;
    key_shaft.position().set(box_w * 0.55, box_h * 0.5, 0);
    mb.add(&key_shaft);
    let key_bar = mesh(pg("OrnateMusicBoxes__keyBar", box_geo!(0.003, 0.014, 0.003)), &gold_mat);
    key_bar.position().set(box_w * 0.55 + 0.010, box_h * 0.5, 0);
    mb.add(&key_bar);
    let key_bar2 = mesh(pg("OrnateMusicBoxes__keyBar2", box_geo!(0.003, 0.003, 0.014)), &gold_mat);
    key_bar2.position().set(box_w * 0.55 + 0.010, box_h * 0.5, 0);
    mb.add(&key_bar2);
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    mb.position().set(px, 0, pz);
    mb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mb);
    // Register position only for playing (spinning) music boxes.
    if is_playing {
        let p = mb.pos();
        g.pp.music_box_positions.push(v3(p.x, 0, p.z));
    }
}
