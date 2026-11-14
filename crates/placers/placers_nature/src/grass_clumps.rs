//! `place_grass_clumps`
use crate::shared::floor_mark;
use crate::shared::thin_lit;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;

// Grass clump: 6% chance per level. A tuft of annual meadow grass grown
// up out of a drift of silt on a damp floor (Blender family `bits_of_the_outside_indoors`): tillers of
// folded, boat-tipped blades, pale and lank from the dark, a flowering stem
// or two, the dead blades round its foot; it stirs in a draught.
pub(crate) fn place_grass_clumps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let grass_colors = [0x407030, 0x506028, 0x608040, 0x304018];
    // the old colour: how pale it has grown in the dark (yellow-green, olive,
    // pale, darker)
    let clump_color = choice(&grass_colors);
    let kc = grass_colors.iter().position(|&c| c == clump_color).unwrap_or(0);
    const BLADE: [u32; 4] = [0x8a9a5a, 0x8c9058, 0xa4a66e, 0x6c7c48];
    let grass_mat = lambert!({ color: BLADE[kc] });
    let flower_mat = lambert!({ color: 0x9a9a78 });
    let clump = group();
    let count = 6 + (rnd() * 6.0).floor() as i32;
    let mut tillers: Vec<(Obj, f64, f64)> = vec![]; // track for sway animator: (tiller, lean, phase)
    let mut first_turn = 0.0;
    for i in 0..count {
        // the old blade's draws: its height (which tiller), where it rises,
        // its turn, its lean, its sway's phase
        let h = 0.040 + rnd() * 0.050;
        let (bx, bz) = ((rnd() - 0.5) * 0.06, (rnd() - 0.5) * 0.06);
        let turn = rnd() * PI;
        let lean = (rnd() - 0.5) * 0.3;
        let phase = rnd() * PI * 2.0;
        if i == 0 {
            first_turn = turn;
        }
        let kind = (((h - 0.040) / 0.050 * 5.0).floor() as usize).min(4);
        let Some(geo) = thin_lit(TILLERS[kind].0) else { continue };
        let t = group();
        t.add(&mesh(geo, &grass_mat));
        // a flowering stem from one tiller in three (by its phase)
        if phase < PI * 2.0 / 3.0 {
            if let Some(pg) = thin_lit("Real__grassPanicle") {
                let p = mesh(pg, &flower_mat);
                p.scale().set_scalar(0.8 + 0.4 * (h - 0.04) / 0.05);
                t.add(&p);
            }
        }
        // tillers lean out from the tuft's middle a little (no further: the
        // blades' tips stay off the floor)
        t.position().set(bx, 0, bz);
        t.rotation().y = turn * 2.0;
        t.rotation().z = lean * 0.15;
        tillers.push((t.clone(), lean * 0.15, phase));
        clump.add(&t);
    }
    if let Some(dg) = thin_lit("Real__grassDead") {
        let d = mesh(dg, lambert!({ color: 0x9c8c6a }));
        d.rotation().y = first_turn * 2.0;
        clump.add(&d);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    clump.position().set(px, 0, pz);
    clump.add(&floor_mark(&silt_tex(salt_of(px, pz)), 0.16, 0.16, 0.0, 0.0, first_turn));
    g.world.add(&clump);
    // A draught stirs it (0.5 Hz, each tiller its own phase)
    g.animate(move |_g, _dt, t| {
        for (o, lean, ph) in &tillers {
            o.rotation().z = lean + (t * 0.5 + ph).sin() * 0.025;
        }
    });
}

// ================================================================== grass
/// The tillers by height (`nature/bits_of_the_outside_indoors.py` TILLERS)
const TILLERS: [(&str, f64); 5] = [
    ("Real__grassTillerA", 0.050),
    ("Real__grassTillerB", 0.065),
    ("Real__grassTillerC", 0.080),
    ("Real__grassTillerD", 0.095),
    ("Real__grassTillerE", 0.115),
];

/// The silt the grass grew in: dried damp, darker at its heart, ragged
fn silt_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w - 0.5, y / h - 0.5);
            let r = u.hypot(v) * 2.0 + 0.3 * (fbm(salt, u * 7.0 + 3.0, v * 7.0 + 3.0, 3) - 0.5);
            let grit = vn(salt ^ 0x6, x / 1.5, y / 1.5);
            let c = mix_unclamped([0.22, 0.19, 0.15], [0.36, 0.32, 0.26], sstep(0.1, 0.8, r) * 0.7 + 0.3 * grit);
            [c[0], c[1], c[2], (1.0 - sstep(0.55, 0.95, r)) * (0.75 + 0.25 * grit)]
        });
    })
}
