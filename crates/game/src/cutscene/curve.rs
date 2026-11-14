//! The curves a cutscene's camera moves on: easing, a centripetal
//! Catmull-Rom spline through its keys' positions, and angles turned the
//! shortest way round. Pure functions of their arguments (no state, no
//! random draw), so a camera is a function of the cutscene's time alone.
use std::f64::consts::{PI, TAU};

/// How a segment of a track eases from its key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ease {
    Linear,
    /// slow out of the last key, then steady
    In,
    /// steady, then slow into this key
    Out,
    /// slow out and slow in
    InOut,
}

impl Ease {
    pub fn parse(s: &str) -> Option<Ease> {
        Some(match s {
            "linear" => Ease::Linear,
            "in" => Ease::In,
            "out" => Ease::Out,
            "in_out" => Ease::InOut,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Ease::Linear => "linear",
            Ease::In => "in",
            Ease::Out => "out",
            Ease::InOut => "in_out",
        }
    }
    /// `u` in 0..=1 eased (cubic: its slope is 0 at an eased end)
    pub fn at(self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        match self {
            Ease::Linear => u,
            Ease::In => u * u * u,
            Ease::Out => 1.0 - (1.0 - u).powi(3),
            Ease::InOut => u * u * (3.0 - 2.0 * u),
        }
    }
}

/// `x` in `[a, b]` smoothed to 0..=1 (smoothstep; 1 when the span is empty
/// and `x` has reached `b`)
pub fn smooth(x: f64, a: f64, b: f64) -> f64 {
    if b <= a {
        return if x >= b { 1.0 } else { 0.0 };
    }
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// An angle wrapped into (-π, π]
pub fn wrap(a: f64) -> f64 {
    let mut a = a % TAU;
    if a <= -PI {
        a += TAU;
    } else if a > PI {
        a -= TAU;
    }
    a
}

/// From angle `a` toward `b` by `t`, the shortest way round; at `t` = 0 it
/// is `a` exactly, at `t` = 1 `b` exactly (not `b` ± 2π)
pub fn lerp_angle(a: f64, b: f64, t: f64) -> f64 {
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    a + wrap(b - a) * t
}

/// The point at `u` (0..=1) between `p1` and `p2` on the centripetal
/// Catmull-Rom spline through `p0, p1, p2, p3` (Barry and Goldman's
/// pyramid, α = 0.5: no cusp or loop within a segment, unlike the uniform
/// spline, and the curve passes through every key). Coincident points fall
/// back to a straight line between `p1` and `p2`.
pub fn catmull(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], u: f64) -> [f64; 3] {
    let knot = |a: [f64; 3], b: [f64; 3]| dist(a, b).sqrt().max(1e-6);
    let d12 = dist(p1, p2);
    if d12 < 1e-9 {
        return p1;
    }
    let t0 = 0.0;
    let t1 = t0 + knot(p0, p1);
    let t2 = t1 + knot(p1, p2);
    let t3 = t2 + knot(p2, p3);
    let t = lerp(t1, t2, u.clamp(0.0, 1.0));
    let mix = |a: [f64; 3], b: [f64; 3], ta: f64, tb: f64| {
        let k = if (tb - ta).abs() < 1e-12 { 0.0 } else { (t - ta) / (tb - ta) };
        lerp3(a, b, k)
    };
    let a1 = mix(p0, p1, t0, t1);
    let a2 = mix(p1, p2, t1, t2);
    let a3 = mix(p2, p3, t2, t3);
    let b1 = mix(a1, a2, t0, t2);
    let b2 = mix(a2, a3, t1, t3);
    let c = mix(b1, b2, t1, t2);
    if u <= 0.0 {
        p1
    } else if u >= 1.0 {
        p2
    } else {
        c
    }
}

/// Yaw and pitch (the camera's Euler YXZ: yaw 0 looks down -z, pitch up is
/// positive) of the direction from `from` to `to`
pub fn look_angles(from: [f64; 3], to: [f64; 3]) -> (f64, f64) {
    let (dx, dy, dz) = (to[0] - from[0], to[1] - from[1], to[2] - from[2]);
    let flat = (dx * dx + dz * dz).sqrt();
    ((-dx).atan2(-dz), dy.atan2(flat))
}

/// A shake: fast, within ±1 on each of yaw, pitch and roll, a sum of sines
/// at frequencies with no common period, so it is a function of the time
/// alone (the same at any frame rate) and needs no random draw
pub fn jitter(t: f64) -> (f64, f64, f64) {
    let j = |a: f64, b: f64, c: f64, p: f64| (t * a + p).sin() * 0.5 + (t * b + p * 1.7).sin() * 0.3 + (t * c + p * 2.3).sin() * 0.2;
    (j(83.0, 131.0, 197.0, 0.3), j(89.0, 139.0, 211.0, 1.9), j(97.0, 149.0, 223.0, 3.1))
}

/// A held camera's sway: slow, small, never repeating within a cutscene,
/// a sum of sines at frequencies with no common period, so it needs no
/// random draw. → (yaw, pitch) in radians for `amount` 1 (about 0.4°).
pub fn sway(t: f64, amount: f64) -> (f64, f64) {
    if amount == 0.0 {
        return (0.0, 0.0);
    }
    let k = 0.007 * amount;
    let yaw = (t * 0.71).sin() * 0.6 + (t * 1.37 + 1.3).sin() * 0.3 + (t * 2.93 + 0.4).sin() * 0.1;
    let pitch = (t * 0.53 + 2.1).sin() * 0.6 + (t * 1.19 + 0.7).sin() * 0.3 + (t * 3.31 + 2.6).sin() * 0.1;
    (yaw * k, pitch * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eases_end_where_they_start_and_stop() {
        for e in [Ease::Linear, Ease::In, Ease::Out, Ease::InOut] {
            assert_eq!(e.at(0.0), 0.0, "{e:?}");
            assert_eq!(e.at(1.0), 1.0, "{e:?}");
            assert_eq!(Ease::parse(e.name()), Some(e));
        }
        // an eased end has no slope: a camera starts and stops gently
        let slope = |e: Ease, u: f64| (e.at(u + 1e-4) - e.at(u)) / 1e-4;
        assert!(slope(Ease::InOut, 0.0) < 1e-3 && slope(Ease::InOut, 1.0 - 1e-4) < 1e-3);
        assert!(slope(Ease::In, 0.0) < 1e-3 && slope(Ease::Out, 1.0 - 1e-4) < 1e-3);
        assert!((slope(Ease::Linear, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_spline_passes_its_keys_and_has_no_loop() {
        let p = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.2, 0.0, 0.1], [5.0, 0.0, 4.0]];
        assert_eq!(catmull(p[0], p[1], p[2], p[3], 0.0), p[1]);
        assert_eq!(catmull(p[0], p[1], p[2], p[3], 1.0), p[2]);
        // a short segment between long ones stays near its chord (the uniform
        // spline overshoots and loops here)
        for k in 1..20 {
            let q = catmull(p[0], p[1], p[2], p[3], k as f64 / 20.0);
            assert!(q[0] > 0.95 && q[0] < 1.3 && q[2].abs() < 0.2, "{q:?}");
        }
        // coincident keys: a hold
        assert_eq!(catmull(p[0], p[1], p[1], p[3], 0.5), p[1]);
    }

    #[test]
    fn angles_turn_the_short_way() {
        assert!((lerp_angle(3.0, -3.0, 0.5).abs() - PI).abs() < 0.01);
        assert_eq!(lerp_angle(3.0, -3.0, 1.0), -3.0);
        assert_eq!(lerp_angle(3.0, -3.0, 0.0), 3.0);
        assert!((wrap(7.0) - (7.0 - TAU)).abs() < 1e-12);
        let (yaw, pitch) = look_angles([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        assert!(yaw.abs() < 1e-12 && pitch.abs() < 1e-12);
        let (yaw, _) = look_angles([0.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
        assert!((yaw - PI / 2.0).abs() < 1e-12, "left is +π/2 as the player's yaw");
        let (_, pitch) = look_angles([0.0, 0.0, 0.0], [0.0, 1.0, -1.0]);
        assert!((pitch - PI / 4.0).abs() < 1e-12);
    }

    #[test]
    fn a_jitter_stays_within_one() {
        let worst = (0..5000).map(|i| jitter(i as f64 * 0.003)).fold(0.0f64, |m, (a, b, c)| m.max(a.abs()).max(b.abs()).max(c.abs()));
        assert!(worst <= 1.0 && worst > 0.6, "{worst}");
    }

    #[test]
    fn sway_is_small_and_still_when_off() {
        assert_eq!(sway(3.0, 0.0), (0.0, 0.0));
        let worst = (0..2000).map(|i| sway(i as f64 * 0.01, 1.0)).fold(0.0f64, |m, (y, p)| m.max(y.abs()).max(p.abs()));
        assert!(worst < 0.008 && worst > 0.003, "{worst}");
    }
}
