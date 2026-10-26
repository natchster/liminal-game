//! The record types the core shares between subsystems. Records that only
//! one module uses are defined in that module.

use engine::prelude::*;
use std::collections::HashMap;

/// A ceiling light panel (laid by `lay_panels`, driven by `update_lights`)
#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub wx: f64,
    pub wz: f64,
    pub bright: f64,
    pub flicker: f64,
    pub seed: f64,
    /// 0 in a pristine room, 1 in a ruined one
    pub rot: f64,
    pub dead: bool,
    pub switched_off: bool,
    /// Ceiling height of the panel (0 = unset → `WALL_H`)
    pub y: f64,
    /// Per-panel brightness breathing: LFO frequency, phase and weather phase offset
    pub lfo_freq: f64,
    pub lfo_phase: f64,
    pub lfo_phase_offset: f64,
    /// Height boost of the panel's light (0 = unset → 1)
    pub h_boost: f64,
    /// Any other per-panel numbers, by key
    pub f: HashMap<String, f64>,
}

/// Per-region surface choice and dressing: wall / ceiling / floor material
/// indices, `RF_*` flags, grime level and light level
#[derive(Clone, Debug, Default)]
pub struct RegionInfo {
    pub wall: usize,
    pub ceil: usize,
    pub floor: usize,
    pub flags: u32,
    pub grime: i32,
    /// (None where the region has no explicit light level)
    pub lit: Option<f64>,
}

/// A floating pickup (almond water bottle or battery)
#[derive(Clone, Debug)]
pub struct Pickup {
    pub g: Obj,
    pub taken: bool,
    pub base: f64,
    pub phase: f64,
}

/// A collectable paper pile
#[derive(Clone, Debug)]
pub struct PaperPile {
    pub g: Obj,
    pub taken: bool,
}

/// A lore note in the level (`id` indexes the lore entries)
#[derive(Clone, Debug)]
pub struct LoreNote {
    pub g: Obj,
    pub taken: bool,
    pub id: usize,
    /// The mark on the nearest wall (an arrow, or READ), if there is a wall
    pub clue: Option<Obj>,
}

/// A paper the player dropped on the floor to mark a trail
#[derive(Clone, Debug)]
pub struct PaperMarker {
    pub mesh: Obj,
    pub world_x: f64,
    pub world_z: f64,
}

/// A hole in the floor (centre and radius)
#[derive(Clone, Copy, Debug, Default)]
pub struct FloorHole {
    pub x: f64,
    pub z: f64,
    pub r: f64,
}

/// The pool room's extent, water and splash heights, and tile bounds
#[derive(Clone, Copy, Debug, Default)]
pub struct PoolBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub water_y: f64,
    pub splash_y: f64,
    pub tiles: [i32; 4],
}

/// The level's safe room (centre, radius, candle flames)
#[derive(Clone, Debug, Default)]
pub struct SafeRoom {
    pub x: f64,
    pub z: f64,
    pub r: f64,
    pub discovered: bool,
    pub flames: Vec<Obj>,
    pub f: HashMap<String, f64>,
}

/// The exit door: position, facing normal and door object
#[derive(Clone, Debug, Default)]
pub struct ExitData {
    pub pos: Vec3,
    pub normal: Vec3,
    pub door: Option<Obj>,
}

/// The TV prop: its light, world position and facing
#[derive(Clone, Debug)]
pub struct TvData {
    pub light: Obj,
    pub world: Vec3,
    pub acc: f64,
    pub normal: Vec3,
}

/// The payphone prop: position, ring state and handset
#[derive(Clone, Debug)]
pub struct PhoneData {
    pub world: Vec3,
    pub ringing: bool,
    pub answered: bool,
    pub ring_t: f64,
    pub normal: Vec3,
    pub handset: Option<Obj>,
    pub f: HashMap<String, f64>,
}

/// The wall clock prop: second-hand pivot, position and tick state
#[derive(Clone, Debug)]
pub struct ClockData {
    pub sec_pivot: Obj,
    pub world: Vec3,
    pub acc: f64,
    pub tock: bool,
    pub normal: Vec3,
}

/// The mannequin figure `place_props` leaves (see `place_props`)
#[derive(Clone, Debug)]
pub struct ManData {
    pub g: Obj,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// The broken elevator prop and its scrape / peek state
#[derive(Clone, Debug)]
pub struct BrokenElevator {
    pub light: Obj,
    pub warn_bar: Option<Obj>,
    pub world: Vec3,
    pub normal: Vec3,
    pub next_scrape: f64,
    pub peered: bool,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// The roaming vacuum bot
#[derive(Clone, Debug)]
pub struct VacuumBot {
    pub g: Obj,
    pub world_pos: Vec3,
    pub dir: f64,
    pub timer: f64,
    pub follow_t: f64,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// A wall light switch (built by `place_props`)
#[derive(Clone, Debug, Default)]
pub struct LightSwitch {
    pub g: Option<Obj>,
    pub x: f64,
    pub z: f64,
    pub on: bool,
    pub faulty: bool,
    /// Indices into `panels` this switch controls
    pub panels: Vec<usize>,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// A haunted candle's state
#[derive(Clone, Debug, Default)]
pub struct CandleState {
    pub lit: bool,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// A cabinet with openable drawers
#[derive(Clone, Debug)]
pub struct DrawerProp {
    pub cabinet: Obj,
    pub drawers: Vec<std::rc::Rc<std::cell::RefCell<Drawer>>>,
}
#[derive(Clone, Debug)]
pub struct Drawer {
    pub mesh: Obj,
    pub opened: bool,
    pub content_type: String,
    pub content_taken: bool,
    pub use_pos: Vec3,
    pub f: HashMap<String, f64>,
}

/// A floor arrow of the guide path (fades out over its life)
#[derive(Clone, Debug)]
pub struct GuideArrow {
    pub mesh: Obj,
    pub life: f64,
    pub start_y: f64,
    pub index: usize,
}

/// A pooled effect sprite (scuffs, sparkles, ripples)
#[derive(Clone, Debug)]
pub struct PooledFx {
    pub obj: Obj,
    pub t: f64,
}

/// One logged ambient event (tag, time, and where it came from)
#[derive(Clone, Debug, Default)]
pub struct AmbientLogEntry {
    pub tag: String,
    pub at: f64,
    pub dist: Option<f64>,
    pub bearing: Option<String>,
    pub pan: Option<f64>,
}

/// One of a level's signature sounds: what it is, the fixed spot it comes
/// from and its rhythm (`Game::pick_signatures`)
#[derive(Clone, Debug)]
pub struct AmbSignature {
    /// sfx-table name (one of `AMBIENT_EVENTS`)
    pub name: &'static str,
    /// Where it sounds from
    pub at: Vec3,
    /// Base period, s
    pub period: f64,
    /// Countdown to the next time it sounds, s
    pub t: f64,
}

/// Where `blip` / `noise_burst` send their voices while `Game::sfx_route` is
/// set (`Game::play_at`)
#[derive(Clone)]
pub struct SfxRoute {
    /// A spatial voice slot's input, instead of master
    pub dest: Option<AudioNode>,
    /// Stereo pan for voices without one of their own (no `dest`)
    pub pan: f64,
    /// Level multiplier for every voice
    pub gain: f64,
}

/// The nearest humming panel: index into `panels` and distance
#[derive(Clone, Copy, Debug, Default)]
pub struct HumPanel {
    pub p: usize,
    pub d: f64,
}

/// A creature. The common fields are typed; per-type state the rigs keep
/// (gait phases, pose blends, hair springs, ...) lives in the maps.
#[derive(Clone, Debug)]
pub struct Entity {
    /// 'smiler' | 'hound' | 'watcher' | 'staff' | 'shadow'
    pub kind: String,
    pub g: Obj,
    pub face: Option<Obj>,
    pub face_y: f64,
    pub eye_y_offset: f64,
    /// Named bones / meshes of the rig
    pub parts: HashMap<String, Obj>,
    pub yaw: f64,
    /// 'wander' | 'hunt' | 'stalking' | ...
    pub state: String,
    pub speed: f64,
    pub w_field: Option<std::rc::Rc<Vec<i32>>>,
    pub w_target: Option<(i32, i32)>,
    pub w_timer: f64,
    pub lose_t: f64,
    pub freeze: f64,
    pub stare_t: f64,
    pub step_acc: f64,
    pub unseen_t: f64,
    pub vx: f64,
    pub vz: f64,
    pub dying: f64,
    pub observed: bool,
    /// Numeric per-entity state, by key (`poseYaw`, `gPh`, `hairVX`, ...)
    pub num: HashMap<String, f64>,
    /// Boolean per-entity state
    pub flag: HashMap<String, bool>,
    /// Vectors, by key (`aimFinal`, ...)
    pub vec: HashMap<String, Vec3>,
    /// Objects, by key (`sprite`, ...; voice handles live in `Game::ent_ext`)
    pub obj: HashMap<String, Obj>,
    /// Strings, by key
    pub s: HashMap<String, String>,
}
impl Entity {
    pub fn new(kind: &str, g: Obj) -> Entity {
        Entity {
            kind: kind.to_string(),
            g,
            face: None,
            face_y: 0.0,
            eye_y_offset: 0.0,
            parts: HashMap::new(),
            yaw: 0.0,
            state: "wander".into(),
            speed: 0.0,
            w_field: None,
            w_target: None,
            w_timer: 0.0,
            lose_t: 0.0,
            freeze: 0.0,
            stare_t: 0.0,
            step_acc: 0.0,
            unseen_t: 0.0,
            vx: 0.0,
            vz: 0.0,
            dying: 0.0,
            observed: false,
            num: HashMap::new(),
            flag: HashMap::new(),
            vec: HashMap::new(),
            obj: HashMap::new(),
            s: HashMap::new(),
        }
    }
    /// The entity's position (its group's position)
    pub fn pos(&self) -> Vec3 {
        self.g.pos()
    }
    pub fn n(&self, k: &str) -> f64 {
        self.num.get(k).copied().unwrap_or(0.0)
    }
    pub fn set_n(&mut self, k: &str, v: f64) {
        self.num.insert(k.to_string(), v);
    }
    pub fn fl(&self, k: &str) -> bool {
        self.flag.get(k).copied().unwrap_or(false)
    }
    pub fn set_fl(&mut self, k: &str, v: bool) {
        self.flag.insert(k.to_string(), v);
    }
    pub fn part(&self, k: &str) -> Option<Obj> {
        self.parts.get(k).cloned()
    }
}

/// The Showroom's state: day / night phase, hiding, the board and the music
#[derive(Clone, Debug)]
pub struct StoreState {
    pub on: bool,
    pub t: f64,
    pub phase: String,
    pub cycle: f64,
    pub hides: Vec<StoreHide>,
    pub hidden: bool,
    pub lights_out: f64,
    pub panels_off: bool,
    pub board_tex: Option<Tex>,
    pub board_canvas: Option<std::rc::Rc<RefCell<Canvas>>>,
    pub board_text: String,
    pub arrow_mat: Option<Mat>,
    pub music: Option<crate::base::records::Voice>,
    pub f: HashMap<String, f64>,
}
impl Default for StoreState {
    fn default() -> Self {
        StoreState {
            on: false,
            t: 0.0,
            phase: "day".into(),
            cycle: 0.0,
            hides: vec![],
            hidden: false,
            lights_out: 0.0,
            panels_off: false,
            board_tex: None,
            board_canvas: None,
            board_text: String::new(),
            arrow_mat: None,
            music: None,
            f: HashMap::new(),
        }
    }
}
use std::cell::RefCell;
/// A hiding spot in the Showroom
#[derive(Clone, Debug, Default)]
pub struct StoreHide {
    pub x: f64,
    pub z: f64,
    pub eye: Vec3,
    pub f: HashMap<String, f64>,
}

/// A handle to a running sound, so it can be stopped.
#[derive(Clone)]
pub struct Voice {
    pub nodes: Vec<AudioNode>,
    pub gain: Option<AudioNode>,
}
impl std::fmt::Debug for Voice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Voice({} nodes)", self.nodes.len())
    }
}
impl Voice {
    pub fn stop(&self, t: f64) {
        for n in &self.nodes {
            n.stop(t);
        }
    }
}

/// The Borrowed House's state: lamps, the nest and its light
#[derive(Clone, Debug, Default)]
pub struct HouseState {
    pub on: bool,
    pub lamps: Vec<HouseLamp>,
    pub nest: Option<(f64, f64)>,
    pub nest_light: Option<Obj>,
    pub out: bool,
    pub beat_t: f64,
}
/// A lamp in the Borrowed House
#[derive(Clone, Debug)]
pub struct HouseLamp {
    pub g: Obj,
    pub out: bool,
    pub light: Option<Obj>,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
}

/// Audio timers while in the Stairwell
#[derive(Clone, Debug, Default)]
pub struct StairAudio {
    pub next_phrase: f64,
    pub next_knock: f64,
    pub heart_t: f64,
    pub step_acc: f64,
}

/// Audio timers while in the Loop
#[derive(Clone, Copy, Debug, Default)]
pub struct LoopAudio {
    pub step_acc: f64,
    pub touch_t: f64,
}

/// The resolution governor: max / current / min pixel ratio, the frame-time
/// EMA, slow / fast timers and the number of changes
#[derive(Clone, Copy, Debug)]
pub struct Quality {
    pub max_pr: f64,
    pub pr: f64,
    pub min_pr: f64,
    pub ema: f64,
    pub slow_t: f64,
    pub fast_t: f64,
    pub changes: u32,
}
impl Default for Quality {
    fn default() -> Self {
        Quality { max_pr: 2.0, pr: 2.0, min_pr: 0.75, ema: 1.0 / 60.0, slow_t: 0.0, fast_t: 0.0, changes: 0 }
    }
}

/// Distance culling of props: (object, bounding radius) list, timer and
/// hidden count
#[derive(Clone, Debug, Default)]
pub struct PropCull {
    pub list: Vec<(Obj, f64)>,
    pub t: f64,
    pub hidden: usize,
}

/// Tracing of seeded draws: on/off, draw count and named marks
#[derive(Clone, Debug, Default)]
pub struct RandTrace {
    pub on: bool,
    pub n: u64,
    pub marks: Vec<(String, u64)>,
}

/// Touch movement input (unused on macOS; kept for the shared code paths)
#[derive(Clone, Copy, Debug, Default)]
pub struct TouchInput {
    pub active: bool,
    pub x: f64,
    pub z: f64,
    pub sprint: bool,
    pub crouch: bool,
}

/// A counter per weather
#[derive(Clone, Copy, Debug, Default)]
pub struct WeatherTally {
    pub wet: f64,
    pub still: f64,
    pub electric: f64,
    pub haunted: f64,
    pub derelict: f64,
    pub normal: f64,
}
impl WeatherTally {
    pub fn bump(&mut self, key: &str) {
        match key {
            "wet" => self.wet += 1.0,
            "still" => self.still += 1.0,
            "electric" => self.electric += 1.0,
            "haunted" => self.haunted += 1.0,
            "derelict" => self.derelict += 1.0,
            _ => self.normal += 1.0,
        }
    }
    pub fn get(&self, key: &str) -> f64 {
        match key {
            "wet" => self.wet,
            "still" => self.still,
            "electric" => self.electric,
            "haunted" => self.haunted,
            "derelict" => self.derelict,
            _ => self.normal,
        }
    }
}

/// Last fire time of each rare event cause
#[derive(Clone, Copy, Debug, Default)]
pub struct RareFire {
    pub knock: f64,
    pub door: f64,
    pub ding: f64,
    pub voice: f64,
    pub r#static: f64,
    pub shift: f64,
    pub pattern: f64,
    pub skip: f64,
}

/// The interactive spot of a hiding place / anything with x, z, r
#[derive(Clone, Copy, Debug, Default)]
pub struct Spot {
    pub x: f64,
    pub z: f64,
    pub r: f64,
}

/// A wall-adjacent spot (see `wall_adj_spot`)
#[derive(Clone, Copy, Debug, Default)]
pub struct WallSpot {
    pub pos: Vec3,
    pub normal: Vec3,
    pub yaw: f64,
}
