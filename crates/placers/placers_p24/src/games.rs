//! Playground / games / dressing-table family.
use crate::{pick_color, pick_open_tile, scatter_pos, scatter_xz};
use game::prelude::*;

// the spinning top: batch `t15`'s (`games/t15.rs`)
mod t15;
pub use t15::place_small_spinning_tops;

// R451 small skateboard: 8% chance per level. Deck + 4 wheels
// + 2 trucks. Playground family.
pub fn place_small_skateboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let deck_colors = [0xd82040, 0x2050d8, 0x30a030, 0xa020a0, 0xd8b040, 0x201a10];
    let deck_mat = lambert!({ color: pick_color(&deck_colors) });
    let truck_mat = lambert!({ color: 0x606870 });
    let wheel_mat = lambert!({ color: 0xf0ece0 });
    let skate = group();
    let deck_len = 0.32;
    let deck_w = 0.08;
    // Deck (main).
    let deck = mesh(pg("SmallSkateboards__deck", box_geo!(deck_len, 0.008, deck_w)), &deck_mat);
    deck.position().y = 0.036;
    skate.add(&deck);
    // 2 trucks under deck.
    for tx in [-0.10, 0.10] {
        let truck = mesh(pg("SmallSkateboards__truck", box_geo!(0.012, 0.010, deck_w - 0.010)), &truck_mat);
        truck.position().set(tx, 0.026, 0);
        skate.add(&truck);
        // 2 wheels at each truck.
        for wz2 in [-(deck_w / 2.0 - 0.006), deck_w / 2.0 - 0.006] {
            let wheel = mesh(pg("SmallSkateboards__wheel", cyl_geo!(0.014, 0.014, 0.010, 10)), &wheel_mat);
            wheel.rotation().x = PI / 2.0;
            wheel.position().set(tx, 0.014, wz2);
            skate.add(&wheel);
        }
    }
    scatter_pos(g, &skate, x, z, 2.0);
    skate.rotation().y = rnd() * PI * 2.0;
    // 15% chance flipped upside down.
    if rnd() < 0.15 {
        skate.rotation().z = PI;
        skate.position().y = 0.048;
    }
    g.world.add(&skate);
}

// R450 small hula hoop: 6% chance per level. Large colorful
// torus lying on floor. Playground family.
pub fn place_small_hula_hoops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let hoop_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xd050a0, 0xe07020];
    let hoop_color = pick_color(&hoop_colors);
    let hoop_mat = lambert!({ color: hoop_color });
    let stripe_colors = [0xf0ece0, 0x181008, 0xd8b040];
    let hoop = group();
    // Main torus (large: 30 cm r, 1.2 cm tube).
    let body = mesh(pg("SmallHulaHoops__hoop", torus_geo!(0.30, 0.012, 6, 24)), &hoop_mat);
    body.rotation().x = PI / 2.0;
    body.position().y = 0.012;
    hoop.add(&body);
    // Optional stripe overlay: 4 small torus segments in accent color.
    if rnd() < 0.60 {
        let stripe_mat = lambert!({ color: pick_color(&stripe_colors) });
        for i in 0..4 {
            let stripe = mesh(pg("SmallHulaHoops__stripe", torus_geo!(0.30, 0.0125, 4, 4, PI / 12.0)), &stripe_mat);
            stripe.rotation().x = PI / 2.0;
            stripe.rotation().z = (i as f64 / 4.0) * PI * 2.0;
            stripe.position().y = 0.012;
            hoop.add(&stripe);
        }
    }
    scatter_pos(g, &hoop, x, z, 1.5);
    hoop.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hoop);
}

// R450 small chalk sticks: 12% chance per level. 2-4 colored
// chalk sticks scattered on floor. Playground family.
pub fn place_small_chalk_sticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let chalk_colors = [0xf0ece0, 0xd0d0d0, 0xd08080, 0x80d080, 0x8080d0, 0xd0d080, 0xd080d0];
    let grp = group();
    let n_sticks = 2 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_sticks {
        let chalk_mat = lambert!({ color: pick_color(&chalk_colors) });
        let chalk = mesh(pg("SmallChalkSticks__stick", cyl_geo!(0.006, 0.006, 0.055, 6)), &chalk_mat);
        chalk.rotation().z = PI / 2.0;
        let cx = (rnd() - 0.5) * 0.12;
        let cz = (rnd() - 0.5) * 0.10;
        chalk.position().set(cx, 0.006, cz);
        chalk.rotation().y = rnd() * PI * 2.0;
        grp.add(&chalk);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R449 hopscotch chalk decal: 4% chance per level. Canvas
// texture with numbered chalk squares on floor. Games family.
game::lazy_static_tex!(hopscotch_texture, {
    let t = canvas_tex_opt(256, 512, false, |ctx, w, h| {
        ctx.set_fill_style("rgba(0,0,0,0)");
        ctx.clear_rect(0, 0, w, h);
        let chalk_color = "#d0d0d0";
        ctx.set_stroke_style(chalk_color);
        ctx.set_fill_style(chalk_color);
        ctx.set_line_width(4);
        // 9 squares in hopscotch layout (1, 2-3, 4, 5-6, 7, 8-9).
        let cell_w = 60.0;
        let cx = w / 2.0;
        let layout: [(i32, f64); 6] = [
            (1, 40.0),
            (2, 40.0 + cell_w + 8.0),
            (1, 40.0 + 2.0 * (cell_w + 8.0)),
            (2, 40.0 + 3.0 * (cell_w + 8.0)),
            (1, 40.0 + 4.0 * (cell_w + 8.0)),
            (2, 40.0 + 5.0 * (cell_w + 8.0)),
        ];
        ctx.set_font("bold 32px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let mut num = 1;
        for &(count, ry) in layout.iter() {
            for i in 0..count {
                let rx = cx + (i as f64 - (count - 1) as f64 / 2.0) * (cell_w + 4.0) - cell_w / 2.0;
                // Rough chalk box.
                ctx.begin_path();
                ctx.set_stroke_style(&format!("rgba(220,220,220,{})", 0.5 + rnd() * 0.4));
                ctx.rect(rx, ry, cell_w, cell_w);
                ctx.stroke();
                // Number.
                ctx.set_fill_style(&format!("rgba(220,220,220,{})", 0.5 + rnd() * 0.4));
                ctx.fill_text(&num.to_string(), rx + cell_w / 2.0, ry + cell_w / 2.0);
                num += 1;
            }
        }
    });
    // TODO: this texture is meant to premultiply alpha on upload; it is
    // currently uploaded straight-alpha.
    t
});

pub fn place_hopscotch_chalks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let decal_mat = basic_mat!({ map: hopscotch_texture(), transparent: true, opacity: 0.75, depthWrite: false });
    let decal = mesh(plane_geo!(1.2, 2.4), &decal_mat);
    decal.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    decal.position().set(px, 0.003, pz);
    decal.rotation().z = rnd() * PI * 2.0;
    g.world.add(&decal);
}

// R448 small dart set: 8% chance per level. 2-3 darts with
// colored fins scattered. Games family.
pub fn place_small_darts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let fin_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0];
    let body_mat = lambert!({ color: 0x606870 });
    let point_mat = lambert!({ color: 0x202028 });
    let shaft_mat = lambert!({ color: 0xf0ece0 });
    let grp = group();
    let n_darts = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_darts {
        let dart = group();
        let fin_mat = lambert!({ color: pick_color(&fin_colors), side: DoubleSide });
        // Point: sharp cone.
        let point = mesh(pg("SmallDarts__point", cone_geo!(0.003, 0.015, 6)), &point_mat);
        point.rotation().z = -PI / 2.0;
        point.position().set(0.008, 0.005, 0);
        dart.add(&point);
        // Barrel: knurled cylinder.
        let barrel = mesh(pg("SmallDarts__barrel", cyl_geo!(0.005, 0.005, 0.040, 8)), &body_mat);
        barrel.rotation().z = PI / 2.0;
        barrel.position().set(-0.020, 0.005, 0);
        dart.add(&barrel);
        // Shaft: thin white cylinder.
        let shaft = mesh(pg("SmallDarts__shaft", cyl_geo!(0.002, 0.002, 0.030, 6)), &shaft_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().set(-0.055, 0.005, 0);
        dart.add(&shaft);
        // Flight fins: 2 crossed planes at tail.
        for f in 0..2 {
            let fin = mesh(plane_geo!(0.020, 0.014), &fin_mat);
            fin.rotation().z = PI / 2.0;
            fin.rotation().x = f as f64 * PI / 2.0;
            fin.position().set(-0.080, 0.005, 0);
            dart.add(&fin);
        }
        let dx = (rnd() - 0.5) * 0.15;
        let dz = (rnd() - 0.5) * 0.10;
        dart.position().set(dx, 0, dz);
        dart.rotation().y = rnd() * PI * 2.0;
        grp.add(&dart);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R448 small pool cue: 6% chance per level. Long tapered wooden
// cue with tip cap. Games family.
pub fn place_small_pool_cues(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let shaft_colors = [0xc8a870, 0x8a6540, 0x603820, 0x201a10];
    let shaft_mat = lambert!({ color: pick_color(&shaft_colors) });
    let tip_mat = lambert!({ color: 0x2050c0 });
    let grip_mat = lambert!({ color: 0x201a10 });
    let cue = group();
    let cue_len = 0.90;
    // Shaft: tapered cylinder (thicker at butt).
    let shaft = mesh(pg("SmallPoolCues__shaft", cyl_geo!(0.006, 0.014, cue_len, 8)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(cue_len / 2.0, 0.010, 0);
    cue.add(&shaft);
    // Tip: small blue chalk pad on far end.
    let tip = mesh(pg("SmallPoolCues__tip", cyl_geo!(0.006, 0.006, 0.008, 6)), &tip_mat);
    tip.rotation().z = PI / 2.0;
    tip.position().set(-0.004, 0.010, 0);
    cue.add(&tip);
    // Grip: dark wrap on butt end.
    let grip = mesh(pg("SmallPoolCues__grip", cyl_geo!(0.015, 0.015, 0.12, 8)), &grip_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(cue_len - 0.06, 0.010, 0);
    cue.add(&grip);
    scatter_pos(g, &cue, x, z, 1.6);
    cue.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cue);
}

/// The pip table of `place_small_dominoes` (half-face layouts, 1-6).
fn domino_pips(count: i32) -> &'static [(f64, f64)] {
    match count {
        1 => &[(0.0, 0.0)],
        2 => &[(-0.005, -0.004), (0.005, 0.004)],
        3 => &[(-0.005, -0.004), (0.0, 0.0), (0.005, 0.004)],
        4 => &[(-0.005, -0.004), (-0.005, 0.004), (0.005, -0.004), (0.005, 0.004)],
        5 => &[(-0.005, -0.004), (-0.005, 0.004), (0.005, -0.004), (0.005, 0.004), (0.0, 0.0)],
        _ => &[(-0.005, -0.005), (-0.005, 0.0), (-0.005, 0.005), (0.005, -0.005), (0.005, 0.0), (0.005, 0.005)],
    }
}

// R447 small dominoes: 10% chance per level. 2-4 dominoes with
// pip dots scattered. Games family.
pub fn place_small_dominoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let bone_mat = lambert!({ color: 0xf0ece0 });
    let pip_mat = lambert!({ color: 0x181008 });
    let divider_mat = lambert!({ color: 0x8a7060 });
    let grp = group();
    let n_dominoes = 2 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_dominoes {
        let dom = group();
        // Body: rectangle 4×2 cm.
        let body = mesh(pg("SmallDominoes__tile", box_geo!(0.040, 0.006, 0.020)), &bone_mat);
        body.position().y = 0.003;
        dom.add(&body);
        // Central divider line.
        let div = mesh(box_geo!(0.001, 0.001, 0.018), &divider_mat);
        div.position().set(0, 0.0065, 0); // resting on the tile top (y 0.006)
        dom.add(&div);
        // 2 half-face pip patterns.
        let left_count = (rnd() * 6.0).floor() as i32 + 1;
        let right_count = (rnd() * 6.0).floor() as i32 + 1;
        for (half, count) in [(-1.0, left_count), (1.0, right_count)] {
            let half_x = half * 0.010;
            for &(px, pz) in domino_pips(count) {
                let pip = mesh(circle_geo!(0.0018, 6), &pip_mat);
                pip.rotation().x = -PI / 2.0;
                pip.position().set(half_x + px, 0.0062, pz); // 0.2 mm over the tile top, not 1 mm
                dom.add(&pip);
            }
        }
        // Scatter placement.
        let dx = (rnd() - 0.5) * 0.10;
        let dz = (rnd() - 0.5) * 0.10;
        dom.position().set(dx, 0, dz);
        dom.rotation().y = rnd() * PI * 2.0;
        grp.add(&dom);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R447 small pool ball: 8% chance per level. Colored sphere
// with white numbered circle stripe. Games family.
pub fn place_small_pool_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let ball_colors = [0xf0e050, 0x2050d8, 0xd82040, 0xa020a0, 0xe07020, 0x30a030, 0x8a2020, 0x201a10, 0xf0ece0];
    let ball_color = pick_color(&ball_colors);
    let ball_mat = lambert!({ color: ball_color });
    let white_mat = lambert!({ color: 0xf0ece0 });
    let num_mat = lambert!({ color: 0x101008 });
    let is_striped = rnd() < 0.40;
    let ball = group();
    let r = 0.028;
    let body = mesh(pg("SmallPoolBalls__ball", sphere_geo!(r, 12, 10)), &ball_mat);
    body.position().y = r;
    ball.add(&body);
    if is_striped {
        // White band across equator for striped ball.
        let stripe = mesh(pg("SmallPoolBalls__stripe", cyl_geo!(r * 1.001, r * 1.001, r * 0.6, 12, 1, true)), &white_mat);
        stripe.position().y = r;
        ball.add(&stripe);
    }
    // White circle patch on top with number spot.
    let patch = mesh(circle_geo!(r * 0.35, 12), &white_mat);
    patch.rotation().x = -PI / 2.0;
    patch.position().y = r * 2.0 + 0.0005;
    ball.add(&patch);
    let num = mesh(circle_geo!(r * 0.10, 8), &num_mat);
    num.rotation().x = -PI / 2.0;
    num.position().y = r * 2.0 + 0.0010;
    ball.add(&num);
    scatter_xz(g, &ball, x, z, 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}

// R446 small chess piece: 8% chance per level. Random pawn /
// rook / bishop / knight silhouette. Games family.
pub fn place_small_chess_pieces(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
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

// R446 small poker chip stack: 8% chance per level. Stack of
// colored casino chips. Games family.
pub fn place_small_poker_chips(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let chip_colors = [0xd82040, 0x2050d8, 0x30a030, 0x181008, 0xf0ece0];
    let stack = group();
    let n_chips = 4 + (rnd() * 6.0).floor() as i32;
    let chip_r = 0.019;
    let chip_t = 0.004;
    for i in 0..n_chips {
        let chip_mat = lambert!({ color: pick_color(&chip_colors) });
        let chip = mesh(pg("SmallPokerChips__chip", cyl_geo!(chip_r, chip_r, chip_t, 12)), &chip_mat);
        let cx = (rnd() - 0.5) * 0.002;
        let cy = chip_t / 2.0 + i as f64 * chip_t;
        let cz = (rnd() - 0.5) * 0.002;
        chip.position().set(cx, cy, cz);
        chip.rotation().y = rnd() * PI * 2.0;
        stack.add(&chip);
    }
    scatter_pos(g, &stack, x, z, 2.4);
    stack.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stack);
}

// R445 small dice pair: 10% chance per level. 2 white cubes
// with random-face pip dots. Games family.
pub fn place_small_dice_pairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let dice_mat = lambert!({ color: 0xf0ece0 });
    let pip_mat = lambert!({ color: 0x181008 });
    let pair = group();
    for i in 0..2 {
        let die = group();
        let size = 0.028;
        let body = mesh(pg("SmallDicePairs__die", box_geo!(size, size, size)), &dice_mat);
        body.position().y = size / 2.0;
        die.add(&body);
        // Top face pips (1-6 count based on random).
        let top_count = 1 + (rnd() * 6.0).floor() as i32;
        let pip_r = 0.003;
        let off = size * 0.28;
        let pip_positions: Vec<(f64, f64)> = match top_count {
            1 => vec![(0.0, 0.0)],
            2 => vec![(-off, -off), (off, off)],
            3 => vec![(-off, -off), (0.0, 0.0), (off, off)],
            4 => vec![(-off, -off), (-off, off), (off, -off), (off, off)],
            5 => vec![(-off, -off), (-off, off), (off, -off), (off, off), (0.0, 0.0)],
            _ => vec![(-off, -off), (-off, 0.0), (-off, off), (off, -off), (off, 0.0), (off, off)],
        };
        for (px, pz) in pip_positions {
            let pip = mesh(circle_geo!(pip_r, 8), &pip_mat);
            pip.rotation().x = -PI / 2.0;
            pip.position().set(px, size + 0.0005, pz);
            die.add(&pip);
        }
        // Position side by side.
        let dx = (if i == 0 { -0.024 } else { 0.024 }) + (rnd() - 0.5) * 0.010;
        die.position().x = dx;
        let dz = (rnd() - 0.5) * 0.024;
        die.position().z = dz;
        die.rotation().y = rnd() * PI * 2.0;
        pair.add(&die);
    }
    scatter_pos(g, &pair, x, z, 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

// R445 small playing card stack: 8% chance per level. A pack of
// poker-size playing cards dropped (Blender batch r15, 63.5 x 88.9 mm, 0.3
// mm a card): three in ten spread in a fan, face down but the one under
// them; else the deck put down unsquared, its top few cards slid, the top
// one turned up half the time. Backs and faces printed, the type a serif;
// grimy at the edges. The old draws are kept call for call.
pub fn place_small_card_stacks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let cards = group();
    let is_fanned = rnd() < 0.30;
    let n_visible = 5 + (rnd() * 4.0).floor() as i32;
    // the loose cards' slips (the old jitter draws, kept as they were)
    let mut slip = vec![];
    for _ in 0..n_visible {
        if !is_fanned {
            let cx = (rnd() - 0.5) * 0.004;
            let cz = (rnd() - 0.5) * 0.004;
            let ry = (rnd() - 0.5) * 0.05;
            slip.push((cx, cz, ry));
        }
    }
    // the old suit draws: the top card turned up, a red or a black one
    let face_up = if !is_fanned && rnd() < 0.50 { Some(rnd() < 0.5) } else { None };
    scatter_pos(g, &cards, x, z, 2.4);
    cards.rotation().y = rnd() * PI * 2.0;
    let salt = r15_cards::salt(&cards);
    let back = lambert!({ map: &r15_cards::back_tex(salt) });
    let card_geo = prop_part_as_modelled("Real__cardSingle");
    let lay = |m: &Mat, x: f64, y: f64, z: f64, ry: f64| {
        if let Some(geo) = &card_geo {
            let c = mesh(geo.clone(), m);
            c.position().set(x, y, z);
            c.rotation().y = ry;
            cards.add(&c);
        }
    };
    if is_fanned {
        // spread in a fan on the floor, the one under them face up
        let face = lambert!({ map: &r15_cards::face_tex(salt % 2 == 0, salt) });
        for i in 0..n_visible {
            let rot = -PI / 6.0 + (i as f64 / (n_visible - 1) as f64) * (PI / 3.0);
            let dx = i as f64 * 0.0115;
            lay(if i == 0 { &face } else { &back }, dx - 0.042, i as f64 * 0.00033, 0.0, rot);
        }
    } else {
        // the deck, and the loose cards slid on it
        if let Some(geo) = prop_part_as_modelled("Real__cardDeck") {
            cards.add(&mesh(geo, &lambert!({ map: &r15_cards::edges_tex(salt) })));
        }
        let top = r15_cards::DECK_H;
        let face = face_up.map(|red| lambert!({ map: &r15_cards::face_tex(red, salt) }));
        for (i, &(cx, cz, ry)) in slip.iter().enumerate() {
            let last = i + 1 == slip.len();
            let m = match (&face, last) {
                (Some(f), true) => f,
                _ => &back,
            };
            lay(m, cx * 1.4, top + i as f64 * 0.00031, cz * 1.4, ry);
        }
    }
    g.world.add(&cards);
}

// R444 small hairbrush: 10% chance per level. Oval brush pad
// with bristle bumps + handle. Dressing-table family.
pub fn place_small_hairbrushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let handle_colors = [0xd050a0, 0x2050d8, 0x201a10, 0xd8b040, 0x8a3820];
    let handle_mat = lambert!({ color: pick_color(&handle_colors) });
    let bristle_mat = lambert!({ color: 0x2a2418 });
    let brush = group();
    // Pad: oval box (scaled cube).
    let pad = mesh(pg("SmallHairbrushes__pad", box_geo!(0.055, 0.012, 0.035)), &handle_mat);
    pad.position().set(0.030, 0.006, 0);
    brush.add(&pad);
    // 5×3 bristle bumps on top of pad.
    for bx in 0..5 {
        for bz in 0..3 {
            let bristle = mesh(pg("SmallHairbrushes__bristle", cyl_geo!(0.002, 0.002, 0.010, 4)), &bristle_mat);
            bristle.position().set(0.030 - 0.020 + bx as f64 * 0.010, 0.017, -0.010 + bz as f64 * 0.010);
            brush.add(&bristle);
        }
    }
    // Handle: tapered cylinder extending sideways.
    let handle = mesh(pg("SmallHairbrushes__handle", cyl_geo!(0.008, 0.012, 0.10, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.046, 0.012, 0); // flared end 1.5 mm into the pad, lying on the floor
    brush.add(&handle);
    scatter_pos(g, &brush, x, z, 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}

/// The playing cards' print (`place_small_card_stacks`), seeded by where
/// the pack lies.
mod r15_cards {
    use game::prelude::*;

    /// The deck under the loose cards is 46 cards, 0.3 mm each
    pub const DECK_H: f64 = 46.0 * 0.0003;

    pub fn salt(o: &Obj) -> u32 {
        let p = o.pos();
        ((p.x * 977.0).floor() as i64 ^ ((p.z * 1181.0).floor() as i64).rotate_left(13)) as u32
    }

    fn rgba(c: (f64, f64, f64), a: f64) -> String {
        format!("rgba({:.0},{:.0},{:.0},{a:.3})", c.0, c.1, c.2)
    }

    fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &rgba(c, a));
        gr.add_color_stop(1, &rgba(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
    }

    /// The card's size on the canvas: 4 px a millimetre
    const W: f64 = 254.0;
    const H: f64 = 356.0;

    /// A card's own wear: grime at the edges where hands held it, the
    /// corners rubbed, the floor's dust, a crease across one corner
    fn worn(ctx: &mut Canvas, r: &mut SplitMix64) {
        for (x0, y0, x1, y1) in [(0.0, 0.0, 14.0, 0.0), (W, 0.0, W - 14.0, 0.0), (0.0, 0.0, 0.0, 14.0), (0.0, H, 0.0, H - 14.0)] {
            let gr = ctx.create_linear_gradient(x0, y0, x1, y1);
            gr.add_color_stop(0, "rgba(96,84,66,0.35)");
            gr.add_color_stop(1, "rgba(96,84,66,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, W, H);
        }
        for _ in 0..14 {
            let at = (r.next() * W, r.next() * H);
            soft(ctx, at, 10.0 + r.next() * 30.0, 0.5 + r.next() * 0.5, r.next() * 3.0, (110.0, 100.0, 84.0), 0.08 + 0.1 * r.next());
        }
        let (cx, cy) = (if r.next() < 0.5 { 0.0 } else { W }, if r.next() < 0.5 { 0.0 } else { H });
        let l = 30.0 + 26.0 * r.next();
        let (sx, sy) = (if cx > 0.0 { -1.0 } else { 1.0 }, if cy > 0.0 { -1.0 } else { 1.0 });
        ctx.set_line_width(1.2);
        ctx.set_stroke_style("rgba(70,60,48,0.35)");
        ctx.begin_path();
        ctx.move_to(cx + sx * l, cy);
        ctx.line_to(cx, cy + sy * l);
        ctx.stroke();
        ctx.set_stroke_style("rgba(250,246,236,0.35)");
        ctx.begin_path();
        ctx.move_to(cx + sx * (l + 1.5), cy);
        ctx.line_to(cx, cy + sy * (l + 1.5));
        ctx.stroke();
        for _ in 0..120 {
            let (x, y) = (r.next() * W, r.next() * H);
            ctx.set_fill_style(rgba((96.0, 88.0, 74.0), 0.15 + 0.25 * r.next()));
            ctx.fill_rect(x, y, 1.0, 1.0);
        }
    }

    /// The back: a white border, a red field printed with a fine lattice
    /// and a rosette in an oval; no maker's mark
    pub fn back_tex(salt: u32) -> Tex {
        canvas_tex_opt(W, H, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0xbac4);
            ctx.set_fill_style("#e4ddcc");
            ctx.fill_rect(0, 0, w, h);
            let red = (136.0, 44.0, 46.0);
            let (bx, by, bw, bh) = (18.0, 18.0, w - 36.0, h - 36.0);
            ctx.save();
            ctx.begin_path();
            ctx.move_to(bx + 10.0, by);
            ctx.arc_to(bx + bw, by, bx + bw, by + bh, 10);
            ctx.arc_to(bx + bw, by + bh, bx, by + bh, 10);
            ctx.arc_to(bx, by + bh, bx, by, 10);
            ctx.arc_to(bx, by, bx + bw, by, 10);
            ctx.close_path();
            ctx.set_fill_style(rgba(red, 1.0));
            ctx.fill();
            ctx.clip();
            // the lattice: two sets of fine diagonals, a dot where they cross
            ctx.set_stroke_style("rgba(226,214,200,0.75)");
            ctx.set_line_width(1.1);
            let step = 9.0;
            let mut k = -h;
            ctx.begin_path();
            while k < w + h {
                ctx.move_to(k, 0);
                ctx.line_to(k + h, h);
                ctx.move_to(k + h, 0);
                ctx.line_to(k, h);
                k += step;
            }
            ctx.stroke();
            // the oval and its rosette
            let (cx, cy) = (w / 2.0, h / 2.0);
            ctx.set_fill_style(rgba(red, 1.0));
            ctx.begin_path();
            ctx.ellipse(cx, cy, 52, 70, 0, 0, 7);
            ctx.fill();
            ctx.set_stroke_style("rgba(226,214,200,0.85)");
            for (rx, ry, lw) in [(50.0, 68.0, 2.0), (44.0, 62.0, 1.0)] {
                ctx.set_line_width(lw);
                ctx.begin_path();
                ctx.ellipse(cx, cy, rx, ry, 0, 0, 7);
                ctx.stroke();
            }
            ctx.set_line_width(1.0);
            ctx.begin_path();
            for k in 0..16 {
                let a = k as f64 / 16.0 * PI * 2.0;
                ctx.move_to(cx + a.cos() * 34.0, cy + a.sin() * 34.0);
                ctx.ellipse(cx + a.cos() * 18.0, cy + a.sin() * 18.0, 16, 5, a, 0, 7);
            }
            ctx.stroke();
            ctx.begin_path();
            ctx.arc(cx, cy, 6, 0, 7);
            ctx.stroke();
            ctx.restore();
            // the ink rubbed off at the field's edges
            for _ in 0..60 {
                let t = r.next();
                let (x, y) = match (r.next() * 4.0) as i32 {
                    0 => (bx + t * bw, by + r.next() * 3.0),
                    1 => (bx + t * bw, by + bh - r.next() * 3.0),
                    2 => (bx + r.next() * 3.0, by + t * bh),
                    _ => (bx + bw - r.next() * 3.0, by + t * bh),
                };
                ctx.set_fill_style("rgba(220,208,192,0.5)");
                ctx.fill_rect(x - 1.0, y - 1.0, 2.0 + r.next() * 4.0, 2.0 + r.next() * 4.0);
            }
            worn(ctx, &mut r);
        })
    }

    /// A suit's pip, `s` px high, centred at (x, y): 0 hearts, 1 diamonds,
    /// 2 spades, 3 clubs
    fn pip(ctx: &mut Canvas, suit: usize, x: f64, y: f64, s: f64, down: bool) {
        ctx.save();
        ctx.translate(x, y);
        if down {
            ctx.rotate(PI);
        }
        ctx.scale(s / 20.0, s / 20.0);
        ctx.begin_path();
        match suit {
            0 => {
                ctx.move_to(0, 10);
                ctx.bezier_curve_to(-4, 5, -10, 1, -10, -4);
                ctx.bezier_curve_to(-10, -9, -4, -11, 0, -5);
                ctx.bezier_curve_to(4, -11, 10, -9, 10, -4);
                ctx.bezier_curve_to(10, 1, 4, 5, 0, 10);
                ctx.fill();
            }
            1 => {
                ctx.move_to(0, -10);
                ctx.quadratic_curve_to(3, -4, 8, 0);
                ctx.quadratic_curve_to(3, 4, 0, 10);
                ctx.quadratic_curve_to(-3, 4, -8, 0);
                ctx.quadratic_curve_to(-3, -4, 0, -10);
                ctx.fill();
            }
            2 => {
                ctx.move_to(0, -10);
                ctx.bezier_curve_to(-4, -5, -10, -1, -10, 3);
                ctx.bezier_curve_to(-10, 8, -4, 9, 0, 4);
                ctx.bezier_curve_to(4, 9, 10, 8, 10, 3);
                ctx.bezier_curve_to(10, -1, 4, -5, 0, -10);
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(0, 3);
                ctx.quadratic_curve_to(1, 8, 4, 10);
                ctx.line_to(-4, 10);
                ctx.quadratic_curve_to(-1, 8, 0, 3);
                ctx.fill();
            }
            _ => {
                for (cx, cy) in [(0.0, -5.0), (-5.2, 2.0), (5.2, 2.0)] {
                    ctx.begin_path();
                    ctx.arc(cx, cy, 4.8, 0, 7);
                    ctx.fill();
                }
                ctx.begin_path();
                ctx.move_to(0, 0);
                ctx.quadratic_curve_to(1, 7, 4, 10);
                ctx.line_to(-4, 10);
                ctx.quadratic_curve_to(-1, 7, 0, 0);
                ctx.fill();
            }
        }
        ctx.restore();
    }

    /// Where a pip card's pips go (as fractions of the pip field: x, y)
    fn pips_of(n: usize) -> Vec<(f64, f64)> {
        let (l, m, rr) = (0.0, 0.5, 1.0);
        match n {
            1 => vec![(m, 0.5)],
            2 => vec![(m, 0.0), (m, 1.0)],
            3 => vec![(m, 0.0), (m, 0.5), (m, 1.0)],
            4 => vec![(l, 0.0), (rr, 0.0), (l, 1.0), (rr, 1.0)],
            5 => vec![(l, 0.0), (rr, 0.0), (m, 0.5), (l, 1.0), (rr, 1.0)],
            6 => vec![(l, 0.0), (rr, 0.0), (l, 0.5), (rr, 0.5), (l, 1.0), (rr, 1.0)],
            7 => vec![(l, 0.0), (rr, 0.0), (m, 0.25), (l, 0.5), (rr, 0.5), (l, 1.0), (rr, 1.0)],
            8 => vec![(l, 0.0), (rr, 0.0), (m, 0.25), (l, 0.5), (rr, 0.5), (m, 0.75), (l, 1.0), (rr, 1.0)],
            9 => vec![(l, 0.0), (rr, 0.0), (l, 1.0 / 3.0), (rr, 1.0 / 3.0), (m, 0.5), (l, 2.0 / 3.0), (rr, 2.0 / 3.0), (l, 1.0), (rr, 1.0)],
            _ => vec![
                (l, 0.0),
                (rr, 0.0),
                (m, 1.0 / 6.0),
                (l, 1.0 / 3.0),
                (rr, 1.0 / 3.0),
                (l, 2.0 / 3.0),
                (rr, 2.0 / 3.0),
                (m, 5.0 / 6.0),
                (l, 1.0),
                (rr, 1.0),
            ],
        }
    }

    /// A pip card's face (an ace to a ten, red or black): its indices in a
    /// serif in two corners, its pips laid out as a deck's are, those in its
    /// lower half turned
    pub fn face_tex(red: bool, salt: u32) -> Tex {
        canvas_tex_opt(W, H, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0xface);
            ctx.set_fill_style("#e6dfcf");
            ctx.fill_rect(0, 0, w, h);
            let n = 1 + (r.next() * 10.0) as usize;
            let suit = if red { (r.next() * 2.0) as usize } else { 2 + (r.next() * 2.0) as usize };
            let ink = if red { "rgba(150,40,40,0.95)" } else { "rgba(34,30,28,0.95)" };
            let rank = if n == 1 { "A".to_string() } else { n.to_string() };
            ctx.set_fill_style(ink);
            ctx.set_text_align("center");
            for flip in [false, true] {
                ctx.save();
                if flip {
                    ctx.translate(w, h);
                    ctx.rotate(PI);
                }
                ctx.set_font("bold 30px Times New Roman");
                ctx.fill_text_max(&rank, 22, 38, 34);
                pip(ctx, suit, 22.0, 54.0, 17.0, false);
                ctx.restore();
            }
            let (fx, fy, fw, fh) = (64.0, 52.0, w - 128.0, h - 104.0);
            for (px, py) in pips_of(n) {
                let s = if n == 1 { 80.0 } else { 40.0 };
                pip(ctx, suit, fx + px * fw, fy + py * fh, s, py > 0.5);
            }
            worn(ctx, &mut r);
        })
    }

    /// The deck's edges (v up the deck): a line for each of the 46 cards,
    /// greyer toward the bottom where it lay in the dust
    pub fn edges_tex(salt: u32) -> Tex {
        canvas_tex_opt(32, 128, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0xed6e);
            ctx.set_fill_style("#d6cfbf");
            ctx.fill_rect(0, 0, w, h);
            for k in 0..46 {
                let y = (k as f64 + 0.5) * h / 46.0;
                ctx.set_fill_style(format!("rgba(80,64,58,{:.2})", 0.25 + 0.25 * r.next()));
                ctx.fill_rect(0, y, w, 0.9);
            }
            let gr = ctx.create_linear_gradient(0, h, 0, 0);
            gr.add_color_stop(0, "rgba(80,70,54,0.5)");
            gr.add_color_stop(1, "rgba(80,70,54,0.1)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, w, h);
        })
    }
}
