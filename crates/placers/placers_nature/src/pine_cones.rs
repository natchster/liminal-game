//! `place_pine_cones`
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_kind_tex {
    ($n:expr_2021, $k:expr_2021, $make:expr_2021) => {{
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; $n]); }
        let k = ($k).min($n - 1);
        T.with(|t| {
            if let Some(x) = &t.borrow()[k] {
                return x.clone();
            }
            let tex = $make(k);
            t.borrow_mut()[k] = Some(tex.clone());
            tex
        })
    }};
}

// Pine cone: 6% chance per level. A Scots pine's cone, open (Blender
// family `a_garden_and_a_wood_indoors`): its woody scales spread from the axis, their rhombic ends
// keeled with the small umbo; weathered grey-brown, the scales' hidden faces
// darker.
pub(crate) fn place_pine_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cone_colors = [0x604020, 0x503018, 0x705030, 0x402010];
    // the old colour's draw: how weathered it is (brown, darker, grey and
    // dry, dark with damp)
    let col = choice(&cone_colors);
    const TINT: [(u32, u32); 4] = [(0xffffff, 0xffffff), (0xd8ccc0, 0xd8d0c8), (0xf0e8e0, 0xffffff), (0xa89a8c, 0xa8a098)];
    let (ts, ta) = TINT[cone_colors.iter().position(|&c| c == col).unwrap_or(0)];
    let cone = group();
    for (name, mat) in [
        ("Real__pineCone", lambert!({ map: &cone_tex(false), color: ts })),
        ("Real__pineConeApo", lambert!({ map: &cone_tex(true), color: ta })),
    ] {
        if let Some(geo) = part(name) {
            cone.add(&mesh(geo, &mat));
        }
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cone.position().set(px, 0, pz);
    cone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cone);
}

// ============================================================== pine cone
/// The cone's woody scales (box-mapped): dark red-brown, darker in their
/// grooves; its scales' ends weathered grey-brown
fn cone_tex(apo: bool) -> Tex {
    cached_kind_tex!(2, apo as usize, |k: usize| {
        canvas_tex_opt(128, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x9c0 + k as u32, u * 14.0, v * 14.0, 4);
                let c = if k == 1 {
                    let mut c = mix_unclamped([0.47, 0.40, 0.31], [0.36, 0.30, 0.23], n);
                    c = mix_unclamped(c, [0.30, 0.22, 0.15], 0.5 * sstep(0.62, 0.75, fbm(0x9c5, u * 40.0, v * 40.0, 2)));
                    c
                } else {
                    let mut c = mix_unclamped([0.36, 0.22, 0.13], [0.26, 0.16, 0.10], n);
                    c = mix_unclamped(c, [0.44, 0.33, 0.22], 0.3 * sstep(0.6, 0.8, fbm(0x9c6, u * 30.0, v * 6.0, 2)));
                    c
                };
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}
