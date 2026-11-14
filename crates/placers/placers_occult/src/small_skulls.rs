//! `place_small_skulls`
use game::prelude::*;
use placers_kit::geometry::tri_geo;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;

// Small skull: 5% chance per level. Bone skull with 2 eye
// sockets + nose gap + teeth. Halloween family.
pub(crate) fn place_small_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bone_mat = lambert!({ color: 0xe8dcc0 });
    let socket_mat = basic_mat!({ color: 0x080604 });
    let skull = group();
    // Cranium (main head sphere).
    let cranium = mesh(pg("SmallSkulls__cranium", sphere_geo!(0.045, 12, 10)), &bone_mat);
    cranium.scale().set(1, 1, 1.1);
    cranium.position().y = 0.045;
    skull.add(&cranium);
    // Jaw (smaller flatter cylinder attached below cranium).
    let jaw = mesh(pg("SmallSkulls__jaw", box_geo!(0.055, 0.014, 0.040)), &bone_mat);
    jaw.position().set(0, 0.014, 0.010);
    skull.add(&jaw);
    // 2 eye sockets (dark discs on front).
    for s in [-1.0, 1.0] {
        let socket = mesh(circle_geo!(0.010, 8), &socket_mat);
        socket.position().set(s * 0.014, 0.052, 0.040);
        skull.add(&socket);
    }
    // Nose triangle (small dark).
    let nose_geo = tri_geo([-0.006, 0.038, 0.045, 0.006, 0.038, 0.045, 0.0, 0.024, 0.045]);
    let nose = mesh(nose_geo, &socket_mat);
    skull.add(&nose);
    // Teeth: small bump grid (5 bumps).
    // Set on the jaw's top edge (0.021), following its horseshoe front
    // (z 0.0242 at the chin curving back to ~0.019 at x ±0.014), each tooth
    // turned to face out of the curve.
    for i in 0..5 {
        let tooth = mesh(pg("SmallSkulls__tooth", box_geo!(0.005, 0.006, 0.002)), &bone_mat);
        let tx = -0.014 + i as f64 * 0.007;
        tooth.position().set(tx, 0.0225, 0.0242 - 0.00545 * (tx / 0.0148).powi(2));
        tooth.rotation().y = (2.0 * 0.00545 * tx / (0.0148 * 0.0148)).atan();
        skull.add(&tooth);
    }
    skull.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    skull.rotation().y = rnd() * PI * 2.0;
    // Tilted to the side sometimes.
    if rnd() < 0.30 {
        let sgn = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let tilt = sgn * (PI / 6.0 + rnd() * PI / 4.0);
        skull.rotation().z = tilt;
    }
    g.world.add(&skull);
}
