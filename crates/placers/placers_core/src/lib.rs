//! The placers of the Backrooms' own: the office building's fixtures, signs, alarms, stains and leaks (the `core` theme of `world::tables::PLACER_THEME`).
//!
//! A module each, named as the placer is registered (`place_<name>` in
//! `<name>.rs`), with the helpers only it uses; `shared` holds what
//! several of them use; what placers of other themes use too is in `placers_kit`.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::base::state::PlacerFn;

mod bottle_caps;
mod broken_light_switches;
mod broken_window_frames;
mod card_readers;
mod ceiling_cables;
mod ceiling_cobwebs;
mod ceiling_cracks;
mod ceiling_fans;
mod ceiling_speakers;
mod ceiling_stains;
mod chalk_arrows;
mod chalk_marks;
mod cigarette_butts;
mod claw_marks;
mod coffee_cups;
mod cracked_mirrors;
mod cracked_wall_holes;
mod crushed_soda_cans;
mod crushed_water_bottles;
mod dead_ceiling_lights;
mod dropped_shoes;
mod dust_bunnies;
mod elevator_buttons;
mod elevator_panels;
mod emergency_lights;
mod emergency_phones;
mod emergency_strobes;
mod exit_arrow_signs;
mod exit_signs;
mod extinguisher_cabinets;
mod fire_alarm_pulls;
mod fire_alarms;
mod fire_extinguishers;
mod floor_puddles;
mod floor_stains;
mod frosted_windows;
mod half_wall_partitions;
mod handrails;
mod keycard_readers;
mod leaning_frames;
mod missing_photo_hooks;
mod mold_patches;
mod motion_detector_domes;
mod motion_sensors;
mod overturned_chair;
mod overturned_chairs;
mod overturned_wastebaskets;
mod peeled_paint_patches;
mod peeling_wallpapers;
mod pinned_photos;
mod scattered_pages;
mod scrap_papers;
mod shared;
mod single_socks;
mod small_chalk_sticks;
mod smoke_detectors;
mod sprinkler_heads;
mod switch_plates;
mod thermostats;
mod trash_cans;
mod wall_doors;
mod wall_handprints;
mod wall_intercoms;
mod wall_outlets;
mod wall_sockets;
mod wall_stains;
mod wall_thermostats;
mod wall_vents;
mod wall_water_streaks;
mod water_puddles;
mod wet_floor_signs;
mod window_signs;

/// Every core placer, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_bottle_caps", bottle_caps::place_bottle_caps));
    r.push(("place_broken_light_switches", broken_light_switches::place_broken_light_switches));
    r.push(("place_broken_window_frames", broken_window_frames::place_broken_window_frames));
    r.push(("place_card_readers", card_readers::place_card_readers));
    r.push(("place_ceiling_cables", ceiling_cables::place_ceiling_cables));
    r.push(("place_ceiling_cobwebs", ceiling_cobwebs::place_ceiling_cobwebs));
    r.push(("place_ceiling_cracks", ceiling_cracks::place_ceiling_cracks));
    r.push(("place_ceiling_fans", ceiling_fans::place_ceiling_fans));
    r.push(("place_ceiling_speakers", ceiling_speakers::place_ceiling_speakers));
    r.push(("place_ceiling_stains", ceiling_stains::place_ceiling_stains));
    r.push(("place_chalk_arrows", chalk_arrows::place_chalk_arrows));
    r.push(("place_chalk_marks", chalk_marks::place_chalk_marks));
    r.push(("place_cigarette_butts", cigarette_butts::place_cigarette_butts));
    r.push(("place_claw_marks", claw_marks::place_claw_marks));
    r.push(("place_coffee_cups", coffee_cups::place_coffee_cups));
    r.push(("place_cracked_mirrors", cracked_mirrors::place_cracked_mirrors));
    r.push(("place_cracked_wall_holes", cracked_wall_holes::place_cracked_wall_holes));
    r.push(("place_crushed_soda_cans", crushed_soda_cans::place_crushed_soda_cans));
    r.push(("place_crushed_water_bottles", crushed_water_bottles::place_crushed_water_bottles));
    r.push(("place_dead_ceiling_lights", dead_ceiling_lights::place_dead_ceiling_lights));
    r.push(("place_dropped_shoes", dropped_shoes::place_dropped_shoes));
    r.push(("place_dust_bunnies", dust_bunnies::place_dust_bunnies));
    r.push(("place_elevator_buttons", elevator_buttons::place_elevator_buttons));
    r.push(("place_elevator_panels", elevator_panels::place_elevator_panels));
    r.push(("place_emergency_lights", emergency_lights::place_emergency_lights));
    r.push(("place_emergency_phones", emergency_phones::place_emergency_phones));
    r.push(("place_emergency_strobes", emergency_strobes::place_emergency_strobes));
    r.push(("place_exit_arrow_signs", exit_arrow_signs::place_exit_arrow_signs));
    r.push(("place_exit_signs", exit_signs::place_exit_signs));
    r.push(("place_extinguisher_cabinets", extinguisher_cabinets::place_extinguisher_cabinets));
    r.push(("place_fire_alarm_pulls", fire_alarm_pulls::place_fire_alarm_pulls));
    r.push(("place_fire_alarms", fire_alarms::place_fire_alarms));
    r.push(("place_fire_extinguishers", fire_extinguishers::place_fire_extinguishers));
    r.push(("place_floor_puddles", floor_puddles::place_floor_puddles));
    r.push(("place_floor_stains", floor_stains::place_floor_stains));
    r.push(("place_frosted_windows", frosted_windows::place_frosted_windows));
    r.push(("place_half_wall_partitions", half_wall_partitions::place_half_wall_partitions));
    r.push(("place_handrails", handrails::place_handrails));
    r.push(("place_keycard_readers", keycard_readers::place_keycard_readers));
    r.push(("place_leaning_frames", leaning_frames::place_leaning_frames));
    r.push(("place_missing_photo_hooks", missing_photo_hooks::place_missing_photo_hooks));
    r.push(("place_mold_patches", mold_patches::place_mold_patches));
    r.push(("place_motion_detector_domes", motion_detector_domes::place_motion_detector_domes));
    r.push(("place_motion_sensors", motion_sensors::place_motion_sensors));
    r.push(("place_overturned_chair", overturned_chair::place_overturned_chair));
    r.push(("place_overturned_chairs", overturned_chairs::place_overturned_chairs));
    r.push(("place_overturned_wastebaskets", overturned_wastebaskets::place_overturned_wastebaskets));
    r.push(("place_peeled_paint_patches", peeled_paint_patches::place_peeled_paint_patches));
    r.push(("place_peeling_wallpapers", peeling_wallpapers::place_peeling_wallpapers));
    r.push(("place_pinned_photos", pinned_photos::place_pinned_photos));
    r.push(("place_scattered_pages", scattered_pages::place_scattered_pages));
    r.push(("place_scrap_papers", scrap_papers::place_scrap_papers));
    r.push(("place_single_socks", single_socks::place_single_socks));
    r.push(("place_small_chalk_sticks", small_chalk_sticks::place_small_chalk_sticks));
    r.push(("place_smoke_detectors", smoke_detectors::place_smoke_detectors));
    r.push(("place_sprinkler_heads", sprinkler_heads::place_sprinkler_heads));
    r.push(("place_switch_plates", switch_plates::place_switch_plates));
    r.push(("place_thermostats", thermostats::place_thermostats));
    r.push(("place_trash_cans", trash_cans::place_trash_cans));
    r.push(("place_wall_doors", wall_doors::place_wall_doors));
    r.push(("place_wall_handprints", wall_handprints::place_wall_handprints));
    r.push(("place_wall_intercoms", wall_intercoms::place_wall_intercoms));
    r.push(("place_wall_outlets", wall_outlets::place_wall_outlets));
    r.push(("place_wall_sockets", wall_sockets::place_wall_sockets));
    r.push(("place_wall_stains", wall_stains::place_wall_stains));
    r.push(("place_wall_thermostats", wall_thermostats::place_wall_thermostats));
    r.push(("place_wall_vents", wall_vents::place_wall_vents));
    r.push(("place_wall_water_streaks", wall_water_streaks::place_wall_water_streaks));
    r.push(("place_water_puddles", water_puddles::place_water_puddles));
    r.push(("place_wet_floor_signs", wet_floor_signs::place_wet_floor_signs));
    r.push(("place_window_signs", window_signs::place_window_signs));
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        assert_eq!(list.len(), 71);
        game::debug::testkit::smoke_placers(&list);
    }
}
