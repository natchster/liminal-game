//! The placers of bathrooms and washrooms: soap, towels, mirrors, toilet things (the `bathroom` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::base::state::PlacerFn;

mod air_fresheners;
mod bath_mats;
mod bathroom_mirrors;
mod bird_baths;
mod bobby_pins;
mod contact_lens_cases;
mod cotton_balls;
mod cotton_pads;
mod cotton_swabs;
mod eyeshadow_palettes;
mod floss_containers;
mod hair_brushes;
mod hair_claws;
mod hair_dryers;
mod hair_ties;
mod hairbrushes;
mod hairdryers;
mod hand_dryers;
mod hand_mirrors;
mod hand_towel_bars;
mod hand_towel_dispensers;
mod lipsticks;
mod makeup_brushes;
mod makeup_compacts;
mod makeup_sponges;
mod mascaras;
mod nail_clippers;
mod nail_files;
mod nail_polish;
mod nail_polishes;
mod paper_towel_dispensers;
mod pencil_eyeliners;
mod perfume_bottles;
mod perfumes;
mod pill_bottles;
mod restroom_signs;
mod shampoo_shelves;
mod shared;
mod shower_caps;
mod single_earrings;
mod sink_faucets;
mod small_compacts;
mod small_hairbrushes;
mod small_hand_mirrors;
mod small_lipsticks;
mod small_nail_polishes;
mod small_perfumes;
mod smudge_mirrors;
mod soap_bars;
mod soap_dispensers;
mod spilled_pill_bottles;
mod spray_bottles;
mod stall_walls;
mod standing_mirrors;
mod toilet_paper_dispensers;
mod toothbrush_holders;
mod towel_bars;
mod tweezers;
mod washcloths;

/// Every bathroom placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_air_fresheners", air_fresheners::place_air_fresheners));
    r.push(("place_bath_mats", bath_mats::place_bath_mats));
    r.push(("place_bathroom_mirrors", bathroom_mirrors::place_bathroom_mirrors));
    r.push(("place_bird_baths", bird_baths::place_bird_baths));
    r.push(("place_bobby_pins", bobby_pins::place_bobby_pins));
    r.push(("place_contact_lens_cases", contact_lens_cases::place_contact_lens_cases));
    r.push(("place_cotton_balls", cotton_balls::place_cotton_balls));
    r.push(("place_cotton_pads", cotton_pads::place_cotton_pads));
    r.push(("place_cotton_swabs", cotton_swabs::place_cotton_swabs));
    r.push(("place_eyeshadow_palettes", eyeshadow_palettes::place_eyeshadow_palettes));
    r.push(("place_floss_containers", floss_containers::place_floss_containers));
    r.push(("place_hair_brushes", hair_brushes::place_hair_brushes));
    r.push(("place_hair_claws", hair_claws::place_hair_claws));
    r.push(("place_hair_dryers", hair_dryers::place_hair_dryers));
    r.push(("place_hair_ties", hair_ties::place_hair_ties));
    r.push(("place_hairbrushes", hairbrushes::place_hairbrushes));
    r.push(("place_hairdryers", hairdryers::place_hairdryers));
    r.push(("place_hand_dryers", hand_dryers::place_hand_dryers));
    r.push(("place_hand_mirrors", hand_mirrors::place_hand_mirrors));
    r.push(("place_hand_towel_bars", hand_towel_bars::place_hand_towel_bars));
    r.push(("place_hand_towel_dispensers", hand_towel_dispensers::place_hand_towel_dispensers));
    r.push(("place_lipsticks", lipsticks::place_lipsticks));
    r.push(("place_makeup_brushes", makeup_brushes::place_makeup_brushes));
    r.push(("place_makeup_compacts", makeup_compacts::place_makeup_compacts));
    r.push(("place_makeup_sponges", makeup_sponges::place_makeup_sponges));
    r.push(("place_mascaras", mascaras::place_mascaras));
    r.push(("place_nail_clippers", nail_clippers::place_nail_clippers));
    r.push(("place_nail_files", nail_files::place_nail_files));
    r.push(("place_nail_polish", nail_polish::place_nail_polish));
    r.push(("place_nail_polishes", nail_polishes::place_nail_polishes));
    r.push(("place_paper_towel_dispensers", paper_towel_dispensers::place_paper_towel_dispensers));
    r.push(("place_pencil_eyeliners", pencil_eyeliners::place_pencil_eyeliners));
    r.push(("place_perfume_bottles", perfume_bottles::place_perfume_bottles));
    r.push(("place_perfumes", perfumes::place_perfumes));
    r.push(("place_pill_bottles", pill_bottles::place_pill_bottles));
    r.push(("place_restroom_signs", restroom_signs::place_restroom_signs));
    r.push(("place_shampoo_shelves", shampoo_shelves::place_shampoo_shelves));
    r.push(("place_shower_caps", shower_caps::place_shower_caps));
    r.push(("place_single_earrings", single_earrings::place_single_earrings));
    r.push(("place_sink_faucets", sink_faucets::place_sink_faucets));
    r.push(("place_small_compacts", small_compacts::place_small_compacts));
    r.push(("place_small_hairbrushes", small_hairbrushes::place_small_hairbrushes));
    r.push(("place_small_hand_mirrors", small_hand_mirrors::place_small_hand_mirrors));
    r.push(("place_small_lipsticks", small_lipsticks::place_small_lipsticks));
    r.push(("place_small_nail_polishes", small_nail_polishes::place_small_nail_polishes));
    r.push(("place_small_perfumes", small_perfumes::place_small_perfumes));
    r.push(("place_smudge_mirrors", smudge_mirrors::place_smudge_mirrors));
    r.push(("place_soap_bars", soap_bars::place_soap_bars));
    r.push(("place_soap_dispensers", soap_dispensers::place_soap_dispensers));
    r.push(("place_spilled_pill_bottles", spilled_pill_bottles::place_spilled_pill_bottles));
    r.push(("place_spray_bottles", spray_bottles::place_spray_bottles));
    r.push(("place_stall_walls", stall_walls::place_stall_walls));
    r.push(("place_standing_mirrors", standing_mirrors::place_standing_mirrors));
    r.push(("place_toilet_paper_dispensers", toilet_paper_dispensers::place_toilet_paper_dispensers));
    r.push(("place_toothbrush_holders", toothbrush_holders::place_toothbrush_holders));
    r.push(("place_towel_bars", towel_bars::place_towel_bars));
    r.push(("place_tweezers", tweezers::place_tweezers));
    r.push(("place_washcloths", washcloths::place_washcloths));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 58);
        game::debug::testkit::smoke_placers(&list);
    }
}
