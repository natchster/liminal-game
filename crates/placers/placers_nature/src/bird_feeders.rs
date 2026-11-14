//! `place_bird_feeders`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Bird seed feeder: 4% chance per level. Tube feeder w/ perches +
// optional visible seeds.
pub(crate) fn place_bird_feeders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tube_mat = lambert!({ color: 0xc0d0d0, transparent: true, opacity: 0.6 });
    let cap_mat = lambert!({ color: 0x408040 });
    let seed_mat = lambert!({ color: 0x604020 });
    let perch_mat = lambert!({ color: 0x606060 });
    let feeder = group();
    // Clear plastic tube.
    let tube = mesh(cyl_geo!(0.030, 0.030, 0.18, 12), &tube_mat);
    tube.position().y = 0.090;
    feeder.add(&tube);
    // Interior seeds (visible through tube).
    let seeds = mesh(pg("BirdFeeders__seeds", cyl_geo!(0.025, 0.025, 0.10, 10)), &seed_mat);
    seeds.position().y = 0.055;
    feeder.add(&seeds);
    // Top cap.
    let top_cap = mesh(pg("BirdFeeders__cap", cyl_geo!(0.036, 0.036, 0.012, 12)), &cap_mat);
    top_cap.position().y = 0.186;
    feeder.add(&top_cap);
    // Bottom base disc.
    let base = mesh(pg("BirdFeeders__base", cyl_geo!(0.045, 0.045, 0.008, 12)), &cap_mat);
    base.position().y = 0.004;
    feeder.add(&base);
    // 4 perches sticking out at 90° each.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let perch = mesh(pg("BirdFeeders__perch", cyl_geo!(0.002, 0.002, 0.040, 4)), &perch_mat);
        perch.rotation().z = PI / 2.0;
        perch.position().set(a.dcos() * 0.048, 0.030 + i as f64 * 0.010, a.dsin() * 0.048);
        perch.rotation().y = -a;
        feeder.add(&perch);
    }
    // Hanging loop at top.
    let lp = mesh(pg("BirdFeeders__loop", torus_geo!(0.010, 0.002, 3, 8)), &perch_mat);
    // Standing upright (XY plane) with its twisted ends 4 mm down in the
    // cap's hanger boss (cap top 0.192).
    lp.position().y = 0.200;
    feeder.add(&lp);
    feeder.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    feeder.rotation().y = rnd() * PI * 2.0;
    g.world.add(&feeder);
}
