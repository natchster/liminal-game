//! `place_shaker_pair`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Salt shaker + pepper mill pair: 6% chance per level.
pub(crate) fn place_shaker_pair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_mat = lambert!({ color: 0xffffff, transparent: true, opacity: 0.75 });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let cap_mat = lambert!({ color: 0xc0c0c0 });
    let salt_mat = lambert!({ color: 0xf8f4e8 });
    let pepper_mat = lambert!({ color: 0x202018 });
    let pair = group();
    let salt_body = mesh(cyl_geo!(0.022, 0.022, 0.058, 10), &glass_mat);
    salt_body.position().set(-0.028, 0.029, 0);
    pair.add(&salt_body);
    let salt_fill = mesh(pg("ShakerPair__salt", cyl_geo!(0.020, 0.020, 0.030, 10)), &salt_mat);
    salt_fill.position().set(-0.028, 0.017, 0);
    pair.add(&salt_fill);
    let salt_cap = mesh(pg("ShakerPair__cap", cyl_geo!(0.024, 0.022, 0.014, 10)), &cap_mat);
    salt_cap.position().set(-0.028, 0.065, 0);
    pair.add(&salt_cap);
    for i in 0..5 {
        let a = (i as f64 / 5.0) * PI * 2.0;
        let hole = mesh(circle_geo!(0.002, 4), lambert!({ color: 0x101010 }));
        hole.rotation().x = -PI / 2.0;
        hole.position().set(-0.028 + a.dcos() * 0.012, 0.073, a.dsin() * 0.012);
        pair.add(&hole);
    }
    let mill_body = mesh(pg("ShakerPair__mill", cyl_geo!(0.020, 0.024, 0.100, 10)), &wood_mat);
    mill_body.position().set(0.028, 0.050, 0);
    pair.add(&mill_body);
    let pepper_fill = mesh(cyl_geo!(0.017, 0.017, 0.030, 8), &pepper_mat);
    pepper_fill.position().set(0.028, 0.020, 0);
    pair.add(&pepper_fill);
    let mill_knob = mesh(pg("ShakerPair__knob", cyl_geo!(0.008, 0.008, 0.014, 8)), &cap_mat);
    mill_knob.position().set(0.028, 0.107, 0);
    pair.add(&mill_knob);
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}
