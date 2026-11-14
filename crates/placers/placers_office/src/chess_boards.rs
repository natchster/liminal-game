//! `place_chess_boards`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Chess board mid-game (floor): 4% chance per level.
pub(crate) fn place_chess_boards(g: &mut Game, _dist: &[i32]) {
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
                cren.position().set(ang.dcos() * cell_size * 0.15, piece_h * 0.90, ang.dsin() * cell_size * 0.15);
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
                spike.position().set(ang.dcos() * cell_size * 0.14, piece_h * 1.10, ang.dsin() * cell_size * 0.14);
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
    for _i in 0..piece_count {
        let (mut r, mut c);
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
        piece.rotation().y = if kind == "knight" { if is_white { 0.0 } else { PI } } else { 0.0 };
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
