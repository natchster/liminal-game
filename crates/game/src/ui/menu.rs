//! The menus: a short list of plain words at the lower left, over the
//! place (`docs/design/UI.md`, designs 3 and 4). The title is a live view of the
//! lobby with Continue, New descent, Settings and Quit; the pause list sits
//! over the frozen frame. A list is worked with the arrows / W S and Enter
//! / E / Space (and ← → on a setting), or hovered and clicked; Escape goes
//! back. `ui/screens.rs` paints it.
use crate::prelude::*;
use crate::session::input::{Act, BINDINGS, key_label};
use crate::session::settings::{ROWS, Row};

/// Which list is up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    /// New descent over a save: the save would be lost
    ConfirmNew,
    Pause,
    /// Leaving for the title: the floor starts again
    ConfirmLeave,
    Settings,
    /// every key, and rebinding
    Controls,
    /// the death card: one line, then Wake up
    Dead,
    /// the end of her story: the card, then on down
    Ending,
}

/// One line of a list.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// the click target (`menu:…`, `set:…`, `bind:…`; a `hud_hit_test` id)
    pub id: &'static str,
    pub label: String,
    /// a quiet word after the label ("level 4"), or a setting's value
    pub detail: String,
}

/// The open list, if any.
#[derive(Clone, Debug, Default)]
pub struct Menu {
    pub screen: Option<Screen>,
    /// a line above the list (a question), or ""
    pub heading: String,
    /// a quieter line under the heading, or ""
    pub note: String,
    pub items: Vec<Item>,
    pub sel: usize,
    /// on the title: the last line of hers you read, in her hand
    pub her_line: String,
    /// the title view: where it stands (`title_view`), its yaw when the
    /// title opened, and the time
    pub view_pos: [f64; 3],
    pub view_yaw: f64,
    pub view_t: f64,
    /// where Settings and Controls go back to (the title or the pause list)
    pub back: Option<Screen>,
    /// Controls: the action waiting for its new key
    pub capture: Option<Act>,
}

/// The title stands this far out along the way from where you begin
/// (metres), when the floor runs that far ...
pub const TITLE_BACK: f64 = 3.5;
/// ... and at least this
pub const TITLE_BACK_MIN: f64 = 2.0;
/// How far a way is measured (metres): beyond this, any way is as good
pub const TITLE_AHEAD: f64 = 24.0;

fn item(id: &'static str, label: &str, detail: &str) -> Item {
    Item { id, label: label.to_string(), detail: detail.to_string() }
}

/// The Controls page's line id of an action
pub fn bind_id(a: Act) -> &'static str {
    match a {
        Act::Forward => "bind:forward",
        Act::Back => "bind:back",
        Act::Left => "bind:left",
        Act::Right => "bind:right",
        Act::Sprint => "bind:sprint",
        Act::Crouch => "bind:crouch",
        Act::CrouchToggle => "bind:crouch-toggle",
        Act::Interact => "bind:interact",
        Act::Flashlight => "bind:flashlight",
        Act::Paper => "bind:paper",
        Act::Drink => "bind:drink",
        Act::Battery => "bind:battery",
        Act::Call => "bind:call",
        Act::Journal => "bind:journal",
        Act::Glance => "bind:glance",
        Act::Pause => "bind:pause",
        Act::Mute => "bind:mute",
        Act::Record => "bind:record",
        Act::Debug => "bind:debug",
    }
}

impl Game {
    /// The keys of `act` as the Controls page writes them ("E / Left click")
    pub fn keys_text(&self, act: Act) -> String {
        let mut seen = vec![];
        for k in self.settings.keys(act) {
            let l = key_label(k);
            if !seen.contains(&l) {
                seen.push(l);
            }
        }
        seen.join(" / ")
    }
    /// A list's lines, from the save and the settings
    fn menu_items(&self, s: Screen) -> Vec<Item> {
        let saved = self.load_game();
        match s {
            Screen::Title => {
                let mut v = vec![];
                if let Some(d) = &saved {
                    v.push(item("menu:continue", "Continue", &format!("level {}", d.level_num)));
                }
                v.extend([item("menu:new", "New descent", ""), item("menu:settings", "Settings", ""), item("menu:quit", "Quit", "")]);
                v
            }
            Screen::ConfirmNew => vec![item("menu:new-yes", "Begin again", ""), item("menu:back", "Back", "")],
            Screen::Pause => vec![
                item("menu:resume", "Resume", ""),
                item("menu:settings", "Settings", ""),
                item("menu:controls", "Controls", ""),
                // (its words follow the app's recording; play never reads them)
                item("menu:record", if self.rec_mark.since_ms.is_some() { "Stop recording" } else { "Record" }, ""),
                item("menu:leave", "Leave for the title", ""),
            ],
            Screen::ConfirmLeave => vec![item("menu:leave-yes", "Leave", ""), item("menu:back", "Back", "")],
            Screen::Dead => vec![item("menu:wake", "Wake up", ""), item("menu:leave-yes", "Leave for the title", "")],
            Screen::Ending => vec![item("menu:go-on", "Go on down", "")],
            Screen::Settings => {
                let mut v: Vec<Item> = ROWS.iter().map(|(r, id, label)| item(id, label, &self.settings.value(*r))).collect();
                v.extend([item("menu:controls", "Controls", ""), item("menu:back", "Back", "")]);
                v
            }
            Screen::Controls => {
                // (the debug panel's key is the developer's, not the player's)
                let mut v: Vec<Item> = BINDINGS
                    .iter()
                    .filter(|b| b.act != Act::Debug)
                    .map(|b| {
                        let detail = if self.menu.capture == Some(b.act) { "Press a key…".to_string() } else { self.keys_text(b.act) };
                        item(bind_id(b.act), b.label, &detail)
                    })
                    .collect();
                v.extend([item("menu:reset-keys", "Put every key back", ""), item("menu:back", "Back", "")]);
                v
            }
        }
    }
    /// Open `s` (its list rebuilt from the save), with the first line chosen
    pub fn open_menu(&mut self, s: Screen) {
        let saved = self.load_game();
        // Settings and Controls go back to where they were opened from
        if matches!(s, Screen::Settings | Screen::Controls)
            && let Some(from) = self.menu.screen.filter(|f| !matches!(f, Screen::Settings | Screen::Controls))
        {
            self.menu.back = Some(from);
        }
        self.menu.capture = None;
        self.menu.items = self.menu_items(s);
        let m = &mut self.menu;
        m.heading.clear();
        m.note.clear();
        match s {
            Screen::ConfirmNew => {
                m.heading = "Begin again?".into();
                if let Some(d) = &saved {
                    m.note = format!("The descent saved on level {} is lost.", d.level_num);
                }
            }
            Screen::Pause => m.heading = "Paused".into(),
            Screen::ConfirmLeave => {
                m.heading = "Leave for the title?".into();
                m.note = "Your notebook is kept. This floor starts again when you come back.".into();
            }
            Screen::Settings => m.heading = "Settings".into(),
            Screen::Controls => {
                m.heading = "Controls".into();
                m.note = "Enter on a line, then the new key. With the arrows, crouch with C: Ctrl and an arrow belong to the Mac.".into();
            }
            // (their words are the caller's: the death's line, her last card)
            Screen::Title | Screen::Dead | Screen::Ending => {}
        }
        // a question opens on its safe answer
        m.sel = m.items.iter().position(|i| i.id == "menu:back" && matches!(s, Screen::ConfirmNew | Screen::ConfirmLeave)).unwrap_or(0);
        m.screen = Some(s);
        if s == Screen::Title {
            // the last line of hers you read, if the save has one
            self.menu.her_line = saved
                .as_ref()
                .and_then(|d| {
                    crate::story::notebook::notebook(if d.run_seed > 0 { d.run_seed } else { self.run_seed })
                        .into_iter()
                        .rev()
                        .find(|(_, p)| d.notebook.iter().any(|n| n == p.id))
                        .map(|(_, p)| format!("\u{201c}{}\u{201d}", p.thread))
                })
                .unwrap_or_default();
            self.menu.view_yaw = self.player.yaw;
            self.menu.view_t = 0.0;
        }
    }
    /// Rebuild the open list's lines in place (a value changed), keeping the chosen line
    pub fn refresh_menu(&mut self) {
        if let Some(s) = self.menu.screen {
            let sel = self.menu.sel;
            self.menu.items = self.menu_items(s);
            self.menu.sel = sel.min(self.menu.items.len().saturating_sub(1));
        }
    }
    /// The list that is up and belongs to the moment (the title's only in
    /// the "menu" state, the pause list's only while paused), if any
    pub fn live_menu(&self) -> Option<Screen> {
        let s = self.menu.screen?;
        let ok = match s {
            Screen::Title | Screen::ConfirmNew => self.state == "menu",
            Screen::Pause | Screen::ConfirmLeave => self.state == "paused",
            Screen::Settings | Screen::Controls => self.state == "menu" || self.state == "paused",
            Screen::Dead => self.state == "dead",
            Screen::Ending => self.state == "ending",
        };
        ok.then_some(s)
    }
    /// Close the list
    pub fn close_menu(&mut self) {
        self.menu.screen = None;
        self.menu.items.clear();
        self.menu.capture = None;
    }
    /// Back one list: from Settings or Controls to where they were opened,
    /// from a question to its list, from the pause list into the world
    pub fn menu_back(&mut self) {
        match self.live_menu() {
            Some(Screen::Settings | Screen::Controls) => {
                let to = self.menu.back.unwrap_or(if self.state == "paused" { Screen::Pause } else { Screen::Title });
                self.open_menu(to);
            }
            Some(Screen::ConfirmNew) => self.open_menu(Screen::Title),
            Some(Screen::ConfirmLeave) => self.open_menu(Screen::Pause),
            Some(Screen::Pause) => self.resume(),
            _ => {}
        }
    }
    /// Keys while a list is up: move, change, choose, go back. True if taken.
    pub fn menu_key(&mut self, code: &str) -> bool {
        if self.live_menu().is_none() {
            return false;
        }
        // Controls: the next key is the new one (Escape keeps the old)
        if let Some(a) = self.menu.capture {
            if code != "Escape" {
                self.settings.rebind(a, code);
                self.save_settings();
            }
            self.menu.capture = None;
            self.refresh_menu();
            return true;
        }
        let n = self.menu.items.len().max(1);
        let row = self.menu.items.get(self.menu.sel).and_then(|i| ROWS.iter().find(|r| r.1 == i.id)).map(|r| r.0);
        match code {
            "ArrowUp" | "KeyW" => self.menu.sel = (self.menu.sel + n - 1) % n,
            "ArrowDown" | "KeyS" | "Tab" => self.menu.sel = (self.menu.sel + 1) % n,
            "ArrowLeft" | "KeyA" | "ArrowRight" | "KeyD" => {
                if let Some(r) = row {
                    self.change_setting(r, if matches!(code, "ArrowLeft" | "KeyA") { -1 } else { 1 });
                }
            }
            "Enter" | "NumpadEnter" | "Space" | "KeyE" => {
                if let Some(id) = self.menu.items.get(self.menu.sel).map(|i| i.id) {
                    self.menu_choose(id);
                }
            }
            "Escape" => self.menu_back(),
            _ => return false,
        }
        true
    }
    /// Step a setting, keep it, and show it at once
    pub fn change_setting(&mut self, r: Row, dir: i32) {
        self.settings.step(r, dir);
        self.save_settings();
        self.refresh_menu();
    }
    /// A line of a list was chosen (by key or click)
    pub fn menu_choose(&mut self, id: &str) {
        if let Some((r, _, _)) = ROWS.iter().find(|r| r.1 == id) {
            return self.change_setting(*r, 1);
        }
        if let Some(b) = BINDINGS.iter().find(|b| bind_id(b.act) == id) {
            // Escape stays the pause key: it is the way out of everything
            if b.act != Act::Pause {
                self.menu.capture = Some(b.act);
                self.refresh_menu();
            }
            return;
        }
        match id {
            "menu:continue" => {
                self.close_menu();
                self.continue_game();
            }
            "menu:new" => {
                if self.load_game().is_some() {
                    self.open_menu(Screen::ConfirmNew);
                } else {
                    self.new_descent();
                }
            }
            "menu:new-yes" => self.new_descent(),
            "menu:settings" => self.open_menu(Screen::Settings),
            "menu:controls" => self.open_menu(Screen::Controls),
            "menu:reset-keys" => {
                self.settings.binds = crate::session::settings::default_binds();
                self.save_settings();
                self.refresh_menu();
            }
            "menu:back" => self.menu_back(),
            "menu:resume" => self.resume(),
            // back into play, the app starting or ending the recording
            "menu:record" => {
                self.want_record = true;
                self.resume();
            }
            "menu:wake" => self.restart(),
            "menu:go-on" => self.end_ending(),
            "menu:leave" => self.open_menu(Screen::ConfirmLeave),
            "menu:leave-yes" => self.leave_for_title(),
            "menu:quit" => self.want_quit = true,
            _ => {}
        }
    }
    /// From the pause list (or the death card) to the title: the notebook is
    /// saved, the floor you were on stays behind the title (built again
    /// after a death, so the thing that found you is not standing in the
    /// view), and Continue starts it again
    pub fn leave_for_title(&mut self) {
        if self.state != "paused" && self.state != "dead" {
            return;
        }
        // (out of the Stairwell or the Loop too, and with nothing that was
        // chasing you frozen in the view)
        let n = self.level_num;
        self.reset_hands();
        self.init_level(n);
        // (a floor of a run played behind the title: New descent starts another)
        self.title_place = None;
        self.ui.fade.set_opacity(0.0);
        self.ui.flash.set_opacity(0.0);
        self.save_game();
        self.state = "menu".into();
        self.journal_open = false;
        self.ui.journal.remove_class("on");
        self.hide_overlay();
        self.open_menu(Screen::Title);
        self.apply_master_gain();
    }
    /// The pointer moved over the window (CSS px): a line under it is chosen.
    /// True if that changed the chosen line.
    pub fn menu_hover(&mut self, x: f64, y: f64) -> bool {
        if self.live_menu().is_none() || self.menu.capture.is_some() {
            return false;
        }
        let hit = self.hud_hit_test(x, y);
        match self.menu.items.iter().position(|i| i.id == hit) {
            Some(k) if k != self.menu.sel => {
                self.menu.sel = k;
                true
            }
            _ => false,
        }
    }
    /// The title's view of the floor, per frame: the camera a few metres
    /// out along the way from where you begin (`title_view`), turning
    /// slowly one way and back, as if someone stood there looking; New
    /// descent walks from there into your eyes (`docs/design/CUTSCENES.md`).
    pub fn update_title_view(&mut self, dt: f64) {
        if self.menu.view_t == 0.0 {
            let (pos, yaw) = self.title_view();
            self.menu.view_pos = pos;
            self.menu.view_yaw = yaw;
        }
        self.menu.view_t += dt;
        let t = self.menu.view_t;
        let cam = self.camera.clone();
        let p = self.menu.view_pos;
        cam.position().set(p[0], p[1] + (t * 0.9).dsin() * 0.004, p[2]);
        cam.rotation().y = self.menu.view_yaw + (t * 0.045).dsin() * 0.22;
        cam.rotation().x = -0.03 + (t * 0.031).dsin() * 0.02;
        cam.rotation().z = 0.0;
    }
    /// Where the title stands, and the way it looks: out along the way the
    /// floor runs furthest from where you begin (level, walkable, clear of
    /// walls and the void: on the Building a gallery, the void beside it),
    /// a few metres out along it (up to `TITLE_BACK`), looking on down it.
    /// Your run begins facing that way (`cutscene::uses`), and New descent
    /// draws the camera back from here into your eyes. With less than
    /// `TITLE_BACK_MIN` of floor any way, at your place looking down the
    /// place. → (its eye, its yaw)
    pub fn title_view(&self) -> ([f64; 3], f64) {
        let p = self.player.pos;
        let f0 = self.floor_y_at(p.x, p.z);
        let clear = |x: f64, z: f64| {
            let (tx, tz) = (self.tx(x), self.tz(z));
            self.in_grid(tx, tz)
                && !self.is_wall_tile(tx, tz)
                && self.ground.get(self.t_idx(tx, tz)).is_none_or(|g| g.walkable())
                && (self.floor_y_at(x, z) - f0).abs() < 0.05
                && !self.circle_hits_wall(x, z, 0.35)
        };
        let mut best: Option<(f64, f64)> = None;
        for k in 0..16 {
            let yaw = k as f64 * PI / 8.0;
            let (dx, dz) = (-yaw.dsin(), -yaw.dcos());
            let mut ahead = 0.0;
            while ahead < TITLE_AHEAD && clear(p.x + dx * (ahead + 0.25), p.z + dz * (ahead + 0.25)) {
                ahead += 0.25;
            }
            if best.is_none_or(|b| ahead > b.1 + 1e-9) {
                best = Some((yaw, ahead));
            }
        }
        match best {
            Some((yaw, ahead)) if ahead - 0.5 >= TITLE_BACK_MIN => {
                let d = (ahead - 0.5).min(TITLE_BACK);
                ([p.x - yaw.dsin() * d, f0 + EYE, p.z - yaw.dcos() * d], yaw)
            }
            _ => ([p.x, f0 + EYE, p.z], self.deepest_view_yaw()),
        }
    }

    /// How far you can see from where you begin, looking `yaw` (up to 60 m)
    fn sight_depth(&self, yaw: f64) -> f64 {
        let p = self.player.pos;
        let (dx, dz) = (-yaw.dsin(), -yaw.dcos());
        let mut d = 0.0;
        while d < 60.0 && !self.circle_hits_wall(p.x + dx * (d + 0.5), p.z + dz * (d + 0.5), 0.25) {
            d += 0.5;
        }
        d
    }

    /// The way the spawn sees furthest (of sixteen), so the title looks
    /// down the place rather than at a wall
    pub fn deepest_view_yaw(&self) -> f64 {
        let p = self.player.pos;
        // on the Building's floors: across the void, toward the door's light
        if let Some(b) = &self.building {
            let (ex, ez) = (self.wx(b.exit.0), self.wz(b.exit.1));
            return (-(ex - p.x)).datan2(-(ez - p.z));
        }
        let mut best = (self.player.yaw, 0.0);
        for k in 0..16 {
            let yaw = k as f64 * PI / 8.0;
            let d = self.sight_depth(yaw);
            if d > best.1 + 1e-9 {
                best = (yaw, d);
            }
        }
        best.0
    }
}
