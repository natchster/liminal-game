//! `place_deer_head_mounts`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::mount_on_wall;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Wall taxidermy deer head: 4% chance per level. A roe buck's head mount (a
// Kopfpraeparat): the hide tanned and set over its form, the neck cut short
// against a carved oak shield, the head turned a little down as if it
// looked ahead, the big ears up and out, glass eyes that catch the light,
// the black nose, the white of its chin and throat; its antlers (a
// six-pointer) on its pedicles. The coat faded and dusty, a few bald spots
// where moths have been.
pub(crate) fn place_deer_head_mounts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let k = salt(g.wx(c.x) + c.dx as f64, g.wz(c.z) + c.dz as f64, 21);
    let mount = group();
    put(&mount, "Real__roeHeadShield", &phong!({ map: &oak_tex(k, 160, 224), specular: 0x141210, shininess: 16 }));
    put(&mount, "Real__roeHeadHide", &lambert!({ map: &hide_tex(k) }));
    put(&mount, "Real__roeHeadPale", &lambert!({ map: &pale_tex(k), side: DoubleSide }));
    // the nose's leather, painted and glossed by the taxidermist: a dark
    // grey-brown with a sheen that shows its shape (a black without one
    // read as a hole straight on: M8)
    put(&mount, "Real__roeHeadNose", &phong!({ color: 0x2c2522, specular: 0x4a4642, shininess: 46 }));
    // glass eyes: they catch the light, they never make it
    put(&mount, "Real__roeHeadEyes", &phong!({ color: 0x1e140c, specular: 0x9a9a9a, shininess: 120 }));
    let antlers = group();
    antlers.position().set(HEAD_ANTLERS[0], HEAD_ANTLERS[1], HEAD_ANTLERS[2]);
    antlers.rotation().x = HEAD_ANTLER_TURN;
    put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
    put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
    mount.add(&antlers);
    mount_on_wall(g, &mount, &c, 0.005, 2.00, 0.6);
    g.world.add(&mount);
}

/// The antlers' place on the head (`victorian/lodge_trophies_and_parlour_ornaments.py` `HEAD_ANTLERS`): the burrs'
/// bases on the pedicles, on top of the skull
const HEAD_ANTLERS: [f64; 3] = [0.0, 0.080_494, 0.161_283];

/// The pair's turn about x: the head's (its nose 25 degrees down,
/// `HEAD_TURN`) less the antlers' lean back off the frontal (`ROE_BETA`,
/// 52 degrees)
const HEAD_ANTLER_TURN: f64 = -0.471_239;

/// A roe's winter coat (u across, v up the head): grizzled grey-brown, the
/// hairs' pale bands flecked through it, lying in streaks, darker over the
/// crown and down the neck's top; faded, dusty, a few bald spots where
/// moths ate the hair to the hide
fn hide_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x1d3;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.42, 0.35, 0.27];
            // darker over the top
            c = mix(c, [0.29, 0.23, 0.17], sstep(0.45, 0.95, v) * 0.7);
            // the hair: fine streaks down, each hair's band paler
            let streak = fbm(seed, u * 140.0, v * 26.0, 3);
            c = mix(c, [0.20, 0.16, 0.12], sstep(0.5, 0.75, streak) * 0.45);
            let fleck = vn(seed ^ 3, u * 220.0, v * 90.0);
            c = mix(c, [0.62, 0.56, 0.47], sstep(0.68, 0.9, fleck) * 0.45);
            // the coat's lie: broad soft waves
            let wave = fbm(seed ^ 5, u * 7.0, v * 7.0, 3);
            c = mix(c, [0.47, 0.40, 0.31], (wave - 0.5).max(0.0) * 0.45);
            // moth-eaten spots: the hair gone to the tanned skin
            let moth = fbm(seed ^ 7, u * 22.0, v * 22.0, 3);
            c = mix(c, [0.54, 0.46, 0.36], sstep(0.73, 0.76, moth) * 0.5);
            // faded and dusty
            c = mix(c, [0.55, 0.52, 0.47], 0.12);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The pale hair of the chin and the lower lip, the throat's patch and
/// the ears' insides: an old white gone cream and grey, streaked
fn pale_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x1e7;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.68, 0.65, 0.58];
            let streak = fbm(seed, u * 90.0, v * 20.0, 3);
            c = mix(c, [0.48, 0.44, 0.38], sstep(0.5, 0.8, streak) * 0.5);
            let grime = fbm(seed ^ 3, u * 6.0, v * 6.0, 3);
            c = mix(c, [0.55, 0.50, 0.41], (grime - 0.45).max(0.0) * 1.2);
            // the ears' insides (the top of the part): grey-brown, the white
            // hairs in tufts along them
            let ear = sstep(0.42, 0.5, v);
            let tuft = sstep(0.55, 0.8, fbm(seed ^ 5, u * 40.0, v * 14.0, 2));
            c = mix(c, mix([0.30, 0.26, 0.22], [0.52, 0.49, 0.44], tuft), ear);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Quartersawn oak under an old dark stain and varnish (u across, v up the
/// shield), as the antlers' shield (`mounted_antlers.rs`)
fn oak_tex(salt: u32, w: u32, h: u32) -> Tex {
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let base = [0.33, 0.22, 0.13];
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, [0.20, 0.12, 0.07], sstep(0.45, 0.72, grain) * 0.5);
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, [0.46, 0.33, 0.20], sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.40, 0.28, 0.17], (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, [0.44, 0.31, 0.19], (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam), as the antlers' trophy's
fn antler_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x0c7;
        fill(ctx, w, h, |u, v| {
            let low = 1.0 - sstep(0.15, 0.6, v);
            let mut c = mix([0.40, 0.30, 0.20], [0.20, 0.14, 0.09], low);
            let gut = fbm(seed, u * 34.0, v * 3.0, 3);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.55, 0.75, gut) * 0.6);
            let pearl = vn(seed ^ 3, u * 60.0, v * 110.0);
            c = mix(c, [0.62, 0.54, 0.42], sstep(0.72, 0.9, pearl) * (0.35 + 0.4 * low));
            let d = vn(seed ^ 5, u * 90.0, v * 160.0);
            c = mix(c, [0.55, 0.52, 0.46], sstep(0.85, 0.97, d) * 0.4);
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
