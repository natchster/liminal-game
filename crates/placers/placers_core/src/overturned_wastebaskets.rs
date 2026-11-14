//! `place_overturned_wastebaskets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Overturned wastebasket: 6% chance per level. Tipped
// metal bin + 4-7 pieces of scattered trash.
pub(crate) fn place_overturned_wastebaskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let bin_colors = [0x404040, 0x8a5030, 0x304060, 0x202020];
    let bin_mat = lambert!({ color: choice(&bin_colors) });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let can_colors = [0xa02040, 0x2050c0, 0x30a030];
    let grp = group();
    // Bin body (tipped on side).
    let bin = mesh(pg("OverturnedWastebaskets__bin", cyl_geo!(0.11, 0.09, 0.25, 12, 1, true)), &bin_mat);
    bin.rotation().z = -PI / 2.0; // wide mouth (top, r 0.11) at +X toward the trash, narrow base at −X
    bin.position().set(-0.05, 0.11, 0);
    grp.add(&bin);
    // Bin bottom.
    let bin_bottom = mesh(pg("OverturnedWastebaskets__base", circle_geo!(0.09, 12)), &bin_mat);
    bin_bottom.rotation().y = -PI / 2.0;
    bin_bottom.position().set(-0.175, 0.11, 0);
    grp.add(&bin_bottom);
    // Trash pieces (crumpled paper + can).
    let n_trash = 4 + (rnd() * 4.0).floor() as i32;
    for _i in 0..n_trash {
        let is_paper = rnd() < 0.7;
        let piece;
        if is_paper {
            piece = mesh(pg("OverturnedWastebaskets__paper", sphere_geo!(0.014 + rnd() * 0.010, 6, 5)), &paper_mat);
            piece.scale().set(1, 0.7, 1.2);
        } else {
            let can_mat = lambert!({ color: choice(&can_colors) });
            piece = mesh(pg("CrushedSodaCans__can", cyl_geo!(0.014, 0.014, 0.030, 8)), &can_mat);
            piece.rotation().z = PI / 2.0;
        }
        let px = 0.05 + rnd() * 0.20;
        let py = if is_paper { 0.015 } else { 0.014 };
        let pz = (rnd() - 0.5) * 0.24;
        piece.position().set(px, py, pz);
        piece.rotation().y = rnd() * PI * 2.0;
        grp.add(&piece);
    }
    scatter(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}
