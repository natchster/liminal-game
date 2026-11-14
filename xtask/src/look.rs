//! Look statistics of a rendered frame: the numbers that showed the props
//! becoming less cartoonish and the levels darker, kept as a gate.
//!
//! - **colourfulness**: Hasler and Süsstrunk's metric on 8-bit sRGB
//!   (√(σ_rg² + σ_yb²) + 0.3·√(μ_rg² + μ_yb²)); about 15 is muted, 40 vivid;
//! - **luminance**: mean relative luminance (0–1);
//! - **contrast**: its standard deviation;
//! - **shadow**: the share of pixels darker than 0.12 (deep shadow).
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub colourfulness: f64,
    pub luminance: f64,
    pub contrast: f64,
    pub shadow: f64,
}

/// Luminance below this counts as deep shadow.
const SHADOW: f64 = 0.12;

/// The statistics of an RGBA frame (every 2nd pixel in each direction).
pub fn look_of(w: u32, h: u32, rgba: &[u8]) -> Look {
    let (mut rg, mut yb, mut lum) = (vec![], vec![], vec![]);
    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let i = ((y * w + x) * 4) as usize;
            let (r, g, b) = (rgba[i] as f64, rgba[i + 1] as f64, rgba[i + 2] as f64);
            rg.push(r - g);
            yb.push(0.5 * (r + g) - b);
            lum.push((0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0);
        }
    }
    let ms = |v: &[f64]| {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (m, (v.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / v.len() as f64).sqrt())
    };
    let ((mrg, srg), (myb, syb), (ml, sl)) = (ms(&rg), ms(&yb), ms(&lum));
    Look {
        colourfulness: (srg * srg + syb * syb).sqrt() + 0.3 * (mrg * mrg + myb * myb).sqrt(),
        luminance: ml,
        contrast: sl,
        shadow: lum.iter().filter(|&&l| l < SHADOW).count() as f64 / lum.len() as f64,
    }
}

impl Look {
    pub fn to_json(self) -> Value {
        let r = |v: f64, d: i32| (v * 10f64.powi(d)).round() / 10f64.powi(d);
        json!({
            "colourfulness": r(self.colourfulness, 2),
            "luminance": r(self.luminance, 4),
            "contrast": r(self.contrast, 4),
            "shadow": r(self.shadow, 4),
        })
    }
    pub fn from_json(v: &Value) -> Option<Look> {
        Some(Look {
            colourfulness: v["colourfulness"].as_f64()?,
            luminance: v["luminance"].as_f64()?,
            contrast: v["contrast"].as_f64()?,
            shadow: v["shadow"].as_f64()?,
        })
    }
    pub fn describe(&self) -> String {
        format!(
            "colourfulness {:.1}, luminance {:.3}, contrast {:.3}, shadow {:.1}%",
            self.colourfulness,
            self.luminance,
            self.contrast,
            self.shadow * 100.0
        )
    }
    /// A shift (self as a difference), signed.
    pub fn describe_shift(&self) -> String {
        format!(
            "colourfulness {:+.2}, luminance {:+.4}, contrast {:+.4}, shadow {:+.4}",
            self.colourfulness, self.luminance, self.contrast, self.shadow
        )
    }
    /// What has moved from `anchor` by more than `band`, worded as the drift
    /// it is; empty when the look holds. Moving toward more shadow, less
    /// colour or a darker frame is allowed up to twice the band (the game's
    /// direction), the other way only up to the band.
    pub fn outside(&self, anchor: &Look, band: &Look) -> Vec<String> {
        let mut out = vec![];
        let mut check = |name: &str, v: f64, a: f64, b: f64, worse_up: bool| {
            let d = v - a;
            let (up, down) = if worse_up { (b, 2.0 * b) } else { (2.0 * b, b) };
            if d > up || -d > down {
                out.push(format!("{name} {v:.3} vs anchor {a:.3} ({}{:.3})", if d > 0.0 { "+" } else { "" }, d));
            }
        };
        check("colourfulness", self.colourfulness, anchor.colourfulness, band.colourfulness, true);
        check("luminance", self.luminance, anchor.luminance, band.luminance, true);
        check("contrast", self.contrast, anchor.contrast, band.contrast, false);
        check("shadow", self.shadow, anchor.shadow, band.shadow, false);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_is_not_colourful_and_red_is() {
        let grey: Vec<u8> = (0..16 * 16).flat_map(|_| [128, 128, 128, 255]).collect();
        let red: Vec<u8> = (0..16 * 16).flat_map(|i| if i % 2 == 0 { [220, 30, 30, 255] } else { [40, 40, 40, 255] }).collect();
        assert!(look_of(16, 16, &grey).colourfulness < 0.01);
        assert!(look_of(16, 16, &red).colourfulness > 40.0);
        assert_eq!(look_of(16, 16, &grey).shadow, 0.0);
    }

    #[test]
    fn a_brighter_more_colourful_frame_is_drift() {
        let a = Look { colourfulness: 15.0, luminance: 0.2, contrast: 0.1, shadow: 0.1 };
        let band = Look { colourfulness: 4.0, luminance: 0.03, contrast: 0.03, shadow: 0.06 };
        assert!(a.outside(&a, &band).is_empty());
        let vivid = Look { colourfulness: 20.0, ..a };
        assert_eq!(vivid.outside(&a, &band).len(), 1);
        // a little more muted is the game's direction: allowed up to twice the band
        let muted = Look { colourfulness: 8.0, ..a };
        assert!(muted.outside(&a, &band).is_empty());
        let washed = Look { shadow: 0.0, ..a };
        assert_eq!(washed.outside(&a, &band).len(), 1);
    }
}
