//! `visuals::pictures` — what the rooms' framed photographs, paintings,
//! maps, plates and prints show, and how a print, a canvas, a printed sheet
//! and a frame are painted (docs/PROPS.md 1, "its picture a picture").
//!
//! The pictures are real: old photographs of anonymous sitters (before
//! 1930), paintings, maps and botanical and anatomical plates, released as
//! CC0 by the Rijksmuseum and the Cleveland Museum of Art, laid each in
//! its cell of one atlas (`crates/game/assets/textures/pictures.jpg`, built
//! by `assets/src/textures/build.py` from the files `sources.json` lists
//! under `pictures`, with their SHA-1s; the licence and makers in
//! `MANIFEST.json`). The atlas is decoded once, when a floor first shows a
//! picture, and is never graded: each print is aged as it is painted
//! ([`print_tex`]): a photograph's blacks lifted, its paper yellowed from
//! the edges, its dark tones silvered, foxed, now and then creased; an oil
//! darkened under yellow varnish, cracked into islands, grimed, a flake of
//! paint lost. Every painter computes its pixels itself on every core
//! (`par_rows`) and draws its strokes after, from a stream of its own mixed
//! with `vary` (where the thing hangs), so two prints of one picture on two
//! floors differ, and no level draw is taken.
//!
//! The frames' finishes are painted as their fronts are mapped (the box
//! faces of a Blender part: the frame's bounds): stained wood, ebonised
//! wood, old gilding and paint, each from where on its moulding's profile a
//! pixel lies ([`moulding_tex`]); the mat a print is cut into ([`mat_tex`]);
//! the glass over it ([`glass_mat`]). A sheet whose design the game sets
//! (a poster's type) is painted and then aged as a print ([`sheet_tex`]); a
//! picture on its wire hangs from a brass hook ([`hang`]).
use crate::base::rng::SplitMix64;
use crate::placers::kit::{canvas_tex_opt, par_rows};
use crate::prelude::*;
use engine::material::Patch;

static ATLAS: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/textures/pictures.jpg"));

/// The pictures and their cells in the atlas, px (x, y, w, h): as
/// `sources.json` lays them (`pictures.files.*.cell`)
pub const CELLS: [(&str, [u32; 4]); 37] = [
    ("baby_basket", [0, 0, 320, 400]),
    ("baby_chair", [320, 0, 320, 400]),
    ("graduate", [640, 0, 320, 400]),
    ("mother_children", [960, 0, 320, 400]),
    ("couple_reading", [1280, 0, 320, 400]),
    ("family_parlour", [0, 400, 400, 320]),
    ("family_room", [400, 400, 400, 320]),
    ("family_door", [800, 400, 400, 320]),
    ("land_pond", [1200, 400, 400, 320]),
    ("garden_family", [0, 720, 320, 400]),
    ("oil_man", [320, 720, 320, 400]),
    ("oil_woman2", [640, 720, 320, 400]),
    ("mini_a", [960, 720, 160, 200]),
    ("mini_b", [1120, 720, 160, 200]),
    ("mini_c", [960, 920, 160, 200]),
    ("mini_d", [1120, 920, 160, 200]),
    ("oil_woman", [1280, 720, 320, 400]),
    ("land_road", [0, 1120, 400, 320]),
    ("still_peaches", [400, 1120, 400, 320]),
    ("still_dark", [800, 1120, 400, 320]),
    // the maps, plates, prints and the triptych (batch t02)
    ("map_world", [0, 1440, 480, 400]),
    ("map_dvina", [480, 1440, 480, 400]),
    ("map_sky", [960, 1440, 480, 400]),
    ("trip_l", [1440, 1440, 156, 400]),
    ("map_mexico", [0, 1840, 480, 400]),
    ("trip_c", [480, 1840, 308, 400]),
    ("trip_r", [788, 1840, 156, 400]),
    ("anat_front", [944, 1840, 287, 400]),
    ("anat_back", [1231, 1840, 287, 400]),
    ("bot_rose", [0, 2240, 285, 400]),
    ("bot_pulmonaria", [285, 2240, 285, 400]),
    ("bot_magnolia", [570, 2240, 285, 400]),
    ("bot_poppy", [855, 2240, 285, 400]),
    ("map_cook", [1140, 2240, 456, 296]),
    ("map_paris", [0, 2640, 398, 300]),
    ("map_trier", [398, 2640, 398, 300]),
    ("map_antwerp", [796, 2640, 456, 296]),
];

/// The decoded atlas: straight RGBA, rows top-down
struct Atlas {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

fn atlas() -> Rc<Atlas> {
    thread_local! { static A: std::cell::OnceCell<Rc<Atlas>> = const { std::cell::OnceCell::new() }; }
    A.with(|a| {
        a.get_or_init(|| {
            let c = Canvas::from_jpeg(ATLAS).expect("assets/textures/pictures.jpg does not decode");
            Rc::new(Atlas { w: c.width() as usize, h: c.height() as usize, px: c.pixels() })
        })
        .clone()
    })
}

/// A picture's cell, px
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// The cell of picture `name` (one of [`CELLS`])
pub fn cell(name: &str) -> Cell {
    let [x, y, w, h] = CELLS.iter().find(|c| c.0 == name).unwrap_or_else(|| panic!("no picture {name}")).1;
    Cell { x: x as f64, y: y as f64, w: w as f64, h: h as f64 }
}

impl Atlas {
    /// The picture at (u, v) of its cell (0..1, v down), bilinear, sRGB 0..1
    fn sample(&self, c: &Cell, u: f64, v: f64) -> [f64; 3] {
        let fx = (c.x + u.clamp(0.0, 1.0) * (c.w - 1.0)).clamp(0.0, (self.w - 1) as f64);
        let fy = (c.y + v.clamp(0.0, 1.0) * (c.h - 1.0)).clamp(0.0, (self.h - 1) as f64);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let at = |x: usize, y: usize, k: usize| f64::from(self.px[(y * self.w + x) * 4 + k]) / 255.0;
        let mut o = [0.0; 3];
        for (k, ok) in o.iter_mut().enumerate() {
            let a = at(x0, y0, k) + (at(x1, y0, k) - at(x0, y0, k)) * tx;
            let b = at(x0, y1, k) + (at(x1, y1, k) - at(x0, y1, k)) * tx;
            *ok = a + (b - a) * ty;
        }
        o
    }
}

/// Something of where a thing hangs, to vary what its painters paint (a
/// painter's own stream starts the same on every floor)
pub fn vary_of(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

// ------------------------------------------------------------------ noise ---
fn hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    f64::from(h) / f64::from(u32::MAX)
}

fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (hash(seed, xi, yi), hash(seed, xi + 1, yi));
    let (c, d) = (hash(seed, xi, yi + 1), hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

fn fbm(seed: u32, x: f64, y: f64) -> f64 {
    let (mut s, mut a, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for o in 0..4 {
        s += a * vnoise(seed.wrapping_add(o * 977), x * f, y * f);
        n += a;
        a *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn put(out: &mut [u8], i: usize, c: [f64; 3]) {
    out[i * 4] = (c[0] * 255.0).round().clamp(0.0, 255.0) as u8;
    out[i * 4 + 1] = (c[1] * 255.0).round().clamp(0.0, 255.0) as u8;
    out[i * 4 + 2] = (c[2] * 255.0).round().clamp(0.0, 255.0) as u8;
    out[i * 4 + 3] = 255;
}

/// Distance to the nearest edge of a Voronoi cell of `size` and that cell's
/// own value (0..1): a craquelure's islands
fn islands(seed: u32, x: f64, y: f64, size: f64) -> (f64, f64) {
    let (gx, gy) = (x / size, y / size);
    let (cx, cy) = (gx.floor() as i32, gy.floor() as i32);
    let (mut d1, mut d2, mut id) = (9.0f64, 9.0f64, 0.0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (ix, iy) = (cx + dx, cy + dy);
            // islands longer one way than the other (the canvas's weave)
            let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
            let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
            let d = ((gx - px) * (gx - px) + (gy - py) * (gy - py) * 1.3).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = hash(seed ^ 0x99, ix, iy);
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    ((d2 - d1) * 0.5 * size, id)
}

// ----------------------------------------------------------------- prints ---
/// How a picture has aged where it hangs
#[derive(Clone, Copy, Debug)]
pub struct Age {
    /// the blacks lifted toward the paper (a photograph's fading), 0..0.6
    pub fade: f64,
    /// the paper's (or varnish's) yellow: a multiply, strongest in the light tones (sRGB)
    pub tone: [f64; 3],
    /// how much more the edges have yellowed and darkened, 0..1
    pub edge: f64,
    /// silvering-out in the dark tones near the edges (a silver gelatin print), 0..1
    pub silver: f64,
    /// foxing: brown spots
    pub spots: usize,
    /// a crease across a corner, its emulsion cracked along it
    pub crease: bool,
    /// a damp tide line up from the foot, 0..1
    pub stain: f64,
    /// an oil's craquelure: its islands' size (m; 0 none) and how dark its cracks
    pub crack: (f64, f64),
    /// grime over it all (a darkening), 0..1
    pub grime: f64,
    /// flakes of paint lost to the ground
    pub losses: usize,
    /// a canvas's weave under the paint, 0..1
    pub weave: f64,
    /// a white border round a photograph (m)
    pub border: f64,
}

impl Age {
    /// A photograph that has hung for decades in a room without sun:
    /// faded, yellowed, foxed; `vary` sets how much
    pub fn photo(vary: u32) -> Age {
        let mut r = SplitMix64::new(vary, 0xf070);
        let y = r.range(0.0, 1.0);
        Age {
            fade: r.range(0.05, 0.2),
            tone: [1.0, 0.93 - 0.04 * y, 0.78 - 0.08 * y],
            edge: r.range(0.3, 0.8),
            silver: r.range(0.0, 0.6),
            spots: (r.range(4.0, 26.0)) as usize,
            crease: r.next() < 0.3,
            stain: if r.next() < 0.25 { r.range(0.3, 0.8) } else { 0.0 },
            crack: (0.0, 0.0),
            grime: r.range(0.0, 0.12),
            losses: 0,
            weave: 0.0,
            border: 0.0,
        }
    }
    /// An engraving, a map or a plate on rag paper, decades on a wall:
    /// browned all over (the museum's scan is of a sheet kept in the dark;
    /// one on a wall goes the tan of old paper), yellowed most at its edges,
    /// foxed, now and then a damp tide line or a crease; `vary` sets how much
    pub fn engraving(vary: u32) -> Age {
        let mut r = SplitMix64::new(vary, 0xe9a7);
        let y = r.range(0.0, 1.0);
        Age {
            fade: r.range(0.0, 0.08),
            tone: [0.93, 0.87 - 0.05 * y, 0.71 - 0.08 * y],
            edge: r.range(0.3, 0.7),
            silver: 0.0,
            spots: (r.range(10.0, 40.0)) as usize,
            crease: r.next() < 0.15,
            stain: if r.next() < 0.3 { r.range(0.3, 0.7) } else { 0.0 },
            crack: (0.0, 0.0),
            grime: r.range(0.02, 0.12),
            losses: 0,
            weave: 0.0,
            border: 0.0,
        }
    }
    /// An oil painting of two or three centuries: its varnish gone amber,
    /// cracked into islands, grimed
    pub fn oil(vary: u32) -> Age {
        let mut r = SplitMix64::new(vary, 0x011);
        Age {
            fade: 0.0,
            tone: [1.0, 0.86 - r.range(0.0, 0.06), 0.62 - r.range(0.0, 0.10)],
            edge: r.range(0.2, 0.5),
            silver: 0.0,
            spots: 0,
            crease: false,
            stain: 0.0,
            crack: (r.range(0.004, 0.007), r.range(0.22, 0.38)),
            grime: r.range(0.08, 0.22),
            losses: (r.range(0.0, 5.0)) as usize,
            weave: 0.6,
            border: 0.0,
        }
    }
}

/// The print of picture `name` on something `size` m (w, h), at `ppm` px a
/// metre: the picture laid on it as a framer would (its middle, the rest cut
/// off), aged by `age`
pub fn print_tex(name: &str, size: (f64, f64), ppm: f64, age: Age, vary: u32) -> Tex {
    let c = cell(name);
    let (pw, ph) = ((size.0 * ppm).round().max(8.0), (size.1 * ppm).round().max(8.0));
    canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let at = atlas();
        let atr: &Atlas = &at;
        // the picture's middle, cut to this print's shape
        let (aspect, ca) = (size.0 / size.1, c.w / c.h);
        let (ku, kv) = if aspect > ca { (1.0, ca / aspect) } else { (aspect / ca, 1.0) };
        age_sheet(ctx, w, h, size, ppm, age, vary, &|u, v, _, _| atr.sample(&c, 0.5 + (u - 0.5) * ku, 0.5 + (v - 0.5) * kv));
    })
}

/// A sheet `size` m at `ppm` px a metre whose design the game paints
/// (`draw`, in px: a poster's type and inks, on its own stream), then aged
/// by `age` as [`print_tex`] ages a print
pub fn sheet_tex(size: (f64, f64), ppm: f64, age: Age, vary: u32, draw: impl FnOnce(&mut Canvas, f64, f64)) -> Tex {
    let (pw, ph) = ((size.0 * ppm).round().max(8.0), (size.1 * ppm).round().max(8.0));
    canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        draw(ctx, w, h);
        let src = ctx.get_image_data(0, 0, w, h);
        let n = w as usize;
        let px = |i: usize, j: usize| {
            let k = (j * n + i) * 4;
            [f64::from(src.data[k]) / 255.0, f64::from(src.data[k + 1]) / 255.0, f64::from(src.data[k + 2]) / 255.0]
        };
        age_sheet(ctx, w, h, size, ppm, age, vary, &|_, _, i, j| px(i, j));
    })
}

/// The ageing of a print or a sheet (w x h px, `size` m): every pixel from
/// `src` (u, v of the sheet, 0..1 v down, or its px i, j), faded, yellowed
/// from its edges, silvered, cracked, foxed, stained and grimed by `age`,
/// computed on every core; then a crease and scratches drawn
#[allow(clippy::too_many_arguments)]
fn age_sheet(
    ctx: &mut Canvas,
    w: f64,
    h: f64,
    size: (f64, f64),
    ppm: f64,
    age: Age,
    vary: u32,
    src: &(dyn Fn(f64, f64, usize, usize) -> [f64; 3] + Sync),
) {
    let mut r = SplitMix64::new(vary, 0x9a1c);
    let seed = (r.next() * 4.0e9) as u32;
    // foxing: soft brown spots, a few with a darker heart
    let spots: Vec<(f64, f64, f64, f64)> =
        (0..age.spots).map(|_| (r.next() * w, r.next() * h, r.range(0.6, 3.2) * ppm / 1000.0, r.range(0.15, 0.55))).collect();
    // the losses: flakes off an oil, the ground showing
    let losses: Vec<(f64, f64, f64)> =
        (0..age.losses).map(|_| (r.next() * w, r.range(0.4, 1.0) * h, r.range(0.4, 1.4) * ppm / 1000.0)).collect();
    let tide = r.range(0.06, 0.16);
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            // m from the print's nearest edge
            let e = (x.min(w - x).min(y).min(h - y)) / ppm;
            let (mut u, mut v) = (x / w, y / h);
            let bd = age.border;
            let in_border = bd > 0.0 && e < bd;
            if bd > 0.0 {
                u = (x / ppm - bd) / (size.0 - 2.0 * bd);
                v = (y / ppm - bd) / (size.1 - 2.0 * bd);
            }
            let s = src(u, v, i, j);
            let mut o = if in_border { [0.93, 0.92, 0.88] } else { s };
            // faded: the blacks lifted toward the paper
            o = o.map(|k| 1.0 - (1.0 - k) * (1.0 - age.fade));
            // the paper (or varnish) yellowed, most in the light tones and
            // more toward the edges
            let edge = age.edge * (1.0 - smooth(0.0, 0.03, e));
            let lum = 0.3 * o[0] + 0.59 * o[1] + 0.11 * o[2];
            for k in 0..3 {
                let t = age.tone[k];
                let tt = 1.0 - (1.0 - t) * (0.55 + 0.45 * lum) * (1.0 + edge);
                o[k] *= tt.max(0.0);
            }
            o = o.map(|k| k * (1.0 - 0.25 * edge));
            // silvered out: the dark tones near the edges gone a grey metallic sheen
            if age.silver > 0.0 {
                let sv = age.silver * (1.0 - smooth(0.0, 0.025, e)) * (1.0 - lum).powi(2) * (0.6 + 0.4 * fbm(seed, x * 0.08, y * 0.08));
                o = mix3(o, [0.48, 0.5, 0.52], sv.min(0.7));
            }
            // an oil's cracks: dark lines between islands, each island
            // cupped a little (its own shade)
            if age.crack.0 > 0.0 {
                let (d, id) = islands(seed ^ 0x3c, x / ppm, y / ppm, age.crack.0);
                let line = 1.0 - smooth(0.0, 0.7 / ppm, d);
                o = o.map(|k| k * (1.0 - age.crack.1 * line) * (0.985 + 0.03 * id));
            }
            // the canvas's weave under the paint
            if age.weave > 0.0 {
                let wv = (x * 2.4).sin() * (y * 2.4).sin();
                o = o.map(|k| k * (1.0 + 0.03 * age.weave * wv));
            }
            // foxing
            for &(sx, sy, rr, k) in &spots {
                let d2 = ((x - sx) * (x - sx) + (y - sy) * (y - sy)) / (rr * rr);
                if d2 < 9.0 {
                    let f = k * (-d2).exp();
                    o = [o[0] * (1.0 - 0.25 * f), o[1] * (1.0 - 0.45 * f), o[2] * (1.0 - 0.65 * f)];
                }
            }
            // a damp tide line up from the foot: browned below, a darker rim
            if age.stain > 0.0 {
                let line = h * (1.0 - tide) + (fbm(seed ^ 7, x * 0.03, 0.5) - 0.5) * h * 0.06;
                if y > line {
                    o = [o[0] * 0.95, o[1] * 0.9, o[2] * 0.8];
                }
                let rim = (-((y - line) / 1.5).powi(2)).exp() * age.stain;
                o = [o[0] * (1.0 - 0.18 * rim), o[1] * (1.0 - 0.26 * rim), o[2] * (1.0 - 0.36 * rim)];
            }
            // losses: the ground (a warm off-white) where paint has flaked
            for &(lx, ly, rr) in &losses {
                let d = ((x - lx) * (x - lx) + (y - ly) * (y - ly)).sqrt();
                let edge_n = fbm(seed ^ 11, x * 0.4, y * 0.4);
                if d < rr * (0.6 + 0.8 * edge_n) {
                    o = [0.6, 0.54, 0.43];
                }
            }
            // grime: a darkening, blotchy, heavier at the foot
            let g = age.grime * (0.6 + 0.8 * fbm(seed ^ 13, x * 0.02, y * 0.02)) * (0.7 + 0.6 * y / h);
            o = o.map(|k| k * (1.0 - g.min(0.6)));
            // the paper's tooth
            let tooth = 1.0 + 0.035 * (hash(seed ^ 17, i as i32, j as i32) - 0.5);
            put(out, i, o.map(|k| k * tooth));
        }
    });
    ctx.put_image_data(&img, 0, 0);
    if age.crease {
        // a crease across a corner: a pale ridge with its shadow, the
        // emulsion cracked off along it
        let (fx, fy) = if r.next() < 0.5 { (0.0, h) } else { (w, h) };
        let k = r.range(0.18, 0.32);
        let (ax, ay) = (fx + if fx > 0.0 { -w * k } else { w * k }, fy);
        let (bx, by) = (fx, fy - h * k * 1.1);
        ctx.set_line_width(1.0);
        ctx.set_stroke_style("rgba(250,244,226,0.55)");
        ctx.begin_path();
        ctx.move_to(ax, ay);
        ctx.quadratic_curve_to((ax + bx) / 2.0 + 2.0, (ay + by) / 2.0 + 2.0, bx, by);
        ctx.stroke();
        ctx.set_stroke_style("rgba(40,30,20,0.35)");
        ctx.begin_path();
        ctx.move_to(ax + 1.0, ay + 0.5);
        ctx.quadratic_curve_to((ax + bx) / 2.0 + 3.0, (ay + by) / 2.0 + 3.0, bx + 1.0, by + 0.5);
        ctx.stroke();
        for _ in 0..12 {
            let t = r.next();
            let (px, py) = (ax + (bx - ax) * t + r.range(-1.5, 1.5), ay + (by - ay) * t + r.range(-1.5, 1.5));
            ctx.set_fill_style("rgba(238,232,214,0.8)");
            ctx.fill_rect(px, py, r.range(0.6, 1.8), r.range(0.6, 1.4));
        }
    }
    // a few scratches in the emulsion or the varnish, catching the light
    for _ in 0..(r.range(2.0, 8.0) as usize) {
        let (x, y) = (r.next() * w, r.next() * h);
        let a = r.next() * PI;
        let l = r.range(4.0, 22.0);
        ctx.set_stroke_style(format!("rgba(236,228,206,{:.2})", r.range(0.12, 0.3)));
        ctx.set_line_width(0.5);
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
}

/// A picture's hanging: a brass picture hook on the wall at its wire's apex
/// (`apex` above the picture's middle at `mount_y`, the wall `wall` behind
/// the spot), the frame swinging about it by `tilt` and leaning out `lean`
/// at its top from its foot (`h` its height); returns the body its parts go
/// in (the frame of `t01.py` and `t02.py`: z = 0 the wall)
pub fn hang(o: &Obj, mount_y: f64, wall: f64, apex: f64, h: f64, lean: f64, tilt: f64) -> Obj {
    let nail = group();
    nail.position().set(0, mount_y + apex, -wall);
    nail.rotation().z = tilt;
    o.add(&nail);
    if let Some(geo) = crate::assets::props::prop_part_as_modelled("Real__oldPortraitHook") {
        nail.add(&mesh(geo, phong!({ color: 0x6a5630, specular: 0x5a4a2a, shininess: 36 })));
    }
    let foot = group();
    foot.position().y = -apex - h / 2.0;
    foot.rotation().x = (lean / h).atan();
    nail.add(&foot);
    let body = group();
    body.position().y = h / 2.0;
    foot.add(&body);
    body
}

/// The parts of a picture (`Real__…` names, each with its material), as
/// modelled, into `body`; a glass drawn after what is behind it
pub fn add_parts(body: &Obj, parts: &[(&str, Mat)]) {
    for (name, mat) in parts {
        if let Some(geo) = crate::assets::props::prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            if name.ends_with("Glass") {
                m.set_render_order(2);
            }
            body.add(&m);
        }
    }
}

// -------------------------------------------------------------------- mats ---
/// A mat's window (m, about the mat's middle; y up)
#[derive(Clone, Copy, Debug)]
pub enum Window {
    /// an oval: semi-axes, raised by
    Oval(f64, f64, f64),
    /// a rectangle: w, h, raised by
    Rect(f64, f64, f64),
}

/// A mat (card `size` m, its window cut with a 2 mm bevel) as its front is
/// mapped: the card's surface `card` (sRGB), gone yellow toward its window
/// and spotted; the bevel's core gone brown (mat burn); a pencilled line
/// under the window when `note` is given
pub fn mat_tex(size: (f64, f64), win: Window, card: [u8; 3], vary: u32, note: Option<String>) -> Tex {
    let ppm = 1400.0;
    let (pw, ph) = ((size.0 * ppm).round(), (size.1 * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x3a7);
        let seed = (r.next() * 4.0e9) as u32;
        let base = card.map(|k| f64::from(k) / 255.0);
        let spots: Vec<(f64, f64, f64)> =
            (0..(r.range(3.0, 14.0) as usize)).map(|_| (r.next() * w, r.next() * h, r.range(0.5, 2.0))).collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                // m from the mat's middle, y up
                let x = (i as f64 + 0.5) / ppm - size.0 / 2.0;
                let y = size.1 / 2.0 - (j as f64 + 0.5) / ppm;
                // how far outside the window's back edge (m)
                let d = match win {
                    Window::Oval(a, b, dy) => {
                        let (u, v) = (x / a, (y - dy) / b);
                        let rho = (u * u + v * v).sqrt();
                        (rho - 1.0) * a.min(b)
                    }
                    Window::Rect(ww, hh, dy) => ((x.abs() - ww / 2.0).max((y - dy).abs() - hh / 2.0)).max(-1.0),
                };
                let mut o = base;
                // the card yellowed toward its window (light came through it)
                let near = 1.0 - smooth(0.0, 0.012, d);
                o = [o[0] * (1.0 - 0.02 * near), o[1] * (1.0 - 0.06 * near), o[2] * (1.0 - 0.14 * near)];
                // the bevel: the card's core, gone brown with acid
                if d < 0.0024 {
                    let k = 0.75 + 0.25 * fbm(seed, x * 900.0, y * 900.0);
                    o = [0.72 * k, 0.6 * k, 0.42 * k];
                }
                let blot = fbm(seed ^ 3, x * 60.0, y * 60.0);
                o = o.map(|k| k * (0.94 + 0.08 * blot));
                for &(sx, sy, rr) in &spots {
                    let px = (i as f64 - sx, j as f64 - sy);
                    let d2 = (px.0 * px.0 + px.1 * px.1) / (rr * rr);
                    if d2 < 9.0 {
                        let f = 0.4 * (-d2).exp();
                        o = [o[0] * (1.0 - 0.2 * f), o[1] * (1.0 - 0.4 * f), o[2] * (1.0 - 0.6 * f)];
                    }
                }
                put(out, i, o);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        if let Some(t) = &note {
            // pencil, in an old hand, under the window
            let (wy, below) = match win {
                Window::Oval(_, b, dy) => (dy, b),
                Window::Rect(_, hh, dy) => (dy, hh / 2.0),
            };
            let y = (size.1 / 2.0 - (wy - below - 0.010)) * ppm;
            ctx.set_font("italic 15px cursive");
            ctx.set_text_align("center");
            ctx.set_fill_style("rgba(84,80,78,0.55)");
            ctx.fill_text(t, w / 2.0 + r.range(-6.0, 6.0), y);
        }
    })
}

// ---------------------------------------------------------------- frames ---
/// A moulding's finish
#[derive(Clone, Copy, Debug)]
pub enum Finish {
    /// stained wood, its stain (sRGB)
    Wood([u8; 3]),
    /// ebonised: black over the wood, rubbed brown on its crests
    Ebony,
    /// gold leaf over red bole and white gesso, dulled and rubbed
    Gilt,
    /// paint (sRGB) over a white ground, chipped to it and to the wood
    Paint([u8; 3]),
}

/// A moulding's outline: a rectangle w x h, or an oval with those axes
#[derive(Clone, Copy, Debug)]
pub enum Outline {
    Rect,
    Oval,
}

/// Where on a moulding's face a point lies: (d, z) of its profile, d in from
/// the outer edge (m) to the sight, z its height
fn profile_z(prof: &[(f64, f64)], d: f64) -> (f64, f64) {
    for k in 1..prof.len() {
        let (d0, z0) = prof[k - 1];
        let (d1, z1) = prof[k];
        if d <= d1 {
            let t = if d1 > d0 { ((d - d0) / (d1 - d0)).clamp(0.0, 1.0) } else { 0.0 };
            return (z0 + (z1 - z0) * t, if d1 > d0 { (z1 - z0) / (d1 - d0) } else { 0.0 });
        }
    }
    (prof[prof.len() - 1].1, 0.0)
}

/// A moulding (`outer` m) as its front is mapped, `ppm` px a metre: its
/// finish laid by where each pixel lies on its profile (`prof`: the face
/// from the outer edge, d up, to the sight): grain along each rail and a
/// mitre at each corner, worn through on its crests and its outer edge,
/// grime in its hollows, dust along the top
pub fn moulding_tex(outer: (f64, f64), prof: &'static [(f64, f64)], outline: Outline, finish: Finish, ppm: f64, vary: u32) -> Tex {
    let (pw, ph) = ((outer.0 * ppm).round(), (outer.1 * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x6f1d);
        let seed = (r.next() * 4.0e9) as u32;
        let shade: Vec<f64> = (0..4).map(|_| r.range(0.9, 1.1)).collect();
        let zmax = prof.iter().map(|p| p.1).fold(0.0, f64::max);
        let zmin = prof.iter().map(|p| p.1).fold(1.0, f64::min);
        let mould = prof[prof.len() - 1].0;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = ((i as f64 + 0.5) / ppm, (j as f64 + 0.5) / ppm);
                // in from the outer edge (m), and the rail: 0 top 1 bottom 2 left 3 right
                let (d, rail, along) = match outline {
                    Outline::Rect => {
                        let ds = [y, outer.1 - y, x, outer.0 - x];
                        let mut k = 0;
                        for q in 1..4 {
                            if ds[q] < ds[k] {
                                k = q;
                            }
                        }
                        (ds[k], k, if k < 2 { x } else { y })
                    }
                    Outline::Oval => {
                        let (a, b) = (outer.0 / 2.0, outer.1 / 2.0);
                        let (px, py) = (x - a, b - y);
                        let ang = py.atan2(px);
                        let rl = 1.0 / ((ang.cos() / a).powi(2) + (ang.sin() / b).powi(2)).sqrt();
                        let rho = (px * px + py * py).sqrt();
                        (rl - rho, if py > 0.0 { 0 } else { 1 }, ang * a.max(b))
                    }
                };
                // inside the sight (seen only by the moulding's inner and outer
                // sides, which are mapped across it): the finish of its sight edge
                let d = d.min(mould);
                let (z, _) = profile_z(prof, d);
                let hn = ((z - zmin) / (zmax - zmin).max(1e-6)).clamp(0.0, 1.0);
                // how exposed to a hand or a duster: the crest and the outer arris
                let crest = smooth(0.82, 1.0, hn) + (1.0 - smooth(0.0, 0.0012, d));
                // a hollow: below the line between its neighbours
                let (zl, _) = profile_z(prof, (d - 0.002).max(0.0));
                let (zr, _) = profile_z(prof, (d + 0.002).min(mould));
                let hollow = smooth(0.0, 0.0008, (zl + zr) / 2.0 - z);
                let wobble = fbm(seed ^ 5, along * 40.0, d * 300.0);
                let wear = crest * smooth(0.35, 0.7, wobble);
                let mut o = match finish {
                    Finish::Wood(c) => {
                        let base = c.map(|k| f64::from(k) / 255.0 * shade[rail]);
                        // the grain: long streaks along the rail, the figure
                        // darker among them, pores
                        let g = fbm(seed ^ (rail as u32 * 31), along * 18.0, d * 900.0 + rail as f64 * 7.0);
                        let fig = ((d * 2600.0 + g * 9.0 + along * 3.0).sin() * 0.5 + 0.5).powf(6.0);
                        let pore = if hash(seed ^ 9, (along * 1400.0) as i32, (d * 9000.0) as i32) > 0.985 { 0.7 } else { 1.0 };
                        let k = (0.82 + 0.3 * g) * (1.0 - 0.25 * fig) * pore;
                        let mut o = base.map(|v| v * k);
                        // the stain rubbed thin: the wood's own paler colour
                        o = mix3(o, [0.5, 0.38, 0.26], 0.4 * wear);
                        o
                    }
                    Finish::Ebony => {
                        let g = fbm(seed ^ 21, along * 25.0, d * 1200.0);
                        let mut o = [0.075 * (0.9 + 0.2 * g), 0.066 * (0.9 + 0.2 * g), 0.058 * (0.9 + 0.2 * g)];
                        o = mix3(o, [0.3, 0.2, 0.13], 0.8 * wear);
                        o
                    }
                    Finish::Gilt => {
                        // the leaf: laid in squares, each a shade of its own
                        let sq = hash(seed ^ 33, (along / 0.08).floor() as i32, rail as i32);
                        let mut o = [0.6 + 0.06 * sq, 0.47 + 0.05 * sq, 0.25 + 0.03 * sq];
                        // dulled with age toward brown
                        o = mix3(o, [0.4, 0.32, 0.19], 0.45 + 0.25 * fbm(seed ^ 35, along * 12.0, d * 400.0));
                        // burnished brighter on the crests, rubbed through to the
                        // red bole, and on the arris to the white gesso
                        o = mix3(o, [0.78, 0.64, 0.36], 0.4 * smooth(0.8, 1.0, hn));
                        o = mix3(o, [0.46, 0.22, 0.13], 0.8 * wear);
                        let chip = (1.0 - smooth(0.0, 0.0018, d)) * smooth(0.6, 0.75, fbm(seed ^ 37, along * 90.0, d * 900.0));
                        mix3(o, [0.78, 0.74, 0.64], chip)
                    }
                    Finish::Paint(c) => {
                        let base = c.map(|k| f64::from(k) / 255.0);
                        // brushed along the rail, yellowed
                        let b = fbm(seed ^ 41, along * 6.0, d * 1800.0);
                        let mut o = [base[0] * (0.93 + 0.1 * b), base[1] * (0.92 + 0.1 * b), base[2] * (0.86 + 0.1 * b)];
                        // chipped through to the white ground, and further to the wood
                        let chips = fbm(seed ^ 43, along * 120.0, d * 1200.0);
                        let ch = (crest * 0.8 + 0.15) * smooth(0.55, 0.62, chips);
                        o = mix3(o, [0.86, 0.84, 0.78], ch);
                        mix3(o, [0.55, 0.42, 0.28], ch * smooth(0.66, 0.72, chips))
                    }
                };
                // grime in the hollows, dust on what faces up (the top rail)
                o = o.map(|k| k * (1.0 - 0.35 * hollow));
                let up = match rail {
                    0 => 1.0,
                    2 | 3 => 0.3,
                    _ => 0.15,
                };
                let dust = (0.04 + 0.16 * up) * (0.6 + 0.8 * fbm(seed ^ 51, along * 30.0, d * 300.0));
                o = mix3(o, [0.55, 0.52, 0.47], dust.min(0.45));
                // the mitres: each joint opened a hair, dark
                if let Outline::Rect = outline {
                    let dd = [y, outer.1 - y, x, outer.0 - x];
                    let m = ((dd[0].min(dd[1])) - (dd[2].min(dd[3]))).abs();
                    if m < 0.6 / ppm && d < mould {
                        o = o.map(|k| k * 0.45);
                    }
                }
                put(out, i, o);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // knocks: a few dents and scuffs
        for _ in 0..(r.range(3.0, 10.0) as usize) {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(format!("rgba(30,24,18,{:.2})", r.range(0.15, 0.4)));
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.5, 1.6), 0, 7);
            ctx.fill();
        }
    })
}

// ----------------------------------------------------------------- glass ---
/// The glass over a print, as the windows' (`Patch::Glass`): the room seen
/// in it by Fresnel, its dust and the smears of hands (`dirt` of the map's
/// grey), a part of the windows' dirt of its own (`vary`)
pub fn glass_mat(vary: u32, dirt: f32) -> Mat {
    let mut r = SplitMix64::new(vary, 0x6a55);
    let map = crate::visuals::photo::glass_dirt_tex(r.next(), r.next(), 0.18 + 0.12 * r.next());
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 240, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "picture_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, nothing lit behind it
        f[0] = 0.16;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = dirt;
        f[9] = 0.21;
        f[10] = 0.19;
        f[11] = 0.16;
        f[12] = 0.0;
        // two faces of float glass: 4% each, square on
        f[13] = 0.07;
    }
    m
}

/// Cracks in a picture's glass (`size` m) from a knock at (u, v) (0..1, v
/// down): lines run out from it, a few rings round it; the cracked edges
/// catch the light. Transparent elsewhere
pub fn cracked_glass_tex(size: (f64, f64), at: (f64, f64), vary: u32) -> Tex {
    let ppm = 1600.0;
    let (pw, ph) = ((size.0 * ppm).round(), (size.1 * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0xc4ac);
        ctx.clear_rect(0, 0, w, h);
        let (cx, cy) = (at.0 * w, at.1 * h);
        let rays = 5 + (r.next() * 4.0) as usize;
        for k in 0..rays {
            let mut a = k as f64 / rays as f64 * PI * 2.0 + r.range(-0.3, 0.3);
            let (mut x, mut y) = (cx, cy);
            let len = r.range(0.4, 1.3) * w.max(h);
            let steps = 8;
            let mut pts = vec![(x, y)];
            for _ in 0..steps {
                a += r.range(-0.25, 0.25);
                x += a.cos() * len / steps as f64;
                y += a.sin() * len / steps as f64;
                pts.push((x, y));
            }
            for (style, wd, off) in [("rgba(20,20,20,0.35)", 1.4, 0.7), ("rgba(236,236,230,0.6)", 0.7, 0.0)] {
                ctx.set_stroke_style(style);
                ctx.set_line_width(wd);
                ctx.begin_path();
                ctx.move_to(pts[0].0 + off, pts[0].1 + off);
                for p in &pts[1..] {
                    ctx.line_to(p.0 + off, p.1 + off);
                }
                ctx.stroke();
            }
        }
        // rings between the rays near the knock
        for ring in 0..2 {
            let rr = (0.12 + 0.12 * ring as f64) * w.max(h);
            ctx.set_stroke_style("rgba(232,232,226,0.45)");
            ctx.set_line_width(0.6);
            for k in 0..rays {
                if r.next() < 0.35 {
                    continue;
                }
                let a0 = k as f64 / rays as f64 * PI * 2.0;
                let a1 = a0 + PI * 2.0 / rays as f64 * r.range(0.6, 1.0);
                ctx.begin_path();
                ctx.move_to(cx + a0.cos() * rr, cy + a0.sin() * rr);
                ctx.line_to(cx + a1.cos() * rr * r.range(0.9, 1.1), cy + a1.sin() * rr * r.range(0.9, 1.1));
                ctx.stroke();
            }
        }
        // the knock itself: a small crushed star, white
        ctx.set_fill_style("rgba(240,240,236,0.7)");
        ctx.begin_path();
        ctx.arc(cx, cy, 2.2, 0, 7);
        ctx.fill();
    })
}
