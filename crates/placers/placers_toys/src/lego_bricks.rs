//! `place_lego_bricks`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// LEGO brick scatter: 6% chance per level. 5-9 small blocks w/ studs, some stacked.
pub(crate) fn place_lego_bricks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let brick_colors = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xe0e0e0, 0xd050a0, 0x202020];
    let cluster = group();
    let count = 5 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let brick_mat = lambert!({ color: choice(&brick_colors) });
        let brick = group();
        // Random brick size: 2x2, 2x4, or 2x3.
        let w = 0.020;
        let l = 0.020 + (rnd() * 3.0).floor() * 0.020;
        let stud_cols = 2.0;
        let stud_rows = round_half_up(l / 0.020);
        let body = mesh(pg("LegoBricks__body", box_geo!(w, 0.010, l)), &brick_mat);
        body.position().y = 0.005;
        brick.add(&body);
        // Studs on top (on the body top, 0.010).
        for r in 0..stud_rows as i32 {
            for c in 0..stud_cols as i32 {
                let stud = mesh(pg("LegoBricks__stud", cyl_geo!(0.004, 0.004, 0.003, 8)), &brick_mat);
                stud.position().set(-w / 2.0 + (c as f64 + 0.5) * (w / stud_cols), 0.0114, -l / 2.0 + (r as f64 + 0.5) * (l / stud_rows));
                brick.add(&stud);
            }
        }
        let px = (rnd() - 0.5) * 0.16;
        let stacked = rnd() < 0.3; // some stacked
        let pz = (rnd() - 0.5) * 0.16;
        let ry = rnd() * PI * 2.0;
        let below = if stacked && cluster.child_count() > 0 { cluster.children().last().cloned() } else { None };
        if let Some(below) = below {
            // On the previous brick, square to its stud grid and shifted along it by up to one stud.
            let off = round_half_up(px / 0.08) * 0.010;
            let br = below.rot();
            let bp = below.pos();
            brick.rotation().y = br.y + if ry < PI { 0.0 } else { PI / 2.0 };
            brick.position().set(bp.x + br.y.sin() * off, bp.y + 0.010, bp.z + br.y.cos() * off);
        } else {
            brick.position().set(px, 0, pz);
            brick.rotation().y = ry;
        }
        cluster.add(&brick);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
