//! Prop placers: building-services fixtures (towel bars, elevator panels,
//! extinguisher cabinets, card readers, mail slots, outlets, drain grates,
//! doors, motion detectors, ceiling pipes, fuse boxes, light bulbs, intercoms,
//! sprinkler heads, emergency phones and strobes, pipe leaks, peeled paint) and
//! small litter (clipboards, books, tape, receipts, sticky notes, safety cones
//! and glasses, socks, flashlights, lanyards, pens, bottle caps, paper clips,
//! air fresheners, phones, toilet paper).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_hand_towel_bars", place_hand_towel_bars));
    r.push(("place_elevator_panels", place_elevator_panels));
    r.push(("place_dropped_clipboards", place_dropped_clipboards));
    r.push(("place_extinguisher_cabinets", place_extinguisher_cabinets));
    r.push(("place_safety_glasses", place_safety_glasses));
    r.push(("place_card_readers", place_card_readers));
    r.push(("place_paperback_books", place_paperback_books));
    r.push(("place_mail_slots", place_mail_slots));
    r.push(("place_tape_rolls", place_tape_rolls));
    r.push(("place_wall_outlets", place_wall_outlets));
    r.push(("place_crumpled_receipts", place_crumpled_receipts));
    r.push(("place_wall_drain_grates", place_wall_drain_grates));
    r.push(("place_post_it_stacks", place_post_it_stacks));
    r.push(("place_safety_cones", place_safety_cones));
    r.push(("place_wall_doors", place_wall_doors));
    r.push(("place_single_socks", place_single_socks));
    r.push(("place_motion_detector_domes", place_motion_detector_domes));
    r.push(("place_peeled_paint_patches", place_peeled_paint_patches));
    r.push(("place_broken_flashlights", place_broken_flashlights));
    r.push(("place_ceiling_pipes", place_ceiling_pipes));
    r.push(("place_lanyards", place_lanyards));
    r.push(("place_broken_pens", place_broken_pens));
    r.push(("place_bottle_caps", place_bottle_caps));
    r.push(("place_fuse_boxes", place_fuse_boxes));
    r.push(("place_hanging_light_bulbs", place_hanging_light_bulbs));
    r.push(("place_wall_intercoms", place_wall_intercoms));
    r.push(("place_paperclip_clusters", place_paperclip_clusters));
    r.push(("place_sprinkler_heads", place_sprinkler_heads));
    r.push(("place_air_fresheners", place_air_fresheners));
    r.push(("place_emergency_phones", place_emergency_phones));
    r.push(("place_dropped_cell_phones", place_dropped_cell_phones));
    r.push(("place_wall_pipe_leaks", place_wall_pipe_leaks));
    r.push(("place_unrolled_tp_rolls", place_unrolled_tp_rolls));
    r.push(("place_emergency_strobes", place_emergency_strobes));
}

// ------------------------------------------------------------------ shared scaffolding
// Every placer in this crate repeats the same two tile scans and the same
// wall-mount tail; they are factored here with identical draws.

/// `{ x, z, dx, dz }`: an open tile with a wall on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallC {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The wall-adjacent candidate scan: one entry per open tile and wall (or grid-edge) side.
fn wall_candidates(g: &Game) -> Vec<WallC> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
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

/// The open-tile scan, row-major.
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

/// The wall-face point: `wx(c.x) + c.dx * (CELL * 0.5 - inset)`, same for z.
fn wall_xz(g: &Game, c: WallC, inset: f64) -> (f64, f64) {
    (g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset), g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset))
}

/// The wall-mount tail: face the room, then position, with an optional
/// `(`rnd()` - 0.5) * jitter` slide along the wall.
fn mount(o: &Obj, c: WallC, wall_x: f64, wall_z: f64, center_y: f64, jitter: Option<f64>) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let zz = match jitter {
            Some(j) => wall_z + (rnd() - 0.5) * j,
            None => wall_z,
        };
        o.position().set(wall_x, center_y, zz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let xx = match jitter {
            Some(j) => wall_x + (rnd() - 0.5) * j,
            None => wall_x,
        };
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

// R562 wall towel bar: 6% chance per level. Chrome bar + 2
// support brackets + hanging cloth towel.
fn place_hand_towel_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R560 elevator button panel: 4% chance per level. Silver panel
// w/ 12-button number grid + emergency red button.
fn place_elevator_panels(g: &mut Game, _dist: &[i32]) {
    g.pp.elevator_panel_positions = vec![]; // R561 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R560 dropped clipboard: 8% chance per level. Wooden clipboard
// + white paper w/ ink lines + clip.
fn place_dropped_clipboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R559 wall extinguisher cabinet: 4% chance per level. Metal
// box w/ glass front + small extinguisher visible.
fn place_extinguisher_cabinets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    ext.position().set(0, -0.06, -0.034); // front (z −0.002) stays behind the glass (z 0.001)
    cab.add(&ext);
    // Cap.
    let cap = mesh(pg("ExtinguisherCabinets__valve", cyl_geo!(0.024, 0.032, 0.020, 8)), &cap_mat);
    cap.position().set(0, 0.070, -0.034); // seated on the cylinder's top (y 0.06)
    cab.add(&cap);
    let (wall_x, wall_z) = wall_xz(g, c, 0.040);
    mount(&cab, c, wall_x, wall_z, 1.30, Some(0.4));
    g.world.add(&cab);
}

// R559 safety glasses: 8% chance per level. Clear plastic
// frame + tinted lenses + temple arms.
fn place_safety_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R558 swipe card reader: 4% chance per level. Dark box w/
// slot + status LED.
fn place_card_readers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R558 paperback book: 8% chance per level. Open book laid on
// floor w/ visible page lines.
fn place_paperback_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
                line.position().set((if side == 0 { -1.0 } else { 1.0 }) * (bw / 4.0), bd + 0.0015, -bh / 3.0 + l as f64 * 0.014);
                book.add(&line);
            }
        }
    }
    scatter(g, &book, x, z, 2.4);
    g.world.add(&book);
}

// R557 wall mail slot: 4% chance per level. Brass plate w/
// dark slot cavity + flap.
fn place_mail_slots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    flap.position().set(0, 0.02, 0.002); // 1 mm out: its back clears the cavity plane (hinge edge still in the plate)
    flap.rotation().x = -0.3;
    slot.add(&flap);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&slot, c, wall_x, wall_z, 1.10, Some(0.6));
    g.world.add(&slot);
}

// R557 tape roll: 10% chance per level. Adhesive tape roll w/
// hole + colored side.
fn place_tape_rolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R556 wall outlet: 7% chance per level. Cream plate w/ 2
// slot pairs + optional plug inserted.
fn place_wall_outlets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
        plug.position().set(0, 0.025, 0.010); // back on the plate face (z 0)
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
    mount(&outlet, c, wall_x, wall_z, 0.32, Some(0.6)); // Standard outlet height.
    g.world.add(&outlet);
}

// R556 crumpled receipt: 10% chance per level. Small crumpled
// paper ball w/ visible ink lines.
fn place_crumpled_receipts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R555 wall drain grate: 5% chance per level. Rectangular grate
// w/ 5 horizontal slots + dark cavity.
fn place_wall_drain_grates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    mount(&grate, c, wall_x, wall_z, 0.25, Some(0.6)); // Near floor.
    g.world.add(&grate);
}

// R555 post-it stack: 8% chance per level. 3-5 stacked
// colored sticky notes.
fn place_post_it_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R554 safety cone: 6% chance per level. Orange cone + white
// reflective stripe + square base.
fn place_safety_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
    stripe.scale().set(0.95, 2.96, 0.95); // hug the cone's taper (flared 5-15 mm off it); facet midpoints on the surface
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

// R554 wall door with peephole: 4% chance per level. Wooden
// door plane + handle + peephole disc.
fn place_wall_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R553 single sock: 10% chance per level: a crew sock lost and lying limp on
// the floor, grey with dirt (Blender batch r08: the knit body, its heel and
// toe, the two bands knitted round a sports sock's cuff).
fn place_single_socks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for the socks people lose: a white sports sock
    // (grey heel and toe, navy bands), black, burgundy, navy, grey marl
    let sock_colors = [0xf0ece0, 0x181410, 0xa02040, 0x2050c0, 0xd8b040];
    let (body, heel, bands) = match choice(&sock_colors) {
        0xf0ece0 => (0xb6b1a4, 0x86827a, Some(0x2e3550)),
        0x181410 => (0x2b2927, 0x34312d, None),
        0xa02040 => (0x5a2c30, 0x4a2528, None),
        0x2050c0 => (0x2e3446, 0x262b38, None),
        _ => (0x878480, 0x6a6763, Some(0x4a4844)),
    };
    let knit = sock_knit_tex();
    let sock = group();
    let mut part = |name: &str, color: u32| {
        if let Some(geo) = prop_part_as_modelled(name) {
            sock.add(&mesh(geo, lambert!({ map: &knit, color: color })));
        }
    };
    part("Real__sock", body);
    part("Real__sockHeelToe", heel);
    if let Some(b) = bands {
        part("Real__sockBands", b);
    }
    scatter(g, &sock, x, z, 2.4);
    g.world.add(&sock);
}
/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
fn r08_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The sock's cotton as it lies (laid over it from above: u along x, v
/// against z of `r08.py`'s sock): the knit's fine grain, washed-out patches,
/// and the dirt of a floor, heaviest at its toe (u 0.95, v 0.83) and heel
/// (u 0.3, v 0.7) where it was walked in.
fn sock_knit_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#e9e6df");
        ctx.fill_rect(0, 0, w, h);
        // the knit: fine wales, a fleck of fibre
        for i in 0..(w as i32) {
            if i % 3 == 0 {
                ctx.set_fill_style("rgba(120,116,108,0.10)");
                ctx.fill_rect(i as f64, 0, 1, h);
            }
        }
        for _ in 0..1600 {
            let v = 150.0 + rnd() * 100.0;
            ctx.set_fill_style(format!("rgba({v:.0},{v:.0},{:.0},0.22)", v - 6.0));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0 + rnd() * 2.0);
        }
        // greyed all over, in washes
        for _ in 0..40 {
            let at = (rnd() * w, rnd() * h);
            r08_soft(ctx, at, 18.0 + rnd() * 40.0, 0.6 + rnd() * 0.4, rnd() * 3.0, (110, 102, 90, 0.08 + rnd() * 0.12));
        }
        // walked in: the toe and the heel
        for (u, v, n) in [(0.95, 0.83, 40), (0.30, 0.70, 30)] {
            for _ in 0..n {
                let at = (u * w + (rnd() - 0.5) * 70.0, v * h + (rnd() - 0.5) * 60.0);
                r08_soft(ctx, at, 8.0 + rnd() * 16.0, 0.6 + rnd() * 0.4, rnd() * 3.0, (74, 64, 50, 0.10 + rnd() * 0.16));
            }
        }
        // a few bits of the floor caught in it
        for _ in 0..60 {
            ctx.set_fill_style(format!("rgba(60,52,40,{:.2})", 0.3 + rnd() * 0.4));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
    })
}

// R553 ceiling motion detector: 5% chance per level. Small
// dome + tiny LED indicator.
fn place_motion_detector_domes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let body_mat = lambert!({ color: 0xd0d0d0 });
    let dome_mat = basic_mat!({ color: 0x101820, transparent: true, opacity: 0.6 });
    let led_mat = basic_mat!({ color: 0x30d030 });
    let det = group();
    let dome_y = WALL_H - 0.005; // dome top = base underside; base top on the ceiling
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

// R552 peeled paint (`assets/src/blender/props/r11b.notes.md`): where damp
// came through a painted wall the paint let go: cracked into flakes along
// a network of fine cracks, the flakes come away in patches showing the
// older coat beneath, and under that the plaster (or the board's paper), a
// white bloom of salts on it; the edges of what is left a real step (the
// film's broken edge catching the light, its shadow below), lifted here
// and there, a few flakes curled right off the wall (modelled). On bare
// masonry it is what is left of an old coat: an island of paint flaking
// away from the middle out. Painted on a plane the old decal's size.
fn place_peeled_paint_patches(g: &mut Game, _dist: &[i32]) {
    use r11b_kit::*;
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let (pw, ph) = (0.50, 0.65);
    let patch = group();
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&patch, c, wall_x, wall_z, 1.4, Some(0.5));
    // what it is, from where it is (no draws)
    let p = patch.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    let key = wall_key_at(g, c.x, c.z);
    let painted = key.starts_with("wp:") || key.starts_with("drywall") || key.starts_with("paint");
    let flakes = Flakes::new(r.u(), painted, pw, ph);
    // the coats: the older one under the top coat (or the island's paint on
    // masonry), what is under that, and the back of a flake
    const COATS: [[f64; 3]; 4] = [[0.53, 0.58, 0.5], [0.71, 0.66, 0.53], [0.55, 0.59, 0.61], [0.69, 0.68, 0.64]];
    let coat = COATS[(r.f() * 4.0) as usize % 4];
    let base = if key.starts_with("drywall") { [0.62, 0.58, 0.5] } else { [0.72, 0.7, 0.64] };
    // a few flakes curled off the wall, at the edges of the holes
    let curls = flakes.curls(&mut r);
    let tex = peeled_paint_tex(&flakes, coat, base, &curls);
    let decal = mesh(plane_geo!(pw, ph), &decal_mat(&tex));
    decal.position().z = -0.007;
    patch.add(&decal);
    // the flakes: the top coat's face (the wall's own surface, or the
    // island's paint) and the film's back
    let top = if painted { g.arch_mat(&key).duplicate() } else { lambert!({ color: hex_of(coat) }) };
    let under = lambert!({ color: 0xcfcabd });
    for &(x, y, ang, k, sc) in &curls {
        for (part, mat) in [("Top", &top), ("Under", &under)] {
            let name = format!("Real__peeledPaintFlake{}{part}", ["A", "B", "C"][k]);
            let geo = game::assets::props::prop_part_as_modelled(&name).unwrap_or_else(|| box_geo!(0.03, 0.03, 0.01));
            let m = mesh(geo, mat);
            m.position().set(x, y, -0.008);
            m.rotation().z = ang;
            m.scale().set(sc, sc, sc);
            patch.add(&m);
        }
    }
    // a mark on the wall: the placement pass leaves it where the settle put it
    patch.user().floats = true;
    g.world.add(&patch);
}

// R552 broken flashlight: 8% chance per level. Metal cylinder
// w/ head lens (cracked) + faint dim glow inside.
fn place_broken_flashlights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R551 ceiling pipe run: 5% chance per level. Horizontal pipe
// segment attached to ceiling w/ 2 hanger straps.
fn place_ceiling_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R551 office lanyard w/ badge: 8% chance per level. Colored
// strap loop + ID badge card.
fn place_lanyards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R550 broken pen cluster: 8% chance per level. 4-8 pens
// (some snapped in half) scattered.
fn place_broken_pens(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R550 bottle caps: 10% chance per level: 8-14 crown caps prised off bottles
// and dropped in one place (Blender batch r08: 29 mm across, their 21 flutes
// crimped, their printed tops faded and scratched to the tin, some bent by
// the opener, some lying on their backs showing the liner).
fn place_bottle_caps(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for five printed caps (none a real brand)
    let cap_colors = [0xa02020, 0x30a030, 0x2050c0, 0xd8b040, 0xc0c0c0];
    let mut crowns: Vec<Option<Mat>> = vec![None; cap_colors.len()];
    let mut tops: Vec<Option<Mat>> = vec![None; cap_colors.len()];
    let liner = lambert!({ color: 0xb2ac9c });
    let grp = group();
    let mut down: Vec<(f64, f64)> = vec![];
    let n_caps = 8 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_caps {
        let col = choice(&cap_colors);
        let k = cap_colors.iter().position(|&c| c == col).unwrap_or(0);
        let px = (rnd() - 0.5) * 0.25;
        let pz = (rnd() - 0.5) * 0.25;
        // 25% on its back (it was edge-on)
        let flipped = rnd() < 0.25;
        let ry = rnd() * PI * 2.0;
        // a third bent by the opener (from where it fell: no new draw)
        let bent = ((px + 0.5) * 53.0).fract() < 0.34;
        // clear of the caps already down
        let (mut qx, mut qz) = (px, pz);
        for _ in 0..4 {
            for &(ax, az) in &down {
                let (dx, dz) = (qx - ax, qz - az);
                let d = (dx * dx + dz * dz).sqrt();
                if d < 0.031 {
                    let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (1.0, 0.0) };
                    qx += ux * (0.032 - d);
                    qz += uz * (0.032 - d);
                }
            }
        }
        down.push((qx, qz));
        let (crown_n, top_n, liner_n) = if bent {
            ("Real__capCrownBent", "Real__capTopBent", "Real__capLinerBent")
        } else {
            ("Real__capCrown", "Real__capTop", "Real__capLiner")
        };
        let Some(crown_geo) = prop_part_as_modelled(crown_n) else { continue };
        let height = crown_geo.bounding_box().max.y;
        let crown_mat = crowns[k].get_or_insert_with(|| cap_crown_mat(k)).clone();
        let cap = group();
        cap.add(&mesh(crown_geo, &crown_mat));
        if flipped {
            if let Some(geo) = prop_part_as_modelled(liner_n) {
                cap.add(&mesh(geo, &liner));
            }
            cap.rotation().x = PI;
            cap.position().set(qx, height, qz);
        } else {
            if let Some(geo) = prop_part_as_modelled(top_n) {
                let top_mat = tops[k].get_or_insert_with(|| cap_top_mat(k)).clone();
                cap.add(&mesh(geo, &top_mat));
            }
            cap.position().set(qx, 0, qz);
        }
        cap.rotation().y = ry;
        grp.add(&cap);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

/// The five caps' lacquer (sRGB): red, bottle green, navy, gold, tin.
const CAP_PAINT: [u32; 5] = [0x7e2a24, 0x2f4b33, 0x2b3a5e, 0x9a7c3a, 0x9c9c96];

/// A cap's skirt (round it, u; up it, v): its lacquer, the flutes' crests
/// rubbed to the tin, rust along the cut edge, scratches.
fn cap_crown_mat(k: usize) -> Mat {
    let paint = CAP_PAINT[k];
    let tex = canvas_tex(126, 32, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // 21 flutes: their crests rubbed bright, toward the edge
        for f in 0..21 {
            let x = (f as f64 + 0.5) / 21.0 * w;
            let gr = ctx.create_linear_gradient(0, 0, 0, h);
            gr.add_color_stop(0, "rgba(176,174,166,0)");
            gr.add_color_stop(1, "rgba(176,174,166,0.55)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(x - 1.0, h * 0.3, 2.0, h * 0.7);
        }
        // rust at the cut edge (it is tinplate: the edge is bare steel)
        for _ in 0..50 {
            ctx.set_fill_style(format!("rgba({:.0},{:.0},30,{:.2})", 100.0 + rnd() * 30.0, 56.0 + rnd() * 14.0, 0.35 + rnd() * 0.45));
            ctx.fill_rect(rnd() * w, h - 1.0 - rnd() * rnd() * 10.0, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        ctx.set_line_width(0.7);
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_stroke_style(format!("rgba(178,176,168,{:.2})", 0.3 + rnd() * 0.4));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + (rnd() - 0.5) * 16.0, y + (rnd() - 0.5) * 6.0);
            ctx.stroke();
        }
    });
    phong!({ map: &tex, color: 0xffffff, specular: 0x484848, shininess: 46 })
}

/// A cap's printed top (seen from above: the disc fills the square): a
/// plain mark of a beer or a soda that never was, faded, rubbed, scratched
/// to the tin, a ring pressed in it by the bottle's lip.
fn cap_top_mat(k: usize) -> Mat {
    let paint = CAP_PAINT[k];
    let tex = canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let ink = if k == 3 || k == 4 { "rgba(40,30,22,0.85)" } else { "rgba(226,220,204,0.85)" };
        ctx.set_fill_style(ink);
        ctx.set_stroke_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let ring = |ctx: &mut Canvas, rr: f64, lw: f64| {
            ctx.set_line_width(lw);
            ctx.begin_path();
            ctx.arc(cx, cy, rr, 0, 7);
            ctx.stroke();
        };
        match k {
            0 => {
                ring(ctx, 50.0, 3.0);
                ctx.set_font("bold 25px Futura");
                ctx.fill_text_max("LAGER", cx, cy + 6.0, 86);
                // a small star over it
                ctx.begin_path();
                for i in 0..10 {
                    let a = -PI / 2.0 + i as f64 * PI / 5.0;
                    let rr = if i % 2 == 0 { 9.0 } else { 3.8 };
                    let (x, y) = (cx + a.cos() * rr, cy - 22.0 + a.sin() * rr);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
            1 => {
                ring(ctx, 52.0, 1.5);
                ring(ctx, 46.0, 1.5);
                ctx.set_font("bold 19px Gill Sans");
                ctx.fill_text_max("PILSNER", cx, cy, 84);
                ctx.set_font("10px Gill Sans");
                ctx.fill_text("BREWED SINCE 1898", cx, cy + 20.0);
            }
            2 => {
                ctx.set_font("bold 30px Futura");
                ctx.fill_text_max("SODA", cx, cy - 4.0, 90);
                ctx.set_line_width(2.5);
                ctx.begin_path();
                ctx.move_to(cx - 34.0, cy + 18.0);
                ctx.bezier_curve_to(cx - 14.0, cy + 10.0, cx + 6.0, cy + 26.0, cx + 34.0, cy + 16.0);
                ctx.stroke();
            }
            3 => {
                // a shield, EXPORT across it
                ctx.set_line_width(2.5);
                ctx.begin_path();
                ctx.move_to(cx - 26.0, cy - 30.0);
                ctx.line_to(cx + 26.0, cy - 30.0);
                ctx.line_to(cx + 26.0, cy + 2.0);
                ctx.quadratic_curve_to(cx + 24.0, cy + 26.0, cx, cy + 36.0);
                ctx.quadratic_curve_to(cx - 24.0, cy + 26.0, cx - 26.0, cy + 2.0);
                ctx.close_path();
                ctx.stroke();
                ctx.set_font("bold 15px Rockwell");
                ctx.fill_text_max("EXPORT", cx, cy - 2.0, 50);
            }
            _ => {
                ring(ctx, 50.0, 2.0);
                for i in 0..24 {
                    let a = i as f64 / 24.0 * PI * 2.0;
                    ctx.begin_path();
                    ctx.arc(cx + a.cos() * 42.0, cy + a.sin() * 42.0, 1.8, 0, 7);
                    ctx.fill();
                }
                ctx.set_font("bold 26px Copperplate");
                ctx.fill_text_max("BEER", cx, cy, 70);
            }
        }
        // faded: the print rubbed thin in patches, the lacquer worn off
        ctx.set_global_alpha(1.0);
        for _ in 0..26 {
            let at = (rnd() * w, rnd() * h);
            r08_soft(ctx, at, 8.0 + rnd() * 16.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (r as u8, g as u8, b as u8, 0.25 + rnd() * 0.45));
        }
        // scratched to the tin
        ctx.set_line_width(0.9);
        for _ in 0..22 {
            let (x, y, a) = (rnd() * w, rnd() * h, rnd() * PI);
            let l = 6.0 + rnd() * 40.0;
            ctx.set_stroke_style(format!("rgba(186,184,176,{:.2})", 0.35 + rnd() * 0.45));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // the ring the bottle's lip pressed into it, a little bright
        ctx.set_stroke_style("rgba(190,186,176,0.25)");
        ctx.set_line_width(2.0);
        ctx.begin_path();
        ctx.arc(cx, cy, 56.0, 0, 7);
        ctx.stroke();
        // dirt
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(60,52,40,{:.2})", 0.1 + rnd() * 0.25));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
        }
    });
    phong!({ map: &tex, color: 0xffffff, specular: 0x404040, shininess: 44 })
}

// R549 wall fuse box: 4% chance per level. Gray metal box w/
// hinge + 6-8 switches inside.
fn place_fuse_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R549 hanging bare bulb: 5% chance per level. Bulb + socket +
// cord from ceiling + faint warm glow.
fn place_hanging_light_bulbs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R548 wall intercom: 4% chance per level. Small speaker box +
// grille + button.
fn place_wall_intercoms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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

// R548 paperclip cluster: 10% chance per level. 6-14 metal
// paperclip torus scattered.
fn place_paperclip_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R547 ceiling sprinkler head: 6% chance per level. Small
// brass dome + 4 deflector arms hanging from ceiling.
fn place_sprinkler_heads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let brass_mat = lambert!({ color: 0xc89040 });
    let red_mat = lambert!({ color: 0xd82040 });
    let spr = group();
    let spr_y = WALL_H; // stub top on the ceiling (was 2 cm below it)
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

// R547 air freshener can: 10% chance per level. Small aerosol
// can w/ cap.
fn place_air_fresheners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R546 emergency phone: 4% chance per level. Red wall box +
// handset on hook + "EMERGENCY" label.
fn place_emergency_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    handset.position().set(0, -0.02, 0.016); // back 2 mm proud of the face, over the trough
    phone.add(&handset);
    // Ear/mouth cups.
    for s in [-1.0, 1.0] {
        let cup = mesh(pg("EmergencyPhones__cup", sphere_geo!(0.018, 8, 6)), &dark_mat);
        cup.position().set(s * 0.055, -0.02, 0.018); // dished face resting on the box face (z 0)
        phone.add(&cup);
    }
    // box (local z −0.09…0) back on the wall, not half buried in it
    let (wall_x, wall_z) = wall_xz(g, c, 0.09);
    mount(&phone, c, wall_x, wall_z, 1.3, Some(0.4));
    g.world.add(&phone);
}

// R546 dropped cell phone: 10% chance per level. Small
// rectangular phone (flip or slab).
fn place_dropped_cell_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R545 wall pipe leak (`assets/src/blender/props/r11b.notes.md`): a short
// run of 2 in steel pipe out of the wall, along it and back in (batch r06's
// malleable elbows, a coupling, the collars at the wall), painted once and
// rusting; one elbow's thread by the wall weeps: a crust of lime and rust
// under it, a drop hanging, and down the wall from its collar the wet run,
// rust-brown at its edges, and the stain where it spread and dried.
fn place_wall_pipe_leaks(g: &mut Game, _dist: &[i32]) {
    use r11b_kit::*;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let leak = group();
    // the old pipe's axis 2.6 cm off the wall: the group's origin stays there
    let (wall_x, wall_z) = wall_xz(g, c, 0.026);
    mount(&leak, c, wall_x, wall_z, 1.90, Some(0.3));
    let p = leak.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    // the frame: the wall's face at z = WALL; the run's axis 85 mm off it, as
    // high as keeps the collars' tops where the old pipe's top was; its
    // elbows' collars at the old pipe's ends
    const WALL: f64 = -0.026;
    const EA: f64 = 0.057; // a 2 in elbow, centre to end
    const IN: f64 = 0.017; // a thread into its fitting
    let (zp, ya, xe) = (WALL + 0.085, 0.025927 - 0.060, 0.300 - 0.0575);
    let paint = (r.f() * 3.0) as usize % 3;
    let pipe_mat = lambert!({ map: &pipe_paint_tex(paint, false), color: 0xffffff });
    let fit_mat = lambert!({ map: &pipe_paint_tex(paint, true), color: 0xffffff });
    let plate = phong!({ color: 0x77766f, specular: 0x262626, shininess: 26 });
    let part = |name: &str, w: f64| game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, w, w));
    // the run, between the elbows, through a coupling
    let xc = (r.f() - 0.5) * 0.12;
    leak.add(&pipe_length(&pipe_mat, v3(-xe + EA - IN, ya, zp), v3(xc - 0.015, ya, zp)));
    leak.add(&pipe_length(&pipe_mat, v3(xc + 0.015, ya, zp), v3(xe - EA + IN, ya, zp)));
    let cp = mesh(part("Real__pipeCoupling", 0.064), &fit_mat);
    cp.position().set(xc, ya, zp);
    leak.add(&cp);
    for sx in [-1.0, 1.0] {
        let x = sx * xe;
        // the elbow (its legs +x and +y as modelled): one leg along the run,
        // the other back into the wall
        let el = group();
        el.position().set(x, ya, zp);
        el.rotation().y = if sx < 0.0 { 0.0 } else { PI };
        let m = mesh(part("Real__pipeElbow", 0.08), &fit_mat);
        // its +y leg to -z (into the wall); the right one is turned round
        // after (its +x leg to -x), so its +y leg goes to +z first
        m.rotation().x = sx * PI / 2.0;
        el.add(&m);
        leak.add(&el);
        leak.add(&pipe_length(&pipe_mat, v3(x, ya, zp - EA + IN), v3(x, ya, WALL)));
        let col = mesh(part("Real__pipeCollar", 0.115), &plate);
        col.position().set(x, ya, WALL);
        leak.add(&col);
    }
    // the leaking one, at its thread by the wall
    let side = if r.f() < 0.5 { -1.0 } else { 1.0 };
    let lx = side * xe;
    let crust = mesh(part("Real__pipeLeakCrust", 0.03), &lambert!({ color: 0x9c8466 }));
    crust.position().set(lx, ya - 0.0385, zp - EA + 0.006);
    crust.rotation().y = r.f() * 0.6 - 0.3;
    leak.add(&crust);
    let drop = mesh(
        part("Real__pipeLeakDrop", 0.006),
        &phong!({ color: 0x4a5256, specular: 0xd8dcdc, shininess: 120, transparent: true, opacity: 0.55 }),
    );
    drop.position().set(lx - 0.003, ya - 0.0385 - 0.0128, zp - EA + 0.007);
    leak.add(&drop);
    // down the wall from its collar: the wet run and its stain, to where
    // the old stain's foot was
    let (top, bot) = (ya + 0.03, -0.26);
    let (w, hgt) = (0.10, top - bot);
    let run = mesh(plane_geo!(w, hgt), &decal_mat(&pipe_leak_tex(r.u(), w, hgt, top - (ya - 0.0575))));
    run.position().set(lx, (top + bot) / 2.0, WALL + 0.001);
    leak.add(&run);
    g.world.add(&leak);
}

// R545 unrolled TP: 8% chance per level. Cardboard tube +
// unrolled paper trail.
fn place_unrolled_tp_rolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R544 emergency strobe: 3% chance per level. Red dome bulb
// on wall base + occasional flash animator.
fn place_emergency_strobes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    dome.rotation().x = PI / 2.0; // +π/2 turns the hemisphere's +Y cap to +Z (into the room, not the wall)
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
        game::debug::testkit::smoke_placers(&list);
    }
}

/// Batch r11b's painting kit (the marks on the walls): hashed noise, a seeded
/// generator for a mark's layout (no level draws), and a per-pixel painter
/// filled on every core (`par_rows`).
mod r11b_kit {
    use game::prelude::*;

    /// A small generator for a mark's own layout, seeded from where it is
    pub struct Rng(pub u32);
    impl Rng {
        pub fn u(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
        /// in [0, 1)
        pub fn f(&mut self) -> f64 {
            f64::from(self.u()) / 4_294_967_296.0
        }
    }

    /// A seed from values already fixed (a position, a side): no draws
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

    /// A hash of (seed, i, j) in [0, 1)
    pub fn h01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    /// Smooth value noise in [0, 1]
    pub fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (h01(seed, i, j), h01(seed, i + 1, j));
        let (c, d) = (h01(seed, i, j + 1), h01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    /// Fractal value noise of `oct` octaves, in [0, 1]
    pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }

    /// 0 below `e0`, 1 above `e1`, smooth between
    pub fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    /// `c` at alpha `a` over straight colour and alpha `dst`
    pub fn lay(dst: &mut [f64; 4], c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let out = a + dst[3] * (1.0 - a);
        if out > 1e-6 {
            for k in 0..3 {
                dst[k] = (c[k] * a + dst[k] * dst[3] * (1.0 - a)) / out;
            }
        }
        dst[3] = out;
    }

    /// Specks: a dot in each `cell` (m) with likelihood `like`, its radius
    /// `r0` + up to `r1`, anti-aliased over `aa`: (coverage, the dot's hash)
    #[allow(clippy::too_many_arguments)]
    pub fn specks(seed: u32, x: f64, y: f64, cell: f64, like: f64, r0: f64, r1: f64, aa: f64) -> (f64, f64) {
        let (gi, gj) = ((x / cell).floor() as i32, (y / cell).floor() as i32);
        let (mut best, mut hk) = (0.0f64, 0.0);
        for k in 0..9 {
            let (ci, cj) = (gi + k % 3 - 1, gj + k / 3 - 1);
            let h = h01(seed, ci, cj);
            if h >= like {
                continue;
            }
            let (px, py) = ((f64::from(ci) + h01(seed ^ 0x77, ci, cj)) * cell, (f64::from(cj) + h01(seed ^ 0x78, ci, cj)) * cell);
            let r = r0 + r1 * h01(seed ^ 0x79, ci, cj);
            let cov = (0.5 - ((x - px).hypot(y - py) - r) / aa).clamp(0.0, 1.0);
            if cov > best {
                best = cov;
                hk = h01(seed ^ 0x7a, ci, cj);
            }
        }
        (best, hk)
    }

    /// A transparent mark `tw` x `th` px across a plane `pw` x `ph` m: each
    /// pixel `px(x, y, aa)` (m from the plane's top left, y down; `aa` a
    /// pixel's size) in straight sRGB and alpha. Where it is clear its colour
    /// is `bleed`, so a mark's edge does not filter toward black.
    pub fn decal_tex(tw: usize, th: usize, pw: f64, ph: f64, bleed: [f64; 3], px: impl Fn(f64, f64, f64) -> [f64; 4] + Sync) -> Tex {
        let tex = canvas_tex_opt(tw as f64, th as f64, false, move |ctx, cw, ch| {
            let (sx, sy) = (pw / cw, ph / ch);
            let aa = sx.max(sy) * 1.1;
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, tw * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..tw {
                    let x = (i as f64 + 0.5) * sx;
                    let p = px(x, y, aa);
                    for (c, v) in p.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        });
        // what is clear (it came back black through the canvas) takes the
        // bleed colour, and what is nearly clear goes toward it
        {
            let img = tex.image();
            let mut im = img.borrow_mut();
            for p in im.data.chunks_exact_mut(4) {
                let t = smooth(0.0, 0.06, f64::from(p[3]) / 255.0);
                for (c, b) in p.iter_mut().zip(bleed) {
                    *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * t).round().clamp(0.0, 255.0) as u8;
                }
            }
            im.version += 1;
        }
        tex
    }

    /// The decal material: lit, see-through, drawn over the wall
    pub fn decal_mat(tex: &Tex) -> Mat {
        lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
    }

    /// The surface of the wall seen from open tile (`x`, `z`): its arch
    /// material's key (`wp:<variant>` a papered wall), as the walls are drawn
    pub fn wall_key_at(g: &Game, x: i32, z: i32) -> String {
        if g.layout.is_none() || !g.in_grid(x, z) {
            return "wp:0".into();
        }
        g.wall_mat_key(g.t_idx(x, z))
    }

    /// An opaque, repeating surface `tw` x `th` px over `pw` x `ph` m: each
    /// pixel `px(x, y)` (m from its top left) in sRGB
    pub fn surface_tex(tw: usize, th: usize, pw: f64, ph: f64, px: impl Fn(f64, f64) -> [f64; 4] + Sync) -> Tex {
        canvas_tex_opt(tw as f64, th as f64, true, move |ctx, cw, ch| {
            let (sx, sy) = (pw / cw, ph / ch);
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, tw * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..tw {
                    let p = px((i as f64 + 0.5) * sx, y);
                    for (c, v) in p.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        })
    }

    /// sRGB (0-1) as a hex colour
    pub fn hex_of(c: [f64; 3]) -> u32 {
        c.iter().fold(0u32, |h, v| (h << 8) | (v.clamp(0.0, 1.0) * 255.0).round() as u32)
    }

    /// The paint's flakes on a patch `pw` x `ph` (x right, y up, from its
    /// middle): cells of a cracked film (jittered Voronoi, its edges wavering),
    /// each gone or still there by how far the damp got (`env`), and under the
    /// gone ones the older coat, itself gone in places.
    pub struct Flakes {
        pub seed: u32,
        pub painted: bool,
        pub pw: f64,
        pub ph: f64,
        /// the patch's middle and its half extents
        pub mid: (f64, f64),
        pub reach: (f64, f64),
    }

    /// What the film is at a point
    pub struct Here {
        pub gone: bool,
        /// how far (m) to the edge between gone and still there
        pub d: f64,
        /// that edge's normal, from what is there into what is gone (y up)
        pub n: (f64, f64),
        /// the cell still there beside it is lifted at its edge
        pub lifted: bool,
        /// how far to the nearest crack between two cells
        pub crack: f64,
        /// the older coat gone here too, and how far to its edge, its normal
        pub deep: bool,
        pub dd: f64,
        pub dn: (f64, f64),
        /// how far the damp got (0-1)
        pub env: f64,
    }

    impl Flakes {
        const CELL: f64 = 0.017;
        const SUB: f64 = 0.014;

        pub fn new(seed: u32, painted: bool, pw: f64, ph: f64) -> Self {
            let mut r = Rng(seed | 1);
            let mid = ((r.f() - 0.5) * 0.06, (r.f() - 0.5) * 0.08);
            let reach = (0.18 + 0.03 * r.f(), 0.23 + 0.04 * r.f());
            Flakes { seed: r.u(), painted, pw, ph, mid, reach }
        }

        /// How far the damp got, 0-1 (none by the plane's edges)
        pub fn env(&self, x: f64, y: f64) -> f64 {
            let (dx, dy) = ((x - self.mid.0) / self.reach.0, (y - self.mid.1) / self.reach.1);
            let q = dx.hypot(dy) * (1.0 + 0.5 * (fbm(self.seed ^ 0x61, x * 7.0, y * 7.0, 3) - 0.5));
            smooth(1.0, 0.25, q) * smooth(0.0, 0.03, (self.pw / 2.0 - x.abs()).min(self.ph / 2.0 - y.abs()))
        }

        /// Whether a cell (its point at `c`, its hash `h`) has lost its paint
        pub fn cell_gone(&self, c: (f64, f64), h: f64) -> bool {
            let e = self.env(c.0, c.1);
            if self.painted {
                h < 0.9 * smooth(0.06, 0.5, e)
            } else {
                h >= 0.97 * smooth(0.0, 0.2, e)
            }
        }

        /// The nearest cell point to `q` on a grid of `cell` (each cell's point
        /// in its middle 70%, so the nearest is among the 3 x 3 round it), and
        /// the nearest edge to a cell in the other state (`state`): (its state,
        /// the distance to that edge, the edge's normal toward the other cell,
        /// the distance to any edge, this cell, the other cell)
        #[allow(clippy::type_complexity)]
        pub fn voronoi(
            &self,
            salt: u32,
            cell: f64,
            q: (f64, f64),
            state: &dyn Fn((f64, f64), i32, i32) -> bool,
        ) -> (bool, f64, (f64, f64), f64, (i32, i32), (i32, i32)) {
            let s = self.seed ^ salt;
            let (gi, gj) = ((q.0 / cell).floor() as i32, (q.1 / cell).floor() as i32);
            let mut pts = [((0.0, 0.0), (0, 0)); 9];
            let mut best = (f64::INFINITY, 0usize);
            for (k, pt) in pts.iter_mut().enumerate() {
                let (i, j) = (gi + k as i32 % 3 - 1, gj + k as i32 / 3 - 1);
                let c = ((f64::from(i) + 0.15 + 0.7 * h01(s, i, j)) * cell, (f64::from(j) + 0.15 + 0.7 * h01(s ^ 0x5, i, j)) * cell);
                let d = (q.0 - c.0).hypot(q.1 - c.1);
                *pt = (c, (i, j));
                if d < best.0 {
                    best = (d, k);
                }
            }
            let (a, ija) = pts[best.1];
            let ga = state(a, ija.0, ija.1);
            // the distance to the bisector with each other point
            let (mut d_edge, mut n, mut d_crack, mut ijb) = (f64::INFINITY, (0.0, 1.0), f64::INFINITY, ija);
            for (k, &(b, ij)) in pts.iter().enumerate() {
                if k == best.1 {
                    continue;
                }
                let (vx, vy) = (b.0 - a.0, b.1 - a.1);
                let l = vx.hypot(vy).max(1e-9);
                let d = ((a.0 + b.0) / 2.0 - q.0) * vx / l + ((a.1 + b.1) / 2.0 - q.1) * vy / l;
                d_crack = d_crack.min(d);
                if d < d_edge && state(b, ij.0, ij.1) != ga {
                    d_edge = d;
                    n = (vx / l, vy / l);
                    ijb = ij;
                }
            }
            (ga, d_edge, n, d_crack, ija, ijb)
        }

        pub fn at(&self, x: f64, y: f64) -> Here {
            // the cracks waver
            let q = (
                x + 0.004 * (fbm(self.seed ^ 0x71, x * 55.0, y * 55.0, 2) - 0.5)
                    + 0.0012 * (vnoise(self.seed ^ 0x72, x * 260.0, y * 260.0) - 0.5),
                y + 0.004 * (fbm(self.seed ^ 0x73, x * 55.0, y * 55.0, 2) - 0.5)
                    + 0.0012 * (vnoise(self.seed ^ 0x74, x * 260.0, y * 260.0) - 0.5),
            );
            let s = self.seed;
            let st = |c: (f64, f64), i: i32, j: i32| self.cell_gone(c, h01(s ^ 0x11, i, j));
            let (gone, d, n0, crack, ija, ijb) = self.voronoi(0x1, Self::CELL, q, &st);
            // n0 points from this cell to the other: make it from what is there to what is gone
            let n = if gone { (-n0.0, -n0.1) } else { n0 };
            // the cell still there beside the edge (this one, or the one across it) is lifted at it
            let kept = if gone { ijb } else { ija };
            let lifted = h01(s ^ 0x13, kept.0, kept.1) < 0.45;
            let env = self.env(x, y);
            // the older coat, gone in its own smaller flakes where the damp was worst
            let (deep, dd, dn) = if gone && self.painted {
                let sd = |c: (f64, f64), i: i32, j: i32| h01(s ^ 0x21, i, j) < 0.4 * smooth(0.5, 1.0, self.env(c.0, c.1)) + 0.04;
                let (dp, dd, dn0, _, _, _) = self.voronoi(0x2, Self::SUB, q, &sd);
                (dp, dd, if dp { (-dn0.0, -dn0.1) } else { dn0 })
            } else {
                (false, f64::INFINITY, (0.0, 1.0))
            };
            Here { gone, d, n, lifted, crack, deep, dd, dn, env }
        }

        /// Where flakes have curled off: 3-5 points on the edge of a hole, each
        /// (x, y, its turn on the wall, which flake, its size)
        pub fn curls(&self, r: &mut Rng) -> Vec<(f64, f64, f64, usize, f64)> {
            let mut found: Vec<(f64, f64, (f64, f64))> = vec![];
            let (nx, ny) = (36, 46);
            for j in 0..ny {
                for i in 0..nx {
                    let (x, y) =
                        ((i as f64 + 0.5) / nx as f64 * self.pw - self.pw / 2.0, (j as f64 + 0.5) / ny as f64 * self.ph - self.ph / 2.0);
                    if x.abs() > self.pw / 2.0 - 0.05 || y.abs() > self.ph / 2.0 - 0.05 {
                        continue;
                    }
                    let h = self.at(x, y);
                    // at a hole's edge, on the paint's side, a hole beyond it wide enough to have held the flake
                    if h.gone || h.d > 0.004 {
                        continue;
                    }
                    let (bx, by) = (x + h.n.0 * (h.d + 0.022), y + h.n.1 * (h.d + 0.022));
                    if !self.at(bx, by).gone || !self.at(x + h.n.0 * (h.d + 0.01), y + h.n.1 * (h.d + 0.01)).gone {
                        continue;
                    }
                    found.push((x + h.n.0 * h.d, y + h.n.1 * h.d, h.n));
                }
            }
            let mut out: Vec<(f64, f64, f64, usize, f64)> = vec![];
            let want = 3 + (r.f() * 3.0) as usize;
            let mut tries = 0;
            while out.len() < want && !found.is_empty() && tries < 60 {
                tries += 1;
                let (x, y, n) = found[(r.f() * found.len() as f64) as usize % found.len()];
                if out.iter().any(|o| (o.0 - x).hypot(o.1 - y) < 0.05) {
                    continue;
                }
                // the flake's free side (its +y) into the hole
                let ang = (-n.0).atan2(n.1);
                out.push((x, y, ang, (r.f() * 3.0) as usize % 3, 0.85 + 0.3 * r.f()));
            }
            out
        }
    }

    /// The patch painted: holes in the top coat (on a painted wall: the wall
    /// itself is the top coat, clear here) or the island of paint (on
    /// masonry), their edges, the coats and plaster under them, the curled
    /// flakes' shadows.
    pub fn peeled_paint_tex(f: &Flakes, coat: [f64; 3], base: [f64; 3], curls: &[(f64, f64, f64, usize, f64)]) -> Tex {
        const SHADE: [f64; 3] = [0.04, 0.035, 0.03];
        const LIT: [f64; 3] = [0.93, 0.92, 0.88];
        const SALT: [f64; 3] = [0.9, 0.9, 0.86];
        // the flakes' sizes (A, B, C as modelled): free length
        const LEN: [f64; 3] = [0.024, 0.016, 0.030];
        let shadows: Vec<(f64, f64, f64)> = curls
            .iter()
            .map(|&(x, y, ang, k, sc)| {
                let l = LEN[k] * sc;
                // the curl stands over the hole beside its foot; its shadow falls below it
                let (nx, ny) = (-ang.sin(), ang.cos());
                (x + nx * l * 0.35, y + ny * l * 0.35 - 0.004, l * 0.55)
            })
            .collect();
        let (pw, ph, painted, s) = (f.pw, f.ph, f.painted, f.seed);
        let tex = decal_tex(400, 520, pw, ph, coat, move |cx, cy, aa| {
            let (x, y) = (cx - pw / 2.0, ph / 2.0 - cy);
            let h = f.at(x, y);
            let mut px = [coat[0], coat[1], coat[2], 0.0];
            let grain = fbm(s ^ 0x81, x * 180.0, y * 180.0, 2);
            if h.gone == painted {
                // what shows in a hole (on a painted wall), or the island (on masonry)
                if painted {
                    let c = if h.deep {
                        let m = 0.9 + 0.14 * grain;
                        [base[0] * m, base[1] * m, base[2] * m]
                    } else {
                        let m = 0.94 + 0.08 * grain;
                        [coat[0] * m, coat[1] * m, coat[2] * m]
                    };
                    px = [c[0], c[1], c[2], (0.5 + h.d / aa).clamp(0.0, 1.0)];
                    // the older coat's own broken edge, and its shadow
                    if !h.deep && h.dd < 0.0015 {
                        lay(&mut px, LIT, 0.4 * h.dn.1.max(0.0) * smooth(0.0007, 0.0, h.dd));
                    } else if h.deep && h.dd < 0.002 {
                        lay(&mut px, SHADE, 0.35 * (-h.dn.1).max(0.0) * smooth(0.0012, 0.0, h.dd));
                    }
                    // dust and grime settled in it
                    lay(&mut px, [0.3, 0.27, 0.22], 0.18 * fbm(s ^ 0x85, x * 30.0, y * 30.0, 3));
                    // salts the damp brought out, worst in the middle
                    lay(&mut px, SALT, 0.35 * smooth(0.5, 0.9, h.env) * smooth(0.55, 0.8, fbm(s ^ 0x82, x * 40.0, y * 40.0, 3)));
                } else {
                    let m = 0.93 + 0.1 * grain;
                    px = [coat[0] * m, coat[1] * m, coat[2] * m, (0.5 + h.d / aa).clamp(0.0, 1.0)];
                    // grime on the old paint
                    lay(&mut px, [0.35, 0.32, 0.27], 0.2 * fbm(s ^ 0x83, x * 25.0, y * 25.0, 3));
                }
            }
            if h.gone {
                // in a hole, just past the edge: the shadow of the film's step
                // where the edge is above it (and wider under a lifted edge)
                let below = (-h.n.1).max(0.0);
                let w = if h.lifted { 0.0035 } else { 0.0012 };
                lay(&mut px, SHADE, (0.55 * below + 0.12) * (-h.d / w).exp() * smooth(-aa, 0.0, h.d));
            } else {
                // the film's broken edge catching the light where it faces up
                let up = h.n.1.max(0.0);
                lay(&mut px, LIT, (0.5 * up + 0.15) * smooth(0.0009, 0.0002, h.d));
                // a lifted edge: tipped toward the light or away from it
                if h.lifted {
                    let k = smooth(0.004, 0.0, h.d);
                    if h.n.1 > 0.0 {
                        lay(&mut px, LIT, 0.3 * h.n.1 * k);
                    } else {
                        lay(&mut px, SHADE, 0.25 * -h.n.1 * k);
                    }
                }
                // the cracks the film broke along before it let go
                let ck = smooth(0.1, 0.45, h.env) * smooth(0.25, 0.6, vnoise(s ^ 0x84, x * 90.0, y * 90.0));
                lay(&mut px, SHADE, 0.45 * ck * (0.5 - (h.crack - 0.00025) / aa).clamp(0.0, 1.0));
            }
            // the curled flakes' shadows
            for &(sx, sy, sr) in &shadows {
                let q = (x - sx).hypot((y - sy) * 1.4) / sr;
                if q < 1.5 {
                    lay(&mut px, SHADE, 0.32 * (-q * q * 2.0).exp());
                }
            }
            px
        });
        tex
    }

    /// A length of 2 in pipe (60.3 mm) from `a` to `b` along one axis, its
    /// paint wrapped round it at its real scale.
    pub fn pipe_length(mat: &Mat, a: Vec3, b: Vec3) -> Obj {
        let d = b - a;
        let len = d.length().max(0.001);
        let geo = cyl_geo!(0.03015, 0.03015, len, 16, 1, true);
        let uv: Vec<f32> = geo.r().uv.clone();
        // the texture is 0.4 m along
        let uv: Vec<f32> = uv.iter().enumerate().map(|(i, &v)| if i % 2 == 1 { v * (len / 0.4) as f32 } else { v }).collect();
        geo.set_uv(uv);
        let m = mesh(geo, mat);
        m.position().set((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0);
        if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
            m.rotation().z = PI / 2.0;
        } else if d.z.abs() > d.y.abs() {
            m.rotation().x = PI / 2.0;
        }
        m
    }

    /// A pipe's old paint (`paint`: grey, green, cream) flaked off its steel
    /// in patches, rust in them and blooming round them, a run of rust along
    /// its underside, dust on its top; a fitting's (`fitting`) rustier.
    /// Round it 0.19 m, along it 0.4 m (one per kind, shared).
    pub fn pipe_paint_tex(paint: usize, fitting: bool) -> Tex {
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; 6]); }
        let i = paint % 3 + if fitting { 3 } else { 0 };
        T.with(|t| {
            if let Some(x) = &t.borrow()[i] {
                return x.clone();
            }
            const PAINTS: [[f64; 3]; 3] = [[0.37, 0.38, 0.37], [0.34, 0.38, 0.31], [0.6, 0.57, 0.48]];
            let pc = PAINTS[paint % 3];
            let rusty = if fitting { 0.56 } else { 0.62 };
            let tex = surface_tex(96, 256, 0.19, 0.4, move |x, y| {
                let seed = 0x9e3 + paint as u32 * 31 + u32::from(fitting) * 7;
                // round the pipe: x 0-0.19 (a run's underside at 0.1425, its
                // top at 0.0475), along it: y
                let under = (-((x - 0.1425) / 0.03).powi(2)).exp();
                let top = (-((x - 0.0475) / 0.03).powi(2)).exp();
                let fl = fbm(seed, x * 70.0, y * 70.0, 4);
                let bare = smooth(rusty, rusty + 0.03, fl + 0.12 * under);
                let bloom = smooth(rusty - 0.12, rusty, fl + 0.12 * under) * (1.0 - bare);
                let rust = mix([0.24, 0.14, 0.08], [0.38, 0.21, 0.1], fbm(seed ^ 0x3, x * 300.0, y * 300.0, 2));
                let mut c = mix(pc, rust, bare);
                c = mix(c, [0.36, 0.22, 0.12], 0.4 * bloom);
                c = mix(c, [0.32, 0.19, 0.1], 0.35 * under);
                // grime
                c = mix(c, [0.2, 0.18, 0.15], 0.25 * fbm(seed ^ 0x7, x * 25.0, y * 25.0, 3));
                c = mix(c, [0.52, 0.5, 0.45], 0.25 * top);
                let m = 0.92 + 0.12 * fbm(seed ^ 0x5, x * 140.0, y * 30.0, 2);
                [c[0] * m, c[1] * m, c[2] * m, 1.0]
            });
            t.borrow_mut()[i] = Some(tex.clone());
            tex
        })
    }

    /// The run down the wall under a leaking collar, across a plane `pw` x `ph`
    /// (y down from its top; the collar's foot `from` down): wet and dark under
    /// the collar, rust at its edges, narrowing, then where it spread and
    /// dried, a stain with its tide lines.
    pub fn pipe_leak_tex(seed: u32, pw: f64, ph: f64, from: f64) -> Tex {
        let mut r = Rng(seed | 1);
        let s = r.u();
        const WET: [f64; 3] = [0.22, 0.16, 0.09];
        const STAIN: [f64; 3] = [0.5, 0.37, 0.2];
        const RUST: [f64; 3] = [0.45, 0.25, 0.1];
        const TIDE: [f64; 3] = [0.3, 0.18, 0.08];
        let (tw, th) = (96usize, 288usize);
        let sx = pw / tw as f64;
        let mid = pw / 2.0 + (r.f() - 0.5) * 0.008;
        // how far down it got below the collar's foot (m), in tongues: (x,
        // half width, depth), the deepest first
        let reach = ph - from - 0.02 - 0.03 * r.f();
        let lobes: Vec<(f64, f64, f64)> = (0..2 + (r.f() * 2.0) as usize)
            .map(|k| {
                if k == 0 {
                    (mid + (r.f() - 0.5) * 0.008, 0.024 + 0.01 * r.f(), reach)
                } else {
                    (mid + (r.f() - 0.5) * 0.05, 0.008 + 0.01 * r.f(), reach * (0.35 + 0.45 * r.f()))
                }
            })
            .collect();
        // each wetting's reach at each column, and its edge's slope; the
        // last (outermost) first, the earlier ones inside it
        let rims: Vec<f64> = vec![1.0, 0.78 + 0.08 * r.f(), 0.55 + 0.1 * r.f()];
        let reach_at = |x: f64, k: usize| -> f64 {
            let mut l = 0.004f64;
            for &(c, half, depth) in &lobes {
                let u = (x - c) / half;
                if u.abs() < 1.0 {
                    let tongue = depth * rims[k] * (1.0 - u.abs().powf(2.2)).powf(0.55);
                    let kk = 0.012;
                    let hh = (0.5 + 0.5 * (tongue - l) / kk).clamp(0.0, 1.0);
                    l = l + (tongue - l) * hh + kk * hh * (1.0 - hh);
                }
            }
            let wav =
                (fbm(s ^ (0x40 + k as u32), x * 40.0, 0.5, 3) - 0.5) * 0.03 + (vnoise(s ^ 0x41, x * 160.0, k as f64 * 3.0) - 0.5) * 0.004;
            (l + wav * (l / 0.03).min(1.0)).max(0.0)
        };
        let cols: Vec<Vec<(f64, f64)>> = (0..rims.len())
            .map(|k| {
                (0..tw)
                    .map(|i| {
                        let x = (i as f64 + 0.5) * sx;
                        let (l0, l1, l) = (reach_at(x - sx, k), reach_at(x + sx, k), reach_at(x, k));
                        (l, (1.0 + ((l1 - l0) / (2.0 * sx)).powi(2)).sqrt())
                    })
                    .collect()
            })
            .collect();
        decal_tex(tw, th, pw, ph, STAIN, move |x, y, aa| {
            let mut px = [STAIN[0], STAIN[1], STAIN[2], 0.0];
            let i = ((x / sx) as usize).min(tw - 1);
            // the damp round the hole behind the collar, seen past its rim
            let hole = (x - pw / 2.0).hypot(y - (from - 0.0575)) / 0.06;
            lay(&mut px, WET, 0.22 * smooth(1.25, 0.92, hole) * smooth(0.01, 0.03, y) * (0.6 + 0.4 * fbm(s ^ 0x1, x * 80.0, y * 80.0, 2)));
            let dy = y - from;
            // the wetted patch hanging from the collar's foot, and its rims:
            // sharp outside, what the water carried left along them
            let fine = (vnoise(s ^ 0x51, x * 260.0, y * 260.0) - 0.5) * 0.0015;
            let (l, slope) = cols[0][i];
            let top = (-dy - 0.006).max(0.0);
            let d0 = (dy - l) / slope + fine + top * 4.0 * smooth(0.012, 0.02, (x - mid).abs());
            let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
            let pale = 0.55 + 0.45 * (d0 / 0.012).exp();
            lay(&mut px, STAIN, inside * 0.45 * pale * (0.65 + 0.35 * fbm(s ^ 0x52, x * 50.0, y * 50.0, 2)));
            let tide = |d: f64| (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / 0.0018).exp();
            let along = |k: u32| 0.5 + 0.5 * fbm(s ^ (0x60 + k), x * 30.0, y * 12.0, 2);
            let mut lines = tide(d0) * along(0);
            for k in 1..rims.len() {
                let (lk, sk) = cols[k][i];
                lines = 1.0 - (1.0 - lines) * (1.0 - tide((dy - lk) / sk + fine) * 0.7 * along(k as u32) * inside);
            }
            lay(&mut px, TIDE, lines * 0.7);
            // still wet down its middle under the leak, rust at its edges
            let tt = (dy / reach).clamp(0.0, 1.0);
            let rw = 0.0045 * (1.0 - 0.5 * tt) * (0.6 + 0.8 * fbm(s ^ 0x6, 1.5, y * 20.0, 2));
            let xr = mid + (fbm(s ^ 0x7, 0.5, y * 11.0, 3) - 0.5) * 0.012 * tt;
            let dr = ((x - xr).abs() - rw).max(dy - l * 0.9).max(-dy - 0.004);
            lay(&mut px, WET, (0.5 - dr / aa).clamp(0.0, 1.0) * (0.4 - 0.2 * tt));
            lay(&mut px, RUST, 0.4 * smooth(-0.002, 0.0, dr) * smooth(0.0012, 0.0, dr));
            // nothing at the plane's edges
            px[3] *= smooth(0.0, 0.006, x.min(pw - x).min(y).min(ph - y));
            px
        })
    }
}
