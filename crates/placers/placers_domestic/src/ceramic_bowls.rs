//! `place_ceramic_bowls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Small ceramic bowl: 10% chance per level. Shallow wide
// cylinder with dark interior disc.
pub(crate) fn place_ceramic_bowls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let bowl_colors = [0xf0f0e8, 0x304868, 0x684030, 0x707078, 0xd0a850];
    let bowl_mat = lambert!({ color: bowl_colors[pick(bowl_colors.len())] });
    let inner_mat = lambert!({ color: 0x0a0806 });
    let bowl = group();
    // Body: shallow wide cylinder.
    let body = mesh(pg("CeramicBowls__bowl", cyl_geo!(0.06, 0.045, 0.04, 12)), &bowl_mat);
    body.position().y = 0.020;
    bowl.add(&body);
    // 40% chance liquid disc replaces empty interior.
    if rnd() < 0.40 {
        let b_liq_colors = [0x8a5030, 0xa06840, 0x605a30, 0x3a2818]; // soup/tea/water/broth
        let b_liq_mat = lambert!({ color: b_liq_colors[pick(b_liq_colors.len())] });
        let b_liq = mesh(circle_geo!(0.056, 12), &b_liq_mat);
        b_liq.rotation().x = -PI / 2.0;
        b_liq.position().y = 0.033;
        bowl.add(&b_liq);
    } else {
        let inner = mesh(circle_geo!(0.056, 12), &inner_mat);
        inner.rotation().x = -PI / 2.0;
        inner.position().y = 0.038;
        bowl.add(&inner);
    }
    bowl.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    bowl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bowl);
}
