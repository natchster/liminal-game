//! The Loop (`modes::loop_rooms`): the line of rooms that comes back round.
//!
//! Inspired by SCP-970 ("The Recursive Room" by Freudian, CC BY-SA 3.0,
//! https://scp-wiki.wikidot.com/scp-970). Our own names and text: this is
//! "the Loop". A short, straight row of ordinary rooms joined by doorways,
//! where walking out of the last room brings you into the first one again —
//! and every lap forward the rooms are a little less like they were. Walking
//! back undoes it. The way out is a door that only exists where the drift is
//! zero.
//!
//! Plan (metres, local to `group`, which the game keeps at the origin): five
//! identical rooms in a row along +z. Room k's floor runs from z = k·P to
//! (k+1)·P (P = room length + wall thickness); the dividing wall between
//! rooms k−1 and k is centred on z = k·P with a doorway on the axis. x runs
//! across the room, −W/2 … W/2.
//!
//! The seam: the player's z always stays in [0, STRIP). Crossing z = STRIP
//! moves them back by STRIP and one lap on; crossing z = 0 moves them on by
//! STRIP and one lap back. What is drawn is a window of rooms centred on the
//! player's *global* room g = lap·5 + room: VIEW rooms each way (bare shells
//! far off, furnished within NEAR). A room's look depends only on g, and the
//! jump does not change g, so the jump cannot show. Laps never report below
//! 0: every lap below 0 looks like lap 0 (it has the exit door too), and the
//! internal count stops at LAP_MIN.
//!
//! Drift (cumulative, per lap): 1 the wallpaper and the light tint drift;
//! 2 the chairs turn to face the doorways, the plant is on the other side;
//! 3 the lights flicker, the hum drops; 4 red light, dark stains, the paper
//! rots; 5 a seated faceless figure in room 3, nearer each time you look
//! back, never moves while seen, touching it takes your mind; 6+ fog
//! thickens, the rooms go bare, the mind drains faster every lap. Pickups
//! (Almond Water, batteries) only lie in rooms at lap ≥ 3.
//!
//! Everything is instanced: one instanced mesh per part, one instance per
//! room drawn. The window is rebuilt only when you pass a doorway.
//!
//! `LoopMode::new(LoopDeps)` builds it; the pub fields and methods below are
//! what the game glue (`modes/mod.rs`) uses.

use crate::base::numeric::{hypot2, hypot3};
use crate::modes::stairwell::{CanvasTexFn, SharedRng};
use engine::prelude::*;
use std::collections::BTreeSet;
use std::f64::consts::PI;

// ------------------------------------------------------------- constants ---
#[allow(non_camel_case_types)]
pub struct LOOP;
impl LOOP {
    pub const ROOMS: i32 = 5;
    /// room width, length, ceiling, wall thickness
    pub const W: f64 = 5.0;
    pub const L: f64 = 5.0;
    pub const H: f64 = 2.7;
    pub const T: f64 = 0.2;
    /// clear opening of the doorways between rooms
    pub const DOOR_W: f64 = 1.0;
    pub const DOOR_H: f64 = 2.1;
    /// rooms drawn each way from yours (shell and lamp)
    pub const VIEW: i32 = 10;
    /// …of which furnished: through 4+ doorways only |x| < 0.9 shows
    pub const NEAR: i32 = 3;
    /// "room 3": the side door lets you in here
    pub const ENTRY_ROOM: i32 = 2;
    /// "room 1": the only way out, only at lap 0
    pub const EXIT_ROOM: i32 = 0;
    /// where it sits
    pub const FIGURE_ROOM: i32 = 2;
    pub const PICKUP_LAP: i32 = 3;
    pub const FIGURE_LAP: i32 = 5;
    pub const BARE_LAP: i32 = 6;
}
/// (5.2 m)
pub const LOOP_PITCH: f64 = LOOP::L + LOOP::T;
/// (26 m)
pub const LOOP_STRIP: f64 = LOOP::ROOMS as f64 * LOOP_PITCH;
/// = −ceil((VIEW + ROOMS) / ROOMS): below this the internal lap stops counting.
pub const LAP_MIN: i32 = -((LOOP::VIEW + LOOP::ROOMS + LOOP::ROOMS - 1) / LOOP::ROOMS);

const ROOMS: i32 = LOOP::ROOMS;
const W: f64 = LOOP::W;
const H: f64 = LOOP::H;
const DOOR_W: f64 = LOOP::DOOR_W;
const DOOR_H: f64 = LOOP::DOOR_H;
const TH: f64 = LOOP::T;
const P: f64 = LOOP_PITCH;
const STRIP: f64 = LOOP_STRIP;
const K: i32 = LOOP::VIEW;
const KN: i32 = LOOP::NEAR;
/// slots in the drawn window (2·VIEW + 1)
pub const LOOP_SLOTS: usize = (2 * LOOP::VIEW + 1) as usize;
const NS: usize = LOOP_SLOTS;
const NN: usize = (2 * KN + 1) as usize;

#[inline]
fn imod(a: i32, n: i32) -> i32 {
    ((a % n) + n) % n
}
#[inline]
fn clamp(v: f64, a: f64, b: f64) -> f64 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}
#[inline]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
#[inline]
fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
#[inline]
fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}
/// A 32-bit integer hash to [0, 1).
pub fn hash01(n: f64) -> f64 {
    let mut n = (engine::math::to_i32(n) as u32) ^ 0x9e3779b9;
    n = n.wrapping_mul(0x85ebca6b);
    n ^= n >> 13;
    n = n.wrapping_mul(0xc2b2ae35);
    n ^= n >> 16;
    n as f64 / 4294967296.0
}

// ------------------------------------------------------------ the layout ---
/// A box of furniture: x across, v along the strip from the room's centre.
#[derive(Clone, Copy, Debug)]
pub struct FurnBox {
    pub x: f64,
    pub v: f64,
    pub w: f64,
    pub d: f64,
    pub h: f64,
}
/// A seat: yaw turns its front (local +z) to (sin yaw, cos yaw); `turned` from lap 2.
#[derive(Clone, Copy, Debug)]
pub struct FurnSeat {
    pub x: f64,
    pub v: f64,
    pub yaw: f64,
    pub turned: f64,
}
/// Where a pickup lies (x, y, v), and where it lies in the bare rooms (bx, by, bv).
#[derive(Clone, Copy, Debug)]
pub struct FurnSpot {
    pub x: f64,
    pub y: f64,
    pub v: f64,
    pub bx: f64,
    pub by: f64,
    pub bv: f64,
}
/// The furniture layout (`FURN`), in room coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Furn {
    pub desk: FurnBox,
    /// lap ≥ 2: faces the far doorway
    pub desk_chair: FurnSeat,
    /// lap ≥ 2: faces the near doorway
    pub visit_chair: FurnSeat,
    pub cabinet: FurnBox,
    pub clock_v: f64,
    pub clock_y: f64,
    /// lap ≥ 2: x mirrored
    pub plant_x: f64,
    pub plant_v: f64,
    /// right wall of room 3
    pub entry_door_v: f64,
    /// left wall of room 1
    pub exit_door_v: f64,
    pub figure: FurnSeat,
    /// on the desk; bare: on the floor
    pub almond: FurnSpot,
    pub battery: FurnSpot,
}
pub const FURN: Furn = Furn {
    desk: FurnBox { x: -W / 2.0 + 0.02 + 0.375, v: -0.3, w: 1.4, d: 0.75, h: 0.74 },
    desk_chair: FurnSeat { x: -1.3, v: -0.3, yaw: -PI / 2.0, turned: 0.0 },
    visit_chair: FurnSeat { x: 1.95, v: -0.55, yaw: -PI / 2.0, turned: PI },
    cabinet: FurnBox { x: W / 2.0 - 0.02 - 0.31, v: -1.75, w: 0.46, d: 0.62, h: 1.32 },
    clock_v: -0.55,
    clock_y: 2.02,
    plant_x: 2.02,
    plant_v: 2.02,
    entry_door_v: 0.95,
    exit_door_v: 1.2,
    figure: FurnSeat { x: 0.95, v: 0.15, yaw: PI, turned: PI },
    almond: FurnSpot { x: -1.905, y: 0.74, v: -0.42, bx: -1.7, by: 0.0, bv: -0.2 },
    battery: FurnSpot { x: W / 2.0 - 0.02 - 0.31, y: 1.32, v: -1.75, bx: 1.75, by: 0.0, bv: -1.3 },
};
const CHAIR_R: f64 = 0.3;
const PLANT_R: f64 = 0.24;
const FIG_R: f64 = 0.36;
const BODY_R: f64 = 0.3;

// ------------------------------------------------------------ the drift ----
const TINT: [[f64; 3]; 7] =
    [[1.0, 1.0, 1.0], [0.96, 1.0, 0.84], [0.93, 0.98, 0.78], [0.88, 0.92, 0.74], [0.92, 0.72, 0.64], [0.82, 0.6, 0.54], [0.6, 0.48, 0.44]];
const LIGHT: [[f64; 3]; 7] =
    [[1.0, 0.95, 0.82], [0.92, 1.0, 0.8], [0.88, 1.0, 0.82], [0.84, 0.96, 0.9], [1.0, 0.36, 0.27], [1.0, 0.25, 0.19], [0.85, 0.17, 0.13]];
const BRIGHT: [f64; 7] = [1.0, 1.0, 0.97, 0.9, 0.86, 0.74, 0.55];

/// What the rooms of a lap look like (`lap_look(lap)`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LapLook {
    pub lap: i32,
    pub tint: [f64; 3],
    pub light: [f64; 3],
    pub bright: f64,
    pub flicker: f64,
    /// chairs face the doorways, the plant has moved
    pub turned: bool,
    pub red: bool,
    pub stains: bool,
    pub dirty: bool,
    pub figure: bool,
    pub bare: bool,
    pub pickups: bool,
    pub exit: bool,
}
/// What the rooms of lap `lap` look like (pure). Laps below 0 look like lap 0.
pub fn lap_look(lap: i32) -> LapLook {
    let l = 0.max(lap);
    let k = l.min(6) as usize;
    let past = 0.max(l - 6);
    let fade = 0.35f64.max(0.93f64.powf(past as f64));
    LapLook {
        lap: l,
        tint: [TINT[k][0] * fade, TINT[k][1] * fade, TINT[k][2] * fade],
        light: LIGHT[k],
        bright: 0.22f64.max(BRIGHT[k] * 0.9f64.powf(past as f64)),
        flicker: if l >= 3 { 1f64.min(0.3 + 0.18 * (l - 3) as f64) } else { 0.0 },
        turned: l >= 2,
        red: l >= 4,
        stains: l >= 4,
        dirty: l >= 4,
        figure: l >= LOOP::FIGURE_LAP,
        bare: l >= LOOP::BARE_LAP,
        pickups: l >= LOOP::PICKUP_LAP,
        exit: l == 0,
    }
}

/// What a pickup is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Almond,
    Battery,
}
impl PickupKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PickupKind::Almond => "almond",
            PickupKind::Battery => "battery",
        }
    }
    /// where this kind of pickup lies (its `FURN` spot)
    pub fn spot(&self) -> FurnSpot {
        match self {
            PickupKind::Almond => FURN.almond,
            PickupKind::Battery => FURN.battery,
        }
    }
}
/// What lies in room `room` of lap `lap`.
pub fn pickup_at(lap: i32, room: i32) -> Option<PickupKind> {
    if lap < LOOP::PICKUP_LAP {
        return None;
    }
    let h = hash01((lap * 131 + room * 17 + 7) as f64);
    if h < 0.35 {
        return None;
    }
    Some(if h < 0.7 { PickupKind::Almond } else { PickupKind::Battery })
}
/// The reverb zone parameters `loop_reverb(lap)` returns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopReverb {
    pub t60: f64,
    pub damp: f64,
    pub wet: f64,
    pub pre: f64,
    pub er: i32,
}
/// Reverb for the rooms of a lap, one more step every lap, up to six.
pub fn loop_reverb(lap: f64) -> LoopReverb {
    let k = clamp(engine::math::round_half_up(lap), 0.0, 6.0);
    let ki = k as i32;
    LoopReverb { t60: 0.5 + 0.4 * k, damp: 0.8 - 0.06 * k, wet: 0.1 + 0.045 * k, pre: 0.004 + 0.006 * k, er: 6 - (ki >> 1) }
}
/// The ballast hum's pitch in cents at a (fractional) lap.
pub fn loop_hum_cents(lam: f64) -> f64 {
    (-110f64).max(-7.0 * lam - 30.0 * smooth(2.5, 3.0, lam))
}
// global atmosphere per lap: fog rgb, fog density, ambient, hemisphere,
// hum strength, sanity drain /s (negative heals), glow around the player
const ENV: [[f64; 9]; 7] = [
    [0.66, 0.61, 0.44, 0.028, 0.7, 0.95, 0.9, -0.5, 0.3],
    [0.6, 0.6, 0.42, 0.03, 0.66, 0.9, 0.85, -0.25, 0.3],
    [0.54, 0.56, 0.4, 0.032, 0.58, 0.8, 0.8, 0.0, 0.35],
    [0.43, 0.45, 0.36, 0.036, 0.45, 0.64, 0.7, 0.0, 0.45],
    [0.3, 0.09, 0.07, 0.044, 0.3, 0.44, 0.55, 0.15, 0.45],
    [0.2, 0.05, 0.04, 0.054, 0.22, 0.32, 0.4, 0.3, 0.5],
    [0.05, 0.03, 0.025, 0.08, 0.1, 0.14, 0.08, 0.7, 0.55],
];
pub fn env_row(l: i32) -> [f64; 9] {
    let k = 0.max(l).min(6) as usize;
    let mut o = ENV[k];
    let past = 0.max(l - 6);
    if past > 0 {
        let p = past as f64;
        o[3] = 0.16f64.min(0.08 + 0.014 * p);
        o[4] *= 0.88f64.powf(p);
        o[5] *= 0.88f64.powf(p);
        o[6] = 0.05;
        o[7] = 0.7 + 0.45 * p;
        o[8] = 0.6;
    }
    o
}
/// Sanity drain per second at lap `lap` (negative: the calm laps heal).
pub fn loop_drain(lap: i32) -> f64 {
    env_row(lap)[7]
}

// ----------------------------------------------------------------- types ---
/// The game's own textures (`LoopDeps::textures`, shared, never disposed here).
#[derive(Clone, Debug, Default)]
pub struct LoopTextures {
    pub wall: Option<Tex>,
    pub wall_dirty: Option<Tex>,
    pub carpet: Option<Tex>,
    pub ceiling: Option<Tex>,
    pub panel: Option<Tex>,
}
/// What the Loop is built with.
pub struct LoopDeps {
    /// the canvas texture maker (None: the stains and the EXIT sign are drawn plain)
    pub canvas_tex: Option<CanvasTexFn>,
    /// the random source (the game passes `base::rng::native_rnd`)
    pub rng: Box<dyn FnMut() -> f64>,
    pub textures: LoopTextures,
}
impl LoopDeps {
    /// The game's deps: its canvas textures, the native RNG and the given textures.
    pub fn game(textures: LoopTextures) -> LoopDeps {
        LoopDeps { canvas_tex: Some(crate::modes::stairwell::game_canvas_tex()), rng: Box::new(crate::base::rng::native_rnd), textures }
    }
}
impl Default for LoopDeps {
    /// No canvas textures, the seeded `rnd()`, no textures.
    fn default() -> Self {
        LoopDeps { canvas_tex: None, rng: Box::new(crate::base::rng::rnd), textures: LoopTextures::default() }
    }
}

/// The Loop's state (`LoopMode::state()`).
#[derive(Clone, Debug, Default)]
pub struct LoopState {
    /// internal lap: below 0 still looks like lap 0
    pub lap: i32,
    pub x: f64,
    pub z: f64,
    pub yaw: f64,
    pub time: f64,
    pub entry_door: bool,
    pub entry_lap: i32,
    pub entry_t: f64,
    pub entry_unseen: f64,
    /// the pickup key (lap·8 + room) of every pickup taken
    pub taken: BTreeSet<i32>,
    pub seams: i32,
    pub started: bool,
}
/// The one figure that moves (`LoopMode::figure()`).
#[derive(Clone, Debug)]
pub struct LoopFigure {
    pub active: bool,
    pub lap: i32,
    /// global z (lap·STRIP + local z)
    pub gz: f64,
    pub x: f64,
    pub yaw: f64,
    pub seen: bool,
    pub unseen_t: f64,
    pub moved: f64,
    /// its instance in the figure mesh, or -1
    pub inst: i32,
    pub sightings: i32,
}
impl Default for LoopFigure {
    fn default() -> Self {
        LoopFigure { active: false, lap: 0, gz: 0.0, x: 0.0, yaw: 0.0, seen: false, unseen_t: 0.0, moved: 0.0, inst: -1, sightings: 0 }
    }
}
/// The global atmosphere, blended across the seam (`LoopMode::env()`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopEnv {
    pub fog_r: f64,
    pub fog_g: f64,
    pub fog_b: f64,
    pub fog: f64,
    pub amb: f64,
    pub hemi: f64,
    pub sky_r: f64,
    pub sky_g: f64,
    pub sky_b: f64,
    pub ground_r: f64,
    pub ground_g: f64,
    pub ground_b: f64,
    pub hum: f64,
    pub hum_cents: f64,
    /// the reverb zone index (0…6)
    pub reverb: i32,
    pub drain: f64,
    pub glow: f64,
    pub lam: f64,
}
impl Default for LoopEnv {
    fn default() -> Self {
        LoopEnv {
            fog_r: 0.0,
            fog_g: 0.0,
            fog_b: 0.0,
            fog: 0.03,
            amb: 0.5,
            hemi: 0.7,
            sky_r: 1.0,
            sky_g: 1.0,
            sky_b: 1.0,
            ground_r: 0.3,
            ground_g: 0.28,
            ground_b: 0.2,
            hum: 0.8,
            hum_cents: 0.0,
            reverb: 0,
            drain: 0.0,
            glow: 0.3,
            lam: 0.0,
        }
    }
}
/// One of the six lamps `update()` hands the game's pooled lights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopLight {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub i: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
impl Default for LoopLight {
    fn default() -> Self {
        LoopLight { x: 0.0, y: 0.0, z: 0.0, i: 0.0, r: 1.0, g: 1.0, b: 1.0 }
    }
}
/// Events `update()` returns (`LoopOut::events`).
#[derive(Clone, Debug, PartialEq)]
pub enum LoopEvent {
    /// `seam`: the player crossed the seam (`dir` +1 forward, −1 back) into lap `lap`
    Seam { dir: i32, lap: i32 },
    /// `pickup`: the player picked up `kind` in room `room` of lap `lap`
    Pickup { kind: PickupKind, lap: i32, room: i32 },
    /// `entryGone`: the side door the player came in by is gone
    EntryGone,
    /// `figureSeen`: the player catches sight of the figure, `dist` away (it moved `moved` m unseen)
    FigureSeen { dist: f64, moved: f64 },
    /// `figureNearer`: as `figureSeen`, but it moved more than 0.3 m while unseen
    FigureNearer { dist: f64, moved: f64 },
}
impl LoopEvent {
    pub fn type_name(&self) -> &'static str {
        match self {
            LoopEvent::Seam { .. } => "seam",
            LoopEvent::Pickup { .. } => "pickup",
            LoopEvent::EntryGone => "entryGone",
            LoopEvent::FigureSeen { .. } => "figureSeen",
            LoopEvent::FigureNearer { .. } => "figureNearer",
        }
    }
}
/// `update(dt, inp)`'s input: the movement axes, speed and view yaw.
#[derive(Clone, Copy, Debug, Default)]
pub struct LoopInput {
    pub fwd: f64,
    pub strafe: f64,
    pub speed: f64,
    pub yaw: f64,
}
/// What `update` returns (one value reused every frame, so the lights past
/// `light_count` keep their old values).
#[derive(Clone, Debug, Default)]
pub struct LoopOut {
    pub x: f64,
    pub z: f64,
    pub lap: i32,
    pub room: i32,
    pub near_exit: bool,
    /// seconds of touching the figure this frame (dt, or 0)
    pub touch: f64,
    pub events: Vec<LoopEvent>,
    pub env: LoopEnv,
    pub lights: [LoopLight; 6],
    pub light_count: usize,
}

// --------------------------------------------------- geometry builder ---
/// Merges primitives (transformed, painted with a vertex colour) and quads
/// into one geometry.
#[derive(Default)]
struct Builder {
    pos: Vec<f32>,
    nor: Vec<f32>,
    uv: Vec<f32>,
    col: Vec<f32>,
    idx: Vec<u32>,
}
impl Builder {
    fn new() -> Builder {
        Builder::default()
    }
    fn add(&mut self, geo: Geo, m: &Mat4, hex: u32) -> &mut Self {
        let g = geo.r();
        let base = (self.pos.len() / 3) as u32;
        let mut m3 = Mat3::default();
        m3.get_normal_matrix(m);
        let c = Color::hex(hex);
        let count = g.vertex_count();
        let has_uv = !g.uv.is_empty();
        for i in 0..count {
            let mut v = Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
            v.apply_matrix4(m);
            self.pos.extend_from_slice(&[v.x as f32, v.y as f32, v.z as f32]);
            let mut n = Vec3::new(g.normal[i * 3] as f64, g.normal[i * 3 + 1] as f64, g.normal[i * 3 + 2] as f64);
            n.apply_matrix3(&m3).normalize();
            self.nor.extend_from_slice(&[n.x as f32, n.y as f32, n.z as f32]);
            if has_uv {
                self.uv.extend_from_slice(&[g.uv[i * 2], g.uv[i * 2 + 1]]);
            } else {
                self.uv.extend_from_slice(&[0.0, 0.0]);
            }
            self.col.extend_from_slice(&[c.r as f32, c.g as f32, c.b as f32]);
        }
        let flip = m.determinant() < 0.0;
        let cnt = g.index_count();
        let mut i = 0;
        while i < cnt {
            let (a, b, cc) = match &g.index {
                Some(ix) => (ix[i], ix[i + 1], ix[i + 2]),
                None => (i as u32, i as u32 + 1, i as u32 + 2),
            };
            if flip {
                self.idx.extend_from_slice(&[base + a, base + cc, base + b]);
            } else {
                self.idx.extend_from_slice(&[base + a, base + b, base + cc]);
            }
            i += 3;
        }
        drop(g);
        geo.dispose();
        self
    }
    /// a body's mesh in its own colours (`bodies.bin`), moved by `m`; the
    /// shared geometry is left as it was (it is cached)
    fn add_painted(&mut self, geo: &Geo, m: &Mat4) -> &mut Self {
        let g = geo.r();
        let base = (self.pos.len() / 3) as u32;
        let mut m3 = Mat3::default();
        m3.get_normal_matrix(m);
        let count = g.vertex_count();
        let painted = g.color.len() >= count * 3;
        for i in 0..count {
            let mut v = Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
            v.apply_matrix4(m);
            self.pos.extend_from_slice(&[v.x as f32, v.y as f32, v.z as f32]);
            let mut n = Vec3::new(g.normal[i * 3] as f64, g.normal[i * 3 + 1] as f64, g.normal[i * 3 + 2] as f64);
            n.apply_matrix3(&m3).normalize();
            self.nor.extend_from_slice(&[n.x as f32, n.y as f32, n.z as f32]);
            self.uv.extend_from_slice(&[0.0, 0.0]);
            if painted {
                self.col.extend_from_slice(&g.color[i * 3..i * 3 + 3]);
            } else {
                self.col.extend_from_slice(&[0.5, 0.5, 0.5]);
            }
        }
        if let Some(ix) = &g.index {
            self.idx.extend(ix.iter().map(|i| base + i));
        }
        self
    }
    /// quad from four corners (in order around the edge), facing normal n;
    /// uvf(x, y, z) → [u, v]
    fn quad(&mut self, c4: [[f64; 3]; 4], n: [f64; 3], mut uvf: impl FnMut(f64, f64, f64) -> [f64; 2], hex: u32) -> &mut Self {
        let base = (self.pos.len() / 3) as u32;
        let c = Color::hex(hex);
        for q in &c4 {
            self.pos.extend_from_slice(&[q[0] as f32, q[1] as f32, q[2] as f32]);
            self.nor.extend_from_slice(&[n[0] as f32, n[1] as f32, n[2] as f32]);
            let t = uvf(q[0], q[1], q[2]);
            self.uv.extend_from_slice(&[t[0] as f32, t[1] as f32]);
            self.col.extend_from_slice(&[c.r as f32, c.g as f32, c.b as f32]);
        }
        // wind so the front faces n
        let (ax, ay, az) = (c4[1][0] - c4[0][0], c4[1][1] - c4[0][1], c4[1][2] - c4[0][2]);
        let (bx, by, bz) = (c4[2][0] - c4[0][0], c4[2][1] - c4[0][1], c4[2][2] - c4[0][2]);
        let d = (ay * bz - az * by) * n[0] + (az * bx - ax * bz) * n[1] + (ax * by - ay * bx) * n[2];
        if d >= 0.0 {
            self.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        } else {
            self.idx.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
        self
    }
    fn build(&mut self, owned: &mut Vec<Geo>) -> Geo {
        let g = Geo::new();
        g.set_position(std::mem::take(&mut self.pos));
        g.set_normal(std::mem::take(&mut self.nor));
        g.set_uv(std::mem::take(&mut self.uv));
        g.set_color(std::mem::take(&mut self.col), 3);
        g.set_index(std::mem::take(&mut self.idx));
        g.compute_bounding_sphere();
        owned.push(g.clone());
        g
    }
}
/// A box with finely rounded edges — a 3×3×3
/// box whose outer ring of vertices is pulled onto a radius-r bevel, with
/// normals along the bevel.
pub fn bevel_box(w: f64, h: f64, d: f64, r: f64) -> Geo {
    let geo = box_geo!(1, 1, 1, 3, 3, 3);
    {
        let mut g = geo.g();
        let (hx, hy, hz) = (w / 2.0, h / 2.0, d / 2.0);
        let r = r.min(hx * 0.45).min(hy * 0.45).min(hz * 0.45);
        let map = |v: f64, hh: f64| -> f64 {
            let a = v.abs();
            let s = engine::math::sign_of(v);
            if a > 0.4 {
                s * hh
            } else if a > 0.1 {
                s * (hh - r)
            } else {
                0.0
            }
        };
        for i in 0..g.vertex_count() {
            let xx = map(g.position[i * 3] as f64, hx);
            let yy = map(g.position[i * 3 + 1] as f64, hy);
            let zz = map(g.position[i * 3 + 2] as f64, hz);
            let ix = clamp(xx, -hx + r, hx - r);
            let iy = clamp(yy, -hy + r, hy - r);
            let iz = clamp(zz, -hz + r, hz - r);
            let (mut dx, mut dy, mut dz) = (xx - ix, yy - iy, zz - iz);
            let l = hypot3(dx, dy, dz);
            if l > 1e-9 {
                dx /= l;
                dy /= l;
                dz /= l;
                g.position[i * 3] = (ix + dx * r) as f32;
                g.position[i * 3 + 1] = (iy + dy * r) as f32;
                g.position[i * 3 + 2] = (iz + dz * r) as f32;
                g.normal[i * 3] = dx as f32;
                g.normal[i * 3 + 1] = dy as f32;
                g.normal[i * 3 + 2] = dz as f32;
            } else {
                g.position[i * 3] = xx as f32;
                g.position[i * 3 + 1] = yy as f32;
                g.position[i * 3 + 2] = zz as f32;
            }
        }
        g.touch();
    }
    geo
}
/// matrix from position, Euler rotation (XYZ) and scale
fn mtx(x: f64, y: f64, z: f64, rx: f64, ry: f64, rz: f64, sx: f64, sy: f64, sz: f64) -> Mat4 {
    let q = Quat::from_euler(&Euler::new(rx, ry, rz));
    Mat4::compose(&Vec3::new(x, y, z), &q, &Vec3::new(sx, sy, sz))
}
fn mt(x: f64, y: f64, z: f64) -> Mat4 {
    mtx(x, y, z, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
}
fn mtr(x: f64, y: f64, z: f64, rx: f64, ry: f64, rz: f64) -> Mat4 {
    mtx(x, y, z, rx, ry, rz, 1.0, 1.0, 1.0)
}
fn under(parent: &Mat4, local: &Mat4) -> Mat4 {
    Mat4::mul(parent, local)
}

// ------------------------------------------------------------- the shell ---
const HW: f64 = DOOR_W / 2.0 + 0.03;
const DHH: f64 = DOOR_H + 0.03;
const Z_N: f64 = TH / 2.0;
const Z_F: f64 = P - TH / 2.0;
fn wall_uv(along: f64, y: f64) -> [f64; 2] {
    [along / 2.6, (y + 0.25) / 3.1]
}

fn add_chair(b: &mut Builder, p0: &Mat4) {
    const FAB: u32 = 0x4a453d;
    const MET: u32 = 0x262624;
    const DARK: u32 = 0x1a1918;
    for i in 0..5 {
        let a = (i as f64 / 5.0) * PI * 2.0;
        b.add(bevel_box(0.05, 0.035, 0.3, 0.008), &under(p0, &mtr(a.sin() * 0.15, 0.06, a.cos() * 0.15, 0.0, a, 0.0)), MET);
        b.add(cyl_geo!(0.024, 0.024, 0.04, 8), &under(p0, &mtr(a.sin() * 0.29, 0.024, a.cos() * 0.29, 0.0, 0.0, PI / 2.0)), DARK);
    }
    b.add(cyl_geo!(0.024, 0.03, 0.36, 10), &under(p0, &mt(0.0, 0.26, 0.0)), MET);
    b.add(bevel_box(0.46, 0.08, 0.44, 0.025), &under(p0, &mt(0.0, 0.47, 0.02)), FAB);
    b.add(bevel_box(0.43, 0.46, 0.06, 0.022), &under(p0, &mtr(0.0, 0.84, -0.21, -0.1, 0.0, 0.0)), FAB);
    b.add(bevel_box(0.05, 0.3, 0.03, 0.006), &under(p0, &mtr(0.0, 0.6, -0.23, -0.05, 0.0, 0.0)), MET);
    for s in [-1.0f64, 1.0] {
        b.add(bevel_box(0.035, 0.2, 0.035, 0.006), &under(p0, &mt(s * 0.25, 0.6, 0.0)), MET);
        b.add(bevel_box(0.06, 0.03, 0.25, 0.01), &under(p0, &mt(s * 0.25, 0.71, 0.01)), DARK);
    }
}

/// The Loop: the instanced rooms, the figure, the atmosphere and the lap state.
pub struct LoopMode {
    /// the root group (named 'loop', hidden until `enter`)
    pub group: Obj,
    /// the lap state
    pub st: LoopState,
    /// the figure
    pub fig: LoopFigure,
    /// the atmosphere
    pub env: LoopEnv,
    out: LoopOut,
    /// every instanced part, by its name without the 'loop:' prefix
    pub meshes: Vec<(&'static str, Obj)>,
    pub ghost_door: Obj,
    pub m_wall: Obj,
    pub m_wall_dirty: Obj,
    pub m_floor: Obj,
    pub m_ceil: Obj,
    pub m_trim: Obj,
    pub m_trim_far: Obj,
    pub m_panel: Obj,
    pub m_stain: Obj,
    pub m_furn: Obj,
    pub m_chair: Obj,
    pub m_plant: Obj,
    pub m_almond: Obj,
    pub m_battery: Obj,
    pub m_figure: Obj,
    pub m_door: Obj,
    pub m_sign: Obj,
    owned_geos: Vec<Geo>,
    owned_mats: Vec<Mat>,
    owned_texs: Vec<Tex>,
    /// the drawn window: slot j holds global room win_g + j − VIEW
    win_g: i32,
    slot_z: [f32; NS],
    slot_l: [i32; NS],
    slot_r: [i8; NS],
    slot_bright: [f32; NS],
    slot_dist: [f32; NS],
    #[allow(dead_code)]
    rng: SharedRng,
}

fn set_at(mesh: &Obj, i: usize, x: f64, y: f64, z: f64, yaw: f64) {
    let q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw);
    let m = Mat4::compose(&Vec3::new(x, y, z), &q, &Vec3::new(1.0, 1.0, 1.0));
    mesh.set_matrix_at(i, &m);
}
fn put(mesh: &Obj, x: f64, y: f64, z: f64, yaw: f64, r: f64, g: f64, b: f64) -> usize {
    let i = mesh.instance_count();
    mesh.set_count(i + 1);
    set_at(mesh, i, x, y, z, yaw);
    mesh.set_color_at(i, &Color::rgb_linear(r, g, b));
    i
}
fn put1(mesh: &Obj, x: f64, y: f64, z: f64, yaw: f64) -> usize {
    put(mesh, x, y, z, yaw, 1.0, 1.0, 1.0)
}
fn circle_rect(cx: f64, cz: f64, r: f64, x0: f64, z0: f64, x1: f64, z1: f64) -> bool {
    let qx = if cx < x0 {
        x0
    } else if cx > x1 {
        x1
    } else {
        cx
    };
    let qz = if cz < z0 {
        z0
    } else if cz > z1 {
        z1
    } else {
        cz
    };
    (cx - qx) * (cx - qx) + (cz - qz) * (cz - qz) < r * r
}
fn circle(cx: f64, cz: f64, r: f64, ox: f64, oz: f64, or: f64) -> bool {
    (cx - ox) * (cx - ox) + (cz - oz) * (cz - oz) < (r + or) * (r + or)
}
/// the side walls and the dividing walls, open only at the doorways (`wall_hit`)
pub fn wall_hit(x: f64, z: f64, r: f64) -> bool {
    if x < -W / 2.0 + r || x > W / 2.0 - r {
        return true;
    }
    let k = engine::math::round_half_up(z / P);
    let wz = k * P;
    if (z - wz).abs() < TH / 2.0 + r {
        let j = DOOR_W / 2.0;
        if circle_rect(x, z, r, -W, wz - TH / 2.0, -j, wz + TH / 2.0) || circle_rect(x, z, r, j, wz - TH / 2.0, W, wz + TH / 2.0) {
            return true;
        }
    }
    false
}
/// Can you see from a to b along the strip? only through the doorways
pub fn clear_line(ax: f64, az: f64, bx: f64, bz: f64) -> bool {
    let k0 = (az.min(bz) / P).ceil();
    let k1 = (az.max(bz) / P).floor();
    let mut k = k0;
    while k <= k1 {
        let t = (k * P - az) / (bz - az);
        if (0.0..=1.0).contains(&t) && (ax + (bx - ax) * t).abs() > DOOR_W / 2.0 - 0.02 {
            return false;
        }
        k += 1.0;
    }
    true
}
/// is (tx, tz) on screen from (px, pz) looking along yaw? (a generous cone)
fn in_cone(px: f64, pz: f64, yaw: f64, tx: f64, tz: f64, half_deg: f64) -> bool {
    let dx = tx - px;
    let dz = tz - pz;
    let l = hypot2(dx, dz);
    if l < 1e-6 {
        return true;
    }
    (-yaw.sin() * dx - yaw.cos() * dz) / l > (half_deg * PI / 180.0).cos()
}

impl LoopMode {
    pub fn new(deps: LoopDeps) -> LoopMode {
        let rng = SharedRng::new(deps.rng);
        let tx = deps.textures;
        let root = group();
        root.set_name("loop");
        root.set_visible(false);
        let mut owned_geos: Vec<Geo> = vec![];
        let mut owned_mats: Vec<Mat> = vec![];
        let mut owned_texs: Vec<Tex> = vec![];

        // ------------------------------------------------------ materials ---
        let canvas_tex = deps.canvas_tex.clone();
        let mut tex = |w: f64, h: f64, draw: &mut dyn FnMut(&mut Canvas, f64, f64)| -> Option<Tex> {
            canvas_tex.as_ref().map(|ct| {
                let t = (**ct)(w, h, false, draw);
                owned_texs.push(t.clone());
                t
            })
        };
        let wall_mat = lambert!({ map: tx.wall.clone() });
        let wall_dirty_mat = lambert!({ map: tx.wall_dirty.clone().or_else(|| tx.wall.clone()) });
        let floor_mat = lambert!({ map: tx.carpet.clone() });
        let ceil_mat = lambert!({ map: tx.ceiling.clone() });
        let paint_mat = lambert!({ vertexColors: true });
        let panel_mat = basic_mat!({ map: tx.panel.clone(), color: if tx.panel.is_some() { 0xffffff } else { 0xfff4d8 } });
        let pick_mat = lambert!({ vertexColors: true, emissive: 0x1a1a18 });
        let stain_tex = {
            let rr = rng.clone();
            tex(256.0, 128.0, &mut |ctx: &mut Canvas, w: f64, h: f64| {
                ctx.clear_rect(0, 0, w, h);
                let r = |a: f64, b: f64| rr.range(a, b);
                // left half: a pooled blotch with spatter
                for _ in 0..26 {
                    let x = 64.0 + r(-26.0, 26.0);
                    let y = 64.0 + r(-22.0, 22.0);
                    let rad = r(8.0, 30.0);
                    let g = ctx.create_radial_gradient(x, y, 1, x, y, rad);
                    g.add_color_stop(0, &format!("rgba(52,10,6,{})", r(0.35, 0.7)));
                    g.add_color_stop(0.7, "rgba(40,8,5,0.25)");
                    g.add_color_stop(1, "rgba(30,6,4,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(x - rad, y - rad, rad * 2.0, rad * 2.0);
                }
                for _ in 0..60 {
                    let a = r(0.0, 7.0);
                    let d = r(30.0, 60.0);
                    let s = r(0.8, 2.6);
                    ctx.set_fill_style(format!("rgba(48,9,6,{})", r(0.4, 0.8)));
                    ctx.begin_path();
                    ctx.arc(64.0 + a.cos() * d, 64.0 + a.sin() * d * 0.9, s, 0, 7);
                    ctx.fill();
                }
                // right half: runs and a smear from above
                for _ in 0..14 {
                    let x = 128.0 + r(22.0, 106.0);
                    let top = r(4.0, 30.0);
                    let len = r(30.0, 118.0);
                    let wd = r(1.5, 6.0);
                    let g = ctx.create_linear_gradient(0, top, 0, top + len);
                    g.add_color_stop(0, &format!("rgba(50,9,6,{})", r(0.55, 0.85)));
                    g.add_color_stop(1, "rgba(40,8,5,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(x - wd / 2.0, top, wd, len);
                    ctx.begin_path();
                    ctx.arc(x, top + len * 0.8, wd * 0.7, 0, 7);
                    ctx.fill();
                }
                let sm = ctx.create_radial_gradient(192, 24, 2, 192, 24, 56);
                sm.add_color_stop(0, "rgba(46,9,6,0.7)");
                sm.add_color_stop(1, "rgba(40,8,5,0)");
                ctx.set_fill_style(&sm);
                ctx.fill_rect(128, 0, 128, 70);
            })
        };
        let stain_mat = lambert!({ map: stain_tex.clone(), color: if stain_tex.is_some() { 0xffffff } else { 0x3a0a06 }, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
        let sign_tex = tex(256.0, 96.0, &mut |ctx: &mut Canvas, w: f64, h: f64| {
            ctx.set_fill_style("#111");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_fill_style("#0b8f3e");
            ctx.fill_rect(8, 8, w - 16.0, h - 16.0);
            ctx.set_fill_style("#f4fff4");
            ctx.set_font("bold 62px Helvetica, Arial, sans-serif");
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.fill_text("EXIT", w / 2.0 + 2.0, h / 2.0 + 3.0);
            let g = ctx.create_linear_gradient(0, 0, 0, h);
            g.add_color_stop(0, "rgba(255,255,255,0.14)");
            g.add_color_stop(1, "rgba(0,0,0,0.12)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(8, 8, w - 16.0, h - 16.0);
        });
        let sign_mat = basic_mat!({ map: sign_tex.clone(), color: if sign_tex.is_some() { 0xffffff } else { 0x0b8f3e } });
        let ghost_mat = lambert!({ color: 0xfff0b8, transparent: true, opacity: 0.1, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 });
        for m in [&wall_mat, &wall_dirty_mat, &floor_mat, &ceil_mat, &paint_mat, &panel_mat, &pick_mat, &stain_mat, &sign_mat, &ghost_mat] {
            owned_mats.push(m.clone());
        }

        // ---------------------------------------------------------- the shell ---
        // One room: floor from z = 0 to P, the walls and ceiling of the room
        // inside. Each room owns its half of both dividing walls, so tints never bleed.
        let og = &mut owned_geos;
        let shell_geo = {
            let mut b = Builder::new();
            let wx = |b: &mut Builder, xp: f64, za: f64, zb: f64, ya: f64, yb: f64, f: f64| {
                b.quad([[xp, ya, za], [xp, ya, zb], [xp, yb, zb], [xp, yb, za]], [f, 0.0, 0.0], |_x, y, z| wall_uv(z * f, y), 0xffffff);
            };
            let wz = |b: &mut Builder, zp: f64, xa: f64, xb: f64, ya: f64, yb: f64, f: f64| {
                b.quad([[xa, ya, zp], [xb, ya, zp], [xb, yb, zp], [xa, yb, zp]], [0.0, 0.0, f], |x, y, _z| wall_uv(-x * f, y), 0xffffff);
            };
            wx(&mut b, -W / 2.0, 0.0, P, 0.0, H, 1.0);
            wx(&mut b, W / 2.0, 0.0, P, 0.0, H, -1.0);
            for (zp, f) in [(Z_N, 1.0), (Z_F, -1.0)] {
                wz(&mut b, zp, -W / 2.0, -HW, 0.0, H, f);
                wz(&mut b, zp, HW, W / 2.0, 0.0, H, f);
                wz(&mut b, zp, -HW, HW, DHH, H, f);
            }
            b.build(og)
        };
        let floor_geo = Builder::new()
            .quad(
                [[-W / 2.0, 0.0, 0.0], [W / 2.0, 0.0, 0.0], [W / 2.0, 0.0, P], [-W / 2.0, 0.0, P]],
                [0.0, 1.0, 0.0],
                |x, _y, z| [x / 2.6, z / 2.6],
                0xffffff,
            )
            .build(og);
        let ceil_geo = Builder::new()
            .quad(
                [[-W / 2.0, H, Z_N], [W / 2.0, H, Z_N], [W / 2.0, H, Z_F], [-W / 2.0, H, Z_F]],
                [0.0, -1.0, 0.0],
                |x, _y, z| [(x - 0.3) / 1.2, (z - (P / 2.0 - 0.6)) / 1.2],
                0xffffff,
            )
            .build(og);
        let panel_geo = Builder::new()
            .quad(
                [
                    [-0.3, H - 0.012, P / 2.0 - 0.6],
                    [0.3, H - 0.012, P / 2.0 - 0.6],
                    [0.3, H - 0.012, P / 2.0 + 0.6],
                    [-0.3, H - 0.012, P / 2.0 + 0.6],
                ],
                [0.0, -1.0, 0.0],
                |x, _y, z| [(z - (P / 2.0 - 0.6)) / 1.2, (x + 0.3) / 0.6],
                0xffffff,
            )
            .build(og);
        // trim: doorway linings, casings, thresholds, baseboards, the panel's
        // frame, a light switch. Far rooms (seen only through several
        // doorways) get plain boxes instead of bevelled ones.
        let make_trim = |bevel: bool, og: &mut Vec<Geo>| -> Geo {
            let mut b = Builder::new();
            const WOOD: u32 = 0x6b5335;
            const BASE: u32 = 0x4a3c25;
            const METAL: u32 = 0x8c8577;
            const FRAME: u32 = 0xd2cdbd;
            let mut bx = |w: f64, h: f64, d: f64, x: f64, y: f64, z: f64, c: u32, r: f64| {
                b.add(if bevel { bevel_box(w, h, d, r) } else { box_geo!(w, h, d) }, &mt(x, y, z), c);
            };
            for (zc, f) in [(Z_N / 2.0, 1.0f64), (P - Z_N / 2.0, -1.0)] {
                let face = if f > 0.0 { Z_N } else { Z_F };
                for s in [-1.0f64, 1.0] {
                    bx(0.03, DHH, Z_N, s * (HW - 0.015), DHH / 2.0, zc, WOOD, 0.002);
                }
                bx(HW * 2.0, 0.03, Z_N, 0.0, DHH - 0.015, zc, WOOD, 0.002);
                for s in [-1.0f64, 1.0] {
                    bx(0.075, DHH + 0.075, 0.022, s * (HW + 0.0375), (DHH + 0.075) / 2.0, face + f * 0.011, WOOD, 0.004);
                }
                bx(HW * 2.0 + 0.15, 0.075, 0.022, 0.0, DHH + 0.0375, face + f * 0.011, WOOD, 0.004);
                bx(HW * 2.0, 0.012, Z_N, 0.0, 0.006, zc, METAL, 0.002);
                for s in [-1.0f64, 1.0] {
                    bx(
                        W / 2.0 - HW - 0.075,
                        0.1,
                        0.016,
                        s * (HW + 0.075 + (W / 2.0 - HW - 0.075) / 2.0),
                        0.05,
                        face + f * 0.008,
                        BASE,
                        0.003,
                    );
                }
            }
            for s in [-1.0f64, 1.0] {
                bx(0.016, 0.1, Z_F - Z_N, s * (W / 2.0 - 0.008), 0.05, P / 2.0, BASE, 0.003);
            }
            for s in [-1.0f64, 1.0] {
                bx(0.66, 0.035, 0.04, 0.0, H - 0.017, P / 2.0 + s * 0.62, FRAME, 0.004);
                bx(0.04, 0.035, 1.28, s * 0.32, H - 0.017, P / 2.0, FRAME, 0.004);
            }
            bx(0.075, 0.12, 0.012, HW + 0.3, 1.22, Z_N + 0.006, 0xe2dccb, 0.003);
            bx(0.012, 0.03, 0.01, HW + 0.3, 1.22, Z_N + 0.014, 0xf0ece0, 0.002);
            b.build(og)
        };
        let trim_geo = make_trim(true, og);
        let trim_far_geo = make_trim(false, og);

        // ------------------------------------------------------- the furniture ---
        let chair_geo = {
            let mut b = Builder::new();
            add_chair(&mut b, &Mat4::IDENTITY);
            b.build(og)
        };
        // desk, filing cabinet and wall clock, already at their places in the room
        let furn_geo = {
            let mut b = Builder::new();
            let room = |x: f64, v: f64, yaw: f64| mtr(x, 0.0, P / 2.0 + v, 0.0, yaw, 0.0);
            // a steel tanker desk with a laminate top, the monitor facing the chair
            let d = room(FURN.desk.x, FURN.desk.v, PI / 2.0);
            const STEEL: u32 = 0x6c7064;
            const LAM: u32 = 0x8a7456;
            const BEIGE: u32 = 0xbdb398;
            b.add(bevel_box(1.4, 0.035, 0.75, 0.008), &under(&d, &mt(0.0, 0.7225, 0.0)), LAM);
            b.add(bevel_box(0.42, 0.685, 0.68, 0.01), &under(&d, &mt(0.44, 0.3425, -0.02)), STEEL);
            for i in 0..3 {
                let i = i as f64;
                b.add(bevel_box(0.38, 0.2, 0.02, 0.006), &under(&d, &mt(0.44, 0.12 + i * 0.215, 0.325)), 0x787c70);
                b.add(bevel_box(0.12, 0.018, 0.025, 0.005), &under(&d, &mt(0.44, 0.19 + i * 0.215, 0.345)), 0x2c2c2a);
            }
            b.add(bevel_box(0.03, 0.685, 0.68, 0.006), &under(&d, &mt(-0.67, 0.3425, -0.02)), STEEL);
            b.add(bevel_box(0.9, 0.42, 0.02, 0.005), &under(&d, &mt(-0.2, 0.47, -0.33)), STEEL);
            b.add(bevel_box(0.2, 0.02, 0.2, 0.006), &under(&d, &mt(-0.25, 0.75, -0.12)), BEIGE);
            b.add(bevel_box(0.38, 0.33, 0.36, 0.03), &under(&d, &mt(-0.25, 0.925, -0.14)), BEIGE);
            b.add(bevel_box(0.3, 0.24, 0.01, 0.004), &under(&d, &mt(-0.25, 0.935, 0.04)), 0x1b211f);
            b.add(bevel_box(0.42, 0.025, 0.15, 0.006), &under(&d, &mt(-0.25, 0.752, 0.2)), BEIGE);
            b.add(bevel_box(0.21, 0.03, 0.3, 0.003), &under(&d, &mtr(0.38, 0.755, -0.05, 0.0, 0.12, 0.0)), 0xe4e0d2);
            b.add(bevel_box(0.18, 0.06, 0.2, 0.015), &under(&d, &mt(0.52, 0.77, -0.3)), 0x2a2826);
            b.add(cyl_geo!(0.04, 0.038, 0.09, 12), &under(&d, &mt(0.55, 0.785, 0.24)), 0xa84a34);
            // four-drawer filing cabinet against the right wall
            let c = room(FURN.cabinet.x, FURN.cabinet.v, -PI / 2.0);
            b.add(bevel_box(0.46, 1.32, 0.62, 0.012), &under(&c, &mt(0.0, 0.66, 0.0)), 0x8c8876);
            for i in 0..4 {
                let i = i as f64;
                b.add(bevel_box(0.42, 0.29, 0.02, 0.006), &under(&c, &mt(0.0, 0.18 + i * 0.315, 0.315)), 0x96927f);
                b.add(bevel_box(0.14, 0.02, 0.03, 0.006), &under(&c, &mt(0.0, 0.25 + i * 0.315, 0.335)), 0x34332f);
                b.add(bevel_box(0.08, 0.04, 0.006, 0.002), &under(&c, &mt(0.0, 0.29 + i * 0.315, 0.328)), 0xdad5c4);
            }
            // wall clock over the visitor's chair: ten past three, every lap
            let kc = mtr(W / 2.0 - 0.004, FURN.clock_y, P / 2.0 + FURN.clock_v, 0.0, -PI / 2.0, 0.0);
            b.add(cyl_geo!(0.16, 0.16, 0.045, 32), &under(&kc, &mtr(0.0, 0.0, 0.0225, PI / 2.0, 0.0, 0.0)), 0x2c2a26);
            b.add(cyl_geo!(0.145, 0.145, 0.01, 32), &under(&kc, &mtr(0.0, 0.0, 0.046, PI / 2.0, 0.0, 0.0)), 0xebe5d6);
            for i in 0..12 {
                let a = (i as f64 / 12.0) * PI * 2.0;
                b.add(
                    box_geo!(0.01, if i % 3 != 0 { 0.018 } else { 0.03 }, 0.004),
                    &under(&kc, &mtr(a.sin() * 0.122, a.cos() * 0.122, 0.053, 0.0, 0.0, -a)),
                    0x2a2824,
                );
            }
            // a hand pointing `ang` clockwise from twelve
            let mut hand = |len: f64, wd: f64, ang: f64| {
                b.add(
                    box_geo!(wd, len, 0.004),
                    &under(&kc, &mtr(ang.sin() * len * 0.4, ang.cos() * len * 0.4, 0.058, 0.0, 0.0, -ang)),
                    0x1a1917,
                );
            };
            hand(0.075, 0.012, ((3.0 + 10.0 / 60.0) / 12.0) * PI * 2.0);
            hand(0.11, 0.008, (10.0 / 60.0) * PI * 2.0);
            b.add(cyl_geo!(0.01, 0.01, 0.012, 10), &under(&kc, &mtr(0.0, 0.0, 0.062, PI / 2.0, 0.0, 0.0)), 0x1a1917);
            b.build(og)
        };
        // a snake plant in a terracotta pot (the same plant every room: a fixed seed)
        let plant_geo = {
            let mut b = Builder::new();
            b.add(cyl_geo!(0.17, 0.125, 0.3, 18), &mt(0.0, 0.15, 0.0), 0x8e5236);
            b.add(cyl_geo!(0.182, 0.182, 0.045, 18), &mt(0.0, 0.29, 0.0), 0x96593a);
            b.add(cyl_geo!(0.16, 0.16, 0.01, 18), &mt(0.0, 0.308, 0.0), 0x2e2219);
            let mut s: u64 = 91;
            let mut rr = || {
                s = (s * 1664525 + 1013904223) % 4294967296;
                s as f64 / 4294967296.0
            };
            let greens = [0x3b672d, 0x4f7c38, 0x345a27, 0x5b8440];
            for i in 0..11 {
                let h = 0.45 + rr() * 0.42;
                let a = (i as f64 / 11.0) * PI * 2.0 + rr() * 0.4;
                let tilt = 0.08 + rr() * 0.26;
                let mut m = Mat4::IDENTITY;
                m.make_translation(a.sin() * 0.05, 0.3, a.cos() * 0.05);
                let mut ry = Mat4::IDENTITY;
                ry.make_rotation_y(a);
                let mut rx = Mat4::IDENTITY;
                rx.make_rotation_x(tilt);
                let mut t2 = Mat4::IDENTITY;
                t2.make_translation(0.0, h / 2.0, 0.0);
                let mut sc = Mat4::IDENTITY;
                sc.make_scale(1.0, 1.0, 0.2);
                m.multiply(&ry).multiply(&rx).multiply(&t2).multiply(&sc);
                b.add(cyl_geo!(0.004, 0.048, h, 6), &m, greens[i % greens.len()]);
            }
            b.build(og)
        };
        // a plain panelled interior door on the wall plane z = 0, facing +z
        let door_geo = {
            let mut b = Builder::new();
            const WOOD: u32 = 0x6a5236;
            const LEAF: u32 = 0x7c5f40;
            const PANEL: u32 = 0x70553a;
            const BRASS: u32 = 0xb0914e;
            for s in [-1.0f64, 1.0] {
                b.add(bevel_box(0.075, 2.14, 0.025, 0.005), &mt(s * 0.535, 1.07, 0.0125), WOOD);
            }
            b.add(bevel_box(1.145, 0.075, 0.025, 0.005), &mt(0.0, 2.1775, 0.0125), WOOD);
            b.add(bevel_box(0.95, 2.1, 0.04, 0.006), &mt(0.0, 1.05, 0.012), LEAF);
            b.add(bevel_box(0.66, 0.8, 0.014, 0.012), &mt(0.0, 0.56, 0.036), PANEL);
            b.add(bevel_box(0.66, 0.9, 0.014, 0.012), &mt(0.0, 1.55, 0.036), PANEL);
            b.add(cyl_geo!(0.03, 0.03, 0.012, 14), &mtr(0.38, 1.0, 0.038, PI / 2.0, 0.0, 0.0), BRASS);
            b.add(bevel_box(0.13, 0.022, 0.022, 0.008), &mt(0.33, 1.0, 0.065), BRASS);
            b.build(og)
        };
        let sign_geo = Builder::new()
            .quad(
                [[-0.19, -0.07, 0.03], [0.19, -0.07, 0.03], [0.19, 0.07, 0.03], [-0.19, 0.07, 0.03]],
                [0.0, 0.0, 1.0],
                |x, y, _z| [(x + 0.19) / 0.38, (y + 0.07) / 0.14],
                0xffffff,
            )
            .build(og);
        // stains: a pool on the carpet, runs down the left wall, a smear on
        // the far wall, a leak above the desk (left half of the atlas: blots, right: runs)
        let stain_geo = {
            let mut b = Builder::new();
            let blot = |u: f64| u * 0.5;
            let runs = |u: f64| 0.5 + u * 0.5;
            let mut plane_q = |cx: f64, cy: f64, cz: f64, w: f64, h: f64, n: [f64; 3], ax: [f64; 3], uv_map: &dyn Fn(f64) -> f64| {
                // ax: the in-plane "right" axis; up = n × right
                let (ux, uy, uz) = (ax[0], ax[1], ax[2]);
                let (vx, vy, vz) = (n[1] * uz - n[2] * uy, n[2] * ux - n[0] * uz, n[0] * uy - n[1] * ux);
                let c = |s: f64, t: f64| {
                    [
                        cx + ux * s * w / 2.0 + vx * t * h / 2.0,
                        cy + uy * s * w / 2.0 + vy * t * h / 2.0,
                        cz + uz * s * w / 2.0 + vz * t * h / 2.0,
                    ]
                };
                let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
                let mut k = 0;
                b.quad(
                    [c(-1.0, -1.0), c(1.0, -1.0), c(1.0, 1.0), c(-1.0, 1.0)],
                    n,
                    |_, _, _| {
                        let q = uvs[k];
                        k += 1;
                        [uv_map(q[0]), q[1]]
                    },
                    0xffffff,
                );
            };
            let (ca, sa) = (0.5f64.cos(), 0.5f64.sin());
            plane_q(-0.45, 0.004, P / 2.0 + 0.55, 1.9, 1.3, [0.0, 1.0, 0.0], [ca, 0.0, sa], &blot);
            plane_q(-W / 2.0 + 0.004, 1.3, P / 2.0 + 1.35, 0.85, 1.6, [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], &runs);
            plane_q(-1.55, 1.05, Z_F - 0.004, 0.95, 1.2, [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], &runs);
            plane_q(-1.3, H - 0.004, P / 2.0 - 0.9, 1.2, 1.1, [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], &blot);
            b.build(og)
        };
        // the pickups, the same bottles and cells as on the levels
        let almond_geo = {
            let mut b = Builder::new();
            b.add(cyl_geo!(0.075, 0.075, 0.24, 14), &mt(0.0, 0.12, 0.0), 0x9fc4d8);
            b.add(cyl_geo!(0.04, 0.07, 0.05, 14), &mt(0.0, 0.265, 0.0), 0x9fc4d8);
            b.add(cyl_geo!(0.035, 0.035, 0.04, 12), &mt(0.0, 0.305, 0.0), 0x8a8a86);
            b.add(cyl_geo!(0.077, 0.077, 0.08, 14), &mt(0.0, 0.12, 0.0), 0xe6dfc8);
            b.build(og)
        };
        let battery_geo = {
            let mut b = Builder::new();
            b.add(cyl_geo!(0.05, 0.05, 0.16, 14), &mt(0.0, 0.08, 0.0), 0xc79b3d);
            b.add(cyl_geo!(0.051, 0.051, 0.05, 14), &mt(0.0, 0.135, 0.0), 0x2a2a28);
            b.add(cyl_geo!(0.02, 0.02, 0.022, 10), &mt(0.0, 0.171, 0.0), 0xdad3c2);
            b.build(og)
        };
        // the seated figure: a man in office clothes with nothing where his
        // face should be, on an office chair of its own (`bodies.bin`
        // "seated", `assets/src/creatures/body_seated.py`: sat on this chair)
        let figure_geo = {
            let mut b = Builder::new();
            add_chair(&mut b, &Mat4::IDENTITY);
            for m in crate::assets::bodies::body("seated").map(|d| d.meshes.as_slice()).unwrap_or(&[]) {
                if let Some(g) = crate::assets::bodies::body_geometry("seated", &m.name) {
                    b.add_painted(&g, &Mat4::IDENTITY);
                }
            }
            b.build(og)
        };
        let ghost_door_geo = plane_geo!(1.1, 2.18);
        og.push(ghost_door_geo.clone());

        // ---------------------------------------------------------- instances ---
        let mut meshes: Vec<(&'static str, Obj)> = vec![];
        let mut inst = |geo: &Geo, material: &Mat, cap: usize, name: &'static str| -> Obj {
            let m = instanced_mesh(geo, material, cap);
            m.set_name(&format!("loop:{}", name));
            m.set_frustum_culled(false);
            let white = Color::white();
            for i in 0..cap {
                m.set_color_at(i, &white);
            }
            m.set_count(0);
            root.add(&m);
            meshes.push((name, m.clone()));
            m
        };
        let m_wall = inst(&shell_geo, &wall_mat, NS, "wall");
        let m_wall_dirty = inst(&shell_geo, &wall_dirty_mat, NS, "wallDirty");
        let m_floor = inst(&floor_geo, &floor_mat, NS, "floor");
        let m_ceil = inst(&ceil_geo, &ceil_mat, NS, "ceiling");
        let m_trim = inst(&trim_geo, &paint_mat, NN, "trim");
        let m_trim_far = inst(&trim_far_geo, &paint_mat, NS, "trimFar");
        let m_panel = inst(&panel_geo, &panel_mat, NS, "panel");
        let m_stain = inst(&stain_geo, &stain_mat, NS, "stains");
        let m_furn = inst(&furn_geo, &paint_mat, NN, "furniture");
        let m_chair = inst(&chair_geo, &paint_mat, NN * 2, "chairs");
        let m_plant = inst(&plant_geo, &paint_mat, NN, "plant");
        let m_almond = inst(&almond_geo, &pick_mat, NN, "almond");
        let m_battery = inst(&battery_geo, &pick_mat, NN, "battery");
        let m_figure = inst(&figure_geo, &paint_mat, NS + 1, "figure");
        let m_door = inst(&door_geo, &paint_mat, NN + 1, "doors");
        let m_sign = inst(&sign_geo, &sign_mat, NN, "exitSign");
        let ghost_door = mesh(&ghost_door_geo, &ghost_mat);
        ghost_door.set_visible(false);
        ghost_door.rotation().y = -PI / 2.0;
        root.add(&ghost_door);

        LoopMode {
            group: root,
            st: LoopState::default(),
            fig: LoopFigure::default(),
            env: LoopEnv::default(),
            out: LoopOut::default(),
            meshes,
            ghost_door,
            m_wall,
            m_wall_dirty,
            m_floor,
            m_ceil,
            m_trim,
            m_trim_far,
            m_panel,
            m_stain,
            m_furn,
            m_chair,
            m_plant,
            m_almond,
            m_battery,
            m_figure,
            m_door,
            m_sign,
            owned_geos,
            owned_mats,
            owned_texs,
            win_g: 0,
            slot_z: [0.0; NS],
            slot_l: [0; NS],
            slot_r: [0; NS],
            slot_bright: [0.0; NS],
            slot_dist: [0.0; NS],
            rng,
        }
    }

    // -------------------------------------------------------------- helpers
    /// The look lap of local room offset o (o = 0…4 are the rooms of this lap).
    pub fn look_lap_of(&self, o: i32) -> i32 {
        0.max(self.st.lap + o.div_euclid(ROOMS))
    }
    fn reported_lap(&self) -> i32 {
        0.max(self.st.lap)
    }
    fn room_of_z(z: f64) -> i32 {
        clamp((z / P).floor(), 0.0, (ROOMS - 1) as f64) as i32
    }
    fn global_room(&self) -> i32 {
        self.st.lap * ROOMS + (self.st.z / P).floor() as i32
    }
    fn pick_key(lap: i32, room: i32) -> i32 {
        lap * 8 + room
    }
    fn fig_home(l: i32) -> f64 {
        l as f64 * STRIP + LOOP::FIGURE_ROOM as f64 * P + P / 2.0 + FURN.figure.v
    }
    fn fig_local_z(&self) -> f64 {
        self.fig.gz - self.st.lap as f64 * STRIP
    }
    fn entry_g(&self) -> i32 {
        self.st.entry_lap * ROOMS + LOOP::ENTRY_ROOM
    }

    /// Refill every instanced mesh for the window around the player.
    pub fn rebuild(&mut self) {
        for (_, m) in &self.meshes {
            m.set_count(0);
        }
        self.fig.inst = -1;
        self.win_g = self.global_room();
        let base = self.st.lap * ROOMS;
        for j in 0..NS {
            let i = j as i32 - K;
            let g = self.win_g + i;
            let r = imod(g, ROOMS);
            let l = 0.max(g.div_euclid(ROOMS));
            let lk = lap_look(l);
            let z0 = (g - base) as f64 * P;
            let zc = z0 + P / 2.0;
            self.slot_z[j] = zc as f32;
            self.slot_l[j] = l;
            self.slot_r[j] = r as i8;
            let (tr, tg, tb) = (lk.tint[0], lk.tint[1], lk.tint[2]);
            put(if lk.dirty { &self.m_wall_dirty } else { &self.m_wall }, 0.0, 0.0, z0, 0.0, tr, tg, tb);
            put(&self.m_floor, 0.0, 0.0, z0, 0.0, tr * 0.96, tg * 0.96, tb * 0.96);
            let cd = (if lk.red { 0.62 } else { 1.0 }) * (if lk.bare { 0.7 } else { 1.0 }); // nicotine and damp on the tiles
            put(&self.m_ceil, 0.0, 0.0, z0, 0.0, tr * cd, tg * cd * 0.92, tb * cd * 0.85);
            put(
                if (-KN..=KN).contains(&i) { &self.m_trim } else { &self.m_trim_far },
                0.0,
                0.0,
                z0,
                0.0,
                0.6 + 0.4 * tr,
                0.6 + 0.4 * tg,
                0.6 + 0.4 * tb,
            );
            put(&self.m_panel, 0.0, 0.0, z0, 0.0, lk.light[0], lk.light[1], lk.light[2]); // index j: the flicker writes it
            if lk.stains {
                put1(&self.m_stain, 0.0, 0.0, z0, 0.0);
            }
            if lk.figure && r == LOOP::FIGURE_ROOM && !(self.fig.active && self.fig.lap == l) {
                put1(&self.m_figure, FURN.figure.x, 0.0, zc + FURN.figure.v, FURN.figure.yaw);
            }
            if !(-KN..=KN).contains(&i) {
                continue;
            }
            // the near rooms are furnished
            if !lk.bare {
                put1(&self.m_furn, 0.0, 0.0, z0, 0.0);
                let (d, v) = (FURN.desk_chair, FURN.visit_chair);
                put1(&self.m_chair, d.x, 0.0, zc + d.v, if lk.turned { d.turned } else { d.yaw });
                put1(&self.m_chair, v.x, 0.0, zc + v.v, if lk.turned { v.turned } else { v.yaw });
                put1(&self.m_plant, if lk.turned { -FURN.plant_x } else { FURN.plant_x }, 0.0, zc + FURN.plant_v, 0.0);
            }
            if let Some(kind) = pickup_at(l, r)
                && !self.st.taken.contains(&Self::pick_key(l, r))
            {
                let f = kind.spot();
                let (x, y, v) = if lk.bare { (f.bx, f.by, f.bv) } else { (f.x, f.y, f.v) };
                put1(if kind == PickupKind::Almond { &self.m_almond } else { &self.m_battery }, x, y, zc + v, 0.3);
            }
            if lk.exit && r == LOOP::EXIT_ROOM {
                put1(&self.m_door, -W / 2.0, 0.0, zc + FURN.exit_door_v, PI / 2.0);
                put1(&self.m_sign, -W / 2.0, 2.36, zc + FURN.exit_door_v, PI / 2.0);
            }
        }
        // the side door you came in by, until you turn round; then only a
        // patch of paper that hasn't faded where it was
        let eg = self.entry_g();
        let ez = (eg - base) as f64 * P + P / 2.0 + FURN.entry_door_v;
        if self.st.entry_door && (eg - self.win_g).abs() <= KN {
            put1(&self.m_door, W / 2.0, 0.0, ez, -PI / 2.0);
        }
        self.ghost_door.set_visible(!self.st.entry_door && self.st.started && (eg - self.win_g).abs() <= KN);
        if self.ghost_door.visible() {
            self.ghost_door.position().set(W / 2.0 - 0.004, 1.09, ez);
        }
        if self.fig.active && ((self.fig.gz / P).floor() as i32 - self.win_g).abs() <= K {
            let fz = self.fig_local_z();
            self.fig.inst = put1(&self.m_figure, self.fig.x, 0.0, fz, self.fig.yaw) as i32;
        }
        for (_, m) in &self.meshes {
            m.instances_changed();
        }
    }

    // ------------------------------------------------------------ collision
    fn room_thing_hit(&self, o: i32, x: f64, z: f64, r: f64) -> bool {
        let l = self.look_lap_of(o);
        let lk = lap_look(l);
        let zc = o as f64 * P + P / 2.0;
        if !lk.bare {
            let (dk, c) = (FURN.desk, FURN.cabinet);
            if circle_rect(x, z, r, -W / 2.0, zc + dk.v - dk.w / 2.0, dk.x + dk.d / 2.0, zc + dk.v + dk.w / 2.0) {
                return true;
            }
            if circle_rect(x, z, r, c.x - c.d / 2.0, zc + c.v - c.w / 2.0, W / 2.0, zc + c.v + c.w / 2.0) {
                return true;
            }
            if circle(x, z, r, FURN.desk_chair.x, zc + FURN.desk_chair.v, CHAIR_R) {
                return true;
            }
            if circle(x, z, r, FURN.visit_chair.x, zc + FURN.visit_chair.v, CHAIR_R) {
                return true;
            }
            if circle(x, z, r, if lk.turned { -FURN.plant_x } else { FURN.plant_x }, zc + FURN.plant_v, PLANT_R) {
                return true;
            }
        }
        lk.figure
            && imod(o, ROOMS) == LOOP::FIGURE_ROOM
            && !(self.fig.active && self.fig.lap == l)
            && circle(x, z, r, FURN.figure.x, zc + FURN.figure.v, FIG_R)
    }
    /// the furniture and the figure (`thing_hit`)
    fn thing_hit(&self, x: f64, z: f64, r: f64) -> bool {
        let o = (z / P).floor() as i32;
        for d in -1..=1 {
            if self.room_thing_hit(o + d, x, z, r) {
                return true;
            }
        }
        self.fig.active && circle(x, z, r, self.fig.x, self.fig_local_z(), FIG_R)
    }
    /// Would a body of radius r at local (x, z)
    /// overlap a wall, furniture or the figure?
    pub fn blocked(&self, x: f64, z: f64, r: f64) -> bool {
        wall_hit(x, z, r) || self.thing_hit(x, z, r)
    }
    /// walls always hold; if you are somehow inside a piece of furniture you may walk out of it
    fn can_go(&self, x: f64, z: f64, inside: bool) -> bool {
        !wall_hit(x, z, BODY_R) && (inside || !self.thing_hit(x, z, BODY_R))
    }
    pub fn clear_line(&self, ax: f64, az: f64, bx: f64, bz: f64) -> bool {
        clear_line(ax, az, bx, bz)
    }
    fn sees(&self, tx: f64, tz: f64, max_d: f64, half_deg: f64) -> bool {
        hypot2(tx - self.st.x, tz - self.st.z) < max_d
            && in_cone(self.st.x, self.st.z, self.st.yaw, tx, tz, half_deg)
            && clear_line(self.st.x, self.st.z, tx, tz)
    }
    // any of the figure's width in sight counts: its middle and both shoulders
    fn sees_figure(&self, fx: f64, fz: f64, max_d: f64) -> bool {
        self.sees(fx, fz, max_d, 62.0) || self.sees(fx - 0.3, fz, max_d, 62.0) || self.sees(fx + 0.3, fz, max_d, 62.0)
    }

    // ------------------------------------------------------------ lifecycle
    /// Enter at lap `lap`, standing inside the side door of room 3.
    pub fn enter(&mut self, lap: i32) {
        let st = &mut self.st;
        st.lap = 0.max(lap);
        st.x = W / 2.0 - 0.75;
        st.z = LOOP::ENTRY_ROOM as f64 * P + P / 2.0 + FURN.entry_door_v;
        st.yaw = PI / 2.0;
        st.time = 0.0;
        st.entry_door = true;
        st.entry_lap = st.lap;
        st.entry_t = 0.0;
        st.entry_unseen = 0.0;
        st.taken.clear();
        st.seams = 0;
        st.started = true;
        self.fig.active = false;
        self.fig.seen = false;
        self.fig.sightings = 0;
        self.out.events.clear();
        self.rebuild();
        self.group.set_visible(true);
    }
    /// (tools) You on lap `lap`, `dist` m in front of where the seated figure
    /// sits in the third room (it faces back down the rooms), looking at it.
    pub fn face_figure(&mut self, lap: i32, dist: f64) {
        let fz = LOOP::FIGURE_ROOM as f64 * P + P / 2.0 + FURN.figure.v;
        self.place(lap, fz - dist, FURN.figure.x, Some(PI));
    }
    /// Put the player at (lap, z, x) directly (tests, tools).
    pub fn place(&mut self, lap: i32, z: f64, x: f64, yaw: Option<f64>) {
        let yaw = yaw.unwrap_or(self.st.yaw);
        self.st.lap = LAP_MIN.max(lap);
        self.st.z = z;
        self.st.x = x;
        self.st.yaw = yaw;
        if self.fig.active && self.fig.lap != self.reported_lap() {
            self.fig.active = false;
        }
        self.rebuild();
    }

    pub fn update(&mut self, dt: f64, inp: &LoopInput) -> LoopOut {
        self.out.events.clear();
        self.st.time += dt;
        self.st.yaw = inp.yaw;
        let mut dirty = false;
        // --- movement: slide along walls, per axis. Walls always hold; if you
        // are somehow inside a piece of furniture you may walk out of it ---
        let (sy, cy) = (inp.yaw.sin(), inp.yaw.cos());
        let dx = (-sy * inp.fwd + cy * inp.strafe) * inp.speed * dt;
        let dz = (-cy * inp.fwd - sy * inp.strafe) * inp.speed * dt;
        if truthy(dx) || truthy(dz) {
            let inside = self.thing_hit(self.st.x, self.st.z, BODY_R);
            if self.can_go(self.st.x + dx, self.st.z, inside) {
                self.st.x += dx;
            }
            if self.can_go(self.st.x, self.st.z + dz, inside) {
                self.st.z += dz;
            }
        }
        // --- the seam ---
        if self.st.z >= STRIP {
            self.st.z -= STRIP;
            self.st.lap += 1;
            self.st.seams += 1;
            dirty = true;
            let lap = self.reported_lap();
            self.out.events.push(LoopEvent::Seam { dir: 1, lap });
        } else if self.st.z < 0.0 {
            self.st.z += STRIP;
            if self.st.lap > LAP_MIN {
                self.st.lap -= 1;
            }
            self.st.seams += 1;
            dirty = true;
            let lap = self.reported_lap();
            self.out.events.push(LoopEvent::Seam { dir: -1, lap });
        }
        if self.global_room() != self.win_g {
            dirty = true;
        }
        let room = Self::room_of_z(self.st.z);
        let lap_r = self.reported_lap();
        let zc = room as f64 * P + P / 2.0;
        // --- pickups: only at lap ≥ 3, and each only once ---
        if let Some(kind) = pickup_at(lap_r, room)
            && !self.st.taken.contains(&Self::pick_key(lap_r, room))
        {
            let f = kind.spot();
            let bare = lap_look(lap_r).bare;
            let px = if bare { f.bx } else { f.x };
            let pz = zc + if bare { f.bv } else { f.v };
            let (ex, ez) = (px - self.st.x, pz - self.st.z);
            if ex * ex + ez * ez < 0.95 {
                self.st.taken.insert(Self::pick_key(lap_r, room));
                dirty = true;
                self.out.events.push(LoopEvent::Pickup { kind, lap: lap_r, room });
            }
        }
        // --- the side door you came in by: gone the moment it is behind you ---
        if self.st.entry_door {
            self.st.entry_t += dt;
            let ez = (self.entry_g() - self.st.lap * ROOMS) as f64 * P + P / 2.0 + FURN.entry_door_v;
            let seen =
                self.sees(W / 2.0, ez, 40.0, 60.0) || self.sees(W / 2.0, ez - 0.5, 40.0, 60.0) || self.sees(W / 2.0, ez + 0.5, 40.0, 60.0);
            self.st.entry_unseen = if seen { 0.0 } else { self.st.entry_unseen + dt };
            if self.st.entry_t > 0.5 && self.st.entry_unseen > 0.2 {
                self.st.entry_door = false;
                dirty = true;
                self.out.events.push(LoopEvent::EntryGone);
            }
        }
        // --- the figure ---
        self.out.touch = 0.0;
        if lap_r >= LOOP::FIGURE_LAP && (!self.fig.active || self.fig.lap != lap_r) {
            // it takes its seat in this lap's room 3 while you are not looking
            let home = Self::fig_home(self.st.lap) - self.st.lap as f64 * STRIP;
            let far = hypot2(FURN.figure.x - self.st.x, home - self.st.z) > 1.6;
            if (!self.fig.active || !self.fig.seen) && far {
                let f = &mut self.fig;
                f.active = true;
                f.lap = lap_r;
                f.gz = Self::fig_home(self.st.lap);
                f.x = FURN.figure.x;
                f.yaw = FURN.figure.yaw;
                f.seen = false;
                f.unseen_t = 0.0;
                f.moved = 0.0;
                dirty = true;
            }
        }
        if self.fig.active {
            let fz = self.fig_local_z();
            let d = hypot2(self.fig.x - self.st.x, fz - self.st.z);
            let drawn = ((self.fig.gz / P).floor() as i32 - self.global_room()).abs() <= K;
            let seen = drawn && self.sees_figure(self.fig.x, fz, 2.4 / 0.02f64.max(self.env.fog));
            if seen {
                if !self.fig.seen {
                    self.fig.sightings += 1;
                    let moved = self.fig.moved;
                    self.out.events.push(if moved > 0.3 {
                        LoopEvent::FigureNearer { dist: d, moved }
                    } else {
                        LoopEvent::FigureSeen { dist: d, moved }
                    });
                    self.fig.moved = 0.0;
                }
                self.fig.unseen_t = 0.0;
            } else {
                self.fig.unseen_t += dt;
                // it only moves while nobody is looking, and it is patient
                if self.fig.unseen_t > 0.5 && d > 0.74 && drawn {
                    let step = 1.25 * dt;
                    let fr = (fz / P).floor() as i32;
                    let pr = (self.st.z / P).floor() as i32;
                    let want_x = if fr == pr { clamp(self.st.x, -0.9, 1.3) } else { 0.0 };
                    let mut nx = self.fig.x + clamp(want_x - self.fig.x, -step, step);
                    let mut nz = fz + clamp(self.st.z - fz, -step, step);
                    // through the walls only by the doorways
                    let k = engine::math::round_half_up(nz / P);
                    if (nz - k * P).abs() < TH / 2.0 + FIG_R && nx.abs() > DOOR_W / 2.0 - FIG_R + 0.02 {
                        nz = fz;
                    }
                    // …and never back into the laps where it does not exist yet
                    let lapz = self.st.lap as f64 * STRIP;
                    let floor_z = LOOP::FIGURE_LAP as f64 * STRIP + 0.5;
                    if nz + lapz < floor_z {
                        nz = fz.max(floor_z - lapz);
                    }
                    if hypot2(nx - self.st.x, nz - self.st.z) < 0.74 {
                        nx = self.fig.x;
                        nz = fz;
                    }
                    let moved = hypot2(nx - self.fig.x, nz - fz);
                    if moved > 0.0 {
                        self.fig.x = nx;
                        self.fig.gz = nz + lapz;
                        self.fig.moved += moved;
                        self.fig.yaw = (self.st.x - nx).atan2(self.st.z - nz);
                        if self.fig.inst >= 0 {
                            set_at(&self.m_figure, self.fig.inst as usize, self.fig.x, 0.0, nz, self.fig.yaw);
                            self.m_figure.instances_changed();
                        } else {
                            dirty = true;
                        }
                    }
                }
            }
            self.fig.seen = seen;
            if hypot2(self.fig.x - self.st.x, self.fig_local_z() - self.st.z) < 0.82 {
                self.out.touch = dt;
            }
        }
        // any other figure (a lap where it has not moved) is just as bad to touch
        {
            let o = (self.st.z / P).floor() as i32;
            for dd in -1..=1 {
                let oo = o + dd;
                let l = self.look_lap_of(oo);
                if !lap_look(l).figure || imod(oo, ROOMS) != LOOP::FIGURE_ROOM || (self.fig.active && self.fig.lap == l) {
                    continue;
                }
                if hypot2(FURN.figure.x - self.st.x, oo as f64 * P + P / 2.0 + FURN.figure.v - self.st.z) < 0.82 {
                    self.out.touch = dt;
                }
            }
        }
        if dirty {
            self.rebuild();
        }

        // --- the way out: only in room 1, only where nothing has drifted ---
        self.out.near_exit = lap_look(self.look_lap_of(room)).exit
            && room == LOOP::EXIT_ROOM
            && hypot2(-W / 2.0 + 0.45 - self.st.x, zc + FURN.exit_door_v - self.st.z) < 1.15;

        // --- atmosphere: blended across the seam so the jump never shows ---
        let e = STRIP - self.st.z;
        let s = self.st.z;
        let mut lam = self.st.lap as f64;
        if e < P {
            lam += 0.5 * (1.0 - e / P);
        } else if s < P {
            lam -= 0.5 * (1.0 - s / P);
        }
        self.compute_env(0f64.max(lam));

        // --- lamps: panels flicker; the nearest get the game's pooled lights ---
        for j in 0..NS {
            let l = self.slot_l[j];
            let lk = lap_look(l);
            let f = self.panel_state(&lk, l, self.slot_r[j] as i32);
            self.slot_bright[j] = (lk.bright * f) as f32;
            self.slot_dist[j] = (self.slot_z[j] as f64 - self.st.z).abs() as f32;
            self.m_panel.set_color_at(j, &Color::rgb_linear(lk.light[0] * f, lk.light[1] * f, lk.light[2] * f));
        }
        let mut n = 0;
        for _pass in 0..self.out.lights.len() {
            let mut best: i32 = -1;
            let mut bd = 1e9f64;
            for j in 0..NS {
                if (self.slot_dist[j] as f64) < bd {
                    bd = self.slot_dist[j] as f64;
                    best = j as i32;
                }
            }
            if best < 0 {
                break;
            }
            let best = best as usize;
            let lk = lap_look(self.slot_l[best]);
            let lg = &mut self.out.lights[n];
            n += 1;
            lg.x = 0.0;
            lg.y = H - 0.06;
            lg.z = self.slot_z[best] as f64;
            lg.i = self.slot_bright[best] as f64 * smooth(15.6, 10.4, bd);
            lg.r = lk.light[0];
            lg.g = lk.light[1];
            lg.b = lk.light[2];
            self.slot_dist[best] = 1e9;
        }
        self.out.light_count = n;
        self.out.x = self.st.x;
        self.out.z = self.st.z;
        self.out.lap = lap_r;
        self.out.room = room;
        self.out.env = self.env;
        self.out.clone()
    }
    /// a panel's brightness 0…1: steady, flickering (lap ≥ 3) or dead (lap ≥ 6).
    /// A function of (time, lap, room) only, so every copy of a room agrees.
    fn panel_state(&self, lk: &LapLook, l: i32, r: i32) -> f64 {
        if lk.bare && hash01((l * 53 + r * 11 + 5) as f64) < 0.6f64.min(0.25 + 0.08 * (l - 6) as f64) {
            return 0.0;
        }
        if !truthy(lk.flicker) {
            return 1.0;
        }
        let rate = 2.2 + lk.flicker * 2.5;
        let t = self.st.time * rate + hash01((l * 31 + r * 7) as f64) * 97.0;
        let cell = t.floor();
        let h = hash01(cell * 13.0 + (l * 101) as f64 + (r * 3) as f64);
        if h < lk.flicker * 0.32 {
            return 0.06 + 0.3 * hash01(cell * 7.0 + r as f64);
        }
        1.0
    }
    fn compute_env(&mut self, lam: f64) {
        let a = lam.floor();
        let f = lam - a;
        let ai = a as i32;
        let ra = env_row(ai);
        let rb = env_row(ai + 1);
        let env = &mut self.env;
        env.fog_r = lerp(ra[0], rb[0], f);
        env.fog_g = lerp(ra[1], rb[1], f);
        env.fog_b = lerp(ra[2], rb[2], f);
        env.fog = lerp(ra[3], rb[3], f);
        env.amb = lerp(ra[4], rb[4], f);
        env.hemi = lerp(ra[5], rb[5], f);
        env.hum = lerp(ra[6], rb[6], f);
        env.drain = lerp(ra[7], rb[7], f);
        env.glow = lerp(ra[8], rb[8], f);
        let (la, lb) = (lap_look(ai).light, lap_look(ai + 1).light);
        let (ta, tb) = (lap_look(ai).tint, lap_look(ai + 1).tint);
        env.sky_r = lerp(la[0], lb[0], f);
        env.sky_g = lerp(la[1], lb[1], f);
        env.sky_b = lerp(la[2], lb[2], f);
        env.ground_r = 0.42 * lerp(ta[0], tb[0], f);
        env.ground_g = 0.36 * lerp(ta[1], tb[1], f);
        env.ground_b = 0.22 * lerp(ta[2], tb[2], f);
        env.hum_cents = loop_hum_cents(lam);
        env.reverb = 6f64.min(engine::math::round_half_up(lam)) as i32;
        env.lam = lam;
    }

    /// Frees every geometry, material and texture the module made
    /// (the game's textures are left alone) and empties the group.
    pub fn dispose(&mut self) {
        self.group.set_visible(false);
        for (_, m) in &self.meshes {
            self.group.remove(m);
        }
        self.group.remove(&self.ghost_door);
        for g in &self.owned_geos {
            g.dispose();
        }
        for t in &self.owned_texs {
            t.dispose();
        }
        self.owned_geos.clear();
        self.owned_mats.clear();
        self.owned_texs.clear();
        self.meshes.clear();
    }

    // -------------------------------------------------------------- getters
    /// What is drawn, for tests: how many of each part (+ `ghostDoor`).
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        let mut c: Vec<(&'static str, usize)> = self.meshes.iter().map(|(n, m)| (*n, m.instance_count())).collect();
        c.push(("ghostDoor", if self.ghost_door.visible() { 1 } else { 0 }));
        c
    }
    /// the lap state
    pub fn state(&self) -> &LoopState {
        &self.st
    }
    /// the figure
    pub fn figure(&self) -> &LoopFigure {
        &self.fig
    }
    /// the reported lap (never below 0)
    pub fn lap(&self) -> i32 {
        self.reported_lap()
    }
    /// the room the player is in (0…4)
    pub fn room(&self) -> i32 {
        Self::room_of_z(self.st.z)
    }
    /// the atmosphere
    pub fn env(&self) -> &LoopEnv {
        &self.env
    }
    /// the last `update()` result
    pub fn last_out(&self) -> &LoopOut {
        &self.out
    }
    /// the draw calls the window needs (non-empty instanced parts, plus the ghost door)
    pub fn draw_calls(&self) -> usize {
        self.meshes.iter().filter(|(_, m)| m.instance_count() > 0).count() + if self.ghost_door.visible() { 1 } else { 0 }
    }
    /// Local position in front of the exit door (room 1), for tools
    pub fn exit_door_at(&self) -> (f64, f64) {
        (-W / 2.0 + 0.45, LOOP::EXIT_ROOM as f64 * P + P / 2.0 + FURN.exit_door_v)
    }
}
