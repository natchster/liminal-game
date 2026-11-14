//! `place_spilled_backpacks`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_spilled_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pack_colors = [0xa02040, 0x2060a0, 0x408060, 0x101010, 0xd0a040, 0x604020, 0xa04080];
    let pack_mat = lambert!({ color: choice(&pack_colors) });
    let sp = group();
    // Main pack body — slumped rounded box.
    let (body_w, body_h, body_d) = (0.24, 0.18, 0.14);
    let body = mesh(pg("SpilledBackpacks__body", box_geo!(body_w, body_h, body_d)), &pack_mat);
    body.position().set(0, body_h / 2.0 + 0.002, 0);
    body.rotation().x = 0.15;
    sp.add(&body);
    // Main compartment zipper.
    let zipper_mat = lambert!({ color: 0x1a1a1a });
    let zipper = mesh(pg("SpilledBackpacks__zip", box_geo!(body_w * 0.85, 0.003, 0.006)), &zipper_mat);
    zipper.position().set(0, body_h + 0.003, -body_d * 0.05);
    zipper.rotation().x = 0.15;
    sp.add(&zipper);
    // Zipper pull tabs at ends.
    for side in [-1.0, 1.0] {
        let pull = mesh(pg("SpilledBackpacks__pull", box_geo!(0.006, 0.008, 0.003)), &zipper_mat);
        pull.position().set(side * body_w * 0.43, body_h + 0.002, 0);
        sp.add(&pull);
    }
    // Front pocket.
    let (pocket_w, pocket_h, pocket_d) = (body_w * 0.7, body_h * 0.5, 0.020);
    let pocket = mesh(pg("SpilledBackpacks__pocket", box_geo!(pocket_w, pocket_h, pocket_d)), &pack_mat);
    pocket.position().set(0, body_h * 0.35, body_d * 0.5 + pocket_d / 2.0);
    pocket.rotation().x = 0.15;
    sp.add(&pocket);
    // Front pocket zipper.
    let pocket_zip = mesh(pg("SpilledBackpacks__zip", box_geo!(pocket_w * 0.85, 0.003, 0.005)), &zipper_mat);
    pocket_zip.position().set(0, body_h * 0.35 + pocket_h / 2.0, body_d * 0.5 + pocket_d + 0.001);
    pocket_zip.rotation().x = 0.15;
    sp.add(&pocket_zip);
    // 2 shoulder straps on back.
    for side in [-1.0, 1.0] {
        let strap = mesh(pg("SpilledBackpacks__strap", box_geo!(0.030, body_h * 1.1, 0.010)), &pack_mat);
        // On the leaning back face (body-local z -0.074), spanning its height.
        strap.position().set(side * body_w * 0.30, body_h / 2.0 + 0.002 + 0.074 * (0.15f64).sin(), -0.074 * (0.15f64).cos());
        strap.scale().y = 0.9;
        strap.rotation().x = 0.15;
        sp.add(&strap);
    }
    // Small carry handle on top.
    let handle_mat = lambert!({ color: 0x0a0a0a });
    let handle = mesh(pg("SpilledBackpacks__handle", torus_geo!(0.024, 0.005, 3, 8, PI)), &handle_mat);
    handle.rotation().z = 0.0;
    handle.rotation().x = 0.15; // arch standing up from the top, leaning with the body
    handle.position().set(0, body_h + 0.004, -body_d * 0.10);
    sp.add(&handle);
    // 2-3 spilled books tumbling out from opening.
    let book_colors = [0x603030, 0x2060a0, 0x408060, 0x804020, 0x604040, 0x606060];
    let book_count = 2 + (rnd() * 2.0).floor() as i32;
    let page_color_mat = lambert!({ color: 0xf0e8d0 });
    for i in 0..book_count {
        let book_mat = lambert!({ color: choice(&book_colors) });
        let book = group();
        let bw = 0.11 + rnd() * 0.04;
        let bh = 0.014 + rnd() * 0.006;
        let bd = 0.08 + rnd() * 0.03;
        let book_body = mesh(pg("SpilledBackpacks__book", box_geo!(bw, bh, bd)), &book_mat);
        book_body.position().y = bh / 2.0;
        book.add(&book_body);
        // Page-edge trim.
        let page_edge = mesh(pg("SpilledBackpacks__pages", box_geo!(bw * 0.96, bh - 0.001, bd - 0.006)), &page_color_mat);
        page_edge.position().y = bh / 2.0;
        book.add(&page_edge);
        // Position scattered near backpack opening.
        let ang = -0.3 + rnd() * 1.2;
        let rr = 0.15 + rnd() * 0.10;
        book.position().set(ang.cos() * rr, 0.006 + i as f64 * 0.005, ang.sin() * rr * 0.7);
        book.rotation().y = rnd() * PI * 2.0;
        book.rotation().z = (rnd() - 0.5) * 0.3;
        sp.add(&book);
    }
    // 60% chance: pencil scattered nearby.
    if rnd() < 0.6 {
        let pencil_mat = lambert!({ color: 0xf0c040 });
        let pencil = mesh(pg("SpilledBackpacks__pencil", cyl_geo!(0.003, 0.003, 0.10, 6)), &pencil_mat);
        pencil.rotation().z = PI / 2.0;
        pencil.rotation().y = rnd() * PI * 2.0;
        pencil.position().set(0.20, 0.006, -0.05);
        sp.add(&pencil);
    }
    // 40% chance: crumpled paper.
    if rnd() < 0.4 {
        let paper_mat = lambert!({ color: 0xf0e8d0 });
        let paper_ball = mesh(pg("SpilledBackpacks__paper", sphere_geo!(0.020, 5, 4)), &paper_mat);
        paper_ball.position().set(0.24, 0.020, 0.08);
        paper_ball.scale().set(1.1, 0.85, 1.0);
        sp.add(&paper_ball);
    }
    sp.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    sp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sp);
}
