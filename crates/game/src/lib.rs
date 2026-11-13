//! The Backrooms: the game itself, built on the `engine` crate.
//!
//! Module tree:
//! * [`base`] — the [`Game`] state, gameplay constants, numeric helpers, the
//!   seeded random streams, shared record types and lookup tables.
//! * [`level`] — level generation: the zoned architecture layout and the
//!   per-run anomaly plan.
//! * [`world`] — building a level's world: walls, ceilings, lights,
//!   instancing, prop placement and settling.
//! * [`assets`] — the embedded mesh assets: prop parts and creature bodies.
//! * [`placers`] — the kit and shared tables the prop placer crates build with.
//! * [`core_props`] — the core prop pass: pickups, furniture, scatter and the
//!   weather-specific dressing.
//! * [`entities`] — the creatures: rigs, animation, steering and AI.
//! * [`anomalies`] — the level anomalies: the Showroom, the Borrowed House and
//!   the human rig.
//! * [`modes`] — sub-modes that replace the level world: the Stairwell and the
//!   Loop.
//! * [`player`] — input actions, movement, item use and player-side gadgets.
//! * [`cutscene`] — cutscenes as data: the library, its rules, the camera's
//!   curves (`docs/design/CUTSCENES.md`).
//! * [`audio`] — sound effects, persistent beds, the spatial soundscape and
//!   ambient events.
//! * [`visuals`] — lighting, procedural textures and surfaces, full-screen
//!   colour effects.
//! * [`ui`] — the HUD and menus: element model, logic and drawing.
//! * [`session`] — start-up, the per-frame update, input and the game flow.
//! * [`story`] — M.'s trail, her camps and her notebook: the descent's story.
//! * [`debug`] — inspection hooks (level dumps) and test helpers.
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::needless_range_loop, non_snake_case)]

pub mod anomalies;
pub mod assets;
pub mod audio;
pub mod base;
pub mod core_props;
pub mod cutscene;
pub mod debug;
pub mod entities;
pub mod level;
pub mod modes;
pub mod placers;
pub mod player;
pub mod secrets;
pub mod session;
pub mod story;
pub mod ui;
pub mod visuals;
pub mod world;

pub use crate::base::game::Game;
pub use placers::kit::*;

/// What every placer module imports.
pub mod prelude {
    pub use crate::base::config::*;
    pub use crate::base::game::Game;
    pub use crate::base::numeric::*;
    pub use crate::base::rng::*;
    pub use crate::base::state::{Animator, Interactable, PlacerFn, Player, Weather};
    pub use crate::placers::kit::*;
    pub use crate::placers::shared::*;
    pub use engine::prelude::*;
    pub use std::cell::{Cell, RefCell};
    pub use std::f64::consts::PI;
    pub use std::rc::Rc;
}
