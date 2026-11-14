//! `place_overflowing_ashtrays`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_on_floor;

// Overflowing ashtray: 6% chance per level. Ashtray + scattered butts.
pub(crate) fn place_overflowing_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let tray_mat = lambert!({ color: 0x604838 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let filter_mat = lambert!({ color: 0xa08040 });
    let ash_mat = lambert!({ color: 0x2a2418 });
    let grp = group();
    // Tray body: shallow dish.
    let tray = mesh(pg("OverflowingAshtrays__tray", cyl_geo!(0.050, 0.040, 0.015, 12)), &tray_mat);
    tray.position().y = 0.0075;
    grp.add(&tray);
    // Ash pile in center.
    let ash_pile = mesh(pg("OverflowingAshtrays__ash", cone_geo!(0.030, 0.015, 8)), &ash_mat);
    ash_pile.position().y = 0.020;
    grp.add(&ash_pile);
    // 10-16 butts scattered inside + around tray.
    let n_butts = 10 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_butts {
        // Paper part (white).
        let paper = mesh(pg("OverflowingAshtrays__paper", cyl_geo!(0.002, 0.002, 0.014, 4)), &paper_mat);
        paper.rotation().z = PI / 2.0;
        // Filter (tan).
        let filter = mesh(pg("OverflowingAshtrays__filter", cyl_geo!(0.002, 0.002, 0.006, 4)), &filter_mat);
        filter.rotation().z = PI / 2.0;
        // Position: some inside tray, some around it.
        let in_tray = (i as f64) < n_butts as f64 * 0.6;
        let r = if in_tray { rnd() * 0.035 } else { 0.06 + rnd() * 0.15 };
        let a = rnd() * PI * 2.0;
        let px = a.cos() * r;
        let pz = a.sin() * r;
        let py = if in_tray { 0.018 } else { 0.002 };
        let ang = rnd() * PI * 2.0;
        paper.position().set(px, py, pz);
        paper.rotation().y = ang;
        // on the paper's axis: rotation.y = ang lays the tube along (cos, 0, -sin)
        filter.position().set(px + ang.cos() * 0.010, py, pz - ang.sin() * 0.010);
        filter.rotation().y = ang;
        grp.add(&paper);
        grp.add(&filter);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}
