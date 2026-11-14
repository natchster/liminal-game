//! `place_small_darts`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small dart set: 8% chance per level. 2-3 darts with
// colored fins scattered. Games family.
pub(crate) fn place_small_darts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let fin_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0];
    let body_mat = lambert!({ color: 0x606870 });
    let point_mat = lambert!({ color: 0x202028 });
    let shaft_mat = lambert!({ color: 0xf0ece0 });
    let grp = group();
    let n_darts = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_darts {
        let dart = group();
        let fin_mat = lambert!({ color: pick_color(&fin_colors), side: DoubleSide });
        // Point: sharp cone.
        let point = mesh(pg("SmallDarts__point", cone_geo!(0.003, 0.015, 6)), &point_mat);
        point.rotation().z = -PI / 2.0;
        point.position().set(0.008, 0.005, 0);
        dart.add(&point);
        // Barrel: knurled cylinder.
        let barrel = mesh(pg("SmallDarts__barrel", cyl_geo!(0.005, 0.005, 0.040, 8)), &body_mat);
        barrel.rotation().z = PI / 2.0;
        barrel.position().set(-0.020, 0.005, 0);
        dart.add(&barrel);
        // Shaft: thin white cylinder.
        let shaft = mesh(pg("SmallDarts__shaft", cyl_geo!(0.002, 0.002, 0.030, 6)), &shaft_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().set(-0.055, 0.005, 0);
        dart.add(&shaft);
        // Flight fins: 2 crossed planes at tail.
        for f in 0..2 {
            let fin = mesh(plane_geo!(0.020, 0.014), &fin_mat);
            fin.rotation().z = PI / 2.0;
            fin.rotation().x = f as f64 * PI / 2.0;
            fin.position().set(-0.080, 0.005, 0);
            dart.add(&fin);
        }
        let dx = (rnd() - 0.5) * 0.15;
        let dz = (rnd() - 0.5) * 0.10;
        dart.position().set(dx, 0, dz);
        dart.rotation().y = rnd() * PI * 2.0;
        grp.add(&dart);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}
