//! The placers of the occult and Halloween: chalk, candles, skulls, charms (the `occult` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod brass_candlesticks;
mod candle_clusters;
mod dress_forms;
mod religious_triptychs;
mod shared;
mod skull_stacks;
mod small_black_cats;
mod small_candelabras;
mod small_candles;
mod small_cauldrons;
mod small_crows;
mod small_owls;
mod small_pumpkins;
mod small_skeleton_hands;
mod small_skulls;
mod small_treat_pails;

/// Every occult placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_brass_candlesticks", brass_candlesticks::place_brass_candlesticks));
    r.push(("place_candle_clusters", candle_clusters::place_candle_clusters));
    r.push(("place_dress_forms", dress_forms::place_dress_forms));
    r.push(("place_religious_triptychs", religious_triptychs::place_religious_triptychs));
    r.push(("place_skull_stacks", skull_stacks::place_skull_stacks));
    r.push(("place_small_black_cats", small_black_cats::place_small_black_cats));
    r.push(("place_small_candelabras", small_candelabras::place_small_candelabras));
    r.push(("place_small_candles", small_candles::place_small_candles));
    r.push(("place_small_cauldrons", small_cauldrons::place_small_cauldrons));
    r.push(("place_small_crows", small_crows::place_small_crows));
    r.push(("place_small_owls", small_owls::place_small_owls));
    r.push(("place_small_pumpkins", small_pumpkins::place_small_pumpkins));
    r.push(("place_small_skeleton_hands", small_skeleton_hands::place_small_skeleton_hands));
    r.push(("place_small_skulls", small_skulls::place_small_skulls));
    r.push(("place_small_treat_pails", small_treat_pails::place_small_treat_pails));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 15);
        game::debug::testkit::smoke_placers(&list);
    }
}
