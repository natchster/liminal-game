//! `place_dead_potted_plants`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Dead potted plant: 15% chance per level. Small pot with
// 2-3 dark bent stems.
pub(crate) fn place_dead_potted_plants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // Per-weather pot color pool.
    let pot_pool = dead_plant_pot_pool(g.weather_key());
    let pot_mat = lambert!({ color: pot_pool[pick(pot_pool.len())] });
    let stem_mat = lambert!({ color: 0x2a1810 });
    let soil_mat = lambert!({ color: 0x1a0e08 });
    let plant = group();
    // Pot: tapered cylinder.
    let pot = mesh(pg("DeadPottedPlants__pot", cyl_geo!(0.10, 0.08, 0.14, 10)), &pot_mat);
    pot.position().y = 0.07;
    plant.add(&pot);
    // Soil: dark disc at pot rim.
    let soil = mesh(pg("DeadPottedPlants__soil", circle_geo!(0.09, 10)), &soil_mat);
    soil.rotation().x = -PI / 2.0;
    soil.position().y = 0.14;
    plant.add(&soil);
    // 2-3 dead bent stems poking up from soil.
    let stem_count = 2 + (rnd() * 2.0).floor() as i32;
    for _s in 0..stem_count {
        let stem = mesh(pg("DeadPottedPlants__stem", cyl_geo!(0.004, 0.004, 0.22, 4)), &stem_mat);
        let bend_ang = (rnd() - 0.5) * 0.7; // ~40° max bend
        stem.rotation().z = bend_ang;
        stem.rotation().y = rnd() * PI * 2.0;
        let stem_ang = rnd() * PI * 2.0;
        let stem_rad = rnd() * 0.04;
        stem.position().set(stem_ang.cos() * stem_rad, 0.24, stem_ang.sin() * stem_rad);
        plant.add(&stem);
    }
    plant.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
}

/// Dead plant pot pool per weather; other keys get `[0x6a4028, 0x483028, 0x503820]`
fn dead_plant_pot_pool(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x3a4048, 0x2a3038, 0x4a5058],      // damp gray-blue clay
        "electric" => &[0x4a3020, 0x603828, 0x6a4028], // scorched terracotta
        "haunted" => &[0x3a1010, 0x2a0808, 0x4a1010],  // bloody dark red clay
        "derelict" => &[0x5a4830, 0x483828, 0x584030], // aged tan clay
        _ => &[0x6a4028, 0x483028, 0x503820],
    }
}
