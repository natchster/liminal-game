//! The controls: every action the player has, the keys and mouse buttons
//! bound to it, and how a press reaches it.
//!
//! Codes are physical key names ("KeyW", "ShiftLeft", "Tab"); the mouse
//! buttons arrive as "Mouse0" (left), "Mouse1" (middle) and "Mouse2"
//! (right) while the pointer is locked. The layout keeps everything a run
//! needs under the left hand with W and Shift held, or on the mouse
//! (`tests/controls.rs` holds it to that). A held key fires its action once:
//! the OS key repeat only keeps it in `keys`.
use crate::prelude::*;

/// An action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Act {
    Forward,
    Back,
    Left,
    Right,
    Sprint,
    /// held: crouch while it is down
    Crouch,
    /// pressed: crouch until pressed again (or until you sprint)
    CrouchToggle,
    Interact,
    Flashlight,
    Paper,
    Drink,
    Battery,
    Call,
    Journal,
    /// held: what you carry and how you are, on the screen
    Glance,
    Pause,
    Mute,
    /// pressed: start or end a recording (`docs/engineering/REPLAYS.md`)
    Record,
    Debug,
}

/// One action's keys, in the order the controls page names them.
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub act: Act,
    pub keys: &'static [&'static str],
    /// what the controls page calls it
    pub label: &'static str,
    /// needed while running (with W and Shift held): it must be in reach
    pub on_the_run: bool,
}

const fn b(act: Act, keys: &'static [&'static str], label: &'static str, on_the_run: bool) -> Binding {
    Binding { act, keys, label, on_the_run }
}

/// The default layout.
pub static BINDINGS: [Binding; 19] = [
    b(Act::Forward, &["KeyW", "ArrowUp"], "Walk forward", true),
    b(Act::Back, &["KeyS", "ArrowDown"], "Walk back", true),
    b(Act::Left, &["KeyA", "ArrowLeft"], "Step left", true),
    b(Act::Right, &["KeyD", "ArrowRight"], "Step right", true),
    b(Act::Sprint, &["ShiftLeft", "ShiftRight"], "Run (hold)", true),
    b(Act::Crouch, &["ControlLeft", "ControlRight"], "Crouch (hold)", true),
    b(Act::CrouchToggle, &["KeyC"], "Crouch (toggle)", true),
    b(Act::Interact, &["KeyE", "Mouse0"], "Use, take, open", true),
    b(Act::Flashlight, &["KeyF", "Mouse2"], "Light", true),
    b(Act::Paper, &["KeyQ"], "Drop a sheet of paper", true),
    b(Act::Drink, &["KeyG", "Digit1"], "Drink almond water", true),
    b(Act::Battery, &["KeyR", "Digit2"], "Change the battery", true),
    b(Act::Call, &["KeyV", "KeyH"], "Call out", true),
    b(Act::Journal, &["Tab", "KeyJ"], "Notebook", true),
    b(Act::Glance, &["KeyX"], "Look at what you carry (hold)", true),
    b(Act::Pause, &["Escape", "KeyP"], "Pause", false),
    b(Act::Mute, &["KeyM"], "Mute", false),
    b(Act::Record, &["F9"], "Record (start, stop)", false),
    b(Act::Debug, &["F3"], "Debug panel", false),
];

/// What a left hand works while running: W under the middle finger and
/// Shift under the little finger, A/S/D beside them, and the keys a finger
/// reaches without letting go (index on R/F/G/V, ring on Q, thumb on Space
/// and the modifiers), and the mouse buttons.
pub const LEFT_HAND: [&str; 29] = [
    "KeyW",
    "KeyA",
    "KeyS",
    "KeyD",
    "ShiftLeft",
    "Escape",
    "Backquote",
    "Digit1",
    "Digit2",
    "Digit3",
    "Digit4",
    "Tab",
    "KeyQ",
    "KeyE",
    "KeyR",
    "KeyT",
    "CapsLock",
    "KeyF",
    "KeyG",
    "KeyZ",
    "KeyX",
    "KeyC",
    "KeyV",
    "ControlLeft",
    "AltLeft",
    "Space",
    "Mouse0",
    "Mouse1",
    "Mouse2",
];

impl Act {
    /// The binding of this action
    pub fn binding(self) -> &'static Binding {
        BINDINGS.iter().find(|b| b.act == self).expect("every action has a binding")
    }
    pub fn keys(self) -> &'static [&'static str] {
        self.binding().keys
    }
}

/// The actions `code` is bound to by default
pub fn acts_for(code: &str) -> impl Iterator<Item = Act> + '_ {
    BINDINGS.iter().filter(move |b| b.keys.contains(&code)).map(|b| b.act)
}

/// How a key is named on screen: "E", "Shift", "Tab", "Left click", "1"
pub fn key_label(code: &str) -> String {
    match code {
        "Mouse0" => "Left click".into(),
        "Mouse1" => "Middle click".into(),
        "Mouse2" => "Right click".into(),
        "ShiftLeft" | "ShiftRight" => "Shift".into(),
        "ControlLeft" | "ControlRight" => "Ctrl".into(),
        "AltLeft" | "AltRight" => "Option".into(),
        "ArrowUp" => "↑".into(),
        "ArrowDown" => "↓".into(),
        "ArrowLeft" => "←".into(),
        "ArrowRight" => "→".into(),
        "Escape" => "Esc".into(),
        _ => code.strip_prefix("Key").or_else(|| code.strip_prefix("Digit")).unwrap_or(code).to_string(),
    }
}

/// Run state of the controls (latches and the spent-sprint lockout).
#[derive(Clone, Debug, Default)]
pub struct InputState {
    /// crouching by C (until C again, or a sprint)
    pub crouch_latched: bool,
    /// the sprint is spent: it comes back once stamina is up to
    /// `SPRINT_BACK_AT` (no stutter at the bottom of the bar)
    pub winded: bool,
    /// seconds an E pressed with nothing in reach still waits for something
    /// to come into reach (running past a door, E a moment early)
    pub interact_buffer: f64,
    /// running by a press of the run key (Settings: "Press to run"), until
    /// you stop or the run is spent
    pub run_latched: bool,
    /// the window has just lost its focus (the pointer goes next)
    pub blurred: bool,
    /// the wheel's travel not yet turned into a page (px; a trackpad sends
    /// many small deltas and momentum)
    pub wheel_acc: f64,
}

/// The wheel's travel that turns one page (px; a mouse notch is 40)
pub const WHEEL_PAGE: f64 = 40.0;

/// How long an early E waits (s)
pub const INTERACT_BUFFER: f64 = 0.15;

/// Stamina a spent sprint needs back before you can run again
pub const SPRINT_BACK_AT: f64 = 30.0;

impl Game {
    /// Whether any key of `act` is held (the keys as Settings bind them)
    pub fn held(&self, act: Act) -> bool {
        self.settings.keys(act).iter().any(|k| self.keys.contains(k))
    }
    /// The actions `code` is bound to
    pub fn acts_of(&self, code: &str) -> Vec<Act> {
        self.settings.binds.iter().filter(|(_, k)| k.iter().any(|k| k == code)).map(|(a, _)| *a).collect()
    }
    /// The run key is held (or latched by a press, or the touch stick's)
    pub fn wants_sprint(&self) -> bool {
        self.held(Act::Sprint) || self.input.run_latched || self.touch_input.sprint
    }
    /// Crouching: Ctrl held, C latched, or the touch button
    pub fn wants_crouch(&self) -> bool {
        self.held(Act::Crouch) || self.input.crouch_latched || self.touch_input.crouch
    }
    /// Whether a sprint can start or go on. A spent sprint (stamina run out)
    /// comes back at `SPRINT_BACK_AT`, not at the first sliver of breath:
    /// without that, a held run key at the bottom of the bar stutters
    /// between a walk and three frames of running.
    pub fn sprint_ready(&mut self) -> bool {
        if self.player.stamina <= 0.5 {
            self.input.winded = true;
            self.input.run_latched = false;
        } else if self.player.stamina >= SPRINT_BACK_AT {
            self.input.winded = false;
        }
        !self.input.winded
    }
    /// One press of an action key (never a repeat)
    pub fn on_action(&mut self, act: Act) {
        match act {
            Act::Mute => {
                let m = !self.muted;
                self.set_muted(m);
                self.show_msg(if m { "Sound off" } else { "Sound on" }, 1.0);
            }
            Act::Sprint => {
                if self.settings.run_toggle && self.state == "playing" {
                    self.input.run_latched = !self.input.run_latched;
                }
            }
            Act::Interact => {
                let ordinary = self.state == "playing" && !self.in_stairwell && !self.in_loop;
                if ordinary && self.interact_target().is_none() {
                    self.input.interact_buffer = INTERACT_BUFFER;
                } else {
                    self.try_interact();
                }
            }
            Act::Flashlight => self.toggle_flashlight(),
            Act::Paper => self.drop_paper_marker(),
            Act::Drink => {
                self.drink_almond_water();
            }
            Act::Battery => {
                self.swap_battery();
            }
            Act::Call => self.cry_for_help(),
            Act::Journal => self.toggle_journal(),
            Act::Pause => self.toggle_pause(),
            Act::Debug => self.toggle_debug(),
            // the app starts or ends it; nothing in the game hangs on which
            Act::Record => self.want_record = true,
            Act::CrouchToggle => {
                if self.state == "playing" {
                    self.input.crouch_latched = !self.input.crouch_latched;
                }
            }
            Act::Forward | Act::Back | Act::Left | Act::Right | Act::Crouch | Act::Glance => {}
        }
    }
    /// Per frame while playing: an early E lands on what comes into reach.
    pub fn update_input(&mut self, dt: f64) {
        if self.input.interact_buffer <= 0.0 {
            return;
        }
        self.input.interact_buffer -= dt;
        if self.interact_target().is_some() {
            self.input.interact_buffer = 0.0;
            self.try_interact();
        }
    }
    /// A key came up. macOS never sends the key-up of a key pressed or let go
    /// while Command is held, so letting go of Command forgets every held key
    /// (or W, pressed during a Command shortcut, would walk on by itself).
    pub fn on_key_up(&mut self, code: &str) {
        if code == "MetaLeft" || code == "MetaRight" {
            self.keys.clear();
        }
    }
    /// Forget the latches, the early E and the notebook: a new life or a
    /// new run starts with empty hands (`restart`, `start_new_game`,
    /// `continue_game`)
    pub fn reset_hands(&mut self) {
        self.input = InputState::default();
        if self.journal_open {
            self.journal_open = false;
            self.ui.journal.remove_class("on");
        }
    }
    /// Escape: close the notebook, pause, or resume
    pub fn toggle_pause(&mut self) {
        if self.journal_open {
            self.toggle_journal();
        } else if self.state == "playing" {
            self.pause();
        } else if self.state == "paused" {
            self.resume();
        }
    }
    /// Stop the world (nothing moves, the sound drops low) and show the
    /// pause list over the frozen frame; the pointer is released
    pub fn pause(&mut self) {
        if self.state != "playing" {
            return;
        }
        // the notebook goes back in the pocket
        if self.journal_open {
            self.journal_open = false;
            self.ui.journal.remove_class("on");
        }
        self.state = "paused".into();
        self.want_pointer_lock = Some(false);
        self.input.run_latched = false;
        self.open_menu(crate::ui::menu::Screen::Pause);
        self.menu.note = self.pick_text("pauseSub");
        self.apply_master_gain();
    }
    /// Back into the world from the pause list
    pub fn resume(&mut self) {
        if self.state != "paused" {
            return;
        }
        self.audio_init();
        self.state = "playing".into();
        self.close_menu();
        self.hide_overlay();
        self.apply_master_gain();
        self.want_pointer_lock = Some(true);
    }
}
