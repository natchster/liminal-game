//! `place_small_crayons`: Small crayon: 10% chance per level. Thin colored cylinder + small pointed cone tip.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small crayon: 10% chance per level. Thin colored cylinder +
/// small pointed cone tip.
pub(crate) fn place_small_crayons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crayon_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8, 0xd05038, 0xd82060];
    let c_col = pick(&crayon_colors);
    let cray_mat = lambert!({ color: c_col });
    let crayon = group();
    // Body: thin cylinder.
    let body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &cray_mat);
    body.rotation().z = PI / 2.0;
    crayon.add(&body);
    // Pointed tip: cone at one end.
    let tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &cray_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().x = 0.037;
    crayon.add(&tip);
    crayon.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.008, g.wz(z) + (rnd() - 0.5) * 2.4);
    crayon.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crayon);
    // 30% chance: 1-3 sibling crayons scattered nearby.
    if rnd() < 0.30 {
        let sib_count = 1 + (rnd() * 3.0).floor() as i32;
        for _ in 0..sib_count {
            let sib_col = pick(&crayon_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_crayon = group();
            let sib_body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &sib_mat);
            sib_body.rotation().z = PI / 2.0;
            sib_crayon.add(&sib_body);
            let sib_tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &sib_mat);
            sib_tip.rotation().z = -PI / 2.0;
            sib_tip.position().x = 0.037;
            sib_crayon.add(&sib_tip);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.06 + rnd() * 0.12;
            let cp = crayon.pos();
            sib_crayon.position().set(cp.x + s_ang.dcos() * s_rad, 0.008, cp.z + s_ang.dsin() * s_rad);
            sib_crayon.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_crayon);
        }
    }
}
