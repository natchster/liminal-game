//! `place_emergency_lights`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Emergency light: 20% chance per level, 1 wall-mounted 2-bulb fixture.
pub(crate) fn place_emergency_lights(g: &mut Game, _dist: &[i32]) {
    g.pp.glow_light_positions.clear(); // reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let housing_mat = lambert!({ color: 0xf0e8d8 });
    let rim_mat = lambert!({ color: 0x3a3028 });
    let light = group();
    let housing = mesh(pg("EmergencyLights__housing", box_geo!(0.30, 0.10, 0.10)), &housing_mat);
    light.add(&housing);
    // 20% chance the bulbs flicker via animator.
    let flickering = rnd() < 0.20;
    // Per-light material so animator can pulse individually.
    let bulb_mat = lambert!({ color: 0xe8e4c0, transparent: flickering, opacity: 1 });
    for side in [-1.0, 1.0] {
        let bulb = mesh(pg("EmergencyLights__head", cyl_geo!(0.04, 0.045, 0.05, 10)), &bulb_mat);
        bulb.rotation().x = PI / 2.0;
        bulb.position().set(side * 0.09, 0, 0.075);
        light.add(&bulb);
        let rim = mesh(pg("EmergencyLights__bezel", cyl_geo!(0.048, 0.048, 0.008, 10)), &rim_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(side * 0.09, 0, 0.100);
        light.add(&rim);
    }
    if flickering {
        // Random-interval brief opacity dip (5-15 s between dips, 60 ms dip).
        let mut flicker_t = 5.0 + rnd() * 10.0;
        let mut dip_t = 0.0;
        let bm = bulb_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                bm.m().opacity = if dip_t > 0.0 { 0.20 } else { 1.0 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.06;
                    flicker_t = 5.0 + rnd() * 10.0;
                }
            }
        });
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.05); // housing back (z −0.05) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.05);
    let light_height = 2.5 + rnd() * 0.3; // 2.5-2.8 m (above door lintel)
    if c.dx.abs() == 1 {
        light.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        light.position().set(wall_x, light_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        light.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        light.position().set(wall_x + (rnd() - 0.5) * 1.2, light_height, wall_z);
    }
    g.world.add(&light);
    // "on" variant: 10% chance the emergency light emits a real dim point light.
    if rnd() < 0.10 {
        let base_int = 0.6;
        let point_l = point_light!(0xf5e090, base_int, 3.0, 2);
        // 0.15 m forward of housing (in front of bulbs; local +Z).
        let ly = light.rot().y;
        let (fx, fz) = (ly.sin(), ly.cos());
        let lp = light.pos();
        point_l.position().set(lp.x + fx * 0.15, lp.y - 0.02, lp.z + fz * 0.15);
        g.world.add(&point_l);
        bulb_mat.m().color.set_hex(0xfff8c0); // brighter "on" bulb color
                                              // Subtle sin-wave flicker: ±10% at 0.7 Hz with random phase.
        let flicker_phase = rnd() * PI * 2.0;
        let pl = point_l.clone();
        g.animate(move |_g, _dt, t| {
            pl.set_intensity(base_int * (0.90 + 0.10 * (t * 1.4 * PI + flicker_phase).sin()));
        });
        // Track for spatial hum audio events.
        let pp = point_l.pos();
        g.pp.glow_light_positions.push(v3(pp.x, pp.y, pp.z));
    }
}
