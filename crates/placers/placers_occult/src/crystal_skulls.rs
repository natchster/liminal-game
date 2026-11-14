//! `place_crystal_skulls`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Crystal skull: 3% chance per level. A "crystal skull" as they are
// sold: a cast glass ornament, 10.5 cm deep, its jaw fused shut and its base
// ground flat, standing on the floor; the room in its glass by Fresnel, dust
// and a hand's smears on it. It catches the light; it never makes any.
pub(crate) fn place_crystal_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old glow's pulse phase (no light: glass does not shine)
    let _ = rnd();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let skull = group();
    if let Some(m) = put(&skull, "Real__crystalSkull", &glass_mat(&glass_dust_tex(salt(px, pz, 1)))) {
        m.set_render_order(2);
    }
    skull.position().set(px, 0, pz);
    skull.rotation().y = turn;
    g.world.add(&skull);
}

/// Cast glass as the windows' and the wine glass's (`Patch::Glass`): the
/// room seen in it by Fresnel, bright at its edges, clear square on; thick
/// enough to grey what is seen through it a little; its dust lit as dust
fn glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "crystal_skull".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.55;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 0.6;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // the cast's body: grey through 8 cm of glass full of seeds
        f[12] = 0.34;
        // a solid's two faces and its inside: brighter than a pane square on
        f[13] = 0.14;
    }
    m
}

/// Dust on glass that stood on a floor (grey: how much): a haze, heavier
/// over the crown where it settles, smears where it was handled
fn glass_dust_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c1;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 7.0, v * 7.0, 3) * 0.6 + vn(seed ^ 3, u * 29.0, v * 29.0) * 0.4;
            let smear = (fbm(seed ^ 5, u * 3.0, v * 11.0, 2) - 0.58).max(0.0) * 1.6;
            let top = sstep(0.55, 1.0, v) * 0.15;
            let g = (0.16 + 0.42 * d + smear + top).min(1.0);
            [g, g, g, 1.0]
        });
    })
}
