//! The placers of old houses: portraits, clocks, brass, lace, heavy furniture (the `victorian` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod ancient_book_stacks;
mod ancient_portraits;
mod antique_maps;
mod bellows_cameras;
mod birdcages;
mod botanical_prints;
mod brass_sconces;
mod candle_lanterns;
mod chair_arm_books;
mod correspondence_cards;
mod cuckoo_clocks;
mod deer_head_mounts;
mod family_portraits;
mod fire_tool_sets;
mod fireplaces;
mod gentle_rockers;
mod grandfather_clocks;
mod horseshoes;
mod inkwell_letters;
mod miniature_portrait_clusters;
mod mounted_antlers;
mod mounted_fish;
mod open_journals;
mod ornate_cuckoo_clocks;
mod ornate_music_boxes;
mod ornate_wall_mirrors;
mod ottomans;
mod peacock_feather_vases;
mod pet_portraits;
mod record_stacks;
mod rocking_chairs;
mod servant_hall_bell_racks;
mod shared;
mod sheet_covered_furniture;
mod silver_trays;
mod small_globes;
mod small_phonographs;
mod small_trophies;
mod tarnished_mirrors;
mod tripod_telescopes;
mod turntables;
mod victorian_hall_stands;
mod wall_barometers;
mod wingback_armchairs;

/// Every victorian placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_ancient_book_stacks", ancient_book_stacks::place_ancient_book_stacks));
    r.push(("place_ancient_portraits", ancient_portraits::place_ancient_portraits));
    r.push(("place_antique_maps", antique_maps::place_antique_maps));
    r.push(("place_bellows_cameras", bellows_cameras::place_bellows_cameras));
    r.push(("place_birdcages", birdcages::place_birdcages));
    r.push(("place_botanical_prints", botanical_prints::place_botanical_prints));
    r.push(("place_brass_sconces", brass_sconces::place_brass_sconces));
    r.push(("place_candle_lanterns", candle_lanterns::place_candle_lanterns));
    r.push(("place_chair_arm_books", chair_arm_books::place_chair_arm_books));
    r.push(("place_correspondence_cards", correspondence_cards::place_correspondence_cards));
    r.push(("place_cuckoo_clocks", cuckoo_clocks::place_cuckoo_clocks));
    r.push(("place_deer_head_mounts", deer_head_mounts::place_deer_head_mounts));
    r.push(("place_family_portraits", family_portraits::place_family_portraits));
    r.push(("place_fire_tool_sets", fire_tool_sets::place_fire_tool_sets));
    r.push(("place_fireplaces", fireplaces::place_fireplaces));
    r.push(("place_gentle_rockers", gentle_rockers::place_gentle_rockers));
    r.push(("place_grandfather_clocks", grandfather_clocks::place_grandfather_clocks));
    r.push(("place_horseshoes", horseshoes::place_horseshoes));
    r.push(("place_inkwell_letters", inkwell_letters::place_inkwell_letters));
    r.push(("place_miniature_portrait_clusters", miniature_portrait_clusters::place_miniature_portrait_clusters));
    r.push(("place_mounted_antlers", mounted_antlers::place_mounted_antlers));
    r.push(("place_mounted_fish", mounted_fish::place_mounted_fish));
    r.push(("place_open_journals", open_journals::place_open_journals));
    r.push(("place_ornate_cuckoo_clocks", ornate_cuckoo_clocks::place_ornate_cuckoo_clocks));
    r.push(("place_ornate_music_boxes", ornate_music_boxes::place_ornate_music_boxes));
    r.push(("place_ornate_wall_mirrors", ornate_wall_mirrors::place_ornate_wall_mirrors));
    r.push(("place_ottomans", ottomans::place_ottomans));
    r.push(("place_peacock_feather_vases", peacock_feather_vases::place_peacock_feather_vases));
    r.push(("place_pet_portraits", pet_portraits::place_pet_portraits));
    r.push(("place_record_stacks", record_stacks::place_record_stacks));
    r.push(("place_rocking_chairs", rocking_chairs::place_rocking_chairs));
    r.push(("place_servant_hall_bell_racks", servant_hall_bell_racks::place_servant_hall_bell_racks));
    r.push(("place_sheet_covered_furniture", sheet_covered_furniture::place_sheet_covered_furniture));
    r.push(("place_silver_trays", silver_trays::place_silver_trays));
    r.push(("place_small_globes", small_globes::place_small_globes));
    r.push(("place_small_phonographs", small_phonographs::place_small_phonographs));
    r.push(("place_small_trophies", small_trophies::place_small_trophies));
    r.push(("place_tarnished_mirrors", tarnished_mirrors::place_tarnished_mirrors));
    r.push(("place_tripod_telescopes", tripod_telescopes::place_tripod_telescopes));
    r.push(("place_turntables", turntables::place_turntables));
    r.push(("place_victorian_hall_stands", victorian_hall_stands::place_victorian_hall_stands));
    r.push(("place_wall_barometers", wall_barometers::place_wall_barometers));
    r.push(("place_wingback_armchairs", wingback_armchairs::place_wingback_armchairs));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 43);
        game::debug::testkit::smoke_placers(&list);
    }
}
