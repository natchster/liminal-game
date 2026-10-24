//! The soundscape driver (ambient.rs + ambient_*.rs) on a synthetic level with
//! a live, muted AudioContext: every weather, minutes of simulated time in
//! small and large steps, sanity swept 100 → 0 → 100, every `*Positions`
//! list filled. Asserts nothing panics and that the event families fire.
//!
//! Other modules are still contract stubs here (`logAmbient`,
//! `panForWorldPos`, `zoneAtWorld`, `levelDepthMul`, ...), so the checks look
//! at state this module owns (timers, tallies, envelopes, pools). Once the
//! world module's `logAmbient` is ported, `ambient_tally` is checked too.

use engine::audio::AudioContext;
use engine::prelude::*;
use game::ambient::{GLITCH_MSGS, HALLUC_WHISPERS};
use game::records::Panel;
use game::state::WEATHER_BIASES;
use game::Game;
use std::collections::BTreeSet;

/// Every `*Positions` list, as `&mut` (generated from positions.rs).
macro_rules! all_lists {
    ($pp:expr) => {
        [
            &mut $pp.socket_positions, &mut $pp.fountain_positions, &mut $pp.radiator_positions, &mut $pp.glow_light_positions,
            &mut $pp.motion_sensor_positions, &mut $pp.speaker_positions, &mut $pp.fire_alarm_positions, &mut $pp.phone_positions,
            &mut $pp.broken_phone_positions, &mut $pp.vending_positions, &mut $pp.junction_box_positions, &mut $pp.keycard_reader_positions,
            &mut $pp.cardboard_box_positions, &mut $pp.living_plant_positions, &mut $pp.walkie_talkie_positions, &mut $pp.wall_clock_positions,
            &mut $pp.cracked_clock_positions, &mut $pp.ceiling_drip_positions, &mut $pp.laptop_positions, &mut $pp.ceiling_cable_plug_positions,
            &mut $pp.thermostat_positions, &mut $pp.handbell_positions, &mut $pp.digital_clock_positions, &mut $pp.alarm_clock_positions,
            &mut $pp.off_hook_rotary_positions, &mut $pp.drain_grate_positions, &mut $pp.bathroom_bundle_positions, &mut $pp.cauldron_positions,
            &mut $pp.metronome_positions, &mut $pp.phonograph_positions, &mut $pp.brass_lantern_positions, &mut $pp.sewing_positions,
            &mut $pp.pocket_watch_positions, &mut $pp.bat_positions, &mut $pp.globe_positions, &mut $pp.hourglass_positions,
            &mut $pp.ghost_positions, &mut $pp.chain_positions, &mut $pp.paper_positions, &mut $pp.piano_positions,
            &mut $pp.chair_positions, &mut $pp.crt_positions, &mut $pp.exit_sign_positions, &mut $pp.puddle_positions,
            &mut $pp.chandelier_positions, &mut $pp.cassette_positions, &mut $pp.teacup_positions, &mut $pp.locker_positions,
            &mut $pp.bulletin_positions, &mut $pp.alarm_pull_positions, &mut $pp.pay_phone_positions, &mut $pp.radio_positions,
            &mut $pp.keychain_positions, &mut $pp.elevator_panel_positions, &mut $pp.plant_positions, &mut $pp.fan_positions,
            &mut $pp.smoke_detector_positions, &mut $pp.coin_positions, &mut $pp.clock_positions, &mut $pp.cuckoo_clock_positions,
            &mut $pp.plush_positions, &mut $pp.gumball_positions, &mut $pp.teapot_positions, &mut $pp.cookie_tin_positions,
            &mut $pp.kitchen_clock_positions, &mut $pp.off_hook_phone_positions, &mut $pp.hairdryer_positions, &mut $pp.sewing_machine_positions,
            &mut $pp.herb_sieve_positions, &mut $pp.vacuum_positions, &mut $pp.mop_bucket_positions, &mut $pp.wristwatch_positions,
            &mut $pp.hot_iron_positions, &mut $pp.grandfather_clock_positions, &mut $pp.rocking_chair_positions, &mut $pp.tall_lamp_positions,
            &mut $pp.colander_positions, &mut $pp.pet_bowl_positions, &mut $pp.music_box_positions, &mut $pp.turntable_positions,
            &mut $pp.chalkboard_positions, &mut $pp.kitchen_calendar_positions, &mut $pp.telescope_positions, &mut $pp.cuckoo_clock_positions_r741,
            &mut $pp.lit_fireplace_positions, &mut $pp.aquarium_positions, &mut $pp.wingback_armchair_positions, &mut $pp.pocket_watch_positions_r755,
            &mut $pp.bell_pull_positions, &mut $pp.bell_rack_positions, &mut $pp.chess_set_positions, &mut $pp.hobby_horse_wheeled_positions,
        ]
    };
}

/// Every prop / outdoor cadence timer updateAmbient owns (all start at 0 and
/// only ever get a positive cadence when their event fires).
fn prop_timers(g: &Game) -> Vec<(&'static str, f64)> {
    vec![
        ("fountain", g.fountain_drip_timer), ("radiator", g.radiator_hiss_timer), ("glow", g.glow_hum_timer), ("cauldron", g.cauldron_bubble_timer),
        ("metronome", g.metronome_tick_timer), ("phono", g.phono_crackle_timer), ("lantern", g.lantern_creak_timer), ("sewing", g.sewing_whirr_timer),
        ("watch", g.watch_tick_timer), ("bat", g.bat_squeak_timer), ("globe", g.globe_creak_timer), ("hourglass", g.hourglass_whoosh_timer),
        ("ghost", g.ghost_whisper_timer), ("chain", g.chain_jangle_timer), ("paper", g.paper_rustle_timer), ("piano", g.piano_chord_timer),
        ("chair", g.chair_creak_timer), ("crt", g.crt_hiss_timer), ("exit", g.exit_buzz_timer), ("puddle", g.puddle_drip_timer),
        ("chandelier", g.chandelier_chime_timer), ("cassette", g.cassette_hiss_timer), ("teacup", g.teacup_clink_timer), ("locker", g.locker_slam_timer),
        ("bulletin", g.bulletin_flutter_timer), ("alarmPull", g.alarm_beep_timer), ("fanWhoosh", g.fan_whoosh_timer), ("fanSqueak", g.fan_squeak_timer),
        ("detector", g.detector_chirp_timer), ("bird", g.bird_chirp_timer), ("cricket", g.cricket_chirp_timer), ("thunderRumble", g.thunder_rumble_timer),
        ("coin", g.coin_tink_timer), ("windGust", g.wind_gust_timer), ("frog", g.frog_croak_timer), ("owl", g.owl_hoot_timer),
        ("tickTock", g.tick_tock_timer), ("wolf", g.wolf_howl_timer), ("rainPatter", g.rain_patter_timer), ("leaf", g.leaf_rustle_timer),
        ("cuckoo", g.cuckoo_call_timer), ("plush", g.plush_shuffle_timer), ("gumball", g.gumball_rattle_timer), ("teapot", g.teapot_whistle_timer),
        ("cookieTin", g.cookie_tin_rattle_timer), ("kitchenClock", g.kitchen_clock_tick_timer), ("offHookPhone", g.off_hook_phone_hum_timer),
        ("hairdryer", g.hairdryer_hum_timer), ("sewingMachine", g.sewing_machine_click_timer), ("herbSieve", g.herb_sieve_scrape_timer),
        ("vacuum", g.vacuum_whine_timer), ("mopBucket", g.mop_bucket_slosh_timer), ("wristwatch", g.wristwatch_tick_timer), ("iron", g.iron_hiss_timer),
        ("grandfatherTick", g.grandfather_clock_tick_timer), ("rockingChair", g.rocking_chair_creak_timer), ("tallLamp", g.tall_lamp_buzz_timer),
        ("colander", g.colander_tap_timer), ("petBowl", g.pet_bowl_slosh_timer), ("musicBox", g.music_box_melody_timer),
        ("grandfatherChime", g.grandfather_chime_timer), ("turntable", g.turntable_hiss_timer), ("chalkboard", g.chalkboard_scratch_timer),
        ("kitchenCalendar", g.kitchen_calendar_turn_timer), ("telescope", g.telescope_click_timer), ("cuckooR741", g.cuckoo_call_timer_r741),
        ("fireplace", g.fireplace_crackle_timer), ("aquarium", g.aquarium_hum_timer), ("wingback", g.wingback_creak_timer),
        ("pocketWatchR755", g.pocket_watch_tick_timer), ("bellPull", g.bell_pull_ring_timer), ("bellRack", g.bell_rack_ding_timer),
        ("chess", g.chess_move_click_timer), ("hobbyHorse", g.hobby_horse_roll_timer), ("payPhone", g.pay_phone_ring_timer),
        ("radio", g.radio_static_timer), ("keychain", g.keychain_jangle_timer), ("elevatorBeep", g.elevator_beep_timer), ("plant", g.plant_rustle_timer),
        ("speaker", g.speaker_announce_timer), ("fireAlarm", g.fire_alarm_beep_timer), ("phoneRing", g.phone_ring_timer), ("vending", g.vending_hum_timer),
        ("keycard", g.keycard_beep_timer), ("cardboard", g.cardboard_shift_timer), ("plantR291", g.plant_rustle_timer_r291),
        ("walkie", g.walkie_static_timer), ("wallClock", g.wall_clock_tick_timer), ("crackedClock", g.cracked_clock_creak_timer),
        ("ceilingDrip", g.ceiling_drip_audio_timer), ("ceilingCable", g.ceiling_cable_spark_timer), ("thermostat", g.thermostat_click_timer),
        ("handbell", g.handbell_chime_timer), ("digitalClock", g.digital_clock_buzz_timer), ("alarmRing", g.alarm_ring_timer),
        ("dialTone", g.dial_tone_timer), ("drain", g.drain_gurgle_timer), ("faucet", g.faucet_drip_timer), ("brokenPhone", g.broken_phone_spark_timer),
    ]
}

/// The per-weather bed timers that should have fired for weather `wk`.
fn weather_timers(g: &Game, wk: &str) -> Vec<(&'static str, f64)> {
    match wk {
        "wet" => vec![("wetDrip", g.wet_drip_timer), ("wetThunder", g.wet_thunder_timer), ("wetSplash", g.wet_splash_timer)],
        "still" => vec![("stillWind", g.still_wind_timer), ("stillEcho", g.still_echo_timer)],
        "derelict" => vec![("derelictClatter", g.derelict_clatter_timer), ("derelictGrinder", g.derelict_grinder_timer)],
        "electric" => vec![("electricCrackle", g.electric_crackle_timer), ("socketSpark", g.socket_spark_timer), ("junctionArc", g.junction_arc_timer), ("electricSurge", g.electric_surge_timer)],
        "haunted" => vec![("hauntedWhisper", g.haunted_whisper_timer), ("hauntedStep", g.haunted_step_timer)],
        _ => vec![("normalNotice", g.normal_notice_timer)],
    }
}

fn muted_ac() -> AudioContext { let ac = AudioContext::new(); ac.set_muted(true); ac }

/// The synthetic level with the renderer's pools, live muted audio, panels
/// near the spawn, a rain window, and every position list filled with a few
/// entries (close, mid-range and far from the spawn).
fn live_level(seed: u32, weather: usize) -> Game {
    let (mut g, _dist) = Game::test_level(seed, 25);
    g.init_render(&Tex::data_rgba(vec![255; 4], 1, 1));
    g.audio_init_with(muted_ac());
    g.level_weather = &WEATHER_BIASES[weather];
    g.state = "playing".into();
    let (sx, sz) = (g.player.pos.x, g.player.pos.z);
    g.camera.set_pos(sx, 1.62, sz);
    for (dx, dz) in [(0.6, 0.0), (2.5, 3.0), (-6.0, 1.0), (1.0, -1.5)] {
        g.panels.push(Panel { wx: sx + dx, wz: sz + dz, bright: 1.0, lfo_freq: 0.3, ..Default::default() });
    }
    g.panels.push(Panel { wx: sx + 1.0, wz: sz, bright: 0.1, dead: true, ..Default::default() });
    // a rain-window overlay a few metres ahead (yaw 0 faces -z)
    let tex = Tex::data_rgba(vec![255; 4], 1, 1);
    let w = mesh(plane_geo!(1.2, 1.2), basic_mat!({ map: &tex, transparent: true, opacity: 0, depthWrite: false }));
    w.set_pos(sx, 1.4, sz - 3.0);
    g.world.add(&w);
    g.rain_windows.push(w);
    let offs = [(0.4, 0.3), (1.2, -0.8), (-2.6, 2.2), (4.5, -3.5), (11.0, 7.0)];
    for list in all_lists!(g.pp) {
        for (dx, dz) in offs { list.push(v3(sx + dx, 0.3, sz + dz)); }
    }
    g.pp.metronome_periods = vec![1.2, 1.4, 1.5, 1.6, 1.8];
    g.pp.chair_periods = vec![5.0, 6.0, 6.5, 7.0, 8.0];
    g.init_ambient();
    g
}

/// What a run saw (envelopes and pools are transient, so sample them every step).
#[derive(Default, Debug)]
struct Seen {
    ripple: bool, scuff: bool, splash_flash: bool, thunder_flash: bool, surge: bool, settle: bool,
    halluc: bool, halluc_msg: bool, glitch: bool, glitch_msg: bool, shift: bool, pattern: bool, skip: bool,
    knock_tint: bool, chime_flash: bool, hum_panel: bool, rain_hiss: bool, lfo_bent: bool,
}

/// Run one weather for `minutes` of simulated time: alternating stretches of
/// 60 fps frames and coarse 0.25–2 s steps, sanity 100 → 0 over the first
/// 70 % then back to 100, the player turning and drifting, the real-time
/// clock advancing with the sim so the `setTimeout` callbacks run.
fn run(g: &mut Game, minutes: f64, seen: &mut Seen) {
    let total = minutes * 60.0;
    let (sx, sz) = (g.player.pos.x, g.player.pos.z);
    let mut t = 0.0;
    let mut k: u64 = 0;
    let mut forced = 0;
    while t < total {
        // small steps most of the time, coarse ones in between
        let dt = match (k / 300) % 4 { 0 | 2 => 1.0 / 60.0, 1 => 0.25, _ => [0.5, 1.0, 2.0][(k % 3) as usize] };
        k += 1;
        t += dt;
        let f = t / total;
        g.player.sanity = if f < 0.7 { 100.0 * (1.0 - f / 0.7) } else { 100.0 * ((f - 0.7) / 0.3).min(1.0) };
        g.player.stamina = 50.0 + 50.0 * (t * 0.05).sin();
        g.player.battery = 50.0 + 50.0 * (t * 0.01).cos();
        g.cry_dim_lerp = if (t as i64) % 40 < 3 { 0.8 } else { 0.0 };
        g.player.yaw = t * 0.7;
        g.player.pitch = 0.2 * (t * 0.3).sin();
        g.player.pos.x = sx + 0.4 * (t * 0.11).cos();
        g.player.pos.z = sz + 0.4 * (t * 0.11).sin();
        g.player.bob_amp = if (t as i64) % 30 < 20 { 0.05 } else { 0.0 };
        let halluc_at = g.halluc_shape.pos();
        g.perf_t += dt;
        g.now_ms += dt * 1000.0;
        g.run_timeouts();
        // the main loop's order (frame(): glitch, flicker, ghosts, ..., sanity, ambient, soundscape)
        g.update_reality_glitch(dt);
        g.update_flicker_wave(dt);
        let moving = g.player.bob_amp > 0.02;
        g.update_ghost_footsteps(dt, moving);
        g.update_sanity_effects(dt);
        g.update_ambient(dt);
        g.update_soundscape(dt);
        // the rare events only come back after 4 min … 3 h: re-arm a few, a few times
        if forced < 12 && k % 100 == 99 {
            forced += 1;
            g.reality_shift_timer = 0.0;
            g.pattern_break_timer = 0.0;
            g.clock_skip_timer = 0.0;
            g.distant_knock_timer = 0.0;
        }
        seen.ripple |= g.ripples.iter().any(|r| r.t > 0.0);
        seen.scuff |= g.scuffs.iter().any(|s| s.t > 0.0);
        seen.splash_flash |= g.splash_flash_t > 0.0;
        seen.thunder_flash |= g.thunder_flash_t > 0.0;
        seen.surge |= g.electric_surge_t > 0.0;
        seen.settle |= g.settle_visual_t > 0.0;
        // a coarse step can place, show and fade a silhouette in one call: watch it move
        seen.halluc |= g.halluc_shape.visible() || g.halluc_shape.pos() != halluc_at;
        seen.halluc_msg |= HALLUC_WHISPERS.contains(&g.msg_text.as_str());
        seen.glitch |= g.glitch_t > 0.0;
        seen.glitch_msg |= GLITCH_MSGS.contains(&g.msg_text.as_str());
        seen.shift |= g.reality_shift_t > 0.0;
        seen.pattern |= g.pattern_break_t > 0.0;
        seen.skip |= g.clock_skip_t > 0.0;
        seen.knock_tint |= g.knock_tint_t > 0.0;
        seen.chime_flash |= g.chime_flash_t > 0.0;
        seen.hum_panel |= g.hum_panel.is_some();
        seen.rain_hiss |= g.audio.rain_hiss_gain.as_ref().map(|n| n.gain().value() > 1e-4).unwrap_or(false);
        seen.lfo_bent |= g.audio.safe_room_lfo.as_ref().map(|n| (n.frequency().value() - 0.35).abs() > 1e-6).unwrap_or(false);
        // updateSanityEffects' chime flash lives in updateEntities' vignette
        // envelope in JS; count it down here so later crossings are visible
        if g.chime_flash_t > 0.0 { g.chime_flash_t -= dt; }
        if g.knock_tint_t > 0.0 { g.knock_tint_t = (g.knock_tint_t - dt).max(0.0); }
        if g.knock_flash_t > 0.0 { g.knock_flash_t -= dt; }
        for v in [&mut g.door_opened_flash_t, &mut g.elevator_ding_flash_t, &mut g.voice_sample_flash_t, &mut g.static_burst_flash_t, &mut g.settle_visual_t] {
            if *v > 0.0 { *v = (*v - dt).max(0.0); }
        }
    }
}

#[test]
fn every_weather_for_minutes() {
    let causes = ["knock", "door", "ding", "voice", "static", "shift", "pattern", "skip"];
    let mut settle_any = false;
    let mut halluc_any = false;
    let mut halluc_msg_any = false;
    for wi in 0..WEATHER_BIASES.len() {
        let wk = WEATHER_BIASES[wi].key;
        let mut g = live_level(1000 + wi as u32, wi);
        // the ambience loop is armed at module load
        assert!(!g.timeouts.is_empty(), "{wk}: ambienceLoop not scheduled");
        // logAmbient may still be the world module's stub; probe it
        g.log_ambient("probe", None, None, None);
        let log_ported = g.ambient_tally.contains_key("probe");
        let mut seen = Seen::default();
        game::rng::with_seed(0xA11CE + wi as u32, || run(&mut g, 8.0, &mut seen));

        // every prop and every bed of this weather fired at least once (its
        // cadence timer only turns positive when it fires)
        for (name, v) in prop_timers(&g).into_iter().chain(weather_timers(&g, wk)) {
            assert!(v > 0.0, "{wk}: {name} never fired (timer {v})");
        }
        // every rare-event class fired, tallied under this weather
        for c in causes {
            let n = g.rare_event_tally.get(c).and_then(|m| m.get(wk)).copied().unwrap_or(0.0);
            assert!(n >= 1.0, "{wk}: rare event {c} never fired");
        }
        assert!(g.rare_event_tally["knock"][wk] >= 3.0, "{wk}: re-armed knock should fire again");
        assert!(seen.shift && seen.pattern && seen.skip && seen.knock_tint, "{wk}: rare envelopes {seen:?}");
        assert!(seen.lfo_bent, "{wk}: reality shift never bent the safe-room LFO");
        settle_any |= seen.settle;
        // sanity crossings both ways, the breakdown and the pull-back
        assert!(g.sanity_impaired_tally >= 1.0 && g.sanity_breaking_tally >= 1.0, "{wk}: downward sanity chimes");
        assert!(g.sanity_recovery_tally >= 1.0 && g.sanity_mini_tally >= 1.0, "{wk}: upward sanity chimes");
        assert!(seen.chime_flash && g.chime_flash_color.is_some(), "{wk}: chime flash");
        halluc_any |= seen.halluc;
        halluc_msg_any |= seen.halluc_msg;
        // glitch, flicker wave (timeouts ran and set panel flicker), ghost steps
        assert!(seen.glitch && seen.glitch_msg, "{wk}: reality glitch");
        assert!(g.panels.iter().any(|p| p.flicker > 0.0), "{wk}: flicker wave never reached a panel");
        // the motion sensor clicked; the hum found a live panel
        assert!(g.motion_sensor_last_click_t > 0.0, "{wk}: motion sensor");
        assert!(seen.hum_panel, "{wk}: hum panel");
        // R99 room tone settled on the weather's target
        let rt = game::tables::room_tone_by_weather(wk).unwrap();
        assert!((g.room_tone_cur_gain - rt.gain).abs() < 1e-4 && (g.room_tone_cur_lp - rt.lp).abs() < 1.0, "{wk}: room tone");
        // the sprite pools: sparks / splashes / drain bubbles use the scuffs everywhere
        assert!(seen.scuff, "{wk}: scuff sprites");
        match wk {
            "wet" => assert!(seen.ripple && seen.splash_flash && seen.thunder_flash && seen.rain_hiss, "wet: {seen:?}"),
            "electric" => assert!(seen.surge, "electric: surge"),
            _ => assert!(!seen.ripple && !seen.thunder_flash && !seen.rain_hiss, "{wk}: wet-only effects: {seen:?}"),
        }
        if log_ported {
            let tags: BTreeSet<&String> = g.ambient_tally.keys().collect();
            assert!(tags.len() > 80, "{wk}: only {} ambient tags logged", tags.len());
            assert!(g.ambient_log.len() <= 3);
        }
        // the audio thread survived it all. Minutes of sim squeezed into a
        // second of real time leave thousands of overlapping one-shots, so
        // the render can lag; give the clock a few seconds to move on.
        let ac = g.ac.as_ref().unwrap();
        let nodes = ac.create_gain().id();
        let t0 = ac.current_time();
        let start = std::time::Instant::now();
        while ac.current_time() <= t0 && start.elapsed().as_secs_f64() < 10.0 { std::thread::sleep(std::time::Duration::from_millis(20)); }
        eprintln!("{wk}: {nodes} audio nodes; clock moved after {:.3} s", start.elapsed().as_secs_f64());
        assert!(ac.current_time() > t0, "{wk}: audio clock stopped ({nodes} nodes)");
    }
    assert!(settle_any, "no settle chord in any weather");
    assert!(halluc_any, "no hallucination in any weather");
    assert!(halluc_msg_any, "no hallucination whisper text in any weather");
}

#[test]
fn same_seed_same_draws() {
    // the per-frame draws are a pure function of the seed and the inputs
    let snap = |g: &Game| -> Vec<u64> {
        let mut v: Vec<u64> = prop_timers(g).into_iter().map(|(_, x)| x.to_bits()).collect();
        v.extend([g.distant_knock_timer, g.door_opened_timer, g.elevator_ding_timer, g.voice_sample_timer, g.static_burst_timer,
            g.reality_shift_timer, g.pattern_break_timer, g.clock_skip_timer, g.settle_timer, g.wet_drip_timer, g.glitch_next_t,
            g.flicker_wave_next_t, g.ghost_step_t, g.sanity_whisper_t].map(f64::to_bits));
        v.extend(g.panels.iter().map(|p| p.lfo_phase_offset.to_bits()));
        v
    };
    let mut runs = vec![];
    for _ in 0..2 {
        let mut g = live_level(77, 1);   // wet: the weather with the most draws
        // the module-load draws (ambienceLoop's first delay, the glitch and
        // flicker timers) were native: redo them on the seeded stream
        g.timeouts.clear();
        let mut seen = Seen::default();
        game::rng::with_seed(4242, || { g.init_ambient(); run(&mut g, 3.0, &mut seen) });
        runs.push(snap(&g));
    }
    assert_eq!(runs[0], runs[1]);
}

#[test]
fn bare_game_without_audio_or_render_pools() {
    // updateAmbient needs AC and `playing`; the rest must cope with the bare
    // placeholders (halluc shape without a material, empty pools / lists)
    let (mut g, _) = Game::test_level(5, 21);
    g.state = "playing".into();
    g.init_ambient();
    for k in 0..4000 {
        let dt = if k % 2 == 0 { 1.0 / 60.0 } else { 0.7 };
        g.player.sanity = 100.0 - (k as f64 / 40.0).min(100.0);
        g.player.yaw = k as f64 * 0.01;
        g.now_ms += dt * 1000.0;
        g.run_timeouts();
        g.update_reality_glitch(dt);
        g.update_flicker_wave(dt);
        g.update_ghost_footsteps(dt, k % 3 != 0);
        g.update_sanity_effects(dt);
        g.update_ambient(dt);
        g.update_soundscape(dt);
    }
    // no AC: updateAmbient never ran, so its timers are untouched
    assert_eq!(g.fountain_drip_timer, 0.0);
    assert_eq!(g.distant_knock_timer, 0.0);
    assert!(g.rare_event_tally["knock"].is_empty());
    // the sanity crossings stay silent without AC (no tallies), the glitch still runs
    assert_eq!(g.sanity_impaired_tally, 0.0);
    assert!(g.glitch_next_t > 0.0);
    // the menu: nothing ambient happens outside `playing`
    let mut g2 = live_level(9, 5);
    g2.state = "menu".into();
    g2.update_ambient(0.5);
    assert_eq!(g2.normal_notice_timer, 0.0);
}

#[test]
fn period_lists_and_fallbacks() {
    let mut g = live_level(3, 5);
    // one metronome right at the player: the tick re-arms at half its period
    g.pp.metronome_positions = vec![v3(g.player.pos.x + 0.3, 0.06, g.player.pos.z)];
    g.pp.metronome_periods = vec![1.7];
    g.pp.chair_positions = vec![v3(g.player.pos.x + 0.3, 0.3, g.player.pos.z)];
    g.pp.chair_periods = vec![7.5];
    g.player.sanity = 100.0; g.player.stamina = 100.0;
    g.update_ambient(1.0 / 60.0);
    let mul = 1.0 / g.descent_boost_mul(0.15);
    assert!((g.metronome_tick_timer - 1.7 * 0.5 * mul).abs() < 1e-12, "metronome {}", g.metronome_tick_timer);
    assert!((g.chair_creak_timer - 7.5 * mul).abs() < 1e-12, "chair {}", g.chair_creak_timer);
    // the placers have not pushed periods: the JS defaults' middle
    g.pp.metronome_periods.clear();
    g.pp.chair_periods.clear();
    g.metronome_tick_timer = 0.0; g.chair_creak_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    assert!((g.metronome_tick_timer - game::ambient::METRONOME_PERIOD_DEFAULT * 0.5 * mul).abs() < 1e-12);
    assert!((g.chair_creak_timer - game::ambient::CHAIR_PERIOD_DEFAULT * mul).abs() < 1e-12);
    // low sanity shrinks every cadence (R143): 2/3 at zero sanity
    g.player.sanity = 0.0;
    g.metronome_tick_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    assert!((g.metronome_tick_timer - game::ambient::METRONOME_PERIOD_DEFAULT * 0.5 * mul / 1.5).abs() < 1e-12);
}

#[test]
fn floor_surface_and_zone_fallbacks() {
    // without an archgen layout every floor is carpet; wet weather soaks it
    let mut g = live_level(11, 5);
    assert_eq!(g.floor_surface_at(0.0, 0.0), "carpet");
    g.level_weather = &WEATHER_BIASES[1];
    assert_eq!(g.weather_key(), "wet");
    assert_eq!(g.floor_surface_at(0.0, 0.0), "wetcarpet");
    // no zone: the lobby reverb, no caption
    g.update_soundscape(0.1);
    assert_eq!(g.soundscape().unwrap().zone().as_deref(), Some("lobby"));
    assert!(g.last_zone_arch.is_none());
    // the hum follows the nearest bright panel
    let hp = g.hum_panel.unwrap();
    assert_eq!(hp.p, 0);
    assert!((hp.d - 0.6).abs() < 1e-9);
    // no live panel: the hum goes silent and no panel is held
    for p in g.panels.iter_mut() { p.bright = 0.0; }
    g.hummer_t = 0.0;
    g.update_soundscape(0.1);
    assert!(g.hum_panel.is_none());
}

#[test]
fn reality_glitch_and_ghosts() { game::rng::with_seed(99, reality_glitch_and_ghosts_body); }
fn reality_glitch_and_ghosts_body() {
    let mut g = live_level(21, 3);
    g.glitch_next_t = 0.0;
    g.update_reality_glitch(0.016);
    assert!(g.glitch_t >= 1.6 && g.glitch_t <= 3.2);
    assert!(g.glitch_next_t >= 60.0 && g.glitch_next_t <= 130.0);
    assert!((g.fog_density - game::ambient::FOG_BASE * 3.5).abs() < 1e-12);
    assert_eq!(g.dom.aberration.opacity(), Some(0.85));
    assert!(GLITCH_MSGS.contains(&g.msg_text.as_str()));
    assert_eq!(g.shake_t, 0.6);
    // the glitch ends: the weather's fog comes back, aberration follows sanity
    g.player.sanity = 50.0;
    g.update_reality_glitch(5.0);
    assert!((g.fog_density - game::ambient::FOG_BASE * WEATHER_BIASES[3].fog_mul).abs() < 1e-12);
    assert_eq!(g.dom.aberration.opacity(), Some(0.35));
    // ghost footsteps: standing still resets the timer
    g.ghost_step_t = 5.0;
    g.update_ghost_footsteps(0.1, false);
    assert_eq!(g.ghost_step_t, 0.0);
    // walking: within a few dozen tries some steps get scheduled
    let before = g.timeouts.len();
    for _ in 0..40 { g.ghost_step_t = 0.0; g.update_ghost_footsteps(0.1, true); }
    assert!(g.timeouts.len() > before + 2);
    assert!(g.ghost_step_t >= 22.0 && g.ghost_step_t <= 55.0);
    // the scheduled steps play once their time comes
    g.now_ms += 5000.0;
    g.run_timeouts();
    // the flicker wave staggers one timeout per panel, closest first
    let n = g.timeouts.len();
    g.flicker_wave_next_t = 0.0;
    g.update_flicker_wave(0.1);
    assert_eq!(g.timeouts.len(), n + g.panels.len());
    for p in g.panels.iter_mut() { p.flicker = 0.0; }
    g.now_ms += 1000.0;
    g.run_timeouts();
    assert!(g.panels.iter().all(|p| p.flicker >= 0.35 && p.flicker <= 0.9));
}

#[test]
fn hallucination_lifecycle() { game::rng::with_seed(7, hallucination_lifecycle_body); }
fn hallucination_lifecycle_body() {
    let mut g = live_level(31, 5);
    g.player.sanity = 5.0;
    // 10 s of crit sanity, then whispers every 6–14 s; the shape fades in 0.7 s
    let mut shown = 0;
    let mut was = false;
    for k in 0..6000 {
        g.player.yaw = k as f64 * 0.013;
        g.now_ms += 16.0;
        g.run_timeouts();
        g.update_sanity_effects(1.0 / 60.0);
        let vis = g.halluc_shape.visible();
        if vis && !was { shown += 1; }
        if vis { assert!(g.halluc_shape.mat().opacity_v() <= 0.65 + 1e-9); }
        was = vis;
    }
    assert!(shown >= 1, "no silhouette in 100 s of crit sanity");
    assert!(g.crit_sanity_accum > 90.0);
    // the whisper's aberration spike comes back down after 220 ms
    g.now_ms += 1000.0;
    g.run_timeouts();
    assert_eq!(g.dom.aberration.style("transition"), Some("opacity 0.4s"));
    assert_ne!(g.dom.aberration.style("opacity"), Some("0.98"));
    // recovering hides it at once
    g.halluc_shape.set_visible(true);
    g.player.sanity = 60.0;
    g.update_sanity_effects(0.016);
    assert!(!g.halluc_shape.visible());
    assert_eq!(g.crit_sanity_accum, 0.0);
}

#[test]
fn gated_props_play_only_in_range() {
    // one entry per list: right at the player, or 40 m away. The gated props
    // (watch, chain, CRT, ... most of R481–R767) only make sound in range;
    // count the audio nodes one frame creates.
    let nodes_for = |off: f64| -> u32 {
        let mut g = live_level(55, 5);
        let p = v3(g.player.pos.x + off, 0.3, g.player.pos.z + off * 0.5);
        for list in all_lists!(g.pp) { list.clear(); list.push(p); }
        g.pp.metronome_periods = vec![1.5];
        g.pp.chair_periods = vec![6.0];
        let ac = g.ac.clone().unwrap();
        let before = ac.create_gain().id();
        game::rng::with_seed(1, || g.update_ambient(1.0 / 60.0));
        ac.create_gain().id() - before
    };
    let near = nodes_for(0.3);
    let far = nodes_for(40.0);
    assert!(far > 100, "always-audible props + rare events made only {far} nodes");
    assert!(near > far + 150, "gated props: {near} nodes in range vs {far} out of range");
}
