//! Prop placers: graduation photos, sandwiches, reading glasses and books, coffee
//! mugs, newspapers, letters, open diaries, off-hook phones, chess boards, hair
//! dryers, wine glasses, sewing machines and pill bottles.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod textures;
use textures::*;
mod t01;
use t01::place_graduation_photos;

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

// R691 reading glasses and books (floor): 5% chance per level. A cloth
// hardback and a paperback on it (the book stack's books, batch r02), a slip
// of paper kept in the hardback for a place, and a pair of plastic reading
// glasses (batch r14b, 50-20-135) folded and put down on the books or beside
// them; the lenses smeared and dusty.
fn place_reading_glasses_and_books(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old draws, call for call: the cover, the frame, how the glasses
    // lie, the bookmark (now a slip of paper) and its colour, where and how
    // it all lies
    let cover_colors = [0x603020, 0x2040a0, 0x408060, 0x804020, 0x202050, 0xa06040];
    let cover_c = pick(&cover_colors);
    let cover = cover_colors.iter().position(|&c| c == cover_c).unwrap_or(0);
    let frame_colors = [0x101010, 0x604020, 0xa02040, 0xc0a040, 0x2060a0];
    let frame_c = pick(&frame_colors);
    let frame = frame_colors.iter().position(|&c| c == frame_c).unwrap_or(0);
    let glasses_turn = (rnd() - 0.5) * 0.8;
    let slip = if rnd() < 0.4 {
        let bookmark_colors = [0xa02040, 0x2060a0, 0xc0a040, 0x408060];
        let c = pick(&bookmark_colors);
        Some(bookmark_colors.iter().position(|&b| b == c).unwrap_or(0))
    } else {
        None
    };
    let gx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let gz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let yaw = rnd() * PI * 2.0;
    let seed = r14b_kit::seed_of(&[gx, gz]);
    let grp = group();
    // the hardback: which of the three, by its cloth (boards, spine, pages,
    // thickness, board width)
    const HARDBACKS: [(&str, &str, &str, f64, f64); 3] = [
        ("Real__bookHbABoards", "Real__bookHbASpine", "Real__bookHbAPages", 0.0352, 0.146),
        ("Real__bookHbBBoards", "Real__bookHbBSpine", "Real__bookHbBPages", 0.0492, 0.164),
        ("Real__bookHbCBoards", "Real__bookHbCSpine", "Real__bookHbCPages", 0.0262, 0.128),
    ];
    let (boards, spine, block, thick, width) = HARDBACKS[cover % 3];
    let pages = lambert!({ map: &r14b_kit::pages_tex(seed) });
    let hb = group();
    let hb_cover = lambert!({ map: &r14b_kit::cloth_cover_tex(cover, seed) });
    let hb_spine = lambert!({ map: &r14b_kit::spine_tex(cover, false, seed) });
    for (name, mat) in [(boards, &hb_cover), (spine, &hb_spine), (block, &pages)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            hb.add(&mesh(geo, mat));
        }
    }
    if let (Some(k), Some(geo)) = (slip, prop_part_as_modelled("Real__bookSlip")) {
        let tint = [0xa8786a, 0x929ca2, 0xc6b68a, 0x9ca48c][k];
        let m = mesh(geo, lambert!({ color: tint }));
        m.position().set(0.03 - 0.06 * f64::from(seed % 7) / 6.0, thick * 0.5, -width / 2.0 + 0.04);
        hb.add(&m);
    }
    grp.add(&hb);
    // the paperback laid on it, askew
    let pb = group();
    let pb_k = ((seed >> 3) % 8) as usize;
    let pb_cover = lambert!({ map: &r14b_kit::print_cover_tex(pb_k, seed) });
    let pb_spine = lambert!({ map: &r14b_kit::spine_tex(pb_k, true, seed) });
    for (name, mat) in [("Real__bookPbCovers", &pb_cover), ("Real__bookPbSpine", &pb_spine), ("Real__bookPbPages", &pages)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            pb.add(&mesh(geo, mat));
        }
    }
    let skew = 0.25 + 0.35 * f64::from((seed >> 6) % 16) / 15.0;
    pb.position().set(0.006 * f64::from((seed >> 10) % 5) - 0.012, thick, 0.008);
    pb.rotation().y = if seed & 2 == 0 { skew } else { -skew };
    grp.add(&pb);
    // the glasses, folded: on the paperback by its spine, or on the floor
    let glasses = group();
    let fr = r14b_kit::frame_mat(frame, seed);
    if let Some(geo) = prop_part_as_modelled("Real__readersFrame") {
        glasses.add(&mesh(geo, &fr));
    }
    if let Some(geo) = prop_part_as_modelled("Real__readersLenses") {
        let lens = mesh(geo, r14b_kit::lens_mat(&r14b_kit::lens_dirt_tex(seed)));
        lens.set_render_order(2);
        glasses.add(&lens);
    }
    if (seed >> 14) % 2 == 0 {
        glasses.position().set(0.0, 0.0236, 0.034);
        glasses.rotation().y = glasses_turn * 0.5 + PI;
        pb.add(&glasses);
    } else {
        glasses.position().set(0.215, 0, 0.04);
        glasses.rotation().y = glasses_turn;
        grp.add(&glasses);
    }
    grp.position().set(gx, 0, gz);
    grp.rotation().y = yaw;
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

// R695 hair dryer: batch `t04`'s (`t04.rs`)
mod t04;
use t04::place_hairdryers;

// R696 a wine glass left on the floor (Blender batch `r16`): a stemmed 8 oz
// glass of the Libbey Citation class (171 mm tall), the wine in it long gone
// down to a dark crust in the bottom of the bowl, dried rings where it stood,
// a lipstick print on the rim now and then; or knocked over, lying on its
// foot's rim and its bowl's belly, the wine dried into the carpet in front
// of its mouth. 5% chance per level.
fn place_wine_glasses(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // which wine it was: red, an older red, rosé, white (the old colours)
    let wine_colors = [0x601020, 0x400820, 0xa02040, 0xd0a040];
    let wine_color = pick(&wine_colors);
    let wine = wine_colors.iter().position(|&c| c == wine_color).unwrap_or(0);
    let is_spilled = rnd() < 0.35;
    // standing: how full it was left (the old fill level, 65-75% of the old
    // bowl), now the dried ring's height; knocked over: the old tilt's draw,
    // how far the wine ran
    let (level, run) = if !is_spilled { (0.65 + rnd() * 0.10, 0.0) } else { (0.0, (rnd() - 0.5) * 0.3) };
    // (35%) a lipstick print on the rim; the draw is made either way
    let lipstick = rnd() < 0.35 && !is_spilled;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let yaw = rnd() * PI * 2.0;
    // what is painted differs floor to floor: from where it stands (no draw)
    let salt = ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32;
    let grp = group();
    if !is_spilled {
        let tide = WINE_TIDE.0 + (WINE_TIDE.1 - WINE_TIDE.0) * ((level - 0.65) / 0.10).clamp(0.0, 1.0);
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassDregs") {
            grp.add(&mesh(geo, phong!({ color: WINE_DREGS[wine], specular: 0x3a3436, shininess: 90 })));
        }
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassFoot") {
            // the foot writes depth: the placement measures the glass by it
            // (a glass that writes none is not there to it, and the dregs
            // alone would read as hovering)
            let foot = mesh(geo, wine_glass_mat(&wine_glass_dust_tex(), 0.5));
            foot.mat().depthWrite(true);
            grp.add(&foot);
        }
        if let Some(geo) = prop_part_as_modelled("Real__wineGlassBowl") {
            let bowl = mesh(geo.clone(), wine_glass_mat(&wine_glass_dust_tex(), 0.45));
            bowl.set_render_order(2);
            grp.add(&bowl);
            // the wine's film and rings on the glass, and the lipstick
            let film = mesh(
                geo,
                phong!({
                    map: &wine_glass_film_tex(wine, tide, lipstick, salt),
                    color: 0xffffff,
                    specular: 0x2a2a2a,
                    shininess: 60,
                    transparent: true,
                    depthWrite: false
                }),
            );
            film.set_render_order(3);
            grp.add(&film);
        }
    } else if let Some(geo) = prop_part_as_modelled("Real__wineGlassLying") {
        let lying = mesh(geo, wine_glass_mat(&wine_glass_dust_tex(), 0.55));
        lying.set_render_order(2);
        grp.add(&lying);
    }
    grp.position().set(px, 0, pz);
    grp.rotation().y = yaw;
    g.world.add(&grp);
    if is_spilled {
        // where it ran out of the mouth (+x of the lying glass) and dried
        // into the floor: a stain, flat on it, its own object as the old pool
        let k = 1.0 + run * 1.6;
        let stain = mesh(
            plane_geo!(0.24 * k, 0.18 * k),
            lambert!({
                map: &wine_stain_tex(wine, salt),
                color: 0xffffff,
                transparent: true,
                depthWrite: false,
                polygonOffset: true,
                polygonOffsetFactor: -2,
                polygonOffsetUnits: -2
            }),
        );
        let reach = 0.10 + 0.04 * k;
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        stain.position().set(px + yaw.cos() * reach, 0.0012, pz - yaw.sin() * reach);
        g.world.add(&stain);
    }
}

/// Where the wine stood in the bowl (`Real__wineGlassBowl`, y in metres):
/// the old fill level's range, a third to a half of the bowl
const WINE_TIDE: (f64, f64) = (0.100, 0.122);
/// The bowl part's height (its bounds: from where the stem joins, 77 mm, to
/// the rim) and the dregs' top (r16.py)
const WINE_BOWL_Y: (f64, f64) = (0.0770, 0.1710);
const WINE_DREGS_TOP: f64 = 0.0905;
/// What each wine leaves in the glass: dried crust (sRGB)
const WINE_DREGS: [u32; 4] = [0x2a0c12, 0x1e0a0e, 0x4a1a1e, 0x6a4a1c];
/// Its dried film and rings (r, g, b) and how strong
const WINE_FILM: [(u8, u8, u8, f64); 4] = [(92, 26, 38, 0.55), (70, 20, 30, 0.6), (140, 64, 70, 0.4), (150, 118, 52, 0.32)];

/// A hash of an integer lattice point
fn wine_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}
/// Smooth value noise in [0, 1]
fn wine_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (wine_hash(seed, xi, yi), wine_hash(seed, xi + 1, yi));
    let (c, d) = (wine_hash(seed, xi, yi + 1), wine_hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Clear glass as the windows' and the mirror's (`Patch::Glass`): the room
/// seen in it by Fresnel, bright at its edges where it is seen at a graze,
/// clear square on; its dust (the map's grey, `dust` of it) lit as dust
fn wine_glass_mat(map: &Tex, dust: f32) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 260, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "wine_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.32;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = dust;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        // two faces of glass: 4% each, square on
        f[13] = 0.07;
    }
    m
}

/// Dust and handling on glass (grey: how much): a haze, heavier blotches,
/// smears. Shared by every part of the glass.
fn wine_glass_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64 / 128.0, j as f64 / 128.0);
                        let d = wine_noise(71, x * 7.0, y * 7.0) * 0.6 + wine_noise(72, x * 29.0, y * 29.0) * 0.4;
                        let smear = (wine_noise(73, x * 3.0, y * 11.0) - 0.6).max(0.0) * 1.5;
                        let v = ((0.18 + 0.45 * d + smear) * 255.0).min(255.0) as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

/// The wine on the bowl, round (u, from +z toward +x) and up (v, over the
/// bowl part's height): its dried film from the crust in the bottom up to
/// the ring where it stood, its tears run down from that ring, fainter rings
/// where it went down in steps; fingerprints' grease; and now and then a
/// lipstick print on the rim. Transparent where there is none.
fn wine_glass_film_tex(wine: usize, tide: f64, lipstick: bool, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let vy = |y: f64| (1.0 - (y - WINE_BOWL_Y.0) / (WINE_BOWL_Y.1 - WINE_BOWL_Y.0)) * h; // canvas y of a height
        let (fr, fg, fb, fa) = WINE_FILM[wine];
        let seed = salt ^ 0x5eed;
        let mut r = SplitMix64::new(salt, 0x7716);
        // the rings it left going down: the top one strongest
        let rings: Vec<(f64, f64)> = (0..3).map(|k| (tide - k as f64 * r.range(0.004, 0.009), [1.0, 0.45, 0.3][k])).collect();
        let (y_top, y_bot) = (vy(tide), vy(WINE_DREGS_TOP));
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let u = i as f64 / w;
                // the film below the ring, thicker toward the bottom and in
                // the tears that ran down
                let wob = (wine_noise(seed ^ 3, u * 22.0, 0.5) - 0.5) * 3.0;
                let mut film = 0.0;
                if y > y_top + wob {
                    let t = ((y - y_top) / (y_bot - y_top).max(1.0)).clamp(0.0, 1.0);
                    let tears = (wine_noise(seed ^ 5, u * 70.0, y / h * 2.5) - 0.62).max(0.0) * 3.0;
                    film = 0.22 + 0.4 * t * t + tears * 0.5;
                }
                for &(ry, k) in &rings {
                    let dy = y - (vy(ry) + wob * k);
                    film += k * 1.2 * (-(dy * dy) / 2.2).exp() * (0.7 + 0.3 * wine_noise(seed ^ 7, u * 30.0, 0.0));
                }
                let a = (film * fa).min(0.92);
                out[i * 4..i * 4 + 4].copy_from_slice(&[fr, fg, fb, (a * 255.0) as u8]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // fingerprints where it was held: soft ovals of grease, ridged
        for _ in 0..3 {
            let (cx, cy) = ((0.3 + 0.4 * r.next()) * w, (0.35 + 0.3 * r.next()) * h);
            ctx.save();
            ctx.translate(cx, cy);
            ctx.rotate(r.range(-0.6, 0.6));
            ctx.scale(1.0, 1.35);
            for k in 0..7 {
                ctx.set_stroke_style(&format!("rgba(196,198,190,{:.3})", 0.16 + 0.06 * (k % 2) as f64));
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.ellipse(0, 0, 3.0 + k as f64 * 1.6, 2.4 + k as f64 * 1.3, 0, 0, PI * 2.0);
                ctx.stroke();
            }
            ctx.restore();
        }
        if lipstick {
            // a lower lip's print on the rim, outside, its crease lines,
            // faded and gone brown-pink
            let cx = (0.18 + 0.08 * r.next()) * w;
            let cy = vy(0.1675);
            ctx.save();
            ctx.translate(cx, cy);
            ctx.scale(1.0, 0.55);
            let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, 16);
            gr.add_color_stop(0, "rgba(140,52,62,0.7)");
            gr.add_color_stop(0.7, "rgba(140,52,62,0.5)");
            gr.add_color_stop(1, "rgba(140,52,62,0)");
            ctx.set_fill_style(gr);
            ctx.begin_path();
            ctx.ellipse(0, 0, 16, 9, 0, 0, PI);
            ctx.fill();
            ctx.set_stroke_style("rgba(214,200,196,0.35)");
            ctx.set_line_width(0.8);
            for k in -5..=5 {
                ctx.begin_path();
                ctx.move_to(k as f64 * 2.6, 0.5);
                ctx.line_to(k as f64 * 2.4 + r.range(-0.6, 0.6), 7.5 - (k as f64).abs() * 0.6);
                ctx.stroke();
            }
            ctx.restore();
        }
    })
}

/// Wine run out of a glass and dried into the floor: a soaked patch of a
/// few lobes round its middle, darker at its edge where it dried last (the
/// ring), its edge wavering a little; a drop or two flung past it.
fn wine_stain_tex(wine: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 192, false, move |ctx, w, h| {
        let (fr, fg, fb, fa) = WINE_FILM[wine];
        let mut r = SplitMix64::new(salt, 0x57a1);
        let seed = salt ^ 0x2bad;
        // the lobes: (x, y, radius) in px, round the middle
        let mut lobes = vec![(w * 0.5, h * 0.5, h * 0.28)];
        for _ in 0..4 {
            let a = r.range(0.0, PI * 2.0);
            let d = h * r.range(0.08, 0.18);
            lobes.push((w * 0.5 + a.cos() * d * 1.3, h * 0.5 + a.sin() * d, h * r.range(0.14, 0.22)));
        }
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                // the stain's field: a smooth union of its lobes (> 0 inside)
                let mut f: f64 = 0.0;
                for &(cx, cy, cr) in &lobes {
                    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / cr;
                    f += (1.0 - d * d).max(0.0).powi(2);
                }
                let edge =
                    f - 0.25 + 0.06 * (wine_noise(seed, x / 26.0, y / 26.0) - 0.5) + 0.02 * (wine_noise(seed ^ 1, x / 6.0, y / 6.0) - 0.5);
                if edge <= 0.0 {
                    out[i * 4 + 3] = 0;
                    continue;
                }
                // the ring where it dried last, and the soaked middle
                let ring = (-(edge / 0.035).powi(2)).exp();
                let body = (edge / 0.08).min(1.0) * (0.5 + 0.25 * wine_noise(seed ^ 3, x / 18.0, y / 18.0));
                let a = (body * 0.75 + ring * 0.55).min(1.0) * fa * 1.3;
                out[i * 4..i * 4 + 4].copy_from_slice(&[fr, fg, fb, (a.min(0.8) * 255.0) as u8]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // a few drops flung past it
        for _ in 0..5 {
            let a = r.range(0.0, PI * 2.0);
            let (x, y) = (w * 0.5 + a.cos() * w * r.range(0.3, 0.45), h * 0.5 + a.sin() * h * r.range(0.3, 0.45));
            let rr = r.range(1.2, 3.0);
            ctx.set_fill_style(&format!("rgba({fr},{fg},{fb},{:.2})", fa));
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, PI * 2.0);
            ctx.fill();
        }
    })
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

/// Batch r14b's kit (the reading glasses and their books): a seed from
/// where they lie, hashed noise, the books' cloth, print and pages, the
/// frame's acetate and the lenses (the book stack's painting, `r02`, done
/// over for one book at a time)
mod r14b_kit {
    use game::prelude::*;

    /// A seed from values already fixed (a position): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    fn h01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    /// Smooth value noise in [0, 1]
    fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (h01(seed, i, j), h01(seed, i + 1, j));
        let (c, d) = (h01(seed, i, j + 1), h01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    fn rgb(c: (f64, f64, f64), a: f64) -> String {
        format!("rgba({},{},{},{a})", c.0 as u8, c.1 as u8, c.2 as u8)
    }

    /// The hardback's cloth (sRGB), faded, for the old cover colours:
    /// brown, navy, bottle green, red-brown, dark navy, tan buckram
    const CLOTH: [(f64, f64, f64); 6] =
        [(92.0, 50.0, 40.0), (50.0, 58.0, 82.0), (52.0, 76.0, 60.0), (110.0, 62.0, 44.0), (40.0, 44.0, 62.0), (150.0, 126.0, 94.0)];
    /// A paperback's printed ground and band
    const PRINT: [((f64, f64, f64), (f64, f64, f64)); 8] = [
        ((172.0, 118.0, 76.0), (204.0, 194.0, 170.0)),
        ((86.0, 106.0, 88.0), (202.0, 192.0, 168.0)),
        ((80.0, 96.0, 118.0), (200.0, 190.0, 168.0)),
        ((198.0, 190.0, 168.0), (128.0, 64.0, 54.0)),
        ((112.0, 60.0, 54.0), (198.0, 186.0, 160.0)),
        ((72.0, 98.0, 100.0), (200.0, 190.0, 168.0)),
        ((158.0, 136.0, 82.0), (52.0, 48.0, 44.0)),
        ((118.0, 116.0, 110.0), (196.0, 188.0, 170.0)),
    ];
    /// Titles and authors, none real
    const TITLES: [&str; 12] = [
        "THE QUIET YEARS",
        "A FIELD GUIDE TO MOSSES",
        "LETTERS HOME",
        "THE LONG CORRIDOR",
        "COLLECTED STORIES",
        "NIGHT TRAINS",
        "THE ATLAS OF ROOMS",
        "MIDDLEMARCH ROAD",
        "BIRDS IN WINTER",
        "A HOUSE OF DOORS",
        "THE LAST TENANT",
        "SEA AND SKY",
    ];
    const AUTHORS: [&str; 8] =
        ["E. L. BRANT", "MARGARET ASH", "J. W. COLE", "H. PEMBER", "A. DORAN", "R. LISLE", "C. M. FROST", "O. TARRANT"];

    fn title(seed: u32, k: u32) -> (&'static str, &'static str) {
        (TITLES[((seed >> k) % TITLES.len() as u32) as usize], AUTHORS[((seed >> (k + 5)) % AUTHORS.len() as u32) as usize])
    }

    fn weave(ctx: &mut Canvas, r: &mut SplitMix64, w: f64, h: f64, n: usize) {
        for k in 0..n {
            ctx.set_fill_style(if k % 2 == 0 { "rgba(255,245,225,0.10)" } else { "rgba(0,0,0,0.12)" });
            ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), 1.0 + r.next(), 1);
        }
    }

    fn blot(ctx: &mut Canvas, x: f64, y: f64, rad: f64, ring: bool, col: &str) {
        let gr = ctx.create_radial_gradient(x, y, rad * if ring { 0.75 } else { 0.0 }, x, y, rad);
        gr.add_color_stop(0, "rgba(0,0,0,0)");
        gr.add_color_stop(if ring { 0.85 } else { 0.0 }, col);
        gr.add_color_stop(1, "rgba(0,0,0,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(x - rad, y - rad, 2.0 * rad, 2.0 * rad);
    }

    /// A cloth board (its head to the left, its fore-edge up, as the
    /// boards' top takes it): faded toward the fore-edge, the weave, a
    /// blind rule, the title stamped, the corners bumped, a ring where a cup
    /// stood on it
    pub fn cloth_cover_tex(c: usize, seed: u32) -> Tex {
        let (t, _) = title(seed, 1);
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(seed, 0xc0e1);
            let col = CLOTH[c];
            ctx.set_fill_style(rgb(col, 1.0));
            ctx.fill_rect(0, 0, w, h);
            let gr = ctx.create_linear_gradient(0, h, 0, 0);
            gr.add_color_stop(0, "rgba(210,200,180,0.0)");
            gr.add_color_stop(1, &format!("rgba(210,200,180,{:.2})", r.range(0.12, 0.26)));
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h);
            weave(ctx, &mut r, w, h, 1800);
            ctx.set_stroke_style("rgba(0,0,0,0.22)");
            ctx.set_line_width(1.3);
            ctx.stroke_rect(7, 7, 114, 114);
            ctx.save();
            ctx.translate(0, h);
            ctx.rotate(-PI / 2.0);
            ctx.scale(128.0 / 146.0, 128.0 / 222.0);
            ctx.set_fill_style("rgba(170,146,92,0.7)");
            ctx.set_text_align("center");
            ctx.set_font("bold 10px Times New Roman");
            ctx.fill_text_max(t, 73, 50, 100);
            ctx.restore();
            for (x, y) in [(0.0, 0.0), (w, 0.0)] {
                blot(ctx, x, y, r.range(7.0, 13.0), false, "rgba(176,164,140,0.75)");
            }
            for (x, y, ww, hh) in [(0.0, 0.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
                ctx.set_fill_style("rgba(200,190,170,0.22)");
                ctx.fill_rect(x, y, ww, hh);
            }
            let (x, y) = (r.range(30.0, 98.0), r.range(30.0, 98.0));
            blot(ctx, x, y, r.range(16.0, 24.0), true, "rgba(40,30,20,0.28)");
        })
    }

    /// A paperback's front (as the cloth board): its printed ground and
    /// band, title and author, the publisher's mark; creased at the spine,
    /// its edges worn white
    pub fn print_cover_tex(c: usize, seed: u32) -> Tex {
        let (t, a) = title(seed, 9);
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(seed, 0x9b1c);
            let (ground, band) = PRINT[c];
            ctx.save();
            ctx.translate(0, h);
            ctx.rotate(-PI / 2.0);
            ctx.scale(128.0 / 140.0, 128.0 / 216.0);
            ctx.set_fill_style(rgb(ground, 1.0));
            ctx.fill_rect(0, 0, 140, 216);
            ctx.set_fill_style(rgb(band, 1.0));
            ctx.fill_rect(0, 72, 140, 66);
            ctx.set_fill_style(rgb(ground, 0.9));
            ctx.set_text_align("center");
            ctx.set_font("bold 13px Gill Sans");
            let mut parts = t.splitn(2, ' ');
            if let Some(l) = parts.next() {
                ctx.fill_text_max(l, 70, 100, 124);
            }
            if let Some(l) = parts.next() {
                ctx.fill_text_max(l, 70, 117, 124);
            }
            ctx.set_fill_style(rgb(band, 0.9));
            ctx.set_font("10px Gill Sans");
            ctx.fill_text_max(a, 70, 42, 120);
            ctx.begin_path();
            ctx.arc(70, 186, 8, 0, 7);
            ctx.fill();
            ctx.set_stroke_style("rgba(235,228,212,0.7)");
            ctx.set_line_width(1);
            for x in [9.0, 13.0] {
                ctx.begin_path();
                ctx.move_to(x, 0);
                ctx.line_to(x + 1.0, 216);
                ctx.stroke();
            }
            ctx.set_fill_style("rgba(235,228,212,0.35)");
            for (x, y, ww, hh) in [(0.0, 0.0, 140.0, 3.0), (0.0, 213.0, 140.0, 3.0), (137.0, 0.0, 3.0, 216.0)] {
                ctx.fill_rect(x, y, ww, hh);
            }
            ctx.restore();
            for (x, y) in [(0.0, 0.0), (w, 0.0)] {
                blot(ctx, x, y, r.range(6.0, 11.0), false, "rgba(236,230,214,0.7)");
            }
            weave(ctx, &mut r, w, h, 500);
            let (x, y) = (r.range(20.0, 108.0), r.range(20.0, 108.0));
            blot(ctx, x, y, r.range(12.0, 26.0), r.next() < 0.6, "rgba(60,44,24,0.25)");
        })
    }

    /// A spine (head at the left, the front board up): the cloth faded
    /// more than its boards, title and author stamped in gilt, rules at head
    /// and tail; or a paperback's printed spine, cracked where it was read
    pub fn spine_tex(c: usize, paperback: bool, seed: u32) -> Tex {
        let (t, a) = title(seed, if paperback { 9 } else { 1 });
        canvas_tex_opt(256, 32, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(seed, if paperback { 0x5b2 } else { 0x5b1 });
            let (ink, ground) = if paperback {
                let (g0, b0) = PRINT[c];
                ctx.set_fill_style(rgb(g0, 1.0));
                ctx.fill_rect(0, 0, w, h);
                ctx.set_fill_style(rgb(b0, 1.0));
                ctx.fill_rect(70, 0, 110, h);
                (rgb(g0, 0.95), b0)
            } else {
                let col = CLOTH[c];
                let f = r.range(0.18, 0.38);
                let faded = (col.0 + (196.0 - col.0) * f, col.1 + (186.0 - col.1) * f, col.2 + (166.0 - col.2) * f);
                ctx.set_fill_style(rgb(faded, 1.0));
                ctx.fill_rect(0, 0, w, h);
                weave(ctx, &mut r, w, h, 900);
                ctx.set_fill_style("rgba(180,152,92,0.8)");
                for x in [12.0, 15.0, 241.0, 244.0] {
                    ctx.fill_rect(x, 0, 1.2, h);
                }
                ("rgba(184,156,96,0.85)".to_string(), faded)
            };
            ctx.set_fill_style(ink.as_str());
            ctx.set_text_align("left");
            ctx.set_text_baseline("middle");
            ctx.set_font(if paperback { "bold 9px Gill Sans" } else { "bold 9px Times New Roman" });
            ctx.fill_text_max(t, if paperback { 78.0 } else { 30.0 }, 17, if paperback { 96.0 } else { 140.0 });
            ctx.set_font(if paperback { "7px Gill Sans" } else { "7px Times New Roman" });
            if paperback {
                ctx.set_fill_style(rgb(ground, 0.9));
            }
            ctx.fill_text_max(a, if paperback { 186.0 } else { 182.0 }, 17, if paperback { 64.0 } else { 54.0 });
            ctx.set_text_baseline("alphabetic");
            for _ in 0..140 {
                ctx.set_fill_style(rgb(ground, 0.6));
                ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), 1.5, 1.5);
            }
            if paperback {
                ctx.set_stroke_style("rgba(236,230,214,0.75)");
                ctx.set_line_width(0.8);
                for _ in 0..(2.0 + r.next() * 4.0) as usize {
                    let y = r.range(4.0, 28.0);
                    let x = r.range(0.0, 60.0);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + r.range(60.0, 200.0), y + r.range(-1.0, 1.0));
                    ctx.stroke();
                }
            } else {
                for x in [0.0, 250.0] {
                    ctx.set_fill_style("rgba(200,188,166,0.35)");
                    ctx.fill_rect(x, 0, 6, h);
                }
            }
            let gr = ctx.create_linear_gradient(0, 0, 0, 8);
            gr.add_color_stop(0, "rgba(150,140,120,0.35)");
            gr.add_color_stop(1, "rgba(150,140,120,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, 8);
        })
    }

    /// Page edges: yellowed leaves, tanned toward the boards, foxed
    pub fn pages_tex(seed: u32) -> Tex {
        canvas_tex_opt(256, 64, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(seed, 0x9a6e);
            ctx.set_fill_style("#d0c19c");
            ctx.fill_rect(0, 0, w, h);
            let mut y = 0.0;
            while y < h {
                ctx.set_fill_style(&format!("rgba(90,70,40,{:.2})", r.range(0.05, 0.17)));
                ctx.fill_rect(0, y, w, r.range(0.6, 1.2));
                y += r.range(1.0, 2.5);
            }
            for (ya, yb) in [(0.0, 9.0), (h, h - 9.0)] {
                let gr = ctx.create_linear_gradient(0, ya, 0, yb);
                gr.add_color_stop(0, "rgba(120,86,40,0.45)");
                gr.add_color_stop(1, "rgba(120,86,40,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            }
            for _ in 0..40 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                blot(ctx, x, y, r.range(1.0, 3.0), false, "rgba(130,84,40,0.35)");
            }
        })
    }

    /// The frame's acetate: black, dark tortoiseshell, burgundy, honey
    /// tortoiseshell, navy (for the old frame colours); tortoiseshell's
    /// mottle painted, the shine of polished plastic
    pub fn frame_mat(k: usize, seed: u32) -> Mat {
        let tortoise = k == 1 || k == 3;
        if !tortoise {
            let col = [0x161412, 0, 0x3e161c, 0, 0x1c2232][k];
            return phong!({ color: col, specular: 0x303030, shininess: 60 });
        }
        let honey = k == 3;
        let tex = canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64 / w, j as f64 / h);
                    let m = vnoise(seed, x * 9.0, y * 5.0) * 0.6 + vnoise(seed ^ 0x33, x * 23.0, y * 14.0) * 0.4;
                    let spot = ((m - 0.45) * 3.0).clamp(0.0, 1.0);
                    let (a, b) = if honey { ((150.0, 104.0, 46.0), (52.0, 30.0, 14.0)) } else { ((92.0, 54.0, 24.0), (26.0, 16.0, 10.0)) };
                    let px = |u: f64, v: f64| (u + (v - u) * spot) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[px(a.0, b.0), px(a.1, b.1), px(a.2, b.2), 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
        });
        phong!({ map: &tex, specular: 0x303030, shininess: 60 })
    }

    /// The lenses' dirt (grey: how much): a haze, smears, dust
    pub fn lens_dirt_tex(seed: u32) -> Tex {
        canvas_tex_opt(128, 64, false, move |ctx, w, h| {
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64 / w, j as f64 / h);
                    let d = vnoise(seed, x * 7.0, y * 4.0) * 0.6 + vnoise(seed ^ 7, x * 26.0, y * 13.0) * 0.4;
                    let smear = (vnoise(seed ^ 0x55, x * 3.0, y * 2.0) - 0.58).max(0.0) * 1.8;
                    let v = ((0.18 + 0.36 * d + smear) * 255.0).min(255.0) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
        })
    }

    /// The lenses (glass, `Patch::Glass`): the room seen in them by
    /// Fresnel, their dirt from the map's grey
    pub fn lens_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "reading_lens".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.5;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.8;
            f[9] = 0.2;
            f[10] = 0.19;
            f[11] = 0.17;
            f[12] = 0.0;
            f[13] = 0.08;
        }
        m
    }
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
