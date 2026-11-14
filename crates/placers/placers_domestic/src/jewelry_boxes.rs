//! `place_jewelry_boxes`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Opened jewelry box w/ spilled contents (floor): 4% chance per level.
pub(crate) fn place_jewelry_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let box_colors = [0x603020, 0x201010, 0x101020, 0xa02040, 0x804a20];
    let box_mat = lambert!({ color: choice(&box_colors) });
    let lining_colors = [0xa02020, 0x2060a0, 0x101010, 0x804060, 0x606060];
    let lining_mat = lambert!({ color: choice(&lining_colors) });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let silver_mat = lambert!({ color: 0xd0d0d8 });
    let jb = group();
    // Bottom half — hinged box (larger than the glasses case).
    let (box_w, box_d, box_h) = (0.16, 0.11, 0.05);
    let bottom = mesh(pg("JewelryBoxes__base", box_geo!(box_w, box_h, box_d)), &box_mat);
    bottom.position().y = box_h / 2.0 + 0.002;
    jb.add(&bottom);
    // Interior lining (velvet — colored lining pool).
    let inside_lining = mesh(pg("JewelryBoxes__pad", box_geo!(box_w - 0.012, 0.004, box_d - 0.012)), &lining_mat);
    inside_lining.position().y = box_h + 0.002;
    jb.add(&inside_lining);
    // Compartment dividers on inside (small ridges).
    for dx in [-box_w * 0.25, box_w * 0.25] {
        let divider = mesh(pg("JewelryBoxes__divider", box_geo!(0.002, 0.006, box_d - 0.014)), &box_mat);
        divider.position().set(dx, box_h + 0.005, 0);
        jb.add(&divider);
    }
    let horiz_divider = mesh(pg("JewelryBoxes__crossDivider", box_geo!(box_w - 0.014, 0.006, 0.002)), &box_mat);
    horiz_divider.position().set(0, box_h + 0.005, 0);
    jb.add(&horiz_divider);
    // Top half tilted open — swings about the hinge line on the base's back top edge.
    let lid_pivot = group();
    lid_pivot.position().set(0, box_h + 0.002, -box_d * 0.5);
    lid_pivot.rotation().x = -PI / 3.0;
    jb.add(&lid_pivot);
    let top = mesh(pg("JewelryBoxes__lid", box_geo!(box_w, box_h * 0.5, box_d)), &box_mat);
    top.position().set(0, box_h * 0.25, box_d * 0.5);
    lid_pivot.add(&top);
    // Interior lining top: against the ceiling of the lid's hollow underside.
    let top_lining = mesh(pg("JewelryBoxes__lidLining", box_geo!(box_w - 0.012, 0.003, box_d - 0.012)), &lining_mat);
    top_lining.position().set(0, box_h * 0.25 + 0.0008, box_d * 0.5);
    lid_pivot.add(&top_lining);
    // Small dark hinge, on the hinge axis.
    let hinge = mesh(pg("JewelryBoxes__hinge", cyl_geo!(0.003, 0.003, box_w * 0.85, 5)), lambert!({ color: 0x606060 }));
    hinge.rotation().z = PI / 2.0;
    hinge.position().set(0, box_h + 0.002, -box_d * 0.5);
    jb.add(&hinge);
    // Small gold clasp on front.
    let clasp = mesh(pg("JewelryBoxes__clasp", box_geo!(0.010, 0.006, 0.003)), &gold_mat);
    clasp.position().set(0, box_h * 0.7, box_d * 0.5 + 0.002);
    jb.add(&clasp);
    // 2-3 rings (small tori).
    let ring_count = 2 + (rnd() * 2.0).floor() as i32;
    for _i in 0..ring_count {
        let is_gold = rnd() < 0.5;
        let ring_mat2 = if is_gold { &gold_mat } else { &silver_mat };
        let ring = mesh(pg("JewelryBoxes__ring", torus_geo!(0.010, 0.002, 3, 10)), ring_mat2);
        ring.rotation().x = PI / 2.0 + (rnd() - 0.5) * 0.6;
        ring.rotation().z = (rnd() - 0.5) * 0.6;
        let rx = -box_w * 0.3 + (rnd() - 0.5) * 0.04;
        let rz = -box_d * 0.2 + (rnd() - 0.5) * 0.04;
        ring.position().set(rx, box_h + 0.008, rz);
        jb.add(&ring);
        // 40% ring w/ stone.
        if rnd() < 0.4 {
            let stone_colors = [0xa02020, 0x2060a0, 0x408060, 0xc0a0d0, 0xf0f0f0];
            let stone_mat = lambert!({ color: choice(&stone_colors) });
            let stone = mesh(pg("JewelryBoxes__stone", octa_geo!(0.004, 0)), &stone_mat);
            stone.position().copy(&ring.pos());
            stone.position().y += 0.004;
            jb.add(&stone);
        }
    }
    // Necklace — 8-segment chain trailing out of box.
    let necklace_colors = [0xc0a060, 0xd0d0d8, 0xf0e0a0];
    let necklace_mat = lambert!({ color: choice(&necklace_colors) });
    // Starts on the pad in the +X compartment, drapes over the +X rim and
    // falls down the outside onto the floor.
    let chain_start = v3(box_w * 0.30, box_h + 0.006, box_d * 0.15);
    let chain_end = v3(box_w * 0.85, 0.0013, box_d * 0.30);
    let chain_ys = [box_h + 0.006, box_h + 0.006, box_h + 0.0045, 0.030, 0.010, 0.0025, 0.0013, 0.0013];
    let chain_segs = 8;
    let mut prev = chain_start;
    for i in 0..chain_segs {
        let t = (i + 1) as f64 / chain_segs as f64;
        let arc_amt = (t * PI).dsin() * 0.02;
        let cx = chain_start.x + (chain_end.x - chain_start.x) * t;
        let cy = chain_ys[i];
        let cz = chain_start.z + (chain_end.z - chain_start.z) * t + (t * PI * 2.0).dcos() * arc_amt * 0.6;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("JewelryBoxes__chain", cyl_geo!(0.0012, 0.0012, dir_len, 3)), &necklace_mat);
            let mut mid = prev;
            mid.add(&next).multiply_scalar(0.5);
            seg.position().copy(&mid);
            let mut v = next;
            v.sub(&prev).normalize();
            seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
            jb.add(&seg);
        }
        prev = next;
    }
    // Pendant at chain end (small teardrop / gem).
    let pendant_colors = [0xa02020, 0x2060a0, 0x408060, 0xc0a0d0, 0xf0f0f0];
    let pendant_mat = lambert!({ color: choice(&pendant_colors) });
    let pendant = mesh(pg("JewelryBoxes__pendant", octa_geo!(0.006, 0)), &pendant_mat);
    pendant.position().copy(&prev);
    pendant.position().y += 0.005;
    jb.add(&pendant);
    // 60% chance: pair of earrings scattered nearby.
    if rnd() < 0.6 {
        for _i in 0..2 {
            let earring_mat = &necklace_mat;
            let earring = mesh(pg("JewelryBoxes__earring", torus_geo!(0.005, 0.0008, 3, 6)), earring_mat);
            let ear_tilt = (rnd() - 0.5) * 0.4;
            earring.rotation().x = PI / 2.0 + ear_tilt;
            let ex = -box_w * 0.4 - rnd() * 0.05;
            let ey = 0.005 * ear_tilt.dsin().abs() + 0.0008; // low edge on the floor
            let ez = box_d * 0.4 + (rnd() - 0.5) * 0.06;
            earring.position().set(ex, ey, ez);
            jb.add(&earring);
            // Small stone dangling below (resting on the floor).
            let ear_stone = mesh(pg("JewelryBoxes__earStone", octa_geo!(0.003, 0)), &pendant_mat);
            ear_stone.position().copy(&earring.pos());
            ear_stone.position().y = 0.003;
            ear_stone.position().z += 0.004;
            jb.add(&ear_stone);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    jb.position().set(px, 0, pz);
    jb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&jb);
}
