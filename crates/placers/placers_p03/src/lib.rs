//! Prop placers: smoke detectors, exit arrows and small floor debris (tissues,
//! dirt, pebbles, glass shards, paper airplanes, keyrings), forest-floor props
//! (leaf litter, twigs, acorns, pine cones, feathers, mushrooms, moss, ferns,
//! grass, bird nests, spider webs, ...) and makeup (palettes, perfume, nail
//! polish, mascara, brushes, tweezers, ...).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_smoke_detectors", place_smoke_detectors));
    r.push(("place_floss_containers", place_floss_containers));
    r.push(("place_makeup_compacts", place_makeup_compacts));
    r.push(("place_exit_arrow_signs", place_exit_arrow_signs));
    r.push(("place_shoe_boxes", place_shoe_boxes));
    r.push(("place_crumpled_tissues", place_crumpled_tissues));
    r.push(("place_dirt_clods", place_dirt_clods));
    r.push(("place_pebbles", place_pebbles));
    r.push(("place_glass_shards", place_glass_shards));
    r.push(("place_paper_airplanes", place_paper_airplanes));
    r.push(("place_keyring_with_keys", place_keyring_with_keys));
    r.push(("place_leaf_litter", place_leaf_litter));
    r.push(("place_twigs", place_twigs));
    r.push(("place_acorns", place_acorns));
    r.push(("place_pine_cones", place_pine_cones));
    r.push(("place_feathers", place_feathers));
    r.push(("place_mushrooms", place_mushrooms));
    r.push(("place_dandelions", place_dandelions));
    r.push(("place_moss_patches", place_moss_patches));
    r.push(("place_ferns", place_ferns));
    r.push(("place_grass_clumps", place_grass_clumps));
    r.push(("place_butterfly_wings", place_butterfly_wings));
    r.push(("place_snail_shells", place_snail_shells));
    r.push(("place_pine_needles", place_pine_needles));
    r.push(("place_bird_nests", place_bird_nests));
    r.push(("place_spider_webs", place_spider_webs));
    r.push(("place_chestnuts", place_chestnuts));
    r.push(("place_eyeshadow_palettes", place_eyeshadow_palettes));
    r.push(("place_perfume_bottles", place_perfume_bottles));
    r.push(("place_nail_polish", place_nail_polish));
    r.push(("place_mascaras", place_mascaras));
    r.push(("place_bobby_pins", place_bobby_pins));
    r.push(("place_cotton_pads", place_cotton_pads));
    r.push(("place_makeup_brushes", place_makeup_brushes));
    r.push(("place_hair_claws", place_hair_claws));
    r.push(("place_tweezers", place_tweezers));
}

// ------------------------------------------------------------------ helpers
/// The repeated `open_tiles` scan plus one random pick.
/// No draw when the list is empty (the placer returns first).
fn pick_open(g: &Game) -> Option<(i32, i32)> {
    let mut open: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open.push((x, z));
            }
        }
    }
    if open.is_empty() {
        return None;
    }
    Some(open[(rnd() * open.len() as f64).floor() as usize])
}

/// The repeated `wall_tiles` scan (first wall neighbour in [[1,0],[-1,0],[0,1],[0,-1]] order)
/// + one draw to pick a tile.
fn pick_wall(g: &Game) -> Option<(i32, i32, i32, i32)> {
    let mut wall: Vec<(i32, i32, i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        wall.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    if wall.is_empty() {
        return None;
    }
    Some(wall[(rnd() * wall.len() as f64).floor() as usize])
}

/// `wx(x) + (`rnd()` - 0.5) * s, 0, wz(z) + (`rnd()` - 0.5) * s` (x draw first).
fn jitter(g: &Game, x: i32, z: i32, s: f64) -> (f64, f64) {
    let px = g.wx(x) + (rnd() - 0.5) * s;
    let pz = g.wz(z) + (rnd() - 0.5) * s;
    (px, pz)
}

/// `rotation.y = `atan2`(-dx, -dz)` + position on the wall face `CELL * 0.5 - inset` out.
fn wall_mount(g: &Game, o: &Obj, x: i32, z: i32, dx: i32, dz: i32, inset: f64) {
    o.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    o.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - inset), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - inset));
}

/// A custom geometry: positions + index, vertex normals computed.
fn tri_geo(verts: Vec<f32>, idx: Vec<u32>) -> Geo {
    let geom = Geo::new();
    geom.set_position(verts);
    geom.set_index(idx);
    geom.compute_vertex_normals();
    geom
}

// R582 ceiling smoke detector: 6% chance per level. Round puck on ceiling + red status LED.
fn place_smoke_detectors(g: &mut Game, _dist: &[i32]) {
    g.pp.smoke_detector_positions.clear(); // R584 reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let body_mat = lambert!({ color: 0xf0efe8 });
    let grill_mat = lambert!({ color: 0x808070 });
    let det = group();
    let ceil_y = WALL_H; // puck top on the ceiling
    let puck = mesh(pg("SmokeDetectors__body", cyl_geo!(0.075, 0.075, 0.028, 16)), &body_mat);
    puck.position().y = ceil_y - 0.014;
    det.add(&puck);
    let grill = mesh(pg("SmokeDetectors__grill", cyl_geo!(0.045, 0.045, 0.010, 16)), &grill_mat);
    grill.position().y = ceil_y - 0.030;
    det.add(&grill);
    // Red status LED (self-lit).
    let led = mesh(sphere_geo!(0.003, 6, 4), basic_mat!({ color: 0xff2020 }));
    led.position().set(0.030, ceil_y - 0.030, 0.030);
    det.add(&led);
    // R583 slow LED throb (0.4 Hz).
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let pulse = 0.7 + 0.3 * (t * 2.5 + phase).sin();
        led.scale().set_scalar(pulse);
    });
    let (px, pz) = jitter(g, x, z, 1.0);
    det.position().set(px, 0, pz);
    g.world.add(&det);
    let p = det.pos();
    g.pp.smoke_detector_positions.push(v3(p.x, ceil_y, p.z)); // R584 track for chirp
}

// R584 dental floss container: 8% chance per level.
fn place_floss_containers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0xe0e0e0, 0x4080c0, 0xc0c0f0, 0xf0e0f0];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let lid_mat = lambert!({ color: 0x606080 });
    let floss = group();
    let body = mesh(pg("FlossContainers__body", box_geo!(0.045, 0.014, 0.038)), &body_mat);
    body.position().y = 0.007;
    floss.add(&body);
    let lid = mesh(pg("FlossContainers__lid", box_geo!(0.042, 0.004, 0.035)), &lid_mat);
    lid.position().y = 0.016;
    floss.add(&lid);
    let notch = mesh(pg("FlossContainers__notch", box_geo!(0.010, 0.004, 0.006)), lambert!({ color: 0x202020 }));
    notch.position().set(0.018, 0.014, 0);
    floss.add(&notch);
    let (px, pz) = jitter(g, x, z, 2.4);
    floss.position().set(px, 0, pz);
    floss.rotation().y = rnd() * PI * 2.0;
    g.world.add(&floss);
}

// R585 makeup compact: 8% chance per level. Sometimes open (mirror + powder visible).
fn place_makeup_compacts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let case_colors = [0xd050a0, 0x202020, 0xe0c040, 0xffffff, 0x40a0a0];
    let case_mat = lambert!({ color: choice(&case_colors) });
    let compact = group();
    let is_open = rnd() < 0.4; // 40% open
    let bottom = mesh(pg("MakeupCompacts__base", cyl_geo!(0.028, 0.028, 0.006, 12)), &case_mat);
    bottom.position().y = 0.003;
    compact.add(&bottom);
    if is_open {
        let top = mesh(pg("MakeupCompacts__lidOpen", cyl_geo!(0.028, 0.028, 0.006, 12)), &case_mat);
        top.position().set(0, 0.028, 0.028);
        top.rotation().x = -PI / 2.0;
        compact.add(&top);
        // Mirror faces -Z, out of the lid toward the powder; 0.3 mm proud of the seat.
        let mirror = mesh(circle_geo!(0.021, 12), lambert!({ color: 0xf8f8f8 }));
        mirror.rotation().y = PI;
        mirror.position().set(0, 0.028, 0.0253);
        compact.add(&mirror);
        let powder = mesh(circle_geo!(0.022, 12), lambert!({ color: 0xe0b090 }));
        powder.rotation().x = -PI / 2.0;
        powder.position().y = 0.0065;
        compact.add(&powder);
    } else {
        let top = mesh(pg("MakeupCompacts__lid", cyl_geo!(0.028, 0.028, 0.006, 12)), &case_mat);
        top.position().y = 0.009;
        compact.add(&top);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    compact.position().set(px, 0, pz);
    compact.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compact);
}

// R586 exit-arrow sign: 5% chance per level. Small green directional sign w/ white arrow.
fn place_exit_arrow_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let bg_mat = lambert!({ color: 0x208040 });
    let arrow_mat = lambert!({ color: 0xf8f8f0 });
    let sign = group();
    let mount_y = 2.10;
    let bg = mesh(pg("ExitArrowSigns__panel", box_geo!(0.30, 0.14, 0.010)), &bg_mat);
    bg.position().y = mount_y;
    sign.add(&bg);
    let flip = rnd() < 0.5;
    let dir: f64 = if flip { -1.0 } else { 1.0 };
    let shaft = mesh(pg("ExitArrowSigns__shaft", box_geo!(0.16, 0.028, 0.006)), &arrow_mat);
    shaft.position().set(-0.02 * dir, mount_y, 0.008);
    sign.add(&shaft);
    let head = mesh(pg("ExitArrowSigns__head", cone_geo!(0.036, 0.060, 3)), &arrow_mat);
    head.rotation().z = if dir > 0.0 { -PI / 2.0 } else { PI / 2.0 };
    head.position().set(0.080 * dir, mount_y, 0.010);
    sign.add(&head);
    wall_mount(g, &sign, x, z, dx, dz, 0.006);
    g.world.add(&sign);
}

// R587 shoe box: 5% chance per level. Sometimes lid off/tilted.
fn place_shoe_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let box_colors = [0xc8a878, 0xd8b078, 0xa08858, 0x606060, 0xf0e0d0];
    let box_mat = lambert!({ color: choice(&box_colors) });
    let bx = group();
    let lid_off = rnd() < 0.35;
    let body = mesh(pg("ShoeBoxes__body", box_geo!(0.34, 0.12, 0.20)), &box_mat);
    body.position().y = 0.06;
    bx.add(&body);
    if lid_off {
        let lid = mesh(pg("ShoeBoxes__lid", box_geo!(0.36, 0.020, 0.22)), &box_mat);
        lid.position().set(0.40, 0.010, 0.10); // clear of the box
        lid.rotation().y = 0.4;
        bx.add(&lid);
    } else {
        let lid = mesh(pg("ShoeBoxes__lid", box_geo!(0.36, 0.020, 0.22)), &box_mat);
        lid.position().y = 0.130;
        bx.add(&lid);
    }
    let (px, pz) = jitter(g, x, z, 2.0);
    bx.position().set(px, 0, pz);
    bx.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bx);
}

// R588 crumpled tissue paper: 10% chance per level.
fn place_crumpled_tissues(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let tissue_mat = lambert!({ color: 0xf8f4e8 });
    let tissue = group();
    // Irregular ball of tissue: 3 overlapping cubes rotated randomly.
    for _i in 0..3 {
        let s = 0.020 + rnd() * 0.010;
        let chunk = mesh(pg("CrumpledTissues__chunk", box_geo!(s, s, s)), &tissue_mat);
        chunk.position().set((rnd() - 0.5) * 0.020, s / 2.0 + (rnd() - 0.5) * 0.008, (rnd() - 0.5) * 0.020);
        chunk.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        tissue.add(&chunk);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    tissue.position().set(px, 0, pz);
    g.world.add(&tissue);
}

// R589 dirt clod cluster: 8% chance per level. 3-6 small brown clumps.
fn place_dirt_clods(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let dirt_colors = [0x604030, 0x503020, 0x705040, 0x402010];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let dirt_mat = lambert!({ color: choice(&dirt_colors) });
        let r = 0.010 + rnd() * 0.008;
        let clod = mesh(pg("DirtClods__clod", octa_geo!(r, 0)), &dirt_mat);
        clod.position().set((rnd() - 0.5) * 0.14, r * 0.7, (rnd() - 0.5) * 0.14);
        clod.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        clod.scale().y = 0.6 + rnd() * 0.3;
        cluster.add(&clod);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R590 pebble cluster: 8% chance per level.
fn place_pebbles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let stone_colors = [0x707068, 0x8a8888, 0x605850, 0x504840, 0x9a9080];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let stone_mat = lambert!({ color: choice(&stone_colors) });
        let r = 0.006 + rnd() * 0.008;
        let pebble = mesh(pg("Pebbles__pebble", sphere_geo!(r, 6, 4)), &stone_mat);
        pebble.position().set((rnd() - 0.5) * 0.14, r * 0.7, (rnd() - 0.5) * 0.14);
        pebble.scale().set(1.0 + rnd() * 0.4, 0.5 + rnd() * 0.3, 1.0 + rnd() * 0.4);
        pebble.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        cluster.add(&pebble);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R590 broken glass shards: 5% chance per level. Small angular triangles on floor.
fn place_glass_shards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let glass_mat = lambert!({ color: 0xc0d0d8, transparent: true, opacity: 0.55, side: DoubleSide });
    let cluster = group();
    let count = 4 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        // Small custom triangle.
        let s = 0.015 + rnd() * 0.015;
        let vx = (rnd() - 0.3) * s;
        let vz = s * (0.7 + rnd() * 0.4);
        let verts: Vec<f32> = vec![0.0, 0.0, 0.0, s as f32, 0.0, 0.0, vx as f32, 0.0, vz as f32];
        let geom = tri_geo(verts, vec![0, 1, 2]);
        let shard = mesh(geom, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.13, 0.002 + rnd() * 0.003, (rnd() - 0.5) * 0.13);
        shard.rotation().y = rnd() * PI * 2.0;
        shard.rotation().z = (rnd() - 0.5) * 0.4;
        cluster.add(&shard);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R591 paper airplane: 5% chance per level. Folded dart resting on floor.
fn place_paper_airplanes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let paper_colors = [0xf8f4e8, 0xd0d8f0, 0xf0e0d0, 0xe8f0d8];
    let paper_mat = lambert!({ color: choice(&paper_colors), side: DoubleSide });
    let plane = group();
    // Central fuselage: 2 triangles (top-down V shape).
    let fuse_geom = tri_geo(
        vec![
            // Left wing triangle.
            0.08, 0.005, 0.0, -0.05, 0.005, -0.06, -0.05, 0.005, 0.0, // Right wing triangle.
            0.08, 0.005, 0.0, -0.05, 0.005, 0.06, -0.05, 0.005, 0.0,
        ],
        vec![0, 1, 2, 3, 4, 5],
    );
    let wings = mesh(fuse_geom, &paper_mat);
    plane.add(&wings);
    // Center keel (vertical fold).
    let keel = mesh(plane_geo!(0.12, 0.015), &paper_mat);
    keel.position().set(0.015, 0.010, 0);
    keel.rotation().x = PI / 2.0;
    keel.rotation().z = 0.1;
    plane.add(&keel);
    let (px, pz) = jitter(g, x, z, 2.5);
    plane.position().set(px, 0, pz);
    plane.rotation().y = rnd() * PI * 2.0;
    plane.rotation().z = (rnd() - 0.5) * 0.3; // slight tilt
    g.world.add(&plane);
}

// R591 keyring w/ keys: 6% chance per level. Metal ring + 2-3 keys.
fn place_keyring_with_keys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let ring_mat = lambert!({ color: 0xc0c0c0 });
    let key_colors = [0xd0a040, 0xb08830, 0xa0a0a0, 0x808080];
    let ring = group();
    let r = 0.020;
    let lp = mesh(pg("KeyringWithKeys__ring", torus_geo!(r, 0.003, 4, 12)), &ring_mat);
    lp.rotation().x = PI / 2.0;
    lp.position().y = 0.003;
    ring.add(&lp);
    let key_count = 2 + (rnd() * 2.0).floor() as i32;
    for k in 0..key_count {
        let key_mat = lambert!({ color: choice(&key_colors) });
        let key = group();
        let bow = mesh(pg("KeyringWithKeys__bow", cyl_geo!(0.008, 0.008, 0.002, 8)), &key_mat);
        bow.position().set(0, 0.001, 0);
        key.add(&bow);
        let shaft = mesh(pg("KeyringWithKeys__shaft", box_geo!(0.038, 0.002, 0.005)), &key_mat);
        shaft.position().set(0.023, 0.001, 0);
        key.add(&shaft);
        for t in 0..2 {
            let tooth = mesh(pg("KeyringWithKeys__tooth", box_geo!(0.006, 0.002, 0.003)), &key_mat);
            tooth.position().set(0.030 + t as f64 * 0.008, 0.001, 0.004);
            key.add(&tooth);
        }
        let angle = (k as f64 / key_count as f64) * PI * 0.6 - PI * 0.3;
        key.position().set(angle.cos() * (r + 0.010), 0, angle.sin() * (r + 0.010));
        key.rotation().y = angle;
        ring.add(&key);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    ring.position().set(px, 0, pz);
    ring.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ring);
}

// R592 crushed leaf litter: 8% chance per level. 4-8 small leaf-shape circles.
fn place_leaf_litter(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let leaf_colors = [0x805020, 0xa06030, 0x604020, 0x907040, 0xc08040];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    // R636 track for drift animator: (leaf, base_x, base_z, phase_x, phase_z)
    let mut leaves: Vec<(Obj, f64, f64, f64, f64)> = vec![];
    for _i in 0..count {
        let leaf_mat = lambert!({ color: choice(&leaf_colors), side: DoubleSide });
        // Elongated ellipse: a scaled circle geometry.
        let leaf = mesh(circle_geo!(0.020, 6), &leaf_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.scale().set(1.5, 0.7, 1);
        let base_x = (rnd() - 0.5) * 0.16;
        let base_z = (rnd() - 0.5) * 0.16;
        leaf.position().set(base_x, 0.003 + rnd() * 0.004, base_z);
        leaf.rotation().z = rnd() * PI * 2.0;
        let phase_x = rnd() * PI * 2.0;
        let phase_z = rnd() * PI * 2.0;
        {
            let mut u = leaf.user();
            u.base_x = base_x;
            u.base_z = base_z;
            u.set("phaseX", phase_x);
            u.set("phaseZ", phase_z);
        }
        leaves.push((leaf.clone(), base_x, base_z, phase_x, phase_z));
        cluster.add(&leaf);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
    // R636 subtle wind-drift animator (leaves shift 4-6 mm in wind).
    g.animate(move |_g, _dt, t| {
        for (l, bx, bz, phx, phz) in &leaves {
            let mut p = l.position();
            p.x = bx + (t * 0.3 + phx).sin() * 0.005;
            p.z = bz + (t * 0.28 + phz).cos() * 0.004;
        }
    });
}

// R593 small twigs cluster: 7% chance per level.
fn place_twigs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x503020, 0x604030, 0x402010, 0x705040];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let twig_mat = lambert!({ color: choice(&twig_colors) });
        let len = 0.040 + rnd() * 0.060;
        let rr = 0.0025 + rnd() * 0.0015;
        let twig = mesh(pg("Twigs__twig", cyl_geo!(rr * 0.7, rr, len, 5)), &twig_mat);
        twig.rotation().z = PI / 2.0;
        twig.rotation().y = rnd() * PI * 2.0;
        twig.position().set((rnd() - 0.5) * 0.16, rr, (rnd() - 0.5) * 0.16);
        twig.rotation().x = (rnd() - 0.5) * 0.3;
        cluster.add(&twig);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R593 acorn/nut cluster: 6% chance per level.
fn place_acorns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let nut_colors = [0x8b5a2b, 0x6b3f1f, 0x9b7a3b];
    let cap_mat = lambert!({ color: 0x4a2f1a });
    let cluster = group();
    let count = 2 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let nut_mat = lambert!({ color: choice(&nut_colors) });
        let acorn = group();
        // Body (ellipsoid via scaled sphere).
        let body = mesh(pg("Acorns__nut", sphere_geo!(0.011, 8, 6)), &nut_mat);
        body.scale().y = 1.3;
        body.position().y = 0.012;
        acorn.add(&body);
        // Cap (half-sphere on top).
        let cap = mesh(pg("Acorns__cap", sphere_geo!(0.011, 8, 4, 0, PI * 2.0, 0, PI / 2.0)), &cap_mat);
        cap.position().y = 0.020;
        acorn.add(&cap);
        // Tiny stem.
        let stem = mesh(cyl_geo!(0.001, 0.001, 0.005, 4), &cap_mat);
        stem.position().y = 0.024;
        acorn.add(&stem);
        acorn.position().set((rnd() - 0.5) * 0.13, 0, (rnd() - 0.5) * 0.13);
        acorn.rotation().y = rnd() * PI * 2.0;
        acorn.rotation().z = (rnd() - 0.5) * 0.4;
        cluster.add(&acorn);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R594 pine cone: 6% chance per level. Layered scale ellipsoid.
fn place_pine_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cone_colors = [0x604020, 0x503018, 0x705030, 0x402010];
    let cone_mat = lambert!({ color: choice(&cone_colors) });
    let cone = group();
    let core = mesh(pg("PineCones__core", cyl_geo!(0.012, 0.008, 0.055, 6)), &cone_mat);
    core.rotation().z = PI / 2.0;
    core.position().y = 0.014;
    cone.add(&core);
    // 4 rings of scale bumps around core.
    for ring in 0..4 {
        let ring_radius = 0.014 - ring as f64 * 0.002;
        let ring_x = -0.020 + ring as f64 * 0.012;
        let bumps = 6;
        for b in 0..bumps {
            let a = (b as f64 / bumps as f64) * PI * 2.0 + ring as f64 * 0.4;
            let scale = mesh(pg("PineCones__scale", box_geo!(0.006, 0.005, 0.008)), &cone_mat);
            scale.position().set(ring_x, 0.014 + a.cos() * ring_radius, a.sin() * ring_radius);
            scale.rotation().x = a;
            cone.add(&scale);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cone.position().set(px, 0, pz);
    cone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cone);
}

// R594 fallen bird feather: 7% chance per level.
fn place_feathers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let feather_colors = [0xf8f4e8, 0x606060, 0x203020, 0x704020, 0x202020];
    let feather_mat = lambert!({ color: choice(&feather_colors), side: DoubleSide });
    let shaft_mat = lambert!({ color: 0xe8e0d0 });
    let feather = group();
    let shaft = mesh(pg("Feathers__shaft", cyl_geo!(0.001, 0.0015, 0.10, 4)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().y = 0.002;
    feather.add(&shaft);
    // Vane (elongated ellipse: a scaled circle geometry).
    let vane = mesh(circle_geo!(0.040, 8), &feather_mat);
    vane.rotation().x = -PI / 2.0;
    vane.scale().set(1.2, 0.35, 1);
    vane.position().set(0.005, 0.003, 0);
    feather.add(&vane);
    let (px, pz) = jitter(g, x, z, 2.5);
    feather.position().set(px, 0, pz);
    feather.rotation().y = rnd() * PI * 2.0;
    feather.rotation().z = (rnd() - 0.5) * 0.4; // slight curl
    g.world.add(&feather);
}

// R595 mushroom cluster: 5% chance per level.
fn place_mushrooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cap_colors = [0xa02020, 0xc08040, 0xd0b060, 0x604020, 0xf0f0e0];
    let stem_mat = lambert!({ color: 0xf0e8d0 });
    let cluster = group();
    let count = 2 + (rnd() * 4.0).floor() as i32;
    let cap_color = choice(&cap_colors);
    for _i in 0..count {
        let mushroom = group();
        let s = 0.7 + rnd() * 0.5; // scale variance
        let stem_h = 0.030 * s;
        let stem = mesh(pg("Mushrooms__stem", cyl_geo!(0.005 * s, 0.007 * s, stem_h, 6)), &stem_mat);
        stem.position().y = stem_h / 2.0;
        mushroom.add(&stem);
        let cap_mat = lambert!({ color: cap_color });
        let cap = mesh(pg("Mushrooms__cap", sphere_geo!(0.014 * s, 8, 5, 0, PI * 2.0, 0, PI / 2.0)), &cap_mat);
        cap.position().y = stem_h;
        mushroom.add(&cap);
        mushroom.position().set((rnd() - 0.5) * 0.12, 0, (rnd() - 0.5) * 0.12);
        cluster.add(&mushroom);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R595 dandelion puff: 6% chance per level. Fluffy white sphere w/ stem.
fn place_dandelions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let stem_mat = lambert!({ color: 0x608040 });
    let puff_mat = lambert!({ color: 0xf8f8f0, transparent: true, opacity: 0.7 });
    let dandelion = group();
    let stem_h = 0.06;
    let stem = mesh(pg("Dandelions__stem", cyl_geo!(0.001, 0.001, stem_h, 4)), &stem_mat);
    stem.position().y = stem_h / 2.0;
    dandelion.add(&stem);
    let puff = mesh(sphere_geo!(0.020, 10, 8), &puff_mat);
    puff.position().y = stem_h + 0.014;
    dandelion.add(&puff);
    // 8 tiny outer wisp spikes radiating (small cylinders).
    let wisp_mat = lambert!({ color: 0xf8f8f0 });
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        let wisp = mesh(pg("Dandelions__wisp", cyl_geo!(0.0004, 0.0004, 0.010, 3)), &wisp_mat);
        // Radiate from the puff's centre: achene (-Y) 1 mm inside the puff, pappus (+Y) outward.
        let el = -0.3 + 0.9 * (a * 0.5).sin();
        let w_dir = v3(el.cos() * a.cos(), el.sin(), el.cos() * a.sin());
        wisp.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &w_dir);
        wisp.position().set(0, stem_h + 0.014, 0).add_scaled_vector(&w_dir, 0.024);
        dandelion.add(&wisp);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    dandelion.position().set(px, 0, pz);
    dandelion.rotation().y = rnd() * PI * 2.0;
    let base_z = (rnd() - 0.5) * 0.2;
    dandelion.rotation().z = base_z; // slight lean
    g.world.add(&dandelion);
    // R633 gentle dandelion sway animator.
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        dandelion.rotation().z = base_z + (t * 0.35 + phase).sin() * 0.05;
    });
}

// R596 moss patch: 5% chance per level. Small bumpy green carpet on floor.
fn place_moss_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let moss_colors = [0x407030, 0x506040, 0x304018, 0x607040];
    let patch = group();
    let base_color = choice(&moss_colors);
    let base = mesh(pg("MossPatches__mat", circle_geo!(0.14, 8)), lambert!({ color: base_color }));
    base.rotation().x = -PI / 2.0;
    base.position().y = 0.003;
    patch.add(&base);
    // 12 tiny bumps for tufted texture.
    for _i in 0..12 {
        let bump_mat = lambert!({ color: choice(&moss_colors) });
        let bump_r = 0.008 + rnd() * 0.010;
        let bump = mesh(pg("MossPatches__tuft", sphere_geo!(bump_r, 6, 4)), &bump_mat);
        let a = rnd() * PI * 2.0;
        let r = rnd() * 0.10;
        bump.position().set(a.cos() * r, bump_r * 0.5, a.sin() * r);
        bump.scale().y = 0.5;
        patch.add(&bump);
    }
    let (px, pz) = jitter(g, x, z, 2.2);
    patch.position().set(px, 0, pz);
    patch.rotation().y = rnd() * PI * 2.0;
    g.world.add(&patch);
}

// R596 fern frond: 6% chance per level. Central stem + pairs of leaflets.
fn place_ferns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let stem_mat = lambert!({ color: 0x506030 });
    let leaflet_colors = [0x407030, 0x306028, 0x507838];
    let leaflet_mat = lambert!({ color: choice(&leaflet_colors), side: DoubleSide });
    let fern = group();
    let stem_len = 0.14;
    let stem = mesh(pg("Ferns__stem", cyl_geo!(0.0015, 0.0025, stem_len, 4)), &stem_mat);
    stem.rotation().z = PI / 2.0;
    stem.position().set(0, 0.003, 0);
    fern.add(&stem);
    // Pairs of leaflets along stem, decreasing size toward tip.
    let pairs = 6;
    let mut leaflets: Vec<Obj> = vec![]; // R608 track for sway animator
    for p in 0..pairs {
        let t = p as f64 / (pairs - 1) as f64;
        let stem_x = -stem_len / 2.0 + t * stem_len + 0.010;
        let size = 0.030 * (1.0 - t * 0.5);
        for side in [-1.0f64, 1.0] {
            let leaflet = mesh(circle_geo!(size, 5), &leaflet_mat);
            leaflet.rotation().x = -PI / 2.0;
            let base_z = side * (0.8 - t * 0.2);
            leaflet.rotation().z = base_z;
            leaflet.position().set(stem_x, 0.004, side * size * 0.6);
            leaflet.scale().set(1.4, 0.5, 1);
            leaflet.user().base_z = base_z;
            leaflet.user().phase = rnd() * PI * 2.0 + p as f64 * 0.4;
            leaflets.push(leaflet.clone());
            fern.add(&leaflet);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    fern.position().set(px, 0, pz);
    fern.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fern);
    // R608 gentle sway animator (per-leaflet phase).
    g.animate(move |_g, _dt, t| {
        for l in &leaflets {
            let (bz, ph) = {
                let n = l.r();
                (n.user.base_z, n.user.phase)
            };
            l.rotation().z = bz + (t * 0.4 + ph).sin() * 0.06;
        }
    });
}

// R597 grass blade clump: 6% chance per level.
fn place_grass_clumps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let grass_colors = [0x407030, 0x506028, 0x608040, 0x304018];
    let clump_color = choice(&grass_colors);
    let grass_mat = lambert!({ color: clump_color, side: DoubleSide });
    let clump = group();
    let count = 6 + (rnd() * 6.0).floor() as i32;
    let mut blades: Vec<Obj> = vec![]; // R601 track for sway animator
    for _i in 0..count {
        let h = 0.040 + rnd() * 0.050;
        let blade = mesh(plane_geo!(0.005, h), &grass_mat);
        blade.position().set((rnd() - 0.5) * 0.06, h / 2.0, (rnd() - 0.5) * 0.06);
        blade.rotation().y = rnd() * PI;
        let base_z = (rnd() - 0.5) * 0.3;
        blade.rotation().z = base_z; // slight lean
        blade.user().base_z = base_z;
        blade.user().phase = rnd() * PI * 2.0;
        blades.push(blade.clone());
        clump.add(&blade);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    clump.position().set(px, 0, pz);
    g.world.add(&clump);
    // R601 gentle sway animator (0.5 Hz sinusoid + per-blade phase).
    g.animate(move |_g, _dt, t| {
        for b in &blades {
            let (bz, ph) = {
                let n = b.r();
                (n.user.base_z, n.user.phase)
            };
            b.rotation().z = bz + (t * 0.5 + ph).sin() * 0.08;
        }
    });
}

// R597 butterfly wing: 5% chance per level. Single detached translucent wing.
fn place_butterfly_wings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let wing_colors = [0xd08040, 0x4060c0, 0x8040c0, 0xd0c040, 0x804020];
    let wing_mat = lambert!({ color: choice(&wing_colors), transparent: true, opacity: 0.75, side: DoubleSide });
    let dot_mat = lambert!({ color: 0x101010 });
    let wing = group();
    // Custom teardrop shape via 2 triangles.
    let wing_geom = tri_geo(
        vec![
            // Upper triangle.
            0.0, 0.0, 0.0, 0.05, 0.005, 0.04, 0.02, 0.005, 0.055, // Lower triangle.
            0.0, 0.0, 0.0, 0.05, 0.005, 0.04, 0.045, 0.005, -0.010,
        ],
        vec![0, 1, 2, 3, 4, 5],
    );
    let wing_mesh = mesh(wing_geom, &wing_mat);
    wing.add(&wing_mesh);
    // 2 small dark spots on wing (eyespots).
    for i in 0..2 {
        let dot = mesh(circle_geo!(0.005, 6), &dot_mat);
        dot.rotation().x = -PI / 2.0;
        dot.position().set(0.020 + i as f64 * 0.010, 0.006, 0.020 + i as f64 * 0.008);
        wing.add(&dot);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    wing.position().set(px, 0, pz);
    wing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wing);
}

// R598 snail shell: 5% chance per level. Small spiral shell (stacked descending rings).
fn place_snail_shells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0xc8a888, 0xa08060, 0xd0b090, 0x604020];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let shell = group();
    // 6 descending ring segments simulating spiral.
    let rings = 6;
    for i in 0..rings {
        let t = i as f64 / rings as f64;
        let r = 0.020 * (1.0 - t * 0.85);
        let ring = mesh(pg("SnailShells__ring", torus_geo!(r, r * 0.35, 4, 10)), &shell_mat);
        ring.rotation().x = PI / 2.0;
        // Spiral upward + slight side offset per ring.
        ring.position().set((t * PI * 3.0).cos() * 0.006, 0.006 + i as f64 * 0.007, (t * PI * 3.0).sin() * 0.006);
        shell.add(&ring);
    }
    // Small opening lip (front torus half): the aperture rim, a C in the YZ plane.
    let lip = mesh(pg("SnailShells__lip", torus_geo!(0.018, 0.006, 4, 8, PI)), &shell_mat);
    lip.rotation().set(-0.52, -PI / 2.0, -PI / 2.0);
    lip.scale().set_scalar(0.83);
    lip.position().set(0.006, 0.015, 0.0145);
    shell.add(&lip);
    let (px, pz) = jitter(g, x, z, 2.5);
    shell.position().set(px, 0, pz);
    shell.rotation().y = rnd() * PI * 2.0;
    shell.rotation().z = if rnd() < 0.4 { PI * 0.5 } else { 0.0 }; // sometimes on side
    g.world.add(&shell);
}

// R598 pine needle cluster: 7% chance per level.
fn place_pine_needles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let needle_colors = [0x304018, 0x405020, 0x203010];
    let cluster = group();
    let count = 10 + (rnd() * 10.0).floor() as i32;
    for _i in 0..count {
        let needle_mat = lambert!({ color: choice(&needle_colors) });
        let needle = mesh(pg("PineNeedles__needle", cyl_geo!(0.0007, 0.0005, 0.030, 3)), &needle_mat);
        needle.rotation().z = PI / 2.0;
        needle.rotation().y = rnd() * PI * 2.0;
        needle.position().set((rnd() - 0.5) * 0.14, 0.0008, (rnd() - 0.5) * 0.14);
        cluster.add(&needle);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R599 bird nest: 4% chance per level. Woven twig bowl on floor.
fn place_bird_nests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x604020, 0x503018, 0x705030];
    let nest = group();
    // Outer ring of woven twigs (12 short cylinders in a circle).
    let outer_r = 0.070;
    for i in 0..12 {
        let twig_mat = lambert!({ color: choice(&twig_colors) });
        let twig = mesh(pg("BirdNests__twig", cyl_geo!(0.003, 0.003, 0.055, 4)), &twig_mat);
        let a = (i as f64 / 12.0) * PI * 2.0;
        twig.position().set(a.cos() * outer_r, 0.020, a.sin() * outer_r);
        // Laid tangent to the ring, weaving up/down by the random tilt.
        let tilt = (rnd() - 0.5) * 0.5;
        {
            let mut r = twig.rotation();
            r.set(0, PI / 2.0 - a, PI / 2.0 + tilt);
            r.order = EulerOrder::YZX;
        }
        nest.add(&twig);
    }
    // Inner ring (smaller radius, higher).
    let inner_r = 0.055;
    for i in 0..10 {
        let twig_mat = lambert!({ color: choice(&twig_colors) });
        let twig = mesh(pg("BirdNests__twigThin", cyl_geo!(0.002, 0.002, 0.040, 4)), &twig_mat);
        let a = (i as f64 / 10.0) * PI * 2.0 + 0.2;
        twig.position().set(a.cos() * inner_r, 0.035, a.sin() * inner_r);
        let tilt = (rnd() - 0.5) * 0.3;
        {
            let mut r = twig.rotation();
            r.set(0, PI / 2.0 - a, PI / 2.0 + tilt);
            r.order = EulerOrder::YZX;
        }
        nest.add(&twig);
    }
    // 30% chance 1-2 tiny eggs in the middle.
    if rnd() < 0.30 {
        let egg_mat = lambert!({ color: 0xd8d8c0 });
        let egg_count = 1 + (rnd() * 2.0).floor() as i32;
        for _e in 0..egg_count {
            let egg = mesh(pg("BirdNests__egg", sphere_geo!(0.010, 8, 6)), &egg_mat);
            egg.scale().y = 1.3;
            egg.position().set((rnd() - 0.5) * 0.020, 0.013, (rnd() - 0.5) * 0.020);
            nest.add(&egg);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    nest.position().set(px, 0, pz);
    nest.rotation().y = rnd() * PI * 2.0;
    g.world.add(&nest);
}

// R599 spider web: 5% chance per level. Small cobweb in the wall-ceiling corner.
fn place_spider_webs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let web_mat = lambert!({ color: 0xf0f0f0, transparent: true, opacity: 0.5, side: DoubleSide });
    let web = group();
    let mount_y = WALL_H - 0.10;
    // Triangular web plane: 3 triangles fanning from corner.
    let web_geom = tri_geo(
        vec![
            0.0, 0.0, 0.0, 0.18, -0.15, 0.0, 0.18, 0.0, 0.10, 0.0, 0.0, 0.0, 0.18, -0.15, 0.0, -0.18, -0.15, 0.0, 0.0, 0.0, 0.0, -0.18,
            -0.15, 0.0, -0.18, 0.0, 0.10,
        ],
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
    );
    let web_mesh = mesh(web_geom, &web_mat);
    web_mesh.position().y = mount_y;
    web.add(&web_mesh);
    wall_mount(g, &web, x, z, dx, dz, 0.020);
    g.world.add(&web);
}

// R602 chestnut cluster: 5% chance per level. Spiky husks + smooth brown nuts.
fn place_chestnuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let nut_mat = lambert!({ color: 0x5a3018 });
    let husk_mat = lambert!({ color: 0x808040 });
    let cluster = group();
    let count = 2 + (rnd() * 3.0).floor() as i32;
    for _i in 0..count {
        let chestnut = group();
        let nut = mesh(pg("Chestnuts__nut", sphere_geo!(0.014, 8, 6)), &nut_mat);
        nut.scale().set(1, 0.7, 1);
        nut.position().y = 0.010;
        chestnut.add(&nut);
        let tip = mesh(pg("Chestnuts__tip", cone_geo!(0.004, 0.005, 5)), lambert!({ color: 0xa08060 }));
        tip.position().y = 0.018;
        chestnut.add(&tip);
        // Optional spiky husk (60% chance).
        if rnd() < 0.60 {
            for s in 0..8 {
                let spike = mesh(pg("Chestnuts__spike", cone_geo!(0.0015, 0.008, 4)), &husk_mat);
                let a = (s as f64 / 8.0) * PI * 2.0;
                // Radiate from the nut at two alternating latitudes, apex along the normal.
                let e = 0.25 + 0.3 * (s % 2) as f64;
                let sp = v3(0.014 * e.cos() * a.cos(), 0.010 + 0.0098 * e.sin(), 0.014 * e.cos() * a.sin());
                let mut sn = v3(e.cos() * a.cos() / 0.014, e.sin() / 0.0098, e.cos() * a.sin() / 0.014);
                sn.normalize();
                spike.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &sn);
                spike.position().copy(&sp).add_scaled_vector(&sn, 0.003);
                chestnut.add(&spike);
            }
        }
        chestnut.position().set((rnd() - 0.5) * 0.13, 0, (rnd() - 0.5) * 0.13);
        chestnut.rotation().y = rnd() * PI * 2.0;
        cluster.add(&chestnut);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R602 eyeshadow palette: 7% chance per level. Case w/ colored pans, sometimes open.
fn place_eyeshadow_palettes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let case_mat = lambert!({ color: 0x202020 });
    let shadow_colors = [0xc06040, 0xe0c060, 0x8060a0, 0x40a0c0, 0xd0d0d0, 0x604020, 0xa080a0, 0xf0c090, 0x306030];
    let palette = group();
    let is_open = rnd() < 0.5;
    let base = mesh(pg("EyeshadowPalettes__base", box_geo!(0.080, 0.006, 0.050)), &case_mat);
    base.position().y = 0.003;
    palette.add(&base);
    if is_open {
        let cols = 3;
        let rows = 2;
        for r in 0..rows {
            for c in 0..cols {
                let idx = ((r * cols + c) % shadow_colors.len() as i32) as usize;
                let pan_mat = lambert!({ color: shadow_colors[idx] });
                let pan = mesh(pg("EyeshadowPalettes__pan", box_geo!(0.020, 0.004, 0.014)), &pan_mat);
                pan.position().set(-0.026 + c as f64 * 0.026, 0.008, -0.013 + r as f64 * 0.026);
                palette.add(&pan);
            }
        }
        // Mirror upright (hinged top).
        let mirror = mesh(pg("EyeshadowPalettes__mirror", plane_geo!(0.076, 0.045)), lambert!({ color: 0xe8e8e8 }));
        mirror.position().set(0, 0.028, -0.028);
        mirror.rotation().x = -0.3;
        palette.add(&mirror);
    } else {
        let lid = mesh(pg("EyeshadowPalettes__lid", box_geo!(0.080, 0.006, 0.050)), &case_mat);
        lid.position().y = 0.009;
        palette.add(&lid);
        // Small logo dot on lid.
        let logo = mesh(circle_geo!(0.006, 8), lambert!({ color: 0xd0a040 }));
        logo.rotation().x = -PI / 2.0;
        logo.position().y = 0.0125;
        palette.add(&logo);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    palette.position().set(px, 0, pz);
    palette.rotation().y = rnd() * PI * 2.0;
    g.world.add(&palette);
}

// R603 perfume bottle: 5% chance per level. Glass body + atomizer bulb.
fn place_perfume_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let glass_colors = [0xe0c8a0, 0xd0a0c0, 0xa0c0e0, 0xf0d060, 0xe08080];
    let glass_mat = lambert!({ color: choice(&glass_colors), transparent: true, opacity: 0.75 });
    let cap_mat = lambert!({ color: 0xd0a040 });
    let bulb_mat = lambert!({ color: 0xc02040, transparent: true, opacity: 0.65 });
    let perfume = group();
    let body = mesh(box_geo!(0.040, 0.055, 0.030), &glass_mat);
    body.position().y = 0.028;
    perfume.add(&body);
    let cap = mesh(pg("PerfumeBottles__cap", cyl_geo!(0.012, 0.014, 0.010, 8)), &cap_mat);
    cap.position().y = 0.060;
    perfume.add(&cap);
    // Atomizer bulb (40% chance).
    if rnd() < 0.4 {
        let bulb = mesh(sphere_geo!(0.010, 8, 6), &bulb_mat);
        bulb.position().set(0.030, 0.045, 0);
        perfume.add(&bulb);
        // Thin connector tube from the cap's side down into the bulb.
        let tube = mesh(pg("PerfumeBottles__tube", cyl_geo!(0.001, 0.001, 0.018, 4)), &cap_mat);
        tube.rotation().z = 1.131;
        tube.position().set(0.0205, 0.056, 0);
        perfume.add(&tube);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    perfume.position().set(px, 0, pz);
    perfume.rotation().y = rnd() * PI * 2.0;
    g.world.add(&perfume);
}

// R603 nail polish bottle: 8% chance per level.
fn place_nail_polish(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let polish_colors = [0xc02040, 0xd050a0, 0x804060, 0x40a0c0, 0x808080, 0x202020, 0xe0d040];
    let polish_mat = lambert!({ color: choice(&polish_colors) });
    let cap_mat = lambert!({ color: 0xe0e0e0 });
    let polish = group();
    let body = mesh(pg("NailPolish__body", cyl_geo!(0.014, 0.014, 0.020, 8)), &polish_mat);
    body.position().y = 0.010;
    polish.add(&body);
    let cap = mesh(pg("NailPolish__cap", cyl_geo!(0.012, 0.012, 0.025, 8)), &cap_mat);
    cap.position().y = 0.032;
    polish.add(&cap);
    let (px, pz) = jitter(g, x, z, 2.4);
    polish.position().set(px, 0, pz);
    polish.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped over.
    if rnd() < 0.25 {
        polish.rotation().z = PI / 2.0;
    }
    g.world.add(&polish);
}

// R604 mascara wand: 7% chance per level. Tube + optional separated brush wand.
fn place_mascaras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let tube_mat = lambert!({ color: 0x202020 });
    let bristle_mat = lambert!({ color: 0x101010 });
    let mascara = group();
    let wand_out = rnd() < 0.55;
    let tube = mesh(pg("Mascaras__tube", cyl_geo!(0.007, 0.007, 0.060, 8)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().y = 0.007;
    mascara.add(&tube);
    if wand_out {
        // Cap, wand and brush share one axis (local -X from the cap centre); the set is
        // turned 0.3 rad and tipped 0.166 rad so the cap and the brush rest on the floor.
        let wand_set = group();
        wand_set.position().set(0.055, 0.0101, 0.020);
        wand_set.rotation().set(0, 0.3, 0.166);
        mascara.add(&wand_set);
        let cap = mesh(pg("Mascaras__cap", cyl_geo!(0.009, 0.009, 0.015, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        wand_set.add(&cap);
        // Wand stem: collar 3 mm into the cap's socket.
        let wand = mesh(pg("Mascaras__wand", cyl_geo!(0.0015, 0.0015, 0.030, 4)), &tube_mat);
        wand.rotation().z = PI / 2.0;
        wand.position().set(-0.0195, 0, 0);
        wand_set.add(&wand);
        // Bristle brush, centred on the wand tip.
        let brush = mesh(pg("Mascaras__brush", sphere_geo!(0.006, 6, 5)), &bristle_mat);
        brush.scale().set(1.5, 0.7, 0.7);
        brush.position().set(-0.0345, 0, 0);
        wand_set.add(&brush);
    } else {
        let cap = mesh(pg("Mascaras__cap", cyl_geo!(0.009, 0.009, 0.015, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.038, 0.007, 0);
        mascara.add(&cap);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    mascara.position().set(px, 0, pz);
    mascara.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mascara);
}

// R604 bobby pin cluster: 9% chance per level. 4-8 tiny U-shaped hair pins.
fn place_bobby_pins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.91 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let pin_colors = [0x202020, 0x605040, 0xa08060, 0x904030];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let pin_mat = lambert!({ color: choice(&pin_colors) });
        let pin = group();
        let prong1 = mesh(pg("BobbyPins__prong", cyl_geo!(0.0007, 0.0007, 0.024, 3)), &pin_mat);
        prong1.rotation().z = PI / 2.0;
        prong1.position().set(0, 0.001, -0.002);
        pin.add(&prong1);
        let prong2 = mesh(pg("BobbyPins__wavy", cyl_geo!(0.0007, 0.0007, 0.024, 3)), &pin_mat);
        prong2.rotation().z = PI / 2.0;
        prong2.position().set(0, 0.001, 0.002);
        pin.add(&prong2);
        // Curved end (half-torus), lying flat, bulging towards -X.
        let end = mesh(pg("BobbyPins__bend", torus_geo!(0.002, 0.0007, 3, 6, PI)), &pin_mat);
        end.rotation().x = PI / 2.0;
        end.rotation().z = PI / 2.0;
        end.position().set(-0.012, 0.001, 0);
        pin.add(&end);
        pin.position().set((rnd() - 0.5) * 0.13, 0, (rnd() - 0.5) * 0.13);
        pin.rotation().y = rnd() * PI * 2.0;
        cluster.add(&pin);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R605 cotton pads cluster: 8% chance per level. 3-6 flat circular pads.
fn place_cotton_pads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        // Sometimes tinted (light beige if 'used', white if fresh).
        let used = rnd() < 0.4;
        let pad_mat = lambert!({ color: if used { 0xe8dcbc } else { 0xf8f4e8 } });
        let pad = mesh(pg("CottonPads__pad", cyl_geo!(0.018, 0.018, 0.003, 10)), &pad_mat);
        pad.position().set((rnd() - 0.5) * 0.13, 0.002 + rnd() * 0.003, (rnd() - 0.5) * 0.13);
        pad.rotation().z = (rnd() - 0.5) * 0.4; // slight tilt for stack
        cluster.add(&pad);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// R605 makeup brush: 7% chance per level. Handle + ferrule + bristle head.
fn place_makeup_brushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let handle_colors = [0xd0a040, 0x202020, 0xc0a080, 0x804060, 0xe0d0a0];
    let handle_mat = lambert!({ color: choice(&handle_colors) });
    let ferrule_mat = lambert!({ color: 0xc0c0c0 });
    let bristle_colors = [0xf0e8d8, 0x604030, 0x202020, 0xa08060];
    let bristle_mat = lambert!({ color: choice(&bristle_colors) });
    let brush = group();
    let handle = mesh(pg("MakeupBrushes__handle", cyl_geo!(0.005, 0.008, 0.085, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.030, 0.008, 0);
    brush.add(&handle);
    let ferrule = mesh(pg("MakeupBrushes__ferrule", cyl_geo!(0.009, 0.009, 0.014, 8)), &ferrule_mat);
    ferrule.rotation().z = PI / 2.0;
    ferrule.position().set(0.018, 0.009, 0);
    brush.add(&ferrule);
    // Bristle head (dome or cone depending on type).
    let bristle_type = rnd();
    let bristle_head;
    if bristle_type < 0.5 {
        // Dome (powder brush).
        bristle_head = mesh(pg("MakeupBrushes__dome", sphere_geo!(0.011, 8, 5)), &bristle_mat);
        bristle_head.scale().x = 1.4;
        bristle_head.position().set(0.038, 0.011, 0); // rests on the floor
    } else {
        // Angled cone (blush/contour).
        bristle_head = mesh(pg("MakeupBrushes__cone", cone_geo!(0.010, 0.024, 8)), &bristle_mat);
        bristle_head.rotation().z = -PI / 2.0;
        bristle_head.position().set(0.036, 0.010, 0); // base 1 mm inside the ferrule mouth
    }
    brush.add(&bristle_head);
    let (px, pz) = jitter(g, x, z, 2.4);
    brush.position().set(px, 0, pz);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}

// R606 hair claw clip: 7% chance per level. Two-jaw claw shape on floor.
fn place_hair_claws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let claw_colors = [0x202020, 0xd050a0, 0xa0a0e0, 0xc0a060, 0x408060];
    let claw_mat = lambert!({ color: choice(&claw_colors) });
    let clip = group();
    let bottom = mesh(pg("HairClaws__jaw", box_geo!(0.048, 0.012, 0.024)), &claw_mat);
    bottom.position().y = 0.006;
    clip.add(&bottom);
    let top = mesh(pg("HairClaws__topJaw", box_geo!(0.044, 0.010, 0.022)), &claw_mat);
    top.position().set(0.002, 0.018, 0);
    clip.add(&top);
    // 4-6 comb teeth on bottom jaw front edge.
    let teeth_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..teeth_count {
        let tooth = mesh(pg("HairClaws__tooth", box_geo!(0.003, 0.004, 0.010)), &claw_mat);
        // Long axis along +X, sticking 8 mm out of the jaw's front rim.
        tooth.rotation().y = PI / 2.0;
        tooth.position().set(0.027, 0.008, -0.011 + i as f64 * (0.022 / (teeth_count - 1) as f64));
        clip.add(&tooth);
    }
    // Hinge pin at rear.
    let hinge = mesh(pg("HairClaws__hinge", cyl_geo!(0.003, 0.003, 0.024, 6)), lambert!({ color: 0xc0c0c0 }));
    hinge.rotation().x = PI / 2.0;
    hinge.position().set(-0.020, 0.012, 0);
    clip.add(&hinge);
    let (px, pz) = jitter(g, x, z, 2.4);
    clip.position().set(px, 0, pz);
    clip.rotation().y = rnd() * PI * 2.0;
    g.world.add(&clip);
}

// R606 tweezers: 7% chance per level. Two thin metal arms meeting at a point.
fn place_tweezers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let metal_colors = [0xc0c0c0, 0xa0a0a0, 0xd0b060, 0x606060];
    let tweezer_mat = lambert!({ color: choice(&metal_colors) });
    let tweezers = group();
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("Tweezers__arm", box_geo!(0.055, 0.002, 0.004)), &tweezer_mat);
        arm.position().set(-0.008, 0.002, side * 0.003);
        arm.rotation().y = side * 0.05; // slight taper toward point
        tweezers.add(&arm);
    }
    let base = mesh(pg("Tweezers__base", box_geo!(0.005, 0.003, 0.008)), &tweezer_mat);
    base.position().set(-0.032, 0.002, 0);
    tweezers.add(&base);
    let (px, pz) = jitter(g, x, z, 2.4);
    tweezers.position().set(px, 0, pz);
    tweezers.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tweezers);
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
