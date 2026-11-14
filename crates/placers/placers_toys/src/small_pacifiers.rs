//! `place_small_pacifiers`: Small pacifier: 8% chance per level. Nipple cylinder + guard disc + ring handle.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small pacifier: 8% chance per level. Nipple cylinder +
/// guard disc + ring handle.
pub(crate) fn place_small_pacifiers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let guard_colors = [0xd8a0c0, 0xa0c0d8, 0xd8d0a0, 0xf0f0f0];
    let guard_mat = lambert!({ color: pick(&guard_colors) });
    let nip_mat = lambert!({ color: 0xe8dcc0 });
    let paci = group();
    // Nipple: soft small cylinder.
    let nipple = mesh(pg("SmallPacifiers__nipple", cyl_geo!(0.008, 0.010, 0.020, 6)), &nip_mat);
    nipple.rotation().z = PI / 2.0;
    nipple.position().set(0.015, 0.014, 0);
    paci.add(&nipple);
    // Guard disc: shield behind nipple.
    let guard = mesh(pg("SmallPacifiers__shield", cyl_geo!(0.020, 0.020, 0.005, 10)), &guard_mat);
    guard.rotation().z = PI / 2.0;
    guard.position().set(0.005, 0.014, 0);
    paci.add(&guard);
    // Ring handle behind guard (half-torus).
    let ring = mesh(pg("SmallPacifiers__ring", torus_geo!(0.010, 0.002, 4, 10)), &guard_mat);
    ring.position().set(-0.008, 0.014, 0);
    paci.add(&ring);
    paci.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    paci.rotation().y = rnd() * PI * 2.0;
    g.world.add(&paci);
}
