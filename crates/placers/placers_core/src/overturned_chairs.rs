//! `place_overturned_chairs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_y;

// Overturned wooden chair: 5% chance per level. Basic
// 4-leg chair silhouette on side.
pub(crate) fn place_overturned_chairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let wood_colors = [0x603820, 0x8a5030, 0x201a10, 0xa07050];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let chair = group();
    // Seat.
    let seat = mesh(pg("OverturnedChairs__seat", box_geo!(0.34, 0.024, 0.30)), &wood_mat);
    seat.position().y = 0.40;
    chair.add(&seat);
    // 4 legs.
    for (lx, lz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let leg = mesh(pg("OverturnedChairs__leg", cyl_geo!(0.012, 0.012, 0.40, 5)), &wood_mat);
        leg.position().set(lx * 0.14, 0.20, lz * 0.12);
        chair.add(&leg);
    }
    // Back rest bars.
    for i in 0..4 {
        let bar = mesh(pg("OverturnedChairs__spindle", cyl_geo!(0.008, 0.008, 0.28, 4)), &wood_mat);
        bar.position().set(-0.14 + i as f64 * 0.093, 0.56, -0.12);
        chair.add(&bar);
    }
    // Top rail.
    let rail = mesh(pg("OverturnedChairs__rail", box_geo!(0.30, 0.024, 0.020)), &wood_mat);
    rail.position().set(0, 0.70, -0.12);
    chair.add(&rail);
    // Tip on side: local x becomes height, so lift by the seat's half-width
    // (0.17) to rest its edge on the floor instead of half under it.
    chair.rotation().z = PI / 2.0;
    scatter_y(g, &chair, x, z, 1.6, 0.17);
    g.world.add(&chair);
}
