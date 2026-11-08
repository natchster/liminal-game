//! The HUD renderer: the HUD's CSS rules applied to the element model
//! (`crate::ui::model`).
//!
//! Every element of the HUD with its own opacity (or animation) is painted
//! into its own small premultiplied canvas at device resolution; the canvas
//! is only repainted when what it shows changes (a hash of its content), so
//! CSS opacity transitions and keyframe animations are just layer opacities.
//! The full-screen layers (#vignette, #weatherTint, #safeRoomGlow,
//! #aberration, #flash, #fade, #achievementFlash) are radial-gradient / fill
//! ops with their CSS mix-blend-mode; #grain is the 480×300 grain canvas
//! stretched over the view (`image-rendering: pixelated`, overlay blend).
//!
//! Layout: "Courier New" metrics (ascent/descent rounded per size, 0.6em
//! advances, trailing letter-spacing counted), line boxes with floored
//! half-leading, CSS white-space collapsing and greedy line breaking, flex
//! centring of the menu and the journal. Text shadows are blurred alpha masks
//! (σ = blur/2).
use crate::base::game::Game;
use crate::ui::model::{Element, HudLayer, LayerBlend};
use engine::canvas::Canvas;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub(crate) type Rgba = [f64; 4];

// ------------------------------------------------------------ CSS values
/// parse a CSS colour (straight alpha)
pub fn css_color(s: &str) -> Option<Rgba> {
    engine::css::parse_css_color(s)
}
pub(crate) fn col(s: &str) -> Rgba {
    css_color(s).unwrap_or([0.0, 0.0, 0.0, 1.0])
}
pub(crate) fn rgba(r: f64, g: f64, b: f64, a: f64) -> Rgba {
    [r / 255.0, g / 255.0, b / 255.0, a]
}
pub(crate) fn css_rgba(c: Rgba) -> String {
    format!("rgba({},{},{},{})", (c[0] * 255.0).round(), (c[1] * 255.0).round(), (c[2] * 255.0).round(), c[3])
}
fn f32x4(c: Rgba) -> [f32; 4] {
    [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32]
}

/// split at top-level commas (not inside parentheses)
fn split_top(s: &str, sep: char) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if c == sep && depth == 0 {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}
/// whitespace tokens, keeping `rgba(...)` together
fn tokens(s: &str) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if c.is_whitespace() && depth == 0 {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
fn px_len(t: &str) -> Option<f64> {
    let t = t.trim();
    if t == "0" {
        return Some(0.0);
    }
    t.strip_suffix("px").and_then(|v| v.parse().ok())
}

/// One shadow of `text-shadow` / `box-shadow` (CSS px).
#[derive(Clone, Debug, PartialEq)]
pub struct Shadow {
    pub ox: f64,
    pub oy: f64,
    pub blur: f64,
    pub spread: f64,
    pub color: Rgba,
    pub inset: bool,
}
/// `text-shadow` / `box-shadow` lists
pub fn parse_shadows(s: &str) -> Vec<Shadow> {
    if s.trim() == "none" {
        return vec![];
    }
    split_top(s, ',')
        .iter()
        .filter_map(|one| {
            let mut lens = vec![];
            let mut color = [0.0, 0.0, 0.0, 1.0];
            let mut inset = false;
            for t in tokens(one) {
                if t == "inset" {
                    inset = true;
                } else if let Some(v) = px_len(&t) {
                    lens.push(v);
                } else if let Some(c) = css_color(&t) {
                    color = c;
                }
            }
            if lens.len() < 2 {
                return None;
            }
            Some(Shadow {
                ox: lens[0],
                oy: lens[1],
                blur: *lens.get(2).unwrap_or(&0.0),
                spread: *lens.get(3).unwrap_or(&0.0),
                color,
                inset,
            })
        })
        .collect()
}

/// A CSS radial-gradient, resolved for a box of w×h.
#[derive(Clone, Debug, PartialEq)]
pub struct Radial {
    pub center: [f64; 2],
    pub radius: [f64; 2],
    pub stops: Vec<(f64, Rgba)>,
}
/// `radial-gradient([circle|ellipse] [at X% Y%], stops...)` with the default
/// `farthest-corner` size; center/radius in 0..1 of the box.
pub fn parse_radial(s: &str, w: f64, h: f64) -> Option<Radial> {
    let inner = s.trim().strip_prefix("radial-gradient(")?.strip_suffix(')')?;
    let parts = split_top(inner, ',');
    let mut idx = 0;
    let (mut circle, mut cx, mut cy) = (false, 0.5, 0.5);
    let first = parts.first()?.to_ascii_lowercase();
    if first.starts_with("circle") || first.starts_with("ellipse") || first.starts_with("at ") {
        circle = first.starts_with("circle");
        if let Some(at) = first.split("at ").nth(1) {
            let t: Vec<&str> = at.split_whitespace().collect();
            let pos = |v: &str| -> f64 {
                match v {
                    "center" => 0.5,
                    "left" | "top" => 0.0,
                    "right" | "bottom" => 1.0,
                    _ => v.trim_end_matches('%').parse::<f64>().map(|p| p / 100.0).unwrap_or(0.5),
                }
            };
            if !t.is_empty() {
                cx = pos(t[0]);
                cy = if t.len() > 1 { pos(t[1]) } else { 0.5 };
            }
        }
        idx = 1;
    }
    // farthest-corner
    let (dx, dy) = (cx.max(1.0 - cx) * w, cy.max(1.0 - cy) * h);
    let radius = if circle {
        let r = dx.hypot(dy);
        [r / w, r / h]
    } else {
        [dx * std::f64::consts::SQRT_2 / w, dy * std::f64::consts::SQRT_2 / h]
    };
    let n = parts.len() - idx;
    let mut stops: Vec<(Option<f64>, Rgba)> = vec![];
    for p in &parts[idx..] {
        let t = tokens(p);
        let c = css_color(t.first()?)?;
        let pos = t.get(1).and_then(|v| v.strip_suffix('%')).and_then(|v| v.parse::<f64>().ok()).map(|v| v / 100.0);
        stops.push((pos, c));
    }
    // missing positions: first 0, last 1, others spread evenly
    let mut out = vec![];
    for (i, (p, c)) in stops.iter().enumerate() {
        let def = if n <= 1 { 0.0 } else { i as f64 / (n - 1) as f64 };
        let mut t = p.unwrap_or(def);
        if let Some(&(prev, _)) = out.last() {
            if t < prev {
                t = prev;
            }
        }
        out.push((t, *c));
    }
    Some(Radial { center: [cx, cy], radius, stops: out })
}
/// a radial's colour at gradient position t (CSS interpolates premultiplied)
fn radial_at(r: &Radial, t: f64) -> Rgba {
    let s = &r.stops;
    if s.is_empty() {
        return [0.0; 4];
    }
    if t <= s[0].0 {
        return s[0].1;
    }
    for k in 0..s.len() - 1 {
        if t >= s[k].0 && t < s[k + 1].0 {
            let f = (t - s[k].0) / (s[k + 1].0 - s[k].0).max(1e-9);
            let (a, b) = (s[k].1, s[k + 1].1);
            let al = a[3] + (b[3] - a[3]) * f;
            if al <= 0.0 {
                return [0.0; 4];
            }
            let m = |i: usize| (a[i] * a[3] + (b[i] * b[3] - a[i] * a[3]) * f) / al;
            return [m(0), m(1), m(2), al];
        }
    }
    s[s.len() - 1].1
}

/// CSS `filter` colour functions (hue-rotate / saturate / contrast /
/// brightness) as a 3×4 matrix on straight-alpha colours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorMatrix(pub [[f64; 4]; 3]);
impl ColorMatrix {
    pub const ID: ColorMatrix = ColorMatrix([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]]);
    fn then(&self, o: &ColorMatrix) -> ColorMatrix {
        // o ∘ self
        let mut m = [[0.0; 4]; 3];
        for r in 0..3 {
            for c in 0..4 {
                let mut v = if c == 3 { o.0[r][3] } else { 0.0 };
                for k in 0..3 {
                    v += o.0[r][k] * self.0[k][c];
                }
                m[r][c] = v;
            }
        }
        ColorMatrix(m)
    }
    pub fn apply(&self, c: Rgba) -> Rgba {
        let m = &self.0;
        let f = |r: usize| (m[r][0] * c[0] + m[r][1] * c[1] + m[r][2] * c[2] + m[r][3]).clamp(0.0, 1.0);
        [f(0), f(1), f(2), c[3]]
    }
    pub fn parse(s: &str) -> ColorMatrix {
        let mut out = ColorMatrix::ID;
        for t in tokens(s) {
            let Some(open) = t.find('(') else { continue };
            let name = &t[..open];
            let arg = t[open + 1..].trim_end_matches(')').trim();
            let num = |a: &str| -> f64 {
                if let Some(p) = a.strip_suffix('%') {
                    p.parse::<f64>().unwrap_or(100.0) / 100.0
                } else {
                    a.trim_end_matches("deg").parse().unwrap_or(1.0)
                }
            };
            let m = match name {
                "saturate" => {
                    let s = num(arg);
                    ColorMatrix([
                        [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s, 0.0],
                        [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s, 0.0],
                        [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s, 0.0],
                    ])
                }
                "hue-rotate" => {
                    let a = num(arg).to_radians();
                    let (c, s) = (a.cos(), a.sin());
                    ColorMatrix([
                        [0.213 + c * 0.787 - s * 0.213, 0.715 - c * 0.715 - s * 0.715, 0.072 - c * 0.072 + s * 0.928, 0.0],
                        [0.213 - c * 0.213 + s * 0.143, 0.715 + c * 0.285 + s * 0.140, 0.072 - c * 0.072 - s * 0.283, 0.0],
                        [0.213 - c * 0.213 - s * 0.787, 0.715 - c * 0.715 + s * 0.715, 0.072 + c * 0.928 + s * 0.072, 0.0],
                    ])
                }
                "contrast" => {
                    let k = num(arg);
                    let o = 0.5 - 0.5 * k;
                    ColorMatrix([[k, 0.0, 0.0, o], [0.0, k, 0.0, o], [0.0, 0.0, k, o]])
                }
                "brightness" => {
                    let k = num(arg);
                    ColorMatrix([[k, 0.0, 0.0, 0.0], [0.0, k, 0.0, 0.0], [0.0, 0.0, k, 0.0]])
                }
                _ => continue,
            };
            out = out.then(&m);
        }
        out
    }
}

// ------------------------------------------------------------ timing
/// CSS timing functions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    Linear,
    Ease,
    EaseOut,
    EaseIn,
    EaseInOut,
}
fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    let bx = |t: f64| 3.0 * (1.0 - t) * (1.0 - t) * t * x1 + 3.0 * (1.0 - t) * t * t * x2 + t * t * t;
    let by = |t: f64| 3.0 * (1.0 - t) * (1.0 - t) * t * y1 + 3.0 * (1.0 - t) * t * t * y2 + t * t * t;
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let m = (lo + hi) * 0.5;
        if bx(m) < x {
            lo = m;
        } else {
            hi = m;
        }
    }
    by((lo + hi) * 0.5)
}
impl Ease {
    pub fn at(self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Ease::Linear => x,
            Ease::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, x),
            Ease::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, x),
            Ease::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, x),
            Ease::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, x),
        }
    }
    fn parse(s: &str) -> Option<Ease> {
        Some(match s {
            "linear" => Ease::Linear,
            "ease" => Ease::Ease,
            "ease-out" => Ease::EaseOut,
            "ease-in" => Ease::EaseIn,
            "ease-in-out" => Ease::EaseInOut,
            _ => return None,
        })
    }
}
/// the `opacity` duration/easing of an inline `transition` (e.g. 'opacity 0.08s')
fn inline_transition(el: &Element, prop: &str) -> Option<(f64, Ease)> {
    let t = el.style("transition")?;
    for part in split_top(t, ',') {
        let tk = tokens(&part);
        if tk.first().map(|p| p == prop || p == "all") != Some(true) {
            continue;
        }
        let dur = tk
            .get(1)
            .and_then(|d| {
                if let Some(ms) = d.strip_suffix("ms") {
                    ms.parse::<f64>().ok().map(|v| v / 1000.0)
                } else {
                    d.strip_suffix('s').and_then(|v| v.parse().ok())
                }
            })
            .unwrap_or(0.0);
        let ease = tk.get(2).and_then(|e| Ease::parse(e)).unwrap_or(Ease::Ease);
        return Some((dur, ease));
    }
    None
}

/// A running CSS transition of one numeric property.
#[derive(Clone, Copy, Debug)]
struct Trans {
    from: f64,
    to: f64,
    start: f64,
    dur: f64,
    ease: Ease,
}
impl Trans {
    fn value(&self, now: f64) -> f64 {
        if self.dur <= 0.0 {
            return self.to;
        }
        let x = (now - self.start) / (self.dur * 1000.0);
        if x >= 1.0 {
            self.to
        } else {
            self.from + (self.to - self.from) * self.ease.at(x)
        }
    }
}

// ------------------------------------------------------------ fonts & text
use crate::ui::faces::face_metrics;
pub use crate::ui::faces::{Face, Font};

/// A styled piece of inline text (a text node or a `<span style>`).
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub text: String,
    pub color: Rgba,
    pub shadows: Vec<Shadow>,
    pub opacity: f64,
    pub ls: f64,
}

/// An element's `markup` (the HUD's inline markup: `<span style=...>`, `<br>`,
/// entities) → runs, inheriting `base` for anything a span doesn't set.
pub fn parse_inline_markup(markup: &str, base: &Run) -> Vec<Run> {
    let mut out: Vec<Run> = vec![];
    let mut b0 = base.clone();
    b0.opacity = 1.0;
    let mut stack: Vec<Run> = vec![b0];
    let mut text = String::new();
    let flush = |text: &mut String, out: &mut Vec<Run>, st: &Run| {
        if !text.is_empty() {
            let mut r = st.clone();
            r.text = decode_entities(text);
            out.push(r);
            text.clear();
        }
    };
    let mut i = 0;
    let b = markup.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let end = markup[i..].find('>').map(|e| i + e).unwrap_or(b.len() - 1);
            let tag = &markup[i + 1..end];
            flush(&mut text, &mut out, stack.last().unwrap());
            let tl = tag.trim().to_ascii_lowercase();
            if tl.starts_with("br") {
                let mut r = stack.last().unwrap().clone();
                r.text = "\n".into();
                r.ls = -1e9;
                out.push(r);
            } else if tl.starts_with('/') {
                if stack.len() > 1 {
                    stack.pop();
                }
            } else {
                let mut r = stack.last().unwrap().clone();
                r.opacity = 1.0; // opacity is not inherited: a span's own group
                if let Some(si) = tag.find("style=\"") {
                    let st = &tag[si + 7..];
                    let st = &st[..st.find('"').unwrap_or(st.len())];
                    for decl in st.split(';') {
                        let Some((k, v)) = decl.split_once(':') else { continue };
                        let (k, v) = (k.trim(), v.trim());
                        match k {
                            "color" => {
                                if let Some(c) = css_color(v) {
                                    r.color = c;
                                }
                            }
                            "text-shadow" => r.shadows = parse_shadows(v),
                            "opacity" => r.opacity = v.parse().unwrap_or(1.0),
                            "letter-spacing" => r.ls = px_len(v).unwrap_or(0.0),
                            _ => {}
                        }
                    }
                }
                stack.push(r);
            }
            i = end + 1;
        } else {
            let ch = markup[i..].chars().next().unwrap();
            text.push(ch);
            i += ch.len_utf8();
        }
    }
    flush(&mut text, &mut out, stack.last().unwrap());
    // the parent's opacity multiplies each span group
    for r in out.iter_mut() {
        r.opacity *= base.opacity;
    }
    out
}
fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", "\u{a0}")
        .replace("&middot;", "·")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// CSS `white-space`
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ws {
    Normal,
    NoWrap,
    Pre,
    PreLine,
}

/// one laid-out glyph (x relative to the line start)
#[derive(Clone, Copy, Debug)]
struct Glyph {
    ch: char,
    x: f64,
    run: usize,
}
#[derive(Clone, Debug, Default)]
struct Line {
    glyphs: Vec<Glyph>,
    width: f64,
}

/// Lay out runs into lines: collapse white space as `ws` says, break greedily
/// at spaces (and after '-' / around '—') when wider than `max_w`.
fn layout_runs(runs: &[Run], font: Font, ws: Ws, max_w: Option<f64>) -> Vec<Line> {
    // 1. the character stream after white-space processing ('\n' = forced break)
    let mut chars: Vec<(char, usize)> = vec![];
    let collapse = ws != Ws::Pre;
    let keep_nl = ws == Ws::Pre || ws == Ws::PreLine;
    for (ri, r) in runs.iter().enumerate() {
        if r.ls == -1e9 {
            while chars.last().map(|c| c.0 == ' ').unwrap_or(false) {
                chars.pop();
            }
            chars.push(('\n', ri));
            continue;
        }
        for ch in r.text.chars() {
            let ch = if ch == '\t' { ' ' } else { ch };
            if ch == '\n' || ch == '\r' {
                if keep_nl {
                    while collapse && chars.last().map(|c| c.0 == ' ').unwrap_or(false) {
                        chars.pop();
                    }
                    chars.push(('\n', ri));
                    continue;
                }
                if collapse {
                    if chars.last().map(|c| c.0 == ' ' || c.0 == '\n').unwrap_or(true) {
                        continue;
                    }
                    chars.push((' ', ri));
                    continue;
                }
            }
            if ch == ' ' && collapse && chars.last().map(|c| c.0 == ' ' || c.0 == '\n').unwrap_or(true) {
                continue;
            }
            chars.push((ch, ri));
        }
    }
    if collapse {
        while chars.last().map(|c| c.0 == ' ').unwrap_or(false) {
            chars.pop();
        }
    }
    let wrap = (ws == Ws::Normal || ws == Ws::PreLine) && max_w.is_some();
    let adv = |c: char, ri: usize| font.advance(if c == '\u{a0}' { ' ' } else { c }) + runs[ri].ls.max(0.0);
    // 2. pieces ending at break opportunities
    let mut lines: Vec<Line> = vec![];
    let mut line = Line::default();
    let mut x = 0.0;
    let mut i = 0;
    while i < chars.len() {
        if chars[i].0 == '\n' {
            line.width = trimmed_width(&line, &chars, &adv);
            lines.push(std::mem::take(&mut line));
            x = 0.0;
            i += 1;
            continue;
        }
        // a piece: up to and including the next break opportunity
        let mut j = i;
        let mut w_vis = 0.0; // width without a trailing space
        let mut w = 0.0;
        while j < chars.len() && chars[j].0 != '\n' {
            let (c, ri) = chars[j];
            let a = adv(c, ri);
            w += a;
            if c != ' ' {
                w_vis = w;
            }
            j += 1;
            if c == ' ' {
                break;
            }
            if c == '—' {
                break;
            }
            if c == '-' && j >= 2 && chars[j - 2].0.is_alphanumeric() && chars.get(j).map(|n| n.0.is_alphabetic()).unwrap_or(false) {
                break;
            }
            if chars.get(j).map(|n| n.0 == '—').unwrap_or(false) {
                break;
            }
        }
        if wrap && !line.glyphs.is_empty() && x + w_vis > max_w.unwrap() + 0.01 {
            line.width = trimmed_width(&line, &chars, &adv);
            lines.push(std::mem::take(&mut line));
            x = 0.0;
        }
        for k in i..j {
            let (c, ri) = chars[k];
            if line.glyphs.is_empty() && c == ' ' && collapse {
                continue;
            }
            line.glyphs.push(Glyph { ch: c, x, run: ri });
            x += adv(c, ri);
        }
        i = j;
    }
    line.width = trimmed_width(&line, &chars, &adv);
    // a forced break at the very end opens no new line box
    if !line.glyphs.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}
fn trimmed_width(l: &Line, _c: &[(char, usize)], adv: &dyn Fn(char, usize) -> f64) -> f64 {
    let mut n = l.glyphs.len();
    while n > 0 && l.glyphs[n - 1].ch == ' ' {
        n -= 1;
    }
    if n == 0 {
        0.0
    } else {
        let g = l.glyphs[n - 1];
        g.x + adv(g.ch, g.run)
    }
}

/// A laid-out block of text ready to paint: lines with their baselines.
#[derive(Clone, Debug)]
pub struct TextBlock {
    runs: Vec<Run>,
    font: Font,
    lines: Vec<Line>,
    bases: Vec<f64>,
    total_h: f64,
}
impl TextBlock {
    pub fn new(runs: Vec<Run>, font: Font, ws: Ws, max_w: Option<f64>, line_h: Option<f64>) -> TextBlock {
        let lines = layout_runs(&runs, font, ws, max_w);
        // each line box is the union of its faces' inline boxes (line-height
        // normal: ascent/descent; explicit: centred in the line height)
        let (mut bases, mut y) = (vec![], 0.0);
        for l in &lines {
            let mut faces = vec![(font.ascent(), font.descent())];
            // (fallback faces only count with `line-height: normal`, and
            // only Courier New lines use them)
            if line_h.is_none() && font.face == Face::Typed {
                for g in &l.glyphs {
                    if let Some(m) = face_metrics(g.ch, font.size) {
                        if !faces.contains(&m) {
                            faces.push(m);
                        }
                    }
                }
            }
            let (mut above, mut below) = (0.0f64, 0.0f64);
            for (a, d) in faces {
                // the half-leading above the text is floored
                let (up, down) = match line_h {
                    Some(lh) => {
                        let hl = ((lh - (a + d)) / 2.0).floor();
                        (a + hl, lh - (a + hl))
                    }
                    None => (a, d),
                };
                above = above.max(up);
                below = below.max(down);
            }
            bases.push(y + above);
            y += above + below;
        }
        TextBlock { runs, font, lines, bases, total_h: y }
    }
    pub fn plain(
        text: &str,
        color: Rgba,
        shadows: &[Shadow],
        ls: f64,
        font: Font,
        ws: Ws,
        max_w: Option<f64>,
        line_h: Option<f64>,
    ) -> TextBlock {
        TextBlock::new(vec![Run { text: text.to_string(), color, shadows: shadows.to_vec(), opacity: 1.0, ls }], font, ws, max_w, line_h)
    }
    /// widest line (the block's max-content width when not wrapped)
    pub fn width(&self) -> f64 {
        self.lines.iter().map(|l| l.width).fold(0.0, f64::max)
    }
    pub fn height(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self.total_h
        }
    }
    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.glyphs.is_empty()) && self.lines.len() <= 1
    }
}

// ------------------------------------------------------------ painting
/// σ of a CSS blur radius (CSS: σ = blur / 2)
const BLUR_SIGMA: f64 = 0.5;

/// A device-resolution canvas for one HUD box, drawn in CSS px.
pub struct Painter {
    pub c: Canvas,
    pub ox: f64,
    pub oy: f64,
    pub dpr: f64,
}
impl Painter {
    /// a canvas covering the CSS-px rect (x, y, w, h), snapped out to device pixels
    pub fn new(x: f64, y: f64, w: f64, h: f64, dpr: f64) -> Painter {
        let (x0, y0) = ((x * dpr).floor(), (y * dpr).floor());
        let (x1, y1) = (((x + w) * dpr).ceil(), ((y + h) * dpr).ceil());
        let c = Canvas::new((x1 - x0).max(1.0) as u32, (y1 - y0).max(1.0) as u32);
        Painter { c, ox: x0 / dpr, oy: y0 / dpr, dpr }
    }
    pub(crate) fn tr(&mut self) {
        let d = self.dpr;
        self.c.set_transform(d, 0.0, 0.0, d, -self.ox * d, -self.oy * d);
    }
    /// a filled rect (CSS px)
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: Rgba) {
        if w <= 0.0 || h <= 0.0 || c[3] <= 0.0 {
            return;
        }
        // box edges snap to device pixels
        let d = self.dpr;
        let (x0, y0, x1, y1) = ((x * d).round() / d, (y * d).round() / d, ((x + w) * d).round() / d, ((y + h) * d).round() / d);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        self.tr();
        self.c.set_global_alpha(1.0);
        self.c.set_fill_style(css_rgba(c).as_str());
        self.c.fill_rect(x0, y0, x1 - x0, y1 - y0);
    }
    /// a CSS border of width `bw` inside the border box (x, y, w, h)
    pub fn border(&mut self, x: f64, y: f64, w: f64, h: f64, bw: f64, c: Rgba) {
        self.rect(x, y, w, bw, c);
        self.rect(x, y + h - bw, w, bw, c);
        self.rect(x, y + bw, bw, h - 2.0 * bw, c);
        self.rect(x + w - bw, y + bw, bw, h - 2.0 * bw, c);
    }
    /// paint a laid-out text block with its line boxes starting at (x, y), each
    /// line aligned in a box of width `box_w` ("left" | "center" | "right").
    pub fn text(&mut self, tb: &TextBlock, x: f64, y: f64, box_w: f64, align: &str, alpha: f64) {
        if tb.is_empty() {
            return;
        }
        // group glyphs by run style (shadows + opacity), in order
        let mut groups: Vec<(Vec<Shadow>, f64, Vec<(char, f64, f64, usize)>)> = vec![];
        for (li, l) in tb.lines.iter().enumerate() {
            let lx = match align {
                "center" => x + (box_w - l.width) / 2.0,
                "right" => x + box_w - l.width,
                _ => x,
            };
            let ly = (y + tb.bases[li]).round(); // baselines sit on whole CSS px
            for g in &l.glyphs {
                if g.ch == ' ' || g.ch == '\u{a0}' {
                    continue;
                }
                let r = &tb.runs[g.run];
                let key = (r.shadows.clone(), r.opacity);
                match groups.last_mut() {
                    Some(gr) if gr.0 == key.0 && gr.1 == key.1 => gr.2.push((g.ch, lx + g.x, ly, g.run)),
                    _ => groups.push((key.0, key.1, vec![(g.ch, lx + g.x, ly, g.run)])),
                }
            }
        }
        // the part of the view this canvas shows (glyphs outside are skipped)
        let (vx0, vy0) = (self.ox, self.oy);
        let (vx1, vy1) = (self.ox + self.c.width() / self.dpr, self.oy + self.c.height() / self.dpr);
        let d = self.dpr;
        for (shadows, opacity, glyphs) in groups {
            let fsz = tb.font.size;
            let m = shadows.iter().map(|s| s.blur * 1.6 + s.ox.abs().max(s.oy.abs())).fold(0.0, f64::max) + 1.0;
            let glyphs: Vec<(char, f64, f64, usize)> = glyphs
                .into_iter()
                .filter(|&(ch, gx, gy, _)| {
                    gx + tb.font.advance(ch) + fsz * 0.2 + m >= vx0
                        && gx - fsz * 0.2 - m <= vx1
                        && gy + fsz * 0.45 + m >= vy0
                        && gy - fsz * 1.1 - m <= vy1
                })
                .collect();
            if glyphs.is_empty() {
                continue;
            }
            // bounds of the group (+ shadow margin), clipped to this canvas
            let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for &(ch, gx, gy, _) in &glyphs {
                x0 = x0.min(gx - fsz * 0.2);
                x1 = x1.max(gx + tb.font.advance(ch) + fsz * 0.2);
                y0 = y0.min(gy - fsz * 1.1);
                y1 = y1.max(gy + fsz * 0.45);
            }
            let (bx0, by0) = ((x0 - m).max(vx0 - m), (y0 - m).max(vy0 - m));
            let (bx1, by1) = ((x1 + m).min(vx1 + m), (y1 + m).min(vy1 + m));
            let mut out = Painter::new(bx0, by0, bx1 - bx0, by1 - by0, d);
            let (gw, gh) = (out.c.width() as usize, out.c.height() as usize);
            // glyph segments: runs of plain Courier New glyphs on one line share a CTLine
            let mut segs: Vec<(String, f64, f64, Rgba, f64)> = vec![];
            let mut prev: Option<(f64, f64, usize, f64)> = None; // (next x, y, run, kern)
            for &(ch, gx, gy, ri) in &glyphs {
                let ls = tb.runs[ri].ls.max(0.0);
                let plain = tb.font.draws_runs(ch, ls);
                let (dx, dy) = ((gx - out.ox) * d, (gy - out.oy) * d);
                let cont = match (prev, segs.last()) {
                    (Some((nx, py, pr, _)), Some(_)) => plain && pr == ri && (py - gy).abs() < 1e-6 && (nx - gx).abs() < 1e-6,
                    _ => false,
                };
                if cont {
                    segs.last_mut().unwrap().0.push(ch);
                } else {
                    segs.push((ch.to_string(), dx, dy, tb.runs[ri].color, ls * d));
                }
                prev = if plain { Some((gx + tb.font.advance(ch) + ls, gy, ri, ls)) } else { None };
            }
            let glyph_px = ct_glyphs(gw, gh, &segs, tb.font, d);
            // shadows (the last listed at the bottom), then the glyphs, premultiplied
            let n = gw * gh;
            let mut acc = vec![[0f32; 4]; n];
            if !shadows.is_empty() {
                let mask: Vec<f32> = glyph_px.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
                for sh in shadows.iter().rev() {
                    if sh.color[3] <= 0.0 {
                        continue;
                    }
                    let blurred = blur_mask(&mask, gw, gh, sh.blur * BLUR_SIGMA * d);
                    let (ox, oy) = ((sh.ox * d).round() as i64, (sh.oy * d).round() as i64);
                    let c = [sh.color[0] as f32, sh.color[1] as f32, sh.color[2] as f32, sh.color[3] as f32];
                    let put = |px: &mut [f32; 4], a: f32| {
                        if a <= 0.0 {
                            return;
                        }
                        let k = 1.0 - a;
                        px[0] = c[0] * a + px[0] * k;
                        px[1] = c[1] * a + px[1] * k;
                        px[2] = c[2] * a + px[2] * k;
                        px[3] = a + px[3] * k;
                    };
                    if ox == 0 && oy == 0 {
                        for (px, b) in acc.iter_mut().zip(blurred.iter()) {
                            put(px, b * c[3]);
                        }
                    } else {
                        for yy in 0..gh as i64 {
                            for xx in 0..gw as i64 {
                                let (sx, sy) = (xx - ox, yy - oy);
                                if sx < 0 || sy < 0 || sx >= gw as i64 || sy >= gh as i64 {
                                    continue;
                                }
                                put(&mut acc[(yy as usize) * gw + xx as usize], blurred[(sy as usize) * gw + sx as usize] * c[3]);
                            }
                        }
                    }
                }
            }
            let mut img = out.c.create_image_data(gw as f64, gh as f64);
            for i in 0..n {
                let g = &glyph_px[i * 4..i * 4 + 4];
                let ga = g[3] as f32 / 255.0;
                let px = &mut acc[i];
                let k = 1.0 - ga;
                let (r, gg, b, a) =
                    (g[0] as f32 / 255.0 + px[0] * k, g[1] as f32 / 255.0 + px[1] * k, g[2] as f32 / 255.0 + px[2] * k, ga + px[3] * k);
                if a > 0.0 {
                    img.data[i * 4] = (r / a * 255.0).round().clamp(0.0, 255.0) as u8;
                    img.data[i * 4 + 1] = (gg / a * 255.0).round().clamp(0.0, 255.0) as u8;
                    img.data[i * 4 + 2] = (b / a * 255.0).round().clamp(0.0, 255.0) as u8;
                    img.data[i * 4 + 3] = (a * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
            out.c.put_image_data(&img, 0, 0);
            self.blit(&out, alpha * opacity);
        }
    }
    /// composite another painter's canvas (same dpr) at its view position
    pub fn blit(&mut self, p: &Painter, alpha: f64) {
        self.c.reset_transform();
        self.c.set_global_alpha(alpha.clamp(0.0, 1.0));
        self.c.draw_image(&p.c, ((p.ox - self.ox) * self.dpr).round(), ((p.oy - self.oy) * self.dpr).round());
        self.c.set_global_alpha(1.0);
    }
    /// the device-px rect of this canvas in 0..1 screen units
    fn screen_rect(&self, w: f64, h: f64) -> [f32; 4] {
        [(self.ox / w) as f32, (self.oy / h) as f32, (self.c.width() / self.dpr / w) as f32, (self.c.height() / self.dpr / h) as f32]
    }
}

// ------------------------------------------------------------ glyphs
// The engine canvas maps "Courier New" to macOS Courier (a heavier face); the
// HUD wants the real faces, so it rasterises its glyphs itself with
// CoreText (engine::cg; the faces are `ui::faces`).
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextSetShouldSmoothFonts(c: engine::cg::CGContextRef, b: bool);
}
/// draw text segments (device px baselines; `kern` = letter-spacing in device
/// px after every glyph) into a w×h image: premultiplied RGBA
fn ct_glyphs(w: usize, h: usize, segs: &[(String, f64, f64, Rgba, f64)], font: Font, dpr: f64) -> Vec<u8> {
    use engine::cg::*;
    let mut buf = vec![0u8; w * h * 4];
    unsafe {
        let cs = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        let ctx = CGBitmapContextCreate(
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            w,
            h,
            8,
            w * 4,
            cs,
            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
        );
        CGContextTranslateCTM(ctx, 0.0, h as f64);
        CGContextScaleCTM(ctx, 1.0, -1.0);
        CGContextSetShouldSmoothFonts(ctx, false);
        for (text, x, y, c, kern) in segs {
            let s = cfstr(text);
            let dict = crate::ui::faces::text_attrs(font, font.size * dpr, *kern);
            let astr = CFAttributedStringCreate(std::ptr::null(), s, dict);
            let line = CTLineCreateWithAttributedString(astr);
            CGContextSaveGState(ctx);
            CGContextTranslateCTM(ctx, *x, *y);
            CGContextScaleCTM(ctx, 1.0, -1.0);
            CGContextSetTextMatrix(ctx, CGAffineTransform::IDENTITY);
            CGContextSetTextPosition(ctx, 0.0, 0.0);
            CGContextSetRGBFillColor(ctx, c[0], c[1], c[2], c[3]);
            CGContextSetTextDrawingMode(ctx, kCGTextFill);
            CTLineDraw(line, ctx);
            CGContextRestoreGState(ctx);
            CFRelease(line as CFTypeRef);
            CFRelease(astr);
            CFRelease(dict);
            CFRelease(s);
        }
        CGContextFlush(ctx);
        CGContextRelease(ctx);
        CGColorSpaceRelease(cs);
    }
    buf
}

/// a Gaussian blur of a mask; wide blurs run at half resolution (they are smooth)
fn blur_mask(src: &[f32], w: usize, h: usize, sigma: f64) -> Vec<f32> {
    if sigma < 3.0 || w < 4 || h < 4 {
        return gaussian_blur(src, w, h, sigma);
    }
    let (hw, hh) = ((w + 1) / 2, (h + 1) / 2);
    let mut small = vec![0f32; hw * hh];
    for y in 0..hh {
        for x in 0..hw {
            let mut s = 0.0;
            let mut n = 0.0;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (sx, sy) = (x * 2 + dx, y * 2 + dy);
                if sx < w && sy < h {
                    s += src[sy * w + sx];
                    n += 1.0;
                }
            }
            small[y * hw + x] = s / n;
        }
    }
    let b = gaussian_blur(&small, hw, hh, sigma / 2.0);
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = ((x as f32 + 0.5) / 2.0 - 0.5, (y as f32 + 0.5) / 2.0 - 0.5);
            let (x0, y0) = (fx.floor().max(0.0) as usize, fy.floor().max(0.0) as usize);
            let (x1, y1) = ((x0 + 1).min(hw - 1), (y0 + 1).min(hh - 1));
            let (tx, ty) = ((fx - x0 as f32).clamp(0.0, 1.0), (fy - y0 as f32).clamp(0.0, 1.0));
            let a = b[y0 * hw + x0] * (1.0 - tx) + b[y0 * hw + x1] * tx;
            let c = b[y1 * hw + x0] * (1.0 - tx) + b[y1 * hw + x1] * tx;
            out[y * w + x] = a * (1.0 - ty) + c * ty;
        }
    }
    out
}

/// separable Gaussian (three box blurs) of an alpha mask
fn gaussian_blur(src: &[f32], w: usize, h: usize, sigma: f64) -> Vec<f32> {
    if sigma < 0.3 {
        return src.to_vec();
    }
    // box sizes for 3 passes (Kovesi)
    let n = 3.0;
    let wi = ((12.0 * sigma * sigma / n) + 1.0).sqrt();
    let mut wl = wi.floor() as i64;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;
    let mi = ((12.0 * sigma * sigma - n * (wl * wl) as f64 - 4.0 * n * wl as f64 - 3.0 * n) / (-4.0 * wl as f64 - 4.0)).round() as i64;
    let sizes: Vec<i64> = (0..3).map(|i| if i < mi { wl } else { wu }).collect();
    let mut a = src.to_vec();
    let mut b = vec![0f32; src.len()];
    for &s in &sizes {
        let r = ((s - 1) / 2) as usize;
        box_h(&a, &mut b, w, h, r);
        box_v(&b, &mut a, w, h, r);
    }
    a
}
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let inv = 1.0 / (2 * r + 1) as f32;
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let mut acc = 0f32;
        for x in 0..=r.min(w.saturating_sub(1)) {
            acc += row[x];
        }
        for x in 0..w {
            dst[y * w + x] = acc * inv;
            let add = x + r + 1;
            if add < w {
                acc += row[add];
            }
            if x >= r {
                acc -= row[x - r];
            }
        }
    }
}
fn box_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    // row-wise running sums (cache friendly)
    let inv = 1.0 / (2 * r + 1) as f32;
    let mut acc = vec![0f32; w];
    for y in 0..=r.min(h.saturating_sub(1)) {
        let row = &src[y * w..(y + 1) * w];
        for x in 0..w {
            acc[x] += row[x];
        }
    }
    for y in 0..h {
        let out = &mut dst[y * w..(y + 1) * w];
        for x in 0..w {
            out[x] = acc[x] * inv;
        }
        let add = y + r + 1;
        if add < h {
            let row = &src[add * w..(add + 1) * w];
            for x in 0..w {
                acc[x] += row[x];
            }
        }
        if y >= r {
            let row = &src[(y - r) * w..(y - r + 1) * w];
            for x in 0..w {
                acc[x] -= row[x];
            }
        }
    }
}
/// Φ — the normal CDF (Abramowitz–Stegun erf)
fn phi(x: f64) -> f64 {
    let z = x / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.3275911 * z.abs());
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-z * z).exp();
    0.5 * (1.0 + if z >= 0.0 { y } else { -y })
}
/// coverage of the Gaussian-blurred interval [a, b] at x
fn blurred_span(x: f64, a: f64, b: f64, sigma: f64) -> f64 {
    if sigma <= 0.0 {
        return if x >= a && x < b { 1.0 } else { 0.0 };
    }
    phi((x - a) / sigma) - phi((x - b) / sigma)
}

// ------------------------------------------------------------ state
/// `ver` counts repaints: a repaint makes a new Canvas under the same id, whose own
/// version could equal the old one's, so the renderer's texture cache keys on this
struct BoxCanvas {
    canvas: Rc<RefCell<Canvas>>,
    id: u64,
    key: u64,
    rect: [f32; 4],
    ver: u64,
}

/// The renderer's own state, kept on `Game::hud_ui`.
#[derive(Default)]
pub struct HudUi {
    boxes: HashMap<&'static str, BoxCanvas>,
    trans: HashMap<String, Trans>,
    next_id: u64,
    grain_id: u64,
    /// click targets of the last frame, topmost first: (id, rect css px, enabled)
    pub(crate) hits: Vec<(&'static str, [f64; 4], bool)>,
    /// the frame's layers in paint order, by box name (tests dump them)
    pub painted: Vec<&'static str>,
}
fn hash_of(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// The frame being built.
pub(crate) struct Frame<'a> {
    pub(crate) ui: &'a mut HudUi,
    out: Vec<HudLayer>,
    pub(crate) w: f64,
    pub(crate) h: f64,
    pub(crate) dpr: f64,
    pub(crate) now: f64,
}
impl Frame<'_> {
    /// a CSS transition of `key` toward `target` (starts when the target changes)
    pub(crate) fn tr(&mut self, key: &str, target: f64, dur: f64, ease: Ease) -> f64 {
        let now = self.now;
        match self.ui.trans.get_mut(key) {
            None => {
                self.ui.trans.insert(key.to_string(), Trans { from: target, to: target, start: now, dur: 0.0, ease });
                target
            }
            Some(t) => {
                if t.to != target {
                    let cur = t.value(now);
                    *t = Trans { from: cur, to: target, start: now, dur, ease };
                }
                t.value(now)
            }
        }
    }
    /// the cached canvas `name`, repainted by `paint` when `key` changes
    pub(crate) fn boxed(
        &mut self,
        name: &'static str,
        key: &str,
        paint: impl FnOnce(f64) -> Painter,
    ) -> (Rc<RefCell<Canvas>>, u64, u64, [f32; 4]) {
        let k = hash_of(&format!("{}|{}|{}|{}", key, self.w, self.h, self.dpr));
        let (w, h, dpr) = (self.w, self.h, self.dpr);
        if self.ui.boxes.get(name).map(|b| b.key != k).unwrap_or(true) {
            let p = paint(dpr);
            let rect = p.screen_rect(w, h);
            let (id, ver) = match self.ui.boxes.get(name) {
                Some(b) => (b.id, b.ver + 1),
                None => {
                    self.ui.next_id += 1;
                    (0x4855_4400 + self.ui.next_id, 1)
                }
            };
            self.ui.boxes.insert(name, BoxCanvas { canvas: Rc::new(RefCell::new(p.c)), id, key: k, rect, ver });
        }
        let b = &self.ui.boxes[name];
        // repaint count, plus in-place redraws of the same canvas
        let v = (b.ver << 32) + b.canvas.borrow().version;
        (b.canvas.clone(), b.id, v, b.rect)
    }
    /// paint (if needed) and composite box `name` at `opacity`
    pub(crate) fn layer(&mut self, name: &'static str, key: &str, opacity: f64, paint: impl FnOnce(f64) -> Painter) {
        if opacity <= 0.0005 {
            return;
        }
        let (canvas, id, version, rect) = self.boxed(name, key, paint);
        self.push_canvas(name, canvas, id, version, rect, opacity, LayerBlend::Normal, false);
    }
    pub(crate) fn push_canvas(
        &mut self,
        name: &'static str,
        canvas: Rc<RefCell<Canvas>>,
        id: u64,
        version: u64,
        rect: [f32; 4],
        opacity: f64,
        blend: LayerBlend,
        smooth: bool,
    ) {
        self.ui.painted.push(name);
        self.out.push(HudLayer::Canvas {
            canvas,
            id,
            version,
            rect,
            opacity: opacity.clamp(0.0, 1.0) as f32,
            blend,
            tile: [1.0, 1.0],
            offset: [0.0, 0.0],
            smooth,
        });
    }
    fn radial(&mut self, r: &Radial, blend: LayerBlend, opacity: f64, cm: Option<ColorMatrix>) {
        if opacity <= 0.0005 || r.stops.iter().all(|s| s.1[3] <= 0.0) {
            return;
        }
        let stops = r.stops.iter().take(4).map(|(t, c)| (*t as f32, f32x4(cm.map(|m| m.apply(*c)).unwrap_or(*c)))).collect();
        self.ui.painted.push("radial");
        self.out.push(HudLayer::Radial {
            center: [r.center[0] as f32, r.center[1] as f32],
            radius: [r.radius[0] as f32, r.radius[1] as f32],
            stops,
            blend,
            opacity: opacity.min(1.0) as f32,
        });
    }
    pub(crate) fn fill(&mut self, c: Rgba, blend: LayerBlend) {
        if c[3] <= 0.0005 {
            return;
        }
        self.ui.painted.push("fill");
        self.out.push(HudLayer::Fill { color: f32x4(c), blend });
    }
    /// an element's `background`: a colour fill or a radial gradient
    pub(crate) fn background(&mut self, bg: &str, opacity: f64, blend: LayerBlend) {
        if bg.trim_start().starts_with("radial-gradient") {
            if let Some(r) = parse_radial(bg, self.w, self.h) {
                self.radial(&r, blend, opacity, None);
            }
        } else if let Some(mut c) = css_color(bg) {
            c[3] *= opacity.min(1.0);
            self.fill(c, blend);
        }
    }
}

/// `el.style.opacity` if set, else the stylesheet's value
fn opacity_of(el: &Element, css: f64) -> f64 {
    el.opacity().unwrap_or(css)
}
/// a percentage width (`'57.3%'`) as 0..1
fn pct_of(el: &Element) -> f64 {
    el.style("width").and_then(|w| w.strip_suffix('%')).and_then(|v| v.trim().parse::<f64>().ok()).map(|v| v / 100.0).unwrap_or(1.0)
}
fn lerp_c(a: Rgba, b: Rgba, t: f64) -> Rgba {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

fn hud_shadows() -> Vec<Shadow> {
    parse_shadows("0 0 4px #000, 0 0 2px #000")
}
/// the text of an element (its `markup` wins when set)
pub(crate) fn runs_of(el: &Element, base: &Run) -> Vec<Run> {
    if !el.markup.is_empty() {
        parse_inline_markup(&el.markup, base)
    } else {
        let mut r = base.clone();
        r.text = el.text.clone();
        vec![r]
    }
}
pub(crate) fn base_run(color: Rgba, shadows: Vec<Shadow>, ls: f64) -> Run {
    Run { text: String::new(), color, shadows, opacity: 1.0, ls }
}

/// the text margin a box needs for its shadows
fn shadow_margin(sh: &[Shadow]) -> f64 {
    sh.iter().map(|s| s.blur * 1.6 + s.ox.abs().max(s.oy.abs()) + s.spread.abs()).fold(0.0, f64::max) + 2.0
}

/// a one-element text box: block `tb` with its line boxes at (x, y), `box_w` wide
pub(crate) fn text_layer(f: &mut Frame, name: &'static str, tb: TextBlock, x: f64, y: f64, box_w: f64, align: &'static str, opacity: f64) {
    if tb.is_empty() {
        return;
    }
    let m = tb.runs.iter().map(|r| shadow_margin(&r.shadows)).fold(2.0, f64::max);
    let key = format!("{:?}|{}|{}|{}|{}|{:?}", tb.runs, x, y, box_w, align, tb.font);
    let (bw, bh) = (box_w.max(tb.width()), tb.height());
    f.layer(name, &key, opacity, move |dpr| {
        let mut p = Painter::new(x - m, y - m, bw + 2.0 * m, bh + 2.0 * m, dpr);
        p.text(&tb, x, y, box_w, align, 1.0);
        p
    });
}

// ------------------------------------------------------------ HUD layers
/// The HUD's layers over the 3D view, in CSS paint order.
pub fn hud_layers(g: &mut Game, w: f64, h: f64, dpr: f64) -> Vec<HudLayer> {
    let mut ui = std::mem::take(&mut g.hud_ui);
    ui.painted.clear();
    ui.hits.clear();
    let (w, h) = (w.max(1.0), h.max(1.0));
    let mut f = Frame { ui: &mut ui, out: vec![], w, h, dpr: dpr.max(0.5), now: g.now_ms };
    let d = &g.ui;
    // (the `#gl` canvas filter of body.low-sanity / .crit-sanity belongs to
    // the scene compositor, `crate::visuals::scene_fx` — not a HUD layer)

    // ---- z 3: #vignette, #weatherTint, #safeRoomGlow
    {
        let el = &d.vignette;
        let op = f.tr("vignette.opacity", opacity_of(el, 0.0), 0.25, Ease::Ease);
        let grad = parse_radial("radial-gradient(ellipse at center, transparent 42%, rgba(0,0,0,0.95) 100%)", w, h).unwrap();
        let cm = el.style("filter").map(ColorMatrix::parse);
        let inset = el.style("box-shadow").map(parse_shadows).unwrap_or_default().into_iter().find(|s| s.inset);
        if op > 0.0005 {
            match inset {
                Some(sh) => {
                    let key = format!("{:?}{:?}", sh, cm);
                    let (canvas, id, ver, rect) =
                        f.boxed("vignette", &key, |_| paint_vignette(&grad, &sh, cm.unwrap_or(ColorMatrix::ID), w, h));
                    f.push_canvas("vignette", canvas, id, ver, rect, op, LayerBlend::Normal, true);
                }
                None => f.radial(&grad, LayerBlend::Normal, op, cm),
            }
        }
    }
    {
        let el = &d.weather_tint;
        let tint = ["wet", "electric", "haunted", "derelict", "still"].iter().find(|k| el.has_class(k)).copied();
        let bg = match tint {
            Some("wet") => "radial-gradient(ellipse at center, transparent 30%, rgba(30,60,90,0.15) 70%, rgba(20,40,70,0.45) 100%)",
            Some("electric") => "radial-gradient(ellipse at center, transparent 30%, rgba(70,60,120,0.10) 70%, rgba(30,25,60,0.35) 100%)",
            Some("haunted") => "radial-gradient(ellipse at center, transparent 30%, rgba(80,10,20,0.15) 70%, rgba(50,5,10,0.55) 100%)",
            Some("derelict") => "radial-gradient(ellipse at center, transparent 30%, rgba(90,60,20,0.14) 70%, rgba(60,40,10,0.45) 100%)",
            Some("still") => "radial-gradient(ellipse at center, transparent 30%, rgba(15,15,15,0.10) 70%, rgba(0,0,0,0.35) 100%)",
            _ => "",
        };
        // (gradients do not interpolate: the 1.2 s transition is a swap)
        if !bg.is_empty() {
            f.background(bg, 1.0, LayerBlend::Multiply);
        }
    }
    {
        let el = &d.safe_room_glow;
        let op = f.tr("safeRoomGlow.opacity", opacity_of(el, if el.has_class("on") { 1.0 } else { 0.0 }), 0.8, Ease::EaseOut);
        f.background(
            "radial-gradient(ellipse at center, transparent 25%, rgba(255, 176, 96, 0.10) 65%, rgba(255, 140, 60, 0.28) 100%)",
            op,
            LayerBlend::Screen,
        );
    }
    // ---- z 4: #aberration (three gradients, multiply), then #hud
    {
        let el = &d.aberration;
        let (dur, ease) = inline_transition(el, "opacity").unwrap_or((0.4, Ease::Ease));
        let op = f.tr("aberration.opacity", opacity_of(el, 0.0), dur, ease);
        for bg in [
            "radial-gradient(circle at 68% 58%, rgba(220,0,60,0.10) 0%, transparent 32%)",
            "radial-gradient(circle at 32% 42%, rgba(0,180,220,0.10) 0%, transparent 32%)",
            "radial-gradient(ellipse at center, transparent 40%, rgba(120,0,0,0.55) 90%, rgba(60,0,0,0.85) 100%)",
        ] {
            f.background(bg, op, LayerBlend::Multiply);
        }
    }
    paint_hud(&mut f, g);
    crate::ui::screens::paint_level_card(&mut f, g);
    // ---- z 5: #flash, #grain
    {
        let el = &d.flash;
        let op = f.tr("flash.opacity", opacity_of(el, 0.0), 0.12, Ease::Ease);
        let bg = el
            .style("background")
            .unwrap_or("radial-gradient(ellipse at center, rgba(120,0,0,0.55) 0%, rgba(60,0,0,0.9) 100%)")
            .to_string();
        if op > 0.0005 {
            f.background(&bg, op, LayerBlend::Normal);
        }
    }
    if let Some(gc) = g.grain_canvas.clone() {
        let op = opacity_of(&d.grain, 0.10) * if g.settings.grain { 1.0 } else { 0.0 };
        if f.ui.grain_id == 0 {
            f.ui.next_id += 1;
            f.ui.grain_id = 0x4855_4400 + f.ui.next_id;
        }
        let (id, ver) = (f.ui.grain_id, gc.borrow().version);
        if op > 0.0005 {
            f.push_canvas("grain", gc, id, ver, [0.0, 0.0, 1.0, 1.0], op, LayerBlend::Overlay, false);
        }
    }
    // ---- z 6: #fade
    {
        let el = &d.fade;
        let op = f.tr("fade.opacity", opacity_of(el, 0.0), 0.55, Ease::Ease);
        let bg = el.style("background").unwrap_or("#000").to_string();
        if op > 0.0005 {
            f.background(&bg, op, LayerBlend::Normal);
        }
    }
    // ---- z 10: the menus and cards (`ui/screens.rs`), then Settings' brightness over everything
    crate::ui::screens::paint_menu(&mut f, g);
    crate::ui::screens::paint_brightness(&mut f, g);
    let out = std::mem::take(&mut f.out);
    drop(f);
    g.hud_ui = ui;
    out
}

/// #vignette with an inline inset box-shadow: gradient + shadow, filtered,
/// on a low-resolution canvas (both are smooth).
fn paint_vignette(grad: &Radial, sh: &Shadow, cm: ColorMatrix, w: f64, h: f64) -> Painter {
    let s = (160.0 / w).max(0.125).min(1.0);
    let (cw, chh) = ((w * s).ceil().max(2.0), (h * s).ceil().max(2.0));
    let mut c = Canvas::new(cw as u32, chh as u32);
    let mut img = c.create_image_data(cw, chh);
    let sigma = sh.blur * BLUR_SIGMA;
    for y in 0..chh as usize {
        for x in 0..cw as usize {
            let (px, py) = ((x as f64 + 0.5) / s, (y as f64 + 0.5) / s);
            let (u, v) = (px / w, py / h);
            let t = (((u - grad.center[0]) / grad.radius[0]).powi(2) + ((v - grad.center[1]) / grad.radius[1]).powi(2)).sqrt();
            let gcol = radial_at(grad, t);
            // inset shadow: everything outside the inner rect, blurred
            let inner = blurred_span(px, sh.spread + sh.ox, w - sh.spread + sh.ox, sigma)
                * blurred_span(py, sh.spread + sh.oy, h - sh.spread + sh.oy, sigma);
            let sa = sh.color[3] * (1.0 - inner).clamp(0.0, 1.0);
            // shadow over background
            let a = sa + gcol[3] * (1.0 - sa);
            let mix = |i: usize| if a > 0.0 { (sh.color[i] * sa + gcol[i] * gcol[3] * (1.0 - sa)) / a } else { 0.0 };
            let o = cm.apply([mix(0), mix(1), mix(2), a]);
            let k = (y * cw as usize + x) * 4;
            for i in 0..3 {
                img.data[k + i] = (o[i] * 255.0).round() as u8;
            }
            img.data[k + 3] = (a * 255.0).round() as u8;
        }
    }
    c.put_image_data(&img, 0, 0);
    // a painter whose canvas spans the view (dpr = canvas px per css px)
    Painter { c, ox: 0.0, oy: 0.0, dpr: s }
}

/// #hud and its children (z-index 4 stacking context)
fn paint_hud(f: &mut Frame, g: &Game) {
    // the title is the place alone, and the pause list stands over it alone
    if g.state == "menu" || g.state == "paused" {
        return;
    }
    let d = &g.ui;
    let (w, h) = (f.w, f.h);
    let body = &d.body;
    let special = body.has_class("in-stairwell") || body.has_class("in-loop");
    // The HUD at rest is the place alone (`ui/rest.rs`): each part fades in
    // when it has something to say, and out after; X shows it all
    let r = &g.rest;
    let ink = col(crate::ui::screens::INK);
    let dark = parse_shadows("0 0 4px rgba(0,0,0,0.85), 0 1px 2px rgba(0,0,0,0.6)");
    let vis = |on: bool| if on || r.glance { 1.0 } else { 0.0 };
    let plain = |t: &str, font: Font, c: Rgba| TextBlock::plain(t, c, &dark, 0.0, font, Ws::NoWrap, None, None);
    let x0 = 24.0;
    // her thread (top left, in her hand) and, glancing, the level
    if !special {
        let op = f.tr("rest.thread", vis(r.thread > 0.0), 1.2, Ease::Ease);
        let tb = TextBlock::plain(&d.objective.text, ink, &dark, 0.0, Font::hand(15.0), Ws::Normal, Some((w * 0.5).max(240.0)), None);
        text_layer(f, "objective", tb, x0, 20.0, 0.0, "left", op * 0.8);
    }
    {
        let op = f.tr("rest.level", vis(false), 0.3, Ease::Ease);
        let t = format!("Level {}", g.level_num);
        text_layer(f, "level", plain(&t, Font::sans(12.0), ink), w - x0 - 60.0, 18.0, 60.0, "right", op * 0.6);
    }
    // the pockets (bottom left), and what would help now, with its key
    {
        let op = f.tr("rest.pockets", vis(r.pockets > 0.0), 0.6, Ease::Ease);
        let line = g.pockets_line();
        let tb = plain(&line, Font::sans(13.0), ink);
        let y = h - 26.0 - tb.height();
        text_layer(f, "bottles", tb, x0, y, 0.0, "left", op * 0.75);
        let hint = g.pocket_hint();
        let op = f.tr("rest.hint", if hint.is_some() { 1.0 } else { 0.0 }, 0.4, Ease::Ease);
        if let Some((key, what)) = hint {
            let k = plain(&key, Font::sans(13.0).bold(), ink);
            let t = plain(what, Font::sans(13.0), ink);
            let (kw, th) = (k.width(), t.height());
            let y = h - 26.0 - 22.0 - th;
            let key_s = format!("{key}|{what}|{y}");
            f.layer("batteries", &key_s, op * 0.9, move |dpr| {
                let mut p = Painter::new(x0 - 6.0, y - 6.0, kw + t.width() + 26.0, th + 12.0, dpr);
                p.text(&k, x0, y, kw, "left", 1.0);
                p.text(&t, x0 + kw + 10.0, y, t.width(), "left", 0.85);
                p
            });
        }
    }
    // the meters: thin, unlabelled, only while they say something
    let bar = |f: &mut Frame, name: &'static str, x: f64, y: f64, bw: f64, frac: f64, c: Rgba, op: f64| {
        let key = format!("{x}|{y}|{bw}|{frac:.3}|{c:?}");
        f.layer(name, &key, op, move |dpr| {
            let mut p = Painter::new(x - 1.0, y - 1.0, bw + 2.0, 4.0, dpr);
            p.rect(x, y, bw, 2.0, [0.0, 0.0, 0.0, 0.35]);
            p.rect(x, y, (bw * frac).clamp(0.0, bw), 2.0, c);
            p
        });
    };
    let label = |f: &mut Frame, name: &'static str, t: &str, x: f64, y: f64, op: f64| {
        let tb = TextBlock::plain(t, ink, &dark, 0.0, Font::sans(10.0), Ws::NoWrap, None, None);
        let tw = tb.width();
        text_layer(f, name, tb, x - tw - 10.0, y - 6.0, tw, "left", op * 0.55);
    };
    let (bx, bw) = ((w - 180.0) / 2.0, 180.0);
    {
        let frac = f.tr("staminabar.width", pct_of(&d.staminabar), 0.08, Ease::Linear);
        let low = d.stamina.has_class("low");
        let op = f.tr("rest.stamina", vis(r.stamina > 0.0), if r.stamina > 0.0 { 0.25 } else { 0.9 }, Ease::Ease);
        let c = if low { col("#b8503e") } else { ink };
        bar(f, "stamina", bx, h - 30.0, bw, frac, c, op * 0.8);
        let op_l = f.tr("rest.labels", vis(false), 0.3, Ease::Ease);
        label(f, "staminaLabel", "Stamina", bx, h - 30.0, op_l);
    }
    {
        let frac = f.tr("sanitybar.width", pct_of(&d.sanitybar), 0.2, Ease::Linear);
        let lowk = f.tr("sanitybar.low", if d.sanity.has_class("low") { 1.0 } else { 0.0 }, 0.3, Ease::Ease);
        let op = f.tr("rest.sanity", vis(r.sanity > 0.0), 0.8, Ease::Ease);
        bar(f, "sanity", bx, h - 44.0, bw, frac, lerp_c(col("#a8b8d8"), col("#b53a3a"), lowk), op * 0.7);
        let op_l = f.tr("rest.labels", vis(false), 0.3, Ease::Ease);
        label(f, "sanityLabel", "Sanity", bx, h - 44.0, op_l);
    }
    // the light's cell (bottom right): a battery outline and its charge
    {
        let frac = f.tr("batterybar.width", pct_of(&d.batterybar), 0.15, Ease::Linear);
        let low = d.battery.has_class("low");
        let op = f.tr("rest.battery", vis(r.battery > 0.0), 0.6, Ease::Ease);
        let (cw, ch) = (26.0, 11.0);
        let (cx, cy) = (w - x0 - cw - 3.0, h - 26.0 - ch);
        let fill = if low { col("#cf6c2e") } else { ink };
        let on = g.player.flashlight_on;
        let key = format!("{cx}|{cy}|{frac:.3}|{low}|{on}");
        f.layer("battery", &key, op * 0.8, move |dpr| {
            let mut p = Painter::new(cx - 2.0, cy - 2.0, cw + 8.0, ch + 4.0, dpr);
            p.border(cx, cy, cw, ch, 1.0, rgba(226.0, 218.0, 189.0, 0.8));
            p.rect(cx + cw, cy + 3.0, 2.0, ch - 6.0, rgba(226.0, 218.0, 189.0, 0.8));
            p.rect(cx + 2.0, cy + 2.0, ((cw - 4.0) * frac).max(0.0), ch - 4.0, [fill[0], fill[1], fill[2], if on { 0.95 } else { 0.5 }]);
            p
        });
        let op_l = f.tr("rest.labels", vis(false), 0.3, Ease::Ease);
        let tb = TextBlock::plain("Light", ink, &dark, 0.0, Font::sans(10.0), Ws::NoWrap, None, None);
        let tw = tb.width();
        text_layer(f, "batteryLabel", tb, cx - tw - 8.0, cy, tw, "left", op_l * 0.55);
    }
    // what you face: a ring where the eye rests, and its verb beside it
    {
        let (cx, cy) = (w / 2.0, h / 2.0);
        // (not while reading)
        let target = !d.prompt.text.is_empty() && !g.journal_open;
        let op = f.tr("rest.dot", if target { 1.0 } else { 0.0 }, 0.15, Ease::Ease);
        f.layer("dot", &format!("{cx}|{cy}"), op * 0.8, move |dpr| {
            let mut p = Painter::new(cx - 7.0, cy - 7.0, 14.0, 14.0, dpr);
            p.tr();
            p.c.set_stroke_style("rgba(235,228,205,0.9)");
            p.c.set_line_width(1.2);
            p.c.begin_path();
            p.c.arc(cx, cy, 5.0, 0.0, std::f64::consts::PI * 2.0);
            p.c.stroke();
            p
        });
        if target {
            let key = g
                .settings
                .keys(crate::session::input::Act::Interact)
                .first()
                .map(|k| crate::session::input::key_label(k))
                .unwrap_or_default();
            let (k, words) = crate::ui::rest::prompt_parts(&d.prompt.text, &key);
            let kt = plain(&k, Font::sans(13.0).bold(), ink);
            let tt = plain(&words, Font::sans(14.0), ink);
            let (kw, tw, th) = (kt.width(), tt.width(), tt.height());
            let (px, py) = (cx + 16.0, cy - th / 2.0);
            let key_s = format!("{k}|{words}|{px}|{py}");
            f.layer("prompt", &key_s, op, move |dpr| {
                let mut p = Painter::new(px - 6.0, py - 6.0, kw + tw + 30.0, th + 12.0, dpr);
                p.text(&kt, px, py + 1.0, kw, "left", 0.75);
                p.text(&tt, px + kw + 8.0, py, tw, "left", 0.95);
                p
            });
        }
    }
    // a toast (#msg) and an omen: the building's words, calm, in the middle
    {
        let el = &d.msg;
        let color = el.style("color").and_then(css_color).unwrap_or(ink);
        let sh = el.style("text-shadow").map(parse_shadows).unwrap_or_else(|| dark.clone());
        let op = f.tr("msg.opacity", opacity_of(el, 0.0), 0.4, Ease::Ease);
        let t = crate::ui::rest::calm(&el.text);
        let tb = TextBlock::plain(&t, color, &sh, 0.0, Font::sans(16.0), Ws::NoWrap, None, None);
        let bw = tb.width();
        text_layer(f, "msg", tb, w / 2.0 - bw / 2.0, (h * 0.34).floor(), bw, "center", op);
    }
    {
        let el = &d.omen;
        let op = f.tr("omen.opacity", opacity_of(el, 0.0), 1.8, Ease::Ease);
        let tb = TextBlock::plain(
            &el.text,
            col("#c9bea4"),
            &parse_shadows("0 0 6px rgba(0,0,0,0.9)"),
            0.0,
            Font::sans(13.0).italic(),
            Ws::NoWrap,
            None,
            None,
        );
        let bw = tb.width();
        text_layer(f, "omen", tb, w / 2.0 - bw / 2.0, (h * 0.34).floor() + 30.0, bw, "center", op);
    }
    // the Stairwell's camcorder counter is footage: it keeps the camera's capitals
    if body.has_class("in-stairwell") {
        let tb = TextBlock::new(
            runs_of(&d.stair_hud, &base_run(rgba(235.0, 228.0, 205.0, 0.78), parse_shadows("0 0 6px rgba(0,0,0,0.9)"), 13.0 * 0.12)),
            Font::new(13.0).bold(),
            Ws::Normal,
            None,
            Some(13.0),
        );
        let y = h - 22.0 - tb.height();
        text_layer(f, "stairHud", tb, 22.0, y, 0.0, "left", 1.0);
    }
    // #debug (z 5 inside #hud)
    if d.debug.has_class("on") {
        paint_debug(f, g);
    }
    // the notebook, low in your hands (`ui/paper.rs`)
    if d.journal.has_class("on") {
        crate::ui::paper::paint_notebook(f, g);
    }
}

/// the debug panel's bands (each repainted only when its lines change)
const DEBUG_BANDS: [&str; 24] = [
    "debug-0", "debug-1", "debug-2", "debug-3", "debug-4", "debug-5", "debug-6", "debug-7", "debug-8", "debug-9", "debug-10", "debug-11",
    "debug-12", "debug-13", "debug-14", "debug-15", "debug-16", "debug-17", "debug-18", "debug-19", "debug-20", "debug-21", "debug-22",
    "debug-23",
];
const DEBUG_BAND_LINES: usize = 6;

/// #debug: `font: 11px/1.5`, `white-space: pre`, padding 10/14, min-width 240.
/// The text changes every frame (timers, fps), so it is painted in bands of
/// lines; a band is repainted only when a line that reaches it changed.
fn paint_debug(f: &mut Frame, g: &Game) {
    let el = &g.ui.debug;
    let tb =
        Rc::new(TextBlock::new(runs_of(el, &base_run(col("#e2e2c8"), hud_shadows(), 0.5)), Font::new(11.0), Ws::Pre, None, Some(16.5)));
    let (cw, ch) = (tb.width().max(240.0), tb.height());
    // top: 100px, plus the default `pre { margin: 1em 0 }`
    let (x, y, bw, bh) = (20.0, 100.0 + 11.0, cw + 30.0, ch + 22.0);
    f.ui.hits.push(("debug", [x, y, bw, bh], false));
    let (vw, vh) = (f.w, f.h);
    let (cx0, cx1) = (x, (x + bw).min(vw));
    let (cy0, cy1) = (y, (y + bh).min(vh));
    if cx1 <= cx0 || cy1 <= cy0 {
        return;
    }
    f.layer("debug", &format!("{bw}|{bh}"), 1.0, move |dpr| {
        let mut p = Painter::new(cx0, cy0, cx1 - cx0, cy1 - cy0, dpr);
        p.rect(x, y, bw, bh, [0.0, 0.0, 0.0, 0.65]);
        p.border(x, y, bw, bh, 1.0, rgba(216.0, 207.0, 159.0, 0.35));
        p
    });
    // the text, in bands of whole device-pixel rows
    let (tx, ty) = (x + 15.0, y + 11.0);
    let band_h = 16.5 * DEBUG_BAND_LINES as f64;
    let reach = shadow_margin(&hud_shadows()) + 16.5;
    let line_text: Vec<String> = tb.lines.iter().map(|l| l.glyphs.iter().map(|g| g.ch).collect()).collect();
    for (bi, name) in DEBUG_BANDS.iter().enumerate() {
        let b0 = ty + bi as f64 * band_h;
        if b0 >= cy1 {
            break;
        }
        let b1 = (b0 + band_h).min(cy1);
        let (b0d, b1d) = ((b0 * f.dpr).floor() / f.dpr, (b1 * f.dpr).floor() / f.dpr);
        let (b0c, b1c) = (if bi == 0 { cy0 } else { b0d }, b1d);
        if b1c <= b0c {
            continue;
        }
        // the lines whose glyphs or shadows reach this band
        let key: String = (0..tb.lines.len())
            .filter(|&li| {
                let ly = ty + tb.bases[li];
                ly + reach >= b0c && ly - reach <= b1c
            })
            .map(|li| line_text[li].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let tb2 = tb.clone();
        f.layer(name, &format!("{key}|{b0c}|{b1c}|{cw}"), 1.0, move |dpr| {
            let mut p = Painter::new(cx0, b0c, cx1 - cx0, b1c - b0c, dpr);
            p.text(&tb2, tx, ty, cw, "left", 1.0);
            p
        });
    }
}

/// The topmost click target at CSS px (x, y)
pub fn hud_hit_test(g: &Game, x: f64, y: f64) -> String {
    for (id, r, enabled) in &g.hud_ui.hits {
        if x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3] {
            return if *enabled { id.to_string() } else { String::new() };
        }
    }
    String::new()
}
