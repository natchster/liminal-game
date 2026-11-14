//! `place_small_candles`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Small candle: 12% chance per level. Small wax cylinder with
// tiny wick + optional dark melt drip on side.
pub(crate) fn place_small_candles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let wax_colors = [0xe8d8b0, 0xd8c898, 0xc8b8a0, 0xa08890];
    let wax_mat = lambert!({ color: wax_colors[ri(wax_colors.len())] });
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
    // 25% chance (independent of flame): thin smoke wisp above
    // wick, implying "candle was just blown out".
    let has_wisp = rnd() < 0.25;
    if has_wisp {
        let wisp_mat = basic_mat!({ color: 0x707070, transparent: true, opacity: 0.35, depthWrite: false });
        let wisp = mesh(cyl_geo!(0.005, 0.012, 0.08, 5), &wisp_mat);
        wisp.position().y = body_h + 0.05;
        wisp.rotation().z = (rnd() - 0.5) * 0.3;
        candle.add(&wisp);
    }
    // 15% chance: small emissive flame tip with animator flicker.
    // (Skip if wisp already spawned.)
    if !has_wisp && rnd() < 0.15 {
        let flame_mat = basic_mat!({ color: 0xf0c060, transparent: true, opacity: 0.85 });
        let flame = mesh(sphere_geo!(0.008, 6, 5), &flame_mat);
        flame.scale().set(0.7, 1.6, 0.7); // tall teardrop
        flame.position().y = body_h + 0.018;
        candle.add(&flame);
        // Small warm PointLight for actual scene illumination.
        let flame_light = point_light!(0xf0c060, 0.35, 1.6, 2);
        flame_light.position().y = body_h + 0.020;
        candle.add(&flame_light);
        // Flame flicker animator: scale + opacity + light intensity jitter.
        let fl_phase = rnd() * PI * 2.0;
        let fl_period = 0.4 + rnd() * 0.4; // fast 0.4-0.8 s
        let (flame2, flame_mat2, light2) = (flame.clone(), flame_mat.clone(), flame_light.clone());
        g.animate(move |_g, _dt, t| {
            let wobble = (t * (PI * 2.0 / fl_period) + fl_phase).dsin();
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
    // 30% chance: wax pool circle stain on floor beneath candle.
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
    // 25% chance: 1-2 sibling candles nearby via clone (shared
    // material; independent positions/rotations). Ritual scene.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        for _s in 0..sib_count {
            let sib_candle = clone_obj(&candle);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.10 + rnd() * 0.14;
            sib_candle.position().x += s_ang.dcos() * s_rad;
            sib_candle.position().z += s_ang.dsin() * s_rad;
            sib_candle.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_candle);
        }
    }
}
