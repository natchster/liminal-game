//! The placers of parties: balloons, streamers, cake, music (the `party` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod birthday_hats;
mod bitten_donuts;
mod cake_stands;
mod candy_corn_scatters;
mod confetti_scatters;
mod cornbread_muffins;
mod donuts;
mod hanging_balloon_bundles;
mod hanging_streamers;
mod paper_garlands;
mod pie_slices;
mod sandwiches;
mod shared;
mod small_balloons;
mod small_birthday_candles;
mod small_cupcake_wrappers;
mod small_feather_boas;
mod small_gift_boxes;
mod small_kazoos;
mod small_paper_crowns;
mod small_paper_cups;
mod small_paper_napkins;
mod small_paper_plates;
mod small_party_banners;
mod small_party_horns;
mod small_pinatas;
mod small_tiaras;
mod small_trumpets;

/// Every party placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_birthday_hats", birthday_hats::place_birthday_hats));
    r.push(("place_bitten_donuts", bitten_donuts::place_bitten_donuts));
    r.push(("place_cake_stands", cake_stands::place_cake_stands));
    r.push(("place_candy_corn_scatters", candy_corn_scatters::place_candy_corn_scatters));
    r.push(("place_confetti_scatters", confetti_scatters::place_confetti_scatters));
    r.push(("place_cornbread_muffins", cornbread_muffins::place_cornbread_muffins));
    r.push(("place_donuts", donuts::place_donuts));
    r.push(("place_hanging_balloon_bundles", hanging_balloon_bundles::place_hanging_balloon_bundles));
    r.push(("place_hanging_streamers", hanging_streamers::place_hanging_streamers));
    r.push(("place_paper_garlands", paper_garlands::place_paper_garlands));
    r.push(("place_pie_slices", pie_slices::place_pie_slices));
    r.push(("place_sandwiches", sandwiches::place_sandwiches));
    r.push(("place_small_balloons", small_balloons::place_small_balloons));
    r.push(("place_small_birthday_candles", small_birthday_candles::place_small_birthday_candles));
    r.push(("place_small_cupcake_wrappers", small_cupcake_wrappers::place_small_cupcake_wrappers));
    r.push(("place_small_feather_boas", small_feather_boas::place_small_feather_boas));
    r.push(("place_small_gift_boxes", small_gift_boxes::place_small_gift_boxes));
    r.push(("place_small_kazoos", small_kazoos::place_small_kazoos));
    r.push(("place_small_paper_crowns", small_paper_crowns::place_small_paper_crowns));
    r.push(("place_small_paper_cups", small_paper_cups::place_small_paper_cups));
    r.push(("place_small_paper_napkins", small_paper_napkins::place_small_paper_napkins));
    r.push(("place_small_paper_plates", small_paper_plates::place_small_paper_plates));
    r.push(("place_small_party_banners", small_party_banners::place_small_party_banners));
    r.push(("place_small_party_horns", small_party_horns::place_small_party_horns));
    r.push(("place_small_pinatas", small_pinatas::place_small_pinatas));
    r.push(("place_small_tiaras", small_tiaras::place_small_tiaras));
    r.push(("place_small_trumpets", small_trumpets::place_small_trumpets));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 27);
        game::debug::testkit::smoke_placers(&list);
    }
}
