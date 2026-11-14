//! Every prop placer, in one list.
//!
//! The placers live in the crates beside this one (`crates/placers/*/`),
//! each registering its own by name; the game and every test that builds a
//! level register them all here, through [`register_all`]. A level runs its
//! placers by name (`Game::placer_fn`), so the order of this list changes
//! nothing. `tests/fingerprints.rs` records what each placer makes, so code
//! moved between these crates is held to making the same things.
use game::base::state::PlacerFn;

/// Every placer, by its registered name.
pub fn register_all(r: &mut Vec<(&'static str, PlacerFn)>) {
    placers_bathroom::register(r);
    placers_core::register(r);
    placers_domestic::register(r);
    placers_industrial::register(r);
    placers_nature::register(r);
    placers_occult::register(r);
    placers_office::register(r);
    placers_party::register(r);
    placers_toys::register(r);
    placers_victorian::register(r);
}
