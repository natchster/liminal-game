//! `place_matchbooks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Matchbook: 8% chance per level. Small booklet + 4-6
// scattered matches.
pub(crate) fn place_matchbooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let cover_colors = [0xa02040, 0x2050a0, 0x30a030, 0xd8b040];
    let cover_mat = lambert!({ color: choice(&cover_colors) });
    let match_stick_mat = lambert!({ color: 0xe8dcc0 });
    let match_head_mat = lambert!({ color: 0x8a3020 });
    let grp = group();
    // Booklet body.
    let booklet = mesh(pg("Matchbooks__booklet", box_geo!(0.030, 0.005, 0.040)), &cover_mat);
    booklet.position().y = 0.003;
    grp.add(&booklet);
    // 5 scattered matches.
    let n_matches = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_matches {
        let m = group();
        let stick = mesh(pg("Matchbooks__stick", cyl_geo!(0.0008, 0.0008, 0.024, 4)), &match_stick_mat);
        stick.rotation().z = PI / 2.0;
        stick.position().x = 0.012;
        m.add(&stick);
        let head = mesh(pg("Matchbooks__head", sphere_geo!(0.0015, 5, 4)), &match_head_mat);
        head.position().x = 0.024;
        m.add(&head);
        let px = (rnd() - 0.5) * 0.12;
        let pz = (rnd() - 0.5) * 0.08;
        m.position().set(px, 0.0012, pz);
        m.rotation().y = rnd() * PI * 2.0;
        grp.add(&m);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
