//! `place_soap_bars`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Soap bar: 10% chance per level.
pub(crate) fn place_soap_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let soap_colors = [0xf0d0d0, 0xd0e8d0, 0xf0e0a0, 0xe0d8f0];
    let soap_mat = lambert!({ color: choice(&soap_colors) });
    let dish_mat = lambert!({ color: 0xe8e0c8 });
    let soap = group();
    // Bar (rounded box, worn).
    let bar = mesh(pg("SoapBars__bar", box_geo!(0.07, 0.020, 0.04)), &soap_mat);
    bar.position().y = 0.010;
    soap.add(&bar);
    // 40%: sitting in dish tray.
    if rnd() < 0.40 {
        let dish = mesh(pg("SoapBars__dish", box_geo!(0.09, 0.008, 0.06)), &dish_mat);
        dish.position().y = 0.004;
        soap.add(&dish);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    soap.position().set(px, 0, pz);
    soap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&soap);
}
