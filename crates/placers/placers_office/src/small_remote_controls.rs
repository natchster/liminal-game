//! `place_small_remote_controls`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small remote control: 10% chance per level. Rectangular
// remote with buttons + antenna. Household family.
pub(crate) fn place_small_remote_controls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x181008, 0x201a10, 0x606870, 0xc8c8c8];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let button_colors = [0xd82040, 0x2050d8, 0x30a030, 0xd8b040, 0xf0ece0];
    let remote = group();
    // Body: elongated box.
    let body = mesh(pg("SmallRemoteControls__body", box_geo!(0.045, 0.010, 0.15)), &body_mat);
    body.position().y = 0.005;
    remote.add(&body);
    // 4×2 grid of button pips on top face.
    for bx in 0..4usize {
        for bz in 0..2usize {
            let button_mat = lambert!({ color: button_colors[(bx + bz) % button_colors.len()] });
            let btn = mesh(pg("SmallRemoteControls__button", cyl_geo!(0.004, 0.004, 0.002, 6)), &button_mat);
            btn.position().set(-0.014 + bx as f64 * 0.009, 0.011, -0.030 + bz as f64 * 0.008);
            remote.add(&btn);
        }
    }
    // Big rectangular button at bottom (mute/select).
    let big_btn = mesh(pg("SmallRemoteControls__bigbutton", box_geo!(0.024, 0.002, 0.014)), lambert!({ color: 0xd82040 }));
    big_btn.position().set(0, 0.011, 0.036);
    remote.add(&big_btn);
    // Antenna: thin cylinder poking out.
    let antenna_mat = lambert!({ color: 0x808080 });
    let antenna = mesh(pg("SmallRemoteControls__antenna", cyl_geo!(0.002, 0.002, 0.06, 4)), &antenna_mat);
    antenna.position().set(0.014, 0.010, -0.075);
    antenna.rotation().x = PI / 2.0 + 0.4;
    remote.add(&antenna);
    scatter_pos(g, &remote, x, z, 2.4);
    remote.rotation().y = rnd() * PI * 2.0;
    g.world.add(&remote);
}
