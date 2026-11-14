//! `place_coin_clusters`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dropped coin cluster: 8% chance per level. 3-6 coins, some face-up + some on edge.
pub(crate) fn place_coin_clusters(g: &mut Game, _dist: &[i32]) {
    g.pp.coin_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let coin_colors = [0xd0a040, 0xc0c0c0, 0xa06030, 0xb0b0b0, 0xc0a860];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let coin_mat = lambert!({ color: choice(&coin_colors) });
        // Coin: flat cylinder. Size varies.
        let r = 0.008 + rnd() * 0.005;
        let coin = mesh(pg("CoinScatters__coin", cyl_geo!(r, r, 0.0015, 12)), &coin_mat);
        // 20% chance on edge (rolled).
        if rnd() < 0.20 {
            coin.rotation().x = PI / 2.0;
            coin.rotation().z = rnd() * PI * 2.0;
            coin.position().set((rnd() - 0.5) * 0.14, r, (rnd() - 0.5) * 0.14);
        } else {
            coin.position().set((rnd() - 0.5) * 0.14, 0.001, (rnd() - 0.5) * 0.14);
        }
        cluster.add(&coin);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
    let p = cluster.pos();
    g.pp.coin_positions.push(v3(p.x, 0, p.z)); // track for tink audio
}
