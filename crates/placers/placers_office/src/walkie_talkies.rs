//! `place_walkie_talkies`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Walkie talkie: 12% chance per level, 1 handheld radio on floor.
pub(crate) fn place_walkie_talkies(g: &mut Game, _dist: &[i32]) {
    g.pp.walkie_talkie_positions.clear(); // reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0x18201a });
    let grille_mat = lambert!({ color: 0x080a08 });
    let antenna_mat = lambert!({ color: 0x0a0a0a });
    let led_mat = basic_mat!({ color: 0xe04030 });
    let wt = group();
    let body = mesh(pg("WalkieTalkies__body", box_geo!(0.06, 0.14, 0.03)), &body_mat);
    body.position().y = 0.015;
    wt.add(&body);
    let grille = mesh(plane_geo!(0.04, 0.03), &grille_mat);
    grille.position().set(0, 0.05, 0.016);
    wt.add(&grille);
    let led = mesh(circle_geo!(0.004, 6), &led_mat);
    led.position().set(0.02, 0.005, 0.016);
    wt.add(&led);
    let antenna = mesh(pg("WalkieTalkies__antenna", cyl_geo!(0.003, 0.003, 0.11, 5)), &antenna_mat);
    antenna.position().set(-0.02, 0.11, 0);
    wt.add(&antenna);
    // Lying on its side — rotate 90° around Z.
    wt.rotation().z = PI / 2.0;
    wt.rotation().y = rnd() * PI * 2.0;
    let ox = (rnd() - 0.5) * 2.4;
    let oz = (rnd() - 0.5) * 2.4;
    wt.position().set(g.wx(x) + ox, 0.03, g.wz(z) + oz); // on its side: half-width 0.03 is now the height
    g.world.add(&wt);
    // Track for static-crackle audio events.
    let p = wt.pos();
    g.pp.walkie_talkie_positions.push(v3(p.x, 0.10, p.z));
}
