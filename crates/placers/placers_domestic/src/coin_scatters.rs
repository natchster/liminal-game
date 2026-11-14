//! `place_coin_scatters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Coin scatter: 10% chance per level. 5-12 loose coins on floor.
pub(crate) fn place_coin_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let coin_colors = [0xd8b040, 0xa08050, 0xc0c0c0, 0xe0a060];
    let grp = group();
    let n_coins = 5 + (rnd() * 8.0).floor() as i32;
    for _i in 0..n_coins {
        let coin_mat = lambert!({ color: pick_hex(&coin_colors) });
        let coin = mesh(pg("CoinScatters__coin", cyl_geo!(0.008, 0.008, 0.001, 10)), &coin_mat);
        coin.position().set((rnd() - 0.5) * 0.30, 0.001, (rnd() - 0.5) * 0.30);
        coin.rotation().y = rnd() * PI * 2.0;
        // 20% chance edge-on tilted.
        if rnd() < 0.20 {
            coin.rotation().z = PI / 2.0;
            coin.position().y = 0.008;
        }
        grp.add(&coin);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
