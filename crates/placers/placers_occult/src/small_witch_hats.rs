//! `place_small_witch_hats`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;

// Small witch hat: 8% chance per level. Cone with brim on
// floor. Halloween family.
pub(crate) fn place_small_witch_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let hat_colors: [u32; 3] = [0x101010, 0x201820, 0x8a2040];
    let hat_mat = lambert!({ color: pick_hex(&hat_colors) });
    let band_mat = lambert!({ color: 0xd8b040 });
    let buckle_mat = lambert!({ color: 0xd8b040 });
    let hat = group();
    // Brim: flat disc.
    let brim = mesh(pg("SmallWitchHats__brim", cyl_geo!(0.080, 0.080, 0.006, 16)), &hat_mat);
    brim.position().y = 0.003;
    hat.add(&brim);
    // Cone (tall).
    let cone = mesh(pg("SmallWitchHats__cone", cone_geo!(0.045, 0.14, 12, 1, true)), &hat_mat);
    cone.position().y = 0.076;
    hat.add(&cone);
    // Slight bent tip: nothing needed (bendable via slight rotation).
    cone.rotation().z = (rnd() - 0.5) * 0.2;
    // Tilt about the crown's base (centred on the brim, y 0.006), not its middle.
    let rz = cone.rot().z;
    cone.position().x = -rz.dsin() * 0.07;
    cone.position().y = 0.006 + rz.dcos() * 0.07;
    // Band around cone base.
    let band = mesh(pg("SmallWitchHats__band", cyl_geo!(0.045, 0.045, 0.010, 12)), &band_mat);
    band.position().y = 0.014;
    hat.add(&band);
    // Buckle.
    let buckle = mesh(pg("SmallWitchHats__buckle", box_geo!(0.014, 0.010, 0.001)), &buckle_mat);
    buckle.position().set(0, 0.014, 0.046);
    hat.add(&buckle);
    // 25% chance tipped/knocked over.
    if rnd() < 0.25 {
        hat.rotation().z = PI / 2.0;
        hat.position().y = 0.04;
    }
    hat.position().x = g.wx(x) + (rnd() - 0.5) * 2.0;
    hat.position().z = g.wz(z) + (rnd() - 0.5) * 2.0;
    hat.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hat);
}
