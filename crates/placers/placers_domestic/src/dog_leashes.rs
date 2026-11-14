//! `place_dog_leashes`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Dog leash coiled + hanging harness (floor): 5% chance per level.
pub(crate) fn place_dog_leashes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leash_colors = [0xa02040, 0x2060a0, 0x101010, 0x604020, 0x408060, 0xd0a040, 0xa060a0];
    let leash_mat = lambert!({ color: choice(&leash_colors) });
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let dl = group();
    // Coiled leash — 3 concentric flattened torus rings stacked to look like a coil.
    for i in 0..3 {
        let coil = mesh(pg("DogLeashes__coil", torus_geo!(0.05 - i as f64 * 0.008, 0.006, 4, 12)), &leash_mat);
        coil.rotation().x = PI / 2.0;
        coil.position().y = 0.004 + i as f64 * 0.006;
        dl.add(&coil);
    }
    // Handle loop — larger torus offset to side.
    let handle = mesh(pg("DogLeashes__handle", torus_geo!(0.030, 0.006, 4, 10)), &leash_mat);
    handle.rotation().x = 0.3;
    handle.position().set(0.06, 0.018, 0.02);
    dl.add(&handle);
    // Metal clip at end (small D-ring).
    let clip = mesh(pg("DogLeashes__clip", torus_geo!(0.010, 0.002, 3, 8)), &metal_mat);
    clip.rotation().x = PI / 2.0;
    clip.position().set(-0.055, 0.010, 0.03);
    dl.add(&clip);
    // Metal snap-hook bar.
    let clip_bar = mesh(pg("DogLeashes__snap", box_geo!(0.014, 0.003, 0.006)), &metal_mat);
    clip_bar.position().set(-0.062, 0.010, 0.03);
    dl.add(&clip_bar);
    // 60% chance: dog harness nearby (thicker strap loop + 2 buckles).
    if rnd() < 0.6 {
        let harness_color = choice(&leash_colors);
        let harness_mat = lambert!({ color: harness_color });
        // Chest strap — larger rectangular loop.
        let chest_loop = mesh(pg("DogLeashes__chest", torus_geo!(0.06, 0.008, 4, 14)), &harness_mat);
        chest_loop.rotation().x = -PI / 2.0;
        chest_loop.rotation().z = 0.2;
        chest_loop.position().set(-0.12, 0.010, -0.02);
        dl.add(&chest_loop);
        // Neck strap — smaller loop.
        let neck_loop = mesh(pg("DogLeashes__neck", torus_geo!(0.038, 0.006, 4, 12)), &harness_mat);
        neck_loop.rotation().x = -PI / 2.0;
        neck_loop.rotation().z = -0.2;
        neck_loop.position().set(-0.16, 0.008, -0.05);
        dl.add(&neck_loop);
        // 2 metal buckles connecting straps.
        for off in [[-0.11, -0.06], [-0.16, 0.0]] {
            let buckle = mesh(pg("DogLeashes__buckle", box_geo!(0.014, 0.008, 0.010)), &metal_mat);
            buckle.position().set(off[0], 0.010, off[1]);
            dl.add(&buckle);
        }
        // Small metal D-ring on chest strap for leash attachment.
        let d_ring = mesh(pg("DogLeashes__dRing", torus_geo!(0.008, 0.0015, 3, 8)), &metal_mat);
        d_ring.rotation().x = 0.0;
        d_ring.position().set(-0.06, 0.010, -0.02);
        dl.add(&d_ring);
        // 30% chance: dog tag on chest strap (small circular disc).
        if rnd() < 0.3 {
            let tag_mat = lambert!({ color: 0xc0a060 });
            let tag = mesh(pg("DogLeashes__tag", cyl_geo!(0.010, 0.010, 0.002, 12)), &tag_mat);
            tag.position().set(-0.06, 0.006, -0.014);
            dl.add(&tag);
            // Small text-line indicator.
            let text_mat = lambert!({ color: 0x101010 });
            let text_line = mesh(box_geo!(0.008, 0.001, 0.001), &text_mat);
            text_line.position().set(-0.06, 0.008, -0.014);
            dl.add(&text_line);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    dl.position().set(px, 0, pz);
    dl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&dl);
}
