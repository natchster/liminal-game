//! `place_servant_hall_bell_racks`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;
use std::collections::HashSet;

// Servant's-hall bell rack (wall): 2.5% chance per level.
pub(crate) fn place_servant_hall_bell_racks(g: &mut Game, _dist: &[i32]) {
    g.pp.bell_rack_positions.clear(); // reset per level
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let wood_colors = [0x2a1810, 0x402015, 0x604020];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let br = group();
    // Wooden mounting board.
    let bell_count = 6 + (rnd() * 3.0).floor() as i32; // 6, 7, or 8
    let bwidth = bell_count as f64 * 0.070 + 0.030;
    let bheight = 0.28;
    let board = mesh(pg("ServantHallBellRacks__board", box_geo!(bwidth, bheight, 0.020)), &wood_mat);
    br.add(&board);
    // Ornamental top trim (back onto the wall, 9 mm past the board face).
    let top_trim = mesh(pg("ServantHallBellRacks__topTrim", box_geo!(bwidth + 0.020, 0.020, 0.030)), &wood_mat);
    top_trim.position().set(0, bheight / 2.0 + 0.010, 0.004);
    br.add(&top_trim);
    // Bottom trim.
    let bot_trim = mesh(pg("ServantHallBellRacks__botTrim", box_geo!(bwidth + 0.020, 0.015, 0.030)), &wood_mat);
    bot_trim.position().set(0, -bheight / 2.0 - 0.008, 0.004);
    br.add(&bot_trim);
    // Title strip at top centre.
    let title_plate = mesh(pg("ServantHallBellRacks__title", box_geo!(bwidth * 0.4, 0.020, 0.001)), &brass_mat);
    title_plate.position().set(0, bheight / 2.0 - 0.020, 0.011);
    br.add(&title_plate);
    // 6-8 small brass bells in a row + numbered labels.
    let _room_names = ["DRAWING", "LIBRARY", "DINING", "BILLIARDS", "STUDY", "PARLOUR", "MASTER", "NURSERY"];
    for i in 0..bell_count {
        let bx = -bwidth / 2.0 + 0.030 + i as f64 * 0.070;
        // Spring wire: six 60° arcs overlapping into one 210° curl in the
        // plane across the board (YZ).
        let spring_seg = 6;
        for s in 0..spring_seg {
            let spring = mesh(pg("ServantHallBellRacks__spring", torus_geo!(0.010, 0.0008, 3, 5, PI / 3.0)), &brass_mat);
            spring.rotation().y = -PI / 2.0;
            spring.rotation().z = -PI / 3.0 + s as f64 * (PI / 6.0);
            spring.position().set(bx, 0.0540, 0.0195);
            br.add(&spring);
        }
        // Bell body — small dome.
        let small_bell = mesh(pg("ServantHallBellRacks__bell", cyl_geo!(0.014, 0.010, 0.024, 8, 1, true)), &brass_mat);
        small_bell.position().set(bx, 0.030, 0.025);
        br.add(&small_bell);
        // Bell top cap.
        let bell_cap = mesh(pg("ServantHallBellRacks__cap", cyl_geo!(0.010, 0.010, 0.003, 8)), &brass_mat);
        bell_cap.position().set(bx, 0.043, 0.025);
        br.add(&bell_cap);
        // Numbered label plate below bell.
        let label_plate = mesh(pg("ServantHallBellRacks__label", box_geo!(0.050, 0.020, 0.001)), lambert!({ color: 0xf0e8d0 }));
        label_plate.position().set(bx, -0.030, 0.011);
        br.add(&label_plate);
        // Number engraving.
        let num_mark = mesh(box_geo!(0.015, 0.010, 0.0005), lambert!({ color: 0x1a1a1a }));
        num_mark.position().set(bx, -0.030, 0.012);
        br.add(&num_mark);
        // Room label plate below number.
        let room_plate = mesh(pg("ServantHallBellRacks__roomPlate", box_geo!(0.055, 0.016, 0.001)), lambert!({ color: 0xd8c8a8 }));
        room_plate.position().set(bx, -0.060, 0.011);
        br.add(&room_plate);
        // Room name mark.
        let room_mark = mesh(box_geo!(0.040, 0.006, 0.0005), lambert!({ color: 0x2a1810 }));
        room_mark.position().set(bx, -0.060, 0.012);
        br.add(&room_mark);
    }
    mount_on_wall(g, &br, &c, 0.011, 1.80, 0.3);
    br.rotation().z = (rnd() - 0.5) * 0.010;
    g.world.add(&br);
    let p = br.pos();
    g.pp.bell_rack_positions.push(v3(p.x, 0, p.z));
}
