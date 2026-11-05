//! Batch `t22`'s cigarette butts, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t22.py`, its notes
//! `t22.notes.md`): king-size filter cigarettes smoked down and dropped,
//! stubbed out on the floor, and the ash knocked off beside some. It keeps
//! the old placer's draws, call for call.
//!
//! The butts stand on about half of all floors, so every part of one is a
//! mesh shared by every butt (one painted strip for all four shapes, one
//! per brand), and `instancify_static` draws each kind of butt in a chunk
//! at once.
use crate::{open_tiles, shuffle};
use game::prelude::*;
use std::cell::OnceCell;
use t22_kit::*;

/// The cigarette (`t22.py` `CIG_*`): its filter's and its tipping paper's
/// lengths from the mouth end, metres
const CIG_FILTER: f64 = 0.021;
const CIG_TIPPING: f64 = 0.025;
/// The length the butts' strip is laid over, from the mouth end
const SPAN: f64 = 0.056;
/// The four butts (`Real__cigButtA..D`), and where each one's paper ends:
/// stubbed out and kinked, smoked to the tipping and trodden oval, dropped
/// half smoked, trodden flat
const BUTTS: [(&str, f64); 4] = [
    ("Real__cigButtA", CIG_FILTER + 0.016),
    ("Real__cigButtB", CIG_FILTER + 0.005),
    ("Real__cigButtC", CIG_FILTER + 0.028),
    ("Real__cigButtD", CIG_FILTER + 0.012),
];

// R251 cigarette butts: 0-2 small piles per level (2-6 butts per pile),
// where someone stood and smoked: king-size filter cigarettes (84 mm,
// 7.8 mm across, a 21 mm filter under cork-printed or white tipping paper)
// smoked down and dropped, most stubbed out underfoot (the burning end
// crushed flat and kinked), some smoked to the tipping, some dropped half
// smoked and gone out, a few trodden flat; the ash knocked off beside some.
// One pile is one smoker's: one brand.
pub fn place_cigarette_butts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.55 {
        return;
    } // 45% chance of pile(s)
    let pile_count = 1 + (rnd() * 2.0).floor() as usize; // 1-2 piles
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let Some(kit) = butt_kit() else { return };
    let mut p = 0;
    while p < pile_count && p < open_tiles.len() {
        let (x, z) = open_tiles[p];
        // Pile center within cell, biased toward wall corners.
        let corner_bias = rnd() < 0.7;
        let px = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.9 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.0 };
        let pz = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.9 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.0 };
        let n_butts = 2 + (rnd() * 5.0).floor() as i32; // 2-6 per pile
                                                        // the smoker's brand: cork tipping, or one in four white (no draw)
        let brand = usize::from(salt(g.wx(x) + px, g.wz(z) + pz, 251) % 4 == 0);
        for _ in 0..n_butts {
            // Position within ~0.15 m radius of pile center.
            let bx = px + (rnd() - 0.5) * 0.30;
            let bz = pz + (rnd() - 0.5) * 0.30;
            let yaw = rnd() * PI * 2.0;
            // the old tilt's draw: which butt (stubbed out the most)
            let which = rnd();
            let k = match which {
                w if w < 0.42 => 0,
                w if w < 0.66 => 1,
                w if w < 0.82 => 2,
                _ => 3,
            };
            let (wx, wz) = (g.wx(x) + bx, g.wz(z) + bz);
            let butt = group();
            let (geo, len) = &kit.butts[k];
            let m = mesh(geo.clone(), &kit.paper[brand]);
            m.position().x = -len / 2.0;
            butt.add(&m);
            // the ash knocked off it, beside its burnt end (not under a shoe)
            let s = salt(wx, wz, 0x251);
            if k != 3 && s % 3 == 0 {
                let ash = mesh(kit.ash.clone(), &kit.ash_mat);
                let side = if (s >> 4) & 1 == 0 { 1.0 } else { -1.0 };
                ash.position().set(len / 2.0 + 0.002, 0.0, side * 0.006);
                ash.rotation().y = f64::from((s >> 5) % 628) / 100.0;
                butt.add(&ash);
            }
            butt.position().set(wx, 0.0, wz);
            butt.rotation().y = yaw;
            g.world.add(&butt);
        }
        p += 1;
    }
}

/// What every butt shares: the four butts (each with its painted strip laid
/// along it, and its length), the ash, and their materials (a brand each)
struct ButtKit {
    butts: Vec<(Geo, f64)>,
    paper: [Mat; 2],
    ash: Geo,
    ash_mat: Mat,
}

fn butt_kit() -> Option<std::rc::Rc<ButtKit>> {
    thread_local! { static K: OnceCell<Option<std::rc::Rc<ButtKit>>> = const { OnceCell::new() }; }
    K.with(|c| {
        c.get_or_init(|| {
            let mut butts = vec![];
            for (row, (name, _)) in BUTTS.iter().enumerate() {
                butts.push(along_x(name, SPAN, row, BUTTS.len())?);
            }
            let ash = game::assets::props::prop_part_as_modelled("Real__cigButtAsh")?;
            Some(std::rc::Rc::new(ButtKit {
                butts,
                paper: [lambert!({ map: &butt_tex(0) }), lambert!({ map: &butt_tex(1) })],
                ash,
                ash_mat: lambert!({ color: 0x8c8781 }),
            }))
        })
        .clone()
    })
}

/// The butts' strip, one row a butt (A..D from the bottom), u along it from
/// the mouth end over `SPAN`, v round it (0 underneath, 1 on top): the
/// filter's end stained with tar, the tipping (cork-printed, or white for
/// `brand` 1) and the lip's stain on it, its printed line where it laps
/// the paper; the paper gone cream, browned by the tar toward the burnt
/// end, the ragged black edge, the char and its ash; grime underneath
fn butt_tex(brand: usize) -> Tex {
    let rows = BUTTS.len() as f64;
    let burns: Vec<f64> = BUTTS.iter().map(|b| b.1).collect();
    let paint = move |ctx: &mut Canvas, w: f64, h: f64| {
        fill(ctx, w, h, |u, v| {
            let row = ((v * rows).floor() as usize).min(BUTTS.len() - 1);
            let ring = (v * rows - row as f64 - 0.08) / 0.84;
            let x = u * SPAN;
            let burn = burns[row];
            let (xm, rm) = (x * 1000.0, ring * 24.5);
            let n = fbm(31 + row as u32, xm * 0.9, rm * 0.9, 3);
            let fine = vn(57, xm * 3.1, rm * 3.1);
            let mut c = if x < 0.0005 {
                // the filter's end: acetate tow, stained by the smoke drawn through it
                mix([0.80, 0.70, 0.50], [0.62, 0.48, 0.30], n)
            } else if x < CIG_TIPPING.min(burn) {
                let mut t = if brand == 0 {
                    // cork tipping: its printed mottle
                    let m = fbm(77, xm * 0.55, rm * 0.55, 3);
                    let c = mix([0.74, 0.57, 0.38], [0.55, 0.40, 0.25], m * 1.7 - 0.35);
                    if fine > 0.78 {
                        mix(c, [0.84, 0.72, 0.55], (fine - 0.78) * 4.0)
                    } else {
                        c
                    }
                } else {
                    mix([0.87, 0.86, 0.82], [0.80, 0.79, 0.75], n)
                };
                // its printed line where it laps the paper
                if (CIG_FILTER + 0.0006..CIG_FILTER + 0.0013).contains(&x) {
                    t = if brand == 0 { mix(t, [0.78, 0.66, 0.36], 0.8) } else { mix(t, [0.58, 0.58, 0.60], 0.8) };
                }
                // the lips' stain and the fingers' grime toward the mouth end
                mix(t, [0.48, 0.36, 0.24], 0.35 * (1.0 - sstep(0.0, 0.006, x)) + 0.12 * n)
            } else {
                // the paper, gone cream; the tar browning it toward the end
                let paper = mix([0.88, 0.85, 0.77], [0.80, 0.76, 0.66], n + 0.15 * fine);
                let tar = sstep(burn - 0.004, burn - 0.0008, x + (fine - 0.5) * 0.0008);
                mix(paper, [0.42, 0.29, 0.17], tar)
            };
            // the burnt end: a ragged black edge, the char, ash greying it
            let edge = burn - 0.0004 + (vn(91, rm * 1.4, 0.0) - 0.5) * 0.0012;
            if x > edge {
                let ash = vn(93, xm * 2.0, rm * 2.0);
                let char_ = mix([0.08, 0.07, 0.06], [0.20, 0.18, 0.17], sstep(edge + 0.0006, edge + 0.002, x));
                c = if ash > 0.62 { mix(char_, [0.46, 0.44, 0.41], (ash - 0.62) * 2.5) } else { char_ };
            }
            // dropped on a floor: grime on its underside, dust over all
            let under = 1.0 - sstep(0.0, 0.45, ring);
            let c = mix(c, [0.30, 0.27, 0.23], 0.35 * under * (0.6 + 0.4 * n));
            let c = mix(c, [0.55, 0.52, 0.47], 0.10);
            [c[0], c[1], c[2], 1.0]
        });
    };
    canvas_tex_opt(256, 64, false, paint)
}

mod t22_kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    /// A salt from where a thing is (no draw)
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` with its texture laid along x: u from its least x
    /// over `span` metres, v round it (0 underneath, 1 on top) in row `row`
    /// of `rows`; and its length. A copy of the part, made once and shared
    pub(super) fn along_x(name: &str, span: f64, row: usize, rows: usize) -> Option<(Geo, f64)> {
        let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
        let len;
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
            len = f64::from(hi[0] - lo[0]);
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
        Some((geo, len))
    }
}
