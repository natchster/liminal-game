//! `place_old_crtt_vs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Old CRT TV: 4% chance per level. Boxy TV with static screen + antennae.
pub(crate) fn place_old_crt_tvs(g: &mut Game, _dist: &[i32]) {
    g.pp.crt_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let case_mat = lambert!({ color: 0x8a7060 });
    let dark_mat = lambert!({ color: 0x181410 });
    let screen_mat = basic_mat!({ map: crt_static_texture() });
    let antenna_mat = lambert!({ color: 0x606870 });
    let tv = group();
    let (bw, bh, bd) = (0.28, 0.22, 0.24);
    // Case body.
    let body = mesh(pg("OldCRTTVs__body", box_geo!(bw, bh, bd)), &case_mat);
    body.position().y = bh / 2.0;
    tv.add(&body);
    // Screen (dark bezel).
    let bezel = mesh(pg("OldCRTTVs__bezel", box_geo!(bw * 0.85, bh * 0.75, 0.005)), &dark_mat);
    bezel.position().set(0, bh / 2.0, bd / 2.0 + 0.001);
    tv.add(&bezel);
    // Screen face (canvas static).
    let screen = mesh(plane_geo!(bw * 0.78, bh * 0.68), &screen_mat);
    screen.position().set(0, bh / 2.0, bd / 2.0 + 0.006);
    tv.add(&screen);
    // 3 knob dials on side.
    for i in 0..3 {
        let knob = mesh(pg("OldCRTTVs__knob", cyl_geo!(0.008, 0.008, 0.006, 6)), &dark_mat);
        knob.rotation().z = PI / 2.0;
        knob.position().set(bw / 2.0 + 0.003, bh / 2.0 - 0.02 - i as f64 * 0.04, bd / 2.0 - 0.04);
        tv.add(&knob);
    }
    // 2 antennae (rabbit ears).
    for s in [-1.0, 1.0] {
        let ant = mesh(pg("OldCRTTVs__antenna", cyl_geo!(0.002, 0.002, 0.18, 3)), &antenna_mat);
        ant.rotation().z = s * 0.5;
        ant.position().set(s * 0.06, bh + 0.09, -bd / 2.0 + 0.03);
        tv.add(&ant);
    }
    // Faint self-lit screen glow point light.
    let glow = point_light!(0x506080, 0.20, 0.6, 2);
    glow.position().set(0, bh / 2.0, bd / 2.0 + 0.10);
    tv.add(&glow);
    // Static flicker animator (slight intensity wobble).
    let flicker_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = t * 12.0 + flicker_phase;
        glow.set_intensity(0.20 + p.dsin() * 0.06 + (p * 3.3).dsin() * 0.04);
    });
    scatter(g, &tv, x, z, 1.6);
    g.world.add(&tv);
    // Track for static hiss audio.
    let p = tv.pos();
    g.pp.crt_positions.push(v3(p.x, 0.20, p.z));
}

/// Convert to a clamped byte (round half to even, clamp to 0..255, NaN → 0).
fn u8_clamped(v: f64) -> u8 {
    if !(v > 0.0) {
        return 0;
    }
    if v >= 255.0 {
        return 255;
    }
    let f = v.floor();
    let d = v - f;
    let r = if d > 0.5 {
        f + 1.0
    } else if d < 0.5 {
        f
    } else if f % 2.0 == 0.0 {
        f
    } else {
        f + 1.0
    };
    r as u8
}

game::lazy_static_tex!(
    crt_static_texture,
    canvas_tex(128, 96, |ctx, w, h| {
        // Gray background.
        ctx.set_fill_style("#404448");
        ctx.fill_rect(0, 0, w, h);
        // Random static noise pixels.
        let mut img = ctx.get_image_data(0, 0, w, h);
        let n = img.data.len();
        let mut i = 0;
        while i < n {
            let v = u8_clamped(40.0 + rnd() * 180.0);
            img.data[i] = v;
            img.data[i + 1] = v;
            img.data[i + 2] = v;
            img.data[i + 3] = 255;
            i += 4;
        }
        ctx.put_image_data(&img, 0, 0);
        // Horizontal scan lines.
        ctx.set_fill_style("rgba(0,0,0,0.15)");
        let mut y = 0.0;
        while y < h {
            ctx.fill_rect(0, y, w, 1);
            y += 3.0;
        }
        // Faint color banding.
        ctx.set_fill_style("rgba(80,120,180,0.10)");
        let mut band_y = 20.0;
        while band_y < h {
            ctx.fill_rect(0, band_y, w, 8);
            band_y += 30.0;
        }
    })
);

#[cfg(test)]
mod tests {
    /// The canvas's `Uint8ClampedArray` rounding, which the screen's static was
    /// first painted with: clamped, half to even.
    #[test]
    fn u8_clamped_matches_uint8clampedarray() {
        use super::u8_clamped;
        assert_eq!(u8_clamped(-3.0), 0);
        assert_eq!(u8_clamped(300.0), 255);
        assert_eq!(u8_clamped(40.4), 40);
        assert_eq!(u8_clamped(40.6), 41);
        assert_eq!(u8_clamped(40.5), 40); // ties to even
        assert_eq!(u8_clamped(41.5), 42);
    }
}
