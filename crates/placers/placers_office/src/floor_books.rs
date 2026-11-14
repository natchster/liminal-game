//! `place_floor_books`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Small book on floor: 15% chance per level. Hardcover book
// with visible colored cover.
pub(crate) fn place_floor_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // Per-weather cover color pool.
    let cover_pool = book_cover_pool(g.weather_key());
    let cover_mat = lambert!({ color: cover_pool[pick(cover_pool.len())] });
    let pages_mat = lambert!({ color: 0xd8ccb0 });
    let book = group();
    // Cover (top face): 12×18 cm cover.
    let cover = mesh(pg("FloorBooks__cover", box_geo!(0.12, 0.005, 0.18)), &cover_mat);
    cover.position().y = 0.024;
    book.add(&cover);
    // Pages block (main body): slightly narrower cream box.
    let pages = mesh(pg("FloorBooks__pages", box_geo!(0.115, 0.020, 0.175)), &pages_mat);
    pages.position().y = 0.011;
    book.add(&pages);
    // Back cover: identical size to top cover, at bottom.
    let back = mesh(pg("FloorBooks__cover", box_geo!(0.12, 0.005, 0.18)), &cover_mat);
    back.position().y = -0.001;
    book.add(&back);
    book.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    book.rotation().y = rnd() * PI * 2.0;
    // Slight tilt for organic drop.
    book.rotation().x = (rnd() - 0.5) * 0.15;
    g.world.add(&book);
    // 25% chance: 1-2 more books stacked on top with slight offset.
    if rnd() < 0.25 {
        let stack_count = 1 + (rnd() * 2.0).floor() as i32;
        for s in 1..=stack_count {
            let sib_book = clone_obj(&book);
            // Give each stacked book its own cover color for variety.
            let sib_color = cover_pool[pick(cover_pool.len())];
            let sib_cover_mat = lambert!({ color: sib_color });
            // Swap cover-color meshes (index 0 top cover, 2 back cover).
            let kids = sib_book.children();
            kids[0].set_mat(&sib_cover_mat);
            kids[2].set_mat(&sib_cover_mat);
            let bp = book.pos();
            let bry = book.rot().y;
            sib_book.position().y = bp.y + s as f64 * 0.028;
            sib_book.position().x += (rnd() - 0.5) * 0.04;
            sib_book.position().z += (rnd() - 0.5) * 0.04;
            sib_book.rotation().y = bry + (rnd() - 0.5) * 0.30;
            sib_book.rotation().x = (rnd() - 0.5) * 0.10;
            g.world.add(&sib_book);
        }
    }
}

/// Book cover pool per weather; other keys get `[0x6a2020, 0x1a3040, 0x203020, 0x4a3820, 0x1a1a2a]`
/// (still / normal have no entry, so they take the baseline pool).
fn book_cover_pool(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x1a3040, 0x203040, 0x2a3438],      // cool blue-gray covers
        "electric" => &[0x4a3820, 0x604828, 0x503820], // warm brown-gold
        "haunted" => &[0x6a2020, 0x4a1010, 0x300808],  // dark red only
        "derelict" => &[0x4a3820, 0x3a3018, 0x584030], // aged warm brown
        _ => &[0x6a2020, 0x1a3040, 0x203020, 0x4a3820, 0x1a1a2a],
    }
}
