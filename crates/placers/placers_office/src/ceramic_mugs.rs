//! `place_ceramic_mugs`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Ceramic mug: 12% chance per level. Straight-walled cylinder
// mug + curved handle.
pub(crate) fn place_ceramic_mugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let mug_colors = [0xf0f0e8, 0x304868, 0x684030, 0x505050, 0xd0a850];
    let mug_mat = lambert!({ color: mug_colors[ri(mug_colors.len())] });
    let inside_mat = lambert!({ color: 0x0a0806 });
    let mug = group();
    // 25% chance: sit mug on a matching saucer plate.
    let has_saucer = rnd() < 0.25;
    let mut body_y = 0.045;
    if has_saucer {
        let saucer = mesh(pg("CeramicMugs__saucer", cyl_geo!(0.06, 0.055, 0.008, 12)), &mug_mat);
        saucer.position().y = 0.004;
        mug.add(&saucer);
        body_y += 0.008; // lift mug atop saucer
    }
    // Body: straight cylinder.
    let body = mesh(pg("CeramicMugs__body", cyl_geo!(0.036, 0.036, 0.09, 12)), &mug_mat);
    body.position().y = body_y;
    mug.add(&body);
    // 40% chance liquid disc replaces dark empty interior.
    let has_liquid = rnd() < 0.40;
    if has_liquid {
        let liquid_colors = [0x2a1810, 0x4a3018, 0x1a0a08, 0x8a6030]; // coffee/tea/dark/light-tea
        let liquid_mat = lambert!({ color: liquid_colors[ri(liquid_colors.len())] });
        let liquid = mesh(circle_geo!(0.032, 10), &liquid_mat);
        liquid.rotation().x = -PI / 2.0;
        liquid.position().y = body_y + 0.035; // slightly below rim
        mug.add(&liquid);
    } else {
        let inside = mesh(circle_geo!(0.032, 10), &inside_mat);
        inside.rotation().x = -PI / 2.0;
        inside.position().y = body_y + 0.042;
        mug.add(&inside);
    }
    // Handle: half-torus attached to side.
    let handle = mesh(pg("CeramicMugs__handle", torus_geo!(0.020, 0.005, 4, 8, PI)), &mug_mat);
    handle.rotation().z = -PI / 2.0; // arc bulges out of the mug, not into it
    handle.position().set(0.036, body_y, 0);
    mug.add(&handle);
    mug.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mug.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mug);
    // 25% chance: small tea/coffee stain on floor nearby.
    if rnd() < 0.25 {
        let stain_colors = [0x2a1810, 0x4a3018, 0x1a0a08, 0x5a3818];
        let stain_mat = basic_mat!({ color: stain_colors[ri(stain_colors.len())], transparent: true, opacity: 0.55, depthWrite: false });
        let tea_stain = mesh(circle_geo!(0.04 + rnd() * 0.03, 10), &stain_mat);
        tea_stain.rotation().x = -PI / 2.0;
        tea_stain.rotation().z = rnd() * PI * 2.0;
        tea_stain.scale().set(1.3, 0.85, 1);
        let s_ang = rnd() * PI * 2.0;
        let s_rad = 0.10 + rnd() * 0.12;
        let mp = mug.pos();
        tea_stain.position().set(mp.x + s_ang.cos() * s_rad, 0.004, mp.z + s_ang.sin() * s_rad);
        g.world.add(&tea_stain);
    }
    // 20% chance: 1-2 sibling mugs nearby (break room scene).
    if rnd() < 0.20 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _s in 0..sib_count {
            let sib_mug = clone_obj(&mug);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.15 + rnd() * 0.20;
            sib_mug.position().x += s_ang.cos() * s_rad;
            sib_mug.position().z += s_ang.sin() * s_rad;
            sib_mug.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_mug);
        }
    }
}
