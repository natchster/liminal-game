//! `place_cigarette_packs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_xz_turned;

// Cigarette pack: 10% chance per level. Small flip-top
// box in red/blue/green colors.
pub(crate) fn place_cigarette_packs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let pack_colors = [0xa02020, 0x2050c0, 0x30a030, 0xd8b040, 0xe8dcc0];
    let pack_mat = lambert!({ color: choice(&pack_colors) });
    let foil_mat = basic_mat!({ color: 0xd0d0d0 });
    let pack = group();
    // Body.
    let body = mesh(pg("CigarettePacks__body", box_geo!(0.05, 0.08, 0.020)), &pack_mat);
    body.position().y = 0.040;
    pack.add(&body);
    // 50% chance flipped open — silver foil visible.
    if rnd() < 0.50 {
        let foil = mesh(plane_geo!(0.048, 0.018), &foil_mat);
        foil.rotation().x = -PI / 2.0;
        foil.position().set(0, 0.081, 0);
        pack.add(&foil);
        // Lid tilted back.
        let lid = mesh(pg("CigarettePacks__lid", box_geo!(0.05, 0.020, 0.004)), &pack_mat);
        lid.position().set(0, 0.086, -0.0187); // lower (hinge) end on the top back edge, z -0.010
        lid.rotation().x = -PI / 3.0;
        pack.add(&lid);
    }
    // 30% tipped on side.
    if rnd() < 0.30 {
        pack.rotation().z = PI / 2.0;
        pack.position().y = 0.025;
    }
    scatter_xz_turned(g, &pack, x, z, 2.4);
    g.world.add(&pack);
}
