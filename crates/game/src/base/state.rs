//! Core state records: animators, interactables, weathers, the player, the
//! shared materials and the placer function type.

use crate::base::config::*;
use engine::prelude::*;

/// A per-frame animator, called every frame with the game, `dt` and the sim time.
pub type Animator = Box<dyn FnMut(&mut Game, f64, f64)>;

/// Something the player can use: position, reach radius, prompt, enabled
/// test and use action.
pub struct Interactable {
    pub pos: Vec3,
    pub r: f64,
    pub prompt: String,
    pub enabled: Box<dyn Fn(&Game) -> bool>,
    pub on_use: Box<dyn FnMut(&mut Game)>,
}
impl Interactable {
    pub fn new(
        pos: Vec3,
        r: f64,
        prompt: &str,
        enabled: impl Fn(&Game) -> bool + 'static,
        on_use: impl FnMut(&mut Game) + 'static,
    ) -> Interactable {
        Interactable { pos, r, prompt: prompt.to_string(), enabled: Box::new(enabled), on_use: Box::new(on_use) }
    }
}

/// One of `WEATHER_BIASES`.
#[derive(Clone, Debug)]
pub struct Weather {
    pub key: &'static str,
    pub label: Option<&'static str>,
    pub fog_hex: u32,
    pub bg_hex: u32,
    pub fog_mul: f64,
    pub boosts: &'static [(&'static str, f64)],
    pub dampens: &'static [(&'static str, f64)],
}

pub static WEATHER_BIASES: [Weather; 6] = [
    Weather {
        key: "still",
        label: Some("THE AIR IS STILL"),
        fog_hex: 0x0a0806,
        bg_hex: 0x0a0806,
        fog_mul: 0.9,
        boosts: &[("chime", 3.0), ("breath", 3.0), ("piano", 2.0)],
        dampens: &[("thud", 0.4), ("clank", 0.4)],
    },
    Weather {
        key: "wet",
        label: Some("SOMEWHERE, WATER DRIPS"),
        fog_hex: 0x081018,
        bg_hex: 0x061018,
        fog_mul: 1.35,
        boosts: &[("drip", 4.0), ("dripHeavy", 4.0), ("paperRustle", 2.0)],
        dampens: &[("clank", 0.5)],
    },
    Weather {
        key: "electric",
        label: Some("THE LIGHTS BUZZ"),
        fog_hex: 0x0a0a12,
        bg_hex: 0x0a0a14,
        fog_mul: 0.85,
        boosts: &[("radioStatic", 4.0), ("dialTone", 3.0), ("clank", 2.0)],
        dampens: &[("moan", 0.5)],
    },
    Weather {
        key: "haunted",
        label: Some("SOMETHING IS AWAKE"),
        fog_hex: 0x0a0408,
        bg_hex: 0x0a0408,
        fog_mul: 1.15,
        boosts: &[("moan", 3.0), ("breath", 3.0), ("distantScream", 3.0), ("laugh", 2.0)],
        dampens: &[("piano", 0.4)],
    },
    Weather {
        key: "derelict",
        label: Some("THIS FLOOR IS COMING APART"),
        fog_hex: 0x100a05,
        bg_hex: 0x100a05,
        fog_mul: 1.25,
        boosts: &[("thud", 3.0), ("clank", 3.0), ("doorSlam", 3.0), ("distantFootsteps", 2.0)],
        dampens: &[("chime", 0.4)],
    },
    Weather { key: "normal", label: None, fog_hex: 0x070604, bg_hex: 0x070604, fog_mul: 1.0, boosts: &[], dampens: &[] },
];

/// The player: position, view angles, meters, inventory and movement state.
#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub stamina: f64,
    pub stam_delay: f64,
    pub sanity: f64,
    pub battery: f64,
    pub flashlight_on: bool,
    pub crouch: bool,
    pub eye_h: f64,
    pub bob_phase: f64,
    pub bob_amp: f64,
    pub bottles: i32,
    pub batteries: i32,
    pub papers: i32,
    pub was_in_pool: bool,
    pub speed_boost_until: f64,
    pub audible_range: f64,
    /// Eye height while hiding in the Showroom (None when not hiding)
    pub hide_eye: Option<f64>,
    /// Eased crouch tilt (starts at 0)
    pub crouch_pitch_lerp: f64,
}
impl Default for Player {
    fn default() -> Self {
        Player {
            pos: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            stamina: STAM_MAX,
            stam_delay: 0.0,
            sanity: SAN_MAX,
            battery: BAT_MAX,
            flashlight_on: false,
            crouch: false,
            eye_h: EYE,
            bob_phase: 0.0,
            bob_amp: 0.0,
            bottles: 0,
            batteries: 0,
            papers: 0,
            was_in_pool: false,
            speed_boost_until: 0.0,
            audible_range: 0.0,
            hide_eye: None,
            crouch_pitch_lerp: 0.0,
        }
    }
}
impl Player {
    /// Horizontal forward unit vector for the current yaw.
    pub fn fwd(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
    }
}

/// Materials shared by many props.
#[derive(Clone, Debug)]
pub struct SharedMats {
    pub dark: Mat,
    pub wood: Mat,
    pub metal: Mat,
    pub chrome: Mat,
    pub rust: Mat,
    pub plastic_yellow: Mat,
    pub plastic_orange: Mat,
    pub plastic_pink: Mat,
    pub plastic_red: Mat,
    pub plastic_white: Mat,
    pub cardboard: Mat,
    pub paper: Mat,
    pub glass: Mat,
}
impl Default for SharedMats {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedMats {
    pub fn new() -> SharedMats {
        SharedMats {
            dark: lambert!({ color: 0x0b0b0e }),
            wood: lambert!({ color: 0x4c3b22 }),
            metal: lambert!({ color: 0x3a3f45 }),
            chrome: lambert!({ color: 0x9fa7b0, emissive: 0x101418 }),
            rust: lambert!({ color: 0x6a3f22 }),
            plastic_yellow: lambert!({ color: 0xd9b03a, emissive: 0x2a1f08 }),
            plastic_orange: lambert!({ color: 0xcf5a1c }),
            plastic_pink: lambert!({ color: 0xd9a3b3 }),
            plastic_red: lambert!({ color: 0xa02020 }),
            plastic_white: lambert!({ color: 0xe8e2d4 }),
            cardboard: lambert!({ color: 0x9a774a }),
            paper: lambert!({ color: 0xe8ddc0 }),
            glass: lambert!({ color: 0x2a3438, emissive: 0x0a0e10, transparent: true, opacity: 0.55 }),
        }
    }
}

/// The gameplay-trigger wall switch and the distant ceiling panel it lights.
pub struct GameplayR1State {
    pub switch_x: f64,
    pub switch_z: f64,
    pub switch_tile: (i32, i32),
    pub target_x: f64,
    pub target_z: f64,
    pub target_tile: (i32, i32),
    pub on: bool,
    pub btn: Obj,
    pub btn_on: Mat,
    pub btn_off: Mat,
    pub lens: Obj,
    pub light: Obj,
}

/// A prop placer: builds its props into the level, given the BFS distance
/// field from the spawn tile.
pub type PlacerFn = fn(&mut Game, &[i32]);

pub use crate::base::game::Game;
