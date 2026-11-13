//! The HUD's typefaces: three voices, each with a reason (`docs/design/UI.md`).
//!
//! * [`Face::Sans`]: the building's voice, Helvetica Neue (menus, prompts,
//!   cards). Institutional type makes the uncanny routine.
//! * [`Face::Hand`]: M.'s hand, Noteworthy Light (her notebook, her thread).
//! * [`Face::Typed`]: Courier New, for what is typed on paper (the torn
//!   notes) and the debug panel. Its metrics are the fixed ones the HUD was
//!   laid out with (0.6 em advances, rounded ascent and descent), so the
//!   frames set in it do not move.
//!
//! The proportional faces are measured with CoreText: each glyph's advance
//! with kerning and ligatures off (as they are drawn, `ui::draw::ct_glyphs`),
//! cached per face at a reference size, so a laid-out line and the line
//! CoreText draws are the same width. (Spaced text is drawn glyph by glyph
//! at its laid-out places: see `draws_runs`.)
use engine::cg::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// A voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Typed,
    Sans,
    /// Helvetica Neue Light (titles; `bold` is ignored)
    SansLight,
    Hand,
}

/// A face at a size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub face: Face,
}

/// The reference size the proportional advances are measured at
const REF_PX: f64 = 100.0;

thread_local! {
    static CT_FONTS: RefCell<HashMap<(Face, bool, bool, u64), usize>> = RefCell::new(HashMap::new());
    static ADVANCES: RefCell<HashMap<(Face, bool, bool, char), f64>> = RefCell::new(HashMap::new());
}

impl Font {
    /// Courier New (the typed voice)
    pub fn new(size: f64) -> Font {
        Font { size, bold: false, italic: false, face: Face::Typed }
    }
    pub fn sans(size: f64) -> Font {
        Font { face: Face::Sans, ..Font::new(size) }
    }
    pub fn sans_light(size: f64) -> Font {
        Font { face: Face::SansLight, ..Font::new(size) }
    }
    pub fn hand(size: f64) -> Font {
        Font { face: Face::Hand, ..Font::new(size) }
    }
    pub fn bold(mut self) -> Font {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Font {
        self.italic = true;
        self
    }
    /// The PostScript name CoreText is asked for
    pub fn ps_name(&self) -> &'static str {
        match (self.face, self.bold, self.italic) {
            (Face::Typed, false, false) => "CourierNewPSMT",
            (Face::Typed, true, false) => "CourierNewPS-BoldMT",
            (Face::Typed, false, true) => "CourierNewPS-ItalicMT",
            (Face::Typed, true, true) => "CourierNewPS-BoldItalicMT",
            (Face::Sans, false, false) => "HelveticaNeue",
            (Face::Sans, true, false) => "HelveticaNeue-Medium",
            (Face::Sans, false, true) => "HelveticaNeue-Italic",
            (Face::Sans, true, true) => "HelveticaNeue-MediumItalic",
            (Face::SansLight, _, false) => "HelveticaNeue-Light",
            (Face::SansLight, _, true) => "HelveticaNeue-LightItalic",
            (Face::Hand, false, _) => "Noteworthy-Light",
            (Face::Hand, true, _) => "Noteworthy-Bold",
        }
    }
    /// Ascent, rounded to whole px per size
    pub fn ascent(&self) -> f64 {
        match self.face {
            Face::Typed => (self.size * 0.83252).round(),
            _ => unsafe { CTFontGetAscent(ct_font(*self, self.size)) }.round(),
        }
    }
    pub fn descent(&self) -> f64 {
        match self.face {
            Face::Typed => (self.size * 0.30029).round(),
            _ => unsafe { CTFontGetDescent(ct_font(*self, self.size)) }.round(),
        }
    }
    /// `line-height: normal`
    pub fn normal_lh(&self) -> f64 {
        self.ascent() + self.descent()
    }
    /// A glyph's advance. Courier New is 0.6001 em; its geometric symbols
    /// come from a fallback face at 0.602 em (⛧ wider). The other faces are
    /// measured.
    pub fn advance(&self, ch: char) -> f64 {
        if self.face == Face::Typed {
            let em = match ch {
                '⛧' => 0.788,
                '◆' | '◇' | '★' | '✦' | '◀' | '▶' | '▮' | '▯' => 0.6020625,
                _ => 0.600109375,
            };
            return em * self.size;
        }
        let key = (self.face, self.bold, self.italic, ch);
        let em = ADVANCES.with(|a| a.borrow().get(&key).copied());
        let em = em.unwrap_or_else(|| {
            let w = line_width(*self, &ch.to_string(), REF_PX, 0.0) / REF_PX;
            ADVANCES.with(|a| a.borrow_mut().insert(key, w));
            w
        });
        em * self.size
    }
    /// Whether glyphs set with letter spacing `ls` are drawn as one CoreText
    /// line. Courier New sends its symbols to fallback faces, so they go one
    /// by one; a proportional face goes one by one when spaced, since
    /// CoreText puts pair kerning back on for any spacing but zero ("Ta"
    /// would close up by a pixel and the line no longer match its layout).
    pub fn draws_runs(&self, ch: char, ls: f64) -> bool {
        match self.face {
            Face::Typed => face_metrics(ch, self.size).is_none() && (ch as u32) < 0x2000,
            _ => ls == 0.0,
        }
    }
}

/// (ascent, descent) of the fallback faces Courier New lines use for the
/// symbols (a Menlo-like face for ◆★✦◀▶▮▯◇, another for ⛧)
pub fn face_metrics(ch: char, size: f64) -> Option<(f64, f64)> {
    match ch {
        '◆' | '◇' | '★' | '✦' | '◀' | '▶' | '▮' | '▯' => Some(((size * 0.928).round(), (size * 0.236).round())),
        '⛧' => Some(((size * 0.88).round(), (size * 0.37).round())),
        _ => None,
    }
}

/// The CoreText font for `font` at `px` device pixels (cached; never freed)
pub fn ct_font(font: Font, px: f64) -> CTFontRef {
    CT_FONTS.with(|c| {
        let key = (font.face, font.bold, font.italic, px.to_bits());
        if let Some(&f) = c.borrow().get(&key) {
            return f as CTFontRef;
        }
        let f = unsafe {
            let n = cfstr(font.ps_name());
            let f = CTFontCreateWithName(n, px, std::ptr::null());
            CFRelease(n);
            f
        };
        c.borrow_mut().insert(key, f as usize);
        f
    })
}

/// The attributes the HUD draws with: the font, the context's colour, the
/// letter spacing (`kern`, which also turns pair kerning off), and for the
/// proportional faces no optional ligatures. The caller releases it.
pub fn text_attrs(font: Font, px: f64, kern: f64) -> CFTypeRef {
    unsafe {
        let f = ct_font(font, px);
        let kv = CFNumberCreate(std::ptr::null(), 6 /* kCFNumberFloat64Type */, &kern as *const f64 as *const std::ffi::c_void);
        let zero: i32 = 0;
        let lv = CFNumberCreate(std::ptr::null(), 3 /* kCFNumberSInt32Type */, &zero as *const i32 as *const std::ffi::c_void);
        let mut keys = vec![kCTFontAttributeName, kCTForegroundColorFromContextAttributeName, kCTKernAttributeName];
        let mut vals = vec![f as CFTypeRef, kCFBooleanTrue, kv];
        if font.face != Face::Typed {
            keys.push(kCTLigatureAttributeName);
            vals.push(lv);
        }
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            vals.as_ptr(),
            keys.len() as isize,
            &kCFTypeDictionaryKeyCallBacks as *const _ as _,
            &kCFTypeDictionaryValueCallBacks as *const _ as _,
        );
        CFRelease(kv);
        CFRelease(lv);
        dict
    }
}

/// The width CoreText gives `text` set in `font` at `px` with letter
/// spacing `kern` (px after every glyph)
pub fn line_width(font: Font, text: &str, px: f64, kern: f64) -> f64 {
    unsafe {
        let s = cfstr(text);
        let dict = text_attrs(font, px, kern);
        let astr = CFAttributedStringCreate(std::ptr::null(), s, dict);
        let line = CTLineCreateWithAttributedString(astr);
        let w = CTLineGetTypographicBounds(line, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
        CFRelease(line as CFTypeRef);
        CFRelease(astr);
        CFRelease(dict);
        CFRelease(s);
        w
    }
}

/// The PostScript name of the font CoreText actually gave for `font` (it
/// substitutes silently when a face is missing)
pub fn resolved_name(font: Font) -> String {
    unsafe {
        let n = CTFontCopyPostScriptName(ct_font(font, 12.0));
        let s = cfstring_to_string(n);
        CFRelease(n);
        s
    }
}
