//! `place_hairbrushes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Hairbrush w/ tangled hair strands: 7% chance per level.
// Oval brush w/ bristles + optional wisp of dark hair.
pub(crate) fn place_hairbrushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Brush body color pool.
    let body_colors: [u32; 6] = [0x101010, 0xa02040, 0x6040a0, 0xd0a040, 0xe0c0e0, 0x606060];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let bristle_mat = lambert!({ color: 0x0a0a0a });
    // Hair strand color pool.
    let hair_colors: [u32; 5] = [0x1a1010, 0x402010, 0x604020, 0xa07040, 0xd0a060];
    let hair_mat = lambert!({ color: pick(&hair_colors) });
    let brush = group();
    // Oval brush pad (paddle) — flat ellipse via scaled cylinder.
    let pad = mesh(pg("Hairbrushes__pad", cyl_geo!(0.045, 0.045, 0.014, 16)), &body_mat);
    // Lies flat, cushion up (y 0.007-0.021); oval along Z like the bristle grid.
    pad.scale().z = 1.4; // Elongate to oval.
    pad.position().set(0, 0.014, 0);
    brush.add(&pad);
    // Handle — cylinder extending out.
    let handle = mesh(pg("Hairbrushes__handle", cyl_geo!(0.012, 0.015, 0.10, 8)), &body_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.093, 0.014, 0); // near end 2 mm into the pad rim (x -0.045)
    brush.add(&handle);
    // Handle knob at end.
    let knob = mesh(pg("Hairbrushes__knob", sphere_geo!(0.016, 6, 5)), &body_mat);
    knob.scale().set_scalar(0.875); // r 14 mm: rests on the floor, on the handle axis
    knob.position().set(-0.155, 0.014, 0);
    brush.add(&knob);
    // Bristles — grid of tiny cylinders on top of pad (5x7 grid = 35).
    for iy in -3..=3 {
        for ix in -2..=2 {
            let bx = ix as f64 * 0.014;
            let by = iy as f64 * 0.014;
            // Reject corners for oval mask.
            if (bx * bx * 1.4 + by * by * 0.7) > 0.045 * 0.045 * 1.15 {
                continue;
            }
            let bristle = mesh(pg("Hairbrushes__bristle", cyl_geo!(0.001, 0.001, 0.012, 3)), &bristle_mat);
            bristle.position().set(bx, 0.026, by);
            brush.add(&bristle);
        }
    }
    // 60% chance: tangled hair strands caught in bristles.
    if rnd() < 0.6 {
        // Use an oriented-segment-chain pattern to draw curly hair.
        let strand_count = 3 + (rnd() * 3.0).floor() as i32;
        for _s in 0..strand_count {
            // Start point near random spot on pad top.
            let ang_start = rnd() * PI * 2.0;
            let r_start = rnd() * 0.035;
            let mut prev = v3(ang_start.dcos() * r_start, 0.030, ang_start.dsin() * r_start * 1.3);
            // Chain 6 segments arcing outward from pad.
            let mut dir_ang = ang_start + (rnd() - 0.5) * 1.5;
            for i in 0..6 {
                let step = 0.008 + rnd() * 0.008;
                dir_ang += (rnd() - 0.5) * 1.2; // Wander.
                let mut next = prev;
                next.add(&v3(dir_ang.dcos() * step, -0.001 * i as f64, dir_ang.dsin() * step));
                let dir_len = next.distance_to(&prev);
                let mut mid = prev;
                mid.add(&next).multiply_scalar(0.5);
                let seg = mesh(cyl_geo!(0.0008, 0.0008, dir_len, 3), &hair_mat);
                seg.position().copy(&mid);
                let mut v = next;
                v.sub(&prev).normalize();
                seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
                brush.add(&seg);
                prev = next;
            }
        }
    }
    brush.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}
