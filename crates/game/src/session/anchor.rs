//! A level's start as an anchor (`docs/plan/REPLAYS.md`, R2b).
//!
//! Play enters a new floor in one place, `Game::enter_level`, always from a
//! scheduled step at the start of a frame (the door, the lift, a fall, the
//! Stairwell's door, the end of her story). Entering, it takes an anchor:
//! everything about the game that the new floor and the frames after it
//! read and that a fresh game would not already have. Nothing is left
//! pending from the floor before (its scheduled steps are dropped), so a
//! fresh game handed the anchor (`Game::start_from_anchor`) enters the
//! floor through the same code and stands exactly where play stood, frame
//! after frame: a replay starts there, and a save is one.
//!
//! What it keeps: the save (`Game::save_json`); the per-frame stream and
//! its draws; the clocks; the settings and the view; the keys held and
//! the input's latches; and what crosses floors unsaved (her lamp and her
//! crouch, the run's memory of answers and cries, the cutscenes seen, the
//! rare cadences, the asides' timing). Floats are kept as their bits, so
//! a time of `f64::INFINITY` comes back as it went.
use crate::base::records::WayAnswer;
use crate::base::rng::{native_state, set_native_state};
use crate::prelude::*;
use crate::secrets::Thing;
use crate::session::flow::SaveData;
use serde_json::{Value, json};

/// How play entered a floor: each way ends its entry its own way (its card,
/// its fade, its sounds), and an anchor's replay ends it the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// the floor's door (`next_level`)
    Door,
    /// the lift (`next_level_via_elevator`)
    Lift,
    /// a fall through a hole (`begin_fall_down`)
    Fall,
    /// the Stairwell's door
    Stairwell,
    /// on from the end of her story
    Ending,
}

impl How {
    pub fn id(self) -> &'static str {
        match self {
            How::Door => "door",
            How::Lift => "lift",
            How::Fall => "fall",
            How::Stairwell => "stairwell",
            How::Ending => "ending",
        }
    }
    pub fn from_id(s: &str) -> Option<How> {
        [How::Door, How::Lift, How::Fall, How::Stairwell, How::Ending].into_iter().find(|h| h.id() == s)
    }
}

/// A floor's start: which floor, how it was entered, the interval of the
/// frame it was entered in, and the game's state just before (`state`, JSON).
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    pub level: i32,
    pub how: How,
    pub ticks: u32,
    pub state: Value,
}

impl Anchor {
    pub fn to_json(&self) -> Value {
        json!({ "level": self.level, "how": self.how.id(), "ticks": self.ticks, "state": self.state })
    }
    pub fn from_json(v: &Value) -> Option<Anchor> {
        Some(Anchor {
            level: i32::try_from(v["level"].as_i64()?).ok()?,
            how: How::from_id(v["how"].as_str()?)?,
            ticks: u32::try_from(v["ticks"].as_u64()?).ok()?,
            state: v["state"].clone(),
        })
    }
}

type F64Field = (&'static str, fn(&mut Game) -> &mut f64);
type BoolField = (&'static str, fn(&mut Game) -> &mut bool);

/// The numbers that cross a floor and the anchor keeps (by name).
const F64S: &[F64Field] = &[
    ("perf_t", |g| &mut g.perf_t),
    ("now_ms", |g| &mut g.now_ms),
    ("level_start_t", |g| &mut g.level_start_t),
    ("eye_h", |g| &mut g.player.eye_h),
    ("bob_phase", |g| &mut g.player.bob_phase),
    ("bob_amp", |g| &mut g.player.bob_amp),
    ("last_step", |g| &mut g.last_step),
    ("stam_delay", |g| &mut g.player.stam_delay),
    ("roll", |g| &mut g.player.roll),
    ("crouch_pitch_lerp", |g| &mut g.player.crouch_pitch_lerp),
    ("audible_range", |g| &mut g.player.audible_range),
    ("dark_heart_t", |g| &mut g.dark_heart_t),
    ("crit_sanity_accum", |g| &mut g.crit_sanity_accum),
    ("sanity_beat_t", |g| &mut g.sanity_beat_t),
    ("sanity_whisper_t", |g| &mut g.sanity_whisper_t),
    ("hummer_t", |g| &mut g.hummer_t),
    ("zone_toast_t", |g| &mut g.zone_toast_t),
    ("cry_cooldown", |g| &mut g.cry_cooldown),
    ("asides.clock", |g| &mut g.asides.clock),
    ("asides.voice_until", |g| &mut g.asides.voice_until),
    ("asides.active_at", |g| &mut g.asides.active_at),
    ("input.interact_buffer", |g| &mut g.input.interact_buffer),
    ("input.wheel_acc", |g| &mut g.input.wheel_acc),
    ("stair.next_phrase", |g| &mut g.stair_audio.next_phrase),
    ("stair.voice_until", |g| &mut g.stair_audio.voice_until),
    ("stair.next_knock", |g| &mut g.stair_audio.next_knock),
    ("stair.heart_t", |g| &mut g.stair_audio.heart_t),
    ("stair.step_acc", |g| &mut g.stair_audio.step_acc),
    ("loop.step_acc", |g| &mut g.loop_audio.step_acc),
    ("loop.touch_t", |g| &mut g.loop_audio.touch_t),
];

/// The flags that cross a floor and the anchor keeps (by name).
const BOOLS: &[BoolField] = &[
    ("flashlight_on", |g| &mut g.player.flashlight_on),
    ("crouch", |g| &mut g.player.crouch),
    ("was_in_pool", |g| &mut g.player.was_in_pool),
    ("sanity_was_impaired", |g| &mut g.sanity_was_impaired),
    ("sanity_was_breaking", |g| &mut g.sanity_was_breaking),
    ("archive_milestone_shown", |g| &mut g.archive_milestone_shown),
    ("hinted_bottle", |g| &mut g.hinted_bottle),
    ("hinted_battery", |g| &mut g.hinted_battery),
    ("shaft_looked_back", |g| &mut g.shaft_looked_back),
    ("asides.nerve_low", |g| &mut g.asides.nerve_low),
    ("input.crouch_latched", |g| &mut g.input.crouch_latched),
    ("input.winded", |g| &mut g.input.winded),
    ("input.run_latched", |g| &mut g.input.run_latched),
    ("input.blurred", |g| &mut g.input.blurred),
    ("pointer_locked", |g| &mut g.pointer_locked),
];

fn bits(x: f64) -> Value {
    json!(x.to_bits())
}
fn unbits(v: &Value) -> Option<f64> {
    v.as_u64().map(f64::from_bits)
}
/// A pooled effect (a scuff, a ripple, a sparkle): its timer, which decides
/// whether it draws to start again, and its look.
fn fx_state(fx: &crate::base::records::PooledFx) -> Value {
    let o = &fx.obj;
    // (one borrow of the node at a time: two in one expression panic)
    let p = *o.position();
    let s = *o.scale();
    let m = o.mat();
    let m = m.m();
    json!({
        "t": bits(fx.t),
        "pos": [bits(p.x), bits(p.y), bits(p.z)],
        "scale": [bits(s.x), bits(s.y), bits(s.z)],
        "visible": o.visible(),
        "opacity": bits(m.opacity),
        "color": [bits(m.color.r), bits(m.color.g), bits(m.color.b)],
    })
}
fn set_fx_state(fx: &mut crate::base::records::PooledFx, v: &Value) {
    let three = |k: &str| -> Option<[f64; 3]> { <[f64; 3]>::try_from(v[k].as_array()?.iter().filter_map(unbits).collect::<Vec<_>>()).ok() };
    if let Some(t) = unbits(&v["t"]) {
        fx.t = t;
    }
    let o = fx.obj.clone();
    if let Some([x, y, z]) = three("pos") {
        o.position().set(x, y, z);
    }
    if let Some([x, y, z]) = three("scale") {
        o.scale().set(x, y, z);
    }
    if let Some(vis) = v["visible"].as_bool() {
        o.set_visible(vis);
    }
    let m = o.mat();
    let mut m = m.m();
    if let Some(a) = unbits(&v["opacity"]) {
        m.opacity = a;
    }
    if let Some([r, g, b]) = three("color") {
        m.color.set_rgb(r, g, b);
    }
}

fn way_id(w: WayAnswer) -> &'static str {
    match w {
        WayAnswer::Lights => "lights",
        WayAnswer::Prints => "prints",
        WayAnswer::Call => "call",
    }
}
fn way_from_id(s: &str) -> Option<WayAnswer> {
    [WayAnswer::Lights, WayAnswer::Prints, WayAnswer::Call].into_iter().find(|w| way_id(*w) == s)
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect()
}

impl Game {
    /// Everything a floor's start reads that a fresh game would not have
    /// (the module's doc says what).
    pub fn anchor_state(&mut self) -> Value {
        let (s, draws) = native_state();
        let f64s: serde_json::Map<String, Value> = F64S.iter().map(|(k, f)| (k.to_string(), bits(*f(self)))).collect();
        let bools: serde_json::Map<String, Value> = BOOLS.iter().map(|(k, f)| (k.to_string(), json!(*f(self)))).collect();
        let rare: serde_json::Map<String, Value> =
            crate::audio::ambient::CADENCES.iter().filter(|c| c.run).map(|c| (c.tag.to_string(), bits(*(c.timer)(self)))).collect();
        let mut keys: Vec<&String> = self.keys.iter().collect();
        keys.sort();
        let mut zones: Vec<&String> = self.zones_seen.iter().collect();
        zones.sort();
        let dust_color = {
            let c = self.dust_mat.m().color;
            [bits(c.r), bits(c.g), bits(c.b)]
        };
        let (light_color, light_distance) = {
            let l = self.player_light.light();
            ([bits(l.color.r), bits(l.color.g), bits(l.color.b)], bits(l.distance))
        };
        json!({
            "save": self.save_json(),
            "native": [s[0], s[1], draws],
            "clock_ticks": self.clock_ticks,
            "settings": self.settings.to_json(),
            "view": [bits(self.view_width), bits(self.view_height), bits(self.scale_factor)],
            "f64": f64s,
            "bool": bools,
            "rare": rare,
            "keys": keys,
            "hide_eye": self.player.hide_eye.map(bits),
            "last_way": self.last_way.map(way_id),
            "ways_shown": self.ways_shown,
            "cries": self.cries,
            "cry_last_at": self.cry_last_at.map(bits),
            "cry_last_line": self.cry_last_line,
            "last_used": self.last_used.map(Thing::id),
            "cut_seen": self.cut_seen,
            "zones_seen": zones,
            "last_zone_arch": self.last_zone_arch,
            "journal_newest": self.journal_newest,
            "asides.last": self.asides.last.map(|(t, k)| json!([bits(t), k])),
            "asides.floor": self.asides.floor,
            "look": {
                "dust_pos": self.dust_positions.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                "dust_vel": self.dust_velocities.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                "dust_color": dust_color,
                "light_color": light_color,
                "light_distance": light_distance,
                "scuffs": self.scuffs.iter().map(fx_state).collect::<Vec<_>>(),
                "ripples": self.ripples.iter().map(fx_state).collect::<Vec<_>>(),
                "sparkles": self.sparkles.iter().map(fx_state).collect::<Vec<_>>(),
            },
            "sound": self.soundscape().map(|s| {
                let (preset, hum) = s.carried();
                json!({ "preset": preset, "hum": hum.map(bits) })
            }),
        })
    }

    /// A fresh game put where `anchor_state` found play (before the floor is
    /// entered: `start_from_anchor` enters it).
    pub fn restore_anchor_state(&mut self, v: &Value) -> Result<(), String> {
        let save = SaveData::from_json(&v["save"]).ok_or("an anchor without its save")?;
        if save.run_seed > 0 {
            self.run_seed = save.run_seed;
        }
        self.level_num = save.level_num;
        self.apply_saved(&save);
        let n = v["native"].as_array().ok_or("an anchor without its stream")?;
        let word = |i: usize| n.get(i).and_then(Value::as_u64).ok_or("an anchor's stream is short");
        set_native_state([word(0)?, word(1)?], word(2)?);
        self.clock_ticks = v["clock_ticks"].as_u64().ok_or("an anchor without its clock")?;
        self.settings = crate::session::settings::Settings::from_json(&v["settings"]);
        if let Some([w, h, s]) = v["view"].as_array().map(|a| a.iter().filter_map(unbits).collect::<Vec<_>>()).as_deref() {
            (self.view_width, self.view_height, self.scale_factor) = (*w, *h, *s);
        }
        for (k, f) in F64S {
            *f(self) = unbits(&v["f64"][*k]).ok_or_else(|| format!("an anchor without {k}"))?;
        }
        for (k, f) in BOOLS {
            *f(self) = v["bool"][*k].as_bool().ok_or_else(|| format!("an anchor without {k}"))?;
        }
        for c in crate::audio::ambient::CADENCES.iter().filter(|c| c.run) {
            *(c.timer)(self) = unbits(&v["rare"][c.tag]).ok_or_else(|| format!("an anchor without the cadence {}", c.tag))?;
        }
        self.keys = strings(&v["keys"]).into_iter().collect();
        self.player.hide_eye = unbits(&v["hide_eye"]);
        self.last_way = v["last_way"].as_str().and_then(way_from_id);
        self.ways_shown = v["ways_shown"].as_u64().unwrap_or(0) as u32;
        self.cries = v["cries"].as_u64().unwrap_or(0) as u32;
        self.cry_last_at = unbits(&v["cry_last_at"]);
        self.cry_last_line = v["cry_last_line"].as_str().map(String::from);
        self.last_used = v["last_used"].as_str().and_then(Thing::from_id);
        self.cut_seen =
            v["cut_seen"].as_object().into_iter().flatten().filter_map(|(k, n)| Some((k.clone(), n.as_u64()? as u32))).collect();
        self.zones_seen = strings(&v["zones_seen"]).into_iter().collect();
        self.last_zone_arch = v["last_zone_arch"].as_str().map(String::from);
        self.journal_newest = v["journal_newest"].as_i64();
        self.asides.last = v["asides.last"].as_array().and_then(|a| Some((unbits(a.first()?)?, a.get(1)?.as_u64()? as u8)));
        self.asides.floor = v["asides.floor"].as_u64().unwrap_or(0) as u32;
        // the dust and her light as play had them (the dust's motes move and
        // are put back by the per-frame stream, so where they are is drawn on)
        let look = &v["look"];
        let f32s = |k: &str| -> Vec<f32> {
            look[k].as_array().into_iter().flatten().filter_map(|x| x.as_u64().map(|b| f32::from_bits(b as u32))).collect()
        };
        let (pos, vel) = (f32s("dust_pos"), f32s("dust_vel"));
        if pos.len() == self.dust_positions.len() && vel.len() == self.dust_velocities.len() {
            (self.dust_positions, self.dust_velocities) = (pos, vel);
        }
        let rgb =
            |k: &str| -> Option<[f64; 3]> { <[f64; 3]>::try_from(look[k].as_array()?.iter().filter_map(unbits).collect::<Vec<_>>()).ok() };
        if let Some([r, g, b]) = rgb("dust_color") {
            let mut m = self.dust_mat.m();
            (m.color.r, m.color.g, m.color.b) = (r, g, b);
        }
        if let Some([r, g, b]) = rgb("light_color") {
            let mut l = self.player_light.light();
            (l.color.r, l.color.g, l.color.b) = (r, g, b);
        }
        if let Some(d) = unbits(&look["light_distance"]) {
            self.player_light.light().distance = d;
        }
        for (pool, k) in [(&mut self.scuffs, "scuffs"), (&mut self.ripples, "ripples"), (&mut self.sparkles, "sparkles")] {
            let saved = look[k].as_array().cloned().unwrap_or_default();
            if saved.len() == pool.len() {
                for (fx, s) in pool.iter_mut().zip(&saved) {
                    set_fx_state(fx, s);
                }
            }
        }
        if let (Some(s), Some(sound)) = (self.soundscape(), v["sound"].as_object()) {
            let hum: Vec<f64> = sound.get("hum").and_then(Value::as_array).into_iter().flatten().filter_map(unbits).collect();
            if let Ok(hum) = <[f64; 4]>::try_from(hum) {
                s.set_carried(sound.get("preset").and_then(Value::as_str).map(String::from), hum);
            }
        }
        Ok(())
    }

    /// Play enters floor `n` (the scheduled step of a door, the lift, a
    /// fall…): the anchor is taken (`entered`), what the floor before left
    /// pending is dropped, and the floor is built and begun as `how` begins
    /// it.
    pub fn enter_level(&mut self, n: i32, how: How) {
        let state = self.anchor_state();
        self.entered = Some(Anchor { level: n, how, ticks: self.last_ticks, state });
        self.begin_level(n, how);
    }

    /// The entry itself, the same for play and for a replay from an anchor.
    fn begin_level(&mut self, n: i32, how: How) {
        // nothing scheduled on the floor before carries onto this one: a
        // fresh game handed the anchor has none of it
        self.timeouts.clear();
        match how {
            How::Door => {
                self.init_level(n);
                self.save_game();
                self.state = "playing".into();
                let title = self.level_title();
                self.show_level_card(&title);
                self.scp_arrival();
                self.ui.fade.set_opacity(0.0);
            }
            How::Lift => {
                self.init_level(n);
                self.save_game();
                self.state = "playing".into();
                let sub = self.pick_text("levelSubtitle");
                self.show_level_card(&sub);
                self.ui.fade.set_opacity(0.0);
            }
            How::Fall => {
                self.init_level(n);
                self.save_game();
                self.camera.rotation().z = 0.0;
                self.state = "playing".into();
                self.show_level_card("You fell");
                self.schedule(400.0, |g| g.ui.fade.set_opacity(0.0));
            }
            How::Stairwell => self.enter_from_stairwell(n),
            How::Ending => {
                self.init_level(n);
                self.save_game();
                self.state = "playing".into();
                let title = self.level_title();
                self.show_level_card(&title);
                self.ui.fade.set_opacity(0.0);
                self.want_pointer_lock = Some(true);
            }
        }
    }

    /// A fresh game started where `anchor` was taken: its state put back,
    /// the floor entered as play entered it. Returns the step of the frame
    /// it was entered in, which the caller then runs (`Game::frame`), as
    /// play ran the rest of that frame.
    pub fn start_from_anchor(&mut self, anchor: &Anchor) -> Result<f64, String> {
        // out of the title and into a run, as New Game and Continue go (the
        // sound built here draws from the per-frame stream, which the
        // anchor then puts back where play had it)
        self.close_menu();
        self.title_place = None;
        self.reset_hands();
        self.audio_init();
        self.hide_overlay();
        self.restore_anchor_state(&anchor.state)?;
        self.last_ticks = anchor.ticks;
        self.begin_level(anchor.level, anchor.how);
        Ok(f64::from(anchor.ticks) / f64::from(crate::base::game::CLOCK_TICKS_PER_SECOND))
    }
}
