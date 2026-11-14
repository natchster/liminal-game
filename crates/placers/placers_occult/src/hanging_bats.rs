//! `place_hanging_bats`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;

// Hanging bat: 5% chance per level. A hanging Halloween bat (48 cm
// across its wings): a flocked foam body on a plastic skeleton, its wings
// plastic over mesh between moulded ribs, spread as in flight, hung level by
// a cord taped to the ceiling; it turns and sways a little on its cord.
pub(crate) fn place_hanging_bats(g: &mut Game, _dist: &[i32]) {
    g.pp.bat_positions = vec![]; // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bat_y = WALL_H - 0.15;
    let bat = group();
    bat.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bat.rotation().y = rnd() * PI * 2.0;
    // modelled hanging from the ceiling (its cord's top at the origin): it
    // swings about where the cord is taped
    let hang = group();
    hang.position().y = WALL_H;
    bat.add(&hang);
    put(&hang, "Real__batWings", &phong!({ color: 0x232120, specular: 0x262626, shininess: 18, side: DoubleSide }));
    put(&hang, "Real__batBody", &lambert!({ color: 0x302b28 }));
    put(&hang, "Real__batEyes", &phong!({ color: 0x5c1e18, specular: 0x6a6a6a, shininess: 70 }));
    put(&hang, "Real__batCord", &lambert!({ color: 0x1e1d1c }));
    put(&hang, "Real__batTape", &lambert!({ color: 0xb4a684, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }));
    // Slow sway on its cord (the old draws: 60%, its phase and period).
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let hang2 = hang.clone();
        g.animate(move |_g, _dt, t| {
            let a = t * (PI * 2.0 / sway_period) + sway_phase;
            hang2.rotation().z = a.dsin() * 0.025;
            hang2.rotation().x = (a * 0.71 + 1.3).dsin() * 0.015;
            hang2.rotation().y = (a * 0.37).dsin() * 0.35;
        });
    }
    g.world.add(&bat);
    // Track for rare squeak audio.
    let bp = bat.pos();
    g.pp.bat_positions.push(v3(bp.x, bat_y, bp.z));
}
