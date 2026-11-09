//! Batch `t05`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! bits of the outside indoors (Blender batch `t05`; what each real thing
//! is and where its size comes from: `assets/src/blender/props/t05.notes.md`).
//! Each placer keeps its old draws call for call (the comments at each say
//! what a draw now chooses, or that it is kept for its place in the stream).
use super::*;
use game::assets::props::{prop_part_as_modelled, prop_part_info};

// ---------------------------------------------------------------- the kit
/// The Blender part `name` as modelled (its own frame, a scale of one)
fn part(name: &str) -> Option<Geo> {
    prop_part_as_modelled(name)
}

/// The bounds (frame) part `name` was modelled in
fn frame(name: &str) -> ([f64; 3], [f64; 3]) {
    prop_part_info(name).map(|m| (m.fmin, m.fmax)).unwrap_or(([-0.05; 3], [0.05; 3]))
}

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! t05_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

/// One of `N` textures of a kind, made the first time it is asked for
macro_rules! t05_kind_tex {
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

/// A hash of an integer lattice point, in [0, 1)
fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Value noise in [0, 1]
fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// `rgba(…)` of an sRGB colour (0-1) at alpha `a`
fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &css(c, a));
    gr.add_color_stop(1, &css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Fill the canvas pixel by pixel: `f(x, y)` (pixel centres) gives a
/// straight sRGB colour and alpha. Pure (no draws, no canvas calls), so its
/// rows are filled on every core
fn paint_px(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let c = f(i as f64 + 0.5, j as f64 + 0.5);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// Where a texture is clear, its colour becomes `rgb` (sRGB 0-1), so its
/// smaller mips keep the thing's colour at a cut-out edge instead of black
fn bleed(tex: &Tex, rgb: [f64; 3]) {
    let img = tex.image();
    let mut im = img.borrow_mut();
    for px in im.data.chunks_exact_mut(4) {
        let k = sstep(0.0, 0.5, f64::from(px[3]) / 255.0);
        for (c, b) in px.iter_mut().zip(rgb) {
            *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
        }
    }
    im.version += 1;
}

/// A salt from where a thing lies (no draw): a painter's own stream starts
/// the same on every floor, so what it paints for each thing mixes this in
fn salt_of(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
}

/// The decal material of a mark painted in `tex`, lit as the floor under it
fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A thin part (grass blades, stems) lit as one: drawn from both sides, its
/// normals turned up toward the light above, so that the side turned from
/// a lamp does not go black (a blade lets the light through)
fn thin_lit(name: &str) -> Option<Geo> {
    thread_local! { static C: RefCell<Vec<(String, Geo)>> = const { RefCell::new(vec![]) }; }
    if let Some(g) = C.with(|c| c.borrow().iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())) {
        return Some(g);
    }
    let g = part(name)?.duplicate();
    {
        let mut gg = g.g();
        for n in gg.normal.chunks_exact_mut(3) {
            let (x, y, z) = (n[0] * 0.35, n[1].abs() * 0.35 + 0.75, n[2] * 0.35);
            let l = (x * x + y * y + z * z).sqrt().max(1e-6);
            n.copy_from_slice(&[x / l, y / l, z / l]);
        }
        if let Some(idx) = gg.index.as_mut() {
            let back: Vec<u32> = idx.chunks_exact(3).flat_map(|t| [t[0], t[2], t[1]]).collect();
            idx.extend(back);
        }
        gg.version += 1;
    }
    C.with(|c| c.borrow_mut().push((name.to_string(), g.clone())));
    Some(g)
}

/// A painted mark lying flat on the floor, `w` × `d` m, its middle at (x, z)
fn floor_mark(tex: &Tex, w: f64, d: f64, x: f64, z: f64, turn: f64) -> Obj {
    let m = mesh(plane_geo!(w, d), mark_mat(tex));
    m.rotation().x = -PI / 2.0;
    m.rotation().z = turn;
    m.position().set(x, 0.0015, z);
    m
}

// ================================================================== twigs
/// The twiglets (`t05.py` TWIGS): the part, its snapped end's wood if it has
/// one, its length as modelled
const TWIGS: [(&str, Option<&str>, f64); 6] = [
    ("Real__twigA", Some("Real__twigWoodA"), 0.046),
    ("Real__twigB", None, 0.056),
    ("Real__twigC", Some("Real__twigWoodC"), 0.066),
    ("Real__twigD", None, 0.076),
    ("Real__twigE", None, 0.086),
    ("Real__twigF", Some("Real__twigWoodF"), 0.098),
];

/// Oak bark on a one-year twig (u along it): grey-brown, fine streaks along
/// it, pale lenticels (short dashes), a bloom of grey-green algae here and
/// there; one texture for every twig, laid along each one's length
fn bark_tex() -> Tex {
    t05_tex!(256, 64, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w, y / h);
            let streak = fbm(0x0b1, u * 40.0, v * 3.0, 3);
            let mut c = mix([0.50, 0.44, 0.37], [0.36, 0.31, 0.26], streak);
            // lenticels: short pale dashes across the twig
            let (cx, cy) = ((u * 70.0).floor(), (v * 9.0).floor());
            let hh = hash(0x1e7, cx as i32, cy as i32);
            if hh > 0.72 {
                let (fx, fy) = (u * 70.0 - cx - 0.5, v * 9.0 - cy - 0.5);
                let d = (fx / 0.18).hypot(fy / 0.42);
                c = mix(c, [0.66, 0.60, 0.50], 0.8 * (1.0 - sstep(0.6, 1.0, d)));
            }
            // algae and lichen, grey-green, dry
            let alg = sstep(0.62, 0.75, fbm(0x2a9, u * 9.0, v * 2.0, 4));
            c = mix(c, [0.46, 0.48, 0.40], 0.5 * alg);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A twig laid down: where it lies, its turn, its half length, how high its
/// top is, and (when it rests across another) its pivot end and its tilt
struct Laid {
    c: (f64, f64),
    d: (f64, f64),
    hl: f64,
    top: f64,
    tilt: Option<(f64, f64)>,
}

impl Laid {
    fn end(&self, sign: f64) -> (f64, f64) {
        (self.c.0 + sign * self.hl * self.d.0, self.c.1 + sign * self.hl * self.d.1)
    }
    /// How high its top is over the floor at (x, z) on it
    fn height_at(&self, p: (f64, f64)) -> f64 {
        match self.tilt {
            None => self.top,
            Some((sign, th)) => {
                let e = self.end(sign);
                self.top + (p.0 - e.0).hypot(p.1 - e.1) * th.tan()
            }
        }
    }
}

/// Where segments a0-a1 and b0-b1 cross, if they do
fn cross2(a0: (f64, f64), a1: (f64, f64), b0: (f64, f64), b1: (f64, f64)) -> Option<(f64, f64)> {
    let (rx, rz) = (a1.0 - a0.0, a1.1 - a0.1);
    let (sx, sz) = (b1.0 - b0.0, b1.1 - b0.1);
    let den = rx * sz - rz * sx;
    if den.abs() < 1e-12 {
        return None;
    }
    let t = ((b0.0 - a0.0) * sz - (b0.1 - a0.1) * sx) / den;
    let u = ((b0.0 - a0.0) * rz - (b0.1 - a0.1) * rx) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then(|| (a0.0 + rx * t, a0.1 + rz * t))
}

// Small twigs: 7% chance per level. Oak twiglets the tree shed (Blender
// batch `t05`): grey-brown bark with its lenticels, the buds at the nodes
// and the tip's cluster, a side shoot, a snapped end's pale wood; one lying
// across another rests on it.
pub(super) fn place_twigs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x503020, 0x604030, 0x402010, 0x705040];
    // the bark's tint by the old colour: weathered grey-brown, brown, dark
    // with damp, bleached
    const TINT: [u32; 4] = [0xc4bcb0, 0xd2b8a0, 0x9a8e80, 0xe2dcd2];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    let bark = bark_tex();
    let wood = lambert!({ color: 0x9c8a6c });
    let mut mats: [Option<Mat>; 4] = Default::default();
    let mut laid: Vec<Laid> = vec![];
    for _i in 0..count {
        // the old twig's draws: its colour (the bark's tint), its length
        // (which twiglet, and its size within 12%), its radius (kept for its
        // draw), its turn, where it lies, its roll (kept for its draw)
        let col = choice(&twig_colors);
        let k = twig_colors.iter().position(|&c| c == col).unwrap_or(0);
        let len = 0.040 + rnd() * 0.060;
        let _ = rnd();
        let yaw = rnd() * PI * 2.0;
        let (tx, tz) = ((rnd() - 0.5) * 0.16, (rnd() - 0.5) * 0.16);
        let _ = rnd();
        let kind = (((len - 0.040) / 0.060 * 6.0).floor() as usize).min(5);
        let (name, wood_name, plen) = TWIGS[kind];
        let Some(geo) = part(name) else { continue };
        let s = (len / plen).clamp(0.88, 1.12);
        let (lo, hi) = frame(name);
        let me = Laid { c: (tx, tz), d: (yaw.cos(), -yaw.sin()), hl: (hi[0] - lo[0]) / 2.0 * s, top: hi[1] * s, tilt: None };
        // across one already down: it rests on it, its far end on the floor
        let mut tilt: Option<(f64, f64)> = None;
        for o in &laid {
            if let Some(p) = cross2(me.end(-1.0), me.end(1.0), o.end(-1.0), o.end(1.0)) {
                let h = o.height_at(p);
                let (dm, dp) = {
                    let (a, b) = (me.end(-1.0), me.end(1.0));
                    ((p.0 - a.0).hypot(p.1 - a.1), (p.0 - b.0).hypot(p.1 - b.1))
                };
                let (sign, d) = if dp > dm { (1.0, dp) } else { (-1.0, dm) };
                let th = (h / d.max(0.01)).atan().min(0.5);
                if tilt.is_none_or(|(_, t)| th > t) {
                    tilt = Some((sign, th));
                }
            }
        }
        let mat = mats[k].get_or_insert_with(|| lambert!({ map: &bark, color: TINT[k] })).clone();
        let t = group();
        let m = mesh(geo, &mat);
        m.scale().set_scalar(s);
        t.add(&m);
        if let Some(wg) = wood_name.and_then(part) {
            let w = mesh(wg, &wood);
            w.scale().set_scalar(s);
            t.add(&w);
        }
        match tilt {
            None => {
                t.position().set(tx, 0, tz);
                t.rotation().y = yaw;
                cluster.add(&t);
            }
            Some((sign, th)) => {
                let e = me.end(sign);
                let pivot = group();
                pivot.position().set(e.0, 0, e.1);
                pivot.rotation().y = yaw;
                let lean = group();
                lean.rotation().z = -sign * th;
                t.position().x = -sign * me.hl;
                lean.add(&t);
                pivot.add(&lean);
                cluster.add(&pivot);
            }
        }
        laid.push(Laid { tilt, ..me });
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// ================================================================= leaves
/// The dead leaves (`t05.py` LEAVES): the part, the blade's length, its
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
    t05_kind_tex!(5, k, |k: usize| {
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
                    let c = mix([0.34, 0.24, 0.15], [0.26, 0.18, 0.12], vn(seed, px * 0.3, py * 0.3));
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
                let mut c = mix(base, mix(base, [0.30, 0.22, 0.15], 0.6), sstep(0.45, 0.8, m));
                let margin = 1.0 - sstep(0.0, 0.004, hw - z.abs());
                c = mix(c, [0.30, 0.23, 0.17], 0.5 * margin);
                // fungal spots, dark with a paler ring
                let sp = fbm(seed ^ 0x77, x * 300.0, z * 300.0, 2);
                c = mix(c, [0.24, 0.18, 0.13], 0.55 * sstep(0.76, 0.79, sp));
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
                c = mix(c, mix(c, [0.78, 0.66, 0.50], 0.45), vein);
                c = mix(c, [0.62, 0.55, 0.45], 0.5 * skel);
                [c[0], c[1], c[2], inside * (1.0 - hole)]
            });
        });
        bleed(&tex, base);
        tex
    })
}

// Leaf litter: 8% chance per level. A drift of dead leaves blown in
// (Blender batch `t05`): beech and oak, brown and crisp, curled or folded,
// holed and torn, some lying on others; they stir a little in a draught.
pub(super) fn place_leaf_litter(g: &mut Game, _dist: &[i32]) {
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

// =================================================================== wing
/// A forewing's outline (mm, x from the root along the costa, z back from
/// the costa), Aglais's: the costa, the apex, the outer margin with its
/// angle, the tornus, the inner margin back to the root (`t05.py` WING_FORE:
/// 30.5 x 19 mm)
const FORE: [(f64, f64); 17] = [
    (0.6, 1.6),
    (6.0, 0.4),
    (14.0, 0.0),
    (22.0, 0.3),
    (27.5, 1.0),
    (30.3, 2.3),
    (29.6, 5.4),
    (28.2, 7.9),
    (29.0, 9.6),
    (26.8, 12.6),
    (24.6, 15.6),
    (22.4, 18.7),
    (17.0, 15.8),
    (10.0, 10.4),
    (5.0, 6.4),
    (2.0, 3.8),
    (0.6, 3.0),
];

/// The veins (mm, from, to): the forewing's from its discal cell to the
/// costa, the outer margin and the tornus, and the anal vein
const FORE_VEINS: [((f64, f64), (f64, f64)); 10] = [
    ((1.0, 2.6), (13.0, 3.6)),
    ((1.0, 4.0), (12.0, 7.6)),
    ((8.0, 3.2), (20.0, 0.8)),
    ((11.0, 3.4), (25.0, 1.2)),
    ((13.0, 3.6), (29.0, 2.2)),
    ((13.2, 5.0), (29.6, 5.0)),
    ((13.0, 6.4), (28.4, 8.4)),
    ((12.5, 7.6), (27.2, 11.2)),
    ((10.0, 7.0), (25.4, 14.6)),
    ((1.0, 4.0), (21.6, 18.2)),
];

/// The hindwing's veins (mm)
const HIND_VEINS: [((f64, f64), (f64, f64)); 10] = [
    ((1.0, 3.8), (11.0, 6.0)),
    ((1.0, 6.0), (11.6, 10.0)),
    ((1.0, 3.2), (16.0, 0.8)),
    ((11.0, 6.0), (23.0, 4.5)),
    ((11.4, 7.6), (24.0, 8.5)),
    ((11.6, 9.4), (23.0, 12.5)),
    ((11.4, 10.0), (21.5, 15.5)),
    ((9.0, 9.0), (18.0, 19.2)),
    ((6.0, 8.0), (13.5, 21.0)),
    ((1.0, 6.0), (8.0, 20.5)),
];

/// A hindwing's outline (mm, x from the root out, z back from its costa;
/// 24.5 x 21.5 mm)
const HIND: [(f64, f64); 15] = [
    (0.6, 2.2),
    (8.0, 0.3),
    (16.0, 0.6),
    (21.6, 2.6),
    (24.2, 6.0),
    (23.6, 10.0),
    (22.4, 13.6),
    (21.0, 16.2),
    (17.4, 19.6),
    (13.0, 21.3),
    (9.0, 21.0),
    (5.0, 16.0),
    (2.0, 10.0),
    (0.6, 5.0),
    (0.4, 3.4),
];

/// The five wings the old colour chose: a small tortoiseshell's forewing, a
/// peacock's forewing, a peacock's hindwing, a small tortoiseshell's
/// hindwing (all their upper sides), a peacock's forewing from beneath
fn wing_tex(k: usize) -> Tex {
    t05_kind_tex!(5, k, |k: usize| {
        let hind = k == 2 || k == 3;
        let (name, size, outline): (&str, (f64, f64), &[(f64, f64)]) =
            if hind { ("Real__wingHind", (24.5, 21.5), &HIND) } else { ("Real__wingFore", (30.5, 19.0), &FORE) };
        let (lo, hi) = frame(name);
        let th = (256.0 * (hi[2] - lo[2]) / (hi[0] - lo[0])).round();
        let tex = canvas_tex_opt(256, th, false, move |ctx, w, h| {
            let (sx, sy) = (w / size.0, h / size.1);
            let p = |x: f64, z: f64| (x * sx, z * sy);
            let path = |ctx: &mut Canvas| {
                ctx.begin_path();
                for (i, &(x, z)) in outline.iter().enumerate() {
                    let (a, b) = p(x, z);
                    if i == 0 {
                        ctx.move_to(a, b);
                    } else {
                        ctx.line_to(a, b);
                    }
                }
                ctx.close_path();
            };
            let fill_poly = |ctx: &mut Canvas, pts: &[(f64, f64)], c: [f64; 3], a: f64| {
                ctx.set_fill_style(css(c, a));
                ctx.begin_path();
                for (i, &(x, z)) in pts.iter().enumerate() {
                    let (a, b) = p(x, z);
                    if i == 0 {
                        ctx.move_to(a, b);
                    } else {
                        ctx.line_to(a, b);
                    }
                }
                ctx.close_path();
                ctx.fill();
            };
            let spot = |ctx: &mut Canvas, x: f64, z: f64, r: f64, k2: f64, c: [f64; 3], a: f64| {
                let (cx, cy) = p(x, z);
                blob(ctx, cx, cy, r * sx, k2, 0.3, c, a);
            };
            // a scale-covered disc: crisp at the edge, as the eyespots' rings are
            let disc = |ctx: &mut Canvas, x: f64, z: f64, r: f64, c: [f64; 3], a: f64| {
                let (cx, cy) = p(x, z);
                ctx.set_fill_style(css(c, a));
                ctx.begin_path();
                ctx.ellipse(cx, cy, r * sx, r * sy * 0.92, 0.2, 0, 7);
                ctx.fill();
            };
            ctx.save();
            path(ctx);
            ctx.clip();
            let (rust, black, cream, blue) = ([0.55, 0.21, 0.15], [0.10, 0.08, 0.07], [0.80, 0.72, 0.50], [0.38, 0.44, 0.58]);
            match k {
                // small tortoiseshell, forewing
                0 => {
                    let orange = [0.74, 0.42, 0.17];
                    ctx.set_fill_style(css(orange, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 6.0, 9.0, 1.0, [0.22, 0.14, 0.09], 0.95);
                    // the costa's blocks: black, yellow, black, yellow, black
                    for &(x0, x1, c) in &[
                        (5.0, 9.0, black),
                        (9.0, 12.5, [0.82, 0.70, 0.30]),
                        (12.5, 17.0, black),
                        (17.0, 20.5, [0.82, 0.70, 0.30]),
                        (20.5, 26.0, black),
                    ] {
                        fill_poly(ctx, &[(x0, -1.0), (x1, -1.0), (x1 - 0.8, 5.5), (x0 - 0.4, 5.0)], c, 1.0);
                    }
                    spot(ctx, 27.2, 2.6, 1.3, 1.0, [0.86, 0.84, 0.78], 0.95);
                    for &(x, z, r) in &[(9.5, 9.0, 1.9), (12.5, 12.0, 1.5), (14.8, 16.4, 2.4)] {
                        spot(ctx, x, z, r, 1.0, black, 1.0);
                        spot(ctx, x, z, r * 0.7, 1.0, black, 1.0);
                    }
                }
                // peacock, forewing
                1 => {
                    ctx.set_fill_style(css(rust, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 7.0, 10.0, 1.0, [0.24, 0.12, 0.09], 0.95);
                    fill_poly(ctx, &[(4.0, -1.0), (22.0, -1.0), (21.0, 2.2), (4.0, 1.6)], black, 0.95);
                    fill_poly(ctx, &[(13.5, -1.0), (16.0, -1.0), (15.5, 3.0), (13.2, 2.6)], cream, 0.9);
                    // the eyespot: black patches round it, cream on its inner
                    // side, rust and black within, blue-violet scales on it
                    disc(ctx, 25.6, 5.4, 4.6, black, 0.9);
                    disc(ctx, 24.6, 6.8, 4.0, black, 0.9);
                    spot(ctx, 22.8, 4.6, 3.6, 0.8, cream, 0.95);
                    spot(ctx, 23.6, 7.6, 2.2, 1.0, cream, 0.8);
                    spot(ctx, 25.4, 6.0, 2.6, 0.9, [0.58, 0.28, 0.20], 0.9);
                    disc(ctx, 25.8, 6.2, 1.6, black, 0.9);
                    for i in 0..9 {
                        let (a, d) = (hash(0x3a5, i, 0) * 6.3, 0.8 + 2.4 * hash(0x3a5, i, 1));
                        spot(ctx, 25.8 + a.cos() * d, 6.2 + a.sin() * d * 0.8, 0.8 + 0.6 * hash(0x3a5, i, 2), 1.0, blue, 0.7);
                    }
                    spot(ctx, 28.4, 2.4, 0.7, 1.0, [0.86, 0.84, 0.80], 0.9);
                }
                // peacock, hindwing
                2 => {
                    ctx.set_fill_style(css(rust, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    // its base dark brown, hairy
                    disc(ctx, -2.0, 12.0, 10.5, [0.20, 0.12, 0.09], 0.9);
                    spot(ctx, 0.0, 11.0, 17.0, 1.0, [0.18, 0.11, 0.08], 0.9);
                    // the eyespot toward the front of the wing: a pale ring, black,
                    // the blue-grey scales scattered in it, its black heart
                    disc(ctx, 16.8, 7.4, 5.6, [0.56, 0.52, 0.46], 0.9);
                    disc(ctx, 16.8, 7.6, 4.6, black, 0.97);
                    // the blue-grey scales: a broken ring of them, thickest toward the wing's edge
                    let (cx, cy) = p(17.0, 7.7);
                    ctx.set_stroke_style(css([0.42, 0.48, 0.58], 0.55));
                    ctx.set_line_width(1.3 * sx);
                    ctx.begin_path();
                    ctx.arc(cx, cy, 2.9 * sx, -0.6, 3.6);
                    ctx.stroke();
                    for i in 0..14 {
                        let a = -0.6 + 4.2 * hash(0x3a1, i, 0);
                        let d = 2.3 + 1.4 * hash(0x3a1, i, 1);
                        spot(ctx, 17.0 + a.cos() * d, 7.7 + a.sin() * d, 0.7, 1.0, [0.50, 0.56, 0.66], 0.6);
                    }
                    disc(ctx, 17.0, 7.6, 1.4, black, 0.95);
                }
                // small tortoiseshell, hindwing
                3 => {
                    ctx.set_fill_style(css([0.74, 0.42, 0.17], 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 6.0, 16.0, 1.0, [0.14, 0.10, 0.08], 1.0);
                    spot(ctx, 3.0, 9.0, 12.0, 1.0, [0.14, 0.10, 0.08], 1.0);
                }
                // peacock, forewing from beneath: near black, finely lined like bark
                _ => {
                    ctx.set_fill_style(css([0.17, 0.14, 0.11], 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    ctx.set_line_width(0.7);
                    for i in 0..90 {
                        let x0 = rnd() * size.0;
                        let z0 = rnd() * size.1;
                        let c = if i % 3 == 0 { [0.30, 0.25, 0.20] } else { [0.08, 0.07, 0.06] };
                        ctx.set_stroke_style(css(c, 0.5));
                        ctx.begin_path();
                        let (a, b) = p(x0, z0);
                        ctx.move_to(a, b);
                        let (c2, d2) = p(x0 + 0.8, z0 + 2.0 + rnd());
                        let (e2, f2) = p(x0 - 0.5, z0 + 4.0);
                        ctx.quadratic_curve_to(c2, d2, e2, f2);
                        ctx.stroke();
                    }
                    spot(ctx, 16.0, 9.0, 7.0, 0.6, [0.26, 0.21, 0.16], 0.6);
                }
            }
            // the dark border along the outer margin, and the blue lunules in it
            ctx.set_line_join("round");
            let n = outline.len();
            let margin: Vec<(f64, f64)> = if hind { outline[3..11].to_vec() } else { outline[5..12].to_vec() };
            ctx.set_stroke_style(css(if k == 4 { [0.32, 0.27, 0.22] } else { [0.16, 0.11, 0.08] }, 0.95));
            ctx.set_line_width(2.2 * sx);
            ctx.begin_path();
            for (i, &(x, z)) in margin.iter().enumerate() {
                let (a, b) = p(x, z);
                if i == 0 {
                    ctx.move_to(a, b);
                } else {
                    ctx.line_to(a, b);
                }
            }
            ctx.stroke();
            if k == 0 || k == 3 {
                for win in margin.windows(2) {
                    let (mx, mz) = ((win[0].0 + win[1].0) / 2.0, (win[0].1 + win[1].1) / 2.0);
                    let (cx, cz) = (size.0 * 0.45, size.1 * 0.45);
                    let d = (mx - cx).hypot(mz - cz).max(1e-6);
                    spot(ctx, mx - (mx - cx) / d * 1.0, mz - (mz - cz) / d * 1.0, 0.7, 0.6, [0.40, 0.48, 0.62], 0.9);
                }
            }
            // the veins: the discal cell's edges and the veins out of it,
            // darker lines a little bowed
            ctx.set_line_width(0.3 * sx);
            ctx.set_stroke_style(css([0.12, 0.09, 0.07], 0.3));
            for &((x0, z0), (x1, z1)) in if hind { &HIND_VEINS } else { &FORE_VEINS } {
                let (a, b) = p(x0, z0);
                let (c2, d2) = p((x0 + x1) / 2.0 + 0.3, (z0 + z1) / 2.0 - 0.5);
                let (e2, f2) = p(x1, z1);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.quadratic_curve_to(c2, d2, e2, f2);
                ctx.stroke();
            }
            // worn: scales rubbed off to the grey membrane at the edges and in patches
            for _ in 0..14 {
                let (i, t) = ((rnd() * n as f64) as usize % n, rnd());
                let (a, b) = (outline[i], outline[(i + 1) % n]);
                let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                spot(ctx, x, z, 1.2 + 2.0 * rnd(), 0.6 + 0.4 * rnd(), [0.52, 0.48, 0.42], 0.35 + 0.3 * rnd());
            }
            for _ in 0..4 {
                spot(ctx, size.0 * (0.25 + 0.6 * rnd()), size.1 * (0.2 + 0.6 * rnd()), 1.0 + 2.0 * rnd(), 0.7, [0.50, 0.46, 0.40], 0.25);
            }
            // dust on it
            for _ in 0..120 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(css([0.55, 0.52, 0.46], 0.15 + 0.2 * rnd()));
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
            ctx.restore();
            // torn: bites out of the outer margin, the root torn away ragged
            ctx.set_global_composite_operation("destination-out");
            for _ in 0..5 {
                let (i, t) = ((rnd() * n as f64) as usize % n, rnd());
                let (a, b) = (outline[i], outline[(i + 1) % n]);
                let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                if x < 4.0 {
                    continue;
                }
                let (cx, cy) = p(x, z);
                ctx.set_fill_style("rgba(0,0,0,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, (0.5 + 1.1 * rnd()) * sx, 0, 7);
                ctx.fill();
            }
            ctx.begin_path();
            let (r0x, r0y) = p(0.0, 0.0);
            ctx.move_to(r0x, r0y);
            for i in 0..=8 {
                let z = size.1 * 0.45 * i as f64 / 8.0;
                let x = 1.6 + 1.6 * rnd();
                let (a, b) = p(x, z);
                ctx.line_to(a, b);
            }
            let (a, b) = p(0.0, size.1 * 0.5);
            ctx.line_to(a, b);
            ctx.close_path();
            ctx.set_fill_style("rgba(0,0,0,1)");
            ctx.fill();
            ctx.set_global_composite_operation("source-over");
        });
        bleed(&tex, [0.30, 0.22, 0.17]);
        tex
    })
}

// Butterfly wing: 5% chance per level. A single wing left on the floor
// (Blender batch `t05`): a peacock's or a small tortoiseshell's, hibernating
// indoors and eaten by a mouse over the winter but for its wings; its scales
// rubbed to the membrane at the edges, its root torn.
pub(super) fn place_butterfly_wings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let wing_colors = [0xd08040, 0x4060c0, 0x8040c0, 0xd0c040, 0x804020];
    // the old wing's colour: which wing, and which side up
    let col = choice(&wing_colors);
    let k = wing_colors.iter().position(|&c| c == col).unwrap_or(0);
    let wing = group();
    let name = if k == 2 || k == 3 { "Real__wingHind" } else { "Real__wingFore" };
    if let Some(geo) = part(name) {
        let m = mesh(geo, lambert!({ map: &wing_tex(k), alphaTest: 0.5, side: DoubleSide }));
        // where the old wing's middle was
        m.position().set(0.025, 0, 0.022);
        m.rotation().y = 0.6;
        wing.add(&m);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    wing.position().set(px, 0, pz);
    wing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wing);
}

// ============================================================ snail shell
/// The garden snail's shell as `t05.py` sweeps it (SH_*): its growth a
/// whorl, whorls, the section's centre radius, radius, drop and height over
/// width at the aperture, how oblique the aperture is
const SH_W: f64 = 2.3;
const SH_N: f64 = 4.3;
const SH_RC: f64 = 0.0085;
const SH_RHO: f64 = 0.0100;
const SH_H: f64 = 0.0175;
const SH_B: f64 = 1.12;
const SH_OBL: f64 = 0.25;

fn sh_t_end() -> f64 {
    SH_N * PI * 2.0
}

/// The shell's surface at turn t and section angle phi (`t05.py` sh_point)
fn sh_point(t: f64, phi: f64) -> [f64; 3] {
    let tt = sh_t_end();
    let s = (SH_W.ln() / (PI * 2.0) * (t - tt)).exp();
    let drop = SH_H * s * (1.0 + 0.10 * sstep(tt - 1.4, tt, t));
    let rho = SH_RHO * s * (1.0 + 0.07 * sstep(tt - 0.3, tt, t));
    let (c, sn) = (t.cos(), t.sin());
    [SH_RC * s * c + c * rho * phi.cos(), -drop + rho * SH_B * phi.sin(), -SH_RC * s * sn - sn * rho * phi.cos()]
}

/// How much further round the last whorl goes at section angle phi
fn sh_obl(phi: f64) -> f64 {
    SH_OBL * (0.5 - 0.5 * phi.sin())
}

/// The four shells the old colour chose: (ground, bands, how strong the
/// bands are, how broken into flames, the lip, the inside; sRGB)
const SHELLS: [([f64; 3], [f64; 3], f64, f64, u32, u32); 4] = [
    // a garden snail's, old and bleached, its periostracum worn to the chalk
    ([0.72, 0.65, 0.52], [0.42, 0.32, 0.22], 0.45, 0.55, 0xd4cfc2, 0x8c8272),
    // a garden snail's, its bands dark, broken by yellow flecks
    ([0.62, 0.50, 0.33], [0.27, 0.18, 0.11], 0.9, 0.45, 0xd8d2c4, 0x6e604c),
    // a grove snail's (Cepaea nemoralis), yellow with five dark bands, its lip dark
    ([0.76, 0.66, 0.36], [0.24, 0.16, 0.10], 0.92, 0.0, 0x3e2c22, 0x8a7a50),
    // a garden snail's, dark with damp
    ([0.46, 0.37, 0.25], [0.21, 0.15, 0.10], 0.85, 0.4, 0xbab4a6, 0x5a4e3e),
];

/// Shell `k` in its part's sphere frame (u round the axis, v down from the
/// apex): the ground mottled, the spiral bands following the whorls (painted
/// along the sweep), growth lines across them, the apex worn pale
fn shell_tex(k: usize) -> Tex {
    t05_kind_tex!(4, k, |k: usize| {
        let (lo, hi) = frame("Real__snailShell");
        let (ground, band, strength, flames, _, _) = SHELLS[k];
        let cepaea = k == 2;
        canvas_tex_opt(512, 256, false, move |ctx, w, h| {
            // where a point of the shell lands in the texture (`uv_for`'s sphere)
            let uv = |p: [f64; 3]| {
                let c: Vec<f64> = (0..3).map(|i| (p[i] - lo[i]) / (hi[i] - lo[i]) * 2.0 - 1.0).collect();
                let u = (c[2].atan2(-c[0]) / (PI * 2.0) + 1.0) % 1.0;
                let r = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-9);
                let v = 1.0 - (c[1] / r).clamp(-1.0, 1.0).acos() / PI;
                (u * w, (1.0 - v) * h)
            };
            paint_px(ctx, w, h, |x, y| {
                let m = fbm(0x5e11 + k as u32, x / 18.0, y / 10.0, 4);
                let streak = vn(0x5e12 + k as u32, x / 3.0, y / 40.0);
                let c = mix(ground, mix(ground, [0.30, 0.25, 0.18], 0.35), sstep(0.4, 0.75, m) * 0.7 + 0.15 * streak);
                [c[0], c[1], c[2], 1.0]
            });
            // a quad of the surface between turns t0..t1 and angles p0..p1
            let quad = |ctx: &mut Canvas, t0: f64, t1: f64, p0: f64, p1: f64, c: [f64; 3], a: f64| {
                let pts = [uv(sh_point(t0, p0)), uv(sh_point(t1, p0)), uv(sh_point(t1, p1)), uv(sh_point(t0, p1))];
                let minx = pts.iter().fold(f64::MAX, |m, p| m.min(p.0));
                let maxx = pts.iter().fold(f64::MIN, |m, p| m.max(p.0));
                // across the seam: drawn on both sides of it
                let wrap = maxx - minx > w * 0.5;
                for shift in [0.0, if wrap { -w } else { f64::NAN }] {
                    if shift.is_nan() {
                        continue;
                    }
                    ctx.set_fill_style(css(c, a));
                    ctx.begin_path();
                    for (i, &(px, py)) in pts.iter().enumerate() {
                        let px = if wrap && px < w * 0.5 { px + w } else { px } + shift;
                        if i == 0 {
                            ctx.move_to(px, py);
                        } else {
                            ctx.line_to(px, py);
                        }
                    }
                    ctx.close_path();
                    ctx.fill();
                }
            };
            let tt = sh_t_end();
            // the bands: (section angle, half width), on the last whorls
            let bands: &[(f64, f64)] = if cepaea {
                &[(0.95, 0.07), (0.45, 0.06), (0.05, 0.09), (-0.45, 0.10), (-0.95, 0.12)]
            } else {
                &[(1.05, 0.10), (0.55, 0.14), (0.15, 0.16), (-0.35, 0.18), (-0.85, 0.14)]
            };
            for (bi, &(pc, hw)) in bands.iter().enumerate() {
                let mut t = tt - 2.8 * PI;
                while t < tt + sh_obl(pc) {
                    let dt = 0.025;
                    // the flames: the band broken by flecks of the ground,
                    // fading in and out along it
                    let f = vn(0xf1a + bi as u32 * 7 + k as u32, t * 5.0, bi as f64 * 3.0);
                    let a = strength * (1.0 - flames * sstep(0.45, 0.7, f));
                    let t1 = (t + dt * 1.3).min(tt + sh_obl(pc));
                    // soft-edged: the band's middle, then its two edges fainter
                    quad(ctx, t, t1, pc - hw * 0.6, pc + hw * 0.6, band, a * 0.8);
                    quad(ctx, t, t1, pc - hw, pc + hw, band, a * 0.45);
                    quad(ctx, t, t1, pc - hw * 1.35, pc + hw * 1.35, band, a * 0.15);
                    t += dt;
                }
            }
            // growth lines across the whorls
            let mut t = tt - 3.0 * PI;
            let mut i = 0u32;
            while t < tt {
                let a = 0.06 + 0.14 * hash(0x9a0 + k as u32, i as i32, 0);
                let c = if i % 3 == 0 { [0.86, 0.80, 0.68] } else { [0.30, 0.24, 0.17] };
                for j in 0..14 {
                    let (p0, p1) = (-PI + PI * 2.0 * j as f64 / 14.0, -PI + PI * 2.0 * (j + 1) as f64 / 14.0);
                    quad(ctx, t, t + 0.012, p0, p1, c, a);
                }
                t += 0.04 + 0.08 * hash(0x9a1 + k as u32, i as i32, 1);
                i += 1;
            }
            // the apex worn pale, and dust in the sutures' shadow at the top
            blob(ctx, w * 0.5, 0.0, h * 0.22, w / h * 0.25, 0.0, [0.78, 0.74, 0.66], 0.6);
            ctx.set_fill_style(css([0.78, 0.74, 0.66], 0.35));
            ctx.fill_rect(0, 0, w, h * 0.06);
        })
    })
}

// Snail shell: 5% chance per level. An empty snail's shell (Blender
// batch `t05`): a garden snail's, 4.5 whorls, its bands following them, its
// white lip turned back, or a grove snail's; lying on its aperture or on its
// side.
pub(super) fn place_snail_shells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0xc8a888, 0xa08060, 0xd0b090, 0x604020];
    // the old colour: which shell
    let col = choice(&shell_colors);
    let k = shell_colors.iter().position(|&c| c == col).unwrap_or(0);
    let (_, _, _, _, lip, inside) = SHELLS[k];
    let posed = group();
    // the shell's wall seen from within too, deep in the aperture (its bands
    // show through it, as a real one's do)
    if let Some(geo) = part("Real__snailShell") {
        posed.add(&mesh(geo, phong!({ map: &shell_tex(k), specular: 0x1e1c18, shininess: 22, side: DoubleSide })));
    }
    if let Some(geo) = part("Real__snailLip") {
        posed.add(&mesh(geo, phong!({ color: lip, specular: 0x24221e, shininess: 30 })));
    }
    if let Some(geo) = part("Real__snailInside") {
        posed.add(&mesh(geo, lambert!({ color: inside, side: DoubleSide })));
    }
    // a grove snail's is smaller (22 mm across to the garden snail's 32)
    let s = if k == 2 { 0.7 } else { 1.0 };
    posed.scale().set_scalar(s);
    let shell = group();
    let (px, pz) = jitter(g, x, z, 2.5);
    shell.position().set(px, 0, pz);
    shell.rotation().y = rnd() * PI * 2.0;
    // the old "on its side" (40%): on its side, else on its aperture
    let side = rnd() < 0.4;
    if side {
        posed.rotation().z = PI * 0.5;
    } else {
        // the aperture's face (the sweep's way at its end) turned to the floor
        let tt = sh_t_end();
        let (a, b) = (sh_point(tt - 0.05, -PI / 2.0), sh_point(tt + sh_obl(-PI / 2.0), -PI / 2.0));
        let mut d = Vec3::new(b[0] - a[0], (b[1] - a[1]) * 0.6, b[2] - a[2]);
        d.normalize();
        let mut down = Vec3::new(0.15, -1.0, 0.0);
        down.normalize();
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&d, &down);
        posed.quaternion().copy(&q);
    }
    // its lowest point on the floor, its middle where the old shell's was
    let mut bb = Box3::default();
    bb.set_from_object(&posed);
    posed.position().set(-(bb.min.x + bb.max.x) / 2.0, -bb.min.y, -(bb.min.z + bb.max.z) / 2.0);
    shell.add(&posed);
    g.world.add(&shell);
}

// =================================================================== nest
/// The cup's floor (`t05.py` NEST_FLOOR): the eggs lie on it
const NEST_FLOOR: f64 = 0.026;

/// Dry grass and stems matted over mud, seen from any side (the nest's
/// faces take it by the box's three ways): straw and grey-brown stems every
/// way but mostly round it, mud showing between them, a few dark rootlets
fn nest_grass_tex() -> Tex {
    t05_tex!(512, 512, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let m = fbm(0x7e57, x / 30.0, y / 30.0, 4);
            let c = mix([0.34, 0.28, 0.20], [0.24, 0.20, 0.15], sstep(0.4, 0.7, m));
            [c[0], c[1], c[2], 1.0]
        });
        ctx.set_line_cap("round");
        for i in 0..2600 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = if i % 4 == 0 { rnd() * PI } else { (rnd() - 0.5) * 0.7 };
            let l = 12.0 + rnd() * 50.0;
            let bend = (rnd() - 0.5) * 0.5;
            let tone = rnd();
            let c = if tone < 0.55 {
                mix([0.62, 0.55, 0.40], [0.52, 0.45, 0.32], rnd())
            } else if tone < 0.85 {
                mix([0.44, 0.38, 0.29], [0.36, 0.31, 0.24], rnd())
            } else {
                [0.22, 0.17, 0.12]
            };
            ctx.set_stroke_style(css(c, 0.55 + 0.4 * rnd()));
            ctx.set_line_width(0.8 + rnd() * 2.2);
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            ctx.begin_path();
            ctx.move_to(x - dx, y - dy);
            ctx.quadratic_curve_to(x - dy * bend, y + dx * bend, x + dx, y + dy);
            ctx.stroke();
        }
        // dust settled on it over the years
        for _ in 0..40 {
            blob(ctx, rnd() * w, rnd() * h, 10.0 + rnd() * 30.0, 0.6 + 0.4 * rnd(), rnd() * 3.0, [0.50, 0.47, 0.42], 0.10 + 0.12 * rnd());
        }
    })
}

/// The lining: fine dry grass laid round and round the cup (concentric seen
/// from above, where its floor takes it; round it on its walls)
fn nest_lining_tex() -> Tex {
    t05_tex!(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style(css([0.46, 0.40, 0.31], 1.0));
        ctx.fill_rect(0, 0, w, h);
        ctx.set_line_cap("round");
        for _ in 0..900 {
            let r = rnd().sqrt() * w * 0.5;
            let a0 = rnd() * PI * 2.0;
            let span = 0.4 + rnd() * 1.2;
            let c = mix([0.66, 0.58, 0.44], [0.40, 0.34, 0.26], rnd());
            ctx.set_stroke_style(css(c, 0.5 + 0.45 * rnd()));
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.arc_dir(w / 2.0 + (rnd() - 0.5) * 6.0, h / 2.0 + (rnd() - 0.5) * 6.0, r, a0, a0 + span, false);
            ctx.stroke();
        }
        // the dust and droppings of an old nest, pale specks in its bottom
        for _ in 0..30 {
            let (a, r) = (rnd() * PI * 2.0, rnd() * w * 0.25);
            blob(ctx, w / 2.0 + a.cos() * r, h / 2.0 + a.sin() * r, 2.0 + rnd() * 4.0, 1.0, 0.0, [0.80, 0.78, 0.72], 0.5);
        }
        blob(ctx, w / 2.0, h / 2.0, w * 0.3, 1.0, 0.0, [0.48, 0.45, 0.40], 0.3);
    })
}

/// A robin's egg gone dull: its blue faded, dirt and a stain on it
fn nest_egg_tex() -> Tex {
    t05_tex!(64, 64, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let m = fbm(0xe66, x / 9.0, y / 9.0, 3);
            let c = mix([0.62, 0.72, 0.68], [0.55, 0.60, 0.55], sstep(0.4, 0.8, m));
            [c[0], c[1], c[2], 1.0]
        });
        for _ in 0..24 {
            blob(ctx, rnd() * w, rnd() * h, 1.0 + rnd() * 2.5, 1.0, 0.0, [0.35, 0.30, 0.24], 0.4 + 0.4 * rnd());
        }
        blob(ctx, w * 0.3, h * 0.7, 14.0, 0.6, 0.4, [0.45, 0.40, 0.30], 0.35);
    })
}

// Bird nest: 4% chance per level. A robin's nest fallen to the floor
// (Blender batch `t05`): a cup of dry grass and twigs over mud, lined with
// fine grass, loose stems trailing from it; now and then an egg or two left
// in it, one broken.
pub(super) fn place_bird_nests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let twig_colors = [0x604020, 0x503018, 0x705030];
    let nest = group();
    // the old outer ring's twelve twigs: each one's colour and tilt; the
    // first colour is the nest's (grey-brown, darker, paler)
    let mut tone = 0usize;
    for i in 0..12 {
        let c = choice(&twig_colors);
        if i == 0 {
            tone = twig_colors.iter().position(|&t| t == c).unwrap_or(0);
        }
        let _ = rnd();
    }
    // the inner ring's ten (kept for their draws)
    for _i in 0..10 {
        let _ = choice(&twig_colors);
        let _ = rnd();
    }
    const TONE: [u32; 3] = [0xffffff, 0xd8d2c8, 0xfff8ec];
    let grass = lambert!({ map: &nest_grass_tex(), color: TONE[tone] });
    for (name, mat) in [
        ("Real__nestCup", grass),
        ("Real__nestLining", lambert!({ map: &nest_lining_tex(), color: TONE[tone] })),
        ("Real__nestTwigs", lambert!({ map: &bark_tex(), color: 0xb0a490 })),
    ] {
        if let Some(geo) = part(name) {
            nest.add(&mesh(geo, &mat));
        }
    }
    // the loose stems wound round it and standing out of it: dry grass, thin
    for (name, c) in [("Real__nestStrands", 0x8c7c62), ("Real__nestStray", 0x9a8a6e)] {
        if let Some(geo) = thin_lit(name) {
            nest.add(&mesh(geo, lambert!({ color: c })));
        }
    }
    // 30%: one or two eggs left in it (the second broken)
    if rnd() < 0.30 {
        let egg_mat = lambert!({ map: &nest_egg_tex() });
        let egg_count = 1 + (rnd() * 2.0).floor() as i32;
        for e in 0..egg_count {
            let (ex, ez) = ((rnd() - 0.5) * 0.020, (rnd() - 0.5) * 0.020);
            let name = if e == 0 { "Real__nestEgg" } else { "Real__nestEggBroken" };
            let Some(geo) = part(name) else { continue };
            let egg = mesh(geo, &egg_mat);
            // lying in the cup's floor, turned by where it lies
            let (ox, oz) = if e == 0 { (ex - 0.012, ez) } else { (ex + 0.016, ez + 0.006) };
            egg.position().set(ox, NEST_FLOOR - 0.001 + 0.18 * (ox * ox + oz * oz).sqrt(), oz);
            egg.rotation().y = (ex * 300.0).sin() * PI;
            nest.add(&egg);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    nest.position().set(px, 0, pz);
    nest.rotation().y = rnd() * PI * 2.0;
    g.world.add(&nest);
}

// =================================================================== moss
/// The moss colours the old placer chose among: green, olive, dark, dry
/// (sRGB, as the shoots' tips are)
const MOSS_RGB: [[f64; 3]; 4] = [[0.44, 0.54, 0.24], [0.50, 0.52, 0.28], [0.34, 0.42, 0.20], [0.58, 0.56, 0.36]];

/// A cushion's shoots seen close: tiny leaves packed into a fur, the tips
/// paler (silvery where they are dry), darker in the gaps between
fn moss_shoots_tex() -> Tex {
    t05_tex!(128, 128, false, |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let fine = vn(0x3055, x / 1.6, y / 1.6);
            let mid = fbm(0x3056, x / 7.0, y / 7.0, 3);
            let k = 0.55 + 0.6 * fine * fine + 0.3 * (mid - 0.5);
            let tip = sstep(0.75, 0.95, fine);
            let c = mix([0.62 * k, 0.66 * k, 0.48 * k], [0.86, 0.86, 0.76], 0.35 * tip);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The mat the cushions grow out of, painted on the floor (`salt`: where it
/// is): a felt of shoots, green at its heart, browner and patchy out to a
/// ragged edge where it thins into the floor's dust; silt dried round it
fn moss_mat_tex(base: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w - 0.5, y / h - 0.5);
            let r = u.hypot(v) * 2.0;
            let a = v.atan2(u);
            let edge = 0.62 + 0.22 * (fbm(salt, a.cos() * 2.0 + 5.0, a.sin() * 2.0 + 5.0, 3) - 0.5) * 2.0;
            let rr = r + 0.35 * (fbm(salt ^ 0x5, u * 7.0, v * 7.0, 4) - 0.5);
            // out at its edge it breaks into small colonies, then single tufts
            let tufts = sstep(0.62, 0.7, vn(salt ^ 0x15, x / 4.0, y / 4.0)) * (1.0 - sstep(edge + 0.02, edge + 0.16, rr));
            let cover = (1.0 - sstep(edge - 0.1, edge + 0.0, rr)).max(tufts);
            // the shoots: each a speck, its tip catching the light
            let shoots = vn(salt ^ 0x9, x / 1.1, y / 1.1);
            let tips = sstep(0.62, 0.8, vn(salt ^ 0xa, x / 0.8, y / 0.8));
            // colonies, with bare floor between some of them
            let patch = fbm(salt ^ 0x3, u * 5.0, v * 5.0, 4);
            let mut c = mix(base, [0.46, 0.42, 0.28], sstep(0.4, 0.95, r) * 0.6 + 0.3 * sstep(0.55, 0.75, patch));
            c = mix([c[0] * 0.45, c[1] * 0.45, c[2] * 0.45], c, 0.3 + 0.7 * shoots);
            c = mix(c, [0.78, 0.80, 0.62], 0.35 * tips);
            // the silt the damp brought, dried in a ring round it
            let silt = sstep(edge - 0.1, edge + 0.1, r) * (1.0 - sstep(edge + 0.1, edge + 0.32, r));
            let a_moss = cover * (0.7 + 0.3 * shoots) * sstep(0.32, 0.4, patch + 0.25 * (1.0 - r));
            let a_silt = 0.2 * silt;
            let al = a_moss + a_silt * (1.0 - a_moss);
            let col = if al > 0.0 { mix([0.36, 0.32, 0.25], c, a_moss / al) } else { c };
            [col[0], col[1], col[2], al]
        });
    })
}

// Moss patch: 5% chance per level. Moss grown on a floor kept damp
// (Blender batch `t05`): a felted mat painted on the floor, cushions of
// packed shoots rising out of it, here and there their setae and nodding
// capsules.
pub(super) fn place_moss_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let moss_colors = [0x407030, 0x506040, 0x304018, 0x607040];
    let patch = group();
    // the old mat's colour: the mat's
    let base_color = choice(&moss_colors);
    let kb = moss_colors.iter().position(|&c| c == base_color).unwrap_or(0);
    let shoots = moss_shoots_tex();
    let setae = lambert!({ color: 0x5a3424 });
    let mut tints: [Option<Mat>; 4] = Default::default();
    // the old twelve tufts: each one's colour, size and where it is (now a cushion's)
    let mut first = true;
    for _i in 0..12 {
        let c = choice(&moss_colors);
        let kc = moss_colors.iter().position(|&m| m == c).unwrap_or(0);
        let bump_r = 0.008 + rnd() * 0.010;
        let a = rnd() * PI * 2.0;
        let r = rnd() * 0.10;
        let (name, nominal) = if bump_r < 0.0113 {
            ("Real__mossCushionC", 0.0095)
        } else if bump_r < 0.0147 {
            ("Real__mossCushionB", 0.013)
        } else {
            ("Real__mossCushionA", 0.0175)
        };
        let Some(geo) = part(name) else { continue };
        let tint = MOSS_RGB[kc];
        let mat = tints[kc]
            .get_or_insert_with(|| {
                // the shoots' texture is about (0.52, 0.55, 0.40): the tint brings it to the colour
                let t = ((tint[0] / 0.52 * 255.0).min(255.0) as u32) << 16
                    | ((tint[1] / 0.55 * 255.0).min(255.0) as u32) << 8
                    | (tint[2] / 0.40 * 255.0).min(255.0) as u32;
                lambert!({ map: &shoots, color: t })
            })
            .clone();
        let cushion = group();
        cushion.add(&mesh(geo, &mat));
        // the old tuft's colour: setae on the first and on the green ones, in fruit
        if kc == 0 || first {
            if let Some(sg) = part("Real__mossSetae") {
                let s = mesh(sg, &setae);
                s.position().y = 0.002;
                cushion.add(&s);
            }
            first = false;
        }
        cushion.scale().set_scalar((bump_r / nominal).clamp(0.85, 1.15));
        cushion.position().set(a.cos() * r, 0, a.sin() * r);
        cushion.rotation().y = a * 3.0;
        patch.add(&cushion);
    }
    let (px, pz) = jitter(g, x, z, 2.2);
    patch.position().set(px, 0, pz);
    patch.rotation().y = rnd() * PI * 2.0;
    let mat_tex = moss_mat_tex(MOSS_RGB[kb], salt_of(px, pz));
    patch.add(&floor_mark(&mat_tex, 0.36, 0.36, 0.0, 0.0, 0.0));
    g.world.add(&patch);
}

// ================================================================== grass
/// The tillers by height (`t05.py` TILLERS)
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
            let c = mix([0.22, 0.19, 0.15], [0.36, 0.32, 0.26], sstep(0.1, 0.8, r) * 0.7 + 0.3 * grit);
            [c[0], c[1], c[2], (1.0 - sstep(0.55, 0.95, r)) * (0.75 + 0.25 * grit)]
        });
    })
}

// Grass clump: 6% chance per level. A tuft of annual meadow grass grown
// up out of a drift of silt on a damp floor (Blender batch `t05`): tillers of
// folded, boat-tipped blades, pale and lank from the dark, a flowering stem
// or two, the dead blades round its foot; it stirs in a draught.
pub(super) fn place_grass_clumps(g: &mut Game, _dist: &[i32]) {
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

// ==================================================================== web
/// The finest threads of the tangle, too many and too fine to model (seen
/// together they are a grey veil where the dust has caught in them): the
/// veil thickest in the tangle's heart, wisps along the ways it was spun,
/// strands crossing it, dust caught in clumps, torn through in places
/// (`salt`: where it hangs; the canvas's top is the ceiling's side)
fn web_veil_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x3eb);
        let (bx, by) = (w * (0.42 + 0.16 * r.next()), h * (0.35 + 0.2 * r.next()));
        let (brx, bry) = (w * (0.3 + 0.1 * r.next()), h * (0.28 + 0.1 * r.next()));
        let (wa, wb) = (r.next() * PI, r.next() * PI);
        let dust = [0.66, 0.65, 0.62];
        paint_px(ctx, w, h, |x, y| {
            let (u, v) = (x / w, y / h);
            let q = ((x - bx) / brx).hypot((y - by) / bry);
            let dens = (-q * q * 0.9).exp();
            let rag = fbm(salt ^ 0x99, u * 7.0, v * 3.0, 3) - 0.5;
            let ends = sstep(0.0, 0.18, u.min(1.0 - u) + rag * 0.12) * sstep(0.0, 0.2, v.min(1.0 - v) + rag * 0.2);
            let (u1, u2) = (u * wa.cos() + v * wa.sin(), -u * wa.sin() + v * wa.cos());
            let (v1, v2) = (u * wb.cos() + v * wb.sin(), -u * wb.sin() + v * wb.cos());
            let wisp = 0.5 * vn(salt ^ 0x5e, u1 * 3.0, u2 * 45.0) + 0.5 * vn(salt ^ 0x6f, v1 * 3.0, v2 * 45.0);
            let torn = sstep(0.3, 0.45, fbm(salt ^ 0x7a, u * 4.0, v * 3.0, 4) + 0.2 * dens);
            [dust[0], dust[1], dust[2], ((0.03 + 0.5 * dens) * ends * (0.15 + 1.6 * wisp * wisp * wisp) * torn).min(0.5)]
        });
        ctx.set_line_cap("round");
        let in_blob = |r: &mut SplitMix64, k: f64| {
            let (a, d) = (r.next() * PI * 2.0, r.next().powf(0.6) * k);
            (bx + a.cos() * d * brx, by + a.sin() * d * bry)
        };
        // strands along the two ways it was spun, many over each other, each
        // sagging a little between where it is held
        for k in 0..230 {
            let c = in_blob(&mut r, 1.2);
            let a = if k % 2 == 0 { wa } else { wb } + (r.next() - 0.5) * 1.2;
            let l = w * (0.1 + 0.4 * r.next());
            let (dx, dy) = (a.cos() * l / 2.0, a.sin() * l / 2.0);
            let tone = if k % 7 == 0 { [0.56, 0.55, 0.52] } else { [0.88, 0.88, 0.86] };
            ctx.set_stroke_style(css(tone, 0.2 + 0.3 * r.next()));
            ctx.set_line_width(0.6 + 0.6 * r.next());
            let sag = l * (0.04 + 0.1 * r.next());
            ctx.begin_path();
            ctx.move_to(c.0 - dx, c.1 - dy);
            ctx.quadratic_curve_to(c.0, c.1 + sag, c.0 + dx, c.1 + dy);
            ctx.stroke();
        }
        // lines out to where it is fixed: the ceiling (top) and the wall (bottom)
        for _ in 0..14 {
            let a = in_blob(&mut r, 0.5);
            let f = (w * (0.05 + 0.9 * r.next()), if r.next() < 0.5 { 0.0 } else { h });
            let heavy = r.next() < 0.3;
            ctx.set_stroke_style(css(if heavy { [0.56, 0.54, 0.50] } else { [0.80, 0.79, 0.76] }, if heavy { 0.45 } else { 0.3 }));
            ctx.set_line_width(if heavy { 1.8 } else { 0.9 });
            ctx.begin_path();
            ctx.move_to(a.0, a.1);
            ctx.quadratic_curve_to((a.0 + f.0) / 2.0, (a.1 + f.1) / 2.0 + 6.0, f.0, f.1);
            ctx.stroke();
        }
        // dust: specks all through it, clumps where it is thickest
        for _ in 0..90 {
            let p = in_blob(&mut r, 1.0);
            ctx.set_fill_style(css([0.58, 0.56, 0.52], 0.2 + 0.35 * r.next()));
            ctx.begin_path();
            ctx.arc(p.0, p.1, 0.5 + 1.0 * r.next(), 0, 7);
            ctx.fill();
        }
        for _ in 0..8 {
            let p = in_blob(&mut r, 0.7);
            blob(ctx, p.0, p.1, 4.0 + 8.0 * r.next(), 0.5 + 0.5 * r.next(), r.next() * 3.0, [0.62, 0.60, 0.56], 0.45);
        }
    })
}

// Spider web: 5% chance per level. A cellar spider's web in the angle of
// the wall and the ceiling (Blender batch `t05`): a loose tangle of fine
// threads grey with dust, guy lines to the wall and the ceiling, strands
// heavy with dust; the spider dead in it, hanging, prey wrapped and hung.
// It keeps the old web's place and its width along the wall.
pub(super) fn place_spider_webs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let web = group();
    // the web's frame: the wall at z = 0, the ceiling at y = 0; the group
    // stands 2 cm off the wall, as the old one did
    let o = group();
    o.position().set(0, WALL_H, -0.020);
    web.add(&o);
    let threads = phong!({ color: 0xa8a49a, specular: 0x34332f, shininess: 30, transparent: true, opacity: 0.7, depthWrite: false });
    if let Some(geo) = part("Real__spiderWebThreads") {
        o.add(&mesh(geo, &threads));
    }
    let salt = salt_of(g.wx(x) + dx as f64, g.wz(z) + dz as f64);
    // the spider and its prey, where it hung them (not in every web: by where it is)
    if salt % 4 != 0 {
        if let Some(geo) = part("Real__spiderBody") {
            o.add(&mesh(geo, lambert!({ color: 0x8a7e6c })));
        }
    }
    if salt % 3 != 0 {
        if let Some(geo) = part("Real__spiderWrap") {
            o.add(&mesh(geo, lambert!({ color: 0x9e9a90 })));
        }
    }
    // the finest threads: a sheet painted across the angle (the threads'
    // hammock, `t05.py`: from the wall 20 cm under the ceiling to the
    // ceiling 12.5 cm out, sagging into the room), and a thinner one a
    // little under it where it sags (the old second sheet down the wall's
    // face read as a hairball on the wall, M8)
    let (down, out) = (0.20, 0.125);
    let (len, tilt) = (f64::hypot(down, out), out.atan2(down));
    // out of the angle, into the room: (y, z) = (-out, down) / len
    let sag = |d: f64| (-out / len * d, down / len * d);
    let veil = lambert!({ map: &web_veil_tex(salt), transparent: true, depthWrite: false, side: DoubleSide });
    let across = mesh(plane_geo!(0.32, len), &veil);
    let (sy, sz) = sag(0.010);
    across.position().set(0.0, -down / 2.0 + sy, out / 2.0 + sz);
    across.rotation().x = tilt;
    o.add(&across);
    let under = mesh(
        plane_geo!(0.24, len * 0.7),
        lambert!({ map: &web_veil_tex(salt ^ 0x51), transparent: true, depthWrite: false, side: DoubleSide }),
    );
    let (sy, sz) = sag(0.024);
    under.position().set(0.01, -down / 2.0 + sy, out / 2.0 + sz);
    under.rotation().x = tilt;
    o.add(&under);
    wall_mount(g, &web, x, z, dx, dz, 0.020);
    // it hangs from the ceiling and the wall by its threads: the placement
    // pass leaves it be (it never saw the old see-through web, and would let
    // the spider, the one solid thing in it, down to the floor)
    web.user().floats = true;
    g.world.add(&web);
}
