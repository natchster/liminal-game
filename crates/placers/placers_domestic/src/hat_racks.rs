//! `place_hat_racks`
use game::prelude::*;
use placers_kit::place::wall_tiles;

// Wall hat rack w/ 2-3 hats (wall): 3% chance per level.
pub(crate) fn place_hat_racks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let rack = group();
    let mount_y = 1.85;
    // Horizontal rail.
    let rail_w = 0.50;
    let rail = mesh(pg("HatRacks__rail", box_geo!(rail_w, 0.020, 0.030)), &wood_mat);
    rail.position().set(0, mount_y, 0.015);
    rack.add(&rail);
    // 4 pegs.
    let mut peg_xs: Vec<f64> = vec![];
    for i in 0..4 {
        let px = -rail_w * 0.375 + i as f64 * (rail_w * 0.25);
        peg_xs.push(px);
        let peg = mesh(pg("HatRacks__peg", cyl_geo!(0.004, 0.004, 0.028, 5)), &brass_mat);
        peg.rotation().x = PI / 2.0;
        peg.position().set(px, mount_y, 0.034);
        rack.add(&peg);
        let knob = mesh(pg("HatRacks__knob", sphere_geo!(0.006, 5, 4)), &brass_mat);
        knob.position().set(px, mount_y - 0.002, 0.048);
        rack.add(&knob);
    }
    // Hang 2-3 hats on random pegs (dedup).
    let hat_colors = [0x101010, 0x604020, 0x804a20, 0x201050, 0xa02040, 0x606060, 0xf0e0d0];
    let hat_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut used_pegs: Vec<usize> = vec![];
    for _h in 0..hat_count {
        let mut peg_idx;
        let mut attempts = 0;
        loop {
            peg_idx = (rnd() * peg_xs.len() as f64).floor() as usize;
            attempts += 1;
            if !(used_pegs.contains(&peg_idx) && attempts < 10) {
                break;
            }
        }
        if used_pegs.contains(&peg_idx) {
            break;
        }
        used_pegs.push(peg_idx);
        let px = peg_xs[peg_idx];
        let hat_mat = lambert!({ color: choice(&hat_colors) });
        // Hat type: 50/50 fedora or ball cap.
        let is_fedora = rnd() < 0.5;
        let hat = group();
        if is_fedora {
            // Fedora — brim + crown + optional band.
            let brim = mesh(pg("HatRacks__brim", cyl_geo!(0.075, 0.075, 0.008, 12)), &hat_mat);
            brim.position().y = 0.0;
            hat.add(&brim);
            let crown = mesh(pg("HatRacks__crown", cyl_geo!(0.045, 0.048, 0.055, 10)), &hat_mat);
            crown.position().y = 0.032;
            hat.add(&crown);
            // Pinch/dent — small dark box on top.
            let dent = mesh(pg("HatRacks__dent", box_geo!(0.010, 0.005, 0.045)), lambert!({ color: 0x0a0a0a }));
            dent.position().y = 0.057;
            hat.add(&dent);
            // Band — thin torus ring around base of crown.
            let band_mat = lambert!({ color: 0x1a1a1a });
            let band = mesh(pg("HatRacks__band", torus_geo!(0.047, 0.005, 3, 12)), &band_mat);
            band.rotation().x = PI / 2.0;
            band.position().y = 0.010;
            hat.add(&band);
        } else {
            // Baseball cap — dome + brim.
            let dome = mesh(pg("HatRacks__capDome", sphere_geo!(0.050, 8, 6, 0, PI * 2.0, 0, PI / 2.0)), &hat_mat);
            dome.position().y = 0.008;
            hat.add(&dome);
            // Brim — flat half-disc projecting forward.
            let brim = mesh(pg("HatRacks__capBrim", cyl_geo!(0.055, 0.055, 0.006, 12, 1, false, 0, PI)), &hat_mat);
            brim.rotation().y = -PI / 2.0; // half-disc's round edge (+X) turned to face out (+Z), lying flat
            brim.position().set(0, 0.006, 0.035);
            hat.add(&brim);
            // Small button on top.
            let btn = mesh(pg("HatRacks__button", sphere_geo!(0.005, 5, 4)), &hat_mat);
            btn.position().y = 0.055;
            hat.add(&btn);
        }
        // Crown hooked over the peg: clear of the rail, the knob inside the crown
        // touching its inner top (brim clear of the wall).
        hat.position().set(px, mount_y - (if is_fedora { 0.056 } else { 0.042 }), 0.078);
        hat.rotation().y = (rnd() - 0.5) * 0.15;
        // For fedora, tilt slightly forward. For cap, no tilt.
        if is_fedora {
            hat.rotation().x = 0.15;
        }
        rack.add(&hat);
    }
    rack.rotation().y = (-(dx as f64)).datan2(-(dz as f64));
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015);
    rack.position().set(px, 0, pz);
    g.world.add(&rack);
}
