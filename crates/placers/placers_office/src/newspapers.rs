//! `place_newspapers`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Crumpled newspaper w/ headline text (floor): 5% chance per level.
pub(crate) fn place_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let paper_mat = lambert!({ map: newspaper_texture() });
    let n = group();
    // Main sheet — thin box on floor.
    let (sheet_w, sheet_d) = (0.22, 0.28);
    let sheet = mesh(pg("Newspapers__sheet", box_geo!(sheet_w, 0.003, sheet_d)), &paper_mat);
    sheet.position().y = 0.0015;
    n.add(&sheet);
    // Crumpled tilted fold (upper right corner lifted).
    let fold1 = mesh(pg("Newspapers__fold", box_geo!(sheet_w * 0.4, 0.004, sheet_d * 0.4)), &paper_mat);
    fold1.position().set(sheet_w * 0.30, 0.030, sheet_d * 0.30); // lowest corner rests on the sheet
    fold1.rotation().z = 0.35;
    fold1.rotation().x = 0.2;
    n.add(&fold1);
    // Another wrinkle bulge (small tilted plate).
    let fold2 = mesh(pg("Newspapers__scrap", box_geo!(sheet_w * 0.25, 0.004, sheet_d * 0.20)), &paper_mat);
    fold2.position().set(-sheet_w * 0.30, 0.017, -sheet_d * 0.15);
    fold2.rotation().z = -0.30;
    fold2.rotation().x = -0.15;
    n.add(&fold2);
    n.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    n.rotation().y = rnd() * PI * 2.0;
    g.world.add(&n);
}
