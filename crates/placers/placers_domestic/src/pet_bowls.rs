//! `place_pet_bowls`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Pet bowls — dog food + water bowl pair (floor): 4% chance per level.
pub(crate) fn place_pet_bowls(g: &mut Game, _dist: &[i32]) {
    g.pp.pet_bowl_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bowl_colors = [0xa02020, 0x2060a0, 0x408060, 0xd0a040, 0x606060, 0xf0e8d0, 0xa060a0];
    // 2 bowls — different colors per pair.
    let food_bowl_mat = lambert!({ color: choice(&bowl_colors) });
    let water_bowl_mat = lambert!({ color: choice(&bowl_colors) });
    let pb = group();
    let (bowl_r, bowl_h) = (0.09, 0.045);
    // Food bowl (left).
    let food_bowl = mesh(pg("PetBowls__bowl", cyl_geo!(bowl_r, bowl_r * 0.85, bowl_h, 12, 1, true)), &food_bowl_mat);
    food_bowl.position().set(-0.13, bowl_h / 2.0 + 0.002, 0);
    pb.add(&food_bowl);
    // Food bowl bottom disc.
    let food_bottom = mesh(circle_geo!(bowl_r * 0.85, 12), &food_bowl_mat);
    food_bottom.rotation().x = -PI / 2.0;
    food_bottom.position().set(-0.13, 0.002, 0);
    pb.add(&food_bottom);
    // Food bowl rim torus.
    let food_rim = mesh(pg("PetBowls__rim", torus_geo!(bowl_r, 0.004, 3, 12)), &food_bowl_mat);
    food_rim.rotation().x = PI / 2.0;
    food_rim.position().set(-0.13, bowl_h + 0.002, 0);
    pb.add(&food_rim);
    // 70% chance: kibble inside — brown scattered spheres.
    if rnd() < 0.7 {
        let kibble_mat = lambert!({ color: 0x8b5020 });
        let kibble_count = 12 + (rnd() * 8.0).floor() as i32;
        for _i in 0..kibble_count {
            let k_r = 0.008 + rnd() * 0.004;
            let kibble = mesh(pg("PetBowls__kibble", sphere_geo!(k_r, 5, 4)), &kibble_mat);
            let ang = rnd() * PI * 2.0;
            let rr = rnd() * bowl_r * 0.75;
            let ky = 0.015 + rnd() * 0.010;
            kibble.position().set(-0.13 + ang.cos() * rr, ky, ang.sin() * rr);
            pb.add(&kibble);
        }
    }
    // Water bowl (right).
    let water_bowl = mesh(pg("PetBowls__bowl", cyl_geo!(bowl_r, bowl_r * 0.85, bowl_h, 12, 1, true)), &water_bowl_mat);
    water_bowl.position().set(0.13, bowl_h / 2.0 + 0.002, 0);
    pb.add(&water_bowl);
    let water_bottom = mesh(circle_geo!(bowl_r * 0.85, 12), &water_bowl_mat);
    water_bottom.rotation().x = -PI / 2.0;
    water_bottom.position().set(0.13, 0.002, 0);
    pb.add(&water_bottom);
    let water_rim = mesh(pg("PetBowls__rim", torus_geo!(bowl_r, 0.004, 3, 12)), &water_bowl_mat);
    water_rim.rotation().x = PI / 2.0;
    water_rim.position().set(0.13, bowl_h + 0.002, 0);
    pb.add(&water_rim);
    // 80% chance: water surface disc inside.
    let has_water = rnd() < 0.8;
    if has_water {
        let water_mat = basic_mat!({ color: 0x506080, transparent: true, opacity: 0.65 });
        let water = mesh(circle_geo!(bowl_r * 0.82, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.position().set(0.13, bowl_h * 0.75, 0);
        pb.add(&water);
    }
    // 50% chance: tiny mat under both bowls.
    let mut mat_lift = 0.0;
    if rnd() < 0.5 {
        let mat_colors = [0x404040, 0x603020, 0x2060a0, 0x408060];
        let mat_mat = lambert!({ color: choice(&mat_colors) });
        let mat = mesh(pg("PetBowls__mat", box_geo!(0.35, 0.005, 0.16)), &mat_mat);
        mat.position().y = 0.0025;
        // Bowls (bottoms at 0.002) stand on the mat's top (0.005), not in it.
        mat_lift = 0.003;
        for c in pb.children() {
            c.position().y += mat_lift;
        }
        pb.add(&mat);
    }
    // Optional name label near bowls — 40% chance (tiny label plate).
    if rnd() < 0.4 {
        let label_mat = lambert!({ color: 0xc0a060 });
        let label = mesh(pg("PetBowls__label", box_geo!(0.04, 0.001, 0.010)), &label_mat);
        // Flat on the food bowl's flared outer wall (slope 0.3), facing out.
        label.position().set(-0.13, 0.032 + mat_lift, 0.0765 + 0.3 * 0.030);
        label.rotation().x = PI / 2.0 + 0.3f64.atan();
        pb.add(&label);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    pb.position().set(px, 0, pz);
    pb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pb);
    // Register position only if water bowl variant has water.
    if has_water {
        let p = pb.pos();
        g.pp.pet_bowl_positions.push(v3(p.x, 0, p.z));
    }
}
