//! `place_ironing_boards`
use game::prelude::*;
use placers_kit::geometry::orient_seg;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Ironing board upright w/ iron (floor): 3% chance per level.
pub(crate) fn place_ironing_boards(g: &mut Game, _dist: &[i32]) {
    g.pp.hot_iron_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Board cover pool — pastels + neutrals.
    let cover_colors = [0xf0d0d0, 0xd0e0f0, 0xf0e8d0, 0xa0c0a0, 0xc0c0c8];
    let cover_mat = lambert!({ color: pick(&cover_colors) });
    let leg_mat = lambert!({ color: 0x505050 });
    let iron_colors = [0x101010, 0xa02020, 0x2060a0, 0xc0c0c8];
    let iron_mat = lambert!({ color: pick(&iron_colors) });
    let cord_mat = lambert!({ color: 0x0a0a0a });
    let ib = group();
    // Board — 2 boxes + rounded ends via half cylinders.
    let board_h = 0.85;
    let body_main = mesh(pg("IroningBoards__board", box_geo!(0.32, 0.020, 0.16)), &cover_mat);
    body_main.position().set(0, board_h, 0);
    ib.add(&body_main);
    // Pointed nose end (extends beyond).
    let nose = mesh(pg("IroningBoards__nose", cyl_geo!(0.075, 0.075, 0.020, 8, 1, false, 0, PI)), &cover_mat);
    // Unrotated, the theta 0..PI half-disc already lies flat on the +X side.
    nose.position().set(0.16, board_h, 0);
    ib.add(&nose);
    // Rounded broad end.
    let back_end = mesh(pg("IroningBoards__backEnd", cyl_geo!(0.08, 0.08, 0.020, 8, 1, false, PI, PI)), &cover_mat);
    // Unrotated, the theta PI..2PI half-disc lies flat on the -X side.
    back_end.position().set(-0.16, board_h, 0);
    ib.add(&back_end);
    // 4 folding metal legs — 2 pairs forming X shape (seen from the side).
    for i in 0..2 {
        let mut side = -1;
        while side <= 1 {
            let leg = mesh(pg("IroningBoards__leg", cyl_geo!(0.006, 0.006, board_h * 1.02, 5)), &leg_mat);
            let off = if i == 0 { -0.06 } else { 0.06 };
            leg.position().set(side as f64 * 0.03, 0.42, off);
            leg.rotation().z = side as f64 * 0.22f64.atan2(0.84);
            ib.add(&leg);
            side += 2;
        }
    }
    // Pivot rod through the X crossing, linking the two rails.
    let brace = mesh(pg("IroningBoards__brace", cyl_geo!(0.004, 0.004, 0.30, 5)), &leg_mat);
    brace.rotation().x = PI / 2.0;
    brace.scale().y = 0.44;
    brace.position().set(0, 0.5345, 0);
    ib.add(&brace);
    // Rubber feet on floor (4 small dark caps).
    for ox in [-0.14, 0.14] {
        for oz in [-0.06, 0.06] {
            let foot = mesh(pg("IroningBoards__foot", cyl_geo!(0.008, 0.010, 0.006, 5)), &cord_mat);
            foot.position().set(ox, 0.003, oz);
            ib.add(&foot);
        }
    }
    // Iron resting on the broad end.
    let iron = group();
    // Iron body — flat base + upright body.
    let iron_base = mesh(pg("IroningBoards__ironBase", box_geo!(0.09, 0.010, 0.06)), &iron_mat);
    iron_base.position().y = 0.005;
    iron.add(&iron_base);
    // Nose taper (pointed tip), lying flat: axis along X, flat cut face down.
    let iron_nose = mesh(pg("IroningBoards__ironNose", cyl_geo!(0.032, 0.006, 0.030, 6, 1, false, 0, PI)), &iron_mat);
    iron_nose.rotation().z = PI / 2.0;
    iron_nose.position().set(0.040, 0, 0);
    iron.add(&iron_nose);
    // Upper body — smaller upper block.
    let iron_upper = mesh(pg("IroningBoards__ironUpper", box_geo!(0.07, 0.020, 0.05)), &iron_mat);
    iron_upper.position().set(-0.01, 0.020, 0);
    iron.add(&iron_upper);
    // Handle — curved arch (partial torus), feet on the housing top.
    let handle_arch = mesh(pg("IroningBoards__ironHandle", torus_geo!(0.024, 0.006, 4, 8, PI)), &iron_mat);
    handle_arch.position().set(-0.010, 0.030, 0);
    iron.add(&handle_arch);
    // Temperature dial (small cylinder on side).
    let dial_mat = lambert!({ color: 0xd0d0d0 });
    let dial = mesh(pg("IroningBoards__dial", cyl_geo!(0.008, 0.008, 0.003, 8)), &dial_mat);
    dial.rotation().x = PI / 2.0;
    dial.position().set(-0.030, 0.020, 0.028);
    iron.add(&dial);
    // Cord attachment point (small nub at back).
    let nub = mesh(pg("IroningBoards__cordBoot", cyl_geo!(0.005, 0.005, 0.010, 5)), &cord_mat);
    nub.rotation().z = PI / 2.0; // points out of the back (-X)
    nub.position().set(-0.045, 0.030, 0);
    iron.add(&nub);
    iron.position().set(-0.10, board_h + 0.010, 0);
    iron.rotation().y = PI * 0.25;
    ib.add(&iron);
    // Subtle steam-vent wisp — 40% chance (iron is "hot").
    // Track hot irons for steam-hiss audio pairing.
    let iron_is_hot = rnd() < 0.4;
    if iron_is_hot {
        let steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.15 });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
            base_x: f64,
            base_z: f64,
        }
        let mut wisps: Vec<Wisp> = vec![];
        // 2 small wisps rising from iron top.
        for i in 0..2 {
            let wisp = mesh(sphere_geo!(0.010, 6, 5), &steam_mat);
            wisp.scale().y = 2.0;
            let iry = iron.rot().y;
            let wisp_x = -0.10 + iry.cos() * (-0.02 + rnd() * 0.04);
            let wisp_z = iry.sin() * (-0.02 + rnd() * 0.04);
            wisp.position().set(wisp_x, board_h + 0.070 + i as f64 * 0.025, wisp_z);
            let p = wisp.pos();
            wisps.push(Wisp { mesh: wisp.clone(), phase: rnd() * PI * 2.0, base_y: p.y, base_x: p.x, base_z: p.z });
            ib.add(&wisp);
        }
        g.animate(move |_g, _dt, t| {
            for w in &wisps {
                {
                    let mut p = w.mesh.position();
                    p.y = w.base_y + (t * 0.5 + w.phase).sin() * 0.020 + 0.010;
                    p.x = w.base_x + (t * 0.4 + w.phase).sin() * 0.010;
                    p.z = w.base_z + (t * 0.6 + w.phase).cos() * 0.010;
                }
                let s = 0.85 + (t * 0.7 + w.phase).sin() * 0.18;
                w.mesh.scale().set(s, 2.0 * s, s);
            }
        });
    }
    // Cord snake — 8-segment curve from iron nub trailing to floor.
    // Starts at the cord boot's back end (iron-local (-0.05, 0.03, 0), yawed PI/4).
    let cord_start = v3(-0.135, board_h + 0.040, 0.035);
    let cord_end = v3(-0.35, 0.005, 0.15);
    let cord_segs = 8;
    let mut prev = cord_start;
    for i in 0..cord_segs {
        let t = (i + 1) as f64 / cord_segs as f64;
        let arc_amt = (t * PI).sin() * 0.03;
        let cx = cord_start.x + (cord_end.x - cord_start.x) * t + (t * PI * 2.0).sin() * arc_amt;
        let cy = cord_start.y + (cord_end.y - cord_start.y) * t + (t * PI * 1.5).sin().abs() * 0.01;
        let cz = cord_start.z + (cord_end.z - cord_start.z) * t + (t * PI * 2.0).cos() * arc_amt * 0.7;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("IroningBoards__cord", cyl_geo!(0.003, 0.003, dir_len, 4)), &cord_mat);
            orient_seg(&seg, &prev, &next);
            ib.add(&seg);
        }
        prev = next;
    }
    // 50% chance: folded garment draped over one end.
    if rnd() < 0.5 {
        let shirt_colors = [0xffffff, 0xd0e0f0, 0xf0d0d0, 0xf0e0a0, 0x606080];
        let shirt_mat = lambert!({ color: pick(&shirt_colors) });
        let shirt = mesh(pg("IroningBoards__shirt", box_geo!(0.14, 0.010, 0.10)), &shirt_mat);
        shirt.position().set(0.05, board_h + 0.015, 0);
        shirt.rotation().z = 0.05;
        ib.add(&shirt);
    }
    ib.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    ib.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ib);
    // Register position only for hot irons (with steam wisps).
    if iron_is_hot {
        let p = ib.pos();
        g.pp.hot_iron_positions.push(v3(p.x, 0, p.z));
    }
}
