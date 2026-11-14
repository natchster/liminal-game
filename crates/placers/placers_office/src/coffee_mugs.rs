//! `place_coffee_mugs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Coffee mug w/ steam + coffee stain ring (floor): 7% chance per level.
pub(crate) fn place_coffee_mugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let mug_colors = [0xffffff, 0x2060a0, 0xa02040, 0x606060, 0xd0a040, 0x408060, 0x101010];
    let mug_mat = lambert!({ color: pick(&mug_colors) });
    let coffee_mat = lambert!({ color: 0x2a1808 });
    let m = group();
    // Body — cylinder.
    let (body_r, body_h) = (0.038, 0.085);
    let body = mesh(pg("CoffeeMugs__body", cyl_geo!(body_r, body_r * 0.9, body_h, 14)), &mug_mat);
    body.position().y = body_h / 2.0 + 0.002;
    m.add(&body);
    // Coffee surface disc (near top).
    let coffee = mesh(circle_geo!(body_r * 0.94, 14), &coffee_mat);
    coffee.rotation().x = -PI / 2.0;
    coffee.position().y = body_h - 0.005;
    m.add(&coffee);
    // Subtle liquid ripple — 45% chance.
    if rnd() < 0.45 {
        let coffee_base_y = coffee.pos().y;
        let ripple_phase = rnd() * PI * 2.0;
        let ripple_phase2 = rnd() * PI * 2.0;
        let coffee2 = coffee.clone();
        g.animate(move |_g, _dt, t| {
            // Multi-freq combined sine — imperceptible amplitude.
            coffee2.position().y = coffee_base_y + (t * 1.8 + ripple_phase).dsin() * 0.0004 + (t * 3.5 + ripple_phase2).dsin() * 0.0002;
            // Micro-tilt makes the reflection catch light differently.
            coffee2.rotation().z = (t * 1.2 + ripple_phase).dsin() * 0.008;
            coffee2.rotation().x = -PI / 2.0 + (t * 1.0 + ripple_phase2).dsin() * 0.006;
        });
    }
    // Rim torus.
    let rim = mesh(pg("CoffeeMugs__rim", torus_geo!(body_r, 0.003, 4, 14)), &mug_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = body_h + 0.002;
    m.add(&rim);
    // Handle — partial torus arc on side (C-handle bulging outward).
    let handle = mesh(pg("CoffeeMugs__handle", torus_geo!(0.020, 0.005, 4, 10, PI)), &mug_mat);
    handle.rotation().z = -PI / 2.0;
    handle.position().set(body_r - 0.002, body_h * 0.5 + 0.002, 0);
    m.add(&handle);
    // 70% chance: drifting steam wisps + animator.
    if rnd() < 0.7 {
        let steam_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.18 });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
        }
        let mut steam_wisps: Vec<Wisp> = vec![];
        for i in 0..3 {
            let wisp = mesh(sphere_geo!(0.014, 6, 5), &steam_mat);
            wisp.scale().y = 2.2;
            wisp.position().set((rnd() - 0.5) * 0.02, body_h + 0.04 + i as f64 * 0.03, (rnd() - 0.5) * 0.02);
            steam_wisps.push(Wisp { mesh: wisp.clone(), phase: rnd() * PI * 2.0, base_y: wisp.pos().y });
            m.add(&wisp);
        }
        g.animate(move |_g, _dt, t| {
            for w in &steam_wisps {
                {
                    let mut p = w.mesh.position();
                    // Slow drift upward + gentle horizontal sway.
                    p.y = w.base_y + (t * 0.6 + w.phase).dsin() * 0.015;
                    p.x = (t * 0.4 + w.phase).dsin() * 0.015;
                    p.z = (t * 0.5 + w.phase * 1.3).dcos() * 0.015;
                }
                // Subtle scale pulse.
                let s = 0.9 + (t * 0.8 + w.phase).dsin() * 0.15;
                w.mesh.scale().set(s, 2.2 * s, s);
            }
        });
    }
    // 50% chance: dried coffee stain ring on floor beside mug.
    if rnd() < 0.5 {
        let stain_mat = basic_mat!({ color: 0x4a2810, transparent: true, opacity: 0.55 });
        let stain_ring = mesh(ring_geo!(0.038, 0.048, 20), &stain_mat);
        stain_ring.rotation().x = -PI / 2.0;
        stain_ring.position().set((rnd() - 0.3) * 0.10, 0.001, (rnd() - 0.3) * 0.10);
        m.add(&stain_ring);
    }
    m.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    m.rotation().y = rnd() * PI * 2.0;
    g.world.add(&m);
}
