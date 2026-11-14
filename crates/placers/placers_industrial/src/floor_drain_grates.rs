//! `place_floor_drain_grates`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Floor drain grate: 15% chance per level, 1 drain. Round
// dark plate with cross-bar detail.
pub(crate) fn place_floor_drain_grates(g: &mut Game, _dist: &[i32]) {
    g.pp.drain_grate_positions = vec![]; // reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let grate_mat = lambert!({ color: 0x2a2620 });
    let bar_mat = lambert!({ color: 0x1a1610 });
    let drain = group();
    // Deep-void interior: very dark disc at floor level inside the drain.
    // Haunted-biome variant: faint red glow via a basic-material red tint.
    let is_haunted_drain = g.weather_key() == "haunted";
    let void_mat = basic_mat!({ color: if is_haunted_drain { 0x200404 } else { 0x040404 } });
    let void_disc = mesh(circle_geo!(0.085, 14), &void_mat);
    void_disc.rotation().x = -PI / 2.0;
    void_disc.position().y = 0.002;
    drain.add(&void_disc);
    // Plate now a ring (annulus) so the void disc is visible through the hole.
    let plate = mesh(pg("FloorDrainGrates__plate", ring_geo!(0.085, 0.11, 14)), &grate_mat);
    plate.rotation().x = -PI / 2.0;
    plate.position().y = 0.003;
    drain.add(&plate);
    // Inner rim ring.
    let rim = mesh(pg("FloorDrainGrates__rim", torus_geo!(0.10, 0.006, 4, 14)), &grate_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.006;
    drain.add(&rim);
    // 4 cross bars over the drain opening.
    for b in 0..4 {
        let bar = mesh(pg("FloorDrainGrates__bar", box_geo!(0.18, 0.006, 0.006)), &bar_mat);
        bar.rotation().y = (b as f64 / 4.0) * PI;
        bar.position().y = 0.008;
        drain.add(&bar);
    }
    drain.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    drain.rotation().y = rnd() * PI * 2.0;
    g.world.add(&drain);
    // Track drain position for gurgle audio events.
    let dp = drain.pos();
    g.pp.drain_grate_positions.push(v3(dp.x, 0.02, dp.z));
    // 15% chance: sibling drain 0.6-1.0 m away — shower-room feel.
    if rnd() < 0.15 {
        let sib_drain = clone_obj(&drain);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.6 + rnd() * 0.4;
        sib_drain.position().set(dp.x + s_ang.cos() * s_rad, 0, dp.z + s_ang.sin() * s_rad);
        sib_drain.rotation().y = rnd() * PI * 2.0;
        g.world.add(&sib_drain);
        let sp = sib_drain.pos();
        g.pp.drain_grate_positions.push(v3(sp.x, 0.02, sp.z));
    }
    // 30% chance: shallow 3D puddle overlapping the drain.
    if rnd() < 0.30 {
        let (dp_color, dp_emissive) = puddle_spec_by_weather(g.weather_key());
        let dpuddle_mat = lambert!({ color: dp_color, emissive: dp_emissive, transparent: true, opacity: 0.80 });
        let dp_r = 0.16 + rnd() * 0.10;
        let dpuddle = mesh(sphere_geo!(dp_r, 10, 5, 0, PI * 2.0, 0, PI / 2.0), &dpuddle_mat);
        dpuddle.scale().y = 0.06;
        dpuddle.position().set(dp.x, 0.001, dp.z);
        dpuddle.rotation().y = rnd() * PI * 2.0;
        g.world.add(&dpuddle);
    }
}

/// The water-puddle spec `(color, emissive)` per weather (else normal).
fn puddle_spec_by_weather(key: &str) -> (u32, u32) {
    by_weather(
        key,
        (0x1a2830, 0x08101a),
        (0x201808, 0x100804),
        (0x281010, 0x100404),
        (0x181410, 0x080604),
        (0x101820, 0x040810),
        (0x101820, 0x040810),
    )
}
