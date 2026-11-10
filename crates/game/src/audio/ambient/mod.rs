//! The soundscape driver: `floor_surface_at`, zone entries,
//! `update_soundscape`, `update_ambient`, `update_sanity_effects`; plus the
//! level's signature sounds, the reality glitch, the flicker wave and the
//! ghost footsteps.
//! CONTRACT: the signatures below are called from other modules; keep them.
//!
//! Sounds come from places. A prop sound plays from the nearest prop of its
//! kind within the sound's audible radius (pan from the true direction,
//! inverse-distance level, muffled without line of sight) and is silent
//! otherwise; each level draws a few signature sounds anchored to fixed spots
//! of its own (`pick_signatures`); and when a level starts every cadence
//! begins part-way through its interval (`arm_level_ambient`), so nothing
//! fires together on arrival. `amb_hush` (the pacing director) quiets the
//! ambient one-shots and, above `HUSH_HOLD`, holds their timers.
//!
//! `update_ambient` is split into one private function per event family, in
//! a fixed order (so the per-frame `rnd()` draws stay in order):
//!
//! | step | function | file |
//! |---|---|---|
//! | mix factors (sanity, cadence, gain, level depth, hush) | `amb_mix` | here |
//! | rain-window streaks + rain hiss | `amb_rain_windows` | here |
//! | ripples, scuffs, splash / thunder flash decay | `amb_fx_decay` | here |
//! | room-tone lerp, low-sanity brightening, hush dip | `amb_room_tone` | here |
//! | per-weather beds (wet / still / derelict / electric / haunted / normal) | `amb_weather_events` | weather.rs |
//! | cross-weather rare events (knock … clock skip) | `amb_rare_events` | weather.rs |
//! | the fixtures' prop sounds (fountain … smoke detector) | `amb_props_fixtures` | fixtures.rs |
//! | trinkets (coin … teapot) | `amb_props_trinkets` | fixtures.rs |
//! | household props (cookie tin … turntable) | `amb_props_household` | household.rs |
//! | parlour props (chalkboard … hobby horse) | `amb_props_parlour` | household.rs |
//! | lobby props (pay phone … plant, motion sensor) | `amb_props_lobby` | lobby.rs |
//! | rare fixtures (speaker … broken phone) | `amb_props_rare_fixtures` | lobby.rs |
//! | the level's signature sounds | `amb_signatures` | here |
//! | reality-shift pitch bend, surge decay | `amb_shift_bend` | weather.rs |
pub mod fixtures;
pub mod household;
pub mod lobby;
pub mod weather;

use crate::audio::soundscape::VOICE_MAX_DIST;
use crate::base::records::*;
use crate::base::tables::*;
use crate::prelude::*;
use crate::visuals::lighting::{RF_DARK, RF_MOSS};

pub const FOG_BASE: f64 = 0.052;
/// A metronome's swing period when the placer did not record one: the
/// middle of the placer's `1.2 + rnd() * 0.6` range.
pub const METRONOME_PERIOD_DEFAULT: f64 = 1.5;
/// A rocking chair's rock period when the placer did not record one: the
/// middle of the placer's `5 + rnd() * 3` range.
pub const CHAIR_PERIOD_DEFAULT: f64 = 6.5;
/// Messages from the walls during a reality glitch (`update_reality_glitch`)
pub const GLITCH_MSGS: [&str; 8] =
    ["HELLO", "YOU ARE HERE", "YOU ARE STILL HERE", "AGAIN", "I'M SORRY", "GO BACK", "IT KNOWS YOUR NAME", "ALMOST"];
/// the on-screen whispers that can accompany a hallucination (`update_sanity_effects`)
pub const HALLUC_WHISPERS: [&str; 6] =
    ["IT SAW YOU", "DO NOT TURN", "YOU ARE NOT ALONE", "HELLO AGAIN", "STAY VERY STILL", "I REMEMBER YOU"];
/// Wet-ripple lifetime, s
const RIPPLE_LIFE: f64 = 1.5;
/// Dust-scuff lifetime, s
const SCUFF_LIFE: f64 = 0.7;
/// The share of an ambient one-shot's level that a full `amb_hush` takes away
const HUSH_DUCK: f64 = 0.85;
/// Above this `amb_hush` no new ambient one-shot starts: the timers hold
pub const HUSH_HOLD: f64 = 0.6;
/// The height signature sounds come from
const SIG_Y: f64 = 1.4;
/// How far apart two signature anchors stay where the level allows, m
pub const SIG_APART: f64 = 8.0;
/// Pool sounds that are not sounds of a place (a breath belongs to whatever
/// breathes), never a level's signature
const SIG_NOT_PLACES: [&str; 1] = ["breath"];
/// How far a wall muffles a signature (0 clear … 1 sealed): the open-plan
/// rooms leak sound round their partitions
const SIG_OCCLUDE: f64 = 0.45;

/// How a signature sound carries: how far from the spawn its anchor lies
/// (m), the reference distance of its voice slot (m) and its level.
#[derive(Clone, Copy, Debug)]
pub struct SigCarry {
    pub near: f64,
    pub far: f64,
    pub ref_dist: f64,
    pub gain: f64,
}
/// Loud things are heard from far off (a slam, a scream, steps in an empty
/// hall); quiet ones only from the next room (a drip, a breath, paper).
pub fn sig_carry(name: &str) -> SigCarry {
    // the levels: each sound lands some 6–14 dB over the room's bed at its
    // distance (measured by `tests/listen.rs`; over eight floors the thud,
    // the clank, the moan and the laugh came in near the bed, 6 dB under
    // the slam and the steps they were levelled with, the radio 3 dB, the
    // paper at its far reach 3 dB more: raised by that in the Megastructure
    // work; then the moan 6 dB more again and the dial tone 3 dB, which
    // still came in 2–3 dB short at 24–44 m. What stays under is a thud or a
    // clank behind three walls, as it should be)
    match name {
        "doorSlam" => SigCarry { near: 22.0, far: 55.0, ref_dist: 14.0, gain: 4.5 },
        "thud" | "clank" => SigCarry { near: 22.0, far: 55.0, ref_dist: 14.0, gain: 9.0 },
        "moan" => SigCarry { near: 22.0, far: 55.0, ref_dist: 14.0, gain: 17.5 },
        "distantScream" => SigCarry { near: 22.0, far: 55.0, ref_dist: 14.0, gain: 7.5 },
        "piano" | "distantFootsteps" => SigCarry { near: 20.0, far: 42.0, ref_dist: 14.0, gain: 11.2 },
        "laugh" => SigCarry { near: 12.0, far: 28.0, ref_dist: 8.0, gain: 21.0 },
        "chime" | "dialTone" => SigCarry { near: 12.0, far: 28.0, ref_dist: 8.0, gain: 15.0 },
        "radioStatic" => SigCarry { near: 12.0, far: 28.0, ref_dist: 8.0, gain: 25.0 },
        "paperRustle" | "pageFlip" => SigCarry { near: 7.0, far: 14.0, ref_dist: 4.0, gain: 34.0 },
        _ => SigCarry { near: 7.0, far: 14.0, ref_dist: 4.0, gain: 21.0 },
    }
}
/// Every signature's period stretches by this when the level draws the
/// silent entry of the ambient pool
const SIG_SILENT_GAP: f64 = 1.4;
/// Salt of the per-level signature stream
const SIG_SALT: u64 = 0x5167_A70C_E5B1_D00D;
/// Salt of the per-level stream that places the weather's sources
const BED_SALT: u64 = 0xB3D5_0A2C_7E11_9F43;
/// The places the weather's own sounds come from, and how far from the
/// spawn each lies: a draught (wind, the echoing whisper), a duct (the
/// clatter), a dying fan (the grinder), a room someone is in (whispers,
/// steps). Each level puts them somewhere of their own.
pub const BED_ANCHORS: [(&str, f64, f64); 4] = [("draught", 28.0, 50.0), ("duct", 14.0, 32.0), ("fan", 30.0, 55.0), ("room", 10.0, 22.0)];
/// How the weather's sounds carry from their places: the voice slot's
/// reference distance (m) and the level that makes up for the distance
pub const BED_REF_DIST: f64 = 14.0;
pub const BED_GAIN: f64 = 2.5;
/// How far the exit door is heard, m (straight line; walls muffle it)
pub const EXIT_HEARD: f64 = 40.0;
/// The door calls every `EXIT_CALL_PERIOD` ± 1 s
pub const EXIT_CALL_PERIOD: f64 = 9.0;
/// The exit call's voice slot: reference distance (m) and level
const EXIT_REF_DIST: f64 = 18.0;
/// (+1.5 dB in the Megastructure work: on the renumbered floors one test
/// spot five tiles round a corner fell to +5.8 dB, under the +6 the door
/// should carry there, `tests/listen.rs`)
const EXIT_GAIN: f64 = 35.6;
/// How much a wall muffles the door (0 clear … 1 sealed)
const EXIT_OCCLUDE: f64 = 0.45;
/// Salt of the exit call's jitter
const EXIT_SALT: u64 = 0xE417_CA11_0D00_4B1D;
/// Ghost footfalls: reference distance of their voice slots, m
const GHOST_REF_DIST: f64 = 3.0;

/// The per-frame mix factors at the top of `update_ambient`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AmbMix {
    /// Sanity magnitude — 0 at ≥ 30 % sanity, 1 at 0 %
    pub sanity_mag: f64,
    /// Event-timer multiplier — cadence shrink (sanity × descent)
    pub timer_mul: f64,
    /// Event gain — sanity × descent × stamina muffle × cry muffle × hush
    pub gain: f64,
    /// Level depth — `level_depth_mul()`
    pub depth: f64,
    /// `amb_hush`, clamped to 0..1
    pub hush: f64,
    /// Hushed past `HUSH_HOLD`: no new one-shot starts, the timers hold
    pub hushed: bool,
}

/// An ambient event's cadence timer, the tag it logs under and its usual
/// interval, `base + rnd() * span` s (before the sanity / descent shrink).
/// A `run` cadence counts down across levels: the rare events belong to
/// the run, so a new level does not start them over.
#[derive(Clone, Copy, Debug)]
pub struct Cadence {
    pub tag: &'static str,
    pub timer: fn(&mut Game) -> &mut f64,
    pub base: f64,
    pub span: f64,
    pub run: bool,
}
const fn cad(tag: &'static str, timer: fn(&mut Game) -> &mut f64, base: f64, span: f64) -> Cadence {
    Cadence { tag, timer, base, span, run: false }
}
const fn rare(tag: &'static str, timer: fn(&mut Game) -> &mut f64, base: f64, span: f64) -> Cadence {
    Cadence { tag, timer, base, span, run: true }
}

/// Every ambient event cadence, in `update_ambient` order. `reset_level_timers`
/// zeroes them during the build and `arm_level_ambient` staggers them once
/// the level is up.
pub const CADENCES: &[Cadence] = &[
    // weather beds
    cad("wet-drip", |g| &mut g.wet_drip_timer, 8.0, 10.0),
    cad("wet-thunder", |g| &mut g.wet_thunder_timer, 35.0, 55.0),
    cad("wet-splash", |g| &mut g.wet_splash_timer, 55.0, 85.0),
    cad("thunder-rumble", |g| &mut g.thunder_rumble_timer, 200.0, 300.0),
    cad("rain-patter", |g| &mut g.rain_patter_timer, 220.0, 180.0),
    cad("still-wind", |g| &mut g.still_wind_timer, 10.0, 18.0),
    cad("still-echo", |g| &mut g.still_echo_timer, 90.0, 90.0),
    cad("wind-gust", |g| &mut g.wind_gust_timer, 150.0, 180.0),
    cad("derelict-clatter", |g| &mut g.derelict_clatter_timer, 12.0, 20.0),
    cad("derelict-grinder", |g| &mut g.derelict_grinder_timer, 60.0, 90.0),
    cad("electric-crackle", |g| &mut g.electric_crackle_timer, 8.0, 14.0),
    cad("socket-spark", |g| &mut g.socket_spark_timer, 12.0, 24.0),
    cad("junction-arc", |g| &mut g.junction_arc_timer, 20.0, 25.0),
    cad("electric-surge", |g| &mut g.electric_surge_timer, 60.0, 60.0),
    cad("haunted-whisper", |g| &mut g.haunted_whisper_timer, 14.0, 22.0),
    cad("haunted-step", |g| &mut g.haunted_step_timer, 80.0, 80.0),
    cad("normal-chime", |g| &mut g.normal_notice_timer, 120.0, 120.0),
    // rare events: a few times a run, counted across levels
    rare("distant-knock", |g| &mut g.distant_knock_timer, 240.0, 240.0),
    rare("door-opened", |g| &mut g.door_opened_timer, 480.0, 420.0),
    rare("elevator-ding", |g| &mut g.elevator_ding_timer, 720.0, 780.0),
    rare("voice-sample", |g| &mut g.voice_sample_timer, 900.0, 900.0),
    rare("static-burst", |g| &mut g.static_burst_timer, 420.0, 420.0),
    rare("reality-shift", |g| &mut g.reality_shift_timer, 720.0, 720.0),
    rare("pattern-break", |g| &mut g.pattern_break_timer, 1200.0, 900.0),
    rare("clock-skip", |g| &mut g.clock_skip_timer, 1500.0, 1200.0),
    // fixtures
    cad("fountain-drip", |g| &mut g.fountain_drip_timer, 25.0, 30.0),
    cad("radiator-hiss", |g| &mut g.radiator_hiss_timer, 40.0, 40.0),
    cad("glow-hum", |g| &mut g.glow_hum_timer, 20.0, 25.0),
    cad("cauldron-bubble", |g| &mut g.cauldron_bubble_timer, 12.0, 13.0),
    cad("metronome-tick", |g| &mut g.metronome_tick_timer, METRONOME_PERIOD_DEFAULT * 0.5, 0.0),
    cad("phono-crackle", |g| &mut g.phono_crackle_timer, 8.0, 8.0),
    cad("lantern-creak", |g| &mut g.lantern_creak_timer, 18.0, 22.0),
    cad("sewing-whirr", |g| &mut g.sewing_whirr_timer, 15.0, 15.0),
    cad("watch-tick", |g| &mut g.watch_tick_timer, 3.0, 3.0),
    cad("bat-squeak", |g| &mut g.bat_squeak_timer, 25.0, 30.0),
    cad("globe-creak", |g| &mut g.globe_creak_timer, 22.0, 23.0),
    cad("hourglass-whoosh", |g| &mut g.hourglass_whoosh_timer, 4.0, 4.0),
    cad("ghost-whisper", |g| &mut g.ghost_whisper_timer, 30.0, 40.0),
    cad("chain-jangle", |g| &mut g.chain_jangle_timer, 8.0, 10.0),
    cad("paper-rustle", |g| &mut g.paper_rustle_timer, 20.0, 25.0),
    cad("piano-note", |g| &mut g.piano_chord_timer, 40.0, 50.0),
    cad("chair-creak", |g| &mut g.chair_creak_timer, CHAIR_PERIOD_DEFAULT, 0.0),
    cad("crt-hiss", |g| &mut g.crt_hiss_timer, 4.0, 6.0),
    cad("exit-buzz", |g| &mut g.exit_buzz_timer, 6.0, 6.0),
    cad("puddle-drip", |g| &mut g.puddle_drip_timer, 8.0, 8.0),
    cad("chandelier-chime", |g| &mut g.chandelier_chime_timer, 20.0, 20.0),
    cad("cassette-hiss", |g| &mut g.cassette_hiss_timer, 10.0, 10.0),
    cad("teacup-clink", |g| &mut g.teacup_clink_timer, 25.0, 25.0),
    cad("locker-slam", |g| &mut g.locker_slam_timer, 60.0, 60.0),
    cad("bulletin-flutter", |g| &mut g.bulletin_flutter_timer, 15.0, 15.0),
    cad("alarm-beep", |g| &mut g.alarm_beep_timer, 35.0, 35.0),
    cad("fan-whoosh", |g| &mut g.fan_whoosh_timer, 8.0, 6.0),
    cad("fan-squeak", |g| &mut g.fan_squeak_timer, 25.0, 30.0),
    cad("detector-chirp", |g| &mut g.detector_chirp_timer, 90.0, 110.0),
    // trinkets
    cad("coin-tink", |g| &mut g.coin_tink_timer, 20.0, 25.0),
    cad("tick-tock", |g| &mut g.tick_tock_timer, 12.0, 13.0),
    cad("cuckoo-call", |g| &mut g.cuckoo_call_timer, 90.0, 130.0),
    cad("plush-shuffle", |g| &mut g.plush_shuffle_timer, 150.0, 170.0),
    cad("gumball-rattle", |g| &mut g.gumball_rattle_timer, 40.0, 50.0),
    cad("teapot-whistle", |g| &mut g.teapot_whistle_timer, 200.0, 250.0),
    // household
    cad("cookie-tin-rattle", |g| &mut g.cookie_tin_rattle_timer, 45.0, 45.0),
    cad("kitchen-clock-tick", |g| &mut g.kitchen_clock_tick_timer, 60.0, 60.0),
    cad("dial-tone-hum", |g| &mut g.off_hook_phone_hum_timer, 15.0, 15.0),
    cad("hairdryer-hum", |g| &mut g.hairdryer_hum_timer, 10.0, 10.0),
    cad("sewing-machine-click", |g| &mut g.sewing_machine_click_timer, 8.0, 10.0),
    cad("herb-sieve-scrape", |g| &mut g.herb_sieve_scrape_timer, 40.0, 40.0),
    cad("vacuum-whine", |g| &mut g.vacuum_whine_timer, 30.0, 30.0),
    cad("mop-bucket-slosh", |g| &mut g.mop_bucket_slosh_timer, 50.0, 50.0),
    cad("wristwatch-tick", |g| &mut g.wristwatch_tick_timer, 3.0, 4.0),
    cad("iron-hiss", |g| &mut g.iron_hiss_timer, 20.0, 20.0),
    cad("grandfather-clock-tick", |g| &mut g.grandfather_clock_tick_timer, 2.0, 1.0),
    cad("rocker-creak", |g| &mut g.rocking_chair_creak_timer, 4.0, 4.0),
    cad("lamp-buzz", |g| &mut g.tall_lamp_buzz_timer, 12.0, 13.0),
    cad("colander-tap", |g| &mut g.colander_tap_timer, 30.0, 30.0),
    cad("pet-bowl-slosh", |g| &mut g.pet_bowl_slosh_timer, 40.0, 40.0),
    cad("music-box-melody", |g| &mut g.music_box_melody_timer, 20.0, 20.0),
    cad("grandfather-chime", |g| &mut g.grandfather_chime_timer, 180.0, 180.0),
    cad("turntable-hiss", |g| &mut g.turntable_hiss_timer, 6.0, 8.0),
    // parlour
    cad("chalkboard-scratch", |g| &mut g.chalkboard_scratch_timer, 45.0, 45.0),
    cad("calendar-turn", |g| &mut g.kitchen_calendar_turn_timer, 90.0, 90.0),
    cad("telescope-click", |g| &mut g.telescope_click_timer, 20.0, 20.0),
    cad("cuckoo-clock-call", |g| &mut g.ornate_cuckoo_call_timer, 120.0, 180.0),
    cad("fireplace-crackle", |g| &mut g.fireplace_crackle_timer, 4.0, 6.0),
    cad("aquarium-hum", |g| &mut g.aquarium_hum_timer, 8.0, 7.0),
    cad("wingback-creak", |g| &mut g.wingback_creak_timer, 60.0, 60.0),
    cad("pocket-watch-tick", |g| &mut g.pocket_watch_tick_timer, 3.0, 3.0),
    cad("bell-pull-ring", |g| &mut g.bell_pull_ring_timer, 120.0, 120.0),
    cad("bell-rack-ding", |g| &mut g.bell_rack_ding_timer, 90.0, 90.0),
    cad("chess-move", |g| &mut g.chess_move_click_timer, 45.0, 45.0),
    cad("horse-roll", |g| &mut g.hobby_horse_roll_timer, 90.0, 90.0),
    // lobby
    cad("payphone-ring", |g| &mut g.pay_phone_ring_timer, 50.0, 50.0),
    cad("radio-static", |g| &mut g.radio_static_timer, 10.0, 12.0),
    cad("keychain-jangle", |g| &mut g.keychain_jangle_timer, 20.0, 25.0),
    cad("elevator-beep", |g| &mut g.elevator_beep_timer, 30.0, 30.0),
    cad("plant-rustle", |g| &mut g.plant_rustle_timer, 20.0, 20.0),
    // rare fixtures
    cad("speaker-announce", |g| &mut g.speaker_announce_timer, 180.0, 180.0),
    cad("fire-alarm-beep", |g| &mut g.fire_alarm_beep_timer, 300.0, 300.0),
    cad("phone-ring", |g| &mut g.phone_ring_timer, 600.0, 600.0),
    cad("vending-hum", |g| &mut g.vending_hum_timer, 720.0, 780.0),
    cad("keycard-beep", |g| &mut g.keycard_beep_timer, 120.0, 120.0),
    cad("cardboard-shift", |g| &mut g.cardboard_shift_timer, 180.0, 180.0),
    cad("living-plant-rustle", |g| &mut g.living_plant_rustle_timer, 240.0, 240.0),
    cad("walkie-static", |g| &mut g.walkie_static_timer, 180.0, 180.0),
    cad("wallclock-tick", |g| &mut g.wall_clock_tick_timer, 240.0, 240.0),
    cad("cracked-clock-creak", |g| &mut g.cracked_clock_creak_timer, 180.0, 180.0),
    cad("ceiling-drip", |g| &mut g.ceiling_drip_audio_timer, 30.0, 60.0),
    cad("ceiling-cable-spark", |g| &mut g.ceiling_cable_spark_timer, 90.0, 150.0),
    cad("thermostat-click", |g| &mut g.thermostat_click_timer, 120.0, 120.0),
    cad("handbell-chime", |g| &mut g.handbell_chime_timer, 180.0, 180.0),
    cad("digital-clock-buzz", |g| &mut g.digital_clock_buzz_timer, 240.0, 240.0),
    cad("alarm-ring", |g| &mut g.alarm_ring_timer, 360.0, 360.0),
    cad("rotary-dial-tone", |g| &mut g.dial_tone_timer, 120.0, 120.0),
    cad("drain-gurgle", |g| &mut g.drain_gurgle_timer, 120.0, 180.0),
    cad("faucet-drip", |g| &mut g.faucet_drip_timer, 120.0, 120.0),
    cad("broken-phone-spark", |g| &mut g.broken_phone_spark_timer, 120.0, 180.0),
];

/// Counts an ambient event's timer down by `dt`; true when the event is due.
/// While the ambience is hushed the timer holds: nothing new starts, and
/// nothing piles up to fire together when the hush lifts.
#[inline]
pub(crate) fn amb_due(t: &mut f64, dt: f64, m: &AmbMix) -> bool {
    if m.hushed {
        return false;
    }
    *t -= dt;
    *t <= 0.0
}

/// How soon a prop sound with nothing in earshot looks again, s
#[inline]
fn prop_retry() -> f64 {
    1.0 + rnd() * 2.0
}

/// Level of a prop sound `d` m away whose audible radius is `r` m: full
/// within a reference distance (a third of the radius, at least 1 m),
/// inverse distance past it, tapered smoothly to 0 at the radius.
pub fn prop_falloff(d: f64, r: f64) -> f64 {
    if !(d < r) {
        return 0.0;
    }
    let r0 = (r / 3.0).max(1.0);
    let x = d / r;
    let taper = (1.0 - x * x) * (1.0 - x * x);
    (r0 / d.max(r0)) * taper
}

/// The prop a sound plays from (`Game::prop_near`)
#[derive(Clone, Copy, Debug)]
pub struct PropHit {
    /// Index into the prop list, and the prop's position
    pub i: usize,
    pub p: Vec3,
    /// Stereo pan from the true direction
    pub pan: f64,
    /// Level: the distance falloff, halved without line of sight
    pub g: f64,
    /// Distance as a share of the audible radius (0..1)
    pub dn: f64,
}

impl Game {
    // ------------------------------------------------------------ helpers
    /// The player's yaw, 0 when it is NaN
    #[inline]
    pub(crate) fn yaw0(&self) -> f64 {
        let y = self.player.yaw;
        if y.is_nan() {
            0.0
        } else {
            y
        }
    }

    /// Distance, stereo pan and line of sight from the player to world point
    /// (x, z). The pan is the listener's right · the unit direction, ×0.8.
    /// The sight line stops just short of the point, so a source on a wall is
    /// not hidden by its own wall.
    pub fn ear(&self, x: f64, z: f64) -> (f64, f64, bool) {
        let (d, pan) = self.ear_pan(x, z);
        if !(d > 1e-6) {
            return (0.0, 0.0, true);
        }
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let (dx, dz) = (x - px, z - pz);
        let back = (d * 0.5).min(0.6);
        let clear = self.los(px, pz, x - dx / d * back, z - dz / d * back);
        (d, pan, clear)
    }

    /// `ear` without the sight line: the distance and the stereo pan
    pub fn ear_pan(&self, x: f64, z: f64) -> (f64, f64) {
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let (dx, dz) = (x - px, z - pz);
        let d = (dx * dx + dz * dz).sqrt();
        if !(d > 1e-6) {
            return (0.0, 0.0);
        }
        let yaw = self.yaw0();
        let side = dx * yaw.cos() - dz * yaw.sin();
        (d, clamp(side / d.max(0.5), -1.0, 1.0) * 0.8)
    }

    /// The nearest prop of `list` within `radius` m of the player; None when
    /// none is in earshot.
    pub fn prop_near(&self, list: &[Vec3], radius: f64) -> Option<PropHit> {
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let mut best: Option<(usize, f64)> = None;
        for (i, p) in list.iter().enumerate() {
            let d2 = (p.x - px).powi(2) + (p.z - pz).powi(2);
            if d2 < radius * radius && best.is_none_or(|(_, b)| d2 < b) {
                best = Some((i, d2));
            }
        }
        let (i, _) = best?;
        let p = list[i];
        let (d, pan, clear) = self.ear(p.x, p.z);
        let g = prop_falloff(d, radius) * if clear { 1.0 } else { 0.5 };
        Some(PropHit { i, p, pan, g, dn: (d / radius).min(1.0) })
    }

    /// A prop sound's turn: counts `timer` down by `dt` and, when it is due,
    /// returns the nearest prop of `list` within `radius` m. With none in
    /// earshot the sound stays silent and looks again in 1–3 s instead of a
    /// whole interval. The caller plays the sound and re-arms `timer`.
    pub(crate) fn prop_due(
        &mut self,
        timer: fn(&mut Game) -> &mut f64,
        list: fn(&Game) -> &[Vec3],
        radius: f64,
        dt: f64,
        m: &AmbMix,
    ) -> Option<PropHit> {
        if list(self).is_empty() || !amb_due(timer(self), dt, m) {
            return None;
        }
        let hit = self.prop_near(list(self), radius);
        if hit.is_none() {
            *timer(self) = prop_retry();
        }
        hit
    }

    /// Plays `f`'s one-shots (`blip` / `noise_burst`, so any sfx) from world
    /// point (x, y, z): through a fresh voice slot of the soundscape (3D pan,
    /// distance, the zone's reverb, muffled without line of sight), or,
    /// without the soundscape, panned and attenuated on the stereo mix.
    /// `gain` scales every voice. Returns the stereo pan, for the log.
    pub fn play_at(&mut self, x: f64, y: f64, z: f64, ref_dist: f64, gain: f64, f: impl FnOnce(&mut Game)) -> f64 {
        self.play_from(x, y, z, ref_dist, gain, 0.8, f)
    }
    /// `play_at` with how much a wall in the way muffles it (`occlude`,
    /// 0 … 1).
    pub fn play_from(&mut self, x: f64, y: f64, z: f64, ref_dist: f64, gain: f64, occlude: f64, f: impl FnOnce(&mut Game)) -> f64 {
        let (d, pan, clear) = self.ear(x, z);
        let route = match self.soundscape() {
            Some(ss) => {
                let slot = ss.voice_slot(ref_dist);
                slot.place(x, y, z, if clear { 0.0 } else { occlude });
                // the panner stops attenuating at its max distance; keep falling off past it
                let beyond = VOICE_MAX_DIST / d.max(VOICE_MAX_DIST);
                SfxRoute { dest: Some(slot.input.clone()), pan: 0.0, gain: gain * beyond }
            }
            None => {
                let att = ref_dist / (ref_dist + 1.2 * (d.max(ref_dist) - ref_dist));
                SfxRoute { dest: None, pan, gain: gain * att * if clear { 1.0 } else { 1.0 - occlude * 0.56 } }
            }
        };
        let prev = self.sfx_route.replace(route);
        f(self);
        self.sfx_route = prev;
        pan
    }

    /// The first idle scuff sprite, dressed as a short spark / splash / bubble flash
    pub(crate) fn scuff_flash(&mut self, hex: u32, x: f64, y: f64, z: f64, opacity: f64, scale: f64, life: f64) {
        let Some(i) = self.scuffs.iter().position(|s| s.t <= 0.0) else { return };
        let o = self.scuffs[i].obj.clone();
        if let Some(m) = o.try_mat() {
            m.m().color.set_hex(hex);
        }
        o.set_pos(x, y, z);
        o.set_visible(true);
        if let Some(m) = o.try_mat() {
            m.set_opacity(opacity);
        }
        o.scale().set_scalar(scale);
        self.scuffs[i].t = life;
    }
    /// The first idle ripple placed at (x, 0.015, z); false when the pool is busy
    pub(crate) fn ripple_at(&mut self, x: f64, z: f64, opacity: f64, scale: f64, life: f64) -> bool {
        let Some(i) = self.ripples.iter().position(|r| r.t <= 0.0) else { return false };
        let o = self.ripples[i].obj.clone();
        o.set_pos(x, 0.015, z);
        o.set_visible(true);
        if let Some(m) = o.try_mat() {
            m.set_opacity(opacity);
        }
        o.scale().set(scale, scale, 1);
        self.ripples[i].t = life;
        true
    }

    // ------------------------------------------------------------ startup
    /// Startup state of the soundscape driver and of the glitch /
    /// flicker-wave / ghost-step timers (each level re-arms them, see
    /// `arm_level_ambient`).
    pub fn init_ambient(&mut self) {
        self.hummer_t = 0.0;
        self.hum_panel = None;
        self.last_zone_arch = None;
        self.zone_toast_t = 0.0;
        self.zones_seen.clear();
        self.amb_signatures.clear();
        self.glitch_t = 0.0;
        self.glitch_next_t = 60.0 + rnd() * 90.0;
        self.flicker_wave_next_t = 45.0 + rnd() * 60.0;
        self.ghost_step_t = rand2(45.0, 110.0);
    }

    /// A level's ambient start, once it is built and the player stands at
    /// the spawn (outside the seeded stream): every cadence's first countdown
    /// is 35–100 % of one of its intervals, so nothing fires together on
    /// arrival; the glitch, flicker-wave and ghost-step countdowns restart
    /// part-way too; and the level draws its signature sounds.
    pub fn arm_level_ambient(&mut self) {
        for c in CADENCES {
            // the run's rare events keep counting from the last level
            if c.run && *(c.timer)(self) > 0.0 {
                continue;
            }
            let first = (c.base + rnd() * c.span) * rand2(0.35, 1.0);
            *(c.timer)(self) = first;
        }
        self.glitch_next_t = rand2(60.0, 130.0) * rand2(0.5, 1.0);
        self.flicker_wave_next_t = rand2(90.0, 180.0) * rand2(0.4, 1.0);
        self.ghost_step_t = rand2(45.0, 110.0) * rand2(0.5, 1.0);
        self.pick_signatures();
        self.pick_bed_anchors();
        self.exit_call_t = 3.0;
        self.exit_calls = 0;
    }

    /// Where this level's weather sounds come from (`BED_ANCHORS`), from a
    /// local stream seeded by `current_seed`: the same places for the same
    /// (run seed, level), other places elsewhere.
    pub fn pick_bed_anchors(&mut self) {
        let mut r = SplitMix64::new(self.current_seed, BED_SALT);
        let carry: Vec<SigCarry> =
            BED_ANCHORS.iter().map(|&(_, near, far)| SigCarry { near, far, ref_dist: BED_REF_DIST, gain: BED_GAIN }).collect();
        let spots = self.signature_spots(&carry, &mut r);
        self.amb_anchors = BED_ANCHORS.iter().map(|a| a.0).zip(spots).collect();
    }
    /// A level's source for one of the weather's sounds (`BED_ANCHORS`)
    pub fn bed_anchor(&self, key: &str) -> Option<Vec3> {
        self.amb_anchors.iter().find(|a| a.0 == key).map(|a| a.1)
    }
    /// The light fitting nearest you: the wiring that crackles, the tube
    /// that surges
    pub fn nearest_fixture(&self) -> Option<Vec3> {
        let p = self.player.pos;
        self.panels
            .iter()
            .filter(|q| !q.dead)
            .min_by(|a, b| {
                let da = (a.wx - p.x).powi(2) + (a.wz - p.z).powi(2);
                let db = (b.wx - p.x).powi(2) + (b.wz - p.z).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|q| v3(q.wx, if q.y != 0.0 { q.y } else { WALL_H } - 0.1, q.wz))
    }
    /// Play the weather's sound `f` from `at` (its source on this level), or
    /// centred when the level has none; → (x, z, pan) for the log.
    pub(crate) fn bed_at(&mut self, at: Option<Vec3>, f: impl FnOnce(&mut Game)) -> (Option<f64>, Option<f64>, f64) {
        match at {
            Some(a) => {
                let pan = self.play_from(a.x, a.y.max(1.2), a.z, BED_REF_DIST, BED_GAIN, SIG_OCCLUDE, f);
                (Some(a.x), Some(a.z), pan)
            }
            None => {
                f(self);
                (None, None, 0.0)
            }
        }
    }

    /// The level's acoustic identity: 3–4 sounds of the weather's ambient
    /// pool (`biased_ambient`), each anchored to an open spot 20–60 m from
    /// the spawn in a direction of its own and recurring on a period of its
    /// own (25–70 s; drawing the silent entry stretches the level's gaps).
    /// The same (run seed, level) always gets the same set: the choices come
    /// from a local stream seeded by `current_seed`, never from `rnd()`.
    pub fn pick_signatures(&mut self) {
        self.amb_signatures.clear();
        let mut r = SplitMix64::new(self.current_seed, SIG_SALT);
        let mut pool: Vec<(f64, &'static str)> =
            if self.biased_ambient.is_empty() { AMBIENT_EVENTS.to_vec() } else { self.biased_ambient.clone() };
        pool.retain(|e| !SIG_NOT_PLACES.contains(&e.1));
        let n = if r.next() < 0.5 { 3 } else { 4 };
        let mut names = vec![];
        let mut gap_mul = 1.0;
        while names.len() < n && !pool.is_empty() {
            let total: f64 = pool.iter().map(|e| e.0).sum();
            let mut x = r.next() * total;
            let mut pick = pool.len() - 1;
            for (k, e) in pool.iter().enumerate() {
                x -= e.0;
                if x < 0.0 {
                    pick = k;
                    break;
                }
            }
            let (_, name) = pool.remove(pick);
            if name.is_empty() {
                gap_mul = SIG_SILENT_GAP;
            } else {
                names.push(name);
            }
        }
        let carry: Vec<SigCarry> = names.iter().map(|n| sig_carry(n)).collect();
        let spots = self.signature_spots(&carry, &mut r);
        for (name, at) in names.into_iter().zip(spots) {
            let period = r.range(25.0, 70.0) * gap_mul;
            // the first time part-way into its period, like every cadence
            self.amb_signatures.push(AmbSignature { name, at, period, t: period * rand2(0.35, 1.0) });
        }
    }

    /// An open spot the player can reach for each sound, as far off as it
    /// carries (failing that, at least 6 m off), spread round the compass
    /// from a random heading and at least 12 m apart where the level allows.
    fn signature_spots(&self, carry: &[SigCarry], r: &mut SplitMix64) -> Vec<Vec3> {
        let n = carry.len();
        if n == 0 || self.w <= 0 || self.h <= 0 {
            return vec![];
        }
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let reach = self.bfs(self.tx(px), self.tz(pz));
        // (x, z, distance, heading) of every reachable tile centre
        let mut open = vec![];
        for z in 0..self.h {
            for x in 0..self.w {
                if reach.get(self.t_idx(x, z)).is_some_and(|&d| d >= 0) {
                    let (wx, wz) = (self.wx(x), self.wz(z));
                    open.push((wx, wz, hypot2(wx - px, wz - pz), (wz - pz).atan2(wx - px)));
                }
            }
        }
        if !open.iter().any(|c| c.2 >= 6.0) {
            return vec![];
        }
        let ang_diff = |a: f64, b: f64| {
            let d = (a - b).rem_euclid(2.0 * PI);
            d.min(2.0 * PI - d)
        };
        let a0 = r.next() * 2.0 * PI;
        let mut out: Vec<Vec3> = vec![];
        for (k, c) in carry.iter().enumerate() {
            let want_a = a0 + k as f64 * 2.0 * PI / n as f64 + r.range(-0.3, 0.3);
            let want_d = r.range(c.near, c.far);
            let mut cands: Vec<_> = open.iter().filter(|o| o.2 >= c.near && o.2 <= c.far).copied().collect();
            if cands.is_empty() {
                cands = open.iter().filter(|o| o.2 >= 6.0).copied().collect();
            }
            let score = |c: &(f64, f64, f64, f64)| ang_diff(c.3, want_a) * 20.0 + (c.2 - want_d).abs();
            let best_of = |apart: f64| {
                let mut best: Option<((f64, f64, f64, f64), f64)> = None;
                for c in &cands {
                    if out.iter().any(|o| hypot2(o.x - c.0, o.z - c.1) < apart) {
                        continue;
                    }
                    let s = score(c);
                    if best.is_none_or(|(_, b)| s < b) {
                        best = Some((*c, s));
                    }
                }
                best.map(|(c, _)| c)
            };
            if let Some(c) = best_of(SIG_APART).or_else(|| best_of(0.1)) {
                out.push(v3(c.0, SIG_Y, c.1));
            }
        }
        out
    }

    /// What the floor under a point is made of, for footsteps.
    pub fn floor_surface_at(&self, px: f64, pz: f64) -> &'static str {
        let z = self.zone_at_world(px, pz).and_then(|zi| self.zone_fields(zi));
        let kind = z.as_ref().map(|z| z.floor_mat).unwrap_or("carpet");
        if kind == "carpet" {
            let info = self.region_at(self.tx(px), self.tz(pz));
            let wet = (info.grime >= 2 || (info.flags & RF_MOSS) != 0) || self.weather_key() == "wet";
            return if wet { "wetcarpet" } else { "carpet" };
        }
        match kind {
            // the Building's photographed floors: the Well's decks and the
            // Works' slab are concrete (the Stair Halls' stone keeps the
            // plain step it always had)
            "deck" | "factory" => "concrete",
            k => k, // concrete | tile | office | stone
        }
    }

    /// A quiet caption the first time you walk into a new kind of place
    fn note_zone_entry(&mut self, z: Option<crate::world::zones::ZoneFields>) {
        self.zone_toast_t -= 1.0 / 60.0;
        let Some(z) = z else { return };
        if self.last_zone_arch.as_deref() == Some(z.arch) {
            return;
        }
        self.last_zone_arch = Some(z.arch.to_string());
        if self.zones_seen.contains(z.arch) || self.zone_toast_t > 0.0 {
            return;
        }
        self.zones_seen.insert(z.arch.to_string());
        self.zone_toast_t = 8.0;
        self.show_msg(&format!("{} — {}", z.title, z.lore), 2.4);
    }

    /// The lamp whose ballast hums for one standing at (x, z), and how far
    /// it is (m): the nearest lit lamp of their own storey, the floor under
    /// it within half a storey of theirs (on the rooms every floor is at 0),
    /// the rise between the two floors counted in how far it is (none on a
    /// level walk); with none of theirs lit, the nearest of any so measured.
    /// By the floor plan alone, on the Building a lamp a storey above or
    /// below stood nearest at a sixth of the walks' tiles, and set the hum
    /// at full strength at one in twenty (code review 4b). (x, z) is where
    /// one stands: inside a mass the floor read is its foot's
    /// (`tests/building_floors.rs`, `the_hum_sits_at_a_lamp_of_your_storey`)
    pub fn hum_lamp(&self, x: f64, z: f64) -> Option<(usize, f64)> {
        let floor = self.floor_y_at(x, z);
        let half = crate::level::building::STOREY / 2.0;
        self.panels
            .iter()
            .enumerate()
            .filter(|(_, p)| p.bright >= 0.3)
            .map(|(i, p)| {
                let rise = self.floor_y_at(p.wx, p.wz) - floor;
                (rise.abs() >= half, (p.wx - x).powi(2) + (p.wz - z).powi(2) + rise * rise, i)
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map(|(_, d2, i)| (i, d2.sqrt()))
    }

    /// Listener, reverb zone, zone captions, the ballast hum
    pub fn update_soundscape(&mut self, dt: f64) {
        let Some(ss) = self.soundscape() else { return };
        let fwd = self.player.fwd();
        let cp = self.camera.pos();
        ss.set_listener(cp.x, cp.y, cp.z, fwd.x, self.player.pitch.sin(), fwd.z);
        let (x, z) = (self.tx(self.player.pos.x), self.tz(self.player.pos.z));
        let dark = self.region_flags_at(x, z) & RF_DARK != 0;
        // Blackout zones are silent in the lore: no hum, almost no room
        let zf = self.zone_at_world(self.player.pos.x, self.player.pos.z).and_then(|zi| self.zone_fields(zi));
        ss.set_zone(if dark { "silent" } else { zf.as_ref().map(|z| z.arch).unwrap_or("lobby") }, 1.2);
        self.note_zone_entry(zf);
        // the hum sits at the nearest live fixture of your storey; re-pick a
        // few times a second
        self.hummer_t -= dt;
        if self.hummer_t <= 0.0 {
            self.hummer_t = 0.25;
            self.hum_panel = self.hum_lamp(self.player.pos.x, self.player.pos.z).map(|(p, d)| HumPanel { p, d });
        }
        let hp = match self.hum_panel {
            Some(hp) if !dark && hp.p < self.panels.len() => hp,
            _ => {
                ss.hum.silence();
                return;
            }
        };
        let lit = self.region_light_at(x, z);
        // the lights' buzz itself sinks away while something comes
        let strength = clamp((0.35 + 0.65 * lit) * clamp(1.4 - hp.d / 14.0, 0.0, 1.0), 0.0, 1.0) * (1.0 - 0.75 * self.amb_hush);
        let p = &self.panels[hp.p];
        ss.hum.update(dt, p.wx, (if p.y != 0.0 { p.y } else { WALL_H }) - 0.1, p.wz, strength);
    }

    /// Every ambient event, cadence timer and prop sound
    pub fn update_ambient(&mut self, dt: f64) {
        if self.state != "playing" || self.ac.is_none() {
            return;
        }
        let wk = self.weather_key();
        let m = self.amb_mix();
        self.amb_rain_windows(dt, wk);
        self.amb_fx_decay(dt);
        self.amb_room_tone(dt, wk, &m);
        self.amb_weather_events(dt, wk, &m);
        self.amb_rare_events(dt, wk, &m);
        self.amb_props_fixtures(dt, &m);
        self.amb_props_trinkets(dt, &m);
        self.amb_props_household(dt, &m);
        self.amb_props_parlour(dt, &m);
        self.amb_props_lobby(dt, &m);
        self.amb_props_rare_fixtures(dt, &m);
        self.amb_signatures(dt, &m);
        self.amb_exit(dt, &m);
        self.amb_shift_bend(dt);
        // Electric surge dim countdown (drives panel dim). Outside the
        // electric branch so a mid-surge weather change still decays the dim.
        if self.electric_surge_t > 0.0 {
            self.electric_surge_t = (0.0f64).max(self.electric_surge_t - dt);
        }
    }

    /// The mix factors.
    fn amb_mix(&self) -> AmbMix {
        // Sanity ambient boost: below 30% sanity the next-cooldown values
        // shrink (2/3 of nominal at 0%); low sanity also boosts event gains.
        let sanity_mag = {
            let s = self.player.sanity / SAN_MAX;
            if s >= 0.30 {
                0.0
            } else {
                1.0 - s / 0.30
            }
        };
        // Sanity timer shrink + descent timer shrink compose.
        let timer_mul = (1.0 / (1.0 + sanity_mag * 0.5)) * (1.0 / self.descent_boost_mul(0.15));
        // Exhaustion muffle: below 20% stamina, up to 15% quieter.
        let stam_mag = {
            let s = self.player.stamina / 100.0;
            if s >= 0.20 {
                0.0
            } else {
                1.0 - s / 0.20
            }
        };
        let stam_gain = 1.0 - stam_mag * 0.15;
        // Cry-dim audio pair: up to 25% muffle at `cry_dim_lerp` = 1.
        let cry_gain = 1.0 - 0.25 * self.cry_dim_lerp;
        // the pacing director's hush
        let hush = if self.amb_hush.is_nan() { 0.0 } else { clamp(self.amb_hush, 0.0, 1.0) };
        // And the hush compose multiplicatively.
        let gain = (1.0 + sanity_mag * 0.35) * self.descent_boost_mul(0.25) * stam_gain * cry_gain * (1.0 - HUSH_DUCK * hush);
        // Level-depth multiplier.
        let depth = self.level_depth_mul();
        AmbMix { sanity_mag, timer_mul, gain, depth, hush, hushed: hush > HUSH_HOLD }
    }

    /// Rain streak overlays + rain hiss.
    fn amb_rain_windows(&mut self, dt: f64, wk: &str) {
        let rain_target = if wk == "wet" { 0.55 } else { 0.0 };
        let mut rain_score: f64 = 0.0; // max over all rain windows
        let yaw = self.yaw0();
        let (cam_fx, cam_fz) = (-yaw.sin(), -yaw.cos());
        let windows = self.rain_windows.clone();
        for w in &windows {
            let Some(mat) = w.try_mat() else { continue };
            let cur = mat.opacity_v();
            mat.set_opacity(cur + (rain_target - cur) * (1f64).min(dt * 2.0));
            if wk == "wet" {
                if let Some(t) = mat.map_v() {
                    let y = t.t().offset.y;
                    t.t().offset.y = (y - dt * 0.6) % 1.0;
                }
                // Score = clamp(facing dot) × distance falloff.
                let wp = w.world_position();
                let (dx, dz) = (wp.x - self.player.pos.x, wp.z - self.player.pos.z);
                let d = (dx * dx + dz * dz).sqrt();
                if d < 12.0 {
                    let dot = (dx * cam_fx + dz * cam_fz) / (if d != 0.0 { d } else { 1.0 }); // -1..1
                    if dot > 0.0 {
                        let near = 1.0 - d / 12.0; // 0..1
                        rain_score = rain_score.max(dot * near);
                    }
                }
            }
            w.set_visible(mat.opacity_v() > 0.01);
        }
        // Rain hiss gain: only active on wet weather + facing a window.
        self.rain_score_current = rain_score; // expose to K panel
        if let Some(n) = &self.audio.rain_hiss_gain {
            let target = if wk == "wet" { 0.025 * rain_score } else { 0.0 };
            let v = n.gain().value();
            n.gain().set_value(v + (target - v) * (1f64).min(dt * 3.0));
        }
    }

    /// Ripple fade, scuff decay, splash flash and
    /// thunder flash decay.
    fn amb_fx_decay(&mut self, dt: f64) {
        // Wet-ripple fade (every weather, so a weather change still fades them)
        for r in self.ripples.iter_mut() {
            if r.t <= 0.0 {
                continue;
            }
            r.t -= dt;
            let mat = r.obj.try_mat();
            if r.t <= 0.0 {
                r.obj.set_visible(false);
                if let Some(m) = &mat {
                    m.set_opacity(0.0);
                }
                continue;
            }
            let p = r.t / RIPPLE_LIFE; // 1 → 0
            if let Some(m) = &mat {
                m.set_opacity(0.55 * p);
            }
            let g = 1.0 + (1.0 - p) * 1.6; // grow 1× → 2.6× as it fades
            r.obj.scale().set(g, g, 1);
        }
        // Dust-scuff decay (derelict footsteps). Rises + fades.
        for s in self.scuffs.iter_mut() {
            if s.t <= 0.0 {
                continue;
            }
            s.t -= dt;
            let mat = s.obj.try_mat();
            if s.t <= 0.0 {
                s.obj.set_visible(false);
                if let Some(m) = &mat {
                    m.set_opacity(0.0);
                }
                continue;
            }
            let p = s.t / SCUFF_LIFE;
            if let Some(m) = &mat {
                m.set_opacity(0.55 * p);
            }
            s.obj.scale().set_scalar(0.35 + (1.0 - p) * 0.15);
            s.obj.position().y += dt * 0.25;
        }
        // Splash flash decay — t² curve
        if self.splash_flash_t > 0.0 {
            self.splash_flash_t -= dt;
            if self.splash_flash_t <= 0.0 {
                self.splash_flash_light.set_intensity(0.0);
            } else {
                let p = self.splash_flash_t / 0.30;
                self.splash_flash_light.set_intensity(self.splash_flash_peak * p * p);
            }
        }
        // Thunder flash decay — t² curve, 0.9 s life, 0.35 peak
        if self.thunder_flash_t > 0.0 {
            self.thunder_flash_t -= dt;
            if self.thunder_flash_t <= 0.0 {
                self.thunder_flash_light.set_intensity(0.0);
            } else {
                let p = self.thunder_flash_t / 0.9;
                self.thunder_flash_light.set_intensity(0.35 * p * p);
            }
        }
    }

    /// Room-tone lerp toward the per-weather target. Below 30 %
    /// sanity it brightens toward 2.4 kHz; the director's hush dips its
    /// level by up to half.
    fn amb_room_tone(&mut self, dt: f64, wk: &str, m: &AmbMix) {
        let (Some(g), Some(lp)) = (self.audio.room_tone_gain.clone(), self.audio.room_tone_lp.clone()) else { return };
        let rt = room_tone_by_weather(wk).or(room_tone_by_weather("normal")).unwrap();
        let target_lp = rt.lp + m.sanity_mag * (2400.0 - rt.lp);
        let target_gain = rt.gain * (1.0 - 0.5 * m.hush);
        let k = (1f64).min(dt * 1.5);
        self.room_tone_cur_gain += (target_gain - self.room_tone_cur_gain) * k;
        self.room_tone_cur_lp += (target_lp - self.room_tone_cur_lp) * k;
        g.gain().set_value(self.room_tone_cur_gain);
        lp.frequency().set_value(self.room_tone_cur_lp);
    }

    /// The level's signature sounds: each from its anchor, on its own steady
    /// beat (its period, ±15 % each time, × the sanity / descent shrink).
    fn amb_signatures(&mut self, dt: f64, m: &AmbMix) {
        for k in 0..self.amb_signatures.len() {
            if !amb_due(&mut self.amb_signatures[k].t, dt, m) {
                continue;
            }
            let s = self.amb_signatures[k].clone();
            self.amb_signatures[k].t = s.period * rand2(0.85, 1.15) * m.timer_mul;
            let name = s.name;
            let c = sig_carry(name);
            let pan = self.play_from(s.at.x, s.at.y, s.at.z, c.ref_dist, c.gain * m.gain, SIG_OCCLUDE, move |g| {
                g.sfx_by_name(name);
            });
            self.log_ambient(&format!("sig-{name}"), Some(s.at.x), Some(s.at.z), Some(pan));
            // a sudden sound heard as near as it carries: your asides may
            // answer the first of its kind (`audio::asides`)
            if let Some(kind) = crate::audio::asides::startle_of(name) {
                if hypot2(s.at.x - self.player.pos.x, s.at.z - self.player.pos.z) <= c.far {
                    self.aside(crate::audio::asides::Use::Startle, kind);
                }
            }
        }
    }

    /// The way down calls: air under the exit door and the sign's ballast
    /// (`sfx_exit_call`), every `EXIT_CALL_PERIOD` ± 1 s, from the door
    /// itself, heard from up to `EXIT_HEARD` m and muffled through walls. It
    /// is how the door is found without a compass (with its green light and
    /// her trail). The director's hush quiets it like the rest of the place.
    /// The Loop's door is an ordinary office door, and does not call.
    fn amb_exit(&mut self, dt: f64, m: &AmbMix) {
        let Some((p, n)) = self.exit_data.as_ref().map(|e| (e.pos, e.normal)) else { return };
        if self.level_scp.as_deref() == Some("loop") || !amb_due(&mut self.exit_call_t, dt, m) {
            return;
        }
        // jittered from a counter, not the shared stream
        self.exit_calls += 1;
        let mut r = SplitMix64::new(self.current_seed, EXIT_SALT ^ self.exit_calls as u64);
        self.exit_call_t = EXIT_CALL_PERIOD + r.range(-1.0, 1.0);
        // the door is a step back from where you use it
        let (x, z) = (p.x - n.x * 0.8, p.z - n.z * 0.8);
        let (dx, dz) = (x - self.player.pos.x, z - self.player.pos.z);
        // (the seeds: what calls you is fainter, `secrets::keeps`)
        let heard = EXIT_HEARD * self.body.door;
        if dx * dx + dz * dz > heard * heard {
            return;
        }
        // (at the door's own floor: on the Building's floors that has height)
        let y = p.y - EYE + 1.2;
        let pan = self.play_from(x, y, z, EXIT_REF_DIST * self.body.door, EXIT_GAIN * m.gain, EXIT_OCCLUDE, |g| g.sfx_exit_call());
        self.log_ambient("exit-call", Some(x), Some(z), Some(pan));
    }

    /// The place's answer to a cry on the Building's floors (`show_the_way`):
    /// the door calls at once, louder and heard further than its own call,
    /// and its own waits a full while after
    pub fn exit_call_now(&mut self) {
        let Some((p, n)) = self.exit_data.as_ref().map(|e| (e.pos, e.normal)) else { return };
        let (x, z) = (p.x - n.x * 0.8, p.z - n.z * 0.8);
        let y = p.y - EYE + 1.2;
        self.exit_call_t = EXIT_CALL_PERIOD;
        let pan = self.play_from(x, y, z, EXIT_REF_DIST * 2.0, EXIT_GAIN * 1.4, EXIT_OCCLUDE * 0.7, |g| g.sfx_exit_call());
        self.log_ambient("exit-call-answer", Some(x), Some(z), Some(pan));
    }

    /// Sanity-crossing chimes and flashes, the
    /// critical-sanity heartbeat, whispers and hallucinated silhouettes
    pub fn update_sanity_effects(&mut self, dt: f64) {
        let s = self.player.sanity / SAN_MAX;
        self.sanity_crossing_chimes(s);
        if s >= 0.28 || self.state != "playing" {
            self.crit_sanity_accum = 0.0;
            self.sanity_beat_t = 0.0;
            self.sanity_whisper_t = 0.0;
            // hide any lingering hallucination when we recover
            if self.halluc_shape.visible() {
                self.halluc_shape.set_visible(false);
                if let Some(m) = self.halluc_shape.try_mat() {
                    m.set_opacity(0.0);
                }
            }
            return;
        }
        self.crit_sanity_accum += dt;
        self.sanity_heartbeat(dt, s);
        // whispered hallucinations only after 10 s of continuous crit-sanity
        if self.crit_sanity_accum > 10.0 {
            self.sanity_whisper_t -= dt;
            if self.sanity_whisper_t <= 0.0 {
                self.sanity_whisper_t = rand2(6.0, 14.0);
                self.sanity_whisper();
            }
        }
        self.halluc_animate(dt);
    }

    /// Recovery / impaired / breaking / mini chimes and their
    /// flashes.
    fn sanity_crossing_chimes(&mut self, s: f64) {
        let wk = self.weather_key();
        let playing = self.state == "playing";
        let bat_gain = |g: &Game| 0.7 + 0.3 * clamp(g.player.battery / 100.0, 0.0, 1.0);
        // Recovery chime on the upward crossing back above 55%; an impaired
        // chime on the downward crossing.
        let is_impaired = s < 0.55 && playing;
        if self.sanity_was_impaired && !is_impaired && playing {
            if self.ac.is_some() {
                self.sfx_chime();
                let mel = recovery_melody_by_weather(wk).or(recovery_melody_by_weather("normal")).unwrap();
                let mel_gain = bat_gain(self);
                let mel_depth = self.chime_depth_mul(); // sanity chords treated as chime-tier
                self.blip(mel.root * mel_depth, 0.55, "sine", 0.055 * mel_gain, None, 0.0, 0.0);
                self.blip(mel.up * mel_depth, 0.65, "sine", 0.040 * mel_gain, None, 0.30, 0.0);
                self.blip(mel.top * mel_depth, 0.75, "sine", 0.028 * mel_gain, None, 0.55, 0.0);
                // Recovery visual flash: brief soft green-white pulse (relief).
                self.chime_flash_t = 0.9;
                self.chime_flash_life = 0.9;
                self.chime_flash_peak = 0.18;
                self.chime_flash_color = Some("rgba(200, 240, 200".into());
                self.sanity_recovery_tally += 1.0;
            }
        } else if !self.sanity_was_impaired && is_impaired && self.ac.is_some() {
            // Downward crossing: 3-note descending motif, battery-scaled.
            let imp = impaired_melody_by_weather(wk).or(impaired_melody_by_weather("normal")).unwrap();
            let imp_gain = bat_gain(self);
            let imp_depth = self.chime_depth_mul();
            self.blip(imp.root * imp_depth, 0.45, "sine", 0.040 * imp_gain, None, 0.0, 0.0);
            self.blip(imp.down * imp_depth, 0.60, "sine", 0.030 * imp_gain, None, 0.25, 0.0);
            self.blip(imp.bottom * imp_depth, 0.75, "sine", 0.022 * imp_gain, None, 0.50, 0.0);
            // Impaired visual flash: muted violet pulse (dread creeping in).
            self.chime_flash_t = 0.8;
            self.chime_flash_life = 0.8;
            self.chime_flash_peak = 0.22;
            self.chime_flash_color = Some("rgba(160, 120, 180".into());
            self.sanity_impaired_tally += 1.0;
        }
        // Deep-breakdown chime below 20%; a warm mini-chime back above it.
        let is_breaking = s < 0.20 && playing;
        if self.sanity_was_breaking && !is_breaking && playing && self.ac.is_some() {
            let brk = breaking_melody_by_weather(wk).or(breaking_melody_by_weather("normal")).unwrap();
            // single note two octaves above the breaking chord's `a`
            let bk_gain = bat_gain(self);
            let bk_depth = self.chime_depth_mul();
            self.blip(brk.a * 4.0 * bk_depth, 0.55, "sine", 0.035 * bk_gain, None, 0.0, 0.0);
            self.chime_flash_t = 0.6;
            self.chime_flash_life = 0.6;
            self.chime_flash_peak = 0.14;
            self.chime_flash_color = Some("rgba(220, 240, 210".into());
            self.sanity_mini_tally += 1.0;
        }
        if !self.sanity_was_breaking && is_breaking && playing && self.ac.is_some() {
            let brk = breaking_melody_by_weather(wk).or(breaking_melody_by_weather("normal")).unwrap();
            let brk_gain = bat_gain(self);
            let brk_depth = self.chime_depth_mul();
            self.blip(brk.a * brk_depth, 0.60, "sine", 0.055 * brk_gain, None, 0.0, 0.0);
            self.blip(brk.b * brk_depth, 0.75, "sine", 0.045 * brk_gain, None, 0.30, 0.0);
            self.blip(brk.c * brk_depth, 0.95, "sine", 0.038 * brk_gain, None, 0.65, 0.0);
            self.blip(brk.c * 0.5 * brk_depth, 1.10, "sine", 0.024 * brk_gain, None, 0.80, 0.0);
            // Deep-breakdown visual flash: heavy red-black pulse.
            self.chime_flash_t = 1.4;
            self.chime_flash_life = 1.4;
            self.chime_flash_peak = 0.42;
            self.chime_flash_color = Some("rgba(120, 40, 40".into());
            self.sanity_breaking_tally += 1.0;
        }
        self.sanity_was_breaking = is_breaking;
        self.sanity_was_impaired = is_impaired;
    }

    /// The critical-sanity heartbeat: ~1.4 s at 28 % down to ~0.5 s at 0 %.
    fn sanity_heartbeat(&mut self, dt: f64, s: f64) {
        self.sanity_beat_t -= dt;
        if self.sanity_beat_t <= 0.0 {
            let pct_below = clamp((0.28 - s) / 0.28, 0.0, 1.0); // 0 → 1 as sanity falls
            self.sanity_beat_t = lerp(1.4, 0.5, pct_below);
            if self.ac.is_some() {
                self.sfx_thump(0.65 + pct_below * 0.35);
            }
        }
    }

    /// One whisper: the sound, the aberration spike and maybe a silhouette
    /// at the edge of vision.
    fn sanity_whisper(&mut self) {
        if self.ac.is_some() {
            self.sfx_whisper();
        }
        // fleeting shape: brief spike of the aberration overlay
        {
            let cur = self.ui.aberration.style("opacity").filter(|s| !s.is_empty()).unwrap_or("0").to_string();
            self.ui.aberration.set_style("transition", "opacity 0.08s");
            self.ui.aberration.set_style("opacity", "0.98");
            self.schedule(220.0, move |g| {
                g.ui.aberration.set_style("transition", "opacity 0.4s");
                g.ui.aberration.set_style("opacity", &cur);
            });
        }
        // ~55% chance of an actual visible silhouette, ahead-and-off-axis
        if rnd() < 0.55 {
            let yaw = self.player.yaw;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let side = if rnd() < 0.5 { 1.0 } else { -1.0 };
            let (rx, rz) = (yaw.cos() * side, -yaw.sin() * side);
            let mut placed = false;
            let mut tries = 0;
            while tries < 10 && !placed {
                tries += 1;
                let dist = rand2(4.0, 9.0);
                let off = rand2(1.5, 3.5);
                let px = self.player.pos.x + fx * dist + rx * off;
                let pz = self.player.pos.z + fz * dist + rz * off;
                let (t_x, t_z) = (self.tx(px), self.tz(pz));
                if !self.in_grid(t_x, t_z) || self.is_wall_tile(t_x, t_z) {
                    continue;
                }
                self.halluc_shape.set_pos(px, 0.95, pz);
                // a random pose per appearance
                if !self.halluc_textures.is_empty() {
                    let tex = choice(&self.halluc_textures);
                    if let Some(m) = self.halluc_shape.try_mat() {
                        m.set_map(&tex);
                        m.needs_update();
                    }
                } else {
                    rnd(); // an empty pick still consumes its draw
                }
                self.halluc_shape.set_visible(true);
                if let Some(m) = self.halluc_shape.try_mat() {
                    m.set_opacity(0.65);
                }
                self.halluc_visible_t = 0.0;
                placed = true;
                // on-screen whisper chance ramps 0.30 → 0.60 over 10–40 s of crit sanity
                let ramp_t = clamp((self.crit_sanity_accum - 10.0) / 30.0, 0.0, 1.0);
                let whisper_chance = 0.30 + ramp_t * 0.30;
                if rnd() < whisper_chance {
                    let msg = choice(&HALLUC_WHISPERS);
                    self.show_msg(msg, 2.0);
                }
            }
        }
    }

    /// The hallucination's per-frame billboard, mid-appearance pose swap and
    /// fade.
    fn halluc_animate(&mut self, dt: f64) {
        if !self.halluc_shape.visible() {
            return;
        }
        let hs = self.halluc_shape.clone();
        self.halluc_visible_t += dt;
        // face the camera each frame (billboard)
        let cp = self.camera.pos();
        hs.look_at(cp.x, hs.pos().y, cp.z);
        // 25% chance for a ~30 ms pose swap in the middle of the appearance
        if self.halluc_visible_t > 0.25 && self.halluc_visible_t < 0.35 && hs.r().user.get("swapped") == 0.0 {
            hs.user().set("swapped", 1);
            if rnd() < 0.25 {
                let current = hs.try_mat().and_then(|m| m.map_v());
                let others: Vec<Tex> = self.halluc_textures.iter().filter(|t| Some(*t) != current.as_ref()).cloned().collect();
                if !others.is_empty() {
                    let t = choice(&others);
                    if let Some(m) = hs.try_mat() {
                        m.set_map(&t);
                        m.needs_update();
                    }
                    // faint high shimmer with the visual glitch
                    if self.ac.is_some() {
                        self.blip(rand2(3200.0, 3800.0), 0.06, "sine", 0.04, None, 0.0, 0.0);
                        self.blip(rand2(4200.0, 5200.0), 0.05, "sine", 0.03, None, 0.02, 0.0);
                    }
                }
            }
        }
        // fade over 0.7 s then hide
        let fade = clamp(1.0 - self.halluc_visible_t / 0.7, 0.0, 1.0);
        if let Some(m) = hs.try_mat() {
            m.set_opacity(0.65 * fade);
        }
        if self.halluc_visible_t > 0.7 {
            hs.set_visible(false);
            hs.user().set("swapped", 0);
        }
    }

    /// Rare event that thickens fog, spikes the
    /// aberration overlay, plays a discord and shows a message from the walls
    pub fn update_reality_glitch(&mut self, dt: f64) {
        if self.glitch_t > 0.0 {
            self.glitch_t -= dt;
            // during the glitch `update_lights` dims the panels
            if self.glitch_t <= 0.0 {
                // restore — preserve the level's weather-driven fog density
                self.fog_density = FOG_BASE * self.level_weather.fog_mul;
                let v = clamp((1.0 - self.player.sanity / SAN_MAX) * 0.7, 0.0, 1.0);
                self.ui.aberration.set_opacity(v);
            }
            return;
        }
        // never while the place holds its breath for something coming
        if self.amb_hush > HUSH_HOLD {
            return;
        }
        self.glitch_next_t -= dt;
        if self.glitch_next_t > 0.0 {
            return;
        }
        // trigger glitch
        self.glitch_t = rand2(1.6, 3.2);
        self.glitch_next_t = rand2(60.0, 130.0);
        self.sfx_glitch();
        self.fog_density = FOG_BASE * 3.5;
        self.ui.aberration.set_opacity(0.85);
        // a fake message from the walls
        let msg = choice(&GLITCH_MSGS);
        self.show_msg(msg, 2.0);
        // shakes the player
        self.shake_t = 0.6;
    }

    /// Every 90–180 s a surge ripples flicker
    /// through the panels outward from the player
    pub fn update_flicker_wave(&mut self, dt: f64) {
        if self.amb_hush > HUSH_HOLD {
            return;
        }
        self.flicker_wave_next_t -= dt;
        if self.flicker_wave_next_t > 0.0 {
            return;
        }
        self.flicker_wave_next_t = rand2(90.0, 180.0);
        if self.panels.is_empty() {
            return;
        }
        // low-frequency thump at wave start
        if self.ac.is_some() && self.state == "playing" {
            self.sfx_thud();
            self.noise_burst(0.4, 90.0, 1.2, 0.18, "lowpass", None, 0.0, 0.0);
        }
        // stagger flicker outward from the player: closer panels flicker sooner
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let mut ordered: Vec<(usize, f64, f64, f64)> =
            self.panels.iter().enumerate().map(|(i, p)| (i, p.wx, p.wz, hypot2(p.wx - px, p.wz - pz))).collect();
        ordered.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        let step = 55.0; // ms between successive panel flickers
        for (k, (i, wx, wz, _)) in ordered.into_iter().enumerate() {
            self.schedule(k as f64 * step, move |g| {
                if g.state != "playing" {
                    return;
                }
                // force flicker for a short interval — `update_lights` handles the visuals
                let f = rand2(0.35, 0.9);
                // (only the same panel: a rebuilt level's panels are not it)
                if let Some(p) = g.panels.get_mut(i) {
                    if p.wx == wx && p.wz == wz {
                        p.flicker = f;
                    }
                }
            });
        }
    }

    /// Now and then while you walk, a few steps somewhere behind you: 6–12 m
    /// back and a little to one side, a few paces closer at one steady gait,
    /// then nothing. The countdown runs only while you walk and holds while
    /// you stand.
    pub fn update_ghost_footsteps(&mut self, dt: f64, moving: bool) {
        if !moving {
            return;
        }
        self.ghost_step_t -= dt;
        if self.ghost_step_t > 0.0 {
            return;
        }
        self.ghost_step_t = rand2(45.0, 110.0);
        // never while something real is coming: its own steps are the cue then
        if self.ac.is_none() || self.amb_hush > 0.2 {
            return;
        }
        let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let path = self.ghost_walk_path(side * rand2(0.25, 0.6), rand2(6.0, 12.0), irand(3, 5) as usize);
        let gap = rand2(475.0, 525.0); // ms between footfalls: one steady gait
        let lvl = self.level_start_t;
        let n = path.len();
        for (k, p) in path.iter().copied().enumerate() {
            let w = 0.7 + 0.25 * k as f64 / n as f64; // a touch heavier as they come closer
            self.schedule(k as f64 * gap, move |g| {
                if g.ghost_can_play(lvl) {
                    g.ghost_footfall(p.x, p.z, w);
                }
            });
        }
        // a faint breath often follows, from where they stopped
        if rnd() < 0.35 {
            let last = path[n - 1];
            self.schedule(n as f64 * gap + 250.0, move |g| {
                if g.ghost_can_play(lvl) {
                    g.play_at(last.x, 1.6, last.z, GHOST_REF_DIST, 1.6, |g| g.sfx_breath());
                }
            });
        }
        let (_, pan, _) = self.ear(path[0].x, path[0].z);
        self.log_ambient("ghost-steps", Some(path[0].x), Some(path[0].z), Some(pan));
    }

    /// The footfalls of one ghost walk: from `dist` m behind the player,
    /// `off` radians to one side of directly behind, `n` paces of 0.55 m
    /// toward where the player is now. A start inside a wall comes closer
    /// if it can (never nearer than 6 m).
    pub fn ghost_walk_path(&self, off: f64, dist: f64, n: usize) -> Vec<Vec3> {
        let yaw = self.yaw0();
        let (bx, bz) = (yaw.sin(), yaw.cos()); // behind: −forward
        let (c, s) = (off.cos(), off.sin());
        let (ux, uz) = (bx * c - bz * s, bx * s + bz * c);
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let open = |d: f64| !self.is_wall_tile(self.tx(px + ux * d), self.tz(pz + uz * d));
        let start = [dist, dist - 1.5, dist - 3.0].into_iter().find(|&d| d >= 6.0 && open(d)).unwrap_or(dist);
        (0..n.max(1))
            .map(|k| {
                let r = start - 0.55 * k as f64;
                v3(px + ux * r, 0.0, pz + uz * r)
            })
            .collect()
    }

    /// Whether a scheduled ghost sound may still play (same level, still
    /// walking the Backrooms)
    fn ghost_can_play(&self, level_start_t: f64) -> bool {
        self.ac.is_some() && self.state == "playing" && !self.in_stairwell && !self.in_loop && self.level_start_t == level_start_t
    }

    /// One ghost footfall at (x, z), on whatever floor is there
    fn ghost_footfall(&mut self, x: f64, z: f64, weight: f64) {
        match self.soundscape() {
            Some(ss) => {
                let (_, _, clear) = self.ear(x, z);
                let slot = ss.voice_slot(GHOST_REF_DIST);
                slot.place(x, 0.1, z, if clear { 0.0 } else { 0.7 });
                ss.footstep(self.floor_surface_at(x, z), weight, Some(&slot.input));
            }
            None => {
                self.play_at(x, 0.1, z, GHOST_REF_DIST, 2.0 * weight, |g| g.sfx_ghost_step());
            }
        }
    }

    /// The metronome's swing period (`pp.metronome_periods[i]`), or the
    /// default when the placer did not record one.
    pub(crate) fn metronome_period(&self, i: usize) -> f64 {
        if self.pp.metronome_periods.len() == self.pp.metronome_positions.len() {
            self.pp.metronome_periods.get(i).copied().unwrap_or(METRONOME_PERIOD_DEFAULT)
        } else {
            METRONOME_PERIOD_DEFAULT
        }
    }
    /// The rocking chair's rock period (`pp.chair_periods[i]`), or the
    /// default when the placer did not record one.
    pub(crate) fn chair_period(&self, i: usize) -> f64 {
        if self.pp.chair_periods.len() == self.pp.chair_positions.len() {
            self.pp.chair_periods.get(i).copied().unwrap_or(CHAIR_PERIOD_DEFAULT)
        } else {
            CHAIR_PERIOD_DEFAULT
        }
    }
}
