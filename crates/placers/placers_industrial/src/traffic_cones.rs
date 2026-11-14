//! `place_traffic_cones`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Traffic cone: 15% chance per level, 1 cone. Orange truncated
// cone + white reflective band.
pub(crate) fn place_traffic_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // Per-weather cone body color.
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
    // 25% chance: 1-2 sibling cones spaced 0.6-1.0 m in a row.
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
        // Safety tape line connecting sibling cones. Thin dark-
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

/// Traffic cone per weather (else the normal entry)
fn traffic_cone_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa04010, 0xf07038, 0x903030, 0x8a4830, 0xd85018, 0xd85018)
}
