//! Per-weather tuning tables. Each `foo_by_weather(key)` returns the value
//! for a weather key, or `None` where the table has no entry for it, so every
//! caller keeps its own fallback (the `normal` entry, `1.0`, ...).
#![allow(clippy::excessive_precision, clippy::approx_constant)]

/// Pitches (Hz) of the two-note low-battery hint, high then low.
pub fn battery_hint_dips(key: &str) -> Option<&'static [f64]> {
    Some(match key {
        "wet" => &[420.0, 398.0],
        "electric" => &[440.0, 415.0],
        "haunted" => &[400.0, 375.0],
        "derelict" => &[430.0, 405.0],
        "still" => &[430.0, 410.0],
        "normal" => &[420.0, 392.0],
        _ => return None,
    })
}

/// Safe-room hum LFO rate multiplier.
pub fn lfo_weather_mul(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.75,
        "still" => 1.3,
        _ => return None,
    })
}

/// Safe-room tremolo carrier offset (Hz).
pub fn carrier_weather_detune(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => -10.0,
        "still" => 5.0,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomToneSpec {
    pub lp: f64,
    pub gain: f64,
}
/// Room tone low-pass cutoff (Hz) and gain.
pub fn room_tone_by_weather(key: &str) -> Option<RoomToneSpec> {
    Some(match key {
        "wet" => RoomToneSpec { lp: 600.0, gain: 0.014 },
        "electric" => RoomToneSpec { lp: 1400.0, gain: 0.008 },
        "haunted" => RoomToneSpec { lp: 750.0, gain: 0.012 },
        "derelict" => RoomToneSpec { lp: 850.0, gain: 0.011 },
        "still" => RoomToneSpec { lp: 1100.0, gain: 0.008 },
        "normal" => RoomToneSpec { lp: 900.0, gain: 0.01 },
        _ => return None,
    })
}

/// Vignette pulse amplitude (default `VIGNETTE_PULSE_AMP_DEFAULT`).
pub fn vignette_pulse_amp_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.022,
        "still" => 0.01,
        _ => return None,
    })
}

/// Amplitude of the panels' brightness breathing.
pub fn panel_lfo_amp_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.06,
        "electric" => 0.05,
        "haunted" => 0.05,
        "derelict" => 0.04,
        "still" => 0.02,
        "normal" => 0.04,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IdleBreathSpec {
    pub amp: f64,
    pub freq: f64,
}
/// Idle camera breathing amplitude and frequency.
pub fn idle_breath_by_weather(key: &str) -> Option<IdleBreathSpec> {
    Some(match key {
        "wet" => IdleBreathSpec { amp: 0.008, freq: 0.7 },
        "electric" => IdleBreathSpec { amp: 0.007, freq: 1.1 },
        "haunted" => IdleBreathSpec { amp: 0.007, freq: 0.85 },
        "derelict" => IdleBreathSpec { amp: 0.006, freq: 0.9 },
        "still" => IdleBreathSpec { amp: 0.005, freq: 1.15 },
        "normal" => IdleBreathSpec { amp: 0.006, freq: 0.9 },
        _ => return None,
    })
}

/// Camera tilt magnitude while crouched.
pub fn crouch_tilt_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.075,
        "electric" => 0.06,
        "haunted" => 0.055,
        "derelict" => 0.065,
        "still" => 0.045,
        "normal" => 0.06,
        _ => return None,
    })
}

/// Crouched camera wobble amplitude at low stamina.
pub fn stamina_wobble_amp_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.011,
        "electric" => 0.009,
        "haunted" => 0.008,
        "derelict" => 0.01,
        "still" => 0.006,
        "normal" => 0.008,
        _ => return None,
    })
}

/// Crouched camera wobble frequency at low stamina.
pub fn stamina_wobble_freq_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 4.4,
        "electric" => 5.6,
        "haunted" => 5.2,
        "derelict" => 4.8,
        "still" => 6.0,
        "normal" => 5.2,
        _ => return None,
    })
}

/// Bent-over camera dip at low stamina while standing.
pub fn stamina_dip_mag_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.05,
        "electric" => 0.042,
        "haunted" => 0.038,
        "derelict" => 0.045,
        "still" => 0.032,
        "normal" => 0.04,
        _ => return None,
    })
}

/// Vignette peak while crying.
pub fn cry_vignette_peak_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.2,
        "electric" => 0.14,
        "haunted" => 0.17,
        "derelict" => 0.18,
        "still" => 0.1,
        "normal" => 0.15,
        _ => return None,
    })
}

/// Player light dimming peak while crying.
pub fn cry_beam_peak_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.72,
        "electric" => 0.52,
        "haunted" => 0.62,
        "derelict" => 0.66,
        "still" => 0.5,
        "normal" => 0.6,
        _ => return None,
    })
}

/// Panel dimming peak while crying.
pub fn cry_panel_peak_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.85,
        "electric" => 0.7,
        "haunted" => 0.8,
        "derelict" => 0.82,
        "still" => 0.68,
        "normal" => 0.78,
        _ => return None,
    })
}

/// Crouched footstep gain.
pub fn crouch_step_gain_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.024,
        "electric" => 0.014,
        "haunted" => 0.018,
        "derelict" => 0.02,
        "still" => 0.01,
        "normal" => 0.016,
        _ => return None,
    })
}

/// Crouched footstep filter frequency (Hz).
pub fn crouch_step_freq_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 180.0,
        "electric" => 130.0,
        "haunted" => 140.0,
        "derelict" => 150.0,
        "still" => 110.0,
        "normal" => 140.0,
        _ => return None,
    })
}

/// Crouched footstep length (s).
pub fn crouch_step_len_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 3.6,
        "electric" => 4.0,
        "haunted" => 4.2,
        "derelict" => 3.8,
        "still" => 4.6,
        "normal" => 4.0,
        _ => return None,
    })
}

/// Radius of the panel dimming.
pub fn panel_dim_radius_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 5.0,
        "electric" => 3.5,
        "haunted" => 4.4,
        "derelict" => 4.2,
        "still" => 3.2,
        "normal" => 4.0,
        _ => return None,
    })
}

/// Floor (lowest scale) of the panel dimming.
pub fn panel_dim_floor_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.88,
        "electric" => 0.94,
        "haunted" => 0.9,
        "derelict" => 0.91,
        "still" => 0.95,
        "normal" => 0.92,
        _ => return None,
    })
}

/// Falloff of the panel dimming.
pub fn panel_dim_falloff_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 1.4,
        "electric" => 2.4,
        "haunted" => 1.8,
        "derelict" => 1.7,
        "still" => 2.6,
        "normal" => 2.0,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnockTimbreSpec {
    pub freq: f64,
    pub noise: f64,
    pub gain_mul: f64,
}
/// Timbre of the rare knock event.
pub fn knock_timbre_by_weather(key: &str) -> Option<KnockTimbreSpec> {
    Some(match key {
        "wet" => KnockTimbreSpec { freq: 65.0, noise: 120.0, gain_mul: 1.1 },
        "electric" => KnockTimbreSpec { freq: 95.0, noise: 220.0, gain_mul: 0.9 },
        "haunted" => KnockTimbreSpec { freq: 70.0, noise: 130.0, gain_mul: 1.05 },
        "derelict" => KnockTimbreSpec { freq: 78.0, noise: 150.0, gain_mul: 1.0 },
        "still" => KnockTimbreSpec { freq: 85.0, noise: 160.0, gain_mul: 0.85 },
        "normal" => KnockTimbreSpec { freq: 75.0, noise: 140.0, gain_mul: 1.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoorTimbreSpec {
    pub creak_start: f64,
    pub creak_end: f64,
    pub whoosh: f64,
    pub sub_end: f64,
    pub gain_mul: f64,
}
/// Timbre of the rare door-opening event.
pub fn door_timbre_by_weather(key: &str) -> Option<DoorTimbreSpec> {
    Some(match key {
        "wet" => DoorTimbreSpec { creak_start: 110.0, creak_end: 190.0, whoosh: 260.0, sub_end: 40.0, gain_mul: 1.1 },
        "electric" => DoorTimbreSpec { creak_start: 150.0, creak_end: 260.0, whoosh: 400.0, sub_end: 55.0, gain_mul: 0.9 },
        "haunted" => DoorTimbreSpec { creak_start: 120.0, creak_end: 200.0, whoosh: 300.0, sub_end: 42.0, gain_mul: 1.05 },
        "derelict" => DoorTimbreSpec { creak_start: 135.0, creak_end: 225.0, whoosh: 340.0, sub_end: 48.0, gain_mul: 1.0 },
        "still" => DoorTimbreSpec { creak_start: 155.0, creak_end: 240.0, whoosh: 380.0, sub_end: 52.0, gain_mul: 0.8 },
        "normal" => DoorTimbreSpec { creak_start: 130.0, creak_end: 220.0, whoosh: 320.0, sub_end: 45.0, gain_mul: 1.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DingTimbreSpec {
    pub high: f64,
    pub low: f64,
    pub rumble: f64,
    pub gain_mul: f64,
}
/// Timbre of the rare elevator-ding event.
pub fn ding_timbre_by_weather(key: &str) -> Option<DingTimbreSpec> {
    Some(match key {
        "wet" => DingTimbreSpec { high: 554.0, low: 494.0, rumble: 70.0, gain_mul: 1.05 },
        "electric" => DingTimbreSpec { high: 622.0, low: 554.0, rumble: 90.0, gain_mul: 0.95 },
        "haunted" => DingTimbreSpec { high: 570.0, low: 508.0, rumble: 78.0, gain_mul: 1.1 },
        "derelict" => DingTimbreSpec { high: 580.0, low: 517.0, rumble: 82.0, gain_mul: 1.0 },
        "still" => DingTimbreSpec { high: 622.0, low: 554.0, rumble: 88.0, gain_mul: 0.85 },
        "normal" => DingTimbreSpec { high: 587.0, low: 523.0, rumble: 80.0, gain_mul: 1.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceTimbreSpec {
    pub f_start: f64,
    pub f_end: f64,
    pub breath_lp: f64,
    pub gain_mul: f64,
}
/// Timbre of the rare voice event.
pub fn voice_timbre_by_weather(key: &str) -> Option<VoiceTimbreSpec> {
    Some(match key {
        "wet" => VoiceTimbreSpec { f_start: 780.0, f_end: 340.0, breath_lp: 700.0, gain_mul: 1.1 },
        "electric" => VoiceTimbreSpec { f_start: 1020.0, f_end: 460.0, breath_lp: 900.0, gain_mul: 0.9 },
        "haunted" => VoiceTimbreSpec { f_start: 900.0, f_end: 380.0, breath_lp: 800.0, gain_mul: 1.15 },
        "derelict" => VoiceTimbreSpec { f_start: 860.0, f_end: 400.0, breath_lp: 780.0, gain_mul: 1.0 },
        "still" => VoiceTimbreSpec { f_start: 940.0, f_end: 440.0, breath_lp: 850.0, gain_mul: 0.85 },
        "normal" => VoiceTimbreSpec { f_start: 900.0, f_end: 400.0, breath_lp: 800.0, gain_mul: 1.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticTimbreSpec {
    pub hp_freq: f64,
    pub tail_freq: f64,
    pub clicks: f64,
    pub click_base: f64,
    pub gain_mul: f64,
}
/// Timbre of the rare static-burst event.
pub fn static_timbre_by_weather(key: &str) -> Option<StaticTimbreSpec> {
    Some(match key {
        "wet" => StaticTimbreSpec { hp_freq: 3200.0, tail_freq: 1300.0, clicks: 3.0, click_base: 420.0, gain_mul: 1.1 },
        "electric" => StaticTimbreSpec { hp_freq: 4400.0, tail_freq: 1900.0, clicks: 4.0, click_base: 560.0, gain_mul: 0.95 },
        "haunted" => StaticTimbreSpec { hp_freq: 3600.0, tail_freq: 1450.0, clicks: 3.0, click_base: 460.0, gain_mul: 1.15 },
        "derelict" => StaticTimbreSpec { hp_freq: 3900.0, tail_freq: 1550.0, clicks: 3.0, click_base: 500.0, gain_mul: 1.05 },
        "still" => StaticTimbreSpec { hp_freq: 4200.0, tail_freq: 1750.0, clicks: 2.0, click_base: 540.0, gain_mul: 0.85 },
        "normal" => StaticTimbreSpec { hp_freq: 3800.0, tail_freq: 1600.0, clicks: 3.0, click_base: 500.0, gain_mul: 1.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceHueSpec {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
/// Screen tint of the voice event.
pub fn voice_hue_by_weather(key: &str) -> Option<VoiceHueSpec> {
    Some(match key {
        "wet" => VoiceHueSpec { r: 0.55, g: 0.85, b: 1.0 },
        "electric" => VoiceHueSpec { r: 0.7, g: 1.0, b: 0.75 },
        "haunted" => VoiceHueSpec { r: 0.75, g: 0.75, b: 1.0 },
        "derelict" => VoiceHueSpec { r: 0.85, g: 0.9, b: 0.75 },
        "still" => VoiceHueSpec { r: 0.9, g: 0.95, b: 0.95 },
        "normal" => VoiceHueSpec { r: 0.65, g: 1.0, b: 0.9 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnockTintSpec {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
/// Screen tint of the knock event.
pub fn knock_tint_by_weather(key: &str) -> Option<KnockTintSpec> {
    Some(match key {
        "wet" => KnockTintSpec { r: 1.0, g: 0.7, b: 0.35 },
        "electric" => KnockTintSpec { r: 1.0, g: 0.9, b: 0.2 },
        "haunted" => KnockTintSpec { r: 1.0, g: 0.5, b: 0.3 },
        "derelict" => KnockTintSpec { r: 1.0, g: 0.6, b: 0.28 },
        "still" => KnockTintSpec { r: 1.0, g: 0.75, b: 0.55 },
        "normal" => KnockTintSpec { r: 1.0, g: 0.565, b: 0.251 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiftBendSpec {
    pub bend: f64,
    pub fov: f64,
}
/// Pitch bend and FOV change of the reality-shift event.
pub fn shift_bend_by_weather(key: &str) -> Option<ShiftBendSpec> {
    Some(match key {
        "wet" => ShiftBendSpec { bend: 0.12, fov: 4.0 },
        "electric" => ShiftBendSpec { bend: 0.06, fov: 2.5 },
        "haunted" => ShiftBendSpec { bend: 0.1, fov: 3.5 },
        "derelict" => ShiftBendSpec { bend: 0.09, fov: 3.2 },
        "still" => ShiftBendSpec { bend: 0.04, fov: 1.5 },
        "normal" => ShiftBendSpec { bend: 0.08, fov: 3.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SettleChordSpec {
    pub notes: &'static [f64],
    pub prob: f64,
}
/// Chord notes (Hz) and probability of the settle event.
pub fn settle_chord_by_weather(key: &str) -> Option<SettleChordSpec> {
    Some(match key {
        "wet" => SettleChordSpec { notes: &[440.0, 349.0, 262.0], prob: 0.2 },
        "electric" => SettleChordSpec { notes: &[392.0, 311.0, 247.0], prob: 0.35 },
        "haunted" => SettleChordSpec { notes: &[415.0, 349.0, 277.0], prob: 0.3 },
        "derelict" => SettleChordSpec { notes: &[370.0, 311.0, 233.0], prob: 0.3 },
        "still" => SettleChordSpec { notes: &[440.0, 370.0, 294.0], prob: 0.5 },
        "normal" => SettleChordSpec { notes: &[392.0, 330.0, 262.0], prob: 0.3 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternSyncSpec {
    pub dur: f64,
    pub sub_freq: f64,
    pub sub_gain: f64,
}
/// Duration and sub-bass of the pattern-break event.
pub fn pattern_sync_by_weather(key: &str) -> Option<PatternSyncSpec> {
    Some(match key {
        "wet" => PatternSyncSpec { dur: 5.0, sub_freq: 50.0, sub_gain: 0.02 },
        "electric" => PatternSyncSpec { dur: 3.5, sub_freq: 75.0, sub_gain: 0.014 },
        "haunted" => PatternSyncSpec { dur: 4.5, sub_freq: 58.0, sub_gain: 0.018 },
        "derelict" => PatternSyncSpec { dur: 4.2, sub_freq: 62.0, sub_gain: 0.016 },
        "still" => PatternSyncSpec { dur: 3.0, sub_freq: 80.0, sub_gain: 0.012 },
        "normal" => PatternSyncSpec { dur: 4.0, sub_freq: 65.0, sub_gain: 0.015 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticChromSpec {
    pub contrast: f64,
    pub saturate: f64,
}
/// Contrast and saturation of the static-burst chromatic effect.
pub fn static_chrom_by_weather(key: &str) -> Option<StaticChromSpec> {
    Some(match key {
        "wet" => StaticChromSpec { contrast: 0.2, saturate: 0.45 },
        "electric" => StaticChromSpec { contrast: 0.45, saturate: 0.8 },
        "haunted" => StaticChromSpec { contrast: 0.32, saturate: 0.65 },
        "derelict" => StaticChromSpec { contrast: 0.28, saturate: 0.55 },
        "still" => StaticChromSpec { contrast: 0.15, saturate: 0.35 },
        "normal" => StaticChromSpec { contrast: 0.3, saturate: 0.6 },
        _ => return None,
    })
}

/// Light dimming of the door event.
pub fn door_dim_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.2,
        "electric" => 0.12,
        "haunted" => 0.17,
        "derelict" => 0.16,
        "still" => 0.1,
        "normal" => 0.15,
        _ => return None,
    })
}

/// Light brightening of the ding event.
pub fn ding_bright_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.08,
        "electric" => 0.15,
        "haunted" => 0.11,
        "derelict" => 0.1,
        "still" => 0.06,
        "normal" => 0.1,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaminaBreathSpec {
    pub gain: f64,
    pub rate_base: f64,
    pub rate_gain: f64,
    pub sub_gain: f64,
    pub sub_freq: f64,
    pub pan_depth: f64,
    pub pan_rate: f64,
}
/// Exhausted-breathing sound parameters.
pub fn stamina_breath_by_weather(key: &str) -> Option<StaminaBreathSpec> {
    Some(match key {
        "wet" => StaminaBreathSpec {
            gain: 0.055,
            rate_base: 0.3,
            rate_gain: 0.2,
            sub_gain: 0.026,
            sub_freq: 75.0,
            pan_depth: 0.25,
            pan_rate: 0.6,
        },
        "electric" => StaminaBreathSpec {
            gain: 0.04,
            rate_base: 0.4,
            rate_gain: 0.2,
            sub_gain: 0.016,
            sub_freq: 100.0,
            pan_depth: 0.15,
            pan_rate: 0.85,
        },
        "haunted" => StaminaBreathSpec {
            gain: 0.05,
            rate_base: 0.32,
            rate_gain: 0.22,
            sub_gain: 0.022,
            sub_freq: 85.0,
            pan_depth: 0.22,
            pan_rate: 0.7,
        },
        "derelict" => StaminaBreathSpec {
            gain: 0.048,
            rate_base: 0.34,
            rate_gain: 0.2,
            sub_gain: 0.021,
            sub_freq: 88.0,
            pan_depth: 0.2,
            pan_rate: 0.72,
        },
        "still" => StaminaBreathSpec {
            gain: 0.035,
            rate_base: 0.42,
            rate_gain: 0.25,
            sub_gain: 0.014,
            sub_freq: 105.0,
            pan_depth: 0.1,
            pan_rate: 0.95,
        },
        "normal" => StaminaBreathSpec {
            gain: 0.045,
            rate_base: 0.35,
            rate_gain: 0.2,
            sub_gain: 0.02,
            sub_freq: 90.0,
            pan_depth: 0.2,
            pan_rate: 0.75,
        },
        _ => return None,
    })
}

/// Per-level pitch drop of the descent (see `level_depth_mul`).
pub fn level_depth_slope_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.007,
        "electric" => 0.005,
        "haunted" => 0.006,
        "derelict" => 0.005,
        "still" => 0.003,
        "normal" => 0.005,
        _ => return None,
    })
}

/// Camera roll amplitude at low sanity.
pub fn sanity_roll_amp_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.026,
        "electric" => 0.028,
        "haunted" => 0.03,
        "derelict" => 0.022,
        "still" => 0.017,
        "normal" => 0.02,
        _ => return None,
    })
}

/// Head-bob amplitude multiplier.
pub fn bob_amp_mul_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 1.15,
        "electric" => 1.05,
        "haunted" => 0.95,
        "derelict" => 1.0,
        "still" => 0.9,
        "normal" => 1.0,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SanityHueSpec {
    pub spread: f64,
    pub bias: f64,
}
/// Hue spread and bias of the low-sanity colour shift.
pub fn sanity_hue_by_weather(key: &str) -> Option<SanityHueSpec> {
    Some(match key {
        "wet" => SanityHueSpec { spread: 40.0, bias: -60.0 },
        "electric" => SanityHueSpec { spread: 80.0, bias: -20.0 },
        "haunted" => SanityHueSpec { spread: 50.0, bias: -90.0 },
        "derelict" => SanityHueSpec { spread: 45.0, bias: 20.0 },
        "still" => SanityHueSpec { spread: 30.0, bias: -40.0 },
        "normal" => SanityHueSpec { spread: 60.0, bias: -30.0 },
        _ => return None,
    })
}

/// Period (ms) of the low-sanity dread pulse.
pub fn sanity_dread_period_ms_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 1100.0,
        "electric" => 700.0,
        "haunted" => 1300.0,
        "derelict" => 900.0,
        "still" => 600.0,
        "normal" => 900.0,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakingMelodySpec {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}
/// Notes (Hz) of the sanity-breaking cue.
pub fn breaking_melody_by_weather(key: &str) -> Option<BreakingMelodySpec> {
    Some(match key {
        "wet" => BreakingMelodySpec { a: 220.0, b: 175.0, c: 147.0 },
        "electric" => BreakingMelodySpec { a: 330.0, b: 247.0, c: 165.0 },
        "haunted" => BreakingMelodySpec { a: 208.0, b: 147.0, c: 104.0 },
        "derelict" => BreakingMelodySpec { a: 247.0, b: 208.0, c: 165.0 },
        "still" => BreakingMelodySpec { a: 262.0, b: 220.0, c: 175.0 },
        "normal" => BreakingMelodySpec { a: 262.0, b: 196.0, c: 131.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpairedMelodySpec {
    pub root: f64,
    pub down: f64,
    pub bottom: f64,
}
/// Notes (Hz) of the sanity-impaired cue.
pub fn impaired_melody_by_weather(key: &str) -> Option<ImpairedMelodySpec> {
    Some(match key {
        "wet" => ImpairedMelodySpec { root: 440.0, down: 349.0, bottom: 293.0 },
        "electric" => ImpairedMelodySpec { root: 660.0, down: 494.0, bottom: 330.0 },
        "haunted" => ImpairedMelodySpec { root: 415.0, down: 293.0, bottom: 207.0 },
        "derelict" => ImpairedMelodySpec { root: 493.0, down: 415.0, bottom: 330.0 },
        "still" => ImpairedMelodySpec { root: 523.0, down: 440.0, bottom: 349.0 },
        "normal" => ImpairedMelodySpec { root: 523.0, down: 392.0, bottom: 262.0 },
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoveryMelodySpec {
    pub root: f64,
    pub up: f64,
    pub top: f64,
}
/// Notes (Hz) of the sanity-recovery cue.
pub fn recovery_melody_by_weather(key: &str) -> Option<RecoveryMelodySpec> {
    Some(match key {
        "wet" => RecoveryMelodySpec { root: 440.0, up: 587.0, top: 698.0 },
        "electric" => RecoveryMelodySpec { root: 660.0, up: 990.0, top: 1319.0 },
        "haunted" => RecoveryMelodySpec { root: 415.0, up: 622.0, top: 831.0 },
        "derelict" => RecoveryMelodySpec { root: 493.0, up: 587.0, top: 740.0 },
        "still" => RecoveryMelodySpec { root: 523.0, up: 659.0, top: 784.0 },
        "normal" => RecoveryMelodySpec { root: 523.0, up: 784.0, top: 1047.0 },
        _ => return None,
    })
}

/// Camera roll frequency at low sanity.
pub fn sanity_roll_freq_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 1.4,
        "electric" => 2.1,
        "haunted" => 1.9,
        "derelict" => 1.7,
        "still" => 1.3,
        "normal" => 1.7,
        _ => return None,
    })
}

/// Frequency multiplier of the panels' brightness breathing.
pub fn panel_lfo_freq_mul_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.75,
        "electric" => 1.35,
        "haunted" => 0.9,
        "derelict" => 1.0,
        "still" => 1.2,
        "normal" => 1.0,
        _ => return None,
    })
}

/// Amplitude of the candle flames' flicker.
pub fn flame_breath_amp_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.08,
        "electric" => 0.07,
        "haunted" => 0.07,
        "derelict" => 0.05,
        "still" => 0.03,
        "normal" => 0.05,
        _ => return None,
    })
}

/// Dust-mote colour (0xRRGGBB as a number).
pub fn dust_tint_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 12638440.0,
        "electric" => 15261856.0,
        "haunted" => 15251656.0,
        "derelict" => 14203008.0,
        "still" => 14209200.0,
        "normal" => 15852214.0,
        _ => return None,
    })
}

/// Player light colour (0xRRGGBB as a number).
pub fn player_light_tint_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 14741240.0,
        "electric" => 16775368.0,
        "haunted" => 16767200.0,
        "derelict" => 16769208.0,
        "still" => 16772560.0,
        "normal" => 16773839.0,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerLightScaleSpec {
    pub int_mul: f64,
    pub dist_mul: f64,
}
/// Player light intensity and distance multipliers.
pub fn player_light_scale_by_weather(key: &str) -> Option<PlayerLightScaleSpec> {
    Some(match key {
        "wet" => PlayerLightScaleSpec { int_mul: 0.85, dist_mul: 0.85 },
        "electric" => PlayerLightScaleSpec { int_mul: 1.1, dist_mul: 1.05 },
        "haunted" => PlayerLightScaleSpec { int_mul: 0.9, dist_mul: 0.95 },
        "derelict" => PlayerLightScaleSpec { int_mul: 0.95, dist_mul: 0.9 },
        "still" => PlayerLightScaleSpec { int_mul: 1.05, dist_mul: 1.1 },
        "normal" => PlayerLightScaleSpec { int_mul: 1.0, dist_mul: 1.0 },
        _ => return None,
    })
}

/// Probability that a candle relight fizzle echoes.
pub fn fizzle_echo_prob_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.45,
        "electric" => 0.2,
        "haunted" => 0.4,
        "derelict" => 0.35,
        "still" => 0.25,
        "normal" => 0.3,
        _ => return None,
    })
}

/// Probability that a candle relight fizzles.
pub fn fizzle_prob_by_weather(key: &str) -> Option<f64> {
    Some(match key {
        "wet" => 0.2,
        "electric" => 0.1,
        "haunted" => 0.18,
        "derelict" => 0.16,
        "still" => 0.13,
        "normal" => 0.15,
        _ => return None,
    })
}

/// Tail-beat blips ~2.8 s after a candle blowout: (freq, jitter, dur, wave, amp, delay ms).
pub fn tail_beat_specs(key: &str) -> Option<&'static [(f64, f64, f64, &'static str, f64, f64)]> {
    Some(match key {
        "wet" => &[(70.0, 8.0, 0.5, "sine", 0.03, 0.0), (48.0, 6.0, 0.7, "sine", 0.022, 80.0)],
        "still" => &[(4400.0, 200.0, 0.6, "sine", 0.025, 0.0), (6600.0, 200.0, 0.5, "sine", 0.018, 100.0)],
        "electric" => &[(1200.0, 40.0, 0.08, "square", 0.022, 0.0), (1600.0, 40.0, 0.05, "square", 0.018, 90.0)],
        "haunted" => &[(330.0, 6.0, 0.7, "sine", 0.024, 0.0), (349.0, 6.0, 0.7, "sine", 0.022, 40.0)],
        "derelict" => &[(180.0, 10.0, 0.35, "triangle", 0.028, 0.0)],
        "normal" => &[(880.0, 40.0, 0.4, "sine", 0.018, 0.0)],
        _ => return None,
    })
}

/// Spark blips of a candle relight: (freq, jitter, dur, wave, amp, delay ms).
pub fn relight_specs(key: &str) -> Option<&'static [(f64, f64, f64, &'static str, f64, f64)]> {
    Some(match key {
        "wet" => &[(520.0, 6.0, 0.14, "sine", 0.045, 40.0)],
        "still" => &[(1760.0, 40.0, 0.1, "sine", 0.035, 30.0)],
        "electric" => &[(880.0, 30.0, 0.05, "square", 0.04, 20.0), (1320.0, 40.0, 0.06, "square", 0.035, 80.0)],
        "haunted" => &[(415.0, 6.0, 0.2, "sine", 0.035, 60.0)],
        "derelict" => &[(220.0, 8.0, 0.16, "triangle", 0.04, 40.0)],
        "normal" => &[],
        _ => return None,
    })
}

pub const VIGNETTE_PULSE_AMP_DEFAULT: f64 = 0.015;
pub const PLAYER_LIGHT_BASE_INTENSITY: f64 = 2.4;
pub const PLAYER_LIGHT_BASE_DISTANCE: f64 = 7.0;
pub const RATE_DRIFT_THRESHOLD: f64 = 0.10;
pub const RATE_DRIFT_MIN_N: f64 = 8.0;
pub const RATE_DRIFT_DEBOUNCE: f64 = 30.0;
pub const RATE_DELTA_WINDOW: f64 = 30.0;
pub const WHISPER_MIN_OPACITY: f64 = 0.28;

/// The ambient sound pool a level draws its signature sounds from (weight,
/// sfx name); `""` is the silent entry (drawing it stretches the level's gaps).
pub const AMBIENT_EVENTS: [(f64, &str); 17] = [
    (3.0, "thud"),
    (3.0, "clank"),
    (3.0, "moan"),
    (2.0, "doorSlam"),
    (2.0, "distantFootsteps"),
    (2.0, "radioStatic"),
    (2.0, "paperRustle"),
    (2.0, "drip"),
    (1.0, "distantScream"),
    (1.0, "dialTone"),
    (1.0, "laugh"),
    (1.0, "chime"),
    (1.0, "breath"),
    (1.0, "piano"),
    (2.0, "dripHeavy"),
    (2.0, "pageFlip"),
    (3.0, ""),
];
