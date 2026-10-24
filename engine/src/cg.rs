//! Minimal CoreGraphics / CoreText / CoreFoundation FFI.
#![allow(non_upper_case_globals, non_snake_case, dead_code, clippy::missing_safety_doc)]

use std::ffi::c_void;

pub type CFTypeRef = *const c_void;
pub type CGContextRef = *mut c_void;
pub type CGColorSpaceRef = *mut c_void;
pub type CGPathRef = *const c_void;
pub type CGMutablePathRef = *mut c_void;
pub type CGGradientRef = *mut c_void;
pub type CGImageRef = *mut c_void;
pub type CGColorRef = *mut c_void;
pub type CTFontRef = *mut c_void;
pub type CTLineRef = *mut c_void;

#[repr(C)] #[derive(Clone, Copy, Debug, Default)] pub struct CGPoint { pub x: f64, pub y: f64 }
#[repr(C)] #[derive(Clone, Copy, Debug, Default)] pub struct CGSize { pub width: f64, pub height: f64 }
#[repr(C)] #[derive(Clone, Copy, Debug, Default)] pub struct CGRect { pub origin: CGPoint, pub size: CGSize }
#[repr(C)] #[derive(Clone, Copy, Debug)] pub struct CGAffineTransform { pub a: f64, pub b: f64, pub c: f64, pub d: f64, pub tx: f64, pub ty: f64 }
impl CGAffineTransform {
    pub const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: 0.0 };
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) { (self.a * x + self.c * y + self.tx, self.b * x + self.d * y + self.ty) }
    /// self then t (point -> self -> t)
    pub fn then(&self, t: &CGAffineTransform) -> CGAffineTransform {
        CGAffineTransform {
            a: self.a * t.a + self.b * t.c, b: self.a * t.b + self.b * t.d,
            c: self.c * t.a + self.d * t.c, d: self.c * t.b + self.d * t.d,
            tx: self.tx * t.a + self.ty * t.c + t.tx, ty: self.tx * t.b + self.ty * t.d + t.ty,
        }
    }
    pub fn invert(&self) -> CGAffineTransform {
        let det = self.a * self.d - self.b * self.c;
        if det == 0.0 { return *self; }
        let id = 1.0 / det;
        CGAffineTransform { a: self.d * id, b: -self.b * id, c: -self.c * id, d: self.a * id, tx: (self.c * self.ty - self.d * self.tx) * id, ty: (self.b * self.tx - self.a * self.ty) * id }
    }
}
pub fn rect(x: f64, y: f64, w: f64, h: f64) -> CGRect { CGRect { origin: CGPoint { x, y }, size: CGSize { width: w, height: h } } }

pub const kCGImageAlphaPremultipliedLast: u32 = 1;
pub const kCGBitmapByteOrder32Big: u32 = 4 << 12;
pub const kCGGradientDrawsBeforeStartLocation: u32 = 1;
pub const kCGGradientDrawsAfterEndLocation: u32 = 2;
pub const kCGLineCapButt: i32 = 0; pub const kCGLineCapRound: i32 = 1; pub const kCGLineCapSquare: i32 = 2;
pub const kCGLineJoinMiter: i32 = 0; pub const kCGLineJoinRound: i32 = 1; pub const kCGLineJoinBevel: i32 = 2;
pub const kCGTextFill: i32 = 0; pub const kCGTextStroke: i32 = 1;
pub const kCGBlendModeNormal: i32 = 0; pub const kCGBlendModeMultiply: i32 = 1; pub const kCGBlendModeScreen: i32 = 2;
pub const kCGBlendModeOverlay: i32 = 3; pub const kCGBlendModeDarken: i32 = 4; pub const kCGBlendModeLighten: i32 = 5;
pub const kCGBlendModeColorDodge: i32 = 6; pub const kCGBlendModeColorBurn: i32 = 7; pub const kCGBlendModeSoftLight: i32 = 8;
pub const kCGBlendModeHardLight: i32 = 9; pub const kCGBlendModeDifference: i32 = 10; pub const kCGBlendModeExclusion: i32 = 11;
pub const kCGBlendModeClear: i32 = 16; pub const kCGBlendModeCopy: i32 = 17; pub const kCGBlendModeSourceIn: i32 = 18;
pub const kCGBlendModeSourceOut: i32 = 19; pub const kCGBlendModeSourceAtop: i32 = 20; pub const kCGBlendModeDestinationOver: i32 = 21;
pub const kCGBlendModeDestinationIn: i32 = 22; pub const kCGBlendModeDestinationOut: i32 = 23; pub const kCGBlendModeDestinationAtop: i32 = 24;
pub const kCGBlendModeXOR: i32 = 25; pub const kCGBlendModePlusDarker: i32 = 26; pub const kCGBlendModePlusLighter: i32 = 27;
pub const kCGInterpolationHigh: i32 = 3;
pub const kCTFontItalicTrait: u32 = 1 << 0;
pub const kCTFontBoldTrait: u32 = 1 << 1;
pub const kCFStringEncodingUTF8: u32 = 0x08000100;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGColorSpaceCreateDeviceRGB() -> CGColorSpaceRef;
    pub fn CGColorSpaceCreateWithName(name: CFTypeRef) -> CGColorSpaceRef;
    pub static kCGColorSpaceSRGB: CFTypeRef;
    pub fn CGColorSpaceRelease(cs: CGColorSpaceRef);
    pub fn CGBitmapContextCreate(data: *mut c_void, w: usize, h: usize, bpc: usize, bpr: usize, cs: CGColorSpaceRef, info: u32) -> CGContextRef;
    pub fn CGBitmapContextCreateImage(c: CGContextRef) -> CGImageRef;
    pub fn CGImageRelease(i: CGImageRef);
    pub fn CGContextRelease(c: CGContextRef);
    pub fn CGContextFlush(c: CGContextRef);
    pub fn CGContextSaveGState(c: CGContextRef);
    pub fn CGContextRestoreGState(c: CGContextRef);
    pub fn CGContextConcatCTM(c: CGContextRef, t: CGAffineTransform);
    pub fn CGContextGetCTM(c: CGContextRef) -> CGAffineTransform;
    pub fn CGContextTranslateCTM(c: CGContextRef, x: f64, y: f64);
    pub fn CGContextScaleCTM(c: CGContextRef, x: f64, y: f64);
    pub fn CGContextSetRGBFillColor(c: CGContextRef, r: f64, g: f64, b: f64, a: f64);
    pub fn CGContextSetRGBStrokeColor(c: CGContextRef, r: f64, g: f64, b: f64, a: f64);
    pub fn CGContextSetAlpha(c: CGContextRef, a: f64);
    pub fn CGContextSetBlendMode(c: CGContextRef, m: i32);
    pub fn CGContextSetLineWidth(c: CGContextRef, w: f64);
    pub fn CGContextSetLineCap(c: CGContextRef, cap: i32);
    pub fn CGContextSetLineJoin(c: CGContextRef, j: i32);
    pub fn CGContextSetMiterLimit(c: CGContextRef, l: f64);
    pub fn CGContextSetLineDash(c: CGContextRef, phase: f64, lengths: *const f64, count: usize);
    pub fn CGContextSetShouldAntialias(c: CGContextRef, b: bool);
    pub fn CGContextSetInterpolationQuality(c: CGContextRef, q: i32);
    pub fn CGContextBeginPath(c: CGContextRef);
    pub fn CGContextAddPath(c: CGContextRef, p: CGPathRef);
    pub fn CGContextFillPath(c: CGContextRef);
    pub fn CGContextEOFillPath(c: CGContextRef);
    pub fn CGContextStrokePath(c: CGContextRef);
    pub fn CGContextClip(c: CGContextRef);
    pub fn CGContextEOClip(c: CGContextRef);
    pub fn CGContextClearRect(c: CGContextRef, r: CGRect);
    pub fn CGContextFillRect(c: CGContextRef, r: CGRect);
    pub fn CGContextDrawImage(c: CGContextRef, r: CGRect, i: CGImageRef);
    pub fn CGContextSetShadowWithColor(c: CGContextRef, off: CGSize, blur: f64, color: CGColorRef);
    pub fn CGContextSetTextMatrix(c: CGContextRef, t: CGAffineTransform);
    pub fn CGContextSetTextPosition(c: CGContextRef, x: f64, y: f64);
    pub fn CGContextSetTextDrawingMode(c: CGContextRef, m: i32);
    pub fn CGColorCreate(cs: CGColorSpaceRef, comps: *const f64) -> CGColorRef;
    pub fn CGColorRelease(c: CGColorRef);
    pub fn CGPathCreateMutable() -> CGMutablePathRef;
    pub fn CGPathRelease(p: CGPathRef);
    pub fn CGPathMoveToPoint(p: CGMutablePathRef, m: *const CGAffineTransform, x: f64, y: f64);
    pub fn CGPathAddLineToPoint(p: CGMutablePathRef, m: *const CGAffineTransform, x: f64, y: f64);
    pub fn CGPathAddQuadCurveToPoint(p: CGMutablePathRef, m: *const CGAffineTransform, cx: f64, cy: f64, x: f64, y: f64);
    pub fn CGPathAddCurveToPoint(p: CGMutablePathRef, m: *const CGAffineTransform, c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64);
    pub fn CGPathCloseSubpath(p: CGMutablePathRef);
    pub fn CGPathIsEmpty(p: CGPathRef) -> bool;
    pub fn CGPathCreateCopyByTransformingPath(p: CGPathRef, m: *const CGAffineTransform) -> CGPathRef;
    pub fn CGGradientCreateWithColorComponents(cs: CGColorSpaceRef, comps: *const f64, locs: *const f64, count: usize) -> CGGradientRef;
    pub fn CGGradientRelease(g: CGGradientRef);
    pub fn CGContextDrawLinearGradient(c: CGContextRef, g: CGGradientRef, s: CGPoint, e: CGPoint, opts: u32);
    pub fn CGContextDrawRadialGradient(c: CGContextRef, g: CGGradientRef, sc: CGPoint, sr: f64, ec: CGPoint, er: f64, opts: u32);
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
    pub fn CTFontCreateWithName(name: CFTypeRef, size: f64, m: *const CGAffineTransform) -> CTFontRef;
    pub fn CTFontCreateCopyWithSymbolicTraits(f: CTFontRef, size: f64, m: *const CGAffineTransform, value: u32, mask: u32) -> CTFontRef;
    pub fn CTFontGetAscent(f: CTFontRef) -> f64;
    pub fn CTFontGetDescent(f: CTFontRef) -> f64;
    pub fn CTLineCreateWithAttributedString(s: CFTypeRef) -> CTLineRef;
    pub fn CTLineDraw(l: CTLineRef, c: CGContextRef);
    pub fn CTLineGetTypographicBounds(l: CTLineRef, ascent: *mut f64, descent: *mut f64, leading: *mut f64) -> f64;
    pub static kCTFontAttributeName: CFTypeRef;
    pub static kCTForegroundColorFromContextAttributeName: CFTypeRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub fn CFStringCreateWithBytes(a: CFTypeRef, bytes: *const u8, n: isize, enc: u32, bom: bool) -> CFTypeRef;
    pub fn CFDictionaryCreate(a: CFTypeRef, k: *const CFTypeRef, v: *const CFTypeRef, n: isize, kc: *const c_void, vc: *const c_void) -> CFTypeRef;
    pub fn CFAttributedStringCreate(a: CFTypeRef, s: CFTypeRef, attrs: CFTypeRef) -> CFTypeRef;
    pub fn CFRelease(o: CFTypeRef);
    pub static kCFTypeDictionaryKeyCallBacks: c_void;
    pub static kCFTypeDictionaryValueCallBacks: c_void;
    pub static kCFBooleanTrue: CFTypeRef;
}

pub fn cfstr(s: &str) -> CFTypeRef { unsafe { CFStringCreateWithBytes(std::ptr::null(), s.as_ptr(), s.len() as isize, kCFStringEncodingUTF8, false) } }
