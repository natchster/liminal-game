//! `place_small_perfumes`: Small perfume bottle: 8% chance per level. A dressing table's perfume atomizer of pressed glass (Blender family `washroom_shelf_and_dressing_table`): a flat fluted bottle, the scent left in it gone darker and down (a dried ring where it stood higher), a gilt collar with its spray head, a rubber bulb in a knotted silk net on its covered hose, lying on the floor with its tassel. The old colours are the scent's: cologne, lavender, rose, lily of the valley, champagne.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::line;
use placers_kit::noise::sstep;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small perfume bottle: 8% chance per level. A dressing table's
/// perfume atomizer of pressed glass (Blender family `washroom_shelf_and_dressing_table`): a flat fluted
/// bottle, the scent left in it gone darker and down (a dried ring where it
/// stood higher), a gilt collar with its spray head, a rubber bulb in a
/// knotted silk net on its covered hose, lying on the floor with its tassel.
/// The old colours are the scent's: cologne, lavender, rose, lily of the
/// valley, champagne.
pub(crate) fn place_small_perfumes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_colors = [0xd0e0f0, 0xe0d0f0, 0xf0d8d8, 0xd8f0e0, 0xf0e8d0];
    let col = pick(&glass_colors);
    let kind = glass_colors.iter().position(|&k| k == col).unwrap_or(0);
    let perfume = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let salt = salt(px, pz);
    let put = |name: &str, mat: &Mat| -> Option<Obj> {
        let m = mesh(prop_part_as_modelled(name)?, mat);
        perfume.add(&m);
        Some(m)
    };
    // how much is left (no draw: from where it stands)
    let level = [0, 1, 1, 2, 2][(salt >> 7) as usize % 5];
    let (lvl_name, lvl_y) =
        [("Real__perfumeAtomizerScentLow", 0.0183), ("Real__perfumeAtomizerScentMid", 0.0316), ("Real__perfumeAtomizerScentHigh", 0.0449)]
            [level];
    // the thick bottom: the solid the floor holds up
    put(
        "Real__perfumeAtomizerGlassBase",
        &phong!({ color: 0x4e5a52, specular: 0xb0b0b0, shininess: 110, transparent: true, opacity: 0.6 }),
    );
    if let Some(m) = put(
        lvl_name,
        &phong!({ color: SCENT[kind], specular: 0x707070, shininess: 90, transparent: true, opacity: 0.66, depthWrite: false }),
    ) {
        m.set_render_order(2);
    }
    // the glass (its walls from 7 mm up to the neck's top, 72 mm): the ring
    // where the scent stood higher, dust, fingers
    let tide = (lvl_y + 0.006 + 0.010 * f64::from((salt >> 11) % 8) / 8.0 - 0.007) / 0.065;
    if let Some(m) = put("Real__perfumeAtomizerGlass", &glass_mat(&glass_tex(salt, tide))) {
        m.set_render_order(3);
    }
    put("Real__perfumeAtomizerCollar", &phong!({ map: &tarnish_tex(salt), color: 0x5e4822, specular: 0xa88c50, shininess: 45 }));
    let silk = SILK[(salt >> 3) as usize % 4];
    put("Real__perfumeAtomizerBulb", &lambert!({ map: &net_tex(), color: silk }));
    put("Real__perfumeAtomizerHose", &lambert!({ color: silk }));
    put("Real__perfumeAtomizerTassel", &lambert!({ color: silk }));
    perfume.position().set(px, 0, pz);
    perfume.rotation().y = rnd() * PI * 2.0;
    g.world.add(&perfume);
}

/// The scent left in it, darkened (sRGB): cologne, lavender, rose, lily of
/// the valley, champagne
const SCENT: [u32; 5] = [0x7a96b0, 0x8a78a8, 0xb07884, 0x86a07a, 0xb89a62];

/// The bulb's silk net: crimson, faded rose, black, old gold
const SILK: [u32; 4] = [0x6a1a26, 0x8a5a60, 0x262426, 0x7a6430];

/// A placed thing's own number, from where it stands (no draw)
fn salt(px: f64, pz: f64) -> u32 {
    ((px * 1013.0).floor() as i64 ^ ((pz * 733.0).floor() as i64).rotate_left(17)) as u32
}

/// Clear glass as the windows' (`Patch::Glass`): the room seen in it by
/// Fresnel, its dirt from the map's grey. Thick pressed glass on the
/// floor gives back the dim room, not the lit floor round it: its edges
/// and its flutes darker than what is seen through it, and a film of
/// dust on it (lit as the room's, it vanished on a pale floor: M8)
fn glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 240, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "atomizer_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.16;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.35;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.16;
        f[12] = 0.0;
        f[13] = 0.11;
    }
    m
}

/// The glass (v up its side): dust heavier toward its shoulder, the
/// brown ring the scent dried at when it stood at `tide`, a thumb's print
fn glass_tex(salt: u32, tide: f64) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xa701;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 6.0, v * 9.0, 3);
            let mut g = 0.12 + 0.24 * d + 0.25 * sstep(0.75, 1.0, v);
            g += 0.5 * line(v - tide - 0.01 * (u * 18.0).dsin(), 0.018);
            let (fx, fy) = (u - 0.62, v - 0.45);
            let fr = (fx * fx / 0.012 + fy * fy / 0.02).sqrt();
            g += 0.25 * sstep(1.0, 0.6, fr) * (0.5 + 0.5 * (fr * 60.0 + fy * 20.0).dsin());
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
