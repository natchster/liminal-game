//! `place_garden_aprons`
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use game::visuals::pictures::Window;
use placers_kit::garden::put_uv;
use placers_kit::garden::top;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// Garden apron: 5% chance per level.
pub(crate) fn place_garden_aprons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A gardener's waist apron of cotton duck, dropped on the floor in a
    // heap, one end folded back over it, its ties loose (Blender batch `t06`,
    // draped by Blender's cloth). The old colour's draw chooses its canvas:
    // olive drab, a brown duck, khaki, the natural, a faded denim blue
    let apron_colors: [u32; 5] = [0x408040, 0x604040, 0x808030, 0xa07040, 0x506080];
    let canvas = match pick(&apron_colors) {
        0x408040 => 0x5f6448,
        0x604040 => 0x6e5640,
        0x808030 => 0x8a7e5c,
        0xa07040 => 0xb0a282,
        _ => 0x56667a,
    };
    let apron = group();
    // where it lies and how it is turned (the old draws)
    apron.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    apron.rotation().y = rnd() * PI * 2.0;
    let p = apron.pos();
    let salt = salt_of(p.x, p.z);
    let cloth = lambert!({ map: &apron_tex(rgb(canvas), salt), side: DoubleSide });
    let lay = || top(-0.17, 0.17, -0.17, 0.17);
    put_uv(&apron, "Real__apronGardenCloth", &cloth, lay());
    put_uv(&apron, "Real__apronGardenPocket", &cloth, lay());
    // the ties: cotton twill tape dyed to match, as dirty as the rest
    put_uv(&apron, "Real__apronGardenTies", &cloth, lay());
    g.world.add(&apron);
}

// ========================================================= garden aprons
/// The canvas of the apron, laid on from above over its heap: the duck's
/// colour gone pale where the light fell and with washing, its weave,
/// soil worked into it in smears and the pockets' mouths, a water stain
fn apron_tex(base: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(384, 384, false, move |ctx, w, h| {
        let seed = salt ^ 0xa960;
        let mut r = SplitMix64::new(salt, 0xa7);
        let stains: Vec<(f64, f64, f64)> = (0..7).map(|_| (r.next(), r.next(), r.range(0.03, 0.1))).collect();
        let ring = (r.range(0.25, 0.75), r.range(0.25, 0.75), r.range(0.08, 0.16));
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.34, v * 0.34);
            // the weave: 18 threads a cm, a fine cross of light and dark
            let wv = ((x * 1800.0 * PI).sin() * (y * 1700.0 * PI).sin()) * 0.5;
            let n = fbm(seed, u * 20.0, v * 20.0, 4);
            let mut c = base.map(|k| k * (0.9 + 0.16 * n + 0.04 * wv));
            // faded in patches
            c = mix(c, [0.66, 0.62, 0.54], 0.12 + 0.2 * fbm(seed ^ 1, u * 4.0, v * 4.0, 3));
            // soil: smears with ragged edges
            let mut dirt = 0.0f64;
            for &(sx, sy, sr) in &stains {
                let d = ((u - sx).hypot(v - sy) / sr) + 0.5 * (fbm(seed ^ 2, u * 30.0, v * 30.0, 3) - 0.5);
                dirt = dirt.max(sstep(1.0, 0.4, d));
            }
            c = mix(c, [0.24, 0.19, 0.14], dirt * 0.55);
            // a dried water stain: its tide line darker than its middle
            let d = (u - ring.0).hypot(v - ring.1) / ring.2 + 0.25 * (fbm(seed ^ 3, u * 12.0, v * 12.0, 2) - 0.5);
            c = mix(c, c.map(|k| k * 0.82), sstep(1.0, 0.95, d) * 0.6 + sstep(0.9, 1.0, d) * sstep(1.06, 1.0, d) * 0.5);
            // dust
            c = mix(c, [0.62, 0.6, 0.55], 0.08 + (vn(seed ^ 4, u * 300.0, v * 300.0) - 0.8).max(0.0) * 2.0);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
