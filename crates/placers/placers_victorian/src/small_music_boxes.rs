//! `place_small_music_boxes`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep as ss;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::opaque;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_pos;
use placers_kit::place::vary;

// Small children's music box: 6% chance per level. A Swiss child's
// hand-crank music box (Blender batch `t20`): a little box of beech,
// stained walnut, mahogany or dark oak, or painted red or yellow ochre (the
// old colour's draw); its lid open, a chromolithograph pasted inside it;
// under a glass the works on their brass bedplate, the pinned cylinder,
// the steel comb, the wheel; the crank on its side with a porcelain knob.
// The stain and paint worn at its corners, the print faded, dust on the
// glass.
pub(crate) fn place_small_music_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old draws, call for call: its finish, where it stands, its turn
    let box_colors = [0xa06030, 0x8a5030, 0x603820, 0xd82060, 0xd8b040];
    let finish = pick_color(&box_colors);
    let bx = group();
    scatter_pos(g, &bx, x, z, 2.4);
    bx.rotation().y = rnd() * PI * 2.0;
    let p = bx.pos();
    let vary = vary(p.x, p.z, bx.rot().y);
    let wood = phong!({ map: &mb_finish_tex(vary, finish), specular: 0x1a1814, shininess: 18 });
    put(&bx, "Real__musicBoxCase", &wood);
    let brass = phong!({ color: 0x6e5832, specular: 0x5a4a2a, shininess: 40 });
    put(&bx, "Real__musicBoxBrass", &brass);
    put(&bx, "Real__musicBoxComb", &phong!({ color: 0x4a4a4c, specular: 0x505058, shininess: 60 }));
    if let Some(m) = put(&bx, "Real__musicBoxGlass", &pictures::glass_mat(vary, 0.6)) {
        m.set_render_order(2);
    }
    put(&bx, "Real__musicBoxKnob", &phong!({ color: 0xc8c0ae, specular: 0x3a3a36, shininess: 50 }));
    // the lid open on its hinges, the print inside it
    let lid = group();
    lid.position().set(0, MB_HB, -MB_D * 0.5);
    lid.rotation().x = -MB_OPEN;
    bx.add(&lid);
    put(&lid, "Real__musicBoxLid", &wood);
    let pics = ["bot_rose", "land_pond", "bot_poppy", "garden_family"];
    let pic = pics[(hash(vary, 5, 5) * pics.len() as f64) as usize % pics.len()];
    put(&lid, "Real__musicBoxPaper", &lambert!({ map: &pictures::print_tex(pic, MB_PRINT, 2400.0, Age::engraving(vary), vary) }));
    g.world.add(&bx);
}

/// The box (t20.py `MB_*`): its depth, the case's height, the lid's
/// opening (radians), the print inside the lid (m)
const MB_D: f64 = 0.070;

const MB_HB: f64 = 0.032;

const MB_OPEN: f64 = 1.80;

const MB_PRINT: (f64, f64) = (0.088, 0.058);

/// The box's finish as its faces are mapped, for the old five colours:
/// beech stained walnut, mahogany or dark oak, or painted (red, yellow
/// ochre) over the wood; worn to the bare beech at its corners and edges,
/// grimed where it was held, dust
fn mb_finish_tex(vary: u32, c: u32) -> Tex {
    let (base, painted) = match c {
        0xa06030 => ([0.36, 0.22, 0.12], false),
        0x8a5030 => ([0.30, 0.14, 0.08], false),
        0x603820 => ([0.17, 0.10, 0.06], false),
        0xd82060 => ([0.48, 0.14, 0.12], true),
        _ => ([0.62, 0.46, 0.20], true),
    };
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x3b0;
        fill(ctx, w, h, |u, v| {
            let mut o = if painted {
                let n = fbm(seed, u * 8.0, v * 8.0, 3);
                base.map(|c| c * (0.9 + 0.15 * n))
            } else {
                wood(seed, u, v, base, 10.0, 40.0)
            };
            // worn to the beech at the edges and the corners
            let e = 1.0 - ss(0.02, 0.10, u.min(1.0 - u).min(v).min(1.0 - v));
            let wear = e * ss(0.5, 0.7, fbm(seed ^ 3, u * 20.0, v * 20.0, 3));
            o = mix(o, [0.58, 0.45, 0.30], wear * if painted { 0.8 } else { 0.4 });
            let dust = 0.08 + 0.12 * fbm(seed ^ 5, u * 6.0, v * 6.0, 3);
            opaque(mix(o, [0.43, 0.40, 0.35], dust))
        });
    })
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Wood seen along its grain (u along it): the growth rings' streaks and
/// the figure, `base` its colour (sRGB)
fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
    let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
    let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
    let streak = fbm(seed, u * along * 0.2, v * across, 3);
    let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
    base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
}
