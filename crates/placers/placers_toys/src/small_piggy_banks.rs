//! `place_small_piggy_banks`
use crate::shared::tri_geo;
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small piggy bank: 6% chance per level. Pink piggy bank
// body + 4 legs + snout + ears + coin slot. Child-toy family.
pub(crate) fn place_small_piggy_banks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let piggy_colors = [0xf0a0a0, 0xf0c0d0, 0xd0e0f0, 0xf0d8c8];
    let piggy_mat = lambert!({ color: pick_color(&piggy_colors) });
    let slot_mat = lambert!({ color: 0x181008 });
    let eye_mat = lambert!({ color: 0x181008 });
    let pig = group();
    // Body: sphere scaled elongated.
    let body = mesh(pg("SmallPiggyBanks__body", sphere_geo!(0.045, 12, 10)), &piggy_mat);
    body.scale().set(1.3, 1, 1);
    body.position().y = 0.048;
    pig.add(&body);
    // 4 legs (small cylinders).
    for (lx, lz) in [(-0.04, -0.02), (0.04, -0.02), (-0.04, 0.02), (0.04, 0.02)] {
        let leg = mesh(pg("SmallPiggyBanks__leg", cyl_geo!(0.008, 0.008, 0.018, 6)), &piggy_mat);
        leg.position().set(lx, 0.009, lz);
        pig.add(&leg);
    }
    // Snout: cylinder at front.
    let snout = mesh(pg("SmallPiggyBanks__snout", cyl_geo!(0.014, 0.014, 0.014, 8)), &piggy_mat);
    snout.rotation().z = PI / 2.0;
    snout.position().set(0.058, 0.048, 0);
    pig.add(&snout);
    // 2 nostrils (tiny dark discs).
    for s in [-1.0, 1.0] {
        let nostril = mesh(circle_geo!(0.002, 5), &slot_mat);
        nostril.rotation().y = PI / 2.0;
        nostril.position().set(0.065, 0.048, s * 0.004);
        pig.add(&nostril);
    }
    // 2 ears (small triangles).
    for s in [-1.0, 1.0] {
        let ear_geo = tri_geo(&[0.0, 0.0, 0.0, 0.012, 0.014, 0.0, -0.010, 0.014, 0.0], vec![0, 1, 2]);
        let ear = mesh(ear_geo, &piggy_mat);
        // Tip up (the triangle is built tip-down) with its base sunk into the
        // back; each ear's front face turned outward so both show from outside.
        ear.position().set(0.020, 0.092, s * 0.024);
        ear.rotation().set(0, if s > 0.0 { 0.4 } else { PI - 0.4 }, PI);
        pig.add(&ear);
    }
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(pg("SmallPiggyBanks__eye", sphere_geo!(0.003, 5, 4)), &eye_mat);
        eye.position().set(0.038, 0.062, s * 0.024);
        pig.add(&eye);
    }
    // Coin slot on back.
    let slot = mesh(pg("SmallPiggyBanks__slot", box_geo!(0.024, 0.001, 0.005)), &slot_mat);
    slot.position().set(-0.02, 0.091, 0);
    pig.add(&slot);
    // Curly tail.
    let tail = mesh(pg("SmallPiggyBanks__tail", torus_geo!(0.008, 0.002, 4, 8, PI * 1.5)), &piggy_mat);
    // Curl stands in the pig's side plane (XY); its thick start (local +x)
    // is sunk 1.5 mm into the rump (surface x -0.057) and it curls up and back.
    tail.position().set(-0.0635, 0.058, 0);
    pig.add(&tail);
    scatter_pos(g, &pig, x, z, 2.4);
    pig.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pig);
}
