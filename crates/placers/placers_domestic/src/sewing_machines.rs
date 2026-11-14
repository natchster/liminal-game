//! `place_sewing_machines`
use game::prelude::*;
use placers_kit::geometry::orient_seg;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Sewing machine on stand (floor): 3% chance per level.
pub(crate) fn place_sewing_machines(g: &mut Game, _dist: &[i32]) {
    g.pp.sewing_machine_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Machine body colors — vintage black/dark green w/ gold accents.
    let body_colors = [0x101010, 0x1a2618, 0x181410, 0x201810];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let metal_mat = lambert!({ color: 0xc0a060 });
    let needle_mat = lambert!({ color: 0xd0d0d0 });
    let fabric_mat = lambert!({ color: [0xf0c0c0, 0xa0c0f0, 0xf0e0a0, 0xf0f0f0, 0xd0e0c0][(rnd() * 5.0).floor() as usize] });
    let m = group();
    // Wooden base plate.
    let (base_w, base_d) = (0.24, 0.14);
    let base = mesh(pg("SewingMachines__base", box_geo!(base_w, 0.014, base_d)), &wood_mat);
    base.position().y = 0.007;
    m.add(&base);
    // Machine bed — flat horizontal block on top of wood base.
    let bed = mesh(pg("SewingMachines__bed", box_geo!(base_w * 0.85, 0.010, base_d * 0.65)), &body_mat);
    bed.position().set(0, 0.020, 0);
    m.add(&bed);
    // Left column (vertical support).
    let column = mesh(pg("SewingMachines__column", box_geo!(0.025, 0.10, base_d * 0.55)), &body_mat);
    column.position().set(-base_w * 0.35, 0.075, 0);
    m.add(&column);
    // Top horizontal arm.
    let arm = mesh(pg("SewingMachines__arm", box_geo!(base_w * 0.75, 0.026, base_d * 0.55)), &body_mat);
    arm.position().set(0, 0.115 + 0.013, 0);
    m.add(&arm);
    // Presser foot descending from arm's right end.
    let foot = mesh(pg("SewingMachines__foot", box_geo!(0.014, 0.022, 0.030)), &body_mat);
    // Shank stretched into the presser bar: from the arm's underside down to the cloth.
    foot.scale().y = 3.91;
    foot.position().set(base_w * 0.32, 0.072, 0);
    m.add(&foot);
    // Needle — very thin cylinder descending, beside the presser bar's shank.
    let needle = mesh(cyl_geo!(0.001, 0.001, 0.020, 4), &needle_mat);
    needle.scale().y = 4.4;
    needle.position().set(base_w * 0.32 - 0.0055, 0.074, 0);
    m.add(&needle);
    // Needle up-down oscillator — 40% chance (machine "running").
    // Track running machines for stitch-click audio.
    let is_running = rnd() < 0.4;
    if is_running {
        let needle_base_y = 0.074;
        let needle_phase = rnd() * PI * 2.0;
        let needle2 = needle.clone();
        g.animate(move |_g, _dt, t| {
            // Fast up-down oscillation ~4 Hz w/ ±3 mm amplitude.
            needle2.position().y = needle_base_y + (t * 8.0 + needle_phase).sin() * 0.003;
        });
    }
    // Note: the `sewing_machine_positions` push happens after world placement below.
    m.user().set("isRunning", is_running);
    // Thread spool on top of arm.
    let spool_mat = lambert!({ color: 0xd8b878 });
    let spool_thread_colors = [0xa02040, 0x2060a0, 0x101010, 0xf0f0f0, 0x408060];
    let spool_thread_mat = lambert!({ color: pick(&spool_thread_colors) });
    let spool_base = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.003, 8)), &spool_mat);
    spool_base.position().set(-base_w * 0.20, 0.155, 0);
    m.add(&spool_base);
    let spool_thread = mesh(pg("SewingMachines__thread", cyl_geo!(0.012, 0.012, 0.028, 8)), &spool_thread_mat);
    spool_thread.position().set(-base_w * 0.20, 0.172, 0);
    m.add(&spool_thread);
    let spool_top = mesh(pg("SewingBaskets__spoolend", cyl_geo!(0.014, 0.014, 0.003, 8)), &spool_mat);
    spool_top.position().set(-base_w * 0.20, 0.187, 0);
    m.add(&spool_top);
    // Thread trailing from spool to needle — oriented-segment chain.
    let thread_path = [
        v3(-base_w * 0.20, 0.187, 0), // Top of spool.
        v3(-base_w * 0.10, 0.155, 0), // Guide 1.
        v3(base_w * 0.10, 0.135, 0),  // Guide 2.
        v3(base_w * 0.32, 0.100, 0),  // Needle top.
    ];
    for i in 0..thread_path.len() - 1 {
        let (p1, p2) = (thread_path[i], thread_path[i + 1]);
        let dir_len = p2.distance_to(&p1);
        let seg = mesh(cyl_geo!(0.0008, 0.0008, dir_len, 3), &spool_thread_mat);
        orient_seg(&seg, &p1, &p2);
        m.add(&seg);
    }
    // Hand crank wheel on right side (large disc).
    let wheel = mesh(pg("SewingMachines__wheel", cyl_geo!(0.028, 0.028, 0.008, 12)), &metal_mat);
    wheel.rotation().z = PI / 2.0;
    wheel.position().set(base_w * 0.42, 0.125, 0);
    m.add(&wheel);
    // Wheel handle knob.
    let knob = mesh(pg("SewingMachines__knob", sphere_geo!(0.005, 5, 4)), &wood_mat);
    knob.position().set(base_w * 0.46, 0.145, 0);
    m.add(&knob);
    // Piece of fabric under presser foot — mid-stitch (small rectangle).
    let fabric = mesh(pg("SewingMachines__cloth", box_geo!(0.06, 0.002, 0.05)), &fabric_mat);
    fabric.position().set(base_w * 0.22, 0.028, 0);
    m.add(&fabric);
    // Gold trim lines on the arm's front/back faces, short of its rounded ends.
    let trim1 = mesh(box_geo!(base_w * 0.75, 0.002, 0.002), &metal_mat);
    trim1.scale().x = 0.9;
    trim1.position().set(0, 0.128, 0.0395);
    m.add(&trim1);
    let trim2 = mesh(box_geo!(base_w * 0.75, 0.002, 0.002), &metal_mat);
    trim2.scale().x = 0.9;
    trim2.position().set(0, 0.128, -0.0395);
    m.add(&trim2);
    m.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    m.rotation().y = rnd() * PI * 2.0;
    g.world.add(&m);
    // Register position only if this machine is "running" (has needle animator).
    if is_running {
        let mp = m.pos();
        g.pp.sewing_machine_positions.push(v3(mp.x, 0, mp.z));
    }
}
