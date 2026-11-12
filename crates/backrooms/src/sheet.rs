//! Contact sheets for the visual playtest (`tour.rs`): tiles in a grid on a
//! dark ground, each with its name written under it, so a reader (or the
//! `visual-inspector`) can say which view a picture is.
use engine::canvas::{Canvas, ImageData};

/// A picture and its name
pub struct Tile {
    pub px: Vec<u8>,
    pub w: usize,
    pub h: usize,
    pub label: String,
}

/// The band under a tile its name is written in (pixels)
const BAND: usize = 16;
const GAP: usize = 6;

/// A picture `k` times smaller each way (a box filter)
pub fn shrink(px: &[u8], w: usize, h: usize, k: usize) -> (Vec<u8>, usize, usize) {
    let (sw, sh) = (w / k, h / k);
    let mut out = vec![0u8; sw * sh * 4];
    for y in 0..sh {
        for x in 0..sw {
            let mut acc = [0u32; 4];
            for dy in 0..k {
                for dx in 0..k {
                    let s = ((y * k + dy) * w + x * k + dx) * 4;
                    for c in 0..4 {
                        acc[c] += px[s + c] as u32;
                    }
                }
            }
            let d = (y * sw + x) * 4;
            for c in 0..4 {
                out[d + c] = (acc[c] / (k * k) as u32) as u8;
            }
        }
    }
    (out, sw, sh)
}

/// Two pictures of one size side by side
pub fn side_by_side(a: &[u8], b: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * 2 * h * 4];
    for y in 0..h {
        out[y * w * 8..y * w * 8 + w * 4].copy_from_slice(&a[y * w * 4..(y + 1) * w * 4]);
        out[y * w * 8 + w * 4..(y + 1) * w * 8].copy_from_slice(&b[y * w * 4..(y + 1) * w * 4]);
    }
    out
}

/// The tiles (all of one size) `cols` across, each named under it
pub fn sheet(tiles: &[Tile], cols: usize) -> (Vec<u8>, usize, usize) {
    let Some(first) = tiles.first() else { return (vec![], 0, 0) };
    let (tw, th) = (first.w, first.h);
    let rows = tiles.len().div_ceil(cols);
    let (w, h) = (cols * tw + (cols + 1) * GAP, rows * (th + BAND) + (rows + 1) * GAP);
    let mut img = vec![0u8; w * h * 4];
    for p in img.chunks_exact_mut(4) {
        p.copy_from_slice(&[20, 20, 22, 255]);
    }
    for (i, t) in tiles.iter().enumerate() {
        let (ox, oy) = (GAP + (i % cols) * (tw + GAP), GAP + (i / cols) * (th + BAND + GAP));
        for y in 0..th.min(t.h) {
            let d = ((oy + y) * w + ox) * 4;
            img[d..d + tw.min(t.w) * 4].copy_from_slice(&t.px[y * t.w * 4..y * t.w * 4 + tw.min(t.w) * 4]);
        }
    }
    // the names, written by the canvas the HUD uses
    let mut c = Canvas::new(w as u32, h as u32);
    c.put_image_data(&ImageData { width: w as u32, height: h as u32, data: img }, 0, 0);
    c.set_font("11px Helvetica, Arial, sans-serif");
    c.set_fill_style("#e8e4da");
    for (i, t) in tiles.iter().enumerate() {
        let (ox, oy) = (GAP + (i % cols) * (tw + GAP), GAP + (i / cols) * (th + BAND + GAP));
        c.fill_text_max(&t.label, ox as f64 + 2.0, (oy + th + 12) as f64, (tw - 4) as f64);
    }
    let out = c.get_image_data(0, 0, w as u32, h as u32);
    (out.data, w, h)
}
