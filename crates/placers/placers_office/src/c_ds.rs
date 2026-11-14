//! `place_c_ds`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// CD/DVD disc: 8% chance per level. Sometimes label side up.
pub(crate) fn place_cds(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let label_side_up = rnd() < 0.5;
    let data_mat = lambert!({ color: 0xd0d0d8 });
    let label_colors = [0xffffff, 0xd050a0, 0x2080c0, 0x40a040, 0xc0a040, 0x202020];
    let label_mat = lambert!({ color: choice(&label_colors) });
    let disc = group();
    let face = mesh(pg("CDs__disc", cyl_geo!(0.060, 0.060, 0.001, 16)), if label_side_up { &label_mat } else { &data_mat });
    face.position().y = 0.0015;
    disc.add(&face);
    let under = mesh(pg("CDs__disc", cyl_geo!(0.060, 0.060, 0.001, 16)), if label_side_up { &data_mat } else { &label_mat });
    under.position().y = 0.0005;
    under.rotation().x = PI;
    disc.add(&under);
    let hole = mesh(cyl_geo!(0.0075, 0.0075, 0.003, 8), lambert!({ color: 0x080808 }));
    hole.position().y = 0.0015;
    disc.add(&hole);
    let ring = mesh(pg("CDs__hubRing", torus_geo!(0.018, 0.001, 3, 12)), lambert!({ color: 0x808088 }));
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.0025;
    disc.add(&ring);
    let (px, pz) = jitter(g, x, z, 2.4);
    disc.position().set(px, 0, pz);
    disc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&disc);
}
