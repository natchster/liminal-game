//! The clue that a cutscene has your hands (`docs/CUTSCENES.md`, design
//! 2): black bars that slide in from the top and the bottom of the view,
//! the cutscene's own veil, and, once a key is pressed, the way out in the
//! lower bar ("Hold E to skip", the key as bound), its hold drawn as a
//! thin line that fills. The bars belong only to a cutscene: play is never
//! letterboxed. Each bar is one black canvas painted once for the view's
//! size and slid by its rect, so a slide repaints nothing.
use crate::base::game::Game;
use crate::cutscene::play::{Playing, SKIP_HOLD};
use crate::cutscene::Skip;
use crate::ui::draw::{col, css_color, Frame, Painter, TextBlock, Ws};
use crate::ui::faces::Font;
use crate::ui::model::LayerBlend;

/// Each bar's height, a share of the view's (with both in, a 16:10 view
/// is about 2.05:1; a 16:9 one about 2.3:1, near the cinema's 2.35:1)
pub const BAR_FRAC: f64 = 0.11;

/// The cutscene's veil, then its bars and the way out, over the HUD
pub(crate) fn paint_cutscene(f: &mut Frame, g: &Game) {
    // the veil: its fades, kept after it ends where it says (a death's
    // black behind its card, a far skip opening on your eyes)
    let v = &g.cut_veil;
    if v.opacity > 0.0005 {
        if let Some(mut c) = css_color(&v.colour) {
            c[3] *= v.opacity.min(1.0);
            f.fill(c, LayerBlend::Normal);
        }
    }
    let Some(p) = g.cut.as_ref() else { return };
    let k = p.bars();
    if k <= 0.0005 {
        return;
    }
    let (w, h) = (f.w, f.h);
    let bar = (h * BAR_FRAC).round();
    for (name, top) in [("cut-bar-top", true), ("cut-bar-bottom", false)] {
        let (canvas, id, ver, _) = f.boxed(name, &format!("{w}|{bar}"), |dpr| {
            let mut pt = Painter::new(0.0, 0.0, w, bar, dpr);
            pt.rect(0.0, 0.0, w, bar, [0.0, 0.0, 0.0, 1.0]);
            pt
        });
        // in from outside the view
        let y = if top { -bar * (1.0 - k) } else { h - bar * k };
        let rect = [0.0, (y / h) as f32, 1.0, (bar / h) as f32];
        f.push_canvas(name, canvas, id, ver, rect, 1.0, LayerBlend::Normal, false);
    }
    paint_way_out(f, g, p, h - bar * k, bar);
}

/// "Hold E to skip" at the right of the lower bar, shown a while after a
/// key; the hold fills a line under it
fn paint_way_out(f: &mut Frame, g: &Game, p: &Playing, bar_top: f64, bar: f64) {
    if p.c.skip != Skip::Hold || (p.hint <= 0.0 && p.skip_held <= 0.0) {
        return;
    }
    let key =
        g.settings.keys(crate::session::input::Act::Interact).first().map(|k| crate::session::input::key_label(k)).unwrap_or_default();
    let words = format!("Hold {key} to skip");
    let held = (p.skip_held / SKIP_HOLD).clamp(0.0, 1.0);
    // in quickly, out over its last half second; brighter while held
    let shown = (p.hint / 0.5).min(1.0).max(if p.skip_held > 0.0 { 1.0 } else { 0.0 });
    let op = shown * (0.55 + 0.4 * held.min(1.0).max(if p.skip_down { 0.4 } else { 0.0 }));
    let ink = col(crate::ui::screens::INK);
    let tb = TextBlock::plain(&words, ink, &[], 0.0, Font::sans(13.0), Ws::NoWrap, None, None);
    let (w, tw, th) = (f.w, tb.width(), tb.height());
    let x = w - crate::ui::screens::margin(w) - tw;
    let y = bar_top + (bar - th) / 2.0 - 3.0;
    // (the hold, in thirtieths: a repaint a step, not a frame)
    let steps = (held * 30.0).round() / 30.0;
    let key_s = format!("{words}|{x}|{y}|{steps}");
    f.layer("cut-way-out", &key_s, op, move |dpr| {
        let mut pt = Painter::new(x - 2.0, y - 2.0, tw + 4.0, th + 10.0, dpr);
        pt.text(&tb, x, y, tw, "left", 1.0);
        if steps > 0.0 {
            pt.rect(x, y + th + 3.0, tw * steps, 1.0, [ink[0], ink[1], ink[2], 0.9]);
        }
        pt
    });
}
