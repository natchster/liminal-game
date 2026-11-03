//! Prop placers: candles, records, books, dead plants, traffic cones, fallen
//! ceiling tiles, water streaks, drains, rotary phones, puddles, rugs, ashtrays,
//! coat hooks, table lamps, buckets, newspapers, and ceiling damage (cracks,
//! cobwebs, dead lights, hanging cables).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod tables;
use tables::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_small_candles", place_small_candles));
    r.push(("place_vinyl_records", place_vinyl_records));
    r.push(("place_floor_books", place_floor_books));
    r.push(("place_dead_potted_plants", place_dead_potted_plants));
    r.push(("place_traffic_cones", place_traffic_cones));
    r.push(("place_fallen_ceiling_tiles", place_fallen_ceiling_tiles));
    r.push(("place_wall_water_streaks", place_wall_water_streaks));
    r.push(("place_floor_drain_grates", place_floor_drain_grates));
    r.push(("place_rotary_phones", place_rotary_phones));
    r.push(("place_floor_puddles", place_floor_puddles));
    r.push(("place_floor_rugs", place_floor_rugs));
    r.push(("place_ashtrays", place_ashtrays));
    r.push(("place_wall_coat_hooks", place_wall_coat_hooks));
    r.push(("place_table_lamps", place_table_lamps));
    r.push(("place_overturned_buckets", place_overturned_buckets));
    r.push(("place_floor_newspapers", place_floor_newspapers));
    r.push(("place_ceiling_cracks", place_ceiling_cracks));
    r.push(("place_ceiling_cobwebs", place_ceiling_cobwebs));
    r.push(("place_dead_ceiling_lights", place_dead_ceiling_lights));
    r.push(("place_ceiling_cables", place_ceiling_cables));
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

/// The common wall-candidate scan (z, x, [[1,0],[-1,0],[0,1],[0,-1]] order).
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut c = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
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

/// Every open interior tile (`tiles[t_idx(x, z)] == 0`), z-major.
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

/// In-place Fisher-Yates exactly loops write it.
fn shuffle<T>(v: &mut [T]) {
    let n = v.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// A deep clone whose Euler angles are re-derived from each node's quaternion
/// (a yaw past π/2 comes back as (±π, π - θ, ±π)), so later
/// `clone.rotation().y = v` writes act on those angles (kept on purpose).
/// `Obj::duplicate` keeps the Euler as it was, hence this wrapper.
fn clone_obj(o: &Obj) -> Obj {
    let c = o.duplicate();
    c.traverse(&mut |n| {
        let q = n.quat();
        *n.quaternion() = q;
    });
    c
}

/// `floor(`rnd()` * len)`
#[inline]
fn pick(len: usize) -> usize {
    (rnd() * len as f64).floor() as usize
}

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

/// A one-triangle geometry with an index and vertex normals.
fn tri_geo(v: [f64; 9], idx: [u32; 3]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

/// Bounds of `o` and its subtree (not precise): each geometry's bounding box
/// through its object's world matrix (call after `update_matrix_world`).
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

// R384 small candle: 12% chance per level. Small wax cylinder with
// tiny wick + optional dark melt drip on side.
fn place_small_candles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        flame.scale().set(0.7, 1.6, 0.7); // tall teardrop
        flame.position().y = body_h + 0.018;
        candle.add(&flame);
        // R387 small warm PointLight for actual scene illumination.
        let flame_light = point_light!(0xf0c060, 0.35, 1.6, 2);
        flame_light.position().y = body_h + 0.020;
        candle.add(&flame_light);
        // Flame flicker animator: scale + opacity + light intensity jitter.
        let fl_phase = rnd() * PI * 2.0;
        let fl_period = 0.4 + rnd() * 0.4; // fast 0.4-0.8 s
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

// R382 vinyl record: 12% chance per level. Round black disc with
// color label at center.
fn place_vinyl_records(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R380 small book on floor: 15% chance per level. Hardcover book
// with visible colored cover.
fn place_floor_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R379 dead potted plant: 15% chance per level. Small pot with
// 2-3 dark bent stems.
fn place_dead_potted_plants(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        let bend_ang = (rnd() - 0.5) * 0.7; // ~40° max bend
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

// R375 traffic cone: 15% chance per level, 1 cone. Orange truncated
// cone + white reflective band.
fn place_traffic_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        // yellow Line between consecutive cone tops at ~0.3 m height.
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

// R374 fallen ceiling tile: 15% chance per level. Small flat
// rectangle on floor representing a dropped acoustic-ceiling tile.
fn place_fallen_ceiling_tiles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
    gap.rotation().x = PI / 2.0; // face down from ceiling
    let tp = tile.pos();
    gap.position().set(tp.x, WALL_H - 0.01, tp.z);
    g.world.add(&gap);
}

/// The narrowest a streak's plane is: just under the width (8 cm) past which
/// `settle_props` probes a wall's face at its sides as well as its middle
const STREAK_W: f64 = 0.0799;

// R368 wall water streak: 20% chance per level, 1-2 streaks. Where water has
// come down a wall from the ceiling's edge, again and again, and dried
// (`r07.notes.md`): it runs straight down from where it came through, narrow
// there and wider below, splitting into runs that end in drips; the edge of
// each run is a darker deposit (a tide line), its middle a fainter wash of
// what it carried: grey-brown on paint and paper, a dark damp run with a
// white salt crust on concrete and brick, a faint rust on glazed tile.
// Painted on a transparent plane from the ceiling's edge to where the old
// streak ended.
fn place_wall_water_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let surface = wall_surface(g, c.x, c.z);
        // the old streak's draws: its width (how far its runs spread, 6-10
        // cm) and its length (0.8-1.8 m below its old top, 10 cm under the
        // ceiling). Its plane's middle is where the old one's was and it is
        // as wide, or 8 cm where it was narrower (`settle_props` probes the
        // wall's face from there, across the width when it is 8 cm or more:
        // so it finds what it found); it reaches up to the ceiling and as
        // far again below its foot
        let streak_w = 0.06 + rnd() * 0.04;
        let streak_h = 0.8 + rnd() * 1.0;
        let streak_top = WALL_H - 0.10;
        let up = WALL_H - 0.005 - streak_top;
        let streak = streak_mesh(streak_w.max(STREAK_W), streak_h + 2.0 * up, streak_h + up, surface, true);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        wall_mount(&streak, &c, wall_x, wall_z, streak_top - streak_h / 2.0, 1.4);
        g.world.add(&streak);
        // R372 25% chance: where it dripped and stood, a dried ring on the
        // floor below (its old radius 9-16 cm, 1.3 × 0.85 of it)
        if rnd() < 0.25 {
            // (on the old disc, 1.3 × 0.85 of its radius: its outline the
            // old one's, the ring painted inside it)
            let r = 0.09 + rnd() * 0.07;
            let floor = floor_kind(g, c.x, c.z);
            let fs = mesh(circle_geo!(r, 12), &foot_ring_mat(r, floor));
            fs.rotation().x = -PI / 2.0;
            fs.rotation().z = rnd() * PI * 2.0;
            fs.scale().set(1.3, 0.85, 1);
            // just below the streak, out from the wall
            let sp = streak.pos();
            fs.position().set(sp.x - c.dx as f64 * 0.14, 0.004, sp.z - c.dz as f64 * 0.14);
            g.world.add(&fs);
        }
        // R370 25% chance: 1-2 more runs close by, from the same edge
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                // the old sibling's draws: width, length, side, offset, drop
                let sib_w = 0.04 + rnd() * 0.04;
                let sib_h = 0.5 + rnd() * 0.7;
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.10 + rnd() * 0.10);
                let drop = rnd() * 0.12;
                // from the ceiling's edge to where the old one ended (its
                // middle and width as the old one's)
                let up = WALL_H - 0.005 - (streak_top - drop);
                let sib = streak_mesh(sib_w.max(STREAK_W), sib_h + 2.0 * up, sib_h + up, surface, false);
                sib.rotation().y = streak.rot().y;
                let sp = streak.pos();
                let y = streak_top - drop - sib_h / 2.0;
                if c.dx.abs() == 1 {
                    sib.position().set(sp.x, y, sp.z + sib_off);
                } else {
                    sib.position().set(sp.x + sib_off, y, sp.z);
                }
                g.world.add(&sib);
            }
        }
        i += 1;
    }
}

// R368 floor drain grate: 15% chance per level, 1 drain. Round
// dark plate with cross-bar detail.
fn place_floor_drain_grates(g: &mut Game, _dist: &[i32]) {
    g.pp.drain_grate_positions = vec![]; // R369 reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let grate_mat = lambert!({ color: 0x2a2620 });
    let bar_mat = lambert!({ color: 0x1a1610 });
    let drain = group();
    // R371 deep-void interior: very dark disc at floor level inside the drain.
    // R373 haunted-biome variant: faint red glow via a basic-material red tint.
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

// R364 rotary desk phone: 12% chance per level. Vintage-style
// phone on floor: base box + rotary disc + handset with coiled cord.
fn place_rotary_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.off_hook_rotary_positions = vec![]; // R366 reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R362 floor puddle: 15% chance per level (25% in wet biome). Water on the
// floor (`r07.notes.md`): on carpet it has soaked in, a dark wet patch with a
// soft, ragged edge and a darker ring where the dirt wicked out to it; on a
// hard floor it stands, a thin film darker than the dry floor round it, its
// edge sharp and irregular, a damp margin outside it and a sheen of the
// lights on it. Painted on a transparent plane flat on the floor.
fn place_floor_puddles(g: &mut Game, _dist: &[i32]) {
    let chance = if g.weather_key() == "wet" { 0.25 } else { 0.15 };
    if rnd() > chance {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // the old puddle's draws: its radius (16-30 cm) and how much longer
    // than wide it was (1-1.3)
    let puddle_r = 0.16 + rnd() * 0.14;
    let long = 1.0 + rnd() * 0.3;
    let (rx, rz) = (puddle_r, puddle_r * long);
    let floor = floor_kind(g, x, z);
    let (puddle_mat, sheen) = puddle_mat(rx, rz, floor);
    let geo = plane_geo!(rx * 2.5, rz * 2.5);
    geo.rotate_x(-PI / 2.0);
    let puddle = mesh(geo, &puddle_mat);
    puddle.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0.003, g.wz(z) + (rnd() - 0.5) * 2.2);
    puddle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&puddle);
    // R363 25% chance: where it is fed by a drip, the sheen on it trembles
    // (period 4-7 s; it was a swell of the old disc)
    if rnd() < 0.25 {
        let r_phase = rnd() * PI * 2.0;
        let r_period = 4.0 + rnd() * 3.0;
        let m2 = puddle_mat.clone();
        g.animate(move |_g, _dt, t| {
            let s = 1.0 + (t * (PI * 2.0 / r_period) + r_phase).sin() * 0.15;
            m2.m().specular.set_rgb(sheen * s, sheen * s, sheen * s);
        });
    }
}

// R361 floor rug remnant with optional peeled corner, lamp and newspaper.
fn place_floor_rugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        // Peeled corner triangle (custom geometry): quarter of rug near corner.
        let c_size = 0.16 + rnd() * 0.06;
        let peel_geo = tri_geo([0.0, 0.0, 0.0, c_size, 0.0, 0.0, 0.0, 0.0, c_size], [0, 2, 1]);
        let peel = mesh(peel_geo, &rug_mat);
        // Rotate around the "root corner" with a pivot group.
        let peel_pivot = group();
        peel_pivot.add(&peel);
        peel_pivot.rotation().x = -0.5 - rnd() * 0.3; // lift 28-46°
                                                      // Position at one of the 4 rug corners in local rug space.
        let corner_idx = (rnd() * 4.0).floor() as usize;
        let corner_offsets =
            [(-rug_w / 2.0, -rug_d / 2.0), (rug_w / 2.0, -rug_d / 2.0), (-rug_w / 2.0, rug_d / 2.0), (rug_w / 2.0, rug_d / 2.0)];
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
            0.007, // slightly above rug (0.005) to avoid z-fight
            rp.z + (rnd() - 0.5) * 0.3,
        );
        g.world.add(&rpaper);
    }
}

// R360 ashtray: 12% chance per level. Small shallow disc with
// 2-3 tiny cigarette butt cylinders on top.
fn place_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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
        filter.position().set(bp.x + tz.cos() * ty.cos() * 0.016, bp.y + tz.sin() * 0.016, bp.z - tz.cos() * ty.sin() * 0.016);
        ashtray.add(&filter);
    }
    ashtray.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    ashtray.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ashtray);
}

// R360 wall coat hook: 15% chance per level, 1-2 hooks. Small metal
// hook + bent tip protruding from wall, at 1.7-1.9 m.
fn place_wall_coat_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
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
        tip.rotation().x = PI / 4.0; // 45° upward bend: up and out from the shaft's end
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
            let sib_step = 0.14 + rnd() * 0.06; // spacing per sibling
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

// R359 table lamp: 15% chance per level. 3-piece Group: circular
// base + narrower vertical stem + tapered shade at top. Dark/off.
fn place_table_lamps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R358 overturned bucket: 15% chance per level. Bucket tipped on
// its side — "someone kicked or knocked over a bucket".
fn place_overturned_buckets(g: &mut Game, _dist: &[i32]) {
    // A 10 l moulded polypropylene pail (batch `r18`), kicked over: on its
    // side, resting on its rim and its foot, or upside down on its rim; its
    // wire bail fallen to the floor; scuffed, grimed at its foot, the litre
    // marks inside it and the muck dried in its bottom (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // the old colour draw: yellow, red or grey polypropylene
    let kind = pick(3).min(2);
    let bucket_yaw = rnd() * PI * 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    // one in three lies upside down (from the yaw's draw: no new draw)
    let upside = (bucket_yaw * 977.0).fract() < 0.33;
    let salt = ((bucket_yaw * 1.0e6) as u32) ^ ((px * 911.0) as i32 as u32).rotate_left(11) ^ ((pz * 613.0) as i32 as u32).rotate_left(21);
    let col = r18_pail::PLASTIC[kind];
    let bucket = group();
    // each part turned as it rests, the turn baked into its own copy (a turned
    // mesh is measured by its turned box, which would lift it off the floor)
    let (turn, lift) = if upside { (PI, r18_pail::HEIGHT) } else { (r18_pail::TILT, r18_pail::LIFT) };
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let geo = geo.duplicate();
            geo.rotate_x(turn);
            geo.translate(0, lift, 0);
            geo.compute_bounding_box();
            bucket.add(&mesh(geo, mat));
        }
    };
    put("Real__overturnedPailBody", &phong!({ map: &r18_pail::body_tex(col, salt), specular: 0x161616, shininess: 18 }));
    put("Real__overturnedPailInside", &phong!({ map: &r18_pail::inside_tex(col, salt), specular: 0x101010, shininess: 14 }));
    put("Real__overturnedPailInsideFloor", &lambert!({ map: &r18_pail::floor_tex(col, salt) }));
    let (bail, grip) = if upside {
        ("Real__overturnedPailBailDown", "Real__overturnedPailGripDown")
    } else {
        ("Real__overturnedPailBailSide", "Real__overturnedPailGripSide")
    };
    put(bail, &phong!({ color: 0x77736c, specular: 0x2a2a28, shininess: 30 }));
    put(grip, &phong!({ color: 0x1d1c1a, specular: 0x1a1a1a, shininess: 26 }));
    let parent = group();
    parent.add(&bucket);
    parent.rotation().y = bucket_yaw;
    parent.position().set(px, 0, pz);
    g.world.add(&parent);
    // R359 40% chance: what was in it, run out and dried on the floor: from its
    // mouth when it lies on its side, all round its rim when upside down
    if rnd() < 0.40 {
        let r = 0.20 + rnd() * 0.10;
        let turn = rnd() * PI * 2.0;
        let hard = floor_kind(g, x, z) != 0;
        let (rx, rz) = if upside { (r * 1.15, r * 1.05) } else { (r * 0.85, r * 1.6) };
        let mat = r18_pail::spill_mat(rx, rz, hard, salt ^ ((turn * 1.0e5) as u32));
        let geo = plane_geo!(rx * 2.5, rz * 2.5);
        geo.rotate_x(-PI / 2.0);
        let spill = mesh(geo, &mat);
        // its long axis along the pail's, its near edge under the mouth
        spill.rotation().y = bucket_yaw;
        let reach = if upside { 0.0 } else { 0.17 + rz * 0.75 };
        let (fx, fz) = (bucket_yaw.sin(), bucket_yaw.cos());
        let pp = parent.pos();
        spill.position().set(pp.x + fx * reach, 0.004, pp.z + fz * reach);
        g.world.add(&spill);
    }
}

// R356 folded newspaper on floor: 20% chance per level, 1 paper.
fn place_floor_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// R347 ceiling cracks: 20% chance per level, 1-2 cracks. A crack is in what
// the ceiling is (`r07.notes.md`): a suspended ceiling's mineral-fibre tile
// cracks within itself, edge to edge across the one tile (the grid holds
// each tile apart), its two halves a little out of line; a concrete slab or a
// plaster ceiling cracks in a jagged, branching line that thins to nothing at
// its ends, white with leached salts along a slab's. Painted on a
// transparent plane under the ceiling.
fn place_ceiling_cracks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open = open_tiles(g);
    shuffle(&mut open);
    let mut i = 0;
    while i < count && i < open.len() {
        let (x, z) = open[i];
        // the old crack's draws: its heading, then three runs, each its
        // length and the turn after it (the line it follows on a slab or in
        // plaster; its kinks across a tile)
        let mut ang = rnd() * PI * 2.0;
        let mut path = vec![(0.0f64, 0.0f64)];
        let mut turns = vec![];
        for _s in 0..3 {
            let seg_len = 0.35 + rnd() * 0.25;
            let (px, pz) = *path.last().unwrap();
            path.push((px + ang.cos() * seg_len, pz + ang.sin() * seg_len));
            let turn = (rnd() - 0.5) * 1.0;
            turns.push(turn);
            ang += turn;
        }
        let crack_group = group();
        crack_group.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, WALL_H - 0.005, g.wz(z) + (rnd() - 0.5) * 1.6);
        let yaw = rnd() * PI * 2.0;
        let cp = crack_group.pos();
        let ceiling = ceiling_kind(g, x, z);
        if ceiling == 0 {
            // one tile of the grid, the one over where it was (or the next
            // one along, off a fitting); edge to edge at the old heading
            let (cx, cz) = ceiling_cell(g, cp.x, cp.z);
            let side = CELL / 6.0;
            let plane = mesh(plane_geo!(side, side), &tile_crack_mat(yaw, &turns, path[1].0.hypot(path[1].1)));
            plane.rotation().x = PI / 2.0;
            plane.position().set(cx - cp.x, 0, cz - cp.z);
            crack_group.add(&plane);
        } else {
            crack_group.rotation().y = yaw;
            let (x0, x1) = path.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
            let (z0, z1) = path.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
            let (pw, ph) = (x1 - x0 + 0.3, z1 - z0 + 0.3);
            let (mx, mz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
            let local: Vec<(f64, f64)> = path.iter().map(|p| (p.0 - mx, p.1 - mz)).collect();
            let plane = mesh(plane_geo!(pw, ph), &slab_crack_mat(pw, ph, &local, ceiling == 1));
            plane.rotation().x = PI / 2.0;
            plane.position().set(mx, 0, mz);
            crack_group.add(&plane);
        }
        g.world.add(&crack_group);
        // R348 30% chance: 2-4 bits fallen from it on the floor below (8-12
        // when a piece came down): of the tile (aged off-white mineral
        // fibre), or chips of the slab or the plaster (`r07.py`)
        if rnd() < 0.30 {
            // (what it was made of, at its own albedo: a tile's aged
            // mineral fibre, the slab's concrete, the plaster's lime)
            let chunk_color = match ceiling {
                0 => 0xb0a998,
                1 => 0x8e897e,
                _ => 0xc2baa8,
            };
            let chunk_mat = lambert!({ color: chunk_color });
            // R352 10% chance: major-fall cluster (8-12 chunks).
            let is_major = rnd() < 0.10;
            let chunk_count = if is_major { 8 + (rnd() * 5.0).floor() as i32 } else { 2 + (rnd() * 3.0).floor() as i32 };
            let bits = if ceiling == 0 {
                ["Real__ceilingCracksTileBitA", "Real__ceilingCracksTileBitB", "Real__ceilingCracksTileBitC"]
            } else {
                ["Real__ceilingCracksChipA", "Real__ceilingCracksChipB", "Real__ceilingCracksChipC"]
            };
            for _ch in 0..chunk_count {
                // the old chunk's three sizes: now which bit it is, and how
                // big (within 13% of as modelled)
                let (sa, sb, sc) = (rnd(), rnd(), rnd());
                let geo = game::assets::props::prop_part_as_modelled(bits[(sa * 3.0) as usize % 3])
                    .unwrap_or_else(|| box_geo!(0.04, 0.016, 0.04));
                let chunk = mesh(geo, &chunk_mat);
                let s = 0.87 + 0.13 * (sb + sc);
                chunk.scale().set(s, s, s);
                let ch_ang = rnd() * PI * 2.0;
                let ch_rad = 0.08 + rnd() * (if is_major { 0.55 } else { 0.25 });
                let ch_sink = ((rnd() - 0.5) * 0.005).abs();
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

// R346 ceiling cobwebs: 25% chance per level, 1-3 cobwebs. An old cobweb in
// the angle of a wall and the ceiling (`r07.notes.md`): a cobweb spider's
// tangle, long abandoned, its threads grey with the dust they caught, slung
// across the angle and sagging, thick where the tangle is and thin out to
// the few anchor threads that hold it to the wall and the ceiling. Its veil
// and its threads painted on a sagging sheet across the angle, torn and
// ragged at its edges, seen from both sides.
fn place_ceiling_cobwebs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    // R347 per-weather cobweb tint (pale greys), greyed by the dust in it
    let dusty = srgb3(cobweb_color_by_weather(g.weather_key())).map(|c| c * 0.92);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // the old web's draws: its size (now its reach along the wall,
        // 0.34-0.6 m, and across the angle), where along the wall, its
        // threads' anchors, the sway, a caught insect and its pose, and
        // smaller webs beside it
        let size = 0.20 + rnd() * 0.15;
        let web = group();
        // Position at ceiling corner along the wall face.
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        wall_mount(&web, &c, wall_x, wall_z, WALL_H - 0.02, 1.5);
        let strand_count = 3 + (rnd() * 2.0).floor() as i32;
        let anchors: Vec<f64> = (0..strand_count).map(|_| 0.1 + rnd() * 0.85).collect();
        // R350 20% chance: very slow sway in the air that moves through
        let sway = if rnd() < 0.20 { Some((rnd() * PI * 2.0, 12.0 + rnd() * 8.0)) } else { None };
        // R350 5% chance: a dead insect caught in it (its turn, and where:
        // the old draws, in the old order for the wall it is on)
        let insect = if rnd() < 0.05 {
            let (spin, a, b) = (rnd(), rnd(), rnd());
            let (drop, along) = if c.dx.abs() == 1 { (a, b) } else { (b, a) };
            Some((spin, drop, along))
        } else {
            None
        };
        // R349 25% chance: 1-2 smaller webs in the same stretch of the angle
        let mut sibs = vec![];
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                let sib_size = 0.10 + rnd() * 0.10;
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.20 + rnd() * 0.20);
                let drop = rnd() * 0.08;
                sibs.push((sib_size, sib_off, drop));
            }
        }
        // as long along the wall as the old one and as deep in all (its
        // middle where the old one's was: what `settle_props` probes with)
        web.add(&web_sheet(size, size * 0.62, size * 0.5, -0.9 * size - 0.018, dusty, &anchors, insect, 320.0));
        g.world.add(&web);
        if let Some((sway_phase, sway_period)) = sway {
            let base_rot_y = web.rot().y;
            let w2 = web.clone();
            g.animate(move |_g, _dt, t| {
                w2.rotation().y = base_rot_y + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.012;
            });
        }
        for (sib_size, sib_off, drop) in sibs {
            // anchored under the same ceiling: its drop is how much further
            // down the wall it reaches
            let low = -2.0 * drop - 0.9 * sib_size - 0.018;
            let sib = web_sheet(sib_size, sib_size * 0.62 + drop, sib_size * 0.5, low, dusty, &[], None, 192.0);
            sib.rotation().y = web.rot().y;
            let wp = web.pos();
            if c.dx.abs() == 1 {
                sib.position().set(wp.x, wp.y, wp.z + sib_off);
            } else {
                sib.position().set(wp.x + sib_off, wp.y, wp.z);
            }
            g.world.add(&sib);
        }
        i += 1;
    }
}

// R345 dead ceiling light fixture: 20% chance per level, 1-2 units.
// Small dark disc + tiny protruding bulb-remnant on ceiling.
fn place_dead_ceiling_lights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
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
        plate.position().y = WALL_H - 0.0054; // the canopy's 5.4 mm back reaches the ceiling
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
            let dw_geo = tri_geo([0.0, 0.0, 0.0, dw_size, 0.0, 0.0, 0.0, -dw_size * 0.7, 0.0], [0, 1, 2]);
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

// R344 ceiling hanging cable: 15% chance per level, 1 unit.
// Vertical thin cylinder dangling from ceiling + small plug head at bottom.
fn place_ceiling_cables(g: &mut Game, _dist: &[i32]) {
    g.pp.ceiling_cable_plug_positions = vec![]; // R355 reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
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

// ------------------------------------------------- painted marks (r07)
//
// The water streaks and the rings they leave, the puddles, the cracks and the
// cobwebs are painted as transparent textures on planes, soft and uneven,
// each from its painter's own stream (`canvas_tex_opt`): never a polygon's
// hard edge. (`assets/src/blender/props/r07.notes.md`: what real ones are.)

/// What the wall seen from open tile (x, z) is: 0 paper or paint (wallpaper,
/// drywall), 1 masonry (concrete, brick), 2 glazed tile
fn wall_surface(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.wall_mat) {
        Some("concrete" | "brick") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// What the floor of tile (x, z) is: 0 carpet (or carpet tiles), 1 concrete,
/// 2 glazed tile
fn floor_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.floor_mat) {
        Some("concrete") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// What the ceiling over tile (x, z) is: 0 mineral-fibre tiles on a T-bar
/// grid, 1 a concrete slab, 2 plaster (`World::ceil_mat_key`)
fn ceiling_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.ceil_mat) {
        Some("slab") => 1,
        Some("plaster") => 2,
        _ => 0,
    }
}

/// The centre of the ceiling tile over world (px, pz) on the T-bar grid
/// (`visuals::textures::CEIL_CELLS`: 6 tiles a 4 m tile, the bars through the
/// tile's edge along x and half a tile off it along z), or the next one along
/// z where a fitting hung at the tile's centre fills it (2 × 1 of them)
fn ceiling_cell(g: &Game, px: f64, pz: f64) -> (f64, f64) {
    let c = CELL / 6.0;
    let (tx, tz) = (g.wx(g.tx(px)), g.wz(g.tz(pz)));
    let cx = tx + ((px - tx) / c).floor() * c + c / 2.0;
    let mut cz = tz + ((pz - tz - c / 2.0) / c).floor() * c + c;
    if (cx - tx).abs() < c && (cz - tz).abs() < c / 2.0 {
        cz += if pz > tz { c } else { -c };
    }
    (cx, cz)
}

/// An sRGB `0xRRGGBB` as three 0..1 values
fn srgb3(h: u32) -> [f64; 3] {
    [((h >> 16) & 255) as f64 / 255.0, ((h >> 8) & 255) as f64 / 255.0, (h & 255) as f64 / 255.0]
}

/// A hash of (seed, i, j) in [0, 1)
fn hash01(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Smooth value noise in [0, 1]
fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash01(seed, i, j), hash01(seed, i + 1, j));
    let (c, d) = (hash01(seed, i, j + 1), hash01(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// Smoothstep from `e0` to `e1` (either way round)
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The smooth minimum of two distances, blended over `k`
fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// How a dried water mark shades at signed distance `d` (m, < 0 inside) from
/// one of its rims: the rim sharp on its outside (over `aa`, about a texel),
/// what the water left there fading inward over `ew`
fn tide(d: f64, aa: f64, ew: f64) -> f64 {
    (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / ew).exp()
}

/// The white salts water leaves on concrete and brick (sRGB)
const SALT: [f64; 3] = [0.78, 0.77, 0.72];

/// A pixel's straight sRGB colour and alpha, built up by laying marks over it
#[derive(Clone, Copy)]
struct Px {
    c: [f64; 3],
    a: f64,
}
impl Px {
    const CLEAR: Px = Px { c: [0.0; 3], a: 0.0 };
    /// `c` at alpha `a` laid over what is there
    fn lay(&mut self, c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        let out = a + self.a * (1.0 - a);
        if out > 1e-6 {
            for (k, v) in self.c.iter_mut().enumerate() {
                *v = (c[k] * a + *v * self.a * (1.0 - a)) / out;
            }
        }
        self.a = out;
    }
    fn rgba(self) -> [f64; 4] {
        [self.c[0], self.c[1], self.c[2], self.a]
    }
}

/// Paints the canvas pixel by pixel: `f(i, j)` gives pixel (i, j)'s straight
/// sRGB and alpha (0..1)
fn paint_px(ctx: &mut Canvas, w: f64, h: f64, mut f: impl FnMut(usize, usize) -> [f64; 4]) {
    let (iw, ih) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    for j in 0..ih {
        for i in 0..iw {
            let c = f(i, j);
            let k = (j * iw + i) * 4;
            for (n, v) in c.iter().enumerate() {
                img.data[k + n] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
    ctx.put_image_data(&img, 0, 0);
}

/// A transparent mark `w × h` px painted by `paint` (on its own stream);
/// where it is clear its colour is `bleed` (sRGB), so its smaller mips keep
/// the mark's colour at its soft edges instead of darkening them
fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, paint);
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// The decal material of a mark painted in `tex`, lit as the surface under it
fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// One run of a water streak, in its plane's metres (x across, y down from
/// the ceiling's edge): where it starts, how it leans, where it ends, its
/// half width at its start and at its widest, how much it narrows again
/// toward its foot, its noise
struct Run {
    xs: f64,
    lean: f64,
    y0: f64,
    y1: f64,
    w0: f64,
    w1: f64,
    taper: f64,
    s: u32,
}
impl Run {
    /// Its middle at depth `y`: straight down, wavering a little as it goes
    fn centre(&self, y: f64) -> f64 {
        let wander = (fbm(self.s, y * 3.0, 0.5, 3) - 0.5) * 0.016 + (vnoise(self.s ^ 0x5bd1, y * 40.0, 0.5) - 0.5) * 0.0025;
        self.xs + self.lean * (y - self.y0) + wander * ((y - self.y0) / 0.1).clamp(0.0, 1.0)
    }
    /// Its half width at depth `y`: narrow where it starts, wider below as
    /// it slowed and spread, narrowing again to the drop it ended in
    fn half(&self, y: f64) -> f64 {
        let t = ((y - self.y0) / (self.y1 - self.y0)).clamp(0.0, 1.0);
        let grow = self.w0 + (self.w1 - self.w0) * (t / 0.45).min(1.0).powf(0.6);
        grow * (1.0 - self.taper * sstep(0.45, 1.0, t)) * (0.6 + 0.8 * fbm(self.s ^ 0x2c1b, y * 7.0, 2.5, 2))
    }
}

/// A dried run of dirty water down a wall from the ceiling's edge (the top
/// of a plane `w × ph` m facing +z), its runs spreading across it to about
/// `w` and the longest ending `foot` m down; on `surface` (`wall_surface`);
/// `main`: the first of a group, with more runs
fn streak_mesh(w: f64, ph: f64, foot: f64, surface: u8, main: bool) -> Obj {
    let pw = w;
    let (tw, th) = (128.0, (ph / 0.003).round().clamp(160.0, 640.0));
    // (sRGB) its tide lines, the wash inside them, and how strong each is
    let (line, fill, k_line, k_fill) = match surface {
        1 => ([0.12, 0.115, 0.105], [0.17, 0.165, 0.15], 0.42, 0.56),
        2 => ([0.45, 0.34, 0.22], [0.55, 0.46, 0.35], 0.38, 0.26),
        _ => ([0.29, 0.22, 0.14], [0.38, 0.31, 0.21], 0.45, 0.52),
    };
    let tex = mark_tex(tw, th, line, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy * 0.5);
        // where it came through at the ceiling's edge, spread a little along it
        let x0 = pw * (0.45 + 0.1 * rnd());
        let (src_rx, src_ry) = (w * (0.32 + 0.14 * rnd()), 0.035 + 0.05 * rnd());
        // the runs: side by side from there, the first to the old foot, the
        // rest shorter and fainter, some branching off lower down
        let n = if main { 3 + (rnd() * 3.0) as usize } else { 2 + (rnd() * 2.0) as usize };
        let mut runs: Vec<(Run, f64)> = vec![];
        for k in 0..n {
            let s = seed.wrapping_add(1 + k as u32 * 7919);
            let (y0, xs) = if k == 0 || rnd() < 0.55 {
                (0.0, (x0 + if k == 0 { 0.0 } else { (rnd() - 0.5) * 2.2 * src_rx }).clamp(0.008, pw - 0.008))
            } else {
                let y0 = ph * (0.05 + 0.3 * rnd());
                (y0, runs[0].0.centre(y0))
            };
            let y1 = if k == 0 { foot - 0.01 * rnd() } else { y0 + (foot - y0) * (0.25 + 0.65 * rnd()) };
            let lean = (rnd() - 0.5) * w * 0.2 / ph;
            // a few millimetres wide where it set off, wider as it slowed:
            // the first a broad wet run, the rest rivulets
            let w0 = (0.002 + 0.002 * rnd()) / (1.0 + 0.3 * k as f64);
            let w1 = if k == 0 { w * (0.2 + 0.1 * rnd()) } else { (0.003 + 0.006 * rnd()) / (1.0 + 0.2 * k as f64) };
            let taper = if k == 0 { 0.5 + 0.2 * rnd() } else { 0.1 + 0.2 * rnd() };
            let strength = if k == 0 { 1.0 } else { 0.4 + 0.5 * rnd() };
            runs.push((Run { xs, lean, y0, y1, w0, w1, taper, s }, strength));
        }
        // the drop each ended in, dried a little fuller than its trail:
        // (x, y, half width, half length)
        let tips: Vec<(f64, f64, f64, f64)> = runs
            .iter()
            .map(|(r, _)| {
                let rb = r.half(r.y1) * 1.2 + 0.0008;
                (r.centre(r.y1 - rb * 1.4), r.y1 - rb * 1.4, rb, rb * 1.4)
            })
            .collect();
        let rows: Vec<Vec<(f64, f64)>> = (0..ch as usize)
            .map(|j| {
                let y = (j as f64 + 0.5) * sy;
                runs.iter().map(|(r, _)| (r.centre(y), r.half(y))).collect()
            })
            .collect();
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx, (j as f64 + 0.5) * sy);
            let wob = (vnoise(seed, x * 260.0, y * 90.0) - 0.5) * 0.0014 + (vnoise(seed ^ 0x77, x * 60.0, y * 14.0) - 0.5) * 0.004;
            // the wetted patch it came from, under the ceiling
            let src = (((x - x0) / src_rx).hypot(y / src_ry) - 1.0) * src_ry + wob;
            let (mut fill_a, mut line_a, mut near) = (0.0f64, 0.0f64, f64::MAX);
            let mut add = |d: f64, strength: f64, fade: f64| {
                near = near.min(d);
                if d > 0.006 {
                    return;
                }
                let inside = (0.5 - d / aa).clamp(0.0, 1.0);
                // streaked down the way it ran, thinner in places
                let streaks = 0.5 + 0.5 * vnoise(seed ^ 0xabc, x * 380.0, y * 7.0);
                let f = inside * k_fill * strength * fade * streaks * (0.7 + 0.3 * vnoise(seed ^ 0x3c3c, x * 40.0, y * 12.0));
                let t0 = tide(d, aa, 0.003);
                let t1 = tide(d + 0.0028 + (vnoise(seed ^ 0x1f, x * 150.0, y * 40.0) - 0.5) * 0.002, aa, 0.0018) * 0.4;
                let l = (1.0 - (1.0 - t0) * (1.0 - t1))
                    * k_line
                    * strength
                    * (0.6 + 0.4 * fade)
                    * (0.4 + 0.6 * vnoise(seed ^ 0x6e, x * 30.0, y * 5.0));
                fill_a = 1.0 - (1.0 - fill_a) * (1.0 - f);
                line_a = 1.0 - (1.0 - line_a) * (1.0 - l);
            };
            add(src, 1.0, 1.0);
            for (((r, strength), &(xc, hw)), &(bx, by, bw, bh)) in runs.iter().zip(&rows[j]).zip(&tips) {
                let along = if y < r.y0 {
                    (x - r.xs).hypot(y - r.y0) - r.w0
                } else if y <= by {
                    (x - xc).abs() - hw
                } else {
                    1.0
                };
                let tip = (((x - bx) / bw).hypot((y - by) / bh) - 1.0) * bw;
                let d = smin(along, tip, 0.002) + wob;
                // fainter as it ran thin, darker again where the drop dried
                let t = ((y - r.y0) / (r.y1 - r.y0)).clamp(0.0, 1.0);
                let fade = 1.1 - 0.5 * t + 0.4 * sstep(by - bh, by, y);
                add(d, *strength, fade);
            }
            let mut px = Px::CLEAR;
            // the faint wetting round them all, soft-edged
            let halo = sstep(0.018, 0.0, near + (vnoise(seed ^ 0x4a, x * 40.0, y * 8.0) - 0.5) * 0.012);
            px.lay(fill, halo * k_fill * 0.22);
            px.lay(fill, fill_a);
            px.lay(line, line_a);
            if surface == 1 {
                // the salts it left: a soft white bloom along its edges and
                // where the drops dried, in patches
                let bloom = sstep(0.45, 0.72, fbm(seed ^ 0x99, x * 110.0, y * 22.0, 3));
                let crust = ((line_a * 1.3 + fill_a * 0.6) * bloom * 0.75).min(0.6);
                px.lay(SALT, crust);
            }
            // (and nothing at the plane's sides)
            let side = sstep(0.0, 0.004, x.min(pw - x));
            [px.c[0], px.c[1], px.c[2], px.a * side]
        });
    });
    mesh(plane_geo!(pw, ph), &mark_mat(&tex))
}

/// The dried ring where water dripped and stood on a floor of `floor`
/// (`floor_kind`), for a disc of radius `r` (`circle_geo!`, its UVs the
/// square round it) scaled 1.3 × 0.85: an irregular ellipse inside the
/// scaled disc, a tide line at its rim and older ones inside it, its middle
/// paler (painted as it will look once scaled)
fn foot_ring_mat(r: f64, floor: u8) -> Mat {
    let (rx, rz) = (r * 1.3 * 0.78, r * 0.85 * 0.78);
    let (pw, ph) = (r * 2.0 * 1.3, r * 2.0 * 0.85);
    let (tw, th) = (160.0, 160.0);
    let (line, fill, k_line, k_fill) = match floor {
        1 => ([0.15, 0.145, 0.135], [0.22, 0.21, 0.2], 0.45, 0.2),
        2 => ([0.45, 0.38, 0.28], [0.6, 0.55, 0.47], 0.4, 0.1),
        _ => ([0.2, 0.15, 0.09], [0.3, 0.24, 0.16], 0.55, 0.18),
    };
    let tex = mark_tex(tw, th, line, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.1;
        let r0 = rx.min(rz);
        // each wetting's rim: (how far inside the last, how strong, its noise)
        let rings: Vec<(f64, f64, u32)> = (0..2 + (rnd() * 2.0) as usize)
            .map(|k| {
                let (o, s) = if k == 0 { (0.0, 1.0) } else { (0.08 + 0.3 * rnd(), 0.35 + 0.4 * rnd()) };
                (o, s, seed.wrapping_add(k as u32 * 977 + 1))
            })
            .collect();
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.hypot(v);
            if rho > 1.45 {
                return [line[0], line[1], line[2], 0.0];
            }
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let fine = (vnoise(seed ^ 0x51, x * 160.0, y * 160.0) - 0.5) * 0.0018;
            let (mut d0, mut lines) = (0.0, 0.0);
            for (k, &(o, s, rs)) in rings.iter().enumerate() {
                let lobe = 1.0
                    + 0.3 * (fbm(rs, cu * 1.3 + 7.0, cv * 1.3 + 7.0, 3) - 0.5)
                    + 0.07 * (vnoise(rs ^ 0x33, cu * 5.0 + 3.0, cv * 5.0 + 3.0) - 0.5);
                let d = (rho - (1.0 - o) * lobe) * r0 + fine;
                if k == 0 {
                    d0 = d;
                }
                lines = 1.0 - (1.0 - lines) * (1.0 - tide(d, aa, 0.004) * s);
            }
            let mut px = Px::CLEAR;
            let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
            let pale = 0.55 + 0.45 * sstep(0.0, 0.8, rho);
            px.lay(fill, inside * k_fill * pale * (0.7 + 0.3 * vnoise(seed ^ 0x3c, x * 40.0, y * 40.0)));
            px.lay(line, lines * k_line);
            if floor == 1 {
                px.lay(SALT, (lines * (vnoise(seed ^ 0x99, x * 300.0, y * 300.0) - 0.45).max(0.0) * 1.5).min(0.5));
            }
            px.rgba()
        });
    });
    mark_mat(&tex)
}

/// Water on a floor of `floor` (`floor_kind`), about `rx × rz` m, on a plane
/// `2.5 rx × 2.5 rz`: soaked into carpet, a dark patch with a soft ragged
/// edge and a darker rim where the dirt wicked out; standing on a hard floor,
/// a dark film with a sharp, irregular edge, a damp margin outside it and a
/// sheen. Its material, and the sheen (linear specular) it was given.
fn puddle_mat(rx: f64, rz: f64, floor: u8) -> (Mat, f64) {
    let (pw, ph) = (rx * 2.5, rz * 2.5);
    let (tw, th) = (192.0, (192.0 * rz / rx).round().clamp(96.0, 256.0));
    let hard = floor != 0;
    let wet = if hard { [0.09, 0.09, 0.095] } else { [0.12, 0.1, 0.08] };
    let tex = mark_tex(tw, th, wet, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.2;
        let r0 = rx.min(rz);
        // on a hard floor, smaller pools beside it, joined to it or not
        let sats: Vec<(f64, f64, f64)> = if hard {
            (0..(rnd() * 3.5) as usize)
                .map(|_| {
                    let (a, rr) = (rnd() * PI * 2.0, 0.65 + 0.25 * rnd());
                    (a.cos() * rx * rr, a.sin() * rz * rr, r0 * (0.1 + 0.15 * rnd()))
                })
                .collect()
        } else {
            vec![]
        };
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.hypot(v);
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let lobe = 1.0
                + 0.5 * (fbm(seed, cu * 1.2 + 4.0, cv * 1.2 + 4.0, 3) - 0.5)
                + 0.16 * (vnoise(seed ^ 0x9, cu * 4.0 + 2.0, cv * 4.0 + 2.0) - 0.5)
                + 0.07 * (vnoise(seed ^ 0xa, cu * 10.0 + 1.0, cv * 10.0 + 1.0) - 0.5);
            let mut d = (rho - 0.88 * lobe) * r0;
            for &(qx, qz, qr) in &sats {
                d = smin(d, (x - qx).hypot(y - qz) - qr, 0.012);
            }
            let mut px = Px::CLEAR;
            if hard {
                d += (vnoise(seed ^ 0x4d, x * 90.0, y * 90.0) - 0.5) * 0.004;
                // the damp margin outside it, the film, and the dark line of
                // its edge
                if d > 0.0 {
                    px.lay([0.11, 0.105, 0.1], 0.2 * (-d / 0.015).exp() * (0.6 + 0.4 * vnoise(seed ^ 0x5e, x * 50.0, y * 50.0)));
                }
                px.lay(wet, (0.5 - d / aa).clamp(0.0, 1.0) * (0.4 + 0.08 * vnoise(seed ^ 0x3c, x * 25.0, y * 25.0)));
                px.lay([0.035, 0.035, 0.04], tide(d, aa, 0.0015) * 0.35);
            } else {
                // soaked in: the edge ragged and soft, a darker rim at it
                let dd = d + (fbm(seed ^ 0x77, x * 14.0, y * 14.0, 3) - 0.5) * 0.035;
                px.lay(wet, sstep(0.012, -0.025, dd) * (0.55 + 0.15 * vnoise(seed ^ 0x3c, x * 30.0, y * 30.0)));
                px.lay([0.09, 0.07, 0.05], 0.22 * (-(dd + 0.004).abs() / 0.008).exp());
            }
            px.rgba()
        });
    });
    let (sheen, shine) = if hard { (0.3, 40.0) } else { (0.03, 18.0) };
    let m = phong!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, shininess: shine });
    m.m().specular.set_rgb(sheen, sheen, sheen);
    (m, sheen)
}

/// A jagged path from `a` to `b`: midpoint displacement, `depth` times, each
/// midpoint pushed sideways by up to `rough` of its span (own stream)
fn jagged(a: (f64, f64), b: (f64, f64), depth: u32, rough: f64) -> Vec<(f64, f64)> {
    let mut pts = vec![a, b];
    for _ in 0..depth {
        let mut next = Vec::with_capacity(pts.len() * 2);
        for w in pts.windows(2) {
            let (p, q) = (w[0], w[1]);
            let off = (rnd() - 0.5) * 2.0 * rough;
            next.push(p);
            next.push(((p.0 + q.0) / 2.0 - (q.1 - p.1) * off, (p.1 + q.1) / 2.0 + (q.0 - p.0) * off));
        }
        next.push(b);
        pts = next;
    }
    pts
}

/// Strokes `pts` (px) segment by segment, `width(t)` px wide at t (0..1
/// along it), round-capped, in `style`
fn stroke_tapered(ctx: &mut Canvas, pts: &[(f64, f64)], style: &str, width: impl Fn(f64) -> f64) {
    ctx.set_stroke_style(style);
    ctx.set_line_cap("round");
    let n = pts.len().saturating_sub(1);
    for k in 0..n {
        ctx.set_line_width(width((k as f64 + 0.5) / n as f64).max(0.2));
        ctx.begin_path();
        ctx.move_to(pts[k].0, pts[k].1);
        ctx.line_to(pts[k + 1].0, pts[k + 1].1);
        ctx.stroke();
    }
}

/// Strokes `pts` (px) as one path, `width` px, in `style`
fn stroke_path(ctx: &mut Canvas, pts: &[(f64, f64)], style: &str, width: f64) {
    if pts.len() < 2 {
        return;
    }
    ctx.set_stroke_style(style);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.set_line_width(width);
    ctx.begin_path();
    ctx.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        ctx.line_to(p.0, p.1);
    }
    ctx.stroke();
}

/// A branch off a crack at `pts[k]`, turned by `turn` from the crack's
/// heading there, `len` long (in `pts`' units), jagged
fn crack_branch(pts: &[(f64, f64)], k: usize, turn: f64, len: f64) -> Vec<(f64, f64)> {
    let (p, q) = (pts[k], pts[(k + 1).min(pts.len() - 1)]);
    let a = (q.1 - p.1).atan2(q.0 - p.0) + turn;
    jagged(p, (p.0 + a.cos() * len, p.1 + a.sin() * len), 4, 0.2)
}

/// A crack across one mineral-fibre ceiling tile: its plane is the tile
/// (`CELL / 6` square, the grid's bars at its edges), the crack edge to edge
/// at heading `yaw`, kinked by the old crack's `turns`, its halves a little
/// out of line (a broken edge lit on one side, a shadow on the other)
fn tile_crack_mat(yaw: f64, turns: &[f64], span: f64) -> Mat {
    let side = CELL / 6.0;
    let turns = turns.to_vec();
    let tex = mark_tex(512.0, 512.0, [0.1, 0.09, 0.08], move |ctx, cw, _ch| {
        let s = side / cw; // m a px
        let hs = side / 2.0 - 0.0125; // the tile's face inside the bars' flanges
        let to_px = |p: (f64, f64)| ((p.0 + side / 2.0) / s, (side / 2.0 - p.1) / s);
        // nothing on the bars: only the tile's face
        let inset = (side / 2.0 - hs) / s;
        ctx.begin_path();
        ctx.rect(inset, inset, cw - 2.0 * inset, cw - 2.0 * inset);
        ctx.clip();
        // the line it breaks along: through a point near the middle, at yaw
        let (dx, dz) = (yaw.cos(), yaw.sin());
        let off = (rnd() - 0.5) * 0.3 * hs;
        let c0 = (-dz * off, dx * off);
        // where that line meets the tile's edges either way
        let reach = |sg: f64| {
            let tx = if dx.abs() > 1e-9 { (hs * dx.signum() * sg - c0.0) / (dx * sg) } else { f64::MAX };
            let tz = if dz.abs() > 1e-9 { (hs * dz.signum() * sg - c0.1) / (dz * sg) } else { f64::MAX };
            tx.min(tz)
        };
        let (ta, tb) = (reach(-1.0), reach(1.0));
        let a = (c0.0 - dx * ta, c0.1 - dz * ta);
        let mut b = (c0.0 + dx * tb, c0.1 + dz * tb);
        // sometimes it has not broken through yet: it stops inside the tile
        let open_end = rnd() < 0.4;
        if open_end {
            let k = 0.55 + 0.3 * rnd();
            b = (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
        }
        // kinked where the old crack turned
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let (nx, nz) = (-(b.1 - a.1) / len, (b.0 - a.0) / len);
        let at = |t: f64, kink: f64| (a.0 + (b.0 - a.0) * t + nx * kink * len * 0.12, a.1 + (b.1 - a.1) * t + nz * kink * len * 0.12);
        let macro_pts = [a, at(1.0 / 3.0, turns[0]), at(2.0 / 3.0, turns[1]), b];
        let mut pts = vec![];
        for w in macro_pts.windows(2) {
            let mut seg = jagged(w[0], w[1], 5, 0.16);
            if !pts.is_empty() {
                seg.remove(0);
            }
            pts.extend(seg);
        }
        let clamp_in = |p: (f64, f64)| (p.0.clamp(-hs, hs), p.1.clamp(-hs, hs));
        let pts: Vec<(f64, f64)> = pts.into_iter().map(|p| to_px(clamp_in(p))).collect();
        // how wide it is open: most at the edge it started from, thinning
        // to nothing where it stops inside
        let wmax = (0.0045 + 0.002 * (span - 0.35) / 0.25) / s;
        let width = |t: f64| wmax * if open_end { (1.0 - t).powf(0.7) } else { 0.75 + 0.25 * (PI * t).sin() };
        // the edge of the half that dropped, lit; the shadow it casts
        let lit: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 + nx * 1.6, p.1 - nz * 1.6)).collect();
        let shade: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - nx * 4.0, p.1 + nz * 4.0)).collect();
        let sag: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - nx * 30.0, p.1 + nz * 30.0)).collect();
        // the dropped half a shade darker, most near the break
        stroke_path(ctx, &sag, "rgba(40,34,28,0.07)", 60.0);
        stroke_path(ctx, &shade, "rgba(38,33,27,0.2)", 9.0);
        stroke_tapered(ctx, &lit, "rgba(222,214,196,0.6)", |t| width(t) * 0.8 + 0.8);
        stroke_tapered(ctx, &pts, "rgb(24,21,18)", width);
        // a branch or two, finer
        for _ in 0..(rnd() * 2.6) as usize {
            let k = ((0.2 + 0.6 * rnd()) * pts.len() as f64) as usize;
            let turn = (0.5 + 0.6 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let br: Vec<(f64, f64)> = crack_branch(&pts, k, turn, (0.04 + 0.1 * rnd()) / s)
                .into_iter()
                .map(|p| (p.0.clamp(0.0125 / s, cw - 0.0125 / s), p.1.clamp(0.0125 / s, cw - 0.0125 / s)))
                .collect();
            let w0 = width(k as f64 / pts.len() as f64) * 0.6;
            stroke_tapered(ctx, &br, "rgb(30,26,22)", |t| w0 * (1.0 - t).powf(0.8));
        }
        // crumbs of the broken face along it
        ctx.set_fill_style("rgba(40,35,30,0.6)");
        for _ in 0..(6.0 + 10.0 * rnd()) as usize {
            let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
            ctx.begin_path();
            ctx.arc(p.0 + (rnd() - 0.5) * 4.0, p.1 + (rnd() - 0.5) * 4.0, 0.5 + rnd(), 0, PI * 2.0);
            ctx.fill();
        }
    });
    mark_mat(&tex)
}

/// A crack in a concrete slab (`slab`) or a plaster ceiling, along `path`
/// (m, the plane's own frame), on a plane `pw × ph` m: jagged and branching,
/// widest along its middle and thinning to nothing at its ends; on a slab,
/// white with the salts the water leached out of it here and there, in
/// plaster the paint flaked along its lips
fn slab_crack_mat(pw: f64, ph: f64, path: &[(f64, f64)], slab: bool) -> Mat {
    let s = (pw.max(ph) / 1024.0).max(0.0016);
    let (tw, th) = ((pw / s).round(), (ph / s).round());
    let path = path.to_vec();
    let tex = mark_tex(tw, th, [0.1, 0.09, 0.08], move |ctx, _, _| {
        let to_px = |p: (f64, f64)| ((p.0 + pw / 2.0) / s, (ph / 2.0 - p.1) / s);
        let mut pts = vec![];
        for w in path.windows(2) {
            let mut seg = jagged(to_px(w[0]), to_px(w[1]), 6, 0.13);
            if !pts.is_empty() {
                seg.remove(0);
            }
            pts.extend(seg);
        }
        let seed = (rnd() * 4.0e9) as u32;
        let wmax = (0.003 + 0.0015 * rnd()) / s;
        let width = |t: f64| wmax * (PI * t).sin().powf(0.6) * (0.7 + 0.6 * vnoise(seed, t * 9.0, 0.5));
        let mut branches = vec![];
        for _ in 0..2 + (rnd() * 3.0) as usize {
            let t = 0.1 + 0.8 * rnd();
            let k = (t * pts.len() as f64) as usize;
            let turn = (0.4 + 0.6 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let br = crack_branch(&pts, k, turn, (0.06 + 0.24 * rnd()) / s);
            let w0 = width(t) * 0.6;
            branches.push((br, w0));
        }
        if slab {
            // the salts: a soft white along stretches of it, a rust stain
            for _ in 0..1 + (rnd() * 2.0) as usize {
                let (a, b) = (rnd(), rnd());
                let (k0, k1) = ((a.min(b) * pts.len() as f64) as usize, (a.max(b) * pts.len() as f64) as usize);
                if k1 > k0 + 1 {
                    stroke_path(ctx, &pts[k0..k1], "rgba(214,209,196,0.12)", 12.0 + 8.0 * rnd());
                    stroke_path(ctx, &pts[k0..k1], "rgba(222,218,206,0.22)", 4.0);
                }
            }
            let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
            let r = 10.0 + 16.0 * rnd();
            let gr = ctx.create_radial_gradient(p.0, p.1, 1, p.0, p.1, r);
            gr.add_color_stop(0, "rgba(110,78,48,0.16)");
            gr.add_color_stop(1, "rgba(110,78,48,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(p.0 - r, p.1 - r, r * 2.0, r * 2.0);
        } else {
            // the paint's lips along it, and flakes of it gone
            let lip: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 + 1.2, p.1 - 1.2)).collect();
            stroke_tapered(ctx, &lip, "rgba(232,226,212,0.4)", |t| width(t) * 0.7 + 0.5);
            ctx.set_fill_style("rgba(150,138,120,0.35)");
            for _ in 0..(4.0 + 6.0 * rnd()) as usize {
                let p = pts[(rnd() * pts.len() as f64) as usize % pts.len()];
                ctx.begin_path();
                ctx.ellipse(
                    p.0 + (rnd() - 0.5) * 6.0,
                    p.1 + (rnd() - 0.5) * 6.0,
                    1.5 + 3.0 * rnd(),
                    1.0 + 2.0 * rnd(),
                    rnd() * PI,
                    0,
                    PI * 2.0,
                );
                ctx.fill();
            }
        }
        // its shadow side, then the crack itself
        let shade: Vec<(f64, f64)> = pts.iter().map(|p| (p.0 - 1.0, p.1 + 1.0)).collect();
        stroke_tapered(ctx, &shade, "rgba(30,27,24,0.25)", |t| width(t) + 1.0);
        let core = if slab { "rgb(26,24,22)" } else { "rgb(34,30,26)" };
        stroke_tapered(ctx, &pts, core, width);
        for (br, w0) in &branches {
            stroke_tapered(ctx, br, core, |t| w0 * (1.0 - t).powf(0.8));
            // and now and then a finer one off the branch
            if rnd() < 0.4 && br.len() > 4 {
                let k = br.len() / 2;
                let sub = crack_branch(br, k, (rnd() - 0.5) * 1.6, (0.03 + 0.06 * rnd()) / s);
                stroke_tapered(ctx, &sub, core, |t| w0 * 0.5 * (1.0 - t));
            }
        }
    });
    mark_mat(&tex)
}

/// A sheet across the angle of a wall and the ceiling, from x = 0 to `len`
/// along the wall: from the wall `down` below the ceiling to the ceiling
/// `out` from the wall, narrowing to the angle at both ends and sagging into
/// the room between its anchors, deepened or shallowed so its lowest point
/// is at `low` (its box is then the old web's, what `settle_props` probes
/// the wall with). In `wall_mount`'s frame (x along the wall, y up, z out of
/// it) with its origin `gap` off the wall and the ceiling; UVs u along the
/// wall, v from the wall (0) to the ceiling (1)
fn web_geo(len: f64, down: f64, out: f64, gap: f64, low: f64) -> Geo {
    let (nu, nv) = (12usize, 6usize);
    let (mut pos, mut uv, mut idx): (Vec<[f64; 3]>, Vec<f32>, Vec<u32>) = (vec![], vec![], vec![]);
    for j in 0..=nv {
        for i in 0..=nu {
            let (u, v) = (i as f64 / nu as f64, j as f64 / nv as f64);
            let env = 0.4 + 0.6 * (PI * u).sin().powf(0.45);
            let (dn, ot) = (down * env, out * env);
            // straight from the wall (2 mm off it) to the ceiling (2 mm under it)
            let (wy, wz) = (-dn, -gap + 0.002);
            let (cy, cz) = (gap - 0.002, ot);
            let sag = 0.2 * dn.hypot(ot) * 4.0 * v * (1.0 - v);
            let y = wy + (cy - wy) * v - sag * std::f64::consts::FRAC_1_SQRT_2;
            let z = wz + (cz - wz) * v + sag * std::f64::consts::FRAC_1_SQRT_2;
            pos.push([u * len, y, z]);
            uv.extend([u as f32, v as f32]);
        }
    }
    let (top, bottom) = (gap - 0.002, pos.iter().fold(f64::MAX, |m, p| m.min(p[1])));
    let k = (low - top) / (bottom - top);
    let pos: Vec<f32> = pos.iter().flat_map(|p| [p[0] as f32, (top + (p[1] - top) * k) as f32, p[2] as f32]).collect();
    for j in 0..nv {
        for i in 0..nu {
            let a = (j * (nu + 1) + i) as u32;
            let (b, c) = (a + 1, a + (nu + 1) as u32);
            idx.extend([a, b, c, b, c + 1, c]);
        }
    }
    let geo = Geo::new();
    geo.set_position(pos);
    geo.set_uv(uv);
    geo.set_index(idx);
    geo.compute_vertex_normals();
    geo
}

/// An old cobweb across the angle of a wall and the ceiling (`web_geo`), its
/// silk `col` (sRGB) gone grey with the dust it caught: a cobweb spider's
/// tangle, long abandoned. A veil of silk too fine to see thread by thread,
/// thickest in the tangle and torn in places; taut threads criss-crossing it
/// from junction to junction, out to anchors on the wall and the ceiling
/// (some at `anchors`, 0..1 along it); the dust beaded along them and matted
/// in clumps; and `insect` (turn, drop, along: 0..1) caught in it. `tw`: its
/// texture's width (px).
#[allow(clippy::too_many_arguments)]
fn web_sheet(len: f64, down: f64, out: f64, low: f64, col: [f64; 3], anchors: &[f64], insect: Option<(f64, f64, f64)>, tw: f64) -> Obj {
    let geo = web_geo(len, down, out, 0.02, low);
    let th = (tw * down.hypot(out) * 1.2 / len).round().clamp(64.0, 256.0);
    let anchors = anchors.to_vec();
    let tex = mark_tex(tw, th, col, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let mm = cw / (len * 1000.0); // px a millimetre
        let shade = |k: f64| col.map(|c| (c * k).min(1.0));
        let rgba = |k: f64, a: f64| {
            let c = shade(k);
            format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a)
        };
        // where the tangle is thick (the canvas's top row is the ceiling's
        // edge of the sheet, its bottom the wall's)
        let (bx, by) = (cw * (0.42 + 0.16 * rnd()), ch * (0.35 + 0.3 * rnd()));
        let (brx, bry) = (cw * (0.44 + 0.12 * rnd()), ch * (0.4 + 0.14 * rnd()));
        let in_blob = |k: f64| {
            let (a, r) = (rnd() * PI * 2.0, rnd().powf(0.6) * k);
            ((bx + a.cos() * r * brx).clamp(0.0, cw), (by + a.sin() * r * bry).clamp(0.0, ch))
        };
        // 1. the veil: silk too fine to see thread by thread, thickest in the
        // tangle, thin out to the ends, wispy along the way it was spun
        let veil = shade(0.97);
        let (wa, wb) = (rnd() * PI, rnd() * PI);
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) / cw, (j as f64 + 0.5) / ch);
            let q = ((x * cw - bx) / brx).hypot((y * ch - by) / bry);
            let dens = (-q * q * 0.8).exp();
            // its edges ragged: torn short of the wall (the bottom) and of its ends
            let rag = fbm(seed ^ 0x99, x * 7.0, y * 2.0, 3) - 0.5;
            let ends = sstep(0.0, 0.16, x.min(1.0 - x) + rag * 0.12) * sstep(0.0, 0.3, 1.0 - y + rag * 0.45);
            // wisps: noise stretched along two ways it was spun
            let (u1, u2) = (x * wa.cos() + y * wa.sin(), -x * wa.sin() + y * wa.cos());
            let (v1, v2) = (x * wb.cos() + y * wb.sin(), -x * wb.sin() + y * wb.cos());
            let wisp = 0.5 * vnoise(seed ^ 0x5e, u1 * 3.0, u2 * 50.0) + 0.5 * vnoise(seed ^ 0x6f, v1 * 3.0, v2 * 50.0);
            [veil[0], veil[1], veil[2], ((0.06 + 0.62 * dens) * ends * (0.35 + 1.1 * wisp * wisp)).min(0.72)]
        });
        ctx.set_line_cap("round");
        let thread = |ctx: &mut Canvas, a: (f64, f64), b: (f64, f64), sag: f64, style: &str, w: f64| {
            let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            let l = (b.0 - a.0).hypot(b.1 - a.1).max(1e-6);
            let (nx, ny) = (-(b.1 - a.1) / l, (b.0 - a.0) / l);
            ctx.set_stroke_style(style);
            ctx.set_line_width(w);
            ctx.begin_path();
            ctx.move_to(a.0, a.1);
            ctx.quadratic_curve_to(mx + nx * sag * l, my + ny * sag * l, b.0, b.1);
            ctx.stroke();
        };
        let px_w = |w_mm: f64| (w_mm * mm).max(0.8);
        // 2. the strands: long, nearly straight, crossing at shallow angles
        // along the two ways it was spun, so many over each other in the
        // tangle that they felt into a grey mat
        let span = cw.max(ch);
        for k in 0..140 + (rnd() * 80.0) as usize {
            let c = in_blob(1.1);
            let a = if k % 2 == 0 { wa } else { wb } + (rnd() - 0.5) * 0.9;
            let l = span * (0.15 + 0.5 * rnd());
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            thread(
                ctx,
                (c.0 - dx, c.1 - dy),
                (c.0 + dx, c.1 + dy),
                (rnd() - 0.5) * 0.08,
                &rgba(1.08 + 0.12 * rnd(), 0.12 + 0.24 * rnd()),
                px_w(0.6 + 0.6 * rnd()),
            );
        }
        // the lines it hangs by: from the tangle out to anchors on the
        // ceiling (top) and the wall (bottom), a few heavy with dust (thicker,
        // greyer, sagging)
        let mut fixed: Vec<(f64, f64)> = anchors.iter().map(|&t| (t * cw, 0.0)).collect();
        for _ in 0..8 + (rnd() * 8.0) as usize {
            fixed.push((cw * (0.04 + 0.92 * rnd()), if rnd() < 0.5 { 0.0 } else { ch }));
        }
        for &f in &fixed {
            let a = in_blob(0.6);
            let dusty = rnd() < 0.3;
            let (k_col, al, w, sag) = if dusty {
                (0.84, 0.3 + 0.2 * rnd(), 1.0 + 0.9 * rnd(), 0.05 + 0.06 * rnd())
            } else {
                (1.12, 0.25 + 0.25 * rnd(), 0.6 + 0.5 * rnd(), (rnd() - 0.5) * 0.04)
            };
            thread(ctx, a, f, sag, &rgba(k_col, al), px_w(w));
        }
        // 3. the dust: specks caught all through the tangle, matted in
        // clumps where it is thickest
        for _ in 0..90 + (rnd() * 80.0) as usize {
            let p = in_blob(1.0);
            ctx.set_fill_style(rgba(0.85 + 0.25 * rnd(), 0.15 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.arc(p.0, p.1, px_w(0.25 + 0.6 * rnd()), 0, PI * 2.0);
            ctx.fill();
        }
        for _ in 0..3 + (rnd() * 5.0) as usize {
            let p = in_blob(0.7);
            let r = (3.0 + 7.0 * rnd()) * mm;
            let gr = ctx.create_radial_gradient(p.0, p.1, 0, p.0, p.1, r);
            gr.add_color_stop(0, &rgba(0.92, 0.4));
            gr.add_color_stop(0.5, &rgba(0.92, 0.18));
            gr.add_color_stop(1, &rgba(0.92, 0.0));
            ctx.set_fill_style(&gr);
            ctx.fill_rect(p.0 - r, p.1 - r, r * 2.0, r * 2.0);
        }
        // 4. torn: holes through all of it where it has come away
        let mut img = ctx.get_image_data(0, 0, cw, ch);
        let (iw, ih) = (cw as usize, ch as usize);
        for j in 0..ih {
            for i in 0..iw {
                let (x, y) = ((i as f64 + 0.5) / cw, (j as f64 + 0.5) / ch);
                let q = ((x * cw - bx) / brx).hypot((y * ch - by) / bry);
                let keep = sstep(0.3, 0.46, fbm(seed ^ 0x7a, x * 4.0, y * 2.5, 4) + 0.18 * (-q * q).exp());
                let k = (j * iw + i) * 4 + 3;
                img.data[k] = (f64::from(img.data[k]) * keep).round() as u8;
            }
        }
        ctx.put_image_data(&img, 0, 0);
        // a fly, long dead, wrapped in it
        if let Some((spin, drop, along)) = insect {
            let p = (bx + (along - 0.5) * brx, by + (drop - 0.5) * bry);
            ctx.save();
            ctx.translate(p.0, p.1);
            ctx.rotate(spin * PI * 2.0);
            ctx.set_stroke_style("rgba(36,30,24,0.8)");
            ctx.set_line_width(0.5 * mm);
            for k in 0..6 {
                let (sx, sy) = (if k < 3 { -1.0 } else { 1.0 }, (k % 3) as f64 - 1.0);
                ctx.begin_path();
                ctx.move_to(sx * 0.8 * mm, sy * 1.2 * mm);
                ctx.line_to(sx * 2.6 * mm, sy * 2.2 * mm);
                ctx.stroke();
            }
            ctx.set_fill_style("rgba(190,186,176,0.35)");
            ctx.begin_path();
            ctx.ellipse(0.0, 1.8 * mm, 1.4 * mm, 2.6 * mm, 0.3, 0, PI * 2.0);
            ctx.fill();
            ctx.set_fill_style("rgba(34,28,22,0.92)");
            ctx.begin_path();
            ctx.ellipse(0.0, 0.0, 1.5 * mm, 2.6 * mm, 0.0, 0, PI * 2.0);
            ctx.fill();
            ctx.restore();
        }
    });
    mesh(geo, &lambert!({ map: &tex, transparent: true, depthWrite: false, side: DoubleSide }))
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

/// The pail of `place_overturned_buckets` (batch `r18`): how it rests, its
/// plastic and what is painted on it.
mod r18_pail {
    use super::*;

    /// Its own stream starts the same on every floor: a placed thing's salt
    /// moves where it starts, so the strokes differ from floor to floor
    fn skip(salt: u32) {
        for _ in 0..(salt % 61) {
            let _ = rnd();
        }
    }

    /// Lying on its side: its turn about x and lift, so the flange and the
    /// foot touch the floor (printed by `r18.py`)
    pub(super) const TILT: f64 = 1.41368;
    pub(super) const LIFT: f64 = 0.10265;
    /// To the top of its rim
    pub(super) const HEIGHT: f64 = 0.230;
    /// Polypropylene gone dull: yellow, oxide red, grey (sRGB 0..1)
    pub(super) const PLASTIC: [[f64; 3]; 3] = [[0.78, 0.63, 0.24], [0.56, 0.22, 0.16], [0.44, 0.44, 0.42]];

    /// fbm seamless round the pail (u in 0..1 is once round)
    fn wrap_fbm(seed: u32, u: f64, y: f64, fu: f64, fy: f64, oct: u32) -> f64 {
        fbm(seed, u * fu, y * fy, oct) * (1.0 - u) + fbm(seed, (u - 1.0) * fu, y * fy, oct) * u
    }

    fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
        let (iw, ih) = (w as usize, h as usize);
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }

    /// Scratches and scuffs: short strokes, paler (stress-whitened) or dark,
    /// most where it was dragged (the foot) and carried (the rim)
    fn scuffs(ctx: &mut Canvas, w: f64, h: f64, col: [f64; 3], n: usize) {
        let pale = format!(
            "rgba({},{},{},",
            (col[0] * 255.0 + 50.0).min(255.0) as u8,
            (col[1] * 255.0 + 50.0).min(255.0) as u8,
            (col[2] * 255.0 + 45.0).min(255.0) as u8
        );
        for k in 0..n {
            let x = rnd() * w;
            let y = if rnd() < 0.6 { h * (1.0 - rnd() * rnd() * 0.5) } else { h * rnd() * rnd() * 0.4 };
            let a = (rnd() - 0.5) * 1.2 + if rnd() < 0.5 { 0.0 } else { PI / 2.0 };
            let l = 3.0 + rnd() * 18.0;
            if k % 3 == 0 {
                ctx.set_stroke_style("rgba(40,34,28,0.35)");
            } else {
                ctx.set_stroke_style(format!("{pale}{:.2})", 0.25 + 0.35 * rnd()).as_str());
            }
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l * 0.35);
            ctx.stroke();
        }
    }

    /// Its outside (round it, up it): the plastic mottled where the sun and
    /// the cleaning left it, grime up from the foot, streaks of dried
    /// limescale run down from the rim, scuffs
    pub(super) fn body_tex(col: [f64; 3], salt: u32) -> Tex {
        canvas_tex_opt(512, 128, false, move |ctx, w, h| {
            skip(salt);
            let seed = salt ^ 0x9a11;
            fill(ctx, w, h, |u, v| {
                let y = v * HEIGHT;
                let mot = wrap_fbm(seed, u, y, 40.0, 30.0, 3);
                let mut c = mix(col, [col[0] * 0.86, col[1] * 0.84, col[2] * 0.82], (mot - 0.4) * 2.0);
                // the dirt film everywhere, thicker toward the foot
                let film = 0.18 + 0.25 * wrap_fbm(seed ^ 3, u, y, 14.0, 9.0, 3);
                c = mix(c, [0.33, 0.30, 0.25], film * 0.45);
                let foot = sstep(0.055, 0.0, y) * (0.55 + 0.45 * wrap_fbm(seed ^ 5, u, y, 60.0, 80.0, 2));
                c = mix(c, [0.2, 0.18, 0.15], foot * 0.8);
                // dried runs down from the rim: pale, uneven, fading out
                let run = (wrap_fbm(seed ^ 7, u, 0.0, 140.0, 1.0, 2) - 0.62).max(0.0) * 4.0;
                let reach = 0.05 + 0.12 * wrap_fbm(seed ^ 9, u, 0.0, 40.0, 1.0, 1);
                let runs = run * sstep(HEIGHT - reach, HEIGHT - 0.01, y) * (0.7 + 0.3 * vnoise(seed ^ 11, u * 300.0, y * 40.0));
                c = mix(c, [0.74, 0.72, 0.66], runs * 0.5);
                c
            });
            scuffs(ctx, w, h, col, 140);
        })
    }

    /// Its inside wall: darker, the tide line where something stood in it
    /// and dried, the muck below it thicker toward the bottom
    pub(super) fn inside_tex(col: [f64; 3], salt: u32) -> Tex {
        canvas_tex_opt(512, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0x1de;
            let tide_y = 0.07 + 0.08 * hash01(seed, 1, 2);
            fill(ctx, w, h, |u, v| {
                let y = v * HEIGHT;
                let mut c = [col[0] * 0.82, col[1] * 0.8, col[2] * 0.78];
                c = mix(c, [0.3, 0.28, 0.24], 0.2 + 0.2 * wrap_fbm(seed, u, y, 20.0, 15.0, 3));
                let wob = 0.004 * (wrap_fbm(seed ^ 2, u, 0.0, 30.0, 1.0, 2) - 0.5);
                let d = y - (tide_y + wob);
                // under the line: the film it left, browner and thicker down to the foot
                let under = sstep(0.002, -0.004, d);
                let thick = under * (0.45 + 0.35 * sstep(tide_y, 0.0, y)) * (0.75 + 0.25 * wrap_fbm(seed ^ 4, u, y, 90.0, 60.0, 2));
                c = mix(c, [0.27, 0.23, 0.17], thick);
                // the line itself: a dark crust, a paler band just above it
                c = mix(c, [0.17, 0.14, 0.1], (-(d / 0.0014).powi(2)).exp() * 0.7);
                c = mix(c, [0.6, 0.57, 0.5], (-((d - 0.003) / 0.0018).powi(2)).exp() * 0.25);
                c
            });
        })
    }

    /// Its bottom, seen from above: what was in it dried to a crust and
    /// cracked into flakes, curled at their edges
    pub(super) fn floor_tex(col: [f64; 3], salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0xf100;
            fill(ctx, w, h, |u, v| {
                let (x, y) = (u * 0.21, v * 0.21);
                // cracks: where the two nearest of a jittered lattice are near equal
                let (cx, cy) = (x / 0.011, y / 0.011);
                let (mut d1, mut d2) = (9.0f64, 9.0f64);
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let (ii, jj) = (cx.floor() as i32 + di, cy.floor() as i32 + dj);
                        let fx = ii as f64 + hash01(seed, ii, jj);
                        let fy = jj as f64 + hash01(seed ^ 77, ii, jj);
                        let d = (fx - cx).hypot(fy - cy);
                        if d < d1 {
                            d2 = d1;
                            d1 = d;
                        } else if d < d2 {
                            d2 = d;
                        }
                    }
                }
                // the crust lies thick in the middle and in patches, thin to
                // the plastic at the edge
                let rho = ((u - 0.5).hypot(v - 0.5) * 2.0).min(1.0);
                let cover = sstep(0.38, 0.55, fbm(seed ^ 2, x * 25.0, y * 25.0, 3) + 0.25 * (1.0 - rho));
                let crack = sstep(0.06, 0.0, d2 - d1) * cover;
                let mut c = [col[0] * 0.55, col[1] * 0.55, col[2] * 0.55];
                c = mix(c, [0.25, 0.22, 0.18], 0.6);
                let crust = mix([0.27, 0.24, 0.19], [0.34, 0.31, 0.25], fbm(seed ^ 1, x * 80.0, y * 80.0, 3));
                c = mix(c, crust, cover);
                // the flakes' edges curled up and catching the light
                c = mix(c, [0.4, 0.37, 0.31], sstep(0.16, 0.05, d2 - d1) * cover * 0.3);
                c = mix(c, [0.12, 0.1, 0.08], crack * 0.8);
                c
            });
        })
    }

    /// What ran out of it, dried on the floor: on a hard floor a grey film
    /// with a dark tide line at its edge and silt settled in lobes; on carpet
    /// a dark patch soaked in, its rim darker
    pub(super) fn spill_mat(rx: f64, rz: f64, hard: bool, salt: u32) -> Mat {
        let (pw, ph) = (rx * 2.5, rz * 2.5);
        let (tw, th) = (160.0, (160.0 * rz / rx).round().clamp(80.0, 256.0));
        let bleed = if hard { [0.36, 0.35, 0.33] } else { [0.16, 0.13, 0.1] };
        let tex = mark_tex(tw, th, bleed, move |ctx, cw, ch| {
            let seed = salt ^ 0x5b11;
            let (sx, sy) = (pw / cw, ph / ch);
            let aa = sx.max(sy) * 1.2;
            let r0 = rx.min(rz);
            paint_px(ctx, cw, ch, |i, j| {
                let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
                let (u, v) = (x / rx, y / rz);
                let rho = u.hypot(v);
                let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
                let lobe = 1.0
                    + 0.5 * (fbm(seed, cu * 1.3 + 4.0, cv * 1.3 + 4.0, 3) - 0.5)
                    + 0.14 * (vnoise(seed ^ 9, cu * 5.0 + 2.0, cv * 5.0 + 2.0) - 0.5);
                let d = (rho - 0.85 * lobe) * r0;
                let mut px = Px::CLEAR;
                if hard {
                    let n = fbm(seed ^ 3, x * 40.0, y * 40.0, 3);
                    if d < 0.0 {
                        px.lay([0.4, 0.39, 0.36], (0.16 + 0.14 * n) * sstep(0.0, -0.01, d));
                    }
                    px.lay([0.22, 0.2, 0.17], tide(d, aa, 0.006) * (0.55 + 0.3 * n));
                    // silt dried in the low spots
                    let silt = (fbm(seed ^ 5, x * 18.0, y * 18.0, 3) - 0.55).max(0.0) * 3.0 * sstep(0.0, -0.03, d);
                    px.lay([0.3, 0.27, 0.22], silt * 0.5);
                } else {
                    let soak = sstep(0.02, -0.02, d) * (0.5 + 0.3 * fbm(seed ^ 3, x * 25.0, y * 25.0, 3));
                    px.lay([0.12, 0.1, 0.08], soak * 0.6);
                    px.lay([0.09, 0.07, 0.05], tide(d, aa * 3.0, 0.02) * 0.45);
                }
                px.rgba()
            });
        });
        mark_mat(&tex)
    }
}
