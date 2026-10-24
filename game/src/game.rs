//! `Game`: the whole mutable state of backrooms.js, one field per JS global.
//!
//! Naming rule: the JS global `fooBar` is the field `g.foo_bar`. The few
//! exceptions are whole subsystems kept in their own structs: `g.tex`
//! (textures section), `g.audio` (audio nodes), `g.dom` (the page's
//! elements), `g.pp` (`*Positions` lists), `g.mats` (shared materials).
//! Pure data tables (`LORE_ENTRIES`, `*_BY_WEATHER`, ...) are not fields:
//! they live as items in the module that owns them.

use crate::config::*;
use crate::positions::PropPositions;
use crate::records::*;
use crate::state::*;
use engine::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// A pending `setTimeout` callback.
pub struct Timeout { pub id: u64, pub at_ms: f64, pub f: Option<Box<dyn FnOnce(&mut Game)>> }

pub struct Game {
    // ------------------------------------------------------------ level grid
    /// `W`, `H` — tile grid dimensions (odd)
    pub w: i32,
    pub h: i32,
    /// `tiles` — 1 = wall, 0 = open, 2 = hidden from the current placer
    pub tiles: Vec<u8>,
    /// `layout` is set (archgen layout of the current level)
    pub has_layout: bool,
    // ------------------------------------------------------------ scene
    pub scene: Obj,
    pub camera: Obj,
    /// `world` — per-level group every placer adds to
    pub world: Obj,
    pub animators: Vec<Animator>,
    pub interactables: Vec<Interactable>,
    /// every `*Positions` list
    pub pp: PropPositions,
    /// shared materials (`woodMat`, `paperMat`, `glassMat`, ...)
    pub mats: SharedMats,
    /// `dustMat` (the dust-mote PointsMaterial)
    pub dust_mat: Mat,
    /// `flashlight` (the player's SpotLight)
    pub flashlight: Obj,
    // ------------------------------------------------------------ run state
    pub level_weather: &'static Weather,
    pub player: Player,
    /// `perfT` — simulation clock (seconds)
    pub perf_t: f64,
    /// `currentSeed` — seed of the level being built / played
    pub current_seed: u32,
    /// `runSeed`
    pub run_seed: u32,
    /// `levelNum`
    pub level_num: i32,
    pub gameplay_r1_state: Option<GameplayR1State>,
    // ------------------------------------------------------------ audio
    /// `AC`
    pub ac: Option<AudioContext>,
    /// `master`
    pub master: Option<AudioNode>,
    /// `noiseBuf`
    pub noise_buf: Option<AudioBuffer>,
    /// `muted`
    pub muted: bool,
    /// every other persistent audio node (`droneGain`, `staminaBreath*`, `soundscape`, ...)
    pub audio: crate::audio_sys::AudioSys,
    // ------------------------------------------------------------ hud (legacy placer-facing)
    pub msg_text: String,
    /// `msgT`
    pub msg_t: f64,
    // ------------------------------------------------------------ registry
    /// every prop placer by JS name (filled by the app at startup)
    pub placers: Vec<(&'static str, PlacerFn)>,

    // ============================================================ platform
    /// `keys[e.code]` — keys held down
    pub keys: HashSet<String>,
    /// mouse motion since the last frame (pointer-locked movementX/Y)
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    /// `document.pointerLockElement === canvas`
    pub pointer_locked: bool,
    /// requests the app applies after the frame
    pub want_pointer_lock: Option<bool>,
    pub want_fullscreen: Option<bool>,
    pub want_quit: bool,
    /// `warmUpGPU()` asked the renderer to compile the level's programs
    pub want_warm_up: bool,
    /// `innerWidth` / `innerHeight` (CSS px) and `devicePixelRatio`
    pub inner_width: f64,
    pub inner_height: f64,
    pub device_pixel_ratio: f64,
    /// `setTimeout` queue and the real-time clock (`performance.now()`, ms)
    pub timeouts: Vec<Timeout>,
    pub next_timeout_id: u64,
    pub now_ms: f64,
    /// `location.hash` (debug switches: '#nosound', '#scp=...', ...)
    pub location_hash: String,
    /// `localStorage`
    pub storage: BTreeMap<String, String>,
    pub storage_path: Option<std::path::PathBuf>,
    /// the page's elements
    pub dom: crate::dom::Dom,

    // ============================================================ base
    /// `randTrace`
    pub rand_trace: RandTrace,

    // ============================================================ world / architecture (1381–1612, 3046–5351)
    /// `tileCeil` — ceiling height per tile (a Float32Array in JS)
    pub tile_ceil: Vec<f32>,
    /// `layout` — the archgen layout of the current level
    pub layout: Option<crate::archgen::Layout>,
    /// `solids` — archgen solid boxes (collision, sight, rendering)
    pub solids: Option<crate::archgen::Solids>,
    /// the textures section (`wallpaperVariants`, `carpetVariants`, `dustSprite`, `staticTex`, ...)
    pub tex: crate::textures::Textures,
    /// `surfaces` = `makeSurfaces(canvasTex, nativeRandom)`
    pub surfaces: crate::surfaces::Surfaces,
    /// `levelScp` — the level's anomaly ('stairwell' | 'store' | 'house' | 'loop')
    pub level_scp: Option<String>,
    /// `omenTimers` (timeout ids)
    pub omen_timers: Vec<u64>,
    /// `panels`
    pub panels: Vec<Panel>,
    /// `panelMesh` — instanced ceiling panels
    pub panel_mesh: Option<Obj>,
    /// `pooled` — the panel spotlights (POOLED_LIGHTS), `userData.panel` / `.w` in their user data
    pub pooled: Vec<Obj>,
    /// `pooled[i].userData.panel` (index into panels)
    pub pooled_panel: Vec<Option<usize>>,
    /// `pooled[i].userData.w` (fade weight)
    pub pooled_w: Vec<f64>,
    pub almonds: Vec<Pickup>,
    /// `batteryItems`
    pub battery_items: Vec<Pickup>,
    /// `regionMap` — region id per tile
    pub region_map: Vec<u8>,
    /// `regionInfo`
    pub region_info: Vec<RegionInfo>,
    /// `ceilRegion`
    pub ceil_region: Vec<u8>,
    pub floor_holes: Vec<FloorHole>,
    /// `barricadeTiles` (tile indices)
    pub barricade_tiles: Vec<usize>,
    pub pool_bounds: Option<PoolBounds>,
    pub papers: Vec<PaperPile>,
    pub lore_notes: Vec<LoreNote>,
    pub safe_room: Option<SafeRoom>,
    pub broken_elevator: Option<BrokenElevator>,
    pub light_switches: Vec<LightSwitch>,
    pub haunted_candles: Vec<CandleState>,
    /// `rateDeltaBuf`
    pub rate_delta_buf: Vec<f64>,
    /// `lastRareFireByCause`
    pub last_rare_fire_by_cause: RareFire,
    /// `rareEventTally[cause][weather]`
    pub rare_event_tally: BTreeMap<String, BTreeMap<String, f64>>,
    pub ambient_log: Vec<AmbientLogEntry>,
    /// `ambientTally[tag][weather]`
    pub ambient_tally: BTreeMap<String, BTreeMap<String, f64>>,
    pub footstep_fx_tally: WeatherTally,
    pub crouch_fx_tally: WeatherTally,
    /// `chimeFlashColor` (a CSS 'rgba(r, g, b' prefix)
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
    /// `archMats` (material cache by key)
    pub arch_mats: HashMap<String, Mat>,
    pub unit_box_geo: Geo,
    pub unit_plane_geo: Geo,
    /// `disposeStats`
    pub dispose_stats: [u32; 3],
    pub prop_cull: PropCull,
    pub placement_stats: BTreeMap<String, f64>,
    pub settle_stats: BTreeMap<String, f64>,
    pub placer_theme_cache: HashMap<String, &'static str>,
    /// `themeStats` (ran, skipped, byTheme[theme] = (ran, skipped))
    pub theme_stats_ran: u32,
    pub theme_stats_skipped: u32,
    pub theme_stats_by_theme: BTreeMap<String, (u32, u32)>,
    /// `placerDist`
    pub placer_dist: Option<Vec<i32>>,
    /// `buildProfile.t`
    pub build_profile: BTreeMap<String, f64>,

    // ============================================================ anomalies (1581–2556)
    pub store_state: StoreState,
    pub staff_mat: Option<Mat>,
    pub house_state: HouseState,
    /// `SHADOW_U.uTime.value` / `SHADOW_U.uSmoke.value`
    pub shadow_u_time: f64,
    pub shadow_u_smoke: f64,
    pub shadow_mat: Mat,
    pub shadow_edge_mat: Mat,

    // ============================================================ renderer / lighting (2556–3046)
    /// `scene.background`
    pub background: Color,
    /// `scene.fog` (FogExp2)
    pub fog_color: Color,
    pub fog_density: f64,
    /// the `LL` uniforms of the local-light patch
    pub ll_on: f64,
    pub ll_map: Option<Tex>,
    pub ll_grid: [f64; 4],
    pub ll_scale: f64,
    pub ll_amb_d: Vec3,
    pub ll_sky_d: Vec3,
    pub ll_gnd_d: Vec3,
    pub ll_fog_lit: Color,
    pub ll_fog_ratio: f64,
    /// `ambLight`, `hemiLight`
    pub amb_light: Obj,
    pub hemi_light: Obj,
    /// `lightMapCpu`
    pub light_map_cpu: Vec<f64>,
    pub player_light: Obj,
    pub halluc_textures: Vec<Tex>,
    pub halluc_shape: Obj,
    pub flashlight_target: Obj,
    pub dust_geo: Geo,
    pub dust_positions: Vec<f32>,
    pub dust_velocities: Vec<f32>,
    /// `dust` (Points)
    pub dust: Obj,
    pub rain_streak_tex: Tex,
    /// `rainWindows` (overlay meshes whose map scrolls)
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

    // ============================================================ entities (61153–62870)
    pub entities: Vec<Entity>,
    /// `playerField` — BFS distance field to the player
    pub player_field: Option<std::rc::Rc<Vec<i32>>>,
    /// `humPanel`
    pub hum_panel: Option<HumPanel>,
    /// `lastZoneArch`
    pub last_zone_arch: Option<String>,
    pub zones_seen: HashSet<String>,

    // ============================================================ player (66024–67072)
    pub paper_marker_geo: Geo,
    pub paper_marker_mat: Mat,
    pub guide_arrows: Vec<GuideArrow>,
    pub arrow_geo: Geo,
    pub arrow_mat: Mat,
    pub touch_input: TouchInput,

    // ============================================================ hud (67072–68246)
    /// `lastRenderedCur`
    pub last_rendered_cur: Option<i64>,
    /// `loreFound` (note ids)
    pub lore_found: BTreeSet<usize>,
    pub lore_arcs_completed: BTreeSet<String>,
    /// `lastDeathCause`
    pub last_death_cause: Option<String>,
    /// `deathTally[cause][weather]`
    pub death_tally: BTreeMap<String, BTreeMap<String, f64>>,

    // ---- added by wb
    /// `pendingDispose` — the last level's resources, held until the new level is up
    pub pending_dispose: Option<crate::world_arch::Resources>,
    /// `propCull.list[i].x` / `.z` (the sphere centres; `prop_cull.list` holds `o` and `r`)
    pub prop_cull_xz: Vec<(f64, f64)>,
    /// `window.__poolPlan` — the pool room buildWorld reserved; placeProps takes it
    pub pool_plan: Option<crate::world::PoolPlan>,
    /// `placerRegistry`-style index of `placers` by name (and the registry length it was built from)
    pub placer_index: (usize, HashMap<&'static str, PlacerFn>),
    /// test seam: build the world with placeProps / placeStore / placeHouse as no-ops
    /// (what tools/world_fixtures.mjs does on the JS side)
    pub test_stub_core_placers: bool,
    // ---- end added by wb

    // ---- added by hud
    /// `grainCanvas` / `grainCtx` (line 67078; 480×300 film grain, made by initHud)
    pub grain_canvas: Option<std::rc::Rc<std::cell::RefCell<Canvas>>>,
    /// the page renderer's own state (per-element canvases, running CSS
    /// transitions/animations, hit rects) — see hud_draw.rs
    pub hud_ui: crate::hud_draw::HudUi,
    // ---- end added by hud

    // ---- added by EN (entities)
    /// what the JS hangs on an entity object that `records::Entity` has no
    /// typed slot for (the rig of `e.parts`, `e.aimField`, `e.tic`, `e.voice`),
    /// by `e.g.id()` (see entities.rs)
    pub ent_ext: HashMap<u64, crate::entities::EntExt>,
    // ---- end EN

    // ---- added by an (anomalies)
    /// `storeState.hides` as the JS keeps them (`{ obj, ...obj.userData.hide }`);
    /// `store_state.hides` mirrors their positions
    pub store_hide_spots: Vec<crate::anomalies::StoreHideSpot>,
    /// the pending tick of `storeMusic`'s `setInterval(step, 3200)` (cleared by `music.stop()`)
    pub store_music_timer: Option<u64>,
    // ---- added by md (modes glue)
    /// `stairAudio.voice` — the child crying far below (`createChildVoice`), while in the Stairwell
    pub stair_voice: Option<crate::soundscape::ChildVoice>,
    /// `stairAudio.slot` — its spatial voice slot (`voiceSlot(4)`)
    pub stair_slot: Option<crate::soundscape::VoiceSlot>,

    // ============================================================ stairwell / loop (68246–68603)
    /// the 3D canvas's CSS filter transition (scene_fx.rs)
    pub scene_filter: crate::scene_fx::SceneFilterState,
    /// `stairwell` = `createStairwell(...)` (built in `boot()`; None in bare test games)
    pub stairwell: Option<crate::stairwell::Stairwell>,
    /// `loopRooms` — `createLoop()` while inside the Loop, else None
    pub loop_rooms: Option<crate::loop_mode::LoopMode>,
    pub stair_audio: StairAudio,
    /// `loopPaper` (the Loop's wallpapers, clean and rotten)
    pub loop_paper: Vec<Tex>,
    pub loop_audio: LoopAudio,

    // ============================================================ flow (68603–70145)
    pub god_causes: BTreeMap<String, f64>,
    /// `poolEdge`
    pub pool_edge: f64,
    /// `poolWanted` (panel indices)
    pub pool_wanted: HashSet<usize>,
    /// `biasedAmbient` — (weight, sfx name)
    pub biased_ambient: Vec<(f64, &'static str)>,
    /// `glitchNextT`, `flickerWaveNextT`
    pub glitch_next_t: f64,
    pub flicker_wave_next_t: f64,

    // ============================================================ main loop
    pub quality: Quality,

    // ============================================================ simple globals (numbers / flags / strings), by section
    // ---- render
    /// `lightMapOff` (line 2693)
    pub light_map_off: f64,
    /// `hallucVisibleT` (line 2885)
    pub halluc_visible_t: f64,
    /// `splashFlashT` (line 2975)
    pub splash_flash_t: f64,
    /// `splashFlashPeak` (line 2976)
    pub splash_flash_peak: f64,
    /// `thunderFlashT` (line 2981)
    pub thunder_flash_t: f64,
    // ---- world build
    /// `motionSensorLastClickT` (line 3076)
    pub motion_sensor_last_click_t: f64,
    /// `faultyTripCount` (line 3094)
    pub faulty_trip_count: f64,
    /// `candleBlowoutTally` (line 3096)
    pub candle_blowout_tally: f64,
    /// `candleRelightTally` (line 3097)
    pub candle_relight_tally: f64,
    /// `candleFizzleTally` (line 3098)
    pub candle_fizzle_tally: f64,
    /// `rateDriftTicks` (line 3099)
    pub rate_drift_ticks: f64,
    /// `wetDripTimer` (line 3101)
    pub wet_drip_timer: f64,
    /// `stillWindTimer` (line 3102)
    pub still_wind_timer: f64,
    /// `derelictClatterTimer` (line 3103)
    pub derelict_clatter_timer: f64,
    /// `electricCrackleTimer` (line 3104)
    pub electric_crackle_timer: f64,
    /// `socketSparkTimer` (line 3105)
    pub socket_spark_timer: f64,
    /// `fountainDripTimer` (line 3106)
    pub fountain_drip_timer: f64,
    /// `radiatorHissTimer` (line 3107)
    pub radiator_hiss_timer: f64,
    /// `glowHumTimer` (line 3108)
    pub glow_hum_timer: f64,
    /// `cauldronBubbleTimer` (line 3109)
    pub cauldron_bubble_timer: f64,
    /// `metronomeTickTimer` (line 3110)
    pub metronome_tick_timer: f64,
    /// `phonoCrackleTimer` (line 3113)
    pub phono_crackle_timer: f64,
    /// `lanternCreakTimer` (line 3114)
    pub lantern_creak_timer: f64,
    /// `sewingWhirrTimer` (line 3117)
    pub sewing_whirr_timer: f64,
    /// `watchTickTimer` (line 3118)
    pub watch_tick_timer: f64,
    /// `batSqueakTimer` (line 3121)
    pub bat_squeak_timer: f64,
    /// `globeCreakTimer` (line 3122)
    pub globe_creak_timer: f64,
    /// `hourglassWhooshTimer` (line 3125)
    pub hourglass_whoosh_timer: f64,
    /// `ghostWhisperTimer` (line 3127)
    pub ghost_whisper_timer: f64,
    /// `chainJangleTimer` (line 3129)
    pub chain_jangle_timer: f64,
    /// `paperRustleTimer` (line 3131)
    pub paper_rustle_timer: f64,
    /// `pianoChordTimer` (line 3133)
    pub piano_chord_timer: f64,
    /// `chairCreakTimer` (line 3135)
    pub chair_creak_timer: f64,
    /// `crtHissTimer` (line 3137)
    pub crt_hiss_timer: f64,
    /// `exitBuzzTimer` (line 3139)
    pub exit_buzz_timer: f64,
    /// `puddleDripTimer` (line 3141)
    pub puddle_drip_timer: f64,
    /// `chandelierChimeTimer` (line 3143)
    pub chandelier_chime_timer: f64,
    /// `cassetteHissTimer` (line 3145)
    pub cassette_hiss_timer: f64,
    /// `teacupClinkTimer` (line 3147)
    pub teacup_clink_timer: f64,
    /// `lockerSlamTimer` (line 3149)
    pub locker_slam_timer: f64,
    /// `bulletinFlutterTimer` (line 3151)
    pub bulletin_flutter_timer: f64,
    /// `alarmBeepTimer` (line 3153)
    pub alarm_beep_timer: f64,
    /// `payPhoneRingTimer` (line 3155)
    pub pay_phone_ring_timer: f64,
    /// `radioStaticTimer` (line 3157)
    pub radio_static_timer: f64,
    /// `keychainJangleTimer` (line 3159)
    pub keychain_jangle_timer: f64,
    /// `elevatorBeepTimer` (line 3161)
    pub elevator_beep_timer: f64,
    /// `plantRustleTimer` (line 3163)
    pub plant_rustle_timer: f64,
    /// `fanWhooshTimer` (line 3165)
    pub fan_whoosh_timer: f64,
    /// `fanSqueakTimer` (line 3167)
    pub fan_squeak_timer: f64,
    /// `detectorChirpTimer` (line 3168)
    pub detector_chirp_timer: f64,
    /// `birdChirpTimer` (line 3170)
    pub bird_chirp_timer: f64,
    /// `cricketChirpTimer` (line 3171)
    pub cricket_chirp_timer: f64,
    /// `thunderRumbleTimer` (line 3172)
    pub thunder_rumble_timer: f64,
    /// `coinTinkTimer` (line 3173)
    pub coin_tink_timer: f64,
    /// `windGustTimer` (line 3175)
    pub wind_gust_timer: f64,
    /// `frogCroakTimer` (line 3176)
    pub frog_croak_timer: f64,
    /// `owlHootTimer` (line 3177)
    pub owl_hoot_timer: f64,
    /// `tickTockTimer` (line 3178)
    pub tick_tock_timer: f64,
    /// `wolfHowlTimer` (line 3180)
    pub wolf_howl_timer: f64,
    /// `rainPatterTimer` (line 3181)
    pub rain_patter_timer: f64,
    /// `leafRustleTimer` (line 3182)
    pub leaf_rustle_timer: f64,
    /// `cuckooCallTimer` (line 3183)
    pub cuckoo_call_timer: f64,
    /// `plushShuffleTimer` (line 3185)
    pub plush_shuffle_timer: f64,
    /// `gumballRattleTimer` (line 3187)
    pub gumball_rattle_timer: f64,
    /// `teapotWhistleTimer` (line 3189)
    pub teapot_whistle_timer: f64,
    /// `cookieTinRattleTimer` (line 3191)
    pub cookie_tin_rattle_timer: f64,
    /// `kitchenClockTickTimer` (line 3193)
    pub kitchen_clock_tick_timer: f64,
    /// `offHookPhoneHumTimer` (line 3195)
    pub off_hook_phone_hum_timer: f64,
    /// `hairdryerHumTimer` (line 3197)
    pub hairdryer_hum_timer: f64,
    /// `sewingMachineClickTimer` (line 3199)
    pub sewing_machine_click_timer: f64,
    /// `herbSieveScrapeTimer` (line 3201)
    pub herb_sieve_scrape_timer: f64,
    /// `vacuumWhineTimer` (line 3203)
    pub vacuum_whine_timer: f64,
    /// `mopBucketSloshTimer` (line 3205)
    pub mop_bucket_slosh_timer: f64,
    /// `wristwatchTickTimer` (line 3207)
    pub wristwatch_tick_timer: f64,
    /// `ironHissTimer` (line 3209)
    pub iron_hiss_timer: f64,
    /// `grandfatherClockTickTimer` (line 3211)
    pub grandfather_clock_tick_timer: f64,
    /// `rockingChairCreakTimer` (line 3213)
    pub rocking_chair_creak_timer: f64,
    /// `tallLampBuzzTimer` (line 3215)
    pub tall_lamp_buzz_timer: f64,
    /// `colanderTapTimer` (line 3217)
    pub colander_tap_timer: f64,
    /// `petBowlSloshTimer` (line 3219)
    pub pet_bowl_slosh_timer: f64,
    /// `musicBoxMelodyTimer` (line 3221)
    pub music_box_melody_timer: f64,
    /// `grandfatherChimeTimer` (line 3223)
    pub grandfather_chime_timer: f64,
    /// `turntableHissTimer` (line 3224)
    pub turntable_hiss_timer: f64,
    /// `chalkboardScratchTimer` (line 3226)
    pub chalkboard_scratch_timer: f64,
    /// `kitchenCalendarTurnTimer` (line 3228)
    pub kitchen_calendar_turn_timer: f64,
    /// `telescopeClickTimer` (line 3230)
    pub telescope_click_timer: f64,
    /// `cuckooCallTimerR741` (line 3232)
    pub cuckoo_call_timer_r741: f64,
    /// `fireplaceCrackleTimer` (line 3234)
    pub fireplace_crackle_timer: f64,
    /// `aquariumHumTimer` (line 3236)
    pub aquarium_hum_timer: f64,
    /// `wingbackCreakTimer` (line 3238)
    pub wingback_creak_timer: f64,
    /// `pocketWatchTickTimer` (line 3240)
    pub pocket_watch_tick_timer: f64,
    /// `bellPullRingTimer` (line 3242)
    pub bell_pull_ring_timer: f64,
    /// `bellRackDingTimer` (line 3244)
    pub bell_rack_ding_timer: f64,
    /// `chessMoveClickTimer` (line 3246)
    pub chess_move_click_timer: f64,
    /// `hobbyHorseRollTimer` (line 3248)
    pub hobby_horse_roll_timer: f64,
    /// `speakerAnnounceTimer` (line 3250)
    pub speaker_announce_timer: f64,
    /// `fireAlarmBeepTimer` (line 3251)
    pub fire_alarm_beep_timer: f64,
    /// `phoneRingTimer` (line 3252)
    pub phone_ring_timer: f64,
    /// `brokenPhoneSparkTimer` (line 3253)
    pub broken_phone_spark_timer: f64,
    /// `vendingHumTimer` (line 3254)
    pub vending_hum_timer: f64,
    /// `junctionArcTimer` (line 3255)
    pub junction_arc_timer: f64,
    /// `keycardBeepTimer` (line 3256)
    pub keycard_beep_timer: f64,
    /// `cardboardShiftTimer` (line 3257)
    pub cardboard_shift_timer: f64,
    /// `plantRustleTimerR291` (line 3258)
    pub plant_rustle_timer_r291: f64,
    /// `walkieStaticTimer` (line 3259)
    pub walkie_static_timer: f64,
    /// `wallClockTickTimer` (line 3260)
    pub wall_clock_tick_timer: f64,
    /// `crackedClockCreakTimer` (line 3261)
    pub cracked_clock_creak_timer: f64,
    /// `ceilingDripAudioTimer` (line 3262)
    pub ceiling_drip_audio_timer: f64,
    /// `ceilingCableSparkTimer` (line 3263)
    pub ceiling_cable_spark_timer: f64,
    /// `thermostatClickTimer` (line 3264)
    pub thermostat_click_timer: f64,
    /// `handbellChimeTimer` (line 3265)
    pub handbell_chime_timer: f64,
    /// `digitalClockBuzzTimer` (line 3266)
    pub digital_clock_buzz_timer: f64,
    /// `alarmRingTimer` (line 3267)
    pub alarm_ring_timer: f64,
    /// `dialToneTimer` (line 3268)
    pub dial_tone_timer: f64,
    /// `drainGurgleTimer` (line 3269)
    pub drain_gurgle_timer: f64,
    /// `faucetDripTimer` (line 3270)
    pub faucet_drip_timer: f64,
    /// `hauntedWhisperTimer` (line 3271)
    pub haunted_whisper_timer: f64,
    /// `wetThunderTimer` (line 3272)
    pub wet_thunder_timer: f64,
    /// `wetSplashTimer` (line 3273)
    pub wet_splash_timer: f64,
    /// `derelictGrinderTimer` (line 3274)
    pub derelict_grinder_timer: f64,
    /// `stillEchoTimer` (line 3275)
    pub still_echo_timer: f64,
    /// `hauntedStepTimer` (line 3276)
    pub haunted_step_timer: f64,
    /// `electricSurgeTimer` (line 3277)
    pub electric_surge_timer: f64,
    /// `normalNoticeTimer` (line 3278)
    pub normal_notice_timer: f64,
    /// `distantKnockTimer` (line 3279)
    pub distant_knock_timer: f64,
    /// `doorOpenedTimer` (line 3280)
    pub door_opened_timer: f64,
    /// `doorOpenedFlashT` (line 3281)
    pub door_opened_flash_t: f64,
    /// `doorDimPulseT` (line 3282)
    pub door_dim_pulse_t: f64,
    /// `elevatorDingTimer` (line 3283)
    pub elevator_ding_timer: f64,
    /// `elevatorDingFlashT` (line 3284)
    pub elevator_ding_flash_t: f64,
    /// `dingBrightPulseT` (line 3285)
    pub ding_bright_pulse_t: f64,
    /// `voiceSampleTimer` (line 3286)
    pub voice_sample_timer: f64,
    /// `voiceSampleFlashT` (line 3287)
    pub voice_sample_flash_t: f64,
    /// `staticBurstTimer` (line 3288)
    pub static_burst_timer: f64,
    /// `staticBurstFlashT` (line 3289)
    pub static_burst_flash_t: f64,
    /// `realityShiftTimer` (line 3290)
    pub reality_shift_timer: f64,
    /// `realityShiftT` (line 3291)
    pub reality_shift_t: f64,
    /// `settleTimer` (line 3292)
    pub settle_timer: f64,
    /// `settleVisualT` (line 3293)
    pub settle_visual_t: f64,
    /// `patternBreakTimer` (line 3294)
    pub pattern_break_timer: f64,
    /// `patternBreakT` (line 3295)
    pub pattern_break_t: f64,
    /// `clockSkipTimer` (line 3296)
    pub clock_skip_timer: f64,
    /// `clockSkipT` (line 3297)
    pub clock_skip_t: f64,
    /// `voiceHueShiftT` (line 3298)
    pub voice_hue_shift_t: f64,
    /// `staticChromaticT` (line 3299)
    pub static_chromatic_t: f64,
    /// `lastRareFireT` (line 3300)
    pub last_rare_fire_t: f64,
    /// `knockFlashT` (line 3309)
    pub knock_flash_t: f64,
    /// `knockTintT` (line 3310)
    pub knock_tint_t: f64,
    /// `knockTintPan` (line 3311)
    pub knock_tint_pan: f64,
    /// `lastKnockPan` (line 3312)
    pub last_knock_pan: f64,
    /// `roomToneCurGain` (line 3369)
    pub room_tone_cur_gain: f64,
    /// `roomToneCurLp` (line 3370)
    pub room_tone_cur_lp: f64,
    /// `humPulsePhase` (line 3371)
    pub hum_pulse_phase: f64,
    /// `humPulseAmpCurrent` (line 3372)
    pub hum_pulse_amp_current: f64,
    /// `chimeFlashT` (line 3373)
    pub chime_flash_t: f64,
    /// `chimeFlashLife` (line 3374)
    pub chime_flash_life: f64,
    /// `chimeFlashPeak` (line 3376)
    pub chime_flash_peak: f64,
    /// `crouchTiltMagCurrent` (line 3377)
    pub crouch_tilt_mag_current: f64,
    /// `sanityRollAmpCurrent` (line 3378)
    pub sanity_roll_amp_current: f64,
    /// `sanityRollFreqCurrent` (line 3379)
    pub sanity_roll_freq_current: f64,
    /// `rainScoreCurrent` (line 3380)
    pub rain_score_current: f64,
    /// `electricSurgeT` (line 3381)
    pub electric_surge_t: f64,
    /// `panelLfoAmpCurrent` (line 3382)
    pub panel_lfo_amp_current: f64,
    /// `panelLfoFreqMulCurrent` (line 3383)
    pub panel_lfo_freq_mul_current: f64,
    /// `blowoutMilestoneShown` (line 3399)
    pub blowout_milestone_shown: bool,
    /// `relightMilestoneShown` (line 3400)
    pub relight_milestone_shown: bool,
    /// `archiveMilestoneShown` (line 3401)
    pub archive_milestone_shown: bool,
    /// `escalationToastFired` (line 3402)
    pub escalation_toast_fired: bool,
    // ---- entities
    /// `playerFieldT` (line 62280)
    pub player_field_t: f64,
    /// `heartT` (line 62850)
    pub heart_t: f64,
    /// `darkHeartT` (line 62851)
    pub dark_heart_t: f64,
    /// `critSanityAccum` (line 62855)
    pub crit_sanity_accum: f64,
    /// `sanityBeatT` (line 62856)
    pub sanity_beat_t: f64,
    /// `sanityWhisperT` (line 62857)
    pub sanity_whisper_t: f64,
    /// `sanityWasImpaired` (line 62860)
    pub sanity_was_impaired: bool,
    /// `sanityWasBreaking` (line 62861)
    pub sanity_was_breaking: bool,
    /// `sanityRecoveryTally` (line 62862)
    pub sanity_recovery_tally: f64,
    /// `sanityImpairedTally` (line 62863)
    pub sanity_impaired_tally: f64,
    /// `sanityBreakingTally` (line 62864)
    pub sanity_breaking_tally: f64,
    /// `sanityMiniTally` (line 62865)
    pub sanity_mini_tally: f64,
    // ---- ambient
    /// `hummerT` (line 62882)
    pub hummer_t: f64,
    /// `zoneToastT` (line 62884)
    pub zone_toast_t: f64,
    // ---- player
    /// `itemHintWater` (line 66078)
    pub item_hint_water: String,
    /// `itemHintCell` (line 66079)
    pub item_hint_cell: String,
    /// `hintedBottle` (line 66080)
    pub hinted_bottle: bool,
    /// `hintedBattery` (line 66080)
    pub hinted_battery: bool,
    /// `cryCooldown` (line 66160)
    pub cry_cooldown: f64,
    /// `cryDimT` (line 66162)
    pub cry_dim_t: f64,
    /// `cryDimLerp` (line 66163)
    pub cry_dim_lerp: f64,
    /// `lastStep` (line 67062)
    pub last_step: f64,
    /// `shakeT` (line 67062)
    pub shake_t: f64,
    /// `cameraDipT` (line 67063)
    pub camera_dip_t: f64,
    // ---- hud
    /// `journalOpen` (line 67102)
    pub journal_open: bool,
    /// `journalCursor` (line 67103)
    pub journal_cursor: f64,
    /// `debugOn` (line 67340)
    pub debug_on: bool,
    /// `grainT` (line 67704)
    pub grain_t: f64,
    /// `grainOpacityCur` (line 67705)
    pub grain_opacity_cur: f64,
    /// `aberrationOpacityCur` (line 67705)
    pub aberration_opacity_cur: f64,
    /// `loreArchivistShown` (line 67983)
    pub lore_archivist_shown: bool,
    /// `survivorShown` (line 67984)
    pub survivor_shown: bool,
    /// `storytellerShown` (line 67985)
    pub storyteller_shown: bool,
    /// `completionistShown` (line 68011)
    pub completionist_shown: bool,
    /// `deathCount` (line 68012)
    pub death_count: f64,
    /// `lastDeathT` (line 68014)
    pub last_death_t: f64,
    /// `ironwillShown` (line 68019)
    pub ironwill_shown: bool,
    // ---- stairwell / loop
    /// `inStairwell` (line 68256)
    pub in_stairwell: bool,
    /// `stairNearExit` (line 68258)
    pub stair_near_exit: bool,
    /// `inLoop` (line 68440)
    pub in_loop: bool,
    /// `loopNearExit` (line 68441)
    pub loop_near_exit: bool,
    // ---- flow
    /// `state` (line 68604)
    pub state: String,
    /// `fallingT` (line 68839)
    pub falling_t: f64,
    /// `godMode` (line 68868)
    pub god_mode: bool,
    /// `godDeaths` (line 68868)
    pub god_deaths: f64,
    /// `levelStartT` (line 68970)
    pub level_start_t: f64,
    /// `poolT` (line 69093)
    pub pool_t: f64,
    /// `glitchT` (line 70070)
    pub glitch_t: f64,
    /// `ghostStepT` (line 70128)
    pub ghost_step_t: f64,
    // ---- main loop
    /// `fpsFrames` (line 70147)
    pub fps_frames: f64,
    /// `fpsT` (line 70147)
    pub fps_t: f64,
    /// `fps` (line 70147)
    pub fps: f64,
    /// `fixedStepDt` (line 70150)
    pub fixed_step_dt: f64,
    /// `simFrozen` (line 70151)
    pub sim_frozen: bool,
    /// `simNoRender` (line 70215)
    pub sim_no_render: bool,
}

impl Game {
    /// A bare game: every field at its JS initial value, the scene empty.
    /// The subsystems' init functions build the rest in JS module order.
    pub fn new_bare() -> Game {
        let scene = new_scene();
        let camera = perspective_camera(74.0, 1.0, 0.05, 90.0);
        camera.set_rotation_order(EulerOrder::YXZ);
        let flashlight = crate::spot_light_js(0xfff2cf, 0.0, 28.0, std::f64::consts::PI / 5.5, 0.35, 1.2);
        let mut g = Game {
            w: 0, h: 0, tiles: vec![], has_layout: false,
            scene, camera, world: group(), animators: vec![], interactables: vec![],
            pp: PropPositions::default(), mats: SharedMats::new(),
            dust_mat: points_mat!({ size: 0.045, transparent: true, depthWrite: false, blending: AdditiveBlending, color: 0xf1e2b6, opacity: 0.32 }),
            flashlight,
            level_weather: &WEATHER_BIASES[5], player: Player::default(), perf_t: 0.0, current_seed: 0, run_seed: 0, level_num: 1,
            gameplay_r1_state: None,
            ac: None, master: None, noise_buf: None, muted: false, audio: Default::default(),
            msg_text: String::new(), msg_t: 0.0,
            placers: vec![],
            keys: HashSet::new(), mouse_dx: 0.0, mouse_dy: 0.0, pointer_locked: false, want_pointer_lock: None, want_fullscreen: None, want_quit: false, want_warm_up: false,
            inner_width: 1280.0, inner_height: 800.0, device_pixel_ratio: 2.0,
            timeouts: vec![], next_timeout_id: 1, now_ms: 0.0, location_hash: String::new(), storage: BTreeMap::new(), storage_path: None,
            dom: crate::dom::Dom::new(),
            rand_trace: RandTrace::default(),
            tile_ceil: vec![], layout: None, solids: None, tex: crate::textures::Textures::placeholder(), surfaces: crate::surfaces::make_surfaces(crate::rng::native_rnd), level_scp: None, omen_timers: vec![], panels: vec![], panel_mesh: None,
            pooled: vec![], pooled_panel: vec![], pooled_w: vec![],
            almonds: vec![], battery_items: vec![], region_map: vec![], region_info: vec![], ceil_region: vec![], floor_holes: vec![],
            barricade_tiles: vec![], pool_bounds: None, papers: vec![], lore_notes: vec![], safe_room: None, broken_elevator: None,
            light_switches: vec![], haunted_candles: vec![], rate_delta_buf: vec![], last_rare_fire_by_cause: RareFire::default(),
            rare_event_tally: ["knock", "door", "ding", "voice", "static", "shift", "pattern", "skip"].iter().map(|k| (k.to_string(), BTreeMap::new())).collect(),
            ambient_log: vec![], ambient_tally: BTreeMap::new(), footstep_fx_tally: WeatherTally::default(), crouch_fx_tally: WeatherTally::default(),
            chime_flash_color: None, paper_markers: vec![], drawer_props: vec![], stairwell: None, loop_rooms: None, scene_filter: Default::default(),
            // ---- added by wb
            pending_dispose: None, prop_cull_xz: vec![], pool_plan: None, placer_index: (0, HashMap::new()), test_stub_core_placers: false,
            // ---- added by hud
            grain_canvas: None, hud_ui: Default::default(),
            ent_ext: HashMap::new(),
            store_hide_spots: vec![], store_music_timer: None, stair_voice: None, stair_slot: None,
            // ---- end added by wb
 vacuum_bot: None,
            exit_data: None, tv_data: None, phone_data: None, clock_data: None, man_data: None, spawn_pos: Vec3::ZERO,
            arch_mats: HashMap::new(), unit_box_geo: box_geo!(1, 1, 1), unit_plane_geo: plane_geo!(1, 1),
            dispose_stats: [0; 3], prop_cull: PropCull::default(), placement_stats: BTreeMap::new(), settle_stats: BTreeMap::new(),
            placer_theme_cache: HashMap::new(), theme_stats_ran: 0, theme_stats_skipped: 0, theme_stats_by_theme: BTreeMap::new(),
            placer_dist: None, build_profile: BTreeMap::new(),
            store_state: StoreState::default(), staff_mat: None, house_state: HouseState::default(), shadow_u_time: 0.0, shadow_u_smoke: 1.0,
            shadow_mat: basic_mat!(), shadow_edge_mat: basic_mat!(),
            background: Color::hex(0x070604), fog_color: Color::hex(0x070604), fog_density: 0.052,
            ll_on: 0.0, ll_map: None, ll_grid: [1.0, 1.0, 0.5, 0.5], ll_scale: 1.0, ll_amb_d: Vec3::ZERO, ll_sky_d: Vec3::ZERO, ll_gnd_d: Vec3::ZERO,
            ll_fog_lit: Color::default(), ll_fog_ratio: 0.013 / 0.055,
            amb_light: ambient_light(0xffffff, 0), hemi_light: hemisphere_light(0xffffff, 0xffffff, 0), light_map_cpu: vec![], player_light: point_light(0xffffff, 0, 0, 2),
            halluc_textures: vec![], halluc_shape: object3d(), flashlight_target: object3d(),
            dust_geo: Geo::new(), dust_positions: vec![], dust_velocities: vec![], dust: object3d(),
            rain_streak_tex: Tex::data_rgba(vec![0; 4], 1, 1), rain_windows: vec![], scuffs: vec![], ripple_tex: Tex::data_rgba(vec![0; 4], 1, 1),
            splash_flash_light: point_light(0xffffff, 0, 0, 2), thunder_flash_light: point_light(0xffffff, 0, 0, 2), ripples: vec![], sparkles: vec![],
            wall_geo: box_geo!(CELL, WALL_H, CELL), panel_geo: box_geo!(1.9, 0.06, 1.25),
            ceil_tile_geo: { let g = plane_geo!(CELL, CELL); g.rotate_x(std::f64::consts::PI / 2.0); g },
            floor_tile_geo: { let g = plane_geo!(CELL, CELL); g.rotate_x(-std::f64::consts::PI / 2.0); g },
            wall_mats: vec![], ceil_mats: vec![], floor_mats: vec![],
            entities: vec![], player_field: None, hum_panel: None, last_zone_arch: None, zones_seen: HashSet::new(),
            paper_marker_geo: box_geo!(0.22, 0.008, 0.28), paper_marker_mat: lambert!({ color: 0xf2ead0, emissive: 0x0a0906 }),
            guide_arrows: vec![], arrow_geo: Geo::new(), arrow_mat: basic_mat!(), touch_input: TouchInput::default(),
            last_rendered_cur: None, lore_found: BTreeSet::new(), lore_arcs_completed: BTreeSet::new(), last_death_cause: None, death_tally: BTreeMap::new(),
            stair_audio: StairAudio::default(), loop_paper: vec![], loop_audio: LoopAudio::default(),
            god_causes: BTreeMap::new(), pool_edge: f64::INFINITY, pool_wanted: HashSet::new(), biased_ambient: vec![],
            glitch_next_t: 0.0, flicker_wave_next_t: 0.0,
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
            bird_chirp_timer: 0.0,
            cricket_chirp_timer: 0.0,
            thunder_rumble_timer: 0.0,
            coin_tink_timer: 0.0,
            wind_gust_timer: 0.0,
            frog_croak_timer: 0.0,
            owl_hoot_timer: 0.0,
            tick_tock_timer: 0.0,
            wolf_howl_timer: 0.0,
            rain_patter_timer: 0.0,
            leaf_rustle_timer: 0.0,
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
            cuckoo_call_timer_r741: 0.0,
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
            plant_rustle_timer_r291: 0.0,
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
    /// `tIdx(x, z)`
    #[inline] pub fn t_idx(&self, x: impl Num, z: impl Num) -> usize { (z.f() as i64 * self.w as i64 + x.f() as i64) as usize }
    /// `inGrid(x, z)`
    #[inline] pub fn in_grid(&self, x: impl Num, z: impl Num) -> bool { let (x, z) = (x.f(), z.f()); x >= 0.0 && z >= 0.0 && x < self.w as f64 && z < self.h as f64 }
    /// `tiles[tIdx(x, z)]` (out of grid reads as wall)
    #[inline] pub fn tile(&self, x: impl Num, z: impl Num) -> u8 { if self.in_grid(x, z) { self.tiles[self.t_idx(x, z)] } else { 1 } }
    /// `isWallTile(x, z)`
    #[inline] pub fn is_wall_tile(&self, x: impl Num, z: impl Num) -> bool { !self.in_grid(x, z) || self.tiles[self.t_idx(x, z)] == 1 }
    /// `wx(t)` tile → world x
    #[inline] pub fn wx(&self, t: impl Num) -> f64 { (t.f() - self.w as f64 / 2.0 + 0.5) * CELL }
    /// `wz(t)` tile → world z
    #[inline] pub fn wz(&self, t: impl Num) -> f64 { (t.f() - self.h as f64 / 2.0 + 0.5) * CELL }
    /// `tx(w)` world → tile x
    #[inline] pub fn tx(&self, w: impl Num) -> i32 { (w.f() / CELL + self.w as f64 / 2.0).floor() as i32 }
    /// `tz(w)` world → tile z
    #[inline] pub fn tz(&self, w: impl Num) -> i32 { (w.f() / CELL + self.h as f64 / 2.0).floor() as i32 }
    /// `toU(w)` / `toV(w)` (world → fractional tile)
    #[inline] pub fn to_u(&self, w: f64) -> f64 { w / CELL + self.w as f64 / 2.0 }
    #[inline] pub fn to_v(&self, w: f64) -> f64 { w / CELL + self.h as f64 / 2.0 }
    /// `levelWeather?.key`
    #[inline] pub fn weather_key(&self) -> &'static str { self.level_weather.key }

    /// `animators.push(fn)`
    pub fn animate(&mut self, f: impl FnMut(&mut Game, f64, f64) + 'static) { self.animators.push(Box::new(f)); }
    /// `interactables.push({ pos, r, prompt, enabled, use })`
    pub fn interact(&mut self, pos: Vec3, r: f64, prompt: &str, enabled: impl Fn(&Game) -> bool + 'static, on_use: impl FnMut(&mut Game) + 'static) {
        self.interactables.push(Interactable::new(pos, r, prompt, enabled, on_use));
    }

    // ------------------------------------------------------------ browser services
    /// `performance.now()` (ms, real time)
    pub fn performance_now(&self) -> f64 { self.now_ms }
    /// `setTimeout(f, ms)` — runs `f` on the main thread once `ms` real milliseconds have passed
    pub fn set_timeout(&mut self, ms: f64, f: impl FnOnce(&mut Game) + 'static) -> u64 {
        let id = self.next_timeout_id; self.next_timeout_id += 1;
        self.timeouts.push(Timeout { id, at_ms: self.now_ms + ms.max(0.0), f: Some(Box::new(f)) });
        id
    }
    /// `clearTimeout(id)`
    pub fn clear_timeout(&mut self, id: u64) { self.timeouts.retain(|t| t.id != id); }
    /// Run every timeout that is due (the app calls this every frame).
    pub fn run_timeouts(&mut self) {
        loop {
            let now = self.now_ms;
            let Some(i) = self.timeouts.iter().enumerate().filter(|(_, t)| t.at_ms <= now).min_by(|a, b| a.1.at_ms.partial_cmp(&b.1.at_ms).unwrap()).map(|(i, _)| i) else { break };
            let mut t = self.timeouts.remove(i);
            if let Some(f) = t.f.take() { f(self); }
        }
    }
    /// `localStorage.getItem(k)`
    pub fn storage_get(&self, k: &str) -> Option<String> { self.storage.get(k).cloned() }
    /// `localStorage.setItem(k, v)` (persisted to the save file)
    pub fn storage_set(&mut self, k: &str, v: &str) { self.storage.insert(k.to_string(), v.to_string()); self.storage_flush(); }
    /// `localStorage.removeItem(k)`
    pub fn storage_remove(&mut self, k: &str) { self.storage.remove(k); self.storage_flush(); }
    fn storage_flush(&self) {
        if let Some(p) = &self.storage_path {
            let mut s = String::new();
            for (k, v) in &self.storage { s.push_str(&format!("{}\t{}\n", escape(k), escape(v))); }
            let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
            let _ = std::fs::write(p, s);
        }
    }
    /// Load `localStorage` from the save file.
    pub fn storage_load(&mut self, path: std::path::PathBuf) {
        if let Ok(s) = std::fs::read_to_string(&path) {
            for line in s.lines() { if let Some((k, v)) = line.split_once('\t') { self.storage.insert(unescape(k), unescape(v)); } }
        }
        self.storage_path = Some(path);
    }
    /// `location.hash.includes(s)`
    pub fn hash_has(&self, s: &str) -> bool { self.location_hash.contains(s) }
    /// `keys[code]`
    pub fn key(&self, code: &str) -> bool { self.keys.contains(code) }

    // ------------------------------------------------------------ audio helpers
    /// `AC.currentTime` (0 without audio)
    pub fn ac_time(&self) -> f64 { self.ac.as_ref().map(|a| a.current_time()).unwrap_or(0.0) }
    /// `blip(freq, dur, type='sine', gain=0.2, slideTo=null, delay=0, pan=0)`
    pub fn blip(&self, freq: f64, dur: f64, ty: &str, gain: f64, slide_to: Option<f64>, delay: f64, pan: f64) {
        let (Some(ac), Some(master)) = (&self.ac, &self.master) else { return };
        let t0 = ac.current_time() + delay;
        let o = ac.create_oscillator(); o.set_type(ty); o.frequency().set_value_at_time(freq, t0);
        if let Some(s) = slide_to { if s != 0.0 { o.frequency().exponential_ramp_to_value_at_time(s.max(20.0), t0 + dur); } }
        let g = ac.create_gain();
        g.gain().set_value_at_time(gain, t0);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
        o.connect(&g);
        if pan != 0.0 {
            let p = ac.create_stereo_panner();
            p.pan().set_value(pan.clamp(-1.0, 1.0));
            g.connect(&p); p.connect(master);
        } else { g.connect(master); }
        o.start(t0); o.stop(t0 + dur + 0.05);
    }
    /// `noiseBurst(dur, freq, q, gain, type='bandpass', slideTo=null, delay=0, pan=0)`
    pub fn noise_burst(&self, dur: f64, freq: f64, q: f64, gain: f64, ty: &str, slide_to: Option<f64>, delay: f64, pan: f64) {
        let (Some(ac), Some(master), Some(nb)) = (&self.ac, &self.master, &self.noise_buf) else { return };
        let t0 = ac.current_time() + delay;
        let s = ac.create_buffer_source(); s.set_buffer(nb); s.set_loop(true);
        let f = ac.create_biquad_filter(); f.set_type(ty); f.frequency().set_value_at_time(freq, t0); f.q().set_value(q);
        if let Some(sl) = slide_to { if sl != 0.0 { f.frequency().exponential_ramp_to_value_at_time(sl.max(30.0), t0 + dur); } }
        let g = ac.create_gain();
        g.gain().set_value_at_time(gain, t0);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
        s.connect(&f); f.connect(&g);
        if pan != 0.0 {
            let p = ac.create_stereo_panner();
            p.pan().set_value(pan.clamp(-1.0, 1.0));
            g.connect(&p); p.connect(master);
        } else { g.connect(master); }
        s.start(t0); s.stop(t0 + dur + 0.05);
    }
}

fn escape(s: &str) -> String { s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t") }
fn unescape(s: &str) -> String {
    let mut o = String::new(); let mut it = s.chars();
    while let Some(c) = it.next() { if c == '\\' { match it.next() { Some('n') => o.push('\n'), Some('t') => o.push('\t'), Some(x) => o.push(x), None => {} } } else { o.push(c); } }
    o
}
