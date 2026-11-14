//! `place_pruning_shears`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Pruning shears: 6% chance per level. 2 curved blades + 2 handles + pivot.
pub(crate) fn place_pruning_shears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0xc02020, 0xd0a040, 0x408040, 0x202020];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let shears = group();
    // Upper blade (curved via short cylinder + cone tip).
    // Lying flat, jaws and handles sprung open in the floor plane about a
    // vertical pivot bolt at (0.005, 0): the blade (-Z side, edge +Z) and
    // the hook (+Z side, edge -Z) open ±0.15 rad from their roots at the bolt,
    // their tips on their ends; the handles open ±0.2 rad from the bolt.
    let upper_blade = mesh(pg("PruningShears__blade", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    upper_blade.rotation().y = -0.15;
    upper_blade.position().set(0.005 - 0.020 * 0.15f64.dcos(), 0.0025, -0.004 - 0.020 * 0.15f64.dsin());
    shears.add(&upper_blade);
    // Upper blade tip cone (point outward, flattened to the blade's thickness).
    let upper_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    upper_tip.rotation().set(0, -0.15, -PI / 2.0);
    upper_tip.scale().x = 0.45;
    upper_tip.position().set(0.005 - 0.0485 * 0.15f64.dcos(), 0.0025, -0.004 - 0.0485 * 0.15f64.dsin());
    shears.add(&upper_tip);
    // Lower blade (matches, angled other way).
    let lower_blade = mesh(pg("PruningShears__hook", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    lower_blade.rotation().y = 0.15;
    lower_blade.position().set(0.005 - 0.020 * 0.15f64.dcos(), 0.0025, 0.004 + 0.020 * 0.15f64.dsin());
    shears.add(&lower_blade);
    let lower_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    lower_tip.rotation().set(0, 0.15, -PI / 2.0);
    lower_tip.scale().x = 0.45;
    lower_tip.position().set(0.005 - 0.0485 * 0.15f64.dcos(), 0.0025, 0.004 + 0.0485 * 0.15f64.dsin());
    shears.add(&lower_tip);
    // Pivot (rivet in middle): vertical bolt through blades and handles.
    let pivot = mesh(pg("PruningShears__pivot", cyl_geo!(0.004, 0.004, 0.020, 6)), &blade_mat);
    pivot.scale().y = 0.75;
    pivot.position().set(0.005, 0.0075, 0);
    shears.add(&pivot);
    // 2 handles extending back.
    let handle_upper = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_upper.rotation().y = 0.2;
    handle_upper.position().set(0.005 + 0.0245 * 0.2f64.dcos(), 0.007, -0.006 - 0.0245 * 0.2f64.dsin());
    shears.add(&handle_upper);
    let handle_lower = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_lower.rotation().y = -0.2;
    handle_lower.position().set(0.005 + 0.0245 * 0.2f64.dcos(), 0.007, 0.006 + 0.0245 * 0.2f64.dsin());
    shears.add(&handle_lower);
    shears.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    shears.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shears);
}
