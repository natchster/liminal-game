//! `place_emergency_strobes`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t04_put;
use placers_kit::noise::fbm as t04_fbm;
use placers_kit::noise::sstep as t04_ss;
use placers_kit::paint::fill as t04_fill;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Emergency strobe: 3% chance per level. A fire alarm's wall strobe of
// the compact kind (Blender batch `t04`, the Edwards Genesis G1RF-VM): a red
// moulded housing faded and dusty, FIRE in white under a clear prismatic
// lens, the xenon tube and its reflector behind it. Now and then it fires:
// a white flash that lights the room (the old light, its timing and its
// draws), the tube and the lens lit with it.
pub(crate) fn place_emergency_strobes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let strobe = group();
    // the parts are modelled with the wall at z = 0: the group's origin is
    // 0.5 mm off it
    let o = group();
    o.position().z = -0.0005;
    strobe.add(&o);
    t04_put(&o, "Real__strobeHousing", &lambert!({ map: &st_housing_tex() }));
    let reflector = phong!({ color: 0xb4b4b0, specular: 0x5a5a58, shininess: 40, side: DoubleSide });
    t04_put(&o, "Real__strobeReflector", &reflector);
    let tube_m = lambert!({ color: 0xc8c8c4 });
    tube_m.set_emissive(0xeef0ff);
    tube_m.set_emissive_intensity(0.0);
    t04_put(&o, "Real__strobeTube", &tube_m);
    let lens_m = phong!({ color: 0xd4d8d6, specular: 0x5a5a5a, shininess: 90, transparent: true, opacity: 0.32, depthWrite: false });
    lens_m.set_emissive(0xeef0ff);
    lens_m.set_emissive_intensity(0.0);
    if let Some(l) = t04_put(&o, "Real__strobeLens", &lens_m) {
        l.set_render_order(2);
    }
    let print = mesh(
        plane_geo!(0.046, 0.023),
        lambert!({ map: &st_fire_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().set(ST_FIRE[0], ST_FIRE[1], ST_FIRE[2]);
    o.add(&print);
    // the flash: a xenon tube's white (the old light's strength and reach)
    let flash = point_light!(0xf2f4ff, 0.0, 1.5, 2.0);
    flash.position().set(0, 0.026, ST_LENS_FRONT + 0.004);
    strobe.add(&flash);
    // Flash animator (occasional bright pop).
    let mut next_flash = 8.0 + rnd() * 20.0;
    {
        let flash = flash.clone();
        let (tube_m, lens_m) = (tube_m.clone(), lens_m.clone());
        g.animate(move |_g, _dt, t| {
            if t >= next_flash {
                // Bright pop.
                flash.set_intensity(1.5);
                next_flash = t + 4.0 + rnd() * 8.0;
            } else {
                // Rapid decay.
                let v = flash.intensity() * 0.85;
                flash.set_intensity(v);
            }
            // the tube and the lens lit with it
            let k = flash.intensity() / 1.5;
            tube_m.set_emissive_intensity(k * 2.5);
            lens_m.set_emissive_intensity(k * 0.5);
        });
    }
    // group origin = the plate's back (local z 0): 0.5 mm off the wall
    let (wall_x, wall_z) = wall_xz(g, c, 0.0005);
    mount(&strobe, c, wall_x, wall_z, 2.30, Some(0.6));
    g.world.add(&strobe);
}

/// The strobe's lens, its middle (x, y) and front (`t04.py` ST_*); where
/// FIRE is printed under it
const ST_LENS_FRONT: f64 = 0.0296;

const ST_FIRE: [f64; 3] = [0.0, -0.031, 0.0213];

/// The housing's red (the part's box faces: its front laid across u, v up):
/// moulded plastic faded paler toward its top, dust settled on it, a scuff
fn st_housing_tex() -> Tex {
    canvas_tex_opt(48, 80, false, move |ctx, w, h| {
        let seed = 0x5707_e11u32;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 4.0, v * 6.0, 3);
            let fade = t04_ss(0.3, 1.0, v) * 0.10;
            let dust = (0.06 + 0.10 * t04_ss(0.7, 1.0, v)) * (0.6 + 0.8 * n);
            let red = [0.55 + 0.05 * n + fade, 0.13 + fade * 0.6, 0.11 + fade * 0.5];
            let d = [0.55, 0.52, 0.46];
            [red[0] * (1.0 - dust) + d[0] * dust, red[1] * (1.0 - dust) + d[1] * dust, red[2] * (1.0 - dust) + d[2] * dust, 1.0]
        });
    })
}

/// FIRE in white under the lens, as the makers print it, worn a little
fn st_fire_tex() -> Tex {
    canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("#e6e2d8");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica Neue");
        ctx.fill_text_max("FIRE", w / 2.0, h / 2.0 + 2.0, w - 8.0);
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_global_alpha(0.2 + 0.4 * rnd());
            ctx.begin_path();
            ctx.ellipse(x, y, 0.8 + rnd() * 2.0, 0.5 + rnd(), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_alpha(1.0);
        ctx.set_global_composite_operation("source-over");
    })
}
