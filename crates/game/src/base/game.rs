//! `Game`: the whole mutable state of the game.
//!
//! Most state is a flat field of `Game`. The exceptions are whole subsystems
//! kept in their own structs: `g.tex` (textures), `g.audio` (persistent audio
//! nodes), `g.ui` (the HUD element tree), `g.pp` (the `*_positions` lists),
//! `g.mats` (shared materials).
//! Pure data tables (`LORE_ENTRIES`, `*_BY_WEATHER`, ...) are not fields:
//! they live as items in the module that owns them.

use crate::base::config::*;
use crate::base::positions::PropPositions;
use crate::base::records::*;
use crate::base::state::*;
use engine::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// A pending scheduled callback (see [`Game::schedule`]).
pub struct Timeout {
    pub id: u64,
    pub at_ms: f64,
    pub f: Option<Box<dyn FnOnce(&mut Game)>>,
}

pub struct Game {
    // ------------------------------------------------------------ level grid
    /// Tile grid dimensions (odd)
    pub w: i32,
    pub h: i32,
    /// 1 = wall, 0 = open, 2 = hidden from the current placer
    pub tiles: Vec<u8>,
    /// Whether the current level has an archgen layout
    pub has_layout: bool,
    // ------------------------------------------------------------ scene
    pub scene: Obj,
    pub camera: Obj,
    /// Per-level group every placer adds to
    pub world: Obj,
    pub animators: Vec<Animator>,
    pub interactables: Vec<Interactable>,
    /// Every `*_positions` list
    pub pp: PropPositions,
    /// Shared materials (wood, paper, glass, ...)
    pub mats: SharedMats,
    /// Point material of the dust motes
    pub dust_mat: Mat,
    /// The player's flashlight (a spot light)
    pub flashlight: Obj,
    // ------------------------------------------------------------ run state
    pub level_weather: &'static Weather,
    pub player: Player,
    /// Simulation clock (seconds)
    pub perf_t: f64,
    /// Seed of the level being built / played
    pub current_seed: u32,
    pub run_seed: u32,
    pub level_num: i32,
    pub gameplay_r1_state: Option<GameplayR1State>,
    // ------------------------------------------------------------ audio
    /// The audio context (None without audio)
    pub ac: Option<AudioContext>,
    pub master: Option<AudioNode>,
    pub noise_buf: Option<AudioBuffer>,
    pub muted: bool,
    /// Every other persistent audio node (drone gain, stamina breathing, the soundscape, ...)
    pub audio: crate::audio::sys::AudioSys,
    // ------------------------------------------------------------ hud (legacy placer-facing)
    pub msg_text: String,
    pub msg_t: f64,
    // ------------------------------------------------------------ registry
    /// Every prop placer by name (filled by the app at startup)
    pub placers: Vec<(&'static str, PlacerFn)>,

    // ============================================================ platform
    /// Key codes currently held down (mouse buttons as "Mouse0".."Mouse2")
    pub keys: HashSet<String>,
    /// The controls' latches (`session::input`)
    pub input: crate::session::input::InputState,
    /// The open menu, if any (`ui::menu`)
    pub menu: crate::ui::menu::Menu,
    /// The player's settings (`session::settings`)
    pub settings: crate::session::settings::Settings,
    /// What the HUD at rest is showing (`ui::rest`)
    pub rest: crate::ui::rest::Rest,
    /// The level card on arrival: its line and when it went up (ms)
    pub level_card: Option<(String, f64)>,
    /// Mouse motion since the last frame (relative motion while the pointer is locked)
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    /// Whether the pointer is locked to the game view
    pub pointer_locked: bool,
    /// Requests the app applies after the frame
    pub want_pointer_lock: Option<bool>,
    pub want_fullscreen: Option<bool>,
    pub want_quit: bool,
    /// Set by `warm_up_gpu` to have the renderer compile the level's shader programs
    pub want_warm_up: bool,
    /// View size in logical pixels, and the backing scale factor
    pub view_width: f64,
    pub view_height: f64,
    pub scale_factor: f64,
    /// Scheduled callbacks (see [`Game::schedule`]) and the real-time clock (ms)
    pub timeouts: Vec<Timeout>,
    pub next_timeout_id: u64,
    pub now_ms: f64,
    /// Launch flags (debug switches: '#nosound', '#scp=...', ...)
    pub launch_flags: String,
    /// Persistent key/value store (settings and progress), saved to `storage_path`
    pub storage: BTreeMap<String, String>,
    pub storage_path: Option<std::path::PathBuf>,
    /// The HUD and menu element tree
    pub ui: crate::ui::model::UiModel,

    // ============================================================ base
    pub rand_trace: RandTrace,

    // ============================================================ world / architecture
    /// Ceiling height per tile
    pub tile_ceil: Vec<f32>,
    /// What each tile stands on (`archgen::Ground`); empty on the rooms,
    /// whose floors are all at y = 0
    pub ground: Vec<crate::level::archgen::Ground>,
    /// The Building's plan of this floor (levels 1–3), or None on the rooms
    pub building: Option<crate::level::building::BuildingPlan>,
    /// The loudspeakers on the Building's walls (where the siren comes from)
    pub building_speakers: Vec<Vec3>,
    /// The self-assembly on this floor of the Building (`world::assembly`)
    pub assembly: crate::world::assembly::Assembly,
    /// The cells' hermetic doors, as they swing
    pub cell_doors: Vec<crate::world::assembly::CellDoor>,
    /// The fog's surface, a few layers deep
    pub fog_layers: Vec<Obj>,
    /// The figure that stands far across the void now and then, and where
    /// it may stand (`world::far_figures`)
    pub far_figure: Option<crate::world::far_figures::FarFigure>,
    pub figure_perches: Vec<Vec3>,
    /// Where the Well's lift cage hangs, if it has one
    pub lift_at: Option<Vec3>,
    /// The Well's sealed doors and what answers a knock at them (`world::knock`)
    pub knocks: crate::world::knock::Knocks,
    /// The archgen layout of the current level
    pub layout: Option<crate::level::archgen::Layout>,
    /// Archgen solid boxes (collision, sight, rendering)
    pub solids: Option<crate::level::archgen::Solids>,
    /// the partitions that lean, whose staircases stand in for them in `solids` (`world::angled`)
    pub angled: Vec<crate::world::angled::Wall>,
    /// The level's vending machines (`core_props::vending`)
    pub vending: Vec<crate::core_props::vending::Machine>,
    /// The level's windows (`core_props::window`): the rows chosen in the
    /// build, and the windows finished on its final walls
    pub window_plans: Vec<crate::core_props::window::Plan>,
    pub windows: Vec<crate::core_props::window::Window>,
    /// The one behind the glass of this level, and its window (`entities::glass`)
    pub behind_glass: Option<crate::entities::glass::Bind>,
    /// The game's textures (wallpaper and carpet variants, dust sprite, static, ...)
    pub tex: crate::visuals::textures::Textures,
    /// Procedural surface textures (from `make_surfaces`)
    pub surfaces: crate::visuals::surfaces::Surfaces,
    /// The level's anomaly ('stairwell' | 'store' | 'house' | 'loop')
    pub level_scp: Option<String>,
    /// Ids of the omen's scheduled callbacks
    pub omen_timers: Vec<u64>,
    pub panels: Vec<Panel>,
    /// Instanced ceiling panels
    pub panel_mesh: Option<Obj>,
    /// The pooled panel spot lights (`POOLED_LIGHTS` of them), reassigned to nearby panels
    pub pooled: Vec<Obj>,
    /// Panel each pooled light is assigned to (index into `panels`)
    pub pooled_panel: Vec<Option<usize>>,
    /// Fade weight of each pooled light
    pub pooled_w: Vec<f64>,
    pub almonds: Vec<Pickup>,
    pub battery_items: Vec<Pickup>,
    /// What each zone of the layout is for (see `world::purpose`)
    pub zone_purposes: Vec<crate::world::purpose::ZonePurpose>,
    /// Region id per tile
    pub region_map: Vec<u8>,
    pub region_info: Vec<RegionInfo>,
    pub ceil_region: Vec<u8>,
    pub floor_holes: Vec<FloorHole>,
    /// Barricaded tiles (tile indices): while the level is built, the old
    /// choice's (its draws kept: `tail_barricades`), then the barricades'
    /// own (`barricades_settled`)
    pub barricade_tiles: Vec<usize>,
    /// The barricades as built: the slab each closes its opening with
    pub barricades: Vec<Barricade>,
    pub pool_bounds: Option<PoolBounds>,
    pub papers: Vec<PaperPile>,
    pub lore_notes: Vec<LoreNote>,
    pub safe_room: Option<SafeRoom>,
    pub broken_elevator: Option<BrokenElevator>,
    pub light_switches: Vec<LightSwitch>,
    pub haunted_candles: Vec<CandleState>,
    pub rate_delta_buf: Vec<f64>,
    pub last_rare_fire_by_cause: RareFire,
    /// Rare-event counts by cause, then weather
    pub rare_event_tally: BTreeMap<String, BTreeMap<String, f64>>,
    /// The last three ambient sounds (`log_ambient`); older ones drop off,
    /// so tell a new one by its `at`, not by counting what is here
    pub ambient_log: Vec<AmbientLogEntry>,
    /// Ambient-event counts by tag, then weather
    pub ambient_tally: BTreeMap<String, BTreeMap<String, f64>>,
    pub footstep_fx_tally: WeatherTally,
    pub crouch_fx_tally: WeatherTally,
    /// Chime flash colour, as a CSS 'rgba(r, g, b' prefix
    pub chime_flash_color: Option<String>,
    pub paper_markers: Vec<PaperMarker>,
    pub drawer_props: Vec<DrawerProp>,
    pub vacuum_bot: Option<VacuumBot>,
    pub exit_data: Option<ExitData>,
    pub tv_data: Option<TvData>,
    pub phone_data: Option<PhoneData>,
    pub clock_data: Option<ClockData>,
    pub man_data: Option<ManData>,
    pub spawn_pos: Vec3,
    /// Architecture material cache by key
    pub arch_mats: HashMap<String, Mat>,
    pub unit_box_geo: Geo,
    pub unit_plane_geo: Geo,
    pub dispose_stats: [u32; 3],
    pub prop_cull: PropCull,
    pub placement_stats: BTreeMap<String, f64>,
    pub settle_stats: BTreeMap<String, f64>,
    pub placer_theme_cache: HashMap<String, &'static str>,
    /// Placer theme gating: placers run, placers skipped, and (run, skipped) per theme
    pub theme_stats_ran: u32,
    pub theme_stats_skipped: u32,
    pub theme_stats_by_theme: BTreeMap<String, (u32, u32)>,
    pub placer_dist: Option<Vec<i32>>,
    /// Level build timings: `t.<phase>` durations (ms) and the `last` mark
    pub build_profile: BTreeMap<String, f64>,

    // ============================================================ anomalies
    pub store_state: StoreState,
    pub house_state: HouseState,
    /// Time and smoke uniforms of the shadow materials
    pub shadow_u_time: f64,
    pub shadow_u_smoke: f64,
    pub shadow_mat: Mat,
    pub shadow_edge_mat: Mat,

    // ============================================================ renderer / lighting
    /// Scene clear colour
    pub background: Color,
    /// Exponential-squared fog
    pub fog_color: Color,
    pub fog_density: f64,
    /// Uniforms of the local-light (light map) shader patch
    pub ll_on: f64,
    pub ll_map: Option<Tex>,
    pub ll_grid: [f64; 4],
    pub ll_scale: f64,
    pub ll_amb_d: Vec3,
    pub ll_sky_d: Vec3,
    pub ll_gnd_d: Vec3,
    pub ll_fog_lit: Color,
    pub ll_fog_ratio: f64,
    /// Ambient and hemisphere lights
    pub amb_light: Obj,
    pub hemi_light: Obj,
    pub light_map_cpu: Vec<f64>,
    /// the light map's G and B (`visuals::occlusion`), and what they were
    /// made of (`shape_key`): made once a level, kept while panels switch
    pub ll_shape: Vec<u8>,
    pub ll_shape_key: u64,
    pub player_light: Obj,
    pub halluc_textures: Vec<Tex>,
    pub halluc_shape: Obj,
    pub flashlight_target: Obj,
    pub dust_geo: Geo,
    pub dust_positions: Vec<f32>,
    pub dust_velocities: Vec<f32>,
    /// The dust-mote point cloud
    pub dust: Obj,
    pub rain_streak_tex: Tex,
    /// Rain overlay meshes whose texture scrolls
    pub rain_windows: Vec<Obj>,
    pub scuffs: Vec<PooledFx>,
    pub ripple_tex: Tex,
    pub splash_flash_light: Obj,
    pub thunder_flash_light: Obj,
    pub ripples: Vec<PooledFx>,
    pub sparkles: Vec<PooledFx>,
    pub wall_geo: Geo,
    pub panel_geo: Geo,
    pub ceil_tile_geo: Geo,
    pub floor_tile_geo: Geo,
    pub wall_mats: Vec<Mat>,
    pub ceil_mats: Vec<Mat>,
    pub floor_mats: Vec<Mat>,

    // ============================================================ entities
    pub entities: Vec<Entity>,
    /// BFS distance field to the player
    pub player_field: Option<std::rc::Rc<Vec<i32>>>,
    pub hum_panel: Option<HumPanel>,
    pub last_zone_arch: Option<String>,
    pub zones_seen: HashSet<String>,

    // ============================================================ player
    pub paper_marker_geo: Geo,
    pub paper_marker_mat: Mat,
    /// The place's footprints toward the door (`show_the_way`), drying
    pub way_prints: Vec<WayPrint>,
    pub print_geo: Geo,
    pub print_mat: Mat,
    /// How it last answered a cry, and how many times it has
    pub last_way: Option<WayAnswer>,
    pub ways_shown: u32,
    /// The secrets of this level (`secrets`): where things are put away
    pub secrets: crate::secrets::SecretsLevel,
    /// The run's: the places emptied (their key, and how many times used),
    /// what you took, and what that has made of you (`secrets::keeps`)
    pub secrets_taken: std::collections::BTreeMap<String, u32>,
    pub took: Vec<crate::secrets::keeps::Took>,
    pub body: crate::secrets::keeps::Body,
    /// Your size as it has come over you so far (eye height added, m)
    pub size_eye: f64,
    /// What you last used up (a bottle drunk, a battery changed): what the
    /// hole in the wall gives back
    pub last_used: Option<crate::secrets::Thing>,
    pub touch_input: TouchInput,

    // ============================================================ hud
    pub last_rendered_cur: Option<i64>,
    /// Found lore notes (note ids)
    pub lore_found: BTreeSet<usize>,
    pub lore_arcs_completed: BTreeSet<String>,
    pub last_death_cause: Option<String>,
    /// Death counts by cause, then weather
    pub death_tally: BTreeMap<String, BTreeMap<String, f64>>,

    // ---- world build
    /// The last level's resources, held until the new level is up
    pub pending_dispose: Option<crate::world::arch::Resources>,
    /// Prop culling sphere centres (x, z), parallel to `prop_cull.list` (which holds object and radius)
    pub prop_cull_xz: Vec<(f64, f64)>,
    /// The pool room `build_world` reserved; `place_props` takes it
    pub pool_plan: Option<crate::world::PoolPlan>,
    /// Index of `placers` by name, with the registry length it was built from
    pub placer_index: (usize, HashMap<&'static str, PlacerFn>),
    /// Test seam: build the world with `place_props`, `place_store` and
    /// `place_house` as no-ops (the world fixtures cover the architecture alone)
    pub test_stub_core_placers: bool,
    /// Test seam: leave the props' materials as their placers made them
    /// (the world fixtures record them before `settle_prop_looks`)
    pub test_keep_prop_looks: bool,

    // ---- hud
    /// 480×300 film-grain canvas (made by `init_hud`)
    pub grain_canvas: Option<std::rc::Rc<std::cell::RefCell<Canvas>>>,
    /// The HUD renderer's own state (per-element canvases, running CSS
    /// transitions/animations, hit rects); see [`crate::ui::draw`]
    pub hud_ui: crate::ui::draw::HudUi,

    // ---- entities
    /// Per-entity state that `records::Entity` has no field for (the rig, aim
    /// field, tic and voice), keyed by the entity object's id (see [`crate::entities`])
    pub ent_ext: HashMap<u64, crate::entities::EntExt>,

    // ---- anomalies
    /// The Showroom's hiding spots (object and hide data); `store_state.hides`
    /// mirrors their positions
    pub store_hide_spots: Vec<crate::anomalies::StoreHideSpot>,
    /// Pending tick of the Showroom music's 3.2 s step (cleared when the music stops)
    pub store_music_timer: Option<u64>,
    // ---- modes
    /// The child crying far below while in the Stairwell (`create_child_voice`)
    pub stair_voice: Option<crate::audio::soundscape::ChildVoice>,
    /// Its spatial voice slot (`voice_slot(4)`)
    pub stair_slot: Option<crate::audio::soundscape::VoiceSlot>,

    // ============================================================ stairwell / loop
    /// The 3D view's CSS-style filter transition (see [`crate::visuals::scene_fx`])
    pub scene_filter: crate::visuals::scene_fx::SceneFilterState,
    /// The Stairwell sub-mode (built by `init_modes` at boot; None in bare test games)
    pub stairwell: Option<crate::modes::stairwell::Stairwell>,
    /// The Loop sub-mode while inside the Loop, else None
    pub loop_rooms: Option<crate::modes::loop_rooms::LoopMode>,
    pub stair_audio: StairAudio,
    /// The Loop's wallpapers, clean and rotten
    pub loop_paper: Vec<Tex>,
    pub loop_audio: LoopAudio,

    // ============================================================ flow
    pub god_causes: BTreeMap<String, f64>,
    pub pool_edge: f64,
    /// Panels the pool wants lit (panel indices)
    pub pool_wanted: HashSet<usize>,
    /// Weather-biased ambient sounds: (weight, sfx name)
    pub biased_ambient: Vec<(f64, &'static str)>,
    /// The level's signature sounds (`pick_signatures`)
    pub amb_signatures: Vec<AmbSignature>,
    /// Where the weather's own sounds come from on this level (`ambient::BED_ANCHORS`)
    pub amb_anchors: Vec<(&'static str, Vec3)>,
    /// The story layer of this level: her camp, her walk and her sheets (`story`)
    pub story: crate::story::StoryLevel,
    /// Her pages taken this run (`story::notebook` ids), and the last line
    /// read (shown at the top of the screen)
    pub notebook: std::collections::BTreeSet<String>,
    pub thread: String,
    /// The journal entry to open on next (the page just taken)
    pub journal_newest: Option<i64>,
    /// Her story has ended (`story::ending`): you left the last camp's floor
    pub arc_done: bool,
    /// Seconds to the exit's next call (`ambient::amb_exit`), and how many it
    /// has made on this level (its jitter's counter)
    pub exit_call_t: f64,
    pub exit_calls: u32,
    /// How much the pacing director wants the ambience quiet (1 = only
    /// essential sounds): ambient one-shots lose up to 85 % of their level,
    /// none starts above 0.6, and the room tone dips up to half
    pub amb_hush: f64,
    /// While set, `blip` / `noise_burst` play through this route (`play_at`)
    pub sfx_route: Option<SfxRoute>,
    /// Next glitch and flicker-wave times
    pub glitch_next_t: f64,
    pub flicker_wave_next_t: f64,

    // ============================================================ main loop
    pub quality: Quality,

    // ============================================================ simple state (numbers / flags / strings), by section
    // ---- render
    pub light_map_off: f64,
    pub halluc_visible_t: f64,
    pub splash_flash_t: f64,
    pub splash_flash_peak: f64,
    pub thunder_flash_t: f64,
    // ---- world build
    pub motion_sensor_last_click_t: f64,
    pub faulty_trip_count: f64,
    pub candle_blowout_tally: f64,
    pub candle_relight_tally: f64,
    pub candle_fizzle_tally: f64,
    pub rate_drift_ticks: f64,
    pub wet_drip_timer: f64,
    pub still_wind_timer: f64,
    pub derelict_clatter_timer: f64,
    pub electric_crackle_timer: f64,
    pub socket_spark_timer: f64,
    pub fountain_drip_timer: f64,
    pub radiator_hiss_timer: f64,
    pub glow_hum_timer: f64,
    pub cauldron_bubble_timer: f64,
    pub metronome_tick_timer: f64,
    pub phono_crackle_timer: f64,
    pub lantern_creak_timer: f64,
    pub sewing_whirr_timer: f64,
    pub watch_tick_timer: f64,
    pub bat_squeak_timer: f64,
    pub globe_creak_timer: f64,
    pub hourglass_whoosh_timer: f64,
    pub ghost_whisper_timer: f64,
    pub chain_jangle_timer: f64,
    pub paper_rustle_timer: f64,
    pub piano_chord_timer: f64,
    pub chair_creak_timer: f64,
    pub crt_hiss_timer: f64,
    pub exit_buzz_timer: f64,
    pub puddle_drip_timer: f64,
    pub chandelier_chime_timer: f64,
    pub cassette_hiss_timer: f64,
    pub teacup_clink_timer: f64,
    pub locker_slam_timer: f64,
    pub bulletin_flutter_timer: f64,
    pub alarm_beep_timer: f64,
    pub pay_phone_ring_timer: f64,
    pub radio_static_timer: f64,
    pub keychain_jangle_timer: f64,
    pub elevator_beep_timer: f64,
    pub plant_rustle_timer: f64,
    pub fan_whoosh_timer: f64,
    pub fan_squeak_timer: f64,
    pub detector_chirp_timer: f64,
    pub thunder_rumble_timer: f64,
    pub coin_tink_timer: f64,
    pub wind_gust_timer: f64,
    pub tick_tock_timer: f64,
    pub rain_patter_timer: f64,
    pub cuckoo_call_timer: f64,
    pub plush_shuffle_timer: f64,
    pub gumball_rattle_timer: f64,
    pub teapot_whistle_timer: f64,
    pub cookie_tin_rattle_timer: f64,
    pub kitchen_clock_tick_timer: f64,
    pub off_hook_phone_hum_timer: f64,
    pub hairdryer_hum_timer: f64,
    pub sewing_machine_click_timer: f64,
    pub herb_sieve_scrape_timer: f64,
    pub vacuum_whine_timer: f64,
    pub mop_bucket_slosh_timer: f64,
    pub wristwatch_tick_timer: f64,
    pub iron_hiss_timer: f64,
    pub grandfather_clock_tick_timer: f64,
    pub rocking_chair_creak_timer: f64,
    pub tall_lamp_buzz_timer: f64,
    pub colander_tap_timer: f64,
    pub pet_bowl_slosh_timer: f64,
    pub music_box_melody_timer: f64,
    pub grandfather_chime_timer: f64,
    pub turntable_hiss_timer: f64,
    pub chalkboard_scratch_timer: f64,
    pub kitchen_calendar_turn_timer: f64,
    pub telescope_click_timer: f64,
    pub ornate_cuckoo_call_timer: f64,
    pub fireplace_crackle_timer: f64,
    pub aquarium_hum_timer: f64,
    pub wingback_creak_timer: f64,
    pub pocket_watch_tick_timer: f64,
    pub bell_pull_ring_timer: f64,
    pub bell_rack_ding_timer: f64,
    pub chess_move_click_timer: f64,
    pub hobby_horse_roll_timer: f64,
    pub speaker_announce_timer: f64,
    pub fire_alarm_beep_timer: f64,
    pub phone_ring_timer: f64,
    pub broken_phone_spark_timer: f64,
    pub vending_hum_timer: f64,
    pub junction_arc_timer: f64,
    pub keycard_beep_timer: f64,
    pub cardboard_shift_timer: f64,
    pub living_plant_rustle_timer: f64,
    pub walkie_static_timer: f64,
    pub wall_clock_tick_timer: f64,
    pub cracked_clock_creak_timer: f64,
    pub ceiling_drip_audio_timer: f64,
    pub ceiling_cable_spark_timer: f64,
    pub thermostat_click_timer: f64,
    pub handbell_chime_timer: f64,
    pub digital_clock_buzz_timer: f64,
    pub alarm_ring_timer: f64,
    pub dial_tone_timer: f64,
    pub drain_gurgle_timer: f64,
    pub faucet_drip_timer: f64,
    pub haunted_whisper_timer: f64,
    pub wet_thunder_timer: f64,
    pub wet_splash_timer: f64,
    pub derelict_grinder_timer: f64,
    pub still_echo_timer: f64,
    pub haunted_step_timer: f64,
    pub electric_surge_timer: f64,
    pub normal_notice_timer: f64,
    pub distant_knock_timer: f64,
    pub door_opened_timer: f64,
    pub door_opened_flash_t: f64,
    pub door_dim_pulse_t: f64,
    pub elevator_ding_timer: f64,
    pub elevator_ding_flash_t: f64,
    pub ding_bright_pulse_t: f64,
    pub voice_sample_timer: f64,
    pub voice_sample_flash_t: f64,
    pub static_burst_timer: f64,
    pub static_burst_flash_t: f64,
    pub reality_shift_timer: f64,
    pub reality_shift_t: f64,
    pub settle_timer: f64,
    pub settle_visual_t: f64,
    pub pattern_break_timer: f64,
    pub pattern_break_t: f64,
    pub clock_skip_timer: f64,
    pub clock_skip_t: f64,
    pub voice_hue_shift_t: f64,
    pub static_chromatic_t: f64,
    pub last_rare_fire_t: f64,
    pub knock_flash_t: f64,
    pub knock_tint_t: f64,
    pub knock_tint_pan: f64,
    pub last_knock_pan: f64,
    pub room_tone_cur_gain: f64,
    pub room_tone_cur_lp: f64,
    pub hum_pulse_phase: f64,
    pub hum_pulse_amp_current: f64,
    pub chime_flash_t: f64,
    pub chime_flash_life: f64,
    pub chime_flash_peak: f64,
    pub crouch_tilt_mag_current: f64,
    pub sanity_roll_amp_current: f64,
    pub sanity_roll_freq_current: f64,
    pub rain_score_current: f64,
    pub electric_surge_t: f64,
    pub panel_lfo_amp_current: f64,
    pub panel_lfo_freq_mul_current: f64,
    pub blowout_milestone_shown: bool,
    pub relight_milestone_shown: bool,
    pub archive_milestone_shown: bool,
    pub escalation_toast_fired: bool,
    // ---- entities
    /// The pacing director (`entities::director`)
    pub director: crate::entities::director::Director,
    pub player_field_t: f64,
    pub heart_t: f64,
    pub dark_heart_t: f64,
    pub crit_sanity_accum: f64,
    pub sanity_beat_t: f64,
    pub sanity_whisper_t: f64,
    pub sanity_was_impaired: bool,
    pub sanity_was_breaking: bool,
    pub sanity_recovery_tally: f64,
    pub sanity_impaired_tally: f64,
    pub sanity_breaking_tally: f64,
    pub sanity_mini_tally: f64,
    // ---- ambient
    pub hummer_t: f64,
    pub zone_toast_t: f64,
    // ---- player
    pub item_hint_water: String,
    pub item_hint_cell: String,
    pub hinted_bottle: bool,
    pub hinted_battery: bool,
    pub cry_cooldown: f64,
    /// Your cries this run and on this floor, when you last cried and what
    /// you said (`audio::voice::choose_cry`)
    pub cries: u32,
    pub cries_on_floor: u32,
    pub cry_last_at: Option<f64>,
    pub cry_last_line: Option<String>,
    pub cry_dim_t: f64,
    pub cry_dim_lerp: f64,
    pub last_step: f64,
    pub shake_t: f64,
    pub camera_dip_t: f64,
    // ---- hud
    pub journal_open: bool,
    pub journal_cursor: f64,
    pub debug_on: bool,
    pub grain_t: f64,
    pub grain_opacity_cur: f64,
    pub aberration_opacity_cur: f64,
    pub lore_archivist_shown: bool,
    pub survivor_shown: bool,
    pub storyteller_shown: bool,
    pub completionist_shown: bool,
    pub death_count: f64,
    pub last_death_t: f64,
    pub ironwill_shown: bool,
    // ---- stairwell / loop
    pub in_stairwell: bool,
    pub stair_near_exit: bool,
    pub in_loop: bool,
    pub loop_near_exit: bool,
    // ---- flow
    pub state: String,
    pub falling_t: f64,
    pub god_mode: bool,
    /// What this run remembers of the creatures: the kinds met, your habits
    pub creature_run: crate::entities::creature::RunMemory,
    /// `--showcase watcher`: its furniture placed (once)
    pub showcase_company: bool,
    pub god_deaths: f64,
    pub level_start_t: f64,
    pub pool_t: f64,
    pub glitch_t: f64,
    pub ghost_step_t: f64,
    // ---- main loop
    pub fps_frames: f64,
    pub fps_t: f64,
    pub fps: f64,
    pub fixed_step_dt: f64,
    pub sim_frozen: bool,
    pub sim_no_render: bool,
}

impl Game {
    /// A bare game: every field at its initial value, the scene empty.
    /// The subsystems' init functions build the rest, in a fixed order.
    pub fn new_bare() -> Game {
        let scene = new_scene();
        let camera = perspective_camera(crate::session::settings::FOV_DEFAULT, 1.0, 0.05, 90.0);
        camera.set_rotation_order(EulerOrder::YXZ);
        let flashlight = engine::scene::spot_light(0xfff2cf, 0.0, 28.0, std::f64::consts::PI / 5.5, 0.35, 1.2);
        let mut g = Game {
            w: 0,
            h: 0,
            tiles: vec![],
            has_layout: false,
            scene,
            camera,
            world: group(),
            animators: vec![],
            interactables: vec![],
            pp: PropPositions::default(),
            mats: SharedMats::new(),
            dust_mat: points_mat!({ size: 0.016, transparent: true, depthWrite: false, blending: AdditiveBlending, color: 0xf1e2b6, opacity: 0.20 }),
            flashlight,
            level_weather: &WEATHER_BIASES[5],
            player: Player::default(),
            perf_t: 0.0,
            current_seed: 0,
            run_seed: 0,
            level_num: 1,
            gameplay_r1_state: None,
            ac: None,
            master: None,
            noise_buf: None,
            muted: false,
            audio: Default::default(),
            msg_text: String::new(),
            msg_t: 0.0,
            placers: vec![],
            keys: HashSet::new(),
            input: Default::default(),
            menu: Default::default(),
            settings: Default::default(),
            rest: Default::default(),
            level_card: None,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            pointer_locked: false,
            want_pointer_lock: None,
            want_fullscreen: None,
            want_quit: false,
            want_warm_up: false,
            view_width: 1280.0,
            view_height: 800.0,
            scale_factor: 2.0,
            timeouts: vec![],
            next_timeout_id: 1,
            now_ms: 0.0,
            launch_flags: String::new(),
            storage: BTreeMap::new(),
            storage_path: None,
            ui: crate::ui::model::UiModel::new(),
            rand_trace: RandTrace::default(),
            tile_ceil: vec![],
            ground: vec![],
            building: None,
            building_speakers: vec![],
            assembly: Default::default(),
            cell_doors: vec![],
            fog_layers: vec![],
            far_figure: None,
            figure_perches: vec![],
            lift_at: None,
            knocks: Default::default(),
            layout: None,
            solids: None,
            angled: vec![],
            vending: vec![],
            window_plans: vec![],
            windows: vec![],
            behind_glass: None,
            tex: crate::visuals::textures::Textures::placeholder(),
            // (`rnd`: each painter draws inside `canvas_tex`, on its own stream)
            surfaces: crate::visuals::surfaces::make_surfaces(crate::base::rng::rnd),
            level_scp: None,
            omen_timers: vec![],
            panels: vec![],
            panel_mesh: None,
            pooled: vec![],
            pooled_panel: vec![],
            pooled_w: vec![],
            almonds: vec![],
            battery_items: vec![],
            zone_purposes: vec![],
            region_map: vec![],
            region_info: vec![],
            ceil_region: vec![],
            floor_holes: vec![],
            barricade_tiles: vec![],
            barricades: vec![],
            pool_bounds: None,
            papers: vec![],
            lore_notes: vec![],
            safe_room: None,
            broken_elevator: None,
            light_switches: vec![],
            haunted_candles: vec![],
            rate_delta_buf: vec![],
            last_rare_fire_by_cause: RareFire::default(),
            rare_event_tally: ["knock", "door", "ding", "voice", "static", "shift", "pattern", "skip"]
                .iter()
                .map(|k| (k.to_string(), BTreeMap::new()))
                .collect(),
            ambient_log: vec![],
            ambient_tally: BTreeMap::new(),
            footstep_fx_tally: WeatherTally::default(),
            crouch_fx_tally: WeatherTally::default(),
            chime_flash_color: None,
            paper_markers: vec![],
            drawer_props: vec![],
            stairwell: None,
            loop_rooms: None,
            scene_filter: Default::default(),
            // ---- world build
            pending_dispose: None,
            prop_cull_xz: vec![],
            pool_plan: None,
            placer_index: (0, HashMap::new()),
            test_stub_core_placers: false,
            test_keep_prop_looks: false,
            // ---- hud
            grain_canvas: None,
            hud_ui: Default::default(),
            ent_ext: HashMap::new(),
            store_hide_spots: vec![],
            store_music_timer: None,
            stair_voice: None,
            stair_slot: None,
            vacuum_bot: None,
            exit_data: None,
            tv_data: None,
            phone_data: None,
            clock_data: None,
            man_data: None,
            spawn_pos: Vec3::ZERO,
            arch_mats: HashMap::new(),
            unit_box_geo: box_geo!(1, 1, 1),
            unit_plane_geo: plane_geo!(1, 1),
            dispose_stats: [0; 3],
            prop_cull: PropCull::default(),
            placement_stats: BTreeMap::new(),
            settle_stats: BTreeMap::new(),
            placer_theme_cache: HashMap::new(),
            theme_stats_ran: 0,
            theme_stats_skipped: 0,
            theme_stats_by_theme: BTreeMap::new(),
            placer_dist: None,
            build_profile: BTreeMap::new(),
            store_state: StoreState::default(),
            house_state: HouseState::default(),
            shadow_u_time: 0.0,
            shadow_u_smoke: 1.0,
            shadow_mat: basic_mat!(),
            shadow_edge_mat: basic_mat!(),
            background: Color::hex(0x070604),
            fog_color: Color::hex(0x070604),
            fog_density: 0.052,
            ll_on: 0.0,
            ll_map: None,
            ll_grid: [1.0, 1.0, 0.5, 0.5],
            ll_scale: 1.0,
            ll_amb_d: Vec3::ZERO,
            ll_sky_d: Vec3::ZERO,
            ll_gnd_d: Vec3::ZERO,
            ll_fog_lit: Color::default(),
            ll_fog_ratio: 0.013 / 0.055,
            amb_light: ambient_light(0xffffff, 0),
            hemi_light: hemisphere_light(0xffffff, 0xffffff, 0),
            light_map_cpu: vec![],
            ll_shape: vec![],
            ll_shape_key: 0,
            player_light: point_light(0xffffff, 0, 0, 2),
            halluc_textures: vec![],
            halluc_shape: node(),
            flashlight_target: node(),
            dust_geo: Geo::new(),
            dust_positions: vec![],
            dust_velocities: vec![],
            dust: node(),
            rain_streak_tex: Tex::data_rgba(vec![0; 4], 1, 1),
            rain_windows: vec![],
            scuffs: vec![],
            ripple_tex: Tex::data_rgba(vec![0; 4], 1, 1),
            splash_flash_light: point_light(0xffffff, 0, 0, 2),
            thunder_flash_light: point_light(0xffffff, 0, 0, 2),
            ripples: vec![],
            sparkles: vec![],
            wall_geo: box_geo!(CELL, WALL_H, CELL),
            // (a fitting two ceiling tiles by one: `textures::CEIL_CELLS`)
            panel_geo: box_geo!(4.0 / 3.0, 0.06, 2.0 / 3.0),
            ceil_tile_geo: {
                let g = plane_geo!(CELL, CELL);
                g.rotate_x(std::f64::consts::PI / 2.0);
                g
            },
            floor_tile_geo: {
                let g = plane_geo!(CELL, CELL);
                g.rotate_x(-std::f64::consts::PI / 2.0);
                g
            },
            wall_mats: vec![],
            ceil_mats: vec![],
            floor_mats: vec![],
            entities: vec![],
            player_field: None,
            hum_panel: None,
            last_zone_arch: None,
            zones_seen: HashSet::new(),
            paper_marker_geo: box_geo!(0.22, 0.008, 0.28),
            paper_marker_mat: lambert!({ color: 0xf2ead0, emissive: 0x0a0906 }),
            way_prints: vec![],
            print_geo: Geo::new(),
            print_mat: basic_mat!(),
            last_way: None,
            ways_shown: 0,
            secrets: Default::default(),
            secrets_taken: Default::default(),
            took: vec![],
            body: Default::default(),
            size_eye: 0.0,
            last_used: None,
            touch_input: TouchInput::default(),
            last_rendered_cur: None,
            lore_found: BTreeSet::new(),
            lore_arcs_completed: BTreeSet::new(),
            last_death_cause: None,
            death_tally: BTreeMap::new(),
            stair_audio: StairAudio::default(),
            loop_paper: vec![],
            loop_audio: LoopAudio::default(),
            god_causes: BTreeMap::new(),
            pool_edge: f64::INFINITY,
            pool_wanted: HashSet::new(),
            biased_ambient: vec![],
            amb_signatures: vec![],
            story: Default::default(),
            notebook: Default::default(),
            thread: String::new(),
            journal_newest: None,
            arc_done: false,
            exit_call_t: 0.0,
            exit_calls: 0,
            amb_anchors: vec![],
            amb_hush: 0.0,
            sfx_route: None,
            glitch_next_t: 0.0,
            flicker_wave_next_t: 0.0,
            quality: Quality::default(),
            light_map_off: 0.0,
            halluc_visible_t: 0.0,
            splash_flash_t: 0.0,
            splash_flash_peak: 0.0,
            thunder_flash_t: 0.0,
            motion_sensor_last_click_t: 0.0,
            faulty_trip_count: 0.0,
            candle_blowout_tally: 0.0,
            candle_relight_tally: 0.0,
            candle_fizzle_tally: 0.0,
            rate_drift_ticks: 0.0,
            wet_drip_timer: 0.0,
            still_wind_timer: 0.0,
            derelict_clatter_timer: 0.0,
            electric_crackle_timer: 0.0,
            socket_spark_timer: 0.0,
            fountain_drip_timer: 0.0,
            radiator_hiss_timer: 0.0,
            glow_hum_timer: 0.0,
            cauldron_bubble_timer: 0.0,
            metronome_tick_timer: 0.0,
            phono_crackle_timer: 0.0,
            lantern_creak_timer: 0.0,
            sewing_whirr_timer: 0.0,
            watch_tick_timer: 0.0,
            bat_squeak_timer: 0.0,
            globe_creak_timer: 0.0,
            hourglass_whoosh_timer: 0.0,
            ghost_whisper_timer: 0.0,
            chain_jangle_timer: 0.0,
            paper_rustle_timer: 0.0,
            piano_chord_timer: 0.0,
            chair_creak_timer: 0.0,
            crt_hiss_timer: 0.0,
            exit_buzz_timer: 0.0,
            puddle_drip_timer: 0.0,
            chandelier_chime_timer: 0.0,
            cassette_hiss_timer: 0.0,
            teacup_clink_timer: 0.0,
            locker_slam_timer: 0.0,
            bulletin_flutter_timer: 0.0,
            alarm_beep_timer: 0.0,
            pay_phone_ring_timer: 0.0,
            radio_static_timer: 0.0,
            keychain_jangle_timer: 0.0,
            elevator_beep_timer: 0.0,
            plant_rustle_timer: 0.0,
            fan_whoosh_timer: 0.0,
            fan_squeak_timer: 0.0,
            detector_chirp_timer: 0.0,
            thunder_rumble_timer: 0.0,
            coin_tink_timer: 0.0,
            wind_gust_timer: 0.0,
            tick_tock_timer: 0.0,
            rain_patter_timer: 0.0,
            cuckoo_call_timer: 0.0,
            plush_shuffle_timer: 0.0,
            gumball_rattle_timer: 0.0,
            teapot_whistle_timer: 0.0,
            cookie_tin_rattle_timer: 0.0,
            kitchen_clock_tick_timer: 0.0,
            off_hook_phone_hum_timer: 0.0,
            hairdryer_hum_timer: 0.0,
            sewing_machine_click_timer: 0.0,
            herb_sieve_scrape_timer: 0.0,
            vacuum_whine_timer: 0.0,
            mop_bucket_slosh_timer: 0.0,
            wristwatch_tick_timer: 0.0,
            iron_hiss_timer: 0.0,
            grandfather_clock_tick_timer: 0.0,
            rocking_chair_creak_timer: 0.0,
            tall_lamp_buzz_timer: 0.0,
            colander_tap_timer: 0.0,
            pet_bowl_slosh_timer: 0.0,
            music_box_melody_timer: 0.0,
            grandfather_chime_timer: 0.0,
            turntable_hiss_timer: 0.0,
            chalkboard_scratch_timer: 0.0,
            kitchen_calendar_turn_timer: 0.0,
            telescope_click_timer: 0.0,
            ornate_cuckoo_call_timer: 0.0,
            fireplace_crackle_timer: 0.0,
            aquarium_hum_timer: 0.0,
            wingback_creak_timer: 0.0,
            pocket_watch_tick_timer: 0.0,
            bell_pull_ring_timer: 0.0,
            bell_rack_ding_timer: 0.0,
            chess_move_click_timer: 0.0,
            hobby_horse_roll_timer: 0.0,
            speaker_announce_timer: 0.0,
            fire_alarm_beep_timer: 0.0,
            phone_ring_timer: 0.0,
            broken_phone_spark_timer: 0.0,
            vending_hum_timer: 0.0,
            junction_arc_timer: 0.0,
            keycard_beep_timer: 0.0,
            cardboard_shift_timer: 0.0,
            living_plant_rustle_timer: 0.0,
            walkie_static_timer: 0.0,
            wall_clock_tick_timer: 0.0,
            cracked_clock_creak_timer: 0.0,
            ceiling_drip_audio_timer: 0.0,
            ceiling_cable_spark_timer: 0.0,
            thermostat_click_timer: 0.0,
            handbell_chime_timer: 0.0,
            digital_clock_buzz_timer: 0.0,
            alarm_ring_timer: 0.0,
            dial_tone_timer: 0.0,
            drain_gurgle_timer: 0.0,
            faucet_drip_timer: 0.0,
            haunted_whisper_timer: 0.0,
            wet_thunder_timer: 0.0,
            wet_splash_timer: 0.0,
            derelict_grinder_timer: 0.0,
            still_echo_timer: 0.0,
            haunted_step_timer: 0.0,
            electric_surge_timer: 0.0,
            normal_notice_timer: 0.0,
            distant_knock_timer: 0.0,
            door_opened_timer: 0.0,
            door_opened_flash_t: 0.0,
            door_dim_pulse_t: 0.0,
            elevator_ding_timer: 0.0,
            elevator_ding_flash_t: 0.0,
            ding_bright_pulse_t: 0.0,
            voice_sample_timer: 0.0,
            voice_sample_flash_t: 0.0,
            static_burst_timer: 0.0,
            static_burst_flash_t: 0.0,
            reality_shift_timer: 0.0,
            reality_shift_t: 0.0,
            settle_timer: 0.0,
            settle_visual_t: 0.0,
            pattern_break_timer: 0.0,
            pattern_break_t: 0.0,
            clock_skip_timer: 0.0,
            clock_skip_t: 0.0,
            voice_hue_shift_t: 0.0,
            static_chromatic_t: 0.0,
            last_rare_fire_t: 0.0,
            knock_flash_t: 0.0,
            knock_tint_t: 0.0,
            knock_tint_pan: 0.0,
            last_knock_pan: 0.0,
            room_tone_cur_gain: 0.0,
            room_tone_cur_lp: 0.0,
            hum_pulse_phase: 0.0,
            hum_pulse_amp_current: 0.0,
            chime_flash_t: 0.0,
            chime_flash_life: 0.0,
            chime_flash_peak: 0.0,
            crouch_tilt_mag_current: 0.0,
            sanity_roll_amp_current: 0.0,
            sanity_roll_freq_current: 0.0,
            rain_score_current: 0.0,
            electric_surge_t: 0.0,
            panel_lfo_amp_current: 0.0,
            panel_lfo_freq_mul_current: 0.0,
            blowout_milestone_shown: false,
            relight_milestone_shown: false,
            archive_milestone_shown: false,
            escalation_toast_fired: false,
            director: Default::default(),
            player_field_t: 0.0,
            heart_t: 0.0,
            dark_heart_t: 0.0,
            crit_sanity_accum: 0.0,
            sanity_beat_t: 0.0,
            sanity_whisper_t: 0.0,
            sanity_was_impaired: false,
            sanity_was_breaking: false,
            sanity_recovery_tally: 0.0,
            sanity_impaired_tally: 0.0,
            sanity_breaking_tally: 0.0,
            sanity_mini_tally: 0.0,
            hummer_t: 0.0,
            zone_toast_t: 0.0,
            item_hint_water: String::new(),
            item_hint_cell: String::new(),
            hinted_bottle: false,
            hinted_battery: false,
            cry_cooldown: 0.0,
            cries: 0,
            cries_on_floor: 0,
            cry_last_at: None,
            cry_last_line: None,
            cry_dim_t: 0.0,
            cry_dim_lerp: 0.0,
            last_step: 0.0,
            shake_t: 0.0,
            camera_dip_t: 0.0,
            journal_open: false,
            journal_cursor: 0.0,
            debug_on: false,
            grain_t: 0.0,
            grain_opacity_cur: 0.0,
            aberration_opacity_cur: 0.0,
            lore_archivist_shown: false,
            survivor_shown: false,
            storyteller_shown: false,
            completionist_shown: false,
            death_count: 0.0,
            last_death_t: 0.0,
            ironwill_shown: false,
            in_stairwell: false,
            stair_near_exit: false,
            in_loop: false,
            loop_near_exit: false,
            state: String::new(),
            falling_t: 0.0,
            god_mode: false,
            creature_run: Default::default(),
            showcase_company: false,
            god_deaths: 0.0,
            level_start_t: 0.0,
            pool_t: 0.0,
            glitch_t: 0.0,
            ghost_step_t: 0.0,
            fps_frames: 0.0,
            fps_t: 0.0,
            fps: 0.0,
            fixed_step_dt: 0.0,
            sim_frozen: false,
            sim_no_render: false,
        };
        g.splash_flash_peak = 1.6;
        g.room_tone_cur_gain = 0.01;
        g.room_tone_cur_lp = 900.0;
        g.hum_pulse_amp_current = 0.015;
        g.chime_flash_life = 0.9;
        g.chime_flash_peak = 0.25;
        g.crouch_tilt_mag_current = 0.06;
        g.sanity_roll_amp_current = 0.02;
        g.sanity_roll_freq_current = 1.7;
        g.panel_lfo_amp_current = 0.04;
        g.panel_lfo_freq_mul_current = 1.0;
        g.item_hint_water = "[G] DRINK".to_string();
        g.item_hint_cell = "[R] SWAP".to_string();
        g.grain_opacity_cur = 0.08;
        g.state = "menu".to_string();
        g.death_tally.insert("entity".into(), BTreeMap::new());
        g.death_tally.insert("sanity".into(), BTreeMap::new());
        g
    }

    // ------------------------------------------------------------ tile helpers
    /// Index of tile (x, z) in `tiles`
    #[inline]
    pub fn t_idx(&self, x: impl Num, z: impl Num) -> usize {
        (z.f() as i64 * self.w as i64 + x.f() as i64) as usize
    }
    /// Whether tile (x, z) is inside the grid
    #[inline]
    pub fn in_grid(&self, x: impl Num, z: impl Num) -> bool {
        let (x, z) = (x.f(), z.f());
        x >= 0.0 && z >= 0.0 && x < self.w as f64 && z < self.h as f64
    }
    /// Tile value at (x, z); outside the grid reads as wall
    #[inline]
    pub fn tile(&self, x: impl Num, z: impl Num) -> u8 {
        if self.in_grid(x, z) {
            self.tiles[self.t_idx(x, z)]
        } else {
            1
        }
    }
    /// Whether tile (x, z) is a wall; outside the grid counts as wall
    #[inline]
    pub fn is_wall_tile(&self, x: impl Num, z: impl Num) -> bool {
        !self.in_grid(x, z) || self.tiles[self.t_idx(x, z)] == 1
    }
    /// The height of the floor under world point (x, z): 0 on the rooms; on
    /// the Building's floors the tile's floor, or along a stair's run
    pub fn floor_y_at(&self, px: f64, pz: f64) -> f64 {
        if self.ground.is_empty() {
            return 0.0;
        }
        let (u, v) = (self.to_u(px), self.to_v(pz));
        let (x, z) = (u.floor().clamp(0.0, (self.w - 1) as f64) as i32, v.floor().clamp(0.0, (self.h - 1) as f64) as i32);
        match self.ground.get((z * self.w + x) as usize) {
            Some(g) => g.at(x, z, u, v),
            None => 0.0,
        }
    }
    /// The height of the floor of tile (x, z) at its centre
    pub fn floor_y_of(&self, x: i32, z: i32) -> f64 {
        self.floor_y_at(self.wx(x), self.wz(z))
    }
    /// Tile x → world x of the tile centre
    #[inline]
    pub fn wx(&self, t: impl Num) -> f64 {
        (t.f() - self.w as f64 / 2.0 + 0.5) * CELL
    }
    /// Tile z → world z of the tile centre
    #[inline]
    pub fn wz(&self, t: impl Num) -> f64 {
        (t.f() - self.h as f64 / 2.0 + 0.5) * CELL
    }
    /// World x → tile x
    #[inline]
    pub fn tx(&self, w: impl Num) -> i32 {
        (w.f() / CELL + self.w as f64 / 2.0).floor() as i32
    }
    /// World z → tile z
    #[inline]
    pub fn tz(&self, w: impl Num) -> i32 {
        (w.f() / CELL + self.h as f64 / 2.0).floor() as i32
    }
    /// World x / z → fractional tile coordinate
    #[inline]
    pub fn to_u(&self, w: f64) -> f64 {
        w / CELL + self.w as f64 / 2.0
    }
    #[inline]
    pub fn to_v(&self, w: f64) -> f64 {
        w / CELL + self.h as f64 / 2.0
    }
    /// Key of the level's weather
    #[inline]
    pub fn weather_key(&self) -> &'static str {
        self.level_weather.key
    }

    /// Registers a per-frame animator `f(g, dt, t)`
    pub fn animate(&mut self, f: impl FnMut(&mut Game, f64, f64) + 'static) {
        self.animators.push(Box::new(f));
    }
    /// Registers an interactable: position, reach radius, prompt, enabled test and use action
    pub fn interact(
        &mut self,
        pos: Vec3,
        r: f64,
        prompt: &str,
        enabled: impl Fn(&Game) -> bool + 'static,
        on_use: impl FnMut(&mut Game) + 'static,
    ) {
        self.interactables.push(Interactable::new(pos, r, prompt, enabled, on_use));
    }

    // ------------------------------------------------------------ platform services
    /// Real-time clock (ms)
    pub fn clock_ms(&self) -> f64 {
        self.now_ms
    }
    /// Runs `f` on the main thread once `ms` real milliseconds have passed; returns its id
    pub fn schedule(&mut self, ms: f64, f: impl FnOnce(&mut Game) + 'static) -> u64 {
        let id = self.next_timeout_id;
        self.next_timeout_id += 1;
        self.timeouts.push(Timeout { id, at_ms: self.now_ms + ms.max(0.0), f: Some(Box::new(f)) });
        id
    }
    /// Cancels a scheduled callback by id
    pub fn cancel_scheduled(&mut self, id: u64) {
        self.timeouts.retain(|t| t.id != id);
    }
    /// Run every timeout that is due (the app calls this every frame).
    pub fn run_scheduled(&mut self) {
        loop {
            let now = self.now_ms;
            let Some(i) = self
                .timeouts
                .iter()
                .enumerate()
                .filter(|(_, t)| t.at_ms <= now)
                .min_by(|a, b| a.1.at_ms.partial_cmp(&b.1.at_ms).unwrap())
                .map(|(i, _)| i)
            else {
                break;
            };
            let mut t = self.timeouts.remove(i);
            if let Some(f) = t.f.take() {
                f(self);
            }
        }
    }
    /// Reads a stored value
    pub fn storage_get(&self, k: &str) -> Option<String> {
        self.storage.get(k).cloned()
    }
    /// Stores a value (persisted to the save file)
    pub fn storage_set(&mut self, k: &str, v: &str) {
        self.storage.insert(k.to_string(), v.to_string());
        self.storage_flush();
    }
    /// Removes a stored value (persisted to the save file)
    pub fn storage_remove(&mut self, k: &str) {
        self.storage.remove(k);
        self.storage_flush();
    }
    fn storage_flush(&self) {
        if let Some(p) = &self.storage_path {
            let mut s = String::new();
            for (k, v) in &self.storage {
                s.push_str(&format!("{}\t{}\n", escape(k), escape(v)));
            }
            let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
            let _ = std::fs::write(p, s);
        }
    }
    /// Loads the store from the save file and remembers its path for later writes.
    pub fn storage_load(&mut self, path: std::path::PathBuf) {
        if let Ok(s) = std::fs::read_to_string(&path) {
            for line in s.lines() {
                if let Some((k, v)) = line.split_once('\t') {
                    self.storage.insert(unescape(k), unescape(v));
                }
            }
        }
        self.storage_path = Some(path);
    }
    /// Whether the launch flags contain `s`
    pub fn has_flag(&self, s: &str) -> bool {
        self.launch_flags.contains(s)
    }
    /// Whether key `code` is held down
    pub fn key(&self, code: &str) -> bool {
        self.keys.contains(code)
    }

    // ------------------------------------------------------------ audio helpers
    /// Current audio-clock time (0 without audio)
    pub fn ac_time(&self) -> f64 {
        self.ac.as_ref().map(|a| a.current_time()).unwrap_or(0.0)
    }
    /// A one-shot's level and pan after `sfx_route` (its gain, and its pan
    /// for a voice without one of its own)
    pub(crate) fn routed(&self, gain: f64, pan: f64) -> (f64, f64) {
        match &self.sfx_route {
            Some(r) => (gain * r.gain, if pan == 0.0 { r.pan } else { pan }),
            None => (gain, pan),
        }
    }
    /// Connects a one-shot's last stage: to `sfx_route`'s voice slot when it
    /// has one, else to master, through a stereo panner when `pan` is not 0
    pub(crate) fn one_shot_out(&self, ac: &AudioContext, master: &AudioNode, g: &AudioNode, pan: f64) {
        if let Some(dest) = self.sfx_route.as_ref().and_then(|r| r.dest.as_ref()) {
            g.connect(dest);
        } else if pan != 0.0 {
            let p = ac.create_stereo_panner();
            p.pan().set_value(pan.clamp(-1.0, 1.0));
            g.connect(&p);
            p.connect(master);
        } else {
            g.connect(master);
        }
    }
    pub fn blip(&self, freq: f64, dur: f64, ty: &str, gain: f64, slide_to: Option<f64>, delay: f64, pan: f64) {
        let (Some(ac), Some(master)) = (&self.ac, &self.master) else { return };
        let (gain, pan) = self.routed(gain, pan);
        let t0 = ac.current_time() + delay;
        let o = ac.create_oscillator();
        o.set_type(ty);
        o.frequency().set_value_at_time(freq, t0);
        if let Some(s) = slide_to {
            if s != 0.0 {
                o.frequency().exponential_ramp_to_value_at_time(s.max(20.0), t0 + dur);
            }
        }
        let g = ac.create_gain();
        g.gain().set_value_at_time(gain, t0);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
        o.connect(&g);
        self.one_shot_out(ac, master, &g, pan);
        o.start(t0);
        o.stop(t0 + dur + 0.05);
    }
    pub fn noise_burst(&self, dur: f64, freq: f64, q: f64, gain: f64, ty: &str, slide_to: Option<f64>, delay: f64, pan: f64) {
        let (Some(ac), Some(master), Some(nb)) = (&self.ac, &self.master, &self.noise_buf) else { return };
        let (gain, pan) = self.routed(gain, pan);
        let t0 = ac.current_time() + delay;
        let s = ac.create_buffer_source();
        s.set_buffer(nb);
        s.set_loop(true);
        let f = ac.create_biquad_filter();
        f.set_type(ty);
        f.frequency().set_value_at_time(freq, t0);
        f.q().set_value(q);
        if let Some(sl) = slide_to {
            if sl != 0.0 {
                f.frequency().exponential_ramp_to_value_at_time(sl.max(30.0), t0 + dur);
            }
        }
        let g = ac.create_gain();
        g.gain().set_value_at_time(gain, t0);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
        s.connect(&f);
        f.connect(&g);
        self.one_shot_out(ac, master, &g, pan);
        // from a random point of the 1 s noise loop, so no two bursts sound alike
        s.start_at(t0, crate::base::rng::native_rnd() * 0.95, None);
        s.stop(t0 + dur + 0.05);
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}
fn unescape(s: &str) -> String {
    let mut o = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => o.push('\n'),
                Some('t') => o.push('\t'),
                Some(x) => o.push(x),
                None => {}
            }
        } else {
            o.push(c);
        }
    }
    o
}
