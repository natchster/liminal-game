//! A 2D canvas API on CoreGraphics + CoreText: paths, fills and strokes,
//! gradients, shadows, text, image drawing and pixel access. Origin top-left,
//! y down, angles in radians (clockwise on screen), a save / restore state stack.

use crate::cg::*;
use crate::css::{parse_css_color, parse_css_font, FontSpec};
use crate::math::Num;
use crate::texture::{Image, Tex};
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::ffi::c_void;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum GradKind {
    Linear { x0: f64, y0: f64, x1: f64, y1: f64 },
    Radial { x0: f64, y0: f64, r0: f64, x1: f64, y1: f64, r1: f64 },
}
#[derive(Debug)]
pub struct GradientData {
    pub kind: GradKind,
    pub stops: Vec<(f64, [f64; 4])>,
}
/// A linear or radial gradient, used as a fill or stroke style.
#[derive(Clone, Debug)]
pub struct Gradient(pub Rc<RefCell<GradientData>>);
impl Gradient {
    /// Add a stop at `offset` (clamped to 0..1) with a CSS colour.
    pub fn add_color_stop(&self, offset: impl Num, color: &str) {
        let c = parse_css_color(color).unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let o = offset.f().clamp(0.0, 1.0);
        let mut g = self.0.borrow_mut();
        // stops keep insertion order for equal offsets
        let pos = g.stops.iter().position(|s| s.0 > o).unwrap_or(g.stops.len());
        g.stops.insert(pos, (o, c));
    }
}

#[derive(Clone, Debug)]
pub enum Style {
    Color([f64; 4]),
    Gradient(Gradient),
}
pub trait IntoStyle {
    fn into_style(self) -> Style;
}
impl IntoStyle for &str {
    fn into_style(self) -> Style {
        Style::Color(parse_css_color(self).unwrap_or([0.0, 0.0, 0.0, 1.0]))
    }
}
impl IntoStyle for String {
    fn into_style(self) -> Style {
        self.as_str().into_style()
    }
}
impl IntoStyle for &String {
    fn into_style(self) -> Style {
        self.as_str().into_style()
    }
}
impl IntoStyle for &Gradient {
    fn into_style(self) -> Style {
        Style::Gradient(self.clone())
    }
}
impl IntoStyle for Gradient {
    fn into_style(self) -> Style {
        Style::Gradient(self)
    }
}
impl IntoStyle for &Style {
    fn into_style(self) -> Style {
        self.clone()
    }
}
impl IntoStyle for Style {
    fn into_style(self) -> Style {
        self
    }
}

#[derive(Clone, Debug)]
struct State {
    ctm: CGAffineTransform,
    fill: Style,
    stroke: Style,
    line_width: f64,
    line_cap: i32,
    line_join: i32,
    miter: f64,
    dash: Vec<f64>,
    dash_offset: f64,
    alpha: f64,
    composite: i32,
    font: FontSpec,
    font_str: String,
    align: String,
    baseline: String,
    shadow_blur: f64,
    shadow_color: [f64; 4],
    shadow_ox: f64,
    shadow_oy: f64,
}
impl Default for State {
    fn default() -> Self {
        State {
            ctm: CGAffineTransform::IDENTITY,
            fill: Style::Color([0.0, 0.0, 0.0, 1.0]),
            stroke: Style::Color([0.0, 0.0, 0.0, 1.0]),
            line_width: 1.0,
            line_cap: kCGLineCapButt,
            line_join: kCGLineJoinMiter,
            miter: 10.0,
            dash: vec![],
            dash_offset: 0.0,
            alpha: 1.0,
            composite: kCGBlendModeNormal,
            font: parse_css_font("10px sans-serif"),
            font_str: "10px sans-serif".into(),
            align: "start".into(),
            baseline: "alphabetic".into(),
            shadow_blur: 0.0,
            shadow_color: [0.0, 0.0, 0.0, 0.0],
            shadow_ox: 0.0,
            shadow_oy: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextMetrics {
    pub width: f64,
    pub actual_bounding_box_ascent: f64,
    pub actual_bounding_box_descent: f64,
}

thread_local! {
    static FONT_CACHE: RefCell<HashMap<(String, u64, bool, bool), usize>> = RefCell::new(HashMap::new());
    static SRGB: usize = unsafe { CGColorSpaceCreateWithName(kCGColorSpaceSRGB) as usize };
}
fn srgb() -> CGColorSpaceRef {
    SRGB.with(|c| *c as CGColorSpaceRef)
}

/// Which macOS font a CSS font family resolves to (the usual macOS faces for the
/// generic families).
fn family_font_name(family: &str) -> &'static str {
    let f = family.split(',').next().unwrap_or("").trim().trim_matches(|c| c == '\'' || c == '"').to_ascii_lowercase();
    match f.as_str() {
        "serif" | "times" | "times new roman" | "georgia" => "Times New Roman",
        "sans-serif" | "helvetica" | "arial" | "system-ui" | "-apple-system" | "verdana" | "tahoma" => "Helvetica",
        "monospace" | "courier" | "courier new" | "menlo" | "monaco" | "consolas" => "Courier",
        "cursive" | "comic sans ms" | "brush script mt" => "Apple Chancery",
        "impact" => "Impact",
        "fantasy" => "Papyrus",
        // faces of printed packaging and signs (all shipped with macOS)
        "futura" => "Futura",
        "gill sans" => "Gill Sans",
        "rockwell" => "Rockwell",
        "american typewriter" => "American Typewriter",
        "arial rounded mt bold" => "Arial Rounded MT Bold",
        "copperplate" => "Copperplate",
        "optima" => "Optima",
        "avenir next condensed" => "Avenir Next Condensed",
        "marker felt" => "Marker Felt",
        _ => "Helvetica",
    }
}
fn ct_font(spec: &FontSpec) -> CTFontRef {
    let name = family_font_name(&spec.family);
    let key = (name.to_string(), spec.size.to_bits(), spec.bold, spec.italic);
    FONT_CACHE.with(|c| {
        if let Some(&f) = c.borrow().get(&key) {
            return f as CTFontRef;
        }
        unsafe {
            let nm = cfstr(name);
            let mut f = CTFontCreateWithName(nm, spec.size, std::ptr::null());
            CFRelease(nm);
            let mut traits = 0u32;
            if spec.bold {
                traits |= kCTFontBoldTrait;
            }
            if spec.italic {
                traits |= kCTFontItalicTrait;
            }
            if traits != 0 {
                let f2 = CTFontCreateCopyWithSymbolicTraits(f, spec.size, std::ptr::null(), traits, traits);
                if !f2.is_null() {
                    CFRelease(f as CFTypeRef);
                    f = f2;
                } else if spec.italic {
                    // no italic face (Apple Chancery): try bold alone
                    let f3 =
                        CTFontCreateCopyWithSymbolicTraits(f, spec.size, std::ptr::null(), traits & kCTFontBoldTrait, kCTFontBoldTrait);
                    if !f3.is_null() {
                        CFRelease(f as CFTypeRef);
                        f = f3;
                    }
                }
            }
            c.borrow_mut().insert(key, f as usize);
            f
        }
    })
}

/// An offscreen 2D canvas: a premultiplied RGBA bitmap plus its drawing state.
pub struct Canvas {
    w: u32,
    h: u32,
    buf: Vec<u8>,
    ctx: CGContextRef,
    st: State,
    stack: Vec<State>,
    path: CGMutablePathRef,
    cur: Option<(f64, f64)>,   // current point (device space)
    start: Option<(f64, f64)>, // subpath start (device space)
    pub version: u64,
    /// transparency layers open (`begin_layer`): inside one, nothing casts
    /// a shadow of its own; the layer casts it once
    layers: u32,
}

impl std::fmt::Debug for Canvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Canvas({}x{})", self.w, self.h)
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            CGPathRelease(self.path);
            CGContextRelease(self.ctx);
        }
    }
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Canvas {
        let w = w.max(1);
        let h = h.max(1);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let ctx = unsafe {
            CGBitmapContextCreate(
                buf.as_mut_ptr() as *mut c_void,
                w as usize,
                h as usize,
                8,
                (w * 4) as usize,
                srgb(),
                kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
            )
        };
        unsafe {
            // canvas space: origin top-left, y down
            CGContextTranslateCTM(ctx, 0.0, h as f64);
            CGContextScaleCTM(ctx, 1.0, -1.0);
            CGContextSetInterpolationQuality(ctx, kCGInterpolationHigh);
        }
        Canvas {
            w,
            h,
            buf,
            ctx,
            st: State::default(),
            stack: vec![],
            path: unsafe { CGPathCreateMutable() },
            cur: None,
            start: None,
            version: 1,
            layers: 0,
        }
    }
    pub fn width(&self) -> f64 {
        self.w as f64
    }
    pub fn height(&self) -> f64 {
        self.h as f64
    }

    // ------------------------------------------------------------ state
    pub fn save(&mut self) {
        self.stack.push(self.st.clone());
        unsafe { CGContextSaveGState(self.ctx) }
    }
    pub fn restore(&mut self) {
        if let Some(s) = self.stack.pop() {
            self.st = s;
            unsafe { CGContextRestoreGState(self.ctx) }
        }
    }
    pub fn set_fill_style(&mut self, s: impl IntoStyle) {
        self.st.fill = s.into_style();
    }
    pub fn set_stroke_style(&mut self, s: impl IntoStyle) {
        self.st.stroke = s.into_style();
    }
    pub fn fill_style(&self) -> Style {
        self.st.fill.clone()
    }
    pub fn set_line_width(&mut self, w: impl Num) {
        let w = w.f();
        if w > 0.0 && w.is_finite() {
            self.st.line_width = w;
        }
    }
    pub fn line_width(&self) -> f64 {
        self.st.line_width
    }
    pub fn set_line_cap(&mut self, c: &str) {
        self.st.line_cap = match c {
            "round" => kCGLineCapRound,
            "square" => kCGLineCapSquare,
            _ => kCGLineCapButt,
        };
    }
    pub fn set_line_join(&mut self, j: &str) {
        self.st.line_join = match j {
            "round" => kCGLineJoinRound,
            "bevel" => kCGLineJoinBevel,
            _ => kCGLineJoinMiter,
        };
    }
    pub fn set_miter_limit(&mut self, m: impl Num) {
        self.st.miter = m.f();
    }
    pub fn set_line_dash(&mut self, d: &[f64]) {
        let mut v = d.to_vec();
        if v.len() % 2 == 1 {
            let c = v.clone();
            v.extend(c);
        }
        self.st.dash = v;
    }
    pub fn set_line_dash_offset(&mut self, o: impl Num) {
        self.st.dash_offset = o.f();
    }
    pub fn set_global_alpha(&mut self, a: impl Num) {
        let a = a.f();
        if (0.0..=1.0).contains(&a) {
            self.st.alpha = a;
        }
    }
    pub fn global_alpha(&self) -> f64 {
        self.st.alpha
    }
    pub fn set_global_composite_operation(&mut self, op: &str) {
        self.st.composite = match op {
            "source-over" => kCGBlendModeNormal,
            "destination-out" => kCGBlendModeDestinationOut,
            "destination-in" => kCGBlendModeDestinationIn,
            "destination-over" => kCGBlendModeDestinationOver,
            "destination-atop" => kCGBlendModeDestinationAtop,
            "source-in" => kCGBlendModeSourceIn,
            "source-out" => kCGBlendModeSourceOut,
            "source-atop" => kCGBlendModeSourceAtop,
            "lighter" => kCGBlendModePlusLighter,
            "copy" => kCGBlendModeCopy,
            "xor" => kCGBlendModeXOR,
            "multiply" => kCGBlendModeMultiply,
            "screen" => kCGBlendModeScreen,
            "overlay" => kCGBlendModeOverlay,
            "darken" => kCGBlendModeDarken,
            "lighten" => kCGBlendModeLighten,
            "color-dodge" => kCGBlendModeColorDodge,
            "color-burn" => kCGBlendModeColorBurn,
            "hard-light" => kCGBlendModeHardLight,
            "soft-light" => kCGBlendModeSoftLight,
            "difference" => kCGBlendModeDifference,
            "exclusion" => kCGBlendModeExclusion,
            _ => self.st.composite,
        };
    }
    pub fn set_font(&mut self, f: &str) {
        self.st.font = parse_css_font(f);
        self.st.font_str = f.to_string();
    }
    pub fn font(&self) -> String {
        self.st.font_str.clone()
    }
    pub fn set_text_align(&mut self, a: &str) {
        self.st.align = a.to_string();
    }
    pub fn set_text_baseline(&mut self, b: &str) {
        self.st.baseline = b.to_string();
    }
    pub fn set_shadow_blur(&mut self, b: impl Num) {
        self.st.shadow_blur = b.f().max(0.0);
    }
    pub fn set_shadow_color(&mut self, c: &str) {
        self.st.shadow_color = parse_css_color(c).unwrap_or([0.0; 4]);
    }
    pub fn set_shadow_offset_x(&mut self, v: impl Num) {
        self.st.shadow_ox = v.f();
    }
    pub fn set_shadow_offset_y(&mut self, v: impl Num) {
        self.st.shadow_oy = v.f();
    }

    // ------------------------------------------------------------ transforms
    pub fn translate(&mut self, x: impl Num, y: impl Num) {
        let t = CGAffineTransform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: x.f(), ty: y.f() };
        self.st.ctm = t.then(&self.st.ctm);
    }
    pub fn rotate(&mut self, a: impl Num) {
        let a = a.f();
        let (s, c) = a.sin_cos();
        let t = CGAffineTransform { a: c, b: s, c: -s, d: c, tx: 0.0, ty: 0.0 };
        self.st.ctm = t.then(&self.st.ctm);
    }
    pub fn scale(&mut self, x: impl Num, y: impl Num) {
        let t = CGAffineTransform { a: x.f(), b: 0.0, c: 0.0, d: y.f(), tx: 0.0, ty: 0.0 };
        self.st.ctm = t.then(&self.st.ctm);
    }
    pub fn transform(&mut self, a: impl Num, b: impl Num, c: impl Num, d: impl Num, e: impl Num, f: impl Num) {
        let t = CGAffineTransform { a: a.f(), b: b.f(), c: c.f(), d: d.f(), tx: e.f(), ty: f.f() };
        self.st.ctm = t.then(&self.st.ctm);
    }
    pub fn set_transform(&mut self, a: impl Num, b: impl Num, c: impl Num, d: impl Num, e: impl Num, f: impl Num) {
        self.st.ctm = CGAffineTransform { a: a.f(), b: b.f(), c: c.f(), d: d.f(), tx: e.f(), ty: f.f() };
    }
    pub fn reset_transform(&mut self) {
        self.st.ctm = CGAffineTransform::IDENTITY;
    }
    fn dev(&self, x: f64, y: f64) -> (f64, f64) {
        self.st.ctm.apply(x, y)
    }

    // ------------------------------------------------------------ paths
    pub fn begin_path(&mut self) {
        unsafe {
            CGPathRelease(self.path);
            self.path = CGPathCreateMutable();
        }
        self.cur = None;
        self.start = None;
    }
    pub fn move_to(&mut self, x: impl Num, y: impl Num) {
        let (dx, dy) = self.dev(x.f(), y.f());
        if !(dx.is_finite() && dy.is_finite()) {
            return;
        }
        unsafe { CGPathMoveToPoint(self.path, std::ptr::null(), dx, dy) };
        self.cur = Some((dx, dy));
        self.start = Some((dx, dy));
    }
    fn ensure_sub(&mut self, dx: f64, dy: f64) {
        if self.cur.is_none() {
            unsafe { CGPathMoveToPoint(self.path, std::ptr::null(), dx, dy) };
            self.cur = Some((dx, dy));
            self.start = Some((dx, dy));
        }
    }
    pub fn line_to(&mut self, x: impl Num, y: impl Num) {
        let (dx, dy) = self.dev(x.f(), y.f());
        if !(dx.is_finite() && dy.is_finite()) {
            return;
        }
        if self.cur.is_none() {
            self.ensure_sub(dx, dy);
            return;
        }
        unsafe { CGPathAddLineToPoint(self.path, std::ptr::null(), dx, dy) };
        self.cur = Some((dx, dy));
    }
    pub fn quadratic_curve_to(&mut self, cx: impl Num, cy: impl Num, x: impl Num, y: impl Num) {
        let (dcx, dcy) = self.dev(cx.f(), cy.f());
        let (dx, dy) = self.dev(x.f(), y.f());
        if !(dx.is_finite() && dy.is_finite() && dcx.is_finite() && dcy.is_finite()) {
            return;
        }
        self.ensure_sub(dcx, dcy);
        unsafe { CGPathAddQuadCurveToPoint(self.path, std::ptr::null(), dcx, dcy, dx, dy) };
        self.cur = Some((dx, dy));
    }
    pub fn bezier_curve_to(&mut self, c1x: impl Num, c1y: impl Num, c2x: impl Num, c2y: impl Num, x: impl Num, y: impl Num) {
        let (a, b) = self.dev(c1x.f(), c1y.f());
        let (c, d) = self.dev(c2x.f(), c2y.f());
        let (e, f) = self.dev(x.f(), y.f());
        if ![a, b, c, d, e, f].iter().all(|v| v.is_finite()) {
            return;
        }
        self.ensure_sub(a, b);
        unsafe { CGPathAddCurveToPoint(self.path, std::ptr::null(), a, b, c, d, e, f) };
        self.cur = Some((e, f));
    }
    pub fn close_path(&mut self) {
        if self.cur.is_some() {
            unsafe { CGPathCloseSubpath(self.path) };
            self.cur = self.start;
        }
    }
    pub fn rect(&mut self, x: impl Num, y: impl Num, w: impl Num, h: impl Num) {
        let (x, y, w, h) = (x.f(), y.f(), w.f(), h.f());
        self.move_to(x, y);
        self.line_to(x + w, y);
        self.line_to(x + w, y + h);
        self.line_to(x, y + h);
        self.close_path();
    }
    /// (clockwise on screen)
    pub fn arc(&mut self, x: impl Num, y: impl Num, r: impl Num, a0: impl Num, a1: impl Num) {
        self.ellipse_dir(x, y, r, r, 0.0, a0, a1, false);
    }
    /// `arc(x, y, r, start, end, anticlockwise)`
    pub fn arc_dir(&mut self, x: impl Num, y: impl Num, r: impl Num, a0: impl Num, a1: impl Num, ccw: bool) {
        let r = r.f();
        self.ellipse_dir(x, y, r, r, 0.0, a0, a1, ccw);
    }
    pub fn ellipse(&mut self, x: impl Num, y: impl Num, rx: impl Num, ry: impl Num, rot: impl Num, a0: impl Num, a1: impl Num) {
        self.ellipse_dir(x, y, rx, ry, rot, a0, a1, false);
    }
    /// `ellipse(x, y, rx, ry, rotation, start, end, anticlockwise)`
    pub fn ellipse_dir(
        &mut self,
        x: impl Num,
        y: impl Num,
        rx: impl Num,
        ry: impl Num,
        rot: impl Num,
        a0: impl Num,
        a1: impl Num,
        ccw: bool,
    ) {
        let (x, y, rx, ry, rot, a0, a1) = (x.f(), y.f(), rx.f(), ry.f(), rot.f(), a0.f(), a1.f());
        if rx < 0.0 || ry < 0.0 || ![x, y, rx, ry, rot, a0, a1].iter().all(|v| v.is_finite()) {
            return;
        }
        let tau = 2.0 * PI;
        let sweep = if !ccw {
            let d = a1 - a0;
            if d >= tau {
                tau
            } else {
                ((d % tau) + tau) % tau
            }
        } else {
            let d = a0 - a1;
            if d >= tau {
                -tau
            } else {
                -(((d % tau) + tau) % tau)
            }
        };
        let (cr, sr) = (rot.cos(), rot.sin());
        let pt = |t: f64| {
            let (c, s) = (t.cos(), t.sin());
            let ex = rx * c;
            let ey = ry * s;
            (x + ex * cr - ey * sr, y + ex * sr + ey * cr)
        };
        let (sx, sy) = pt(a0);
        let (dsx, dsy) = self.dev(sx, sy);
        if self.cur.is_none() {
            self.ensure_sub(dsx, dsy);
        } else {
            unsafe { CGPathAddLineToPoint(self.path, std::ptr::null(), dsx, dsy) };
            self.cur = Some((dsx, dsy));
        }
        if sweep == 0.0 {
            return;
        }
        let n = ((sweep.abs() / (PI / 2.0)).ceil() as usize).max(1);
        let step = sweep / n as f64;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let mut t = a0;
        for _ in 0..n {
            let t2 = t + step;
            let (c1, s1) = (t.cos(), t.sin());
            let (c2, s2) = (t2.cos(), t2.sin());
            // control points on the unit circle, then scaled/rotated
            let p = |ux: f64, uy: f64| {
                let ex = rx * ux;
                let ey = ry * uy;
                (x + ex * cr - ey * sr, y + ex * sr + ey * cr)
            };
            let (q1x, q1y) = p(c1 - k * s1, s1 + k * c1);
            let (q2x, q2y) = p(c2 + k * s2, s2 - k * c2);
            let (ex, ey) = p(c2, s2);
            let (a, b) = self.dev(q1x, q1y);
            let (c, d) = self.dev(q2x, q2y);
            let (e, f) = self.dev(ex, ey);
            unsafe { CGPathAddCurveToPoint(self.path, std::ptr::null(), a, b, c, d, e, f) };
            self.cur = Some((e, f));
            t = t2;
        }
    }
    pub fn arc_to(&mut self, x1: impl Num, y1: impl Num, x2: impl Num, y2: impl Num, r: impl Num) {
        // tangent arc, computed in user space
        let (x1, y1, x2, y2, r) = (x1.f(), y1.f(), x2.f(), y2.f(), r.f());
        let inv = self.st.ctm.invert();
        let (x0, y0) = match self.cur {
            Some((dx, dy)) => inv.apply(dx, dy),
            None => {
                self.move_to(x1, y1);
                return;
            }
        };
        let (v1x, v1y) = (x0 - x1, y0 - y1);
        let (v2x, v2y) = (x2 - x1, y2 - y1);
        let l1 = (v1x * v1x + v1y * v1y).sqrt();
        let l2 = (v2x * v2x + v2y * v2y).sqrt();
        if l1 == 0.0 || l2 == 0.0 || r == 0.0 {
            self.line_to(x1, y1);
            return;
        }
        let cos = (v1x * v2x + v1y * v2y) / (l1 * l2);
        let ang = cos.clamp(-1.0, 1.0).acos();
        if ang.abs() < 1e-9 || (PI - ang).abs() < 1e-9 {
            self.line_to(x1, y1);
            return;
        }
        let d = r / (ang / 2.0).tan();
        let (t1x, t1y) = (x1 + v1x / l1 * d, y1 + v1y / l1 * d);
        let (t2x, t2y) = (x1 + v2x / l2 * d, y1 + v2y / l2 * d);
        let cross = v1x * v2y - v1y * v2x;
        let (nx, ny) = if cross > 0.0 { (v1y / l1, -v1x / l1) } else { (-v1y / l1, v1x / l1) };
        let (cx, cy) = (t1x + nx * r, t1y + ny * r);
        let a0 = (t1y - cy).atan2(t1x - cx);
        let a1 = (t2y - cy).atan2(t2x - cx);
        self.line_to(t1x, t1y);
        self.ellipse_dir(cx, cy, r, r, 0.0, a0, a1, cross > 0.0);
    }

    /// Draw what follows as one transparency layer, cast its shadow (the
    /// current one) once when it ends (`end_layer`): strokes that touch
    /// share one shadow, and the blur is made once, not once a stroke
    pub fn begin_layer(&mut self) {
        self.apply_shadow();
        unsafe { CGContextBeginTransparencyLayer(self.ctx, std::ptr::null()) };
        self.layers += 1;
    }
    /// `begin_layer`, the layer only as large as the box (`x, y, w, h` in
    /// the canvas's space as transformed now): a layer is a bitmap, and one
    /// the size of the canvas costs more than the shadows it saves
    pub fn begin_layer_in(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.apply_shadow();
        let corners = [self.dev(x, y), self.dev(x + w, y), self.dev(x, y + h), self.dev(x + w, y + h)];
        let (x0, x1) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), c| (a.min(c.0), b.max(c.0)));
        let (y0, y1) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), c| (a.min(c.1), b.max(c.1)));
        let r = CGRect { origin: CGPoint { x: x0, y: y0 }, size: CGSize { width: x1 - x0, height: y1 - y0 } };
        // (paths are made in device space, the context at its base
        // transform, so the box is given in device space too)
        unsafe { CGContextBeginTransparencyLayerWithRect(self.ctx, r, std::ptr::null()) };
        self.layers += 1;
    }
    pub fn end_layer(&mut self) {
        if self.layers == 0 {
            return;
        }
        self.layers -= 1;
        unsafe { CGContextEndTransparencyLayer(self.ctx) };
        self.version += 1;
    }

    fn apply_shadow(&self) {
        unsafe {
            if self.layers > 0 {
                CGContextSetShadowWithColor(self.ctx, CGSize::default(), 0.0, std::ptr::null_mut());
            } else if self.st.shadow_color[3] > 0.0 && (self.st.shadow_blur > 0.0 || self.st.shadow_ox != 0.0 || self.st.shadow_oy != 0.0) {
                let c = CGColorCreate(srgb(), self.st.shadow_color.as_ptr());
                // device space is y-up: canvas offset y flips
                CGContextSetShadowWithColor(
                    self.ctx,
                    CGSize { width: self.st.shadow_ox, height: -self.st.shadow_oy },
                    self.st.shadow_blur,
                    c,
                );
                CGColorRelease(c);
            } else {
                CGContextSetShadowWithColor(self.ctx, CGSize::default(), 0.0, std::ptr::null_mut());
            }
        }
    }
    fn begin_draw(&self) {
        unsafe {
            CGContextSaveGState(self.ctx);
            CGContextSetAlpha(self.ctx, self.st.alpha);
            CGContextSetBlendMode(self.ctx, self.st.composite);
        }
        self.apply_shadow();
    }
    fn end_draw(&mut self) {
        unsafe { CGContextRestoreGState(self.ctx) };
        self.version += 1;
    }

    fn fill_path_with(&mut self, path: CGPathRef, evenodd: bool) {
        self.begin_draw();
        unsafe {
            match &self.st.fill {
                Style::Color(c) => {
                    CGContextSetRGBFillColor(self.ctx, c[0], c[1], c[2], c[3]);
                    CGContextBeginPath(self.ctx);
                    CGContextAddPath(self.ctx, path);
                    if evenodd {
                        CGContextEOFillPath(self.ctx)
                    } else {
                        CGContextFillPath(self.ctx)
                    }
                }
                Style::Gradient(g) => {
                    CGContextBeginPath(self.ctx);
                    CGContextAddPath(self.ctx, path);
                    if evenodd {
                        CGContextEOClip(self.ctx)
                    } else {
                        CGContextClip(self.ctx)
                    }
                    self.draw_gradient(&g.clone());
                }
            }
        }
        self.end_draw();
    }
    unsafe fn draw_gradient(&self, g: &Gradient) {
        unsafe {
            let gd = g.0.borrow();
            if gd.stops.is_empty() {
                return;
            }
            let mut comps = vec![];
            let mut locs = vec![];
            for (o, c) in &gd.stops {
                comps.extend_from_slice(c);
                locs.push(*o);
            }
            let cg = CGGradientCreateWithColorComponents(srgb(), comps.as_ptr(), locs.as_ptr(), locs.len());
            CGContextConcatCTM(self.ctx, self.st.ctm);
            let opts = kCGGradientDrawsBeforeStartLocation | kCGGradientDrawsAfterEndLocation;
            match gd.kind {
                GradKind::Linear { x0, y0, x1, y1 } => {
                    CGContextDrawLinearGradient(self.ctx, cg, CGPoint { x: x0, y: y0 }, CGPoint { x: x1, y: y1 }, opts)
                }
                GradKind::Radial { x0, y0, r0, x1, y1, r1 } => {
                    // a soft blob (its end stop clear, its start circle inside its end
                    // circle) paints nothing outside the end circle under source-over:
                    // shade only that circle's box, not the whole clip (the same pixels)
                    let clear_end = gd.stops.last().is_some_and(|s| s.1[3] == 0.0);
                    let inside = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() + r0 <= r1;
                    if clear_end && inside && self.st.composite == kCGBlendModeNormal {
                        let r = r1 + 1.0;
                        CGContextClipToRect(
                            self.ctx,
                            CGRect { origin: CGPoint { x: x1 - r, y: y1 - r }, size: CGSize { width: 2.0 * r, height: 2.0 * r } },
                        );
                    }
                    CGContextDrawRadialGradient(self.ctx, cg, CGPoint { x: x0, y: y0 }, r0, CGPoint { x: x1, y: y1 }, r1, opts)
                }
            }
            CGGradientRelease(cg);
        }
    }
    pub fn fill(&mut self) {
        let p = self.path as CGPathRef;
        if unsafe { CGPathIsEmpty(p) } {
            return;
        }
        self.fill_path_with(p, false);
    }
    /// Fill the current path with the even-odd rule.
    pub fn fill_evenodd(&mut self) {
        let p = self.path as CGPathRef;
        if unsafe { CGPathIsEmpty(p) } {
            return;
        }
        self.fill_path_with(p, true);
    }
    fn stroke_path_with(&mut self, path: CGPathRef) {
        self.begin_draw();
        unsafe {
            CGContextConcatCTM(self.ctx, self.st.ctm);
            let inv = self.st.ctm.invert();
            let up = CGPathCreateCopyByTransformingPath(path, &inv);
            CGContextSetLineWidth(self.ctx, self.st.line_width);
            CGContextSetLineCap(self.ctx, self.st.line_cap);
            CGContextSetLineJoin(self.ctx, self.st.line_join);
            CGContextSetMiterLimit(self.ctx, self.st.miter);
            if self.st.dash.is_empty() {
                CGContextSetLineDash(self.ctx, 0.0, std::ptr::null(), 0);
            } else {
                CGContextSetLineDash(self.ctx, self.st.dash_offset, self.st.dash.as_ptr(), self.st.dash.len());
            }
            match &self.st.stroke {
                Style::Color(c) => {
                    CGContextSetRGBStrokeColor(self.ctx, c[0], c[1], c[2], c[3]);
                    CGContextBeginPath(self.ctx);
                    CGContextAddPath(self.ctx, up);
                    CGContextStrokePath(self.ctx);
                }
                Style::Gradient(g) => {
                    // stroke with a gradient: clip to the stroke outline
                    CGContextBeginPath(self.ctx);
                    CGContextAddPath(self.ctx, up);
                    CGContextReplacePathWithStrokedPath(self.ctx);
                    CGContextClip(self.ctx);
                    // undo the CTM concat: draw_gradient concatenates it again
                    CGContextConcatCTM(self.ctx, inv);
                    self.draw_gradient(&g.clone());
                }
            }
            CGPathRelease(up);
        }
        self.end_draw();
    }
    pub fn stroke(&mut self) {
        let p = self.path as CGPathRef;
        if unsafe { CGPathIsEmpty(p) } {
            return;
        }
        self.stroke_path_with(p);
    }
    pub fn clip(&mut self) {
        unsafe {
            CGContextBeginPath(self.ctx);
            CGContextAddPath(self.ctx, self.path);
            CGContextClip(self.ctx);
        }
    }
    fn rect_path(&self, x: f64, y: f64, w: f64, h: f64) -> CGMutablePathRef {
        unsafe {
            let p = CGPathCreateMutable();
            let pts = [self.dev(x, y), self.dev(x + w, y), self.dev(x + w, y + h), self.dev(x, y + h)];
            CGPathMoveToPoint(p, std::ptr::null(), pts[0].0, pts[0].1);
            for q in &pts[1..] {
                CGPathAddLineToPoint(p, std::ptr::null(), q.0, q.1);
            }
            CGPathCloseSubpath(p);
            p
        }
    }
    pub fn fill_rect(&mut self, x: impl Num, y: impl Num, w: impl Num, h: impl Num) {
        let (x, y, w, h) = (x.f(), y.f(), w.f(), h.f());
        if ![x, y, w, h].iter().all(|v| v.is_finite()) || w == 0.0 || h == 0.0 {
            return;
        }
        let p = self.rect_path(x, y, w, h);
        self.fill_path_with(p, false);
        unsafe { CGPathRelease(p) };
    }
    pub fn stroke_rect(&mut self, x: impl Num, y: impl Num, w: impl Num, h: impl Num) {
        let (x, y, w, h) = (x.f(), y.f(), w.f(), h.f());
        if ![x, y, w, h].iter().all(|v| v.is_finite()) {
            return;
        }
        let p = self.rect_path(x, y, w, h);
        self.stroke_path_with(p);
        unsafe { CGPathRelease(p) };
    }
    pub fn clear_rect(&mut self, x: impl Num, y: impl Num, w: impl Num, h: impl Num) {
        let (x, y, w, h) = (x.f(), y.f(), w.f(), h.f());
        let p = self.rect_path(x, y, w, h);
        unsafe {
            CGContextSaveGState(self.ctx);
            CGContextSetShadowWithColor(self.ctx, CGSize::default(), 0.0, std::ptr::null_mut());
            CGContextSetBlendMode(self.ctx, kCGBlendModeClear);
            CGContextBeginPath(self.ctx);
            CGContextAddPath(self.ctx, p);
            CGContextFillPath(self.ctx);
            CGContextRestoreGState(self.ctx);
            CGPathRelease(p);
        }
        self.version += 1;
    }

    // ------------------------------------------------------------ gradients
    pub fn create_linear_gradient(&self, x0: impl Num, y0: impl Num, x1: impl Num, y1: impl Num) -> Gradient {
        Gradient(Rc::new(RefCell::new(GradientData {
            kind: GradKind::Linear { x0: x0.f(), y0: y0.f(), x1: x1.f(), y1: y1.f() },
            stops: vec![],
        })))
    }
    pub fn create_radial_gradient(&self, x0: impl Num, y0: impl Num, r0: impl Num, x1: impl Num, y1: impl Num, r1: impl Num) -> Gradient {
        Gradient(Rc::new(RefCell::new(GradientData {
            kind: GradKind::Radial { x0: x0.f(), y0: y0.f(), r0: r0.f().max(0.0), x1: x1.f(), y1: y1.f(), r1: r1.f().max(0.0) },
            stops: vec![],
        })))
    }

    // ------------------------------------------------------------ text
    unsafe fn make_line(&self, text: &str) -> (CTLineRef, f64, f64, f64) {
        unsafe {
            let font = ct_font(&self.st.font);
            let keys = [kCTFontAttributeName, kCTForegroundColorFromContextAttributeName];
            let vals = [font as CFTypeRef, kCFBooleanTrue];
            let dict = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr(),
                vals.as_ptr(),
                2,
                &kCFTypeDictionaryKeyCallBacks as *const _ as _,
                &kCFTypeDictionaryValueCallBacks as *const _ as _,
            );
            let s = cfstr(text);
            let astr = CFAttributedStringCreate(std::ptr::null(), s, dict);
            let line = CTLineCreateWithAttributedString(astr);
            CFRelease(astr);
            CFRelease(s);
            CFRelease(dict);
            let (mut a, mut d, mut l) = (0.0, 0.0, 0.0);
            let w = CTLineGetTypographicBounds(line, &mut a, &mut d, &mut l);
            (line, w, CTFontGetAscent(font), CTFontGetDescent(font))
        }
    }
    pub fn measure_text(&self, text: &str) -> TextMetrics {
        unsafe {
            let (line, w, a, d) = self.make_line(text);
            CFRelease(line as CFTypeRef);
            TextMetrics { width: w, actual_bounding_box_ascent: a, actual_bounding_box_descent: d }
        }
    }
    fn draw_text(&mut self, text: &str, x: f64, y: f64, max_w: Option<f64>, stroke: bool) {
        if text.is_empty() || !x.is_finite() || !y.is_finite() {
            return;
        }
        unsafe {
            let (line, w, ascent, descent) = self.make_line(text);
            let sx = match max_w {
                Some(m) if m > 0.0 && w > m => m / w,
                _ => 1.0,
            };
            let ww = w * sx;
            let ox = match self.st.align.as_str() {
                "center" => -ww / 2.0,
                "right" | "end" => -ww,
                _ => 0.0,
            };
            let oy = match self.st.baseline.as_str() {
                "top" | "hanging" => ascent,
                "middle" => (ascent - descent) / 2.0,
                "bottom" | "ideographic" => -descent,
                _ => 0.0,
            };
            self.begin_draw();
            CGContextConcatCTM(self.ctx, self.st.ctm);
            CGContextTranslateCTM(self.ctx, x + ox, y + oy);
            CGContextScaleCTM(self.ctx, sx, -1.0);
            CGContextSetTextMatrix(self.ctx, CGAffineTransform::IDENTITY);
            CGContextSetTextPosition(self.ctx, 0.0, 0.0);
            let col = |s: &Style| match s {
                Style::Color(c) => *c,
                Style::Gradient(g) => g.0.borrow().stops.first().map(|s| s.1).unwrap_or([0.0, 0.0, 0.0, 1.0]),
            };
            if stroke {
                let c = col(&self.st.stroke);
                CGContextSetRGBStrokeColor(self.ctx, c[0], c[1], c[2], c[3]);
                CGContextSetLineWidth(self.ctx, self.st.line_width);
                CGContextSetTextDrawingMode(self.ctx, kCGTextStroke);
            } else {
                let c = col(&self.st.fill);
                CGContextSetRGBFillColor(self.ctx, c[0], c[1], c[2], c[3]);
                CGContextSetTextDrawingMode(self.ctx, kCGTextFill);
            }
            CTLineDraw(line, self.ctx);
            CFRelease(line as CFTypeRef);
            self.end_draw();
        }
    }
    pub fn fill_text(&mut self, text: &str, x: impl Num, y: impl Num) {
        self.draw_text(text, x.f(), y.f(), None, false);
    }
    pub fn fill_text_max(&mut self, text: &str, x: impl Num, y: impl Num, max_w: impl Num) {
        self.draw_text(text, x.f(), y.f(), Some(max_w.f()), false);
    }
    pub fn stroke_text(&mut self, text: &str, x: impl Num, y: impl Num) {
        self.draw_text(text, x.f(), y.f(), None, true);
    }
    pub fn stroke_text_max(&mut self, text: &str, x: impl Num, y: impl Num, max_w: impl Num) {
        self.draw_text(text, x.f(), y.f(), Some(max_w.f()), true);
    }

    // ------------------------------------------------------------ pixels
    pub fn create_image_data(&self, w: impl Num, h: impl Num) -> ImageData {
        let (w, h) = (w.f().max(1.0) as u32, h.f().max(1.0) as u32);
        ImageData { width: w, height: h, data: vec![0; (w * h * 4) as usize] }
    }
    /// unpremultiplied RGBA, rows top-down
    pub fn get_image_data(&mut self, x: impl Num, y: impl Num, w: impl Num, h: impl Num) -> ImageData {
        unsafe { CGContextFlush(self.ctx) };
        let (x0, y0, w, h) = (x.f() as i64, y.f() as i64, w.f().max(1.0) as i64, h.f().max(1.0) as i64);
        let mut out = vec![0u8; (w * h * 4) as usize];
        for yy in 0..h {
            for xx in 0..w {
                let (sx, sy) = (x0 + xx, y0 + yy);
                if sx < 0 || sy < 0 || sx >= self.w as i64 || sy >= self.h as i64 {
                    continue;
                }
                let si = ((sy * self.w as i64 + sx) * 4) as usize;
                let di = ((yy * w + xx) * 4) as usize;
                let a = self.buf[si + 3];
                if a == 0 {
                    continue;
                }
                for k in 0..3 {
                    out[di + k] = ((self.buf[si + k] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
                }
                out[di + 3] = a;
            }
        }
        ImageData { width: w as u32, height: h as u32, data: out }
    }
    pub fn put_image_data(&mut self, img: &ImageData, x: impl Num, y: impl Num) {
        unsafe { CGContextFlush(self.ctx) };
        let (x0, y0) = (x.f() as i64, y.f() as i64);
        for yy in 0..img.height as i64 {
            for xx in 0..img.width as i64 {
                let (dx, dy) = (x0 + xx, y0 + yy);
                if dx < 0 || dy < 0 || dx >= self.w as i64 || dy >= self.h as i64 {
                    continue;
                }
                let si = ((yy * img.width as i64 + xx) * 4) as usize;
                let di = ((dy * self.w as i64 + dx) * 4) as usize;
                let a = img.data[si + 3] as u32;
                for k in 0..3 {
                    self.buf[di + k] = ((img.data[si + k] as u32 * a + 127) / 255) as u8;
                }
                self.buf[di + 3] = a as u8;
            }
        }
        self.version += 1;
    }
    /// A canvas holding the JPEG image `bytes` as decoded: an RGB image is
    /// drawn into a bitmap of its own colour space (no conversion), so the
    /// pixels are the file's values (read as sRGB); a grey or CMYK one is
    /// converted to sRGB. None if CoreGraphics cannot read it. The bytes are
    /// `'static` (embedded files): the data provider reads them in place.
    pub fn from_jpeg(bytes: &'static [u8]) -> Option<Canvas> {
        unsafe {
            let p = CGDataProviderCreateWithData(std::ptr::null_mut(), bytes.as_ptr() as *const c_void, bytes.len(), None);
            if p.is_null() {
                return None;
            }
            let img = CGImageCreateWithJPEGDataProvider(p, std::ptr::null(), false, 0);
            CGDataProviderRelease(p);
            if img.is_null() {
                return None;
            }
            let (w, h) = (CGImageGetWidth(img), CGImageGetHeight(img));
            let mut c = Canvas::new(w as u32, h as u32);
            let mut raw = vec![0u8; w * h * 4];
            let own = CGImageGetColorSpace(img);
            let cs = if !own.is_null() && CGColorSpaceGetNumberOfComponents(own) == 3 { own } else { srgb() };
            let ctx = CGBitmapContextCreate(
                raw.as_mut_ptr() as *mut c_void,
                w,
                h,
                8,
                w * 4,
                cs,
                kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
            );
            if ctx.is_null() {
                CGImageRelease(img);
                return None;
            }
            // a bitmap context is y-up with rows stored top-down: the image lands upright
            CGContextDrawImage(ctx, rect(0.0, 0.0, w as f64, h as f64), img);
            CGContextFlush(ctx);
            CGContextRelease(ctx);
            CGImageRelease(img);
            c.buf.copy_from_slice(&raw);
            c.version += 1;
            Some(c)
        }
    }
    pub fn draw_image(&mut self, src: &Canvas, dx: impl Num, dy: impl Num) {
        let (w, h) = (src.width(), src.height());
        self.draw_image_scaled(src, dx, dy, w, h);
    }
    /// `draw_image(canvas, dx, dy, dw, dh)`
    pub fn draw_image_scaled(&mut self, src: &Canvas, dx: impl Num, dy: impl Num, dw: impl Num, dh: impl Num) {
        let (dx, dy, dw, dh) = (dx.f(), dy.f(), dw.f(), dh.f());
        unsafe {
            CGContextFlush(src.ctx);
            let img = CGBitmapContextCreateImage(src.ctx);
            self.begin_draw();
            CGContextConcatCTM(self.ctx, self.st.ctm);
            // image space is y-up; flip into the destination rect
            CGContextTranslateCTM(self.ctx, dx, dy + dh);
            CGContextScaleCTM(self.ctx, 1.0, -1.0);
            CGContextDrawImage(self.ctx, rect(0.0, 0.0, dw, dh), img);
            CGImageRelease(img);
            self.end_draw();
        }
    }

    /// Unpremultiplied RGBA pixels, rows top-down (the texture upload format).
    pub fn to_image(&self) -> Image {
        Image::new_rgba(self.w, self.h, self.pixels())
    }
    /// The pixels as RGBA8 with straight alpha, rows top-down (what
    /// `to_image` holds)
    pub fn pixels(&self) -> Vec<u8> {
        unsafe { CGContextFlush(self.ctx) };
        let mut data = self.buf.clone();
        for px in data.chunks_exact_mut(4) {
            let a = px[3] as u32;
            if a == 0 || a == 255 {
                continue;
            }
            for k in 0..3 {
                px[k] = ((px[k] as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
        data
    }
    /// Premultiplied pixels (for compositing the HUD).
    pub fn premultiplied(&self) -> &[u8] {
        unsafe { CGContextFlush(self.ctx) };
        &self.buf
    }
    /// A texture of the canvas's current pixels.
    pub fn to_tex(&self) -> Tex {
        Tex::from_image(self.to_image())
    }
    /// Copy the current pixels into a texture made from this canvas and mark it for re-upload.
    pub fn update_tex(&self, t: &Tex) {
        let img = self.to_image();
        let rc = t.image();
        let mut im = rc.borrow_mut();
        im.width = img.width;
        im.height = img.height;
        im.data = img.data;
        im.version += 1;
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextReplacePathWithStrokedPath(c: CGContextRef);
}
