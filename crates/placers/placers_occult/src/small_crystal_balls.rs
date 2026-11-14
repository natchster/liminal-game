//! `place_small_crystal_balls`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small crystal ball: 5% chance per level. A glass ball 80 mm across
// in the cupped top of a turned, dark-stained hardwood stand: the room seen
// in its glass by Fresnel, a faint tint in the glass (by the old colour),
// dust settled on its crown and the smears of hands; the stand's stain worn
// through on its edges. No light of its own (the old inner light and its
// spark are gone; their phase's draw is kept).
pub(crate) fn place_small_crystal_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let ball_colors: [u32; 4] = [0xc0e0f0, 0xe0d0f0, 0xd8e8d8, 0xf0e0d0];
    let tint = pick_hex(&ball_colors);
    // the old inner light's shimmer phase (no light: glass does not shine)
    let _ = rnd();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 0xba1);
    let ball = group();
    if let Some(m) = put(&ball, "Real__crystalBallGlass", &ball_glass_mat(&ball_dust_tex(salt), tint)) {
        m.set_render_order(2);
    }
    put(&ball, "Real__crystalBallStand", &phong!({ map: &stand_tex(salt), specular: 0x241c14, shininess: 22 }));
    ball.position().set(px, 0, pz);
    ball.rotation().y = turn;
    g.world.add(&ball);
}

/// Glass as the windows' and t09's skull's (`Patch::Glass`): the room seen
/// in it by Fresnel, bright at its rim, clear through its middle; 80 mm of
/// soda glass greying what is seen through it a little, the old colour a
/// faint tint in it; its dust lit as dust
fn ball_glass_mat(map: &Tex, tint: u32) -> Mat {
    // the old colour, mostly gone: a glass's own faint tint
    let t = |c: u32, s: u32| 0.82 + 0.18 * f64::from((c >> s) & 0xff) / 255.0;
    let col = (((t(tint, 16) * 255.0) as u32) << 16) | (((t(tint, 8) * 255.0) as u32) << 8) | ((t(tint, 0) * 255.0) as u32);
    let m = phong!({ color: col, specular: 0x9a9a9a, shininess: 240, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "crystal_ball".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.5;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 0.55;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // its body: clear glass, a little grey through 8 cm
        f[12] = 0.18;
        // a solid's two faces and its inside
        f[13] = 0.12;
    }
    m
}

/// Dust on a glass ball that stood on a floor (grey: how much; laid round
/// it, v up): heavier toward its crown where it settles, smears where it
/// was handled round its middle
fn ball_dust_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c2;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 9.0, v * 5.0, 3) * 0.6 + vn(seed ^ 3, u * 40.0, v * 20.0) * 0.4;
            let smear = (fbm(seed ^ 5, u * 3.0, v * 9.0, 2) - 0.58).max(0.0) * 1.6 * (1.0 - (v - 0.5).abs() * 2.0);
            let top = sstep(0.55, 1.0, v) * 0.35;
            let g = (0.08 + 0.32 * d * (0.4 + v) + smear + top).min(1.0);
            [g, g, g, 1.0]
        });
    })
}

/// The stand's wood (laid round it, v up): walnut stained dark, its grain
/// running round, the stain rubbed through to the paler wood on its rims,
/// dust on its top
fn stand_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x7d1;
        fill(ctx, w, h, |u, v| {
            let grain = vn(seed, u * 60.0, v * 3.0) * 0.6 + fbm(seed ^ 2, u * 9.0, v * 9.0, 2) * 0.4;
            let mut c = mix([0.17, 0.11, 0.07], [0.11, 0.07, 0.045], grain);
            // the rims (the foot's edge and the top's): worn to the wood
            let rim = (1.0 - sstep(0.0, 0.12, v)).max(sstep(0.82, 0.95, v) * (1.0 - sstep(0.95, 1.0, v)));
            c = mix(c, [0.36, 0.25, 0.16], rim * 0.55 * vn(seed ^ 4, u * 30.0, v * 4.0));
            c = mix(c, [0.40, 0.38, 0.35], sstep(0.85, 1.0, v) * 0.25);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
