//! audioInit + every sfx + every soundscape method on a live (muted)
//! AudioContext, then a second of the update loop's parameter pokes. The
//! engine renders on its own thread (the device callback, or a silent clock
//! without a device); a panic there aborts the process or stops the clock,
//! so the clock must keep moving.

use engine::audio::AudioContext;
use game::audio_sys::SFX_NAMES;
use game::soundscape::{ReverbPreset, FOOTSTEP_SURFACES, PHRASES, REVERB_PRESETS};
use game::Game;
use std::time::Duration;

fn live_game() -> Game {
    let mut g = Game::new_bare();
    let ac = AudioContext::new();
    ac.set_muted(true);
    g.audio_init_with(ac);
    g
}
fn sleep(s: f64) { std::thread::sleep(Duration::from_secs_f64(s)); }

#[test]
fn audio_init_builds_the_graph() {
    let g = live_game();
    assert!(g.ac.is_some() && g.master.is_some() && g.noise_buf.is_some());
    let a = &g.audio;
    for (n, v) in [
        ("droneGain", &a.drone_gain), ("phoneGain", &a.phone_gain), ("humGain", &a.hum_gain), ("vacuumGain", &a.vacuum_gain),
        ("safeRoomGain", &a.safe_room_gain), ("safeRoomLFO", &a.safe_room_lfo), ("safeRoomTremCarrier", &a.safe_room_trem_carrier),
        ("staminaBreathGain", &a.stamina_breath_gain), ("staminaBreathLFO", &a.stamina_breath_lfo), ("staminaBreathLp", &a.stamina_breath_lp),
        ("staminaBreathSubGain", &a.stamina_breath_sub_gain), ("staminaBreathPanner", &a.stamina_breath_panner),
        ("staminaBreathPanLFO", &a.stamina_breath_pan_lfo), ("staminaBreathSubOsc", &a.stamina_breath_sub_osc),
        ("roomToneGain", &a.room_tone_gain), ("roomToneLp", &a.room_tone_lp), ("rainHissGain", &a.rain_hiss_gain),
    ] { assert!(v.is_some(), "{n} missing"); }
    assert!(a.soundscape.is_some());
    let ac = g.ac.as_ref().unwrap();
    assert_eq!(g.noise_buf.as_ref().unwrap().length(), ac.sample_rate() as usize);
    // the values the update loop reads back
    assert_eq!(g.master.as_ref().unwrap().gain().value(), 0.9);
    assert_eq!(a.safe_room_lfo.as_ref().unwrap().frequency().value(), 0.35);
    assert_eq!(a.safe_room_trem_carrier.as_ref().unwrap().frequency().value(), 330.0);
    assert_eq!(a.stamina_breath_lfo.as_ref().unwrap().frequency().value(), 0.35);
    assert_eq!(a.stamina_breath_lp.as_ref().unwrap().frequency().value(), 220.0);
    assert_eq!(a.stamina_breath_lp.as_ref().unwrap().q().value(), 0.4);
    assert_eq!(a.stamina_breath_sub_osc.as_ref().unwrap().frequency().value(), 90.0);
    assert_eq!(a.room_tone_lp.as_ref().unwrap().frequency().value(), 900.0);
    assert_eq!(a.drone_gain.as_ref().unwrap().gain().value(), 0.0);
}

#[test]
fn audio_init_twice_keeps_the_context() {
    let mut g = Game::new_bare();
    g.audio_init();
    g.ac.as_ref().unwrap().set_muted(true);
    let id = g.master.as_ref().unwrap().id();
    g.audio_init();
    assert_eq!(g.master.as_ref().unwrap().id(), id);
    g.set_muted(true);
    assert!(g.muted);
    assert_eq!(g.master.as_ref().unwrap().gain().value(), 0.0);
    g.set_muted(false);
    assert_eq!(g.master.as_ref().unwrap().gain().value(), 0.9);
}

#[test]
fn muted_before_init_starts_silent() {
    let mut g = Game::new_bare();
    g.muted = true;
    let ac = AudioContext::new(); ac.set_muted(true);
    g.audio_init_with(ac);
    assert_eq!(g.master.as_ref().unwrap().gain().value(), 0.0);
}

#[test]
fn nosound_skips_the_soundscape() {
    let mut g = Game::new_bare();
    g.audio.nosound = true;
    let ac = AudioContext::new(); ac.set_muted(true);
    g.audio_init_with(ac);
    assert!(g.audio.soundscape.is_none() && g.soundscape().is_none());
    assert!(g.audio.drone_gain.is_some());
    for n in SFX_NAMES { assert!(g.sfx_by_name(n)); }
}

#[test]
fn sfx_without_audio_are_silent_noops() {
    let mut g = Game::new_bare();
    for n in SFX_NAMES { assert!(g.sfx_by_name(n), "{n}"); }
    assert!(!g.sfx_by_name("nope"));
    g.sfx_cry();
    g.blip(440.0, 0.1, "sine", 0.2, None, 0.0, 0.0);
    g.noise_burst(0.1, 440.0, 1.0, 0.2, "bandpass", None, 0.0, 0.0);
    assert!(g.ac.is_none());
}

#[test]
fn every_sound_then_a_second_of_updates() {
    let mut g = live_game();
    let ac = g.ac.clone().unwrap();
    let t_start = ac.current_time();
    // ---- the sfx table, directly and by name
    g.sfx_step(false); g.sfx_step(true); g.sfx_pickup(); g.sfx_door(); g.sfx_scream(); g.sfx_thump(0.65); g.sfx_whisper();
    g.sfx_tick(0.7); g.sfx_thud(); g.sfx_clank(); g.sfx_moan(); g.sfx_door_slam(); g.sfx_distant_scream(); g.sfx_radio_static();
    g.sfx_paper_rustle(); g.sfx_drip(); g.sfx_dial_tone(); g.sfx_distant_footsteps(); g.sfx_glitch(); g.sfx_ghost_step();
    g.sfx_breath(); g.sfx_laugh(); g.sfx_chime(); g.sfx_cry(); g.sfx_guide_chime(); g.sfx_splash(true); g.sfx_splash(false);
    g.sfx_piano(); g.sfx_drip_heavy(); g.sfx_page_flip(); g.sfx_scrape(0.10); g.sfx_scrape(0.3);
    for n in SFX_NAMES { assert!(g.sfx_by_name(n), "{n}"); }
    for n in ["door_slam", "distant_scream", "radio_static", "paper_rustle", "dial_tone", "distant_footsteps", "ghost_step", "guide_chime", "drip_heavy", "page_flip"] { assert!(g.sfx_by_name(n), "{n}"); }
    assert!(!g.sfx_by_name("nope"));
    // blip / noiseBurst with slides, delays and pans
    g.blip(300.0, 0.2, "triangle", 0.1, Some(120.0), 0.05, -0.6);
    g.blip(30.0, 0.5, "sine", 0.01, None, 0.0, 2.0);
    g.noise_burst(0.3, 900.0, 1.2, 0.05, "highpass", Some(10.0), 0.1, 0.4);

    // ---- every soundscape api member
    let s = g.soundscape().unwrap();
    for (name, _) in REVERB_PRESETS { s.set_zone(name, 0.05); assert_eq!(s.zone().as_deref(), Some(name)); }
    s.set_zone("not-a-zone", 1.2); assert_eq!(s.zone().as_deref(), Some("lobby"));
    for k in 0..7 { s.define_zone(&format!("loop{k}"), ReverbPreset::new(0.5 + 0.4 * k as f64, 0.8 - 0.06 * k as f64, 0.1 + 0.045 * k as f64, 0.004 + 0.006 * k as f64, 6 - (k >> 1))); }
    s.set_zone("loop0", 1.2); s.set_zone("loop6", 0.3);   // mid-fade switch
    assert_eq!(s.zone().as_deref(), Some("loop6"));
    s.set_zone("stairwell", 0.4);
    s.set_listener(0.0, 1.6, 0.0, 0.0, 0.0, -1.0);
    let p = s.make_panner(2.0, 1.2, 60.0); p.connect(&s.world);
    s.place_panner(&p, 1.0, 2.0, 3.0, 0.05);
    let slot = s.voice_slot(2.2);
    slot.at(3.0, 1.9, -2.0); slot.occlude(1.0); slot.occlude(0.0);
    for surf in FOOTSTEP_SURFACES { s.footstep(surf, 0.62, None); s.footstep(surf, 1.0, Some(&slot.input)); }
    s.footstep("mud", 0.3, None);
    s.screech(1.0, 0.9); s.screech(0.4, 0.4);
    s.heartbeat(80.0, 4, 0.0, None); s.heartbeat(96.0, 4, 1.0, Some(&s.world));
    s.knock(5, 0.0, 6.0, -1.5, 0.35);
    let t = ac.current_time();
    // entityCall / updateEntityVoice shapes
    s.burst(&slot.input, t, 0.5, "bandpass", 900.0, 2.0, 0.2, Some(2400.0), 0.01); s.tone(&slot.input, t, 0.4, 180.0, 0.08, "sawtooth", Some(90.0), 0.003);
    s.tone(&slot.input, t, 2.4, 52.0, 0.14, "sawtooth", Some(44.0), 0.2); s.burst(&slot.input, t, 2.4, "lowpass", 320.0, 1.5, 0.09, Some(220.0), 0.2);
    s.tone(&s.dry, t, 2.2, 5200.0, 0.006 * 0.5, "sine", None, 0.6);
    s.tone(&s.world, t, 0.1, 300.0, 0.0, "square", None, 0.003);   // a zero gain must not break anything
    let voice = s.create_child_voice(Some(&slot.input));
    voice.set_level(0.55, 2.5);
    let mut t0 = ac.current_time() + 0.05;
    for (ph, _) in PHRASES { let d = voice.say(ph, Some(t0), 1.0, 1.0); assert!(d > 0.0); t0 += d; }
    assert!(voice.busy());
    let far = s.create_child_voice(None); far.set_level(0.14, 0.3); far.say("sob", None, 1.0, 0.6);
    let horn = ac.create_biquad_filter(); horn.connect(&s.world);
    let pa = s.create_child_voice(Some(&horn)); pa.say("please", None, 0.42, 1.0); pa.stop();
    s.hum.detune(-30.0, 0.6); assert_eq!(s.hum.cents(), -30.0);
    s.hum.detune(0.0, 0.1);
    let _ = s.hum.spiking();

    // ---- one second of the update loop
    let a = &g.audio;
    for frame in 0..60 {
        let dt = 1.0 / 60.0;
        let now = ac.current_time();
        let k = (dt * 3.0f64).min(1.0);
        for (node, target) in [(&a.drone_gain, 0.3), (&a.vacuum_gain, 0.2), (&a.rain_hiss_gain, 0.05), (&a.safe_room_gain, 0.4), (&a.phone_gain, 0.1), (&a.room_tone_gain, 0.02)] {
            let gp = node.as_ref().unwrap().gain();
            gp.set_value(gp.value() + (target - gp.value()) * k);
        }
        a.safe_room_lfo.as_ref().unwrap().frequency().set_value(0.35 - 0.1 * (frame as f64 / 60.0));
        a.safe_room_trem_carrier.as_ref().unwrap().frequency().set_value(330.0 - 15.0 * (frame as f64 / 60.0));
        a.room_tone_lp.as_ref().unwrap().frequency().set_value(900.0 + frame as f64 * 10.0);
        a.stamina_breath_gain.as_ref().unwrap().gain().set_target_at_time(0.05, now, 0.3);
        a.stamina_breath_lfo.as_ref().unwrap().frequency().set_target_at_time(0.5, now, 0.5);
        a.stamina_breath_lp.as_ref().unwrap().frequency().set_target_at_time(400.0, now, 0.5);
        a.stamina_breath_lp.as_ref().unwrap().q().set_target_at_time(0.8, now, 0.5);
        a.stamina_breath_sub_gain.as_ref().unwrap().gain().set_target_at_time(0.02, now, 0.3);
        a.stamina_breath_sub_osc.as_ref().unwrap().frequency().set_target_at_time(80.0, now, 0.5);
        a.stamina_breath_panner.as_ref().unwrap().pan().set_target_at_time(0.2 * (frame as f64 * 0.2).sin(), now, 0.3);
        s.set_listener(frame as f64 * 0.05, 1.6, 0.0, (frame as f64 * 0.1).sin(), 0.0, -(frame as f64 * 0.1).cos());
        if frame % 20 == 10 { s.hum.silence(); } else { s.hum.update(dt, 2.0, 2.9, -1.0, 0.8); }
        slot.at(3.0 - frame as f64 * 0.05, 1.9, -2.0);
        slot.occlude(if frame % 30 < 15 { 0.0 } else { 1.0 });
        s.set_zone(if frame < 30 { "hall" } else { "office" }, 1.2);
        if frame % 15 == 0 { g.sfx_step(frame % 30 == 0); s.footstep("carpet", 0.62, None); }
        if frame == 40 {
            // the stamina finale (line 68887): cancel, set, ramp
            let t0 = ac.current_time();
            let bg = a.stamina_breath_gain.as_ref().unwrap().gain();
            bg.cancel_scheduled_values(t0); bg.set_value_at_time(0.08, t0); bg.exponential_ramp_to_value_at_time(0.0001, t0 + 1.2);
            let bf = a.stamina_breath_lfo.as_ref().unwrap().frequency();
            bf.cancel_scheduled_values(t0); bf.set_value_at_time(0.9, t0); bf.linear_ramp_to_value_at_time(0.3, t0 + 1.2);
        }
        sleep(dt);
    }
    voice.stop(); far.stop();
    sleep(0.2);
    let elapsed = ac.current_time() - t_start;
    assert!(elapsed > 0.5, "audio clock stalled: {elapsed} s in ~1.2 s");
}
