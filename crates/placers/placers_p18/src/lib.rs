//! Prop placers: dolls, heraldic crests, hobby horses, skull plaques, bellows
//! cameras, brass sconces, half-wall partitions, inspirational posters, and
//! bathroom props (rubber ducks, faucets, soap bars, potted plants, mirrors).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

// the carved shield of arms and the roe's skull: batch `t19`'s (`t19.rs`)
mod t19;
use t19::{place_heraldic_crests, place_skull_plaques};

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_broken_whole_dolls", place_broken_whole_dolls));
    r.push(("place_heraldic_crests", place_heraldic_crests));
    r.push(("place_hobby_horses", place_hobby_horses));
    r.push(("place_skull_plaques", place_skull_plaques));
    r.push(("place_bellows_cameras", place_bellows_cameras));
    r.push(("place_brass_sconces", place_brass_sconces));
    r.push(("place_half_wall_partitions", place_half_wall_partitions));
    r.push(("place_inspirational_posters", place_inspirational_posters));
    r.push(("place_rubber_ducks", place_rubber_ducks));
    r.push(("place_sink_faucets", place_sink_faucets));
    r.push(("place_soap_bars", place_soap_bars));
    r.push(("place_potted_plants", place_potted_plants));
    r.push(("place_bathroom_mirrors", place_bathroom_mirrors));
}

// ------------------------------------------------------------------ helpers
/// `{ x, z, dx, dz }`: an open tile with a wall (or the grid edge) on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The candidate scan every wall placer repeats.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// Every open tile of the interior, row by row.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// The shared wall-mount tail: face the room, sit `inset` off the wall face,
/// slide along the wall by `(`rnd()` - 0.5) * jitter`.
fn mount_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64, jitter: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let dz = (rnd() - 0.5) * jitter;
        o.position().set(wall_x, center_y, wall_z + dz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let dx = (rnd() - 0.5) * jitter;
        o.position().set(wall_x + dx, center_y, wall_z);
    }
}

/// Place `o` at `a + d * s`.
fn place_along(o: &Obj, a: &Vec3, d: &Vec3, s: f64) {
    let mut p = o.position();
    p.copy(a);
    p.add_scaled_vector(d, s);
}

/// `o.rotation.set(x, y, z, order)`
fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    let mut r = o.rotation();
    r.set(x, y, z);
    r.order = order;
}

// Whole broken porcelain doll on floor (floor): 3% chance per level.
fn place_broken_whole_dolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Porcelain — aged white w/ slight yellow tint.
    let porcelain_colors = [0xe8e0d0, 0xd8d0c0, 0xf0e8d8];
    let porcelain_mat = lambert!({ color: choice(&porcelain_colors) });
    // Dress fabric — 5-colour pool (tea-stained pastels).
    let dress_colors = [0xa06060, 0x6080a0, 0xa0a060, 0x808040, 0x8060a0];
    let dress_mat = lambert!({ color: choice(&dress_colors) });
    let hair_colors = [0xd0a060, 0x604020, 0x8b5a2b, 0x2a1810];
    let hair_mat = lambert!({ color: choice(&hair_colors) });
    let crack_mat = lambert!({ color: 0x2a1810 });
    let bd = group();
    // Body — small oval torso, laid flat.
    let body = mesh(pg("BrokenWholeDolls__torso", sphere_geo!(0.05, 10, 7)), &porcelain_mat);
    body.scale().set(0.85, 1.6, 0.85);
    body.rotation().z = PI / 2.0; // Lay horizontal.
    body.position().set(0, 0.045, 0);
    bd.add(&body);
    // Dress covering torso: narrow bodice toward the head, wide hem toward
    // the legs; flattened so the hem rests on the floor.
    let dress = mesh(pg("BrokenWholeDolls__dress", cyl_geo!(0.055, 0.070, 0.14, 10)), &dress_mat);
    dress.rotation().z = -PI / 2.0;
    dress.scale().x = 0.71;
    dress.position().set(-0.02, 0.0497, 0);
    bd.add(&dress);
    // Dress fringe fold at hem (3 tiny box bumps).
    for i in 0..3 {
        let fringe = mesh(pg("BrokenWholeDolls__ruffle", box_geo!(0.008, 0.015, 0.015)), &dress_mat);
        fringe.position().set(-0.093, 0.045 - 0.02, -0.020 + i as f64 * 0.020); // on the hem's end
        bd.add(&fringe);
    }
    // Head — sphere on top-end of body.
    let head = mesh(pg("BrokenWholeDolls__head", sphere_geo!(0.038, 10, 7)), &porcelain_mat);
    head.position().set(0.080, 0.038, 0);
    bd.add(&head);
    // Hair on top of head (curled ringlets).
    for i in 0..5 {
        let fi = i as f64;
        let curl = mesh(pg("BrokenWholeDolls__curl", sphere_geo!(0.010, 6, 5)), &hair_mat);
        curl.position().set(0.080 + (fi * 1.2).cos() * 0.020, 0.070 + (fi * 0.7).sin() * 0.008, (fi * 1.2).sin() * 0.020);
        bd.add(&curl);
    }
    // 2 painted eye dots, in the eye sockets of the face (+X).
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.002, 5, 4)), lambert!({ color: 0x1a1030 }));
        eye.position().set(0.1091, 0.0409, side * 0.0122);
        bd.add(&eye);
    }
    // Painted mouth — tiny dark bar across the face on the lips.
    let mouth = mesh(box_geo!(0.006, 0.001, 0.001), lambert!({ color: 0x8b1010 }));
    mouth.rotation().y = PI / 2.0;
    mouth.position().set(0.1123, 0.0264, 0);
    bd.add(&mouth);
    // Rosy cheek dots, on the cheeks and facing out of them.
    for side in [-1.0, 1.0] {
        let cheek = mesh(circle_geo!(0.005, 6), lambert!({ color: 0xd08080 }));
        set_rot_order(&cheek, 0.3026, 0.844f64.atan2(side * 0.447), 0.0, EulerOrder::YXZ);
        cheek.position().set(0.1110, 0.0271, side * 0.0164);
        bd.add(&cheek);
    }
    // Legs — 2 cylinders (one might be missing).
    // 0 = left leg, 1 = right leg, 2 = left arm, 3 = right arm
    let missing_limb: i32 = if rnd() < 0.4 { (rnd() * 4.0).floor() as i32 } else { -1 };
    for i in 0..2 {
        if missing_limb == i {
            continue;
        }
        let leg = mesh(pg("BrokenWholeDolls__leg", cyl_geo!(0.014, 0.014, 0.075, 6)), &porcelain_mat);
        leg.rotation().z = PI / 2.0;
        leg.position().set(-0.11, 0.014, -0.020 + i as f64 * 0.040); // out of the hem, lying on the floor
        bd.add(&leg);
        // Tiny shoe at end.
        let shoe = mesh(pg("BrokenWholeDolls__shoe", sphere_geo!(0.010, 6, 5)), lambert!({ color: 0x2a1810 }));
        shoe.scale().set(1.0, 0.6, 1.0);
        shoe.position().set(-0.15, 0.006, -0.020 + i as f64 * 0.040); // on the floor round the ankle
        bd.add(&shoe);
    }
    // Arms — from the shoulder inside the side of the dress down and out to
    // the hand on the floor.
    let arm_dir = |i: i32| -> Vec3 {
        let a = if i == 0 { 0.25f64 } else { 0.6f64 };
        let mut v = v3(-0.912 * a.cos(), -0.411, (if i == 0 { -1.0 } else { 1.0 }) * 0.912 * a.sin());
        v.normalize();
        v
    };
    let arm_shoulder = |i: i32| -> Vec3 { v3(0.035, 0.040, (if i == 0 { -1.0 } else { 1.0 }) * 0.042) };
    for i in 0..2 {
        if missing_limb == i + 2 {
            continue;
        }
        let arm = mesh(pg("BrokenWholeDolls__arm", cyl_geo!(0.010, 0.010, 0.065, 6)), &porcelain_mat);
        let u = arm_dir(i);
        arm.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &u); // wrist (+Y) outward
        place_along(&arm, &arm_shoulder(i), &u, 0.0325);
        bd.add(&arm);
        // Tiny hand sphere, on the wrist, resting on the floor.
        let hand = mesh(pg("BrokenWholeDolls__hand", sphere_geo!(0.010, 6, 5)), &porcelain_mat);
        place_along(&hand, &arm_shoulder(i), &u, 0.073);
        bd.add(&hand);
    }
    // If a limb is missing, place it detached nearby.
    if missing_limb >= 0 {
        let (detached_geo, is_leg) = if missing_limb < 2 {
            (pg("BrokenWholeDolls__leg", cyl_geo!(0.014, 0.014, 0.075, 6)), true)
        } else {
            (pg("BrokenWholeDolls__arm", cyl_geo!(0.010, 0.010, 0.065, 6)), false)
        };
        let detached = mesh(detached_geo, &porcelain_mat);
        let det_angle = rnd() * PI * 2.0;
        let dx = det_angle.cos() * (0.15 + rnd() * 0.10);
        let dz = det_angle.sin() * (0.15 + rnd() * 0.10);
        detached.position().set(dx, if is_leg { 0.014 } else { 0.010 }, dz);
        // Lying flat on the floor; the old tilt draw now only adds to its heading.
        let det_turn = rnd() * 0.8;
        detached.rotation().z = PI / 2.0;
        detached.rotation().y = rnd() * PI + det_turn;
        bd.add(&detached);
        // Dark socket where the limb was.
        let socket = mesh(circle_geo!(0.010, 6), &crack_mat);
        socket.rotation().y = -PI / 2.0;
        if missing_limb == 0 {
            socket.position().set(-0.0915, 0.014, -0.020);
        } else if missing_limb == 1 {
            socket.position().set(-0.0915, 0.014, 0.020);
        } else {
            let ai = missing_limb - 2;
            let u = arm_dir(ai);
            place_along(&socket, &arm_shoulder(ai), &u, if ai == 0 { 0.036 } else { 0.022 });
            set_rot_order(&socket, -u.y.asin(), u.x.atan2(u.z), 0.0, EulerOrder::YXZ);
        }
        bd.add(&socket);
    }
    // 3-4 visible cracks on head/body.
    let crack_count = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..crack_count {
        let crack = mesh(box_geo!(0.030 + rnd() * 0.020, 0.001, 0.001), &crack_mat);
        // Random position on head or body.
        let on_head = rnd() < 0.6;
        if on_head {
            let cx = 0.090 + (rnd() - 0.5) * 0.030;
            let cz = (rnd() - 0.5) * 0.030;
            crack.position().set(cx, 0.040, cz);
        } else {
            let cx = -0.02 + (rnd() - 0.5) * 0.06;
            let cz = (rnd() - 0.5) * 0.04;
            crack.position().set(cx, 0.060, cz);
        }
        crack.rotation().y = rnd() * PI;
        crack.rotation().z = (rnd() - 0.5) * 0.8;
        bd.add(&crack);
    }
    // 40%: tea-stain on dress.
    if rnd() < 0.4 {
        let stain_mat = lambert!({ color: 0x603818, transparent: true, opacity: 0.55 });
        let stain = mesh(circle_geo!(0.020, 6), &stain_mat);
        stain.rotation().x = -PI / 2.0;
        let sx = -0.020 + (rnd() - 0.5) * 0.030;
        let sz = (rnd() - 0.5) * 0.030;
        stain.position().set(sx, 0.0955, sz); // on the dress top
        bd.add(&stain);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bd.position().set(px, 0, pz);
    bd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bd);
}

// Old wooden hobby horse child's toy (floor): 3% chance per level.
fn place_hobby_horses(g: &mut Game, _dist: &[i32]) {
    g.pp.hobby_horse_wheeled_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // 4-colour wood pool.
    let wood_colors = [0x604020, 0x2a1810, 0x8b5a2b, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    // 4-colour mane/hair pool (yarn or horsehair).
    let mane_colors = [0x181818, 0x604020, 0xd0a060, 0x8b5a2b];
    let mane_mat = lambert!({ color: choice(&mane_colors) });
    // Rope reins colour.
    let reins_mat = lambert!({ color: 0xa08050 });
    let hh = group();
    // Pole — long horizontal cylinder laid on floor.
    let pole_len = 0.80;
    let pole = mesh(pg("HobbyHorses__pole", cyl_geo!(0.010, 0.010, pole_len, 6)), &wood_mat);
    pole.rotation().z = PI / 2.0;
    pole.position().set(0, 0.014, 0); // resting on the knob
    hh.add(&pole);
    // Small end cap / handle on trailing end.
    let handle = mesh(pg("HobbyHorses__knob", sphere_geo!(0.014, 6, 5)), &wood_mat);
    handle.position().set(-pole_len / 2.0, 0.014, 0);
    hh.add(&handle);
    // Horse head at leading end — angled up slightly, standing on the back
    // corner of its jaw.
    let head_group = group();
    head_group.position().set(pole_len / 2.0, 0.0412, 0);
    head_group.rotation().z = 0.25; // nose up
    hh.add(&head_group);
    // Main head shape.
    let head_body = mesh(pg("HobbyHorses__head", box_geo!(0.14, 0.08, 0.05)), &wood_mat);
    head_body.position().set(0.06, 0, 0);
    head_group.add(&head_body);
    // Snout/nose.
    let snout = mesh(pg("HobbyHorses__muzzle", box_geo!(0.05, 0.055, 0.04)), &wood_mat);
    snout.position().set(0.14, -0.010, 0);
    head_group.add(&snout);
    // 2 ears.
    for side in [-1.0, 1.0] {
        let ear = mesh(pg("HobbyHorses__ear", cone_geo!(0.008, 0.030, 4)), &wood_mat);
        ear.position().set(0.02, 0.040, side * 0.015);
        ear.rotation().z = -0.3;
        head_group.add(&ear);
    }
    // 2 painted/carved eye dots.
    for side in [-1.0, 1.0] {
        let eye = mesh(pg("TeddyBears__eye", sphere_geo!(0.003, 5, 4)), lambert!({ color: 0x080808 }));
        eye.position().set(0.10, 0.010, side * 0.026);
        head_group.add(&eye);
    }
    // 2 painted nostril dots on snout.
    for side in [-1.0, 1.0] {
        let nostril = mesh(sphere_geo!(0.002, 4, 4), lambert!({ color: 0x1a1010 }));
        nostril.position().set(0.16, -0.015, side * 0.010);
        head_group.add(&nostril);
    }
    // Mane — 8-12 small yarn strands hung from the crest down one side,
    // hanging plumb in the world (-0.25 undoes the head tilt).
    let mane_strand_count = 8 + (rnd() * 5.0).floor() as i32;
    for i in 0..mane_strand_count {
        let strand_len = 0.030 + rnd() * 0.025;
        let strand = mesh(pg("HobbyHorses__strand", cyl_geo!(0.0015, 0.0015, strand_len, 3)), &mane_mat);
        let along = 0.015 + (i as f64 - mane_strand_count as f64 / 2.0) * 0.004 * rnd();
        let side_z = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let hang = -0.25 + (rnd() - 0.5) * 0.5;
        strand.rotation().z = hang;
        strand.position().set(along + (strand_len / 2.0) * hang.sin(), 0.036 - (strand_len / 2.0) * hang.cos(), side_z * 0.0265);
        head_group.add(&strand);
    }
    // Rope reins — from snout side down to the floor.
    for side in [-1.0, 1.0] {
        // Attachment ring on snout side.
        let ring = mesh(pg("HobbyHorses__ring", torus_geo!(0.004, 0.001, 3, 6)), &reins_mat);
        ring.rotation().y = PI / 2.0;
        ring.position().set(0.15, -0.010, side * 0.025);
        head_group.add(&ring);
        // Rein cord from the ring back along the cheek to the floor.
        let rein = mesh(pg("HobbyHorses__rein", cyl_geo!(0.0015, 0.0015, 0.10, 3)), &reins_mat);
        rein.rotation().z = -1.084;
        rein.position().set(0.1058, -0.0334, side * 0.027);
        head_group.add(&rein);
    }
    // 30%: small wooden wheel at trailing end.
    let is_wheeled = rnd() < 0.3;
    if is_wheeled {
        for side in [-1.0, 1.0] {
            let wheel = mesh(pg("HobbyHorses__wheel", cyl_geo!(0.025, 0.025, 0.010, 10)), &wood_mat);
            wheel.rotation().x = PI / 2.0; // axle across the pole (Z)
            wheel.position().set(-pole_len / 2.0 + 0.030, 0.025, side * 0.020);
            hh.add(&wheel);
            // Small axle.
            let axle = mesh(pg("HobbyHorses__axle", cyl_geo!(0.002, 0.002, 0.050, 4)), &wood_mat);
            axle.rotation().x = PI / 2.0;
            axle.position().set(-pole_len / 2.0 + 0.030, 0.025, 0);
            hh.add(&axle);
        }
        // the pole's trailing end rides on the axle
        pole.position().y = 0.025;
        handle.position().y = 0.025;
    }
    // 40%: dust patina on pole.
    if rnd() < 0.4 {
        let dust_mat = lambert!({ color: 0x605030, transparent: true, opacity: 0.35 });
        let dust = mesh(cyl_geo!(0.012, 0.012, 0.15, 6), &dust_mat);
        dust.rotation().z = PI / 2.0;
        dust.position().set(0, pole.pos().y, 0); // round the pole
        hh.add(&dust);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    hh.position().set(px, 0, pz);
    hh.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hh);
    if is_wheeled {
        let p = hh.pos();
        g.pp.hobby_horse_wheeled_positions.push(v3(p.x, 0, p.z));
    }
}

// Victorian bellows camera on tripod (floor): 2.5% chance per level.
fn place_bellows_cameras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x2a1810, 0x8b5a2b, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let bellows_mat = lambert!({ color: 0x181818 });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let bc = group();
    // Tripod — 3 splayed wooden legs.
    let leg_h = 1.15; // Tripod ~1.15 m tall.
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("BellowsCameras__leg", cyl_geo!(0.008, 0.014, leg_h, 5)), &wood_mat);
        // Splay: foot on its rubber foot at r 0.20, top leaning 0.16 rad
        // inward so the three tops meet in the hub.
        let leg_dir = v3(-angle.cos() * 0.16f64.sin(), 0.16f64.cos(), -angle.sin() * 0.16f64.sin());
        leg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &leg_dir);
        leg.position().set(
            angle.cos() * 0.20 + leg_dir.x * leg_h / 2.0,
            0.008 + leg_dir.y * leg_h / 2.0,
            angle.sin() * 0.20 + leg_dir.z * leg_h / 2.0,
        );
        bc.add(&leg);
        // Foot at bottom.
        let foot = mesh(pg("BellowsCameras__foot", cyl_geo!(0.010, 0.012, 0.008, 6)), &bellows_mat);
        foot.position().set(angle.cos() * 0.20, 0.004, angle.sin() * 0.20);
        bc.add(&foot);
        // Metal brace between legs.
        if i < 2 {
            let brace = mesh(pg("BellowsCameras__brace", cyl_geo!(0.002, 0.002, 0.14, 3)), &brass_mat);
            let next_angle = ((i + 1) as f64 / 3.0) * PI * 2.0;
            // At the height where the legs' surfaces are 0.14 apart.
            let brace_r = (0.14 + 0.02) / 3f64.sqrt();
            let brace_y = 0.008 + (0.20 - brace_r) / 0.16f64.tan();
            brace.position().set(
                (angle.cos() + next_angle.cos()) * brace_r / 2.0,
                brace_y,
                (angle.sin() + next_angle.sin()) * brace_r / 2.0,
            );
            brace.look_at(next_angle.cos() * brace_r, brace_y, next_angle.sin() * brace_r);
            brace.rotate_x(PI / 2.0);
            bc.add(&brace);
        }
    }
    // Tripod head hub.
    let hub = mesh(pg("BellowsCameras__hub", sphere_geo!(0.020, 8, 6)), &brass_mat);
    hub.position().y = leg_h;
    bc.add(&hub);
    // Camera body — resting on tripod head.
    let cam_y = leg_h + 0.030;
    // Rear plate (film/plate holder).
    let rear_plate = mesh(pg("BellowsCameras__rearPlate", box_geo!(0.16, 0.14, 0.020)), &wood_mat);
    rear_plate.position().set(0, cam_y, -0.06);
    bc.add(&rear_plate);
    // Ground-glass viewing screen.
    let screen = mesh(plane_geo!(0.10, 0.08), lambert!({ color: 0x2a2a30 }));
    screen.rotation().y = PI;
    screen.position().set(0, cam_y, -0.070);
    bc.add(&screen);
    // Bellows (accordion).
    let bellows_len = 0.14;
    let bellows_body = mesh(pg("BellowsCameras__bellows", cyl_geo!(0.055, 0.055, bellows_len, 8)), &bellows_mat);
    bellows_body.rotation().x = PI / 2.0;
    bellows_body.position().set(0, cam_y, -0.06 + 0.010 + bellows_len / 2.0);
    bc.add(&bellows_body);
    // Accordion ridges — 5 tori, shrinking with the bellows' taper.
    for i in 0..5 {
        let t = i as f64 / 4.0;
        let ridge = mesh(pg("BellowsCameras__ridge", torus_geo!(0.056, 0.003, 3, 12)), &bellows_mat);
        let ridge_s = (0.055 - 0.008 * t) / 0.056;
        ridge.scale().set(ridge_s, ridge_s, 1);
        ridge.position().set(0, cam_y, -0.06 + 0.010 + t * bellows_len);
        bc.add(&ridge);
    }
    // Front lens board.
    let front_board = mesh(pg("BellowsCameras__frontBoard", box_geo!(0.14, 0.12, 0.014)), &wood_mat);
    front_board.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.007);
    bc.add(&front_board);
    // Lens barrel.
    let lens_barrel = mesh(pg("BellowsCameras__lens", cyl_geo!(0.028, 0.028, 0.045, 12)), &brass_mat);
    lens_barrel.rotation().x = PI / 2.0;
    lens_barrel.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.014 + 0.022);
    bc.add(&lens_barrel);
    // Lens glass.
    let lens_glass = mesh(circle_geo!(0.024, 12), lambert!({ color: 0x102030 }));
    lens_glass.position().set(0, cam_y, -0.06 + 0.010 + bellows_len + 0.014 + 0.045);
    bc.add(&lens_glass);
    // Focus knob on lens barrel side.
    let focus_knob = mesh(pg("BellowsCameras__knob", cyl_geo!(0.006, 0.006, 0.008, 6)), &brass_mat);
    focus_knob.position().set(0.028, cam_y, -0.06 + 0.010 + bellows_len + 0.022);
    bc.add(&focus_knob);
    // Top adjustment/leveling knob on camera body.
    let adjust_knob = mesh(pg("BellowsCameras__knob", cyl_geo!(0.006, 0.006, 0.006, 6)), &brass_mat);
    adjust_knob.position().set(0, cam_y + 0.073, -0.06); // on the rear plate top
    bc.add(&adjust_knob);
    // 60%: black hood cloth draped over rear plate.
    if rnd() < 0.6 {
        let cloth_mat = lambert!({ color: 0x0a0a0a });
        let cloth_top = mesh(pg("BellowsCameras__clothTop", box_geo!(0.20, 0.008, 0.12)), &cloth_mat);
        cloth_top.scale().x = 0.88;
        cloth_top.position().set(0, cam_y + 0.074, -0.06);
        bc.add(&cloth_top);
        let cloth_left = mesh(pg("BellowsCameras__clothSide", box_geo!(0.008, 0.20, 0.10)), &cloth_mat);
        cloth_left.position().set(-0.084, cam_y - 0.022, -0.06);
        bc.add(&cloth_left);
        let cloth_right = mesh(pg("BellowsCameras__clothSide", box_geo!(0.008, 0.20, 0.10)), &cloth_mat);
        cloth_right.position().set(0.084, cam_y - 0.022, -0.06);
        bc.add(&cloth_right);
        let cloth_back = mesh(pg("BellowsCameras__clothBack", box_geo!(0.20, 0.15, 0.008)), &cloth_mat);
        cloth_back.scale().x = 0.88;
        cloth_back.position().set(0, cam_y - 0.001, -0.116);
        bc.add(&cloth_back);
    }
    // 30%: magnesium flash tray beside camera (T-shaped rod + shallow pan).
    if rnd() < 0.3 {
        // Rod clamped to tripod leg 0 (the +X leg), rising beside the bellows.
        let flash_rod = mesh(pg("BellowsCameras__flashRod", cyl_geo!(0.003, 0.003, 0.25, 4)), &brass_mat);
        let rod_lo = v3(0.20 - (0.987 - 0.008) * 0.16f64.tan(), 0.987, 0);
        let rod_hi = v3(0.12, cam_y + 0.045, 0);
        {
            let mut p = flash_rod.position();
            p.copy(&rod_lo);
            p.add(&rod_hi);
            p.multiply_scalar(0.5);
        }
        let mut rod_d = rod_hi;
        rod_d.sub(&rod_lo);
        rod_d.normalize();
        flash_rod.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &rod_d);
        flash_rod.scale().y = rod_hi.distance_to(&rod_lo) / 0.25;
        bc.add(&flash_rod);
        // Horizontal T-bar at top.
        let t_bar = mesh(pg("BellowsCameras__tBar", cyl_geo!(0.003, 0.003, 0.08, 4)), &brass_mat);
        t_bar.rotation().z = PI / 2.0;
        t_bar.position().set(0.12, cam_y + 0.045, 0);
        bc.add(&t_bar);
        // Shallow flash pan at one end of T.
        let flash_pan = mesh(pg("BellowsCameras__flashPan", cyl_geo!(0.020, 0.015, 0.008, 8)), &brass_mat);
        flash_pan.position().set(0.16, cam_y + 0.045, 0);
        bc.add(&flash_pan);
        // Small pile of magnesium powder in pan.
        let powder = mesh(pg("BellowsCameras__powder", sphere_geo!(0.008, 6, 5)), lambert!({ color: 0xd0d0d0 }));
        powder.scale().set(1.0, 0.35, 1.0);
        powder.position().set(0.16, cam_y + 0.052, 0);
        bc.add(&powder);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.0;
    bc.position().set(px, 0, pz);
    bc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bc);
}

/// Stretch a Y-axis segment between two points (`arm_span`).
fn arm_span(m: &Obj, a: &Vec3, b: &Vec3, len: f64) {
    {
        let mut p = m.position();
        p.copy(a);
        p.add(b);
        p.multiply_scalar(0.5);
    }
    let mut d = *b;
    d.sub(a);
    d.normalize();
    m.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &d);
    m.scale().y = a.distance_to(b) / len;
}

// Wall-mounted brass sconce w/ candle (wall): 4% chance per level.
fn place_brass_sconces(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let candle_colors = [0xf0e8c8, 0xd0d0d0, 0xe8d0c0];
    let candle_mat = lambert!({ color: choice(&candle_colors) });
    let sc = group();
    // Wall mounting plate.
    let mount_plate = mesh(pg("BrassSconces__plate", box_geo!(0.045, 0.070, 0.006)), &brass_mat);
    mount_plate.position().set(0, 0, 0.003);
    sc.add(&mount_plate);
    // 2 small mounting screws.
    for side in [-1.0, 1.0] {
        let screw = mesh(pg("BrassSconces__screw", cyl_geo!(0.002, 0.002, 0.003, 4)), &brass_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(0, side * 0.028, 0.006);
        sc.add(&screw);
    }
    // Ornate scrolled arm: plate boss -> knop -> under the drip cup.
    let arm_a = v3(0, -0.004, 0.003);
    let arm_k = v3(0.030, 0.015, 0.062);
    let arm_c = v3(0.056, 0.078, 0.090);
    let arm_seg1 = mesh(pg("BrassSconces__arm", cyl_geo!(0.006, 0.006, 0.045, 5)), &brass_mat);
    arm_span(&arm_seg1, &arm_a, &arm_k, 0.045);
    sc.add(&arm_seg1);
    // Small mid decorative sphere.
    let arm_mid = mesh(pg("BrassSconces__knop", sphere_geo!(0.010, 6, 5)), &brass_mat);
    arm_mid.position().copy(&arm_k);
    sc.add(&arm_mid);
    // Upper arm segment.
    let arm_seg2 = mesh(pg("BrassSconces__arm", cyl_geo!(0.005, 0.005, 0.045, 5)), &brass_mat);
    arm_span(&arm_seg2, &arm_k, &arm_c, 0.045);
    sc.add(&arm_seg2);
    // Drip cup.
    let drip_cup = mesh(pg("BrassSconces__dripCup", cyl_geo!(0.025, 0.020, 0.010, 12)), &brass_mat);
    drip_cup.position().set(0.056, 0.080, 0.090);
    sc.add(&drip_cup);
    let drip_rim = mesh(pg("BrassSconces__dripRim", torus_geo!(0.023, 0.002, 3, 12)), &brass_mat);
    drip_rim.rotation().x = PI / 2.0;
    drip_rim.position().set(0.056, 0.086, 0.090); // on the cup's lip
    sc.add(&drip_rim);
    // Candle sitting in drip cup (foot 2 mm down in the socket).
    let candle_h = 0.10 + rnd() * 0.04;
    let candle = mesh(pg("BrassSconces__candle", cyl_geo!(0.012, 0.013, candle_h, 8)), &candle_mat);
    candle.position().set(0.056, 0.083 + candle_h / 2.0, 0.090);
    sc.add(&candle);
    // Wick.
    let wick = mesh(pg("BrassSconces__wick", cyl_geo!(0.0008, 0.0008, 0.004, 3)), lambert!({ color: 0x101010 }));
    wick.position().set(0.056, 0.083 + candle_h + 0.002, 0.090);
    sc.add(&wick);
    // 55%: candle is lit — flame + tiny PointLight + flicker animator.
    if rnd() < 0.55 {
        let flame_mat = basic_mat!({ color: 0xffa040 });
        let flame = mesh(sphere_geo!(0.006, 6, 5), &flame_mat);
        flame.scale().y = 2.0;
        flame.position().set(0.056, 0.083 + candle_h + 0.010, 0.090);
        sc.add(&flame);
        let flame_light = point_light!(0xffa040, 0.30, 1.2);
        flame_light.position().set(0.056, 0.083 + candle_h + 0.012, 0.090);
        sc.add(&flame_light);
        let phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let flick = 0.85 + (t * 7.0 + phase).sin() * 0.11 + (t * 11.0 + phase * 1.3).sin() * 0.06;
            flame_light.set_intensity(0.30 * flick);
            flame.scale().set(flick, 2.0 * flick, flick);
        });
        // Small wax drip melting down.
        if rnd() < 0.5 {
            let wax = mesh(pg("BrassSconces__wax", cyl_geo!(0.001, 0.002, 0.014, 4)), &candle_mat);
            wax.position().set(0.078, 0.078, 0.090);
            sc.add(&wax);
        }
    }
    // 30%: small chain hanging from the knop.
    if rnd() < 0.3 {
        for i in 0..3 {
            let chain_link = mesh(pg("BrassSconces__link", torus_geo!(0.004, 0.001, 3, 6)), &brass_mat);
            // Upright links, each turned 90° to the last.
            chain_link.rotation().y = if i % 2 == 0 { PI / 2.0 } else { 0.0 };
            chain_link.position().set(arm_k.x, arm_k.y - 0.012 - i as f64 * 0.006, arm_k.z);
            sc.add(&chain_link);
        }
    }
    mount_on_wall(g, &sc, &c, 0.003, 1.55, 0.5);
    g.world.add(&sc);
}

// Half-height wall partition zone (floor scenery): 25% chance per level.
// Purely decorative — no collision blocking.
fn place_half_wall_partitions(g: &mut Game, _dist: &[i32]) {
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

// Inspirational poster texture.
game::lazy_static_tex!(
    poster_texture,
    canvas_tex(256, 320, |ctx, w, h| {
        // Sky gradient background.
        let grad = ctx.create_linear_gradient(0, 0, 0, h * 0.65);
        grad.add_color_stop(0, "#80c0e0");
        grad.add_color_stop(1, "#f0e0a0");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Mountain silhouettes.
        ctx.set_fill_style("#403050");
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w * 0.2, h * 0.30);
        ctx.line_to(w * 0.35, h * 0.45);
        ctx.line_to(w * 0.55, h * 0.25);
        ctx.line_to(w * 0.75, h * 0.40);
        ctx.line_to(w * 0.9, h * 0.30);
        ctx.line_to(w, h * 0.5);
        ctx.line_to(w, h * 0.65);
        ctx.line_to(0, h * 0.65);
        ctx.close_path();
        ctx.fill();
        // Text area (dark bar at bottom).
        ctx.set_fill_style("#181410");
        ctx.fill_rect(0, h * 0.65, w, h * 0.35);
        // Motivational word.
        ctx.set_fill_style("#f0ece0");
        ctx.set_font("bold 42px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let words = ["BELIEVE", "DREAM", "CONQUER", "IMAGINE", "STRIVE"];
        let word = words[(rnd() * words.len() as f64).floor() as usize];
        ctx.fill_text(word, w / 2.0, h * 0.78);
        // Subtitle.
        ctx.set_font("italic 14px serif");
        ctx.fill_text("The only limit is you.", w / 2.0, h * 0.92);
    })
);

fn place_inspirational_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x181410 });
    let poster_mat = lambert!({ map: poster_texture(), side: DoubleSide });
    let poster = group();
    let (w, h, t, fw) = (0.32, 0.40, 0.010, 0.014);
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("InspirationalPosters__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        poster.add(&bar);
    }
    let canvas = mesh(plane_geo!(w, h), &poster_mat);
    canvas.position().set(0, 0, t / 2.0 + 0.001);
    poster.add(&canvas);
    mount_on_wall(g, &poster, &c, 0.007, 1.65, 0.5);
    g.world.add(&poster);
}

// Rubber duck: 10% chance per level.
fn place_rubber_ducks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_mat = lambert!({ color: 0xf0d020 });
    let beak_mat = lambert!({ color: 0xe07020 });
    let eye_mat = basic_mat!({ color: 0x181410 });
    let duck = group();
    // Body (round).
    let body = mesh(pg("RubberDucks__body", sphere_geo!(0.030, 10, 8)), &body_mat);
    body.scale().set(1.2, 0.9, 1);
    body.position().y = 0.024;
    duck.add(&body);
    // Head.
    let head = mesh(pg("RubberDucks__head", sphere_geo!(0.020, 10, 8)), &body_mat);
    head.position().set(0.024, 0.048, 0);
    duck.add(&head);
    // Beak.
    let beak = mesh(pg("RubberDucks__bill", box_geo!(0.014, 0.006, 0.010)), &beak_mat);
    beak.position().set(0.048, 0.045, 0); // root 3 mm in the head, the rest out
    duck.add(&beak);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.002, 5, 4), &eye_mat);
        eye.position().set(0.0357, 0.0539, s * 0.0144); // half out of the head surface
        duck.add(&eye);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    duck.position().set(px, 0, pz);
    duck.rotation().y = rnd() * PI * 2.0;
    g.world.add(&duck);
}

// Wall sink faucet: batch `t04`'s (`t04.rs`)
mod t04;
use t04::place_sink_faucets;

// Soap bar: 10% chance per level.
fn place_soap_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let soap_colors = [0xf0d0d0, 0xd0e8d0, 0xf0e0a0, 0xe0d8f0];
    let soap_mat = lambert!({ color: choice(&soap_colors) });
    let dish_mat = lambert!({ color: 0xe8e0c8 });
    let soap = group();
    // Bar (rounded box, worn).
    let bar = mesh(pg("SoapBars__bar", box_geo!(0.07, 0.020, 0.04)), &soap_mat);
    bar.position().y = 0.010;
    soap.add(&bar);
    // 40%: sitting in dish tray.
    if rnd() < 0.40 {
        let dish = mesh(pg("SoapBars__dish", box_geo!(0.09, 0.008, 0.06)), &dish_mat);
        dish.position().y = 0.004;
        soap.add(&dish);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    soap.position().set(px, 0, pz);
    soap.rotation().y = rnd() * PI * 2.0;
    g.world.add(&soap);
}

// Potted plant: 5% chance per level.
fn place_potted_plants(g: &mut Game, _dist: &[i32]) {
    g.pp.plant_positions.clear(); // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pot_colors = [0xa05030, 0x604020, 0xe0e0d0, 0x30302a];
    let pot_mat = lambert!({ color: choice(&pot_colors) });
    let dirt_mat = lambert!({ color: 0x3a2818 });
    let leaf_colors = [0x408040, 0x306030, 0x506050, 0x8a5030]; // last = withered
    let leaf_mat = lambert!({ color: choice(&leaf_colors), side: DoubleSide });
    let plant = group();
    // Pot (tapered cylinder).
    let pot = mesh(pg("PottedPlants__pot", cyl_geo!(0.055, 0.045, 0.08, 10)), &pot_mat);
    pot.position().y = 0.04;
    plant.add(&pot);
    // Dirt disc.
    let dirt = mesh(circle_geo!(0.048, 10), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().y = 0.081;
    plant.add(&dirt);
    // 5-8 leaves radiating up + out.
    let n_leaves = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_leaves {
        let a = (i as f64 / n_leaves as f64) * PI * 2.0 + rnd() * 0.2;
        let leaf_h = 0.10 + rnd() * 0.08;
        // Custom triangle leaf.
        let leaf_geo = Geo::new();
        let verts: Vec<f32> =
            vec![0.0, 0.0, 0.0, 0.015, (leaf_h * 0.4) as f32, 0.0, 0.0, leaf_h as f32, 0.0, -0.015, (leaf_h * 0.4) as f32, 0.0];
        leaf_geo.set_position(verts);
        leaf_geo.set_index(vec![0, 1, 2, 0, 2, 3]);
        leaf_geo.compute_vertex_normals();
        let leaf = mesh(leaf_geo, &leaf_mat);
        leaf.position().set(a.cos() * 0.02, 0.082, a.sin() * 0.02);
        leaf.rotation().y = -a + PI / 2.0;
        leaf.rotation().z = 0.3 + rnd() * 0.4;
        plant.add(&leaf);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    plant.position().set(px, 0, pz);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
    // Track for rustle audio.
    let p = plant.pos();
    g.pp.plant_positions.push(v3(p.x, 0.15, p.z));
}

// Bathroom mirror: 5% chance per level.
fn place_bathroom_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0xc0c8d0 });
    let mirror_mat = basic_mat!({ color: 0x506878 });
    let mirror = group();
    let (w, h, t, fw) = (0.30, 0.40, 0.012, 0.014);
    // Frame.
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("BathroomMirrors__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        mirror.add(&bar);
    }
    // Mirror face.
    let face = mesh(plane_geo!(w, h), &mirror_mat);
    face.position().set(0, 0, t / 2.0 + 0.001);
    mirror.add(&face);
    mount_on_wall(g, &mirror, &c, 0.008, 1.55, 0.6);
    g.world.add(&mirror);
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
