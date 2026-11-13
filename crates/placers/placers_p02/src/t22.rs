//! Batch `t22`'s cotton swabs, made real (`docs/design/PROPS.md` P3; the Blender
//! batch `assets/src/blender/props/t22.py`, its notes `t22.notes.md`):
//! 3 in paper-stick swabs spilled from their box. It keeps the old placer's
//! draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use std::cell::OnceCell;

/// A swab end to end (`t22.py` `SWAB_L`, `SWAB_TUFT_L`), metres
const SWAB_L: f64 = 0.0762;
const TUFT_L: f64 = 0.013;
/// The four swabs (`Real__swabA..D`): new, its stick kinked, used (the +x
/// tuft pressed and stained), a tuft pulled off
const SWABS: [&str; 4] = ["Real__swabA", "Real__swabB", "Real__swabC", "Real__swabD"];

// Scattered cotton swabs: 10% chance per level, 4-8 of them spilled
// where a box of them was dropped: 3 in double-tipped swabs (rolled white
// paper sticks 2.2 mm across, cotton wound on each end), gone grey with
// dust; one with its stick kinked, one used, one with a tuft pulled off.
pub(crate) fn place_cotton_swabs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let Some(kit) = swab_kit() else { return };
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    // Two by two, a swab and the next are the same (the odd last one too):
    // none is the only one of its kind, so every swab is drawn instanced, as
    // the old ones were
    let mut kind: Option<usize> = None;
    for i in 0..count {
        let sx = (rnd() - 0.5) * 0.12;
        let sz = (rnd() - 0.5) * 0.12;
        let yaw = rnd() * PI * 2.0;
        // which swab: from where the first of two fell (no draw); most are
        // as they came
        if (i % 2 == 0 && i + 1 < count) || kind.is_none() {
            kind = Some(match salt(sx * 100.0, sz * 100.0, 579) % 10 {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => 0,
            });
        }
        let k = kind.unwrap_or(0);
        let swab = mesh(kit.swabs[k].clone(), &kit.mat);
        // spread a little wider than the old sticks (twice as long)
        swab.position().set(sx * 1.5, 0.0, sz * 1.5);
        swab.rotation().y = yaw;
        cluster.add(&swab);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}

/// What every swab shares: the four swabs (their strip laid along each, a
/// row each) and the one material
struct SwabKit {
    swabs: Vec<Geo>,
    mat: Mat,
}

fn swab_kit() -> Option<std::rc::Rc<SwabKit>> {
    thread_local! { static K: OnceCell<Option<std::rc::Rc<SwabKit>>> = const { OnceCell::new() }; }
    K.with(|c| {
        c.get_or_init(|| {
            let mut swabs = vec![];
            for (row, name) in SWABS.iter().enumerate() {
                swabs.push(along_x(name, SWAB_L, row, SWABS.len())?);
            }
            Some(std::rc::Rc::new(SwabKit { swabs, mat: lambert!({ map: &swab_tex() }) }))
        })
        .clone()
    })
}

/// The swabs' strip, a row a swab (A..D from the bottom), u along it from
/// its -x tip, v round it (0 underneath): the cotton (wound fibres, greyed
/// by dust on top and by the floor underneath), the rolled paper stick a
/// shade creamier; the used one's +x tuft pressed and stained, the pulled
/// one's bare stick end
fn swab_tex() -> Tex {
    let rows = SWABS.len() as f64;
    canvas_tex_opt(256, 64, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let row = ((v * rows).floor() as usize).min(SWABS.len() - 1);
            let ring = ((v * rows - row as f64 - 0.08) / 0.84).clamp(0.0, 1.0);
            let x = u * SWAB_L;
            let tuft = x < TUFT_L || x > SWAB_L - TUFT_L;
            let (xm, rm) = (x * 1000.0, ring * 17.0);
            let n = fbm(5 + row as u32, xm * 0.8, rm * 0.8, 3);
            let mut c = if tuft {
                // wound cotton: fibres along the winding, lumps
                let fib = vn(41, xm * 0.7, rm * 4.0);
                mix([0.90, 0.89, 0.86], [0.74, 0.73, 0.70], 0.7 * fib + 0.5 * n - 0.3)
            } else {
                mix([0.88, 0.86, 0.80], [0.80, 0.78, 0.72], n)
            };
            // the used one: its +x tuft pressed grey-brown
            if row == 2 && x > SWAB_L - TUFT_L * 0.85 {
                c = mix(c, [0.60, 0.50, 0.36], 0.55 + 0.3 * n);
            }
            // the pulled one: a bare stick end, frayed, a wisp on it
            if row == 3 && x > SWAB_L - 0.009 {
                c = mix(c, [0.78, 0.74, 0.66], 0.6);
            }
            // dust over it, the floor's grime underneath
            let under = 1.0 - sstep(0.0, 0.4, ring);
            let c = mix(c, [0.36, 0.33, 0.29], 0.4 * under);
            let c = mix(c, [0.55, 0.53, 0.49], 0.14 + 0.1 * vn(17, xm * 0.3, rm * 0.3));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::salt;
/// The part `name` with its texture laid along x: u from its least x over
/// `span` metres, v round it (0 underneath, 1 on top) in row `row` of
/// `rows`. A copy of the part, made once and shared
fn along_x(name: &str, span: f64, row: usize, rows: usize) -> Option<Geo> {
    let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for i in 0..n {
            for k in 0..3 {
                lo[k] = lo[k].min(gg.position[i * 3 + k]);
                hi[k] = hi[k].max(gg.position[i * 3 + k]);
            }
        }
        let (cy, ry) = (f64::from(lo[1] + hi[1]) / 2.0, (f64::from(hi[1] - lo[1]) / 2.0).max(1e-6));
        let mut uv = vec![0f32; n * 2];
        for i in 0..n {
            let x = f64::from(gg.position[i * 3] - lo[0]);
            let y = f64::from(gg.position[i * 3 + 1]);
            let ring = (0.5 + 0.5 * (y - cy) / ry).clamp(0.0, 1.0);
            uv[i * 2] = (x / span) as f32;
            uv[i * 2 + 1] = ((row as f64 + 0.08 + 0.84 * ring) / rows as f64) as f32;
        }
        gg.uv = uv;
        gg.touch();
    }
    Some(geo)
}
