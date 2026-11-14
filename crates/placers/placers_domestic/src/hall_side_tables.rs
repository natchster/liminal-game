//! `place_hall_side_tables`
use game::prelude::*;
use placers_kit::place::wall_mount as mount_on_wall;
use placers_kit::place::wall_tiles;

// Hall side table w/ key tray + mail stack (floor near wall): 3% chance per level.
pub(crate) fn place_hall_side_tables(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let hst = group();
    // Tabletop.
    let (table_w, table_d, table_t) = (0.50, 0.24, 0.020);
    let table_y = 0.72;
    let top = mesh(pg("HallSideTables__top", box_geo!(table_w, table_t, table_d)), &wood_mat);
    top.position().set(0, table_y, 0);
    hst.add(&top);
    // Trim rim on top edges (decorative).
    let trim = mesh(pg("HallSideTables__trim", box_geo!(table_w + 0.008, 0.006, table_d + 0.008)), &wood_mat);
    trim.position().set(0, table_y + table_t / 2.0 + 0.003, 0);
    hst.add(&trim);
    // 4 tapered legs.
    for cx in [-table_w * 0.42, table_w * 0.42] {
        for cz in [-table_d * 0.35, table_d * 0.35] {
            let leg = mesh(pg("HallSideTables__leg", cyl_geo!(0.010, 0.014, table_y - table_t / 2.0, 6)), &wood_mat);
            leg.position().set(cx, (table_y - table_t / 2.0) / 2.0, cz);
            hst.add(&leg);
        }
    }
    // Small drawer front (thin panel + brass knob).
    let drawer = mesh(pg("HallSideTables__drawer", box_geo!(table_w - 0.020, 0.06, 0.003)), &wood_mat);
    // Apron-mounted: top edge under the tabletop, back on the front legs' faces.
    drawer.position().set(0, table_y - table_t / 2.0 - 0.030, table_d * 0.35 + 0.0115);
    drawer.scale().x = 0.44 / (table_w - 0.020);
    hst.add(&drawer);
    let knob_mat = lambert!({ color: 0xc0a060 });
    let knob = mesh(pg("HallSideTables__knob", sphere_geo!(0.008, 6, 5)), &knob_mat);
    knob.position().set(0, table_y - table_t / 2.0 - 0.030, table_d * 0.35 + 0.0145);
    hst.add(&knob);
    // Key tray on top (shallow dish).
    let tray_colors = [0xc0a060, 0x606060, 0x2060a0, 0xa02020];
    let tray_mat = lambert!({ color: choice(&tray_colors) });
    let tray = mesh(pg("HallSideTables__tray", cyl_geo!(0.05, 0.045, 0.008, 12)), &tray_mat);
    tray.position().set(-table_w * 0.25, table_y + table_t / 2.0 + 0.004, 0);
    hst.add(&tray);
    // 60% chance: key inside tray.
    if rnd() < 0.6 {
        let key_mat = lambert!({ color: 0xa0a0a8 });
        let key_body = mesh(pg("HallSideTables__keyBlade", box_geo!(0.020, 0.001, 0.003)), &key_mat);
        key_body.position().set(-table_w * 0.25 + 0.008, table_y + table_t / 2.0 + 0.0055, 0); // on the dish floor
        key_body.rotation().y = 0.3;
        hst.add(&key_body);
        let key_head = mesh(pg("HallSideTables__keyBow", cyl_geo!(0.006, 0.006, 0.001, 6)), &key_mat);
        // Bow lying flat on the blade's axis, just past its back end.
        key_head.position().set(-table_w * 0.25 + 0.008 - 0.015 * 0.3f64.cos(), table_y + table_t / 2.0 + 0.0055, 0.015 * 0.3f64.sin());
        key_head.rotation().y = 0.3;
        hst.add(&key_head);
    }
    // Mail stack on right side of table (2-4 stacked envelopes).
    let mail_count = 2 + (rnd() * 3.0).floor() as i32;
    let envelope_colors = [0xf0e8d0, 0xe8dcc8, 0xf0d8c8, 0xd8d0c0, 0xf0e0e0];
    for i in 0..mail_count {
        let env_mat = lambert!({ color: choice(&envelope_colors) });
        let env = mesh(pg("HallSideTables__envelope", box_geo!(0.10, 0.002, 0.06)), &env_mat);
        let ex = table_w * 0.25 + (rnd() - 0.5) * 0.010;
        let ey = table_y + table_t / 2.0 + 0.002 + i as f64 * 0.003;
        let ez = (rnd() - 0.5) * 0.020;
        env.position().set(ex, ey, ez);
        env.rotation().y = (rnd() - 0.5) * 0.20;
        hst.add(&env);
        // 40% envelope has address text lines.
        if rnd() < 0.4 {
            let text_mat = lambert!({ color: 0x202020 });
            let addr_line = mesh(box_geo!(0.05, 0.0015, 0.001), &text_mat);
            let ep = env.pos();
            let ery = env.rot().y;
            addr_line.position().set(ep.x - 0.01, ep.y + 0.001, ep.z - 0.008);
            addr_line.rotation().y = ery;
            hst.add(&addr_line);
            // 30% chance of stamp square top-right.
            if rnd() < 0.3 {
                let stamp_mat = lambert!({ color: 0xa02040 });
                let stamp = mesh(pg("HallSideTables__stamp", box_geo!(0.012, 0.002, 0.014)), &stamp_mat);
                stamp.position().set(ep.x + 0.035, ep.y + 0.001, ep.z - 0.018);
                stamp.rotation().y = ery;
                hst.add(&stamp);
            }
        }
    }
    // Optional small vase center-back (40%).
    if rnd() < 0.4 {
        let vase_colors = [0xa02020, 0x2060a0, 0x408060, 0xd0a040, 0xf0e8d0];
        let vase_mat = lambert!({ color: choice(&vase_colors) });
        let vase = mesh(pg("HallSideTables__vase", cyl_geo!(0.030, 0.024, 0.10, 8)), &vase_mat);
        vase.position().set(0, table_y + table_t / 2.0 + 0.050, -table_d * 0.3);
        hst.add(&vase);
    }
    // Position at wall — table backs against wall.
    mount_on_wall(g, &hst, x, z, dx, dz, 0.150);
    g.world.add(&hst);
}
