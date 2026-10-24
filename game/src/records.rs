//! The JS object literals the core shares between subsystems, as structs.
//! Field names are snake_case of the JS fields. Records that only one module
//! uses are defined in that module.

use engine::prelude::*;
use std::collections::HashMap;

/// `panels[i]` — a ceiling light panel (layPanels / updateLights)
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
    /// `p.y` (ceiling height of the panel; 0 = unset → WALL_H)
    pub y: f64,
    /// R100 per-panel breath: `lfoFreq`, `lfoPhase`, `lfoPhaseOffset`
    pub lfo_freq: f64,
    pub lfo_phase: f64,
    pub lfo_phase_offset: f64,
    /// `p.hBoost` (0 = unset → 1)
    pub h_boost: f64,
    /// any other per-panel numbers the JS hangs on it
    pub f: HashMap<String, f64>,
}

/// `regionInfo[i]`
#[derive(Clone, Debug, Default)]
pub struct RegionInfo {
    pub wall: usize,
    pub ceil: usize,
    pub floor: usize,
    pub flags: u32,
    pub grime: i32,
    /// `lit` (None where JS leaves it undefined)
    pub lit: Option<f64>,
}

/// `almonds[i]` / `batteryItems[i]` — floating pickups
#[derive(Clone, Debug)]
pub struct Pickup { pub g: Obj, pub taken: bool, pub base: f64, pub phase: f64 }

/// `papers[i]`
#[derive(Clone, Debug)]
pub struct PaperPile { pub g: Obj, pub taken: bool }

/// `loreNotes[i]`
#[derive(Clone, Debug)]
pub struct LoreNote { pub g: Obj, pub taken: bool, pub id: usize }

/// `paperMarkers[i]`
#[derive(Clone, Debug)]
pub struct PaperMarker { pub mesh: Obj, pub world_x: f64, pub world_z: f64 }

/// `floorHoles[i]`
#[derive(Clone, Copy, Debug, Default)]
pub struct FloorHole { pub x: f64, pub z: f64, pub r: f64 }

/// `poolBounds`
#[derive(Clone, Copy, Debug, Default)]
pub struct PoolBounds { pub min_x: f64, pub max_x: f64, pub min_z: f64, pub max_z: f64, pub water_y: f64, pub splash_y: f64, pub tiles: [i32; 4] }

/// `safeRoom`
#[derive(Clone, Debug, Default)]
pub struct SafeRoom { pub x: f64, pub z: f64, pub r: f64, pub discovered: bool, pub flames: Vec<Obj>, pub f: HashMap<String, f64> }

/// `exitData`
#[derive(Clone, Debug, Default)]
pub struct ExitData { pub pos: Vec3, pub normal: Vec3, pub door: Option<Obj> }

/// `tvData`
#[derive(Clone, Debug)]
pub struct TvData { pub light: Obj, pub world: Vec3, pub acc: f64, pub normal: Vec3 }

/// `phoneData`
#[derive(Clone, Debug)]
pub struct PhoneData { pub world: Vec3, pub ringing: bool, pub answered: bool, pub ring_t: f64, pub normal: Vec3, pub handset: Option<Obj>, pub f: HashMap<String, f64> }

/// `clockData`
#[derive(Clone, Debug)]
pub struct ClockData { pub sec_pivot: Obj, pub world: Vec3, pub acc: f64, pub tock: bool, pub normal: Vec3 }

/// `manData` (the figure placeProps leaves: see placeProps)
#[derive(Clone, Debug)]
pub struct ManData { pub g: Obj, pub f: HashMap<String, f64>, pub b: HashMap<String, bool> }

/// `brokenElevator`
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

/// `vacuumBot`
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

/// `lightSwitches[i]` (the `sw` objects placeProps builds)
#[derive(Clone, Debug, Default)]
pub struct LightSwitch {
    pub g: Option<Obj>,
    pub x: f64,
    pub z: f64,
    pub on: bool,
    pub faulty: bool,
    /// indices into `panels` this switch controls
    pub panels: Vec<usize>,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// `hauntedCandles[i]` (candleState)
#[derive(Clone, Debug, Default)]
pub struct CandleState {
    pub lit: bool,
    pub objs: HashMap<String, Obj>,
    pub f: HashMap<String, f64>,
    pub b: HashMap<String, bool>,
}

/// `drawerProps[i]`
#[derive(Clone, Debug)]
pub struct DrawerProp { pub cabinet: Obj, pub drawers: Vec<std::rc::Rc<std::cell::RefCell<Drawer>>> }
#[derive(Clone, Debug)]
pub struct Drawer { pub mesh: Obj, pub opened: bool, pub content_type: String, pub content_taken: bool, pub use_pos: Vec3, pub f: HashMap<String, f64> }

/// `guideArrows[i]`
#[derive(Clone, Debug)]
pub struct GuideArrow { pub mesh: Obj, pub life: f64, pub start_y: f64, pub index: usize }

/// `scuffs[i]` / `sparkles[i]` / `ripples[i]` — pooled effect sprites
#[derive(Clone, Debug)]
pub struct PooledFx { pub obj: Obj, pub t: f64 }

/// `ambientLog[i]`
#[derive(Clone, Debug, Default)]
pub struct AmbientLogEntry { pub tag: String, pub at: f64, pub dist: Option<f64>, pub bearing: Option<String>, pub pan: Option<f64> }

/// `humPanel`
#[derive(Clone, Copy, Debug, Default)]
pub struct HumPanel { pub p: usize, pub d: f64 }

/// `entities[i]`. The common fields are typed; per-type state the rigs keep
/// (gait phases, pose blends, hair springs, ...) lives in the maps.
#[derive(Clone, Debug)]
pub struct Entity {
    /// 'smiler' | 'hound' | 'watcher' | 'staff' | 'shadow'
    pub kind: String,
    pub g: Obj,
    pub face: Option<Obj>,
    pub face_y: f64,
    pub eye_y_offset: f64,
    /// `parts` (named bones / meshes of the rig)
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
    /// numeric per-entity state (`e.poseYaw`, `e.gPh`, `e.hairVX`, ...), by JS name
    pub num: HashMap<String, f64>,
    /// boolean per-entity state
    pub flag: HashMap<String, bool>,
    /// vectors (`e.aimFinal`, ...)
    pub vec: HashMap<String, Vec3>,
    /// objects (`e.sprite`, `e.voice` handles are elsewhere)
    pub obj: HashMap<String, Obj>,
    /// strings
    pub s: HashMap<String, String>,
}
impl Entity {
    pub fn new(kind: &str, g: Obj) -> Entity {
        Entity {
            kind: kind.to_string(), g, face: None, face_y: 0.0, eye_y_offset: 0.0, parts: HashMap::new(), yaw: 0.0,
            state: "wander".into(), speed: 0.0, w_field: None, w_target: None, w_timer: 0.0, lose_t: 0.0, freeze: 0.0,
            stare_t: 0.0, step_acc: 0.0, unseen_t: 0.0, vx: 0.0, vz: 0.0, dying: 0.0, observed: false,
            num: HashMap::new(), flag: HashMap::new(), vec: HashMap::new(), obj: HashMap::new(), s: HashMap::new(),
        }
    }
    /// `e.pos` (it is `e.g.position`)
    pub fn pos(&self) -> Vec3 { self.g.pos() }
    pub fn n(&self, k: &str) -> f64 { self.num.get(k).copied().unwrap_or(0.0) }
    pub fn set_n(&mut self, k: &str, v: f64) { self.num.insert(k.to_string(), v); }
    pub fn fl(&self, k: &str) -> bool { self.flag.get(k).copied().unwrap_or(false) }
    pub fn set_fl(&mut self, k: &str, v: bool) { self.flag.insert(k.to_string(), v); }
    pub fn part(&self, k: &str) -> Option<Obj> { self.parts.get(k).cloned() }
}

/// `storeState`
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
    pub music: Option<crate::records::Voice>,
    pub f: HashMap<String, f64>,
}
impl Default for StoreState {
    fn default() -> Self {
        StoreState { on: false, t: 0.0, phase: "day".into(), cycle: 0.0, hides: vec![], hidden: false, lights_out: 0.0, panels_off: false,
            board_tex: None, board_canvas: None, board_text: String::new(), arrow_mat: None, music: None, f: HashMap::new() }
    }
}
use std::cell::RefCell;
/// a hiding spot in the Showroom (`storeState.hides[i]`)
#[derive(Clone, Debug, Default)]
pub struct StoreHide { pub x: f64, pub z: f64, pub eye: Vec3, pub f: HashMap<String, f64> }

/// A running sound the JS keeps a handle to (`{ stop() }`-style objects).
#[derive(Clone)]
pub struct Voice { pub nodes: Vec<AudioNode>, pub gain: Option<AudioNode> }
impl std::fmt::Debug for Voice { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Voice({} nodes)", self.nodes.len()) } }
impl Voice {
    pub fn stop(&self, t: f64) { for n in &self.nodes { n.stop(t); } }
}

/// `houseState`
#[derive(Clone, Debug, Default)]
pub struct HouseState {
    pub on: bool,
    pub lamps: Vec<HouseLamp>,
    pub nest: Option<(f64, f64)>,
    pub nest_light: Option<Obj>,
    pub out: bool,
    pub beat_t: f64,
}
/// `houseState.lamps[i]`
#[derive(Clone, Debug)]
pub struct HouseLamp { pub g: Obj, pub out: bool, pub light: Option<Obj>, pub objs: HashMap<String, Obj>, pub f: HashMap<String, f64> }

/// `stairAudio`
#[derive(Clone, Debug, Default)]
pub struct StairAudio { pub next_phrase: f64, pub next_knock: f64, pub heart_t: f64, pub step_acc: f64 }

/// `loopAudio`
#[derive(Clone, Copy, Debug, Default)]
pub struct LoopAudio { pub step_acc: f64, pub touch_t: f64 }

/// `quality` (the governor)
#[derive(Clone, Copy, Debug)]
pub struct Quality { pub max_pr: f64, pub pr: f64, pub min_pr: f64, pub ema: f64, pub slow_t: f64, pub fast_t: f64, pub changes: u32 }
impl Default for Quality { fn default() -> Self { Quality { max_pr: 2.0, pr: 2.0, min_pr: 0.75, ema: 1.0 / 60.0, slow_t: 0.0, fast_t: 0.0, changes: 0 } } }

/// `propCull`
#[derive(Clone, Debug, Default)]
pub struct PropCull { pub list: Vec<(Obj, f64)>, pub t: f64, pub hidden: usize }

/// `randTrace`
#[derive(Clone, Debug, Default)]
pub struct RandTrace { pub on: bool, pub n: u64, pub marks: Vec<(String, u64)> }

/// `touchInput` (unused on macOS; kept for the shared code paths)
#[derive(Clone, Copy, Debug, Default)]
pub struct TouchInput { pub active: bool, pub x: f64, pub z: f64, pub sprint: bool, pub crouch: bool }

/// `{ wet, still, electric, haunted, derelict, normal }` counters
#[derive(Clone, Copy, Debug, Default)]
pub struct WeatherTally { pub wet: f64, pub still: f64, pub electric: f64, pub haunted: f64, pub derelict: f64, pub normal: f64 }
impl WeatherTally {
    pub fn bump(&mut self, key: &str) { match key { "wet" => self.wet += 1.0, "still" => self.still += 1.0, "electric" => self.electric += 1.0, "haunted" => self.haunted += 1.0, "derelict" => self.derelict += 1.0, _ => self.normal += 1.0 } }
    pub fn get(&self, key: &str) -> f64 { match key { "wet" => self.wet, "still" => self.still, "electric" => self.electric, "haunted" => self.haunted, "derelict" => self.derelict, _ => self.normal } }
}

/// `lastRareFireByCause`
#[derive(Clone, Copy, Debug, Default)]
pub struct RareFire { pub knock: f64, pub door: f64, pub ding: f64, pub voice: f64, pub r#static: f64, pub shift: f64, pub pattern: f64, pub skip: f64 }

/// The interactive spot of a hiding place / anything with x, z, r
#[derive(Clone, Copy, Debug, Default)]
pub struct Spot { pub x: f64, pub z: f64, pub r: f64 }

/// `wallAdjSpot(x, z)` result
#[derive(Clone, Copy, Debug, Default)]
pub struct WallSpot { pub pos: Vec3, pub normal: Vec3, pub yaw: f64 }
