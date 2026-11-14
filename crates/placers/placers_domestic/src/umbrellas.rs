//! `place_umbrellas`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Leaning umbrella: 15% chance per level, 1 closed umbrella against a
// wall: a classic stick umbrella (Blender family `crate_sash_umbrella_and_things_put_away`), 0.89 m, furled and
// strapped, stood on its ferrule with its crook against the wall.
pub(crate) fn place_umbrellas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    // Bias primary color to per-weather palette, 20% grab-bag.
    let fabric_colors: [u32; 4] = [0x1a1a2a, 0x2a1a1a, 0x1a2a1a, 0x0a0a0a];
    let weather_color = umbrella_color_by_weather(g.weather_key());
    let fabric_color = if rnd() < 0.8 { weather_color } else { pick(&fabric_colors) };
    // (the cloth as dyed nylon is after years: those dark colours, faded
    // toward grey, as dark as black cloth really is)
    let fade = |c: u32, sh: u32| ((c >> sh) & 255) * 3 / 4 + 0x24;
    let cloth = (fade(fabric_color, 16) << 16) | (fade(fabric_color, 8) << 8) | fade(fabric_color, 0);
    let umbrella = group();
    {
        use game::assets::props::prop_part_as_modelled as part;
        let cloth = lambert!({ color: cloth });
        for (name, mat) in [
            ("Real__umbrellaCanopy", cloth.clone()),
            ("Real__umbrellaStrap", cloth),
            ("Real__umbrellaShaft", phong!({ color: 0x1e1d1c, specular: 0x3a3a38, shininess: 40 })),
            ("Real__umbrellaHandle", phong!({ color: 0x5a3c26, specular: 0x2a2018, shininess: 24 })),
            ("Real__umbrellaMetal", phong!({ color: 0x5e5c58, specular: 0x8a8884, shininess: 60 })),
        ] {
            if let Some(geo) = part(name) {
                umbrella.add(&mesh(geo, mat));
            }
        }
    }
    // Stood on its ferrule, leaning toward the wall (local -z) till its crook
    // touches it, a little sideways sway; yaw applied last.
    umbrella.set_rotation_order(EulerOrder::YXZ);
    umbrella.rotation().x = -UMBRELLA_LEAN;
    umbrella.rotation().z = -0.05 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.232);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.232);
    if c.dx.abs() == 1 {
        umbrella.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        umbrella.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        umbrella.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        umbrella.position().set(wall_x + (rnd() - 0.5) * 1.0, 0, wall_z);
    }
    g.world.add(&umbrella);
    // 30% chance: the wet it dripped, dried to a mark round its ferrule
    if rnd() < 0.30 {
        let r = 0.08 + rnd() * 0.05;
        let stain = mesh(
            plane_geo!(r * 2.8, r * 1.6),
            lambert!({ color: 0xffffff, map: &umbrella_drip_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        stain.rotation().x = -PI / 2.0;
        stain.rotation().z = rnd() * PI * 2.0;
        let uy = umbrella.rot().y;
        let fx = -uy.sin();
        let fz = -uy.cos();
        let up = umbrella.pos();
        stain.position().set(up.x + fx * 0.03, 0.003, up.z + fz * 0.03);
        g.world.add(&stain);
    }
}

/// Umbrella colour per weather
fn umbrella_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a1218, 0x282010, 0x2a0a0a, 0x3a3020, 0x1a1a2a, 0x1a1a1a)
}

/// How far the umbrella leans back, radians: its crook's top (0.887 m up,
/// 11.5 mm round) just touches a wall 0.232 m from its ferrule
const UMBRELLA_LEAN: f64 = 0.2489;

/// What an umbrella stood dripping leaves: a soft, uneven dark mark, its
/// rim darker where it dried (transparent round it)
fn umbrella_drip_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for k in 0..6 {
            let (x, y) = (w / 2.0 + (rnd() - 0.5) * w * 0.3, h / 2.0 + (rnd() - 0.5) * h * 0.3);
            let rr = w * (0.16 + rnd() * 0.16) * if k == 0 { 1.5 } else { 1.0 };
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(14,22,26,0.30)");
            gr.add_color_stop(0.8, "rgba(14,22,26,0.36)");
            gr.add_color_stop(1, "rgba(14,22,26,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, 7);
            ctx.fill();
        }
    })
}
