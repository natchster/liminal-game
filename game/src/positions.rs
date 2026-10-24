//! Every `*Positions` list the props register for ambient audio (generated from
//! the JS declarations: `fooBarPositions` -> `foo_bar_positions`).

use engine::math::Vec3;

#[derive(Default, Clone, Debug)]
pub struct PropPositions {
    /// `socketPositions`
    pub socket_positions: Vec<Vec3>,
    /// `fountainPositions`
    pub fountain_positions: Vec<Vec3>,
    /// `radiatorPositions`
    pub radiator_positions: Vec<Vec3>,
    /// `glowLightPositions`
    pub glow_light_positions: Vec<Vec3>,
    /// `motionSensorPositions`
    pub motion_sensor_positions: Vec<Vec3>,
    /// `speakerPositions`
    pub speaker_positions: Vec<Vec3>,
    /// `fireAlarmPositions`
    pub fire_alarm_positions: Vec<Vec3>,
    /// `phonePositions`
    pub phone_positions: Vec<Vec3>,
    /// `brokenPhonePositions`
    pub broken_phone_positions: Vec<Vec3>,
    /// `vendingPositions`
    pub vending_positions: Vec<Vec3>,
    /// `junctionBoxPositions`
    pub junction_box_positions: Vec<Vec3>,
    /// `keycardReaderPositions`
    pub keycard_reader_positions: Vec<Vec3>,
    /// `cardboardBoxPositions`
    pub cardboard_box_positions: Vec<Vec3>,
    /// `livingPlantPositions`
    pub living_plant_positions: Vec<Vec3>,
    /// `walkieTalkiePositions`
    pub walkie_talkie_positions: Vec<Vec3>,
    /// `wallClockPositions`
    pub wall_clock_positions: Vec<Vec3>,
    /// `crackedClockPositions`
    pub cracked_clock_positions: Vec<Vec3>,
    /// `ceilingDripPositions`
    pub ceiling_drip_positions: Vec<Vec3>,
    /// `laptopPositions`
    pub laptop_positions: Vec<Vec3>,
    /// `ceilingCablePlugPositions`
    pub ceiling_cable_plug_positions: Vec<Vec3>,
    /// `thermostatPositions`
    pub thermostat_positions: Vec<Vec3>,
    /// `handbellPositions`
    pub handbell_positions: Vec<Vec3>,
    /// `digitalClockPositions`
    pub digital_clock_positions: Vec<Vec3>,
    /// `alarmClockPositions`
    pub alarm_clock_positions: Vec<Vec3>,
    /// `offHookRotaryPositions`
    pub off_hook_rotary_positions: Vec<Vec3>,
    /// `drainGratePositions`
    pub drain_grate_positions: Vec<Vec3>,
    /// `bathroomBundlePositions`
    pub bathroom_bundle_positions: Vec<Vec3>,
    /// `cauldronPositions`
    pub cauldron_positions: Vec<Vec3>,
    /// `metronomePositions`
    pub metronome_positions: Vec<Vec3>,
    /// `metronomePositions[i].period` — parallel to `metronome_positions`:
    /// the pendulum's swing period (`1.2 + Math.random() * 0.6` s). The
    /// metronome placer pushes one per position; updateAmbient ticks at half
    /// of it (falls back to 1.5 s when the two lists disagree).
    pub metronome_periods: Vec<f64>,
    /// `phonographPositions`
    pub phonograph_positions: Vec<Vec3>,
    /// `brassLanternPositions`
    pub brass_lantern_positions: Vec<Vec3>,
    /// `sewingPositions`
    pub sewing_positions: Vec<Vec3>,
    /// `pocketWatchPositions`
    pub pocket_watch_positions: Vec<Vec3>,
    /// `batPositions`
    pub bat_positions: Vec<Vec3>,
    /// `globePositions`
    pub globe_positions: Vec<Vec3>,
    /// `hourglassPositions`
    pub hourglass_positions: Vec<Vec3>,
    /// `ghostPositions`
    pub ghost_positions: Vec<Vec3>,
    /// `chainPositions`
    pub chain_positions: Vec<Vec3>,
    /// `paperPositions`
    pub paper_positions: Vec<Vec3>,
    /// `pianoPositions`
    pub piano_positions: Vec<Vec3>,
    /// `chairPositions`
    pub chair_positions: Vec<Vec3>,
    /// `chairPositions[i].period` — parallel to `chair_positions`: the
    /// rocking chair's rock period (`5 + Math.random() * 3` s). The chair
    /// placer pushes one per position; updateAmbient creaks once per rock
    /// (falls back to 6.5 s when the two lists disagree).
    pub chair_periods: Vec<f64>,
    /// `crtPositions`
    pub crt_positions: Vec<Vec3>,
    /// `exitSignPositions`
    pub exit_sign_positions: Vec<Vec3>,
    /// `puddlePositions`
    pub puddle_positions: Vec<Vec3>,
    /// `chandelierPositions`
    pub chandelier_positions: Vec<Vec3>,
    /// `cassettePositions`
    pub cassette_positions: Vec<Vec3>,
    /// `teacupPositions`
    pub teacup_positions: Vec<Vec3>,
    /// `lockerPositions`
    pub locker_positions: Vec<Vec3>,
    /// `bulletinPositions`
    pub bulletin_positions: Vec<Vec3>,
    /// `alarmPullPositions`
    pub alarm_pull_positions: Vec<Vec3>,
    /// `payPhonePositions`
    pub pay_phone_positions: Vec<Vec3>,
    /// `radioPositions`
    pub radio_positions: Vec<Vec3>,
    /// `keychainPositions`
    pub keychain_positions: Vec<Vec3>,
    /// `elevatorPanelPositions`
    pub elevator_panel_positions: Vec<Vec3>,
    /// `plantPositions`
    pub plant_positions: Vec<Vec3>,
    /// `fanPositions`
    pub fan_positions: Vec<Vec3>,
    /// `smokeDetectorPositions`
    pub smoke_detector_positions: Vec<Vec3>,
    /// `coinPositions`
    pub coin_positions: Vec<Vec3>,
    /// `clockPositions`
    pub clock_positions: Vec<Vec3>,
    /// `cuckooClockPositions`
    pub cuckoo_clock_positions: Vec<Vec3>,
    /// `plushPositions`
    pub plush_positions: Vec<Vec3>,
    /// `gumballPositions`
    pub gumball_positions: Vec<Vec3>,
    /// `teapotPositions`
    pub teapot_positions: Vec<Vec3>,
    /// `cookieTinPositions`
    pub cookie_tin_positions: Vec<Vec3>,
    /// `kitchenClockPositions`
    pub kitchen_clock_positions: Vec<Vec3>,
    /// `offHookPhonePositions`
    pub off_hook_phone_positions: Vec<Vec3>,
    /// `hairdryerPositions`
    pub hairdryer_positions: Vec<Vec3>,
    /// `sewingMachinePositions`
    pub sewing_machine_positions: Vec<Vec3>,
    /// `herbSievePositions`
    pub herb_sieve_positions: Vec<Vec3>,
    /// `vacuumPositions`
    pub vacuum_positions: Vec<Vec3>,
    /// `mopBucketPositions`
    pub mop_bucket_positions: Vec<Vec3>,
    /// `wristwatchPositions`
    pub wristwatch_positions: Vec<Vec3>,
    /// `hotIronPositions`
    pub hot_iron_positions: Vec<Vec3>,
    /// `grandfatherClockPositions`
    pub grandfather_clock_positions: Vec<Vec3>,
    /// `rockingChairPositions`
    pub rocking_chair_positions: Vec<Vec3>,
    /// `tallLampPositions`
    pub tall_lamp_positions: Vec<Vec3>,
    /// `colanderPositions`
    pub colander_positions: Vec<Vec3>,
    /// `petBowlPositions`
    pub pet_bowl_positions: Vec<Vec3>,
    /// `musicBoxPositions`
    pub music_box_positions: Vec<Vec3>,
    /// `turntablePositions`
    pub turntable_positions: Vec<Vec3>,
    /// `chalkboardPositions`
    pub chalkboard_positions: Vec<Vec3>,
    /// `kitchenCalendarPositions`
    pub kitchen_calendar_positions: Vec<Vec3>,
    /// `telescopePositions`
    pub telescope_positions: Vec<Vec3>,
    /// `cuckooClockPositionsR741`
    pub cuckoo_clock_positions_r741: Vec<Vec3>,
    /// `litFireplacePositions`
    pub lit_fireplace_positions: Vec<Vec3>,
    /// `aquariumPositions`
    pub aquarium_positions: Vec<Vec3>,
    /// `wingbackArmchairPositions`
    pub wingback_armchair_positions: Vec<Vec3>,
    /// `pocketWatchPositionsR755`
    pub pocket_watch_positions_r755: Vec<Vec3>,
    /// `bellPullPositions`
    pub bell_pull_positions: Vec<Vec3>,
    /// `bellRackPositions`
    pub bell_rack_positions: Vec<Vec3>,
    /// `chessSetPositions`
    pub chess_set_positions: Vec<Vec3>,
    /// `hobbyHorseWheeledPositions`
    pub hobby_horse_wheeled_positions: Vec<Vec3>,
}

impl PropPositions {
    /// `resetPropPositionArrays()` — every list emptied for a new level.
    pub fn reset(&mut self) { *self = PropPositions::default(); }
}
