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
    assert_eq!(labels(&g), ["New descent", "Quit"]);
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
    assert_eq!(g.menu.sel, 1, "the list does not wrap round (up from the top is the bottom)");
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
    assert_eq!(labels(&g), ["Continue", "New descent", "Quit"]);
    assert_eq!(g.menu.items[0].detail, "level 1");
    let her = game::story::notebook::notebook(run).into_iter().find(|(_, p)| p.id == "f1").map(|(_, p)| p.thread).unwrap();
    assert!(g.menu.her_line.contains(her), "her last line is not on the title: {:?}", g.menu.her_line);
    // New descent asks; Escape and Back both go back
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    assert_eq!(g.menu.screen, Some(Screen::ConfirmNew));
    assert!(g.menu.note.contains("level 1"), "{}", g.menu.note);
    press(&mut g, "Escape");
    assert_eq!(g.menu.screen, Some(Screen::Title));
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter");
    press(&mut g, "ArrowDown");
    press(&mut g, "Enter"); // Back
    assert_eq!(g.menu.screen, Some(Screen::Title));
    assert!(g.load_game().is_some(), "Back lost the save");
    // Continue picks the save up
    press(&mut g, "Enter");
    assert_eq!(g.state, "playing");
    assert_eq!(g.run_seed, run);
    assert!(g.notebook.contains("f1"));
}
