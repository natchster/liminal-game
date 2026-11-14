//! `place_skull_plaques`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::mount_on_wall;
use placers_kit::place::salt;
use placers_kit::place::wall_candidates;

// Mounted animal skull on a plaque (wall): 2.5% chance. A roe deer's
// skull, boiled and bleached (the European mount: the whole skull bar the
// jaw, its palate to the board, its nose down, its frontal to the room)
// on a carved oak shield or an oval board (the old plaque's shape draw); a
// buck's with its pedicles and the antlers they carried (family `lodge_trophies_and_parlour_ornaments`'s),
// or a doe's (the old fox and raven); a small brass plate under it. The
// bone ivory (the old bone colour's pick), yellowed toward the occiput and
// the antlers' sap, grimed in its sutures and pits, dust on all of it.
pub(crate) fn place_skull_plaques(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old plaque wood's pick: the oak's stain
    let plaque_colors = [0x2a1810, 0x402015, 0x1a0805, 0x502010];
    let stain = match choice(&plaque_colors) {
        0x2a1810 => [0.30, 0.20, 0.12],
        0x402015 => [0.36, 0.24, 0.14],
        0x1a0805 => [0.22, 0.14, 0.08],
        _ => [0.40, 0.27, 0.15],
    };
    // the old bone colour's pick: how far it has yellowed
    let bone_colors = [0xe0d8c0, 0xd8d0b8, 0xf0e8d0];
    let bone = match choice(&bone_colors) {
        0xe0d8c0 => [0.70, 0.65, 0.54],
        0xd8d0b8 => [0.64, 0.58, 0.46],
        _ => [0.75, 0.72, 0.63],
    };
    let sp = group();
    let is_shield_shape = rnd() < 0.5;
    // the old fox, deer and raven: a doe's skull, a buck's, a doe's
    let variant = (rnd() * 3.0).floor() as i32;
    let buck = variant == 1;
    mount_on_wall(g, &sp, &c, 0.006, 1.70, 0.4);
    sp.rotation().z = (rnd() - 0.5) * 0.010;
    let p = sp.pos();
    let k = salt(p.x, p.z, 29);
    let board = if is_shield_shape { "Real__skullPlaqueShield" } else { "Real__skullPlaqueOval" };
    put(&sp, board, &phong!({ map: &oak_tex(k, stain), specular: 0x141210, shininess: 16 }));
    let skull = if buck { "Real__skullPlaqueBuck" } else { "Real__skullPlaqueDoe" };
    put(&sp, skull, &lambert!({ map: &skull_bone_tex(k, bone, buck) }));
    put(&sp, "Real__skullPlaqueTeeth", &phong!({ color: 0x4e4232, specular: 0x1a1814, shininess: 30 }));
    put(&sp, "Real__skullPlaquePlate", &phong!({ map: &plate_tex(k), specular: 0x5a4a2c, shininess: 40 }));
    if buck {
        let antlers = group();
        antlers.position().set(SKULL_ANTLERS[0], SKULL_ANTLERS[1], SKULL_ANTLERS[2]);
        antlers.rotation().x = SKULL_ANTLER_TURN;
        put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
        put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
        sp.add(&antlers);
    }
    g.world.add(&sp);
}

/// The burrs' bases on the skull plaque (`victorian/things_on_a_wall_and_one_box.py` `SP_O`), and the antlers'
/// turn about x there: the skull's (its frontal to the room, 90 degrees)
/// less the antlers' lean off the frontal (52 degrees), as family `lodge_trophies_and_parlour_ornaments`'s
const SKULL_ANTLERS: [f64; 3] = [0.0, 0.072, 0.0883];

const SKULL_ANTLER_TURN: f64 = 0.663_225;

/// Oak under an old stain and varnish (family `lodge_trophies_and_parlour_ornaments`'s): the grain's lines,
/// the rays' pale flecks across them, the stain uneven, paler where hands
/// rubbed the edges, the varnish crazed, dust in the pores
fn oak_tex(salt: u32, base: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 176, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, base.map(|k| k * 0.6), sstep(0.45, 0.72, grain) * 0.5);
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, base.map(|k| k * 1.4), sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, base.map(|k| k * 1.25), (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, base.map(|k| k * 1.35), (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The skull's bone as its face is laid (u across, v up the plaque: the
/// occiput at the top, the nose at the foot): ivory, yellowed toward the
/// top and (a buck's) brown up round the pedicles where the antlers' sap
/// ran, the sutures dark wandering lines (the coronal across behind the
/// pedicles, the frontonasal ahead of the orbits, the midline), grime in
/// its pits, the orbits' edges darker, dust
fn skull_bone_tex(salt: u32, base: [f64; 3], buck: bool) -> Tex {
    canvas_tex_opt(96, 224, false, move |ctx, w, h| {
        let seed = salt ^ 0x0b1;
        fill(ctx, w, h, |u, v| {
            let mut c = base;
            let n = fbm(seed, u * 6.0, v * 14.0, 3);
            c = mix(c, base.map(|k| k * 0.82), sstep(0.4, 0.75, n) * 0.7);
            c = mix(c, [0.55, 0.46, 0.32], sstep(0.55, 1.0, v) * 0.35);
            if buck {
                let ped = (-(((u - 0.26).dpowi(2) + (v - 0.68).dpowi(2)) / 0.004)).dexp()
                    + (-(((u - 0.74).dpowi(2) + (v - 0.68).dpowi(2)) / 0.004)).dexp();
                c = mix(c, [0.40, 0.30, 0.19], ped * 0.7);
            }
            let x = (u - 0.5) * 0.083;
            let y = v * 0.2006 - 0.064;
            let wob = |s: u32, t: f64| (fbm(seed ^ s, t * 140.0, 0.5, 2) - 0.5) * 0.004;
            let d1 = (y - 0.082 - wob(21, x)).abs();
            let d2 = (y - 0.032 - wob(23, x)).abs();
            let d3 = if y > -0.03 { (x - wob(25, y)).abs() } else { 1.0 };
            let sut = 1.0 - sstep(0.0, 0.0009, d1.min(d2).min(d3));
            c = mix(c, [0.36, 0.30, 0.22], sut * 0.45);
            let pit = vn(seed ^ 3, u * 70.0, v * 160.0);
            c = mix(c, [0.50, 0.44, 0.34], sstep(0.84, 0.95, pit) * 0.35);
            let side = (u - 0.5).abs();
            c = mix(c, [0.30, 0.25, 0.19], sstep(0.40, 0.5, side) * 0.5);
            let d = vn(seed ^ 5, u * 70.0, v * 150.0);
            c = mix(c, [0.62, 0.60, 0.55], sstep(0.82, 0.95, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam; family `lodge_trophies_and_parlour_ornaments`'s): the burr and the pearls
/// dark with old sap, their crowns rubbed paler, the gutters darker
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

/// The brass plate: the day it was shot engraved on it, the engraving
/// grimed dark, the brass tarnished brown at its edges
fn plate_tex(salt: u32) -> Tex {
    canvas_tex_opt(168, 48, false, move |ctx, w, h| {
        let seed = salt ^ 0x0e3;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.46, 0.37, 0.19];
            c = mix(c, [0.24, 0.18, 0.10], sstep(0.45, 0.85, fbm(seed, u * 6.0, v * 2.0, 3)) * 0.6);
            let edge = (u.min(1.0 - u) * 3.5).min(v.min(1.0 - v));
            c = mix(c, [0.20, 0.15, 0.09], (1.0 - sstep(0.0, 0.12, edge)) * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
        let months = ["V", "VI", "VII", "VIII", "IX", "X"];
        let day = 1 + salt % 28;
        let date = format!("{}. {}. 19{}", day, months[(salt / 28 % 6) as usize], 52 + salt / 168 % 17);
        ctx.set_fill_style("rgba(28,20,12,0.85)");
        ctx.set_font("italic 22px serif");
        ctx.set_text_align("center");
        ctx.fill_text(&date, w / 2.0, h * 0.68);
    })
}
