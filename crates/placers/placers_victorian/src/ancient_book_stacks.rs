//! `place_ancient_book_stacks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Ancient book stack: 8% chance per level. 3-4 leather-bound tomes stacked.
pub(crate) fn place_ancient_book_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x502010, 0x3a1808, 0x6a3820, 0x201a10, 0x8a2020];
    let page_mat = lambert!({ color: 0xe8dcc0 });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let stack = group();
    let n_books = 3 + (rnd() * 2.0).floor() as i32;
    let mut y = 0.0;
    for i in 0..n_books {
        let leather_mat = lambert!({ color: pick_hex(&leather_colors) });
        let book = group();
        let bw = 0.09 + rnd() * 0.04;
        let bh = 0.020 + rnd() * 0.008;
        let bd = 0.06 + rnd() * 0.02;
        // Cover.
        let cover = mesh(pg("AncientBookStacks__cover", box_geo!(bw, bh, bd)), &leather_mat);
        cover.position().y = bh / 2.0;
        book.add(&cover);
        // Page block (slightly smaller inset).
        let pages = mesh(pg("AncientBookStacks__pages", box_geo!(bw - 0.003, bh - 0.006, bd - 0.003)), &page_mat);
        pages.position().y = bh / 2.0;
        book.add(&pages);
        // Gold spine bands, across the spine at 0.2 and 0.8 of its length.
        for sy in [bh * 0.3, bh * 0.7] {
            let band = mesh(pg("AncientBookStacks__band", box_geo!(0.001, 0.003, bd)), &gold_mat);
            band.rotation().x = PI / 2.0;
            band.scale().z = 0.6 * bh / bd;
            band.position().set(-bw / 2.0 - 0.0004, bh / 2.0, (sy / bh - 0.5) * bd * 1.5);
            book.add(&band);
        }
        book.position().y = y;
        book.position().x = (rnd() - 0.5) * 0.010;
        book.position().z = (rnd() - 0.5) * 0.010;
        book.rotation().y = (rnd() - 0.5) * 0.15;
        stack.add(&book);
        y += bh; // each book rests on the one below
    }
    scatter(g, &stack, x, z, 2.4);
    g.world.add(&stack);
}
