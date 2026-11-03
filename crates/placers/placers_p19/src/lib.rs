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

// R560 lift button panel: 4% chance per level. A car operating panel's
// faceplate (Blender batch `r12`): brushed stainless, twelve floor buttons in
// their bezels with the raised numeral and braille beside each, door open and
// close, the alarm, the fire service key switch; the lit buttons' rings glow.
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
    let panel = group();
    // the parts are modelled with the wall at z = 0: the plate's back on it
    let o = group();
    o.position().z = -0.008;
    panel.add(&o);
    let brushed = phong!({ map: &lift_plate_tex(), color: 0xb0b0ac, specular: 0x3c3c3a, shininess: 26 });
    o.add(&mesh(r12_part("Real__elevatorPanelPlate", 0.165, 0.36, 0.003), &brushed));
    let print = mesh(
        plane_geo!(0.165, 0.36),
        lambert!({ map: &lift_print_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().z = EP_T + 0.0002;
    o.add(&print);
    let polished = phong!({ color: 0x8e8e8a, specular: 0x66666a, shininess: 70 });
    o.add(&mesh(r12_part("Real__elevatorPanelBezels", 0.15, 0.32, 0.003), &polished));
    o.add(&mesh(r12_part("Real__elevatorPanelScrews", 0.15, 0.34, 0.002), &polished));
    o.add(&mesh(r12_part("Real__elevatorPanelKey", 0.016, 0.016, 0.004), phong!({ color: 0x9a8a62, specular: 0x4a4436, shininess: 50 })));
    o.add(&mesh(r12_part("Real__elevatorPanelRings", 0.13, 0.2, 0.001), lambert!({ color: 0x2e2d2b })));
    o.add(&mesh(r12_part("Real__elevatorPanelButtons", 0.13, 0.2, 0.003), phong!({ color: 0xa4a4a0, specular: 0x404040, shininess: 40 })));
    let numerals = mesh(
        plane_geo!(0.130, 0.210),
        lambert!({ map: &lift_numerals_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    numerals.position().set(0.009, -0.0125, EP_T + 0.0029);
    o.add(&numerals);
    o.add(&mesh(r12_part("Real__elevatorPanelAlarm", 0.027, 0.027, 0.004), lambert!({ color: 0x8c2318 })));
    let bell = mesh(
        plane_geo!(0.016, 0.016),
        lambert!({ map: &lift_bell_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    bell.position().set(EP_ALARM.0, EP_ALARM.1, EP_T + 0.0037);
    o.add(&bell);
    // 4×3 = 12 floor buttons, top row first; R561 20% chance a button's ring is lit.
    let halo_mat = basic_mat!({ color: 0xffc274 });
    for by in 0..4 {
        for bx in 0..3 {
            if rnd() < 0.20 {
                let halo = mesh(r12_part("Real__elevatorPanelHalo", 0.022, 0.022, 0.001), &halo_mat);
                halo.position().set(EP_COL[bx], EP_ROW[by], EP_T + 0.0001);
                o.add(&halo);
            }
        }
    }
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

// R559 wall extinguisher cabinet: 4% chance per level. A surface-mounted
// steel cabinet for a 5 lb extinguisher (Blender batch `r12`): white enamel
// gone yellow and chipped, a full pane in its door frame, FIRE EXTINGUISHER
// down the glass; in it the extinguisher stands on its foot, its hose in its
// band, the pin sealed, its tag hanging.
fn place_extinguisher_cabinets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let cab = group();
    // the parts are modelled with the wall at z = 0: the tub's back on it
    let o = group();
    o.position().z = -0.04;
    cab.add(&o);
    let enamel = lambert!({ map: &ec_enamel_tex(), color: 0xd4cfc2 });
    o.add(&mesh(r12_part("Real__extCabinetTub", 0.205, 0.456, 0.134), &enamel));
    o.add(&mesh(r12_part("Real__extCabinetDoor", 0.205, 0.456, 0.022), &enamel));
    o.add(&mesh(r12_part("Real__extCabinetHardware", 0.21, 0.45, 0.04), phong!({ color: 0x75746f, specular: 0x3a3a38, shininess: 36 })));
    // the extinguisher, standing on the tub's floor, turned so its label
    // and gauge face out past the door's edge
    let ex = group();
    ex.position().set(-0.014, -EC_H / 2.0 + 0.0014, 0.073);
    ex.rotation().y = PI - 0.4;
    o.add(&ex);
    ex.add(&mesh(r12_part("Real__extCabinetExBody", 0.108, 0.24, 0.108), lambert!({ map: &ex_label_tex(), color: 0xffffff })));
    ex.add(&mesh(r12_part("Real__extCabinetExEnds", 0.108, 0.3, 0.108), lambert!({ color: 0x7e1c14 })));
    ex.add(&mesh(r12_part("Real__extCabinetExValve", 0.14, 0.07, 0.05), phong!({ color: 0x8a8984, specular: 0x505050, shininess: 44 })));
    ex.add(&mesh(r12_part("Real__extCabinetExSeal", 0.02, 0.02, 0.015), lambert!({ color: 0xb49a2e })));
    ex.add(&mesh(r12_part("Real__extCabinetExHose", 0.04, 0.2, 0.03), lambert!({ color: 0x1f1e1c })));
    let gauge = mesh(circle_geo!(0.0098, 20), lambert!({ map: &ex_gauge_tex() }));
    gauge.rotation().y = PI;
    gauge.position().set(0, EX_BODY + 0.034, -0.0252);
    ex.add(&gauge);
    let tag = mesh(plane_geo!(0.042, 0.078), lambert!({ map: &ex_tag_tex(), side: DoubleSide }));
    tag.rotation().set(0.08, PI + 0.25, 0.04);
    tag.position().set(0.030, EX_BODY - 0.040, -0.060);
    ex.add(&tag);
    // the pane, clear, and what is on it: the lettering, dust and old tape
    o.add(&mesh(
        r12_part("Real__extCabinetGlass", 0.169, 0.42, 0.003),
        lambert!({ color: 0xcdd9d4, transparent: true, opacity: 0.16, depthWrite: false }),
    ));
    let print = mesh(
        plane_geo!(0.169, 0.420),
        lambert!({ map: &ec_glass_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().z = EC_D - 0.0025 + 0.0002;
    o.add(&print);
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

// R552 peeled paint patch: 6% chance per level. Wall decal
// w/ curled paint edge revealing darker base beneath.
game::lazy_static_tex!(
    peeled_paint_texture,
    canvas_tex(256, 320, |ctx, w, h| {
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
    })
);

fn place_peeled_paint_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let mat = lambert!({ map: peeled_paint_texture(), transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
    let decal = mesh(plane_geo!(0.50, 0.65), &mat);
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&decal, c, wall_x, wall_z, 1.4, Some(0.5));
    g.world.add(&decal);
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

// R549 wall fuse box: 4% chance per level. A 6-space breaker panel on the
// surface (Blender batch `r12`): the grey enclosure, its front cover and dead
// front, the door hanging open, six single-pole breakers on or off, the
// circuits written on the card inside the door.
fn place_fuse_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let fuse_box = group();
    // the parts are modelled with the wall at z = 0: the enclosure's back on it
    let o = group();
    o.position().z = -0.03;
    fuse_box.add(&o);
    let paint = lambert!({ map: &fb_enamel_tex(), color: 0x8e908b });
    o.add(&mesh(r12_part("Real__fuseBoxTub", 0.215, 0.32, 0.083), &paint));
    o.add(&mesh(r12_part("Real__fuseBoxCover", 0.229, 0.334, 0.012), &paint));
    o.add(&mesh(r12_part("Real__fuseBoxScrews", 0.22, 0.32, 0.002), phong!({ color: 0x77766f, specular: 0x333333, shininess: 30 })));
    let bakelite = lambert!({ color: 0x232220 });
    o.add(&mesh(r12_part("Real__fuseBoxBreakers", 0.13, 0.08, 0.02), &bakelite));
    let print = mesh(
        plane_geo!(FB_DOOR.0, FB_DOOR.1),
        lambert!({ map: &fb_print_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().set(FB_DOOR_C.0, FB_DOOR_C.1, FB_D + 0.0014);
    o.add(&print);
    // the door, hanging open on its hinge at the opening's left edge
    let hinge = group();
    hinge.position().set(FB_DOOR_C.0 - FB_DOOR.0 / 2.0, FB_DOOR_C.1, FB_D + 0.0035);
    hinge.rotation().y = -1.85;
    o.add(&hinge);
    hinge.add(&mesh(r12_part("Real__fuseBoxDoor", 0.15, 0.2, 0.006), &paint));
    let card = mesh(plane_geo!(0.112, 0.150), lambert!({ map: &fb_directory_tex(), side: DoubleSide }));
    card.rotation().y = PI;
    card.position().set(FB_DOOR.0 * 0.5, 0.0, -0.0014);
    hinge.add(&card);
    // six breakers, 1-3-5 down the left and 2-4-6 down the right, each on or off
    for i in 0..6 {
        let (col, row) = (i % 2, i / 2);
        let sw = mesh(r12_part("Real__fuseBoxHandle", 0.011, 0.018, 0.013), &bakelite);
        sw.position().set(FB_COL[col], FB_ROW[row], FB_D + 0.001);
        // Random tilt for on/off.
        sw.rotation().y = if rnd() < 0.5 { 0.3 } else { -0.3 };
        o.add(&sw);
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

// R546 emergency phone: 4% chance per level. A ring-down phone (Blender batch
// `r12`): the 2554-class wall set in red with a blank plate where the keys
// would be, EMERGENCY on it, its handset on the hook and the cord looping
// under it; the engraved sign above.
fn place_emergency_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let phone = group();
    // the parts are modelled with the wall at z = 0: the housing's back on it
    let o = group();
    o.position().z = -0.09;
    phone.add(&o);
    let red = lambert!({ map: &em_plastic_tex(), color: 0x9a2a1e });
    o.add(&mesh(r12_part("Real__wallPhoneHousing", 0.123, 0.232, 0.058), &red));
    o.add(&mesh(r12_part("Real__emergencyPhonePlate", 0.056, 0.088, 0.004), lambert!({ map: &em_plate_tex() })));
    o.add(&mesh(r12_part("Real__wallPhoneHook", 0.032, 0.03, 0.02), phong!({ color: 0x575752, specular: 0x3a3a3a, shininess: 22 })));
    let handset = mesh(r12_part("Real__wallPhoneHandset", 0.06, 0.21, 0.04), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e }));
    handset.position().set(-0.026, 0.001, 0.058 - 0.004);
    o.add(&handset);
    o.add(&mesh(r12_part("Real__wallPhoneCordOn", 0.07, 0.2, 0.05), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e })));
    // the sign screwed to the wall over it
    let sign = mesh(r12_part("Real__emergencyPhoneSign", 0.2, 0.06, 0.003), lambert!({ map: &em_sign_tex() }));
    sign.position().y = 0.200;
    o.add(&sign);
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

// R545 wall pipe leak: 5% chance per level. Horizontal pipe
// fixture + drip line + water stain patch.
fn place_wall_pipe_leaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
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
    stain.position().set(0.05, -0.19, -0.024); // wet patch on the wall below the drip (was a flat disc in mid-air)
    leak.add(&stain);
    // pipe axis 2.6 cm off the wall (r 2.4 cm): was sunk 1 cm into it
    let (wall_x, wall_z) = wall_xz(g, c, 0.026);
    mount(&leak, c, wall_x, wall_z, 1.90, Some(0.3));
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

// ------------------------------------------------------------ batch r12
// The parts of Blender batch `r12` (assets/src/blender/props/r12.py) are
// placed as modelled, each prop's parts sharing one frame: x across its front,
// y up, z out of the wall, the wall at z = 0. Sizes there and in r12.notes.md.

/// A Blender part of batch `r12` as modelled (a scale of one), or a box
/// standing in for it when the props are switched off.
fn r12_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// The lift panel's plate thickness, its columns and rows of floor buttons
/// (the top row first) and its alarm button (`r12.py`, EP_*).
const EP_T: f64 = 0.003;
const EP_COL: [f64; 3] = [-0.041, 0.009, 0.059];
const EP_ROW: [f64; 4] = [0.080, 0.035, -0.010, -0.055];
const EP_ALARM: (f64, f64) = (0.003, -0.150);
/// The extinguisher cabinet's height and depth, the extinguisher's cylinder (`r12.py`, EC_*, EX_*).
const EC_H: f64 = 0.456;
const EC_D: f64 = 0.156;
const EX_BODY: f64 = 0.300;
/// The breaker panel: the enclosure's depth, the door (w, h) and its middle,
/// the breakers' columns and rows (`r12.py`, FB_*).
const FB_D: f64 = 0.083;
const FB_DOOR: (f64, f64) = (0.150, 0.198);
const FB_DOOR_C: (f64, f64) = (0.0, -0.032);
const FB_COL: [f64; 2] = [-0.036, 0.036];
const FB_ROW: [f64; 3] = [0.000, -0.026, -0.052];

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! r12_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

/// A soft round blob (squashed by the context, never the ellipse: no rim).
fn r12_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, rgba: (u8, u8, u8, f64)) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({},{},{},{})", rgba.0, rgba.1, rgba.2, rgba.3));
    gr.add_color_stop(1, &format!("rgba({},{},{},0)", rgba.0, rgba.1, rgba.2));
    ctx.set_fill_style(gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Short straight scratches.
fn r12_scratches(ctx: &mut Canvas, w: f64, h: f64, n: usize, rgb: (u8, u8, u8), a0: f64, len: f64) {
    ctx.set_line_cap("round");
    for _ in 0..n {
        let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, len * (0.2 + rnd()));
        ctx.set_line_width(0.5 + rnd() * 0.7);
        ctx.set_stroke_style(&format!("rgba({},{},{},{})", rgb.0, rgb.1, rgb.2, a0 + rnd() * a0));
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
}

// ---- the emergency phone

/// Red ABS gone dull: the housing's wear (the colour is the material's), hands
/// round the handset's seats, dust on top, grime at the foot.
fn em_plastic_tex() -> Tex {
    r12_tex!(128, 240, false, |ctx, w, h| {
        ctx.set_fill_style("#f2eeea");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(&format!("rgba(70,60,56,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        for &(x, y, r, a) in &[(0.29, 0.16, 0.30, 0.26), (0.30, 0.83, 0.26, 0.2), (0.6, 0.45, 0.3, 0.1)] {
            r12_blob(ctx, x * w, y * h, r * w, 1.3, (50, 36, 30, a));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,125,0.35)");
        gd.add_color_stop(0.06, "rgba(150,140,125,0)");
        gd.add_color_stop(0.9, "rgba(40,30,26,0)");
        gd.add_color_stop(1, "rgba(40,30,26,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 24, (200, 170, 160), 0.15, 14.0);
    })
}

/// The handset's and its cord's wear: grime at the grip.
fn em_handset_tex() -> Tex {
    r12_tex!(64, 160, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece8");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(50,36,30,0.12)");
        gr.add_color_stop(0.5, "rgba(50,36,30,0.32)");
        gr.add_color_stop(1, "rgba(50,36,30,0.14)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 10, (200, 170, 160), 0.15, 8.0);
    })
}

/// The ring-down set's plate (the whole canvas is its face): a printed card,
/// EMERGENCY over what to do, yellowed, a thumb's grime at its foot.
fn em_plate_tex() -> Tex {
    r12_tex!(112, 176, false, |ctx, w, h| {
        ctx.set_fill_style("#e4ddcc");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#8e2016");
        ctx.fill_rect(0, 0, w, 44);
        ctx.set_fill_style("#eee8da");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 17px Helvetica");
        ctx.fill_text_max("EMERGENCY", w / 2.0, 23, w - 10.0);
        ctx.set_fill_style("#2a2622");
        ctx.set_font("bold 12px Helvetica");
        for (i, l) in ["LIFT", "RECEIVER", "TO CALL"].iter().enumerate() {
            ctx.fill_text(l, w / 2.0, 66.0 + i as f64 * 15.0);
        }
        ctx.set_font("9px Helvetica");
        for (i, l) in ["STAY ON", "THE LINE", "UNTIL", "ANSWERED"].iter().enumerate() {
            ctx.fill_text(l, w / 2.0, 120.0 + i as f64 * 11.0);
        }
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(180,150,80,0.12)");
        gy.add_color_stop(1, "rgba(80,60,40,0.28)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 10, (60, 50, 40), 0.12, 10.0);
    })
}

/// The engraved sign: red laminate cut through to its white core, the
/// letters' edges dirty, its face scratched and dusty.
fn em_sign_tex() -> Tex {
    r12_tex!(400, 120, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2218");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..600 {
            ctx.set_fill_style(&format!("rgba(40,10,6,{})", 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        ctx.set_fill_style("#e9e4d8");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica");
        ctx.fill_text_max("EMERGENCY", w / 2.0, 42, w - 40.0);
        ctx.set_font("bold 30px Helvetica");
        ctx.fill_text_max("TELEPHONE", w / 2.0, 88, w - 60.0);
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,120,0.30)");
        gd.add_color_stop(0.25, "rgba(150,140,120,0.05)");
        gd.add_color_stop(1, "rgba(30,20,16,0.15)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 30, (230, 200, 190), 0.12, 30.0);
    })
}

// ---- the lift's panel

/// Brushed stainless: the grain across, fingers' smudges round the buttons,
/// the edges grimy (the whole canvas is the plate's face, 0.165 x 0.36 m).
fn lift_plate_tex() -> Tex {
    r12_tex!(200, 436, false, |ctx, w, h| {
        ctx.set_fill_style("#e6e6e2");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..900 {
            let y = rnd() * h;
            let x = rnd() * w;
            ctx.set_fill_style(&format!("rgba({},{},{},{})", if rnd() < 0.5 { 255 } else { 90 }, 255, 250, 0.05 + rnd() * 0.08));
            ctx.fill_rect(x, y, 6.0 + rnd() * 40.0, 0.6);
        }
        // smudges where hands go: over the buttons, heaviest on the lower floors
        let to = |x: f64, y: f64| ((x / 0.165 + 0.5) * w, (0.5 - y / 0.36) * h);
        for &(bx, by, a) in &[
            (-0.041, -0.055, 0.22),
            (0.009, -0.055, 0.25),
            (0.059, -0.055, 0.15),
            (0.009, 0.035, 0.12),
            (-0.022, -0.105, 0.2),
            (0.028, -0.105, 0.24),
            (0.003, -0.150, 0.1),
        ] {
            let (x, y) = to(bx, by);
            r12_blob(ctx, x, y, 18.0, 1.2, (70, 62, 52, a));
        }
        for _ in 0..14 {
            r12_blob(ctx, rnd() * w, h * (0.25 + rnd() * 0.6), 4.0 + rnd() * 6.0, 1.4, (80, 70, 60, 0.10 + rnd() * 0.08));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(120,110,95,0.25)");
        gd.add_color_stop(0.05, "rgba(120,110,95,0)");
        gd.add_color_stop(0.95, "rgba(50,44,36,0)");
        gd.add_color_stop(1, "rgba(50,44,36,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 30, (60, 58, 54), 0.12, 18.0);
    })
}

/// Braille: the dots of a digit (cells of the numeric indicator and the letter's).
fn braille_dots(c: char) -> &'static [u8] {
    match c {
        '#' => &[3, 4, 5, 6],
        '1' => &[1],
        '2' => &[1, 2],
        '3' => &[1, 4],
        '4' => &[1, 4, 5],
        '5' => &[1, 5],
        '6' => &[1, 2, 4],
        '7' => &[1, 2, 4, 5],
        '8' => &[1, 2, 5],
        '9' => &[2, 4],
        '0' => &[2, 4, 5],
        'B' => &[1, 2],
        'S' => &[2, 3, 4],
        'T' => &[2, 3, 4, 5],
        'A' => &[1],
        'L' => &[1, 2, 3],
        'M' => &[1, 3, 4],
        _ => &[],
    }
}

/// One Braille cell at (x, y), its dot pitch `p` px: raised dots, lit from
/// above (a light crescent over a shadow).
fn braille_cell(ctx: &mut Canvas, x: f64, y: f64, p: f64, dots: &[u8]) {
    for &d in dots {
        let (col, row) = (if d > 3 { 1.0 } else { 0.0 }, ((d - 1) % 3) as f64);
        let (cx, cy) = (x + col * p, y + row * p);
        ctx.set_fill_style("rgba(20,18,16,0.55)");
        ctx.begin_path();
        ctx.arc(cx + 0.4, cy + 0.5, p * 0.36, 0, 7);
        ctx.fill();
        ctx.set_fill_style("rgba(255,255,250,0.75)");
        ctx.begin_path();
        ctx.arc(cx - 0.2, cy - 0.3, p * 0.28, 0, 7);
        ctx.fill();
    }
}

/// What is on the plate beside the buttons, 2 px a mm: each floor's plate
/// (its numeral raised and its Braille under it) to the left of its button,
/// the star at the main floor, FIRE OPERATION round the key switch, OPEN and
/// CLOSE, ALARM, the capacity engraved at the foot.
fn lift_print_tex() -> Tex {
    r12_tex!(330, 720, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x / 0.165 + 0.5) * w, (0.5 - y / 0.36) * h);
        let names = [["9", "10", "11"], ["6", "7", "8"], ["3", "4", "5"], ["B", "1", "2"]];
        let cols = [-0.041, 0.009, 0.059];
        let rows = [0.080, 0.035, -0.010, -0.055];
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for (r, row) in names.iter().enumerate() {
            for (c, name) in row.iter().enumerate() {
                let (x, y) = to(cols[c] - 0.022, rows[r]);
                // the plate: black anodised, 13 x 24 mm
                ctx.set_fill_style("rgba(28,27,26,0.92)");
                ctx.fill_rect(x - 13.0, y - 24.0, 26, 48);
                ctx.set_fill_style("rgba(225,222,214,0.9)");
                ctx.set_font(if name.len() > 1 { "bold 15px Helvetica" } else { "bold 18px Helvetica" });
                ctx.fill_text(name, x, y - 9.0);
                // its Braille: the numeric indicator before a digit
                let mut cx = x - 9.0;
                if name.chars().all(|ch| ch.is_ascii_digit()) {
                    braille_cell(ctx, cx, y + 7.0, 4.6, braille_dots('#'));
                    cx += 10.0;
                }
                for ch in name.chars() {
                    braille_cell(ctx, cx, y + 7.0, 4.6, braille_dots(ch));
                    cx += 10.0;
                }
                if *name == "1" {
                    // the main entry floor's star
                    ctx.set_fill_style("rgba(225,222,214,0.9)");
                    ctx.set_font("bold 12px Helvetica");
                    ctx.fill_text("\u{2605}", x, y - 26.0 + 2.0);
                }
            }
        }
        ctx.set_fill_style("rgba(30,29,28,0.85)");
        ctx.set_font("bold 11px Helvetica");
        let (kx, ky) = to(0.040, 0.142);
        ctx.fill_text("FIRE OPERATION", to(-0.020, 0.150).0, to(0.0, 0.150).1);
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text("OFF", kx - 30.0, ky);
        ctx.fill_text("HOLD", kx, ky - 30.0);
        ctx.fill_text("ON", kx + 30.0, ky);
        // the fire hat pictogram under its words
        let (hx, hy) = to(-0.020, 0.128);
        ctx.begin_path();
        ctx.move_to(hx - 16.0, hy + 6.0);
        ctx.quadratic_curve_to(hx - 14.0, hy - 12.0, hx, hy - 12.0);
        ctx.quadratic_curve_to(hx + 14.0, hy - 12.0, hx + 16.0, hy + 6.0);
        ctx.close_path();
        ctx.fill();
        ctx.fill_rect(hx - 22.0, hy + 4.0, 44, 4);
        ctx.set_font("bold 9px Helvetica");
        for (i, l) in ["OPEN", "CLOSE"].iter().enumerate() {
            let (x, y) = to(if i == 0 { -0.022 } else { 0.028 }, -0.105);
            ctx.fill_text(l, x, y + 34.0);
        }
        let (ax, ay) = to(-0.032, -0.150);
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("ALARM", ax - 14.0, ay);
        ctx.set_font("bold 8px Helvetica");
        let (cx, cy) = to(0.0, -0.172);
        ctx.fill_text("CAPACITY 2500 LBS  16 PERSONS", cx, cy);
    })
}

/// The numerals engraved in the buttons and filled black, 2 px a mm over the
/// buttons' field (0.130 x 0.210 m, its middle at x 0.009, y -0.0125), and
/// the door buttons' arrows; rubbed paler where pressed most.
fn lift_numerals_tex() -> Tex {
    r12_tex!(260, 420, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x - 0.009 + 0.065) * 2000.0, (0.105 - (y + 0.0125)) * 2000.0);
        let names = [["9", "10", "11"], ["6", "7", "8"], ["3", "4", "5"], ["B", "1", "2"]];
        let cols = [-0.041, 0.009, 0.059];
        let rows = [0.080, 0.035, -0.010, -0.055];
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for (r, row) in names.iter().enumerate() {
            for (c, name) in row.iter().enumerate() {
                let (x, y) = to(cols[c], rows[r]);
                ctx.set_fill_style(&format!("rgba(24,22,20,{})", 0.9 - rnd() * 0.3));
                ctx.set_font(if name.len() > 1 { "bold 17px Helvetica" } else { "bold 21px Helvetica" });
                ctx.fill_text(name, x, y + 1.0);
            }
        }
        // the door buttons: open (outward arrows) and close (inward)
        ctx.set_fill_style("rgba(24,22,20,0.85)");
        for (i, &bx) in [-0.022, 0.028].iter().enumerate() {
            let (x, y) = to(bx, -0.105);
            ctx.fill_rect(x - 1.0, y - 8.0, 2, 16);
            for s in [-1.0, 1.0] {
                let dir = if i == 0 { s } else { -s };
                let tip = x + s * if i == 0 { 13.0 } else { 3.0 };
                ctx.begin_path();
                ctx.move_to(tip, y);
                ctx.line_to(tip - dir * 8.0, y - 6.0);
                ctx.line_to(tip - dir * 8.0, y + 6.0);
                ctx.close_path();
                ctx.fill();
            }
        }
    })
}

/// The alarm button's bell, white on its red.
fn lift_bell_tex() -> Tex {
    r12_tex!(64, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(236,230,220,0.92)");
        ctx.begin_path();
        ctx.move_to(14, 44);
        ctx.quadratic_curve_to(20, 40, 20, 28);
        ctx.quadratic_curve_to(20, 12, 32, 12);
        ctx.quadratic_curve_to(44, 12, 44, 28);
        ctx.quadratic_curve_to(44, 40, 50, 44);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.arc(32, 49, 5, 0, 7);
        ctx.fill();
        ctx.fill_rect(30, 7, 4, 6);
    })
}

// ---- the extinguisher cabinet

/// White baked enamel on steel, gone cream: chipped to the metal at the
/// corners and the door's edge, rust in the chips at the foot, grime where
/// the door is pulled, dust on top (the canvas is the cabinet's face).
fn ec_enamel_tex() -> Tex {
    r12_tex!(128, 284, false, |ctx, w, h| {
        ctx.set_fill_style("#f3efe4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(&format!("rgba(120,105,80,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(190,160,100,0.30)");
        gy.add_color_stop(0.5, "rgba(190,160,100,0.12)");
        gy.add_color_stop(1, "rgba(80,60,40,0.30)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        // the pull's grime (right edge, middle)
        r12_blob(ctx, w * 0.92, h * 0.5, w * 0.22, 2.0, (60, 50, 40, 0.3));
        // chips: grey metal, rust in the low ones
        for k in 0..22 {
            let edge = k % 4;
            let t = rnd();
            let (x, y) = match edge {
                0 => (t * w, 2.0 + rnd() * 4.0),
                1 => (t * w, h - 2.0 - rnd() * 5.0),
                2 => (2.0 + rnd() * 4.0, t * h),
                _ => (w - 2.0 - rnd() * 4.0, t * h),
            };
            let low = y > h * 0.7;
            let r = 1.5 + rnd() * 3.5;
            ctx.set_fill_style(if low { "rgba(120,62,30,0.85)" } else { "rgba(118,116,110,0.85)" });
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.5 + rnd() * 0.5), rnd() * 3.0, 0, 7);
            ctx.fill();
            if low {
                ctx.set_fill_style("rgba(130,70,30,0.25)");
                ctx.fill_rect(x - 1.0, y, 2.0, 6.0 + rnd() * 14.0);
            }
        }
        r12_scratches(ctx, w, h, 20, (90, 80, 70), 0.12, 16.0);
    })
}

/// The pane's print (the canvas is the pane, 0.169 x 0.420 m): FIRE
/// EXTINGUISHER in red vinyl down the glass, reading up, a letter or two
/// peeled; dust thick at the top and the corners, a hand's smear, old tape.
fn ec_glass_tex() -> Tex {
    r12_tex!(236, 588, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // the letters, turned to read up the pane
        ctx.save();
        ctx.translate(w * 0.5, h * 0.5);
        ctx.rotate(-PI / 2.0);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica");
        ctx.set_fill_style("rgba(150,30,22,0.95)");
        ctx.fill_text_max("FIRE EXTINGUISHER", 0, 0, h - 60.0);
        ctx.restore();
        // a peeled letter's ghost: some vinyl gone, its glue left
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..30 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.4 + rnd() * 0.6));
            ctx.begin_path();
            ctx.ellipse(w * (0.35 + rnd() * 0.3), h * (0.1 + rnd() * 0.8), 1.0 + rnd() * 4.0, 1.0 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_composite_operation("source-over");
        // dust: a film, thicker up top and in the corners
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(170,160,140,0.40)");
        gd.add_color_stop(0.3, "rgba(170,160,140,0.12)");
        gd.add_color_stop(1, "rgba(150,140,120,0.22)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        for &(x, y) in &[(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
            r12_blob(ctx, x, y, 60.0, 1.0, (160, 150, 130, 0.35));
        }
        // a hand's smear at the pull side, and a strip of old tape's glue
        r12_blob(ctx, w * 0.85, h * 0.5, 40.0, 1.6, (90, 80, 70, 0.18));
        ctx.set_fill_style("rgba(190,170,120,0.35)");
        ctx.fill_rect(w * 0.08, h * 0.06, 48, 14);
        r12_scratches(ctx, w, h, 16, (220, 220, 215), 0.15, 30.0);
    })
}

/// The extinguisher's wall (the canvas wraps it, u round from its back,
/// its front at the middle; v up its 0.238 m): red enamel, the label at the
/// front, scuffed where it stood among things, chipped low.
fn ex_label_tex() -> Tex {
    r12_tex!(512, 360, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2016");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..800 {
            ctx.set_fill_style(&format!("rgba({},20,12,{})", if rnd() < 0.5 { 40 } else { 180 }, 0.04 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        // the label: 0.10 m round, 0.15 m tall, its middle at u = 0.5
        let (lw, lh) = (152.0, 228.0);
        let (lx, ly) = (w / 2.0 - lw / 2.0, h * 0.5 - lh / 2.0 - 6.0);
        ctx.set_fill_style("#e2dccb");
        ctx.fill_rect(lx, ly, lw, lh);
        ctx.set_fill_style("#8a2016");
        ctx.fill_rect(lx, ly, lw, 30);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("#efe9dc");
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text_max("FIRE EXTINGUISHER", w / 2.0, ly + 15.0, lw - 8.0);
        ctx.set_fill_style("#24201c");
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text_max("MULTIPURPOSE DRY CHEMICAL", w / 2.0, ly + 41.0, lw - 8.0);
        // the classes: A in a triangle, B in a square, C in a circle
        let cy = ly + 66.0;
        for (i, (letter, col)) in [("A", "#3c6a3c"), ("B", "#8a2a20"), ("C", "#2e4a78")].iter().enumerate() {
            let cx = w / 2.0 - 40.0 + i as f64 * 40.0;
            ctx.set_fill_style(*col);
            ctx.begin_path();
            match i {
                0 => {
                    ctx.move_to(cx, cy - 14.0);
                    ctx.line_to(cx + 14.0, cy + 11.0);
                    ctx.line_to(cx - 14.0, cy + 11.0);
                    ctx.close_path();
                }
                1 => ctx.rect(cx - 12.0, cy - 12.0, 24, 24),
                _ => ctx.arc(cx, cy, 12.5, 0, 7),
            }
            ctx.fill();
            ctx.set_fill_style("#efe9dc");
            ctx.set_font("bold 13px Helvetica");
            ctx.fill_text(letter, cx, cy + if i == 0 { 3.0 } else { 1.0 });
        }
        ctx.set_fill_style("#24201c");
        ctx.set_font("bold 8px Helvetica");
        ctx.fill_text("RATING 2-A:10-B:C", w / 2.0, ly + 92.0);
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text("TO OPERATE", w / 2.0, ly + 108.0);
        ctx.set_text_align("left");
        ctx.set_font("8px Helvetica");
        for (i, l) in ["1  PULL PIN", "2  AIM AT BASE OF FIRE", "3  SQUEEZE LEVER", "4  SWEEP SIDE TO SIDE"].iter().enumerate() {
            ctx.fill_text(l, lx + 14.0, ly + 122.0 + i as f64 * 12.0);
        }
        ctx.set_text_align("center");
        ctx.set_font("7px Helvetica");
        for (i, l) in ["RECHARGE IMMEDIATELY AFTER ANY USE", "INSPECT MONTHLY", "CONTENTS UNDER PRESSURE"].iter().enumerate() {
            ctx.fill_text_max(l, w / 2.0, ly + 178.0 + i as f64 * 10.0, lw - 10.0);
        }
        ctx.fill_rect(lx + 18.0, ly + 210.0, lw - 36.0, 1);
        // the label yellowed and grimy, its edges lifting dark
        let gy = ctx.create_linear_gradient(0, ly, 0, ly + lh);
        gy.add_color_stop(0, "rgba(170,140,70,0.15)");
        gy.add_color_stop(1, "rgba(90,60,30,0.3)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(lx, ly, lw, lh);
        ctx.set_stroke_style("rgba(60,40,30,0.4)");
        ctx.set_line_width(1.5);
        ctx.stroke_rect(lx + 0.5, ly + 0.5, lw - 1.0, lh - 1.0);
        // scuffs, chips low down, dust settled on what faces up
        r12_scratches(ctx, w, h, 40, (200, 190, 180), 0.12, 20.0);
        for _ in 0..14 {
            ctx.set_fill_style("rgba(70,66,62,0.8)");
            ctx.begin_path();
            ctx.ellipse(rnd() * w, h - 4.0 - rnd() * 40.0, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,120,0.25)");
        gd.add_color_stop(0.15, "rgba(150,140,120,0)");
        gd.add_color_stop(0.85, "rgba(40,30,25,0)");
        gd.add_color_stop(1, "rgba(40,30,25,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// The gauge's dial: the green band of a charged cylinder between red, the
/// needle fallen into RECHARGE, its glass dusty.
fn ex_gauge_tex() -> Tex {
    r12_tex!(64, 64, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_fill_style("#d8d4c8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_line_width(7);
        for (a0, a1, col) in [(PI * 0.85, PI * 1.25, "#8a2a20"), (PI * 1.25, PI * 1.75, "#3c6a3c"), (PI * 1.75, PI * 2.15, "#8a2a20")] {
            ctx.set_stroke_style(col);
            ctx.begin_path();
            ctx.arc(cx, cy, 20, a0, a1);
            ctx.stroke();
        }
        ctx.set_fill_style("#222");
        ctx.set_font("bold 6px Helvetica");
        ctx.set_text_align("center");
        ctx.fill_text("PSI", cx, cy + 14.0);
        ctx.set_stroke_style("#1a1a1a");
        ctx.set_line_width(2);
        let a = PI * 0.95;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + a.cos() * 22.0, cy + a.sin() * 22.0);
        ctx.stroke();
        r12_blob(ctx, cx, cy, 34.0, 1.0, (160, 150, 130, 0.35));
    })
}

/// The inspection tag on the neck: a yellow card, its year's months
/// punched out until they stopped, initials in pen.
fn ex_tag_tex() -> Tex {
    r12_tex!(84, 156, false, |ctx, w, h| {
        ctx.set_fill_style("#c9b25a");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#2a241a");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 8px Helvetica");
        ctx.fill_text("INSPECTION", w / 2.0, 22);
        ctx.fill_text("RECORD", w / 2.0, 31);
        ctx.set_font("6px Helvetica");
        ctx.fill_text("DO NOT REMOVE", w / 2.0, 41);
        let months = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
        ctx.set_line_width(0.6);
        ctx.set_stroke_style("#2a241a");
        for (i, m) in months.iter().enumerate() {
            let (col, row) = ((i % 4) as f64, (i / 4) as f64);
            let (x, y) = (14.0 + col * 18.0, 58.0 + row * 18.0);
            ctx.stroke_rect(x - 7.0, y - 7.0, 14, 14);
            ctx.set_fill_style("#2a241a");
            ctx.fill_text(m, x, y);
            if i < 7 {
                ctx.set_fill_style("#3a3630");
                ctx.begin_path();
                ctx.arc(x, y, 4.0, 0, 7);
                ctx.fill();
            }
        }
        ctx.set_fill_style("rgba(30,30,70,0.85)");
        ctx.set_font("13px Marker Felt");
        ctx.fill_text("R.K.", w / 2.0, 126);
        // the hole and its brass eyelet
        ctx.set_fill_style("#5a4a2a");
        ctx.begin_path();
        ctx.arc(w / 2.0, 8, 4.5, 0, 7);
        ctx.fill();
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(90,70,40,0.1)");
        gd.add_color_stop(1, "rgba(60,40,20,0.35)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

// ---- the breaker panel

/// Grey baked enamel (ANSI 61) on steel: rust blooming from the screws and
/// the knockouts, grime where the door is pulled and below it, dust on top.
fn fb_enamel_tex() -> Tex {
    r12_tex!(160, 232, false, |ctx, w, h| {
        ctx.set_fill_style("#efefea");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..600 {
            ctx.set_fill_style(&format!("rgba(80,80,76,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        for &(x, y) in &[(0.05, 0.04), (0.95, 0.04), (0.05, 0.96), (0.95, 0.96)] {
            r12_blob(ctx, x * w, y * h, 9.0, 1.0, (120, 60, 25, 0.55));
            ctx.set_fill_style("rgba(120,60,25,0.25)");
            ctx.fill_rect(x * w - 1.0, y * h, 2.0, 10.0 + rnd() * 20.0);
        }
        r12_blob(ctx, w * 0.8, h * 0.6, 30.0, 1.4, (50, 46, 40, 0.3));
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(140,130,110,0.35)");
        gd.add_color_stop(0.08, "rgba(140,130,110,0)");
        gd.add_color_stop(0.85, "rgba(40,36,30,0)");
        gd.add_color_stop(1, "rgba(40,36,30,0.28)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 26, (60, 58, 52), 0.12, 18.0);
        for _ in 0..10 {
            ctx.set_fill_style("rgba(110,70,40,0.7)");
            ctx.begin_path();
            ctx.ellipse(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 0.8 + rnd() * 1.5, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
    })
}

/// What is printed on the dead front (the canvas spans the door's opening,
/// 0.150 x 0.198 m): the circuit numbers by the breakers, the rating label.
fn fb_print_tex() -> Tex {
    r12_tex!(300, 396, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x + 0.075) * 2000.0, (0.099 - (y + 0.032)) * 2000.0);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("rgba(28,28,26,0.9)");
        ctx.set_font("bold 13px Helvetica");
        for (i, &y) in [0.000, -0.026, -0.052].iter().enumerate() {
            let (lx, ly) = to(-0.071, y);
            ctx.fill_text(&format!("{}", i * 2 + 1), lx, ly);
            let (rx, ry) = to(0.071, y);
            ctx.fill_text(&format!("{}", i * 2 + 2), rx, ry);
        }
        // the rating label above the breakers: paper, printed, curling
        let (x0, y0) = to(-0.050, 0.060);
        ctx.set_fill_style("rgba(226,222,208,0.95)");
        ctx.fill_rect(x0, y0, 200, 46);
        ctx.set_fill_style("rgba(30,30,28,0.9)");
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("LOAD CENTER", x0 + 100.0, y0 + 11.0);
        ctx.set_font("8px Helvetica");
        ctx.fill_text("120/240 VAC  1 PH  3 W  100 A MAX", x0 + 100.0, y0 + 24.0);
        ctx.fill_text("6 SPACES  USE COPPER WIRE ONLY", x0 + 100.0, y0 + 35.0);
        ctx.set_fill_style("rgba(90,70,40,0.25)");
        ctx.fill_rect(x0, y0 + 36.0, 200, 10);
        // DANGER at its foot
        let (dx, dy) = to(-0.040, -0.075);
        ctx.set_fill_style("rgba(150,40,26,0.9)");
        ctx.fill_rect(dx, dy, 160, 18);
        ctx.set_fill_style("rgba(235,230,220,0.95)");
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("DANGER  HIGH VOLTAGE", dx + 80.0, dy + 9.5);
    })
}

/// The circuit directory on the inside of the door: the printed card, what
/// each circuit fed written in, one crossed out, the card foxed.
fn fb_directory_tex() -> Tex {
    r12_tex!(224, 300, false, |ctx, w, h| {
        ctx.set_fill_style("#ddd6c2");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#26241f");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 14px Helvetica");
        ctx.fill_text("CIRCUIT DIRECTORY", w / 2.0, 18);
        ctx.set_stroke_style("rgba(40,38,34,0.8)");
        ctx.set_line_width(1);
        let entries = ["HALL LIGHTS", "RECEPT. NORTH", "COPIER", "COFFEE / SINK", "EXIT SIGNS", "SPARE"];
        for (i, e) in entries.iter().enumerate() {
            let y = 44.0 + i as f64 * 40.0;
            ctx.begin_path();
            ctx.move_to(10, y + 30.0);
            ctx.line_to(w - 10.0, y + 30.0);
            ctx.stroke();
            ctx.set_fill_style("#26241f");
            ctx.set_font("bold 12px Helvetica");
            ctx.set_text_align("left");
            ctx.fill_text(&format!("{}", i + 1), 12, y + 18.0);
            ctx.set_fill_style("rgba(30,32,80,0.85)");
            ctx.set_font("17px Marker Felt");
            ctx.fill_text(e, 34, y + 18.0);
        }
        // the copier struck out, something else written over it
        ctx.set_stroke_style("rgba(30,32,80,0.8)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(32, 44.0 + 2.0 * 40.0 + 18.0);
        ctx.line_to(104, 44.0 + 2.0 * 40.0 + 16.0);
        ctx.stroke();
        ctx.set_font("13px Marker Felt");
        ctx.set_fill_style("rgba(30,32,80,0.8)");
        ctx.fill_text("FRIDGE", 120, 44.0 + 2.0 * 40.0 + 18.0);
        for _ in 0..30 {
            r12_blob(ctx, rnd() * w, rnd() * h, 3.0 + rnd() * 8.0, 1.0, (140, 100, 50, 0.08 + rnd() * 0.12));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(160,130,70,0.15)");
        gd.add_color_stop(1, "rgba(90,70,40,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
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
