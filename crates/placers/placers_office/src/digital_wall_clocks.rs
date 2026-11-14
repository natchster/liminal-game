//! `place_digital_wall_clocks`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::mount_slid_ref;
use placers_kit::place::ri;
use placers_kit::place::wall_candidates;

// Digital wall clock: 15% chance per level. Small black
// rectangle with red LED time display. Distinct from the analog
// wall clock.
pub(crate) fn place_digital_wall_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.digital_clock_positions = vec![]; // reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    // Use a basic (unlit) material so the LED display stays lit regardless of scene light.
    let dc_mat = basic_mat!({ map: digital_clock_texture(), side: DoubleSide });
    let dc = mesh(plane_geo!(0.20, 0.10), &dc_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let dc_height = 1.9 + rnd() * 0.30;
    mount_slid_ref(&dc, &c, wall_x, wall_z, dc_height, 1.0);
    dc.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&dc);
    // Track digital clock position for periodic mains-hum audio.
    let dp = dc.pos();
    g.pp.digital_clock_positions.push(v3(dp.x, dc_height, dp.z));
    // 40% chance: brief LED flicker animator (opacity/tint dip
    // simulating glitching digital display).
    if rnd() < 0.40 {
        // Convert to transparent material for opacity control.
        dc_mat.m().transparent = true;
        dc_mat.m().opacity = 1.0;
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let m = dc_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                m.m().opacity = if dip_t > 0.0 { 0.25 } else { 1.0 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.05;
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
    // 25% chance: 1-2 sibling digital clocks 0.28-0.36 m in row.
    if rnd() < 0.25 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        for s in 1..=sib_count {
            let sib_clock = clone_obj(&dc);
            let sib_off = sib_dir * (0.28 + rnd() * 0.08) * s as f64;
            if c.dx.abs() == 1 {
                sib_clock.position().z += sib_off;
            } else {
                sib_clock.position().x += sib_off;
            }
            g.world.add(&sib_clock);
            let sp = sib_clock.pos();
            g.pp.digital_clock_positions.push(v3(sp.x, dc_height, sp.z));
        }
    }
    // 50% chance: small power cord from clock bottom running
    // down to floor. Reads as "clock is plugged into unseen socket".
    if rnd() < 0.50 {
        let cord_mat = lambert!({ color: 0x181410 });
        let cord_len = dc_height - 0.05; // reach to just above floor
        let cord = mesh(pg("DigitalWallClocks__cord", cyl_geo!(0.004, 0.004, cord_len, 4)), &cord_mat);
        let dp = dc.pos();
        cord.position().set(dp.x, dc_height - 0.05 - cord_len / 2.0, dp.z);
        // Offset cord slightly out from wall.
        cord.position().x -= c.dx as f64 * 0.005;
        cord.position().z -= c.dz as f64 * 0.005;
        g.world.add(&cord);
    }
}

// Digital wall clock texture — black bg with bright red LED
// segmented time display like "12:34".
game::lazy_static_tex!(
    digital_clock_texture,
    canvas_tex_opt(128, 64, false, |ctx, w, h| {
        // Black bg.
        ctx.set_fill_style("#080605");
        ctx.fill_rect(0, 0, w, h);
        // Red LED segment digits — big block font.
        ctx.set_fill_style("#e02818");
        ctx.set_font("bold 44px monospace");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        // Randomize the time once, when the texture is built, for horror-value variance.
        let hours = ["12", "03", "11", "13", "99", "00"];
        let mins = ["00", "34", "11", "33", "66"];
        let hh = hours[(rnd() * hours.len() as f64).floor() as usize];
        let mm = mins[(rnd() * mins.len() as f64).floor() as usize];
        let t = format!("{}:{}", hh, mm);
        ctx.fill_text(&t, w / 2.0, h / 2.0);
    })
);
