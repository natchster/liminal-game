//! `place_broken_pens`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Broken pen cluster: 8% chance per level. 4-8 pens
// (some snapped in half) scattered.
pub(crate) fn place_broken_pens(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let pen_colors = [0x2050c0, 0xa02040, 0x30a030, 0x181410, 0xd8b040];
    let grp = group();
    let n_pens = 4 + (rnd() * 5.0).floor() as i32;
    for _i in 0..n_pens {
        let pen_mat = lambert!({ color: choice(&pen_colors) });
        let is_broken = rnd() < 0.4;
        if is_broken {
            // 2 halves.
            for h in 0..2 {
                let hf = h as f64;
                let half = mesh(pg("BrokenPens__half", cyl_geo!(0.005, 0.005, 0.05, 5)), &pen_mat);
                half.rotation().z = PI / 2.0;
                let px = (rnd() - 0.5) * 0.20 + hf * 0.02;
                let pz = (rnd() - 0.5) * 0.15 + hf * 0.02;
                half.position().set(px, 0.005, pz);
                half.rotation().y = rnd() * PI * 2.0;
                grp.add(&half);
            }
        } else {
            let pen = mesh(pg("BrokenPens__pen", cyl_geo!(0.005, 0.005, 0.12, 5)), &pen_mat);
            pen.rotation().z = PI / 2.0;
            let px = (rnd() - 0.5) * 0.20;
            let pz = (rnd() - 0.5) * 0.15;
            pen.position().set(px, 0.005, pz);
            pen.rotation().y = rnd() * PI * 2.0;
            grp.add(&pen);
        }
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
