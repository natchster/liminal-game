//! The menus (`ui/menu.rs`, `docs/UI.md` design 3): the title is the lobby,
//! live, with a short list worked by keys or by the mouse; a new descent
//! over a save asks first; Quit quits.
mod common;
use common::new_game;
use game::ui::menu::Screen;
use game::Game;

const W: f64 = 1280.0;
const H: f64 = 800.0;

fn frame(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        let _ = g.hud_layers(W, H, 1.0);
    }
}
fn press(g: &mut Game, k: &str) {
    g.key_down(k, false);
    g.key_up(k);
}
fn labels(g: &Game) -> Vec<String> {
    g.menu.items.iter().map(|i| i.label.clone()).collect()
}
/// where on screen the line `id` can be clicked
fn spot(g: &mut Game, id: &str) -> Option<(f64, f64)> {
    let _ = g.hud_layers(W, H, 1.0);
    for y in (0..H as i32).step_by(3) {
        for x in (0..(W / 2.0) as i32).step_by(6) {
            if g.hud_hit_test(x as f64, y as f64) == id {
                return Some((x as f64, y as f64));
            }
        }
    }
    None
}

#[test]
fn the_title_is_the_place_and_a_short_list() {
    let mut g = new_game();
    assert_eq!(g.state, "menu");
    assert_eq!(g.menu.screen, Some(Screen::Title));
    assert!(g.ui.overlay.has_class("hidden"), "the old card is up over the title");
    // a fresh install: no Continue
    assert_eq!(labels(&g), ["New descent", "Settings", "Quit"]);
    // the view is alive: the camera turns slowly over a few seconds
    frame(&mut g, 1);
    let yaw0 = g.camera.rot().y;
    frame(&mut g, 240);
    let yaw1 = g.camera.rot().y;
    assert!((yaw1 - yaw0).abs() > 0.01 && (yaw1 - yaw0).abs() < 0.4, "the title view turned {:.3} rad in 4 s", yaw1 - yaw0);
    // the HUD is not drawn over it
    let _ = g.hud_layers(W, H, 1.0);
    for name in ["level", "sanity", "stamina", "battery", "bottles"] {
        assert!(!g.hud_ui.painted.contains(&name), "{name} drawn on the title");
    }
    // keys: down, up (wrapping), Quit
    press(&mut g, "ArrowDown");
    assert_eq!(g.menu.sel, 1);
    press(&mut g, "KeyW");
    press(&mut g, "KeyW");
    assert_eq!(g.menu.sel, 2, "the list does not wrap round (up from the top is the bottom)");
    press(&mut g, "Enter");
    assert!(g.want_quit, "Quit did not quit");
}

#[test]
fn the_mouse_chooses_by_pointing_and_starts_by_clicking() {
    let mut g = new_game();
    let (qx, qy) = spot(&mut g, "menu:quit").expect("Quit is not clickable");
    assert!(qx < W * 0.3 && qy > H * 0.6, "the list is not at the lower left ({qx}, {qy})");
    assert!(g.menu_hover(qx, qy), "pointing at Quit did not choose it");
    assert_eq!(g.menu.items[g.menu.sel].id, "menu:quit");
    let (nx, ny) = spot(&mut g, "menu:new").expect("New descent is not clickable");
    g.menu_hover(nx, ny);
    g.click(&g.hud_hit_test(nx, ny));
    assert_eq!(g.state, "playing", "a click on New descent did not start it");
    assert!(g.menu.screen.is_none());
    assert_eq!(g.want_pointer_lock, Some(true));
}

#[test]
fn a_new_descent_over_a_save_asks_first() {
    let mut g = new_game();
    press(&mut g, "Enter"); // New descent (no save yet)
    assert_eq!(g.state, "playing");
    g.notebook.insert("f1".into());
    g.save_game();
    // back at the title (a relaunch)
    let run = g.run_seed;
    g.state = "menu".into();
    g.open_menu(Screen::Title);
    assert_eq!(labels(&g), ["Continue", "New descent", "Settings", "Quit"]);
    assert_eq!(g.menu.items[0].detail, "level 1");
    let her = game::story::notebook::notebook(run).into_iter().find(|(_, p)| p.id == "f1").map(|(_, p)| p.thread).unwrap();
    assert!(g.menu.her_line.contains(her), "her last line is not on the title: {:?}", g.menu.her_line);
    // New descent asks, on its safe answer; Escape and Back both go back
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    assert_eq!(g.menu.screen, Some(Screen::ConfirmNew));
    assert_eq!(g.menu.items[g.menu.sel].id, "menu:back", "the question opened on its unsafe answer");
    assert!(g.menu.note.contains("level 1"), "{}", g.menu.note);
    press(&mut g, "Escape");
    assert_eq!(g.menu.screen, Some(Screen::Title));
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    press(&mut g, "Enter"); // Back
    assert_eq!(g.menu.screen, Some(Screen::Title));
    assert!(g.load_game().is_some(), "Back lost the save");
    // Continue picks the save up
    press(&mut g, "Enter");
    assert_eq!(g.state, "playing");
    assert_eq!(g.run_seed, run);
    assert!(g.notebook.contains("f1"));
}

#[test]
fn the_pause_stops_the_world_and_leads_to_settings_and_back() {
    let mut g = new_game();
    press(&mut g, "Enter"); // New descent
    frame(&mut g, 30);
    // Escape: paused; nothing moves
    press(&mut g, "Escape");
    assert_eq!(g.state, "paused");
    assert_eq!(g.live_menu(), Some(Screen::Pause));
    assert_eq!(labels(&g), ["Resume", "Settings", "Controls", "Leave for the title"]);
    let (t, pos) = (g.director.t, g.player.pos);
    let ents: Vec<_> = g.entities.iter().map(|e| e.pos()).collect();
    g.key_down("KeyW", false); // (in a list, W moves the choice up)
    frame(&mut g, 120);
    g.key_up("KeyW");
    g.menu.sel = 0;
    assert_eq!((g.director.t, g.player.pos), (t, pos), "the world moved while paused");
    assert!(g.entities.iter().map(|e| e.pos()).eq(ents.into_iter()), "a creature moved while paused");
    let _ = g.hud_layers(W, H, 1.0);
    assert!(!g.hud_ui.painted.contains(&"stamina"), "the HUD is drawn over the pause list");
    // Settings from the pause list, a change, and back to the pause list
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    assert_eq!(g.live_menu(), Some(Screen::Settings));
    press(&mut g, "ArrowRight"); // look speed
    assert!((g.settings.look - 1.05).abs() < 1e-9);
    assert_eq!(g.menu.items[0].detail, "1.05", "the line did not show the new value");
    assert!(g.storage_get(game::session::settings::SETTINGS_KEY).is_some(), "the change was not kept");
    // the brightness line shows its mark
    g.menu.sel = g.menu.items.iter().position(|i| i.id == "set:brightness").unwrap();
    let _ = g.hud_layers(W, H, 1.0);
    assert!(g.hud_ui.painted.contains(&"menu-mark"));
    press(&mut g, "ArrowRight");
    let _ = g.hud_layers(W, H, 1.0);
    assert!(g.hud_ui.painted.contains(&"fill") && g.settings.brightness == 1);
    press(&mut g, "Escape");
    assert_eq!(g.live_menu(), Some(Screen::Pause), "Escape in Settings did not go back to the pause list");
    // Controls: Enter on a line, then a key; it is that action's now
    press(&mut g, "ArrowDown");
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    assert_eq!(g.live_menu(), Some(Screen::Controls));
    let k = g.menu.items.iter().position(|i| i.id == "bind:drink").unwrap();
    g.menu.sel = k;
    press(&mut g, "Enter");
    assert_eq!(g.menu.items[k].detail, "Press a key…");
    press(&mut g, "KeyX");
    assert_eq!(g.menu.items[k].detail, "X / 1");
    assert_eq!(g.settings.keys(game::session::input::Act::Drink)[0], "KeyX");
    press(&mut g, "Escape");
    press(&mut g, "Escape"); // the pause list: Escape resumes
    assert_eq!(g.state, "playing");
    assert!(g.live_menu().is_none());
}

#[test]
fn leaving_for_the_title_keeps_the_notebook() {
    let mut g = new_game();
    press(&mut g, "Enter");
    frame(&mut g, 10);
    g.notebook.insert("f1".into());
    press(&mut g, "Escape");
    g.menu_choose("menu:leave");
    assert_eq!(g.live_menu(), Some(Screen::ConfirmLeave));
    assert_eq!(g.menu.items[g.menu.sel].id, "menu:back");
    g.menu_choose("menu:leave-yes");
    assert_eq!((g.state.as_str(), g.live_menu()), ("menu", Some(Screen::Title)));
    assert_eq!(g.menu.items[0].label, "Continue");
    assert!(g.load_game().expect("saved").notebook.contains(&"f1".to_string()));
    press(&mut g, "Enter");
    assert_eq!(g.state, "playing");
    assert!(g.notebook.contains("f1"));
}

#[test]
fn death_is_black_a_line_and_wake_up() {
    let mut g = new_game();
    press(&mut g, "Enter");
    // New descent: the way in from the title (docs/CUTSCENES.md), the card
    // once your hands are back
    frame(&mut g, 30);
    assert!(g.hands_taken() && g.level_card.is_none(), "the way in, its card after");
    for _ in 0..600 {
        if !g.hands_taken() {
            break;
        }
        frame(&mut g, 1);
    }
    assert!(g.level_card.is_some(), "no level card on arrival");
    g.on_death("entity");
    assert_eq!(g.state, "dead");
    // the moment that leads to the card cannot be skipped by a key
    press(&mut g, "KeyE");
    press(&mut g, "KeyR");
    assert_eq!(g.state, "dead", "a key skipped the death before its card was up");
    frame(&mut g, 100);
    assert_eq!(g.live_menu(), Some(Screen::Dead), "no death card after 1.7 s");
    assert!(!g.menu.heading.is_empty() && !g.menu.heading.contains("seconds"), "the card: {:?}", g.menu.heading);
    assert_eq!(labels(&g), ["Wake up", "Leave for the title"]);
    // any of the keys that go on wakes you, with empty hands
    g.input.crouch_latched = true;
    g.journal_open = true;
    press(&mut g, "KeyR");
    assert_eq!(g.state, "playing");
    assert!(g.live_menu().is_none());
    assert!(!g.input.crouch_latched && !g.journal_open, "the last life's crouch or notebook carried over");
    // again, and this time leave for the title
    frame(&mut g, 10);
    g.on_death("entity");
    frame(&mut g, 100);
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    assert_eq!((g.state.as_str(), g.live_menu()), ("menu", Some(Screen::Title)));
    assert_eq!(g.menu.items[0].label, "Continue");
}
