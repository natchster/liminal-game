//! `place_leaf_litter`
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::bleed;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// One of `N` textures of a kind, made the first time it is asked for
macro_rules! cached_kind_tex {
    ($n:expr, $k:expr, $make:expr) => {{
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

// Leaf litter: 8% chance per level. A drift of dead leaves blown in
// (Blender batch `t05`): beech and oak, brown and crisp, curled or folded,
// holed and torn, some lying on others; they stir a little in a draught.
pub(crate) fn place_leaf_litter(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let leaf_colors = [0x805020, 0xa06030, 0x604020, 0x907040, 0xc08040];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    // Track for drift animator: (leaf, base_x, base_z, phase_x, phase_z)
    let mut leaves: Vec<(Obj, f64, f64, f64, f64)> = vec![];
    let mut down: Vec<(f64, f64, f64, f64)> = vec![]; // where each lies, its reach, the height of its top
    for _i in 0..count {
        // the old leaf's draws: its colour (now which leaf), where it lies,
        // its height (now which face is up), its turn, the drift's phases
        let col = choice(&leaf_colors);
        let k = leaf_colors.iter().position(|&c| c == col).unwrap_or(0);
        let base_x = (rnd() - 0.5) * 0.16;
        let base_z = (rnd() - 0.5) * 0.16;
        let face = rnd();
        let turn = rnd() * PI * 2.0;
        let phase_x = rnd() * PI * 2.0;
        let phase_z = rnd() * PI * 2.0;
        let (name, l, _, _) = LEAVES[k];
        let Some(geo) = part(name) else { continue };
        let (_, hi) = frame(name);
        let top = hi[1];
        // a little lighter or darker, by its phase
        let shade = 0.86 + 0.14 * (phase_x / (PI * 2.0));
        let tint = ((shade * 255.0) as u32) * 0x010101;
        let leaf = group();
        let m = mesh(geo, lambert!({ map: &leaf_tex(k), color: tint, alphaTest: 0.5, side: DoubleSide }));
        if face < 0.35 {
            // lying on its face, its back up
            m.rotation().x = PI;
            m.position().y = top;
        }
        leaf.add(&m);
        leaf.rotation().y = turn;
        // on another where it overlaps one already down
        let reach = 0.42 * l;
        let mut y = 0.0f64;
        for &(ox, oz, or, ot) in &down {
            if (base_x - ox).hypot(base_z - oz) < (reach + or) * 0.7 {
                y = y.max(ot * 0.55);
            }
        }
        leaf.position().set(base_x, y, base_z);
        down.push((base_x, base_z, reach, y + top));
        {
            let mut u = leaf.user();
            u.base_x = base_x;
            u.base_z = base_z;
            u.set("phaseX", phase_x);
            u.set("phaseZ", phase_z);
        }
        leaves.push((leaf.clone(), base_x, base_z, phase_x, phase_z));
        cluster.add(&leaf);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
    // Subtle drift in a draught (leaves shift 4-5 mm).
    g.animate(move |_g, _dt, t| {
        for (l, bx, bz, phx, phz) in &leaves {
            let mut p = l.position();
            p.x = bx + (t * 0.3 + phx).sin() * 0.005;
            p.z = bz + (t * 0.28 + phz).cos() * 0.004;
        }
    });
}

// ================================================================= leaves
/// The dead leaves (`nature/bits_of_the_outside_indoors.py` LEAVES): the part, the blade's length, its
/// envelope's half width, oak or beech
const LEAVES: [(&str, f64, f64, bool); 5] = [
    ("Real__leafBeechA", 0.072, 0.0225, false),
    ("Real__leafBeechB", 0.062, 0.0195, false),
    ("Real__leafBeechC", 0.056, 0.0180, false),
    ("Real__leafOakA", 0.086, 0.0265, true),
    ("Real__leafOakB", 0.070, 0.0220, true),
];

/// Each leaf's colour (sRGB): beech russet, beech brown, beech pale tan, oak
/// brown, oak dark (wet once)
const LEAF_RGB: [[f64; 3]; 5] = [[0.56, 0.37, 0.21], [0.48, 0.34, 0.22], [0.63, 0.49, 0.33], [0.54, 0.42, 0.28], [0.42, 0.32, 0.22]];

/// The half width of leaf `k`'s blade at a fraction `a` of its length, on
/// side `side` (±1): beech ovate with a wavy margin, oak lobed, each a little
/// ragged, and one tear
fn leaf_half(k: usize, a: f64, side: f64) -> f64 {
    let (_, _, w, oak) = LEAVES[k];
    if !(0.0..=1.0).contains(&a) {
        return 0.0;
    }
    let seed = 0x1ea0 + k as u32 * 17;
    let mut hw = if oak {
        let env = (PI * a.powf(0.62)).sin().max(0.0).powf(0.75) * (1.0 - 0.15 * (1.0 - a)) + 0.18 * (-((a - 0.03) / 0.05).powi(2)).exp();
        // rounded lobes between narrow sinuses
        let t = (a * 5.2 + 0.2 + 0.08 * side).fract();
        let lobe = if a > 0.1 { 0.58 + 0.42 * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt() } else { 1.0 };
        w * env * lobe
    } else {
        let env = (PI * a.powf(1.15)).sin().max(0.0).powf(0.8);
        // the margin waves between the veins' ends
        w * env * (1.0 - 0.04 * (0.5 + 0.5 * (2.0 * PI * a * 8.5).cos()))
    } - 0.0006;
    // ragged where it dried and broke
    hw *= 1.0 - 0.07 * sstep(0.55, 0.8, fbm(seed, a * 18.0, side * 3.0, 3));
    // a tear in from the margin on one side
    let ta = 0.35 + 0.3 * hash(seed, 1, 0);
    if side > 0.0 && k % 2 == 0 {
        let d = ((a - ta) / 0.025).abs();
        hw *= 1.0 - 0.7 * (1.0 - d).max(0.0);
    }
    hw.max(0.0)
}

/// Leaf `k` seen from above in its part's frame: the blade's outline cut out
/// (lobes, the ragged margin, a tear, holes eaten through), its colour
/// mottled and darker at the dried margin, fungal spots, the midrib and the
/// veins paler; the petiole
fn leaf_tex(k: usize) -> Tex {
    cached_kind_tex!(5, k, |k: usize| {
        let (name, l, _, oak) = LEAVES[k];
        let (lo, hi) = frame(name);
        let (ex, ez) = (hi[0] - lo[0], hi[2] - lo[2]);
        let th = (256.0 * ez / ex).round().max(32.0);
        let base = LEAF_RGB[k];
        let seed = 0x1ea7 + k as u32 * 31;
        let tex = canvas_tex_opt(256, th, false, move |ctx, w, h| {
            let px_m = ex / w;
            paint_px(ctx, w, h, |px, py| {
                let x = l - (hi[0] - (lo[0] + px / w * ex));
                let z = lo[2] + py / h * ez;
                let a = x / l;
                if x < 0.0004 {
                    // the petiole (its whole column: however its sides are mapped)
                    let c = mix_unclamped([0.34, 0.24, 0.15], [0.26, 0.18, 0.12], vn(seed, px * 0.3, py * 0.3));
                    return [c[0], c[1], c[2], 1.0];
                }
                let side = if z > 0.0 { 1.0 } else { -1.0 };
                let hw = leaf_half(k, a, side);
                let edge = (hw - z.abs()) / px_m;
                let inside = (edge + 0.5).clamp(0.0, 1.0);
                // eaten through: a few holes, skeletonised round their edges
                let hole_n = fbm(seed ^ 0x40, x * 260.0, z * 260.0, 3);
                let hole = sstep(0.77, 0.79, hole_n);
                let skel = sstep(0.70, 0.77, hole_n) * (1.0 - hole);
                // colour: mottled, darker and greyer toward the dried margin
                let m = fbm(seed ^ 0x11, x * 120.0, z * 120.0, 4);
                let mut c = mix_unclamped(base, mix_unclamped(base, [0.30, 0.22, 0.15], 0.6), sstep(0.45, 0.8, m));
                let margin = 1.0 - sstep(0.0, 0.004, hw - z.abs());
                c = mix_unclamped(c, [0.30, 0.23, 0.17], 0.5 * margin);
                // fungal spots, dark with a paler ring
                let sp = fbm(seed ^ 0x77, x * 300.0, z * 300.0, 2);
                c = mix_unclamped(c, [0.24, 0.18, 0.13], 0.55 * sstep(0.76, 0.79, sp));
                // veins: the midrib, and the laterals running out to the margin
                let mid_w = 0.0007 - 0.0005 * a;
                let midrib = 1.0 - sstep(mid_w * 0.5, mid_w, z.abs());
                let (pairs, slope) = if oak { (6.0, 0.62) } else { (8.0, 0.9) };
                let q = (x - z.abs() * slope) / l * pairs - 0.35;
                let dq = (q - q.round()).abs() * l / pairs;
                let lateral =
                    (1.0 - sstep(0.00018, 0.00042, dq)) * sstep(0.0, 0.02, q / pairs) * (1.0 - sstep(0.85, 1.0, z.abs() / hw.max(1e-6)));
                let net = (1.0 - sstep(0.0, 0.08, (vn(seed ^ 0x9, x * 900.0, z * 900.0) - 0.5).abs())) * 0.25;
                let vein = midrib.max(lateral * 0.8).max(net * 0.4);
                c = mix_unclamped(c, mix_unclamped(c, [0.78, 0.66, 0.50], 0.45), vein);
                c = mix_unclamped(c, [0.62, 0.55, 0.45], 0.5 * skel);
                [c[0], c[1], c[2], inside * (1.0 - hole)]
            });
        });
        bleed(&tex, base);
        tex
    })
}
