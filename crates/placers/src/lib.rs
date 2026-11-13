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
    placers_p00::register(r);
    placers_p01::register(r);
    placers_p02::register(r);
    placers_p03::register(r);
    placers_p04::register(r);
    placers_p05::register(r);
    placers_p06::register(r);
    placers_p07::register(r);
    placers_p08::register(r);
    placers_p09::register(r);
    placers_p10::register(r);
    placers_p11::register(r);
    placers_p12::register(r);
    placers_p13::register(r);
    placers_p14::register(r);
    placers_p15::register(r);
    placers_p16::register(r);
    placers_p17::register(r);
    placers_p18::register(r);
    placers_p19::register(r);
    placers_p20::register(r);
    placers_p21::register(r);
    placers_p22::register(r);
    placers_p23::register(r);
    placers_p24::register(r);
    placers_p25::register(r);
    placers_p26::register(r);
    placers_p27::register(r);
    placers_p28::register(r);
    placers_p29::register(r);
    placers_p30::register(r);
    placers_p31::register(r);
    placers_p32::register(r);
}
