//! `place_spilled_pill_bottles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Spilled pill bottle: 8% chance per level. Tipped bottle + scattered capsules.
pub(crate) fn place_spilled_pill_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bottle_mat = basic_mat!({ color: 0xe07030, transparent: true, opacity: 0.85 });
    let cap_mat = lambert!({ color: 0xf0ece0 });
    let pill_colors = [0xf0ece0, 0xe0a0a0, 0xa0d0e0, 0xd0d0d0];
    let grp = group();
    // Bottle body (tipped on side).
    let body = mesh(cyl_geo!(0.014, 0.014, 0.045, 10), &bottle_mat);
    body.rotation().z = PI / 2.0 - 0.045; // neck end raised onto the wider cap
    body.position().set(-0.04, 0.015, 0);
    grp.add(&body);
    // White cap on side, pushed over the bottle's neck end.
    let cap = mesh(pg("SpilledPillBottles__cap", cyl_geo!(0.016, 0.016, 0.010, 10)), &cap_mat);
    cap.rotation().z = PI / 2.0;
    cap.position().set(-0.0645, 0.016, 0);
    grp.add(&cap);
    // 8-14 scattered pills (capsules).
    let n_pills = 8 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_pills {
        let pill_mat = lambert!({ color: pick_hex(&pill_colors) });
        let pill = mesh(pg("SpilledPillBottles__pill", cyl_geo!(0.003, 0.003, 0.008, 5)), &pill_mat);
        pill.rotation().z = PI / 2.0;
        pill.rotation().y = rnd() * PI * 2.0;
        pill.position().set((rnd() - 0.5) * 0.20, 0.003, (rnd() - 0.5) * 0.18);
        grp.add(&pill);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}
