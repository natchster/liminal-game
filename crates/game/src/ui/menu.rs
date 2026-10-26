//! The menus: a short list of plain words at the lower left, over the
//! place (`docs/UI.md`, design 3). The title is a live view of the lobby
//! with Continue, New descent and Quit; the list is worked with the
//! arrows / W S and Enter / E / Space, or hovered and clicked, and Escape
//! goes back. `ui/screens.rs` paints it.
use crate::prelude::*;

/// Which list is up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    /// New descent over a save: the save would be lost
    ConfirmNew,
}

/// One line of a list.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// the click target (`menu:…`, a `hud_hit_test` id)
    pub id: &'static str,
    pub label: String,
    /// a quiet word after the label ("level 4")
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
    /// the title view: the camera's yaw when the title opened, and the time
    pub view_yaw: f64,
    pub view_t: f64,
}

fn item(id: &'static str, label: &str, detail: &str) -> Item {
    Item { id, label: label.to_string(), detail: detail.to_string() }
}

impl Game {
    /// Open `s` (its list rebuilt from the save), with the first line chosen
    pub fn open_menu(&mut self, s: Screen) {
        let saved = self.load_game();
        let m = &mut self.menu;
        m.heading.clear();
        m.note.clear();
        m.items = match s {
            Screen::Title => {
                let mut v = vec![];
                if let Some(d) = &saved {
                    v.push(item("menu:continue", "Continue", &format!("level {}", d.level_num)));
                }
                v.extend([item("menu:new", "New descent", ""), item("menu:quit", "Quit", "")]);
                v
            }
            Screen::ConfirmNew => {
                m.heading = "Begin again?".into();
                m.note = match &saved {
                    Some(d) => format!("The descent saved on level {} is lost.", d.level_num),
                    None => String::new(),
                };
                vec![item("menu:new-yes", "Begin again", ""), item("menu:back", "Back", "")]
            }
        };
        m.sel = 0;
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
    /// The list that is up and belongs to the moment (the title's only in
    /// the "menu" state), if any
    pub fn live_menu(&self) -> Option<Screen> {
        let s = self.menu.screen?;
        let state = match s {
            Screen::Title | Screen::ConfirmNew => "menu",
        };
        (self.state == state).then_some(s)
    }
    /// Close the list
    pub fn close_menu(&mut self) {
        self.menu.screen = None;
        self.menu.items.clear();
    }
    /// Keys while a list is up: move, choose, go back. True if taken.
    pub fn menu_key(&mut self, code: &str) -> bool {
        let Some(s) = self.live_menu() else { return false };
        let n = self.menu.items.len().max(1);
        match code {
            "ArrowUp" | "KeyW" => self.menu.sel = (self.menu.sel + n - 1) % n,
            "ArrowDown" | "KeyS" | "Tab" => self.menu.sel = (self.menu.sel + 1) % n,
            "Enter" | "NumpadEnter" | "Space" | "KeyE" => {
                if let Some(id) = self.menu.items.get(self.menu.sel).map(|i| i.id) {
                    self.menu_choose(id);
                }
            }
            "Escape" => match s {
                Screen::ConfirmNew => self.open_menu(Screen::Title),
                Screen::Title => {}
            },
            _ => return false,
        }
        true
    }
    /// A line of a list was chosen (by key or click)
    pub fn menu_choose(&mut self, id: &str) {
        match id {
            "menu:continue" => {
                self.close_menu();
                self.continue_game();
            }
            "menu:new" => {
                if self.load_game().is_some() {
                    self.open_menu(Screen::ConfirmNew);
                } else {
                    self.close_menu();
                    self.start_new_game();
                }
            }
            "menu:new-yes" => {
                self.close_menu();
                self.start_new_game();
            }
            "menu:back" => self.open_menu(Screen::Title),
            "menu:quit" => self.want_quit = true,
            _ => {}
        }
    }
    /// The pointer moved over the window (CSS px): a line under it is chosen.
    /// True if that changed the chosen line.
    pub fn menu_hover(&mut self, x: f64, y: f64) -> bool {
        if self.live_menu().is_none() {
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
    /// The title's view of the lobby, per frame: the camera at the spawn,
    /// turning slowly one way and back, as if someone stood there looking.
    pub fn update_title_view(&mut self, dt: f64) {
        if self.menu.view_t == 0.0 {
            self.menu.view_yaw = self.deepest_view_yaw();
        }
        self.menu.view_t += dt;
        let t = self.menu.view_t;
        let cam = self.camera.clone();
        cam.position().set(self.player.pos.x, EYE + (t * 0.9).sin() * 0.004, self.player.pos.z);
        cam.rotation().y = self.menu.view_yaw + (t * 0.045).sin() * 0.22;
        cam.rotation().x = -0.03 + (t * 0.031).sin() * 0.02;
        cam.rotation().z = 0.0;
    }
    /// The way the spawn sees furthest (of sixteen), so the title looks
    /// down the place rather than at a wall
    pub fn deepest_view_yaw(&self) -> f64 {
        let p = self.player.pos;
        let mut best = (self.player.yaw, 0.0);
        for k in 0..16 {
            let yaw = k as f64 * PI / 8.0;
            let (dx, dz) = (-yaw.sin(), -yaw.cos());
            let mut d = 0.0;
            while d < 60.0 && !self.circle_hits_wall(p.x + dx * (d + 0.5), p.z + dz * (d + 0.5), 0.25) {
                d += 0.5;
            }
            if d > best.1 + 1e-9 {
                best = (yaw, d);
            }
        }
        best.0
    }
}
