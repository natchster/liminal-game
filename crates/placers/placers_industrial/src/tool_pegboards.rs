//! `place_tool_pegboards`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_tool_pegboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let board_mat = lambert!({ map: pegboard_texture() });
    let tool_mat = lambert!({ color: 0x505050 });
    let handle_mat = lambert!({ color: 0xc02020 });
    let peg = group();
    let mount_y = 1.65;
    // Main board.
    let board = mesh(pg("ToolPegboards__board", box_geo!(0.55, 0.42, 0.008)), &board_mat);
    board.position().y = mount_y;
    peg.add(&board);
    // Hanging tools: hammer + wrench + screwdriver silhouettes.
    // Hammer.
    let hammer_head = mesh(pg("ToolPegboards__hammerHead", box_geo!(0.008, 0.025, 0.055)), &tool_mat);
    // head sticks out of the board with its claw resting on the face (it went
    // 2 cm through the board); the handle hangs from the head's eye
    hammer_head.position().set(-0.20, mount_y + 0.10, 0.0315);
    peg.add(&hammer_head);
    let hammer_handle = mesh(pg("ToolPegboards__hammerHandle", cyl_geo!(0.005, 0.005, 0.11, 6)), &handle_mat);
    hammer_handle.position().set(-0.20, mount_y + 0.035, 0.0315);
    peg.add(&hammer_handle);
    // Wrench.
    let wrench = mesh(pg("ToolPegboards__wrench", box_geo!(0.006, 0.10, 0.010)), &tool_mat);
    wrench.position().set(-0.06, mount_y + 0.05, 0.008);
    peg.add(&wrench);
    let wrench_head = mesh(pg("ToolPegboards__wrenchHead", box_geo!(0.028, 0.020, 0.010)), &tool_mat);
    wrench_head.position().set(-0.06, mount_y + 0.11, 0.008);
    peg.add(&wrench_head);
    // Screwdriver.
    let sd_handle = mesh(pg("ToolPegboards__sdHandle", cyl_geo!(0.010, 0.008, 0.055, 8)), &handle_mat);
    sd_handle.position().set(0.06, mount_y + 0.09, 0.008);
    peg.add(&sd_handle);
    let sd_shaft = mesh(pg("ToolPegboards__sdShaft", cyl_geo!(0.003, 0.003, 0.060, 6)), &tool_mat);
    sd_shaft.position().set(0.06, mount_y + 0.035, 0.008); // enters the handle (was a 1.25 cm gap)
    peg.add(&sd_shaft);
    // Pliers.
    let pliers = mesh(pg("ToolPegboards__pliers", box_geo!(0.010, 0.10, 0.010)), &tool_mat);
    pliers.position().set(0.20, mount_y + 0.05, 0.008);
    peg.add(&pliers);
    let pliers_head = mesh(pg("ToolPegboards__pliersHead", box_geo!(0.020, 0.030, 0.010)), &tool_mat);
    pliers_head.position().set(0.20, mount_y + 0.115, 0.008);
    peg.add(&pliers_head);
    peg.rotation().y = face_wall(dx, dz);
    peg.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&peg);
}

// Hand tool wall pegboard: 3% chance per level. Board w/ hole grid +
// hanging tool silhouettes.
game::lazy_static_tex!(
    pegboard_texture,
    canvas_tex(160, 120, |ctx, w, h| {
        // Brown board.
        ctx.set_fill_style("#a08050");
        ctx.fill_rect(0, 0, w, h);
        // Dark hole grid.
        ctx.set_fill_style("#302010");
        let (cols, rows) = (12, 9);
        for r in 0..rows {
            for c in 0..cols {
                ctx.begin_path();
                ctx.arc(
                    6.0 + c as f64 * ((w - 12.0) / (cols - 1) as f64),
                    6.0 + r as f64 * ((h - 12.0) / (rows - 1) as f64),
                    2,
                    0,
                    PI * 2.0,
                );
                ctx.fill();
            }
        }
    })
);
