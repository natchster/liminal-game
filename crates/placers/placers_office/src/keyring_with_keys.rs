//! `place_keyring_with_keys`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Keyring w/ keys: 6% chance per level. Metal ring + 2-3 keys.
pub(crate) fn place_keyring_with_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let ring_mat = lambert!({ color: 0xc0c0c0 });
    let key_colors = [0xd0a040, 0xb08830, 0xa0a0a0, 0x808080];
    let ring = group();
    let r = 0.020;
    let lp = mesh(pg("KeyringWithKeys__ring", torus_geo!(r, 0.003, 4, 12)), &ring_mat);
    lp.rotation().x = PI / 2.0;
    lp.position().y = 0.003;
    ring.add(&lp);
    let key_count = 2 + (rnd() * 2.0).floor() as i32;
    for k in 0..key_count {
        let key_mat = lambert!({ color: choice(&key_colors) });
        let key = group();
        let bow = mesh(pg("KeyringWithKeys__bow", cyl_geo!(0.008, 0.008, 0.002, 8)), &key_mat);
        bow.position().set(0, 0.001, 0);
        key.add(&bow);
        let shaft = mesh(pg("KeyringWithKeys__shaft", box_geo!(0.038, 0.002, 0.005)), &key_mat);
        shaft.position().set(0.023, 0.001, 0);
        key.add(&shaft);
        for t in 0..2 {
            let tooth = mesh(pg("KeyringWithKeys__tooth", box_geo!(0.006, 0.002, 0.003)), &key_mat);
            tooth.position().set(0.030 + t as f64 * 0.008, 0.001, 0.004);
            key.add(&tooth);
        }
        let angle = (k as f64 / key_count as f64) * PI * 0.6 - PI * 0.3;
        key.position().set(angle.dcos() * (r + 0.010), 0, angle.dsin() * (r + 0.010));
        key.rotation().y = angle;
        ring.add(&key);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    ring.position().set(px, 0, pz);
    ring.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ring);
}
