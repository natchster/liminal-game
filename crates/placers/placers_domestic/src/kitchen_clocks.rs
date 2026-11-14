//! `place_kitchen_clocks`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Kitchen wall clock: 4% chance per level. Round face
// w/ 4 tick marks + hands.
pub(crate) fn place_kitchen_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.kitchen_clock_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let rim_colors: [u32; 5] = [0xa02020, 0xd0a040, 0x2060a0, 0x606060, 0x408060];
    let rim_mat = lambert!({ color: pick(&rim_colors) });
    let face_mat = lambert!({ color: 0xf8f4e8 });
    let hand_mat = lambert!({ color: 0x101010 });
    let clock = group();
    let mount_y = 1.95;
    let rim = mesh(pg("KitchenClocks__case", cyl_geo!(0.11, 0.11, 0.024, 20)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    clock.add(&rim);
    let face = mesh(circle_geo!(0.105, 20), &face_mat);
    face.position().set(0, mount_y, 0.013);
    clock.add(&face);
    // 4 tick marks at cardinal positions.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 - PI / 2.0;
        let tick = mesh(pg("KitchenClocks__tick", box_geo!(0.006, 0.020, 0.003)), &hand_mat);
        tick.position().set(a.dcos() * 0.090, mount_y + a.dsin() * 0.090, 0.014);
        tick.rotation().z = a + PI / 2.0; // radial baton (was tangential)
        clock.add(&tick);
    }
    // Frozen hands, each turned about the hub (pivot group), not its own centre.
    let hour = mesh(pg("KitchenClocks__hour", box_geo!(0.006, 0.050, 0.003)), &hand_mat);
    let hour_pivot = group();
    hour_pivot.position().set(0, mount_y, 0.016);
    hour.position().set(0, 0.025, 0);
    hour_pivot.rotation().z = rnd() * PI * 2.0;
    hour_pivot.add(&hour);
    clock.add(&hour_pivot);
    let minute = mesh(pg("KitchenClocks__minute", box_geo!(0.004, 0.075, 0.003)), &hand_mat);
    let minute_pivot = group();
    minute_pivot.position().set(0, mount_y, 0.017);
    minute.position().set(0, 0.037, 0);
    minute_pivot.rotation().z = rnd() * PI * 2.0;
    minute_pivot.add(&minute);
    clock.add(&minute_pivot);
    let center = mesh(circle_geo!(0.006, 8), &hand_mat);
    center.position().set(0, mount_y, 0.018);
    clock.add(&center);
    clock.rotation().y = face_wall(dx, dz);
    clock.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&clock);
    let p = clock.pos();
    g.pp.kitchen_clock_positions.push(v3(p.x, 0, p.z));
}
