//! `place_overturned_chair`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Overturned chair: very rare — a simple wooden chair lying on its side.
pub(crate) fn place_overturned_chair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance per level
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let pk = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x3d2818 });
    let chair = group();
    // Seat (0.42 wide × 0.04 thick × 0.42 deep).
    let seat = mesh(pg("OverturnedChair__seat", box_geo!(0.42, 0.04, 0.42)), &wood_mat);
    seat.position().y = 0.22;
    chair.add(&seat);
    // Backrest (0.42 wide × 0.50 tall × 0.04 thick).
    let back = mesh(pg("OverturnedChair__back", box_geo!(0.42, 0.50, 0.04)), &wood_mat);
    back.position().set(0, 0.47, -0.19);
    chair.add(&back);
    // 4 legs (0.04 × 0.4 × 0.04).
    for (lx, lz) in [(-0.18, -0.18), (0.18, -0.18), (-0.18, 0.18), (0.18, 0.18)] {
        let leg = mesh(pg("OverturnedChair__leg", box_geo!(0.04, 0.40, 0.04)), &wood_mat);
        leg.position().set(lx, 0.20, lz);
        chair.add(&leg);
    }
    // Tip on side: rotate Z 90° so backrest becomes horizontal.
    chair.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    // After tip, base needs to translate — approximate by nudging up.
    let cx = g.wx(pk.0) + (rnd() - 0.5) * 1.2;
    let cz = g.wz(pk.1) + (rnd() - 0.5) * 1.2;
    chair.position().set(cx, 0.21, cz);
    chair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&chair);
}
