//! `place_dropped_shoes`
use game::prelude::*;

// Dropped shoes: very rare small worn-shoe mesh on floor.
pub(crate) fn place_dropped_shoes(g: &mut Game, _dist: &[i32]) {
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
    // Variant: 60% solo, 30% pair (matched), 10% trail.
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
        let (cx, cz) = ((rot + PI / 2.0).dcos(), (rot + PI / 2.0).dsin());
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
        let (dx, dz) = (dir.dcos(), dir.dsin());
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

/// Shoe colour per weather (else the normal entry)
fn shoe_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a2018, 0x3c2818, 0x1a0f0a, 0x4a3020, 0x4a3a28, 0x3c2818)
}
