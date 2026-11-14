//! `place_small_hats`: Small hat: 10% chance per level. Cylinder crown + flat brim.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small hat: 10% chance per level. Cylinder crown + flat brim.
pub(crate) fn place_small_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hat_colors = [0x181410, 0x4a3018, 0x2a2418, 0x604828, 0x681818];
    let hat_mat = lambert!({ color: pick(&hat_colors) });
    let band_mat = lambert!({ color: 0x2a2418 });
    let hat = group();
    // Brim: wider flat disc.
    let brim = mesh(pg("SmallHats__brim", cyl_geo!(0.10, 0.10, 0.008, 14)), &hat_mat);
    brim.position().y = 0.004;
    hat.add(&brim);
    // Crown: taller narrower cylinder.
    let crown = mesh(pg("SmallHats__crown", cyl_geo!(0.055, 0.055, 0.08, 12)), &hat_mat);
    crown.position().y = 0.048;
    hat.add(&crown);
    // Small band ring around crown base.
    let band = mesh(pg("SmallHats__band", torus_geo!(0.055, 0.005, 4, 12)), &band_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.014;
    hat.add(&band);
    hat.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hat.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped on side (brim flat instead of vertical).
    if rnd() < 0.25 {
        hat.rotation().z = PI / 2.0;
        hat.position().y = 0.10;
    }
    g.world.add(&hat);
}
