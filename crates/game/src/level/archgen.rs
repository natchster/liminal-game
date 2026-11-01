//! archgen — the Backrooms architecture generator.
//!
//! Pure module (no scene, no engine objects). Two jobs:
//!
//! * [`generate_layout`] carves a level out of *zones*. Each zone is a
//!   rectangle of the tile grid given an archetype from the lore — the Lobby's
//!   randomly segmented rooms, pillar fields, low abandoned offices, the old
//!   thick-walled maze, narrow utility tunnels, tall concrete halls — each with
//!   its own ceiling height (shared by neighbours of the same kind), wall
//!   thickness, doorway jambs and materials. The carvers' rooms are kept in
//!   `Layout::rooms`. A repair pass then guarantees every open tile is
//!   reachable from spawn.
//! * [`build_solids`] turns wall *tiles* into solid *boxes*. In thin-walled
//!   zones a wall tile becomes a partition along the tile's centre line, lone
//!   wall tiles become pillars, doorways get jambs and a lintel, and tunnel
//!   walls bulge into the corridor to narrow it. Collision, sight lines and
//!   rendering all read the boxes.
//!
//! Coordinates are *tile units*: tile (x, z) covers [x, x+1] × [z, z+1].
//! Heights (y) are metres. The game converts tile units to metres with CELL.
//! Randomness comes from an injected `rng` so a level seed reproduces a level
//! (the game passes `&mut || rnd()` inside `with_seed`).
//!
//! Bit-reproducible: a fixed draw order, all real math in f64, decimal
//! rounding on the exact binary value ([`round_decimals`]), boxes stored as
//! f32 and the CSR index as i32. Queries loop over f64 tile coordinates, so
//! even NaN / infinite inputs have defined answers. `tests/archgen.rs` checks
//! all of it against recorded fixtures.

use crate::base::numeric::clamp;
use crate::base::numeric::*;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

/// +x, -x, +z, -z
pub const DIRS: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
/// TileFlags bit: low partition (cubicle wall)
pub const TF_LOW: u8 = 1;

// Box kinds (the 8th field of a box record).
pub const BOX_WALL: i32 = 0;
pub const BOX_PILLAR: i32 = 1;
pub const BOX_JAMB: i32 = 2;
pub const BOX_LINTEL: i32 = 3;
pub const BOX_ANCHOR: i32 = 4;
/// A void (the Building's floors): no floor, a drop into the fog. It stops a
/// body but not a look.
pub const BOX_VOID: i32 = 5;
/// The slab under a floor that has height (the Building's floors): it
/// stops a look, never a body, which stands on it.
pub const BOX_FLOOR: i32 = 6;
/// A step of a leaning wall (`world::angled`): collides like a wall; drawn as the wall it stands in for.
pub const BOX_ANGLED: i32 = 7;

// ---------------------------------------------------------------------------
// Ground: floors with height. The rooms stand on y = 0 and have none (an
// empty `Layout::ground`); the Building's floors (`level::building`) give
// every tile one. Heights change only along a stair tile or across a wall
// or a void, never between two open neighbours, so every search over the
// tile grid (reachability, her walk, the exit) stays true.
// ---------------------------------------------------------------------------
/// `Ground::kind`: an open floor at `h0`
pub const GROUND_FLOOR: u8 = 0;
/// `Ground::kind`: no floor (the tile is closed: a drop into the fog)
pub const GROUND_VOID: u8 = 1;
/// `Ground::kind`: a flight of stairs from `h0` on its back edge to `h1` on
/// its front edge (the side `DIRS[dir]` points to)
pub const GROUND_STAIR: u8 = 2;
/// `Ground::kind`: a wall or a mass of building, from `h0` up to `h1`
pub const GROUND_MASS: u8 = 3;

/// What one tile stands on (metres).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ground {
    pub kind: u8,
    pub h0: f32,
    pub h1: f32,
    /// a stair's direction (a `DIRS` index): its floor goes from `h0` to `h1` that way
    pub dir: u8,
}
impl Ground {
    pub const fn floor(h: f32) -> Ground {
        Ground { kind: GROUND_FLOOR, h0: h, h1: h, dir: 0 }
    }
    pub const fn void() -> Ground {
        Ground { kind: GROUND_VOID, h0: 0.0, h1: 0.0, dir: 0 }
    }
    pub const fn mass(foot: f32, top: f32) -> Ground {
        Ground { kind: GROUND_MASS, h0: foot, h1: top, dir: 0 }
    }
    pub const fn stair(h0: f32, h1: f32, dir: u8) -> Ground {
        Ground { kind: GROUND_STAIR, h0, h1, dir }
    }
    /// Whether a body can stand here
    pub fn walkable(&self) -> bool {
        self.kind == GROUND_FLOOR || self.kind == GROUND_STAIR
    }
    /// The height of the floor at (u, v) in tile units inside tile (x, z):
    /// level, or along a stair's run
    pub fn at(&self, x: i32, z: i32, u: f64, v: f64) -> f64 {
        if self.kind != GROUND_STAIR {
            return self.h0 as f64;
        }
        let s = match self.dir {
            0 => u - x as f64,
            1 => (x + 1) as f64 - u,
            2 => v - z as f64,
            _ => (z + 1) as f64 - v,
        };
        let s = s.clamp(0.0, 1.0);
        self.h0 as f64 + (self.h1 as f64 - self.h0 as f64) * s
    }
    /// The height where this tile meets its neighbour in `DIRS[d]`: its
    /// floor's edge on that side (a stair's ends; its sides are closed)
    pub fn edge(&self, d: usize) -> Option<f64> {
        match self.kind {
            GROUND_FLOOR => Some(self.h0 as f64),
            GROUND_STAIR if d == self.dir as usize => Some(self.h1 as f64),
            GROUND_STAIR if d == (self.dir as usize ^ 1) => Some(self.h0 as f64),
            _ => None,
        }
    }
}

/// Whether two neighbouring grounds (b in `DIRS[d]` of a) meet where both
/// can be walked: open on both sides, at the same height
pub fn grounds_meet(a: &Ground, b: &Ground, d: usize) -> bool {
    // two flights side by side, the same run (a wide flight)
    if a.kind == GROUND_STAIR && b.kind == GROUND_STAIR && (a.dir as usize) / 2 != d / 2 {
        return a.dir == b.dir && (a.h0 - b.h0).abs() < 0.05 && (a.h1 - b.h1).abs() < 0.05;
    }
    match (a.edge(d), b.edge(d ^ 1)) {
        (Some(ha), Some(hb)) => (ha - hb).abs() < 0.05,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Archetypes. Ranges are [min, max] in metres unless noted.
//   wall:  "thin"   partition down the middle of the wall tile (t = thickness)
//          "solid"  whole 4 m tile (the classic look)
//          "thick"  whole tile plus `protrude` metres into the corridor
//   jamb:  metres each doorway side is narrowed by, lintel: header height
//   light_every: ceiling fixture spacing in tiles
// ---------------------------------------------------------------------------
/// One entry of `ARCHETYPES` (`key` is its lookup key).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Archetype {
    pub key: &'static str,
    pub title: &'static str,
    pub lore: &'static str,
    pub ceil: [f64; 2],
    pub wall: &'static str,
    pub t: [f64; 2],
    /// (only the thick-walled archetypes have one)
    pub protrude: Option<[f64; 2]>,
    pub pillar: [f64; 2],
    pub jamb: [f64; 2],
    pub jamb_chance: f64,
    pub lintel: [f64; 2],
    /// (office, store)
    pub low_h: Option<[f64; 2]>,
    pub wall_mat: &'static str,
    pub floor_mat: &'static str,
    pub ceil_mat: &'static str,
    pub light_every: i32,
    /// (store)
    pub no_spawn: bool,
}

/// Every archetype, in a fixed order.
pub static ARCHETYPES: [Archetype; 9] = [
    Archetype {
        key: "lobby",
        title: "THE LOBBY",
        lore: "Level 0",
        ceil: [2.7, 3.05],
        wall: "thin",
        t: [0.16, 0.26],
        protrude: None,
        pillar: [0.5, 0.8],
        jamb: [0.8, 1.35],
        jamb_chance: 0.55,
        lintel: [2.15, 2.4],
        low_h: None,
        wall_mat: "wallpaper",
        floor_mat: "carpet",
        ceil_mat: "tiles",
        light_every: 2,
        no_spawn: false,
    },
    Archetype {
        key: "pillars",
        title: "PILLAR FIELD",
        lore: "Level 0 · pillar grid",
        ceil: [3.2, 3.7],
        wall: "thin",
        t: [0.2, 0.3],
        protrude: None,
        pillar: [0.7, 1.15],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.4, 2.6],
        low_h: None,
        wall_mat: "wallpaper",
        floor_mat: "carpet",
        ceil_mat: "tiles",
        light_every: 2,
        no_spawn: false,
    },
    Archetype {
        key: "office",
        title: "ABANDONED OFFICE",
        lore: "Level 4",
        ceil: [2.5, 2.75],
        wall: "thin",
        t: [0.12, 0.18],
        protrude: None,
        pillar: [0.4, 0.55],
        jamb: [1.1, 1.3],
        jamb_chance: 0.9,
        lintel: [2.05, 2.15],
        low_h: Some([1.35, 1.6]),
        wall_mat: "drywall",
        floor_mat: "office",
        ceil_mat: "tiles",
        light_every: 2,
        no_spawn: false,
    },
    Archetype {
        key: "maze",
        title: "THE MAZE",
        lore: "Level 0 · deep rot",
        ceil: [2.95, 3.3],
        wall: "solid",
        t: [4.0, 4.0],
        protrude: None,
        pillar: [4.0, 4.0],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.4, 2.6],
        low_h: None,
        wall_mat: "wallpaper",
        floor_mat: "carpet",
        ceil_mat: "tiles",
        light_every: 2,
        no_spawn: false,
    },
    Archetype {
        key: "tunnels",
        title: "UTILITY HALLS",
        lore: "Level 2",
        ceil: [2.35, 2.65],
        wall: "thick",
        t: [4.0, 4.0],
        protrude: Some([0.55, 1.15]),
        pillar: [4.0, 4.0],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.0, 2.1],
        low_h: None,
        wall_mat: "concrete",
        floor_mat: "concrete",
        ceil_mat: "slab",
        light_every: 3,
        no_spawn: false,
    },
    Archetype {
        key: "hall",
        title: "HABITABLE ZONE",
        lore: "Level 1",
        ceil: [5.4, 7.6],
        wall: "thin",
        t: [0.35, 0.5],
        protrude: None,
        pillar: [0.95, 1.4],
        jamb: [0.6, 1.0],
        jamb_chance: 0.4,
        lintel: [3.2, 3.6],
        low_h: None,
        wall_mat: "concrete",
        floor_mat: "concrete",
        ceil_mat: "slab",
        light_every: 3,
        no_spawn: false,
    },
    Archetype {
        key: "electrical",
        title: "ELECTRICAL STATION",
        lore: "Level 3",
        ceil: [2.6, 3.1],
        wall: "thick",
        t: [4.0, 4.0],
        protrude: Some([0.3, 0.8]),
        pillar: [4.0, 4.0],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.1, 2.2],
        low_h: None,
        wall_mat: "brick",
        floor_mat: "concrete",
        ceil_mat: "slab",
        light_every: 3,
        no_spawn: false,
    },
    // the Showroom (an anomaly level, scp 'store'): room-sets behind 2.4 m
    // showroom partitions under a high warehouse slab, bright as a shop
    Archetype {
        key: "store",
        title: "THE SHOWROOM",
        lore: "a store that never closes",
        ceil: [4.4, 4.9],
        wall: "thin",
        t: [0.1, 0.14],
        protrude: None,
        pillar: [0.45, 0.6],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.6, 2.8],
        low_h: Some([2.35, 2.5]),
        wall_mat: "drywall",
        floor_mat: "concrete",
        ceil_mat: "slab",
        light_every: 2,
        no_spawn: true,
    },
    Archetype {
        key: "pool",
        title: "THE POOLROOMS",
        lore: "Level 37",
        ceil: [3.4, 4.6],
        wall: "thin",
        t: [0.3, 0.4],
        protrude: None,
        pillar: [0.7, 1.0],
        jamb: [0.5, 0.9],
        jamb_chance: 0.3,
        lintel: [2.6, 3.0],
        low_h: None,
        wall_mat: "tile",
        floor_mat: "tile",
        ceil_mat: "plaster",
        light_every: 2,
        no_spawn: false,
    },
];

/// `ARCHETYPES[key]` (None for keys that are not archetypes, e.g. "house").
pub fn archetype(key: &str) -> Option<&'static Archetype> {
    ARCHETYPES.iter().find(|a| a.key == key).or_else(|| crate::level::building::BUILDING_ARCHETYPES.iter().find(|a| a.key == key))
}
fn arch_of(key: &str) -> &'static Archetype {
    archetype(key).unwrap_or_else(|| panic!("archgen: unknown archetype {key:?}"))
}

// Which archetypes a level mixes, and which one it is known for. Game level n
// is "Backrooms level n-1": 1 → the Lobby, 2 → the Habitable Zone, 3 → the
// Utility Halls, 4 → the Electrical Station, 5 → the Office, then it churns.
const DOMINANT_BY_LEVEL: [&str; 5] = ["lobby", "hall", "tunnels", "electrical", "office"];
const DEEP_DOMINANT: [&str; 8] = ["lobby", "hall", "tunnels", "office", "maze", "pillars", "electrical", "pool"];

/// The return value of `level_schedule`. `weights` keeps a fixed key order
/// (it decides `pick_weighted`).
#[derive(Clone, Debug, PartialEq)]
pub struct LevelSchedule {
    pub dominant: &'static str,
    pub weights: Vec<(&'static str, f64)>,
    pub patchwork: bool,
}

fn weight_mut<'a>(w: &'a mut [(&'static str, f64)], k: &str) -> &'a mut f64 {
    &mut w.iter_mut().find(|(n, _)| *n == k).expect("weight key").1
}

pub fn level_schedule(level: i32, scp: Option<&str>) -> LevelSchedule {
    let n = (level as i64).max(1);
    // anomaly levels (scp) bring their own mix
    if scp == Some("store") {
        return LevelSchedule { dominant: "store", weights: vec![("store", 1.0)], patchwork: false };
    }
    if scp == Some("house") {
        // the Borrowed House: rooms lifted from every kind of place, small and
        // stitched together so that no two neighbours come from the same one
        return LevelSchedule {
            dominant: "house",
            patchwork: true,
            weights: vec![
                ("lobby", 1.0),
                ("office", 1.0),
                ("hall", 0.7),
                ("tunnels", 0.8),
                ("electrical", 0.8),
                ("pool", 0.7),
                ("pillars", 0.6),
            ],
        };
    }
    let deep = n > DOMINANT_BY_LEVEL.len() as i64;
    let dominant = if deep { DEEP_DOMINANT[((n * 7 + 3) % 8) as usize] } else { DOMINANT_BY_LEVEL[(n - 1) as usize] };
    // the canonical Lobby leaks into every level; the rest opens up with depth
    let mut w: Vec<(&'static str, f64)> = vec![
        ("lobby", 3.0),
        ("pillars", 1.2),
        ("maze", 0.0),
        ("office", 0.0),
        ("hall", 0.0),
        ("tunnels", 0.0),
        ("electrical", 0.0),
        ("pool", 0.0),
    ];
    if n >= 2 {
        *weight_mut(&mut w, "maze") = 1.0;
        *weight_mut(&mut w, "office") = 0.8;
        *weight_mut(&mut w, "hall") = 0.8;
    }
    if n >= 3 {
        *weight_mut(&mut w, "tunnels") = 1.0;
        *weight_mut(&mut w, "pillars") = 1.0;
    }
    if n >= 4 {
        *weight_mut(&mut w, "electrical") = 0.8;
        *weight_mut(&mut w, "office") = 1.2;
        *weight_mut(&mut w, "lobby") = 2.0;
    }
    if n >= 6 {
        *weight_mut(&mut w, "pool") = 0.35;
        *weight_mut(&mut w, "maze") = 1.4;
    }
    *weight_mut(&mut w, dominant) += if n == 1 { 5.0 } else { 4.0 };
    LevelSchedule { dominant, weights: w, patchwork: false }
}

// ---------------------------------------------------------------------------
// small helpers
// ---------------------------------------------------------------------------
/// A mulberry32 stream as a closure (the generator of
/// [`crate::base::rng::Mulberry32`]).
pub fn mulberry32(seed: u32) -> impl FnMut() -> f64 {
    let mut m = crate::base::rng::Mulberry32::new(seed);
    move || m.next()
}

#[inline]
fn iclamp(v: i32, a: i32, b: i32) -> i32 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}

fn pick_weighted(rng: &mut dyn FnMut() -> f64, weights: &[(&'static str, f64)]) -> &'static str {
    let mut total = 0.0;
    for &(_, w) in weights {
        total += nan_max(0.0, w);
    }
    let mut r = rng() * total;
    for &(k, w) in weights {
        r -= nan_max(0.0, w);
        if r < 0.0 {
            return k;
        }
    }
    weights[0].0
}
#[inline]
fn range_in(rng: &mut dyn FnMut() -> f64, ab: [f64; 2]) -> f64 {
    let t = rng();
    ab[0] + (ab[1] - ab[0]) * t
}
#[inline]
fn irange(rng: &mut dyn FnMut() -> f64, a: i32, b: i32) -> i32 {
    a + (rng() * (b - a + 1) as f64).floor() as i32
}

// ---------------------------------------------------------------------------
// layout
// ---------------------------------------------------------------------------
/// Options of [`generate_layout`] (the `rng` is a separate argument).
/// `level` 0 means 1 and `cell` 0 means 4.
#[derive(Clone, Copy, Debug)]
pub struct LayoutOpts<'a> {
    /// grid side in tiles (made odd)
    pub size: i32,
    /// game level (1-based)
    pub level: i32,
    /// metres per tile
    pub cell: f64,
    /// the anomaly on this level (`level_scp`), or None
    pub scp: Option<&'a str>,
}
impl Default for LayoutOpts<'_> {
    fn default() -> Self {
        LayoutOpts { size: 31, level: 1, cell: 4.0, scp: None }
    }
}

/// A zone of the layout. `id` is its index in `Layout::zones`.
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub id: usize,
    pub arch: &'static str,
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
    pub title: &'static str,
    pub lore: &'static str,
    pub ceil_h: f64,
    pub wall: &'static str,
    pub wall_t: f64,
    pub pillar: f64,
    pub protrude: f64,
    pub jamb: f64,
    pub lintel_h: f64,
    pub low_h: f64,
    pub wall_mat: &'static str,
    pub floor_mat: &'static str,
    pub ceil_mat: &'static str,
    pub light_every: i32,
}
impl Zone {
    fn rect(&self) -> Rect {
        Rect { x0: self.x0, z0: self.z0, x1: self.x1, z1: self.z1 }
    }
    /// Tiles covered by the zone rectangle
    pub fn area(&self) -> i32 {
        (self.x1 - self.x0 + 1) * (self.z1 - self.z0 + 1)
    }
    /// The zones share an edge (not just a corner)
    pub fn touches(&self, o: &Zone) -> bool {
        let x_overlap = self.x0.max(o.x0) <= self.x1.min(o.x1);
        let z_overlap = self.z0.max(o.z0) <= self.z1.min(o.z1);
        (x_overlap && (self.z1 + 1 == o.z0 || o.z1 + 1 == self.z0)) || (z_overlap && (self.x1 + 1 == o.x0 || o.x1 + 1 == self.x0))
    }
}

/// A room the carver laid out inside a zone (tiles, inclusive): a leaf of
/// the room splitter, or the whole zone where the carver makes no rooms.
/// Seams and repair tunnels may have opened its walls since.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Room {
    pub zone: usize,
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}
impl Room {
    pub fn contains(&self, x: i32, z: i32) -> bool {
        x >= self.x0 && x <= self.x1 && z >= self.z0 && z <= self.z1
    }
}

/// The return value of `generate_layout`.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// Grid dimensions in tiles
    pub w: i32,
    pub h: i32,
    pub cell: f64,
    /// 1 wall, 0 open
    pub tiles: Vec<u8>,
    /// zone id per tile
    pub zone_of: Vec<u8>,
    /// `TF_LOW` bits per tile
    pub tile_flags: Vec<u8>,
    pub zones: Vec<Zone>,
    /// the level's dominant archetype ("house" on the Borrowed House)
    pub dominant: &'static str,
    pub level: i32,
    pub scp: Option<String>,
    /// the carvers' rooms, zone by zone
    pub rooms: Vec<Room>,
    /// what each tile stands on (empty: the rooms, all floors at y = 0)
    pub ground: Vec<Ground>,
}
impl Layout {
    /// Tile index `z * w + x`
    #[inline]
    pub fn idx(&self, x: i32, z: i32) -> usize {
        (z * self.w + x) as usize
    }
    /// The zone a tile belongs to, coordinates clamped into the grid.
    pub fn zone_at(&self, x: i32, z: i32) -> &Zone {
        &self.zones[self.zone_of[self.idx(iclamp(x, 0, self.w - 1), iclamp(z, 0, self.h - 1))] as usize]
    }
    /// The room around tile (x, z), if the carver laid one out there
    pub fn room_at(&self, x: i32, z: i32) -> Option<&Room> {
        self.rooms.iter().find(|r| r.contains(x, z))
    }
    /// A borrowed view for [`build_solids_view`] / [`layout_stats_view`].
    pub fn view(&self) -> LayoutView<'_> {
        LayoutView {
            w: self.w,
            h: self.h,
            cell: self.cell,
            tiles: &self.tiles,
            zone_of: &self.zone_of,
            tile_flags: Some(&self.tile_flags),
            zones: &self.zones,
        }
    }
}

/// A borrowed layout (grid size, tiles, zone map, zones, tile flags): lets
/// the game build solids over its own copy of the tile grid.
#[derive(Clone, Copy, Debug)]
pub struct LayoutView<'a> {
    pub w: i32,
    pub h: i32,
    /// 0 = fall back to `opt.cell`, then 4
    pub cell: f64,
    pub tiles: &'a [u8],
    pub zone_of: &'a [u8],
    /// None = all zero
    pub tile_flags: Option<&'a [u8]>,
    pub zones: &'a [Zone],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x0: i32,
    z0: i32,
    x1: i32,
    z1: i32,
}

/// The carving context.
struct Ctx<'r> {
    w: i32,
    h: i32,
    tiles: Vec<u8>,
    tile_flags: Vec<u8>,
    rng: &'r mut dyn FnMut() -> f64,
    /// the zone being carved, and the rooms laid out so far
    zone: usize,
    rooms: Vec<Room>,
}
impl Ctx<'_> {
    #[inline]
    fn idx(&self, x: i32, z: i32) -> usize {
        (z * self.w + x) as usize
    }
    #[inline]
    fn r(&mut self) -> f64 {
        (self.rng)()
    }
    #[inline]
    fn irange(&mut self, a: i32, b: i32) -> i32 {
        irange(self.rng, a, b)
    }
    #[inline]
    fn t(&self, x: i32, z: i32) -> u8 {
        self.tiles[self.idx(x, z)]
    }
    #[inline]
    fn set(&mut self, x: i32, z: i32, v: u8) {
        let i = self.idx(x, z);
        self.tiles[i] = v;
    }
}

/// Generates a level layout from `o`, drawing from `rng` (the game passes
/// `&mut || rnd()`).
pub fn generate_layout(o: &LayoutOpts, rng: &mut dyn FnMut() -> f64) -> Layout {
    let cell = if o.cell != 0.0 && !o.cell.is_nan() { o.cell } else { 4.0 };
    let w = if o.size % 2 != 0 { o.size } else { o.size + 1 };
    let h = w;
    let n = (w * h) as usize;
    let level = if o.level != 0 { o.level } else { 1 };
    let scp = o.scp.filter(|s| !s.is_empty());
    let sched = level_schedule(level, scp);
    let dominant = sched.dominant;
    let weights = &sched.weights;
    let idx = |x: i32, z: i32| (z * w + x) as usize;

    // ---- 1. partition the interior into zone rectangles (BSP) ----
    let mut rects: Vec<Rect> = vec![];
    let (min, max) = if sched.patchwork { (4, 8) } else { (7, 17) };
    let mut stack = vec![Rect { x0: 1, z0: 1, x1: w - 2, z1: h - 2 }];
    while let Some(r) = stack.pop() {
        let rw = r.x1 - r.x0 + 1;
        let rh = r.z1 - r.z0 + 1;
        let can_x = rw >= min * 2;
        let can_z = rh >= min * 2;
        let small = rw <= max && rh <= max;
        if (!can_x && !can_z) || (small && rng() < 0.45) {
            rects.push(r);
            continue;
        }
        let axis_x = if can_x && can_z {
            if rw as f64 > rh as f64 * 1.25 {
                true
            } else if rh as f64 > rw as f64 * 1.25 {
                false
            } else {
                rng() < 0.5
            }
        } else {
            can_x
        };
        if axis_x {
            let s = irange(rng, r.x0 + min, r.x1 - min + 1);
            stack.push(Rect { x0: r.x0, z0: r.z0, x1: s - 1, z1: r.z1 });
            stack.push(Rect { x0: s, z0: r.z0, x1: r.x1, z1: r.z1 });
        } else {
            let s = irange(rng, r.z0 + min, r.z1 - min + 1);
            stack.push(Rect { x0: r.x0, z0: r.z0, x1: r.x1, z1: s - 1 });
            stack.push(Rect { x0: r.x0, z0: s, x1: r.x1, z1: r.z1 });
        }
    }

    // ---- 2. archetype + concrete style per zone ----
    let mut zones: Vec<Zone> = Vec::with_capacity(rects.len());
    for (i, r) in rects.iter().enumerate() {
        let has_spawn = r.x0 <= 1 && r.z0 <= 1;
        let mut arch = pick_weighted(rng, weights);
        // the first room of the first level is the photograph, always
        if has_spawn && level == 1 {
            arch = "lobby";
        }
        // never start a level inside a tunnel or a maze pocket: spawn reads as open space
        if has_spawn && (arch == "tunnels" || arch == "maze" || arch == "electrical" || arch_of(arch).no_spawn) {
            arch = if dominant == "hall" { "hall" } else { "lobby" };
        }
        zones.push(make_zone(arch, r, i, rng, cell));
    }
    // a patchwork never puts two rooms from the same kind of place side by side
    if sched.patchwork {
        let touch = |a: &Zone, b: &Zone| {
            a.x0 <= b.x1 + 1
                && b.x0 <= a.x1 + 1
                && a.z0 <= b.z1 + 1
                && b.z0 <= a.z1 + 1
                && !((a.x0 == b.x1 + 1 || b.x0 == a.x1 + 1) && (a.z0 == b.z1 + 1 || b.z0 == a.z1 + 1))
        };
        let kinds: Vec<&'static str> = weights.iter().map(|&(k, _)| k).collect();
        for _pass in 0..4 {
            for zi in 0..zones.len() {
                let near: Vec<&'static str> =
                    (0..zones.len()).filter(|&yi| yi != zi && touch(&zones[yi], &zones[zi])).map(|yi| zones[yi].arch).collect();
                let z = &zones[zi];
                if !near.contains(&z.arch) {
                    continue;
                }
                let has_spawn = z.x0 <= 1 && z.z0 <= 1;
                let free: Vec<&'static str> = kinds
                    .iter()
                    .copied()
                    .filter(|k| !near.contains(k) && !(has_spawn && (*k == "tunnels" || *k == "electrical")))
                    .collect();
                if !free.is_empty() {
                    let (rect, id) = (z.rect(), z.id);
                    let k = free[(rng() * free.len() as f64).floor() as usize];
                    zones[id] = make_zone(k, &rect, id, rng, cell);
                }
            }
        }
    }
    share_ceilings(&mut zones);
    // a zone id per tile (the outer border belongs to the nearest zone)
    let mut zone_of = vec![0u8; n];
    for z in &zones {
        for zz in z.z0..=z.z1 {
            for xx in z.x0..=z.x1 {
                zone_of[idx(xx, zz)] = z.id as u8;
            }
        }
    }
    for x in 0..w {
        zone_of[idx(x, 0)] = zone_of[idx(x, 1)];
        zone_of[idx(x, h - 1)] = zone_of[idx(x, h - 2)];
    }
    for z in 0..h {
        zone_of[idx(0, z)] = zone_of[idx(1, z)];
        zone_of[idx(w - 1, z)] = zone_of[idx(w - 2, z)];
    }

    // ---- 3. carve each zone ----
    let mut ctx = Ctx { w, h, tiles: vec![1u8; n], tile_flags: vec![0u8; n], rng: &mut *rng, zone: 0, rooms: vec![] };
    for z in &zones {
        let r = z.rect();
        ctx.zone = z.id;
        let rooms_before = ctx.rooms.len();
        match z.arch {
            "store" => carve_store(&mut ctx, r),
            "lobby" => carve_lobby(&mut ctx, r),
            "pillars" => carve_pillars(&mut ctx, r),
            "office" => carve_office(&mut ctx, r),
            "maze" => carve_maze(&mut ctx, r),
            "tunnels" => carve_walkers(&mut ctx, r, 0.36, [3, 7]),
            "electrical" => carve_walkers(&mut ctx, r, 0.42, [2, 5]),
            "hall" => carve_hall(&mut ctx, r),
            "pool" => carve_pool(&mut ctx, r),
            other => panic!("archgen: no carver for {other:?}"),
        }
        if z.arch == "store" {
            for c in store_cells(z) {
                ctx.rooms.push(Room { zone: z.id, x0: c.x0, z0: c.z0, x1: c.x1, z1: c.z1 });
            }
        } else if ctx.rooms.len() == rooms_before {
            ctx.rooms.push(Room { zone: z.id, x0: r.x0, z0: r.z0, x1: r.x1, z1: r.z1 });
        }
    }

    // ---- 4. zone seams: a wall with a few openings between unlike zones ----
    build_seams(&mut ctx, &zones);

    // ---- 5. spawn is open, border is closed ----
    {
        let c = &mut ctx;
        let s = c.idx(1, 1);
        c.tiles[s] = 0;
        c.tile_flags[s] = 0;
        if c.t(2, 1) != 0 && c.t(1, 2) != 0 {
            c.set(2, 1, 0);
        }
        for x in 0..w {
            c.set(x, 0, 1);
            c.set(x, h - 1, 1);
        }
        for z in 0..h {
            c.set(0, z, 1);
            c.set(w - 1, z, 1);
        }
    }

    // ---- 6. everything reachable from spawn ----
    connect_all(&mut ctx);
    // a low partition must never be the only way through: clear any that a
    // repair tunnel or seam opening ran into
    for i in 0..n {
        if ctx.tiles[i] == 0 {
            ctx.tile_flags[i] = 0;
        }
    }

    let Ctx { tiles, tile_flags, rooms, .. } = ctx;
    Layout { w, h, cell, tiles, zone_of, tile_flags, zones, dominant, level, scp: scp.map(|s| s.to_string()), rooms, ground: vec![] }
}

/// Neighbouring zones of the same archetype are one building: they take the
/// ceiling of the first of them, so the slab only steps where the kind of
/// place changes.
fn share_ceilings(zones: &mut [Zone]) {
    let n = zones.len();
    let mut group: Vec<usize> = (0..n).collect();
    fn root(g: &mut [usize], mut i: usize) -> usize {
        while g[i] != i {
            g[i] = g[g[i]];
            i = g[i];
        }
        i
    }
    for a in 0..n {
        for b in a + 1..n {
            if zones[a].arch == zones[b].arch && zones[a].touches(&zones[b]) {
                let (ra, rb) = (root(&mut group, a), root(&mut group, b));
                // the lower id is the root, so a group keeps its first zone's ceiling
                group[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    for i in 0..n {
        let r = root(&mut group, i);
        if r != i {
            let ceil_h = zones[r].ceil_h;
            let z = &mut zones[i];
            z.ceil_h = ceil_h;
            z.lintel_h = nan_min(ceil_h - 0.25, z.lintel_h);
        }
    }
}

fn make_zone(arch: &'static str, r: &Rect, id: usize, rng: &mut dyn FnMut() -> f64, cell: f64) -> Zone {
    let a = arch_of(arch);
    let ceil_h = round_decimals(range_in(rng, a.ceil), 2);
    // draw order: wall_t, pillar, protrude, jamb, lintel_h, low_h
    let wall_t = round_decimals(range_in(rng, a.t), 3);
    let pillar = round_decimals(range_in(rng, a.pillar), 3);
    let protrude = match a.protrude {
        Some(p) => round_decimals(range_in(rng, p), 3),
        None => 0.0,
    };
    let jamb = if rng() < a.jamb_chance { round_decimals(range_in(rng, a.jamb), 3) } else { 0.0 };
    let lintel_h = nan_min(ceil_h - 0.25, round_decimals(range_in(rng, a.lintel), 2));
    let low_h = match a.low_h {
        Some(l) => round_decimals(range_in(rng, l), 2),
        None => 1.5,
    };
    let mut z = Zone {
        id,
        arch,
        x0: r.x0,
        z0: r.z0,
        x1: r.x1,
        z1: r.z1,
        title: a.title,
        lore: a.lore,
        ceil_h,
        wall: a.wall,
        wall_t,
        pillar,
        protrude,
        jamb,
        lintel_h,
        low_h,
        wall_mat: a.wall_mat,
        floor_mat: a.floor_mat,
        ceil_mat: a.ceil_mat,
        light_every: a.light_every,
    };
    // the protrusion must leave a walkable corridor of ≥ 1.6 m in a 1-tile hall
    z.protrude = nan_min(z.protrude, (cell - 1.6) / 2.0);
    // a doorway keeps at least 1.4 m clear
    z.jamb = nan_min(z.jamb, (cell - 1.4) / 2.0);
    z
}

/// One room-set cell of `store_cells` (tiles, inclusive).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreCell {
    pub x0: i32,
    pub x1: i32,
    pub z0: i32,
    pub z1: i32,
}

fn store_lines(z: Rect) -> (Vec<i32>, Vec<i32>) {
    let mut xs = vec![];
    let mut zs = vec![];
    let mut x = z.x0 + 2;
    while x < z.x1 - 1 {
        xs.push(x);
        x += 3;
    }
    let mut zz = z.z0 + 2;
    while zz < z.z1 - 1 {
        zs.push(zz);
        zz += 3;
    }
    (xs, zs)
}
// cells between the lines: [a, b] spans on each axis
fn spans(lines: &[i32], lo: i32, hi: i32) -> Vec<(i32, i32)> {
    let mut out = vec![];
    let mut a = lo;
    for &l in lines {
        if l > a {
            out.push((a, l - 1));
        }
        a = l + 1;
    }
    if hi >= a {
        out.push((a, hi));
    }
    out
}

/// The Showroom's room-set cells in a 'store' zone, as the
/// carver laid them out, between the partitions.
pub fn store_cells(z: &Zone) -> Vec<StoreCell> {
    let (xs, zs) = store_lines(z.rect());
    let mut out = vec![];
    for (a, b) in spans(&xs, z.x0, z.x1) {
        for (c, d) in spans(&zs, z.z0, z.z1) {
            out.push(StoreCell { x0: a, x1: b, z0: c, z1: d });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// carvers. Each works inside its zone rect [x0..x1] × [z0..z1] (inclusive).
// ---------------------------------------------------------------------------
fn fill_rect(c: &mut Ctx, z: Rect, v: u8) {
    for zz in z.z0..=z.z1 {
        for xx in z.x0..=z.x1 {
            c.set(xx, zz, v);
        }
    }
}

struct SplitOpt {
    min_room: i32,
    max_room: i32,
    stop_chance: f64,
    open_frac: [f64; 2],
    doors: bool,
}

// Recursive room splitter: draws wall lines along split lines, then punches
// openings. `open_frac` is the share of each line knocked out (the Lobby's
// randomly segmented rooms), `doors` a fixed count of single-tile doorways
// instead (offices).
fn split_rooms(c: &mut Ctx, rect: Rect, opt: &SplitOpt) {
    let mut stack = vec![rect];
    let mut lines: Vec<Vec<(i32, i32)>> = vec![];
    while let Some(r) = stack.pop() {
        let w = r.x1 - r.x0 + 1;
        let h = r.z1 - r.z0 + 1;
        let can_x = w > opt.min_room * 2;
        let can_z = h > opt.min_room * 2;
        let room = Room { zone: c.zone, x0: r.x0, z0: r.z0, x1: r.x1, z1: r.z1 };
        if !can_x && !can_z {
            c.rooms.push(room);
            continue;
        }
        if w <= opt.max_room && h <= opt.max_room && c.r() < opt.stop_chance {
            c.rooms.push(room);
            continue;
        }
        let axis_x = if can_x && can_z {
            if w > h {
                true
            } else if h > w {
                false
            } else {
                c.r() < 0.5
            }
        } else {
            can_x
        };
        if axis_x {
            let s = c.irange(r.x0 + opt.min_room, r.x1 - opt.min_room);
            lines.push((r.z0..=r.z1).map(|zz| (s, zz)).collect());
            stack.push(Rect { x0: r.x0, z0: r.z0, x1: s - 1, z1: r.z1 });
            stack.push(Rect { x0: s + 1, z0: r.z0, x1: r.x1, z1: r.z1 });
        } else {
            let s = c.irange(r.z0 + opt.min_room, r.z1 - opt.min_room);
            lines.push((r.x0..=r.x1).map(|xx| (xx, s)).collect());
            stack.push(Rect { x0: r.x0, z0: r.z0, x1: r.x1, z1: s - 1 });
            stack.push(Rect { x0: r.x0, z0: s + 1, x1: r.x1, z1: r.z1 });
        }
    }
    for line in &lines {
        for &(x, z) in line {
            c.set(x, z, 1);
        }
        let l = line.len() as i32;
        if opt.doors {
            let n = if l > 7 { 2 } else { 1 };
            for _ in 0..n {
                let k = c.irange(1, 1.max(l - 2));
                let (x, z) = line[k as usize];
                c.set(x, z, 0);
            }
        } else {
            // alternate wall runs and gaps until roughly open_frac of it is open
            let t = c.r();
            let target = opt.open_frac[0] + (opt.open_frac[1] - opt.open_frac[0]) * t;
            let mut k = c.irange(0, 2);
            let mut opened = 0i32;
            while k < l {
                let run = c.irange(1, 4);
                k += run;
                if k >= l {
                    break;
                }
                let gap = if c.r() < 0.6 { 1 } else { c.irange(2, 3) };
                if (opened as f64) / (l as f64) < target {
                    let mut g = 0;
                    while g < gap && k + g < l {
                        let (x, z) = line[(k + g) as usize];
                        c.set(x, z, 0);
                        opened += 1;
                        g += 1;
                    }
                }
                k += gap;
            }
            if opened == 0 {
                let (x, z) = line[c.irange(0, l - 1) as usize];
                c.set(x, z, 0);
            }
        }
    }
}

// lone pillars in the middle of open floor
fn scatter_pillars(c: &mut Ctx, z: Rect, chance: f64) {
    let (w, h) = (c.w, c.h);
    for zz in (z.z0 + 1)..z.z1 {
        for xx in (z.x0 + 1)..z.x1 {
            if c.t(xx, zz) != 0 {
                continue;
            }
            let mut clear = true;
            let mut dz = -1;
            while dz <= 1 && clear {
                for dx in -1..=1 {
                    let nx = xx + dx;
                    let nz = zz + dz;
                    if nx < 0 || nz < 0 || nx >= w || nz >= h || c.t(nx, nz) != 0 {
                        clear = false;
                        break;
                    }
                }
                dz += 1;
            }
            if clear && c.r() < chance {
                c.set(xx, zz, 1);
            }
        }
    }
}

// the Showroom: a grid of 2×2-tile room-sets behind showroom partitions,
// joined by a spanning tree of openings (the one winding path a store
// wants you to take) plus a few shortcuts, and here and there two sets
// merged into a bigger hall
fn carve_store(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    let (xs, zs) = store_lines(z);
    if xs.is_empty() && zs.is_empty() {
        return;
    }
    let wall = |c: &mut Ctx, x: i32, zz: i32| {
        let i = c.idx(x, zz);
        c.tiles[i] = 1;
        c.tile_flags[i] |= TF_LOW;
    };
    for &x in &xs {
        for zz in z.z0..=z.z1 {
            wall(c, x, zz);
        }
    }
    for &zz in &zs {
        for x in z.x0..=z.x1 {
            wall(c, x, zz);
        }
    }
    let cx = spans(&xs, z.x0, z.x1);
    let cz = spans(&zs, z.z0, z.z1);
    let nx = cx.len() as i32;
    let nz = cz.len() as i32;
    let open = |c: &mut Ctx, x: i32, zz: i32| {
        let i = c.idx(x, zz);
        c.tiles[i] = 0;
        c.tile_flags[i] = 0;
    };
    // the opening between cell (i,j) and its neighbour in direction d
    let gap = |c: &mut Ctx, i: i32, j: i32, di: i32, dj: i32, all: bool| {
        if di != 0 {
            let x = xs[i.min(i + di) as usize];
            let (a, b) = cz[j as usize];
            if all {
                for zz in a..=b {
                    open(c, x, zz);
                }
            } else {
                let zz = c.irange(a, b);
                open(c, x, zz);
            }
        } else {
            let zz = zs[j.min(j + dj) as usize];
            let (a, b) = cx[i as usize];
            if all {
                for x in a..=b {
                    open(c, x, zz);
                }
            } else {
                let x = c.irange(a, b);
                open(c, x, zz);
            }
        }
    };
    let mut seen = vec![0u8; (nx * nz) as usize];
    let s0 = c.irange(0, nx - 1);
    let s1 = c.irange(0, nz - 1);
    let mut st: Vec<(i32, i32)> = vec![(s0, s1)];
    seen[(s1 * nx + s0) as usize] = 1;
    while let Some(&(i, j)) = st.last() {
        let opts: Vec<[i32; 2]> = DIRS
            .iter()
            .copied()
            .filter(|&[di, dj]| i + di >= 0 && j + dj >= 0 && i + di < nx && j + dj < nz && seen[((j + dj) * nx + i + di) as usize] == 0)
            .collect();
        if opts.is_empty() {
            st.pop();
            continue;
        }
        let [di, dj] = opts[(c.r() * opts.len() as f64).floor() as usize];
        let all = c.r() < 0.05;
        gap(c, i, j, di, dj, all);
        seen[((j + dj) * nx + i + di) as usize] = 1;
        st.push((i + di, j + dj));
    }
    // shortcuts, so the path is a store's path and not a perfect maze
    for j in 0..nz {
        for i in 0..nx {
            if i + 1 < nx && c.r() < 0.07 {
                gap(c, i, j, 1, 0, false);
            }
            if j + 1 < nz && c.r() < 0.07 {
                gap(c, i, j, 0, 1, false);
            }
        }
    }
}

// Level 0: open floor cut into irregular rooms by partitions full of gaps
fn carve_lobby(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    split_rooms(c, z, &SplitOpt { min_room: 2, max_room: 6, stop_chance: 0.25, open_frac: [0.3, 0.55], doors: false });
    scatter_pillars(c, z, 0.06);
}

// Level 0 pillar grid: huge rooms, columns on a lattice, a few stray walls
fn carve_pillars(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    let k = if c.r() < 0.6 { 2 } else { 3 };
    let ox = c.irange(0, k - 1);
    let oz = c.irange(0, k - 1);
    for zz in (z.z0 + 1)..z.z1 {
        for xx in (z.x0 + 1)..z.x1 {
            if (xx - z.x0) % k == ox && (zz - z.z0) % k == oz && c.r() < 0.9 {
                c.set(xx, zz, 1);
            }
        }
    }
}

// Level 4: rooms along partitions with a doorway each, cubicle rows inside
fn carve_office(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    split_rooms(c, z, &SplitOpt { min_room: 2, max_room: 4, stop_chance: 0.15, open_frac: [0.0, 0.0], doors: true });
    // cubicle farms: short low partitions in some of the bigger open patches
    let mut zz = z.z0 + 1;
    while zz < z.z1 {
        let mut xx = z.x0 + 1;
        while xx < z.x1 - 1 {
            if c.r() > 0.18 {
                xx += 1;
                continue;
            }
            let run = c.irange(1, 2);
            let mut ok = true;
            let mut r = -1;
            while r <= run && ok {
                for dz in [-1, 0, 1] {
                    if xx + r < z.x0 || xx + r > z.x1 || c.t(xx + r, zz + dz) != 0 {
                        ok = false;
                        break;
                    }
                }
                r += 1;
            }
            if !ok {
                xx += 1;
                continue;
            }
            for r in 0..run {
                let i = c.idx(xx + r, zz);
                c.tiles[i] = 1;
                c.tile_flags[i] |= TF_LOW;
            }
            xx += run + 1;
            xx += 1;
        }
        zz += 2;
    }
}

// the classic generator: a braided DFS maze with thick walls
fn carve_maze(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 1);
    let nx = (z.x1 - z.x0).div_euclid(2);
    let nz = (z.z1 - z.z0).div_euclid(2);
    if nx < 1 || nz < 1 {
        fill_rect(c, z, 0);
        return;
    }
    let node = |i: i32, j: i32| (z.x0 + 1 + i * 2, z.z0 + 1 + j * 2);
    let mut seen = vec![0u8; (nx * nz) as usize];
    let s0 = c.irange(0, nx - 1);
    let s1 = c.irange(0, nz - 1);
    let mut st: Vec<(i32, i32)> = vec![(s0, s1)];
    seen[(s1 * nx + s0) as usize] = 1;
    {
        let (x, zz) = node(s0, s1);
        c.set(x, zz, 0);
    }
    while let Some(&(i, j)) = st.last() {
        let mut opts: Vec<(i32, i32, i32, i32)> = vec![];
        for [di, dj] in DIRS {
            let ni = i + di;
            let nj = j + dj;
            if ni >= 0 && nj >= 0 && ni < nx && nj < nz && seen[(nj * nx + ni) as usize] == 0 {
                opts.push((ni, nj, di, dj));
            }
        }
        if opts.is_empty() {
            st.pop();
            continue;
        }
        let (ni, nj, di, dj) = opts[(c.r() * opts.len() as f64).floor() as usize];
        seen[(nj * nx + ni) as usize] = 1;
        let (ax, az) = node(i, j);
        c.set(ax + di, az + dj, 0);
        let (bx, bz) = node(ni, nj);
        c.set(bx, bz, 0);
        st.push((ni, nj));
    }
    // braid: knock out ~28% of the walls between two corridors
    for zz in (z.z0 + 1)..z.z1 {
        for xx in (z.x0 + 1)..z.x1 {
            if c.t(xx, zz) == 0 {
                continue;
            }
            let h_open = c.t(xx - 1, zz) == 0 && c.t(xx + 1, zz) == 0;
            let v_open = c.t(xx, zz - 1) == 0 && c.t(xx, zz + 1) == 0;
            if h_open != v_open && c.r() < 0.28 {
                c.set(xx, zz, 0);
            }
        }
    }
}

struct Walker {
    x: i32,
    z: i32,
    d: usize,
    left: i32,
}

// Level 2 (tunnels): long narrow service corridors in solid rock, right-angle
// turns. Level 3 (electrical): the same bones, tighter and more tangled.
fn carve_walkers(c: &mut Ctx, z: Rect, target_frac: f64, run_len: [i32; 2]) {
    fill_rect(c, z, 1);
    let inside = |x: i32, zz: i32| x > z.x0 && zz > z.z0 && x < z.x1 && zz < z.z1;
    let area = (z.x1 - z.x0 - 1) * (z.z1 - z.z0 - 1);
    let mut carved = 0i32;
    let mut walkers: Vec<Walker> = vec![];
    let spawn_walker = |c: &mut Ctx, walkers: &mut Vec<Walker>| {
        let x = c.irange(z.x0 + 1, z.x1 - 1);
        let zz = c.irange(z.z0 + 1, z.z1 - 1);
        let d = (c.r() * 4.0).floor() as usize;
        let left = c.irange(run_len[0], run_len[1]);
        walkers.push(Walker { x, z: zz, d, left });
    };
    spawn_walker(c, &mut walkers);
    spawn_walker(c, &mut walkers);
    let mut guard = area * 8;
    const LEFT: [usize; 4] = [2, 3, 1, 0];
    const RIGHT: [usize; 4] = [3, 2, 0, 1];
    while (carved as f64) < area as f64 * target_frac && {
        let g = guard;
        guard -= 1;
        g > 0
    } {
        if walkers.is_empty() {
            spawn_walker(c, &mut walkers);
        }
        let wi = (c.r() * walkers.len() as f64).floor() as usize;
        {
            let (wx, wz) = (walkers[wi].x, walkers[wi].z);
            if c.t(wx, wz) != 0 {
                c.set(wx, wz, 0);
                carved += 1;
            }
        }
        let old_left = walkers[wi].left;
        walkers[wi].left -= 1;
        if old_left <= 0 {
            let d = walkers[wi].d;
            walkers[wi].d = if c.r() < 0.5 { LEFT[d] } else { RIGHT[d] }; // turn left / right
            walkers[wi].left = c.irange(run_len[0], run_len[1]);
            if c.r() < 0.3 && walkers.len() < 6 {
                let (x, zz, d) = (walkers[wi].x, walkers[wi].z, LEFT[walkers[wi].d]);
                let left = c.irange(run_len[0], run_len[1]);
                walkers.push(Walker { x, z: zz, d, left });
            }
        }
        let [dx, dz] = DIRS[walkers[wi].d];
        // corridors never run side by side: refuse a step that would touch a
        // parallel corridor (keeps walls between them, and walls solid)
        let nx = walkers[wi].x + dx;
        let nz = walkers[wi].z + dz;
        if !inside(nx, nz) {
            walkers[wi].left = 0;
            continue;
        }
        let sx = if dz != 0 { 1 } else { 0 };
        let sz = if dx != 0 { 1 } else { 0 };
        let beside = (c.t(nx + sx, nz + sz) == 0) as i32 + (c.t(nx - sx, nz - sz) == 0) as i32;
        let ahead = if inside(nx + dx, nz + dz) { (c.t(nx + dx, nz + dz) == 0) as i32 } else { 0 };
        if c.t(nx, nz) != 0 && beside != 0 && ahead == 0 && c.r() < 0.85 {
            walkers[wi].left = 0;
            continue;
        }
        walkers[wi].x = nx;
        walkers[wi].z = nz;
        if c.r() < 0.004 {
            walkers.remove(wi);
        }
    }
}

// Level 1: a concrete hall with columns on a wide grid
fn carve_hall(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    let k = 3;
    let ox = c.irange(1, 2);
    let oz = c.irange(1, 2);
    for zz in (z.z0 + 1)..z.z1 {
        for xx in (z.x0 + 1)..z.x1 {
            if (xx - z.x0) % k == ox % k && (zz - z.z0) % k == oz % k {
                c.set(xx, zz, 1);
            }
        }
    }
    // a couple of long partial walls so the hall is not a plain grid
    if z.x1 - z.x0 > 8 && c.r() < 0.7 {
        let s = c.irange(z.z0 + 2, z.z1 - 2);
        for xx in (z.x0 + 1)..z.x1 {
            if c.r() < 0.7 {
                c.set(xx, s, 1);
            }
        }
    }
}

// Level 37: tiled halls, too many pillars
fn carve_pool(c: &mut Ctx, z: Rect) {
    fill_rect(c, z, 0);
    for zz in (z.z0 + 1)..z.z1 {
        for xx in (z.x0 + 1)..z.x1 {
            if (xx - z.x0) % 2 == 1 && (zz - z.z0) % 2 == 1 && c.r() < 0.75 {
                c.set(xx, zz, 1);
            }
        }
    }
    split_rooms(c, z, &SplitOpt { min_room: 3, max_room: 8, stop_chance: 0.5, open_frac: [0.4, 0.6], doors: false });
}

// Zones of a different kind are separated by a wall with 1-3 openings.
fn build_seams(c: &mut Ctx, zones: &[Zone]) {
    let (w, h) = (c.w, c.h);
    let openish = |a: &str| matches!(a, "lobby" | "pillars" | "office" | "hall" | "pool");
    for ai in 0..zones.len() {
        for bi in 0..zones.len() {
            if ai == bi {
                continue;
            }
            let (a, b) = (&zones[ai], &zones[bi]);
            // b directly east of a / directly south of a
            let mut line: Option<Vec<(i32, i32)>> = None;
            if b.x0 == a.x1 + 1 {
                let z0 = a.z0.max(b.z0);
                let z1 = a.z1.min(b.z1);
                if z1 - z0 >= 1 {
                    line = Some((z0..=z1).map(|z| (a.x1, z)).collect());
                }
            } else if b.z0 == a.z1 + 1 {
                let x0 = a.x0.max(b.x0);
                let x1 = a.x1.min(b.x1);
                if x1 - x0 >= 1 {
                    line = Some((x0..=x1).map(|x| (x, a.z1)).collect());
                }
            }
            let Some(line) = line else { continue };
            let both_open = openish(a.arch) && openish(b.arch);
            if !both_open {
                continue;
            } // mazes/tunnels already have walls
            if a.arch == b.arch && c.r() < 0.6 {
                continue;
            }
            for &(x, z) in &line {
                if x > 0 && z > 0 && x < w - 1 && z < h - 1 {
                    let i = c.idx(x, z);
                    c.tiles[i] = 1;
                    c.tile_flags[i] = 0;
                }
            }
            let l = line.len() as i32;
            let n = if l > 10 { c.irange(2, 3) } else { c.irange(1, 2) };
            for _ in 0..n {
                let ww = if c.r() < 0.5 { 1 } else { 2 };
                let s = c.irange(1, 1.max(l - 1 - ww));
                for g in 0..ww {
                    if let Some(&(px, pz)) = line.get((s + g) as usize) {
                        c.set(px, pz, 0);
                    }
                }
            }
        }
    }
}

// Label open components; tunnel from each stray one to the spawn component
// along the cheapest path (walls cost 1, open floor 0).
fn connect_all(c: &mut Ctx) {
    let (w, h) = (c.w, c.h);
    let n = (w * h) as usize;
    for _pass in 0..64 {
        let mut comp = vec![-1i32; n];
        let mut n_comp = 0i32;
        for i in 0..n {
            if c.tiles[i] != 0 || comp[i] >= 0 {
                continue;
            }
            let mut q = vec![i];
            comp[i] = n_comp;
            let mut qi = 0;
            while qi < q.len() {
                let cc = q[qi] as i32;
                let (cx, cz) = (cc % w, cc / w);
                for [dx, dz] in DIRS {
                    let nx = cx + dx;
                    let nz = cz + dz;
                    if nx < 0 || nz < 0 || nx >= w || nz >= h {
                        continue;
                    }
                    let ni = c.idx(nx, nz);
                    if c.tiles[ni] == 0 && comp[ni] < 0 {
                        comp[ni] = n_comp;
                        q.push(ni);
                    }
                }
                qi += 1;
            }
            n_comp += 1;
        }
        let main = comp[c.idx(1, 1)];
        if n_comp <= 1 {
            return;
        }
        // pick any stray component and 0-1 BFS from it to the main one:
        // layered by cost, open floor expands within a layer, walls cost one more
        let mut stray = -1;
        for cc in 0..n_comp {
            if cc != main {
                stray = cc;
                break;
            }
        }
        let mut dist = vec![1i32 << 30; n];
        let mut prev = vec![-1i32; n];
        let mut cur: Vec<usize> = vec![];
        for i in 0..n {
            if comp[i] == stray {
                dist[i] = 0;
                cur.push(i);
            }
        }
        let mut level = 0i32;
        let mut hit: i64 = -1;
        while !cur.is_empty() && hit < 0 {
            let mut next: Vec<usize> = vec![];
            let mut qi = 0;
            while qi < cur.len() {
                let cc = cur[qi];
                qi += 1;
                if dist[cc] != level {
                    continue;
                }
                if comp[cc] == main {
                    hit = cc as i64;
                    break;
                }
                let (cx, cz) = (cc as i32 % w, cc as i32 / w);
                for [dx, dz] in DIRS {
                    let nx = cx + dx;
                    let nz = cz + dz;
                    if nx < 1 || nz < 1 || nx >= w - 1 || nz >= h - 1 {
                        continue;
                    }
                    let ni = c.idx(nx, nz);
                    let wgt = if c.tiles[ni] != 0 { 1 } else { 0 };
                    if level + wgt < dist[ni] {
                        dist[ni] = level + wgt;
                        prev[ni] = cc as i32;
                        if wgt != 0 {
                            next.push(ni);
                        } else {
                            cur.push(ni);
                        }
                    }
                }
            }
            cur = next;
            level += 1;
        }
        if hit < 0 {
            // unreachable pocket (should not happen inside the border) — wall it up
            for i in 0..n {
                if comp[i] == stray {
                    c.tiles[i] = 1;
                }
            }
            continue;
        }
        let mut cc = hit;
        while cc >= 0 && comp[cc as usize] != stray {
            c.tiles[cc as usize] = 0;
            cc = prev[cc as usize] as i64;
        }
    }
}

// ---------------------------------------------------------------------------
// solids
// ---------------------------------------------------------------------------
/// Options of [`build_solids`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SolidsOpts<'a> {
    /// faces pinned to the tile boundary: key = wall_tile * 4 + dir, dir = the
    /// side of the wall tile the face is on (DIRS index)
    pub anchors: Option<&'a HashSet<usize>>,
    /// per-tile ceiling height (metres); defaults to the zone's
    pub ceil: Option<&'a [f32]>,
    /// used only when the layout has no cell (0 = 4)
    pub cell: f64,
}

/// 8 floats per box `x0, z0, x1, z1, y0, y1, owner_tile, kind`, plus
/// a CSR index per tile (`start` offsets into `items`, the box ids
/// overlapping each tile). `stamp`/`tick` are `segment_clear`'s scratch.
#[derive(Clone, Debug)]
pub struct Solids {
    pub w: i32,
    pub h: i32,
    pub cell: f64,
    pub boxes: Vec<f32>,
    pub count: usize,
    pub start: Vec<i32>,
    pub items: Vec<i32>,
    pub stamp: RefCell<Vec<u32>>,
    pub tick: Cell<u32>,
}
impl Solids {
    /// The 8 floats of box `b`.
    #[inline]
    pub fn box_at(&self, b: usize) -> &[f32] {
        &self.boxes[b * 8..b * 8 + 8]
    }
    /// The kind (`BOX_*`) of box `b`.
    #[inline]
    pub fn kind(&self, b: usize) -> i32 {
        self.boxes[b * 8 + 7] as i32
    }
    /// The box ids overlapping tile `t`.
    #[inline]
    pub fn tile_items(&self, t: usize) -> &[i32] {
        &self.items[self.start[t] as usize..self.start[t + 1] as usize]
    }
    /// Advances `tick` (on u32 wrap the stamps are cleared, so no stale stamp can match)
    fn next_tick(&self) -> u32 {
        let mut t = self.tick.get().wrapping_add(1);
        if t == 0 {
            self.stamp.borrow_mut().iter_mut().for_each(|s| *s = 0);
            t = 1;
        }
        self.tick.set(t);
        t
    }
    /// Tile index of in-grid (or NaN) f64 tile coordinates; NaN → None, and
    /// the caller skips the tile
    #[inline]
    pub fn tile_of(&self, x: f64, z: f64) -> Option<usize> {
        if x.is_nan() || z.is_nan() {
            None
        } else {
            Some((z * self.w as f64 + x) as usize)
        }
    }
}

/// The solid boxes of every wall tile.
pub fn build_solids(l: &Layout, opt: &SolidsOpts) -> Solids {
    build_solids_view(&l.view(), opt)
}

/// `build_solids` over any tile grid + zones (see [`LayoutView`]).
pub fn build_solids_view(l: &LayoutView, opt: &SolidsOpts) -> Solids {
    let (w, h) = (l.w, l.h);
    let tiles = l.tiles;
    let zone_of = l.zone_of;
    let zones = l.zones;
    let zero_flags;
    let tile_flags: &[u8] = match l.tile_flags {
        Some(f) => f,
        None => {
            zero_flags = vec![0u8; (w * h) as usize];
            &zero_flags
        }
    };
    let ok = |c: f64| c != 0.0 && !c.is_nan();
    let cell = if ok(l.cell) {
        l.cell
    } else if ok(opt.cell) {
        opt.cell
    } else {
        4.0
    };
    let anchors = opt.anchors;
    let ceil = opt.ceil;
    let mut out: Vec<f32> = vec![];
    let idx = |x: i32, z: i32| (z * w + x) as usize;
    let is_wall = |x: i32, z: i32| x < 0 || z < 0 || x >= w || z >= h || tiles[idx(x, z)] == 1;
    let zone_at = |x: i32, z: i32| &zones[zone_of[idx(iclamp(x, 0, w - 1), iclamp(z, 0, h - 1))] as usize];
    // box values are computed in f64, then each is rounded to the nearest f32
    let push = |out: &mut Vec<f32>, b: [f64; 8]| {
        for v in b {
            out.push(v as f32);
        }
    };

    for z in 0..h {
        for x in 0..w {
            if !is_wall(x, z) {
                continue;
            }
            let t = idx(x, z);
            let zn = zone_at(x, z);
            let low = (tile_flags[t] & TF_LOW) != 0;
            let top = if low {
                zn.low_h
            } else if let Some(c) = ceil {
                c[t] as f64
            } else {
                zn.ceil_h
            };
            let border = x == 0 || z == 0 || x == w - 1 || z == h - 1;
            let nb = [is_wall(x + 1, z), is_wall(x - 1, z), is_wall(x, z + 1), is_wall(x, z - 1)];
            let enclosed = nb[0] && nb[1] && nb[2] && nb[3];
            let a = if zn.wall == "thin" { clamp(zn.wall_t / cell / 2.0, 0.01, 0.5) } else { 0.5 };
            let (xf, zf, tf) = (x as f64, z as f64, t as f64);
            let cx = xf + 0.5;
            let cz = zf + 0.5;
            if border || enclosed || a >= 0.5 || zn.wall != "thin" {
                let (mut x0, mut z0, mut x1, mut z1) = (xf, zf, xf + 1.0, zf + 1.0);
                if zn.wall == "thick" && !border && zn.protrude > 0.0 {
                    let p = zn.protrude / cell;
                    let same = |nx: i32, nz: i32| !is_wall(nx, nz) && zone_of[idx(nx, nz)] == zone_of[t];
                    if !nb[0] && same(x + 1, z) {
                        x1 += p;
                    }
                    if !nb[1] && same(x - 1, z) {
                        x0 -= p;
                    }
                    if !nb[2] && same(x, z + 1) {
                        z1 += p;
                    }
                    if !nb[3] && same(x, z - 1) {
                        z0 -= p;
                    }
                }
                push(&mut out, [x0, z0, x1, z1, 0.0, top, tf, BOX_WALL as f64]);
                continue;
            }
            let has_x = nb[0] || nb[1];
            let has_z = nb[2] || nb[3];
            if !has_x && !has_z {
                let p = clamp(zn.pillar / cell / 2.0, 0.02, 0.5);
                push(&mut out, [cx - p, cz - p, cx + p, cz + p, 0.0, top, tf, BOX_PILLAR as f64]);
            } else {
                // an end of a run stays flush with its tile edge so a one-tile gap is
                // a clean 4 m opening; corners and tees meet at the centre
                if has_x {
                    let x0 = if nb[1] {
                        xf
                    } else if has_z {
                        cx - a
                    } else {
                        xf
                    };
                    let x1 = if nb[0] {
                        xf + 1.0
                    } else if has_z {
                        cx + a
                    } else {
                        xf + 1.0
                    };
                    push(&mut out, [x0, cz - a, x1, cz + a, 0.0, top, tf, BOX_WALL as f64]);
                }
                if has_z {
                    let z0 = if nb[3] {
                        zf
                    } else if has_x {
                        cz - a
                    } else {
                        zf
                    };
                    let z1 = if nb[2] {
                        zf + 1.0
                    } else if has_x {
                        cz + a
                    } else {
                        zf + 1.0
                    };
                    push(&mut out, [cx - a, z0, cx + a, z1, 0.0, top, tf, BOX_WALL as f64]);
                }
            }
            // pinned faces: fill from the partition out to the tile boundary
            if let Some(anchors) = anchors {
                for d in 0..4 {
                    if !anchors.contains(&(t * 4 + d)) || nb[d] {
                        continue;
                    }
                    let k = BOX_ANCHOR as f64;
                    match d {
                        0 => push(&mut out, [cx - a, zf, xf + 1.0, zf + 1.0, 0.0, top, tf, k]),
                        1 => push(&mut out, [xf, zf, cx + a, zf + 1.0, 0.0, top, tf, k]),
                        2 => push(&mut out, [xf, cz - a, xf + 1.0, zf + 1.0, 0.0, top, tf, k]),
                        _ => push(&mut out, [xf, zf, xf + 1.0, cz + a, 0.0, top, tf, k]),
                    }
                }
            }
        }
    }

    // doorway jambs + lintels: a single open tile between two thin wall ends
    for z in 1..h - 1 {
        for x in 1..w - 1 {
            if is_wall(x, z) {
                continue;
            }
            let zn = zone_at(x, z);
            if !(zn.jamb > 0.0) || zn.wall != "thin" {
                continue;
            }
            let t = idx(x, z);
            let (xf, zf, tf) = (x as f64, z as f64, t as f64);
            for axis_x in [true, false] {
                let (ax, az) = if axis_x { (1, 0) } else { (0, 1) };
                let (pa, pb) = ((x - ax, z - az), (x + ax, z + az));
                if !is_wall(pa.0, pa.1) || !is_wall(pb.0, pb.1) {
                    continue;
                }
                if is_wall(x + az, z + ax) || is_wall(x - az, z - ax) {
                    continue;
                } // must be a pass-through
                let za = zone_at(pa.0, pa.1);
                let zb = zone_at(pb.0, pb.1);
                if za.wall != "thin" || zb.wall != "thin" {
                    continue;
                }
                if ((tile_flags[idx(pa.0, pa.1)] | tile_flags[idx(pb.0, pb.1)]) & TF_LOW) != 0 {
                    continue;
                }
                // skip where either side is a pillar / corner (they don't run along this axis)
                let runs = |px: i32, pz: i32| is_wall(px - ax, pz - az) || is_wall(px + ax, pz + az);
                if !runs(pa.0, pa.1) || !runs(pb.0, pb.1) {
                    continue;
                }
                let j = zn.jamb / cell;
                let a = clamp(nan_min(za.wall_t, zb.wall_t) / cell / 2.0, 0.01, 0.5);
                let top = if let Some(c) = ceil { c[t] as f64 } else { zn.ceil_h };
                let (kj, kl) = (BOX_JAMB as f64, BOX_LINTEL as f64);
                if axis_x {
                    let cz = zf + 0.5;
                    push(&mut out, [xf, cz - a, xf + j, cz + a, 0.0, top, tf, kj]);
                    push(&mut out, [xf + 1.0 - j, cz - a, xf + 1.0, cz + a, 0.0, top, tf, kj]);
                    if zn.lintel_h < top - 0.05 {
                        push(&mut out, [xf + j, cz - a, xf + 1.0 - j, cz + a, zn.lintel_h, top, tf, kl]);
                    }
                } else {
                    let cx = xf + 0.5;
                    push(&mut out, [cx - a, zf, cx + a, zf + j, 0.0, top, tf, kj]);
                    push(&mut out, [cx - a, zf + 1.0 - j, cx + a, zf + 1.0, 0.0, top, tf, kj]);
                    if zn.lintel_h < top - 0.05 {
                        push(&mut out, [cx - a, zf + j, cx + a, zf + 1.0 - j, zn.lintel_h, top, tf, kl]);
                    }
                }
            }
        }
    }

    make_solids(w, h, cell, out)
}

/// Solids from their boxes (8 floats each), indexed by tile
pub fn make_solids(w: i32, h: i32, cell: f64, boxes: Vec<f32>) -> Solids {
    let count = boxes.len() / 8;
    let n = (w * h) as usize;
    let mut counts = vec![0i32; n + 1];
    let span = |b: usize| -> (i32, i32, i32, i32) {
        let o = b * 8;
        (
            nan_max(0.0, (boxes[o] as f64 + 1e-6).floor()) as i32,
            nan_max(0.0, (boxes[o + 1] as f64 + 1e-6).floor()) as i32,
            nan_min((w - 1) as f64, (boxes[o + 2] as f64 - 1e-6).ceil() - 1.0) as i32,
            nan_min((h - 1) as f64, (boxes[o + 3] as f64 - 1e-6).ceil() - 1.0) as i32,
        )
    };
    for b in 0..count {
        let (x0, z0, x1, z1) = span(b);
        for z in z0..=z1 {
            for x in x0..=x1 {
                counts[(z * w + x + 1) as usize] += 1;
            }
        }
    }
    for i in 1..=n {
        counts[i] += counts[i - 1];
    }
    let start = counts.clone();
    let mut fill = counts[..n].to_vec();
    let mut items = vec![0i32; start[n] as usize];
    for b in 0..count {
        let (x0, z0, x1, z1) = span(b);
        for z in z0..=z1 {
            for x in x0..=x1 {
                let f = &mut fill[(z * w + x) as usize];
                items[*f as usize] = b as i32;
                *f += 1;
            }
        }
    }
    Solids { w, h, cell, boxes, count, start, items, stamp: RefCell::new(vec![0u32; count]), tick: Cell::new(1) }
}

// ---------------------------------------------------------------------------
// queries (all in tile units). Tile loops run on f64 coordinates.
// ---------------------------------------------------------------------------
/// Does a circle at (u,v) of radius r
/// overlap a box that reaches below `y_max`? Outside the map is solid.
pub fn circle_hits(s: &Solids, u: f64, v: f64, r: f64, y_max: f64) -> bool {
    let (w, h) = (s.w as f64, s.h as f64);
    let b = &s.boxes;
    let x0 = (u - r).floor();
    let x1 = (u + r).floor();
    let z0 = (v - r).floor();
    let z1 = (v + r).floor();
    let r2 = r * r;
    let mut z = z0;
    while z <= z1 {
        let mut x = x0;
        while x <= x1 {
            if x < 0.0 || z < 0.0 || x >= w || z >= h {
                return true;
            } // outside the map is solid
            if let Some(t) = s.tile_of(x, z) {
                for k in s.start[t]..s.start[t + 1] {
                    let o = s.items[k as usize] as usize * 8;
                    if b[o + 4] as f64 >= y_max || b[o + 7] as i32 == BOX_FLOOR {
                        continue;
                    } // lintel overhead; the floor you stand on
                    let (bx0, bz0, bx1, bz1) = (b[o] as f64, b[o + 1] as f64, b[o + 2] as f64, b[o + 3] as f64);
                    let cx = if u < bx0 {
                        bx0
                    } else if u > bx1 {
                        bx1
                    } else {
                        u
                    };
                    let cz = if v < bz0 {
                        bz0
                    } else if v > bz1 {
                        bz1
                    } else {
                        v
                    };
                    let dx = u - cx;
                    let dz = v - cz;
                    if dx * dx + dz * dz < r2 {
                        return true;
                    }
                }
            }
            x += 1.0;
        }
        z += 1.0;
    }
    false
}

/// The id of a box containing the point and
/// spanning height y, -1 for none, -2 outside the map.
pub fn point_in_solid(s: &Solids, u: f64, v: f64, y: f64) -> i32 {
    let x = u.floor();
    let z = v.floor();
    if x < 0.0 || z < 0.0 || x >= s.w as f64 || z >= s.h as f64 {
        return -2;
    }
    let Some(t) = s.tile_of(x, z) else { return -1 };
    let b = &s.boxes;
    for k in s.start[t]..s.start[t + 1] {
        let bi = s.items[k as usize];
        let o = bi as usize * 8;
        if u > b[o] as f64
            && u < b[o + 2] as f64
            && v > b[o + 1] as f64
            && v < b[o + 3] as f64
            && y >= b[o + 4] as f64
            && y <= b[o + 5] as f64
        {
            return bi;
        }
    }
    -1
}

/// Line of sight at height y —
/// true when the segment is clear of every box that spans y. Tiles are
/// walked with a DDA and each box is tested once.
pub fn segment_clear(s: &Solids, u0: f64, v0: f64, u1: f64, v1: f64, y: f64) -> bool {
    let tick = s.next_tick();
    let mut stamp = s.stamp.borrow_mut();
    let (w, h) = (s.w as f64, s.h as f64);
    let b = &s.boxes;
    let du = u1 - u0;
    let dv = v1 - v0;
    let mut x = u0.floor();
    let mut z = v0.floor();
    let xe = u1.floor();
    let ze = v1.floor();
    let sx = if du > 0.0 { 1.0 } else { -1.0 };
    let sz = if dv > 0.0 { 1.0 } else { -1.0 };
    let tdx = if du != 0.0 { (1.0 / du).abs() } else { f64::INFINITY };
    let tdz = if dv != 0.0 { (1.0 / dv).abs() } else { f64::INFINITY };
    let mut tmx = if du != 0.0 { (if sx > 0.0 { x + 1.0 - u0 } else { u0 - x }) * tdx } else { f64::INFINITY };
    let mut tmz = if dv != 0.0 { (if sz > 0.0 { z + 1.0 - v0 } else { v0 - z }) * tdz } else { f64::INFINITY };
    for _guard in 0..4 * (s.w + s.h) {
        if x < 0.0 || z < 0.0 || x >= w || z >= h {
            return false;
        }
        if let Some(t) = s.tile_of(x, z) {
            for k in s.start[t]..s.start[t + 1] {
                let bi = s.items[k as usize] as usize;
                if stamp[bi] == tick {
                    continue;
                }
                stamp[bi] = tick;
                let o = bi * 8;
                if y < b[o + 4] as f64 || y > b[o + 5] as f64 || b[o + 7] as i32 == BOX_VOID {
                    continue;
                }
                if seg_box(u0, v0, du, dv, b[o] as f64, b[o + 1] as f64, b[o + 2] as f64, b[o + 3] as f64) {
                    return false;
                }
            }
        }
        if x == xe && z == ze {
            return true;
        }
        if tmx < tmz {
            tmx += tdx;
            x += sx;
        } else {
            tmz += tdz;
            z += sz;
        }
        if tmx > 1.0 + tdx && tmz > 1.0 + tdz {
            return true;
        }
    }
    true
}

/// Line of sight between two points at their own heights (tile units, y in
/// metres): true when the segment is clear of every box it passes through
/// at the height it passes. Voids never block; floor slabs do. With
/// `y0 == y1` it is [`segment_clear`] at that height.
pub fn segment_clear_3d(s: &Solids, u0: f64, v0: f64, y0: f64, u1: f64, v1: f64, y1: f64) -> bool {
    let tick = s.next_tick();
    let mut stamp = s.stamp.borrow_mut();
    let (w, h) = (s.w as f64, s.h as f64);
    let b = &s.boxes;
    let du = u1 - u0;
    let dv = v1 - v0;
    let dy = y1 - y0;
    let mut x = u0.floor();
    let mut z = v0.floor();
    let xe = u1.floor();
    let ze = v1.floor();
    let sx = if du > 0.0 { 1.0 } else { -1.0 };
    let sz = if dv > 0.0 { 1.0 } else { -1.0 };
    let tdx = if du != 0.0 { (1.0 / du).abs() } else { f64::INFINITY };
    let tdz = if dv != 0.0 { (1.0 / dv).abs() } else { f64::INFINITY };
    let mut tmx = if du != 0.0 { (if sx > 0.0 { x + 1.0 - u0 } else { u0 - x }) * tdx } else { f64::INFINITY };
    let mut tmz = if dv != 0.0 { (if sz > 0.0 { z + 1.0 - v0 } else { v0 - z }) * tdz } else { f64::INFINITY };
    for _guard in 0..4 * (s.w + s.h) {
        if x < 0.0 || z < 0.0 || x >= w || z >= h {
            return false;
        }
        if let Some(t) = s.tile_of(x, z) {
            for k in s.start[t]..s.start[t + 1] {
                let bi = s.items[k as usize] as usize;
                if stamp[bi] == tick {
                    continue;
                }
                stamp[bi] = tick;
                let o = bi * 8;
                if b[o + 7] as i32 == BOX_VOID {
                    continue;
                }
                let Some((ta, tb)) = seg_box_t(u0, v0, du, dv, b[o] as f64, b[o + 1] as f64, b[o + 2] as f64, b[o + 3] as f64) else {
                    continue;
                };
                let (ya, yb) = (y0 + dy * ta, y0 + dy * tb);
                if ya.max(yb) >= b[o + 4] as f64 && ya.min(yb) <= b[o + 5] as f64 {
                    return false;
                }
            }
        }
        if x == xe && z == ze {
            return true;
        }
        if tmx < tmz {
            tmx += tdx;
            x += sx;
        } else {
            tmz += tdz;
            z += sz;
        }
        if tmx > 1.0 + tdx && tmz > 1.0 + tdz {
            return true;
        }
    }
    true
}

/// [`seg_box`]'s overlap as the part of the segment inside the box's
/// footprint (`t` from 0 to 1), or None
fn seg_box_t(u0: f64, v0: f64, du: f64, dv: f64, x0: f64, z0: f64, x1: f64, z1: f64) -> Option<(f64, f64)> {
    let mut tmin = 0.0;
    let mut tmax = 1.0;
    for (p0, dp, lo, hi) in [(u0, du, x0, x1), (v0, dv, z0, z1)] {
        if dp.abs() < 1e-9 {
            if p0 <= lo || p0 >= hi {
                return None;
            }
        } else {
            let mut a = (lo - p0) / dp;
            let mut b = (hi - p0) / dp;
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            tmin = nan_max(tmin, a);
            tmax = nan_min(tmax, b);
            if tmin >= tmax {
                return None;
            }
        }
    }
    Some((tmin, tmax))
}

/// Where the height rule breaks: every pair of neighbouring tiles that are
/// both walkable but do not meet at one height (tile, `DIRS` index). Empty
/// when heights change only along a stair or across a wall or a void.
pub fn ground_breaks(w: i32, h: i32, ground: &[Ground]) -> Vec<((i32, i32), usize)> {
    let mut out = vec![];
    for z in 0..h {
        for x in 0..w {
            let a = &ground[(z * w + x) as usize];
            if !a.walkable() {
                continue;
            }
            for d in [0usize, 2] {
                let (nx, nz) = (x + DIRS[d][0], z + DIRS[d][1]);
                if nx >= w || nz >= h {
                    continue;
                }
                let b = &ground[(nz * w + nx) as usize];
                if b.walkable() && !grounds_meet(a, b, d) {
                    out.push(((x, z), d));
                }
            }
        }
    }
    out
}

/// How thick the slab under a floor with height is, metres
pub const SLAB: f64 = 0.4;

/// The solids of a grid whose tiles stand on `ground` (the Building's
/// floors): a mass is one box from its foot to its top, a void one box
/// that stops a body and no look, a floor (or a stair) a slab under it.
pub fn build_ground_solids(w: i32, h: i32, cell: f64, ground: &[Ground]) -> Solids {
    build_ground_solids_with(w, h, cell, ground, &[])
}

/// [`build_ground_solids`] and `extra` boxes (a cell's front wall, …)
pub fn build_ground_solids_with(w: i32, h: i32, cell: f64, ground: &[Ground], extra: &[[f32; 8]]) -> Solids {
    let mut boxes: Vec<f32> = Vec::with_capacity(ground.len() * 8 + extra.len() * 8);
    for z in 0..h {
        for x in 0..w {
            let g = ground[(z * w + x) as usize];
            let t = (z * w + x) as f32;
            let (x0, z0, x1, z1) = (x as f32, z as f32, (x + 1) as f32, (z + 1) as f32);
            match g.kind {
                GROUND_MASS => boxes.extend_from_slice(&[x0, z0, x1, z1, g.h0, g.h1, t, BOX_WALL as f32]),
                GROUND_VOID => boxes.extend_from_slice(&[x0, z0, x1, z1, -1.0e4, 1.0e4, t, BOX_VOID as f32]),
                _ => {
                    let top = g.h0.min(g.h1);
                    boxes.extend_from_slice(&[x0, z0, x1, z1, top - SLAB as f32, top, t, BOX_FLOOR as f32]);
                }
            }
        }
    }
    for b in extra {
        boxes.extend_from_slice(b);
    }
    make_solids(w, h, cell, boxes)
}

fn seg_box(u0: f64, v0: f64, du: f64, dv: f64, x0: f64, z0: f64, x1: f64, z1: f64) -> bool {
    let mut tmin = 0.0;
    let mut tmax = 1.0;
    if du.abs() < 1e-9 {
        if u0 <= x0 || u0 >= x1 {
            return false;
        }
    } else {
        let mut a = (x0 - u0) / du;
        let mut b = (x1 - u0) / du;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        tmin = nan_max(tmin, a);
        tmax = nan_min(tmax, b);
        if tmin >= tmax {
            return false;
        }
    }
    if dv.abs() < 1e-9 {
        if v0 <= z0 || v0 >= z1 {
            return false;
        }
    } else {
        let mut a = (z0 - v0) / dv;
        let mut b = (z1 - v0) / dv;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        tmin = nan_max(tmin, a);
        tmax = nan_min(tmax, b);
        if tmin >= tmax {
            return false;
        }
    }
    true
}

/// Distance along an axis-aligned ray
/// (dir = DIRS index) to the first box face whose extent covers (the other
/// coordinate, y). Returns infinity past `max_t`.
pub fn ray_axis(s: &Solids, u: f64, v: f64, dir: usize, max_t: f64, y: f64) -> f64 {
    let (w, h) = (s.w as f64, s.h as f64);
    let b = &s.boxes;
    let [dxi, dzi] = DIRS[dir];
    let (dx, dz) = (dxi as f64, dzi as f64);
    let mut best = f64::INFINITY;
    let steps = max_t.ceil() + 1.0;
    let mut st = 0.0;
    while st <= steps {
        let x = (u + dx * st).floor();
        let z = (v + dz * st).floor();
        if x < 0.0 || z < 0.0 || x >= w || z >= h {
            break;
        }
        if let Some(t) = s.tile_of(x, z) {
            for k in s.start[t]..s.start[t + 1] {
                let o = s.items[k as usize] as usize * 8;
                if y < b[o + 4] as f64 || y > b[o + 5] as f64 || matches!(b[o + 7] as i32, BOX_VOID | BOX_FLOOR) {
                    continue;
                }
                let (bx0, bz0, bx1, bz1) = (b[o] as f64, b[o + 1] as f64, b[o + 2] as f64, b[o + 3] as f64);
                let mut d;
                if dxi != 0 {
                    if v <= bz0 || v >= bz1 {
                        continue;
                    }
                    d = if dx > 0.0 { bx0 - u } else { u - bx1 };
                    if d < -1e-6 && (if dx > 0.0 { bx1 > u } else { bx0 < u }) {
                        d = 0.0;
                    } // starting inside
                } else {
                    if u <= bx0 || u >= bx1 {
                        continue;
                    }
                    d = if dz > 0.0 { bz0 - v } else { v - bz1 };
                    if d < -1e-6 && (if dz > 0.0 { bz1 > v } else { bz0 < v }) {
                        d = 0.0;
                    }
                }
                if d >= -1e-6 && d < best {
                    best = nan_max(0.0, d);
                }
            }
        }
        if best <= st {
            break;
        }
        st += 1.0;
    }
    if best <= max_t {
        best
    } else {
        f64::INFINITY
    }
}

/// Per-tile ceiling heights (metres) from the zones
/// (for `Game::tile_ceil`).
pub fn ceiling_heights(l: &Layout) -> Vec<f32> {
    (0..(l.w * l.h) as usize).map(|i| l.zones[l.zone_of[i] as usize].ceil_h as f32).collect()
}

/// The return value of `layout_stats`. `arch` keeps first-seen zone order.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStats {
    pub w: i32,
    pub h: i32,
    pub open: usize,
    pub open_share: f64,
    pub zones: usize,
    /// archetype → tiles covered (zone rect area)
    pub arch: Vec<(&'static str, i32)>,
    pub boxes: usize,
    pub thin_boxes: usize,
    pub pillars: usize,
    pub jambs: usize,
    pub lintels: usize,
    pub anchors: usize,
}

/// Summary numbers, for tests and the debug panel.
pub fn layout_stats(l: &Layout, s: Option<&Solids>) -> LayoutStats {
    layout_stats_view(&l.view(), s)
}

/// `layout_stats` over a [`LayoutView`].
pub fn layout_stats_view(l: &LayoutView, s: Option<&Solids>) -> LayoutStats {
    let n = (l.w * l.h) as usize;
    let open = l.tiles[..n].iter().filter(|&&t| t == 0).count();
    let mut arch: Vec<(&'static str, i32)> = vec![];
    for z in l.zones {
        let area = (z.x1 - z.x0 + 1) * (z.z1 - z.z0 + 1);
        match arch.iter_mut().find(|(k, _)| *k == z.arch) {
            Some(e) => e.1 += area,
            None => arch.push((z.arch, area)),
        }
    }
    let mut kinds = [0usize; 8];
    let mut thin = 0;
    if let Some(s) = s {
        for b in 0..s.count {
            let o = b * 8;
            kinds[s.boxes[o + 7] as usize] += 1;
            let bx = &s.boxes;
            let w = nan_min(bx[o + 2] as f64 - bx[o] as f64, bx[o + 3] as f64 - bx[o + 1] as f64);
            if w < 0.2 {
                thin += 1;
            }
        }
    }
    LayoutStats {
        w: l.w,
        h: l.h,
        open,
        open_share: open as f64 / n as f64,
        zones: l.zones.len(),
        arch,
        boxes: s.map(|s| s.count).unwrap_or(0),
        thin_boxes: thin,
        pillars: kinds[BOX_PILLAR as usize],
        jambs: kinds[BOX_JAMB as usize],
        lintels: kinds[BOX_LINTEL as usize],
        anchors: kinds[BOX_ANCHOR as usize],
    }
}
