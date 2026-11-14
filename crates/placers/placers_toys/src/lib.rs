//! The placers of toys and games: dolls, blocks, cards, dice, children's things (the `toys` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod action_figures;
mod baby_bottles;
mod baby_crib_fragments;
mod baby_rattles;
mod broken_whole_dolls;
mod building_blocks;
mod child_drawings;
mod children_shoes;
mod childrens_books;
mod childrens_tea_sets;
mod coloring_books;
mod conductor_batons;
mod crayon_graffiti;
mod crayons;
mod deflated_balls;
mod doll_body_parts;
mod doll_heads;
mod drawing_papers;
mod gameboys;
mod hobby_horses;
mod hopscotch_chalks;
mod jump_ropes;
mod kids_drawings;
mod lego_bricks;
mod magnetic_letters;
mod marble_scatters;
mod marbles;
mod music_sheets;
mod overturned_tricycles;
mod pacifiers;
mod pet_toys;
mod plush_bears;
mod puzzle_pieces;
mod rubber_balls;
mod rubber_ducks;
mod rubiks_cubes;
mod shared;
mod single_child_shoes;
mod sippy_cups;
mod skipping_ropes;
mod small_castanets;
mod small_crayons;
mod small_dolls;
mod small_hula_hoops;
mod small_jacks;
mod small_kites;
mod small_maracas;
mod small_metronomes;
mod small_pacifiers;
mod small_piggy_banks;
mod small_pinwheels;
mod small_princess_wands;
mod small_recorders;
mod small_skateboards;
mod small_spinning_tops;
mod small_tambourines;
mod small_toy_boats;
mod small_toy_dinos;
mod small_toy_microphones;
mod small_toy_phones;
mod small_toy_pianos;
mod small_toy_planes;
mod small_toy_robots;
mod small_toy_swords;
mod small_triangles;
mod small_xylophones;
mod small_yo_yos;
mod stuffed_elephants;
mod stuffed_rabbits;
mod teddy_bears;
mod teddy_paws;
mod teething_rings;
mod toy_cars;
mod toy_dinosaurs;
mod toy_drums;
mod worn_teddy_bears;
mod yoyos;

/// Every toys placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_action_figures", action_figures::place_action_figures));
    r.push(("place_baby_bottles", baby_bottles::place_baby_bottles));
    r.push(("place_baby_crib_fragments", baby_crib_fragments::place_baby_crib_fragments));
    r.push(("place_baby_rattles", baby_rattles::place_baby_rattles));
    r.push(("place_broken_whole_dolls", broken_whole_dolls::place_broken_whole_dolls));
    r.push(("place_building_blocks", building_blocks::place_building_blocks));
    r.push(("place_child_drawings", child_drawings::place_child_drawings));
    r.push(("place_children_shoes", children_shoes::place_children_shoes));
    r.push(("place_childrens_books", childrens_books::place_childrens_books));
    r.push(("place_childrens_tea_sets", childrens_tea_sets::place_childrens_tea_sets));
    r.push(("place_coloring_books", coloring_books::place_coloring_books));
    r.push(("place_conductor_batons", conductor_batons::place_conductor_batons));
    r.push(("place_crayon_graffiti", crayon_graffiti::place_crayon_graffiti));
    r.push(("place_crayons", crayons::place_crayons));
    r.push(("place_deflated_balls", deflated_balls::place_deflated_balls));
    r.push(("place_doll_body_parts", doll_body_parts::place_doll_body_parts));
    r.push(("place_doll_heads", doll_heads::place_doll_heads));
    r.push(("place_drawing_papers", drawing_papers::place_drawing_papers));
    r.push(("place_gameboys", gameboys::place_gameboys));
    r.push(("place_hobby_horses", hobby_horses::place_hobby_horses));
    r.push(("place_hopscotch_chalks", hopscotch_chalks::place_hopscotch_chalks));
    r.push(("place_jump_ropes", jump_ropes::place_jump_ropes));
    r.push(("place_kids_drawings", kids_drawings::place_kids_drawings));
    r.push(("place_lego_bricks", lego_bricks::place_lego_bricks));
    r.push(("place_magnetic_letters", magnetic_letters::place_magnetic_letters));
    r.push(("place_marble_scatters", marble_scatters::place_marble_scatters));
    r.push(("place_marbles", marbles::place_marbles));
    r.push(("place_music_sheets", music_sheets::place_music_sheets));
    r.push(("place_overturned_tricycles", overturned_tricycles::place_overturned_tricycles));
    r.push(("place_pacifiers", pacifiers::place_pacifiers));
    r.push(("place_pet_toys", pet_toys::place_pet_toys));
    r.push(("place_plush_bears", plush_bears::place_plush_bears));
    r.push(("place_puzzle_pieces", puzzle_pieces::place_puzzle_pieces));
    r.push(("place_rubber_balls", rubber_balls::place_rubber_balls));
    r.push(("place_rubber_ducks", rubber_ducks::place_rubber_ducks));
    r.push(("place_rubiks_cubes", rubiks_cubes::place_rubiks_cubes));
    r.push(("place_single_child_shoes", single_child_shoes::place_single_child_shoes));
    r.push(("place_sippy_cups", sippy_cups::place_sippy_cups));
    r.push(("place_skipping_ropes", skipping_ropes::place_skipping_ropes));
    r.push(("place_small_castanets", small_castanets::place_small_castanets));
    r.push(("place_small_crayons", small_crayons::place_small_crayons));
    r.push(("place_small_dolls", small_dolls::place_small_dolls));
    r.push(("place_small_hula_hoops", small_hula_hoops::place_small_hula_hoops));
    r.push(("place_small_jacks", small_jacks::place_small_jacks));
    r.push(("place_small_kites", small_kites::place_small_kites));
    r.push(("place_small_maracas", small_maracas::place_small_maracas));
    r.push(("place_small_metronomes", small_metronomes::place_small_metronomes));
    r.push(("place_small_pacifiers", small_pacifiers::place_small_pacifiers));
    r.push(("place_small_piggy_banks", small_piggy_banks::place_small_piggy_banks));
    r.push(("place_small_pinwheels", small_pinwheels::place_small_pinwheels));
    r.push(("place_small_princess_wands", small_princess_wands::place_small_princess_wands));
    r.push(("place_small_recorders", small_recorders::place_small_recorders));
    r.push(("place_small_skateboards", small_skateboards::place_small_skateboards));
    r.push(("place_small_spinning_tops", small_spinning_tops::place_small_spinning_tops));
    r.push(("place_small_tambourines", small_tambourines::place_small_tambourines));
    r.push(("place_small_toy_boats", small_toy_boats::place_small_toy_boats));
    r.push(("place_small_toy_dinos", small_toy_dinos::place_small_toy_dinos));
    r.push(("place_small_toy_microphones", small_toy_microphones::place_small_toy_microphones));
    r.push(("place_small_toy_phones", small_toy_phones::place_small_toy_phones));
    r.push(("place_small_toy_pianos", small_toy_pianos::place_small_toy_pianos));
    r.push(("place_small_toy_planes", small_toy_planes::place_small_toy_planes));
    r.push(("place_small_toy_robots", small_toy_robots::place_small_toy_robots));
    r.push(("place_small_toy_swords", small_toy_swords::place_small_toy_swords));
    r.push(("place_small_triangles", small_triangles::place_small_triangles));
    r.push(("place_small_xylophones", small_xylophones::place_small_xylophones));
    r.push(("place_small_yo_yos", small_yo_yos::place_small_yo_yos));
    r.push(("place_stuffed_elephants", stuffed_elephants::place_stuffed_elephants));
    r.push(("place_stuffed_rabbits", stuffed_rabbits::place_stuffed_rabbits));
    r.push(("place_teddy_bears", teddy_bears::place_teddy_bears));
    r.push(("place_teddy_paws", teddy_paws::place_teddy_paws));
    r.push(("place_teething_rings", teething_rings::place_teething_rings));
    r.push(("place_toy_cars", toy_cars::place_toy_cars));
    r.push(("place_toy_dinosaurs", toy_dinosaurs::place_toy_dinosaurs));
    r.push(("place_toy_drums", toy_drums::place_toy_drums));
    r.push(("place_worn_teddy_bears", worn_teddy_bears::place_worn_teddy_bears));
    r.push(("place_yoyos", yoyos::place_yoyos));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 76);
        game::debug::testkit::smoke_placers(&list);
    }
}
