//! Prop placers: graduation photos, sandwiches, reading glasses and books, coffee
//! mugs, newspapers, letters, open diaries, off-hook phones, chess boards, hair
//! dryers, wine glasses, sewing machines and pill bottles.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_graduation_photos", place_graduation_photos));
    r.push(("place_sandwiches", place_sandwiches));
    r.push(("place_reading_glasses_and_books", place_reading_glasses_and_books));
    r.push(("place_coffee_mugs", place_coffee_mugs));
    r.push(("place_newspapers", place_newspapers));
    r.push(("place_letters", place_letters));
    r.push(("place_open_diaries", place_open_diaries));
    r.push(("place_off_hook_phones", place_off_hook_phones));
    r.push(("place_chess_boards", place_chess_boards));
    r.push(("place_hairdryers", place_hairdryers));
    r.push(("place_wine_glasses", place_wine_glasses));
    r.push(("place_sewing_machines", place_sewing_machines));
    r.push(("place_pill_bottles", place_pill_bottles));
}

// ------------------------------------------------------------------ helpers
/// The `open_tiles` scan every floor placer starts with (`tiles[t_idx(x, z)] == 0`).
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// The `wall_tiles` scan of the wall placers: open tiles with the first
/// neighbouring wall (`== 1`) in `[(1,0), (-1,0), (0,1), (0,-1)]` order.
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(a: &[T]) -> T {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// Oriented cylinder segment between two points (`mid`, `set_from_unit_vectors`).
fn orient_seg(seg: &Obj, prev: &Vec3, next: &Vec3) {
    let mut mid = *prev;
    mid.add(next).multiply_scalar(0.5);
    seg.position().copy(&mid);
    let mut v = *next;
    v.sub(prev).normalize();
    seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
}

// R690 framed graduation photo w/ tassel (wall): 2.5% chance per level.
fn place_graduation_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0xd0a040, 0x808080, 0x8b5a2b];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: graduation_photo_texture() });
    let gp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.18, 0.22);
    let frame = mesh(pg("GraduationPhotos__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    gp.add(&frame);
    let photo = mesh(plane_geo!(frame_w - 0.018, frame_h - 0.018), &photo_mat);
    photo.position().set(0, mount_y, 0.019);
    gp.add(&photo);
    // Add a small real hanging fabric tassel below the frame — 3D touch.
    let tassel_color_pool = [0xd0a040, 0xa02040, 0x2060a0, 0x408060];
    let tassel_mat = lambert!({ color: pick(&tassel_color_pool) });
    let tassel_cord = mesh(pg("HangingColanders__cord", cyl_geo!(0.001, 0.001, 0.04, 3)), &tassel_mat);
    tassel_cord.position().set(frame_w * 0.35, mount_y - frame_h * 0.55, 0.014);
    gp.add(&tassel_cord);
    let tassel_knot = mesh(sphere_geo!(0.004, 5, 4), &tassel_mat);
    tassel_knot.position().set(frame_w * 0.35, mount_y - frame_h * 0.55 - 0.024, 0.014);
    gp.add(&tassel_knot);
    // Tassel strands — 5 thin cylinders splaying down.
    for i in 0..5 {
        let fi = i as f64;
        let strand = mesh(cyl_geo!(0.0006, 0.0006, 0.026, 3), &tassel_mat);
        strand.position().set(frame_w * 0.35 + (fi - 2.0) * 0.002, mount_y - frame_h * 0.55 - 0.038, 0.014);
        strand.rotation().z = (fi - 2.0) * 0.05;
        gp.add(&strand);
    }
    gp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    gp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    gp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&gp);
}

// R691 half-eaten sandwich on plate (floor): 6% chance per level.
fn place_sandwiches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let plate_colors = [0xffffff, 0xf0e8d8, 0xe8e0c8, 0xd0d0e0];
    let plate_mat = lambert!({ color: pick(&plate_colors) });
    // Bread color pool.
    let bread_colors = [0xd0a060, 0xa06030, 0xe0c088, 0xc09858];
    let bread_mat = lambert!({ color: pick(&bread_colors) });
    // Filling color pool — cheese/ham/turkey tones.
    let filling_colors = [0xf0d040, 0xe0a080, 0xd0c0a0, 0xc06040, 0x804020];
    let filling_mat = lambert!({ color: pick(&filling_colors) });
    let s = group();
    // Plate — shallow disc + torus rim.
    let plate = mesh(pg("Sandwiches__plate", cyl_geo!(0.10, 0.092, 0.006, 16)), &plate_mat);
    plate.position().y = 0.003;
    s.add(&plate);
    let plate_rim = mesh(pg("Sandwiches__rim", torus_geo!(0.095, 0.004, 4, 16)), &plate_mat);
    plate_rim.rotation().x = PI / 2.0;
    plate_rim.position().y = 0.008;
    s.add(&plate_rim);
    // Sandwich cut diagonally — bottom bread slice (half-square).
    let (half_w, half_h) = (0.08, 0.020);
    let bottom_bread = mesh(pg("Sandwiches__bread", box_geo!(half_w, half_h, half_w)), &bread_mat);
    bottom_bread.position().y = 0.006 + half_h / 2.0;
    bottom_bread.rotation().y = PI / 4.0;
    s.add(&bottom_bread);
    // Filling layer (thinner).
    let filling = mesh(pg("Sandwiches__filling", box_geo!(half_w * 0.98, 0.008, half_w * 0.98)), &filling_mat);
    filling.position().y = 0.006 + half_h + 0.004;
    filling.rotation().y = PI / 4.0;
    s.add(&filling);
    // Optional lettuce — 70% chance.
    if rnd() < 0.7 {
        let lettuce_mat = lambert!({ color: 0x60a040 });
        // 3 small ruffled edges peeking out (thin boxes).
        for i in 0..3 {
            let ruffle = mesh(pg("Sandwiches__lettuce", box_geo!(0.024, 0.004, 0.010)), &lettuce_mat);
            let ang = (i as f64 / 3.0) * PI * 2.0 + PI / 4.0;
            ruffle.position().set(ang.cos() * 0.04, 0.006 + half_h + 0.008, ang.sin() * 0.04);
            ruffle.rotation().y = ang + PI / 2.0;
            ruffle.rotation().z = 0.2;
            s.add(&ruffle);
        }
    }
    // Top bread slice.
    let top_bread = mesh(pg("Sandwiches__bread", box_geo!(half_w, half_h, half_w)), &bread_mat);
    top_bread.position().y = 0.006 + half_h + 0.008 + half_h / 2.0;
    top_bread.rotation().y = PI / 4.0;
    s.add(&top_bread);
    // Bite gap — a slightly darker bread interior wedge.
    let bite_mat = lambert!({ color: 0xa08050 });
    let bite_inner = mesh(pg("Sandwiches__bite", box_geo!(0.030, half_h * 2.0 + 0.008, 0.030)), &bite_mat);
    // Set into the sandwich edge (3 mm proud) and standing on the plate, flush
    // with the top slice.
    bite_inner.position().set(half_w * 0.25, 0.0305, half_w * 0.25);
    bite_inner.rotation().y = PI / 4.0;
    s.add(&bite_inner);
    // Bread crumbs — 6-10 scattered around plate.
    let crumb_count = 6 + (rnd() * 5.0).floor() as i32;
    for i in 0..crumb_count {
        let c_size = 0.003 + rnd() * 0.004;
        let crumb = mesh(pg("Sandwiches__crumb", box_geo!(c_size, c_size, c_size)), &bread_mat);
        let ang = rnd() * PI * 2.0;
        // Resting on the plate's flat, inside the rim.
        let rr = 0.05 + rnd() * 0.035;
        crumb.position().set(ang.cos() * rr, 0.006 + c_size / 2.0, ang.sin() * rr);
        crumb.rotation().y = rnd() * PI * 2.0;
        s.add(&crumb);
    }
    s.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    s.rotation().y = rnd() * PI * 2.0;
    g.world.add(&s);
}

// R691 reading glasses beside open book (floor): 5% chance per level.
fn place_reading_glasses_and_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Book cover color pool.
    let cover_colors = [0x603020, 0x2040a0, 0x408060, 0x804020, 0x202050, 0xa06040];
    let cover_mat = lambert!({ color: pick(&cover_colors) });
    let page_mat = lambert!({ color: 0xf0e8d0 });
    // Glasses colors.
    let frame_colors = [0x101010, 0x604020, 0xa02040, 0xc0a040, 0x2060a0];
    let glasses_mat = lambert!({ color: pick(&frame_colors) });
    let lens_mat = basic_mat!({ color: 0xf0f0f8, transparent: true, opacity: 0.4 });
    let grp = group();
    // Book — open face-down: 2 page-boxes tilted like a tent + spine cylinder.
    let (book_w, book_d) = (0.10, 0.14);
    let left_page = mesh(pg("ReadingGlassesAndBooks__pages", box_geo!(book_w, 0.006, book_d)), &page_mat);
    left_page.position().set(-book_w / 2.0 + 0.005, 0.008, 0);
    left_page.rotation().z = -0.10;
    grp.add(&left_page);
    let right_page = mesh(pg("ReadingGlassesAndBooks__pages", box_geo!(book_w, 0.006, book_d)), &page_mat);
    right_page.position().set(book_w / 2.0 - 0.005, 0.008, 0);
    right_page.rotation().z = 0.10;
    grp.add(&right_page);
    // Spine cylinder linking pages.
    let spine = mesh(pg("ReadingGlassesAndBooks__spine", cyl_geo!(0.003, 0.003, book_d, 4)), &cover_mat);
    spine.rotation().x = PI / 2.0;
    spine.position().set(0, 0.014, 0);
    grp.add(&spine);
    // Cover flap peeking from the outer edges (thin darker box).
    let left_cover = mesh(pg("ReadingGlassesAndBooks__cover", box_geo!(0.003, 0.008, book_d)), &cover_mat);
    left_cover.position().set(-book_w + 0.002, 0.010, 0);
    left_cover.rotation().z = -0.10;
    grp.add(&left_cover);
    let right_cover = mesh(pg("ReadingGlassesAndBooks__cover", box_geo!(0.003, 0.008, book_d)), &cover_mat);
    right_cover.position().set(book_w - 0.002, 0.010, 0);
    right_cover.rotation().z = 0.10;
    grp.add(&right_cover);
    // Reading glasses lying beside — folded flat.
    let glasses_off = 0.15; // Offset from book (clear of its right cover edge at x 0.10).
    let glasses = group();
    // Two lens rings (torus).
    let lens_r = 0.020;
    for side in [-1.0, 1.0] {
        let ring = mesh(pg("ReadingGlassesAndBooks__rim", torus_geo!(lens_r, 0.002, 4, 12)), &glasses_mat);
        ring.rotation().x = PI / 2.0;
        ring.position().set(side * lens_r * 1.2, 0, 0);
        glasses.add(&ring);
        // Lens (translucent disc).
        let lens = mesh(circle_geo!(lens_r - 0.002, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(side * lens_r * 1.2, 0.001, 0);
        glasses.add(&lens);
    }
    // Bridge (short bar).
    let bridge = mesh(pg("ReadingGlassesAndBooks__bridge", box_geo!(0.014, 0.002, 0.003)), &glasses_mat);
    bridge.position().set(0, 0, -0.005);
    glasses.add(&bridge);
    // Temples folded flat across the back of the frame, resting on the rims.
    for side in [-1.0, 1.0] {
        let temple = mesh(pg("ReadingGlassesAndBooks__temple", cyl_geo!(0.001, 0.001, 0.048, 3)), &glasses_mat);
        temple.rotation().z = PI / 2.0;
        temple.rotation().y = 0.0;
        temple.position().set(
            side * (lens_r * 2.2 - 0.024),
            if side > 0.0 { 0.0025 } else { 0.0045 },
            if side > 0.0 { -0.004 } else { 0.002 },
        );
        glasses.add(&temple);
    }
    glasses.position().set(glasses_off, 0.002, 0.02); // rims (tube r 0.002) on the floor
    glasses.rotation().y = (rnd() - 0.5) * 0.8;
    grp.add(&glasses);
    // Optional bookmark slip peeking from book — 40% chance.
    if rnd() < 0.4 {
        let bookmark_colors = [0xa02040, 0x2060a0, 0xc0a040, 0x408060];
        let bk_mat = lambert!({ color: pick(&bookmark_colors) });
        let bookmark = mesh(pg("ReadingGlassesAndBooks__bookmark", box_geo!(0.012, 0.003, 0.05)), &bk_mat);
        // tucked in the gutter at the top edge, its tail drooping out over the edge
        bookmark.position().set(0.004, 0.007, -book_d / 2.0);
        bookmark.rotation().x = -0.2;
        bookmark.rotation().z = 0.2;
        grp.add(&bookmark);
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R692 coffee mug w/ steam + coffee stain ring (floor): 7% chance per level.
fn place_coffee_mugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let mug_colors = [0xffffff, 0x2060a0, 0xa02040, 0x606060, 0xd0a040, 0x408060, 0x101010];
    let mug_mat = lambert!({ color: pick(&mug_colors) });
    let coffee_mat = lambert!({ color: 0x2a1808 });
    let m = group();
    // Body — cylinder.
    let (body_r, body_h) = (0.038, 0.085);
    let body = mesh(pg("CoffeeMugs__body", cyl_geo!(body_r, body_r * 0.9, body_h, 14)), &mug_mat);
    body.position().y = body_h / 2.0 + 0.002;
    m.add(&body);
    // Coffee surface disc (near top).
    let coffee = mesh(circle_geo!(body_r * 0.94, 14), &coffee_mat);
    coffee.rotation().x = -PI / 2.0;
    coffee.position().y = body_h - 0.005;
    m.add(&coffee);
    // R707 subtle liquid ripple — 45% chance.
    if rnd() < 0.45 {
        let coffee_base_y = coffee.pos().y;
        let ripple_phase = rnd() * PI * 2.0;
        let ripple_phase2 = rnd() * PI * 2.0;
        let coffee2 = coffee.clone();
        g.animate(move |_g, _dt, t| {
            // Multi-freq combined sine — imperceptible amplitude.
            coffee2.position().y = coffee_base_y + (t * 1.8 + ripple_phase).sin() * 0.0004 + (t * 3.5 + ripple_phase2).sin() * 0.0002;
            // Micro-tilt makes the reflection catch light differently.
            coffee2.rotation().z = (t * 1.2 + ripple_phase).sin() * 0.008;
            coffee2.rotation().x = -PI / 2.0 + (t * 1.0 + ripple_phase2).sin() * 0.006;
        });
    }
    // Rim torus.
    let rim = mesh(pg("CoffeeMugs__rim", torus_geo!(body_r, 0.003, 4, 14)), &mug_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = body_h + 0.002;
    m.add(&rim);
    // Handle — partial torus arc on side (C-handle bulging outward).
    let handle = mesh(pg("CoffeeMugs__handle", torus_geo!(0.020, 0.005, 4, 10, PI)), &mug_mat);
    handle.rotation().z = -PI / 2.0;
    handle.position().set(body_r - 0.002, body_h * 0.5 + 0.002, 0);
    m.add(&handle);
    // 70% chance: drifting steam wisps + animator.
    if rnd() < 0.7 {
        let steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.18 });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
        }
        let mut steam_wisps: Vec<Wisp> = vec![];
        for i in 0..3 {
            let wisp = mesh(sphere_geo!(0.014, 6, 5), &steam_mat);
            wisp.scale().y = 2.2;
            wisp.position().set((rnd() - 0.5) * 0.02, body_h + 0.04 + i as f64 * 0.03, (rnd() - 0.5) * 0.02);
            steam_wisps.push(Wisp { mesh: wisp.clone(), phase: rnd() * PI * 2.0, base_y: wisp.pos().y });
            m.add(&wisp);
        }
        g.animate(move |_g, _dt, t| {
            for w in &steam_wisps {
                {
                    let mut p = w.mesh.position();
                    // Slow drift upward + gentle horizontal sway.
                    p.y = w.base_y + (t * 0.6 + w.phase).sin() * 0.015;
                    p.x = (t * 0.4 + w.phase).sin() * 0.015;
                    p.z = (t * 0.5 + w.phase * 1.3).cos() * 0.015;
                }
                // Subtle scale pulse.
                let s = 0.9 + (t * 0.8 + w.phase).sin() * 0.15;
                w.mesh.scale().set(s, 2.2 * s, s);
            }
        });
    }
    // 50% chance: dried coffee stain ring on floor beside mug.
    if rnd() < 0.5 {
        let stain_mat = basic_mat!({ color: 0x4a2810, transparent: true, opacity: 0.55 });
        let stain_ring = mesh(ring_geo!(0.038, 0.048, 20), &stain_mat);
        stain_ring.rotation().x = -PI / 2.0;
        stain_ring.position().set((rnd() - 0.3) * 0.10, 0.001, (rnd() - 0.3) * 0.10);
        m.add(&stain_ring);
    }
    m.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    m.rotation().y = rnd() * PI * 2.0;
    g.world.add(&m);
}

// R692 crumpled newspaper w/ headline text (floor): 5% chance per level.
fn place_newspapers(g: &mut Game, _dist: &[i32]) {
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

// R693 crumpled letter w/ opening greeting (floor): 5% chance per level.
fn place_letters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let paper_mat = lambert!({ map: letter_texture() });
    let l = group();
    // Main sheet.
    let (sheet_w, sheet_d) = (0.14, 0.18);
    let sheet = mesh(pg("Letters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &paper_mat);
    sheet.position().y = 0.001;
    l.add(&sheet);
    // Reused crumpled-paper wrinkle pattern (R692) — 2 tilted plane bulges.
    let fold1 = mesh(pg("Letters__fold", box_geo!(sheet_w * 0.5, 0.003, sheet_d * 0.35)), &paper_mat);
    fold1.position().set(sheet_w * 0.25, 0.019, -sheet_d * 0.25); // lowest corner on the sheet
    fold1.rotation().z = 0.30;
    fold1.rotation().x = 0.15;
    l.add(&fold1);
    let fold2 = mesh(pg("Letters__scrap", box_geo!(sheet_w * 0.35, 0.003, sheet_d * 0.30)), &paper_mat);
    fold2.position().set(-sheet_w * 0.20, 0.011, sheet_d * 0.25);
    fold2.rotation().z = -0.20;
    fold2.rotation().x = -0.10;
    l.add(&fold2);
    // Optional envelope nearby (60%).
    if rnd() < 0.6 {
        let env_colors = [0xf0e8d0, 0xf0d8c0, 0xe8dcc0];
        let env_mat = lambert!({ color: pick(&env_colors) });
        // Torn-open envelope: base + tilted flap + edge tear.
        let env = group();
        let env_base = mesh(pg("Letters__envelope", box_geo!(0.10, 0.002, 0.06)), &env_mat);
        env_base.position().y = 0.001;
        env.add(&env_base);
        // Flap tilted up (opened): hinge edge on the envelope's back edge.
        let flap = mesh(pg("Letters__flap", box_geo!(0.10, 0.002, 0.04)), &env_mat);
        flap.position().set(0, 0.0165, -0.043);
        flap.rotation().x = 0.8;
        env.add(&flap);
        // Small red stamp.
        let stamp_mat = lambert!({ color: 0xa02040 });
        let stamp = mesh(pg("Letters__stamp", box_geo!(0.012, 0.003, 0.015)), &stamp_mat);
        stamp.position().set(0.037, 0.0035, 0.020);
        env.add(&stamp);
        env.position().set(0.16, 0, 0.02);
        env.rotation().y = (rnd() - 0.5) * 0.6;
        l.add(&env);
    }
    l.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    l.rotation().y = rnd() * PI * 2.0;
    g.world.add(&l);
}

// R694 open diary w/ pen on page (floor): 5% chance per level.
fn place_open_diaries(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Diary cover colors.
    let cover_colors = [0x603020, 0x201050, 0x804020, 0x2a4030, 0x505050];
    let cover_mat = lambert!({ color: pick(&cover_colors) });
    let page_mat = lambert!({ map: diary_page_texture() });
    let d = group();
    let (book_w, book_d) = (0.11, 0.15);
    // Both pages laid flat (open flat on floor).
    let page_base = mesh(pg("OpenDiaries__pages", box_geo!(book_w * 2.0, 0.006, book_d)), &page_mat);
    page_base.position().y = 0.008;
    d.add(&page_base);
    // Spine slightly raised.
    let spine = mesh(pg("OpenDiaries__spine", cyl_geo!(0.003, 0.003, book_d, 4)), &cover_mat);
    spine.rotation().x = PI / 2.0;
    spine.position().set(0, 0.013, 0);
    d.add(&spine);
    // Left cover flap (thin plate under left page).
    let left_cover = mesh(pg("OpenDiaries__cover", box_geo!(book_w * 1.05, 0.003, book_d * 1.05)), &cover_mat);
    left_cover.position().set(-book_w * 0.5, 0.003, 0);
    d.add(&left_cover);
    // Right cover flap.
    let right_cover = mesh(pg("OpenDiaries__cover", box_geo!(book_w * 1.05, 0.003, book_d * 1.05)), &cover_mat);
    right_cover.position().set(book_w * 0.5, 0.003, 0);
    d.add(&right_cover);
    // Ribbon bookmark hanging out — thin colored strip.
    let ribbon_colors = [0xa02040, 0x2060a0, 0xc0a040, 0x603030];
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors) });
    let ribbon = mesh(pg("OpenDiaries__ribbon", box_geo!(0.008, 0.002, 0.06)), &ribbon_mat);
    // Draped from the page top over the edge down to the floor.
    ribbon.rotation().x = 0.184;
    ribbon.position().set(0.02, 0.0065, book_d * 0.5 + 0.0245);
    d.add(&ribbon);
    // Pen resting diagonally on page — cylinder + tip cone + button.
    let pen_colors = [0x101010, 0x2060a0, 0x603030, 0xc0a040];
    let pen_mat = lambert!({ color: pick(&pen_colors) });
    let pen = group();
    let pen_body = mesh(pg("OpenDiaries__pen", cyl_geo!(0.004, 0.004, 0.10, 6)), &pen_mat);
    pen_body.rotation().z = PI / 2.0;
    pen.add(&pen_body);
    let pen_tip = mesh(pg("OpenDiaries__tip", cone_geo!(0.003, 0.010, 5)), &pen_mat);
    pen_tip.rotation().z = PI / 2.0; // apex outward (-x), base on the barrel end
    pen_tip.position().x = -0.055;
    pen.add(&pen_tip);
    let pen_button = mesh(pg("OpenDiaries__button", cyl_geo!(0.005, 0.005, 0.006, 5)), &pen_mat);
    pen_button.rotation().z = PI / 2.0;
    pen_button.position().x = 0.053;
    pen.add(&pen_button);
    let pen_clip = mesh(pg("OpenDiaries__clip", box_geo!(0.020, 0.001, 0.004)), &pen_mat);
    pen_clip.position().set(0.020, 0.005, 0);
    pen.add(&pen_clip);
    pen.position().set(book_w * 0.35, 0.016, book_d * 0.15);
    pen.rotation().y = -0.6;
    d.add(&pen);
    d.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    d.rotation().y = rnd() * PI * 2.0;
    g.world.add(&d);
}

// R694 telephone off-hook w/ dial tone hum (floor): 4% chance per level.
fn place_off_hook_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.off_hook_phone_positions = vec![]; // R694 reset per level
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let phone_colors = [0x101010, 0xa02020, 0x2060a0, 0x604020, 0xc0a060];
    let phone_mat = lambert!({ color: pick(&phone_colors) });
    let dial_mat = lambert!({ color: 0xe0e0d0 });
    let cord_mat = lambert!({ color: 0x201010 });
    let p = group();
    // Base — rounded rectangular box.
    let base = mesh(pg("OffHookPhones__base", box_geo!(0.14, 0.028, 0.10)), &phone_mat);
    base.position().y = 0.014;
    p.add(&base);
    // Dial disc — flat cylinder on top.
    let dial = mesh(pg("OffHookPhones__dial", cyl_geo!(0.038, 0.038, 0.006, 12)), &dial_mat);
    dial.position().set(-0.02, 0.031, 0);
    p.add(&dial);
    // Dial center hole.
    let dial_hole = mesh(pg("OffHookPhones__hub", cyl_geo!(0.006, 0.006, 0.007, 8)), &phone_mat);
    dial_hole.position().set(-0.02, 0.032, 0);
    p.add(&dial_hole);
    // 10 finger holes around dial edge.
    for i in 0..10 {
        let ang = (i as f64 / 10.0) * PI * 2.0;
        let hole = mesh(pg("OffHookPhones__fingerhole", cyl_geo!(0.0038, 0.0038, 0.007, 5)), &phone_mat);
        hole.position().set(-0.02 + ang.cos() * 0.028, 0.033, ang.sin() * 0.028);
        p.add(&hole);
    }
    // 2 hook cradle bumps on top (empty — handset is off).
    for off in [-0.045, 0.045] {
        let bump = mesh(pg("OffHookPhones__prong", sphere_geo!(0.005, 5, 4)), &phone_mat);
        bump.position().set(0.050, 0.030, off);
        p.add(&bump);
    }
    // Handset — lying beside base.
    let (handset_off_x, handset_off_z) = (0.15, 0.06);
    let handset = group();
    // Earpiece cylinder + shaft + mouthpiece cylinder.
    let earpiece = mesh(pg("OffHookPhones__earcup", cyl_geo!(0.015, 0.015, 0.020, 8)), &phone_mat);
    earpiece.rotation().z = PI / 2.0;
    earpiece.position().set(-0.045, 0, 0);
    handset.add(&earpiece);
    let shaft = mesh(pg("OffHookPhones__grip", box_geo!(0.08, 0.014, 0.020)), &phone_mat);
    shaft.position().set(0, 0, 0);
    handset.add(&shaft);
    let mouthpiece = mesh(pg("OffHookPhones__mouthcup", cyl_geo!(0.015, 0.015, 0.020, 8)), &phone_mat);
    mouthpiece.rotation().z = PI / 2.0;
    mouthpiece.position().set(0.045, 0, 0);
    handset.add(&mouthpiece);
    // Small speaker holes on earpiece + mouthpiece (tiny dark discs).
    let speaker_mat = lambert!({ color: 0x0a0a0a });
    for (xoff, dir) in [(-0.055, -1.0), (0.055, 1.0)] {
        let speaker = mesh(circle_geo!(0.010, 8), &speaker_mat);
        speaker.rotation().y = dir * PI / 2.0;
        speaker.position().set(xoff, 0, 0);
        handset.add(&speaker);
    }
    handset.position().set(handset_off_x, 0.015, handset_off_z);
    handset.rotation().y = 0.4;
    p.add(&handset);
    // Curly cord connecting phone to handset — 8 tiny torus rings in a chain.
    let (cord_start_x, cord_start_z) = (0.06, 0.045);
    for i in 0..8 {
        let t = i as f64 / 7.0;
        let cx = cord_start_x + (handset_off_x - cord_start_x - 0.04) * t;
        let cz = cord_start_z + (handset_off_z - cord_start_z) * t;
        let cy = 0.010 + (t * PI).sin() * 0.010;
        let ring = mesh(pg("OffHookPhones__cord", torus_geo!(0.005, 0.001, 3, 8)), &cord_mat);
        ring.rotation().z = PI / 2.0;
        ring.position().set(cx, cy, cz);
        p.add(&ring);
    }
    p.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    p.rotation().y = rnd() * PI * 2.0;
    g.world.add(&p);
    // Track position — for handset dial-tone audio.
    let (pp, ry) = (p.pos(), p.rot().y);
    let handset_world_x = pp.x + handset_off_x * ry.cos() - handset_off_z * ry.sin();
    let handset_world_z = pp.z + handset_off_x * ry.sin() + handset_off_z * ry.cos();
    g.pp.off_hook_phone_positions.push(v3(handset_world_x, 0, handset_world_z));
}

// R695 chess board mid-game (floor): 4% chance per level.
fn place_chess_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let board_base = if rnd() < 0.5 { 0x603020 } else { 0xa07040 };
    let board_mat = lambert!({ color: board_base });
    let light_square_mat = lambert!({ color: 0xe8dcb8 });
    let dark_square_mat = lambert!({ color: 0x604020 });
    let white_piece_mat = lambert!({ color: 0xf0e8d0 });
    let black_piece_mat = lambert!({ color: 0x201010 });
    let cb = group();
    // Board base — flat box.
    let board_size = 0.24;
    let board = mesh(pg("ChessBoards__board", box_geo!(board_size + 0.02, 0.010, board_size + 0.02)), &board_mat);
    board.position().y = 0.005;
    cb.add(&board);
    // 8×8 alternating squares.
    let cell_size = board_size / 8.0;
    for r in 0..8 {
        for c in 0..8 {
            let is_light = (r + c) % 2 == 0;
            let sq = mesh(
                pg("ChessBoards__square", box_geo!(cell_size * 0.98, 0.002, cell_size * 0.98)),
                if is_light { &light_square_mat } else { &dark_square_mat },
            );
            sq.position().set(
                -board_size / 2.0 + cell_size / 2.0 + c as f64 * cell_size,
                0.011,
                -board_size / 2.0 + cell_size / 2.0 + r as f64 * cell_size,
            );
            cb.add(&sq);
        }
    }
    // Piece builder — simplified silhouettes.
    let piece_h = cell_size * 0.7;
    let make_piece = |kind: &str, mat: &Mat| -> Obj {
        let pg_ = group();
        // Base disc.
        let base = mesh(pg("ChessBoards__base", cyl_geo!(cell_size * 0.35, cell_size * 0.4, piece_h * 0.15, 6)), mat);
        base.position().y = piece_h * 0.075;
        pg_.add(&base);
        // Body cylinder (varies per piece).
        if kind == "pawn" {
            let body = mesh(pg("ChessBoards__pawnbody", cyl_geo!(cell_size * 0.18, cell_size * 0.25, piece_h * 0.55, 6)), mat);
            body.position().y = piece_h * 0.5;
            pg_.add(&body);
            let top = mesh(pg("ChessBoards__ball", sphere_geo!(cell_size * 0.20, 6, 5)), mat);
            top.position().y = piece_h * 0.85;
            pg_.add(&top);
        } else if kind == "rook" {
            let body = mesh(pg("ChessBoards__rookbody", cyl_geo!(cell_size * 0.22, cell_size * 0.28, piece_h * 0.75, 6)), mat);
            body.position().y = piece_h * 0.5;
            pg_.add(&body);
            // 4 crenellation blocks on top.
            for i in 0..4 {
                let ang = (i as f64 / 4.0) * PI * 2.0;
                let cren = mesh(pg("ChessBoards__cren", box_geo!(cell_size * 0.10, piece_h * 0.15, cell_size * 0.10)), mat);
                cren.position().set(ang.cos() * cell_size * 0.15, piece_h * 0.90, ang.sin() * cell_size * 0.15);
                pg_.add(&cren);
            }
        } else if kind == "bishop" {
            let body = mesh(pg("ChessBoards__bishopbody", cyl_geo!(cell_size * 0.15, cell_size * 0.25, piece_h * 0.65, 6)), mat);
            body.position().y = piece_h * 0.5;
            pg_.add(&body);
            let cap = mesh(pg("ChessBoards__mitre", cone_geo!(cell_size * 0.20, piece_h * 0.35, 6)), mat);
            cap.position().y = piece_h * 1.0;
            pg_.add(&cap);
        } else if kind == "knight" {
            let body = mesh(pg("ChessBoards__knightbody", cyl_geo!(cell_size * 0.18, cell_size * 0.25, piece_h * 0.55, 6)), mat);
            body.position().y = piece_h * 0.4;
            pg_.add(&body);
            // Angled head (leaning forward box).
            let head = mesh(pg("ChessBoards__knighthead", box_geo!(cell_size * 0.18, piece_h * 0.35, cell_size * 0.30)), mat);
            head.rotation().x = -0.3;
            head.position().set(0, piece_h * 0.85, cell_size * 0.05);
            pg_.add(&head);
        } else if kind == "queen" {
            let body = mesh(pg("ChessBoards__queenbody", cyl_geo!(cell_size * 0.18, cell_size * 0.28, piece_h * 0.70, 6)), mat);
            body.position().y = piece_h * 0.5;
            pg_.add(&body);
            let crown = mesh(pg("ChessBoards__crown", sphere_geo!(cell_size * 0.22, 6, 5)), mat);
            crown.position().y = piece_h * 0.95;
            pg_.add(&crown);
            // 4 crown spikes.
            for i in 0..4 {
                let ang = (i as f64 / 4.0) * PI * 2.0;
                let spike = mesh(pg("ChessBoards__spike", cone_geo!(cell_size * 0.03, cell_size * 0.12, 4)), mat);
                spike.position().set(ang.cos() * cell_size * 0.14, piece_h * 1.10, ang.sin() * cell_size * 0.14);
                pg_.add(&spike);
            }
        } else if kind == "king" {
            let body = mesh(pg("ChessBoards__kingbody", cyl_geo!(cell_size * 0.18, cell_size * 0.28, piece_h * 0.75, 6)), mat);
            body.position().y = piece_h * 0.5;
            pg_.add(&body);
            let cap = mesh(pg("ChessBoards__kingcap", sphere_geo!(cell_size * 0.20, 6, 5)), mat);
            cap.position().y = piece_h * 0.95;
            pg_.add(&cap);
            // Cross on top.
            let cross1 = mesh(box_geo!(cell_size * 0.03, cell_size * 0.14, cell_size * 0.03), mat);
            cross1.position().y = piece_h * 1.15;
            pg_.add(&cross1);
            let cross2 = mesh(box_geo!(cell_size * 0.10, cell_size * 0.03, cell_size * 0.03), mat);
            cross2.position().y = piece_h * 1.12;
            pg_.add(&cross2);
        }
        pg_
    };
    // Place ~10-14 pieces in mid-game arrangement (heavier in middle 4 rows).
    let kinds = ["pawn", "pawn", "pawn", "pawn", "rook", "knight", "bishop", "queen", "king"];
    let mut positions: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let piece_count = 10 + (rnd() * 5.0).floor() as i32;
    for i in 0..piece_count {
        let (mut r, mut c) = (0, 0);
        let mut attempts = 0;
        loop {
            // Bias to middle rows (rows 2-5).
            r = 2 + (rnd() * 4.0).floor() as i32;
            c = (rnd() * 8.0).floor() as i32;
            attempts += 1;
            if !(positions.contains(&(r, c)) && attempts < 15) {
                break;
            }
        }
        if positions.contains(&(r, c)) {
            continue;
        }
        positions.insert((r, c));
        let kind = kinds[(rnd() * kinds.len() as f64).floor() as usize];
        let is_white = rnd() < 0.5;
        let piece = make_piece(kind, if is_white { &white_piece_mat } else { &black_piece_mat });
        piece.position().set(
            -board_size / 2.0 + cell_size / 2.0 + c as f64 * cell_size,
            0.012,
            -board_size / 2.0 + cell_size / 2.0 + r as f64 * cell_size,
        );
        piece.rotation().y = if kind == "knight" {
            if is_white {
                0.0
            } else {
                PI
            }
        } else {
            0.0
        };
        cb.add(&piece);
    }
    // Optional: 1-2 captured pieces on the side board area.
    if rnd() < 0.7 {
        let cap_count = 1 + (rnd() * 2.0).floor() as i32;
        for i in 0..cap_count {
            let is_white = rnd() < 0.5;
            let cap_kind = ["pawn", "bishop", "knight"][(rnd() * 3.0).floor() as usize];
            let cap = make_piece(cap_kind, if is_white { &white_piece_mat } else { &black_piece_mat });
            // Lying on side (Z-rotation 90°).
            cap.rotation().z = PI / 2.0;
            cap.position().set(
                board_size / 2.0 + 0.03 + i as f64 * 0.02,
                cell_size * 0.40, // rests on its base rim (base r = cell_size * 0.4)
                (rnd() - 0.5) * 0.05,
            );
            cb.add(&cap);
        }
    }
    cb.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    cb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cb);
}

// R695 hairdryer w/ cord + running hum audio (floor): 5% chance per level.
fn place_hairdryers(g: &mut Game, _dist: &[i32]) {
    g.pp.hairdryer_positions = vec![]; // R695 reset per level
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let body_colors = [0xf0d0e0, 0xa02040, 0x2060a0, 0x505050, 0x101010, 0xe0c088];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let grid_mat = lambert!({ color: 0x0a0a0a });
    let cord_mat = lambert!({ color: 0x101010 });
    let hd = group();
    // Barrel — main body cylinder lying on side.
    let (barrel_r, barrel_l) = (0.045, 0.14);
    let barrel = mesh(pg("Hairdryers__barrel", cyl_geo!(barrel_r, barrel_r, barrel_l, 12)), &body_mat);
    barrel.rotation().z = PI / 2.0;
    // The dryer stands on its handle's butt: the barrel rests on the handle top.
    barrel.position().set(0, barrel_r + 0.075, 0);
    hd.add(&barrel);
    // Front nozzle — tapered cone at one end.
    let nozzle = mesh(pg("Hairdryers__nozzle", cyl_geo!(barrel_r * 0.7, barrel_r, 0.025, 12)), &body_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(barrel_l / 2.0 + 0.012, barrel_r + 0.075, 0);
    hd.add(&nozzle);
    // Nozzle grille (dark circle).
    let grille = mesh(circle_geo!(barrel_r * 0.65, 12), &grid_mat);
    grille.rotation().y = PI / 2.0;
    grille.position().set(barrel_l / 2.0 + 0.025, barrel_r + 0.075, 0);
    hd.add(&grille);
    // Back grille — larger dark circle at opposite end.
    let back_grille = mesh(circle_geo!(barrel_r * 0.85, 12), &grid_mat);
    back_grille.rotation().y = -PI / 2.0;
    back_grille.position().set(-barrel_l / 2.0 - 0.001, barrel_r + 0.075, 0);
    hd.add(&back_grille);
    // Handle — angled cylinder underneath, protruding down.
    let handle = mesh(pg("Hairdryers__handle", cyl_geo!(0.018, 0.020, 0.075, 8)), &body_mat);
    handle.rotation().z = -0.15;
    handle.position().set(-barrel_l * 0.15, 0.040, 0); // butt on the floor, top 5 mm into the barrel
    hd.add(&handle);
    // Power switch — small red box on handle.
    let switch_mat = lambert!({ color: 0xa02020 });
    let power_switch = mesh(pg("Hairdryers__switch", box_geo!(0.010, 0.016, 0.006)), &switch_mat);
    power_switch.rotation().z = -0.15;
    // On the handle's +Z side face, 2 mm into the grip.
    power_switch.position().set(-barrel_l * 0.15 + 0.002, 0.060, 0.0175);
    hd.add(&power_switch);
    // Cord — 12-segment curved chain trailing away from handle base.
    let cord_segs = 12;
    let mut prev = v3(-barrel_l * 0.15, 0.010, 0);
    // End point far off to one side, arcing on floor.
    let end_x = -barrel_l * 0.15 - 0.30 + rnd() * 0.20;
    let end_z = (rnd() - 0.5) * 0.30;
    for i in 0..cord_segs {
        let t = (i + 1) as f64 / cord_segs as f64;
        // Curved path — sine bump along arc.
        let bump_amt = (t * PI).sin() * 0.03;
        let cx = -barrel_l * 0.15 + (end_x - -barrel_l * 0.15) * t + (t * PI * 2.0).sin() * bump_amt;
        let cz = 0.0 + (end_z - 0.0) * t + (t * PI * 2.0).cos() * bump_amt * 0.7;
        // Humps up and settles on the floor.
        let cy = 0.003 + 0f64.max((t * PI * 1.5).sin()) * 0.008;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("Hairdryers__cord", cyl_geo!(0.003, 0.003, dir_len, 4)), &cord_mat);
            orient_seg(&seg, &prev, &next);
            hd.add(&seg);
        }
        prev = next;
    }
    // Plug at cord end.
    let plug = mesh(pg("Hairdryers__plug", box_geo!(0.018, 0.010, 0.014)), &cord_mat);
    plug.position().set(end_x, 0.007, end_z);
    hd.add(&plug);
    // 2 prong pins.
    let prong_mat = lambert!({ color: 0x808080 });
    for side in [-1.0, 1.0] {
        let prong = mesh(pg("Hairdryers__prong", box_geo!(0.010, 0.002, 0.003)), &prong_mat);
        prong.position().set(end_x + 0.010, 0.007, end_z + side * 0.003);
        hd.add(&prong);
    }
    hd.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    hd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hd);
    // Track dryer position for hum audio.
    let hp = hd.pos();
    g.pp.hairdryer_positions.push(v3(hp.x, 0, hp.z));
}

// R696 half-drunk wine glass w/ liquid pool (floor): 5% chance per level.
fn place_wine_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Wine color pool: red, dark, rose, white.
    let wine_colors = [0x601020, 0x400820, 0xa02040, 0xd0a040];
    let wine_color = pick(&wine_colors);
    let wine_mat = lambert!({ color: wine_color, transparent: true, opacity: 0.85 });
    // Glass — very transparent.
    let glass_mat = lambert!({ color: 0xf0f0f8, transparent: true, opacity: 0.20 });
    let grp = group();
    let is_spilled = rnd() < 0.35;
    // Stem base (foot) — flat disc.
    let foot = mesh(cyl_geo!(0.030, 0.032, 0.004, 12), &glass_mat);
    foot.position().y = 0.002;
    grp.add(&foot);
    // Stem cylinder.
    let stem = mesh(cyl_geo!(0.003, 0.003, 0.055, 6), &glass_mat);
    stem.position().y = 0.032;
    grp.add(&stem);
    // Bowl — sphere w/ Y-scale (traditional wine goblet cup).
    let bowl = mesh(sphere_geo!(0.028, 10, 8), &glass_mat);
    bowl.scale().y = 1.15;
    bowl.position().y = 0.075;
    grp.add(&bowl);
    // the floor pool of a spilled glass
    let mut spill_pool: Option<Obj> = None;
    if !is_spilled {
        let wine_level = 0.65 + rnd() * 0.10; // 65-75% full = half-drunk
        let wine = mesh(cyl_geo!(0.024, 0.020, 0.024 * wine_level, 10), &wine_mat);
        wine.position().y = 0.075 + 0.024 * wine_level / 2.0 - 0.020;
        grp.add(&wine);
        // Small liquid surface disc (dark) — implies flat surface.
        let surface = mesh(circle_geo!(0.023, 10), &wine_mat);
        surface.rotation().x = -PI / 2.0;
        surface.position().y = 0.075 + 0.024 * wine_level / 2.0 - 0.008;
        grp.add(&surface);
    } else {
        // Spilled variant: glass tilted on side + liquid pool on floor.
        grp.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        // Restore to floor level.
        grp.position().y = 0.030;
        // Liquid pool — flattened ellipse on floor next to glass.
        let pool_mat = basic_mat!({ color: wine_color, transparent: true, opacity: 0.65 });
        // Pool goes on world (not tilted group) so orient separately.
        let pool = mesh(circle_geo!(0.055, 14), &pool_mat);
        pool.rotation().x = -PI / 2.0;
        pool.scale().set(1.4, 1.0, 1.0);
        spill_pool = Some(pool);
    }
    // Optional lipstick smudge on rim (35%).
    if rnd() < 0.35 && !is_spilled {
        let lipstick_mat = basic_mat!({ color: 0xa03040, transparent: true, opacity: 0.65 });
        let smudge = mesh(circle_geo!(0.007, 5), &lipstick_mat);
        smudge.position().set(0.024, 0.100, 0);
        smudge.rotation().y = PI / 2.0;
        grp.add(&smudge);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let py = grp.pos().y;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    grp.position().set(px, py, pz);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
    // Attach spill pool separately in world (not rotated with glass).
    if is_spilled {
        if let Some(pool) = spill_pool {
            let (gp, ry) = (grp.pos(), grp.rot().y);
            pool.position().set(gp.x + 0.06 * ry.cos(), 0.001, gp.z + 0.06 * ry.sin());
            pool.rotation().z = rnd() * PI * 2.0;
            g.world.add(&pool);
        }
    }
}

// R696 sewing machine on stand (floor): 3% chance per level.
fn place_sewing_machines(g: &mut Game, _dist: &[i32]) {
    g.pp.sewing_machine_positions = vec![]; // R699 reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Machine body colors — vintage black/dark green w/ gold accents.
    let body_colors = [0x101010, 0x1a2618, 0x181410, 0x201810];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let metal_mat = lambert!({ color: 0xc0a060 });
    let needle_mat = lambert!({ color: 0xd0d0d0 });
    let fabric_mat = lambert!({ color: [0xf0c0c0, 0xa0c0f0, 0xf0e0a0, 0xf0f0f0, 0xd0e0c0][(rnd() * 5.0).floor() as usize] });
    let m = group();
    // Wooden base plate.
    let (base_w, base_d) = (0.24, 0.14);
    let base = mesh(pg("SewingMachines__base", box_geo!(base_w, 0.014, base_d)), &wood_mat);
    base.position().y = 0.007;
    m.add(&base);
    // Machine bed — flat horizontal block on top of wood base.
    let bed = mesh(pg("SewingMachines__bed", box_geo!(base_w * 0.85, 0.010, base_d * 0.65)), &body_mat);
    bed.position().set(0, 0.020, 0);
    m.add(&bed);
    // Left column (vertical support).
    let column = mesh(pg("SewingMachines__column", box_geo!(0.025, 0.10, base_d * 0.55)), &body_mat);
    column.position().set(-base_w * 0.35, 0.075, 0);
    m.add(&column);
    // Top horizontal arm.
    let arm = mesh(pg("SewingMachines__arm", box_geo!(base_w * 0.75, 0.026, base_d * 0.55)), &body_mat);
    arm.position().set(0, 0.115 + 0.013, 0);
    m.add(&arm);
    // Presser foot descending from arm's right end.
    let foot = mesh(pg("SewingMachines__foot", box_geo!(0.014, 0.022, 0.030)), &body_mat);
    // Shank stretched into the presser bar: from the arm's underside down to the cloth.
    foot.scale().y = 3.91;
    foot.position().set(base_w * 0.32, 0.072, 0);
    m.add(&foot);
    // Needle — very thin cylinder descending, beside the presser bar's shank.
    let needle = mesh(cyl_geo!(0.001, 0.001, 0.020, 4), &needle_mat);
    needle.scale().y = 4.4;
    needle.position().set(base_w * 0.32 - 0.0055, 0.074, 0);
    m.add(&needle);
    // R697 needle up-down oscillator — 40% chance (machine "running").
    // R699: track running machines for stitch-click audio.
    let is_running = rnd() < 0.4;
    if is_running {
        let needle_base_y = 0.074;
        let needle_phase = rnd() * PI * 2.0;
        let needle2 = needle.clone();
        g.animate(move |_g, _dt, t| {
            // Fast up-down oscillation ~4 Hz w/ ±3 mm amplitude.
            needle2.position().y = needle_base_y + (t * 8.0 + needle_phase).sin() * 0.003;
        });
    }
    // Note: the `sewing_machine_positions` push happens after world placement below.
    m.user().set("isRunning", is_running);
    // Thread spool on top of arm.
    let spool_mat = lambert!({ color: 0xd8b878 });
    let spool_thread_colors = [0xa02040, 0x2060a0, 0x101010, 0xf0f0f0, 0x408060];
    let spool_thread_mat = lambert!({ color: pick(&spool_thread_colors) });
    let spool_base = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.003, 8)), &spool_mat);
    spool_base.position().set(-base_w * 0.20, 0.155, 0);
    m.add(&spool_base);
    let spool_thread = mesh(pg("SewingMachines__thread", cyl_geo!(0.012, 0.012, 0.028, 8)), &spool_thread_mat);
    spool_thread.position().set(-base_w * 0.20, 0.172, 0);
    m.add(&spool_thread);
    let spool_top = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.003, 8)), &spool_mat);
    spool_top.position().set(-base_w * 0.20, 0.187, 0);
    m.add(&spool_top);
    // Thread trailing from spool to needle — oriented-segment chain.
    let thread_path = [
        v3(-base_w * 0.20, 0.187, 0), // Top of spool.
        v3(-base_w * 0.10, 0.155, 0), // Guide 1.
        v3(base_w * 0.10, 0.135, 0),  // Guide 2.
        v3(base_w * 0.32, 0.100, 0),  // Needle top.
    ];
    for i in 0..thread_path.len() - 1 {
        let (p1, p2) = (thread_path[i], thread_path[i + 1]);
        let dir_len = p2.distance_to(&p1);
        let seg = mesh(cyl_geo!(0.0008, 0.0008, dir_len, 3), &spool_thread_mat);
        orient_seg(&seg, &p1, &p2);
        m.add(&seg);
    }
    // Hand crank wheel on right side (large disc).
    let wheel = mesh(pg("SewingMachines__wheel", cyl_geo!(0.028, 0.028, 0.008, 12)), &metal_mat);
    wheel.rotation().z = PI / 2.0;
    wheel.position().set(base_w * 0.42, 0.125, 0);
    m.add(&wheel);
    // Wheel handle knob.
    let knob = mesh(pg("SewingMachines__knob", sphere_geo!(0.005, 5, 4)), &wood_mat);
    knob.position().set(base_w * 0.46, 0.145, 0);
    m.add(&knob);
    // Piece of fabric under presser foot — mid-stitch (small rectangle).
    let fabric = mesh(pg("SewingMachines__cloth", box_geo!(0.06, 0.002, 0.05)), &fabric_mat);
    fabric.position().set(base_w * 0.22, 0.028, 0);
    m.add(&fabric);
    // Gold trim lines on the arm's front/back faces, short of its rounded ends.
    let trim1 = mesh(box_geo!(base_w * 0.75, 0.002, 0.002), &metal_mat);
    trim1.scale().x = 0.9;
    trim1.position().set(0, 0.128, 0.0395);
    m.add(&trim1);
    let trim2 = mesh(box_geo!(base_w * 0.75, 0.002, 0.002), &metal_mat);
    trim2.scale().x = 0.9;
    trim2.position().set(0, 0.128, -0.0395);
    m.add(&trim2);
    m.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    m.rotation().y = rnd() * PI * 2.0;
    g.world.add(&m);
    // R699 register position only if this machine is "running" (has needle animator).
    if is_running {
        let mp = m.pos();
        g.pp.sewing_machine_positions.push(v3(mp.x, 0, mp.z));
    }
}

// R697 opened pill bottle w/ scattered pills (floor): 5% chance per level.
fn place_pill_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Bottle color pool — amber, orange, dark blue, brown.
    let bottle_colors = [0xc07030, 0xa04010, 0x304060, 0x603010];
    let bottle_mat = lambert!({ color: pick(&bottle_colors), transparent: true, opacity: 0.75 });
    let cap_colors = [0xffffff, 0xa02020, 0x101010, 0xf0e0a0];
    let cap_mat = lambert!({ color: pick(&cap_colors) });
    let label_mat = lambert!({ color: 0xf0e8d0 });
    // Pill colors — variety per bottle.
    let pill_colors = [0xf0f0f0, 0xf0d0a0, 0xa0c0f0, 0xf0c0c0, 0xd0f0d0, 0xf0e0a0, 0x603030, 0x202020];
    let pill_color1 = pick(&pill_colors);
    let pill_color2 = pick(&pill_colors);
    let pill_mat1 = lambert!({ color: pill_color1 });
    let pill_mat2 = lambert!({ color: pill_color2 });
    let pb = group();
    // Main bottle body — squat cylinder.
    let (bottle_r, bottle_h) = (0.020, 0.055);
    let body = mesh(cyl_geo!(bottle_r, bottle_r, bottle_h, 10), &bottle_mat);
    body.position().y = bottle_h / 2.0 + 0.002;
    pb.add(&body);
    // Label wrapping most of body — slightly larger radius, cream colored.
    let label = mesh(pg("PillBottles__label", cyl_geo!(bottle_r + 0.0005, bottle_r + 0.0005, bottle_h * 0.7, 10, 1, true)), &label_mat);
    label.position().y = bottle_h / 2.0 + 0.002;
    pb.add(&label);
    // Thin dark medication text lines on label — 3 horizontal boxes.
    let text_mat = lambert!({ color: 0x101010 });
    for i in 0..3 {
        let text_line = mesh(box_geo!(bottle_r * 1.5, 0.002, 0.001), &text_mat);
        text_line.position().set(0, bottle_h * 0.75 - i as f64 * 0.010, bottle_r + 0.001);
        pb.add(&text_line);
    }
    // Cap — tilted off to the side (open).
    let cap = mesh(pg("PillBottles__cap", cyl_geo!(bottle_r + 0.001, bottle_r + 0.001, 0.010, 10)), &cap_mat);
    // Cap and tabs are tipped on their side together; the group height
    // puts the tilted cap's rim on the floor.
    let cap_tilt = (rnd() - 0.5) * 0.3;
    let cap_g = group();
    cap_g.rotation().z = PI / 2.0 + cap_tilt;
    cap_g.position().set(0.048, (bottle_r + 0.001) * cap_tilt.cos() + 0.005 * cap_tilt.sin().abs(), 0.02);
    cap_g.add(&cap);
    pb.add(&cap_g);
    // Cap top (child-proof push tabs).
    for i in 0..4 {
        let ang = (i as f64 / 4.0) * PI * 2.0;
        let tab = mesh(pg("PillBottles__tab", box_geo!(0.002, 0.008, 0.004)), &cap_mat);
        // Radial ribs on the cap's skirt, 45 deg off so no rib is under the tipped cap.
        let t_ang = ang + PI / 4.0;
        tab.position().set(t_ang.cos() * (bottle_r + 0.0015), 0, t_ang.sin() * (bottle_r + 0.0015));
        tab.rotation().y = -t_ang;
        // The old per-tab tilt draw is kept (unused) so the later random draws stay the same.
        let _ = rnd();
        cap_g.add(&tab);
    }
    // Scattered pills — 5-12 tiny elongated pill-shaped meshes.
    let pill_count = 5 + (rnd() * 8.0).floor() as i32;
    for i in 0..pill_count {
        // 50/50 pill type — capsule (2 colors) or tablet (short cylinder).
        let is_capsule = rnd() < 0.5;
        let pill = group();
        if is_capsule {
            // 2-tone capsule: 2 short cylinders + rounded ends.
            let half_len = 0.004;
            let half_a = mesh(pg("PillBottles__half", cyl_geo!(0.002, 0.002, half_len, 6)), &pill_mat1);
            half_a.rotation().z = PI / 2.0;
            half_a.position().set(-half_len / 2.0, 0, 0);
            pill.add(&half_a);
            let half_b = mesh(pg("PillBottles__half", cyl_geo!(0.002, 0.002, half_len, 6)), &pill_mat2);
            half_b.rotation().z = PI / 2.0;
            half_b.position().set(half_len / 2.0, 0, 0);
            pill.add(&half_b);
            // Rounded caps.
            let cap_a = mesh(pg("PillBottles__end", sphere_geo!(0.002, 5, 4)), &pill_mat1);
            cap_a.position().set(-half_len, 0, 0);
            pill.add(&cap_a);
            let cap_b = mesh(pg("PillBottles__end", sphere_geo!(0.002, 5, 4)), &pill_mat2);
            cap_b.position().set(half_len, 0, 0);
            pill.add(&cap_b);
        } else {
            // Round tablet — flat cylinder.
            let tab_mat = if rnd() < 0.5 { &pill_mat1 } else { &pill_mat2 };
            let tablet = mesh(pg("PillBottles__tablet", cyl_geo!(0.003, 0.003, 0.002, 6)), tab_mat);
            pill.add(&tablet);
        }
        // Random scatter position.
        let ang = rnd() * PI * 2.0;
        let rr = 0.02 + rnd() * 0.09;
        pill.position().set(ang.cos() * rr, 0.002, ang.sin() * rr);
        pill.rotation().y = rnd() * PI * 2.0;
        pill.rotation().z = (rnd() - 0.5) * 0.5;
        pb.add(&pill);
        // R721 subtle wobble animator — 30% chance per pill.
        if rnd() < 0.3 {
            let wobble_phase = rnd() * PI * 2.0;
            let base_rot_z = pill.rot().z;
            let base_rot_y = pill.rot().y;
            let pill2 = pill.clone();
            g.animate(move |_g, _dt, t| {
                pill2.rotation().z = base_rot_z + (t * 1.6 + wobble_phase).sin() * 0.03;
                pill2.rotation().y = base_rot_y + (t * 1.2 + wobble_phase * 1.3).sin() * 0.02;
            });
        }
    }
    pb.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pb);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
