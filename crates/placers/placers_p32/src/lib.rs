//! Prop placers: wall vents, dropped shoes, wall sockets and wall stains.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_wall_vents", place_wall_vents));
    r.push(("place_dropped_shoes", place_dropped_shoes));
    r.push(("place_wall_sockets", place_wall_sockets));
    r.push(("place_wall_stains", place_wall_stains));
}

/// A wall facing: open tile (x, z) with a wall (or the grid edge) at (x+dx, z+dz).
#[derive(Clone, Copy)]
struct Cand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The wall-facing candidate scan every wall placer repeats.
fn wall_candidates(g: &Game) -> Vec<Cand> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                // off the grid or a wall tile
                if g.is_wall_tile(nx, nz) {
                    candidates.push(Cand { x, z, dx, dz });
                }
            }
        }
    }
    candidates
}

/// In-place Fisher-Yates: for i from n-1 down to 1, swap with j = floor(rnd() * (i+1)).
fn shuffle<T>(v: &mut [T]) {
    if v.len() < 2 {
        return;
    }
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// Shoe colour per weather (else the normal entry)
fn shoe_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a2018, 0x3c2818, 0x1a0f0a, 0x4a3020, 0x4a3a28, 0x3c2818)
}

// R248 wall vents: 0-2 rusted grille boxes per level on random walls
// at head height (1.6-2.1 m).
fn place_wall_vents(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.45 {
        return;
    } // 45% chance NO vents this level
    let count = 1 + (rnd() * 2.0).floor() as usize; // 1-2
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let frame_color = vent_color_by_weather(g.weather_key());
    let frame_mat = lambert!({ color: frame_color });
    let opening_mat = basic_mat!({ color: 0x080604 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.03);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.03);
        let mid_height = 1.6 + rnd() * 0.5; // 1.6 - 2.1 m
                                            // Vent frame (metallic box, proud of wall).
        let frame = mesh(pg("WallVents__grille", box_geo!(0.30, 0.20, 0.04)), &frame_mat);
        // Recessed opening (darker inset).
        let opening = mesh(plane_geo!(0.24, 0.14), &opening_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            // Position opening just proud of the frame face.
            opening.position().set(wall_x + c.dx as f64 * 0.021, mid_height, wall_z);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            opening.position().set(wall_x, mid_height, wall_z + c.dz as f64 * 0.021);
        }
        g.world.add(&frame);
        g.world.add(&opening);
        i += 1;
    }
}

// R248 dropped shoes: very rare small worn-shoe mesh on floor.
fn place_dropped_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    if open_tiles.is_empty() {
        return;
    }
    // R249 variant: 60% solo, 30% pair (matched), 10% trail.
    let variant_roll = rnd();
    let variant = if variant_roll < 0.60 {
        "solo"
    } else if variant_roll < 0.90 {
        "pair"
    } else {
        "trail"
    };
    let shoe_color = shoe_color_by_weather(g.weather_key());
    let sole_mat = lambert!({ color: 0x1a1210 });
    let upper_mat = lambert!({ color: shoe_color });
    let make_shoe = || {
        let shoe = group();
        let sole = mesh(pg("DroppedShoes__sole", box_geo!(0.28, 0.03, 0.11)), &sole_mat);
        sole.position().y = 0.015;
        shoe.add(&sole);
        let upper = mesh(pg("DroppedShoes__upper", box_geo!(0.24, 0.08, 0.10)), &upper_mat);
        upper.position().y = 0.055;
        upper.position().x = -0.02; // heel-forward asymmetry
        shoe.add(&upper);
        let toe = mesh(pg("DroppedShoes__toe", box_geo!(0.08, 0.05, 0.08)), &upper_mat);
        toe.position().set(0.10, 0.045, 0); // toe cap ends flush with the sole's front (x 0.14)
        shoe.add(&toe);
        shoe
    };
    let pick = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let (base_x, base_z) = (g.wx(pick.0), g.wz(pick.1));
    if variant == "solo" {
        let shoe = make_shoe();
        let px = base_x + (rnd() - 0.5) * 2.0;
        let pz = base_z + (rnd() - 0.5) * 2.0;
        shoe.position().set(px, 0, pz);
        shoe.rotation().y = rnd() * PI * 2.0;
        g.world.add(&shoe);
    } else if variant == "pair" {
        // Two shoes side-by-side, both facing same direction.
        let rot = rnd() * PI * 2.0;
        let (cx, cz) = ((rot + PI / 2.0).cos(), (rot + PI / 2.0).sin());
        let cx2 = base_x + (rnd() - 0.5) * 1.6;
        let cz2 = base_z + (rnd() - 0.5) * 1.6;
        for s in 0..2 {
            let off = (if s == 0 { -1.0 } else { 1.0 }) * 0.09; // 18 cm apart
            let shoe = make_shoe();
            shoe.position().set(cx2 + cx * off, 0, cz2 + cz * off);
            shoe.rotation().y = rot;
            g.world.add(&shoe);
        }
    } else {
        // Trail: 2-3 shoes in line pointing same direction — "ran".
        let trail_count = 2 + (rnd() * 2.0).floor() as i32;
        let dir = rnd() * PI * 2.0;
        let (dx, dz) = (dir.cos(), dir.sin());
        let spacing = 0.55 + rnd() * 0.25;
        for s in 0..trail_count {
            let shoe = make_shoe();
            let s = s as f64;
            shoe.position().set(base_x + dx * s * spacing, 0, base_z + dz * s * spacing);
            shoe.rotation().y = dir + (rnd() - 0.5) * 0.3;
            g.world.add(&shoe);
        }
    }
}

// R249 wall sockets: 0-4 small floor-level outlets on random walls.
fn place_wall_sockets(g: &mut Game, _dist: &[i32]) {
    g.pp.socket_positions.clear(); // reset per level for R250 spark events
    if rnd() < 0.30 {
        return;
    } // 30% chance NO sockets
    let wk = g.weather_key();
    // Electric biome gets more (charge-heavy identity).
    let density_mul = if wk == "electric" { 1.6 } else { 1.0 };
    let count = 6.0f64.min(1.0f64.max(((1.0 + rnd() * 3.0) * density_mul).floor())) as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let socket_mat = lambert!({ color: 0x2a2822 });
    let hole_mat = basic_mat!({ color: 0x000000 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // 0.10 × 0.07 outlet plate.
        let plate = mesh(pg("WallSockets__plate", box_geo!(0.10, 0.07, 0.015)), &socket_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let socket_height = 0.28 + rnd() * 0.15; // 0.28 - 0.43 m
                                                 // Two tiny plug holes.
        let hole1 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        let hole2 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let pz = wall_z + (rnd() - 0.5) * 1.4;
            plate.position().set(wall_x, socket_height, pz);
            // 4.5 mm behind the plate face: dark bottoms of the modelled slots
            let hx = wall_x - c.dx as f64 * 0.003;
            hole1.position().set(hx, socket_height, plate.pos().z - 0.018);
            hole2.position().set(hx, socket_height, plate.pos().z + 0.018);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let px = wall_x + (rnd() - 0.5) * 1.4;
            plate.position().set(px, socket_height, wall_z);
            let hz = wall_z - c.dz as f64 * 0.003;
            hole1.position().set(plate.pos().x - 0.018, socket_height, hz);
            hole2.position().set(plate.pos().x + 0.018, socket_height, hz);
        }
        g.world.add(&plate);
        g.world.add(&hole1);
        g.world.add(&hole2);
        // R250 track for socket-spark events (electric biome only).
        let p = plate.pos();
        g.pp.socket_positions.push(v3(p.x, socket_height, p.z));
        i += 1;
    }
}

// R245 wall drip stains: vertical dark streaks on random wall segments,
// hanging from near the ceiling downward.
fn place_wall_stains(g: &mut Game, _dist: &[i32]) {
    let spec = floor_stain_by_weather(g.weather_key());
    let mut candidates = wall_candidates(g);
    // Density: ~1 per 40 wall-facings × density_mul. Cap 8 to keep sparse.
    let count = 8.0f64.min(0.0f64.max((candidates.len() as f64 * spec.density_mul / 40.0).floor())) as usize;
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Streak dimensions: 0.04-0.10 wide, 0.6-1.6 tall.
        let w = 0.04 + rnd() * 0.06;
        let h = 0.6 + rnd() * 1.0;
        let geo = plane_geo!(w, h);
        let mat = basic_mat!({
            color: spec.color,
            transparent: true,
            opacity: (spec.opacity_base + 0.15) * (0.7 + rnd() * 0.5), // +0.15 for wall visibility
            depthWrite: false,
            side: DoubleSide,
        });
        let streak = mesh(geo, &mat);
        // Position on wall face just proud (0.02) to avoid z-fight.
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        if c.dx.abs() == 1 {
            streak.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            let z = wall_z + (rnd() - 0.5) * 1.6;
            streak.position().set(wall_x, WALL_H - h * 0.5 - 0.15, z);
        } else {
            streak.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            let x = wall_x + (rnd() - 0.5) * 1.6;
            streak.position().set(x, WALL_H - h * 0.5 - 0.15, wall_z);
        }
        g.world.add(&streak);
        i += 1;
    }
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
