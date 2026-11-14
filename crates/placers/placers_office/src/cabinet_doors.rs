//! `place_cabinet_doors`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates as wall_cands;

// Supply cabinet doors: 20% chance per level, 1 two-door unit flush on a wall.
pub(crate) fn place_cabinet_doors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let door_mat = lambert!({ color: 0xa89880 });
    let handle_mat = lambert!({ color: 0x2a2018 });
    let trim_mat = lambert!({ color: 0x7a6a54 });
    let cabinet = group();
    // Overall cabinet width 1.4 m, split into 2 doors of 0.68 m each.
    let door_w = 0.68;
    let door_h = 2.0;
    let gap = 0.04;
    // 10% chance one door slightly ajar showing dark interior.
    let ajar_side: i32 = if rnd() < 0.10 {
        if rnd() < 0.5 {
            -1
        } else {
            1
        }
    } else {
        0
    };
    let ajar_f = ajar_side as f64;
    if ajar_side != 0 {
        let interior_mat = basic_mat!({ color: 0x0a0806 });
        let interior = mesh(plane_geo!(0.4, door_h * 0.9), &interior_mat);
        interior.position().set(ajar_f * (door_w / 2.0 + gap / 2.0), 0, -0.02);
        cabinet.add(&interior);
        // 60% chance: small interior contents peek through the ajar gap.
        if rnd() < 0.60 {
            let shelf_mat = lambert!({ color: 0x3a2a20 });
            let shelf_y = -0.15 + rnd() * 0.3; // shelf between -0.15 and 0.15 m
            let shelf = mesh(pg("CabinetDoors__shelf", box_geo!(0.38, 0.006, 0.14)), &shelf_mat);
            shelf.position().set(ajar_f * (door_w / 2.0 + gap / 2.0), shelf_y, -0.01);
            cabinet.add(&shelf);
            let item_colors: [u32; 4] = [0xa08060, 0x606060, 0x807040, 0x704028];
            let item_count = 1 + (rnd() * 2.0).floor() as i32;
            for _i in 0..item_count {
                let is_can = rnd() < 0.5;
                let item_mat = lambert!({ color: pick(&item_colors) });
                let item = if is_can {
                    mesh(pg("CabinetDoors__can", cyl_geo!(0.024, 0.024, 0.08, 6)), &item_mat)
                } else {
                    mesh(pg("CabinetDoors__carton", box_geo!(0.05, 0.07, 0.04)), &item_mat)
                };
                let shelf_center = ajar_f * (door_w / 2.0 + gap / 2.0);
                item.position().set(shelf_center + (rnd() - 0.5) * 0.28, shelf_y + 0.043, -0.02 + 0.04);
                item.rotation().y = (rnd() - 0.5) * 0.4;
                cabinet.add(&item);
            }
        }
    }
    for side in [-1i32, 1] {
        let sf = side as f64;
        let door = mesh(pg("CabinetDoors__door", plane_geo!(door_w, door_h)), &door_mat);
        door.position().set(sf * (door_w / 2.0 + gap / 2.0), 0, 0.005);
        // If this side is ajar, rotate the door outward slightly.
        if side == ajar_side {
            door.rotation().y = sf * -0.35; // ~20° swing
            door.position().z = 0.03; // pull forward as it swings open
        }
        cabinet.add(&door);
        // Frame trim around each door.
        for (ty, sx, sy) in [(door_h / 2.0, door_w, 0.03), (-door_h / 2.0, door_w, 0.03)] {
            let tr = mesh(pg("CabinetDoors__trim", plane_geo!(sx, sy)), &trim_mat);
            tr.position().set(sf * (door_w / 2.0 + gap / 2.0), ty, 0.006);
            cabinet.add(&tr);
        }
        // Handle: small dark box near center-adjoining edge.
        let handle = mesh(pg("CabinetDoors__handle", box_geo!(0.015, 0.10, 0.015)), &handle_mat);
        handle.position().set(sf * (gap / 2.0 + 0.05), 0, 0.014);
        cabinet.add(&handle);
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let center_h = 1.1; // door center at 1.1 m (bottom at 0.1, top at 2.1)
    if c.dx.abs() == 1 {
        cabinet.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        cabinet.position().set(wall_x, center_h, wall_z);
    } else {
        cabinet.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        cabinet.position().set(wall_x, center_h, wall_z);
    }
    g.world.add(&cabinet);
    // 30% chance: toolbox on floor 0.35-0.60 m in front of cabinet.
    if rnd() < 0.30 {
        let body_tool_mat = lambert!({ color: 0x8a2020 });
        let trim_tool_mat = lambert!({ color: 0x2a1010 });
        let handle_tool_mat = lambert!({ color: 0x181410 });
        let toolbox = group();
        let tbody = mesh(pg("CabinetDoors__toolbox", box_geo!(0.24, 0.10, 0.12)), &body_tool_mat);
        tbody.position().y = 0.05;
        toolbox.add(&tbody);
        // 20% chance: hinged-open lid variant showing 2-3 tiny tools inside.
        let is_open = rnd() < 0.20;
        if is_open {
            let interior_mat = basic_mat!({ color: 0x0a0605 });
            let interior = mesh(plane_geo!(0.22, 0.10), &interior_mat);
            interior.rotation().x = -PI / 2.0;
            interior.position().y = 0.101;
            toolbox.add(&interior);
            // Lid rotated up on back hinge.
            let open_lid = mesh(pg("CabinetDoors__lidopen", box_geo!(0.245, 0.010, 0.125)), &body_tool_mat);
            open_lid.position().set(0, 0.10, -0.062); // rotate around back edge
            open_lid.rotation().x = -1.10; // ~63° open
            open_lid.position().y += 0.055; // lift up so hinge stays at back
            open_lid.position().z += 0.017;
            toolbox.add(&open_lid);
            let tool_colors: [u32; 3] = [0x707878, 0x8a6040, 0x5a5850];
            let tool_count = 2 + (rnd() * 2.0).floor() as i32;
            for _ti in 0..tool_count {
                let tool_mat = lambert!({ color: pick(&tool_colors) });
                let is_screw = rnd() < 0.4;
                let tool = if is_screw {
                    mesh(pg("CabinetDoors__screwdriver", cyl_geo!(0.005, 0.005, 0.06, 5)), &tool_mat)
                } else {
                    mesh(pg("CabinetDoors__spanner", box_geo!(0.04, 0.008, 0.014)), &tool_mat)
                };
                if is_screw {
                    tool.rotation().z = PI / 2.0;
                }
                tool.position().set((rnd() - 0.5) * 0.18, 0.106, (rnd() - 0.5) * 0.08);
                tool.rotation().y = rnd() * PI * 2.0;
                toolbox.add(&tool);
            }
        } else {
            // Closed variant: flat lid ridge.
            let ridge = mesh(pg("CabinetDoors__lidtop", box_geo!(0.245, 0.004, 0.125)), &trim_tool_mat);
            ridge.position().y = 0.10;
            toolbox.add(&ridge);
        }
        // Carry handle: folded flat on the closed lid's top, or on the open lid's outer face.
        let thandle = mesh(pg("CabinetDoors__carryhandle", cyl_geo!(0.006, 0.006, 0.14, 5)), &handle_tool_mat);
        thandle.rotation().z = PI / 2.0;
        if is_open {
            thandle.position().set(0, 0.160, -0.055);
        } else {
            thandle.position().y = 0.108;
        }
        toolbox.add(&thandle);
        // Latch: tiny box at front-center (closed variant only).
        if !is_open {
            let latch = mesh(pg("CabinetDoors__latch", box_geo!(0.020, 0.015, 0.006)), &handle_tool_mat);
            latch.position().set(0, 0.055, 0.063);
            toolbox.add(&latch);
        }
        // Position 0.35-0.60 m out from cabinet along wall-normal.
        let fx = (-c.dx) as f64;
        let fz = (-c.dz) as f64;
        let tdist = 0.35 + rnd() * 0.25;
        toolbox.position().set(wall_x + fx * tdist + (rnd() - 0.5) * 0.5, 0, wall_z + fz * tdist + (rnd() - 0.5) * 0.5);
        toolbox.rotation().y = rnd() * PI * 2.0;
        g.world.add(&toolbox);
        // 30% chance: small wrench dropped on floor 0.10-0.25 m from toolbox.
        if rnd() < 0.30 {
            let wrench_mat = lambert!({ color: 0x707878 });
            let wrench = group();
            let w_shaft = mesh(pg("CabinetDoors__wrench", cyl_geo!(0.006, 0.006, 0.12, 5)), &wrench_mat);
            w_shaft.rotation().z = PI / 2.0;
            wrench.add(&w_shaft);
            // Head jaws: 2 small parallel boxes side by side in the plane the wrench lies in.
            for jyoff in [-0.010, 0.010] {
                let jaw = mesh(pg("CabinetDoors__jaw", box_geo!(0.020, 0.005, 0.008)), &wrench_mat);
                jaw.position().set(0.055, 0, jyoff);
                wrench.add(&jaw);
            }
            let knurl = mesh(pg("CabinetDoors__knurl", cyl_geo!(0.010, 0.010, 0.008, 6)), &wrench_mat);
            knurl.rotation().z = PI / 2.0;
            knurl.position().set(0.038, 0, 0);
            wrench.add(&knurl);
            let w_ang = rnd() * PI * 2.0;
            let w_dist = 0.10 + rnd() * 0.15;
            let tp = toolbox.pos();
            wrench.position().set(tp.x + w_ang.cos() * w_dist, 0.010, tp.z + w_ang.sin() * w_dist); // resting on the knurl ring
            wrench.rotation().y = rnd() * PI * 2.0;
            g.world.add(&wrench);
            // 40% chance: small dark oil-stain circle beneath wrench.
            if rnd() < 0.40 {
                // Per-weather oil color.
                let oil_color = oil_stain_by_weather(g.weather_key());
                let oil_mat = basic_mat!({ color: oil_color, transparent: true, opacity: 0.7, depthWrite: false });
                let oil = mesh(circle_geo!(0.04 + rnd() * 0.03, 10), &oil_mat);
                oil.rotation().x = -PI / 2.0;
                oil.rotation().z = rnd() * PI * 2.0;
                oil.scale().set(1.3, 0.9, 1); // slight stretch — organic pool
                let wp = wrench.pos();
                oil.position().set(wp.x + (rnd() - 0.5) * 0.05, 0.003, wp.z + (rnd() - 0.5) * 0.05);
                g.world.add(&oil);
            }
        }
    }
}

/// Oil stain per weather
fn oil_stain_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0c10, 0x0a0805, 0x1a0605, 0x100a08, 0x0a0806, 0x0a0806)
}
