//! Prop placers ported from backrooms.js (chunk p28; see docs/PORTING.md).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod tables;
use tables::*;

/// Every placer in this crate, by its JS name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("placeSmallCandles", place_small_candles));
    r.push(("placeVinylRecords", place_vinyl_records));
    r.push(("placeFloorBooks", place_floor_books));
    r.push(("placeDeadPottedPlants", place_dead_potted_plants));
    r.push(("placeTrafficCones", place_traffic_cones));
    r.push(("placeFallenCeilingTiles", place_fallen_ceiling_tiles));
    r.push(("placeWallWaterStreaks", place_wall_water_streaks));
    r.push(("placeFloorDrainGrates", place_floor_drain_grates));
    r.push(("placeRotaryPhones", place_rotary_phones));
    r.push(("placeFloorPuddles", place_floor_puddles));
    r.push(("placeFloorRugs", place_floor_rugs));
    r.push(("placeAshtrays", place_ashtrays));
    r.push(("placeWallCoatHooks", place_wall_coat_hooks));
    r.push(("placeTableLamps", place_table_lamps));
    r.push(("placeOverturnedBuckets", place_overturned_buckets));
    r.push(("placeFloorNewspapers", place_floor_newspapers));
    r.push(("placeCeilingCracks", place_ceiling_cracks));
    r.push(("placeCeilingCobwebs", place_ceiling_cobwebs));
    r.push(("placeDeadCeilingLights", place_dead_ceiling_lights));
    r.push(("placeCeilingCables", place_ceiling_cables));
}

// ------------------------------------------------------------------ helpers

/// `{ x, z, dx, dz }`: an open tile with a wall (or the grid edge) on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallCand { x: i32, z: i32, dx: i32, dz: i32 }

/// The common wall-candidate scan (z, x, [[1,0],[-1,0],[0,1],[0,-1]] order).
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut c = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 { continue; }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    c.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    c
}

/// Every open interior tile (`tiles[tIdx(x, z)] === 0`), z-major.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 { v.push((x, z)); }
        }
    }
    v
}

/// In-place Fisher-Yates exactly as the JS loops write it.
fn shuffle<T>(v: &mut [T]) {
    let n = v.len();
    if n < 2 { return; }
    for i in (1..n).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// three's `object.clone()`. `Object3D.copy` copies the quaternion, and the
/// quaternion's onChange re-derives the Euler angles from it (a yaw past π/2
/// comes back as (±π, π - θ, ±π)), so later `clone.rotation.y = v` writes keep
/// three's semantics. `Obj::duplicate` copies the Euler verbatim, hence this wrapper.
fn clone_obj(o: &Obj) -> Obj {
    let c = o.duplicate();
    c.traverse(&mut |n| { let q = n.quat(); *n.quaternion() = q; });
    c
}

/// `Math.floor(Math.random() * len)`
#[inline]
fn pick(len: usize) -> usize { (rnd() * len as f64).floor() as usize }

/// The shared wall-mount branch (rotation from the wall side, one random
/// draw for the offset along the wall).
fn wall_mount(o: &Obj, c: &WallCand, wall_x: f64, wall_z: f64, y: f64, spread: f64) {
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, y, wall_z + (rnd() - 0.5) * spread);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * spread, y, wall_z);
    }
}

/// A one-triangle BufferGeometry with an index and vertex normals.
fn tri_geo(v: [f64; 9], idx: [u32; 3]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

/// `new THREE.Box3().setFromObject(o)` (precise = false): each geometry's
/// bounding box through its object's world matrix (call after updateMatrixWorld).
fn box_from_object(o: &Obj) -> Box3 {
    let mut b = Box3::empty();
    o.traverse(&mut |c| {
        c.update_world_matrix(false, false);
        if let Some(geo) = c.try_geo() {
            let mut gb = geo.bounding_box();
            gb.apply_matrix4(&c.matrix_world());
            b.union(&gb);
        }
    });
    b
}

// ------------------------------------------------------------------ line 50206
// R384 small candle: 12% chance per level. Small wax cylinder with
// tiny wick + optional dark melt drip on side.
fn place_small_candles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 { return; }   // 12% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let wax_colors = [0xe8d8b0, 0xd8c898, 0xc8b8a0, 0xa08890];
    let wax_mat = lambert!({ color: wax_colors[pick(wax_colors.len())] });
    let wick_mat = lambert!({ color: 0x0a0806 });
    let candle = group();
    // Body: wax cylinder (`body.geometry.parameters.height` == body_h).
    let body_h = 0.08 + rnd() * 0.04;
    let body = mesh(pg("SmallCandles__body", cyl_geo!(0.020, 0.020, body_h, 8)), &wax_mat);
    body.position().y = body_h * 0.5;
    candle.add(&body);
    // Wick: tiny dark line on top.
    let wick = mesh(cyl_geo!(0.001, 0.001, 0.012, 4), &wick_mat);
    wick.position().y = body_h + 0.006;
    candle.add(&wick);
    // R388 25% chance (independent of flame): thin smoke wisp above
    // wick, implying "candle was just blown out".
    let has_wisp = rnd() < 0.25;
    if has_wisp {
        let wisp_mat = basic_mat!({ color: 0x707070, transparent: true, opacity: 0.35, depthWrite: false });
        let wisp = mesh(cyl_geo!(0.005, 0.012, 0.08, 5), &wisp_mat);
        wisp.position().y = body_h + 0.05;
        wisp.rotation().z = (rnd() - 0.5) * 0.3;
        candle.add(&wisp);
    }
    // R386 15% chance: small emissive flame tip with animator flicker.
    // (Skip if wisp already spawned.)
    if !has_wisp && rnd() < 0.15 {
        let flame_mat = basic_mat!({ color: 0xf0c060, transparent: true, opacity: 0.85 });
        let flame = mesh(sphere_geo!(0.008, 6, 5), &flame_mat);
        flame.scale().set(0.7, 1.6, 0.7);   // tall teardrop
        flame.position().y = body_h + 0.018;
        candle.add(&flame);
        // R387 small warm PointLight for actual scene illumination.
        let flame_light = point_light!(0xf0c060, 0.35, 1.6, 2);
        flame_light.position().y = body_h + 0.020;
        candle.add(&flame_light);
        // Flame flicker animator: scale + opacity + light intensity jitter.
        let fl_phase = rnd() * PI * 2.0;
        let fl_period = 0.4 + rnd() * 0.4;   // fast 0.4-0.8 s
        let (flame2, flame_mat2, light2) = (flame.clone(), flame_mat.clone(), flame_light.clone());
        g.animate(move |_g, _dt, t| {
            let wobble = (t * (PI * 2.0 / fl_period) + fl_phase).sin();
            let s = 1.0 + wobble * 0.15;
            flame2.scale().set(0.7 * s, 1.6 * s, 0.7 * s);
            flame_mat2.m().opacity = 0.85 + wobble * 0.10;
            light2.set_intensity(0.35 + wobble * 0.08);
        });
    }
    // 30% chance: dark melt drip on side.
    if rnd() < 0.30 {
        let drip = mesh(pg("SmallCandles__drip", cyl_geo!(0.003, 0.001, 0.03, 4)), &wax_mat);
        drip.rotation().z = PI / 8.0 + (rnd() - 0.5) * 0.3;
        drip.position().set(0.018, body_h * 0.7, 0);
        candle.add(&drip);
    }
    candle.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    candle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&candle);
    // R385 30% chance: wax pool circle stain on floor beneath candle.
    if rnd() < 0.30 {
        let pool_mat = basic_mat!({ color: wax_mat.color_v().get_hex(), transparent: true, opacity: 0.65, depthWrite: false });
        let pool = mesh(circle_geo!(0.04 + rnd() * 0.03, 12), &pool_mat);
        pool.rotation().x = -PI / 2.0;
        pool.rotation().z = rnd() * PI * 2.0;
        pool.scale().set(1.3, 0.85, 1);
        let cp = candle.pos();
        pool.position().set(cp.x, 0.003, cp.z);
        g.world.add(&pool);
    }
    // R385 25% chance: 1-2 sibling candles nearby via clone (shared
    // material; independent positions/rotations). Ritual scene.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _s in 0..sib_count {
            let sib_candle = clone_obj(&candle);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.10 + rnd() * 0.14;
            sib_candle.position().x += s_ang.cos() * s_rad;
            sib_candle.position().z += s_ang.sin() * s_rad;
            sib_candle.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_candle);
        }
    }
}

// ------------------------------------------------------------------ line 50324
// R382 vinyl record: 12% chance per level. Round black disc with
// color label at center.
fn place_vinyl_records(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 { return; }   // 12% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let disc_mat = lambert!({ color: 0x0a0806 });
    let label_colors = [0xa02020, 0x2040a0, 0xa08020, 0x208020, 0x8020a0];
    let label_mat = lambert!({ color: label_colors[pick(label_colors.len())] });
    let record = group();
    // Main disc: 12-inch LP ≈ 0.15 m radius.
    let disc = mesh(pg("VinylRecords__disc", cyl_geo!(0.15, 0.15, 0.003, 20)), &disc_mat);
    disc.position().y = 0.0015;
    record.add(&disc);
    // Center label: small colored disc.
    let label = mesh(pg("VinylRecords__label", cyl_geo!(0.04, 0.04, 0.0032, 12)), &label_mat);
    label.position().y = 0.003;
    record.add(&label);
    // Tiny spindle hole (near-black disc in label center).
    let hole_mat = basic_mat!({ color: 0x040404 });
    let hole = mesh(cyl_geo!(0.005, 0.005, 0.004, 8), &hole_mat);
    hole.position().y = 0.0034;
    record.add(&hole);
    // R384 20% chance: dark crack line across disc surface.
    if rnd() < 0.20 {
        let crack_mat = basic_mat!({ color: 0x1a1410, transparent: true, opacity: 0.9, depthWrite: false });
        let crack = mesh(plane_geo!(0.003, 0.24), &crack_mat);
        crack.rotation().x = -PI / 2.0;
        crack.rotation().z = rnd() * PI;
        crack.position().y = 0.0033;
        record.add(&crack);
    }
    record.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    record.rotation().y = rnd() * PI * 2.0;
    g.world.add(&record);
    // R383 25% chance: album sleeve (square card) 0.15-0.30 m from record.
    if rnd() < 0.25 {
        let sleeve_colors = [0x2a2020, 0x1a2028, 0x282018, 0x1a1a2a];
        let sleeve_mat = lambert!({ color: sleeve_colors[pick(sleeve_colors.len())] });
        let sleeve = mesh(pg("VinylRecords__sleeve", box_geo!(0.32, 0.005, 0.32)), &sleeve_mat);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.15 + rnd() * 0.15;
        let rp = record.pos();
        sleeve.position().set(rp.x + s_ang.cos() * s_rad, 0.0025, rp.z + s_ang.sin() * s_rad);
        sleeve.rotation().y = rnd() * PI * 2.0;
        sleeve.rotation().x = (rnd() - 0.5) * 0.08;
        g.world.add(&sleeve);
    }
}

// ------------------------------------------------------------------ line 50393
// R380 small book on floor: 15% chance per level. Hardcover book
// with visible colored cover.
fn place_floor_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // R381 per-weather cover color pool.
    let cover_pool = book_cover_pool(g.weather_key());
    let cover_mat = lambert!({ color: cover_pool[pick(cover_pool.len())] });
    let pages_mat = lambert!({ color: 0xd8ccb0 });
    let book = group();
    // Cover (top face): 12×18 cm cover.
    let cover = mesh(pg("FloorBooks__cover", box_geo!(0.12, 0.005, 0.18)), &cover_mat);
    cover.position().y = 0.024;
    book.add(&cover);
    // Pages block (main body): slightly narrower cream box.
    let pages = mesh(pg("FloorBooks__pages", box_geo!(0.115, 0.020, 0.175)), &pages_mat);
    pages.position().y = 0.011;
    book.add(&pages);
    // Back cover: identical size to top cover, at bottom.
    let back = mesh(pg("FloorBooks__cover", box_geo!(0.12, 0.005, 0.18)), &cover_mat);
    back.position().y = -0.001;
    book.add(&back);
    book.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    book.rotation().y = rnd() * PI * 2.0;
    // Slight tilt for organic drop.
    book.rotation().x = (rnd() - 0.5) * 0.15;
    g.world.add(&book);
    // R381 25% chance: 1-2 more books stacked on top with slight offset.
    if rnd() < 0.25 {
        let stack_count = 1 + (rnd() * 2.0).floor() as i32;
        for s in 1..=stack_count {
            let sib_book = clone_obj(&book);
            // Give each stacked book its own cover color for variety.
            let sib_color = cover_pool[pick(cover_pool.len())];
            let sib_cover_mat = lambert!({ color: sib_color });
            // Swap cover-color meshes (index 0 top cover, 2 back cover).
            let kids = sib_book.children();
            kids[0].set_mat(&sib_cover_mat);
            kids[2].set_mat(&sib_cover_mat);
            let bp = book.pos();
            let bry = book.rot().y;
            sib_book.position().y = bp.y + s as f64 * 0.028;
            sib_book.position().x += (rnd() - 0.5) * 0.04;
            sib_book.position().z += (rnd() - 0.5) * 0.04;
            sib_book.rotation().y = bry + (rnd() - 0.5) * 0.30;
            sib_book.rotation().x = (rnd() - 0.5) * 0.10;
            g.world.add(&sib_book);
        }
    }
}

// ------------------------------------------------------------------ line 50456
// R379 dead potted plant: 15% chance per level. Small pot with
// 2-3 dark bent stems.
fn place_dead_potted_plants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // R382 per-weather pot color pool.
    let pot_pool = dead_plant_pot_pool(g.weather_key());
    let pot_mat = lambert!({ color: pot_pool[pick(pot_pool.len())] });
    let stem_mat = lambert!({ color: 0x2a1810 });
    let soil_mat = lambert!({ color: 0x1a0e08 });
    let plant = group();
    // Pot: tapered cylinder.
    let pot = mesh(pg("DeadPottedPlants__pot", cyl_geo!(0.10, 0.08, 0.14, 10)), &pot_mat);
    pot.position().y = 0.07;
    plant.add(&pot);
    // Soil: dark disc at pot rim.
    let soil = mesh(pg("DeadPottedPlants__soil", circle_geo!(0.09, 10)), &soil_mat);
    soil.rotation().x = -PI / 2.0;
    soil.position().y = 0.14;
    plant.add(&soil);
    // 2-3 dead bent stems poking up from soil.
    let stem_count = 2 + (rnd() * 2.0).floor() as i32;
    for _s in 0..stem_count {
        let stem = mesh(pg("DeadPottedPlants__stem", cyl_geo!(0.004, 0.004, 0.22, 4)), &stem_mat);
        let bend_ang = (rnd() - 0.5) * 0.7;   // ~40° max bend
        stem.rotation().z = bend_ang;
        stem.rotation().y = rnd() * PI * 2.0;
        let stem_ang = rnd() * PI * 2.0;
        let stem_rad = rnd() * 0.04;
        stem.position().set(stem_ang.cos() * stem_rad, 0.24, stem_ang.sin() * stem_rad);
        plant.add(&stem);
    }
    plant.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    plant.rotation().y = rnd() * PI * 2.0;
    g.world.add(&plant);
}

// ------------------------------------------------------------------ line 50511
// R375 traffic cone: 15% chance per level, 1 cone. Orange truncated
// cone + white reflective band.
fn place_traffic_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // R376 per-weather cone body color.
    let cone_color = traffic_cone_by_weather(g.weather_key());
    let orange_mat = lambert!({ color: cone_color });
    let band_mat = lambert!({ color: 0xf0f0e8 });
    let base_mat = lambert!({ color: 0x181410 });
    let cone = group();
    // Base square: wide flat plate.
    let base = mesh(pg("TrafficCones__base", box_geo!(0.20, 0.015, 0.20)), &base_mat);
    base.position().y = 0.0075;
    cone.add(&base);
    // Cone body: tapered cylinder wide bottom → narrow top.
    let body = mesh(pg("TrafficCones__body", cyl_geo!(0.02, 0.08, 0.30, 10)), &orange_mat);
    body.position().y = 0.15 + 0.015;
    cone.add(&body);
    // White reflective band around middle.
    let band = mesh(pg("TrafficCones__band", cyl_geo!(0.06, 0.055, 0.03, 10)), &band_mat);
    // flipped so it narrows upward like the cone, sized to hug it (r ≈ 0.04
    // here); it stood 1.5-2 cm proud as a loose sleeve
    band.rotation().x = PI;
    band.scale().set(0.654, 0.8, 0.654);
    band.position().y = 0.20 + 0.015;
    cone.add(&band);
    cone.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cone);
    // R377 25% chance: 1-2 sibling cones spaced 0.6-1.0 m in a row.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        let sib_ang = rnd() * PI * 2.0;
        let (cos_a, sin_a) = (sib_ang.cos(), sib_ang.sin());
        let mut sib_positions = vec![cone.pos()];
        for s in 1..=sib_count {
            let sib_cone = clone_obj(&cone);
            let sib_dist = 0.6 + rnd() * 0.4;
            sib_cone.position().x += cos_a * sib_dist * s as f64;
            sib_cone.position().z += sin_a * sib_dist * s as f64;
            sib_cone.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_cone);
            sib_positions.push(sib_cone.pos());
        }
        // R378 safety tape line connecting sibling cones. Thin dark-
        // yellow THREE.Line between consecutive cone tops at ~0.3 m height.
        let tape_mat = line_mat!({ color: 0xd8b040, transparent: true, opacity: 0.75 });
        for s in 0..sib_positions.len() - 1 {
            let tape_geo = Geo::from_points(&[
                v3(sib_positions[s].x, 0.32, sib_positions[s].z),
                v3(sib_positions[s + 1].x, 0.32, sib_positions[s + 1].z),
            ]);
            g.world.add(&line(tape_geo, &tape_mat));
        }
    }
}

// ------------------------------------------------------------------ line 50583
// R374 fallen ceiling tile: 15% chance per level. Small flat
// rectangle on floor representing a dropped acoustic-ceiling tile.
fn place_fallen_ceiling_tiles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let tile_mat = lambert!({ color: 0xd8d0c0 });
    let tile = mesh(pg("FallenCeilingTiles__tile", box_geo!(0.60, 0.02, 0.60)), &tile_mat);
    tile.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0.01, g.wz(z) + (rnd() - 0.5) * 1.6);
    tile.rotation().y = rnd() * PI * 2.0;
    tile.rotation().x = (rnd() - 0.5) * 0.20;
    tile.rotation().z = (rnd() - 0.5) * 0.20;
    g.world.add(&tile);
    // R378 dark rectangle in ceiling directly above tile position.
    let gap_mat = basic_mat!({ color: 0x0a0806 });
    let gap = mesh(plane_geo!(0.55, 0.55), &gap_mat);
    gap.rotation().x = PI / 2.0;   // face down from ceiling
    let tp = tile.pos();
    gap.position().set(tp.x, WALL_H - 0.01, tp.z);
    g.world.add(&gap);
}

// ------------------------------------------------------------------ line 50617
// R368 wall water streak: 20% chance per level, 1-2 streaks.
// Vertical elongated dark plane on wall.
fn place_wall_water_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; }   // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Biome-influenced dark streak color (matches R343 ceiling stain).
        let streak_color = streak_color_by_weather(g.weather_key());
        let streak_mat = basic_mat!({ color: streak_color, transparent: true, opacity: 0.55, depthWrite: false, side: DoubleSide });
        // Long thin vertical plane: 0.06-0.10 m wide × 0.8-1.8 m tall.
        let streak_w = 0.06 + rnd() * 0.04;
        let streak_h = 0.8 + rnd() * 1.0;
        let streak = mesh(plane_geo!(streak_w, streak_h), &streak_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let streak_top = WALL_H - 0.10;   // starts near ceiling
        wall_mount(&streak, &c, wall_x, wall_z, streak_top - streak_h / 2.0, 1.4);
        g.world.add(&streak);
        // R372 25% chance: small dark floor stain at bottom of streak.
        if rnd() < 0.25 {
            let fs_color = streak_color_by_weather(g.weather_key());
            let fs_mat = basic_mat!({ color: fs_color, transparent: true, opacity: 0.55, depthWrite: false });
            let fs = mesh(circle_geo!(0.09 + rnd() * 0.07, 12), &fs_mat);
            fs.rotation().x = -PI / 2.0;
            fs.rotation().z = rnd() * PI * 2.0;
            fs.scale().set(1.3, 0.85, 1);
            // Position just below the streak bottom, nudged out from wall.
            let nudge_x = -c.dx as f64 * 0.14;
            let nudge_z = -c.dz as f64 * 0.14;
            let sp = streak.pos();
            fs.position().set(sp.x + nudge_x, 0.004, sp.z + nudge_z);
            g.world.add(&fs);
        }
        // R370 25% chance: 1-2 sibling streaks close by.
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                let sib_w = 0.04 + rnd() * 0.04;
                let sib_h = 0.5 + rnd() * 0.7;
                let sib_mesh = mesh(plane_geo!(sib_w, sib_h), &streak_mat);
                sib_mesh.rotation().y = streak.rot().y;
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.10 + rnd() * 0.10);
                let sp = streak.pos();
                if c.dx.abs() == 1 {
                    sib_mesh.position().set(sp.x, streak_top - sib_h / 2.0 - rnd() * 0.12, sp.z + sib_off);
                } else {
                    sib_mesh.position().set(sp.x + sib_off, streak_top - sib_h / 2.0 - rnd() * 0.12, sp.z);
                }
                g.world.add(&sib_mesh);
            }
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 50720
// R368 floor drain grate: 15% chance per level, 1 drain. Round
// dark plate with cross-bar detail.
fn place_floor_drain_grates(g: &mut Game, _dist: &[i32]) {
    g.pp.drain_grate_positions = vec![];   // R369 reset per level
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let grate_mat = lambert!({ color: 0x2a2620 });
    let bar_mat = lambert!({ color: 0x1a1610 });
    let drain = group();
    // R371 deep-void interior: very dark disc at floor level inside the drain.
    // R373 haunted-biome variant: faint red glow via MeshBasic red tint.
    let is_haunted_drain = g.weather_key() == "haunted";
    let void_mat = basic_mat!({ color: if is_haunted_drain { 0x200404 } else { 0x040404 } });
    let void_disc = mesh(circle_geo!(0.085, 14), &void_mat);
    void_disc.rotation().x = -PI / 2.0;
    void_disc.position().y = 0.002;
    drain.add(&void_disc);
    // R371 plate now a ring (annulus) so the void disc is visible through the hole.
    let plate = mesh(pg("FloorDrainGrates__plate", ring_geo!(0.085, 0.11, 14)), &grate_mat);
    plate.rotation().x = -PI / 2.0;
    plate.position().y = 0.003;
    drain.add(&plate);
    // Inner rim ring.
    let rim = mesh(pg("FloorDrainGrates__rim", torus_geo!(0.10, 0.006, 4, 14)), &grate_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.006;
    drain.add(&rim);
    // 4 cross bars over the drain opening.
    for b in 0..4 {
        let bar = mesh(pg("FloorDrainGrates__bar", box_geo!(0.18, 0.006, 0.006)), &bar_mat);
        bar.rotation().y = (b as f64 / 4.0) * PI;
        bar.position().y = 0.008;
        drain.add(&bar);
    }
    drain.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    drain.rotation().y = rnd() * PI * 2.0;
    g.world.add(&drain);
    // R369 track drain position for gurgle audio events.
    let dp = drain.pos();
    g.pp.drain_grate_positions.push(v3(dp.x, 0.02, dp.z));
    // R371 15% chance: sibling drain 0.6-1.0 m away — shower-room feel.
    if rnd() < 0.15 {
        let sib_drain = clone_obj(&drain);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.6 + rnd() * 0.4;
        sib_drain.position().set(dp.x + s_ang.cos() * s_rad, 0, dp.z + s_ang.sin() * s_rad);
        sib_drain.rotation().y = rnd() * PI * 2.0;
        g.world.add(&sib_drain);
        let sp = sib_drain.pos();
        g.pp.drain_grate_positions.push(v3(sp.x, 0.02, sp.z));
    }
    // R370 30% chance: shallow 3D puddle overlapping the drain.
    if rnd() < 0.30 {
        let (dp_color, dp_emissive) = puddle_spec_by_weather(g.weather_key());
        let dpuddle_mat = lambert!({ color: dp_color, emissive: dp_emissive, transparent: true, opacity: 0.80 });
        let dp_r = 0.16 + rnd() * 0.10;
        let dpuddle = mesh(sphere_geo!(dp_r, 10, 5, 0, PI * 2.0, 0, PI / 2.0), &dpuddle_mat);
        dpuddle.scale().y = 0.06;
        dpuddle.position().set(dp.x, 0.001, dp.z);
        dpuddle.rotation().y = rnd() * PI * 2.0;
        g.world.add(&dpuddle);
    }
}

// ------------------------------------------------------------------ line 50818
// R364 rotary desk phone: 12% chance per level. Vintage-style
// phone on floor: base box + rotary disc + handset with coiled cord.
fn place_rotary_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.off_hook_rotary_positions = vec![];   // R366 reset per level
    if rnd() < 0.88 { return; }   // 12% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let body_colors = [0x201818, 0x282a20, 0x1a1a20];
    let body_mat = lambert!({ color: body_colors[pick(body_colors.len())] });
    let dial_mat = lambert!({ color: 0xa8a8a0 });
    let cord_mat = lambert!({ color: 0x080604 });
    let phone = group();
    // Base: wide short box.
    let base = mesh(pg("RotaryPhones__base", box_geo!(0.18, 0.05, 0.14)), &body_mat);
    base.position().y = 0.025;
    phone.add(&base);
    // Rotary dial: circular disc on top-front.
    let dial = mesh(pg("RotaryPhones__dial", cyl_geo!(0.045, 0.045, 0.008, 12)), &dial_mat);
    dial.position().set(0, 0.054, 0.02);
    phone.add(&dial);
    // Small darker inner center on dial.
    let dial_center = mesh(pg("RotaryPhones__dialCenter", cyl_geo!(0.010, 0.010, 0.010, 8)), &body_mat);
    dial_center.position().set(0, 0.060, 0.02);
    phone.add(&dial_center);
    // R365 25% chance: handset off-hook — placed on floor beside base.
    let off_hook = rnd() < 0.25;
    let handset = group();
    let bar = mesh(pg("RotaryPhones__handsetBar", box_geo!(0.16, 0.03, 0.03)), &body_mat);
    handset.add(&bar);
    for off in [-0.075, 0.075] {
        let ear = mesh(pg("RotaryPhones__ear", sphere_geo!(0.025, 8, 6)), &body_mat);
        ear.scale().set(1, 0.7, 1);
        ear.position().set(off, 0, 0);
        handset.add(&ear);
    }
    if off_hook {
        // Handset placed on floor 0.15-0.20 m away from base.
        let oh_ang = rnd() * PI * 2.0;
        let oh_rad = 0.15 + rnd() * 0.05;
        handset.position().set(oh_ang.cos() * oh_rad, 0.017, oh_ang.sin() * oh_rad);
        handset.rotation().y = rnd() * PI * 2.0;
        handset.rotation().z = 0.0;
    } else {
        handset.rotation().z = (rnd() - 0.5) * 0.15;
        // lower ear cup rests on its cradle horn (base top 0.05) whatever the tilt
        let hz = handset.rot().z;
        handset.position().set(0, 0.0685 + hz.abs() * 0.075, -0.045);
    }
    phone.add(&handset);
    // Coiled cord: single short cylinder curling from handset to base.
    let cord = mesh(pg("RotaryPhones__cord", cyl_geo!(0.004, 0.004, if off_hook { 0.20 } else { 0.15 }, 4)), &cord_mat);
    if off_hook {
        // lie along the floor from the base toward the handset
        let hp = handset.pos();
        cord.rotation().z = -PI / 2.0;
        cord.rotation().y = -hp.z.atan2(hp.x);
        cord.position().set(hp.x * 0.5, 0.005, hp.z * 0.5);
    } else {
        cord.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        cord.position().set(0.05, 0.06, -0.05);
    }
    phone.add(&cord);
    phone.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
    // R366 track off-hook variants for dial-tone audio.
    if off_hook {
        let pp = phone.pos();
        g.pp.off_hook_rotary_positions.push(v3(pp.x, 0.10, pp.z));
    }
}

// ------------------------------------------------------------------ line 50903
// R362 floor puddle: 15% chance per level (25% in wet biome).
// Shallow scaled sphere with emissive-hint water color.
fn place_floor_puddles(g: &mut Game, _dist: &[i32]) {
    let chance = if g.weather_key() == "wet" { 0.25 } else { 0.15 };
    if rnd() > chance { return; }
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // Water tone slightly emissive so puddle catches ambient bounce.
    let (spec_color, spec_emissive) = puddle_spec_by_weather(g.weather_key());
    let puddle_mat = lambert!({ color: spec_color, emissive: spec_emissive, transparent: true, opacity: 0.85 });
    let puddle_r = 0.16 + rnd() * 0.14;
    let puddle = mesh(sphere_geo!(puddle_r, 12, 6, 0, PI * 2.0, 0, PI / 2.0), &puddle_mat);
    puddle.scale().y = 0.08;   // very shallow bulge
    puddle.scale().z = 1.0 + rnd() * 0.3;   // stretch for organic shape
    puddle.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0.001, g.wz(z) + (rnd() - 0.5) * 2.2);
    puddle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&puddle);
    // R363 25% chance: slow scale-XZ pulse animator suggesting subtle
    // ripples on the puddle surface. Period 4-7 s, ±3% scale.
    if rnd() < 0.25 {
        let base_sx = puddle.scl().x;
        let base_sz = puddle.scl().z;
        let r_phase = rnd() * PI * 2.0;
        let r_period = 4.0 + rnd() * 3.0;
        let p2 = puddle.clone();
        g.animate(move |_g, _dt, t| {
            let s = 1.0 + (t * (PI * 2.0 / r_period) + r_phase).sin() * 0.03;
            let mut sc = p2.scale();
            sc.x = base_sx * s;
            sc.z = base_sz * s;
        });
    }
}

// ------------------------------------------------------------------ line 50981
// R361 floor rug remnant with optional peeled corner, lamp and newspaper.
fn place_floor_rugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // R362 per-weather rug tint.
    let rug_tint = rug_tint_by_weather(g.weather_key());
    let rug_mat = lambert!({ map: rug_texture(), color: rug_tint, side: DoubleSide });
    let rug_w = 0.9 + rnd() * 0.4;
    let rug_d = 0.6 + rnd() * 0.3;
    let rug = mesh(plane_geo!(rug_w, rug_d), &rug_mat);
    rug.rotation().x = -PI / 2.0;
    rug.rotation().z = rnd() * PI * 2.0;
    rug.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0.005, g.wz(z) + (rnd() - 0.5) * 1.6);
    g.world.add(&rug);
    // R367 25% chance: 2nd smaller triangular plane at rug corner tilted
    // upward — reads as peeled-up rug corner.
    if rnd() < 0.25 {
        // Peeled corner triangle (BufferGeometry): quarter of rug near corner.
        let c_size = 0.16 + rnd() * 0.06;
        let peel_geo = tri_geo([
            0.0, 0.0, 0.0,
            c_size, 0.0, 0.0,
            0.0, 0.0, c_size,
        ], [0, 2, 1]);
        let peel = mesh(peel_geo, &rug_mat);
        // Rotate around the "root corner" with a pivot group.
        let peel_pivot = group();
        peel_pivot.add(&peel);
        peel_pivot.rotation().x = -0.5 - rnd() * 0.3;   // lift 28-46°
        // Position at one of the 4 rug corners in local rug space.
        let corner_idx = (rnd() * 4.0).floor() as usize;
        let corner_offsets = [
            (-rug_w / 2.0, -rug_d / 2.0),
            (rug_w / 2.0, -rug_d / 2.0),
            (-rug_w / 2.0, rug_d / 2.0),
            (rug_w / 2.0, rug_d / 2.0),
        ];
        let (cx, cz) = corner_offsets[corner_idx];
        // Rotate offset by rug.rotation.z so peel lands at correct world corner.
        let rrz = rug.rot().z;
        let cos_r = rrz.cos();
        let sin_r = rrz.sin();
        let wx_c = cx * cos_r - cz * sin_r;
        let wz_c = cx * sin_r + cz * cos_r;
        let rp = rug.pos();
        peel_pivot.position().set(rp.x + wx_c, 0.006, rp.z + wz_c);
        peel_pivot.rotation().y = rrz + corner_idx as f64 * PI / 2.0;
        g.world.add(&peel_pivot);
    }
    // R365 25% chance: R359 table lamp placed on/near rug — reading-
    // corner narrative (inline reproduction).
    if rnd() < 0.25 {
        let base_colors = [0x2a2418, 0x1a1a1e, 0x3a2818];
        let base_color = base_colors[pick(base_colors.len())];
        let shade_colors = [0xa89880, 0x807060, 0x504840];
        let shade_color = shade_colors[pick(shade_colors.len())];
        let base_mat = lambert!({ color: base_color });
        let shade_mat = lambert!({ color: shade_color });
        let lamp = group();
        let base = mesh(pg("TableLamps__base", cyl_geo!(0.10, 0.10, 0.03, 12)), &base_mat);
        base.position().y = 0.015;
        lamp.add(&base);
        let stem = mesh(pg("TableLamps__stem", cyl_geo!(0.010, 0.010, 0.32, 6)), &base_mat);
        stem.position().y = 0.19;
        lamp.add(&stem);
        let shade = mesh(pg("TableLamps__shade", cyl_geo!(0.11, 0.08, 0.14, 8)), &shade_mat);
        shade.position().y = 0.42;
        lamp.add(&shade);
        lamp.rotation().z = (rnd() - 0.5) * 0.08;
        // Place near rug edge (±0.35 m from rug center).
        let l_ang = rnd() * PI * 2.0;
        let l_rad = 0.25 + rnd() * 0.15;
        let rp = rug.pos();
        lamp.position().set(rp.x + l_ang.cos() * l_rad, 0, rp.z + l_ang.sin() * l_rad);
        lamp.rotation().y = rnd() * PI * 2.0;
        g.world.add(&lamp);
    }
    // R366 25% chance: R356 newspaper laid on top of rug.
    if rnd() < 0.25 {
        let paper_tint = newspaper_tint_by_weather(g.weather_key());
        let rp_mat = lambert!({ map: newspaper_texture(), color: paper_tint, side: DoubleSide });
        let rpaper = mesh(plane_geo!(0.24, 0.16), &rp_mat);
        rpaper.rotation().x = -PI / 2.0;
        rpaper.rotation().z = rnd() * PI * 2.0;
        // Position near rug center with slight offset.
        let rp = rug.pos();
        rpaper.position().set(
            rp.x + (rnd() - 0.5) * 0.3,
            0.007,   // slightly above rug (0.005) to avoid z-fight
            rp.z + (rnd() - 0.5) * 0.3,
        );
        g.world.add(&rpaper);
    }
}

// ------------------------------------------------------------------ line 51104
// R360 ashtray: 12% chance per level. Small shallow disc with
// 2-3 tiny cigarette butt cylinders on top.
fn place_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 { return; }   // 12% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let dish_mat = lambert!({ color: 0x484038 });
    let butt_mat = lambert!({ color: 0xd8ccb8 });
    let filter_mat = lambert!({ color: 0xa08050 });
    let ashtray = group();
    // Dish: wide shallow cylinder.
    let dish = mesh(pg("Ashtrays__dish", cyl_geo!(0.06, 0.055, 0.02, 10)), &dish_mat);
    dish.position().y = 0.010;
    ashtray.add(&dish);
    // Rim slightly darker interior.
    let rim = mesh(pg("Ashtrays__rim", torus_geo!(0.056, 0.004, 4, 12)), &dish_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.020;
    ashtray.add(&rim);
    // 2-3 tiny butt cylinders + filter caps at random positions.
    let butt_count = 2 + (rnd() * 2.0).floor() as i32;
    for _i in 0..butt_count {
        let b_ang = rnd() * PI * 2.0;
        let b_rad = rnd() * 0.03;
        let butt = mesh(pg("Ashtrays__butt", cyl_geo!(0.005, 0.005, 0.025, 5)), &butt_mat);
        butt.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.4;
        butt.rotation().y = rnd() * PI;
        butt.position().set(b_ang.cos() * b_rad, 0.023, b_ang.sin() * b_rad);
        ashtray.add(&butt);
        // Tiny brown filter cap at one end.
        let filter = mesh(pg("Ashtrays__filter", cyl_geo!(0.005, 0.005, 0.008, 5)), &filter_mat);
        let br = butt.rot();
        filter.rotation().copy(&br);
        // On the butt's -Y end (its axis after Rz then Ry), away from the lit end.
        let (tz, ty) = (br.z - PI / 2.0, br.y);
        let bp = butt.pos();
        filter.position().set(
            bp.x + tz.cos() * ty.cos() * 0.016,
            bp.y + tz.sin() * 0.016,
            bp.z - tz.cos() * ty.sin() * 0.016,
        );
        ashtray.add(&filter);
    }
    ashtray.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    ashtray.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ashtray);
}

// ------------------------------------------------------------------ line 51161
// R360 wall coat hook: 15% chance per level, 1-2 hooks. Small metal
// hook + bent tip protruding from wall, at 1.7-1.9 m.
fn place_wall_coat_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let metal_mat = lambert!({ color: 0x808080 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let hook = group();
        // Wall mount plate.
        let plate = mesh(pg("WallCoatHooks__plate", box_geo!(0.03, 0.04, 0.008)), &metal_mat);
        plate.position().z = 0.004;
        hook.add(&plate);
        // Straight shaft protruding out from wall.
        let shaft = mesh(pg("WallCoatHooks__shaft", cyl_geo!(0.005, 0.005, 0.05, 5)), &metal_mat);
        shaft.rotation().x = PI / 2.0;
        shaft.position().z = 0.028;
        hook.add(&shaft);
        // Bent tip: small angled cylinder at end of shaft.
        let tip = mesh(pg("WallCoatHooks__tip", cyl_geo!(0.005, 0.005, 0.02, 5)), &metal_mat);
        tip.rotation().x = PI / 4.0;   // 45° upward bend: up and out from the shaft's end
        tip.position().set(0, 0.006, 0.055);
        hook.add(&tip);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.004);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.004);
        let hook_height = 1.7 + rnd() * 0.20;
        wall_mount(&hook, &c, wall_x, wall_z, hook_height, 1.4);
        g.world.add(&hook);
        // R364 20% chance: 2-3 additional sibling hooks in a row along
        // the same wall (cloakroom cluster).
        if rnd() < 0.20 {
            let sib_count = 2 + (rnd() * 2.0).floor() as i32;
            let sib_step = 0.14 + rnd() * 0.06;   // spacing per sibling
            let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
            for sib in 1..=sib_count {
                let s_hook = clone_obj(&hook);
                let hp = hook.pos();
                if c.dx.abs() == 1 {
                    s_hook.position().z = hp.z + sib_dir * sib as f64 * sib_step;
                } else {
                    s_hook.position().x = hp.x + sib_dir * sib as f64 * sib_step;
                }
                g.world.add(&s_hook);
            }
        }
        // R361 25% chance: small hanging cloth remnant beneath hook tip.
        if rnd() < 0.25 {
            let cloth_colors = [0x282018, 0x1a2028, 0x2a1010, 0x3a2820];
            let cloth_mat = lambert!({ color: cloth_colors[pick(cloth_colors.len())], side: DoubleSide, transparent: true, opacity: 0.9 });
            let cloth = mesh(plane_geo!(0.14, 0.22 + rnd() * 0.10), &cloth_mat);
            cloth.rotation().y = hook.rot().y;
            // Slight sway rotation for organic hang.
            cloth.rotation().z = (rnd() - 0.5) * 0.15;
            // Position just below and slightly forward of hook tip.
            let fx = -c.dx as f64 * 0.04;
            let fz = -c.dz as f64 * 0.04;
            let hp = hook.pos();
            cloth.position().set(hp.x + fx, hook_height - 0.15, hp.z + fz);
            g.world.add(&cloth);
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 51256
// R359 table lamp: 15% chance per level. 3-piece Group: circular
// base + narrower vertical stem + tapered shade at top. Dark/off.
fn place_table_lamps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let base_colors = [0x2a2418, 0x1a1a1e, 0x3a2818];
    let base_color = base_colors[pick(base_colors.len())];
    let shade_colors = [0xa89880, 0x807060, 0x504840];
    let shade_color = shade_colors[pick(shade_colors.len())];
    let base_mat = lambert!({ color: base_color });
    let shade_mat = lambert!({ color: shade_color });
    let lamp = group();
    // Base: wide short cylinder.
    let base = mesh(pg("TableLamps__base", cyl_geo!(0.10, 0.10, 0.03, 12)), &base_mat);
    base.position().y = 0.015;
    lamp.add(&base);
    // Stem: narrow tall cylinder.
    let stem = mesh(pg("TableLamps__stem", cyl_geo!(0.010, 0.010, 0.32, 6)), &base_mat);
    stem.position().y = 0.19;
    lamp.add(&stem);
    // Shade: tapered cone (wide top → narrower bottom).
    let shade = mesh(pg("TableLamps__shade", cyl_geo!(0.11, 0.08, 0.14, 8)), &shade_mat);
    shade.position().y = 0.42;
    lamp.add(&shade);
    // Slight lean for character.
    lamp.rotation().z = (rnd() - 0.5) * 0.10;
    lamp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lamp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lamp);
}

// ------------------------------------------------------------------ line 51298
// R358 overturned bucket: 15% chance per level. Bucket tipped on
// its side — "someone kicked or knocked over a bucket".
fn place_overturned_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let bucket_colors = [0xd8b820, 0xa03018, 0x707078];   // yellow, red, gray
    let bucket_mat = lambert!({ color: bucket_colors[pick(bucket_colors.len())] });
    let bucket = group();
    // Body (tapered cylinder — same shape as R278 mop bucket).
    let body = mesh(pg("OverturnedBuckets__body", cyl_geo!(0.14, 0.11, 0.22, 10)), &bucket_mat);
    bucket.add(&body);
    // Rim torus at top.
    let rim = mesh(pg("OverturnedBuckets__rim", torus_geo!(0.14, 0.006, 4, 10)), &bucket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.11;
    bucket.add(&rim);
    // Interior dark disc so tipped-open end reads as "hollow".
    let in_mat = basic_mat!({ color: 0x0a0806 });
    let interior = mesh(circle_geo!(0.135, 12), &in_mat);
    interior.rotation().x = -PI / 2.0;
    interior.position().y = 0.109;
    bucket.add(&interior);
    // Lay bucket on its side: rotate 90° around X axis, then random Y yaw.
    bucket.rotation().x = PI / 2.0;
    bucket.position().y = 0.14;   // radius = 0.14, sits on floor
    let parent = group();
    parent.add(&bucket);
    let bucket_yaw = rnd() * PI * 2.0;
    parent.rotation().y = bucket_yaw;
    parent.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    g.world.add(&parent);
    // R359 40% chance: dark water spill spreading FROM the bucket's open end.
    if rnd() < 0.40 {
        let spill_mat = basic_mat!({ color: 0x0e1218, transparent: true, opacity: 0.5, depthWrite: false });
        let spill = mesh(circle_geo!(0.20 + rnd() * 0.10, 12), &spill_mat);
        spill.rotation().x = -PI / 2.0;
        spill.rotation().z = rnd() * PI * 2.0;
        // Elongate spill along bucket-yaw direction (open end).
        spill.scale().set(1.6, 0.85, 1);
        // Position 0.25 m in front of open end.
        let fx = bucket_yaw.sin();
        let fz = bucket_yaw.cos();
        let pp = parent.pos();
        spill.position().set(pp.x + fx * 0.25, 0.004, pp.z + fz * 0.25);
        g.world.add(&spill);
    }
}

// ------------------------------------------------------------------ line 51396
// R356 folded newspaper on floor: 20% chance per level, 1 paper.
fn place_floor_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; }   // 20% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    // R357 per-weather aged tint.
    let paper_tint = newspaper_tint_by_weather(g.weather_key());
    let paper_mat = lambert!({ map: newspaper_texture_2(), color: paper_tint, side: DoubleSide });
    let paper = mesh(plane_geo!(0.24, 0.16), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = rnd() * PI * 2.0;
    paper.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.004, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&paper);
}

// ------------------------------------------------------------------ line 51425
// R347 ceiling cracks: 20% chance per level, 1-2 cracks.
// Thin dark jagged line on ceiling.
fn place_ceiling_cracks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; }   // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let crack_mat = basic_mat!({ color: 0x0e0a08, transparent: true, opacity: 0.75, depthWrite: false, side: DoubleSide });
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        // Build jagged crack as 3-segment thin plane strip.
        let crack_group = group();
        let (mut cur_x, mut cur_z) = (0.0f64, 0.0f64);
        let mut cur_ang = rnd() * PI * 2.0;
        for _s in 0..3 {
            let seg_len = 0.35 + rnd() * 0.25;
            let seg = mesh(plane_geo!(seg_len, 0.012), &crack_mat);
            seg.rotation().x = PI / 2.0;   // lay flat facing down
            seg.rotation().z = cur_ang;
            seg.position().set(cur_x + cur_ang.cos() * seg_len * 0.5, 0, cur_z + cur_ang.sin() * seg_len * 0.5);
            crack_group.add(&seg);
            cur_x += cur_ang.cos() * seg_len;
            cur_z += cur_ang.sin() * seg_len;
            cur_ang += (rnd() - 0.5) * 1.0;   // jagged angle drift
        }
        crack_group.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, WALL_H - 0.005, g.wz(z) + (rnd() - 0.5) * 1.6);
        crack_group.rotation().y = rnd() * PI * 2.0;
        g.world.add(&crack_group);
        // R348 30% chance: 2-4 tiny concrete chunks on floor directly below crack.
        if rnd() < 0.30 {
            // R351 per-weather chunk color.
            let chunk_color = ceiling_debris_by_weather(g.weather_key());
            let chunk_mat = lambert!({ color: chunk_color });
            // R352 10% chance: major-fall cluster (8-12 chunks).
            let is_major = rnd() < 0.10;
            let chunk_count = if is_major {
                8 + (rnd() * 5.0).floor() as i32
            } else {
                2 + (rnd() * 3.0).floor() as i32
            };
            for _ch in 0..chunk_count {
                let chunk = mesh(pg("CeilingCracks__chunk", box_geo!(
                    0.03 + rnd() * 0.04,
                    0.02 + rnd() * 0.02,
                    0.03 + rnd() * 0.04
                )), &chunk_mat);
                let ch_ang = rnd() * PI * 2.0;
                let ch_rad = 0.08 + rnd() * (if is_major { 0.55 } else { 0.25 });
                let ch_sink = ((rnd() - 0.5) * 0.005).abs();
                let cp = crack_group.pos();
                chunk.position().set(cp.x + ch_ang.cos() * ch_rad, 0, cp.z + ch_ang.sin() * ch_rad);
                chunk.rotation().set((rnd() - 0.5) * 0.6, rnd() * PI * 2.0, (rnd() - 0.5) * 0.6);
                // rest the tilted chunk's lowest corner on the floor (settled in ≤ 2.5 mm)
                chunk.update_matrix_world(true);
                let min_y = box_from_object(&chunk).min.y;
                chunk.position().y = -min_y - ch_sink;
                g.world.add(&chunk);
            }
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 51520
// R346 ceiling cobwebs: 25% chance per level, 1-3 cobwebs.
// Small triangular white plane wedged into a ceiling-wall corner.
fn place_ceiling_cobwebs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 { return; }   // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Build a small right-angle triangle via BufferGeometry.
        let size = 0.20 + rnd() * 0.15;
        let geo = tri_geo([
            0.0, 0.0, 0.0,           // corner (at wall+ceiling meet)
            size, 0.0, 0.0,          // extending along wall (Z-ish)
            0.0, -size * 0.9, 0.0,   // hanging down along wall
        ], [0, 1, 2]);
        // R347 per-weather cobweb tint.
        let web_color = cobweb_color_by_weather(g.weather_key());
        let web_mat = basic_mat!({ color: web_color, transparent: true, opacity: 0.4, depthWrite: false, side: DoubleSide });
        let web = mesh(geo, &web_mat);
        // Position at ceiling corner along the wall face.
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        // Orient so triangle sits flush against wall + tucks under ceiling.
        wall_mount(&web, &c, wall_x, wall_z, WALL_H - 0.02, 1.5);
        g.world.add(&web);
        // R351 add 3-4 thin line strands radiating from web corner.
        let strand_count = 3 + (rnd() * 2.0).floor() as i32;
        let strand_mat = line_mat!({ color: web_color, transparent: true, opacity: 0.3 });
        for _sr in 0..strand_count {
            // Random point on hypotenuse (opposite side of right-angle corner).
            let t_param = 0.1 + rnd() * 0.85;
            let strand_end = v3(size * t_param, -size * 0.9 * (1.0 - t_param), 0);
            let strand_geo = Geo::from_points(&[v3(0, 0, 0), strand_end]);
            let strand_line = line(strand_geo, &strand_mat);
            strand_line.rotation().y = web.rot().y;
            strand_line.position().copy(&web.pos());
            g.world.add(&strand_line);
        }
        // R350 20% chance: very slow rotational sway around the web's hanging axis.
        if rnd() < 0.20 {
            let base_rot_y = web.rot().y;
            let sway_phase = rnd() * PI * 2.0;
            let sway_period = 12.0 + rnd() * 8.0;   // 12-20 s (very slow)
            let w2 = web.clone();
            g.animate(move |_g, _dt, t| {
                w2.rotation().y = base_rot_y + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.025;
            });
        }
        // R350 5% chance: tiny dark speckle "insect" caught inside web.
        if rnd() < 0.05 {
            let insect_mat = basic_mat!({ color: 0x080604, transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
            let insect = mesh(plane_geo!(0.012, 0.008), &insect_mat);
            // Center of web triangle roughly at (size/3, -size*0.3, 0).
            insect.rotation().y = web.rot().y;
            insect.rotation().z = rnd() * PI * 2.0;
            let wp = web.pos();
            if c.dx.abs() == 1 {
                insect.position().set(
                    wp.x,
                    wp.y - 0.06 - rnd() * 0.04,
                    wp.z + (if c.dx > 0 { -1.0 } else { 1.0 }) * (size * 0.25 + rnd() * 0.03),
                );
            } else {
                insect.position().set(
                    wp.x + (if c.dz > 0 { -1.0 } else { 1.0 }) * (size * 0.25 + rnd() * 0.03),
                    wp.y - 0.06 - rnd() * 0.04,
                    wp.z,
                );
            }
            g.world.add(&insect);
        }
        // R349 25% chance: 1-2 smaller "sibling" webs in the same corner region.
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                let sib_size = 0.10 + rnd() * 0.10;
                let sib_geo = tri_geo([
                    0.0, 0.0, 0.0,
                    sib_size, 0.0, 0.0,
                    0.0, -sib_size * 0.9, 0.0,
                ], [0, 1, 2]);
                let sib_web = mesh(sib_geo, &web_mat);
                sib_web.rotation().y = web.rot().y;
                // Offset along wall by 0.20-0.40 m and drop slightly.
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.20 + rnd() * 0.20);
                let wp = web.pos();
                if c.dx.abs() == 1 {
                    sib_web.position().set(wp.x, wp.y - rnd() * 0.08, wp.z + sib_off);
                } else {
                    sib_web.position().set(wp.x + sib_off, wp.y - rnd() * 0.08, wp.z);
                }
                g.world.add(&sib_web);
            }
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 51670
// R345 dead ceiling light fixture: 20% chance per level, 1-2 units.
// Small dark disc + tiny protruding bulb-remnant on ceiling.
fn place_dead_ceiling_lights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 { return; }   // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let disc_mat = lambert!({ color: 0x1a1810 });
    let bulb_mat = lambert!({ color: 0x2a2418 });
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        let light = group();
        // Base plate: small disc.
        let plate = mesh(pg("DeadCeilingLights__canopy", circle_geo!(0.11, 12)), &disc_mat);
        plate.rotation().x = PI / 2.0;
        plate.position().y = WALL_H - 0.0054;   // the canopy's 5.4 mm back reaches the ceiling
        light.add(&plate);
        // Small "socket" cylinder protruding down.
        let socket = mesh(pg("DeadCeilingLights__socket", cyl_geo!(0.028, 0.028, 0.03, 6)), &bulb_mat);
        socket.position().y = WALL_H - 0.0204;
        light.add(&socket);
        // Tiny darker "broken bulb neck" sticking further down.
        if rnd() < 0.55 {
            let bulb = mesh(pg("DeadCeilingLights__bulb", cyl_geo!(0.015, 0.020, 0.025, 5)), &bulb_mat);
            bulb.position().y = WALL_H - 0.0454;
            light.add(&bulb);
        }
        light.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
        g.world.add(&light);
        // R353 30% chance: small draped cobweb over the dead-light socket.
        if rnd() < 0.30 {
            let dw_size = 0.12 + rnd() * 0.06;
            let dw_geo = tri_geo([
                0.0, 0.0, 0.0,
                dw_size, 0.0, 0.0,
                0.0, -dw_size * 0.7, 0.0,
            ], [0, 1, 2]);
            let dw_color = cobweb_color_by_weather(g.weather_key());
            let dw_mat = basic_mat!({ color: dw_color, transparent: true, opacity: 0.35, depthWrite: false, side: DoubleSide });
            let dw = mesh(dw_geo, &dw_mat);
            // Position adjacent to the light socket, dangling.
            dw.rotation().y = rnd() * PI * 2.0;
            let lp = light.pos();
            dw.position().set(lp.x + (rnd() - 0.5) * 0.06, WALL_H - 0.03, lp.z + (rnd() - 0.5) * 0.06);
            g.world.add(&dw);
        }
        // R346 25% chance: matching hanging cable co-located 0.35-0.60 m
        // from the dead light — "wire ripped from fixture" narrative.
        if rnd() < 0.25 {
            let cable_col_mat = lambert!({ color: 0x181410 });
            let plug_col_mat = lambert!({ color: 0x080604 });
            let cable_col = group();
            let cable_len = 0.4 + rnd() * 0.5;
            let shaft = mesh(pg("CeilingCables__cable", cyl_geo!(0.005, 0.005, cable_len, 5)), &cable_col_mat);
            let c_tilt = (rnd() - 0.5) * 0.15;
            shaft.rotation().z = c_tilt;
            // tilted about its centre: top end 3 mm into the ceiling
            shaft.position().y = WALL_H + 0.003 - (cable_len / 2.0) * c_tilt.cos();
            cable_col.add(&shaft);
            let plug = mesh(pg("CeilingCables__plug", box_geo!(0.028, 0.038, 0.024)), &plug_col_mat);
            // on the tilted cable's lower end, the cable entering 4 mm into the plug's top
            plug.position().set(c_tilt.sin() * cable_len / 2.0, WALL_H + 0.003 - cable_len * c_tilt.cos() - 0.015, 0);
            cable_col.add(&plug);
            let c_ang = rnd() * PI * 2.0;
            let c_rad = 0.35 + rnd() * 0.25;
            let lp = light.pos();
            cable_col.position().set(lp.x + c_ang.cos() * c_rad, 0, lp.z + c_ang.sin() * c_rad);
            g.world.add(&cable_col);
        }
        i += 1;
    }
}

// ------------------------------------------------------------------ line 51770
// R344 ceiling hanging cable: 15% chance per level, 1 unit.
// Vertical thin cylinder dangling from ceiling + small plug head at bottom.
fn place_ceiling_cables(g: &mut Game, _dist: &[i32]) {
    g.pp.ceiling_cable_plug_positions = vec![];   // R355 reset per level
    if rnd() < 0.85 { return; }   // 15% chance
    let open = open_tiles(g);
    if open.is_empty() { return; }
    let (x, z) = open[pick(open.len())];
    let cable_mat = lambert!({ color: 0x181410 });
    let plug_mat = lambert!({ color: 0x080604 });
    let cable = group();
    // Cable length varies — 0.5-1.2 m, dangling from ceiling.
    let cable_len = 0.5 + rnd() * 0.7;
    let shaft = mesh(pg("CeilingCables__cable", cyl_geo!(0.006, 0.006, cable_len, 5)), &cable_mat);
    // Slight tilt for organic hang.
    let tilt = (rnd() - 0.5) * 0.2;
    shaft.rotation().z = tilt;
    // tilted about its centre: top end 3 mm into the ceiling
    shaft.position().y = WALL_H + 0.003 - (cable_len / 2.0) * tilt.cos();
    cable.add(&shaft);
    // Plug at bottom (small dark box), the cable entering 4 mm into its top.
    let plug = mesh(pg("CeilingCables__plug", box_geo!(0.035, 0.045, 0.028)), &plug_mat);
    plug.position().set(tilt.sin() * cable_len / 2.0, WALL_H + 0.003 - cable_len * tilt.cos() - 0.0185, 0);
    cable.add(&plug);
    cable.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cable.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cable);
    // R355 track cable plug tip world position for spark events.
    let cp = cable.pos();
    g.pp.ceiling_cable_plug_positions.push(v3(cp.x, plug.pos().y, cp.z));
    // R345 30% chance: very slow pendulum sway on the cable group.
    // Small ±0.03 rad amplitude around Z, period 8-14 s.
    if rnd() < 0.30 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let base_rot_z = cable.rot().z;
        let c2 = cable.clone();
        g.animate(move |_g, _dt, t| {
            c2.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.03;
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::testkit::smoke_placers(&list);
    }
}
