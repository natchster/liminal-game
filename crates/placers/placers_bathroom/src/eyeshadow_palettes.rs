//! `place_eyeshadow_palettes`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Eyeshadow palette: 7% chance per level. Case w/ colored pans, sometimes open.
pub(crate) fn place_eyeshadow_palettes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let case_mat = lambert!({ color: 0x202020 });
    let shadow_colors = [0xc06040, 0xe0c060, 0x8060a0, 0x40a0c0, 0xd0d0d0, 0x604020, 0xa080a0, 0xf0c090, 0x306030];
    let palette = group();
    let is_open = rnd() < 0.5;
    let base = mesh(pg("EyeshadowPalettes__base", box_geo!(0.080, 0.006, 0.050)), &case_mat);
    base.position().y = 0.003;
    palette.add(&base);
    if is_open {
        let cols = 3;
        let rows = 2;
        for r in 0..rows {
            for c in 0..cols {
                let idx = ((r * cols + c) % shadow_colors.len() as i32) as usize;
                let pan_mat = lambert!({ color: shadow_colors[idx] });
                let pan = mesh(pg("EyeshadowPalettes__pan", box_geo!(0.020, 0.004, 0.014)), &pan_mat);
                pan.position().set(-0.026 + c as f64 * 0.026, 0.008, -0.013 + r as f64 * 0.026);
                palette.add(&pan);
            }
        }
        // Mirror upright (hinged top).
        let mirror = mesh(pg("EyeshadowPalettes__mirror", plane_geo!(0.076, 0.045)), lambert!({ color: 0xe8e8e8 }));
        mirror.position().set(0, 0.028, -0.028);
        mirror.rotation().x = -0.3;
        palette.add(&mirror);
    } else {
        let lid = mesh(pg("EyeshadowPalettes__lid", box_geo!(0.080, 0.006, 0.050)), &case_mat);
        lid.position().y = 0.009;
        palette.add(&lid);
        // Small logo dot on lid.
        let logo = mesh(circle_geo!(0.006, 8), lambert!({ color: 0xd0a040 }));
        logo.rotation().x = -PI / 2.0;
        logo.position().y = 0.0125;
        palette.add(&logo);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    palette.position().set(px, 0, pz);
    palette.rotation().y = rnd() * PI * 2.0;
    g.world.add(&palette);
}
