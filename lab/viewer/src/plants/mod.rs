//! Generated plants: the office's potted plants, alive, dying and dead.
//!
//! Each species is a pure function of `(seed, health)` (`docs/research/plant-modeling.md`):
//! health 1 is a plant someone waters, about 0.5 one left a season (brown
//! tips, yellowing, the first leaves down), 0 one left for years (collapsed,
//! dry, its leaves on the floor). What health changes is what a botanist
//! would look for: droop (turgor gone), folding and curling (drying),
//! colour from the tips and margins inward (necrosis after chlorosis), leaf
//! drop (the weeping fig's answer to any stress), the soil pale and
//! shrunk from the pot wall, a salt crust on terracotta.
//!
//! Built with `kit`: a seeded stream and `dmath`, so a seed is the same
//! plant on every Mac.
pub mod fern;
pub mod ficus;
pub mod kit;
pub mod snake;

use engine::dmath::DMath;
use engine::prelude::*;
use kit::*;

/// What a part is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    /// leaves: both faces, a waxy shine (`shine` 0..1)
    Leaf,
    /// stems, bark
    Wood,
    Pot,
    Soil,
}

pub struct Plant {
    pub name: String,
    pub parts: Vec<(Surface, MeshBuf)>,
}

impl Plant {
    pub fn triangles(&self) -> usize {
        self.parts.iter().map(|(_, m)| m.triangles()).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Snake,
    Fern,
    Ficus,
}

impl Species {
    pub const ALL: [Species; 3] = [Species::Snake, Species::Fern, Species::Ficus];
    pub fn name(self) -> &'static str {
        match self {
            Species::Snake => "snake plant",
            Species::Fern => "Boston fern",
            Species::Ficus => "weeping fig",
        }
    }
    pub fn grow(self, seed: u64, health: f64) -> Plant {
        let h = health.clamp(0.0, 1.0);
        match self {
            Species::Snake => snake::grow(seed, h),
            Species::Fern => fern::grow(seed, h),
            Species::Ficus => ficus::grow(seed, h),
        }
    }
}

/// A pot: its kind, its size, and where the soil is.
#[derive(Clone, Copy, Debug)]
pub struct Pot {
    pub terracotta: bool,
    pub r_top: f64,
    pub r_bottom: f64,
    pub height: f64,
}

impl Pot {
    pub fn soil_y(&self) -> f64 {
        self.height - 0.028
    }
    /// The inner radius at the soil.
    pub fn soil_r(&self) -> f64 {
        let t = self.soil_y() / self.height;
        (self.r_bottom + (self.r_top - self.r_bottom) * t) * 0.93
    }

    /// The pot and its soil: plastic off-white with grime low down, or
    /// terracotta with a salt crust where water left it; the soil dark and
    /// moist when tended, pale, cracked and shrunk from the wall when not.
    pub fn build(&self, seed: u64, health: f64) -> Vec<(Surface, MeshBuf)> {
        let mut r = Rng::new(seed ^ 0x0050_4f54);
        let (rt, rb, h) = (self.r_top, self.r_bottom, self.height);
        let wall = 0.008;
        let profile = [
            (rb * 0.9, 0.0),
            (rb, 0.012),
            (rt * 0.985, h * 0.9),
            (rt * 1.045, h * 0.905),
            (rt * 1.05, h),
            (rt - wall * 0.5, h + 0.002),
            (rt - wall, h * 0.97),
            (self.soil_r() / 0.93 - wall, self.soil_y() - 0.01),
        ];
        let terracotta = self.terracotta;
        let tone = r.range(-0.03, 0.03);
        let crust = (1.0 - health) * 0.8;
        let s1 = r.next_u64();
        let mut pot = MeshBuf::default();
        lathe(&mut pot, V(0.0, 0.0, 0.0), &profile, 40, &|t, a| {
            let y = t; // height share of the profile rows
            let n = noise1(s1, a * 23.0) * 0.5 + noise1(s1 ^ 7, a * 61.0) * 0.25;
            if terracotta {
                let base = [0.60 + tone, 0.35 + tone * 0.5, 0.23];
                // the salt crust: white bands near the rim, blotchy
                let band = smooth(0.45, 0.8, y) * crust * (0.6 + 0.4 * n);
                mix(mix(base, [0.52, 0.30, 0.20], smooth(0.0, 0.3, 0.3 - y) * 0.6), [0.80, 0.77, 0.70], band)
            } else {
                let base = [0.76 + tone, 0.74 + tone, 0.68 + tone];
                // grime low down and where hands lift it
                let grime = smooth(0.25, 0.0, y) * 0.5 + (n * 0.5 + 0.5) * 0.12 * (1.2 - health);
                mix(base, [0.40, 0.37, 0.31], grime)
            }
        });
        // the soil: a heap a little higher in the middle, pale and cracked when dry
        let sr = self.soil_r();
        let dry = 1.0 - health;
        let shrink = dry * 0.12;
        let s2 = r.next_u64();
        let mut soil = MeshBuf::default();
        disc(&mut soil, V(0.0, self.soil_y(), 0.0), sr, 8, 40, &|rr, a| {
            let bump = noise1(s2, a * 9.0 + rr * 13.0) * 0.004 + (1.0 - rr * rr) * 0.008;
            let crack = (noise1(s2 ^ 3, a * 31.0).abs() < 0.08 && rr > 0.2) as u8 as f64 * dry;
            let gap = smooth(1.0 - shrink, 1.0, rr) * (shrink > 0.01) as u8 as f64;
            let moist = [0.25, 0.19, 0.13];
            let pale = [0.44, 0.36, 0.27];
            let mut c = mix(moist, pale, dry * (0.75 + 0.25 * noise1(s2 ^ 9, a * 7.0 + rr * 5.0)));
            c = mix(c, [0.10, 0.08, 0.06], crack * 0.8 + gap);
            (bump - gap * 0.02, c)
        });
        vec![(Surface::Pot, pot), (Surface::Soil, soil)]
    }
}

/// A leaf lying where it fell: flat on `y`, curled as it dried.
#[expect(clippy::too_many_arguments, reason = "a leaf's shape, its colour and how dry it is, as each species gives them")]
pub fn fallen_leaf(r: &mut Rng, at: V, y: f64, length: f64, w: f64, width: fn(f64, f64) -> f64, color: [f64; 3], dry: f64) -> MeshBuf {
    let az = r.range(0.0, 2.0 * PI);
    let (s, c) = az.dsin_cos();
    let dir = V(c, r.range(-0.05, 0.12), s);
    let mut b = Blade::new(V(at.0, y + 0.004, at.2), dir, UP, length, w, width);
    b.segs_u = 5;
    b.segs_v = 1;
    // a dry leaf curls up from its margins and its tip lifts
    b.curl = w * (0.3 + 1.2 * dry) * r.range(0.6, 1.2);
    b.fold = r.range(0.0, 0.5) * dry;
    b.twist = r.normal() * 0.4;
    b.droop = -r.range(0.0, 1.5) * dry;
    let mut mb = MeshBuf::default();
    let tint = r.range(-0.05, 0.05);
    let edge = mix(color, [0.36, 0.27, 0.16], 0.4);
    b.build(&mut mb, &|_, v| mix([color[0] + tint, color[1] + tint * 0.7, color[2]], edge, smooth(0.6, 1.0, v.abs())));
    mb
}

/// A place on the floor or soil near the pot: within `r0..r1` of the middle,
/// more often on the side the plant leans to (`lean`, an angle).
pub fn scatter(r: &mut Rng, r0: f64, r1: f64, lean: f64) -> V {
    let a = lean + r.normal() * 1.1;
    let d = r0 + (r1 - r0) * r.f().sqrt();
    let (s, c) = a.dsin_cos();
    V(c * d, 0.0, s * d)
}

/// The leaf colours of health `h` given its living colour: chlorosis
/// (yellow) first, then necrosis (brown), then straw.
pub fn aged(alive: [f64; 3], h: f64) -> [f64; 3] {
    let yellow = [0.66, 0.58, 0.22];
    let brown = [0.42, 0.30, 0.17];
    let straw = [0.55, 0.45, 0.30];
    if h > 0.66 {
        mix(yellow, alive, (h - 0.66) / 0.34 * 0.6 + 0.4)
    } else if h > 0.33 {
        mix(brown, yellow, (h - 0.33) / 0.33)
    } else {
        mix(straw, brown, h / 0.33)
    }
}

/// The gallery's materials for each surface.
pub fn material(s: Surface) -> Mat {
    match s {
        Surface::Leaf => phong!({ color: 0xffffff, vertexColors: true, specular: 0x2a2e22, shininess: 38, side: DoubleSide }),
        Surface::Wood => lambert!({ color: 0xffffff, vertexColors: true }),
        Surface::Pot => phong!({ color: 0xffffff, vertexColors: true, specular: 0x1a1a18, shininess: 20 }),
        Surface::Soil => lambert!({ color: 0xffffff, vertexColors: true }),
    }
}

/// The plant as scene nodes (one mesh a surface).
pub fn to_scene(p: &Plant) -> Obj {
    let g = group();
    g.set_name(&p.name);
    for s in [Surface::Pot, Surface::Soil, Surface::Wood, Surface::Leaf] {
        let mut all = MeshBuf::default();
        for (k, m) in &p.parts {
            if *k == s {
                all.extend(m);
            }
        }
        if all.triangles() > 0 {
            g.add(&mesh(all.to_geo(), material(s)));
        }
    }
    g
}

/// Its health's word, for labels.
pub fn health_word(h: f64) -> &'static str {
    if h > 0.75 {
        "tended"
    } else if h > 0.3 {
        "left a season"
    } else {
        "left for years"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(p: &Plant) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for (_, m) in &p.parts {
            for x in m.pos.iter().chain(m.col.iter()) {
                h = (h ^ x.to_bits() as u64).wrapping_mul(0x100_0000_01b3);
            }
        }
        h
    }

    #[test]
    fn a_seed_is_always_the_same_plant() {
        for s in Species::ALL {
            for h in [1.0, 0.5, 0.0] {
                assert_eq!(digest(&s.grow(7, h)), digest(&s.grow(7, h)), "{} at {h}", s.name());
            }
            assert_ne!(digest(&s.grow(7, 1.0)), digest(&s.grow(8, 1.0)), "{}: seeds differ", s.name());
        }
    }

    #[test]
    fn every_plant_stays_in_budget_and_finite() {
        for s in Species::ALL {
            for h in [1.0, 0.5, 0.0] {
                let p = s.grow(42, h);
                let t = p.triangles();
                assert!(t > 200 && t < 60_000, "{} at {h}: {t} triangles", s.name());
                for (_, m) in &p.parts {
                    assert!(m.pos.iter().all(|x| x.is_finite()), "{} at {h}: a point is not finite", s.name());
                    assert!(m.col.iter().all(|c| (0.0..=1.0).contains(c)), "{} at {h}: a colour out of range", s.name());
                }
            }
        }
    }

    #[test]
    fn neglect_shows_in_colour() {
        // the mean leaf colour moves from green toward brown as health falls
        for s in Species::ALL {
            let green = |h: f64| {
                let p = s.grow(3, h);
                let (mut g, mut rr, mut n) = (0.0f64, 0.0f64, 0.0f64);
                for (k, m) in &p.parts {
                    if *k == Surface::Leaf {
                        for c in m.col.chunks(3) {
                            rr += c[0] as f64;
                            g += c[1] as f64;
                            n += 1.0;
                        }
                    }
                }
                (g - rr) / n.max(1.0)
            };
            assert!(green(1.0) > green(0.5) && green(0.5) > green(0.0), "{}: {} {} {}", s.name(), green(1.0), green(0.5), green(0.0));
        }
    }
}
