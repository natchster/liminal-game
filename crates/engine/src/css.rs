//! Tiny CSS parsers: colours (as the canvas and `Color` style strings use them) and fonts.

/// Parse a CSS colour into sRGB components 0..1 plus alpha.
pub fn parse_css_color(s: &str) -> Option<[f64; 4]> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('#') {
        let hv = |i: usize, n: usize| u32::from_str_radix(&h[i..i + n], 16).ok();
        return match h.len() {
            3 => Some([hv(0, 1)? as f64 * 17.0 / 255.0, hv(1, 1)? as f64 * 17.0 / 255.0, hv(2, 1)? as f64 * 17.0 / 255.0, 1.0]),
            4 => Some([
                hv(0, 1)? as f64 * 17.0 / 255.0,
                hv(1, 1)? as f64 * 17.0 / 255.0,
                hv(2, 1)? as f64 * 17.0 / 255.0,
                hv(3, 1)? as f64 * 17.0 / 255.0,
            ]),
            6 => Some([hv(0, 2)? as f64 / 255.0, hv(2, 2)? as f64 / 255.0, hv(4, 2)? as f64 / 255.0, 1.0]),
            8 => Some([hv(0, 2)? as f64 / 255.0, hv(2, 2)? as f64 / 255.0, hv(4, 2)? as f64 / 255.0, hv(6, 2)? as f64 / 255.0]),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    if let Some(open) = lower.find('(') {
        let name = lower[..open].trim();
        let close = lower.rfind(')').unwrap_or(lower.len());
        let args: Vec<&str> = lower[open + 1..close].split([',', '/', ' ']).map(|a| a.trim()).filter(|a| !a.is_empty()).collect();
        let num = |a: &str| -> Option<f64> { a.trim_end_matches('%').parse::<f64>().ok() };
        let chan = |a: &str| -> Option<f64> {
            if a.ends_with('%') {
                Some((num(a)? / 100.0).clamp(0.0, 1.0))
            } else {
                Some((num(a)? / 255.0).clamp(0.0, 1.0))
            }
        };
        let alpha = |a: Option<&&str>| -> f64 {
            match a {
                Some(a) => {
                    let v = if a.ends_with('%') { num(a).unwrap_or(100.0) / 100.0 } else { num(a).unwrap_or(1.0) };
                    v.clamp(0.0, 1.0)
                }
                None => 1.0,
            }
        };
        match name {
            "rgb" | "rgba" => {
                if args.len() < 3 {
                    return None;
                }
                return Some([chan(args[0])?, chan(args[1])?, chan(args[2])?, alpha(args.get(3))]);
            }
            "hsl" | "hsla" => {
                if args.len() < 3 {
                    return None;
                }
                let h = num(args[0].trim_end_matches("deg"))? / 360.0;
                let sat = (num(args[1])? / 100.0).clamp(0.0, 1.0);
                let l = (num(args[2])? / 100.0).clamp(0.0, 1.0);
                let (r, g, b) = hsl_to_rgb(h, sat, l);
                return Some([r, g, b, alpha(args.get(3))]);
            }
            _ => return None,
        }
    }
    named_color(&lower)
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = ((h % 1.0) + 1.0) % 1.0;
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let f = |mut t: f64| {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (f(h + 1.0 / 3.0), f(h), f(h - 1.0 / 3.0))
}

fn named_color(s: &str) -> Option<[f64; 4]> {
    let hex: u32 = match s {
        "transparent" => return Some([0.0, 0.0, 0.0, 0.0]),
        "black" => 0x000000,
        "white" => 0xffffff,
        "red" => 0xff0000,
        "green" => 0x008000,
        "blue" => 0x0000ff,
        "yellow" => 0xffff00,
        "orange" => 0xffa500,
        "purple" => 0x800080,
        "gray" | "grey" => 0x808080,
        "silver" => 0xc0c0c0,
        "maroon" => 0x800000,
        "navy" => 0x000080,
        "teal" => 0x008080,
        "olive" => 0x808000,
        "lime" => 0x00ff00,
        "aqua" | "cyan" => 0x00ffff,
        "fuchsia" | "magenta" => 0xff00ff,
        "brown" => 0xa52a2a,
        "pink" => 0xffc0cb,
        "gold" => 0xffd700,
        "beige" => 0xf5f5dc,
        "ivory" => 0xfffff0,
        "khaki" => 0xf0e68c,
        "tan" => 0xd2b48c,
        "darkred" => 0x8b0000,
        "darkgreen" => 0x006400,
        "darkblue" => 0x00008b,
        "lightgray" | "lightgrey" => 0xd3d3d3,
        "darkgray" | "darkgrey" => 0xa9a9a9,
        "crimson" => 0xdc143c,
        "salmon" => 0xfa8072,
        "coral" => 0xff7f50,
        "tomato" => 0xff6347,
        "violet" => 0xee82ee,
        "indigo" => 0x4b0082,
        "chocolate" => 0xd2691e,
        "sienna" => 0xa0522d,
        "wheat" => 0xf5deb3,
        "linen" => 0xfaf0e6,
        "skyblue" => 0x87ceeb,
        "steelblue" => 0x4682b4,
        "slategray" | "slategrey" => 0x708090,
        "dimgray" | "dimgrey" => 0x696969,
        "firebrick" => 0xb22222,
        "goldenrod" => 0xdaa520,
        "darkorange" => 0xff8c00,
        "forestgreen" => 0x228b22,
        "seagreen" => 0x2e8b57,
        "olivedrab" => 0x6b8e23,
        "royalblue" => 0x4169e1,
        "midnightblue" => 0x191970,
        "whitesmoke" => 0xf5f5f5,
        "gainsboro" => 0xdcdcdc,
        "snow" => 0xfffafa,
        "cream" => 0xfffdd0,
        _ => return None,
    };
    Some([((hex >> 16) & 255) as f64 / 255.0, ((hex >> 8) & 255) as f64 / 255.0, (hex & 255) as f64 / 255.0, 1.0])
}

/// A parsed CSS `font` shorthand.
#[derive(Clone, Debug, PartialEq)]
pub struct FontSpec {
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub family: String,
}
impl Default for FontSpec {
    fn default() -> Self {
        FontSpec { size: 10.0, bold: false, italic: false, family: "sans-serif".into() }
    }
}

pub fn parse_css_font(s: &str) -> FontSpec {
    let mut f = FontSpec::default();
    let mut rest = Vec::new();
    let mut seen_size = false;
    for tok in s.split_whitespace() {
        if seen_size {
            rest.push(tok);
            continue;
        }
        let t = tok.to_ascii_lowercase();
        match t.as_str() {
            "bold" | "bolder" | "600" | "700" | "800" | "900" => f.bold = true,
            "italic" | "oblique" => f.italic = true,
            "normal" | "small-caps" | "100" | "200" | "300" | "400" | "500" | "lighter" => {}
            _ => {
                let sz = t.split('/').next().unwrap_or("");
                if let Some(px) = sz.strip_suffix("px") {
                    if let Ok(v) = px.parse::<f64>() {
                        f.size = v;
                        seen_size = true;
                        continue;
                    }
                }
                if let Some(pt) = sz.strip_suffix("pt") {
                    if let Ok(v) = pt.parse::<f64>() {
                        f.size = v * 4.0 / 3.0;
                        seen_size = true;
                        continue;
                    }
                }
                if let Some(em) = sz.strip_suffix("em") {
                    if let Ok(v) = em.parse::<f64>() {
                        f.size = v * 16.0;
                        seen_size = true;
                        continue;
                    }
                }
                rest.push(tok);
                seen_size = true;
            }
        }
    }
    if !rest.is_empty() {
        f.family = rest.join(" ");
    }
    f
}

/// CSS filter functions as `RenderGlobals::scene_filter` matrices (Filter Effects 1).
pub mod filter {
    fn m3(a: [f64; 9], off: [f64; 3]) -> [f32; 12] {
        [
            a[0] as f32,
            a[1] as f32,
            a[2] as f32,
            off[0] as f32,
            a[3] as f32,
            a[4] as f32,
            a[5] as f32,
            off[1] as f32,
            a[6] as f32,
            a[7] as f32,
            a[8] as f32,
            off[2] as f32,
        ]
    }
    pub fn saturate(s: f64) -> [f32; 12] {
        m3(
            [
                0.213 + 0.787 * s,
                0.715 - 0.715 * s,
                0.072 - 0.072 * s,
                0.213 - 0.213 * s,
                0.715 + 0.285 * s,
                0.072 - 0.072 * s,
                0.213 - 0.213 * s,
                0.715 - 0.715 * s,
                0.072 + 0.928 * s,
            ],
            [0.0; 3],
        )
    }
    /// `hue-rotate(deg)`
    pub fn hue_rotate(deg: f64) -> [f32; 12] {
        let (s, c) = deg.to_radians().sin_cos();
        m3(
            [
                0.213 + c * 0.787 - s * 0.213,
                0.715 - c * 0.715 - s * 0.715,
                0.072 - c * 0.072 + s * 0.928,
                0.213 - c * 0.213 + s * 0.143,
                0.715 + c * 0.285 + s * 0.140,
                0.072 - c * 0.072 - s * 0.283,
                0.213 - c * 0.213 - s * 0.787,
                0.715 - c * 0.715 + s * 0.715,
                0.072 + c * 0.928 + s * 0.072,
            ],
            [0.0; 3],
        )
    }
    pub fn contrast(c: f64) -> [f32; 12] {
        let o = 0.5 - 0.5 * c;
        m3([c, 0.0, 0.0, 0.0, c, 0.0, 0.0, 0.0, c], [o, o, o])
    }
    pub fn brightness(b: f64) -> [f32; 12] {
        m3([b, 0.0, 0.0, 0.0, b, 0.0, 0.0, 0.0, b], [0.0; 3])
    }
}
