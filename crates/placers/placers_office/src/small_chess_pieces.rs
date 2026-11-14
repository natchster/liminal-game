//! `place_small_chess_pieces`
use game::prelude::*;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_xz;

// Small chess piece: 8% chance per level. Random pawn /
// rook / bishop / knight silhouette. Games family.
pub(crate) fn place_small_chess_pieces(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let is_white = rnd() < 0.5;
    let piece_mat = lambert!({ color: if is_white { 0xe8ded0 } else { 0x201a10 } });
    let piece = group();
    // Common base for all pieces.
    let base = mesh(pg("SmallChessPieces__base", cyl_geo!(0.020, 0.024, 0.014, 12)), &piece_mat);
    base.position().y = 0.007;
    piece.add(&base);
    let kind = (rnd() * 4.0).floor() as i32;
    if kind == 0 {
        // Pawn: bulb + tapered stem.
        let stem = mesh(pg("SmallChessPieces__pawnstem", cyl_geo!(0.010, 0.015, 0.026, 8)), &piece_mat);
        stem.position().y = 0.027;
        piece.add(&stem);
        let bulb = mesh(pg("SmallChessPieces__pawnhead", sphere_geo!(0.014, 10, 6)), &piece_mat);
        bulb.position().y = 0.050;
        piece.add(&bulb);
    } else if kind == 1 {
        // Rook: tall cylinder with 4 crenellations.
        let body = mesh(pg("SmallChessPieces__rook", cyl_geo!(0.016, 0.020, 0.040, 8)), &piece_mat);
        body.position().y = 0.034;
        piece.add(&body);
        for i in 0..4 {
            let a = (i as f64 / 4.0) * PI * 2.0;
            let crenel = mesh(pg("SmallChessPieces__merlon", box_geo!(0.008, 0.008, 0.008)), &piece_mat);
            crenel.position().set(a.cos() * 0.012, 0.058, a.sin() * 0.012);
            piece.add(&crenel);
        }
    } else if kind == 2 {
        // Bishop: tall cone + ball tip.
        let body = mesh(pg("SmallChessPieces__bishop", cone_geo!(0.016, 0.055, 12)), &piece_mat);
        body.position().y = 0.044;
        piece.add(&body);
        let ball = mesh(pg("SmallChessPieces__finial", sphere_geo!(0.008, 8, 6)), &piece_mat);
        ball.position().y = 0.078;
        piece.add(&ball);
    } else {
        // Knight: block silhouette (simplified horse head).
        let neck = mesh(pg("SmallChessPieces__neck", box_geo!(0.014, 0.040, 0.020)), &piece_mat);
        neck.position().y = 0.034;
        piece.add(&neck);
        let head = mesh(pg("SmallChessPieces__head", box_geo!(0.026, 0.014, 0.020)), &piece_mat);
        head.position().set(0.008, 0.058, 0);
        piece.add(&head);
        let snout = mesh(pg("SmallChessPieces__snout", box_geo!(0.014, 0.010, 0.016)), &piece_mat);
        snout.position().set(0.020, 0.056, 0);
        piece.add(&snout);
        let ear = mesh(pg("SmallChessPieces__ear", cone_geo!(0.004, 0.010, 4)), &piece_mat);
        ear.position().set(-0.004, 0.070, 0);
        piece.add(&ear);
    }
    // 30% chance tipped on side.
    if rnd() < 0.30 {
        piece.rotation().z = PI / 2.0;
        piece.position().y = 0.024; // base radius 0.024 is now the height: its rim on the floor
    }
    scatter_xz(g, &piece, x, z, 2.4);
    piece.rotation().y = rnd() * PI * 2.0;
    g.world.add(&piece);
}
