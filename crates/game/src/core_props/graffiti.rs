//! Writing on the walls (GitHub issue #11; `docs/research/graffiti.md`, the
//! work in `docs/plan/PROPS.md`, *Writing on the walls*): what someone shut
//! in here wrote, with what an office gives (a pencil, a ballpoint, a
//! marker, a lipstick, a key to scratch with, correction fluid, their own
//! blood gone brown), at that tool's size, in a hand that drifts the way a
//! frightened one does: lines that slope and wander, letters that vary and
//! grow and crowd at the end of a line, pressure heavy at the start and now
//! and then fading out, a letter gone over again, a word crossed out.
//!
//! What is decided before it is painted (its lines, size and shape) comes
//! from a stream of its own seeded by its place and its words
//! (`SplitMix64`), never the level's; the strokes' small tremors are drawn
//! by its painter on the painter's own stream (`canvas_tex`).

use crate::base::rng::SplitMix64;
use crate::placers::kit::canvas_tex_opt;
use crate::prelude::*;
use crate::visuals::textures::strokes;

/// What it was written with
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Medium {
    /// a finger, in blood that has dried brown
    Blood,
    /// correction fluid, opaque and off-white
    Correction,
    /// a black permanent marker
    Marker,
    /// a lipstick, waxy and broken
    Lipstick,
    /// a key or a staple: the paint scratched to its lighter coat
    Scratch,
    /// a blue ballpoint, skipping on a wall
    Ballpoint,
    /// a pencil
    Pencil,
}

/// The seven media, by the level's draw (`irand(0, 6)`: the old hand
/// styles' seven, in their order, so the draws are the same)
pub const MEDIA: [Medium; 7] =
    [Medium::Blood, Medium::Correction, Medium::Marker, Medium::Lipstick, Medium::Scratch, Medium::Ballpoint, Medium::Pencil];

/// How a medium marks: its colour (sRGB 0-255) and opacity, its stroke's
/// width as a share of a capital's height, the capital's height (m), how
/// often it skips, and its second colour (a pool of blood, a scratch's
/// groove, a marker's darker rim)
#[derive(Clone, Copy, Debug)]
pub struct Ink {
    pub rgb: [f64; 3],
    pub alpha: f64,
    pub width: f64,
    pub cap: (f64, f64),
    pub skips: f64,
    pub second: [f64; 3],
}

/// Each medium's ink (the research's colours: `graffiti.md`, rule 1; its
/// sizes, rule 5)
pub fn ink(m: Medium) -> Ink {
    let i = |rgb: [f64; 3], alpha: f64, width: f64, cap: (f64, f64), skips: f64, second: [f64; 3]| Ink {
        rgb,
        alpha,
        width,
        cap,
        skips,
        second,
    };
    match m {
        Medium::Blood => i([78.0, 28.0, 22.0], 0.9, 0.13, (0.09, 0.15), 0.01, [40.0, 15.0, 13.0]),
        Medium::Correction => i([230.0, 228.0, 214.0], 0.97, 0.11, (0.05, 0.08), 0.0, [205.0, 203.0, 190.0]),
        Medium::Marker => i([36.0, 32.0, 38.0], 0.92, 0.09, (0.05, 0.09), 0.0, [22.0, 20.0, 24.0]),
        Medium::Lipstick => i([150.0, 42.0, 48.0], 0.85, 0.11, (0.05, 0.08), 0.04, [120.0, 30.0, 36.0]),
        Medium::Scratch => i([205.0, 198.0, 182.0], 0.55, 0.035, (0.03, 0.05), 0.02, [30.0, 24.0, 18.0]),
        Medium::Ballpoint => i([52.0, 62.0, 140.0], 0.85, 0.055, (0.025, 0.04), 0.07, [52.0, 62.0, 140.0]),
        Medium::Pencil => i([88.0, 88.0, 90.0], 0.72, 0.05, (0.025, 0.045), 0.02, [70.0, 70.0, 72.0]),
    }
}

/// A capital's height in the texture, px (the texture's scale follows the
/// medium's: a pencil's few centimetres and a finger's span alike are drawn
/// at this many pixels)
const CAP_PX: f64 = 52.0;
/// A letter's box's width, the gap after it and a space's advance, in
/// capitals' heights (a narrow letter takes less room: `advance`)
const GLYPH_W: f64 = 0.72;
const GAP: f64 = 0.24;
const SPACE: f64 = 0.5;
/// From one line's foot to the next's, in capitals' heights: room for the
/// largest letters (grown and varied) over the next line's
const LINE_H: f64 = 1.9;
/// The steepest a line slopes (tan), and the margin, in capitals' heights
const STEEPEST: f64 = 0.12;
const MARGIN: f64 = 0.8;
/// The texture's widest, px
const MAX_W: f64 = 1024.0;

/// Where a letter's strokes start and end across its box (0-1)
fn extent(c: char) -> Option<(f64, f64)> {
    let g = strokes(c)?;
    let (lo, hi) = g.iter().flat_map(|p| p.iter()).fold((1.0f64, 0.0f64), |(lo, hi), &[u, _]| (lo.min(u), hi.max(u)));
    (lo <= hi).then_some((lo, hi))
}

/// A letter's advance, or a space's, in capitals' heights: its own width
/// and a gap, so an I or an apostrophe takes less room than a W
fn advance(c: char) -> f64 {
    match (c, extent(c)) {
        (' ', _) => SPACE,
        (_, Some((lo, hi))) => (hi - lo) * GLYPH_W + GAP,
        _ => GLYPH_W + GAP,
    }
}

/// A piece of writing as it will be painted: its lines, how big, how it
/// slopes, how its pressure goes, what is crossed out
#[derive(Clone, Debug)]
pub struct Writing {
    pub medium: Medium,
    pub lines: Vec<Vec<char>>,
    /// a capital's height, px, and the texture's size and scale (px a metre)
    pub cap: f64,
    pub w: f64,
    pub h: f64,
    pub ppm: f64,
    /// each line's slope (rad) and its left edge's step in (px)
    pub slope: Vec<f64>,
    pub indent: Vec<f64>,
    /// the last letter's size against the first's
    pub growth: f64,
    /// the pressure fades along it to this share (1: it does not)
    pub fade_to: f64,
    /// a word crossed out: its line and its place in the line
    pub crossed: Option<(usize, usize)>,
    /// written sitting (lower on the wall)
    pub low: bool,
}

/// The words broken into short lines (one to four words, as people write on
/// a wall), the break chosen by `r`
fn break_lines(text: &str, r: &mut SplitMix64) -> Vec<Vec<char>> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let max = 7.0 + (r.next() * 8.0).floor();
    let mut lines: Vec<String> = vec![];
    for w in words {
        match lines.last_mut() {
            Some(l) if (l.len() + 1 + w.len()) as f64 <= max => {
                l.push(' ');
                l.push_str(w);
            }
            _ => lines.push(w.to_string()),
        }
    }
    lines.into_iter().map(|l| l.chars().collect()).collect()
}

/// The plan of `text` written in `m`, decided by `seed` (its place and
/// words) and `roll` (a draw the level made for it, in 0..1)
pub fn plan(text: &str, m: Medium, seed: u32, roll: f64) -> Writing {
    let mut r = SplitMix64::new(seed, 0x6EAF_F17E_0D15_7E55);
    let lines = break_lines(text, &mut r);
    let k = ink(m);
    let cap_m = r.range(k.cap.0, k.cap.1);
    let growth = r.range(1.08, 1.3);
    // a writer's slope is a habit: every line falls (or rises) the same way
    // as the first, a little more or less (the draws as they were, a pair a
    // line, so what is drawn after them stays)
    let mut slope: Vec<f64> = vec![];
    for _ in &lines {
        let (a, dir) = (r.range(0.035, 0.1), if r.next() < 0.5 { -1.0 } else { 1.0 });
        slope.push(match slope.first() {
            None => dir * a,
            Some(s0) => s0.signum() * (s0.abs() + (a - 0.0675) * 0.3),
        });
    }
    let indent: Vec<f64> = lines.iter().map(|_| r.range(0.0, 0.5) * CAP_PX).collect();
    // about one in four fades along its length (a dying hand, a pen running dry)
    let fade_to = if roll < 0.25 || r.next() < 0.2 { r.range(0.25, 0.55) } else { 1.0 };
    let words: Vec<usize> = lines.iter().map(|l| l.iter().filter(|c| **c == ' ').count() + 1).collect();
    let crossed = if words.iter().sum::<usize>() >= 3 && r.next() < 0.18 {
        let li = (r.next() * lines.len() as f64) as usize % lines.len();
        Some((li, (r.next() * words[li] as f64) as usize % words[li]))
    } else {
        None
    };
    let low = matches!(m, Medium::Pencil | Medium::Scratch) && r.next() < 0.35;
    // its size: the longest line at the letters' largest, the lines and the tallies
    let longest = lines.iter().map(|l| l.iter().copied().map(advance).sum::<f64>()).fold(0.0, f64::max);
    let mut cap = CAP_PX;
    let w_caps = longest * growth + 0.5 + 2.0 * MARGIN;
    if w_caps * cap > MAX_W {
        cap = MAX_W / w_caps;
    }
    let h_caps = lines.len() as f64 * LINE_H + 2.0 * MARGIN + w_caps * STEEPEST;
    let (w, h) = ((w_caps * cap).ceil(), (h_caps * cap).ceil());
    Writing { medium: m, lines, cap, w, h, ppm: cap / cap_m, slope, indent, growth, fade_to, crossed, low }
}

/// A css colour from an ink's sRGB and an opacity
fn css(rgb: [f64; 3], a: f64) -> String {
    format!("rgba({},{},{},{})", rgb[0].round(), rgb[1].round(), rgb[2].round(), a.clamp(0.0, 1.0))
}

/// One stroke through `pts` (px) in medium `m`: its width `wd` and
/// opacity `a`, its tremor and its skips drawn on the painter's stream
fn ink_stroke(ctx: &mut Canvas, pts: &[[f64; 2]], m: Medium, k: &Ink, wd: f64, a: f64) {
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    // the passes: a marker's bleed and a blood smear under the line, a
    // scratch's groove beside it
    let passes: Vec<([f64; 3], f64, f64, [f64; 2])> = match m {
        Medium::Marker => vec![(k.rgb, a * 0.22, wd * 1.7, [0.0, 0.0]), (k.second, a, wd, [0.0, 0.0])],
        Medium::Blood => vec![(k.rgb, a * 0.3, wd * 1.9, [0.0, wd * 0.4]), (k.rgb, a, wd, [0.0, 0.0])],
        Medium::Scratch => vec![(k.second, a * 0.6, wd * 0.8, [0.6, 1.2]), (k.rgb, a, wd, [0.0, 0.0])],
        _ => vec![(k.rgb, a, wd, [0.0, 0.0])],
    };
    for (rgb, pa, pw, off) in passes {
        ctx.set_stroke_style(css(rgb, pa));
        for i in 0..pts.len().saturating_sub(1) {
            let ([x0, y0], [x1, y1]) = (pts[i], pts[i + 1]);
            let len = (x1 - x0).dhypot(y1 - y0);
            let subs = (len / 5.0).ceil().max(1.0) as usize;
            for s in 0..subs {
                if rnd() < k.skips {
                    continue;
                }
                let (t0, t1) = (s as f64 / subs as f64, (s + 1) as f64 / subs as f64);
                let j = wd * 0.12;
                ctx.set_line_width(pw * rand2(0.75, 1.2));
                ctx.begin_path();
                ctx.move_to(x0 + (x1 - x0) * t0 + rand2(-j, j) + off[0], y0 + (y1 - y0) * t0 + rand2(-j, j) + off[1]);
                ctx.line_to(x0 + (x1 - x0) * t1 + rand2(-j, j) + off[0], y0 + (y1 - y0) * t1 + rand2(-j, j) + off[1]);
                ctx.stroke();
            }
        }
    }
    // now and then correction fluid gathers where the brush lifts, and blood
    // pools where a finger was set down fresh
    if let (Medium::Correction | Medium::Blood, Some(&[x, y])) = (m, pts.first())
        && rnd() < 0.3
    {
        ctx.set_fill_style(css(if m == Medium::Blood { k.second } else { k.rgb }, a * 0.85));
        ctx.begin_path();
        ctx.arc(x, y, wd * rand2(0.35, 0.55), 0, 7);
        ctx.fill();
    }
}

/// The writing painted (transparent where nothing is written)
pub fn writing_tex(wr: &Writing) -> Tex {
    let wr = wr.clone();
    canvas_tex_opt(wr.w, wr.h, false, move |ctx, _w, _h| paint(ctx, &wr))
}

fn paint(ctx: &mut Canvas, wr: &Writing) {
    let k = ink(wr.medium);
    let cap = wr.cap;
    let total: usize = wr.lines.iter().map(Vec::len).sum::<usize>().max(1);
    let mut along = 0usize;
    let margin = cap * MARGIN;
    // lines that rise to the right start low enough that their ends stay on
    let rise = wr.slope.iter().map(|s| (-s.dtan()).max(0.0)).fold(0.0, f64::max) * (wr.w - 2.0 * margin);
    for (li, line) in wr.lines.iter().enumerate() {
        let y0 = margin + cap * 1.25 + rise + li as f64 * cap * LINE_H;
        let mut x = margin + wr.indent[li];
        let tan = wr.slope[li].dtan();
        let mut word_x0 = x;
        let mut word_box: Vec<(f64, f64, f64)> = vec![];
        for (ci, &c) in line.iter().enumerate() {
            let t = along as f64 / total as f64;
            along += 1;
            // a capital's height varies by about a sixth, and grows along it
            let size = cap * (1.0 + (wr.growth - 1.0) * t) * rand2(0.85, 1.15);
            // letters crowd at the end of a line: narrower and closer, the
            // spaces between words kept
            let crowd = if (ci as f64) > line.len() as f64 * 0.7 { rand2(0.7, 0.88) } else { 1.0 };
            if c == ' ' {
                word_box.push((word_x0, x, y0 + (x - margin) * tan));
                x += SPACE * size;
                word_x0 = x;
                continue;
            }
            let (Some(glyph), Some((lo, _))) = (strokes(c), extent(c)) else {
                x += advance(c) * size * crowd;
                continue;
            };
            // heavy at the start; fading along it, if it fades
            let pressure = 1.15 - 0.3 * t;
            let fade = 1.0 + (wr.fade_to - 1.0) * t;
            let wd = cap * k.width * pressure * fade.max(0.5);
            let a = k.alpha * fade;
            let slant = rand2(-0.14, 0.14);
            let base = y0 + (x - margin) * tan + rand2(-0.05, 0.05) * cap;
            let gw = size * GLYPH_W * crowd;
            let to_px = |u: f64, v: f64| {
                // (the letter's box from its first stroke, leaning about its
                // foot by its slant)
                let (dx, dy) = ((u - lo) * gw, (v - 1.0) * size);
                [x + dx - dy * slant, base + dy]
            };
            for poly in glyph {
                // each point of the letter a little off where it belongs: a
                // hurried hand varies the shapes most of all
                let pts: Vec<[f64; 2]> = poly.iter().map(|&[u, v]| to_px(u + rand2(-0.07, 0.07), v + rand2(-0.06, 0.06))).collect();
                ink_stroke(ctx, &pts, wr.medium, &k, wd, a);
                // now and then gone over again, a little off
                if rnd() < 0.15 {
                    let (ox, oy) = (rand2(-1.5, 1.5), rand2(-1.5, 1.5));
                    let again: Vec<[f64; 2]> = pts.iter().map(|&[px, py]| [px + ox, py + oy]).collect();
                    ink_stroke(ctx, &again, wr.medium, &k, wd * 0.8, a * 0.8);
                }
            }
            x += advance(c) * size * crowd;
        }
        word_box.push((word_x0, x, y0 + (x - margin) * tan));
        // a word crossed out: two or three strokes through it
        if let Some((cl, cw)) = wr.crossed
            && cl == li
            && let Some(&(wx0, wx1, wy)) = word_box.get(cw)
        {
            for s in 0..(2 + (rnd() * 2.0) as usize) {
                let y = wy - cap * (0.35 + 0.15 * s as f64) + rand2(-3.0, 3.0);
                ink_stroke(
                    ctx,
                    &[[wx0 - 4.0, y + rand2(-4.0, 4.0)], [wx1 + 2.0, y + rand2(-4.0, 4.0)]],
                    wr.medium,
                    &k,
                    cap * k.width,
                    k.alpha,
                );
            }
        }
    }
}

/// Where on the wall its middle is, m above the floor, from the level's
/// height draw `r` (1.1-2.1, as it was): someone standing writes at
/// 1.2-1.7 m; a pencil or a scratch is sometimes lower, as if sitting
pub fn height(wr: &Writing, r: f64) -> f64 {
    let t = ((r - 1.1) / 1.0).clamp(0.0, 1.0);
    if wr.low { 0.6 + 0.4 * t } else { 1.2 + 0.5 * t }
}

/// The run's deal of a pool of `n` phrases: its order, by the run's seed
/// (no level's stream: a floor's words are its run's and its number's)
pub fn deck(run_seed: u32, n: usize) -> Vec<usize> {
    let mut r = SplitMix64::new(run_seed, 0xD3C4_57A1_7E0F_0A11);
    let mut d: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (r.next() * (i + 1) as f64) as usize % (i + 1);
        d.swap(i, j);
    }
    d
}

/// Whether the run's writing slot `slot` (its floor's number × ten, and its
/// place among the floor's ten) is written: six in ten
pub fn written(run_seed: u32, slot: usize) -> bool {
    SplitMix64::new(run_seed ^ (slot as u32).wrapping_mul(0x9E37_79B9), 0x5107_7E11_0B5D_AA21).next() < 0.6
}

/// A place's and words' seed for a piece of writing
pub fn seed_of(x: f64, z: f64, text: &str) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in ((x * 100.0).round() as i64).to_le_bytes().into_iter().chain(((z * 100.0).round() as i64).to_le_bytes()).chain(text.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 32)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_office_s_media_and_no_aerosol_chalk_or_bright_red() {
        for m in MEDIA {
            let [r, g, b] = ink(m).rgb;
            // no saturated yellow (aerosol), no bright red that never browns
            assert!(!(r > 180.0 && g > 150.0 && b < 80.0), "{m:?}: an aerosol's yellow");
            assert!(!(r > 170.0 && g < 60.0 && b < 60.0), "{m:?}: a red that never browns");
        }
        let blood = ink(Medium::Blood).rgb;
        assert!(blood[0] < 100.0 && blood[0] > blood[1] * 2.0, "blood dried brown, not red: {blood:?}");
    }

    #[test]
    fn the_writing_is_short_lines_at_its_tool_s_size() {
        for (text, m) in [("MOTHER WHERE ARE YOU", Medium::Ballpoint), ("HELP ME", Medium::Blood), ("I'M STILL HERE", Medium::Marker)] {
            for seed in 0..40 {
                let w = plan(text, m, seed, 0.5);
                assert!((1..=4).contains(&w.lines.len()), "{text}: {} lines", w.lines.len());
                assert!(w.lines.iter().all(|l| l.split(|c| *c == ' ').count() <= 4), "{text}: a line of more than four words");
                let cap_m = w.cap / w.ppm;
                let (lo, hi) = ink(m).cap;
                assert!(cap_m >= lo - 1e-9 && cap_m <= hi + 1e-9, "{m:?}: a capital {cap_m} m");
                assert!(w.w <= MAX_W && w.h > 0.0);
            }
        }
    }

    #[test]
    fn about_one_in_four_fades() {
        let fading = (0..400).filter(|s| plan("NO WAY OUT", Medium::Marker, *s, 0.9).fade_to < 1.0).count();
        assert!((40..=140).contains(&fading), "{fading} of 400 fade");
        assert!(plan("NO WAY OUT", Medium::Marker, 1, 0.1).fade_to < 1.0, "the level's roll under 0.25 fades it");
    }

    #[test]
    fn a_floor_never_says_a_thing_twice_and_its_neighbour_says_others() {
        for seed in [1u32, 4242, 77] {
            let d = deck(seed, 25);
            let mut sorted = d.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..25).collect::<Vec<_>>(), "a deal of the whole pool");
            for level in 5..12usize {
                let words = |l: usize| (0..10).map(|i| d[(l * 10 + i) % 25]).collect::<Vec<_>>();
                let here = words(level);
                let mut uniq = here.clone();
                uniq.sort_unstable();
                uniq.dedup();
                assert_eq!(uniq.len(), 10, "a phrase twice on a floor");
                assert!(words(level + 1).iter().filter(|w| here.contains(w)).count() <= 5, "the next floor says the same");
            }
        }
        let some = (0..400).filter(|s| written(7, *s)).count();
        assert!((200..=280).contains(&some), "{some} of 400 slots written");
    }

    /// The words keep the canon (`LORE.md`, *Writing on the walls*; the
    /// lore-keeper's review): no day count (E.'s), no level, no rule, no way,
    /// no instruction; and every letter of them can be written
    #[test]
    fn the_words_keep_the_canon_and_can_be_written() {
        let pool = crate::core_props::tail::SCRAWL_POOL;
        for w in pool {
            let words: Vec<&str> = w.split(|c: char| !c.is_ascii_alphanumeric() && c != '\'').filter(|s| !s.is_empty()).collect();
            for never in
                ["DAY", "DAYS", "LEVEL", "FLOOR", "EXIT", "RUN", "FOLLOW", "TURN", "ALWAYS", "NEVER", "ALMOND", "PHONE", "DOOR", "DOORS"]
            {
                assert!(!words.contains(&never), "{w:?} says {never:?}");
            }
            for c in w.chars() {
                assert!(strokes(c).is_some(), "{w:?}: no stroke for {c:?}");
            }
        }
        let mut uniq: Vec<&&str> = pool.iter().collect();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), pool.len(), "a phrase twice in the pool");
    }

    /// Its lines slope one way, none steeper than the room left for it, so
    /// none runs into the next (the blind review found them crossing); a
    /// narrow letter takes less room than a wide one, and a word's space
    /// more than a letter's gap
    #[test]
    fn its_lines_slope_one_way_and_its_words_stay_apart() {
        for text in crate::core_props::tail::SCRAWL_POOL {
            for seed in 0..60 {
                let w = plan(text, Medium::Marker, seed, 0.5);
                let s0 = w.slope[0];
                assert!(w.slope.iter().all(|s| s.signum() == s0.signum()), "{text} {seed}: lines slope both ways {:?}", w.slope);
                assert!(w.slope.iter().all(|s| s.abs() >= 0.02 && s.abs().dtan() < STEEPEST), "{text} {seed}: {:?}", w.slope);
                let spread = w.slope.iter().map(|s| s.abs()).fold(0.0, f64::max) - w.slope.iter().map(|s| s.abs()).fold(1.0, f64::min);
                assert!(spread <= 0.02, "{text} {seed}: lines diverge {:?}", w.slope);
            }
        }
        assert!(advance('I') < advance('W') && advance('\'') < advance('I'));
        const { assert!(SPACE + GAP > 2.5 * GAP, "a word's space is no wider than a letter's gap") };
    }

    /// A sheet of the hand to look at (not a check): phrases in each medium,
    /// four seeds each, on a wall's colour, to `target/graffiti/sheet.png`
    /// (`cargo test -p game --lib a_sheet_of_the_writing -- --ignored`)
    #[test]
    #[ignore]
    fn a_sheet_of_the_writing() {
        let rows = [
            ("NOT AGAIN", Medium::Marker),
            ("NOT AGAIN", Medium::Pencil),
            ("I DON'T KNOW HOW LONG", Medium::Lipstick),
            ("WAIT FOR ME", Medium::Correction),
            ("HAPPY ANNIVERSARY JO", Medium::Blood),
            ("MOTHER WHERE ARE YOU", Medium::Ballpoint),
            ("SORRY SORRY SORRY", Medium::Scratch),
            ("WE WERE FOUR. NOW TWO", Medium::Marker),
        ];
        let (cw, ch, cols) = (420usize, 220usize, 4usize);
        let (sw, sh) = (cw * cols, ch * rows.len());
        let mut out = vec![0u8; sw * sh * 4];
        for (ri, (text, m)) in rows.iter().enumerate() {
            for col in 0..cols {
                let wr = plan(text, *m, 11 + col as u32 * 7919, if col == 3 { 0.1 } else { 0.5 });
                let tex = writing_tex(&wr);
                let img = tex.image();
                let img = img.borrow();
                let (w, h) = (img.width as usize, img.height as usize);
                let s = (cw as f64 / w as f64).min(ch as f64 / h as f64).min(1.0);
                let bg = if *m == Medium::Correction { [96.0, 88.0, 70.0] } else { [168.0, 156.0, 118.0] };
                for y in 0..ch {
                    for x in 0..cw {
                        let (u, v) = ((x as f64 / s) as usize, (y as f64 / s) as usize);
                        let px = if u < w && v < h { &img.data[(v * w + u) * 4..(v * w + u) * 4 + 4] } else { &[0, 0, 0, 0][..] };
                        let a = f64::from(px[3]) / 255.0;
                        let o = ((ri * ch + y) * sw + col * cw + x) * 4;
                        for k in 0..3 {
                            out[o + k] = (bg[k] * (1.0 - a) + f64::from(px[k]) * a).round() as u8;
                        }
                        out[o + 3] = 255;
                    }
                }
            }
        }
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/graffiti");
        std::fs::create_dir_all(dir).unwrap();
        engine::png::write_rgba(&format!("{dir}/sheet.png"), sw as u32, sh as u32, &out).unwrap();
    }

    #[test]
    fn standing_it_is_written_at_1_2_to_1_7_m() {
        let w = plan("HELP ME", Medium::Marker, 9, 0.5);
        assert!(!w.low);
        assert_eq!(height(&w, 1.1), 1.2);
        assert!((height(&w, 2.1) - 1.7).abs() < 1e-12);
    }
}

/// A piece of writing planned on a wall (by the core props), baked into the
/// wall when the walls are built (`bake`)
#[derive(Clone, Debug)]
pub struct Planned {
    pub writing: Writing,
    /// the wall spot it was planned at, and its middle's height
    pub at: Vec3,
    pub normal: Vec3,
    pub y: f64,
}

/// Along a face whose normal is `n`, the coordinate the walls' world-space
/// UVs run on (`Patch::ArchUV`, the shader's `wuv`), and the way it grows
fn along(n: &Vec3, x: f64, z: f64) -> f64 {
    if n.x.abs() > 0.5 {
        if n.x > 0.0 { -z } else { z }
    } else if n.z > 0.0 {
        x
    } else {
        -x
    }
}
fn along_dir(n: &Vec3) -> Vec3 {
    if n.x.abs() > 0.5 {
        Vec3::new(0.0, 0.0, if n.x > 0.0 { -1.0 } else { 1.0 })
    } else {
        Vec3::new(if n.z > 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0)
    }
}

/// The non-negative fraction of `x`
fn frac(x: f64) -> f64 {
    x - x.floor()
}

/// `img` (RGBA8, rows top-down) sampled bilinearly at texture coordinate
/// (u, v), wrapping, as the shader samples a map uploaded flipped (v up)
fn sample(data: &[u8], w: usize, h: usize, u: f64, v: f64) -> [f64; 4] {
    let x = frac(u) * w as f64 - 0.5;
    let y = (1.0 - frac(v)) * h as f64 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let at = |xi: f64, yi: f64, c: usize| {
        let (xi, yi) = ((xi as i64).rem_euclid(w as i64) as usize, (yi as i64).rem_euclid(h as i64) as usize);
        f64::from(data[(yi * w + xi) * 4 + c])
    };
    let mut out = [0.0; 4];
    for (c, o) in out.iter_mut().enumerate() {
        let a = at(x0, y0, c) * (1.0 - fx) + at(x0 + 1.0, y0, c) * fx;
        let b = at(x0, y0 + 1.0, c) * (1.0 - fx) + at(x0 + 1.0, y0 + 1.0, c) * fx;
        *o = a * (1.0 - fy) + b * fy;
    }
    out
}

/// The planned writing baked into the walls (from `build_walls`, with the
/// solids it draws and the windows' cuts): each on the straight wall piece
/// behind it, as a patch 1 mm off that face drawn with the piece's own
/// material (its light, its photographed detail, its wear), its texture the
/// wall's own surface there with the writing in it. Where the writing would
/// cross a window, an edge of its piece or a leaning wall, it is not written.
pub fn bake(g: &mut Game, s: &crate::level::archgen::Solids, cuts: &[[f64; 6]]) {
    let plans = std::mem::take(&mut g.wall_writing);
    let (gw, gh) = (g.w as f64, g.h as f64);
    for p in plans {
        let wr = &p.writing;
        let (pw, ph) = (wr.w / wr.ppm, wr.h / wr.ppm);
        let n = Vec3::new(p.normal.x.round(), 0.0, p.normal.z.round());
        if (n.x.abs() + n.z.abs() - 1.0).abs() > 1e-6 {
            continue;
        }
        let t = along_dir(&n);
        let (y0, y1) = (p.y - ph / 2.0, p.y + ph / 2.0);
        // the straight wall piece behind it: the first face along its normal
        // (a spot is 6 cm before its tile's edge, and a partition may stand
        // anywhere in its tile, as `settle_props` finds a hung thing's face),
        // under the whole of it
        let mut best: Option<(f64, String, f64)> = None;
        for b in 0..s.count {
            for (key, part) in g.wall_box_parts(s, b) {
                let wb =
                    [(part[0] - gw / 2.0) * CELL, (part[1] - gh / 2.0) * CELL, (part[2] - gw / 2.0) * CELL, (part[3] - gh / 2.0) * CELL];
                if part[4] > y0 || part[5] < y1 {
                    continue;
                }
                // the face toward the room, its distance behind the spot, and
                // the piece's extent along it
                let (face, behind, lo, hi, mid) = if n.x.abs() > 0.5 {
                    let f = if n.x > 0.0 { wb[2] } else { wb[0] };
                    (f, (p.at.x - f) * n.x, wb[1], wb[3], p.at.z)
                } else {
                    let f = if n.z > 0.0 { wb[3] } else { wb[1] };
                    (f, (p.at.z - f) * n.z, wb[0], wb[2], p.at.x)
                };
                if !(-0.05..=CELL + 0.1).contains(&behind) || mid - pw / 2.0 < lo - 1e-6 || mid + pw / 2.0 > hi + 1e-6 {
                    continue;
                }
                if best.as_ref().is_none_or(|b| behind < b.0) {
                    best = Some((behind, key, face));
                }
            }
        }
        let Some((_, key, face)) = best else { continue };
        // the face point under its middle
        let c = if n.x.abs() > 0.5 { Vec3::new(face, p.y, p.at.z) } else { Vec3::new(p.at.x, p.y, face) };
        // no window's opening under it
        let (a0, a1) = (c + t * (-pw / 2.0), c + t * (pw / 2.0));
        let (x_lo, x_hi, z_lo, z_hi) = (a0.x.min(a1.x) - 0.05, a0.x.max(a1.x) + 0.05, a0.z.min(a1.z) - 0.05, a0.z.max(a1.z) + 0.05);
        if cuts.iter().any(|k| k[0] < x_hi && k[3] > x_lo && k[2] < z_hi && k[5] > z_lo && k[1] < y1 && k[4] > y0) {
            continue;
        }
        let top = s_top(g, s, &key, &c, &n);
        let base = g.arch_mat(&key);
        let (map, sx, sy, wallpaper, detail) = {
            let m = base.m();
            let Some(map) = m.map.clone() else { continue };
            let engine::material::Patch::ArchUV { scale_x, scale_y, wallpaper, detail, .. } = &m.patch else { continue };
            (map, *scale_x, *scale_y, *wallpaper, detail.clone())
        };
        // the wall's own vertical scale, as the shader takes it
        let vsc = if wallpaper { sy.max(top) } else { sy };
        let (tw, th) = (wr.w as usize, wr.h as usize);
        let writing = writing_tex(wr);
        let al0 = along(&n, a0.x, a0.z);
        let (wall, wr_ppm) = (map.clone(), wr.ppm);
        let tex = crate::placers::kit::made_tex(tw as u32, th as u32, move || {
            let wimg = wall.image();
            let wi = wimg.borrow();
            let (ww, wh) = (wi.width as usize, wi.height as usize);
            let simg = writing.image();
            let si = simg.borrow();
            let mut out = vec![0u8; tw * th * 4];
            for cy in 0..th {
                for cx in 0..tw {
                    // this texel's world point: the patch's own UVs (along /
                    // its width, y / its height, wrapping) inside its rectangle
                    let u = (cx as f64 + 0.5) / tw as f64;
                    let v = 1.0 - (cy as f64 + 0.5) / th as f64;
                    let a = al0 + frac(u - frac(al0 / pw)) * pw;
                    let y = y0 + frac(v - frac(y0 / ph)) * ph;
                    let mut c = sample(&wi.data, ww, wh, a / sx, y / vsc);
                    // the writing over it (straight alpha)
                    let (lx, ly) = (((a - al0) * wr_ppm) as usize, ((y1 - y) * wr_ppm) as usize);
                    if lx < tw && ly < th {
                        let k = (ly * tw + lx) * 4;
                        let ia = f64::from(si.data[k + 3]) / 255.0;
                        for (ch, o) in c.iter_mut().enumerate().take(3) {
                            *o = *o * (1.0 - ia) + f64::from(si.data[k + ch]) * ia;
                        }
                    }
                    let k = (cy * tw + cx) * 4;
                    for ch in 0..3 {
                        out[k + ch] = c[ch].round().clamp(0.0, 255.0) as u8;
                    }
                    out[k + 3] = 255;
                }
            }
            out
        });
        let mat = base.duplicate();
        {
            let mut m = mat.m();
            m.map = Some(tex);
            m.patch = engine::material::Patch::ArchUV { scale_x: pw, scale_y: ph, wallpaper: false, detail, plane_shift: 0.0 };
        }
        let depth = 0.0016;
        let centre = c + n * (depth / 2.0 + 0.001);
        let size = if n.x.abs() > 0.5 { Vec3::new(depth, ph, pw) } else { Vec3::new(pw, ph, depth) };
        let geo = g.unit_box_geo.clone();
        let m4 = Mat4::compose(&centre, &Quat::IDENTITY, &size);
        if let Some(o) = g.add_instanced(&geo, &mat, &[m4], &|m: &Mat4| *m) {
            let mut u = o.user();
            u.placer = Some("place_props".into());
            u.s.insert("kind".into(), "graffiti".into());
        }
    }
}

/// The top of the wall piece of `key` whose face is at `c` (the top its
/// wallpaper runs to)
fn s_top(g: &Game, s: &crate::level::archgen::Solids, key: &str, c: &Vec3, n: &Vec3) -> f64 {
    let (gw, gh) = (g.w as f64, g.h as f64);
    let mut top: f64 = 0.0;
    for b in 0..s.count {
        for (k, part) in g.wall_box_parts(s, b) {
            if k != key {
                continue;
            }
            let wb = [(part[0] - gw / 2.0) * CELL, (part[1] - gh / 2.0) * CELL, (part[2] - gw / 2.0) * CELL, (part[3] - gh / 2.0) * CELL];
            let on = if n.x.abs() > 0.5 {
                (c.z >= wb[1] - 1e-6 && c.z <= wb[3] + 1e-6) && (c.x - if n.x > 0.0 { wb[2] } else { wb[0] }).abs() < 1e-6
            } else {
                (c.x >= wb[0] - 1e-6 && c.x <= wb[2] + 1e-6) && (c.z - if n.z > 0.0 { wb[3] } else { wb[1] }).abs() < 1e-6
            };
            if on && part[4] <= c.y && part[5] >= c.y {
                top = top.max(part[5]);
            }
        }
    }
    top
}
