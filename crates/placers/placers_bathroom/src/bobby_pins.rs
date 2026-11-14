//! `place_bobby_pins`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Bobby pin cluster: 9% chance per level. 4-8 tiny U-shaped hair pins.
pub(crate) fn place_bobby_pins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let pin_colors = [0x202020, 0x605040, 0xa08060, 0x904030];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let pin_mat = lambert!({ color: choice(&pin_colors) });
        let pin = group();
        let prong1 = mesh(pg("BobbyPins__prong", cyl_geo!(0.0007, 0.0007, 0.024, 3)), &pin_mat);
        prong1.rotation().z = PI / 2.0;
        prong1.position().set(0, 0.001, -0.002);
        pin.add(&prong1);
        let prong2 = mesh(pg("BobbyPins__wavy", cyl_geo!(0.0007, 0.0007, 0.024, 3)), &pin_mat);
        prong2.rotation().z = PI / 2.0;
        prong2.position().set(0, 0.001, 0.002);
        pin.add(&prong2);
        // Curved end (half-torus), lying flat, bulging towards -X.
        let end = mesh(pg("BobbyPins__bend", torus_geo!(0.002, 0.0007, 3, 6, PI)), &pin_mat);
        end.rotation().x = PI / 2.0;
        end.rotation().z = PI / 2.0;
        end.position().set(-0.012, 0.001, 0);
        pin.add(&end);
        pin.position().set((rnd() - 0.5) * 0.13, 0, (rnd() - 0.5) * 0.13);
        pin.rotation().y = rnd() * PI * 2.0;
        cluster.add(&pin);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
