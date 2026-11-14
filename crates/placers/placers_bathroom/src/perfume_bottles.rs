//! `place_perfume_bottles`
use crate::shared::salt;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t03_put;
use placers_kit::noise::fbm;
use placers_kit::noise::line;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Perfume bottle: 5% chance per level. A 30 ml rectangular flacon
// (Blender batch `t03`): heavy-bottomed glass, the scent left in it gone
// darker and down to a fifth, a half or three quarters (a dried ring where it
// stood higher), its dip tube; a crimped spray pump and the square overcap on
// it, or off and lying by it; or (the old 40% bulb) a refill fitted with an
// atomizer's gilt collar and a netted rubber bulb on its hose, the bulb and
// its tassel lying on the floor. The old colours are the scent's.
pub(crate) fn place_perfume_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let glass_colors = [0xe0c8a0, 0xd0a0c0, 0xa0c0e0, 0xf0d060, 0xe08080];
    let col = choice(&glass_colors);
    let kind = glass_colors.iter().position(|&k| k == col).unwrap_or(0);
    let perfume = group();
    let has_bulb = rnd() < 0.4;
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = salt(px, pz);
    // how much is left (no draw: from where it stands): a fifth now and
    // then, mostly a half or three quarters
    let level = [0, 1, 1, 2, 2][(salt >> 7) as usize % 5];
    let (lvl_name, lvl_y) =
        [("Real__perfumeFlaconScentLow", 0.0199), ("Real__perfumeFlaconScentMid", 0.0340), ("Real__perfumeFlaconScentHigh", 0.0480)][level];
    // the thick bottom: the solid the floor holds up
    t03_put(
        &perfume,
        "Real__perfumeFlaconGlassBase",
        &phong!({ color: 0x4e5a52, specular: 0xb0b0b0, shininess: 110, transparent: true, opacity: 0.6 }),
    );
    if let Some(m) = t03_put(
        &perfume,
        lvl_name,
        &phong!({ color: T03_SCENT[kind], specular: 0x707070, shininess: 90, transparent: true, opacity: 0.66, depthWrite: false }),
    ) {
        m.set_render_order(2);
    }
    if let Some(m) = t03_put(
        &perfume,
        "Real__perfumeFlaconDipTube",
        &phong!({ color: 0xc8ccc8, specular: 0x505050, shininess: 60, transparent: true, opacity: 0.5, depthWrite: false }),
    ) {
        m.set_render_order(2);
    }
    // the glass: its ring where the scent stood higher, dust, fingers
    let tide = (lvl_y + 0.006 + 0.010 * f64::from((salt >> 11) % 8) / 8.0 - 0.008) / 0.070;
    if let Some(m) = t03_put(&perfume, "Real__perfumeFlaconGlass", &glass_mat(&flacon_glass_tex(salt, tide))) {
        m.set_render_order(3);
    }
    let metal = |k: u32| match k % 3 {
        0 => phong!({ color: 0x6a5530, specular: 0xa89060, shininess: 50 }),
        1 => phong!({ color: 0x1c1b1c, specular: 0x505050, shininess: 70 }),
        _ => phong!({ color: 0x5c5e60, specular: 0xa0a0a0, shininess: 60 }),
    };
    if has_bulb {
        // an atomizer's head on the flacon's neck (6 mm higher than the
        // pressed glass's it was modelled on)
        let gilt = phong!({ map: &tarnish_tex(salt), color: 0x5e4822, specular: 0xa88c50, shininess: 45 });
        if let Some(m) = t03_put(&perfume, "Real__perfumeAtomizerCollar", &gilt) {
            m.position().y = 0.006;
        }
        let silk = T03_SILK[(salt >> 3) as usize % 4];
        let net = lambert!({ map: &net_tex(), color: silk });
        t03_put(&perfume, "Real__perfumeAtomizerBulb", &net);
        t03_put(&perfume, "Real__perfumeAtomizerHoseFlacon", &lambert!({ color: silk }));
        t03_put(&perfume, "Real__perfumeAtomizerTassel", &lambert!({ color: silk }));
    } else {
        let pump = metal(salt >> 5);
        t03_put(&perfume, "Real__perfumeFlaconPump", &pump);
        let cap_mat = metal(salt >> 5);
        if salt % 3 == 0 {
            // the cap off, lying on its side by it
            if let Some(geo) = prop_part_as_modelled("Real__perfumeFlaconCap") {
                let geo = geo.duplicate();
                geo.rotate_z(PI / 2.0);
                geo.rotate_y(0.4 + f64::from(salt % 7) * 0.2);
                let bb = geo.compute_bounding_box();
                geo.translate(0.048 - (bb.min.x + bb.max.x) / 2.0, -bb.min.y, 0.018 - (bb.min.z + bb.max.z) / 2.0);
                perfume.add(&mesh(geo, &cap_mat));
            }
        } else {
            t03_put(&perfume, "Real__perfumeFlaconCap", &cap_mat);
        }
    }
    perfume.position().set(px, 0, pz);
    perfume.rotation().y = rnd() * PI * 2.0;
    g.world.add(&perfume);
}

/// The scent left in it, darkened (sRGB): amber, rose, pale blue, gold, coral
const T03_SCENT: [u32; 5] = [0xb07a2a, 0xa85a6c, 0x6a8cac, 0xb89a2a, 0xb25a4c];

/// The bulb's silk net: crimson, faded rose, black, old gold
const T03_SILK: [u32; 4] = [0x6a1a26, 0x8a5a60, 0x262426, 0x7a6430];

/// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
/// Fresnel, its dirt from the map's grey
fn glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 240, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "perfume_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.45;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.0;
        f[9] = 0.22;
        f[10] = 0.20;
        f[11] = 0.17;
        f[12] = 0.0;
        f[13] = 0.08;
    }
    m
}

/// The flacon's glass (v up its side): dust heavier toward its shoulder,
/// the brown ring the scent dried at when it stood at `tide`, a
/// thumb's print
fn flacon_glass_tex(salt: u32, tide: f64) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xf1ac;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 6.0, v * 9.0, 3);
            let mut g = 0.10 + 0.22 * d + 0.25 * sstep(0.7, 1.0, v);
            g += 0.5 * line(v - tide - 0.01 * (u * 18.0).sin(), 0.018);
            let (fx, fy) = (u - 0.38, v - 0.40);
            let fr = (fx * fx / 0.012 + fy * fy / 0.02).sqrt();
            g += 0.25 * sstep(1.0, 0.6, fr) * (0.5 + 0.5 * (fr * 60.0 + fy * 20.0).sin());
            let g = g.clamp(0.0, 1.0);
            [g, g, g, 1.0]
        });
    })
}

/// Gilt brass tarnished dark in its knurls and patches (a grey)
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(32, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x7a2;
        fill(ctx, w, h, |u, v| {
            let g = 0.65 + 0.35 * sstep(0.35, 0.7, fbm(seed, u * 5.0, v * 5.0, 3));
            [g, g * 0.97, g * 0.92, 1.0]
        });
    })
}

/// The bulb's knotted silk net over its rubber: a diamond mesh (a grey)
fn net_tex() -> Tex {
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let (a, b) = ((u + v) * 9.0, (u - v) * 9.0);
            let da = (a - a.round()).abs();
            let db = (b - b.round()).abs();
            let cord = line(da, 0.16).max(line(db, 0.16));
            let knot = line(da, 0.2) * line(db, 0.2);
            let g = 0.45 + 0.55 * cord.max(knot * 1.5).min(1.0);
            [g, g, g, 1.0]
        });
    })
}
