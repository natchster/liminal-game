//! `place_mounted_antlers`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::salt;
use placers_kit::place::wall_tiles;

// Mounted deer antler pair: 3% chance per level. A roe buck's
// antlers, a six-pointer, on the skull plate they grew from (the frontal
// bones sawn through the orbits, boiled and bleached), screwed from behind
// to a carved oak shield (15 x 22 cm) and hung on the wall: the burrs and
// the beams' pearling dark with old sap, the tines' points rubbed white,
// the bone gone ivory, dust on all of it.
pub(crate) fn place_mounted_antlers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let (wx, wz) = (g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    let k = salt(wx, wz, 11);
    let mount = group();
    let trophy = group();
    trophy.position().y = 1.80;
    put(&trophy, "Real__roeTrophyShield", &phong!({ map: &oak_tex(k, 128, 192), specular: 0x141210, shininess: 16 }));
    put(&trophy, "Real__roeTrophySkull", &lambert!({ map: &bone_tex(k) }));
    let antlers = group();
    antlers.position().set(TROPHY_ANTLERS[0], TROPHY_ANTLERS[1], TROPHY_ANTLERS[2]);
    antlers.rotation().x = TROPHY_ANTLER_TURN;
    put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
    put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
    trophy.add(&antlers);
    mount.add(&trophy);
    mount.rotation().y = face_wall(dx, dz);
    mount.position().set(wx, 0, wz);
    g.world.add(&mount);
}

/// The antlers' place on the trophy (`victorian/lodge_trophies_and_parlour_ornaments.py` `TROPHY_SKULL`): the burrs'
/// bases above the shield's middle, out from the wall on the skull plate
const TROPHY_ANTLERS: [f64; 3] = [0.0, 0.052, 0.046];

/// The pair's turn about x on the trophy: the skull's (its frontal to the
/// room, `TROPHY_TURN` = 90 degrees) less the antlers' lean off the frontal
/// (`ROE_BETA`, 52 degrees)
const TROPHY_ANTLER_TURN: f64 = 0.663_225;

/// Quartersawn oak under an old dark stain and varnish (u across, v up the
/// shield): the grain's straight lines, the rays' pale flecks across them,
/// the stain uneven, the varnish crazed, dust in the pores
fn oak_tex(salt: u32, w: u32, h: u32) -> Tex {
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let base = [0.33, 0.22, 0.13];
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, [0.20, 0.12, 0.07], sstep(0.45, 0.72, grain) * 0.5);
            // the rays: short pale flakes across the grain
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, [0.46, 0.33, 0.20], sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            // the stain uneven, lighter where hands rubbed the edges
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.40, 0.28, 0.17], (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, [0.44, 0.31, 0.19], (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            // the varnish crazed: fine dark cracks
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            // dust in the pores and over it
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The skull plate's bone: boiled and bleached, gone ivory and yellowed in
/// its pits, stained brown up toward the pedicles where the antlers' sap
/// ran, dust
fn bone_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x0b1;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.70, 0.65, 0.54];
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            c = mix(c, [0.62, 0.53, 0.39], sstep(0.4, 0.75, n) * 0.7);
            // toward the pedicles (the top of the plate as it hangs)
            c = mix(c, [0.48, 0.37, 0.25], sstep(0.5, 0.95, v) * 0.55 * (0.6 + 0.4 * n));
            // grime in the sutures and the orbits' edges
            let su = (fbm(seed ^ 7, u * 9.0, v * 4.0, 2) - 0.5).abs();
            c = mix(c, [0.36, 0.30, 0.22], (1.0 - sstep(0.0, 0.03, su)) * 0.5);
            let pit = vn(seed ^ 3, u * 40.0, v * 40.0);
            c = mix(c, [0.52, 0.47, 0.38], sstep(0.75, 0.9, pit) * 0.6);
            let d = vn(seed ^ 5, u * 70.0, v * 70.0);
            c = mix(c, [0.62, 0.60, 0.55], sstep(0.82, 0.95, d) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam): the burr and the pearls dark with old
/// sap, their crowns rubbed paler, the gutters between them darker, the
/// beam browner and smoother toward the tines
fn antler_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x0c7;
        fill(ctx, w, h, |u, v| {
            let low = 1.0 - sstep(0.15, 0.6, v);
            let mut c = mix([0.40, 0.30, 0.20], [0.20, 0.14, 0.09], low);
            // the gutters: dark streaks along the beam
            let gut = fbm(seed, u * 34.0, v * 3.0, 3);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.55, 0.75, gut) * 0.6);
            // the pearls' crowns rubbed pale
            let pearl = vn(seed ^ 3, u * 60.0, v * 110.0);
            c = mix(c, [0.62, 0.54, 0.42], sstep(0.72, 0.9, pearl) * (0.35 + 0.4 * low));
            let d = vn(seed ^ 5, u * 90.0, v * 160.0);
            c = mix(c, [0.55, 0.52, 0.46], sstep(0.85, 0.97, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
