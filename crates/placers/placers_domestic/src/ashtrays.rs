//! `place_ashtrays`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Ashtray: 12% chance per level. Small shallow disc with
// 2-3 tiny cigarette butt cylinders on top.
pub(crate) fn place_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let dish_mat = lambert!({ color: 0x484038 });
    let butt_mat = lambert!({ color: 0xd8ccb8 });
    let filter_mat = lambert!({ color: 0xa08050 });
    let ashtray = group();
    // Dish: wide shallow cylinder.
    let dish = mesh(pg("Ashtrays__dish", cyl_geo!(0.06, 0.055, 0.02, 10)), &dish_mat);
    dish.position().y = 0.010;
    ashtray.add(&dish);
    // Rim slightly darker interior.
    let rim = mesh(pg("Ashtrays__rim", torus_geo!(0.056, 0.004, 4, 12)), &dish_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.020;
    ashtray.add(&rim);
    // 2-3 tiny butt cylinders + filter caps at random positions.
    let butt_count = 2 + (rnd() * 2.0).floor() as i32;
    for _i in 0..butt_count {
        let b_ang = rnd() * PI * 2.0;
        let b_rad = rnd() * 0.03;
        let butt = mesh(pg("Ashtrays__butt", cyl_geo!(0.005, 0.005, 0.025, 5)), &butt_mat);
        butt.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.4;
        butt.rotation().y = rnd() * PI;
        butt.position().set(b_ang.dcos() * b_rad, 0.023, b_ang.dsin() * b_rad);
        ashtray.add(&butt);
        // Tiny brown filter cap at one end.
        let filter = mesh(pg("Ashtrays__filter", cyl_geo!(0.005, 0.005, 0.008, 5)), &filter_mat);
        let br = butt.rot();
        filter.rotation().copy(&br);
        // On the butt's -Y end (its axis after Rz then Ry), away from the lit end.
        let (tz, ty) = (br.z - PI / 2.0, br.y);
        let bp = butt.pos();
        filter.position().set(bp.x + tz.dcos() * ty.dcos() * 0.016, bp.y + tz.dsin() * 0.016, bp.z - tz.dcos() * ty.dsin() * 0.016);
        ashtray.add(&filter);
    }
    ashtray.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    ashtray.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ashtray);
}
