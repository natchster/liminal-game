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

type Rgba = [f64; 4];

// ------------------------------------------------------------ CSS values
/// parse a CSS colour (straight alpha)
pub fn css_color(s: &str) -> Option<Rgba> {
    engine::css::parse_css_color(s)
}
fn col(s: &str) -> Rgba {
    css_color(s).unwrap_or([0.0, 0.0, 0.0, 1.0])
}
fn rgba(r: f64, g: f64, b: f64, a: f64) -> Rgba {
    [r / 255.0, g / 255.0, b / 255.0, a]
}
fn css_rgba(c: Rgba) -> String {
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
/// a "Courier New" face at a size (the HUD uses no other family)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
}
impl Font {
    pub fn new(size: f64) -> Font {
        Font { size, bold: false, italic: false }
    }
    fn bold(mut self) -> Font {
        self.bold = true;
        self
    }
    fn italic(mut self) -> Font {
        self.italic = true;
        self
    }
    /// Ascent of Courier New, rounded to whole px per size
    pub fn ascent(&self) -> f64 {
        (self.size * 0.83252).round()
    }
    pub fn descent(&self) -> f64 {
        (self.size * 0.30029).round()
    }
    /// `line-height: normal`
    pub fn normal_lh(&self) -> f64 {
        self.ascent() + self.descent()
    }
    /// glyph advance (Courier New 0.6001em; the
    /// geometric symbols come from a fallback face at 0.602em; ⛧ wider)
    fn advance(&self, ch: char) -> f64 {
        let em = match ch {
            '⛧' => 0.788,
            '◆' | '◇' | '★' | '✦' | '◀' | '▶' | '▮' | '▯' => 0.6020625,
            _ => 0.600109375,
        };
        em * self.size
    }
}

/// A styled piece of inline text (a text node or a `<span style>`).
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub text: String,
    pub color: Rgba,
    pub shadows: Vec<Shadow>,
    pub opacity: f64,
    pub ls: f64,
}

/// `inner_html` with the HUD's inline markup (`<span style=...>`, `<br>`,
/// entities) → runs, inheriting `base` for anything a span doesn't set.
pub fn parse_inline_html(html: &str, base: &Run) -> Vec<Run> {
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
    let b = html.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e).unwrap_or(b.len() - 1);
            let tag = &html[i + 1..end];
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
            let ch = html[i..].chars().next().unwrap();
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

/// (ascent, descent) of the faces a line uses: Courier New, plus the fallback
/// faces for the symbols (a Menlo-like face for ◆★✦◀▶▮▯◇, another for ⛧)
fn face_metrics(ch: char, size: f64) -> Option<(f64, f64)> {
    match ch {
        '◆' | '◇' | '★' | '✦' | '◀' | '▶' | '▮' | '▯' => Some(((size * 0.928).round(), (size * 0.236).round())),
        '⛧' => Some(((size * 0.88).round(), (size * 0.37).round())),
        _ => None,
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
            // (fallback faces only count with `line-height: normal`)
            if line_h.is_none() {
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
    fn tr(&mut self) {
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
                let plain = face_metrics(ch, fsz).is_none() && (ch as u32) < 0x2000;
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
// HUD wants the real Courier New, so it rasterises its glyphs itself with
// CoreText (engine::cg): Courier New plus its fallback faces for symbols.
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextSetShouldSmoothFonts(c: engine::cg::CGContextRef, b: bool);
}
thread_local! { static CT_FONTS: RefCell<HashMap<(bool, bool, u64), usize>> = RefCell::new(HashMap::new()); }
fn ct_font(font: Font, px: f64) -> engine::cg::CTFontRef {
    use engine::cg::*;
    CT_FONTS.with(|c| {
        let key = (font.bold, font.italic, px.to_bits());
        if let Some(&f) = c.borrow().get(&key) {
            return f as CTFontRef;
        }
        let name = match (font.bold, font.italic) {
            (false, false) => "CourierNewPSMT",
            (true, false) => "CourierNewPS-BoldMT",
            (false, true) => "CourierNewPS-ItalicMT",
            (true, true) => "CourierNewPS-BoldItalicMT",
        };
        let f = unsafe {
            let n = cfstr(name);
            let f = CTFontCreateWithName(n, px, std::ptr::null());
            CFRelease(n);
            f
        };
        c.borrow_mut().insert(key, f as usize);
        f
    })
}
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    static kCTKernAttributeName: engine::cg::CFTypeRef;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFNumberCreate(a: engine::cg::CFTypeRef, ty: isize, v: *const std::ffi::c_void) -> engine::cg::CFTypeRef;
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
        let f = ct_font(font, font.size * dpr);
        for (text, x, y, c, kern) in segs {
            let s = cfstr(text);
            let kv = CFNumberCreate(std::ptr::null(), 6 /* kCFNumberFloat64Type */, kern as *const f64 as *const std::ffi::c_void);
            let keys = [kCTFontAttributeName, kCTForegroundColorFromContextAttributeName, kCTKernAttributeName];
            let vals = [f as CFTypeRef, kCFBooleanTrue, kv];
            let dict = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr(),
                vals.as_ptr(),
                3,
                &kCFTypeDictionaryKeyCallBacks as *const _ as _,
                &kCFTypeDictionaryValueCallBacks as *const _ as _,
            );
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
            CFRelease(kv);
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
    /// keyframe animations: (start ms, element version seen)
    anims: HashMap<&'static str, (f64, u64)>,
    next_id: u64,
    grain_id: u64,
    /// click targets of the last frame, topmost first: (id, rect css px, enabled)
    hits: Vec<(&'static str, [f64; 4], bool)>,
    /// the frame's layers in paint order, by box name (tests dump them)
    pub painted: Vec<&'static str>,
}
fn hash_of(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// The frame being built.
struct Frame<'a> {
    ui: &'a mut HudUi,
    out: Vec<HudLayer>,
    w: f64,
    h: f64,
    dpr: f64,
    now: f64,
}
impl Frame<'_> {
    /// a CSS transition of `key` toward `target` (starts when the target changes)
    fn tr(&mut self, key: &str, target: f64, dur: f64, ease: Ease) -> f64 {
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
    /// the start time of a keyframe animation that (re)starts whenever the
    /// element changes while it has `class` (the HUD removes and re-adds the
    /// class to restart it)
    fn anim_start(&mut self, name: &'static str, el: &Element, class: &str) -> Option<f64> {
        if !el.has_class(class) {
            self.ui.anims.remove(name);
            return None;
        }
        let now = self.now;
        let e = self.ui.anims.entry(name).or_insert((now, el.version));
        if e.1 != el.version {
            *e = (now, el.version);
        }
        Some(e.0)
    }
    /// the cached canvas `name`, repainted by `paint` when `key` changes
    fn boxed(&mut self, name: &'static str, key: &str, paint: impl FnOnce(f64) -> Painter) -> (Rc<RefCell<Canvas>>, u64, u64, [f32; 4]) {
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
    fn layer(&mut self, name: &'static str, key: &str, opacity: f64, paint: impl FnOnce(f64) -> Painter) {
        if opacity <= 0.0005 {
            return;
        }
        let (canvas, id, version, rect) = self.boxed(name, key, paint);
        self.push_canvas(name, canvas, id, version, rect, opacity, LayerBlend::Normal, false);
    }
    fn push_canvas(
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
    fn fill(&mut self, c: Rgba, blend: LayerBlend) {
        if c[3] <= 0.0005 {
            return;
        }
        self.ui.painted.push("fill");
        self.out.push(HudLayer::Fill { color: f32x4(c), blend });
    }
    /// an element's `background`: a colour fill or a radial gradient
    fn background(&mut self, bg: &str, opacity: f64, blend: LayerBlend) {
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

const HUD_COLOR: &str = "#d8cf9f";
fn hud_shadows() -> Vec<Shadow> {
    parse_shadows("0 0 4px #000, 0 0 2px #000")
}
/// the text of an element (`inner_html` wins when set)
fn runs_of(el: &Element, base: &Run) -> Vec<Run> {
    if !el.html.is_empty() {
        parse_inline_html(&el.html, base)
    } else {
        let mut r = base.clone();
        r.text = el.text.clone();
        vec![r]
    }
}
fn base_run(color: Rgba, shadows: Vec<Shadow>, ls: f64) -> Run {
    Run { text: String::new(), color, shadows, opacity: 1.0, ls }
}

/// the text margin a box needs for its shadows
fn shadow_margin(sh: &[Shadow]) -> f64 {
    sh.iter().map(|s| s.blur * 1.6 + s.ox.abs().max(s.oy.abs()) + s.spread.abs()).fold(0.0, f64::max) + 2.0
}

/// a one-element text box: block `tb` with its line boxes at (x, y), `box_w` wide
fn text_layer(f: &mut Frame, name: &'static str, tb: TextBlock, x: f64, y: f64, box_w: f64, align: &'static str, opacity: f64) {
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
        let op = opacity_of(&d.grain, 0.10);
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
    // ---- z 10: #overlay
    paint_overlay(&mut f, g);
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
    let d = &g.ui;
    let (w, h) = (f.w, f.h);
    let hc = col(HUD_COLOR);
    let hs = hud_shadows();
    let body = &d.body;
    let special = body.has_class("in-stairwell") || body.has_class("in-loop");
    let txt = |el: &Element, size: f64, ls: f64, color: Rgba, sh: &[Shadow], ws: Ws, max_w: Option<f64>| -> TextBlock {
        TextBlock::new(runs_of(el, &base_run(color, sh.to_vec(), ls)), Font::new(size), ws, max_w, None)
    };
    // #level / #seedTag / #objective (top left)
    text_layer(f, "level", txt(&d.level, 15.0, 4.0, hc, &hs, Ws::Normal, Some(w - 20.0)), 20.0, 16.0, 0.0, "left", 1.0);
    text_layer(f, "seedTag", txt(&d.seed_tag, 9.0, 2.0, hc, &hs, Ws::Normal, Some(w - 20.0)), 20.0, 38.0, 0.0, "left", 0.35);
    if !special {
        text_layer(f, "objective", txt(&d.objective, 11.0, 2.0, hc, &hs, Ws::Normal, Some(w - 20.0)), 20.0, 56.0, 0.0, "left", 0.65);
    }
    // #bottles / #batteries / #papers (bottom left)
    for (name, el, bottom) in [("bottles", &d.bottles, 18.0), ("batteries", &d.batteries, 36.0), ("papers", &d.papers, 54.0)] {
        let tb = txt(el, 12.0, 2.0, hc, &hs, Ws::Normal, Some(w - 20.0));
        let y = h - bottom - tb.height();
        text_layer(f, name, tb, 20.0, y, 0.0, "left", 0.85);
    }
    // the meters: border-box (x, y, 2 + width, 2 + height), cells notched by ::after
    let meter = |f: &mut Frame, name: &'static str, x: f64, y: f64, iw: f64, ih: f64, border: Rgba, frac: f64, bar: Rgba, opacity: f64| {
        let key = format!("{x}|{y}|{iw}|{ih}|{border:?}|{frac}|{bar:?}");
        f.layer(name, &key, opacity, move |dpr| {
            let mut p = Painter::new(x, y, iw + 2.0, ih + 2.0, dpr);
            p.rect(x, y, iw + 2.0, ih + 2.0, [0.0, 0.0, 0.0, 0.35]);
            p.border(x, y, iw + 2.0, ih + 2.0, 1.0, border);
            p.rect(x + 1.0, y + 1.0, (iw * frac).clamp(0.0, iw), ih, bar);
            let mut cx = 7.0;
            while cx < iw {
                p.rect(x + 1.0 + cx, y + 1.0, 2.0f64.min(iw - cx), ih, rgba(4.0, 4.0, 2.0, 0.92));
                cx += 9.0;
            }
            p
        });
    };
    {
        let frac = f.tr("sanitybar.width", pct_of(&d.sanitybar), 0.2, Ease::Linear);
        let low = f.tr("sanitybar.low", if d.sanity.has_class("low") { 1.0 } else { 0.0 }, 0.3, Ease::Ease);
        let bar = lerp_c(col("#a8b8d8"), col("#b53a3a"), low);
        meter(f, "sanity", w / 2.0 - 111.0, h - 34.0 - 7.0, 220.0, 5.0, rgba(180.0, 190.0, 220.0, 0.35), frac, bar, 1.0);
    }
    {
        let frac = f.tr("staminabar.width", pct_of(&d.staminabar), 0.08, Ease::Linear);
        let bar = if d.stamina.has_class("low") { col("#a33") } else { hc };
        meter(f, "stamina", w / 2.0 - 111.0, h - 22.0 - 9.0, 220.0, 7.0, rgba(216.0, 207.0, 159.0, 0.4), frac, bar, 1.0);
    }
    for (name, el, bottom) in [("sanityLabel", &d.sanity_label, 31.0), ("staminaLabel", &d.stamina_label, 19.0)] {
        let tb = txt(el, 10.0, 2.0, hc, &hs, Ws::Normal, None);
        let y = h - bottom - tb.height();
        text_layer(f, name, tb, w / 2.0 - 178.0, y, 0.0, "left", 0.6);
    }
    {
        let frac = f.tr("batterybar.width", pct_of(&d.batterybar), 0.15, Ease::Linear);
        let bar = if d.battery.has_class("low") { col("#cf6c2e") } else { col("#f3e9bd") };
        let op = if d.battery.has_class("off") { 0.35 } else { 1.0 };
        meter(f, "battery", w - 20.0 - 182.0, h - 22.0 - 9.0, 180.0, 7.0, rgba(216.0, 207.0, 159.0, 0.4), frac, bar, op);
        let tb = txt(&d.battery_label, 10.0, 2.0, hc, &hs, Ws::Normal, None);
        let (bw, y) = (tb.width(), h - 32.0 - tb.height());
        text_layer(f, "batteryLabel", tb, w - 20.0 - bw, y, 0.0, "left", 0.6);
    }
    // #dot
    {
        let (cx, cy) = (w / 2.0, h / 2.0);
        f.layer("dot", &format!("{cx}|{cy}"), 1.0, move |dpr| {
            let mut p = Painter::new(cx - 2.0, cy - 2.0, 4.0, 4.0, dpr);
            p.tr();
            p.c.set_fill_style("rgba(255,255,255,0.55)");
            p.c.begin_path();
            p.c.arc(cx, cy, 2.0, 0.0, std::f64::consts::PI * 2.0);
            p.c.fill();
            p
        });
    }
    // #prompt / #msg / #omen (centred with translateX(-50%))
    {
        let tb = txt(&d.prompt, 14.0, 3.0, hc, &hs, Ws::NoWrap, None);
        let bw = tb.width();
        text_layer(f, "prompt", tb, w / 2.0 - bw / 2.0, (h * 0.62).floor(), bw, "left", 1.0);
    }
    {
        let el = &d.msg;
        let color = el.style("color").and_then(css_color).unwrap_or(hc);
        let sh = el.style("text-shadow").map(parse_shadows).unwrap_or_else(|| hs.clone());
        let op = f.tr("msg.opacity", opacity_of(el, 0.0), 0.4, Ease::Ease);
        let tb = txt(el, 16.0, 3.0, color, &sh, Ws::NoWrap, None);
        let bw = tb.width();
        text_layer(f, "msg", tb, w / 2.0 - bw / 2.0, (h * 0.34).floor(), bw, "center", op);
    }
    {
        let el = &d.omen;
        let op = f.tr("omen.opacity", opacity_of(el, 0.0), 1.8, Ease::Ease);
        let tb = TextBlock::new(
            runs_of(el, &base_run(col("#c9bea4"), parse_shadows("0 0 6px rgba(0,0,0,0.9)"), 2.0)),
            Font::new(12.0).italic(),
            Ws::NoWrap,
            None,
            None,
        );
        let bw = tb.width();
        text_layer(f, "omen", tb, w / 2.0 - bw / 2.0, (h * 0.34).floor() + 30.0, bw, "center", op);
    }
    {
        let el = &d.crouch_ind;
        let op = f.tr("crouchInd.opacity", opacity_of(el, if el.has_class("on") { 0.7 } else { 0.0 }), 0.3, Ease::Ease);
        let tb = txt(el, 10.0, 3.0, col("#a8b8d8"), &hs, Ws::Normal, None);
        let (bw, y) = (tb.width(), h - 8.0 - tb.height());
        text_layer(f, "crouchInd", tb, w / 2.0 - bw / 2.0, y, bw, "left", op);
    }
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
    // the five HUD medals (top right)
    let medals: [(&'static str, &Element, f64, &str, &str, &str, &str, f64); 5] = [
        (
            "loreMedal",
            &d.lore_medal,
            84.0,
            "#f8e0a0",
            "0 0 6px rgba(248, 224, 160, 0.55)",
            "rgba(248, 224, 160, 0.5)",
            "rgba(20, 15, 8, 0.35)",
            0.85,
        ),
        (
            "survivorMedal",
            &d.survivor_medal,
            106.0,
            "#a8d8f0",
            "0 0 6px rgba(168, 216, 240, 0.55)",
            "rgba(168, 216, 240, 0.5)",
            "rgba(15, 20, 25, 0.35)",
            0.85,
        ),
        (
            "storytellerMedal",
            &d.storyteller_medal,
            128.0,
            "#d8b0f0",
            "0 0 6px rgba(216, 176, 240, 0.55)",
            "rgba(216, 176, 240, 0.5)",
            "rgba(24, 15, 30, 0.35)",
            0.85,
        ),
        (
            "completionistMedal",
            &d.completionist_medal,
            150.0,
            "#ffedb0",
            "0 0 8px rgba(255, 237, 176, 0.7)",
            "rgba(255, 237, 176, 0.6)",
            "rgba(30, 25, 10, 0.4)",
            0.9,
        ),
        (
            "ironwillMedal",
            &d.ironwill_medal,
            172.0,
            "#a0f0d8",
            "0 0 6px rgba(160, 240, 216, 0.55)",
            "rgba(160, 240, 216, 0.55)",
            "rgba(10, 25, 20, 0.4)",
            0.85,
        ),
    ];
    for (name, el, top, color, shadow, border, bg, op) in medals {
        if !el.has_class("on") {
            f.ui.anims.remove(name);
            continue;
        }
        let tb = txt(el, 10.0, 3.0, col(color), &parse_shadows(shadow), Ws::Normal, None);
        let (bw, bh) = (tb.width() + 18.0, tb.height() + 10.0);
        let (x, y) = (w - 20.0 - bw, top);
        let key = format!("{:?}|{x}|{y}", tb.runs);
        let (bc, bgc) = (col(border), col(bg));
        let m = shadow_margin(&tb.runs[0].shadows);
        let (canvas, id, ver, rect) = f.boxed(name, &key, move |dpr| {
            let mut p = Painter::new(x - m, y - m, bw + 2.0 * m, bh + 2.0 * m, dpr);
            p.rect(x, y, bw, bh, bgc);
            p.border(x, y, bw, bh, 1.0, bc);
            p.text(&tb, x + 9.0, y + 5.0, 0.0, "left", 1.0);
            p
        });
        // .medal-replay: 1.4 s ease-out, 20%: scale(1.2) brightness(2.2)
        let (mut scale, mut bright) = (1.0, 1.0);
        if let Some(t0) = f.anim_start(name, el, "medal-replay") {
            let x = (f.now - t0) / 1400.0;
            if x < 1.0 {
                let (k, a, b) = if x < 0.2 { (Ease::EaseOut.at(x / 0.2), 0.0, 1.0) } else { (Ease::EaseOut.at((x - 0.2) / 0.8), 1.0, 0.0) };
                let e = a + (b - a) * k;
                scale = 1.0 + 0.2 * e;
                bright = 1.0 + 1.2 * e;
            }
        }
        let r = if scale != 1.0 {
            let (cx, cy) = ((x + bw / 2.0) / w, (y + bh / 2.0) / h);
            let s = scale as f32;
            [cx as f32 + (rect[0] - cx as f32) * s, cy as f32 + (rect[1] - cy as f32) * s, rect[2] * s, rect[3] * s]
        } else {
            rect
        };
        f.push_canvas(name, canvas.clone(), id, ver, r, op, LayerBlend::Normal, scale != 1.0);
        // brightness(b): the extra (b − 1) of the element added on top
        let mut extra = op * (bright - 1.0);
        while extra > 0.0005 {
            f.push_canvas(name, canvas.clone(), id, ver, r, extra.min(1.0), LayerBlend::Add, scale != 1.0);
            extra -= 1.0;
        }
    }
    // #debug (z 5 inside #hud)
    if d.debug.has_class("on") {
        paint_debug(f, g);
    }
    // #achievementFlash (z 7 inside #hud)
    {
        let el = &d.achievement_flash;
        let op = f.tr("achievementFlash.opacity", opacity_of(el, if el.has_class("on") { 1.0 } else { 0.0 }), 0.5, Ease::EaseOut);
        f.background(
            "radial-gradient(circle at center, rgba(255, 240, 190, 0.85) 0%, rgba(255, 220, 120, 0.35) 40%, transparent 75%)",
            op,
            LayerBlend::Normal,
        );
    }
    // #journal (z 8 inside #hud)
    if d.journal.has_class("on") {
        paint_journal(f, g);
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

/// a `<button>`'s box: (width, height) for its text (buttons have no text-shadow)
fn button_size(tb: &TextBlock, pad_x: f64, pad_y: f64) -> (f64, f64) {
    (tb.width() + 2.0 * pad_x + 2.0, tb.font.normal_lh() + 2.0 * pad_y + 2.0)
}

/// #journal.on: the dim backdrop, the paper panel, and its body paragraph
/// as its own layer (the cover's `journal_cover_fade_in` animates it)
fn paint_journal(f: &mut Frame, g: &Game) {
    let d = &g.ui;
    let (w, h) = (f.w, f.h);
    let ink = col("#1a1408");
    let hs = hud_shadows(); // inherited from #hud
    f.fill(rgba(6.0, 5.0, 2.0, 0.88), LayerBlend::Normal);
    let mk = |el: &Element, font: Font, ls: f64, color: Rgba, sh: &[Shadow], ws: Ws, mw: Option<f64>, lh: Option<f64>| {
        TextBlock::new(runs_of(el, &base_run(color, sh.to_vec(), ls)), font, ws, mw, lh)
    };
    // max-content widths decide the panel width (clamped to 520..680)
    let title0 = mk(&d.journal_title, Font::new(22.0).bold(), 8.0, ink, &hs, Ws::Normal, None, None);
    let prev0 = mk(&d.journal_prev, Font::new(12.0), 2.0, ink, &[], Ws::Normal, None, None);
    let next0 = mk(&d.journal_next, Font::new(12.0), 2.0, ink, &[], Ws::Normal, None, None);
    let idx0 = mk(&d.journal_idx, Font::new(12.0), 2.0, ink, &hs, Ws::Normal, None, None);
    let et0 = mk(&d.journal_entry_title, Font::new(15.0).bold(), 3.0, col("#5a2a1a"), &hs, Ws::Normal, None, Some(25.5));
    let body0 = mk(&d.journal_entry_body, Font::new(13.0), 0.0, ink, &hs, Ws::PreLine, None, Some(13.0 * 1.7));
    let hint0 = mk(&d.journal_hint, Font::new(11.0), 3.0, ink, &hs, Ws::Normal, None, None);
    let (pw, ph) = button_size(&prev0, 14.0, 6.0);
    let (nw, _) = button_size(&next0, 14.0, 6.0);
    let maxc =
        [title0.width(), pw + idx0.width() + nw, et0.width() + 8.0, body0.width() + 8.0, hint0.width()].iter().cloned().fold(0.0, f64::max);
    let cw = maxc.clamp(520.0, 680.0);
    // wrapped at the panel width
    let title = mk(&d.journal_title, Font::new(22.0).bold(), 8.0, ink, &hs, Ws::Normal, Some(cw), None);
    let et = mk(&d.journal_entry_title, Font::new(15.0).bold(), 3.0, col("#5a2a1a"), &hs, Ws::Normal, Some(cw - 8.0), Some(25.5));
    let body = mk(&d.journal_entry_body, Font::new(13.0), 0.0, ink, &hs, Ws::PreLine, Some(cw - 8.0), Some(13.0 * 1.7));
    let hint = mk(&d.journal_hint, Font::new(11.0), 3.0, ink, &hs, Ws::Normal, Some(cw), None);
    let title_h = title.height() + 10.0 + 1.0;
    let nav_h = ph.max(idx0.height());
    let entry_inner = (if et.is_empty() { 0.0 } else { et.height() + 12.0 } + body.height()).max(140.0);
    let entry_h = entry_inner + 24.0;
    let content_h = title_h + 18.0 + nav_h + 14.0 + entry_h + 22.0 + hint.height();
    let (ow, oh) = (cw + 84.0, content_h + 68.0);
    let (ox, oy) = ((w - ow) / 2.0, (h - oh) / 2.0);
    let (cx, cy) = (ox + 42.0, oy + 34.0);
    let nav_y = cy + title_h + 18.0;
    let entry_y = nav_y + nav_h + 14.0;
    let body_y = entry_y + 12.0 + if et.is_empty() { 0.0 } else { et.height() + 12.0 };
    let prev_dis = d.journal_prev.attrs.get("disabled").map(|v| v == "true").unwrap_or(false);
    let next_dis = d.journal_next.attrs.get("disabled").map(|v| v == "true").unwrap_or(false);
    let prev_r = [cx, nav_y + (nav_h - ph) / 2.0, pw, ph];
    let next_r = [cx + cw - nw, nav_y + (nav_h - ph) / 2.0, nw, ph];
    // topmost first: the journal (z 8) over the debug panel
    f.ui.hits
        .splice(0..0, [("journal-prev", prev_r, !prev_dis), ("journal-next", next_r, !next_dis), ("journal", [0.0, 0.0, w, h], false)]);
    let key = format!(
        "{:?}{:?}{:?}{:?}{:?}{:?}|{cw}|{content_h}|{prev_dis}|{next_dis}",
        title.runs, prev0.runs, next0.runs, idx0.runs, et.runs, hint.runs
    );
    let idx_x = prev_r[0] + pw + ((next_r[0] - (prev_r[0] + pw)) - idx0.width()) / 2.0;
    let idx_y = nav_y + (nav_h - idx0.height()) / 2.0;
    f.layer("journal", &key, 1.0, move |dpr| {
        let sm = 12.0 + 32.0 * 1.6;
        let mut p = Painter::new(ox - sm, oy - sm, ow + 2.0 * sm, oh + 2.0 * sm + 12.0, dpr);
        paint_box_shadow(
            &mut p,
            ox,
            oy,
            ow,
            oh,
            &Shadow { ox: 0.0, oy: 12.0, blur: 32.0, spread: 0.0, color: [0.0, 0.0, 0.0, 0.65], inset: false },
        );
        p.rect(ox, oy, ow, oh, rgba(232.0, 218.0, 180.0, 0.94));
        p.border(ox, oy, ow, oh, 2.0, col("#4a3d24"));
        p.text(&title, cx, cy, cw, "center", 1.0);
        p.rect(cx, cy + title.height() + 10.0, cw, 1.0, col("#4a3d24"));
        for (tb, r, dis) in [(&prev0, prev_r, prev_dis), (&next0, next_r, next_dis)] {
            let mut b = Painter::new(r[0], r[1], r[2], r[3], dpr);
            b.border(r[0], r[1], r[2], r[3], 1.0, col("#4a3d24"));
            b.text(tb, r[0] + 15.0, r[1] + 7.0, 0.0, "left", 1.0);
            p.blit(&b, if dis { 0.35 } else { 1.0 });
        }
        p.text(&idx0, idx_x, idx_y, 0.0, "left", 0.6);
        p.text(&et, cx + 4.0, entry_y + 12.0, cw - 8.0, "left", 1.0);
        p.text(&hint, cx, entry_y + entry_h + 22.0, cw, "center", 0.5);
        p
    });
    // the body paragraph; `.cover-fade` = 0.55 s ease-out from opacity 0, translateY(6px)
    let (mut op, mut dy) = (1.0, 0.0);
    if let Some(t0) = f.anim_start("journal-body", &d.journal_entry_body, "cover-fade") {
        let x = (f.now - t0) / 550.0;
        if x < 1.0 {
            let e = Ease::EaseOut.at(x);
            op = e;
            dy = 6.0 * (1.0 - e);
        }
    }
    if !body.is_empty() && op > 0.0005 {
        let key = format!("{:?}|{cx}|{body_y}|{cw}", body.runs);
        let (canvas, id, ver, mut rect) = f.boxed("journal-body", &key, move |dpr| {
            let m = 6.0;
            let mut p = Painter::new(cx + 4.0 - m, body_y - m, cw - 8.0 + 2.0 * m, body.height() + 2.0 * m, dpr);
            p.text(&body, cx + 4.0, body_y, cw - 8.0, "left", 1.0);
            p
        });
        rect[1] += (dy / h) as f32;
        f.push_canvas("journal-body", canvas, id, ver, rect, op, LayerBlend::Normal, dy != 0.0);
    }
}

/// an outset box-shadow of the border box (x, y, w, h), painted outside it
fn paint_box_shadow(p: &mut Painter, x: f64, y: f64, w: f64, h: f64, s: &Shadow) {
    let (cw, ch) = (p.c.width() as usize, p.c.height() as usize);
    let mut img = p.c.create_image_data(cw as f64, ch as f64);
    let sigma = s.blur * BLUR_SIGMA;
    let (sx0, sx1, sy0, sy1) = (x + s.ox - s.spread, x + w + s.ox + s.spread, y + s.oy - s.spread, y + h + s.oy + s.spread);
    for j in 0..ch {
        let py = p.oy + (j as f64 + 0.5) / p.dpr;
        let fy = blurred_span(py, sy0, sy1, sigma);
        for i in 0..cw {
            let px = p.ox + (i as f64 + 0.5) / p.dpr;
            if px >= x && px < x + w && py >= y && py < y + h {
                continue;
            } // clipped under the box
            let a = s.color[3] * blurred_span(px, sx0, sx1, sigma) * fy;
            let k = (j * cw + i) * 4;
            img.data[k] = (s.color[0] * 255.0) as u8;
            img.data[k + 1] = (s.color[1] * 255.0) as u8;
            img.data[k + 2] = (s.color[2] * 255.0) as u8;
            img.data[k + 3] = (a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    let mut sc = Canvas::new(cw as u32, ch as u32);
    sc.put_image_data(&img, 0, 0);
    p.c.reset_transform();
    p.c.draw_image(&sc, 0, 0);
}

/// the menu medals' earned colours: (key, colour, border, shadow, shimmer delay s)
const MENU_MEDALS: [(&str, &str, &str, &str, f64); 5] = [
    ("archivist", "#f8e0a0", "rgba(248, 224, 160, 0.6)", "0 0 6px rgba(248, 224, 160, 0.55)", 0.0),
    ("survivor", "#a8d8f0", "rgba(168, 216, 240, 0.6)", "0 0 6px rgba(168, 216, 240, 0.55)", 0.8),
    ("storyteller", "#d8b0f0", "rgba(216, 176, 240, 0.6)", "0 0 6px rgba(216, 176, 240, 0.55)", 1.6),
    ("completionist", "#ffedb0", "rgba(255, 237, 176, 0.7)", "0 0 8px rgba(255, 237, 176, 0.7)", 2.4),
    ("ironwill", "#a0f0d8", "rgba(160, 240, 216, 0.6)", "0 0 6px rgba(160, 240, 216, 0.55)", 3.2),
];

/// #overlay (unless .hidden): flex column, centred
fn paint_overlay(f: &mut Frame, g: &Game) {
    let d = &g.ui;
    if d.overlay.has_class("hidden") || !d.overlay.shown() {
        return;
    }
    let (w, h) = (f.w, f.h);
    let fg = col(HUD_COLOR);
    f.fill(rgba(6.0, 5.0, 2.0, 0.88), LayerBlend::Normal);
    let mk = |el: &Element, font: Font, ls: f64, sh: Vec<Shadow>, mw: f64, lh: Option<f64>| {
        TextBlock::new(runs_of(el, &base_run(fg, sh, ls)), font, Ws::Normal, Some(mw), lh)
    };
    let title = mk(&d.overlay_title, Font::new(34.0), 12.0, parse_shadows("0 0 18px rgba(216,207,159,0.35)"), w, None);
    let sub = mk(&d.overlay_sub, Font::new(13.0), 2.0, vec![], 560.0f64.min(w), Some(13.0 * 1.8));
    let controls = mk(&d.overlay_controls, Font::new(11.0), 2.0, vec![], w, Some(22.0));
    let show_btns = d.overlay_buttons.style("display") != Some("none");
    let btns: Vec<(&'static str, TextBlock)> = [("btn-continue", &d.btn_continue), ("btn-new", &d.btn_new)]
        .into_iter()
        .filter(|(_, el)| el.shown())
        .map(|(id, el)| {
            (
                id,
                TextBlock::new(
                    vec![Run { text: el.text.to_uppercase(), color: fg, shadows: vec![], opacity: 1.0, ls: 3.0 }],
                    Font::new(13.0),
                    Ws::NoWrap,
                    None,
                    None,
                ),
            )
        })
        .collect();
    let btn_sizes: Vec<(f64, f64)> = btns.iter().map(|(_, tb)| button_size(tb, 24.0, 12.0)).collect();
    let btn_row_w = btn_sizes.iter().map(|s| s.0).sum::<f64>() + 16.0 * (btns.len().max(1) - 1) as f64;
    let btn_row_h = btn_sizes.iter().map(|s| s.1).fold(0.0, f64::max);
    // menu medals: wrap rows within min(max-content, 640)
    let mm: Vec<(&Element, usize)> =
        [&d.menu_medal_archivist, &d.menu_medal_survivor, &d.menu_medal_storyteller, &d.menu_medal_completionist, &d.menu_medal_ironwill]
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e, i))
            .collect();
    let medal_tbs: Vec<(TextBlock, bool, usize)> = mm
        .iter()
        .map(|(el, i)| {
            let earned = el.has_class("earned");
            let (c, sh) =
                if earned { (col(MENU_MEDALS[*i].1), parse_shadows(MENU_MEDALS[*i].3)) } else { (rgba(180.0, 170.0, 140.0, 0.45), vec![]) };
            (TextBlock::new(runs_of(el, &base_run(c, sh, 3.0)), Font::new(10.0), Ws::NoWrap, None, None), earned, *i)
        })
        .collect();
    let item_w: Vec<f64> = medal_tbs.iter().map(|(tb, _, _)| tb.width() + 22.0).collect();
    // flex rows stretch their cells to the tallest (the symbol faces grow the line box)
    let item_hs: Vec<f64> = medal_tbs.iter().map(|(tb, _, _)| tb.height() + 12.0).collect();
    let total_w = item_w.iter().sum::<f64>() + 10.0 * (item_w.len() - 1) as f64;
    let mw = total_w.min(640.0).min(w);
    let mut rows: Vec<Vec<usize>> = vec![vec![]];
    let mut rw = 0.0;
    for (i, iw) in item_w.iter().enumerate() {
        let need = if rows.last().unwrap().is_empty() { *iw } else { rw + 10.0 + iw };
        if need > mw + 0.01 && !rows.last().unwrap().is_empty() {
            rows.push(vec![i]);
            rw = *iw;
        } else {
            rows.last_mut().unwrap().push(i);
            rw = need;
        }
    }
    let row_h: Vec<f64> = rows.iter().map(|r| r.iter().map(|&i| item_hs[i]).fold(0.0, f64::max)).collect();
    let medals_h = row_h.iter().sum::<f64>() + 10.0 * (rows.len() - 1) as f64;
    let hint = mk(&d.overlay_hint, Font::new(13.0), 4.0, vec![], w, None);
    // the column: margins of each item (no collapsing in flex)
    let mut total = title.height() + 10.0 + 6.0 + sub.height() + 6.0 + 18.0 + controls.height() + 6.0;
    if show_btns {
        total += 22.0 + btn_row_h;
    }
    total += 18.0 + medals_h + 20.0 + hint.height() + 13.0;
    let mut y = (h - total) / 2.0;
    let title_y = y;
    y += title.height() + 10.0 + 6.0;
    let sub_y = y;
    y += sub.height() + 6.0 + 18.0;
    let ctrl_y = y;
    y += controls.height() + 6.0;
    let mut btn_rects: Vec<[f64; 4]> = vec![];
    if show_btns {
        y += 22.0;
        let mut x = (w - btn_row_w) / 2.0;
        for s in &btn_sizes {
            btn_rects.push([x, y + (btn_row_h - s.1) / 2.0, s.0, s.1]);
            x += s.0 + 16.0;
        }
        y += btn_row_h;
    }
    y += 18.0;
    let medals_y = y;
    y += medals_h + 20.0;
    let hint_y = y;
    // topmost first: the overlay (z 10) over everything
    let mut hits: Vec<(&'static str, [f64; 4], bool)> = btns.iter().zip(btn_rects.iter()).map(|((id, _), r)| (*id, *r, true)).collect();
    hits.push(("overlay", [0.0, 0.0, w, h], true));
    f.ui.hits.splice(0..0, hits);
    // the static column
    let (tw, sw, ctw) = (
        title.width(),
        sub.width().max(if sub.lines.len() > 1 { 560.0f64.min(w) } else { 0.0 }),
        controls.width().max(if controls.lines.len() > 1 { w } else { 0.0 }),
    );
    let key = format!(
        "{:?}{:?}{:?}{:?}|{title_y}|{show_btns}|{btn_rects:?}",
        title.runs,
        sub.runs,
        controls.runs,
        btns.iter().map(|b| &b.1.runs).collect::<Vec<_>>()
    );
    let top = title_y - 30.0;
    let bottom = medals_y;
    f.layer("overlay", &key, 1.0, move |dpr| {
        let mut p = Painter::new(0.0, top.max(0.0), w, (bottom - top.max(0.0)).max(1.0), dpr);
        p.text(&title, (w - tw) / 2.0, title_y, tw, "center", 1.0);
        p.text(&sub, (w - sw) / 2.0, sub_y, sw, "center", 0.75);
        p.text(&controls, (w - ctw) / 2.0, ctrl_y, ctw, "center", 0.5);
        for ((_, tb), r) in btns.iter().zip(btn_rects.iter()) {
            p.rect(r[0], r[1], r[2], r[3], rgba(20.0, 15.0, 8.0, 0.5));
            p.border(r[0], r[1], r[2], r[3], 1.0, rgba(216.0, 207.0, 159.0, 0.4));
            p.text(tb, r[0] + 25.0, r[1] + 13.0, 0.0, "left", 1.0);
        }
        p
    });
    // the medal row (repainted while an earned cell's shimmer sweeps)
    {
        let now = f.now;
        let mut item_rects = vec![[0.0f64; 4]; item_w.len()];
        let mut ry = medals_y;
        for (ri, row) in rows.iter().enumerate() {
            let rwid = row.iter().map(|&i| item_w[i]).sum::<f64>() + 10.0 * (row.len() - 1) as f64;
            let mut x = (w - mw) / 2.0 + (mw - rwid) / 2.0;
            for &i in row {
                item_rects[i] = [x, ry, item_w[i], row_h[ri]];
                x += item_w[i] + 10.0;
            }
            ry += row_h[ri] + 10.0;
        }
        // ::after left per earned cell (-60% → 130% over the first 55% of 4.5 s, ease-in-out)
        let shimmer: Vec<Option<f64>> = medal_tbs
            .iter()
            .map(|(_, earned, i)| {
                if !earned {
                    return None;
                }
                let t = (now / 1000.0 - MENU_MEDALS[*i].4) / 4.5;
                if t < 0.0 {
                    return Some(-0.6);
                }
                let ph = t.fract();
                Some(if ph < 0.55 { -0.6 + 1.9 * Ease::EaseInOut.at(ph / 0.55) } else { 1.3 })
            })
            .collect();
        let key = format!("{:?}|{item_rects:?}|{shimmer:?}", medal_tbs.iter().map(|m| (&m.0.runs, m.1)).collect::<Vec<_>>());
        f.layer("menu-medals", &key, 1.0, move |dpr| {
            let mut p = Painter::new((w - mw) / 2.0 - 12.0, medals_y - 12.0, mw + 24.0, medals_h + 24.0, dpr);
            for (k, (tb, earned, i)) in medal_tbs.iter().enumerate() {
                let r = item_rects[k];
                let border = if *earned { col(MENU_MEDALS[*i].2) } else { rgba(120.0, 110.0, 90.0, 0.35) };
                p.rect(r[0], r[1], r[2], r[3], rgba(20.0, 15.0, 8.0, 0.35));
                p.border(r[0], r[1], r[2], r[3], 1.0, border);
                p.text(tb, r[0] + 11.0, r[1] + 6.0, 0.0, "left", 1.0);
                if let Some(left) = shimmer[k] {
                    // linear-gradient(105deg, transparent, rgba(255,255,255,0.28), transparent), clipped to the padding box
                    let (px0, py0, pw, ph) = (r[0] + 1.0, r[1] + 1.0, r[2] - 2.0, r[3] - 2.0);
                    let (sx, sw) = (px0 + left * pw, 0.45 * pw);
                    let a = 105f64.to_radians();
                    let (dx, dy) = (a.sin(), -a.cos());
                    let len = (sw * a.sin()).abs() + (ph * a.cos()).abs();
                    let (mx, my) = (sx + sw / 2.0, py0 + ph / 2.0);
                    p.tr();
                    p.c.save();
                    p.c.begin_path();
                    p.c.rect(px0, py0, pw, ph);
                    p.c.clip();
                    let gr = p.c.create_linear_gradient(mx - dx * len / 2.0, my - dy * len / 2.0, mx + dx * len / 2.0, my + dy * len / 2.0);
                    gr.add_color_stop(0.0, "rgba(255,255,255,0)");
                    gr.add_color_stop(0.5, "rgba(255,255,255,0.28)");
                    gr.add_color_stop(1.0, "rgba(255,255,255,0)");
                    p.c.set_fill_style(&gr);
                    p.c.fill_rect(sx, py0, sw, ph);
                    p.c.restore();
                }
            }
            p
        });
    }
    // #overlay-hint: `blink 1.6s infinite` (0.9 → 0.25 → 0.9, ease per half)
    {
        let ph = (f.now / 1600.0).rem_euclid(1.0);
        let op =
            if ph < 0.5 { 0.9 + (0.25 - 0.9) * Ease::Ease.at(ph / 0.5) } else { 0.25 + (0.9 - 0.25) * Ease::Ease.at((ph - 0.5) / 0.5) };
        let hw = hint.width();
        text_layer(f, "overlay-hint", hint, (w - hw) / 2.0, hint_y, hw, "center", op);
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
