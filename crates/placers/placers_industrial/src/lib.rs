//! The placers of plant and service spaces: pipes, valves, cabinets, machinery, worn metal (the `industrial` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses;
//! what placers of other themes use too is in `placers_kit`.
use game::base::state::PlacerFn;

mod biohazard_signs;
mod cardboard_boxes;
mod caution_tapes;
mod ceiling_pipes;
mod cleaning_mop_buckets;
mod emergency_lanterns;
mod extension_cords;
mod fallen_ceiling_tiles;
mod fire_hose_reels;
mod firewood_stacks;
mod floor_drain_grates;
mod fuse_boxes;
mod hanging_light_bulbs;
mod hard_hats;
mod junction_boxes;
mod metal_buckets;
mod mop_buckets;
mod mops;
mod overturned_buckets;
mod radiators;
mod rolled_carpets;
mod rust_stain_streaks;
mod rusted_hammers;
mod rusted_nails;
mod rusted_pipes;
mod rusted_tin_cans;
mod rustic_signs;
mod safety_cones;
mod safety_glasses;
mod safety_vests;
mod small_ladders;
mod tool_pegboards;
mod traffic_cones;
mod vent_grates;
mod wall_drain_grates;
mod wall_pipe_leaks;
mod wooden_buckets;
mod wooden_crates;

/// Every industrial placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_biohazard_signs", biohazard_signs::place_biohazard_signs));
    r.push(("place_cardboard_boxes", cardboard_boxes::place_cardboard_boxes));
    r.push(("place_caution_tapes", caution_tapes::place_caution_tapes));
    r.push(("place_ceiling_pipes", ceiling_pipes::place_ceiling_pipes));
    r.push(("place_cleaning_mop_buckets", cleaning_mop_buckets::place_cleaning_mop_buckets));
    r.push(("place_emergency_lanterns", emergency_lanterns::place_emergency_lanterns));
    r.push(("place_extension_cords", extension_cords::place_extension_cords));
    r.push(("place_fallen_ceiling_tiles", fallen_ceiling_tiles::place_fallen_ceiling_tiles));
    r.push(("place_fire_hose_reels", fire_hose_reels::place_fire_hose_reels));
    r.push(("place_firewood_stacks", firewood_stacks::place_firewood_stacks));
    r.push(("place_floor_drain_grates", floor_drain_grates::place_floor_drain_grates));
    r.push(("place_fuse_boxes", fuse_boxes::place_fuse_boxes));
    r.push(("place_hanging_light_bulbs", hanging_light_bulbs::place_hanging_light_bulbs));
    r.push(("place_hard_hats", hard_hats::place_hard_hats));
    r.push(("place_junction_boxes", junction_boxes::place_junction_boxes));
    r.push(("place_metal_buckets", metal_buckets::place_metal_buckets));
    r.push(("place_mop_buckets", mop_buckets::place_mop_buckets));
    r.push(("place_mops", mops::place_mops));
    r.push(("place_overturned_buckets", overturned_buckets::place_overturned_buckets));
    r.push(("place_radiators", radiators::place_radiators));
    r.push(("place_rolled_carpets", rolled_carpets::place_rolled_carpets));
    r.push(("place_rust_stain_streaks", rust_stain_streaks::place_rust_stain_streaks));
    r.push(("place_rusted_hammers", rusted_hammers::place_rusted_hammers));
    r.push(("place_rusted_nails", rusted_nails::place_rusted_nails));
    r.push(("place_rusted_pipes", rusted_pipes::place_rusted_pipes));
    r.push(("place_rusted_tin_cans", rusted_tin_cans::place_rusted_tin_cans));
    r.push(("place_rustic_signs", rustic_signs::place_rustic_signs));
    r.push(("place_safety_cones", safety_cones::place_safety_cones));
    r.push(("place_safety_glasses", safety_glasses::place_safety_glasses));
    r.push(("place_safety_vests", safety_vests::place_safety_vests));
    r.push(("place_small_ladders", small_ladders::place_small_ladders));
    r.push(("place_tool_pegboards", tool_pegboards::place_tool_pegboards));
    r.push(("place_traffic_cones", traffic_cones::place_traffic_cones));
    r.push(("place_vent_grates", vent_grates::place_vent_grates));
    r.push(("place_wall_drain_grates", wall_drain_grates::place_wall_drain_grates));
    r.push(("place_wall_pipe_leaks", wall_pipe_leaks::place_wall_pipe_leaks));
    r.push(("place_wooden_buckets", wooden_buckets::place_wooden_buckets));
    r.push(("place_wooden_crates", wooden_crates::place_wooden_crates));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 38);
        game::debug::testkit::smoke_placers(&list);
    }
}
