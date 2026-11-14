//! `place_dropped_mail_scatters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Dropped mail scatter: 8% chance per level. 5-9
// envelopes fanned on floor.
pub(crate) fn place_dropped_mail_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let env_colors = [0xf0ece0, 0xe8d8b8, 0xf0d8c8, 0xd8e8d0];
    let ink_mat = lambert!({ color: 0x201810 });
    let grp = group();
    let n_env = 5 + (rnd() * 5.0).floor() as i32;
    for i in 0..n_env {
        let env_mat = lambert!({ color: choice(&env_colors) });
        let env = mesh(pg("DroppedMailScatters__envelope", box_geo!(0.12, 0.001, 0.06)), &env_mat);
        let px = (rnd() - 0.5) * 0.35;
        let py = 0.0005 + i as f64 * 0.0006; // bottom one on the floor (1 mm thick), the rest stacked
        let pz = (rnd() - 0.5) * 0.30;
        env.position().set(px, py, pz);
        env.rotation().y = rnd() * PI * 2.0;
        grp.add(&env);
        // Small address rectangle.
        if rnd() < 0.5 {
            let addr = mesh(plane_geo!(0.030, 0.008), &ink_mat);
            addr.rotation().x = -PI / 2.0;
            // offset 2 cm along the envelope's own long axis (its yaw), just over its top
            let (ep, ery) = (env.pos(), env.rot().y);
            addr.position().set(ep.x + 0.02 * ery.dcos(), ep.y + 0.0006, ep.z - 0.02 * ery.dsin());
            addr.rotation().z = ery;
            grp.add(&addr);
        }
    }
    scatter(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}
