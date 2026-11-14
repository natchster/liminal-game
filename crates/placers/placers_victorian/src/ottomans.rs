//! `place_ottomans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Ottoman / footstool (floor): 4% chance per level.
pub(crate) fn place_ottomans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let uphol_colors: [u32; 7] = [0x603030, 0x2a4a60, 0x604040, 0x603520, 0x304020, 0x604060, 0x505050];
    let uphol_mat = lambert!({ color: uphol_colors[ri(uphol_colors.len())] });
    let wood_mat = lambert!({ color: 0x604020 });
    let ot = group();
    // Bimodal: round tufted ottoman vs square upholstered footstool.
    let is_round = rnd() < 0.5;
    let _ott_h = 0.32;
    if is_round {
        let ot_r = 0.24;
        let cushion = mesh(pg("Ottomans__drum", cyl_geo!(ot_r, ot_r - 0.010, 0.18, 12)), &uphol_mat);
        cushion.position().y = 0.16 + 0.09; // sits on the skirt
        ot.add(&cushion);
        // 5 button-tuft indents on top.
        let button_mat = lambert!({ color: 0x1a1a1a });
        let button_r = 0.006;
        for i in 0..5 {
            let ang = if i == 0 { 0.0 } else { (i as f64 - 1.0) / 4.0 * PI * 2.0 };
            let rr = if i == 0 { 0.0 } else { ot_r * 0.55 };
            let btn = mesh(circle_geo!(button_r, 5), &button_mat);
            btn.rotation().x = -PI / 2.0;
            btn.position().set(ang.cos() * rr, 0.16 + 0.18 - (if i == 0 { 0.011 } else { 0.020 }), ang.sin() * rr);
            ot.add(&btn);
        }
        // Skirt below cushion.
        let skirt = mesh(pg("Ottomans__drumSkirt", cyl_geo!(ot_r - 0.005, ot_r - 0.005, 0.14, 12)), &uphol_mat);
        skirt.position().y = 0.09;
        ot.add(&skirt);
        // 4 small wooden feet.
        for i in 0..4 {
            let ang = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
            let foot = mesh(pg("Ottomans__bunFoot", cyl_geo!(0.014, 0.020, 0.020, 5)), &wood_mat);
            foot.position().set(ang.cos() * (ot_r - 0.020), 0.010, ang.sin() * (ot_r - 0.020));
            ot.add(&foot);
        }
    } else {
        // Square footstool — rectangular cushion + 4 tapered legs.
        let (ot_w, ot_d) = (0.38, 0.30);
        let cushion = mesh(pg("Ottomans__cushion", box_geo!(ot_w, 0.14, ot_d)), &uphol_mat);
        cushion.position().y = 0.22 + 0.07;
        ot.add(&cushion);
        let skirt = mesh(pg("Ottomans__apron", box_geo!(ot_w - 0.008, 0.10, ot_d - 0.008)), &uphol_mat);
        skirt.position().y = 0.17; // top meets the cushion
        ot.add(&skirt);
        for cx in [-ot_w * 0.42, ot_w * 0.42] {
            for cz in [-ot_d * 0.42, ot_d * 0.42] {
                let leg = mesh(pg("Ottomans__leg", cyl_geo!(0.010, 0.014, 0.14, 5)), &wood_mat);
                leg.position().set(cx, 0.07, cz);
                ot.add(&leg);
            }
        }
    }
    // 30% chance: throw pillow on top.
    if rnd() < 0.3 {
        let pillow_colors: [u32; 5] = [0xa02040, 0x408060, 0xd0a040, 0xf0e0c0, 0x2060a0];
        let pillow_mat = lambert!({ color: pillow_colors[ri(pillow_colors.len())] });
        let pillow = mesh(pg("Ottomans__pillow", box_geo!(0.16, 0.04, 0.16)), &pillow_mat);
        pillow.position().set(0, if is_round { 0.16 + 0.18 + 0.020 } else { 0.36 + 0.020 }, 0);
        pillow.rotation().y = (rnd() - 0.5) * 0.4;
        ot.add(&pillow);
    }
    ot.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    ot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ot);
}
