//! Prop placers: occult props (open journals, tarot fans, chalk pentagrams,
//! hourglasses, ouija boards, ornate mirrors, crows, candelabras, chain links),
//! ceiling fans, flower stems, and the bathroom set (bath mats, hair dryers,
//! dispensers, shampoo shelves, towel bars, stall walls, toothbrush holders,
//! switch plates, vent grates and small toiletries).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

pub mod textures;
use textures::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_open_journals", place_open_journals));
    r.push(("place_tarot_card_fans", place_tarot_card_fans));
    r.push(("place_pentagram_chalks", place_pentagram_chalks));
    r.push(("place_small_hourglasses", place_small_hourglasses));
    r.push(("place_ouija_boards", place_ouija_boards));
    r.push(("place_ornate_wall_mirrors", place_ornate_wall_mirrors));
    r.push(("place_small_crows", place_small_crows));
    r.push(("place_small_candelabras", place_small_candelabras));
    r.push(("place_small_chain_links", place_small_chain_links));
    r.push(("place_ceiling_fans", place_ceiling_fans));
    r.push(("place_flower_stems", place_flower_stems));
    r.push(("place_bath_mats", place_bath_mats));
    r.push(("place_hair_dryers", place_hair_dryers));
    r.push(("place_toilet_paper_dispensers", place_toilet_paper_dispensers));
    r.push(("place_shampoo_shelves", place_shampoo_shelves));
    r.push(("place_towel_bars", place_towel_bars));
    r.push(("place_hair_brushes", place_hair_brushes));
    r.push(("place_stall_walls", place_stall_walls));
    r.push(("place_toothbrush_holders", place_toothbrush_holders));
    r.push(("place_single_earrings", place_single_earrings));
    r.push(("place_nail_clippers", place_nail_clippers));
    r.push(("place_washcloths", place_washcloths));
    r.push(("place_switch_plates", place_switch_plates));
    r.push(("place_shower_caps", place_shower_caps));
    r.push(("place_hair_ties", place_hair_ties));
    r.push(("place_contact_lens_cases", place_contact_lens_cases));
    r.push(("place_cotton_swabs", place_cotton_swabs));
    r.push(("place_cotton_balls", place_cotton_balls));
    r.push(("place_vent_grates", place_vent_grates));
    r.push(("place_hand_towel_dispensers", place_hand_towel_dispensers));
}

// ------------------------------------------------------------------ helpers
/// Every open interior tile, row by row (the `open_tiles` loop every placer repeats).
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
/// A uniformly random element (one `rnd()` draw).
fn pick<T: Copy>(v: &[T]) -> T {
    v[(rnd() * v.len() as f64).floor() as usize]
}

/// A wall-facing slot: open tile (x, z) with a wall at (x + dx, z + dz).
#[derive(Clone, Copy)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// Every (open tile, wall direction) pair whose neighbour is a wall or off the grid.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// Open tiles with their FIRST adjacent wall tile (`tiles[..] == 1`; later sides are skipped):
/// the `wall_tiles` loop of the bathroom placers.
fn wall_tiles_first(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push(WallCand { x, z, dx, dz });
                        break;
                    }
                }
            }
        }
    }
    v
}

/// `atan2(-dx, -dz)` with float negation, so a zero component stays `-0.0` (kept on purpose).
fn face_wall_yaw(dx: i32, dz: i32) -> f64 {
    (-(dx as f64)).atan2(-(dz as f64))
}

/// A single-triangle geometry with computed normals.
fn tri_geo(v: [f64; 9]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(vec![0, 1, 2]);
    geo.compute_vertex_normals();
    geo
}

// R471 small open journal: 6% chance per level, optional quill.
fn place_open_journals(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let cover_mat = lambert!({ color: 0x603820 });
    let page_mat = lambert!({ map: journal_texture(), side: DoubleSide });
    let journal = group();
    let (book_w, book_h) = (0.20, 0.14);
    // Cover under pages.
    let cover = mesh(pg("OpenJournals__cover", box_geo!(book_w + 0.008, 0.005, book_h + 0.008)), &cover_mat);
    cover.position().y = 0.0025;
    journal.add(&cover);
    // Pages plane.
    let pages = mesh(plane_geo!(book_w, book_h), &page_mat);
    pages.rotation().x = -PI / 2.0;
    pages.position().y = 0.0055;
    journal.add(&pages);
    // Small quill on top of journal (60% chance).
    if rnd() < 0.60 {
        let quill_mat = lambert!({ color: 0xf0ece0, side: DoubleSide });
        let shaft_mat = lambert!({ color: 0x201a10 });
        // Lies flat on the pages along (cos 30, 0, sin 30), nib (-Y) end first.
        let shaft = mesh(pg("OpenJournals__quill", cyl_geo!(0.001, 0.001, 0.16, 4)), &shaft_mat);
        shaft.rotation().set(0, -PI / 6.0, -PI / 2.0);
        shaft.position().set(0.02, 0.0065, -0.01);
        journal.add(&shaft);
        // Feather plane.
        let feather = mesh(plane_geo!(0.014, 0.08), &quill_mat);
        feather.rotation().x = -PI / 2.0;
        feather.rotation().z = PI / 3.0;
        feather.position().set(0.02 + 0.04 * (PI / 6.0).cos(), 0.006, -0.01 + 0.04 * (PI / 6.0).sin());
        journal.add(&feather);
    }
    journal.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    journal.rotation().y = rnd() * PI * 2.0;
    g.world.add(&journal);
    // R487 track for paper rustle audio (shared with music sheets).
    let jp = journal.pos();
    g.pp.paper_positions.push(v3(jp.x, 0.01, jp.z));
}

// R470 tarot card fan: 5% chance per level, 5 cards with 1 face-up.
fn place_tarot_card_fans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let back_mat = lambert!({ map: tarot_back_texture(), side: DoubleSide });
    let face_mat = lambert!({ map: tarot_face_texture(), side: DoubleSide });
    let cards = group();
    let (card_w, card_h, card_t) = (0.055, 0.090, 0.0012);
    let n_cards = 5;
    for i in 0..n_cards {
        // 1 face-up in center, others face-down fanned.
        let is_face_up = i == 2;
        let mat = if is_face_up { &face_mat } else { &back_mat };
        let card = mesh(plane_geo!(card_w, card_h), mat);
        card.rotation().x = -PI / 2.0;
        // Fan angle.
        let fan_angle = -PI / 8.0 + (i as f64 / (n_cards - 1) as f64) * (PI / 4.0);
        card.rotation().z = fan_angle;
        // Position along arc.
        card.position().set(fan_angle.sin() * 0.045, 0.004 + i as f64 * 0.0005, -fan_angle.cos() * 0.045 + 0.045);
        cards.add(&card);
    }
    cards.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cards.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cards);
}

// R470 pentagram chalk decal: 3% chance per level, faint pulsing red glow.
fn place_pentagram_chalks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let mat = basic_mat!({ map: pentagram_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(1.4, 1.4), &mat);
    decal.rotation().x = -PI / 2.0;
    decal.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0.005, g.wz(z) + (rnd() - 0.5) * 1.5);
    decal.rotation().z = rnd() * PI * 2.0;
    g.world.add(&decal);
    // R486 faint red mystical glow rising from pentagram center.
    let glow = point_light!(0xa02040, 0.30, 0.9, 2);
    let dp = decal.pos();
    glow.position().set(dp.x, 0.20, dp.z);
    g.world.add(&glow);
    // Slow pulsing.
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = (t * 0.9 + pulse_phase).sin() * 0.5 + 0.5;
        glow.set_intensity(0.15 + p * 0.20);
    });
}

// R469 small hourglass: 5% chance per level, animated falling stream.
fn place_small_hourglasses(g: &mut Game, _dist: &[i32]) {
    g.pp.hourglass_positions.clear(); // R483 reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let frame_mat = lambert!({ color: 0x8a5030 });
    let glass_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.20 });
    let sand_mat = lambert!({ color: 0xd8b060 });
    let hg = group();
    // Top frame plate.
    let top_plate = mesh(pg("SmallHourglasses__topPlate", cyl_geo!(0.035, 0.035, 0.006, 8)), &frame_mat);
    top_plate.position().y = 0.157;
    hg.add(&top_plate);
    // Bottom frame plate.
    let bot_plate = mesh(pg("SmallHourglasses__botPlate", cyl_geo!(0.035, 0.035, 0.006, 8)), &frame_mat);
    bot_plate.position().y = 0.003;
    hg.add(&bot_plate);
    // 4 corner posts.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("SmallHourglasses__post", cyl_geo!(0.003, 0.003, 0.15, 4)), &frame_mat);
        post.position().set(a.cos() * 0.028, 0.080, a.sin() * 0.028);
        hg.add(&post);
    }
    // The two bulbs meet at their apexes (the waist, y 0.080) and run to the
    // plates: stretched 1.233 in height.
    // Top glass bulb (upside-down cone).
    let top_bulb = mesh(cone_geo!(0.026, 0.060, 12, 1, true), &glass_mat);
    top_bulb.rotation().x = PI;
    top_bulb.scale().y = 1.233;
    top_bulb.position().y = 0.117;
    hg.add(&top_bulb);
    // Bottom glass bulb.
    let bot_bulb = mesh(cone_geo!(0.026, 0.060, 12, 1, true), &glass_mat);
    bot_bulb.scale().y = 1.233;
    bot_bulb.position().y = 0.043;
    hg.add(&bot_bulb);
    // Top sand: turned point-down into the upper bulb's funnel, narrowed to
    // stay inside the glass.
    let top_sand = mesh(pg("SmallHourglasses__sand", cone_geo!(0.020, 0.035, 8)), &sand_mat);
    top_sand.rotation().x = PI;
    top_sand.scale().set(0.6, 1, 0.6);
    top_sand.position().y = 0.0995;
    hg.add(&top_sand);
    // Bottom sand: a heap on the bottom plate.
    let bot_sand = mesh(pg("SmallHourglasses__sand", cone_geo!(0.020, 0.028, 8)), &sand_mat);
    bot_sand.position().y = 0.020;
    hg.add(&bot_sand);
    // Falling sand stream: from the waist (0.080) down to the heap's crest (0.034).
    let stream_mat = lambert!({ color: 0xd8b060 });
    let stream = mesh(cyl_geo!(0.001, 0.001, 0.030, 4), &stream_mat);
    stream.scale().y = 1.53;
    stream.position().y = 0.057;
    hg.add(&stream);
    // Subtle sand shimmer animator.
    let shim_phase = rnd() * PI * 2.0;
    let stream2 = stream.clone();
    g.animate(move |_g, _dt, t| {
        stream2.position().y = 0.057 + (t * 8.0 + shim_phase).sin() * 0.001;
        stream2.rotation().y = t * 2.0;
    });
    hg.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hg);
    // R483 track for sand whoosh audio.
    let hp = hg.pos();
    g.pp.hourglass_positions.push(v3(hp.x, 0.07, hp.z));
}

// R469 ouija board decal: 4% chance per level, drifting planchette.
fn place_ouija_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let board_mat = lambert!({ map: ouija_texture(), side: DoubleSide });
    let board = mesh(plane_geo!(0.64, 0.40), &board_mat);
    board.rotation().x = -PI / 2.0;
    board.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0.004, g.wz(z) + (rnd() - 0.5) * 2.0);
    board.rotation().z = rnd() * PI * 2.0;
    g.world.add(&board);
    // Planchette (small heart-shaped indicator sitting on board).
    let planchette_mat = lambert!({ color: 0xd0d0d0 });
    let pl = mesh(pg("OuijaBoards__planchette", cyl_geo!(0.030, 0.024, 0.012, 3)), &planchette_mat);
    let base_x = board.pos().x + (rnd() - 0.5) * 0.30;
    let base_z = board.pos().z + (rnd() - 0.5) * 0.15;
    pl.position().set(base_x, 0.010, base_z);
    pl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pl);
    // R484 subtle mysterious sway: the planchette slowly drifts as if moved
    // by unseen hands.
    let sway_phase_x = rnd() * PI * 2.0;
    let sway_phase_z = rnd() * PI * 2.0;
    let sway_period = 14.0 + rnd() * 10.0;
    g.animate(move |_g, _dt, t| {
        let mut p = pl.position();
        p.x = base_x + (t * (PI * 2.0 / sway_period) + sway_phase_x).sin() * 0.010;
        p.z = base_z + (t * (PI * 2.0 / sway_period * 1.3) + sway_phase_z).sin() * 0.008;
    });
}

// R468 ornate wall mirror: 5% chance per level, gilded frame, wall-mounted.
fn place_ornate_wall_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let gold_mat = lambert!({ color: 0xd8b040 });
    let mirror_mat = basic_mat!({ color: 0x506878 });
    let mirror = group();
    let (w, h, t) = (0.28, 0.42, 0.020);
    // Outer gold frame (as 4 boxes).
    let frame_w = 0.020;
    for (fw, fh, fx, fy) in [
        (w + frame_w * 2.0, frame_w, 0.0, h / 2.0 + frame_w / 2.0),
        (w + frame_w * 2.0, frame_w, 0.0, -h / 2.0 - frame_w / 2.0),
        (frame_w, h, -w / 2.0 - frame_w / 2.0, 0.0),
        (frame_w, h, w / 2.0 + frame_w / 2.0, 0.0),
    ] {
        let bar = mesh(pg("OrnateWallMirrors__bar", box_geo!(fw, fh, t)), &gold_mat);
        bar.position().set(fx, fy, 0);
        mirror.add(&bar);
    }
    // 4 corner ornaments (spheres).
    for (cx, cy) in [(-1.0f64, 1.0f64), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        let ornament = mesh(pg("OrnateWallMirrors__ornament", sphere_geo!(0.020, 8, 6)), &gold_mat);
        ornament.position().set(cx * (w / 2.0 + frame_w / 2.0), cy * (h / 2.0 + frame_w / 2.0), 0.010);
        mirror.add(&ornament);
    }
    // Center top crown ornament.
    let crown = mesh(pg("OrnateWallMirrors__crown", cone_geo!(0.020, 0.045, 6)), &gold_mat);
    crown.position().set(0, h / 2.0 + frame_w + 0.020, 0);
    mirror.add(&crown);
    // Mirror face (dark reflective panel).
    let face = mesh(plane_geo!(w - 0.005, h - 0.005), &mirror_mat);
    face.position().set(0, 0, t / 2.0 + 0.001);
    mirror.add(&face);
    // Position on wall.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.014);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.014);
    let center_y = 1.60;
    if c.dx.abs() == 1 {
        mirror.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        mirror.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        mirror.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        mirror.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
    g.world.add(&mirror);
}

// R468 small crow silhouette: 5% chance per level.
fn place_small_crows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let crow_mat = lambert!({ color: 0x0a0806 });
    let beak_mat = lambert!({ color: 0x101008 });
    let eye_mat = basic_mat!({ color: 0xf0d020 });
    let crow = group();
    // Body: elongated sphere.
    let body = mesh(pg("SmallCrows__body", sphere_geo!(0.030, 10, 8)), &crow_mat);
    body.scale().set(1.4, 1, 1);
    body.position().y = 0.038;
    crow.add(&body);
    // Head: smaller sphere.
    let head = mesh(pg("SmallCrows__head", sphere_geo!(0.020, 8, 6)), &crow_mat);
    head.position().set(0.038, 0.058, 0);
    crow.add(&head);
    // Beak: cone.
    let beak = mesh(pg("SmallCrows__beak", cone_geo!(0.005, 0.018, 4)), &beak_mat);
    beak.rotation().z = -PI / 2.0;
    beak.position().set(0.062, 0.058, 0);
    crow.add(&beak);
    // 2 folded wings on sides (flat triangles).
    for s in [-1.0f64, 1.0] {
        let wing_geo = tri_geo([-0.025, 0.038, s * 0.020, 0.020, 0.038, s * 0.020, -0.010, 0.055, s * 0.028]);
        let wing = mesh(wing_geo, &crow_mat);
        crow.add(&wing);
    }
    // Tail: 3 short feathers as triangles.
    for i in 0..3 {
        let k = (i - 1) as f64;
        let tail_geo = tri_geo([-0.038, 0.038, k * 0.006, -0.058, 0.036, k * 0.008, -0.038, 0.048, k * 0.006]);
        let tail = mesh(tail_geo, &crow_mat);
        crow.add(&tail);
    }
    // 2 legs (thin cylinders).
    for s in [-1.0f64, 1.0] {
        let leg = mesh(pg("SmallCrows__leg", cyl_geo!(0.002, 0.002, 0.024, 4)), &beak_mat);
        leg.position().set(0.008, 0.012, s * 0.006);
        crow.add(&leg);
    }
    // 2 tiny glowing yellow eyes, on the side of the head: 18 mm out along
    // (0.45, 0.25, ±0.86).
    for s in [-1.0f64, 1.0] {
        let eye = mesh(sphere_geo!(0.0025, 5, 4), &eye_mat);
        eye.position().set(0.038 + 0.018 * 0.449, 0.058 + 0.018 * 0.249, s * 0.018 * 0.858);
        crow.add(&eye);
    }
    crow.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    crow.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crow);
}

// R467 small vintage candelabra: 4% chance per level, 3 lit candles.
fn place_small_candelabras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let wax_mat = lambert!({ color: 0xf0e8d8 });
    let wick_mat = lambert!({ color: 0x2a2418 });
    let cand = group();
    // Base foot (wide disc).
    let base = mesh(pg("SmallCandelabras__base", cyl_geo!(0.045, 0.055, 0.014, 12)), &brass_mat);
    base.position().y = 0.007;
    cand.add(&base);
    // Central column.
    let col = mesh(pg("SmallCandelabras__column", cyl_geo!(0.010, 0.010, 0.14, 6)), &brass_mat);
    col.position().y = 0.084;
    cand.add(&col);
    // Bulges on column.
    for bulg_y in [0.040, 0.080, 0.120] {
        let bulge = mesh(pg("SmallCandelabras__knop", sphere_geo!(0.018, 8, 6)), &brass_mat);
        bulge.scale().set(1, 0.5, 1);
        bulge.position().y = bulg_y;
        cand.add(&bulge);
    }
    // 3 arms extending sideways + central up (the centre cup sits on the
    // column's top, 0.154).
    let arm_positions: [(f64, f64); 3] = [(-0.06, 0.130), (0.06, 0.130), (0.0, 0.157)];
    for &(ax, ay) in arm_positions.iter() {
        // Arm cylinder (thin diagonal from column to holder).
        if ax.abs() > 0.001 {
            let arm_len = hypot2(ax, ay - 0.120);
            let arm = mesh(pg("SmallCandelabras__arm", cyl_geo!(0.004, 0.004, arm_len, 4)), &brass_mat);
            arm.position().set(ax / 2.0, (0.120 + ay) / 2.0, 0);
            // Turn the arm's Y axis onto the column-to-cup direction.
            arm.rotation().z = (ay - 0.120).atan2(ax) - PI / 2.0;
            cand.add(&arm);
        }
        // Holder cup at top of arm.
        let cup = mesh(pg("SmallCandelabras__cup", cyl_geo!(0.012, 0.010, 0.006, 8)), &brass_mat);
        cup.position().set(ax, ay, 0);
        cand.add(&cup);
        // Candle: white wax cylinder.
        let candle_len = 0.05;
        let candle = mesh(pg("SmallCandelabras__candle", cyl_geo!(0.006, 0.006, candle_len, 6)), &wax_mat);
        candle.position().set(ax, ay + candle_len / 2.0 + 0.003, 0);
        cand.add(&candle);
        // Wick.
        let wick = mesh(cyl_geo!(0.001, 0.001, 0.006, 3), &wick_mat);
        wick.position().set(ax, ay + candle_len + 0.006, 0);
        cand.add(&wick);
        // Flame + point light.
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
        let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
        flame.position().set(ax, ay + candle_len + 0.012, 0);
        cand.add(&flame);
        let glow = point_light!(0xffb060, 0.3, 0.8, 2);
        glow.position().set(ax, ay + candle_len + 0.014, 0);
        cand.add(&glow);
        // R484 warm halo sphere per candle.
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.018, 6, 5), &halo_mat);
        halo.position().set(ax, ay + candle_len + 0.014, 0);
        cand.add(&halo);
        // Flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 6.0 + rnd() * 4.0;
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
            glow.set_intensity(0.3 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
            halo.scale().set_scalar(1.0 + wobble * 0.15);
        });
    }
    cand.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    cand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cand);
}

// R467 small chain links: 8% chance per level, 5-8 interlinked torus rings.
fn place_small_chain_links(g: &mut Game, _dist: &[i32]) {
    g.pp.chain_positions.clear(); // R486 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let chain_colors = [0x606870, 0x808088, 0x40484c];
    let chain_mat = lambert!({ color: pick(&chain_colors) });
    let chain = group();
    let n_links = 5 + (rnd() * 4.0).floor() as i32;
    let link_r = 0.024;
    // Rest pose: the on-edge links stand on the floor (centre R + r), the flat
    // links rest across their lower wires (y 0.0155).
    let link_step = link_r * 1.25;
    for i in 0..n_links {
        let link = mesh(pg("SmallChainLinks__link", torus_geo!(link_r, 0.005, 6, 12)), &chain_mat);
        link.position().set(i as f64 * link_step, if i % 2 == 0 { 0.0155 } else { link_r + 0.005 }, 0);
        // Alternate rotation (vertical / horizontal).
        if i % 2 == 0 {
            link.rotation().x = PI / 2.0;
        } else {
            link.rotation().z = PI / 2.0;
        }
        chain.add(&link);
    }
    chain.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    chain.rotation().y = rnd() * PI * 2.0;
    g.world.add(&chain);
    // R486 track for jangle audio.
    let cp = chain.pos();
    g.pp.chain_positions.push(v3(cp.x, 0.02, cp.z));
}

// R567 ceiling fan: 3% chance per level, spinning blades.
fn place_ceiling_fans(g: &mut Game, _dist: &[i32]) {
    g.pp.fan_positions.clear(); // R568 reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let motor_mat = lambert!({ color: 0x504a40 });
    let blade_colors: [u32; 4] = [0x8a5030, 0x201810, 0x606870, 0xe0d8c0];
    let blade_base_color = pick(&blade_colors);
    let fan = group();
    let fan_y = WALL_H - 0.08; // whole fan 2 cm up so the downrod meets the ceiling
                               // Stem to ceiling.
    let stem = mesh(pg("CeilingFans__downrod", cyl_geo!(0.010, 0.010, 0.08, 6)), &motor_mat);
    stem.position().y = WALL_H - 0.04;
    fan.add(&stem);
    // Motor housing (cylinder).
    let motor = mesh(pg("CeilingFans__motor", cyl_geo!(0.045, 0.045, 0.030, 8)), &motor_mat);
    motor.position().y = fan_y;
    fan.add(&motor);
    // Blade group (rotates).
    let blade_group = group();
    for i in 0..4 {
        // R570 per-blade tint variance (±8% brightness for wear/dust).
        let shade = 0.92 + rnd() * 0.16;
        let r2 = to_i32(255f64.min(((blade_base_color >> 16) & 0xff) as f64 * shade));
        let g2 = to_i32(255f64.min(((blade_base_color >> 8) & 0xff) as f64 * shade));
        let b2 = to_i32(255f64.min((blade_base_color & 0xff) as f64 * shade));
        let per_blade_mat = lambert!({ color: (r2 << 16) | (g2 << 8) | b2, side: DoubleSide });
        let blade = mesh(pg("CeilingFans__blade", box_geo!(0.30, 0.008, 0.06)), &per_blade_mat);
        blade.position().x = 0.15;
        blade.rotation().z = 0.14; // R569 8 deg tilt for realism (blade pitch)
        let arm = group();
        arm.rotation().y = (i as f64 / 4.0) * PI * 2.0;
        arm.add(&blade);
        blade_group.add(&arm);
    }
    blade_group.position().y = fan_y - 0.014;
    fan.add(&blade_group);
    // R570 subtle warm downlight from motor (30% of fans get it).
    if rnd() < 0.30 {
        let fan_light = point_light!(0xffe4b5, 0.12, 1.4);
        fan_light.position().y = fan_y - 0.05;
        fan.add(&fan_light);
    }
    // Slow rotation animator (fan spinning).
    let spin_rate = 1.5 + rnd() * 1.5;
    let bg = blade_group.clone();
    g.animate(move |_g, _dt, t| {
        bg.rotation().y = t * spin_rate;
    });
    fan.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    g.world.add(&fan);
    let fp = fan.pos();
    g.pp.fan_positions.push(v3(fp.x, fan_y, fp.z)); // R568 track for whoosh audio
}

// R567 single flower stem: 8% chance per level, gently drifting petals.
fn place_flower_stems(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let stem_mat = lambert!({ color: 0x408040 });
    let bloom_colors = [0xd82040, 0xf0d020, 0xd050a0, 0xe07020];
    let bloom_mat = lambert!({ color: pick(&bloom_colors), side: DoubleSide });
    let center_mat = lambert!({ color: 0xd8b040 });
    let flower = group();
    // Stem laid flat.
    let stem = mesh(pg("FlowerStems__stem", cyl_geo!(0.002, 0.002, 0.20, 4)), &stem_mat);
    stem.rotation().z = PI / 2.0;
    stem.position().set(-0.06, 0.002, 0);
    flower.add(&stem);
    // 2 leaves.
    for i in 0..2 {
        let leaf = mesh(pg("FlowerStems__leaf", plane_geo!(0.030, 0.014)), &stem_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.position().set(-0.08 + i as f64 * 0.06, 0.003, if i == 0 { 0.016 } else { -0.016 });
        leaf.rotation().z = if i == 0 { 0.5 } else { -0.5 };
        flower.add(&leaf);
    }
    // 5 petal circle around center.
    let mut petals: Vec<Obj> = vec![]; // R642 track for drift animator
    for p in 0..5 {
        let a = (p as f64 / 5.0) * PI * 2.0;
        let petal = mesh(circle_geo!(0.014, 6), &bloom_mat);
        petal.rotation().x = -PI / 2.0;
        let base_x = 0.03 + a.cos() * 0.010;
        let base_z = a.sin() * 0.010;
        petal.position().set(base_x, 0.003, base_z);
        petal.user().base_x = base_x;
        petal.user().base_z = base_z;
        petal.user().phase = a + rnd() * 0.3;
        petals.push(petal.clone());
        flower.add(&petal);
    }
    // Center dot.
    let center = mesh(circle_geo!(0.006, 6), &center_mat);
    center.rotation().x = -PI / 2.0;
    center.position().set(0.03, 0.004, 0);
    flower.add(&center);
    flower.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    flower.rotation().y = rnd() * PI * 2.0;
    g.world.add(&flower);
    // R642 gentle petal drift (tiny X/Z oscillation, breathes-like).
    g.animate(move |_g, _dt, t| {
        for p in &petals {
            let (bx, bz, ph) = {
                let n = p.r();
                (n.user.base_x, n.user.base_z, n.user.phase)
            };
            let mut pos = p.position();
            pos.x = bx + (t * 0.4 + ph).sin() * 0.0015;
            pos.z = bz + (t * 0.4 + ph).cos() * 0.0015;
        }
    });
}

// R571 bath mat: 6% chance per level, flat against a wall.
fn place_bath_mats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
      // Prefer tiles with an adjacent wall.
    let open_tiles = wall_tiles_first(g);
    if open_tiles.is_empty() {
        return;
    }
    let c = pick(&open_tiles);
    let mat_colors = [0x4060a0, 0x808080, 0xa07050, 0xc0a880, 0x805060];
    let mat_mat = lambert!({ color: pick(&mat_colors) });
    let mat = mesh(pg("BathMats__mat", box_geo!(0.60, 0.008, 0.40)), &mat_mat);
    mat.position().set(g.wx(c.x) + c.dx as f64 * 0.55, 0.004, g.wz(c.z) + c.dz as f64 * 0.55);
    // Align long side parallel to wall.
    if c.dx.abs() > 0 {
        mat.rotation().y = PI / 2.0;
    }
    g.world.add(&mat);
}

// R571 hair dryer wall mount: 4% chance per level.
fn place_hair_dryers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let bracket_mat = lambert!({ color: 0x606060 });
    let dryer_colors = [0xe0e0e0, 0x404040, 0xc0a0a0];
    let dryer_mat = lambert!({ color: pick(&dryer_colors) });
    let nozzle_mat = lambert!({ color: 0x303030 });
    let dryer = group();
    let mount_y = 1.30;
    // Wall bracket plate. Group origin is 2 cm off the wall: the plate's back
    // is on the wall face and the dryer rests on the plate face.
    let plate = mesh(pg("HairDryers__plate", box_geo!(0.10, 0.14, 0.02)), &bracket_mat);
    plate.position().set(0, mount_y, -0.010);
    dryer.add(&plate);
    let dryer_z = 0.045;
    // Dryer body (hanging horizontally in bracket).
    let body = mesh(pg("HairDryers__body", cyl_geo!(0.045, 0.045, 0.16, 10)), &dryer_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.06, mount_y - 0.02, dryer_z);
    dryer.add(&body);
    // Nozzle end (narrower).
    let nozzle = mesh(pg("HairDryers__nozzle", cyl_geo!(0.030, 0.038, 0.04, 10)), &nozzle_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(0.16, mount_y - 0.02, dryer_z);
    dryer.add(&nozzle);
    // Handle (small perpendicular stem).
    let handle = mesh(pg("HairDryers__handle", cyl_geo!(0.025, 0.025, 0.10, 8)), &dryer_mat);
    handle.position().set(0.06, mount_y - 0.11, dryer_z);
    dryer.add(&handle);
    // Cord loop: a U hanging from the handle's butt, both sleeved ends 4 mm up into it.
    let cord = mesh(pg("HairDryers__cord", torus_geo!(0.025, 0.005, 4, 10, PI)), &nozzle_mat);
    cord.position().set(0.06, mount_y - 0.156, dryer_z);
    cord.rotation().z = PI;
    cord.scale().x = 0.65;
    dryer.add(&cord);
    // Align + push against wall.
    dryer.rotation().y = face_wall_yaw(c.dx, c.dz);
    dryer.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.020), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.020));
    g.world.add(&dryer);
}

// R572 toilet paper dispenser: 5% chance per level, wall-mounted.
fn place_toilet_paper_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let housing_mat = lambert!({ color: 0x606060 });
    let paper_mat = lambert!({ color: 0xf0e8d8 });
    let tp = group();
    let mount_y = 0.85;
    // Housing plate.
    let housing = mesh(pg("ToiletPaperDispensers__housing", box_geo!(0.24, 0.20, 0.14)), &housing_mat);
    housing.position().y = mount_y;
    tp.add(&housing);
    // Visible roll (horizontal cylinder) hanging out of the open bottom: its
    // axis is 1 cm under the housing.
    let roll = mesh(pg("ToiletPaperDispensers__roll", cyl_geo!(0.055, 0.055, 0.10, 12)), &paper_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(0.070, mount_y - 0.11, 0);
    tp.add(&roll);
    // Roll core (dark inner).
    let core = mesh(pg("ToiletPaperDispensers__core", cyl_geo!(0.015, 0.015, 0.11, 8)), &housing_mat);
    core.rotation().z = PI / 2.0;
    core.position().set(0.070, mount_y - 0.11, 0);
    tp.add(&core);
    // Tear bar along the front bottom edge, over the roll.
    let lip = mesh(pg("ToiletPaperDispensers__lip", box_geo!(0.020, 0.010, 0.14)), &housing_mat);
    lip.rotation().y = PI / 2.0;
    lip.position().set(0.050, mount_y - 0.10, 0.062);
    tp.add(&lip);
    tp.rotation().y = face_wall_yaw(c.dx, c.dz);
    tp.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&tp);
}

// R572 shampoo bottle shelf and R578 shower cap: batch `t03`'s (`t03.rs`)
mod t03;
use t03::{place_shampoo_shelves, place_shower_caps};

// R573 towel bar with hanging towel: 5% chance per level.
fn place_towel_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let bar_mat = lambert!({ color: 0x909090 });
    let towel_colors = [0xffffff, 0xf0e0d0, 0x80a0c0, 0xe08070, 0xa0c0a0];
    let towel_mat = lambert!({ color: pick(&towel_colors), side: DoubleSide });
    let towel = group();
    let bar_y = 1.10;
    // Two bracket ends.
    for i in 0..2 {
        let bracket = mesh(pg("TowelBars__bracket", box_geo!(0.03, 0.06, 0.06)), &bar_mat);
        bracket.position().set(if i == 0 { -0.24 } else { 0.24 }, bar_y, 0.03);
        towel.add(&bracket);
    }
    // Metal bar between brackets.
    let bar = mesh(pg("TowelBars__bar", cyl_geo!(0.010, 0.010, 0.48, 8)), &bar_mat);
    bar.rotation().z = PI / 2.0;
    bar.position().set(0, bar_y, 0.06);
    towel.add(&bar);
    // Draped towel (folded over the bar, hanging down).
    let towel_cloth = mesh(plane_geo!(0.40, 0.34), &towel_mat);
    towel_cloth.position().set(0, bar_y - 0.16, 0.075);
    towel.add(&towel_cloth);
    // Slight second panel (back side).
    let towel_back = mesh(plane_geo!(0.40, 0.32), &towel_mat);
    towel_back.position().set(0, bar_y - 0.16, 0.045);
    towel.add(&towel_back);
    towel.rotation().y = face_wall_yaw(c.dx, c.dz);
    // Origin on the wall face, so the bracket rosettes (local z 0) sit on the wall.
    towel.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5));
    g.world.add(&towel);
}

// R573 hair brush on floor: 8% chance per level.
fn place_hair_brushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let handle_colors = [0xd08040, 0x804020, 0x404040, 0xe0c0a0];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let bristle_mat = lambert!({ color: 0x201810 });
    let brush = group();
    // Bristle pad (flat oval-ish rectangle).
    let pad = mesh(pg("HairBrushes__pad", box_geo!(0.06, 0.010, 0.11)), &handle_mat);
    pad.position().set(0, 0.005, 0);
    brush.add(&pad);
    // Bristles layer (thin dark rectangle on top).
    let bristles = mesh(pg("HairBrushes__bristles", box_geo!(0.055, 0.012, 0.10)), &bristle_mat);
    bristles.position().set(0, 0.014, 0);
    brush.add(&bristles);
    // Handle (elongated cylinder): neck 2 mm into the pad, butt on the floor.
    let handle = mesh(pg("HairBrushes__handle", cyl_geo!(0.014, 0.012, 0.10, 6)), &handle_mat);
    handle.rotation().x = PI / 2.0;
    handle.position().set(0, 0.014, 0.103);
    brush.add(&handle);
    brush.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}

// R574 bathroom stall wall: 3% chance per level, free-standing partition.
fn place_stall_walls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let stall_colors = [0xa0a0a8, 0x6a7080, 0x808070, 0x505060];
    let stall_mat = lambert!({ color: pick(&stall_colors), side: DoubleSide });
    let stall = group();
    // Main panel: 1.4 m wide, 1.7 m tall, 15 cm floor gap.
    let panel = mesh(box_geo!(1.40, 1.70, 0.020), &stall_mat);
    panel.position().y = 1.00; // Bottom at 15 cm.
    stall.add(&panel);
    // 2 small support feet.
    let foot_mat = lambert!({ color: 0x404040 });
    for i in 0..2 {
        let foot = mesh(pg("StallWalls__foot", box_geo!(0.10, 0.15, 0.08)), &foot_mat);
        foot.position().set(if i == 0 { -0.55 } else { 0.55 }, 0.075, 0);
        stall.add(&foot);
    }
    stall.rotation().y = rnd() * PI * 2.0;
    stall.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    g.world.add(&stall);
}

// R574 toothbrush holder cup: 6% chance per level, 1-2 toothbrushes.
fn place_toothbrush_holders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let shelf_mat = lambert!({ color: 0xc0b8a8 });
    let cup_colors = [0xffffff, 0x40a0c0, 0xe08080, 0xf0d040];
    let cup_mat = lambert!({ color: pick(&cup_colors) });
    let brush_colors = [0x40a0e0, 0xe04040, 0x60c060, 0xe0c040];
    let bristle_mat = lambert!({ color: 0xf0e8d8 });
    let holder = group();
    let shelf_y = 1.25;
    // Small shelf.
    let shelf = mesh(pg("ToothbrushHolders__shelf", box_geo!(0.16, 0.015, 0.10)), &shelf_mat);
    shelf.position().y = shelf_y;
    holder.add(&shelf);
    // Cup.
    let cup = mesh(pg("ToothbrushHolders__cup", cyl_geo!(0.035, 0.030, 0.10, 10)), &cup_mat);
    cup.position().set(0, shelf_y + 0.058, 0);
    holder.add(&cup);
    // 1-2 toothbrushes sticking up.
    let brush_count = 1 + (rnd() * 2.0).floor() as i32;
    for b in 0..brush_count {
        let b_color = pick(&brush_colors);
        let brush_mat = lambert!({ color: b_color });
        let handle = mesh(pg("ToothbrushHolders__handle", box_geo!(0.008, 0.11, 0.008)), &brush_mat);
        handle.position().set(
            if brush_count == 2 {
                if b == 0 {
                    -0.014
                } else {
                    0.014
                }
            } else {
                0.0
            },
            shelf_y + 0.14,
            if brush_count == 2 {
                if b == 0 {
                    -0.008
                } else {
                    0.008
                }
            } else {
                0.0
            },
        );
        handle.rotation().z = (rnd() - 0.5) * 0.2; // slight tilt
        holder.add(&handle);
        // Bristle head.
        let head = mesh(pg("ToothbrushHolders__head", box_geo!(0.010, 0.020, 0.012)), &bristle_mat);
        head.position().copy(&handle.pos());
        head.position().y += 0.055;
        let hr = handle.rot();
        head.rotation().copy(&hr);
        holder.add(&head);
    }
    holder.rotation().y = face_wall_yaw(c.dx, c.dz);
    holder.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.050), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.050));
    g.world.add(&holder);
}

// R575 single earring: 9% chance per level.
fn place_single_earrings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    } // 9% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let metal_colors = [0xffd700, 0xe0e0e0, 0xd08040];
    let metal_mat = lambert!({ color: pick(&metal_colors) });
    let gem_colors = [0xff4040, 0x4080ff, 0x40ff80, 0xff40ff, 0xffffff];
    let gem_mat = lambert!({ color: pick(&gem_colors) });
    let earring = group();
    // Tiny loop (torus).
    let lp = mesh(pg("SingleEarrings__loop", torus_geo!(0.010, 0.0015, 4, 12)), &metal_mat);
    lp.rotation().x = PI / 2.0;
    lp.position().y = 0.0015;
    earring.add(&lp);
    // Small gem/stud on the loop.
    let gem = mesh(pg("SingleEarrings__gem", octa_geo!(0.005)), &gem_mat);
    gem.position().set(0, 0.005, 0.010);
    earring.add(&gem);
    earring.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    earring.rotation().y = rnd() * PI * 2.0;
    g.world.add(&earring);
}

// R576 nail clippers: 8% chance per level.
fn place_nail_clippers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let metal_mat = lambert!({ color: 0xc0c0c0 });
    let clippers = group();
    // Bottom jaw plate.
    let bottom = mesh(pg("NailClippers__bottom", box_geo!(0.055, 0.007, 0.020)), &metal_mat);
    bottom.position().y = 0.004;
    clippers.add(&bottom);
    // Top jaw plate (slightly shorter).
    let top = mesh(pg("NailClippers__top", box_geo!(0.048, 0.005, 0.018)), &metal_mat);
    top.position().set(0.002, 0.010, 0);
    clippers.add(&top);
    // Lever arm (pivots at rear, extends forward over top).
    let lever = mesh(pg("NailClippers__lever", box_geo!(0.050, 0.004, 0.008)), &metal_mat);
    lever.position().set(0, 0.016, 0);
    clippers.add(&lever);
    // Pivot rivet.
    let rivet = mesh(pg("NailClippers__rivet", cyl_geo!(0.003, 0.003, 0.018, 6)), &metal_mat);
    rivet.position().set(-0.020, 0.010, 0);
    clippers.add(&rivet);
    clippers.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    clippers.rotation().y = rnd() * PI * 2.0;
    g.world.add(&clippers);
}

// R577 dropped washcloth: 8% chance per level.
fn place_washcloths(g: &mut Game, _dist: &[i32]) {
    // A cotton terry washcloth (batch `r18`), dropped: folded over on itself,
    // its corner curled up off it, lying limp; the pile, the flat-woven band
    // and the hem; gone grey with dirt, spotted with mildew, a stain dried
    // in it (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old colour draw: white, peach, blue, coral, sage, grey, each faded
    let cloth_colors = [0xffffff, 0xf0d8c0, 0x80a0c0, 0xe08070, 0xa0c0a0, 0xd0d0d0];
    let c = pick(&cloth_colors);
    let kind = cloth_colors.iter().position(|&k| k == c).unwrap_or(0);
    let salt = (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b) ^ (kind as u32).rotate_left(5);
    let cloth = group();
    for (name, band) in [("Real__washclothTerry", false), ("Real__washclothBorder", true)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            let bb = geo.bounding_box();
            let tex = r18_cloth::tex(kind, band, [bb.min.x, bb.min.z, bb.max.x, bb.max.z], salt);
            cloth.add(&mesh(geo, &lambert!({ map: &tex })));
        }
    }
    cloth.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cloth.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cloth);
}

// R577 dead light switch plate: 7% chance per level (does nothing).
fn place_switch_plates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let plate_mat = lambert!({ color: 0xe8e4d8 });
    let toggle_mat = lambert!({ color: 0xf8f4e8 });
    let screw_mat = lambert!({ color: 0x808080 });
    let sw = group();
    let mount_y = 1.25;
    // Rectangular face plate.
    let plate = mesh(pg("SwitchPlates__plate", box_geo!(0.075, 0.115, 0.008)), &plate_mat);
    plate.position().y = mount_y;
    sw.add(&plate);
    // Toggle switch (random up/down orientation).
    let flipped = rnd() < 0.5;
    let toggle = mesh(pg("SwitchPlates__toggle", box_geo!(0.010, 0.024, 0.014)), &toggle_mat);
    toggle.position().set(0, mount_y, 0.010);
    toggle.rotation().x = if flipped { 0.35 } else { -0.35 };
    sw.add(&toggle);
    // 2 screws (top + bottom): head's underside on the plate face, shank in the plate.
    for i in 0..2 {
        let screw = mesh(pg("SwitchPlates__screw", cyl_geo!(0.003, 0.003, 0.006, 6)), &screw_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(0, mount_y + if i == 0 { 0.048 } else { -0.048 }, 0.0028);
        sw.add(&screw);
    }
    sw.rotation().y = face_wall_yaw(c.dx, c.dz);
    sw.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005));
    g.world.add(&sw);
}

// R578 hair tie band: 10% chance per level.
fn place_hair_ties(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let tie_colors = [0x202020, 0xe04040, 0x4080e0, 0x40c060, 0xe0c040, 0xd050a0];
    let tie_mat = lambert!({ color: pick(&tie_colors) });
    // Squashed torus lying flat.
    let tie = mesh(pg("HairTies__tie", torus_geo!(0.020, 0.004, 5, 14)), &tie_mat);
    tie.rotation().x = PI / 2.0;
    tie.scale().y = 0.5; // slightly squashed
    tie.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0.004, g.wz(z) + (rnd() - 0.5) * 2.5);
    tie.rotation().z = rnd() * PI;
    g.world.add(&tie);
}

// R579 contact lens case: 7% chance per level, L/R colored caps.
fn place_contact_lens_cases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let base_mat = lambert!({ color: 0xf0f0e8 });
    // L=blue R=green (classic contact-lens color convention).
    let l_mat = lambert!({ color: 0x40a0e0 });
    let r_mat = lambert!({ color: 0x40c060 });
    let c_case = group();
    // Base plate.
    let base = mesh(pg("ContactLensCases__base", box_geo!(0.070, 0.012, 0.036)), &base_mat);
    base.position().y = 0.006;
    c_case.add(&base);
    // Left cap.
    let l_cap = mesh(pg("ContactLensCases__cap", cyl_geo!(0.013, 0.013, 0.010, 10)), &l_mat);
    l_cap.position().set(-0.016, 0.017, 0);
    c_case.add(&l_cap);
    // Right cap.
    let r_cap = mesh(pg("ContactLensCases__cap", cyl_geo!(0.013, 0.013, 0.010, 10)), &r_mat);
    r_cap.position().set(0.016, 0.017, 0);
    c_case.add(&r_cap);
    c_case.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    c_case.rotation().y = rnd() * PI * 2.0;
    g.world.add(&c_case);
}

// R579 scattered cotton swabs (Q-tips): 10% chance per level, 4-8 sticks.
fn place_cotton_swabs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let stick_mat = lambert!({ color: 0xe0d8b0 });
    let fluff_mat = lambert!({ color: 0xffffff });
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    for s in 0..count {
        let swab = group();
        // Stick (thin cylinder laid flat).
        let stick = mesh(pg("CottonSwabs__stick", cyl_geo!(0.0012, 0.0012, 0.040, 4)), &stick_mat);
        stick.rotation().z = PI / 2.0;
        swab.add(&stick);
        // 2 cotton tufts (small spheres each end).
        let t1 = mesh(pg("CottonSwabs__tuft", sphere_geo!(0.0035, 6, 4)), &fluff_mat);
        t1.position().x = -0.020;
        swab.add(&t1);
        let t2 = mesh(pg("CottonSwabs__tuft", sphere_geo!(0.0035, 6, 4)), &fluff_mat);
        t2.position().x = 0.020;
        swab.add(&t2);
        swab.position().set((rnd() - 0.5) * 0.12, 0.004, (rnd() - 0.5) * 0.12);
        swab.rotation().y = rnd() * PI * 2.0;
        cluster.add(&swab);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

// R580 cotton ball cluster: 8% chance per level, 3-6 puffs.
fn place_cotton_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let puff_mat = lambert!({ color: 0xf8f8f0 });
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for i in 0..count {
        let r = 0.010 + rnd() * 0.006;
        let puff = mesh(pg("CottonBalls__puff", sphere_geo!(r, 8, 6)), &puff_mat);
        puff.position().set((rnd() - 0.5) * 0.09, r * 0.9, (rnd() - 0.5) * 0.09);
        puff.scale().y = 0.85; // slight squash for rest-on-floor read
        cluster.add(&puff);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

// R580 bathroom vent grate: 5% chance per level, high on the wall.
fn place_vent_grates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let frame_mat = lambert!({ color: 0x707070 });
    let shadow_mat = lambert!({ color: 0x101010 });
    let vent = group();
    let mount_y = 2.20; // High on wall.
                        // Outer frame rectangle.
    let frame = mesh(pg("VentGrates__frame", box_geo!(0.30, 0.20, 0.010)), &frame_mat);
    frame.position().y = mount_y;
    vent.add(&frame);
    // Dark inset (shadow behind louvers).
    let inset = mesh(box_geo!(0.26, 0.16, 0.005), &shadow_mat);
    inset.position().set(0, mount_y, 0.006);
    vent.add(&inset);
    // 6 horizontal louvers.
    for l in 0..6 {
        let louver = mesh(pg("VentGrates__louver", box_geo!(0.26, 0.014, 0.008)), &frame_mat);
        louver.position().set(0, mount_y - 0.07 + l as f64 * 0.028, 0.010);
        louver.rotation().x = -0.4; // Angled downward.
        vent.add(&louver);
    }
    vent.rotation().y = face_wall_yaw(c.dx, c.dz);
    vent.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.006), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.006));
    g.world.add(&vent);
}

// R582 hand towel dispenser: 4% chance per level, visible paper tongue.
fn place_hand_towel_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let body_mat = lambert!({ color: 0x808070 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let disp = group();
    let mount_y = 1.35;
    // Main body (chunky rectangle).
    let body = mesh(pg("HandTowelDispensers__body", box_geo!(0.30, 0.40, 0.14)), &body_mat);
    body.position().y = mount_y;
    disp.add(&body);
    // Bottom slot lip.
    let lip = mesh(pg("HandTowelDispensers__lip", box_geo!(0.24, 0.014, 0.10)), &body_mat);
    lip.position().set(0, mount_y - 0.20, 0.04);
    disp.add(&lip);
    // Hanging paper tongue (visible torn sheet).
    let tongue = mesh(pg("HandTowelDispensers__tongue", plane_geo!(0.20, 0.14)), &paper_mat);
    tongue.position().set(0, mount_y - 0.27, 0.075);
    disp.add(&tongue);
    // Small "PULL" arrow indicator strip, on the cover face.
    let arrow = mesh(pg("HandTowelDispensers__arrow", box_geo!(0.06, 0.010, 0.005)), lambert!({ color: 0x202020 }));
    arrow.position().set(0, mount_y + 0.10, 0.0725);
    disp.add(&arrow);
    disp.rotation().y = face_wall_yaw(c.dx, c.dz);
    disp.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.070), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.070));
    g.world.add(&disp);
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

/// The washcloth of `place_washcloths` (batch `r18`): its terry and its
/// band, as dirty as years on a floor leave them.
mod r18_cloth {
    use super::*;

    /// Terry faded: white gone to cream, peach, blue, coral, sage, grey (sRGB)
    const COTTON: [[f64; 3]; 6] =
        [[0.8, 0.78, 0.71], [0.8, 0.66, 0.55], [0.43, 0.53, 0.62], [0.7, 0.45, 0.39], [0.53, 0.59, 0.49], [0.6, 0.6, 0.57]];

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
    fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// The two nearest distances to a jittered lattice (cells of 1)
    fn worley(seed: u32, x: f64, y: f64) -> (f64, f64) {
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dj in -1..=1 {
            for di in -1..=1 {
                let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                let fx = ii as f64 + hash(seed, ii, jj);
                let fy = jj as f64 + hash(seed ^ 77, ii, jj);
                let d = (fx - x).hypot(fy - y);
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d1, d2)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let (iw, _) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    fn rgb(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }
    fn css(c: [f64; 3], a: f64) -> String {
        format!("rgba({},{},{},{:.3})", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a)
    }

    /// The cloth seen from above (`bb` its part's x and z bounds): the loops
    /// of the pile (or the flat band's weave), the dirt it took up from the
    /// floor in blotches, mildew in dark clusters, one stain dried with a
    /// tide line, grit caught in it
    pub(super) fn tex(kind: usize, band: bool, bb: [f64; 4], salt: u32) -> Tex {
        canvas_tex_opt(256, 192, false, move |ctx, w, h| {
            let seed = salt ^ 0xc107;
            let col = COTTON[kind.min(5)];
            let (sx, sz) = (0.02 * (hash(seed, 1, 1) - 0.5), 0.02 * (hash(seed, 2, 2) - 0.5));
            fill(ctx, w, h, |u, v| {
                let x = bb[0] + u * (bb[2] - bb[0]);
                let z = bb[1] + (1.0 - v) * (bb[3] - bb[1]);
                let mut c = col;
                if band {
                    let weave = 0.5 + 0.5 * (x * 2400.0).sin() * (z * 2400.0).cos();
                    c = mix(c, [col[0] * 0.86, col[1] * 0.86, col[2] * 0.86], 0.3 + 0.25 * weave);
                } else {
                    // the loops: a texel's worth each, light tops and dark between
                    let loopy = hash(seed ^ 3, (x * 1300.0) as i32, (z * 1300.0) as i32);
                    let clump = vn(seed ^ 4, x * 500.0, z * 500.0);
                    c = mix(
                        [col[0] * 0.7, col[1] * 0.7, col[2] * 0.7],
                        [(col[0] * 1.08).min(1.0), (col[1] * 1.08).min(1.0), (col[2] * 1.08).min(1.0)],
                        loopy * 0.6 + clump * 0.4,
                    );
                }
                // dirt taken up from the floor, greyer where it lay
                let dirt = fbm(seed ^ 5, x * 18.0, z * 18.0, 4);
                c = mix(c, [0.36, 0.33, 0.28], 0.18 + (dirt - 0.45).max(0.0) * 1.2);
                // mildew: clusters of dark specks
                let cluster = sstep(0.58, 0.72, fbm(seed ^ 6, x * 9.0, z * 9.0, 3));
                let speck = (vn(seed ^ 7, x * 700.0, z * 700.0) - 0.7).max(0.0) * 3.3;
                c = mix(c, [0.16, 0.17, 0.14], (cluster * speck).min(1.0) * 0.85);
                // a stain: something brown dried in it, darker at its edge
                let d = ((x - 0.03 - sx) / 0.055).hypot((z + 0.05 - sz) / 0.04) + 0.8 * (fbm(seed ^ 8, x * 14.0, z * 14.0, 4) - 0.5);
                c = mix(c, [0.5, 0.4, 0.28], sstep(1.0, 0.6, d) * 0.28 * (0.6 + 0.4 * fbm(seed ^ 10, x * 60.0, z * 60.0, 2)));
                c = mix(c, [0.4, 0.3, 0.2], (-((d - 0.92) / 0.08).powi(2)).exp() * 0.22);
                rgb(c)
            });
        })
    }
}
