//! Numeric helpers with the exact semantics the game's arithmetic relies on:
//! round-half-up, NaN-propagating min / max, a compensated hypotenuse,
//! exact fixed-point formatting and rounding, and number parsing.
//!
//! Level generation is seeded and bit-reproducible, so these are written to
//! give the same bits on every run, not merely close results.
pub use engine::math::{clamp, lerp, round_half_up, sign_of, to_i32, to_u32};

/// The Euclidean norm of `v`, reproducible bit for bit: the magnitudes are
/// scaled by the largest one and Kahan-summed before the square root. Any
/// infinite component gives +∞ (even alongside a NaN); otherwise any NaN gives
/// NaN. (libm's `hypot` and the naive `sqrt(a² + b²)` differ from this in the
/// last bit about a third of the time.)
pub fn hypot(v: &[f64]) -> f64 {
    let mut max = 0.0f64;
    let mut nan = false;
    for &x in v {
        let a = x.abs();
        if a.is_nan() {
            nan = true;
        } else if a > max {
            max = a;
        }
    }
    if max == f64::INFINITY {
        return f64::INFINITY;
    }
    if nan {
        return f64::NAN;
    }
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut comp) = (0.0f64, 0.0f64);
    for &x in v {
        let n = x.abs() / max;
        let summand = n * n - comp;
        let pre = sum + summand;
        comp = (pre - sum) - summand;
        sum = pre;
    }
    sum.sqrt() * max
}
/// The hypotenuse of `(a, b)` ([`hypot`]).
#[inline]
pub fn hypot2(a: f64, b: f64) -> f64 {
    hypot(&[a, b])
}
/// The length of `(a, b, c)` ([`hypot`]).
#[inline]
pub fn hypot3(a: f64, b: f64, c: f64) -> f64 {
    hypot(&[a, b, c])
}

/// The smaller of `a` and `b`, NaN-propagating (NaN if either is NaN), with
/// -0 ordered below +0.
#[inline]
pub fn nan_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b || (a == b && a.is_sign_negative()) {
        a
    } else {
        b
    }
}
/// The larger of `a` and `b`, NaN-propagating (NaN if either is NaN), with
/// +0 ordered above -0.
#[inline]
pub fn nan_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b || (a == b && b.is_sign_negative()) {
        a
    } else {
        b
    }
}

/// Formats `v` with exactly `d` decimals, rounding the exact binary value
/// (not its shortest decimal) with ties away from zero. A negative value that
/// rounds to zero keeps its minus sign (`fmt_fixed(-0.00001, 4)` is
/// `"-0.0000"`), while -0 itself prints as `"0.0000"`. NaN prints as `"NaN"`,
/// and magnitudes of 1e21 or more fall back to [`fmt_num`].
pub fn fmt_fixed(v: f64, d: usize) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v.abs() >= 1e21 {
        return fmt_num(v);
    }
    if v < 0.0 {
        return format!("-{}", fmt_fixed(-v, d));
    }
    let v = v.abs(); // -0 prints as 0
    // an exact tie: v·10^d·2 is an odd integer and the product is exact
    let m = 10f64.powi(d as i32);
    let y = v * m * 2.0;
    if y.fract() == 0.0 && y < 9.0e15 && (y as u64) % 2 == 1 && v.mul_add(m * 2.0, -y) == 0.0 {
        let n = (y as u64).div_ceil(2);
        if d == 0 {
            return format!("{n}");
        }
        let p = 10u64.pow(d as u32);
        return format!("{}.{:0w$}", n / p, n % p, w = d);
    }
    // otherwise Rust's formatting is the correctly rounded exact decimal
    format!("{v:.d$}")
}

/// Shortest round-trip formatting of a number: integral values below 1e21
/// print without a fraction, non-finite values as `NaN` / `Infinity` /
/// `-Infinity`.
pub fn fmt_num(v: f64) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if v == v.trunc() && v.abs() < 1e21 {
        return format!("{}", v as i64);
    }
    let s = format!("{v}");
    s
}

/// Number of decimal digits after the point in the exact expansion of `x`.
fn exact_frac_digits(x: f64) -> usize {
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    let mut man = bits & ((1u64 << 52) - 1);
    let mut e = if exp == 0 {
        -1074
    } else {
        man |= 1u64 << 52;
        exp - 1075
    };
    if man == 0 {
        return 0;
    }
    e += man.trailing_zeros() as i32;
    if e >= 0 { 0 } else { (-e) as usize }
}

/// Rounds `x` to `d` decimals: the exact binary value is rounded with ties
/// away from zero (as in [`fmt_fixed`]), and that decimal is parsed back,
/// correctly rounded. Non-finite values and magnitudes of 1e21 or more are
/// returned unchanged.
pub fn round_decimals(x: f64, d: usize) -> f64 {
    if !x.is_finite() || x.abs() >= 1e21 {
        return x;
    }
    let neg = x < 0.0;
    let a = x.abs();
    // with this many digits the formatting is the exact expansion (no rounding)
    let p = exact_frac_digits(a).max(d + 1);
    let s = format!("{a:.p$}");
    let (ip, fp) = s.split_once('.').expect("fixed point");
    let mut digits: Vec<u8> = ip.bytes().chain(fp.bytes().take(d)).collect();
    if fp.as_bytes()[d] >= b'5' {
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, b'1');
                break;
            }
            i -= 1;
            if digits[i] == b'9' {
                digits[i] = b'0';
            } else {
                digits[i] += 1;
                break;
            }
        }
    }
    let n = digits.len();
    let int_s = std::str::from_utf8(&digits[..n - d]).unwrap();
    let frac_s = std::str::from_utf8(&digits[n - d..]).unwrap();
    let t = if d == 0 { int_s.to_string() } else { format!("{int_s}.{frac_s}") };
    let v: f64 = t.parse().unwrap();
    if neg { -v } else { v }
}

/// Parses a number from a string. Surrounding whitespace (and U+FEFF) is
/// trimmed and an empty string is 0. Accepted: `Infinity` / `+Infinity` /
/// `-Infinity`; unsigned `0x` / `0o` / `0b` integers (prefix in either case);
/// and decimals with an optional sign, digits before and/or after a `.`, and
/// an optional `e` / `E` exponent. Anything else is NaN.
pub fn parse_number(s: &str) -> f64 {
    let t = s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if t.is_empty() {
        return 0.0;
    }
    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    for (p, radix) in [("0x", 16u32), ("0o", 8), ("0b", 2)] {
        if t.len() >= 2 && t[..2].eq_ignore_ascii_case(p) {
            let digits = &t[2..];
            if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
                return f64::NAN;
            }
            let mut v = 0f64;
            for c in digits.chars() {
                v = v * radix as f64 + c.to_digit(radix).unwrap() as f64;
            }
            return v;
        }
    }
    let b = t.as_bytes();
    let mut i = 0;
    if b[i] == b'+' || b[i] == b'-' {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - ds;
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        frac_digits = i - fs;
    }
    if int_digits + frac_digits == 0 {
        return f64::NAN;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let es = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return f64::NAN;
        }
    }
    if i != b.len() {
        return f64::NAN;
    }
    t.parse::<f64>().unwrap_or(f64::NAN)
}

/// Left-pads `s` with `c` to `n` characters.
pub fn pad_start(s: &str, n: usize, c: char) -> String {
    let l = s.chars().count();
    if l >= n { s.to_string() } else { std::iter::repeat_n(c, n - l).collect::<String>() + s }
}
