//! `place_wall_doors`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall door with peephole: 4% chance per level. Wooden
// door plane + handle + peephole disc.
pub(crate) fn place_wall_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let door_colors = [0x603820, 0x8a5030, 0x201a10, 0xa02040];
    let door_mat = lambert!({ color: choice(&door_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = basic_mat!({ color: 0x080604 });
    let door = group();
    // Door slab.
    let slab = mesh(pg("WallDoors__slab", box_geo!(0.70, 1.90, 0.030)), &door_mat);
    slab.position().set(0, 0, -0.015);
    door.add(&slab);
    // 2 panels (inset rectangles).
    for i in 0..2 {
        let panel = mesh(pg("WallDoors__panel", box_geo!(0.50, 0.60, 0.010)), &door_mat);
        panel.position().set(0, -0.35 + i as f64 * 0.70, 0.001);
        door.add(&panel);
    }
    // Handle.
    let handle = mesh(pg("WallDoors__knob", cyl_geo!(0.014, 0.014, 0.030, 6)), &brass_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.28, 0.05, 0.010);
    door.add(&handle);
    // Peephole.
    let peephole = mesh(circle_geo!(0.006, 8), &dark_mat);
    peephole.position().set(0, 0.65, 0.002);
    door.add(&peephole);
    let (wall_x, wall_z) = wall_xz(g, c, 0.020);
    mount(&door, c, wall_x, wall_z, 0.95, None);
    g.world.add(&door);
}
