//! The soundscape driver (`audio::ambient`) on a synthetic level with an
//! offline AudioContext whose graph is rendered as the run goes (a quantum a
//! step): every weather, minutes of simulated time in small and large steps,
//! sanity swept 100 → 0 → 100, every `*_positions` list filled. Asserts
//! nothing panics (the game nor the render of its graph), that the event
//! families fire, and that sounds
//! come from places: prop sounds only in earshot of their nearest prop, the
//! level's signature sounds from fixed anchors, nothing bunched at the level
//! start, the director's hush honoured.

use engine::audio::AudioContext;
use engine::prelude::*;
use game::Game;
use game::audio::ambient::{CADENCES, GLITCH_MSGS, HALLUC_WHISPERS, SIG_APART, prop_falloff, sig_carry};
use game::base::records::Panel;
use game::base::state::WEATHER_BIASES;
use game::base::tables::AMBIENT_EVENTS;
use std::collections::BTreeSet;

/// Every `*_positions` list, as `&mut` (generated from positions.rs).
macro_rules! all_lists {
    ($pp:expr_2021) => {
        [
            &mut $pp.socket_positions,
            &mut $pp.fountain_positions,
            &mut $pp.radiator_positions,
            &mut $pp.glow_light_positions,
            &mut $pp.motion_sensor_positions,
            &mut $pp.speaker_positions,
            &mut $pp.fire_alarm_positions,
            &mut $pp.phone_positions,
            &mut $pp.broken_phone_positions,
            &mut $pp.vending_positions,
            &mut $pp.junction_box_positions,
            &mut $pp.keycard_reader_positions,
            &mut $pp.cardboard_box_positions,
            &mut $pp.living_plant_positions,
            &mut $pp.walkie_talkie_positions,
            &mut $pp.wall_clock_positions,
            &mut $pp.cracked_clock_positions,
            &mut $pp.ceiling_drip_positions,
            &mut $pp.laptop_positions,
            &mut $pp.ceiling_cable_plug_positions,
            &mut $pp.thermostat_positions,
            &mut $pp.handbell_positions,
            &mut $pp.digital_clock_positions,
            &mut $pp.alarm_clock_positions,
            &mut $pp.off_hook_rotary_positions,
            &mut $pp.drain_grate_positions,
            &mut $pp.bathroom_bundle_positions,
            &mut $pp.cauldron_positions,
            &mut $pp.metronome_positions,
            &mut $pp.phonograph_positions,
            &mut $pp.brass_lantern_positions,
            &mut $pp.sewing_positions,
            &mut $pp.pocket_watch_positions,
            &mut $pp.bat_positions,
            &mut $pp.globe_positions,
            &mut $pp.hourglass_positions,
            &mut $pp.ghost_positions,
            &mut $pp.chain_positions,
            &mut $pp.paper_positions,
            &mut $pp.piano_positions,
            &mut $pp.chair_positions,
            &mut $pp.crt_positions,
            &mut $pp.exit_sign_positions,
            &mut $pp.puddle_positions,
            &mut $pp.chandelier_positions,
            &mut $pp.cassette_positions,
            &mut $pp.teacup_positions,
            &mut $pp.locker_positions,
            &mut $pp.bulletin_positions,
            &mut $pp.alarm_pull_positions,
            &mut $pp.pay_phone_positions,
            &mut $pp.radio_positions,
            &mut $pp.keychain_positions,
            &mut $pp.elevator_panel_positions,
            &mut $pp.plant_positions,
            &mut $pp.fan_positions,
            &mut $pp.smoke_detector_positions,
            &mut $pp.coin_positions,
            &mut $pp.clock_positions,
            &mut $pp.cuckoo_clock_positions,
            &mut $pp.plush_positions,
            &mut $pp.gumball_positions,
            &mut $pp.teapot_positions,
            &mut $pp.cookie_tin_positions,
            &mut $pp.kitchen_clock_positions,
            &mut $pp.off_hook_phone_positions,
            &mut $pp.hairdryer_positions,
            &mut $pp.sewing_machine_positions,
            &mut $pp.herb_sieve_positions,
            &mut $pp.vacuum_positions,
            &mut $pp.mop_bucket_positions,
            &mut $pp.wristwatch_positions,
            &mut $pp.hot_iron_positions,
            &mut $pp.grandfather_clock_positions,
            &mut $pp.rocking_chair_positions,
            &mut $pp.tall_lamp_positions,
            &mut $pp.colander_positions,
            &mut $pp.pet_bowl_positions,
            &mut $pp.music_box_positions,
            &mut $pp.turntable_positions,
            &mut $pp.chalkboard_positions,
            &mut $pp.kitchen_calendar_positions,
            &mut $pp.telescope_positions,
            &mut $pp.ornate_cuckoo_clock_positions,
            &mut $pp.lit_fireplace_positions,
            &mut $pp.aquarium_positions,
            &mut $pp.wingback_armchair_positions,
            &mut $pp.hunter_watch_positions,
            &mut $pp.bell_pull_positions,
            &mut $pp.bell_rack_positions,
            &mut $pp.chess_set_positions,
            &mut $pp.hobby_horse_wheeled_positions,
        ]
    };
}

/// The per-weather bed tags; a bed fires only in its own weather.
const BEDS: [(&str, &[&str]); 6] = [
    ("wet", &["wet-drip", "wet-thunder", "wet-splash", "thunder-rumble", "rain-patter"]),
    ("still", &["still-wind", "still-echo", "wind-gust"]),
    ("derelict", &["derelict-clatter", "derelict-grinder"]),
    ("electric", &["electric-crackle", "socket-spark", "junction-arc", "electric-surge"]),
    ("haunted", &["haunted-whisper", "haunted-step"]),
    ("normal", &["normal-chime"]),
];
/// The rare-event tags (any weather)
const RARE: [&str; 8] =
    ["distant-knock", "door-opened", "elevator-ding", "voice-sample", "static-burst", "reality-shift", "pattern-break", "clock-skip"];
/// The placeless outdoor sounds that are gone for good
const WILDLIFE: [&str; 6] = ["bird-chirp", "cricket-chirp", "frog-croak", "owl-hoot", "wolf-howl", "leaf-rustle"];

/// The weather a bed tag belongs to (None: a rare event or a prop sound)
fn bed_weather(tag: &str) -> Option<&'static str> {
    BEDS.iter().find(|(_, tags)| tags.contains(&tag)).map(|(w, _)| *w)
}
/// Whether a cadence's tag is a prop sound
fn is_prop(tag: &str) -> bool {
    bed_weather(tag).is_none() && !RARE.contains(&tag)
}
/// Whether `tag` (or a variant of it, `keycard-beep-ok`) was logged
fn fired(g: &Game, tag: &str) -> bool {
    g.ambient_tally.keys().any(|k| k == tag || k.starts_with(&format!("{tag}-")))
}

/// The graph's render is checked as it goes: finite, every sample
fn render(ac: &AudioContext, frames: usize) {
    let out = ac.render_offline(frames);
    assert!(out.iter().all(|v| v.is_finite()), "the render made a NaN or an infinity");
}

/// The synthetic level with the renderer's pools, live muted audio, panels
/// near the spawn, a rain window, and every position list filled with a few
/// entries (close, mid-range and far from the spawn); armed the way
/// `init_level` arms a level.
fn live_level(seed: u32, weather: usize) -> Game {
    // the per-frame stream seeded too: what a test hears does not depend on
    // the clock or on the tests run before it
    let shift: u64 = std::env::var("AMBIENT_NATIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    game::base::rng::seed_native(u64::from(seed) * 7919 + weather as u64 + shift * 1_000_003);
    let (mut g, _dist) = Game::test_level(seed, 25);
    g.init_render(&Tex::data_rgba(vec![255; 4], 1, 1));
    g.audio_init_with(AudioContext::new_offline(48000.0));
    g.current_seed = seed.wrapping_mul(2654435761);
    g.apply_weather_bias(&WEATHER_BIASES[weather]);
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
        for (dx, dz) in offs {
            list.push(v3(sx + dx, 0.3, sz + dz));
        }
    }
    g.pp.metronome_periods = vec![1.2, 1.4, 1.5, 1.6, 1.8];
    g.pp.chair_periods = vec![5.0, 6.0, 6.5, 7.0, 8.0];
    g.init_ambient();
    g.arm_level_ambient();
    g
}

/// Every cadence due within the first ~45 s, one after another, so a few
/// simulated minutes reach events that come every 10–90 minutes.
fn due_soon(g: &mut Game) {
    for (k, c) in CADENCES.iter().enumerate() {
        *(c.timer)(g) = 0.5 + k as f64 * 0.37;
    }
}

/// What a run saw (envelopes and pools are transient, so sample them every step).
#[derive(Default, Debug)]
struct Seen {
    ripple: bool,
    scuff: bool,
    splash_flash: bool,
    thunder_flash: bool,
    surge: bool,
    settle: bool,
    halluc: bool,
    halluc_msg: bool,
    glitch: bool,
    glitch_msg: bool,
    shift: bool,
    pattern: bool,
    skip: bool,
    knock_tint: bool,
    chime_flash: bool,
    hum_panel: bool,
    rain_hiss: bool,
    lfo_bent: bool,
}

/// Run one weather for `minutes` of simulated time: alternating stretches of
/// 60 fps frames and coarse 0.25–2 s steps, sanity 100 → 0 over the first
/// 70 % then back to 100, the player turning and drifting, the real-time
/// clock advancing with the sim so the `set_timeout` callbacks run.
fn run(g: &mut Game, minutes: f64, seen: &mut Seen) {
    let total = minutes * 60.0;
    let (sx, sz) = (g.player.pos.x, g.player.pos.z);
    let mut t = 0.0;
    let mut k: u64 = 0;
    let mut forced = 0;
    let ac = g.ac.clone().unwrap();
    while t < total {
        // small steps most of the time, coarse ones in between
        let dt = match (k / 300) % 4 {
            0 | 2 => 1.0 / 60.0,
            1 => 0.25,
            _ => [0.5, 1.0, 2.0][(k % 3) as usize],
        };
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
        g.run_scheduled();
        // the main loop's order (frame(): glitch, flicker, ghosts, ..., sanity, ambient, soundscape)
        g.update_reality_glitch(dt);
        g.update_flicker_wave(dt);
        let moving = g.player.bob_amp > 0.02;
        g.update_ghost_footsteps(dt, moving);
        g.update_sanity_effects(dt);
        g.update_ambient(dt);
        g.update_soundscape(dt);
        // the graph rendered as the step asks, at most a 60th of a second
        // of it: the fine steps in real time, minutes of coarse ones over
        // seconds of audio (its one-shots overlapping in hundreds)
        render(&ac, (dt.min(1.0 / 60.0) * 48000.0).round() as usize);
        // the rare events only come back after 4 min … 3 h: re-arm them, a few times
        if forced < 12 && k % 100 == 99 {
            forced += 1;
            g.reality_shift_timer = 0.0;
            g.pattern_break_timer = 0.0;
            g.clock_skip_timer = 0.0;
            g.distant_knock_timer = 0.0;
            g.door_opened_timer = 0.0;
            g.elevator_ding_timer = 0.0;
            g.voice_sample_timer = 0.0;
            g.static_burst_timer = 0.0;
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
        // the sanity chime flash is counted down by `update_entities`' vignette
        // envelope, which does not run here; count it down so later crossings are visible
        if g.chime_flash_t > 0.0 {
            g.chime_flash_t -= dt;
        }
        if g.knock_tint_t > 0.0 {
            g.knock_tint_t = (g.knock_tint_t - dt).max(0.0);
        }
        if g.knock_flash_t > 0.0 {
            g.knock_flash_t -= dt;
        }
        for v in [
            &mut g.door_opened_flash_t,
            &mut g.elevator_ding_flash_t,
            &mut g.voice_sample_flash_t,
            &mut g.static_burst_flash_t,
            &mut g.settle_visual_t,
        ] {
            if *v > 0.0 {
                *v = (*v - dt).max(0.0);
            }
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
        // the level drew its signature sounds
        assert!((3..=4).contains(&g.amb_signatures.len()), "{wk}: {} signatures", g.amb_signatures.len());
        due_soon(&mut g);
        let mut seen = Seen::default();
        game::base::rng::with_seed(0xA11CE + wi as u32, || run(&mut g, 8.0, &mut seen));

        // every prop sound fired (all have a prop in earshot), and every bed of this weather, none of another
        for c in CADENCES {
            match bed_weather(c.tag) {
                Some(w) if w == wk => assert!(fired(&g, c.tag), "{wk}: bed {} never fired", c.tag),
                Some(w) => assert!(!fired(&g, c.tag), "{wk}: {w}'s bed {} fired", c.tag),
                None if is_prop(c.tag) => assert!(fired(&g, c.tag), "{wk}: prop sound {} never fired", c.tag),
                None => {}
            }
        }
        // no birds, crickets, frogs, owls, wolves or leaves in the Backrooms
        for w in WILDLIFE {
            assert!(!fired(&g, w), "{wk}: {w}");
        }
        // the signatures sounded, from their anchors
        let sig_tags: BTreeSet<&String> = g.ambient_tally.keys().filter(|k| k.starts_with("sig-")).collect();
        assert!(!sig_tags.is_empty(), "{wk}: no signature sound in 8 minutes");
        for t in &sig_tags {
            assert!(g.amb_signatures.iter().any(|s| format!("sig-{}", s.name) == **t), "{wk}: {t} is not a signature");
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
        assert!(fired(&g, "ghost-steps"), "{wk}: no ghost steps in 8 minutes of walking");
        // the motion sensor clicked; the hum found a live panel
        assert!(g.motion_sensor_last_click_t > 0.0, "{wk}: motion sensor");
        assert!(seen.hum_panel, "{wk}: hum panel");
        // Room tone settled on the weather's target
        let rt = game::base::tables::room_tone_by_weather(wk).unwrap();
        assert!((g.room_tone_cur_gain - rt.gain).abs() < 1e-4 && (g.room_tone_cur_lp - rt.lp).abs() < 1.0, "{wk}: room tone");
        // the sprite pools: sparks / splashes / drain bubbles use the scuffs everywhere
        assert!(seen.scuff, "{wk}: scuff sprites");
        match wk {
            "wet" => assert!(seen.ripple && seen.splash_flash && seen.thunder_flash && seen.rain_hiss, "wet: {seen:?}"),
            "electric" => assert!(seen.surge, "electric: surge"),
            _ => assert!(!seen.ripple && !seen.thunder_flash && !seen.rain_hiss, "{wk}: wet-only effects: {seen:?}"),
        }
        assert!(g.ambient_tally.len() > 80, "{wk}: only {} ambient tags logged", g.ambient_tally.len());
        assert!(g.ambient_log.len() <= 3);
        // the graph rendered through it all, and renders on: its clock moves
        // as far as it is rendered
        let ac = g.ac.as_ref().unwrap();
        let nodes = ac.create_gain().id();
        let t0 = ac.current_time();
        render(ac, 4800);
        eprintln!("{wk}: {nodes} audio nodes; {t0:.2} s of audio rendered");
        assert!(t0 > 1.0, "{wk}: only {t0} s of audio rendered");
        assert!((ac.current_time() - t0 - 0.1).abs() < 0.003, "{wk}: the clock moved {} s over 0.1 s rendered", ac.current_time() - t0);
    }
    assert!(settle_any, "no settle chord in any weather");
    assert!(halluc_any, "no hallucination in any weather");
    assert!(halluc_msg_any, "no hallucination whisper text in any weather");
}

#[test]
fn same_seed_same_draws() {
    // the per-frame draws are a pure function of the seed and the inputs
    let snap = |g: &mut Game| -> Vec<u64> {
        let mut v: Vec<u64> = CADENCES.iter().map(|c| (c.timer)(g).to_bits()).collect();
        v.extend([g.settle_timer, g.glitch_next_t, g.flicker_wave_next_t, g.ghost_step_t, g.sanity_whisper_t].map(f64::to_bits));
        v.extend(g.amb_signatures.iter().map(|s| s.t.to_bits()));
        v.extend(g.panels.iter().map(|p| p.lfo_phase_offset.to_bits()));
        v
    };
    let mut runs = vec![];
    for _ in 0..2 {
        let mut g = live_level(77, 1); // wet: the weather with the most draws
        // the startup draws (the level's stagger, the glitch and flicker
        // timers) were native: redo them on the seeded stream
        g.timeouts.clear();
        let mut seen = Seen::default();
        game::base::rng::with_seed(4242, || {
            g.init_ambient();
            g.arm_level_ambient();
            run(&mut g, 3.0, &mut seen)
        });
        runs.push(snap(&mut g));
    }
    assert_eq!(runs[0], runs[1]);
}

#[test]
fn bare_game_without_audio_or_render_pools() {
    // `update_ambient` needs an audio context and `playing`; the rest must cope with the bare
    // placeholders (halluc shape without a material, empty pools / lists)
    let (mut g, _) = Game::test_level(5, 21);
    g.state = "playing".into();
    g.init_ambient();
    for k in 0..4000 {
        let dt = if k % 2 == 0 { 1.0 / 60.0 } else { 0.7 };
        g.player.sanity = 100.0 - (k as f64 / 40.0).min(100.0);
        g.player.yaw = k as f64 * 0.01;
        g.now_ms += dt * 1000.0;
        g.run_scheduled();
        g.update_reality_glitch(dt);
        g.update_flicker_wave(dt);
        g.update_ghost_footsteps(dt, k % 3 != 0);
        g.update_sanity_effects(dt);
        g.update_ambient(dt);
        g.update_soundscape(dt);
    }
    // no audio context: `update_ambient` never ran, so its timers are untouched
    assert_eq!(g.fountain_drip_timer, 0.0);
    assert_eq!(g.distant_knock_timer, 0.0);
    assert!(g.rare_event_tally["knock"].is_empty());
    // the sanity crossings stay silent without an audio context (no tallies), the glitch still runs
    assert_eq!(g.sanity_impaired_tally, 0.0);
    assert!(g.glitch_next_t > 0.0);
    // the menu: nothing ambient happens outside `playing`
    let mut g2 = live_level(9, 5);
    g2.state = "menu".into();
    let before = g2.normal_notice_timer;
    g2.update_ambient(0.5);
    assert_eq!(g2.normal_notice_timer, before);
    assert!(g2.ambient_tally.is_empty());
}

#[test]
fn period_lists_and_fallbacks() {
    let mut g = live_level(3, 5);
    // one metronome right at the player: the tick re-arms at half its period
    g.pp.metronome_positions = vec![v3(g.player.pos.x + 0.3, 0.06, g.player.pos.z)];
    g.pp.metronome_periods = vec![1.7];
    g.pp.chair_positions = vec![v3(g.player.pos.x + 0.3, 0.3, g.player.pos.z)];
    g.pp.chair_periods = vec![7.5];
    g.player.sanity = 100.0;
    g.player.stamina = 100.0;
    g.metronome_tick_timer = 0.0;
    g.chair_creak_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    let mul = 1.0 / g.descent_boost_mul(0.15);
    assert!((g.metronome_tick_timer - 1.7 * 0.5 * mul).abs() < 1e-12, "metronome {}", g.metronome_tick_timer);
    assert!((g.chair_creak_timer - 7.5 * mul).abs() < 1e-12, "chair {}", g.chair_creak_timer);
    // the placers have not pushed periods: the defaults (middle of the placer ranges)
    g.pp.metronome_periods.clear();
    g.pp.chair_periods.clear();
    g.metronome_tick_timer = 0.0;
    g.chair_creak_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    assert!((g.metronome_tick_timer - game::audio::ambient::METRONOME_PERIOD_DEFAULT * 0.5 * mul).abs() < 1e-12);
    assert!((g.chair_creak_timer - game::audio::ambient::CHAIR_PERIOD_DEFAULT * mul).abs() < 1e-12);
    // low sanity shrinks every cadence: 2/3 at zero sanity
    g.player.sanity = 0.0;
    g.metronome_tick_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    assert!((g.metronome_tick_timer - game::audio::ambient::METRONOME_PERIOD_DEFAULT * 0.5 * mul / 1.5).abs() < 1e-12);
    // a metronome across the level is not heard: no tick, and it looks again in 1–3 s
    g.player.sanity = 100.0;
    g.pp.metronome_positions = vec![v3(g.player.pos.x + 30.0, 0.06, g.player.pos.z)];
    g.metronome_tick_timer = 0.0;
    g.ambient_tally.clear();
    g.update_ambient(1.0 / 60.0);
    assert!(!fired(&g, "metronome-tick"));
    assert!((1.0..=3.0).contains(&g.metronome_tick_timer), "retry {}", g.metronome_tick_timer);
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
    for p in g.panels.iter_mut() {
        p.bright = 0.0;
    }
    g.hummer_t = 0.0;
    g.update_soundscape(0.1);
    assert!(g.hum_panel.is_none());
}

#[test]
fn reality_glitch_and_ghosts() {
    game::base::rng::with_seed(99, reality_glitch_and_ghosts_body);
}
fn reality_glitch_and_ghosts_body() {
    let mut g = live_level(21, 3);
    g.glitch_next_t = 0.0;
    g.update_reality_glitch(0.016);
    assert!(g.glitch_t >= 1.6 && g.glitch_t <= 3.2);
    assert!(g.glitch_next_t >= 60.0 && g.glitch_next_t <= 130.0);
    assert!((g.fog_density - game::audio::ambient::FOG_BASE * 3.5).abs() < 1e-12);
    assert_eq!(g.ui.aberration.opacity(), Some(0.85));
    assert!(GLITCH_MSGS.contains(&g.msg_text.as_str()));
    assert_eq!(g.shake_t, 0.6);
    // the glitch ends: the weather's fog comes back, aberration follows sanity
    g.player.sanity = 50.0;
    g.update_reality_glitch(5.0);
    assert!((g.fog_density - game::audio::ambient::FOG_BASE * WEATHER_BIASES[3].fog_mul).abs() < 1e-12);
    assert_eq!(g.ui.aberration.opacity(), Some(0.35));
    // ghost footsteps: standing still holds the countdown (no fresh chance on every first step)
    g.ghost_step_t = 5.0;
    g.update_ghost_footsteps(0.1, false);
    assert_eq!(g.ghost_step_t, 5.0);
    g.update_ghost_footsteps(0.1, true);
    assert!((g.ghost_step_t - 4.9).abs() < 1e-12);
    // walking, due: a few steps from behind (yaw 0 faces -z, so behind is south), every time
    g.player.yaw = 0.0;
    for _ in 0..5 {
        let before = g.timeouts.len();
        g.ghost_step_t = 0.0;
        g.update_ghost_footsteps(0.1, true);
        let added = g.timeouts.len() - before;
        assert!((3..=6).contains(&added), "{added} footfalls scheduled");
        assert!(g.ghost_step_t >= 45.0 && g.ghost_step_t <= 110.0);
        let e = g.ambient_log.last().unwrap();
        assert_eq!(e.tag, "ghost-steps");
        let d = e.dist.unwrap();
        assert!((6.0..=12.0).contains(&d), "ghost walk starts {d} m away");
        assert!(["S", "SE", "SW"].contains(&e.bearing.as_deref().unwrap()), "from {:?}", e.bearing);
    }
    // the walk: behind, off to one side, evenly paced toward the player, then it stops
    let p = g.player.pos;
    let path = g.ghost_walk_path(0.4, 9.0, 4);
    assert_eq!(path.len(), 4);
    let fwd = g.player.fwd();
    let dist = |q: &Vec3| ((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt();
    for w in path.windows(2) {
        assert!(dist(&w[1]) < dist(&w[0]));
        assert!((((w[1].x - w[0].x).powi(2) + (w[1].z - w[0].z).powi(2)).sqrt() - 0.55).abs() < 1e-9);
    }
    for q in &path {
        let (dx, dz) = (q.x - p.x, q.z - p.z);
        assert!(dx * fwd.x + dz * fwd.z < 0.0, "a footfall in front of the player");
        assert!(dist(q) > 4.0);
    }
    // the scheduled steps play once their time comes
    g.now_ms += 5000.0;
    g.run_scheduled();
    // the flicker wave staggers one timeout per panel, closest first
    let n = g.timeouts.len();
    g.flicker_wave_next_t = 0.0;
    g.update_flicker_wave(0.1);
    assert_eq!(g.timeouts.len(), n + g.panels.len());
    for p in g.panels.iter_mut() {
        p.flicker = 0.0;
    }
    g.now_ms += 1000.0;
    g.run_scheduled();
    assert!(g.panels.iter().all(|p| p.flicker >= 0.35 && p.flicker <= 0.9));
}

#[test]
fn hallucination_lifecycle() {
    game::base::rng::with_seed(7, hallucination_lifecycle_body);
}
fn hallucination_lifecycle_body() {
    let mut g = live_level(31, 5);
    g.player.sanity = 5.0;
    // 10 s of crit sanity, then whispers every 6–14 s; the shape fades in 0.7 s
    let mut shown = 0;
    let mut was = false;
    for k in 0..6000 {
        g.player.yaw = k as f64 * 0.013;
        g.now_ms += 16.0;
        g.run_scheduled();
        g.update_sanity_effects(1.0 / 60.0);
        let vis = g.halluc_shape.visible();
        if vis && !was {
            shown += 1;
        }
        if vis {
            assert!(g.halluc_shape.mat().opacity_v() <= 0.65 + 1e-9);
        }
        was = vis;
    }
    assert!(shown >= 1, "no silhouette in 100 s of crit sanity");
    assert!(g.crit_sanity_accum > 90.0);
    // the whisper's aberration spike comes back down after 220 ms
    g.now_ms += 1000.0;
    g.run_scheduled();
    assert_eq!(g.ui.aberration.style("transition"), Some("opacity 0.4s"));
    assert_ne!(g.ui.aberration.style("opacity"), Some("0.98"));
    // recovering hides it at once
    g.halluc_shape.set_visible(true);
    g.player.sanity = 60.0;
    g.update_sanity_effects(0.016);
    assert!(!g.halluc_shape.visible());
    assert_eq!(g.crit_sanity_accum, 0.0);
}

#[test]
fn props_play_only_in_earshot() {
    // one entry per list: right at the player, or 40 m away. Every prop sound
    // is gated by its audible radius; count the audio nodes one frame creates.
    let run_frame = |off: f64| -> (u32, Game) {
        let mut g = live_level(55, 5);
        let p = v3(g.player.pos.x + off, 0.3, g.player.pos.z + off * 0.5);
        for list in all_lists!(g.pp) {
            list.clear();
            list.push(p);
        }
        g.pp.metronome_periods = vec![1.5];
        g.pp.chair_periods = vec![6.0];
        for c in CADENCES {
            *(c.timer)(&mut g) = 0.0;
        }
        let ac = g.ac.clone().unwrap();
        let before = ac.create_gain().id();
        game::base::rng::with_seed(1, || g.update_ambient(1.0 / 60.0));
        (ac.create_gain().id() - before, g)
    };
    let (near, gn) = run_frame(0.3);
    let (far, mut gf) = run_frame(40.0);
    // far off, only the weather bed and the rare events sound
    assert!(far > 0, "the normal bed / rare events made no nodes");
    for c in CADENCES.iter().filter(|c| is_prop(c.tag)) {
        assert!(fired(&gn, c.tag), "{} silent right next to its prop", c.tag);
        assert!(!fired(&gf, c.tag), "{} heard from 40 m", c.tag);
        // nothing in earshot: it looks again in 1–3 s, not a whole interval later
        let t = *(c.timer)(&mut gf);
        assert!((1.0..=3.0).contains(&t), "{} re-armed to {t} with nothing in earshot", c.tag);
    }
    assert!(near > far + 150, "props: {near} nodes in range vs {far} out of range");
}

#[test]
fn nearest_prop_level_and_pan() {
    let mut g = live_level(56, 5);
    g.player.yaw = 0.0; // facing -z: +x is to the right
    let (px, pz) = (g.player.pos.x, g.player.pos.z);
    // a far one to the right, the nearest to the left, one out of range
    let list = vec![v3(px + 9.0, 0.3, pz), v3(px - 2.0, 0.3, pz), v3(px + 30.0, 0.3, pz)];
    let h = g.prop_near(&list, 12.0).unwrap();
    assert_eq!(h.i, 1);
    assert!((h.pan + 0.8).abs() < 1e-9, "hard left from the true direction: {}", h.pan);
    assert!((h.dn - 2.0 / 12.0).abs() < 1e-9);
    assert!(g.prop_near(&list[2..], 12.0).is_none());
    // the pan follows the direction, not the distance to the side
    let h = g.prop_near(&[v3(px + 1.5, 0.3, pz - 6.0)], 12.0).unwrap();
    let want = 0.8 * 1.5 / (1.5f64.powi(2) + 36.0).sqrt();
    assert!((h.pan - want).abs() < 1e-9, "pan {} vs {want}", h.pan);
    // played through `update_ambient`: from the nearest fountain, logged with its distance and side
    g.pp.fountain_positions = list;
    g.fountain_drip_timer = 0.0;
    g.update_ambient(1.0 / 60.0);
    let e = g.ambient_log.iter().rev().find(|e| e.tag == "fountain-drip").unwrap();
    assert!((e.dist.unwrap() - 2.0).abs() < 1e-9 && e.pan.unwrap() < 0.0, "{e:?}");
    // a wall in the way halves the level; a clear line keeps it
    let (w, h) = (g.w, g.h);
    let mut walled = None;
    let mut clear = None;
    for z in 1..h - 1 {
        for x in 2..w - 2 {
            let open = |dx: i32| !g.is_wall_tile(x + dx, z);
            if open(-2) && open(-1) && !open(0) && open(1) && open(2) && walled.is_none() {
                walled = Some((x, z));
            }
            if (-2..=2).all(open) && clear.is_none() {
                clear = Some((x, z));
            }
        }
    }
    for ((x, z), occluded) in [(walled.unwrap(), true), (clear.unwrap(), false)] {
        g.player.pos = v3(g.wx(x - 1), 0.0, g.wz(z));
        let prop = v3(g.wx(x + 1), 0.3, g.wz(z));
        let h = g.prop_near(&[prop], 12.0).unwrap();
        let want = prop_falloff(8.0, 12.0) * if occluded { 0.5 } else { 1.0 };
        assert!((h.g - want).abs() < 1e-9, "occluded {occluded}: level {} vs {want}", h.g);
    }
}

#[test]
fn prop_falloff_shape() {
    for r in [2.0, 3.5, 8.0, 12.0, 25.0] {
        assert_eq!(prop_falloff(0.0, r), 1.0);
        assert_eq!(prop_falloff(r, r), 0.0);
        assert_eq!(prop_falloff(r * 3.0, r), 0.0);
        assert!(prop_falloff(r * 0.999, r) > 0.0 && prop_falloff(r * 0.999, r) < 1e-4, "smooth to 0 at the radius");
        let mut last = 1.0;
        for k in 1..=100 {
            let v = prop_falloff(r * k as f64 / 100.0, r);
            assert!(v <= last + 1e-12, "falloff rises at {k}% of {r} m");
            last = v;
        }
        // inverse distance past the reference (a third of the radius, at
        // least 1 m): half way out is well under half
        if r >= 3.0 {
            assert!(prop_falloff(r * 0.5, r) < 0.5, "{r} m: {}", prop_falloff(r * 0.5, r));
        }
    }
}

#[test]
fn level_start_is_staggered() {
    game::base::rng::with_seed(57, level_start_is_staggered_body);
}
fn level_start_is_staggered_body() {
    // every prop in earshot, every cadence armed the way `init_level` arms them
    let mut g = live_level(57, 1);
    g.player.sanity = 100.0;
    g.player.stamina = 100.0;
    let mut first_frame = 0;
    let mut fired_by = vec![];
    for k in 0..120 {
        g.update_ambient(1.0 / 60.0);
        if k == 0 {
            first_frame = g.ambient_tally.len();
        }
        if k == 119 {
            fired_by = g.ambient_tally.keys().cloned().collect::<Vec<_>>();
        }
    }
    assert_eq!(first_frame, 0, "the first frame of a level played {first_frame} ambient sounds");
    // in the first two seconds only the quick, rhythmic props near the player
    for tag in &fired_by {
        let c = CADENCES.iter().find(|c| tag == c.tag || tag.starts_with(&format!("{}-", c.tag)));
        let c = c.unwrap_or_else(|| panic!("{tag} fired in the first 2 s"));
        assert!(is_prop(c.tag) && c.base * 0.35 <= 2.0, "{tag} fired in the first 2 s");
    }
    // each cadence's first countdown is 35–100 % of one of its intervals
    let mut g = live_level(58, 1);
    for c in CADENCES {
        let t = *(c.timer)(&mut g);
        assert!(t >= c.base * 0.35 - 1e-9 && t <= (c.base + c.span) + 1e-9, "{}: {t}", c.tag);
    }
    assert!(g.glitch_next_t >= 30.0 && g.flicker_wave_next_t >= 36.0 && g.ghost_step_t >= 22.5);
}

#[test]
fn cadences_match_their_events() {
    // every event re-arms within the interval `CADENCES` lists for it
    for wi in 0..WEATHER_BIASES.len() {
        let mut g = live_level(60 + wi as u32, wi);
        g.pp.metronome_periods.clear();
        g.pp.chair_periods.clear();
        g.player.sanity = 100.0;
        g.player.stamina = 100.0;
        for c in CADENCES {
            *(c.timer)(&mut g) = 0.0;
        }
        g.update_ambient(1.0 / 60.0);
        let mul = 1.0 / g.descent_boost_mul(0.15);
        let mut checked = 0;
        for c in CADENCES {
            if !fired(&g, c.tag) {
                continue;
            }
            let t = *(c.timer)(&mut g);
            assert!(t >= c.base * mul - 1e-9 && t <= (c.base + c.span) * mul + 1e-9, "{}: re-armed to {t}", c.tag);
            checked += 1;
        }
        assert!(checked >= 90, "{}: only {checked} events fired", WEATHER_BIASES[wi].key);
    }
}

/// (name, anchor, period) of a level's signatures: what must not change
fn identity(g: &Game) -> Vec<(&'static str, [u64; 3], u64)> {
    g.amb_signatures.iter().map(|s| (s.name, [s.at.x.to_bits(), s.at.y.to_bits(), s.at.z.to_bits()], s.period.to_bits())).collect()
}

#[test]
fn signatures_are_stable_per_level() {
    let level = |current_seed: u32, weather: usize| {
        let (mut g, _) = Game::test_level(4242, 31);
        g.current_seed = current_seed;
        g.apply_weather_bias(&WEATHER_BIASES[weather]);
        g
    };
    let names: BTreeSet<&str> = AMBIENT_EVENTS.iter().map(|e| e.1).filter(|n| !n.is_empty()).collect();
    let mut sets = BTreeSet::new();
    for k in 0..8u32 {
        let seed = 0x9E37_79B9u32.wrapping_mul(k + 1);
        let w = k as usize % WEATHER_BIASES.len();
        // the same (run seed, level) gets the same set, whatever the rnd() stream is doing
        let mut a = level(seed, w);
        game::base::rng::with_seed(1, || a.pick_signatures());
        let mut b = level(seed, w);
        game::base::rng::with_seed(2, || {
            game::base::rng::rnd();
            b.pick_signatures()
        });
        assert_eq!(identity(&a), identity(&b));
        // and it draws nothing from the seeded stream
        let mut c = level(seed, w);
        game::base::rng::with_seed(3, || {
            let n0 = game::base::rng::seeded_draws();
            c.pick_signatures();
            // (only the first countdowns, one draw each, may come from rnd())
            assert!(game::base::rng::seeded_draws() - n0 <= c.amb_signatures.len() as u64);
        });
        assert_eq!(identity(&a), identity(&c));
        // 3–4 different sounds of the pool, never the silent entry
        let sigs = &a.amb_signatures;
        assert!((3..=4).contains(&sigs.len()), "{} signatures", sigs.len());
        let uniq: BTreeSet<&str> = sigs.iter().map(|s| s.name).collect();
        assert_eq!(uniq.len(), sigs.len());
        let (px, pz) = (a.player.pos.x, a.player.pos.z);
        for (i, s) in sigs.iter().enumerate() {
            assert!(names.contains(s.name), "{}", s.name);
            assert!(s.period >= 25.0 && s.period <= 70.0 * 1.4 + 1e-9, "period {}", s.period);
            assert!(s.t > 0.0 && s.t <= s.period);
            // anchored to an open spot as far out as the sound carries, apart from the others
            assert!(!a.is_wall_tile(a.tx(s.at.x), a.tz(s.at.z)));
            let d = ((s.at.x - px).powi(2) + (s.at.z - pz).powi(2)).sqrt();
            let c = sig_carry(s.name);
            assert!((c.near..=c.far).contains(&d), "{} anchored {d} m away", s.name);
            for o in &sigs[..i] {
                assert!(((s.at.x - o.at.x).powi(2) + (s.at.z - o.at.z).powi(2)).sqrt() >= SIG_APART);
            }
        }
        sets.insert(identity(&a));
    }
    // other runs / levels get other sets
    assert!(sets.len() >= 7, "only {} distinct signature sets in 8 levels", sets.len());
}

#[test]
fn signatures_play_from_their_anchors() {
    let mut g = live_level(61, 3);
    g.player.sanity = 100.0;
    g.player.stamina = 100.0;
    for s in g.amb_signatures.iter_mut() {
        s.t = 1e6;
    }
    let s0 = g.amb_signatures[0].clone();
    g.amb_signatures[0].t = 0.0;
    g.update_ambient(1.0 / 60.0);
    let tag = format!("sig-{}", s0.name);
    let e = g.ambient_log.iter().rev().find(|e| e.tag == tag).unwrap();
    let d = ((s0.at.x - g.player.pos.x).powi(2) + (s0.at.z - g.player.pos.z).powi(2)).sqrt();
    assert!((e.dist.unwrap() - d).abs() < 1e-9, "logged {e:?}, anchor {d} m away");
    // its own steady beat: the period ±15 %
    let mul = 1.0 / g.descent_boost_mul(0.15);
    let t = g.amb_signatures[0].t;
    assert!(t >= s0.period * 0.85 * mul - 1e-9 && t <= s0.period * 1.15 * mul + 1e-9, "next in {t} s, period {}", s0.period);
    assert_eq!(g.ambient_tally.keys().filter(|k| k.starts_with("sig-")).count(), 1);
    // the anchors stay put for the level
    assert_eq!(g.amb_signatures[0].at, s0.at);
    // hushed, it waits
    g.amb_hush = 1.0;
    g.amb_signatures[1].t = 0.0;
    let n1 = format!("sig-{}", g.amb_signatures[1].name);
    for _ in 0..60 {
        g.update_ambient(1.0 / 60.0);
    }
    assert!(!fired(&g, &n1));
    assert_eq!(g.amb_signatures[1].t, 0.0);
    g.amb_hush = 0.0;
    g.update_ambient(1.0 / 60.0);
    assert!(fired(&g, &n1));
}

#[test]
fn hush_quiets_and_holds() {
    let mut g = live_level(62, 2);
    g.amb_hush = 0.9;
    for c in CADENCES {
        *(c.timer)(&mut g) = 0.0;
    }
    let ac = g.ac.clone().unwrap();
    let before = ac.create_gain().id();
    for _ in 0..300 {
        g.update_ambient(1.0 / 60.0);
    }
    // nothing new starts, and the timers hold (no pile-up when it lifts)
    assert!(g.ambient_tally.is_empty(), "{:?}", g.ambient_tally.keys().collect::<Vec<_>>());
    assert_eq!(ac.create_gain().id() - before, 1);
    for c in CADENCES {
        assert_eq!(*(c.timer)(&mut g), 0.0, "{}", c.tag);
    }
    // the room tone dips by up to half
    let rt = game::base::tables::room_tone_by_weather("electric").unwrap();
    assert!((g.room_tone_cur_gain - rt.gain * (1.0 - 0.5 * 0.9)).abs() < 1e-4, "room tone {}", g.room_tone_cur_gain);
    // a light hush lets things through (quieter)
    g.amb_hush = 0.5;
    g.update_ambient(1.0 / 60.0);
    assert!(g.ambient_tally.len() > 50);
}

#[test]
fn room_tone_brightens_at_low_sanity() {
    // Below 30 % sanity the room tone opens up toward 2.4 kHz — and stays open
    let mut g = live_level(63, 5);
    let rt = game::base::tables::room_tone_by_weather("normal").unwrap();
    let lp = g.audio.room_tone_lp.clone().unwrap();
    g.player.sanity = 0.0;
    for _ in 0..600 {
        g.update_ambient(1.0 / 60.0);
    }
    assert!(lp.frequency().value() > rt.lp + 0.95 * (2400.0 - rt.lp), "{} Hz", lp.frequency().value());
    g.player.sanity = 100.0;
    for _ in 0..900 {
        g.update_ambient(1.0 / 60.0);
    }
    assert!((lp.frequency().value() - rt.lp).abs() < 1.0, "{} Hz", lp.frequency().value());
}

/// The rare events belong to the run: a new level does not start their
/// countdowns over (or they would never come), while every other cadence
/// is armed afresh for the new place.
#[test]
fn rare_events_count_down_across_levels() {
    let mut g = Game::new_bare();
    g.init_ambient();
    g.arm_level_ambient();
    let before: Vec<f64> = CADENCES.iter().map(|c| *(c.timer)(&mut g)).collect();
    assert!(CADENCES.iter().any(|c| c.run), "some cadences are the run's");
    // a level later, part-way through every countdown
    for c in CADENCES {
        *(c.timer)(&mut g) *= 0.5;
    }
    game::world::tables::reset_level_timers(&mut g);
    g.arm_level_ambient();
    for (c, b) in CADENCES.iter().zip(&before) {
        let now = *(c.timer)(&mut g);
        if c.run {
            assert_eq!(now, b * 0.5, "{}: carried over", c.tag);
        } else {
            assert!(now > 0.0 && now <= c.base + c.span, "{}: armed afresh ({now})", c.tag);
        }
    }
}

/// The weather's own sounds come from places: each level puts a draught, a
/// duct, a dying fan and an occupied room somewhere of their own (the same
/// for the same seed), and the wind, the clatter, the grinder, the whispers
/// and steps sound from there; the crackle and surges from the nearest
/// light fitting.
#[test]
fn weather_sounds_come_from_their_places() {
    let source = |tag: &str| match tag {
        "still-wind" | "still-echo" | "wind-gust" => Some("draught"),
        "derelict-clatter" => Some("duct"),
        "derelict-grinder" => Some("fan"),
        "haunted-whisper" | "haunted-step" => Some("room"),
        "electric-crackle" | "electric-surge" => Some("fixture"),
        _ => None,
    };
    for wk in ["still", "derelict", "haunted", "electric"] {
        let wi = WEATHER_BIASES.iter().position(|w| w.key == wk).unwrap();
        let mut g = live_level(2000 + wi as u32, wi);
        assert_eq!(g.amb_anchors.len(), 4, "{wk}: the level's sources");
        let anchors = g.amb_anchors.clone();
        g.pick_bed_anchors();
        assert_eq!(g.amb_anchors, anchors, "{wk}: the same places for the same level");
        due_soon(&mut g);
        let mut heard = 0;
        let mut last = String::new();
        for _ in 0..(6 * 60 * 4) {
            g.update_ambient(0.25);
            let Some(e) = g.ambient_log.last().cloned() else { continue };
            let key = format!("{}@{}", e.tag, e.at);
            if key == last {
                continue;
            }
            last = key;
            let Some(src) = source(&e.tag) else { continue };
            let at = if src == "fixture" { g.nearest_fixture() } else { g.bed_anchor(src) }.expect("a source");
            let d = ((at.x - g.player.pos.x).powi(2) + (at.z - g.player.pos.z).powi(2)).sqrt();
            assert!((e.dist.expect("placed") - d).abs() < 1e-6, "{wk}: {} from {src}", e.tag);
            heard += 1;
        }
        assert!(heard >= 3, "{wk}: only {heard} of the weather's sounds in six minutes");
    }
}
