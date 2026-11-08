//! Batch `t21`'s prop in this crate, made real (`docs/PROPS.md` P3): the
//! drop-front desk (`assets/src/blender/props/t21.py`, `t21.notes.md`). Its
//! placer and what only it uses; it keeps the old placer's draws, call for
//! call.
use crate::{open_tiles, ri};
use game::prelude::*;
use sd_kit::*;

/// The desk (`t21.py` SD_*): the case's half width inside its sides, its
/// front, the writing bed (the fall's face), the pigeonholes' floor and
/// the fitted interior's front, the back's inner face; the fall's width
/// and depth
const SD_INNER: f64 = 0.550 / 2.0 - 0.018;
const SD_FRONT: f64 = 0.170;
const SD_BED: f64 = 0.740;
const SD_SHELF: f64 = SD_BED + 0.066;
const SD_FIT_Z: f64 = 0.080;
const SD_BACK_IN: f64 = -0.170 + 0.010;
const SD_FALL: (f64, f64) = (0.512, 0.300);
/// The leather's face (under the wood's by 1.6 mm)
const SD_LEATHER_Y: f64 = SD_BED - 0.0016;

// Vertical secretary desk with drop-front writing surface (floor): 2%.
// A petite drop-front desk of the 1920s (Blender batch `t21`): four square
// tapered legs, two drawers with Hepplewhite bail pulls, the fall front
// lying open on its brass stays over the fitted interior (four small
// drawers, six pigeonholes under arched valances), a moulded top and a low
// crest; veneered in one of four woods (the old colour draw), its finish
// worn through at the edges, crazed and dull, the leather of its fall
// scuffed and inked. Letters in some pigeonholes (each its old draw);
// sheets of note paper on the fall and an inkwell, dry, as the old one had.
pub(crate) fn place_secretary_desks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old draws, call for call: the wood, a letter in each pigeonhole,
    // the papers on the fall (how many, where, how turned), the inkwell, the
    // slide off the tile and the turn
    let wood = ri(4);
    let holes: Vec<f64> = (0..6).map(|_| rnd()).collect();
    let mut sheets: Vec<(f64, f64)> = vec![];
    let mut inkwell = false;
    if rnd() < 0.6 {
        let paper_count = 1 + (rnd() * 3.0).floor() as i32;
        for _ in 0..paper_count {
            let px = (rnd() - 0.5) * SD_FALL.0 * 0.5;
            let turn = (rnd() - 0.5) * 0.3;
            sheets.push((px, turn));
        }
        inkwell = rnd() < 0.4;
    }
    let dx = (rnd() - 0.5) * 0.6;
    let dz = (rnd() - 0.5) * 0.6;
    let turn = rnd() * PI * 2.0;
    let (px, pz) = (g.wx(x) + dx, g.wz(z) + dz);
    let salt = salt(px, pz, 0x5d);
    let sec = group();
    put(&sec, "Real__secDeskCase", &lambert!({ map: &wood_tex(salt, wood, false) }));
    put(&sec, "Real__secDeskFronts", &lambert!({ map: &wood_tex(salt ^ 0x77, wood, true) }));
    put(&sec, "Real__secDeskLeather", &lambert!({ map: &leather_tex(salt, wood) }));
    let brass = phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x5a4628, shininess: 26 });
    put(&sec, "Real__secDeskBrass", &brass);
    // what stands in the pigeonholes (its old 30% each): envelopes, or a
    // roll of papers on a folded letter
    let paper = lambert!({ map: &paper_tex(salt), color: 0xe6dccb });
    let hole_w = 2.0 * SD_INNER / 6.0;
    for (i, &r) in holes.iter().enumerate() {
        if r < 0.3 {
            let name = if r < 0.15 { "Real__secDeskLettersA" } else { "Real__secDeskLettersB" };
            if let Some(m) = put(&sec, name, &paper) {
                m.position().set(-SD_INNER + hole_w * (i as f64 + 0.5), SD_SHELF, (SD_FIT_Z + SD_BACK_IN) / 2.0);
            }
        }
    }
    // the papers on the fall, one on another, each a little turned
    let note = lambert!({ map: &note_tex(salt), color: 0xffffff });
    for (i, &(sx, st)) in sheets.iter().enumerate() {
        if let Some(m) = put(&sec, "Real__secDeskSheet", &note) {
            m.position().set(sx, SD_LEATHER_Y + 0.0004 + i as f64 * 0.0009, SD_FRONT + SD_FALL.1 * 0.5);
            m.rotation().y = st;
        }
    }
    // a pressed glass inkwell, its cap thrown back, the ink dried in it
    if inkwell {
        let ink = group();
        ink.position().set(SD_FALL.0 * 0.35, SD_LEATHER_Y, SD_FRONT + SD_FALL.1 * 0.3);
        ink.rotation().y = 0.3;
        if let Some(m) = put(&ink, "Real__secDeskInkwell", &ink_glass_mat()) {
            m.set_render_order(2);
        }
        put(&ink, "Real__secDeskInkCap", &brass);
        put(&ink, "Real__secDeskInk", &lambert!({ color: 0x0c0c10 }));
        sec.add(&ink);
    }
    sec.position().set(px, 0, pz);
    sec.rotation().y = turn;
    g.world.add(&sec);
}

/// A wood's veneer under its old finish (sRGB: the ground, the figure): a
/// walnut, a mahogany stained dark, a golden oak, a red mahogany (the old
/// four colours' order)
const WOODS: [([f64; 3], [f64; 3]); 4] = [
    ([0.40, 0.27, 0.17], [0.22, 0.14, 0.08]),
    ([0.25, 0.15, 0.10], [0.13, 0.075, 0.05]),
    ([0.58, 0.44, 0.27], [0.40, 0.28, 0.15]),
    ([0.43, 0.21, 0.13], [0.24, 0.11, 0.07]),
];

/// The veneer as its part is laid (u right, v up, or across the top): its
/// grain up the part (the case: sides, legs) or across it (the fronts and
/// the fall); the figure and its fine streaks, a shellac gone dull and
/// crazed, worn through to paler wood in patches, a grey bloom of dust
fn wood_tex(salt: u32, wood: usize, across: bool) -> Tex {
    canvas_tex_opt(192, 320, false, move |ctx, w, h| {
        let seed = salt ^ 0x3d0d;
        let (ground, dark) = WOODS[wood.min(3)];
        fill(ctx, w, h, |u, v| {
            let (a, b) = if across { (u * 0.55, v * 1.1) } else { (v * 1.1, u * 0.57) };
            // the figure: a broad tone, thin growth lines along the grain a
            // little waved, fine streaks
            let big = fbm(seed, a * 1.2, b * 4.0, 3);
            let warp = fbm(seed ^ 1, a * 3.0, b * 14.0, 3);
            let lines = ((b * 300.0 + warp * 5.0).sin() * 0.5 + 0.5).powf(3.0);
            let fine = fbm(seed ^ 5, a * 12.0, b * 110.0, 2);
            let mut c = mix(ground, dark, 0.25 + 0.35 * (big - 0.5) + 0.30 * lines + 0.25 * (fine - 0.5));
            // the finish worn through to paler wood, and gone grey-dull
            let wear = (fbm(seed ^ 0x11, u * 5.0, v * 5.0, 3) - 0.62).max(0.0) * 2.6;
            c = mix(c, mix(ground, [0.60, 0.52, 0.42], 0.45), wear);
            // the crazed shellac: a fine net of dark lines
            let cr = vn(seed ^ 0x21, u * 60.0, v * 100.0);
            if (cr - 0.5).abs() < 0.02 {
                c = c.map(|k| k * 0.82);
            }
            // dust and grey bloom over it
            let bloom = 0.10 + 0.10 * fbm(seed ^ 0x31, u * 3.0, v * 3.0, 2);
            c = mix(c, [0.46, 0.43, 0.38], bloom);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The fall's leather (laid across its top: u across, v from its hinge out
/// to its free edge): a dark green or an oxblood skin with its grain, a
/// gilt line tooled round it (most of the gold gone) and a blind one inside
/// it, scuffed pale where hands and sleeves rubbed, an ink blot, a ring
fn leather_tex(salt: u32, wood: usize) -> Tex {
    canvas_tex_opt(192, 104, false, move |ctx, w, h| {
        let seed = salt ^ 0x1ea7;
        let skin = if wood % 2 == 0 { [0.15, 0.21, 0.15] } else { [0.30, 0.11, 0.09] };
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.436, v * 0.232);
            let grain = vn(seed, x * 900.0, y * 900.0) * 0.5 + fbm(seed ^ 3, x * 40.0, y * 40.0, 3) * 0.5;
            let mut c = skin.map(|k| k * (0.85 + 0.3 * grain));
            // the tooled lines, 12 mm and 16 mm in
            let d = x.min(0.436 - x).min(y).min(0.232 - y);
            if (d - 0.012).abs() < 0.0012 {
                let gold = (fbm(seed ^ 7, x * 60.0, y * 60.0, 2) - 0.45).max(0.0) * 2.0;
                c = mix(c.map(|k| k * 0.7), [0.55, 0.45, 0.24], gold);
            } else if (d - 0.016).abs() < 0.0008 {
                c = c.map(|k| k * 0.72);
            }
            // scuffed where it was written on: the middle toward the hinge
            let scuff = (fbm(seed ^ 0x51, x * 40.0, y * 60.0, 3) - 0.55).max(0.0)
                * 2.0
                * (1.0 - sstep(0.05, 0.20, (u - 0.5).abs() * 0.4 + v * 0.2));
            c = mix(c, [0.40, 0.37, 0.32], scuff * 0.5);
            // an ink blot, and a ring where a glass stood
            let (bx, by) = (0.10 + 0.20 * hash(seed, 1, 1), 0.06 + 0.10 * hash(seed, 1, 2));
            let blot = ((x - bx).hypot((y - by) * 1.3) + 0.004 * vn(seed ^ 0xb, x * 300.0, y * 300.0)) < 0.009;
            if blot {
                c = mix(c, [0.04, 0.04, 0.06], 0.85);
            }
            let (rx, ry) = (0.32 + 0.06 * hash(seed, 2, 1), 0.12 + 0.06 * hash(seed, 2, 2));
            let rd = ((x - rx).hypot(y - ry) - 0.032).abs();
            if rd < 0.0018 {
                c = mix(c, [0.40, 0.38, 0.34], 0.35 * (1.0 - rd / 0.0018));
            }
            // dust over it
            c = mix(c, [0.45, 0.43, 0.39], 0.12);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The envelopes' and the letters' paper (on its colour: how bright): gone
/// to ivory and brown at its edges (foxing), dust on what faced up
fn paper_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xe7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let edge = 1.0 - sstep(0.0, 0.12, u.min(1.0 - u).min(v).min(1.0 - v));
            let fox = (vn(seed ^ 3, u * 30.0, v * 30.0) - 0.78).max(0.0) * 3.0;
            let g = 0.92 - 0.10 * n - 0.20 * edge - 0.25 * fox;
            [g, g * 0.97, g * 0.90, 1.0]
        });
    })
}

/// A half sheet of note paper written on in ink, faded (u across, v down
/// the sheet as it lies, its head away from the hinge): a hand's lines,
/// a date and a greeting, a signature; the ink gone brown, the sheet ivory
fn note_tex(salt: u32) -> Tex {
    canvas_tex_opt(140, 216, false, move |ctx, w, h| {
        let seed = salt ^ 0x40e;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 140.0, (1.0 - v) * 216.0); // mm, from its head
            let n = fbm(seed, u * 5.0, v * 7.0, 3);
            let mut c = [0.88 - 0.06 * n, 0.85 - 0.06 * n, 0.76 - 0.07 * n];
            // the hand: lines 8 mm apart, words of loops along each
            let line = ((y - 30.0) / 8.0).floor();
            let in_line = (y - 30.0) - line * 8.0;
            let rows = 18.0;
            if (0.0..rows).contains(&line) {
                let li = line as i32;
                let x0 = if li == 0 { 14.0 } else { 12.0 };
                let x1 = if li == rows as i32 - 1 { 70.0 } else { 128.0 - 10.0 * hash(seed, li, 3) };
                if x > x0 && x < x1 {
                    // words: runs of letters with gaps between them
                    let wpos = x + 7.0 * hash(seed, li, 4);
                    let word = (wpos / 13.0).floor();
                    let inw = wpos - word * 13.0;
                    let wlen = 7.0 + 5.0 * hash(seed, li * 31 + word as i32, 5);
                    if inw < wlen {
                        // a stroke: the loops of a running hand round the line's middle
                        let ph = x * 2.3 + 3.0 * hash(seed, li, word as i32);
                        let mid = 4.6 + 1.4 * (ph * 0.7).sin() * (0.5 + hash(seed, li * 7 + word as i32, 6));
                        let yy = mid + 1.6 * ph.sin();
                        let d = (in_line - yy).abs();
                        if d < 0.35 {
                            let ink = 0.55 + 0.25 * vn(seed ^ 9, x * 0.6, y * 0.6);
                            c = mix(c, [0.30, 0.22, 0.16], ink * (1.0 - d / 0.35));
                        }
                    }
                }
            }
            // the date at its head on the right, foxing, dust
            let fox = (vn(seed ^ 3, u * 24.0, v * 36.0) - 0.8).max(0.0) * 2.5;
            c = mix(c, [0.62, 0.50, 0.34], fox);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Brass gone dull (t09's: grey on the brass's colour, how bright): brown
/// tarnish in patches, darker spots
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xb7a;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix([0.42, 0.38, 0.33], [0.85, 0.82, 0.76], sstep(0.4, 0.75, n));
            let spot = vn(seed ^ 3, u * 40.0, v * 40.0);
            if spot > 0.72 {
                c = mix(c, [0.26, 0.24, 0.21], (spot - 0.72) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Pressed glass as the windows' and t09's (`Patch::Glass`): the room seen
/// in it by Fresnel, greyed through its thick walls; dust on it
fn ink_glass_mat() -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 200, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "inkwell_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.5;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust all over it (no map: all of it)
        f[8] = 0.30;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // the walls' body: grey through 5 mm of glass
        f[12] = 0.18;
        f[13] = 0.10;
    }
    m
}

/// The painting kit (per pixel, pure: safe for `par_rows` on every core), a
/// salt from where a thing is, a part placed.
mod sd_kit {
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
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
