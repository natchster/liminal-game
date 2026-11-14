//! `place_lipsticks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Lipstick tube: 9% chance per level. Slim cylinder w/
// exposed lipstick nub in optional variants.
pub(crate) fn place_lipsticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Tube color pool — metallics.
    let tube_colors: [u32; 5] = [0x101010, 0xc09040, 0xa02040, 0x606060, 0xf0d0e0];
    let tube_mat = lambert!({ color: pick(&tube_colors) });
    // Lipstick color pool — vivid.
    let stick_colors: [u32; 5] = [0xa02020, 0x801020, 0xc04070, 0x603040, 0xd06040];
    let stick_mat = lambert!({ color: pick(&stick_colors) });
    let lip = group();
    // Main tube body.
    let body = mesh(pg("Lipsticks__body", cyl_geo!(0.012, 0.012, 0.055, 10)), &tube_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.012, 0);
    lip.add(&body);
    // Decorative band ring at midpoint.
    let band_mat = lambert!({ color: 0xd0b060 });
    let band = mesh(pg("Lipsticks__band", torus_geo!(0.0125, 0.0025, 4, 10)), &band_mat);
    band.rotation().y = PI / 2.0;
    band.position().set(0, 0.012, 0);
    lip.add(&band);
    // 60% chance: cap is off + lipstick extended.
    if rnd() < 0.6 {
        // Base tube (shorter, cap removed).
        // Twist ring — small torus at exposed end.
        let twist_ring = mesh(pg("Lipsticks__twist", torus_geo!(0.012, 0.003, 4, 10)), &tube_mat);
        twist_ring.rotation().y = PI / 2.0;
        twist_ring.position().set(0.028, 0.012, 0);
        lip.add(&twist_ring);
        // Extended lipstick nub — slanted cone.
        let stick = mesh(pg("Lipsticks__stick", cyl_geo!(0.008, 0.010, 0.024, 8)), &stick_mat);
        stick.rotation().z = PI / 2.0;
        stick.position().set(0.045, 0.012, 0);
        lip.add(&stick);
        // Angled cut at tip (small dark oval).
        let tip = mesh(circle_geo!(0.009, 8), &stick_mat);
        tip.rotation().y = PI / 2.0;
        tip.position().set(0.057, 0.012, 0);
        tip.rotation().x = 0.5;
        lip.add(&tip);
        // Loose cap lying beside — cylinder tilted.
        let cap = mesh(pg("Lipsticks__cap", cyl_geo!(0.014, 0.014, 0.030, 10)), &tube_mat);
        // Lying on its side on the floor, turned 0.4 rad (it stood tilted 0.4 rad, rim 5 mm into the floor).
        cap.rotation().set(0, 0.4, PI / 2.0);
        cap.position().set(-0.05, 0.014, 0.03);
        lip.add(&cap);
    } else {
        // Cap still on — extend body length.
        let cap = mesh(pg("Lipsticks__cap", cyl_geo!(0.014, 0.014, 0.030, 10)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.043, 0.012, 0);
        lip.add(&cap);
        let cap_top = mesh(circle_geo!(0.014, 10), &tube_mat);
        cap_top.rotation().y = PI / 2.0;
        cap_top.position().set(0.058, 0.012, 0);
        lip.add(&cap_top);
    }
    lip.position().set(g.wx(x) + (rnd() - 0.5) * 2.6, 0, g.wz(z) + (rnd() - 0.5) * 2.6);
    lip.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lip);
}
