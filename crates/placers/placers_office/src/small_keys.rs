//! `place_small_keys`
use game::placers::shared::by_weather;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Small key prop: 25% chance per level, 1-3 tiny gold keys on the floor.
pub(crate) fn place_small_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let key_color = key_color_by_weather(g.weather_key());
    let key_mat = lambert!({ color: key_color });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let key = group();
        // Ring at head of key.
        let ring = mesh(pg("SmallKeys__bow", torus_geo!(0.014, 0.003, 4, 10)), &key_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().x = -0.015;
        key.add(&ring);
        // Shaft.
        let shaft = mesh(pg("SmallKeys__blade", cyl_geo!(0.003, 0.003, 0.028, 5)), &key_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().x = 0.012;
        key.add(&shaft);
        // Tooth (tiny bump on shaft end): bitting in the key's horizontal plane.
        let tooth = mesh(pg("SmallKeys__bitting", box_geo!(0.008, 0.006, 0.003)), &key_mat);
        tooth.rotation().x = -PI / 2.0;
        tooth.position().set(0.019, 0, 0.005);
        key.add(&tooth);
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        key.position().set(g.wx(x) + ox, 0.003, g.wz(z) + oz);
        key.rotation().y = rnd() * PI * 2.0;
        g.world.add(&key);
        i += 1;
    }
}

/// Key colour per weather
fn key_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8a7830, 0xc8b850, 0xa88a40, 0x6a5828, 0xd0d0c8, 0xb8a850)
}
