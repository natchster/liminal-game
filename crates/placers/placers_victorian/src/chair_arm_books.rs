//! `place_chair_arm_books`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Open book on chair arm (floor): 5% chance per level.
pub(crate) fn place_chair_arm_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let wood_mat = lambert!({ color: 0x604020 });
    let cab = group();
    // Small side stand / narrow table.
    let (stand_w, stand_d, stand_h) = (0.34, 0.34, 0.55);
    let stand = mesh(pg("ChairArmBooks__top", box_geo!(stand_w, 0.014, stand_d)), &wood_mat);
    stand.position().y = stand_h;
    cab.add(&stand);
    // 4 tapered legs.
    for cx in [-stand_w * 0.42, stand_w * 0.42] {
        for cz in [-stand_d * 0.42, stand_d * 0.42] {
            let leg = mesh(pg("ChairArmBooks__leg", cyl_geo!(0.010, 0.014, stand_h - 0.007, 5)), &wood_mat);
            leg.position().set(cx, (stand_h - 0.007) / 2.0, cz);
            cab.add(&leg);
        }
    }
    // Cross-brace between the back pair of legs.
    let brace = mesh(pg("ChairArmBooks__brace", cyl_geo!(0.006, 0.006, stand_w * 0.75, 4)), &wood_mat);
    brace.rotation().z = PI / 2.0;
    brace.scale().y = (stand_w * 0.84) / (stand_w * 0.75);
    brace.position().set(0, stand_h * 0.35, -stand_d * 0.42);
    cab.add(&brace);
    // Open book face-down on stand.
    let book_colors: [u32; 5] = [0x603030, 0x2a4a60, 0x408030, 0x604040, 0x201050];
    let book_mat = lambert!({ color: book_colors[ri(book_colors.len())] });
    let page_mat = lambert!({ color: 0xf0e8d0 });
    let (book_w, book_d) = (0.12, 0.15);
    let left_page = mesh(pg("ChairArmBooks__pageL", box_geo!(book_w, 0.005, book_d)), &page_mat);
    left_page.position().set(-book_w / 2.0 + 0.004, stand_h + 0.014, 0);
    left_page.rotation().z = -0.10;
    cab.add(&left_page);
    let right_page = mesh(pg("ChairArmBooks__pageR", box_geo!(book_w, 0.005, book_d)), &page_mat);
    right_page.position().set(book_w / 2.0 - 0.004, stand_h + 0.014, 0);
    right_page.rotation().z = 0.10;
    cab.add(&right_page);
    // Central spine under the gutter.
    let spine = mesh(pg("ChairArmBooks__spine", cyl_geo!(0.003, 0.003, book_d, 4)), &book_mat);
    spine.rotation().x = PI / 2.0;
    spine.position().set(0, stand_h + 0.0045, 0);
    cab.add(&spine);
    // Cover flaps peeking from outer edges.
    let left_cover = mesh(pg("ChairArmBooks__cover", box_geo!(0.003, 0.007, book_d)), &book_mat);
    left_cover.position().set(-book_w + 0.002, stand_h + 0.016, 0);
    left_cover.rotation().z = -0.10;
    cab.add(&left_cover);
    let right_cover = mesh(pg("ChairArmBooks__cover", box_geo!(0.003, 0.007, book_d)), &book_mat);
    right_cover.position().set(book_w - 0.002, stand_h + 0.016, 0);
    right_cover.rotation().z = 0.10;
    cab.add(&right_cover);
    // 6 text lines per open page.
    let ink_mat = lambert!({ color: 0x1a1a1a });
    for side in 0..2 {
        for line in 0..6 {
            let line_w = book_w * (0.55 + rnd() * 0.35);
            let text_line = mesh(box_geo!(line_w, 0.002, 0.001), &ink_mat);
            let base_x = if side == 0 { -book_w * 0.5 } else { book_w * 0.5 };
            text_line.position().set(base_x + (rnd() - 0.5) * 0.010, stand_h + 0.0181, -book_d * 0.35 + line as f64 * (book_d * 0.11));
            text_line.rotation().z = if side == 0 { -0.10 } else { 0.10 };
            cab.add(&text_line);
        }
    }
    // 50% chance: bookmark ribbon peeking from spine.
    if rnd() < 0.5 {
        let ribbon_colors: [u32; 5] = [0xa02040, 0x2060a0, 0xc0a040, 0x603030, 0x408060];
        let ribbon_mat = lambert!({ color: ribbon_colors[ri(ribbon_colors.len())] });
        let ribbon = mesh(pg("ChairArmBooks__ribbon", box_geo!(0.008, 0.002, 0.05)), &ribbon_mat);
        ribbon.position().set(0, stand_h + 0.0085, book_d * 0.5 + 0.025);
        cab.add(&ribbon);
    }
    // 40% chance: reading glasses beside book on stand.
    if rnd() < 0.4 {
        let frame_colors: [u32; 4] = [0x101010, 0x604020, 0xa02040, 0xc0a040];
        let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())] });
        let lens_mat = basic_mat!({ color: 0xf0f0f8, transparent: true, opacity: 0.4 });
        let lens_r = 0.018;
        for side in [-1.0f64, 1.0] {
            let ring = mesh(pg("ChairArmBooks__rim", torus_geo!(lens_r, 0.002, 4, 10)), &frame_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(side * lens_r * 1.2 + book_w * 0.5, stand_h + 0.009, 0.125);
            cab.add(&ring);
            let lens = mesh(circle_geo!(lens_r - 0.002, 10), &lens_mat);
            lens.rotation().x = -PI / 2.0;
            lens.position().set(side * lens_r * 1.2 + book_w * 0.5, stand_h + 0.0095, 0.125);
            cab.add(&lens);
        }
        // Bridge.
        let bridge = mesh(pg("ChairArmBooks__bridge", box_geo!(0.012, 0.002, 0.003)), &frame_mat);
        bridge.position().set(book_w * 0.5, stand_h + 0.009, 0.125 - 0.005);
        cab.add(&bridge);
        // Folded temples.
        for side in [-1.0f64, 1.0] {
            let temple = mesh(pg("ChairArmBooks__temple", cyl_geo!(0.001, 0.001, 0.038, 3)), &frame_mat);
            temple.rotation().z = PI / 2.0;
            temple.rotation().y = side * -0.4;
            temple.position().set(side * (lens_r * 1.2 + 0.019) + book_w * 0.5, stand_h + 0.008, 0.125 + 0.014);
            cab.add(&temple);
        }
    }
    cab.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    cab.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cab);
}
