//! The placers of the occult and Halloween: chalk, candles, skulls, charms (the `occult` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod brass_candlesticks;
mod candle_clusters;
mod crystal_skulls;
mod dress_forms;
mod hanging_bats;
mod hanging_censers;
mod memorial_shrines;
mod ouija_boards;
mod pentagram_chalks;
mod religious_triptychs;
mod reliquary_boxes;
mod shared;
mod skull_plaques;
mod skull_stacks;
mod small_black_cats;
mod small_brooms;
mod small_candelabras;
mod small_candles;
mod small_cauldrons;
mod small_chain_links;
mod small_crows;
mod small_crystal_balls;
mod small_ghost_sheets;
mod small_hourglasses;
mod small_masks;
mod small_owls;
mod small_potions;
mod small_pumpkins;
mod small_skeleton_hands;
mod small_skulls;
mod small_spiders;
mod small_tombstones;
mod small_treat_pails;
mod small_witch_hats;
mod stained_glass_windows;
mod tarot_card_fans;
mod wall_crucifixes;

/// Every occult placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_brass_candlesticks", brass_candlesticks::place_brass_candlesticks));
    r.push(("place_candle_clusters", candle_clusters::place_candle_clusters));
    r.push(("place_crystal_skulls", crystal_skulls::place_crystal_skulls));
    r.push(("place_dress_forms", dress_forms::place_dress_forms));
    r.push(("place_hanging_bats", hanging_bats::place_hanging_bats));
    r.push(("place_hanging_censers", hanging_censers::place_hanging_censers));
    r.push(("place_memorial_shrines", memorial_shrines::place_memorial_shrines));
    r.push(("place_ouija_boards", ouija_boards::place_ouija_boards));
    r.push(("place_pentagram_chalks", pentagram_chalks::place_pentagram_chalks));
    r.push(("place_religious_triptychs", religious_triptychs::place_religious_triptychs));
    r.push(("place_reliquary_boxes", reliquary_boxes::place_reliquary_boxes));
    r.push(("place_skull_plaques", skull_plaques::place_skull_plaques));
    r.push(("place_skull_stacks", skull_stacks::place_skull_stacks));
    r.push(("place_small_black_cats", small_black_cats::place_small_black_cats));
    r.push(("place_small_brooms", small_brooms::place_small_brooms));
    r.push(("place_small_candelabras", small_candelabras::place_small_candelabras));
    r.push(("place_small_candles", small_candles::place_small_candles));
    r.push(("place_small_cauldrons", small_cauldrons::place_small_cauldrons));
    r.push(("place_small_chain_links", small_chain_links::place_small_chain_links));
    r.push(("place_small_crows", small_crows::place_small_crows));
    r.push(("place_small_crystal_balls", small_crystal_balls::place_small_crystal_balls));
    r.push(("place_small_ghost_sheets", small_ghost_sheets::place_small_ghost_sheets));
    r.push(("place_small_hourglasses", small_hourglasses::place_small_hourglasses));
    r.push(("place_small_masks", small_masks::place_small_masks));
    r.push(("place_small_owls", small_owls::place_small_owls));
    r.push(("place_small_potions", small_potions::place_small_potions));
    r.push(("place_small_pumpkins", small_pumpkins::place_small_pumpkins));
    r.push(("place_small_skeleton_hands", small_skeleton_hands::place_small_skeleton_hands));
    r.push(("place_small_skulls", small_skulls::place_small_skulls));
    r.push(("place_small_spiders", small_spiders::place_small_spiders));
    r.push(("place_small_tombstones", small_tombstones::place_small_tombstones));
    r.push(("place_small_treat_pails", small_treat_pails::place_small_treat_pails));
    r.push(("place_small_witch_hats", small_witch_hats::place_small_witch_hats));
    r.push(("place_stained_glass_windows", stained_glass_windows::place_stained_glass_windows));
    r.push(("place_tarot_card_fans", tarot_card_fans::place_tarot_card_fans));
    r.push(("place_wall_crucifixes", wall_crucifixes::place_wall_crucifixes));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 36);
        game::debug::testkit::smoke_placers(&list);
    }
}
