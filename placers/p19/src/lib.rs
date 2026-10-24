//! Prop placers ported from backrooms.js (chunk p19; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeHandTowelBars", place_hand_towel_bars));
    r.push(("placeElevatorPanels", place_elevator_panels));
    r.push(("placeDroppedClipboards", place_dropped_clipboards));
    r.push(("placeExtinguisherCabinets", place_extinguisher_cabinets));
    r.push(("placeSafetyGlasses", place_safety_glasses));
    r.push(("placeCardReaders", place_card_readers));
    r.push(("placePaperbackBooks", place_paperback_books));
    r.push(("placeMailSlots", place_mail_slots));
    r.push(("placeTapeRolls", place_tape_rolls));
    r.push(("placeWallOutlets", place_wall_outlets));
    r.push(("placeCrumpledReceipts", place_crumpled_receipts));
    r.push(("placeWallDrainGrates", place_wall_drain_grates));
    r.push(("placePostItStacks", place_post_it_stacks));
    r.push(("placeSafetyCones", place_safety_cones));
    r.push(("placeWallDoors", place_wall_doors));
    r.push(("placeSingleSocks", place_single_socks));
    r.push(("placeMotionDetectorDomes", place_motion_detector_domes));
    r.push(("placePeeledPaintPatches", place_peeled_paint_patches));
    r.push(("placeBrokenFlashlights", place_broken_flashlights));
    r.push(("placeCeilingPipes", place_ceiling_pipes));
    r.push(("placeLanyards", place_lanyards));
    r.push(("placeBrokenPens", place_broken_pens));
    r.push(("placeBottleCaps", place_bottle_caps));
    r.push(("placeFuseBoxes", place_fuse_boxes));
    r.push(("placeHangingLightBulbs", place_hanging_light_bulbs));
    r.push(("placeWallIntercoms", place_wall_intercoms));
    r.push(("placePaperclipClusters", place_paperclip_clusters));
    r.push(("placeSprinklerHeads", place_sprinkler_heads));
    r.push(("placeAirFresheners", place_air_fresheners));
    r.push(("placeEmergencyPhones", place_emergency_phones));
    r.push(("placeDroppedCellPhones", place_dropped_cell_phones));
    r.push(("placeWallPipeLeaks", place_wall_pipe_leaks));
    r.push(("placeUnrolledTPRolls", place_unrolled_tp_rolls));
    r.push(("placeEmergencyStrobes", place_emergency_strobes));
}

// ------------------------------------------------------------------ shared scaffolding
// Every placer in this chunk repeats the same two tile scans and the same
// wall-mount tail; they are factored here with identical draws.

/// `{ x, z, dx, dz }`: an open tile with a wall on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallC { x: i32, z: i32, dx: i32, dz: i32 }

/// The wall-adjacent candidate scan (`candidates.push({ x, z, dx, dz })`).
fn wall_candidates(g: &Game) -> Vec<WallC> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallC { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// The open-tile scan (`openTiles.push({ x, z })`).
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// `wallX = wx(c.x) + c.dx * (CELL * 0.5 - inset)`, same for z.
fn wall_xz(g: &Game, c: WallC, inset: f64) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset), g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset))
}

/// The wall-mount tail: face the room, then position, with an optional
/// `(Math.random() - 0.5) * jitter` slide along the wall.
fn mount(o: &Obj, c: WallC, wall_x: f64, wall_z: f64, center_y: f64, jitter: Option<f64>) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let zz = match jitter { Some(j) => wall_z + (rnd() - 0.5) * j, None => wall_z };
        o.position().set(wall_x, center_y, zz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let xx = match jitter { Some(j) => wall_x + (rnd() - 0.5) * j, None => wall_x };
        o.position().set(xx, center_y, wall_z);
    }
}

/// `o.position.set(wx(x) + (r - 0.5) * s, 0, wz(z) + (r - 0.5) * s); o.rotation.y = r * 2π`
fn scatter(g: &Game, o: &Obj, x: i32, z: i32, spread: f64) {
    let px = g.wx(x) + (rnd() - 0.5) * spread;
    let pz = g.wz(z) + (rnd() - 0.5) * spread;
    o.position().set(px, 0, pz);
    o.rotation().y = rnd() * PI * 2.0;
}

// ------------------------------------------------------------------ line 34946
// R562 wall towel bar: 6% chance per level. Chrome bar + 2
// support brackets + hanging cloth towel.
fn place_hand_towel_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let chrome_mat = lambert!({ color: 0xc0c8d0 });
    let towel_colors = [0xf0ece0, 0xf0c0d0, 0xd0e0f0, 0xd0e0d0];
    let towel_mat = lambert!({ color: choice(&towel_colors), side: DoubleSide });
    let bar = group();
    // Chrome bar.
    let bar_cyl = mesh(pg("HandTowelBars__bar", cyl_geo!(0.006, 0.006, 0.36, 6)), &chrome_mat);
    bar_cyl.rotation().z = PI / 2.0;
    bar.add(&bar_cyl);
    // 2 brackets.
    for s in [-1.0, 1.0] {
        let bracket = mesh(pg("HandTowelBars__bracket", box_geo!(0.010, 0.014, 0.020)), &chrome_mat);
        bracket.position().set(s * 0.16, 0, 0);
        bar.add(&bracket);
    }
    // Hanging towel drape.
    let towel = mesh(plane_geo!(0.28, 0.28), &towel_mat);
    towel.position().set(0, -0.14, 0.010);
    bar.add(&towel);
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&bar, c, wall_x, wall_z, 1.15, None);
    g.world.add(&bar);
}

// ------------------------------------------------------------------ line 34999
// R560 elevator button panel: 4% chance per level. Silver panel
// w/ 12-button number grid + emergency red button.
fn place_elevator_panels(g: &mut Game, _dist: &[i32]) {
    g.pp.elevator_panel_positions = vec![];   // R561 reset per level
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let panel_mat = lambert!({ color: 0xa0a0a8 });
    let button_mat = lambert!({ color: 0x181410 });
    let red_mat = basic_mat!({ color: 0xd82040 });
    let panel = group();
    // Plate.
    let plate = mesh(pg("ElevatorPanels__plate", box_geo!(0.10, 0.24, 0.014)), &panel_mat);
    plate.position().set(0, 0, -0.007);
    panel.add(&plate);
    // 4×3 = 12 numbered buttons.
    let led_mat = basic_mat!({ color: 0xf0d040 });
    for by in 0..4 {
        for bx in 0..3 {
            let (bxf, byf) = (bx as f64, by as f64);
            let btn = mesh(pg("ElevatorPanels__button", cyl_geo!(0.008, 0.008, 0.004, 8)), &button_mat);
            btn.rotation().x = PI / 2.0;
            btn.position().set(-0.025 + bxf * 0.025, 0.08 - byf * 0.025, 0.001);
            panel.add(&btn);
            // R561 20% chance yellow LED lit next to button.
            if rnd() < 0.20 {
                let led = mesh(sphere_geo!(0.002, 5, 4), &led_mat);
                led.position().set(-0.025 + bxf * 0.025 + 0.011, 0.08 - byf * 0.025, 0.005);
                panel.add(&led);
            }
        }
    }
    // Red emergency button below.
    let emerg = mesh(cyl_geo!(0.010, 0.010, 0.004, 8), &red_mat);
    emerg.rotation().x = PI / 2.0;
    emerg.position().set(0, -0.09, 0.001);
    panel.add(&emerg);
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&panel, c, wall_x, wall_z, 1.30, Some(0.6));
    g.world.add(&panel);
    // R561 track for beep audio.
    let p = panel.pos();
    g.pp.elevator_panel_positions.push(v3(p.x, 1.30, p.z));
}

// ------------------------------------------------------------------ line 35062
// R560 dropped clipboard: 8% chance per level. Wooden clipboard
// + white paper w/ ink lines + clip.
fn place_dropped_clipboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let board_mat = lambert!({ color: 0x8a5030 });
    let paper_mat = lambert!({ color: 0xf0ece0 });
    let clip_mat = lambert!({ color: 0xc0c0c0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let clipboard = group();
    // Board.
    let board = mesh(pg("DroppedClipboards__board", box_geo!(0.20, 0.006, 0.28)), &board_mat);
    board.position().y = 0.003;
    clipboard.add(&board);
    // Paper on top.
    let paper = mesh(plane_geo!(0.18, 0.26), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.position().y = 0.007;
    clipboard.add(&paper);
    // Metal clip at top.
    let clip = mesh(pg("DroppedClipboards__clip", box_geo!(0.10, 0.008, 0.030)), &clip_mat);
    clip.position().set(0, 0.010, -0.115);
    clipboard.add(&clip);
    // 8 ink lines on paper.
    for i in 0..8 {
        let line = mesh(plane_geo!(0.14, 0.001), &ink_mat);
        line.rotation().x = -PI / 2.0;
        line.position().set(0, 0.008, -0.10 + i as f64 * 0.024);
        clipboard.add(&line);
    }
    scatter(g, &clipboard, x, z, 2.0);
    g.world.add(&clipboard);
}

// ------------------------------------------------------------------ line 35108
// R559 wall extinguisher cabinet: 4% chance per level. Metal
// box w/ glass front + small extinguisher visible.
fn place_extinguisher_cabinets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let frame_mat = lambert!({ color: 0xa02020 });
    let glass_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.35 });
    let ext_mat = lambert!({ color: 0xa02020 });
    let cap_mat = lambert!({ color: 0x181410 });
    let cab = group();
    // Red cabinet back.
    let back = mesh(pg("ExtinguisherCabinets__cabinet", box_geo!(0.16, 0.44, 0.08)), &frame_mat);
    back.position().set(0, 0, -0.04);
    cab.add(&back);
    // Glass front pane.
    let glass = mesh(plane_geo!(0.14, 0.42), &glass_mat);
    glass.position().set(0, 0, 0.001);
    cab.add(&glass);
    // Extinguisher inside.
    let ext = mesh(pg("ExtinguisherCabinets__extinguisher", cyl_geo!(0.032, 0.032, 0.24, 8)), &ext_mat);
    ext.position().set(0, -0.06, -0.034);   // front (z −0.002) stays behind the glass (z 0.001)
    cab.add(&ext);
    // Cap.
    let cap = mesh(pg("ExtinguisherCabinets__valve", cyl_geo!(0.024, 0.032, 0.020, 8)), &cap_mat);
    cap.position().set(0, 0.070, -0.034);   // seated on the cylinder's top (y 0.06)
    cab.add(&cap);
    let (wall_x, wall_z) = wall_xz(g, c, 0.040);
    mount(&cab, c, wall_x, wall_z, 1.30, Some(0.4));
    g.world.add(&cab);
}

// ------------------------------------------------------------------ line 35162
// R559 safety glasses: 8% chance per level. Clear plastic
// frame + tinted lenses + temple arms.
fn place_safety_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let frame_mat = basic_mat!({ color: 0xf0ece0, transparent: true, opacity: 0.55 });
    let lens_mat = basic_mat!({ color: 0xc0e0d0, transparent: true, opacity: 0.5 });
    let glasses = group();
    // 2 lens shields (one big protective plate).
    let shield = mesh(plane_geo!(0.10, 0.03), &lens_mat);
    shield.rotation().x = -PI / 2.0;
    shield.position().y = 0.008;
    glasses.add(&shield);
    // Top frame bar.
    let top_bar = mesh(cyl_geo!(0.002, 0.002, 0.10, 4), &frame_mat);
    top_bar.rotation().z = PI / 2.0;
    top_bar.position().y = 0.005;
    glasses.add(&top_bar);
    // 2 temple arms extending back.
    for s in [-1.0, 1.0] {
        let temple = mesh(cyl_geo!(0.002, 0.002, 0.10, 4), &frame_mat);
        temple.rotation().x = PI / 2.0;
        temple.position().set(s * 0.050, 0.005, -0.05);
        glasses.add(&temple);
    }
    scatter(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}

// ------------------------------------------------------------------ line 35207
// R558 swipe card reader: 4% chance per level. Dark box w/
// slot + status LED.
fn place_card_readers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let body_mat = lambert!({ color: 0x181410 });
    let led_mat = basic_mat!({ color: 0x30d030 });
    let slot_mat = basic_mat!({ color: 0x080604 });
    let reader = group();
    // Body.
    let body = mesh(pg("CardReaders__body", box_geo!(0.08, 0.12, 0.020)), &body_mat);
    body.position().set(0, 0, -0.010);
    reader.add(&body);
    // Card slot (dark horizontal bar).
    let slot = mesh(box_geo!(0.05, 0.006, 0.008), &slot_mat);
    slot.position().set(0, 0.02, 0.001);
    reader.add(&slot);
    // Status LED.
    let led = mesh(sphere_geo!(0.003, 5, 4), &led_mat);
    led.position().set(0.020, -0.02, 0.003);
    reader.add(&led);
    // Slow LED blink.
    let blink_phase = rnd() * PI * 2.0;
    {
        let led = led.clone();
        g.animate(move |_g, _dt, t| {
            led.set_visible((t * 1.5 + blink_phase).sin() > 0.6);
        });
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&reader, c, wall_x, wall_z, 1.30, Some(0.6));
    g.world.add(&reader);
}

// ------------------------------------------------------------------ line 35259
// R558 paperback book: 8% chance per level. Open book laid on
// floor w/ visible page lines.
fn place_paperback_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let cover_colors = [0xa02040, 0x2050c0, 0x30a030, 0xd8b040];
    let cover_mat = lambert!({ color: choice(&cover_colors) });
    let page_mat = lambert!({ color: 0xe8dcc0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let book = group();
    let (bw, bh, bd) = (0.14, 0.18, 0.008);
    // Face-down cover base.
    let cover = mesh(pg("PaperbackBooks__book", box_geo!(bw, bd, bh)), &cover_mat);
    cover.position().y = bd / 2.0;
    book.add(&cover);
    // 60% open (pages showing spread).
    if rnd() < 0.60 {
        // Page spread on top.
        let pages = mesh(plane_geo!(bw - 0.004, bh - 0.004), &page_mat);
        pages.rotation().x = -PI / 2.0;
        pages.position().y = bd + 0.0005;
        book.add(&pages);
        // Center spine crease.
        let crease = mesh(box_geo!(0.001, 0.001, bh - 0.006), &ink_mat);
        crease.position().y = bd + 0.001;
        book.add(&crease);
        // 8 ink lines per side.
        for side in 0..2 {
            for l in 0..8 {
                let line = mesh(plane_geo!(bw * 0.4, 0.001), &ink_mat);
                line.rotation().x = -PI / 2.0;
                line.position().set(
                    (if side == 0 { -1.0 } else { 1.0 }) * (bw / 4.0),
                    bd + 0.0015,
                    -bh / 3.0 + l as f64 * 0.014,
                );
                book.add(&line);
            }
        }
    }
    scatter(g, &book, x, z, 2.4);
    g.world.add(&book);
}

// ------------------------------------------------------------------ line 35317
// R557 wall mail slot: 4% chance per level. Brass plate w/
// dark slot cavity + flap.
fn place_mail_slots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = basic_mat!({ color: 0x080604 });
    let slot = group();
    // Plate.
    let plate = mesh(pg("MailSlots__plate", box_geo!(0.20, 0.06, 0.006)), &brass_mat);
    plate.position().set(0, 0, -0.003);
    slot.add(&plate);
    // Dark slot cavity.
    let cavity = mesh(plane_geo!(0.14, 0.020), &dark_mat);
    cavity.position().set(0, 0.005, 0.001);
    slot.add(&cavity);
    // Flap (tilted slightly open).
    let flap = mesh(pg("MailSlots__flap", box_geo!(0.14, 0.028, 0.003)), &brass_mat);
    flap.position().set(0, 0.02, 0.002);   // 1 mm out: its back clears the cavity plane (hinge edge still in the plate)
    flap.rotation().x = -0.3;
    slot.add(&flap);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&slot, c, wall_x, wall_z, 1.10, Some(0.6));
    g.world.add(&slot);
}

// ------------------------------------------------------------------ line 35364
// R557 tape roll: 10% chance per level. Adhesive tape roll w/
// hole + colored side.
fn place_tape_rolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let core_mat = lambert!({ color: 0x8a6540 });
    let tape_colors = [0xf0ece0, 0xa02040, 0xd8b040, 0x2050c0];
    let tape_mat = lambert!({ color: choice(&tape_colors) });
    let tape = group();
    // Outer tape roll.
    let outer = mesh(pg("TapeRolls__roll", cyl_geo!(0.030, 0.030, 0.020, 12)), &tape_mat);
    outer.rotation().z = PI / 2.0;
    outer.position().y = 0.030;
    tape.add(&outer);
    // Cardboard inner core (visible on both sides).
    let core = mesh(pg("TapeRolls__core", cyl_geo!(0.014, 0.014, 0.022, 10, 1, true)), &core_mat);
    core.rotation().z = PI / 2.0;
    core.position().y = 0.030;
    tape.add(&core);
    // Small tail flap.
    let tail = mesh(plane_geo!(0.020, 0.020), &tape_mat);
    tail.rotation().x = -PI / 2.0;
    // trails off the roll's circumference where it meets the floor (z 0),
    // across the axis, as wide as the roll; it lay 3 cm off along the axis
    tail.position().set(0, 0.001, 0.020);
    tape.add(&tail);
    scatter(g, &tape, x, z, 2.4);
    g.world.add(&tape);
}

// ------------------------------------------------------------------ line 35408
// R556 wall outlet: 7% chance per level. Cream plate w/ 2
// slot pairs + optional plug inserted.
fn place_wall_outlets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 { return; }   // 7% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let plate_mat = lambert!({ color: 0xe8e0c8 });
    let slot_mat = basic_mat!({ color: 0x080604 });
    let plug_mat = lambert!({ color: 0x181410 });
    let outlet = group();
    // Plate.
    let plate = mesh(pg("WallOutlets__plate", box_geo!(0.070, 0.11, 0.005)), &plate_mat);
    plate.position().set(0, 0, -0.0025);
    outlet.add(&plate);
    // 2 outlet slots (top pair + bottom pair).
    for ii in 0..2 {
        let oy = 0.025 - ii as f64 * 0.05;
        for s in [-1.0, 1.0] {
            let slot = mesh(box_geo!(0.003, 0.014, 0.002), &slot_mat);
            slot.position().set(s * 0.008, oy, 0.001);
            outlet.add(&slot);
        }
        // Ground hole.
        let gnd = mesh(circle_geo!(0.003, 5), &slot_mat);
        gnd.position().set(0, oy - 0.014, 0.002);
        outlet.add(&gnd);
    }
    // 50% chance plug inserted in top outlet.
    if rnd() < 0.50 {
        let plug = mesh(pg("WallOutlets__plug", box_geo!(0.035, 0.036, 0.020)), &plug_mat);
        plug.position().set(0, 0.025, 0.010);   // back on the plate face (z 0)
        outlet.add(&plug);
        // Cord hanging.
        let cord = mesh(pg("WallOutlets__cord", cyl_geo!(0.002, 0.002, 0.20, 3)), &plug_mat);
        // upper end (local y +0.10, tilted 0.3 rad) 4 mm up inside the plug's
        // bottom-centre (x 0, y 0.007); the cord dangles down and to +X from there
        cord.position().set(0.10 * 0.3f64.sin(), 0.011 - 0.10 * 0.3f64.cos(), 0.010);
        cord.rotation().z = 0.3;
        outlet.add(&cord);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&outlet, c, wall_x, wall_z, 0.32, Some(0.6));   // Standard outlet height.
    g.world.add(&outlet);
}

// ------------------------------------------------------------------ line 35473
// R556 crumpled receipt: 10% chance per level. Small crumpled
// paper ball w/ visible ink lines.
fn place_crumpled_receipts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let paper_mat = lambert!({ color: 0xf0ece0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let receipt = group();
    // Crumpled ball (irregular sphere).
    let ball = mesh(pg("CrumpledReceipts__ball", sphere_geo!(0.020, 6, 5)), &paper_mat);
    ball.scale().set(1.2, 0.6, 1.0);
    ball.position().y = 0.010;
    receipt.add(&ball);
    // 2-3 exposed edge crinkles.
    for i in 0..3 {
        let crinkle = mesh(pg("CrumpledReceipts__crinkle", box_geo!(0.010, 0.004, 0.014)), &paper_mat);
        let a = (i as f64 / 3.0) * PI * 2.0;
        crinkle.position().set(a.cos() * 0.018, 0.008, a.sin() * 0.012);
        crinkle.rotation().y = -a;
        receipt.add(&crinkle);
    }
    // Small ink line barely visible.
    let ink = mesh(plane_geo!(0.014, 0.001), &ink_mat);
    ink.rotation().x = -PI / 2.0;
    ink.position().set(0, 0.020, 0);
    receipt.add(&ink);
    scatter(g, &receipt, x, z, 2.4);
    g.world.add(&receipt);
}

// ------------------------------------------------------------------ line 35515
// R555 wall drain grate: 5% chance per level. Rectangular grate
// w/ 5 horizontal slots + dark cavity.
fn place_wall_drain_grates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let frame_mat = lambert!({ color: 0x707078 });
    let dark_mat = basic_mat!({ color: 0x080604 });
    let grate = group();
    // Dark cavity behind.
    let cavity = mesh(plane_geo!(0.10, 0.14), &dark_mat);
    cavity.position().z = 0.001;
    grate.add(&cavity);
    // Frame.
    let fw = 0.006;
    for (bar_w, bar_h, bx, by) in [
        (0.10 + fw * 2.0, fw, 0.0, 0.07 + fw / 2.0),
        (0.10 + fw * 2.0, fw, 0.0, -0.07 - fw / 2.0),
        (fw, 0.14, -0.05 - fw / 2.0, 0.0),
        (fw, 0.14, 0.05 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(box_geo!(bar_w, bar_h, 0.004), &frame_mat);
        bar.position().set(bx, by, 0.002);
        grate.add(&bar);
    }
    // 5 horizontal slot bars.
    for i in 0..5 {
        let slot = mesh(pg("WallDrainGrates__slot", box_geo!(0.08, 0.004, 0.005)), &frame_mat);
        slot.position().set(0, -0.04 + i as f64 * 0.02, 0.003);
        grate.add(&slot);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&grate, c, wall_x, wall_z, 0.25, Some(0.6));   // Near floor.
    g.world.add(&grate);
}

// ------------------------------------------------------------------ line 35571
// R555 post-it stack: 8% chance per level. 3-5 stacked
// colored sticky notes.
fn place_post_it_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let note_colors = [0xf0e050, 0xf0a0c0, 0xa0f0a0, 0xa0d0f0];
    let ink_mat = lambert!({ color: 0x181410 });
    let grp = group();
    // Base stack.
    let n_stack = 3 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_stack {
        let note_mat = lambert!({ color: choice(&note_colors) });
        let note = mesh(pg("PostItStacks__note", box_geo!(0.06, 0.001, 0.06)), &note_mat);
        let px = (rnd() - 0.5) * 0.002;
        let py = 0.0006 + i as f64 * 0.0012;
        let pz = (rnd() - 0.5) * 0.002;
        note.position().set(px, py, pz);
        note.rotation().y = (rnd() - 0.5) * 0.10;
        grp.add(&note);
    }
    // 1-2 loose notes scattered nearby w/ scribbles.
    for i in 0..2 {
        let note_mat = lambert!({ color: choice(&note_colors) });
        let note = mesh(pg("PostItStacks__loose", box_geo!(0.06, 0.001, 0.06)), &note_mat);
        let px = 0.08 + (rnd() - 0.5) * 0.10;
        let pz = (rnd() - 0.5) * 0.14;
        note.position().set(px, 0.001, pz);
        note.rotation().y = rnd() * PI * 2.0;
        grp.add(&note);
        // Ink scribble line on note.
        let scribble = mesh(plane_geo!(0.040, 0.002), &ink_mat);
        scribble.rotation().x = -PI / 2.0;
        let np = note.pos();
        scribble.position().set(np.x, 0.002, np.z);
        scribble.rotation().z = note.rot().y;
        grp.add(&scribble);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// ------------------------------------------------------------------ line 35630
// R554 safety cone: 6% chance per level. Orange cone + white
// reflective stripe + square base.
fn place_safety_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let orange_mat = lambert!({ color: 0xe07020 });
    let white_mat = basic_mat!({ color: 0xf0ece0 });
    let base_mat = lambert!({ color: 0x181410 });
    let cone = group();
    // Cone body.
    let body = mesh(pg("SafetyCones__body", cone_geo!(0.06, 0.28, 8)), &orange_mat);
    body.position().y = 0.15;
    cone.add(&body);
    // White reflective stripe (band around cone).
    let stripe = mesh(cyl_geo!(0.030, 0.045, 0.025, 8, 1, true), &white_mat);
    stripe.scale().set(0.95, 2.96, 0.95);   // hug the cone's taper (flared 5-15 mm off it); facet midpoints on the surface
    stripe.position().y = 0.16;
    cone.add(&stripe);
    // Square base.
    let base = mesh(pg("SafetyCones__base", box_geo!(0.12, 0.010, 0.12)), &base_mat);
    base.position().y = 0.005;
    cone.add(&base);
    // 25% chance tipped.
    if rnd() < 0.25 {
        cone.rotation().z = PI / 2.0;
        cone.position().y = 0.06;
    }
    cone.position().x = g.wx(x) + (rnd() - 0.5) * 2.0;
    cone.position().z = g.wz(z) + (rnd() - 0.5) * 2.0;
    cone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cone);
}

// ------------------------------------------------------------------ line 35670
// R554 wall door with peephole: 4% chance per level. Wooden
// door plane + handle + peephole disc.
fn place_wall_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let door_colors = [0x603820, 0x8a5030, 0x201a10, 0xa02040];
    let door_mat = lambert!({ color: choice(&door_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = basic_mat!({ color: 0x080604 });
    let door = group();
    // Door slab.
    let slab = mesh(pg("WallDoors__slab", box_geo!(0.70, 1.90, 0.030)), &door_mat);
    slab.position().set(0, 0, -0.015);
    door.add(&slab);
    // 2 panels (inset rectangles).
    for i in 0..2 {
        let panel = mesh(pg("WallDoors__panel", box_geo!(0.50, 0.60, 0.010)), &door_mat);
        panel.position().set(0, -0.35 + i as f64 * 0.70, 0.001);
        door.add(&panel);
    }
    // Handle.
    let handle = mesh(pg("WallDoors__knob", cyl_geo!(0.014, 0.014, 0.030, 6)), &brass_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.28, 0.05, 0.010);
    door.add(&handle);
    // Peephole.
    let peephole = mesh(circle_geo!(0.006, 8), &dark_mat);
    peephole.position().set(0, 0.65, 0.002);
    door.add(&peephole);
    let (wall_x, wall_z) = wall_xz(g, c, 0.020);
    mount(&door, c, wall_x, wall_z, 0.95, None);
    g.world.add(&door);
}

// ------------------------------------------------------------------ line 35727
// R553 single sock: 10% chance per level. Flat sock w/ toe
// + heel + cuff.
fn place_single_socks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let sock_colors = [0xf0ece0, 0x181410, 0xa02040, 0x2050c0, 0xd8b040];
    let sock_mat = lambert!({ color: choice(&sock_colors) });
    let stripe_mat = lambert!({ color: 0x181410 });
    let sock = group();
    // Foot + cuff (L-shape).
    let foot = mesh(pg("SingleSocks__foot", box_geo!(0.10, 0.014, 0.036)), &sock_mat);
    foot.position().y = 0.007;
    sock.add(&foot);
    // Cuff angled up.
    let cuff = mesh(pg("SingleSocks__cuff", box_geo!(0.04, 0.014, 0.036)), &sock_mat);
    cuff.position().set(-0.05, 0.007, 0);
    sock.add(&cuff);
    // 2 stripe bands.
    for cx in [-0.045, 0.045] {
        let stripe = mesh(pg("SingleSocks__stripe", box_geo!(0.010, 0.015, 0.036)), &stripe_mat);
        stripe.position().set(cx, 0.007, 0);
        sock.add(&stripe);
    }
    scatter(g, &sock, x, z, 2.4);
    g.world.add(&sock);
}

// ------------------------------------------------------------------ line 35768
// R553 ceiling motion detector: 5% chance per level. Small
// dome + tiny LED indicator.
fn place_motion_detector_domes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let body_mat = lambert!({ color: 0xd0d0d0 });
    let dome_mat = basic_mat!({ color: 0x101820, transparent: true, opacity: 0.6 });
    let led_mat = basic_mat!({ color: 0x30d030 });
    let det = group();
    let dome_y = WALL_H - 0.005;   // dome top = base underside; base top on the ceiling
    // Base plate.
    let base = mesh(pg("MotionDetectorDomes__base", cyl_geo!(0.045, 0.045, 0.005, 12)), &body_mat);
    base.position().y = dome_y + 0.0025;
    det.add(&base);
    // Dark dome.
    let dome = mesh(sphere_geo!(0.035, 12, 8, 0, PI * 2.0, 0, PI / 2.0), &dome_mat);
    dome.rotation().x = PI;
    dome.position().y = dome_y;
    det.add(&dome);
    // Green LED.
    let led = mesh(sphere_geo!(0.003, 5, 4), &led_mat);
    led.position().set(0.028, dome_y, 0);
    det.add(&led);
    // Slow blink.
    let blink_phase = rnd() * PI * 2.0;
    {
        let led = led.clone();
        g.animate(move |_g, _dt, t| {
            led.set_visible((t * 2.0 + blink_phase).sin() > 0.7);
        });
    }
    scatter(g, &det, x, z, 2.0);
    g.world.add(&det);
}

// ------------------------------------------------------------------ line 35814
// R552 peeled paint patch: 6% chance per level. Wall decal
// w/ curled paint edge revealing darker base beneath.
game::lazy_static_tex!(peeled_paint_texture, canvas_tex(256, 320, |ctx, w, h| {
    // Base (darker underlayer showing through).
    ctx.set_fill_style("#3a2818");
    ctx.fill_rect(0, 0, w, h);
    // Peeled paint region (lighter, jagged edge).
    ctx.set_fill_style("#e0d8c0");
    ctx.begin_path();
    ctx.move_to(0, 0);
    ctx.line_to(w, 0);
    ctx.line_to(w, h * 0.4);
    // Jagged separator line.
    let mut py = h * 0.4;
    while py < h {
        ctx.line_to(w * (0.15 + rnd() * 0.20), py);
        py += 15.0;
    }
    ctx.line_to(0, h);
    ctx.close_path();
    ctx.fill();
    // Curled edge lip (darker shadow along tear).
    ctx.set_stroke_style("rgba(20,10,5,0.7)");
    ctx.set_line_width(3);
    ctx.begin_path();
    ctx.move_to(w * 0.30, h * 0.4);
    let mut py = h * 0.4;
    while py < h {
        ctx.line_to(w * (0.15 + rnd() * 0.20), py);
        py += 15.0;
    }
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 35841
fn place_peeled_paint_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let mat = lambert!({ map: peeled_paint_texture(), transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
    let decal = mesh(plane_geo!(0.50, 0.65), &mat);
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&decal, c, wall_x, wall_z, 1.4, Some(0.5));
    g.world.add(&decal);
}

// ------------------------------------------------------------------ line 35878
// R552 broken flashlight: 8% chance per level. Metal cylinder
// w/ head lens (cracked) + faint dim glow inside.
fn place_broken_flashlights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let body_colors = [0x181410, 0x606870, 0xa02020];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let lens_mat = basic_mat!({ color: 0xe0e8f0, transparent: true, opacity: 0.5 });
    let flashlight = group();
    // Handle body.
    let body = mesh(pg("BrokenFlashlights__body", cyl_geo!(0.014, 0.014, 0.12, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.014, 0);
    flashlight.add(&body);
    // Head (wider).
    let head = mesh(pg("BrokenFlashlights__head", cyl_geo!(0.020, 0.014, 0.024, 8)), &body_mat);
    head.rotation().z = -PI / 2.0;
    head.position().set(0.070, 0.014, 0);
    flashlight.add(&head);
    // Cracked lens.
    let lens = mesh(circle_geo!(0.018, 8), &lens_mat);
    lens.rotation().y = PI / 2.0;
    lens.position().set(0.083, 0.014, 0);
    flashlight.add(&lens);
    // 40% chance faint dim glow (dying beam).
    if rnd() < 0.40 {
        let glow = point_light!(0xffffe0, 0.20, 0.5, 2.0);
        glow.position().set(0.09, 0.014, 0);
        flashlight.add(&glow);
        // Very unsteady dying flicker.
        let flicker_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let flick = if (t * 12.0 + flicker_phase).sin() > 0.5 { 1.0 } else { 0.15 };
            glow.set_intensity(0.20 * flick);
        });
    }
    // 30% chance tipped/rolling.
    if rnd() < 0.30 {
        flashlight.rotation().y = PI / 2.0 + (rnd() - 0.5) * 0.5;
    }
    flashlight.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    flashlight.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    flashlight.rotation().y += rnd() * PI * 2.0;
    // The head (r 0.020) is fatter than the barrel (r 0.014): tip it so the head
    // rim and the tail both rest on the floor instead of the head sinking 6 mm.
    flashlight.rotation().z = 0.042;
    flashlight.position().y = 0.0025;
    g.world.add(&flashlight);
}

// ------------------------------------------------------------------ line 35939
// R551 ceiling pipe run: 5% chance per level. Horizontal pipe
// segment attached to ceiling w/ 2 hanger straps.
fn place_ceiling_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let pipe_mat = lambert!({ color: 0x707078 });
    let rust_mat = lambert!({ color: 0x6a3820 });
    let pipe = group();
    let pipe_y = WALL_H - 0.08;
    // Long pipe.
    let body = mesh(pg("CeilingPipes__body", cyl_geo!(0.024, 0.024, 2.0, 8)), &pipe_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = pipe_y;
    pipe.add(&body);
    // Joint band in middle.
    let joint = mesh(pg("WallPipeLeaks__joint", cyl_geo!(0.027, 0.027, 0.020, 8)), &rust_mat);
    joint.rotation().z = PI / 2.0;
    joint.position().y = pipe_y;
    pipe.add(&joint);
    // 2 hanger straps.
    for s in [-0.6, 0.6] {
        let strap = mesh(pg("CeilingPipes__strap", box_geo!(0.014, 0.08, 0.004)), &pipe_mat);
        strap.position().set(s, pipe_y + 0.045, 0);
        pipe.add(&strap);
    }
    scatter(g, &pipe, x, z, 1.0);
    g.world.add(&pipe);
}

// ------------------------------------------------------------------ line 35980
// R551 office lanyard w/ badge: 8% chance per level. Colored
// strap loop + ID badge card.
fn place_lanyards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let strap_colors = [0xa02040, 0x2050c0, 0x30a030, 0x181410, 0xd8b040];
    let strap_mat = lambert!({ color: choice(&strap_colors), side: DoubleSide });
    let badge_mat = lambert!({ color: 0xf0ece0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let lanyard = group();
    // Loop (half-torus lying flat).
    let lp = mesh(torus_geo!(0.06, 0.003, 4, 12, PI * 1.4), &strap_mat);
    lp.rotation().x = -PI / 2.0;
    lp.position().y = 0.003;
    lanyard.add(&lp);
    // Strap extending down to badge.
    let strap = mesh(plane_geo!(0.008, 0.06), &strap_mat);
    // run along +X from the loop's open end (x 0.06, z 0) to the badge's
    // slotted end (x 0.12): it lay across Z, touching neither
    strap.rotation().set(-PI / 2.0, 0, PI / 2.0);
    strap.position().set(0.093, 0.003, 0);
    lanyard.add(&strap);
    // Badge card.
    let badge = mesh(pg("Lanyards__badge", box_geo!(0.06, 0.001, 0.09)), &badge_mat);
    // long axis along +X, its slotted −Z end turned to the strap (spans x 0.12…0.21)
    badge.rotation().y = PI / 2.0;
    badge.position().set(0.165, 0.001, 0);
    lanyard.add(&badge);
    // 3 ink lines on badge.
    for i in 0..3 {
        let line = mesh(plane_geo!(0.045, 0.003), &ink_mat);
        // turned with the badge: across its width (z), stepping along its length (x)
        line.rotation().set(-PI / 2.0, 0, PI / 2.0);
        line.position().set(0.165 - 0.03 + i as f64 * 0.020, 0.0016, 0);
        lanyard.add(&line);
    }
    scatter(g, &lanyard, x, z, 2.0);
    g.world.add(&lanyard);
}

// ------------------------------------------------------------------ line 36035
// R550 broken pen cluster: 8% chance per level. 4-8 pens
// (some snapped in half) scattered.
fn place_broken_pens(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let pen_colors = [0x2050c0, 0xa02040, 0x30a030, 0x181410, 0xd8b040];
    let grp = group();
    let n_pens = 4 + (rnd() * 5.0).floor() as i32;
    for i in 0..n_pens {
        let pen_mat = lambert!({ color: choice(&pen_colors) });
        let is_broken = rnd() < 0.4;
        if is_broken {
            // 2 halves.
            for h in 0..2 {
                let hf = h as f64;
                let half = mesh(pg("BrokenPens__half", cyl_geo!(0.005, 0.005, 0.05, 5)), &pen_mat);
                half.rotation().z = PI / 2.0;
                let px = (rnd() - 0.5) * 0.20 + hf * 0.02;
                let pz = (rnd() - 0.5) * 0.15 + hf * 0.02;
                half.position().set(px, 0.005, pz);
                half.rotation().y = rnd() * PI * 2.0;
                grp.add(&half);
            }
        } else {
            let pen = mesh(pg("BrokenPens__pen", cyl_geo!(0.005, 0.005, 0.12, 5)), &pen_mat);
            pen.rotation().z = PI / 2.0;
            let px = (rnd() - 0.5) * 0.20;
            let pz = (rnd() - 0.5) * 0.15;
            pen.position().set(px, 0.005, pz);
            pen.rotation().y = rnd() * PI * 2.0;
            grp.add(&pen);
        }
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// ------------------------------------------------------------------ line 36089
// R550 bottle caps scatter: 10% chance per level. 8-14 small
// colored discs.
fn place_bottle_caps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let cap_colors = [0xa02020, 0x30a030, 0x2050c0, 0xd8b040, 0xc0c0c0];
    let grp = group();
    let n_caps = 8 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_caps {
        let cap_mat = lambert!({ color: choice(&cap_colors) });
        let cap = mesh(pg("BottleCaps__cap", cyl_geo!(0.010, 0.011, 0.003, 12)), &cap_mat);
        let px = (rnd() - 0.5) * 0.25;
        let pz = (rnd() - 0.5) * 0.25;
        cap.position().set(px, 0.0015, pz);
        // 25% edge-on tipped.
        if rnd() < 0.25 {
            cap.rotation().z = PI / 2.0;
            cap.position().y = 0.010;
        }
        cap.rotation().y = rnd() * PI * 2.0;
        grp.add(&cap);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// ------------------------------------------------------------------ line 36131
// R549 wall fuse box: 4% chance per level. Gray metal box w/
// hinge + 6-8 switches inside.
fn place_fuse_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let box_mat = lambert!({ color: 0x606870 });
    let sw_mat = lambert!({ color: 0x181410 });
    let orange_mat = lambert!({ color: 0xd8b040 });
    let fuse_box = group();
    // Body box.
    let body = mesh(pg("FuseBoxes__body", box_geo!(0.14, 0.20, 0.06)), &box_mat);
    body.position().set(0, 0, -0.03);
    fuse_box.add(&body);
    // Door (partially open).
    let door = mesh(pg("FuseBoxes__door", box_geo!(0.14, 0.20, 0.008)), &box_mat);
    // swung 0.6 rad about its hinge edge (knuckles on -x) at the box's
    // front-left corner, not about its centre (hinge sank 3.5 cm into the box)
    door.position().set(-0.0145, 0, 0.0428);
    door.rotation().y = -0.6;
    fuse_box.add(&door);
    // Switches inside (6 rows of 1, one lever type).
    for i in 0..6 {
        let yy = 0.06 - i as f64 * 0.024;
        let sw = mesh(pg("FuseBoxes__switch", box_geo!(0.010, 0.020, 0.005)), &sw_mat);
        sw.position().set(-0.030, yy, 0.003);
        // Random tilt for on/off.
        sw.rotation().z = if rnd() < 0.5 { 0.3 } else { -0.3 };
        fuse_box.add(&sw);
        // Labels beside switches.
        let label = mesh(plane_geo!(0.020, 0.006), &orange_mat);
        label.position().set(0.01, yy, 0.005);
        fuse_box.add(&label);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.030);
    mount(&fuse_box, c, wall_x, wall_z, 1.50, Some(0.4));
    g.world.add(&fuse_box);
}

// ------------------------------------------------------------------ line 36188
// R549 hanging bare bulb: 5% chance per level. Bulb + socket +
// cord from ceiling + faint warm glow.
fn place_hanging_light_bulbs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let cord_mat = lambert!({ color: 0x181008 });
    let socket_mat = lambert!({ color: 0x606870 });
    let bulb_mat = basic_mat!({ color: 0xf0e8b0, transparent: true, opacity: 0.75 });
    let bulb = group();
    let cord_len = 0.5 + rnd() * 0.5;
    let bulb_y = WALL_H - cord_len - 0.05;
    // Cord.
    let cord = mesh(pg("HangingLightBulbs__cord", cyl_geo!(0.002, 0.002, cord_len, 4)), &cord_mat);
    cord.position().y = WALL_H - cord_len / 2.0 - 0.02;
    bulb.add(&cord);
    // Socket.
    let socket = mesh(pg("HangingLightBulbs__socket", cyl_geo!(0.014, 0.014, 0.020, 6)), &socket_mat);
    socket.position().y = bulb_y + 0.020;
    bulb.add(&socket);
    // Bulb (elongated sphere).
    let bulb_body = mesh(sphere_geo!(0.024, 10, 8), &bulb_mat);
    bulb_body.scale().y = 1.3;
    bulb_body.position().y = bulb_y - 0.010;
    bulb.add(&bulb_body);
    // 40% chance lit — warm glow.
    if rnd() < 0.40 {
        let glow = point_light!(0xffe0a0, 0.4, 1.2, 2.0);
        glow.position().y = bulb_y - 0.010;
        bulb.add(&glow);
        // Subtle flicker.
        let flicker_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            glow.set_intensity(0.4 + (t * 4.0 + flicker_phase).sin() * 0.06);
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bulb.position().set(px, 0, pz);
    // Slight sway.
    if rnd() < 0.50 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 6.0;
        let b = bulb.clone();
        g.animate(move |_g, _dt, t| {
            b.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.015;
        });
    }
    g.world.add(&bulb);
}

// ------------------------------------------------------------------ line 36248
// R548 wall intercom: 4% chance per level. Small speaker box +
// grille + button.
fn place_wall_intercoms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let body_mat = lambert!({ color: 0x808088 });
    let dark_mat = lambert!({ color: 0x181410 });
    let button_mat = lambert!({ color: 0xa02020 });
    let intercom = group();
    // Body plate.
    let body = mesh(pg("WallIntercoms__body", box_geo!(0.10, 0.16, 0.020)), &body_mat);
    body.position().set(0, 0, -0.010);
    intercom.add(&body);
    // Speaker grille (dark disc).
    let grille = mesh(pg("WallIntercoms__grille", circle_geo!(0.032, 12)), &dark_mat);
    grille.position().set(0, 0.03, 0.002);
    intercom.add(&grille);
    // 4 grille slot lines.
    for i in 0..4 {
        let slot = mesh(pg("WallIntercoms__slot", box_geo!(0.040, 0.002, 0.001)), &body_mat);
        slot.position().set(0, 0.02 + i as f64 * 0.008, 0.005);
        intercom.add(&slot);
    }
    // Big talk button.
    let btn = mesh(pg("WallIntercoms__button", cyl_geo!(0.010, 0.010, 0.005, 8)), &button_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0, -0.05, 0.004);
    intercom.add(&btn);
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&intercom, c, wall_x, wall_z, 1.55, Some(0.6));
    g.world.add(&intercom);
}

// ------------------------------------------------------------------ line 36302
// R548 paperclip cluster: 10% chance per level. 6-14 metal
// paperclip torus scattered.
fn place_paperclip_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let clip_mat = lambert!({ color: 0xc0c0c0 });
    let grp = group();
    let n_clips = 6 + (rnd() * 9.0).floor() as i32;
    for i in 0..n_clips {
        let clip = mesh(pg("PaperclipClusters__clip", torus_geo!(0.008, 0.001, 4, 8, PI * 1.8)), &clip_mat);
        clip.rotation().x = -PI / 2.0;
        clip.rotation().z = rnd() * PI * 2.0;
        let px = (rnd() - 0.5) * 0.15;
        let pz = (rnd() - 0.5) * 0.12;
        clip.position().set(px, 0.001, pz);
        // Slight scale variance.
        let sx = 0.9 + rnd() * 0.4;
        let sz = 0.9 + rnd() * 0.4;
        clip.scale().set(sx, 1, sz);
        grp.add(&clip);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// ------------------------------------------------------------------ line 36339
// R547 ceiling sprinkler head: 6% chance per level. Small
// brass dome + 4 deflector arms hanging from ceiling.
fn place_sprinkler_heads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 { return; }   // 6% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let brass_mat = lambert!({ color: 0xc89040 });
    let red_mat = lambert!({ color: 0xd82040 });
    let spr = group();
    let spr_y = WALL_H;   // stub top on the ceiling (was 2 cm below it)
    // Threaded pipe stub.
    let stub = mesh(pg("SprinklerHeads__stub", cyl_geo!(0.008, 0.008, 0.020, 6)), &brass_mat);
    stub.position().y = spr_y - 0.010;
    spr.add(&stub);
    // Central bulb.
    let bulb = mesh(pg("SprinklerHeads__bulb", sphere_geo!(0.008, 6, 5)), &red_mat);
    bulb.position().y = spr_y - 0.025;
    spr.add(&bulb);
    // 4 deflector arms.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let arm = mesh(pg("SprinklerHeads__arm", box_geo!(0.014, 0.002, 0.004)), &brass_mat);
        arm.position().set(a.cos() * 0.010, spr_y - 0.035, a.sin() * 0.010);
        arm.rotation().y = -a;
        spr.add(&arm);
    }
    // Small round deflector disc.
    let disc = mesh(pg("SprinklerHeads__deflector", circle_geo!(0.008, 8)), &brass_mat);
    disc.rotation().x = PI / 2.0;
    disc.position().y = spr_y - 0.038;
    spr.add(&disc);
    scatter(g, &spr, x, z, 2.0);
    g.world.add(&spr);
}

// ------------------------------------------------------------------ line 36385
// R547 air freshener can: 10% chance per level. Small aerosol
// can w/ cap.
fn place_air_fresheners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let can_colors = [0x30a030, 0xd050a0, 0x2050c0, 0xe07020];
    let can_mat = lambert!({ color: choice(&can_colors) });
    let cap_mat = lambert!({ color: 0xf0ece0 });
    let label_mat = lambert!({ color: 0xf0ece0 });
    let can = group();
    // Body.
    let body = mesh(pg("AirFresheners__can", cyl_geo!(0.020, 0.020, 0.14, 10)), &can_mat);
    body.position().y = 0.070;
    can.add(&body);
    // Cap.
    let cap = mesh(pg("AirFresheners__cap", cyl_geo!(0.018, 0.020, 0.018, 8)), &cap_mat);
    cap.position().y = 0.149;
    can.add(&cap);
    // Label band.
    let label = mesh(pg("AirFresheners__label", cyl_geo!(0.0205, 0.0205, 0.030, 10, 1, true)), &label_mat);
    label.position().y = 0.080;
    can.add(&label);
    // 30% tipped.
    if rnd() < 0.30 {
        can.rotation().z = PI / 2.0;
        can.position().y = 0.020;
    }
    can.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    can.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}

// ------------------------------------------------------------------ line 36427
// R546 emergency phone: 4% chance per level. Red wall box +
// handset on hook + "EMERGENCY" label.
fn place_emergency_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 { return; }   // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let red_mat = lambert!({ color: 0xa02020 });
    let dark_mat = lambert!({ color: 0x181410 });
    let white_mat = basic_mat!({ color: 0xf0ece0 });
    let phone = group();
    // Box.
    let bx = mesh(pg("EmergencyPhones__box", box_geo!(0.16, 0.24, 0.09)), &red_mat);
    bx.position().set(0, 0, -0.045);
    phone.add(&bx);
    // "EMERGENCY" label bar.
    let label = mesh(plane_geo!(0.14, 0.020), &white_mat);
    label.position().set(0, 0.10, 0.002);
    phone.add(&label);
    // Handset (dark, angled).
    let handset = mesh(pg("EmergencyPhones__handset", cyl_geo!(0.014, 0.014, 0.12, 6)), &dark_mat);
    handset.rotation().z = PI / 2.0;
    handset.position().set(0, -0.02, 0.016);   // back 2 mm proud of the face, over the trough
    phone.add(&handset);
    // Ear/mouth cups.
    for s in [-1.0, 1.0] {
        let cup = mesh(pg("EmergencyPhones__cup", sphere_geo!(0.018, 8, 6)), &dark_mat);
        cup.position().set(s * 0.055, -0.02, 0.018);   // dished face resting on the box face (z 0)
        phone.add(&cup);
    }
    // box (local z −0.09…0) back on the wall, not half buried in it
    let (wall_x, wall_z) = wall_xz(g, c, 0.09);
    mount(&phone, c, wall_x, wall_z, 1.3, Some(0.4));
    g.world.add(&phone);
}

// ------------------------------------------------------------------ line 36481
// R546 dropped cell phone: 10% chance per level. Small
// rectangular phone (flip or slab).
fn place_dropped_cell_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 { return; }   // 10% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let body_colors = [0x181410, 0xa02040, 0x2050c0, 0xd8b040];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = basic_mat!({ color: 0x101820 });
    let phone = group();
    let is_flip = rnd() < 0.5;
    if is_flip {
        // Flip-phone: 2 halves stacked.
        let bottom = mesh(pg("DroppedCellPhones__flipBase", box_geo!(0.030, 0.006, 0.055)), &body_mat);
        bottom.position().y = 0.003;
        phone.add(&bottom);
        let top = mesh(pg("DroppedCellPhones__flipLid", box_geo!(0.030, 0.006, 0.055)), &body_mat);
        top.position().y = 0.009;
        phone.add(&top);
        // Keypad dots.
        for ky in 0..3 {
            for kx in 0..3 {
                let key = mesh(cyl_geo!(0.002, 0.002, 0.001, 4), &screen_mat);
                key.position().set(-0.008 + kx as f64 * 0.008, 0.007, 0.006 + ky as f64 * 0.008);
                phone.add(&key);
            }
        }
    } else {
        // Slab phone.
        let body = mesh(pg("DroppedCellPhones__slab", box_geo!(0.040, 0.008, 0.075)), &body_mat);
        body.position().y = 0.004;
        phone.add(&body);
        // Dark screen.
        let screen = mesh(plane_geo!(0.032, 0.055), &screen_mat);
        screen.rotation().x = -PI / 2.0;
        screen.position().y = 0.009;
        phone.add(&screen);
    }
    scatter(g, &phone, x, z, 2.4);
    g.world.add(&phone);
}

// ------------------------------------------------------------------ line 36536
// R545 wall pipe leak: 5% chance per level. Horizontal pipe
// fixture + drip line + water stain patch.
fn place_wall_pipe_leaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 { return; }   // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let pipe_mat = lambert!({ color: 0x606870 });
    let rust_mat = lambert!({ color: 0x6a3820 });
    let wet_mat = basic_mat!({ color: 0x30506a, transparent: true, opacity: 0.7 });
    let leak = group();
    // Horizontal pipe cylinder.
    let pipe = mesh(pg("WallPipeLeaks__pipe", cyl_geo!(0.024, 0.024, 0.60, 8)), &pipe_mat);
    pipe.rotation().z = PI / 2.0;
    leak.add(&pipe);
    // Rust joint band.
    let joint = mesh(pg("WallPipeLeaks__joint", cyl_geo!(0.026, 0.026, 0.020, 8)), &rust_mat);
    joint.rotation().z = PI / 2.0;
    joint.position().x = 0.05;
    leak.add(&joint);
    // Drip line hanging.
    let drip = mesh(cyl_geo!(0.002, 0.001, 0.16, 3), &wet_mat);
    drip.position().set(0.05, -0.10, 0.014);
    leak.add(&drip);
    // Water stain patch below.
    let stain = mesh(circle_geo!(0.07, 8), &wet_mat);
    stain.position().set(0.05, -0.19, -0.024);   // wet patch on the wall below the drip (was a flat disc in mid-air)
    leak.add(&stain);
    // pipe axis 2.6 cm off the wall (r 2.4 cm): was sunk 1 cm into it
    let (wall_x, wall_z) = wall_xz(g, c, 0.026);
    mount(&leak, c, wall_x, wall_z, 1.90, Some(0.3));
    g.world.add(&leak);
}

// ------------------------------------------------------------------ line 36591
// R545 unrolled TP: 8% chance per level. Cardboard tube +
// unrolled paper trail.
fn place_unrolled_tp_rolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 { return; }   // 8% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = choice(&open);
    let tube_mat = lambert!({ color: 0x8a5030 });
    let paper_mat = lambert!({ color: 0xf0ece0, side: DoubleSide });
    let tp = group();
    // Cardboard tube.
    let tube = mesh(pg("UnrolledTPRolls__tube", cyl_geo!(0.024, 0.024, 0.10, 10)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().y = 0.024;
    tp.add(&tube);
    // Unrolled paper trail (bent plane strip).
    let trail = mesh(plane_geo!(0.30, 0.10), &paper_mat);
    trail.rotation().x = -PI / 2.0;
    trail.position().set(0.20, 0.001, 0);
    tp.add(&trail);
    // Small second trail patch further out.
    let trail2 = mesh(plane_geo!(0.20, 0.10), &paper_mat);
    trail2.rotation().x = -PI / 2.0;
    trail2.position().set(0.45, 0.001, 0.03);
    trail2.rotation().z = 0.2;
    tp.add(&trail2);
    scatter(g, &tp, x, z, 2.0);
    g.world.add(&tp);
}

// ------------------------------------------------------------------ line 36633
// R544 emergency strobe: 3% chance per level. Red dome bulb
// on wall base + occasional flash animator.
fn place_emergency_strobes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 { return; }   // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() { return; }
    let c = choice(&candidates);
    let base_mat = lambert!({ color: 0x181410 });
    let dome_mat = basic_mat!({ color: 0xa02020, transparent: true, opacity: 0.6 });
    let strobe = group();
    // Base plate.
    let base = mesh(pg("EmergencyStrobes__base", cyl_geo!(0.030, 0.030, 0.010, 8)), &base_mat);
    base.rotation().x = PI / 2.0;
    base.position().z = 0.005;
    strobe.add(&base);
    // Red dome (half sphere).
    let dome = mesh(sphere_geo!(0.030, 12, 8, 0, PI * 2.0, 0, PI / 2.0), &dome_mat);
    dome.rotation().x = PI / 2.0;   // +π/2 turns the hemisphere's +Y cap to +Z (into the room, not the wall)
    dome.position().z = 0.010;
    strobe.add(&dome);
    // Point light inside.
    let flash = point_light!(0xd82020, 0.0, 1.5, 2.0);
    flash.position().z = 0.020;
    strobe.add(&flash);
    // Flash animator (occasional bright pop).
    let mut next_flash = 8.0 + rnd() * 20.0;
    {
        let flash = flash.clone();
        g.animate(move |_g, _dt, t| {
            if t >= next_flash {
                // Bright pop.
                flash.set_intensity(1.5);
                next_flash = t + 4.0 + rnd() * 8.0;
            } else {
                // Rapid decay.
                let v = flash.intensity() * 0.85;
                flash.set_intensity(v);
            }
        });
    }
    // group origin = the base's back face (local z 0): 0.5 mm off the wall
    let (wall_x, wall_z) = wall_xz(g, c, 0.0005);
    mount(&strobe, c, wall_x, wall_z, 2.30, Some(0.6));
    g.world.add(&strobe);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
