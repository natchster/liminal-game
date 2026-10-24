//! The Backrooms, ported from backrooms.js to native Rust.
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::needless_range_loop, non_snake_case)]

pub mod rng;
pub mod js;
pub mod config;
pub mod positions;
pub mod state;
pub mod game;
pub mod dom;
pub mod records;
pub mod tables;
pub mod world;
pub mod world_arch;
pub mod world_settle;
pub mod world_build;
pub mod world_tables;
pub mod lighting;
pub mod anomalies;
pub mod anomalies_rig;
pub mod anomalies_house;
pub mod core_props;
pub mod core_props_tail;
pub mod core_props_tail_fx;
pub mod core_props_tail_seam;
pub mod xstubs_cp2;
pub mod core_props_builders;
pub mod entities;
pub mod entities_anim;
pub mod entities_ai;
pub mod ambient;
pub mod player;
pub mod hud;
pub mod hud_draw;
pub mod flow;
pub mod modes;
pub mod main_loop;
pub mod audio_sys;
pub mod soundscape;
pub mod textures;
pub mod surfaces;
pub mod creatures;
pub mod instancing;
pub mod placement;
pub mod archgen;
pub mod scp;
pub mod stairwell;
pub mod loop_mode;
pub mod store;
pub mod xstubs_player;
pub mod hooks;
pub mod scene_fx;
pub mod xstubs_amb;
pub mod ambient_weather;
pub mod ambient_props;
pub mod ambient_props2;
pub mod ambient_props3;
pub mod api;
pub mod props;
pub mod shared;
pub mod testkit;

pub use api::*;
pub use game::Game;

/// What every placer module imports.
pub mod prelude {
    pub use engine::prelude::*;
    pub use crate::api::*;
    pub use crate::config::*;
    pub use crate::game::Game;
    pub use crate::js::*;
    pub use crate::rng::*;
    pub use crate::shared::*;
    pub use crate::state::{Animator, Interactable, PlacerFn, Player, Weather};
    pub use std::cell::{Cell, RefCell};
    pub use std::rc::Rc;
    pub use std::f64::consts::PI;
}

thread_local! { static STUBS_SEEN: std::cell::RefCell<std::collections::HashSet<&'static str>> = std::cell::RefCell::new(std::collections::HashSet::new()); }
/// Log (once) that a not-yet-ported function was called.
pub fn stub_hit(name: &'static str) {
    STUBS_SEEN.with(|s| { if s.borrow_mut().insert(name) { eprintln!("[stub] {name} not ported yet"); } });
}
/// `stub!("fnName")` inside a placeholder body.
#[macro_export]
macro_rules! stub { ($n:expr) => { $crate::stub_hit($n) }; }
