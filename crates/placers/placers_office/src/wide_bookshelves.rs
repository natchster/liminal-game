//! `place_wide_bookshelves`
use game::prelude::*;
use placers_kit::place::wall_mount;
use placers_kit::place::wall_tiles;

// Wall-mounted bookshelf w/ books (wall): 2.5% chance per level.
pub(crate) fn place_wide_bookshelves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030, 0x2a1a10];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let bs = group();
    let mount_y = 1.20; // Center at chest-height.
    // Shelf dimensions.
    let (shelf_w, shelf_d, shelf_t) = (0.60, 0.16, 0.014);
    // 3 shelves stacked vertically.
    let shelf_spacing = 0.32;
    for i in 0..3 {
        let shelf = mesh(pg("WideBookshelves__shelf", box_geo!(shelf_w, shelf_t, shelf_d)), &wood_mat);
        shelf.position().set(0, mount_y - shelf_spacing + i as f64 * shelf_spacing, 0.08);
        bs.add(&shelf);
    }
    // Back panel (thin) mounted against wall.
    let back = mesh(pg("WideBookshelves__back", box_geo!(shelf_w, shelf_spacing * 2.0 + shelf_t, 0.008)), &wood_mat);
    back.position().set(0, mount_y, 0.008);
    bs.add(&back);
    // 2 side panels.
    for side in [-1.0f64, 1.0] {
        let side_panel = mesh(pg("WideBookshelves__side", box_geo!(0.010, shelf_spacing * 2.0 + shelf_t, shelf_d)), &wood_mat);
        side_panel.position().set(side * shelf_w * 0.5, mount_y, 0.08);
        bs.add(&side_panel);
    }
    // Books on 2 shelves (top + middle only).
    let book_colors =
        [0x603030, 0x2a4a60, 0x408030, 0x604040, 0x201050, 0x8b5a2b, 0x101010, 0xa02040, 0xd0a040, 0x606060, 0x804a20, 0x203060];
    for shelf_idx in 1..3 {
        let shelf_y = mount_y - shelf_spacing + shelf_idx as f64 * shelf_spacing;
        let mut book_x = -shelf_w * 0.42;
        let mut attempts = 0;
        while book_x < shelf_w * 0.42 && attempts < 20 {
            attempts += 1;
            let book_h = 0.14 + rnd() * 0.10;
            let book_w = 0.020 + rnd() * 0.014;
            let book_d = 0.10 + rnd() * 0.04;
            let is_leaning = rnd() < 0.20; // 20% books lean over.
            let book_mat = lambert!({ color: choice(&book_colors) });
            let book = mesh(pg("WideBookshelves__book", box_geo!(book_w, book_h, book_d)), &book_mat);
            book.position().set(book_x + book_w / 2.0, shelf_y + shelf_t / 2.0 + book_h / 2.0, 0.08);
            let mut base_z_rot = 0.0;
            if is_leaning {
                base_z_rot = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.15 + rnd() * 0.15);
                book.rotation().z = base_z_rot;
                // Leaning book needs slight height reduction.
                book.position().y -= 0.008;
            }
            bs.add(&book);
            // Optional dark title-line spine detail.
            let mut spine_line_ref: Option<Obj> = None;
            if rnd() < 0.5 {
                let spine_line_mat = lambert!({ color: 0x1a1a1a });
                let spine_line = mesh(box_geo!(book_w * 0.7, 0.002, 0.001), &spine_line_mat);
                let bx = book.pos().x;
                spine_line.position().set(bx, shelf_y + shelf_t / 2.0 + book_h * 0.7, 0.08 + book_d / 2.0 + 0.001);
                if is_leaning {
                    spine_line.rotation().z = base_z_rot;
                }
                bs.add(&spine_line);
                spine_line_ref = Some(spine_line);
            }
            // Subtle tilt animator on leaning books — 40% chance per leaning book.
            if is_leaning && rnd() < 0.4 {
                let tilt_phase = rnd() * PI * 2.0;
                let book2 = book.clone();
                let spine2 = spine_line_ref.clone();
                g.animate(move |_g, _dt, t| {
                    // Sinusoidal drift ±0.015 rad, 0.4 Hz — very slow settling.
                    let off = (t * 0.8 + tilt_phase).dsin() * 0.015;
                    book2.rotation().z = base_z_rot + off;
                    if let Some(s) = &spine2 {
                        s.rotation().z = base_z_rot + off;
                    }
                });
            }
            book_x += book_w + 0.001;
        }
    }
    // Optional decorative object on top shelf — 60% chance.
    if rnd() < 0.6 {
        let top_shelf_y = mount_y + shelf_spacing;
        let obj_type = (rnd() * 3.0).floor() as i32;
        if obj_type == 0 {
            // Small vase.
            let vase_mat = lambert!({ color: [0xa02020, 0x2060a0, 0x408060, 0xd0a040][(rnd() * 4.0).floor() as usize] });
            let vase = mesh(pg("WideBookshelves__vase", cyl_geo!(0.030, 0.024, 0.070, 8)), &vase_mat);
            vase.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.035, 0.08);
            bs.add(&vase);
        } else if obj_type == 1 {
            // Framed photo (mini).
            let frame_mat = lambert!({ color: 0x604020 });
            let frame = mesh(pg("WideBookshelves__photoframe", box_geo!(0.05, 0.06, 0.004)), &frame_mat);
            frame.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.030, 0.08);
            bs.add(&frame);
            let photo = mesh(pg("WideBookshelves__photo", box_geo!(0.04, 0.05, 0.005)), lambert!({ color: 0xa08060 }));
            photo.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.030, 0.08 + 0.002);
            bs.add(&photo);
        } else {
            // Trophy (small cup on base).
            let trophy_mat = lambert!({ color: 0xc0a060 });
            let trophy_base = mesh(pg("WideBookshelves__trophybase", box_geo!(0.030, 0.010, 0.030)), &trophy_mat);
            trophy_base.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.005, 0.08);
            bs.add(&trophy_base);
            let trophy_stem = mesh(pg("WideBookshelves__trophystem", cyl_geo!(0.004, 0.004, 0.020, 4)), &trophy_mat);
            trophy_stem.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.020, 0.08);
            bs.add(&trophy_stem);
            let trophy_cup = mesh(pg("WideBookshelves__trophycup", cyl_geo!(0.014, 0.010, 0.020, 8)), &trophy_mat);
            trophy_cup.position().set(shelf_w * 0.35, top_shelf_y + shelf_t / 2.0 + 0.038, 0.08);
            bs.add(&trophy_cup);
        }
    }
    wall_mount(g, &bs, x, z, dx, dz, 0.015);
    g.world.add(&bs);
}
