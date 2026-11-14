//! `place_half_wall_partitions`
use game::prelude::*;

// Half-height wall partition zone (floor scenery): 25% chance per level.
// Purely decorative — no collision blocking.
pub(crate) fn place_half_wall_partitions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    }
    // Interior floor tiles ≥4 tiles from any border with a 2-tile open run.
    #[derive(Clone, Copy)]
    struct Spot {
        x: i32,
        z: i32,
        has_h_run: bool,
        has_v_run: bool,
    }
    let mut opens: Vec<Spot> = vec![];
    for z in 4..g.h - 4 {
        for x in 4..g.w - 4 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            let has_h_run = g.tiles[g.t_idx(x + 1, z)] == 0 && g.tiles[g.t_idx(x + 2, z)] == 0;
            let has_v_run = g.tiles[g.t_idx(x, z + 1)] == 0 && g.tiles[g.t_idx(x, z + 2)] == 0;
            if has_h_run || has_v_run {
                opens.push(Spot { x, z, has_h_run, has_v_run });
            }
        }
    }
    if opens.is_empty() {
        return;
    }
    let spot = opens[(rnd() * opens.len() as f64).floor() as usize];
    // 4-colour plaster/wood wall pool.
    let wall_colors = [0xd0c8b0, 0x8b7860, 0xa89880, 0x604020];
    let wall_color = choice(&wall_colors);
    let wall_mat = lambert!({ color: wall_color });
    let cap_mat = lambert!({ color: 0x2a2010 }); // dark wooden top cap
    let part_height = 1.5;
    let part_thick = 0.14;
    let part_group = group();
    // Choose orientation: prefer whichever axis has an open run.
    let use_h = spot.has_h_run && (!spot.has_v_run || rnd() < 0.5);
    // Decide shape: 60% straight run, 40% L-shape.
    let is_l = rnd() < 0.4;
    // Build primary run: 2 tile segments.
    let primary_len = CELL * 2.0 - 0.4; // slight gap at ends for walk-around
    let primary =
        mesh(box_geo!(if use_h { primary_len } else { part_thick }, part_height, if use_h { part_thick } else { primary_len }), &wall_mat);
    primary.position().set(
        if use_h { g.wx(spot.x) + CELL * 0.5 } else { g.wx(spot.x) },
        part_height / 2.0,
        if use_h { g.wz(spot.z) } else { g.wz(spot.z) + CELL * 0.5 },
    );
    part_group.add(&primary);
    // Wooden top cap on primary run.
    let cap1 = mesh(
        box_geo!(if use_h { primary_len } else { part_thick + 0.02 }, 0.03, if use_h { part_thick + 0.02 } else { primary_len }),
        &cap_mat,
    );
    cap1.position().copy(&primary.pos());
    cap1.position().y = part_height + 0.015;
    part_group.add(&cap1);
    // L-shape addition: perpendicular short run at one end.
    if is_l {
        let short_len = CELL - 0.3;
        let perp_dir = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let (ssx, ssz) = if use_h {
            (g.wx(spot.x + 2) - 0.2 - short_len / 2.0, g.wz(spot.z) + perp_dir * (CELL / 2.0) - perp_dir * short_len / 2.0)
        } else {
            (g.wx(spot.x) + perp_dir * (CELL / 2.0) - perp_dir * short_len / 2.0, g.wz(spot.z + 2) - 0.2 - short_len / 2.0)
        };
        let perp =
            mesh(box_geo!(if use_h { part_thick } else { short_len }, part_height, if use_h { short_len } else { part_thick }), &wall_mat);
        perp.position().set(ssx, part_height / 2.0, ssz);
        part_group.add(&perp);
        let cap2 = mesh(
            box_geo!(if use_h { part_thick + 0.02 } else { short_len }, 0.03, if use_h { short_len } else { part_thick + 0.02 }),
            &cap_mat,
        );
        cap2.position().copy(&perp.pos());
        cap2.position().y = part_height + 0.015;
        part_group.add(&cap2);
    }
    // Subtle wall trim strip at bottom (baseboard).
    let baseboard_mat = lambert!({ color: 0x2a1810 });
    let baseboard1 = mesh(
        box_geo!(
            if use_h { primary_len + 0.02 } else { part_thick + 0.04 },
            0.08,
            if use_h { part_thick + 0.04 } else { primary_len + 0.02 }
        ),
        &baseboard_mat,
    );
    baseboard1.position().copy(&primary.pos());
    baseboard1.position().y = 0.04;
    part_group.add(&baseboard1);
    g.world.add(&part_group);
}
