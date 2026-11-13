//! Prop placers: building-services fixtures (towel bars, elevator panels,
//! extinguisher cabinets, card readers, mail slots, outlets, drain grates,
//! doors, motion detectors, ceiling pipes, fuse boxes, light bulbs, intercoms,
//! sprinkler heads, emergency phones and strobes, pipe leaks, peeled paint) and
//! small litter (clipboards, books, tape, receipts, sticky notes, safety cones
//! and glasses, socks, flashlights, lanyards, pens, bottle caps, paper clips,
//! air fresheners, phones, toilet paper).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod r11b;
use r11b::{place_peeled_paint_patches, place_wall_pipe_leaks};
mod r12;
use r12::{place_elevator_panels, place_emergency_phones, place_extinguisher_cabinets, place_fuse_boxes};
mod r14b;
use r14b::place_safety_glasses;
mod r15;
use r15::{place_crumpled_receipts, place_paperback_books, place_tape_rolls};
// Towel bar, emergency strobe: batch `t04`'s (`t04.rs`)
mod t04;
use t04::{place_emergency_strobes, place_hand_towel_bars};
// the paper clips: batch `t22`'s (`t22.rs`)
mod t22;
use t22::place_paperclip_clusters;

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

use placers_kit::place::open_tiles;

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

// Dropped clipboard: 8% chance per level. Wooden clipboard
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

// Swipe card reader: 4% chance per level. Dark box w/
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

// Wall mail slot: 4% chance per level. Brass plate w/
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

// Wall outlet: 7% chance per level. Cream plate w/ 2
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

// Wall drain grate: 5% chance per level. Rectangular grate
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

// Post-it stack: 8% chance per level. 3-5 stacked
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

// Safety cone: 6% chance per level. Orange cone + white
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

// Wall door with peephole: 4% chance per level. Wooden
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

// Single sock: 10% chance per level: a crew sock lost and lying limp on
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

// Ceiling motion detector: 5% chance per level. Small
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

// Broken flashlight: 8% chance per level. Metal cylinder
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

// Ceiling pipe run: 5% chance per level. Horizontal pipe
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

// Office lanyard w/ badge: 8% chance per level. Colored
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

// Broken pen cluster: 8% chance per level. 4-8 pens
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

// Bottle caps: 10% chance per level: 8-14 crown caps prised off bottles
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

// Hanging bare bulb: 5% chance per level. Bulb + socket +
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

// Wall intercom: 4% chance per level. Small speaker box +
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

// Ceiling sprinkler head: 6% chance per level. Small
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

// Air freshener can: 10% chance per level. Small aerosol
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

// Dropped cell phone: 10% chance per level. Small
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

// Unrolled TP: 8% chance per level. Cardboard tube +
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

// ------------------------------------------------------------ batch r12
// The parts of Blender batch `r12` (assets/src/blender/props/r12.py) are
// placed as modelled, each prop's parts sharing one frame: x across its front,
// y up, z out of the wall, the wall at z = 0. Sizes there and in r12.notes.md.

// ---- the emergency phone

// ---- the lift's panel

// ---- the extinguisher cabinet

// ---- the breaker panel

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
