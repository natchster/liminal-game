//! `place_paper_garlands`: Paper garland: 7% chance per level. A tissue paper tassel garland (Blender batch `t12`): tassels of fringed tissue, each twisted into a loop on a jute twine swagged between two pieces of tape on the ceiling; the tissue faded and limp, the strands hanging in clumps, some torn short, a tassel or two gone from the twine.
use crate::shared::TISSUE;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::hash;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

/// Paper garland: 7% chance per level. A tissue paper tassel garland
/// (Blender batch `t12`): tassels of fringed tissue, each twisted into a
/// loop on a jute twine swagged between two pieces of tape on the ceiling;
/// the tissue faded and limp, the strands hanging in clumps, some torn
/// short, a tassel or two gone from the twine.
pub(crate) fn place_paper_garlands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let g_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0, 0xf0ece4];
    let garland = group();
    let span_len = 1.4 + rnd() * 1.0;
    let n_dots = 10 + (rnd() * 6.0).floor() as i32;
    // the old cut-outs' colours: the tassels', one each
    let colours: Vec<usize> = (0..n_dots)
        .map(|_| {
            let c = pick(&g_colors);
            g_colors.iter().position(|&s| s == c).unwrap_or(0)
        })
        .collect();
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    garland.position().set(px, 0, pz);
    garland.rotation().y = rnd() * PI * 2.0;
    let salt = salt_of(px, pz);
    // the twine of the span nearest the old one, the whole garland scaled to
    // it (within 11%)
    let kind = (0..3).min_by(|&a, &b| (GT_SPANS[a] - span_len).abs().total_cmp(&(GT_SPANS[b] - span_len).abs())).unwrap_or(0);
    let span = GT_SPANS[kind];
    let swag = group();
    swag.position().y = WALL_H;
    swag.scale().set_scalar(span_len / span);
    garland.add(&swag);
    put(&swag, ["Real__garlandTwineS", "Real__garlandTwineM", "Real__garlandTwineL"][kind], &lambert!({ color: 0x7a6a50 }));
    let tape = lambert!({ color: 0x8e7b58 });
    for sx in [-1.0, 1.0] {
        let t = group();
        t.position().set(sx * (span / 2.0 + 0.016), -0.0031, 0);
        t.rotation().y = PI / 2.0;
        swag.add(&t);
        if let Some(m) = put(&t, "Real__posterTape", &tape) {
            m.rotation().x = PI / 2.0;
        }
    }
    let a = GT_CAT[kind];
    let n = colours.len();
    for (i, &k) in colours.iter().enumerate() {
        let h = hash(salt, i as i32, 7);
        if h < 0.10 {
            continue; // fallen off the twine long ago
        }
        let tx = ((i as f64 + 0.5) / n as f64 - 0.5) * span;
        let ty = a * ((tx / a).cosh() - (span / (2.0 * a)).cosh()) - GT_DROP;
        let tg = group();
        tg.position().set(tx, ty, 0);
        tg.rotation().y = (hash(salt, i as i32, 8) - 0.5) * 0.6 + if h > 0.55 { PI } else { 0.0 };
        swag.add(&tg);
        let which =
            ["Real__garlandTasselA", "Real__garlandTasselB", "Real__garlandTasselC"][((h - 0.10) / 0.9 * 3.0).floor().min(2.0) as usize];
        put(&tg, which, &lambert!({ color: TISSUE[k], side: DoubleSide }));
    }
    g.world.add(&garland);
}

// ------------------------------------------------------ the tassel garland
/// The garland as modelled (`party/what_a_party_room_leaves.py` GT_*): the twine's spans, each one's
/// catenary, and how far under the ceiling its ends run
const GT_SPANS: [f64; 3] = [1.55, 1.90, 2.25];

const GT_CAT: [f64; 3] = [1.490_950, 1.859_480, 2.228_835];

const GT_DROP: f64 = 0.004;
