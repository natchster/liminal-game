//! The HUD's element model. The game drives its HUD and overlay layers by
//! setting element text, inline styles and classes (`set_text`,
//! `set_opacity`, `toggle_class`, ...), and the HUD renderer (`ui::draw`)
//! lays these `Element`s out and draws them with the HUD's CSS rules.

use std::collections::{BTreeMap, BTreeSet};

/// One element: its text/HTML, classes, inline styles and attributes.
#[derive(Clone, Debug, Default)]
pub struct Element {
    pub text: String,
    /// `inner_html` when set (takes precedence over `text` when non-empty)
    pub html: String,
    pub classes: BTreeSet<String>,
    /// inline style properties in CSS names (`opacity`, `background`, `box-shadow`, `transform`, ...)
    pub style: BTreeMap<String, String>,
    pub attrs: BTreeMap<String, String>,
    /// bumped on every change (renderer redraws when it changes)
    pub version: u64,
}

impl Element {
    /// Set the text content (clears any HTML)
    pub fn set_text(&mut self, s: &str) {
        if self.text != s || !self.html.is_empty() {
            self.text = s.to_string();
            self.html.clear();
            self.version += 1;
        }
    }
    /// Set the inner HTML (inline markup such as `<span style=...>` and `<br>`)
    pub fn set_html(&mut self, s: &str) {
        if self.html != s {
            self.html = s.to_string();
            self.version += 1;
        }
    }
    /// Set an inline style property (name in camel case or CSS form); an
    /// empty value removes it
    pub fn set_style(&mut self, prop: &str, v: &str) {
        let k = css_name(prop);
        if v.is_empty() {
            if self.style.remove(&k).is_some() {
                self.version += 1;
            }
            return;
        }
        if self.style.get(&k).map(|x| x == v) != Some(true) {
            self.style.insert(k, v.to_string());
            self.version += 1;
        }
    }
    /// Set a style property to a number (formatted with `fmt_num`)
    pub fn set_style_num(&mut self, prop: &str, v: f64) {
        self.set_style(prop, &crate::base::numeric::fmt_num(v));
    }
    /// Set the `opacity` style
    pub fn set_opacity(&mut self, v: f64) {
        self.set_style_num("opacity", v);
    }
    pub fn style(&self, prop: &str) -> Option<&str> {
        self.style.get(&css_name(prop)).map(|s| s.as_str())
    }
    /// numeric value of a style property (its leading number), if set
    pub fn style_num(&self, prop: &str) -> Option<f64> {
        self.style(prop).and_then(parse_float_prefix)
    }
    pub fn opacity(&self) -> Option<f64> {
        self.style_num("opacity")
    }
    /// Add class `c`
    pub fn add_class(&mut self, c: &str) {
        if self.classes.insert(c.to_string()) {
            self.version += 1;
        }
    }
    /// Remove class `c`
    pub fn remove_class(&mut self, c: &str) {
        if self.classes.remove(c) {
            self.version += 1;
        }
    }
    /// Add class `c` when `on`, remove it otherwise
    pub fn toggle_class(&mut self, c: &str, on: bool) {
        if on {
            self.add_class(c)
        } else {
            self.remove_class(c)
        }
    }
    /// Flip class `c`; returns the new state
    pub fn flip_class(&mut self, c: &str) -> bool {
        if self.has_class(c) {
            self.remove_class(c);
            false
        } else {
            self.add_class(c);
            true
        }
    }
    pub fn has_class(&self, c: &str) -> bool {
        self.classes.contains(c)
    }
    pub fn set_attr(&mut self, k: &str, v: &str) {
        if self.attrs.get(k).map(|x| x == v) != Some(true) {
            self.attrs.insert(k.to_string(), v.to_string());
            self.version += 1;
        }
    }
    /// Set the `display` style (`"none"` hides the element)
    pub fn set_display(&mut self, v: &str) {
        self.set_style("display", v);
    }
    pub fn shown(&self) -> bool {
        self.style("display") != Some("none")
    }
}

/// A style property name in CSS form: camel case → kebab-case (each
/// uppercase letter becomes `-` + its lowercase)
pub fn css_name(p: &str) -> String {
    let mut s = String::with_capacity(p.len() + 4);
    for c in p.chars() {
        if c.is_ascii_uppercase() {
            s.push('-');
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
    }
    s
}
/// The longest numeric prefix of `s` (after trimming) as f64, e.g. "12px" →
/// 12; `None` when there is no leading number
pub fn parse_float_prefix(s: &str) -> Option<f64> {
    let s = s.trim();
    let mut end = 0;
    let b = s.as_bytes();
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut seen_e = false;
    while end < b.len() {
        let c = b[end] as char;
        if c.is_ascii_digit() {
            seen_digit = true;
        } else if (c == '-' || c == '+') && (end == 0 || matches!(b[end - 1] as char, 'e' | 'E')) {
        } else if c == '.' && !seen_dot && !seen_e {
            seen_dot = true;
        } else if (c == 'e' || c == 'E') && seen_digit && !seen_e {
            seen_e = true;
        } else {
            break;
        }
        end += 1;
    }
    if !seen_digit {
        return None;
    }
    s[..end].parse::<f64>().ok().or_else(|| s[..end.saturating_sub(1)].parse::<f64>().ok())
}

/// Every HUD element with an id, as a named field (`by_id` maps the ids).
#[derive(Clone, Debug, Default)]
pub struct UiModel {
    // full-screen layers
    pub vignette: Element,
    pub weather_tint: Element,
    pub safe_room_glow: Element,
    pub aberration: Element,
    pub flash: Element,
    pub grain: Element,
    pub fade: Element,
    // HUD
    pub hud: Element,
    pub level: Element,
    pub seed_tag: Element,
    pub objective: Element,
    pub bottles: Element,
    pub batteries: Element,
    pub papers: Element,
    pub sanity: Element,
    pub sanitybar: Element,
    pub stamina: Element,
    pub staminabar: Element,
    pub sanity_label: Element,
    pub stamina_label: Element,
    pub battery: Element,
    pub batterybar: Element,
    pub battery_label: Element,
    pub dot: Element,
    pub prompt: Element,
    pub msg: Element,
    pub omen: Element,
    pub crouch_ind: Element,
    pub stair_hud: Element,
    pub lore_medal: Element,
    pub survivor_medal: Element,
    pub storyteller_medal: Element,
    pub completionist_medal: Element,
    pub ironwill_medal: Element,
    pub achievement_flash: Element,
    pub debug: Element,
    // journal
    pub journal: Element,
    pub journal_title: Element,
    pub journal_prev: Element,
    pub journal_idx: Element,
    pub journal_next: Element,
    pub journal_entry_title: Element,
    pub journal_entry_body: Element,
    pub journal_hint: Element,
    // menu overlay
    pub overlay: Element,
    pub overlay_title: Element,
    pub overlay_sub: Element,
    pub overlay_controls: Element,
    pub overlay_buttons: Element,
    pub btn_continue: Element,
    pub btn_new: Element,
    pub menu_medals: Element,
    /// `.menu-medal[data-m=...]` children of #menu-medals
    pub menu_medal_archivist: Element,
    pub menu_medal_survivor: Element,
    pub menu_medal_storyteller: Element,
    pub menu_medal_completionist: Element,
    pub menu_medal_ironwill: Element,
    pub overlay_hint: Element,
    /// The root element; only its classes are used (e.g. 'playing')
    pub body: Element,
}

impl UiModel {
    /// The HUD in its initial state (initial text, classes and inline styles).
    pub fn new() -> UiModel {
        let mut d = UiModel::default();
        d.level.text = "LEVEL 1".into();
        d.objective.text = "FIND THE EXIT DOOR".into();
        d.sanity_label.text = "SANITY".into();
        d.stamina_label.text = "STAMINA".into();
        d.battery_label.text = "FLASHLIGHT [F]".into();
        d.crouch_ind.text = "— CROUCHING —".into();
        d.lore_medal.text = "◆ ARCHIVIST".into();
        d.survivor_medal.text = "◆ SURVIVOR".into();
        d.storyteller_medal.text = "⛧ STORYTELLER".into();
        d.completionist_medal.text = "★ COMPLETIONIST".into();
        d.ironwill_medal.text = "✦ IRONWILL".into();
        d.journal_title.text = "JOURNAL".into();
        d.journal_prev.text = "◀ PREV".into();
        d.journal_idx.text = "0 / 0".into();
        d.journal_next.text = "NEXT ▶".into();
        d.journal_hint.text = "[J] CLOSE".into();
        d.overlay_title.text = "THE BACKROOMS".into();
        d.overlay_sub.text = "You noclipped out of reality. Mono-yellow wallpaper, the hum of fluorescent lights, endless empty rooms. Find the exit door before it finds you.".into();
        d.overlay_controls.html = "WASD — MOVE &nbsp;&middot;&nbsp; MOUSE — LOOK &nbsp;&middot;&nbsp; SHIFT — RUN &nbsp;&middot;&nbsp; CTRL — CROUCH &nbsp;&middot;&nbsp; F — FLASHLIGHT &nbsp;&middot;&nbsp; E — INTERACT &nbsp;&middot;&nbsp; Q — DROP PAPER &nbsp;&middot;&nbsp; G — DRINK WATER &nbsp;&middot;&nbsp; R — SWAP BATTERY &nbsp;&middot;&nbsp; H — CRY &nbsp;&middot;&nbsp; J — JOURNAL &nbsp;&middot;&nbsp; K — DEBUG &nbsp;&middot;&nbsp; M — MUTE<br>\n  ALMOND WATER RESTORES STAMINA WHEN YOU DRINK IT. A SPARE BATTERY RECHARGES YOUR LIGHT WHEN YOU SWAP IT IN. BOTH ARE CARRIED UNTIL YOU SPEND THEM — SAVE THEM FOR WHEN IT IS BAD. PAPER MARKS YOUR PATH. CRYING IS DESPERATE — IT MAY BE HEARD.".into();
        d.btn_continue.text = "CONTINUE".into();
        d.btn_continue.set_style("display", "none");
        d.btn_new.text = "NEW GAME".into();
        d.menu_medal_archivist.text = "◆ ARCHIVIST".into();
        d.menu_medal_survivor.text = "◆ SURVIVOR".into();
        d.menu_medal_storyteller.text = "⛧ STORYTELLER".into();
        d.menu_medal_completionist.text = "★ COMPLETIONIST".into();
        d.menu_medal_ironwill.text = "✦ IRONWILL".into();
        d.overlay_hint.text = "CLICK TO ENTER".into();
        for el in [&mut d.vignette, &mut d.aberration, &mut d.flash, &mut d.grain, &mut d.fade, &mut d.hud] {
            el.classes.insert("layer".into());
        }
        for el in [&mut d.journal_prev, &mut d.journal_next] {
            el.classes.insert("journal-btn".into());
        }
        for el in [&mut d.btn_continue, &mut d.btn_new] {
            el.classes.insert("menu-btn".into());
        }
        for el in [
            &mut d.menu_medal_archivist,
            &mut d.menu_medal_survivor,
            &mut d.menu_medal_storyteller,
            &mut d.menu_medal_completionist,
            &mut d.menu_medal_ironwill,
        ] {
            el.classes.insert("menu-medal".into());
        }
        d
    }
    /// The element with id `id`
    pub fn by_id(&mut self, id: &str) -> Option<&mut Element> {
        Some(match id {
            "vignette" => &mut self.vignette,
            "weatherTint" => &mut self.weather_tint,
            "safeRoomGlow" => &mut self.safe_room_glow,
            "aberration" => &mut self.aberration,
            "flash" => &mut self.flash,
            "grain" => &mut self.grain,
            "fade" => &mut self.fade,
            "hud" => &mut self.hud,
            "level" => &mut self.level,
            "seedTag" => &mut self.seed_tag,
            "objective" => &mut self.objective,
            "bottles" => &mut self.bottles,
            "batteries" => &mut self.batteries,
            "papers" => &mut self.papers,
            "sanity" => &mut self.sanity,
            "sanitybar" => &mut self.sanitybar,
            "stamina" => &mut self.stamina,
            "staminabar" => &mut self.staminabar,
            "sanityLabel" => &mut self.sanity_label,
            "staminaLabel" => &mut self.stamina_label,
            "battery" => &mut self.battery,
            "batterybar" => &mut self.batterybar,
            "batteryLabel" => &mut self.battery_label,
            "dot" => &mut self.dot,
            "prompt" => &mut self.prompt,
            "msg" => &mut self.msg,
            "omen" => &mut self.omen,
            "crouchInd" => &mut self.crouch_ind,
            "stairHud" => &mut self.stair_hud,
            "loreMedal" => &mut self.lore_medal,
            "survivorMedal" => &mut self.survivor_medal,
            "storytellerMedal" => &mut self.storyteller_medal,
            "completionistMedal" => &mut self.completionist_medal,
            "ironwillMedal" => &mut self.ironwill_medal,
            "achievementFlash" => &mut self.achievement_flash,
            "debug" => &mut self.debug,
            "journal" => &mut self.journal,
            "journal-title" => &mut self.journal_title,
            "journal-prev" => &mut self.journal_prev,
            "journal-idx" => &mut self.journal_idx,
            "journal-next" => &mut self.journal_next,
            "journal-entry-title" => &mut self.journal_entry_title,
            "journal-entry-body" => &mut self.journal_entry_body,
            "journal-hint" => &mut self.journal_hint,
            "overlay" => &mut self.overlay,
            "overlay-title" => &mut self.overlay_title,
            "overlay-sub" => &mut self.overlay_sub,
            "overlay-controls" => &mut self.overlay_controls,
            "overlay-buttons" => &mut self.overlay_buttons,
            "btn-continue" => &mut self.btn_continue,
            "btn-new" => &mut self.btn_new,
            "menu-medals" => &mut self.menu_medals,
            "overlay-hint" => &mut self.overlay_hint,
            _ => return None,
        })
    }
    /// sum of every element's version (cheap "did anything change")
    pub fn version(&self) -> u64 {
        let all = [
            &self.vignette,
            &self.weather_tint,
            &self.safe_room_glow,
            &self.aberration,
            &self.flash,
            &self.grain,
            &self.fade,
            &self.hud,
            &self.level,
            &self.seed_tag,
            &self.objective,
            &self.bottles,
            &self.batteries,
            &self.papers,
            &self.sanity,
            &self.sanitybar,
            &self.stamina,
            &self.staminabar,
            &self.sanity_label,
            &self.stamina_label,
            &self.battery,
            &self.batterybar,
            &self.battery_label,
            &self.dot,
            &self.prompt,
            &self.msg,
            &self.omen,
            &self.crouch_ind,
            &self.stair_hud,
            &self.lore_medal,
            &self.survivor_medal,
            &self.storyteller_medal,
            &self.completionist_medal,
            &self.ironwill_medal,
            &self.achievement_flash,
            &self.debug,
            &self.journal,
            &self.journal_title,
            &self.journal_prev,
            &self.journal_idx,
            &self.journal_next,
            &self.journal_entry_title,
            &self.journal_entry_body,
            &self.journal_hint,
            &self.overlay,
            &self.overlay_title,
            &self.overlay_sub,
            &self.overlay_controls,
            &self.overlay_buttons,
            &self.btn_continue,
            &self.btn_new,
            &self.menu_medals,
            &self.menu_medal_archivist,
            &self.menu_medal_survivor,
            &self.menu_medal_storyteller,
            &self.menu_medal_completionist,
            &self.menu_medal_ironwill,
            &self.overlay_hint,
            &self.body,
        ];
        all.iter().map(|e| e.version).sum()
    }
}

/// How a HUD layer combines with the view under it (CSS mix-blend-mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerBlend {
    Normal,
    Multiply,
    Screen,
    Add,
    Overlay,
    SoftLight,
    ColorDodge,
    Darken,
    Lighten,
}

/// One composited HUD layer over the 3D view, in paint order. Rects
/// are in 0..1 screen units (y down); colours straight-alpha sRGB 0..1.
pub enum HudLayer {
    /// a premultiplied canvas (the HUD text and bars, the grain)
    Canvas {
        canvas: std::rc::Rc<std::cell::RefCell<engine::canvas::Canvas>>,
        id: u64,
        version: u64,
        rect: [f32; 4],
        opacity: f32,
        blend: LayerBlend,
        tile: [f32; 2],
        offset: [f32; 2],
        smooth: bool,
    },
    Fill {
        color: [f32; 4],
        blend: LayerBlend,
    },
    /// CSS radial-gradient (center 0..1, radius (rx, ry) 0..1, ≤4 stops)
    Radial {
        center: [f32; 2],
        radius: [f32; 2],
        stops: Vec<(f32, [f32; 4])>,
        blend: LayerBlend,
        opacity: f32,
    },
    /// the chromatic aberration layer (px offset)
    Aberration {
        offset_px: f32,
        opacity: f32,
    },
}
