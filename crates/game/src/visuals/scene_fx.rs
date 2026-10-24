//! The CSS filter on the 3D canvas:
//!
//! ```css
//! #gl { transition: filter 0.6s ease-out; }
//! body.low-sanity #gl  { filter: saturate(0.65) hue-rotate(-6deg)  contrast(1.05); }
//! body.crit-sanity #gl { filter: saturate(0.40) hue-rotate(-14deg) contrast(1.12) brightness(0.92); }
//! ```
//!
//! The HUD toggles the body classes (`update_hud`). Here the filter list is
//! resolved from them and transitioned the way CSS does it: lists of the same
//! functions interpolate per function, a shorter list is padded with the
//! missing functions' initial values (and `none` with all of them), and a
//! transition that is interrupted restarts from the value shown at that moment.
//! The renderer applies the result as colour matrices (`RenderGlobals::scene_filter`).
use crate::prelude::*;
use engine::css::filter as css_filter;

/// (saturate, hue-rotate deg, contrast, brightness); `none` is the identity
pub type FilterParams = [f64; 4];
pub const FILTER_NONE: FilterParams = [1.0, 0.0, 1.0, 1.0];
const FILTER_LOW: FilterParams = [0.65, -6.0, 1.05, 1.0];
const FILTER_CRIT: FilterParams = [0.40, -14.0, 1.12, 0.92];
const TRANSITION_MS: f64 = 600.0;

/// The canvas filter's transition state.
#[derive(Clone, Copy, Debug)]
pub struct SceneFilterState {
    pub from: FilterParams,
    pub to: FilterParams,
    pub t0_ms: f64,
}
impl Default for SceneFilterState {
    fn default() -> Self {
        SceneFilterState { from: FILTER_NONE, to: FILTER_NONE, t0_ms: f64::NEG_INFINITY }
    }
}

/// CSS `ease-out` = cubic-bezier(0, 0, 0.58, 1)
pub fn ease_out(x: f64) -> f64 {
    cubic_bezier(0.0, 0.0, 0.58, 1.0, x)
}

/// The y of a CSS cubic-bezier(x1, y1, x2, y2) timing function at progress x.
pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let (cx, bx) = (3.0 * x1, 3.0 * (x2 - x1) - 3.0 * x1);
    let ax = 1.0 - cx - bx;
    let (cy, by) = (3.0 * y1, 3.0 * (y2 - y1) - 3.0 * y1);
    let ay = 1.0 - cy - by;
    let sx = |t: f64| ((ax * t + bx) * t + cx) * t;
    let dsx = |t: f64| (3.0 * ax * t + 2.0 * bx) * t + cx;
    // Newton, then bisection as a fallback
    let mut t = x;
    for _ in 0..8 {
        let e = sx(t) - x;
        if e.abs() < 1e-7 {
            return ((ay * t + by) * t + cy) * t;
        }
        let d = dsx(t);
        if d.abs() < 1e-6 {
            break;
        }
        t -= e / d;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    t = x;
    for _ in 0..40 {
        let v = sx(t);
        if (v - x).abs() < 1e-7 {
            break;
        }
        if v < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) * 0.5;
    }
    ((ay * t + by) * t + cy) * t
}

impl Game {
    /// The filter the body classes ask for right now.
    fn scene_filter_target(&self) -> FilterParams {
        if self.ui.body.has_class("crit-sanity") {
            FILTER_CRIT
        } else if self.ui.body.has_class("low-sanity") {
            FILTER_LOW
        } else {
            FILTER_NONE
        }
    }
    /// The filter shown at `now_ms`.
    fn scene_filter_at(&self, now_ms: f64) -> FilterParams {
        let s = &self.scene_filter;
        let k = ease_out(((now_ms - s.t0_ms) / TRANSITION_MS).clamp(0.0, 1.0));
        let mut out = [0.0; 4];
        for i in 0..4 {
            out[i] = s.from[i] + (s.to[i] - s.from[i]) * k;
        }
        out
    }
    /// The canvas's CSS filter as renderer matrices (empty when it is `none`);
    /// starts a transition when the body classes changed since the last call.
    pub fn scene_filter_matrices(&mut self) -> Vec<[f32; 12]> {
        let now = self.now_ms;
        let want = self.scene_filter_target();
        if want != self.scene_filter.to {
            let cur = self.scene_filter_at(now);
            self.scene_filter = SceneFilterState { from: cur, to: want, t0_ms: now };
        }
        let p = self.scene_filter_at(now);
        if p == FILTER_NONE {
            return vec![];
        }
        vec![css_filter::saturate(p[0]), css_filter::hue_rotate(p[1]), css_filter::contrast(p[2]), css_filter::brightness(p[3])]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ease_out_matches_css() {
        // cubic-bezier(0,0,.58,1), solved by bisection on the parametric curve
        for (x, y) in [(0.0, 0.0), (0.25, 0.378138130825), (0.5, 0.684643187427), (0.75, 0.906535349281), (1.0, 1.0)] {
            assert!((ease_out(x) - y).abs() < 1e-6, "{x}: {} vs {y}", ease_out(x));
        }
    }
}
