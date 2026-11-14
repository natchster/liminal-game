//! `place_bulletin_boards`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Bulletin boards: 15% chance per level, 1 corkboard with pinned notes.
pub(crate) fn place_bulletin_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let board = group();
    // Corkboard base — brown-tan.
    let board_mat = lambert!({ color: 0x6a4a2e });
    let base = mesh(pg("BulletinBoards__cork", plane_geo!(1.0, 0.6)), &board_mat);
    board.add(&base);
    // Wood frame — 4 thin bars around edges.
    let frame_mat = lambert!({ color: 0x3a2818 });
    let _frame_thick = 0.03;
    for (ox, oy, sx, sy) in [
        (0.0, 0.31, 1.04, 0.03),
        (0.0, -0.31, 1.04, 0.03), // top + bottom
        (-0.51, 0.0, 0.03, 0.6),
        (0.51, 0.0, 0.03, 0.6), // left + right
    ] {
        let fr = mesh(pg("BulletinBoards__framebar", plane_geo!(sx, sy)), &frame_mat);
        fr.position().set(ox, oy, 0.001);
        board.add(&fr);
    }
    // Mini notes: 3-6 small rectangles pinned across the board.
    let note_count = 3 + (rnd() * 4.0).floor() as i32;
    let note_colors = [0xf0e8c8, 0xe0d0a0, 0xd8c0a4, 0xf8f0e0, 0xc8c8b0];
    for _n in 0..note_count {
        let nw = 0.13 + rnd() * 0.08;
        let nh = 0.10 + rnd() * 0.06;
        let note_mat = lambert!({ color: pick(&note_colors) });
        let note = mesh(pg("BulletinBoards__note", plane_geo!(nw, nh)), &note_mat);
        // Position within the board area (leave 0.1 margin from edge).
        let nx = (rnd() - 0.5) * 0.72;
        let ny = (rnd() - 0.5) * 0.32;
        note.position().set(nx, ny, 0.002);
        let base_note_rot_z = (rnd() - 0.5) * 0.25; // slight pin-tilt
        note.rotation().z = base_note_rot_z;
        board.add(&note);
        // Tiny pin (dark dot at top-center).
        let pin = mesh(plane_geo!(0.008, 0.008), basic_mat!({ color: 0x2a1a10 }));
        let np = note.pos();
        pin.position().set(np.x, np.y + nh * 0.42, 0.003);
        board.add(&pin);
        // 10% chance this note flutters — subtle sway animator.
        if rnd() < 0.10 {
            let flutter_phase = rnd() * PI * 2.0;
            let note2 = note.clone();
            g.animate(move |_g, _dt, t| {
                note2.rotation().z = base_note_rot_z + (t * 0.7 * PI + flutter_phase).dsin() * 0.03;
            });
        }
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
    let board_height = 1.4 + rnd() * 0.3;
    if c.dx.abs() == 1 {
        board.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        board.position().set(wall_x, board_height, wall_z + (rnd() - 0.5) * 0.8);
    } else {
        board.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        board.position().set(wall_x + (rnd() - 0.5) * 0.8, board_height, wall_z);
    }
    g.world.add(&board);
}
