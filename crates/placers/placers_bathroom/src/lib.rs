//! The placers of bathrooms and washrooms: soap, towels, mirrors, toilet things (the `bathroom` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod air_fresheners;
mod bird_baths;
mod cotton_swabs;
mod hair_dryers;
mod hairdryers;
mod hand_towel_bars;
mod hand_towel_dispensers;
mod makeup_brushes;
mod nail_polishes;
mod paper_towel_dispensers;
mod perfume_bottles;
mod perfumes;
mod restroom_signs;
mod shampoo_shelves;
mod shared;
mod shower_caps;
mod sink_faucets;
mod small_hairbrushes;
mod small_hand_mirrors;
mod small_lipsticks;
mod smudge_mirrors;
mod soap_bars;
mod soap_dispensers;
mod spilled_pill_bottles;
mod spray_bottles;
mod standing_mirrors;
mod toilet_paper_dispensers;
mod toothbrush_holders;
mod towel_bars;
mod washcloths;

/// Every bathroom placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_air_fresheners", air_fresheners::place_air_fresheners));
    r.push(("place_bird_baths", bird_baths::place_bird_baths));
    r.push(("place_cotton_swabs", cotton_swabs::place_cotton_swabs));
    r.push(("place_hair_dryers", hair_dryers::place_hair_dryers));
    r.push(("place_hairdryers", hairdryers::place_hairdryers));
    r.push(("place_hand_towel_bars", hand_towel_bars::place_hand_towel_bars));
    r.push(("place_hand_towel_dispensers", hand_towel_dispensers::place_hand_towel_dispensers));
    r.push(("place_makeup_brushes", makeup_brushes::place_makeup_brushes));
    r.push(("place_nail_polishes", nail_polishes::place_nail_polishes));
    r.push(("place_paper_towel_dispensers", paper_towel_dispensers::place_paper_towel_dispensers));
    r.push(("place_perfume_bottles", perfume_bottles::place_perfume_bottles));
    r.push(("place_perfumes", perfumes::place_perfumes));
    r.push(("place_restroom_signs", restroom_signs::place_restroom_signs));
    r.push(("place_shampoo_shelves", shampoo_shelves::place_shampoo_shelves));
    r.push(("place_shower_caps", shower_caps::place_shower_caps));
    r.push(("place_sink_faucets", sink_faucets::place_sink_faucets));
    r.push(("place_small_hairbrushes", small_hairbrushes::place_small_hairbrushes));
    r.push(("place_small_hand_mirrors", small_hand_mirrors::place_small_hand_mirrors));
    r.push(("place_small_lipsticks", small_lipsticks::place_small_lipsticks));
    r.push(("place_smudge_mirrors", smudge_mirrors::place_smudge_mirrors));
    r.push(("place_soap_bars", soap_bars::place_soap_bars));
    r.push(("place_soap_dispensers", soap_dispensers::place_soap_dispensers));
    r.push(("place_spilled_pill_bottles", spilled_pill_bottles::place_spilled_pill_bottles));
    r.push(("place_spray_bottles", spray_bottles::place_spray_bottles));
    r.push(("place_standing_mirrors", standing_mirrors::place_standing_mirrors));
    r.push(("place_toilet_paper_dispensers", toilet_paper_dispensers::place_toilet_paper_dispensers));
    r.push(("place_toothbrush_holders", toothbrush_holders::place_toothbrush_holders));
    r.push(("place_towel_bars", towel_bars::place_towel_bars));
    r.push(("place_washcloths", washcloths::place_washcloths));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 29);
        game::debug::testkit::smoke_placers(&list);
    }
}
