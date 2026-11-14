//! `place_birdcages`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_birdcages(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let iron_colors = [0x1a1a1a, 0x2a2a2a, 0xc0a060, 0x606060];
    let iron_mat = lambert!({ color: choice(&iron_colors) });
    let bc = group();
    // Floor stand — wide flat base + tall pole.
    let base = mesh(pg("Birdcages__base", cyl_geo!(0.10, 0.12, 0.014, 8)), &iron_mat);
    base.position().y = 0.007;
    bc.add(&base);
    let pole = mesh(pg("Birdcages__pole", cyl_geo!(0.008, 0.010, 1.10, 6)), &iron_mat);
    pole.position().y = 0.560;
    bc.add(&pole);
    // Cage bottom disc.
    let (cage_r, cage_h) = (0.11, 0.30);
    let cage_y = 1.112; // cage floor (±0.003) sits on the pole top (1.110)
    let cage_bottom = mesh(pg("Birdcages__floor", cyl_geo!(cage_r, cage_r, 0.006, 12)), &iron_mat);
    cage_bottom.position().y = cage_y;
    bc.add(&cage_bottom);
    // Cage top disc.
    let cage_top = mesh(pg("Birdcages__top", cyl_geo!(cage_r * 0.5, cage_r * 0.9, 0.006, 12)), &iron_mat);
    cage_top.position().y = cage_y + cage_h;
    bc.add(&cage_top);
    // Cage dome (small hemisphere on top).
    let cage_dome = mesh(pg("Birdcages__dome", sphere_geo!(cage_r * 0.5, 8, 5, 0, PI * 2.0, 0, PI / 2.0)), &iron_mat);
    cage_dome.position().y = cage_y + cage_h + 0.006;
    bc.add(&cage_dome);
    // Small top ring hook (already upright in the XY plane), its foot in the finial.
    let top_hook = mesh(pg("Birdcages__hook", torus_geo!(0.014, 0.002, 3, 8)), &iron_mat);
    top_hook.position().y = cage_y + cage_h + 0.072;
    bc.add(&top_hook);
    // 12 vertical bars around cage.
    for i in 0..12 {
        let ang = (i as f64 / 12.0) * PI * 2.0;
        let bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, cage_h, 3)), &iron_mat);
        // Slight taper — bars converge upward slightly.
        let bottom_x = ang.cos() * cage_r;
        let bottom_z = ang.sin() * cage_r;
        let top_x = ang.cos() * cage_r * 0.6;
        let top_z = ang.sin() * cage_r * 0.6;
        bar.position().set((bottom_x + top_x) / 2.0, cage_y + cage_h / 2.0, (bottom_z + top_z) / 2.0);
        // Compute bar tilt.
        let mut v = v3(top_x - bottom_x, cage_h, top_z - bottom_z);
        v.normalize();
        bar.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
        bc.add(&bar);
    }
    // 3 horizontal ring bands around cage.
    for y_frac in [0.2, 0.5, 0.8] {
        // Interpolate radius at height (converging upward).
        let r_at_y = cage_r - (cage_r - cage_r * 0.6) * y_frac;
        let ring = mesh(pg("Birdcages__ring", torus_geo!(r_at_y, 0.002, 3, 14)), &iron_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().y = cage_y + cage_h * y_frac;
        bc.add(&ring);
    }
    // Perch — horizontal wooden dowel across cage middle.
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let perch = mesh(pg("Birdcages__perch", cyl_geo!(0.005, 0.005, cage_r * 1.6, 5)), &wood_mat);
    perch.rotation().z = PI / 2.0;
    perch.position().y = cage_y + cage_h * 0.4;
    bc.add(&perch);
    // 70% chance: swing door open (an outward-tilted panel of bars).
    let door_open = rnd() < 0.7;
    if door_open {
        let door_panel = group();
        // 3 mini bars forming a door section.
        for j in 0..3 {
            let door_bar = mesh(pg("Birdcages__bar", cyl_geo!(0.002, 0.002, cage_h * 0.5, 3)), &iron_mat);
            door_bar.position().set(-0.020 + j as f64 * 0.020, 0, 0);
            door_panel.add(&door_bar);
        }
        // Door frame.
        let door_frame = mesh(box_geo!(0.06, cage_h * 0.5, 0.002), &iron_mat);
        door_frame.position().set(0, 0, 0);
        door_frame.set_mat(&lambert!({ color: 0x1a1a1a, transparent: true, opacity: 0 }));
        door_panel.add(&door_frame);
        door_panel.position().set(cage_r - 0.008, cage_y + cage_h * 0.5, 0);
        door_panel.rotation().y = -0.7; // Swung open.
        bc.add(&door_panel);
    }
    // 60% chance: single feather on floor near base — implies departed bird.
    if rnd() < 0.6 {
        let feather_colors = [0xf0f0e8, 0xd0a060, 0x606060, 0xa04040, 0x2060a0];
        let feather_mat = lambert!({ color: choice(&feather_colors) });
        let feather = mesh(pg("Birdcages__feather", sphere_geo!(0.008, 5, 4)), &feather_mat);
        feather.scale().set(0.5, 0.15, 2.5);
        feather.position().set(0.14, 0.005, 0.08);
        feather.rotation().y = rnd() * PI * 2.0;
        bc.add(&feather);
    }
    // Small water dish + food dish at cage bottom.
    let dish_mat = lambert!({ color: 0xf0e0c0 });
    for (dx2, dz2, colored) in [(-0.05, 0.0, false), (0.05, 0.0, true)] {
        let dish = mesh(pg("Birdcages__dish", cyl_geo!(0.020, 0.018, 0.008, 8)), &dish_mat);
        dish.position().set(dx2, cage_y + 0.007, dz2);
        bc.add(&dish);
        if !colored {
            // Water in water dish (translucent blue).
            let water = mesh(circle_geo!(0.018, 8), basic_mat!({ color: 0x506080, transparent: true, opacity: 0.6 }));
            water.rotation().x = -PI / 2.0;
            water.position().set(dx2, cage_y + 0.011, dz2);
            bc.add(&water);
        } else {
            // Seed pile — small dark scatter.
            for _i in 0..5 {
                let seed = mesh(pg("Birdcages__seed", sphere_geo!(0.002, 4, 3)), lambert!({ color: 0x604020 }));
                seed.position().set(dx2 + (rnd() - 0.5) * 0.020, cage_y + 0.012, dz2 + (rnd() - 0.5) * 0.020);
                bc.add(&seed);
            }
        }
    }
    bc.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    bc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bc);
}
