//! `place_alarm_bells`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Broken alarm bell: 4% chance per level. Wall bell dome + dangling clapper.
pub(crate) fn place_alarm_bells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let plate_mat = lambert!({ color: 0x6a3820 });
    let bell_mat = lambert!({ color: 0xc02818 });
    let clapper_mat = lambert!({ color: 0x181410 });
    let bell = group();
    // Back plate.
    let plate = mesh(pg("AlarmBells__plate", box_geo!(0.12, 0.16, 0.014)), &plate_mat);
    plate.position().set(0, 0, -0.007);
    bell.add(&plate);
    // Bell dome (half-sphere).
    let dome = mesh(pg("AlarmBells__gong", sphere_geo!(0.055, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &bell_mat);
    dome.position().set(0, 0.020, 0.020);
    dome.rotation().x = PI / 2.0; // gong faces the room, apex at the nut
    bell.add(&dome);
    // Bell top small button.
    let top = mesh(pg("AlarmBells__nut", cyl_geo!(0.008, 0.008, 0.008, 6)), &bell_mat);
    top.rotation().x = PI / 2.0;
    top.position().set(0, 0.020, 0.075);
    bell.add(&top);
    // Dangling clapper (rusted, off-center).
    let clapper_pivot = group();
    clapper_pivot.position().set(0, 0.020, 0.080); // hangs from the centre nut
    let clapper_arm = mesh(pg("AlarmBells__arm", cyl_geo!(0.001, 0.001, 0.04, 3)), &clapper_mat);
    clapper_arm.position().y = -0.020;
    clapper_pivot.add(&clapper_arm);
    let ball = mesh(pg("AlarmBells__ball", sphere_geo!(0.006, 6, 5)), &clapper_mat);
    ball.position().y = -0.042;
    clapper_pivot.add(&ball);
    clapper_pivot.rotation().z = 0.4; // hanging tilted, broken
    bell.add(&clapper_pivot);
    mount_on_wall(g, &bell, &c, 0.007, 2.20, 0.6);
    g.world.add(&bell);
}
