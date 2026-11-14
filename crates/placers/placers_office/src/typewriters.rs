//! `place_typewriters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Vintage typewriter: 10% chance per level. Box + slanted key
// area + paper roller + platen. Very institutional/office vintage.
pub(crate) fn place_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let body_mat = lambert!({ color: 0x282020 });
    let key_mat = lambert!({ color: 0x4a3a30 });
    let paper_mat = lambert!({ color: 0xe8dcc0 });
    let platen_mat = lambert!({ color: 0x1a1410 });
    let tw = group();
    // Body: main block.
    let body = mesh(pg("Typewriters__body", box_geo!(0.28, 0.10, 0.22)), &body_mat);
    body.position().y = 0.05;
    tw.add(&body);
    // Key area: slanted rectangle on front-top.
    let key_area = mesh(pg("Typewriters__keys", box_geo!(0.22, 0.03, 0.10)), &key_mat);
    key_area.rotation().x = -0.15;
    key_area.position().set(0, 0.10, 0.06);
    tw.add(&key_area);
    // Platen: horizontal cylinder across top-back for paper roller.
    let platen = mesh(pg("Typewriters__platen", cyl_geo!(0.020, 0.020, 0.24, 10)), &platen_mat);
    platen.rotation().z = PI / 2.0;
    platen.position().set(0, 0.12, -0.06);
    tw.add(&platen);
    // Paper sheet: vertical thin rectangle poking up from platen.
    let paper = mesh(pg("Typewriters__paper", plane_geo!(0.18, 0.14)), &paper_mat);
    paper.position().set(0, 0.19, -0.06);
    paper.rotation().x = -0.10;
    tw.add(&paper);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
    // 30% chance: 3-4 typed paper sheets scattered on floor.
    if rnd() < 0.30 {
        let psheet_count = 3 + (rnd() * 2.0).floor() as i32;
        for _ps in 0..psheet_count {
            let psheet = mesh(pg("Typewriters__sheet", plane_geo!(0.18, 0.24)), &paper_mat);
            psheet.rotation().x = -PI / 2.0;
            psheet.rotation().z = rnd() * PI * 2.0;
            let p_ang = rnd() * PI * 2.0;
            let p_rad = 0.25 + rnd() * 0.35;
            let tp = tw.pos();
            psheet.position().set(tp.x + p_ang.dcos() * p_rad, 0.004, tp.z + p_ang.dsin() * p_rad);
            g.world.add(&psheet);
        }
    }
}
