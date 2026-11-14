//! The shape of a level as the light map carries it (`build_light_map`), so
//! corners darken where walls meet without a texture read of their own.
//!
//! The light map is RGBA at `FINE` texels a tile:
//! * R: the light (`build_light_map`'s per-tile value, sampled between the
//!   tiles as the GPU's linear filter sampled the old one-texel-a-tile map);
//! * G: how open the place is: the distance to the nearest wall that runs
//!   up to its ceiling, up to `REACH` (low partitions, under the ceiling,
//!   are left out);
//! * B: the ceiling over the place (its tile's, up to `CEIL_MAX`); or, where
//!   some places have no floor (the Building's void and masses: `gaps`),
//!   whether this one has (255) or not (0), so the shader can read the floor
//!   of the part of a filtered sample that has one (A over B);
//! * A: the floor under the place (`floor_codes`: its height over the
//!   level's lowest in `FLOOR_STEP`s, 0 where there is none, the void).
//!
//! The shader reads G, B and A from the sample it already takes `0.8 m` out
//! from each surface (its indirect light): a wall face beside another wall,
//! a floor or a ceiling beside a wall, there finds a wall nearer than
//! `REACH` and loses some of its indirect light; a wall near its ceiling
//! finds how far up the ceiling is (rooms R3); and a wall finds the floor
//! at its foot, which it darkens toward (the Building's storeys stand from
//! about −18 m to +21 m, a pool's floor under the rooms').
//!
//! The GPU filters the map linearly, so within a texel of a walk's end the
//! sample is part walk, part void: read alone, A there (the walk's code
//! times the walk's share) gave a floor anywhere from the walk's down to the
//! base, and a mass's face over the void beside the walk's end darkened in
//! a thin streak where its height met that floor, from the walk's down to
//! the lowest storey (code review 4). With B holding how much of the sample
//! has a floor, A over B is the walk's own floor there, and the shader
//! fades the floor's occlusion out with B over the void's first quarter
//! metre. The Building has no corners from the map (`corner_occlusion`), so
//! its B was free (its ceilings, open to the sky, all read `CEIL_MAX`).
use crate::base::config::CELL;
use crate::level::archgen::{BOX_ANCHOR, BOX_ANGLED, BOX_JAMB, BOX_PILLAR, BOX_WALL, Solids};

/// Texels of the light map a tile (4 m): 0.5 m each
pub const FINE: usize = 8;
/// How far off a wall stops darkening a place (m): the same as the
/// shader's offset from each surface, so a plain wall (its own face that
/// far off) is not darkened
pub const REACH: f64 = 0.8;
/// The highest ceiling B can hold (m)
pub const CEIL_MAX: f64 = 10.0;
/// A's step (m): a floor's code is its height over the level's base in
/// these (255 codes: 63.5 m over the lowest floor)
pub const FLOOR_STEP: f64 = 0.25;

/// The light map's fixed part: G, B and A, which do not change within a
/// level (its floors, walls and ceilings stand as built). Made once, at the
/// level's first `build_light_map` (`Game::ll_shape`, dropped by
/// `build_world`); a lamp switched off, a vending machine's light or her
/// lamp only repack the light beside it (`pack`). On the Building's floors
/// the lamps the fog passes rebuild the map (16 times a rise on 4242's
/// Well, and again as it falls), and each rebuild had made the floors anew
/// (code review 4)
#[derive(Clone, Debug, PartialEq)]
pub struct LightShape {
    /// the height of A's code 0 (m; `floor_codes`)
    pub base: f64,
    /// what G, B and A are made of (`shape_key`): the level's own shape,
    /// which nothing played changes (`tests/building_floors.rs`)
    pub key: u64,
    /// some places have no floor (the Building's void and masses): B holds
    /// whether each has one, not the ceiling (`shape_channels`)
    pub gaps: bool,
    /// G, B and A for every fine texel (`shape_channels`)
    pub channels: Vec<u8>,
}

impl LightShape {
    /// The shape of a level of `w` × `h` tiles: `floor_at` as `floor_codes`
    /// asks it
    pub fn new(
        w: i32,
        h: i32,
        tiles: &[u8],
        tile_ceil: &[f32],
        solids: Option<&Solids>,
        floor_at: impl Fn(f64, f64) -> Option<f64>,
    ) -> Self {
        let (floors, base) = floor_codes(w, h, floor_at);
        let key = shape_key(w, h, tiles, tile_ceil, solids, &floors);
        LightShape { base, key, gaps: floors.contains(&0), channels: shape_channels(w, h, tile_ceil, solids, &floors) }
    }
    /// Whether it is the shape of a level of `w` × `h` tiles
    pub fn fits(&self, w: i32, h: i32) -> bool {
        self.channels.len() == w.max(0) as usize * FINE * h.max(0) as usize * FINE * 3
    }
}

/// A for every fine texel (one byte a texel, rows of `w * FINE`) and its
/// base (the height of code 0, a step under the lowest floor, on the
/// step's grid so a floor on it is exact): `floor_at(u, v)` is the floor at
/// a texel's centre (tile units), a flight's where it is along its run;
/// None where there is none (the void, a building's mass), coded 0
pub fn floor_codes(w: i32, h: i32, floor_at: impl Fn(f64, f64) -> Option<f64>) -> (Vec<u8>, f64) {
    let (wf, hf) = (w as usize * FINE, h as usize * FINE);
    let floors: Vec<Option<f64>> =
        (0..wf * hf).map(|k| floor_at(((k % wf) as f64 + 0.5) / FINE as f64, ((k / wf) as f64 + 0.5) / FINE as f64)).collect();
    let low = floors.iter().flatten().fold(f64::INFINITY, |a, &b| a.min(b));
    let base = if low.is_finite() { (low / FLOOR_STEP).floor() * FLOOR_STEP - FLOOR_STEP } else { 0.0 };
    let codes = floors.iter().map(|f| f.map_or(0, |y| ((y - base) / FLOOR_STEP).round().clamp(1.0, 255.0) as u8)).collect();
    (codes, base)
}

/// A key for a level's shape: what G, B and A are made of. FNV-1a over the
/// grid, the ceilings, the boxes and the floors (`floor_codes`)
pub fn shape_key(w: i32, h: i32, tiles: &[u8], tile_ceil: &[f32], solids: Option<&Solids>, floors: &[u8]) -> u64 {
    let mut x: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: u8| {
        x ^= u64::from(b);
        x = x.wrapping_mul(0x0100_0000_01b3);
    };
    for b in w.to_le_bytes().into_iter().chain(h.to_le_bytes()) {
        eat(b);
    }
    for &t in tiles {
        eat(t);
    }
    for c in tile_ceil {
        for b in c.to_bits().to_le_bytes() {
            eat(b);
        }
    }
    if let Some(s) = solids {
        for v in &s.boxes {
            for b in v.to_bits().to_le_bytes() {
                eat(b);
            }
        }
    }
    for &f in floors {
        eat(f);
    }
    x
}

/// G, B and A for every fine texel (three bytes a texel, rows of
/// `w * FINE`): how open, the ceiling, and the floor (`floors`, from
/// `floor_codes`). Where some texel has no floor (code 0: the Building's
/// void and masses), B is whether each has one (255 or 0) in place of the
/// ceiling. `solids` in tile units (x, z) and metres (y)
pub fn shape_channels(w: i32, h: i32, tile_ceil: &[f32], solids: Option<&Solids>, floors: &[u8]) -> Vec<u8> {
    let (wf, hf) = (w as usize * FINE, h as usize * FINE);
    let gaps = floors.contains(&0);
    let mut out = vec![0u8; wf * hf * 3];
    let ceil_of = |tx: i32, tz: i32| -> f64 {
        if tx < 0 || tz < 0 || tx >= w || tz >= h {
            return 0.0;
        }
        tile_ceil.get((tz * w + tx) as usize).map_or(0.0, |c| *c as f64)
    };
    // the walls that run up to their ceiling: they shut a corner in
    let tall: Vec<bool> = match solids {
        Some(s) => (0..s.count)
            .map(|b| {
                let bx = s.box_at(b);
                let kind = bx[7] as i32;
                let full = matches!(kind, BOX_WALL | BOX_PILLAR | BOX_JAMB | BOX_ANCHOR | BOX_ANGLED);
                let (cx, cz) = (((bx[0] + bx[2]) * 0.5).floor() as i32, ((bx[1] + bx[3]) * 0.5).floor() as i32);
                full && bx[4] < 0.3 && bx[5] as f64 >= ceil_of(cx, cz) - 0.15
            })
            .collect(),
        None => vec![],
    };
    for j in 0..hf {
        for i in 0..wf {
            // the texel's centre, in tile units
            let (u, v) = ((i as f64 + 0.5) / FINE as f64, (j as f64 + 0.5) / FINE as f64);
            let (tx, tz) = (u.floor() as i32, v.floor() as i32);
            let mut d = REACH;
            if let Some(s) = solids {
                for nz in tz - 1..=tz + 1 {
                    for nx in tx - 1..=tx + 1 {
                        if nx < 0 || nz < 0 || nx >= s.w || nz >= s.h {
                            continue;
                        }
                        let t = (nz * s.w + nx) as usize;
                        for k in s.start[t]..s.start[t + 1] {
                            let b = s.items[k as usize] as usize;
                            if !tall[b] {
                                continue;
                            }
                            let bx = s.box_at(b);
                            let dx = (bx[0] as f64 - u).max(u - bx[2] as f64).max(0.0);
                            let dz = (bx[1] as f64 - v).max(v - bx[3] as f64).max(0.0);
                            d = d.min((dx * dx + dz * dz).sqrt() * CELL);
                        }
                    }
                }
            }
            let o = (j * wf + i) * 3;
            out[o] = (d / REACH * 255.0).round().clamp(0.0, 255.0) as u8;
            let floor = floors.get(j * wf + i).copied().unwrap_or(0);
            out[o + 1] =
                if gaps { if floor > 0 { 255 } else { 0 } } else { (ceil_of(tx, tz) / CEIL_MAX * 255.0).round().clamp(0.0, 255.0) as u8 };
            out[o + 2] = floor;
        }
    }
    out
}

/// The light map's texels: `light` (one byte a tile, rows of `w`) sampled
/// at each fine texel as a linear filter samples it, beside `shape`'s G, B
/// and A
pub fn pack(w: i32, h: i32, light: &[u8], shape: &[u8]) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let (wf, hf) = (w * FINE, h * FINE);
    let at = |x: usize, z: usize| light[z * w + x] as f64;
    let mut out = vec![255u8; wf * hf * 4];
    for j in 0..hf {
        // the coarse map's continuous coordinate (its texel centres at k + 0.5)
        let fz = ((j as f64 + 0.5) / FINE as f64 - 0.5).clamp(0.0, (h - 1) as f64);
        let (z0, tz) = (fz.floor() as usize, fz - fz.floor());
        let z1 = (z0 + 1).min(h - 1);
        for i in 0..wf {
            let fx = ((i as f64 + 0.5) / FINE as f64 - 0.5).clamp(0.0, (w - 1) as f64);
            let (x0, tx) = (fx.floor() as usize, fx - fx.floor());
            let x1 = (x0 + 1).min(w - 1);
            let r = (at(x0, z0) * (1.0 - tx) + at(x1, z0) * tx) * (1.0 - tz) + (at(x0, z1) * (1.0 - tx) + at(x1, z1) * tx) * tz;
            let o = (j * wf + i) * 4;
            let s = (j * wf + i) * 3;
            out[o] = r.round().clamp(0.0, 255.0) as u8;
            out[o + 1] = shape.get(s).copied().unwrap_or(255);
            out[o + 2] = shape.get(s + 1).copied().unwrap_or(0);
            out[o + 3] = shape.get(s + 2).copied().unwrap_or(0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor's code is its height over the base in steps (exact on the
    /// step's grid), the void 0; a flight is read where the texel is along it
    #[test]
    fn the_floor_under_each_texel() {
        // two tiles: a floor at -18 m, then a flight rising +x to -15 m; the
        // void beyond the grid's second row
        let floor_at = |u: f64, v: f64| -> Option<f64> {
            if v >= 1.0 {
                None
            } else if u < 1.0 {
                Some(-18.0)
            } else {
                Some(-18.0 + 3.0 * (u - 1.0))
            }
        };
        let (codes, base) = floor_codes(2, 2, floor_at);
        assert_eq!(base, -18.25);
        let wf = 2 * FINE;
        let at = |i: usize, j: usize| codes[j * wf + i] as f64 * FLOOR_STEP + base;
        assert_eq!(at(3, 2), -18.0, "the floor, exact");
        // the flight's texel k is at (k + 0.5) / FINE along it
        for k in 0..FINE {
            let want = -18.0 + 3.0 * (k as f64 + 0.5) / FINE as f64;
            assert!((at(FINE + k, 2) - want).abs() <= FLOOR_STEP / 2.0, "the flight at {k}: {} vs {want}", at(FINE + k, 2));
        }
        assert_eq!(codes[FINE * wf + 3], 0, "the void");
        // packed into the light map's alpha; with the void beside it, B is
        // whether a texel has a floor
        let shape = shape_channels(2, 2, &[3.0; 4], None, &codes);
        let px = pack(2, 2, &[128; 4], &shape);
        assert_eq!(px[(2 * wf + 3) * 4 + 3], codes[2 * wf + 3]);
        assert_eq!(px[(FINE * wf + 3) * 4 + 3], 0);
        assert_eq!((px[(2 * wf + 3) * 4 + 2], px[(FINE * wf + 3) * 4 + 2]), (255, 0), "B: a floor, the void");
        // a linear filter halfway between the flight's top texel and the
        // void's beside it: A alone is half its code (a floor halfway down
        // to the base), A over B its own
        let (walk, void) = ((FINE - 1) * wf + 2 * FINE - 1, FINE * wf + 2 * FINE - 1);
        let half = |c: usize| (px[walk * 4 + c] as f64 + px[void * 4 + c] as f64) / 2.0;
        let decode = |code: f64| code * FLOOR_STEP + base;
        let top = decode(px[walk * 4 + 3] as f64);
        assert!(top > -15.5, "the flight's top: {top}");
        assert_eq!(decode(half(3) / (half(2) / 255.0)), top, "A over B: its floor");
        assert_eq!(decode(half(3)), (top + base) / 2.0, "A alone: halfway down to the base");
        // where every texel has a floor (the rooms), B is the ceiling
        let rooms = floor_codes(2, 2, |_, _| Some(0.0)).0;
        let shape = shape_channels(2, 2, &[3.0; 4], None, &rooms);
        assert!(shape.chunks(3).all(|t| t[1] == (3.0 / CEIL_MAX * 255.0_f64).round() as u8), "B: the ceiling");
    }
}
