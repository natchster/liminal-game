//! HUD: drive it through a few states, render its layers, and composite them
//! on the CPU the way engine/src/composite.metal does, over a flat "scene"
//! colour. Writes target/hud-<state>.png, the element model as
//! target/hud-<state>.json, and each HUD canvas of the "hud" state as
//! target/hud-canvas-<n>-<name>.png, for visual inspection.
use engine::math::Vec3;
use game::ui::model::{Element, HudLayer, LayerBlend};
use game::Game;

const W: f64 = 1280.0;
const H: f64 = 800.0;
const DPR: f64 = 2.0;
/// the flat stand-in for the 3D view
const BG: [f32; 3] = [0x6b as f32 / 255.0, 0x5e as f32 / 255.0, 0x34 as f32 / 255.0];

fn out_dir() -> std::path::PathBuf {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
    std::fs::create_dir_all(&p).unwrap();
    p
}

// ------------------------------------------------------------ CPU compositor
fn blendf(mode: LayerBlend, cb: f32, cs: f32) -> f32 {
    match mode {
        LayerBlend::Multiply => cb * cs,
        LayerBlend::Screen => cb + cs - cb * cs,
        LayerBlend::Add => (cb + cs).min(1.0),
        LayerBlend::Overlay => {
            if cb <= 0.5 {
                2.0 * cb * cs
            } else {
                1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
            }
        }
        LayerBlend::SoftLight => {
            let d = if cb <= 0.25 { ((16.0 * cb - 12.0) * cb + 4.0) * cb } else { cb.sqrt() };
            if cs <= 0.5 {
                cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
            } else {
                cb + (2.0 * cs - 1.0) * (d - cb)
            }
        }
        LayerBlend::ColorDodge => {
            if cs >= 1.0 {
                1.0
            } else {
                (cb / (1.0 - cs).max(1e-5)).min(1.0)
            }
        }
        LayerBlend::Darken => cb.min(cs),
        LayerBlend::Lighten => cb.max(cs),
        LayerBlend::Normal => cs,
    }
}
fn over(mode: LayerBlend, dst: &mut [f32; 3], cs: [f32; 3], a: f32) {
    for i in 0..3 {
        let m = blendf(mode, dst[i], cs[i]);
        dst[i] += (m - dst[i]) * a;
    }
}
/// composite `layers` over BG into an RGBA8 image of wpx×hpx
pub fn composite(layers: &[HudLayer], wpx: usize, hpx: usize, skip_grain: bool) -> Vec<u8> {
    let mut img: Vec<[f32; 3]> = vec![BG; wpx * hpx];
    for l in layers {
        match l {
            HudLayer::Fill { color, blend } => {
                for p in img.iter_mut() {
                    over(*blend, p, [color[0], color[1], color[2]], color[3]);
                }
            }
            HudLayer::Radial { center, radius, stops, blend, opacity } => {
                for y in 0..hpx {
                    for x in 0..wpx {
                        let (u, v) = ((x as f32 + 0.5) / wpx as f32, (y as f32 + 0.5) / hpx as f32);
                        let t = (((u - center[0]) / radius[0].max(1e-5)).powi(2) + ((v - center[1]) / radius[1].max(1e-5)).powi(2)).sqrt();
                        let n = stops.len();
                        let mut c = stops[0].1;
                        if t >= stops[n - 1].0 {
                            c = stops[n - 1].1;
                        } else {
                            for k in 0..n - 1 {
                                if t >= stops[k].0 && t < stops[k + 1].0 {
                                    let f = (t - stops[k].0) / (stops[k + 1].0 - stops[k].0).max(1e-5);
                                    let (a, b) = (stops[k].1, stops[k + 1].1);
                                    let al = a[3] + (b[3] - a[3]) * f;
                                    let m = |i: usize| if al > 0.0 { (a[i] * a[3] + (b[i] * b[3] - a[i] * a[3]) * f) / al } else { 0.0 };
                                    c = [m(0), m(1), m(2), al];
                                    break;
                                }
                            }
                        }
                        over(*blend, &mut img[y * wpx + x], [c[0], c[1], c[2]], c[3] * opacity);
                    }
                }
            }
            HudLayer::Canvas { canvas, rect, opacity, blend, tile, offset, smooth, .. } => {
                let c = canvas.borrow();
                let (cw, ch) = (c.width() as usize, c.height() as usize);
                if skip_grain && cw == 480 && ch == 300 {
                    continue;
                }
                let px = c.premultiplied();
                let x0 = ((rect[0] * wpx as f32).floor().max(0.0)) as usize;
                let y0 = ((rect[1] * hpx as f32).floor().max(0.0)) as usize;
                let x1 = (((rect[0] + rect[2]) * wpx as f32).ceil() as usize).min(wpx);
                let y1 = (((rect[1] + rect[3]) * hpx as f32).ceil() as usize).min(hpx);
                let fetch = |ix: i64, iy: i64| -> [f32; 4] {
                    let (ix, iy) = (ix.rem_euclid(cw as i64) as usize, iy.rem_euclid(ch as i64) as usize);
                    let k = (iy * cw + ix) * 4;
                    [px[k] as f32 / 255.0, px[k + 1] as f32 / 255.0, px[k + 2] as f32 / 255.0, px[k + 3] as f32 / 255.0]
                };
                for y in y0..y1 {
                    for x in x0..x1 {
                        let (u, v) = ((x as f32 + 0.5) / wpx as f32, (y as f32 + 0.5) / hpx as f32);
                        let lu = (u - rect[0]) / rect[2];
                        let lv = (v - rect[1]) / rect[3];
                        if !(0.0..1.0).contains(&lu) || !(0.0..1.0).contains(&lv) {
                            continue;
                        }
                        let (su, sv) = (lu * tile[0] + offset[0], lv * tile[1] + offset[1]);
                        let s = if *smooth {
                            let (fx, fy) = (su * cw as f32 - 0.5, sv * ch as f32 - 0.5);
                            let (ix, iy) = (fx.floor() as i64, fy.floor() as i64);
                            let (tx, ty) = (fx - fx.floor(), fy - fy.floor());
                            let (a, b, c2, d) = (fetch(ix, iy), fetch(ix + 1, iy), fetch(ix, iy + 1), fetch(ix + 1, iy + 1));
                            let mut o = [0f32; 4];
                            for i in 0..4 {
                                o[i] = (a[i] * (1.0 - tx) + b[i] * tx) * (1.0 - ty) + (c2[i] * (1.0 - tx) + d[i] * tx) * ty;
                            }
                            o
                        } else {
                            fetch((su * cw as f32).floor() as i64, (sv * ch as f32).floor() as i64)
                        };
                        if s[3] <= 0.0 {
                            continue;
                        }
                        over(*blend, &mut img[y * wpx + x], [s[0] / s[3], s[1] / s[3], s[2] / s[3]], s[3] * opacity);
                    }
                }
            }
            HudLayer::Aberration { .. } => {}
        }
    }
    img.iter()
        .flat_map(|p| {
            [
                (p[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                (p[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                (p[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                255,
            ]
        })
        .collect()
}

// ------------------------------------------------------------ element-model dump
fn rec(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
fn el_json(e: &Element) -> String {
    let cls = e.classes.iter().map(|c| rec(c)).collect::<Vec<_>>().join(",");
    let st = e.style.iter().map(|(k, v)| format!("{}:{}", rec(k), rec(v))).collect::<Vec<_>>().join(",");
    let at = e.attrs.iter().map(|(k, v)| format!("{}:{}", rec(k), rec(v))).collect::<Vec<_>>().join(",");
    format!("{{\"text\":{},\"html\":{},\"classes\":[{}],\"style\":{{{}}},\"attrs\":{{{}}}}}", rec(&e.text), rec(&e.html), cls, st, at)
}
const IDS: [&str; 60] = [
    "vignette",
    "weatherTint",
    "safeRoomGlow",
    "aberration",
    "flash",
    "grain",
    "fade",
    "hud",
    "level",
    "seedTag",
    "objective",
    "bottles",
    "batteries",
    "papers",
    "sanity",
    "sanitybar",
    "stamina",
    "staminabar",
    "sanityLabel",
    "staminaLabel",
    "battery",
    "batterybar",
    "batteryLabel",
    "compass",
    "compassArrow",
    "compassLabel",
    "dot",
    "prompt",
    "msg",
    "omen",
    "crouchInd",
    "stairHud",
    "loreMedal",
    "survivorMedal",
    "storytellerMedal",
    "completionistMedal",
    "ironwillMedal",
    "achievementFlash",
    "debug",
    "journal",
    "journal-title",
    "journal-prev",
    "journal-idx",
    "journal-next",
    "journal-entry-title",
    "journal-entry-body",
    "journal-hint",
    "overlay",
    "overlay-title",
    "overlay-sub",
    "overlay-controls",
    "overlay-buttons",
    "btn-continue",
    "btn-new",
    "menu-medals",
    "overlay-hint",
    "",
    "",
    "",
    "",
];
fn ui_json(g: &mut Game) -> String {
    let mut parts = vec![];
    for id in IDS.iter().filter(|i| !i.is_empty()) {
        let e = g.ui.by_id(id).unwrap().clone();
        parts.push(format!("{}:{}", rec(id), el_json(&e)));
    }
    let d = &g.ui;
    let mm = [
        ("archivist", &d.menu_medal_archivist),
        ("survivor", &d.menu_medal_survivor),
        ("storyteller", &d.menu_medal_storyteller),
        ("completionist", &d.menu_medal_completionist),
        ("ironwill", &d.menu_medal_ironwill),
    ]
    .iter()
    .map(|(k, e)| format!("{}:{}", rec(k), el_json(e)))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        "{{\"elements\":{{{}}},\"menuMedals\":{{{}}},\"body\":{},\"bg\":\"rgb({},{},{})\"}}",
        parts.join(","),
        mm,
        el_json(&d.body),
        (BG[0] * 255.0).round(),
        (BG[1] * 255.0).round(),
        (BG[2] * 255.0).round()
    )
}

// ------------------------------------------------------------ driving
fn game() -> Game {
    let mut g = Game::new_bare();
    g.view_width = W;
    g.view_height = H;
    g.scale_factor = DPR;
    g.init_hud();
    g
}
/// `frames` frames of the HUD: update_hud (when playing), update_grain, layers
fn run(g: &mut Game, frames: usize, playing: bool) {
    for _ in 0..frames {
        g.now_ms += 1000.0 / 60.0;
        g.run_scheduled();
        if playing {
            g.update_hud(1.0 / 60.0);
            g.update_grain(1.0 / 60.0);
        }
        let _ = g.hud_layers(W, H, DPR);
    }
}
/// every canvas layer of the frame, un-premultiplied, as PNGs
fn dump_canvases(g: &mut Game, prefix: &str) {
    let layers = g.hud_layers(W, H, DPR);
    let names = g.hud_ui.painted.clone();
    for (i, (l, name)) in layers.iter().zip(names.iter()).enumerate() {
        let HudLayer::Canvas { canvas, .. } = l else { continue };
        let c = canvas.borrow();
        let mut px = c.premultiplied().to_vec();
        for p in px.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a > 0 && a < 255 {
                for k in 0..3 {
                    p[k] = ((p[k] as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        engine::png::write_rgba(
            out_dir().join(format!("{prefix}-{i:02}-{name}.png")).to_str().unwrap(),
            c.width() as u32,
            c.height() as u32,
            &px,
        )
        .unwrap();
    }
}
/// render the current state: PNG of the composite + the element model as JSON
fn shoot(g: &mut Game, name: &str) -> Vec<u8> {
    let layers = g.hud_layers(W, H, DPR);
    let (wp, hp) = ((W * DPR) as usize, (H * DPR) as usize);
    let img = composite(&layers, wp, hp, true);
    let dir = out_dir();
    engine::png::write_rgba(dir.join(format!("hud-{name}.png")).to_str().unwrap(), wp as u32, hp as u32, &img).unwrap();
    std::fs::write(dir.join(format!("hud-{name}.json")), ui_json(g)).unwrap();
    img
}
/// the HUD as it is a moment into a level (bars, items, compass, medals...)
fn playing_state(g: &mut Game) {
    g.state = "playing".into();
    g.hide_overlay();
    g.player.pos = Vec3::new(0.0, 0.0, 0.0);
    g.player.yaw = 0.3;
    g.player.stamina = 18.0;
    g.player.sanity = 24.0;
    g.player.battery = 62.0;
    g.player.flashlight_on = true;
    g.player.bottles = 2;
    g.player.batteries = 1;
    g.player.papers = 4;
    g.player.crouch = true;
    g.lore_found.extend([3usize, 4, 7]);
    g.exit_data = Some(game::base::records::ExitData { pos: Vec3::new(14.0, 0.0, -20.0), ..Default::default() });
    g.ui.level.set_text("LEVEL 3");
    g.ui.seed_tag.set_text("SEED 0x1a2b3c4d  ·  WET");
    g.ui.objective.set_text("STAY OUT OF THE DARK");
    g.ui.weather_tint.add_class("wet");
    g.ui.vignette.set_opacity(0.35);
    g.lore_archivist_shown = true;
    g.survivor_shown = true;
    g.ironwill_shown = true;
    g.refresh_lore_medal();
    g.refresh_survivor_medal();
    g.refresh_ironwill_medal();
    g.refresh_storyteller_medal();
    g.refresh_completionist_medal();
    g.interact(Vec3::new(0.5, 0.0, 0.5), 2.0, "[E] OPEN THE DOOR", |_| true, |_| {});
    g.show_msg("FLOOR 3 — THE WALLS ARE THINNER HERE", 30.0);
    g.ui.omen.set_text("Far below you, somebody is crying.");
    g.ui.omen.set_opacity(0.85);
}

#[test]
fn hud_states() {
    // ---- the title menu
    let mut g = game();
    g.state = "menu".into();
    g.update_hud(1.0 / 60.0);
    g.show_overlay("THE BACKROOMS", "You noclipped out of reality. Mono-yellow wallpaper, the hum of fluorescent lights, endless empty rooms. Find the exit door before it finds you.", "CLICK TO ENTER");
    g.ui.btn_continue.set_display("inline-block");
    g.ui.menu_medal_archivist.add_class("earned");
    g.ui.menu_medal_survivor.add_class("earned");
    run(&mut g, 3, false);
    g.now_ms = 0.0; // blink / shimmer at their first keyframe
    shoot(&mut g, "menu");
    // the buttons answer clicks; anywhere else on the menu is the overlay
    let mut found_new = false;
    for y in (300..700).step_by(2) {
        for x in (500..800).step_by(4) {
            if g.hud_hit_test(x as f64, y as f64) == "btn-new" {
                found_new = true;
            }
        }
    }
    assert!(found_new, "btn-new is clickable");
    assert_eq!(g.hud_hit_test(5.0, 5.0), "overlay");

    // ---- playing: meters, items, compass, toast, omen, medals, prompt
    let mut g = game();
    playing_state(&mut g);
    run(&mut g, 150, true);
    assert!(g.ui.prompt.text.contains("OPEN THE DOOR"));
    assert!(g.ui.sanity.has_class("low") && g.ui.stamina.has_class("low"));
    assert!(g.ui.body.has_class("crit-sanity"));
    let img = shoot(&mut g, "hud");
    dump_canvases(&mut g, "hud-canvas");
    assert!(img.chunks_exact(4).any(|p| p[0] > 200), "HUD text drawn");
    assert_eq!(g.hud_hit_test(640.0, 400.0), "");

    // ---- a medal earned: the toast, the flash, the replay animation
    let mut g = game();
    playing_state(&mut g);
    g.storyteller_shown = false;
    g.fire_storyteller_achievement();
    assert!(g.ui.achievement_flash.has_class("on"));
    run(&mut g, 20, true);
    shoot(&mut g, "achievement");
    run(&mut g, 60, true);
    assert!(!g.ui.achievement_flash.has_class("on"), "flash cleared after 0.9 s");
    g.replay_medal_highlight();
    run(&mut g, 20, true);
    assert!(g.ui.lore_medal.has_class("medal-replay"));
    shoot(&mut g, "medal-replay");

    // ---- the journal: newest entry, the cover, the medals page
    let mut g = game();
    playing_state(&mut g);
    g.lore_found.extend([2usize, 5, 8, 14]);
    g.lore_arcs_completed.insert("M.'S RULES".into());
    g.lore_archivist_shown = false;
    g.toggle_journal();
    assert!(g.journal_open && g.ui.journal.has_class("on"));
    run(&mut g, 30, true);
    assert!(g.ui.journal_entry_title.text.contains("THE RITUAL 2/3"), "{}", g.ui.journal_entry_title.text);
    shoot(&mut g, "journal");
    // prev / next answer clicks while enabled
    let mut hits = std::collections::HashSet::new();
    for y in (200..600).step_by(2) {
        for x in (250..1030).step_by(3) {
            hits.insert(g.hud_hit_test(x as f64, y as f64));
        }
    }
    assert!(hits.contains("journal-prev") && !hits.contains("journal-next"), "{hits:?}");
    g.journal_cursor = 0.0;
    g.journal_render();
    run(&mut g, 60, true);
    assert_eq!(g.ui.journal_entry_title.text, "INDEX");
    shoot(&mut g, "journal-cover");
    g.journal_next();
    run(&mut g, 30, true);
    assert_eq!(g.ui.journal_entry_title.text, "MEDALS");
    shoot(&mut g, "journal-medals");
    g.toggle_journal();
    assert!(!g.ui.journal.has_class("on"));

    // ---- the [K] debug panel
    let mut g = game();
    playing_state(&mut g);
    g.toggle_debug();
    run(&mut g, 10, true);
    g.update_debug();
    assert!(g.ui.debug.text.starts_with("── DEBUG [K] ──\nlevel     1\n"));
    shoot(&mut g, "debug");

    // ---- the screen effects: vignette inset glow + filter, aberration,
    // inline flash / fade backgrounds, the safe-room glow
    let mut g = game();
    playing_state(&mut g);
    g.ui.vignette.set_opacity(0.6);
    g.ui.vignette.set_style("boxShadow", "inset 0 0 220px 40px rgba(180, 220, 255, 0.5)");
    g.ui.vignette.set_style("filter", "hue-rotate(20.0deg) saturate(1.40) contrast(1.20)");
    g.ui.flash.set_style(
        "background",
        "radial-gradient(ellipse at center, rgba(255,255,240,0.7) 0%, rgba(255,240,200,0.35) 60%, transparent 100%)",
    );
    g.ui.flash.set_opacity(0.4);
    g.ui.fade.set_style("background", "rgb(97,96,94)");
    g.ui.fade.set_opacity(0.3);
    g.ui.safe_room_glow.add_class("on");
    g.ui.weather_tint.remove_class("wet");
    g.ui.weather_tint.add_class("haunted");
    run(&mut g, 90, true);
    g.ui.aberration.set_opacity(0.8);
    run(&mut g, 60, false);
    shoot(&mut g, "effects");

    // ---- death / pause: the overlay without its buttons
    let mut g = game();
    playing_state(&mut g);
    g.state = "dead".into();
    g.ui.flash.set_opacity(1.0);
    g.show_overlay("IT FOUND YOU", "You survived 42 seconds on level 3. The hum never stops.", "PRESS R TO WAKE UP");
    assert_eq!(g.ui.overlay_buttons.style("display"), Some("none"));
    run(&mut g, 40, false);
    g.now_ms = 0.0;
    shoot(&mut g, "dead");
}

#[test]
fn hud_logic() {
    let mut g = game();
    // `whisper_for` / `lore_arc_for` / `lore_note_boosted`
    assert_eq!(g.lore_arc_for(8), Some(0));
    assert_eq!(g.lore_arc_for(0), None);
    let wf = g.whisper_for(g.lore_arc_for(38), Some(38));
    assert_eq!(wf.text.as_deref(), Some("...  stairwell, landing 30  ..."));
    assert!((wf.duration - 1.2).abs() < 1e-9 && (wf.opacity - 0.35).abs() < 1e-9);
    assert_eq!(wf.glow.as_deref(), Some("#d8d2c4"));
    assert_eq!(wf.glow_shadow.as_deref(), Some("0 0 6.0px rgba(216, 210, 196, 0.45)"));
    assert!(!g.lore_note_boosted(8));
    g.lore_found.insert(7);
    assert!(g.lore_note_boosted(8) && g.lore_note_boosted(24) && !g.lore_note_boosted(7));
    // `pick_text` draws from its pool
    let s = g.pick_text("objectives");
    assert!(game::ui::hud::text_pool("objectives").unwrap().contains(&s.as_str()));
    // the journal order: cover, medals page, notes, epilogue
    g.lore_found.extend([1usize, 3]);
    assert_eq!(g.journal_entries_in_order(), vec![-2, 1, 3, 7]);
    g.ironwill_shown = true;
    g.lore_archivist_shown = true;
    assert_eq!(g.journal_entries_in_order(), vec![-2, -3, 1, 3, 7, -1]);
    let (t, b) = g.current_epilogue();
    assert_eq!(t, "EPILOGUE");
    assert!(b.ends_with("Every page you found is now inside you.\nYou never fell. The dark never touched you long enough."));
    assert!(g.medals_page_html().starts_with("Medals of this run: <span style=\"color:#f8e0a0\">2</span> of 5."));
    // completionist follows the other three after 4.5 s
    g.survivor_shown = true;
    g.fire_storyteller_achievement();
    assert!(!g.completionist_shown);
    g.now_ms += 4600.0;
    g.run_scheduled();
    assert!(g.completionist_shown && g.ui.completionist_medal.has_class("on"));
    // `try_interact` uses the nearest enabled interactable, once
    let used = std::rc::Rc::new(std::cell::Cell::new(0));
    let u2 = used.clone();
    g.state = "playing".into();
    g.interact(Vec3::new(0.3, 0.0, 0.0), 1.0, "[E] X", |_| true, move |_| u2.set(u2.get() + 1));
    g.try_interact();
    g.try_interact();
    assert_eq!(used.get(), 2);
    // animators run inside `update_hud`
    let n = std::rc::Rc::new(std::cell::Cell::new(0));
    let n2 = n.clone();
    g.animate(move |_g, _dt, _t| n2.set(n2.get() + 1));
    g.update_hud(0.016);
    g.update_hud(0.016);
    assert_eq!(n.get(), 2);
    // the grain canvas refreshes at ~18 fps
    let v0 = g.grain_canvas.as_ref().unwrap().borrow().version;
    g.update_grain(0.06);
    assert!(g.grain_canvas.as_ref().unwrap().borrow().version > v0);
}

#[test]
fn hud_frame_cost() {
    let mut g = game();
    playing_state(&mut g);
    run(&mut g, 30, true);
    let t = std::time::Instant::now();
    for i in 0..60 {
        g.player.yaw += 0.01;
        g.player.stamina = 50.0 + i as f64 * 0.1;
        run(&mut g, 1, true);
    }
    let hud_ms = t.elapsed().as_secs_f64() * 1000.0 / 60.0;
    g.toggle_debug();
    let t = std::time::Instant::now();
    for _ in 0..20 {
        g.fps += 1.0;
        g.update_debug();
        run(&mut g, 1, true);
    }
    let dbg_ms = t.elapsed().as_secs_f64() * 1000.0 / 20.0;
    eprintln!("hud frame {hud_ms:.2} ms, with debug {dbg_ms:.2} ms");
}
