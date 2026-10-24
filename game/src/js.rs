//! Small JS-semantics helpers for ported code.
pub use engine::math::{clamp, lerp, js_round, js_sign, to_i32, to_u32};

/// `Number.prototype.toFixed(d)`
pub fn to_fixed(v: f64, d: usize) -> String { crate::props::js_to_fixed(v, d) }
/// `String(n)` for numbers (JS shortest round-trip formatting).
pub fn num_str(v: f64) -> String {
    if v.is_nan() { return "NaN".into(); }
    if v.is_infinite() { return if v > 0.0 { "Infinity".into() } else { "-Infinity".into() }; }
    if v == v.trunc() && v.abs() < 1e21 { return format!("{}", v as i64); }
    let s = format!("{}", v);
    s
}
/// `str.padStart(n, c)`
pub fn pad_start(s: &str, n: usize, c: char) -> String { let l = s.chars().count(); if l >= n { s.to_string() } else { std::iter::repeat(c).take(n - l).collect::<String>() + s } }
/// `Math.hypot(a, b, c)`
pub fn hypot3(a: f64, b: f64, c: f64) -> f64 { (a * a + b * b + c * c).sqrt() }
/// `Math.hypot(a, b)`
pub fn hypot2(a: f64, b: f64) -> f64 { a.hypot(b) }
