//! The player's settings (`docs/UI.md`, design 4): how the view moves, how
//! the screen looks, how the run key behaves, how loud, and the keys. Kept
//! in the store under `settings`, apart from the save, and applied live.
//! The defaults are the game as it was: a headless run (which reads no
//! settings) looks and plays exactly as before.
use crate::prelude::*;
use crate::session::input::{Act, BINDINGS};
use serde_json::{json, Value};

/// The store key
pub const SETTINGS_KEY: &str = "settings";

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// look speed, × the base (0.0022 rad per count)
    pub look: f64,
    pub invert_y: bool,
    /// the vertical field of view walking (°); running and crouching keep
    /// their offsets from it (+7, −6)
    pub fov: f64,
    /// head bob: 1 full, 0.5 reduced, 0 off
    pub bob: f64,
    /// camera shake (scares, low sanity, the lens dip): on or off
    pub shake: bool,
    /// film grain: on or off
    pub grain: bool,
    /// the run key toggles a run (until you stop, or it is spent)
    pub run_toggle: bool,
    /// master volume, 0..1
    pub volume: f64,
    /// brightness, −5..5 (0: as the place is lit)
    pub brightness: i32,
    /// losing the window's focus pauses
    pub focus_pause: bool,
    /// the keys of each action (the defaults are `BINDINGS`)
    pub binds: Vec<(Act, Vec<String>)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            look: 1.0,
            invert_y: false,
            fov: 74.0,
            bob: 1.0,
            shake: true,
            grain: true,
            run_toggle: false,
            volume: 1.0,
            brightness: 0,
            focus_pause: true,
            binds: default_binds(),
        }
    }
}

pub fn default_binds() -> Vec<(Act, Vec<String>)> {
    BINDINGS.iter().map(|b| (b.act, b.keys.iter().map(|k| k.to_string()).collect())).collect()
}

/// An action by its stored name ("Drink")
fn act_named(s: &str) -> Option<Act> {
    BINDINGS.iter().map(|b| b.act).find(|a| format!("{a:?}") == s)
}

impl Settings {
    pub fn to_json(&self) -> Value {
        let binds: serde_json::Map<String, Value> = self.binds.iter().map(|(a, k)| (format!("{a:?}"), json!(k))).collect();
        json!({
            "look": self.look, "invertY": self.invert_y, "fov": self.fov, "bob": self.bob, "shake": self.shake,
            "grain": self.grain, "runToggle": self.run_toggle, "volume": self.volume, "brightness": self.brightness,
            "focusPause": self.focus_pause, "binds": binds,
        })
    }
    /// From the store, each value clamped; anything missing keeps its default
    pub fn from_json(v: &Value) -> Settings {
        let d = Settings::default();
        let f = |k: &str, lo: f64, hi: f64, dv: f64| v.get(k).and_then(|x| x.as_f64()).map(|x| x.clamp(lo, hi)).unwrap_or(dv);
        let b = |k: &str, dv: bool| v.get(k).and_then(|x| x.as_bool()).unwrap_or(dv);
        let mut binds = d.binds.clone();
        if let Some(m) = v.get("binds").and_then(|m| m.as_object()) {
            for (name, keys) in m {
                let (Some(a), Some(keys)) = (act_named(name), keys.as_array()) else { continue };
                let keys: Vec<String> = keys.iter().filter_map(|k| k.as_str().map(String::from)).collect();
                if let (Some(slot), false) = (binds.iter_mut().find(|(x, _)| *x == a), keys.is_empty()) {
                    slot.1 = keys;
                }
            }
        }
        Settings {
            look: f("look", 0.25, 3.0, d.look),
            invert_y: b("invertY", d.invert_y),
            fov: f("fov", 60.0, 90.0, d.fov),
            bob: f("bob", 0.0, 1.0, d.bob),
            shake: b("shake", d.shake),
            grain: b("grain", d.grain),
            run_toggle: b("runToggle", d.run_toggle),
            volume: f("volume", 0.0, 1.0, d.volume),
            brightness: f("brightness", -5.0, 5.0, d.brightness as f64).round() as i32,
            focus_pause: b("focusPause", d.focus_pause),
            binds,
        }
    }
    /// The keys of `act`
    pub fn keys(&self, act: Act) -> &[String] {
        self.binds.iter().find(|(a, _)| *a == act).map(|(_, k)| k.as_slice()).unwrap_or(&[])
    }
    /// Bind `code` as the first key of `act`. The keys swap: another action
    /// that had `code` gets `act`'s old first key in its place, so no
    /// action is left with fewer keys.
    pub fn rebind(&mut self, act: Act, code: &str) {
        let old_first = self.keys(act).first().cloned();
        let mine = self.keys(act).iter().any(|k| k == code);
        if let Some((_, keys)) = self.binds.iter_mut().find(|(a, _)| *a == act) {
            match keys.iter().position(|k| k == code) {
                // already one of its keys: it moves to the front
                Some(i) => keys.swap(0, i),
                None if keys.is_empty() => keys.push(code.to_string()),
                None => keys[0] = code.to_string(),
            }
        }
        if mine {
            return;
        }
        for (a, keys) in self.binds.iter_mut() {
            if *a != act && keys.iter().any(|k| k == code) {
                for k in keys.iter_mut().filter(|k| *k == code) {
                    *k = old_first.clone().unwrap_or_default();
                }
                let mut seen = std::collections::BTreeSet::new();
                keys.retain(|k| !k.is_empty() && seen.insert(k.clone()));
            }
        }
    }
}

/// One line of the settings list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Look,
    Invert,
    Fov,
    Bob,
    Shake,
    Grain,
    Run,
    Volume,
    Brightness,
    FocusPause,
}

/// The settings list, in order: (row, its menu id, its label)
pub const ROWS: [(Row, &str, &str); 10] = [
    (Row::Look, "set:look", "Look speed"),
    (Row::Invert, "set:invert", "Invert looking up and down"),
    (Row::Fov, "set:fov", "Field of view"),
    (Row::Bob, "set:bob", "Head bob"),
    (Row::Shake, "set:shake", "Camera shake"),
    (Row::Grain, "set:grain", "Film grain"),
    (Row::Run, "set:run", "Run key"),
    (Row::Volume, "set:volume", "Volume"),
    (Row::Brightness, "set:brightness", "Brightness"),
    (Row::FocusPause, "set:focus", "Pause when the window is left"),
];

fn on_off(b: bool) -> String {
    if b { "On" } else { "Off" }.into()
}

impl Settings {
    /// A row's value as the list shows it
    pub fn value(&self, r: Row) -> String {
        match r {
            Row::Look => format!("{:.2}", self.look),
            Row::Invert => on_off(self.invert_y),
            Row::Fov => format!("{:.0}°", self.fov),
            Row::Bob => match self.bob {
                b if b >= 0.99 => "Full".into(),
                b if b > 0.01 => "Reduced".into(),
                _ => "Off".into(),
            },
            Row::Shake => on_off(self.shake),
            Row::Grain => on_off(self.grain),
            Row::Run => if self.run_toggle { "Press to run" } else { "Hold to run" }.into(),
            Row::Volume => format!("{:.0}%", self.volume * 100.0),
            Row::Brightness => format!("{:+}", self.brightness).replace("+0", "0"),
            Row::FocusPause => on_off(self.focus_pause),
        }
    }
    /// Step a row by `dir` (−1 or +1); the switches flip either way
    pub fn step(&mut self, r: Row, dir: i32) {
        let d = dir as f64;
        match r {
            Row::Look => self.look = ((self.look + 0.05 * d) * 20.0).round() / 20.0,
            Row::Invert => self.invert_y = !self.invert_y,
            Row::Fov => self.fov += 2.0 * d,
            Row::Bob => {
                let i = [0.0, 0.5, 1.0].iter().position(|v| (v - self.bob).abs() < 0.01).unwrap_or(2) as i32;
                self.bob = [0.0, 0.5, 1.0][(i + dir).clamp(0, 2) as usize];
            }
            Row::Shake => self.shake = !self.shake,
            Row::Grain => self.grain = !self.grain,
            Row::Run => self.run_toggle = !self.run_toggle,
            Row::Volume => self.volume = ((self.volume + 0.1 * d) * 10.0).round() / 10.0,
            Row::Brightness => self.brightness += dir,
            Row::FocusPause => self.focus_pause = !self.focus_pause,
        }
        self.look = self.look.clamp(0.25, 3.0);
        self.fov = self.fov.clamp(60.0, 90.0);
        self.volume = self.volume.clamp(0.0, 1.0);
        self.brightness = self.brightness.clamp(-5, 5);
    }
}

impl Game {
    /// Read the settings from the store (defaults if there are none)
    pub fn load_settings(&mut self) {
        self.settings = self
            .storage_get(SETTINGS_KEY)
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .map(|v| Settings::from_json(&v))
            .unwrap_or_default();
        self.apply_master_gain();
    }
    /// Keep the settings in the store, and apply what is applied at once
    pub fn save_settings(&mut self) {
        let s = self.settings.to_json().to_string();
        self.storage_set(SETTINGS_KEY, &s);
        self.apply_master_gain();
    }
    /// The master gain: 0 muted, else 0.9 × the volume, low while paused
    pub fn apply_master_gain(&mut self) {
        let paused = if self.state == "paused" { 0.25 } else { 1.0 };
        let g = if self.muted { 0.0 } else { 0.9 * self.settings.volume * paused };
        if let Some(master) = &self.master {
            master.gain().set_value(g);
        }
    }
}
