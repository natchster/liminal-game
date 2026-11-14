//! `place_car_keys`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Dropped car keys w/ keyring + tag (floor): 7% chance per level.
pub(crate) fn place_car_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let key_mat = if rnd() < 0.5 { metal_mat.clone() } else { brass_mat.clone() };
    // Fob color pool (car remote body).
    let fob_colors = [0x101010, 0x2060a0, 0x603030, 0x404040];
    let fob_mat = lambert!({ color: pick(&fob_colors) });
    let k = group();
    // Central keyring — flat torus.
    let ring = mesh(pg("CarKeys__ring", torus_geo!(0.012, 0.0012, 4, 12)), &metal_mat);
    ring.rotation().x = -PI / 2.0;
    ring.position().y = 0.002;
    k.add(&ring);
    // Main house key — flat body + notched teeth.
    let key = group();
    // Head — flat circle-ish disc.
    let key_head = mesh(pg("CarKeys__bow", cyl_geo!(0.008, 0.008, 0.0015, 8)), &key_mat);
    key_head.position().set(0, 0, 0);
    key.add(&key_head);
    // Shaft — thin rectangular box.
    let shaft = mesh(pg("CarKeys__blade", box_geo!(0.028, 0.0015, 0.004)), &key_mat);
    shaft.position().set(0.018, 0, 0);
    key.add(&shaft);
    // 3 tiny tooth notches on bottom edge.
    for i in 0..3 {
        let tooth = mesh(pg("CarKeys__tooth", box_geo!(0.003, 0.001, 0.003)), &key_mat);
        tooth.position().set(0.020 + i as f64 * 0.006, 0, 0.003);
        key.add(&tooth);
    }
    // Head hole (dark).
    let hole_mat = lambert!({ color: 0x0a0a0a });
    let hole = mesh(cyl_geo!(0.002, 0.002, 0.002, 6), &hole_mat);
    hole.position().set(-0.0042, 0.001, 0); // in the bow's hanging hole
    key.add(&hole);
    key.rotation().y = 0.5 + (rnd() - 0.5) * 0.5;
    key.rotation().z = (rnd() - 0.5) * 0.15;
    // threaded: the bow's hole (x = -0.0042) sits on the ring, key pointing out
    let kry = key.rot().y;
    key.position().set(kry.cos() * 0.0162, 0.002, -kry.sin() * 0.0162);
    k.add(&key);
    // 2nd smaller key — mailbox/luggage style (short, no teeth).
    let key2 = group();
    let key2_head = mesh(pg("CarKeys__bow", cyl_geo!(0.006, 0.006, 0.0015, 6)), &key_mat);
    key2.add(&key2_head);
    let shaft2 = mesh(pg("CarKeys__blade2", box_geo!(0.016, 0.0015, 0.003)), &key_mat);
    shaft2.position().set(0.011, 0, 0);
    key2.add(&shaft2);
    key2.rotation().y = 1.8;
    key2.position().set(1.8f64.cos() * 0.0152, 0.002, -1.8f64.sin() * 0.0152); // hole on the ring
    k.add(&key2);
    // Car fob — small rectangular remote w/ buttons.
    let fob = group();
    let fob_body = mesh(pg("CarKeys__fob", box_geo!(0.024, 0.006, 0.014)), &fob_mat);
    fob_body.position().y = 0.003;
    fob.add(&fob_body);
    // 2 tiny raised buttons.
    let btn_mat = lambert!({ color: 0x606060 });
    for i in 0..2 {
        let btn = mesh(pg("CarKeys__button", cyl_geo!(0.0025, 0.0025, 0.001, 5)), &btn_mat);
        btn.position().set(-0.005 + i as f64 * 0.010, 0.007, 0);
        fob.add(&btn);
    }
    // Fob attachment ring (small torus).
    let fob_ring = mesh(pg("CarKeys__smallring", torus_geo!(0.003, 0.0007, 3, 6)), &metal_mat);
    fob_ring.rotation().x = PI / 2.0;
    fob_ring.position().set(-0.014, 0.003, 0);
    fob.add(&fob_ring);
    fob.rotation().y = -0.6 + (rnd() - 0.5) * 0.3;
    // its small ring (local x = -0.014) links the key ring, body pointing out
    let fry = fob.rot().y;
    fob.position().set(fry.cos() * 0.0285, 0, -fry.sin() * 0.0285);
    k.add(&fob);
    // 60% chance: leather tag w/ initial letter.
    if rnd() < 0.6 {
        let tag_colors = [0x603020, 0x804020, 0x201010, 0x8b5a2b];
        let tag_mat = lambert!({ color: pick(&tag_colors) });
        let tag = group();
        // Tag body — small rectangular tag.
        let tag_body = mesh(pg("CarKeys__tag", box_geo!(0.016, 0.002, 0.024)), &tag_mat);
        tag_body.position().y = 0.001;
        tag.add(&tag_body);
        // Ring hole area (darker).
        let tag_ring_spot = mesh(circle_geo!(0.002, 5), &hole_mat);
        tag_ring_spot.rotation().x = -PI / 2.0;
        tag_ring_spot.position().set(0, 0.0021, -0.008);
        tag.add(&tag_ring_spot);
        // Metal attachment ring.
        let tag_ring = mesh(pg("CarKeys__smallring", torus_geo!(0.003, 0.0007, 3, 6)), &metal_mat);
        tag_ring.rotation().x = PI / 2.0;
        tag_ring.position().set(0, 0.002, -0.010);
        tag.add(&tag_ring);
        // Small pale initial letter (rectangle stamped indication — abstract).
        let stamp_mat = lambert!({ color: 0xd0c0a0 });
        let stamp = mesh(pg("CarKeys__tagplate", box_geo!(0.006, 0.002, 0.010)), &stamp_mat);
        stamp.position().set(0, 0.003, 0.003);
        tag.add(&stamp);
        // on the free side of the ring, its small ring (local z = -0.010) linking it
        tag.rotation().y = -1.4 + (rnd() - 0.5) * 0.3;
        let try_ = tag.rot().y;
        tag.position().set(try_.sin() * 0.0245, 0, try_.cos() * 0.0245);
        k.add(&tag);
    }
    k.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    k.rotation().y = rnd() * PI * 2.0;
    g.world.add(&k);
}
