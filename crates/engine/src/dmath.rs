//! The game's own math functions, the same on every Mac, every macOS and
//! every compiler (`docs/plan/REPLAYS.md`, R1).
//!
//! `f64::sin` and its kin call the system's libm, whose results are not
//! correctly rounded (on this Mac's libm, `sin` and `cos` differ from the
//! correct result on about 4% of inputs, `acos` 15%, `tan` 38%) and may
//! change with macOS; `powi` is whatever LLVM makes of it. A level is a pure
//! function of its seed and a replay repeats a run only if every result
//! repeats, so what decides a level or a frame calls these instead:
//!
//! - the transcendental functions are CORE-MATH's (`core-math`), correctly
//!   rounded: the one right answer, which no implementation, version or
//!   system can change;
//! - `dhypot` is `√(x² + y²)` and `dpowi` products, from `+ × √`, which
//!   IEEE 754 makes exact everywhere (rustc never fuses or reorders them).
//!
//! Clippy forbids the std methods (`clippy.toml`, `disallowed-methods`);
//! only the sound's synthesis keeps them (its values reach the speakers and
//! nothing else).

/// Deterministic math on `f64` and `f32`: `x.dsin()` where std has `x.sin()`.
pub trait DMath: Sized + Copy {
    fn dsin(self) -> Self;
    fn dcos(self) -> Self;
    fn dsin_cos(self) -> (Self, Self);
    fn dtan(self) -> Self;
    fn dasin(self) -> Self;
    fn dacos(self) -> Self;
    fn datan(self) -> Self;
    fn datan2(self, x: Self) -> Self;
    fn dsinh(self) -> Self;
    fn dcosh(self) -> Self;
    fn dtanh(self) -> Self;
    fn dexp(self) -> Self;
    fn dexp2(self) -> Self;
    fn dexp_m1(self) -> Self;
    fn dln(self) -> Self;
    fn dln_1p(self) -> Self;
    fn dlog2(self) -> Self;
    fn dlog10(self) -> Self;
    /// The logarithm in `base`: `ln(self) / ln(base)`, as std computes it.
    fn dlog(self, base: Self) -> Self;
    fn dpowf(self, n: Self) -> Self;
    fn dcbrt(self) -> Self;
    /// `√(x² + y²)`, without std's guard against overflow (no length in this
    /// game comes near `1e154`).
    fn dhypot(self, y: Self) -> Self;
    /// `self` to an integer power, by products in a fixed order: squares
    /// for the powers of two, then the odd factors (`x³ = x²·x`).
    fn dpowi(self, n: i32) -> Self;
}

macro_rules! dmath_impl {
    ($t:ty, $sin:path, $cos:path, $sincos:path, $tan:path, $asin:path, $acos:path, $atan:path, $atan2:path,
     $sinh:path, $cosh:path, $tanh:path, $exp:path, $exp2:path, $expm1:path, $ln:path, $ln1p:path, $log2:path,
     $log10:path, $pow:path, $cbrt:path) => {
        impl DMath for $t {
            #[inline]
            fn dsin(self) -> Self {
                $sin(self)
            }
            #[inline]
            fn dcos(self) -> Self {
                $cos(self)
            }
            #[inline]
            fn dsin_cos(self) -> (Self, Self) {
                $sincos(self)
            }
            #[inline]
            fn dtan(self) -> Self {
                $tan(self)
            }
            #[inline]
            fn dasin(self) -> Self {
                $asin(self)
            }
            #[inline]
            fn dacos(self) -> Self {
                $acos(self)
            }
            #[inline]
            fn datan(self) -> Self {
                $atan(self)
            }
            #[inline]
            fn datan2(self, x: Self) -> Self {
                $atan2(self, x)
            }
            #[inline]
            fn dsinh(self) -> Self {
                $sinh(self)
            }
            #[inline]
            fn dcosh(self) -> Self {
                $cosh(self)
            }
            #[inline]
            fn dtanh(self) -> Self {
                $tanh(self)
            }
            #[inline]
            fn dexp(self) -> Self {
                $exp(self)
            }
            #[inline]
            fn dexp2(self) -> Self {
                $exp2(self)
            }
            #[inline]
            fn dexp_m1(self) -> Self {
                $expm1(self)
            }
            #[inline]
            fn dln(self) -> Self {
                $ln(self)
            }
            #[inline]
            fn dln_1p(self) -> Self {
                $ln1p(self)
            }
            #[inline]
            fn dlog2(self) -> Self {
                $log2(self)
            }
            #[inline]
            fn dlog10(self) -> Self {
                $log10(self)
            }
            #[inline]
            fn dlog(self, base: Self) -> Self {
                $ln(self) / $ln(base)
            }
            #[inline]
            fn dpowf(self, n: Self) -> Self {
                $pow(self, n)
            }
            #[inline]
            fn dcbrt(self) -> Self {
                $cbrt(self)
            }
            #[inline]
            fn dhypot(self, y: Self) -> Self {
                (self * self + y * y).sqrt()
            }
            #[inline]
            fn dpowi(self, n: i32) -> Self {
                let mut e = n.unsigned_abs();
                let (mut base, mut acc) = (self, None::<Self>);
                while e > 0 {
                    if e & 1 == 1 {
                        acc = Some(acc.map_or(base, |a| a * base));
                    }
                    e >>= 1;
                    if e > 0 {
                        base = base * base;
                    }
                }
                let r = acc.unwrap_or(1.0);
                if n < 0 { 1.0 / r } else { r }
            }
        }
    };
}

dmath_impl!(
    f64,
    core_math::sin,
    core_math::cos,
    core_math::sincos,
    core_math::tan,
    core_math::asin,
    core_math::acos,
    core_math::atan,
    core_math::atan2,
    core_math::sinh,
    core_math::cosh,
    core_math::tanh,
    core_math::exp,
    core_math::exp2,
    core_math::expm1,
    core_math::log,
    core_math::log1p,
    core_math::log2,
    core_math::log10,
    core_math::pow,
    core_math::cbrt
);
dmath_impl!(
    f32,
    core_math::sinf,
    core_math::cosf,
    core_math::sincosf,
    core_math::tanf,
    core_math::asinf,
    core_math::acosf,
    core_math::atanf,
    core_math::atan2f,
    core_math::sinhf,
    core_math::coshf,
    core_math::tanhf,
    core_math::expf,
    core_math::exp2f,
    core_math::expm1f,
    core_math::logf,
    core_math::log1pf,
    core_math::log2f,
    core_math::log10f,
    core_math::powf,
    core_math::cbrtf
);

#[cfg(test)]
mod tests {
    use super::DMath;

    #[test]
    fn powers_are_products_in_a_fixed_order() {
        for x in [0.3_f64, -1.7, 2.5, 1e-3, 123.456] {
            assert_eq!(x.dpowi(0), 1.0);
            assert_eq!(x.dpowi(1).to_bits(), x.to_bits());
            assert_eq!(x.dpowi(2).to_bits(), (x * x).to_bits());
            assert_eq!(x.dpowi(3).to_bits(), (x * x * x).to_bits());
            assert_eq!(x.dpowi(4).to_bits(), ((x * x) * (x * x)).to_bits());
            assert_eq!(x.dpowi(-2).to_bits(), (1.0 / (x * x)).to_bits());
        }
        assert_eq!(2.0_f32.dpowi(10), 1024.0);
    }

    #[test]
    fn known_answers() {
        // correctly rounded values (CORE-MATH's own tests); std's may differ in the last bit
        assert_eq!(0.0_f64.dsin(), 0.0);
        assert_eq!(0.0_f64.dcos(), 1.0);
        assert_eq!(1.0_f64.dexp(), std::f64::consts::E);
        assert_eq!(std::f64::consts::E.dln(), 1.0);
        assert_eq!(1.0_f64.datan2(1.0), std::f64::consts::FRAC_PI_4);
        assert_eq!(3.0_f64.dhypot(4.0), 5.0);
        assert_eq!(8.0_f64.dcbrt(), 2.0);
        assert_eq!(2.0_f64.dpowf(0.5), std::f64::consts::SQRT_2);
        let (s, c) = 0.5_f64.dsin_cos();
        assert_eq!((s, c), (0.5_f64.dsin(), 0.5_f64.dcos()));
    }
}
