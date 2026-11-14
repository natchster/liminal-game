//! Shared by the newer backrooms tests: the placer registry (as `main.rs`
//! registers it) and a game booted to its title.
#![allow(dead_code, reason = "helpers shared by several test binaries; each uses a part")]
use game::Game;

/// Every prop placer (`placers::register_all`).
pub fn register_placers(r: &mut Vec<(&'static str, game::base::state::PlacerFn)>) {
    placers::register_all(r);
}

/// A game as the app makes it, booted to the title (no sound, no save)
pub fn new_game() -> Game {
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = "nosound".into();
    g.boot();
    g
}
