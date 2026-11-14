//! `place_small_pocket_watches`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small pocket watch: 8% chance per level.
pub(crate) fn place_small_pocket_watches(g: &mut Game, _dist: &[i32]) {
    g.pp.pocket_watch_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case (disc).
    let case_ = mesh(pg("SmallPocketWatches__case", cyl_geo!(0.028, 0.028, 0.010, 14)), &brass_mat);
    case_.position().y = 0.005;
    watch.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.024, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0105;
    watch.add(&face);
    // Crown on side.
    let crown = mesh(pg("SmallPocketWatches__crown", cyl_geo!(0.004, 0.004, 0.006, 6)), &brass_mat);
    crown.rotation().z = PI / 2.0;
    crown.position().set(0.031, 0.005, 0);
    watch.add(&crown);
    // Chain of tiny torus links.
    for i in 0..6 {
        let fi = i as f64;
        let link = mesh(pg("SmallPocketWatches__link", torus_geo!(0.005, 0.001, 4, 8)), &brass_mat);
        link.position().set(0.036 + fi * 0.008, 0.005 + (fi * 0.4).dsin() * 0.003, 0);
        link.rotation().x = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
        watch.add(&link);
    }
    watch.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    watch.rotation().y = rnd() * PI * 2.0;
    g.world.add(&watch);
    // Track for tick audio.
    let wp = watch.pos();
    g.pp.pocket_watch_positions.push(v3(wp.x, 0.01, wp.z));
}
