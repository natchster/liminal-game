//! How the set looks: its photographed surfaces under their baked light,
//! tone-mapped once at load; the lamps' faces; the highlights the lamps put
//! on the wet paint and the glazed tiles (live, they move with your eye);
//! the glass.
//!
//! A baked surface is drawn unlit (`MatKind::Basic`): the photograph (sRGB,
//! linear in the shader) times its tint times the corner's light. The
//! light is Cycles' and unbounded; `tone` brings each corner's light into
//! the frame's range with a shoulder, judged against the material's mean
//! albedo, so a white tile under a bar rolls off like film and a red floor
//! keeps its colour. The highlights are a second pass of the same mesh:
//! Phong with a black body (only its specular term), added over the first,
//! its gloss (paint satin, bare concrete matte, glaze bright, grout dull)
//! in the map's alpha. They are lit by point lights at the bake's own
//! sources, so the shine is where the light is.
use crate::data::{MatDef, MatKind};
use engine::dmath::DMath;
use engine::prelude::*;
use std::collections::BTreeMap;

/// The camera's exposure: Cycles' light to the frame (tuned to the video:
/// the lit tile reads about 0.68 of white).
pub const EXPOSURE: f64 = 1.4;
/// The camera's white balance on the bake's light (the video's LEDs read a
/// little green against its phone's daylight balance).
pub const BALANCE: [f64; 3] = [0.95, 1.0, 0.975];

/// Photographs laid at another scale than the set's uvs give (uv
/// metres / wanted metres): the stone's stones at about 25-35 cm.
const REPEAT: &[(&str, f64)] = &[("stone.jpg", 4.0 / 2.6)];

/// sRGB byte -> linear.
fn lin(b: u8) -> f64 {
    let s = b as f64 / 255.0;
    if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).dpowf(2.4) }
}

/// A photograph and what the look needs of it.
pub struct Photo {
    pub tex: Tex,
    /// its mean albedo (linear rgb)
    pub mean: [f64; 3],
}

#[derive(Default)]
pub struct Textures {
    photos: BTreeMap<String, Photo>,
    gloss: BTreeMap<String, Tex>,
}

/// An embedded photograph (`crate::assets`), decoded.
fn read_jpeg(name: &str) -> Canvas {
    let bytes = crate::assets::texture(name).unwrap_or_else(|| panic!("{name}: not embedded (crate::assets)"));
    Canvas::from_jpeg(bytes).unwrap_or_else(|| panic!("{name}: not a JPEG"))
}

fn surface(t: &Tex, srgb: bool) {
    let mut tt = t.t();
    tt.color_space = if srgb { ColorSpace::Srgb } else { ColorSpace::NoColorSpace };
    tt.wrap_s = Wrap::Repeat;
    tt.wrap_t = Wrap::Repeat;
    tt.flip_y = false;
    tt.anisotropy = 16;
}

impl Textures {
    pub fn new() -> Textures {
        Textures::default()
    }

    /// An albedo photograph (`assets/textures/NAME`), read once.
    pub fn photo(&mut self, name: &str) -> &Photo {
        if !self.photos.contains_key(name) {
            let c = read_jpeg(name);
            let px = c.pixels();
            let mut lut = [0.0f64; 256];
            for (i, l) in lut.iter_mut().enumerate() {
                *l = lin(i as u8);
            }
            let mut sum = [0.0f64; 3];
            let n = px.len() / 4;
            for p in px.as_chunks::<4>().0.iter().step_by(7) {
                for k in 0..3 {
                    sum[k] += lut[p[k] as usize];
                }
            }
            let m = n.div_ceil(7) as f64;
            let tex = c.to_tex();
            surface(&tex, true);
            if let Some(r) = REPEAT.iter().find(|(n, _)| *n == name) {
                tex.t().repeat = Vec2::new(r.1, r.1);
            }
            self.photos.insert(name.to_string(), Photo { tex, mean: [sum[0] / m, sum[1] / m, sum[2] / m] });
        }
        &self.photos[name]
    }

    /// A gloss mask (`assets/textures/NAME`, grey) as a texture whose alpha
    /// is the gloss (white otherwise).
    pub fn gloss(&mut self, name: &str) -> Tex {
        if !self.gloss.contains_key(name) {
            let c = read_jpeg(name);
            let mut px = c.pixels();
            for p in px.as_chunks_mut::<4>().0 {
                p[3] = p[0];
                p[0] = 255;
                p[1] = 255;
                p[2] = 255;
            }
            let t = Tex::data_rgba(px, c.width() as u32, c.height() as u32);
            surface(&t, false);
            self.gloss.insert(name.to_string(), t);
        }
        self.gloss[name].clone()
    }
}

/// The film's shoulder: linear below the knee, rolling off to 1.
pub fn shoulder(x: f64) -> f64 {
    const K: f64 = 0.62;
    if x <= K { x } else { K + (1.0 - K) * (1.0 - (-(x - K) / (1.0 - K)).dexp()) }
}

/// A corner's light, ready to multiply its albedo: exposed, and rolled off
/// so that albedo x light keeps under white (judged at the mean albedo).
pub fn tone(light: [f32; 3], mean: [f64; 3]) -> [f32; 3] {
    let a = (0.2126 * mean[0] + 0.7152 * mean[1] + 0.0722 * mean[2]).max(0.02);
    let l = [light[0] as f64 * EXPOSURE * BALANCE[0], light[1] as f64 * EXPOSURE * BALANCE[1], light[2] as f64 * EXPOSURE * BALANCE[2]];
    let y = 0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2];
    if y <= 1e-9 {
        return [0.0, 0.0, 0.0];
    }
    let out = shoulder(a * y) / a;
    let k = out / y;
    // the brightest light loses its colour a little, as film does
    let sat = 1.0 - 0.35 * ((a * y - 0.62) / 0.6).clamp(0.0, 1.0);
    let mut c = [0.0f32; 3];
    for i in 0..3 {
        let v = l[i] * k;
        let g = out;
        c[i] = (g + (v - g) * sat).max(0.0) as f32;
    }
    c
}

/// The materials of one copy of the set's look: shared by every placement
/// except where a copy needs its own (fading highlights, glass).
pub struct Looks {
    pub mats: BTreeMap<String, Mat>,
    /// for double-sided copies (the mirrored bore)
    pub mats_mirrored: BTreeMap<String, Mat>,
    pub means: BTreeMap<String, [f64; 3]>,
}

/// The colour a lamp's face is drawn in (linear rgb, under 1).
pub fn face(colour: [f64; 3]) -> Color {
    let m = colour[0].max(colour[1]).max(colour[2]).max(1e-6);
    // a little white in the colour: a lamp's face burns out
    let w = 0.35;
    Color::rgb_linear(w + (1.0 - w) * colour[0] / m, w + (1.0 - w) * colour[1] / m, w + (1.0 - w) * colour[2] / m)
}

/// Decal-like surfaces (lines, plates, faces on walls) win the depth test
/// against what they lie on, near and far.
const DECALS: &[&str] =
    &["white_paint", "hatch", "led", "rail_led", "red_led", "signal_green", "sign_exit", "sign_info", "sign_name", "sign_gate", "plaque"];

impl Looks {
    pub fn new(defs: &BTreeMap<String, MatDef>, tex: &mut Textures) -> Looks {
        let mut mats = BTreeMap::new();
        let mut mats_mirrored = BTreeMap::new();
        let mut means = BTreeMap::new();
        for (name, d) in defs {
            let (m, mean) = match d.kind {
                MatKind::Baked | MatKind::Sign => {
                    let m = basic_mat!({ vertexColors: true, color: Color::rgb_linear(d.tint[0], d.tint[1], d.tint[2]) });
                    let mut mean = d.tint;
                    if let Some(t) = &d.texture {
                        let p = tex.photo(t);
                        m.m().map = Some(p.tex.clone());
                        mean = [d.tint[0] * p.mean[0], d.tint[1] * p.mean[1], d.tint[2] * p.mean[2]];
                    }
                    if d.kind == MatKind::Sign {
                        // the picture is painted per sign (signs.rs); white till then
                        m.m().color = Color::rgb_linear(1.0, 1.0, 1.0);
                    }
                    (m, mean)
                }
                MatKind::Emissive => (basic_mat!({ color: face(d.tint) }), [1.0, 1.0, 1.0]),
                MatKind::Glass => {
                    // what a pane shows of the place it is in, by Fresnel (its
                    // corners' colours: the lit tunnel's grey inside, the night outside)
                    let m = basic_mat!({ color: Color::rgb_linear(0.40, 0.44, 0.43), vertexColors: true, transparent: true, opacity: 0.1, depthWrite: false, side: DoubleSide });
                    (m, [1.0, 1.0, 1.0])
                }
            };
            m.name(name);
            if DECALS.contains(&name.as_str()) {
                m.polygonOffset(true).polygonOffsetFactor(-1.0).polygonOffsetUnits(-4.0);
            }
            let mm = m.duplicate();
            mm.side(DoubleSide);
            mats.insert(name.clone(), m);
            mats_mirrored.insert(name.clone(), mm);
            means.insert(name.clone(), mean);
        }
        Looks { mats, mats_mirrored, means }
    }
}

/// A highlight pass's material: only the specular term of the lamps, added
/// over the surface, its gloss in the map's alpha. `strength` scales the
/// specular colour; `shine` is Phong's exponent.
pub fn highlight(gloss: &Tex, strength: f64, shine: f64) -> Mat {
    let m = phong!({
        color: Color::rgb_linear(0.0, 0.0, 0.0),
        specular: Color::rgb_linear(strength, strength, strength),
        shininess: shine,
        map: gloss,
        transparent: true,
        blending: AdditiveBlending,
        depthWrite: false,
        fog: false,
        polygonOffset: true,
        polygonOffsetFactor: -1.0,
        polygonOffsetUnits: -2.0
    });
    m.name("highlight");
    m
}

/// A soft round glow (a lamp's halo in the eye and the haze), white on clear.
pub fn glow_texture() -> Tex {
    let n = 128;
    let mut px = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f64 + 0.5) / n as f64 * 2.0 - 1.0;
            let dy = (y as f64 + 0.5) / n as f64 * 2.0 - 1.0;
            let r = (dx * dx + dy * dy).sqrt();
            // a tight core and a wide, faint skirt
            let a = (0.55 * (-(r / 0.16).dpowi(2)).dexp() + 0.45 * (-(r / 0.5).dpowi(2)).dexp()) * (1.0 - r).clamp(0.0, 1.0);
            let i = (y * n + x) * 4;
            px[i] = 255;
            px[i + 1] = 255;
            px[i + 2] = 255;
            px[i + 3] = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
    let t = Tex::data_rgba(px, n as u32, n as u32);
    {
        let mut tt = t.t();
        tt.flip_y = false;
        tt.color_space = ColorSpace::NoColorSpace;
    }
    t
}

/// A lattice hash in 0..1 (no stream, no state: a place's own number).
fn hash3(i: i64, j: i64, k: i64) -> f64 {
    let mut h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (k as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 32;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Value noise in 0..1 at p (smooth, a feature a unit across).
fn vnoise(p: [f64; 3]) -> f64 {
    let f = [p[0].floor(), p[1].floor(), p[2].floor()];
    let t = [p[0] - f[0], p[1] - f[1], p[2] - f[2]];
    let s = t.map(|x| x * x * (3.0 - 2.0 * x));
    let (i, j, k) = (f[0] as i64, f[1] as i64, f[2] as i64);
    let mut acc = 0.0;
    for c in 0..8 {
        let (a, b, e) = (c & 1, (c >> 1) & 1, (c >> 2) & 1);
        let w =
            (if a == 1 { s[0] } else { 1.0 - s[0] }) * (if b == 1 { s[1] } else { 1.0 - s[1] }) * (if e == 1 { s[2] } else { 1.0 - s[2] });
        acc += w * hash3(i + a as i64, j + b as i64, k + e as i64);
    }
    acc
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The dirt of years, as a factor on a corner's light (its albedo darkened
/// where it would be): the tiles grimed at their foot by mops and splashes
/// and streaked under the pipes and the trim; water stains and soot on the
/// lining; the floor's gutter by the wall; a little unevenness on paint,
/// stone and concrete. `p` is in the piece's frame (the engine's: y up, the
/// walkway's floor at 0.2). A function of the place only.
pub fn grime(material: &str, p: [f32; 3]) -> f64 {
    let (x, y, z) = (p[0] as f64, p[1] as f64, p[2] as f64);
    let slow = vnoise([x * 0.7 + 11.3, y * 0.7, z * 0.7]);
    let fine = vnoise([x * 3.1, y * 3.1 + 5.7, z * 3.1]);
    match material {
        "tile" | "tile_green" if x > 4.0 => {
            // the road's far wall: a band of soot from the exhausts at its foot
            let soot = smooth(1.3, 0.3, y);
            1.0 - 0.32 * soot - 0.08 * smooth(0.45, 0.85, fine)
        }
        "tile" | "tile_green" => {
            let h = y - 0.2;
            // a mop's tide line and the splashes over it
            let foot = 0.30 * smooth(0.42 + 0.12 * slow, 0.05, h);
            // streaks down from the trim and the pipes: they vary along the wall, not down it
            let streak = smooth(0.62, 0.86, vnoise([z * 2.3 + 3.1, 0.0, x * 0.1])) * smooth(0.6, 2.1, h);
            let mottle = 0.10 * smooth(0.45, 0.85, fine);
            1.0 - foot - 0.16 * streak - mottle
        }
        "vault" => {
            // water stains: finer than a module, so they do not march with it
            let stain = smooth(0.55, 0.9, vnoise([x * 1.3, y * 1.3, z * 1.3 + 2.0]));
            let soot = 0.10 * smooth(4.0, 7.0, y);
            1.0 - 0.12 * stain - soot - 0.06 * fine
        }
        "floor" => {
            // the gutter by the wall and the barrier's foot: grit and wet, nearly black
            let gutter = smooth(-1.55, -1.68, x);
            let foot = smooth(-0.34, -0.07, x);
            1.0 - 0.35 * gutter - 0.45 * foot - 0.10 * smooth(0.4, 0.8, slow)
        }
        "panel" | "white_paint" | "steel" | "alu" => 1.0 - 0.10 * smooth(0.4, 0.85, slow) - 0.05 * fine,
        "stone" | "concrete" | "plaza" => {
            let wet = 0.18 * smooth(0.9, 0.2, y);
            1.0 - 0.18 * smooth(0.35, 0.8, slow) - wet - 0.06 * fine
        }
        "road" => {
            // the wheel paths of both lanes of each bore, polished dark
            let tracks: f64 =
                [1.46, 3.16, 5.65, 7.35, 13.45, 15.15, 17.64, 19.34].iter().map(|c| (-((x - c) / 0.32).dpowi(2)).dexp()).sum();
            1.0 - 0.15 * smooth(0.3, 0.8, slow) - 0.06 * fine - 0.2 * tracks.min(1.0)
        }
        _ => 1.0,
    }
}
