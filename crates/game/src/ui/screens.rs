//! Painting the menus (`ui/menu.rs`): no panel, no box, no capitals. A
//! shade darkens the left and the bottom of the view so the words read;
//! the title and the list sit on a left margin at the lower left, in the
//! building's voice (Helvetica Neue), and her last line in her hand. The
//! chosen line is bright with a short rule before it; the others are dim.
use crate::base::game::Game;
use crate::ui::draw::{col, rgba, Frame, Painter, Rgba, TextBlock, Ws};
use crate::ui::faces::Font;
use crate::ui::menu::Screen;
use crate::ui::model::LayerBlend;

/// The words' colour: a warm paper white
pub(crate) const INK: &str = "#e2dabd";
/// One line of a list (CSS px)
const LINE_H: f64 = 30.0;

/// The menu's left margin for a view `w` wide
pub(crate) fn margin(w: f64) -> f64 {
    (w * 0.065).max(40.0).round()
}

/// The shade under the words: dark at the left edge and along the bottom,
/// clear over the middle of the view (a small canvas, stretched smooth)
pub(crate) fn paint_shade(f: &mut Frame, strength: f64) {
    let (w, h) = (f.w, f.h);
    let key = format!("shade|{strength}");
    let (canvas, id, ver, rect) = f.boxed("menu-shade", &key, move |_| {
        let mut p = Painter::new(0.0, 0.0, w, h, 128.0 / w);
        p.c.set_transform(128.0 / w, 0.0, 0.0, 128.0 / w, 0.0, 0.0);
        let a = |x: f64| format!("rgba(5,4,2,{})", x * strength);
        let g = p.c.create_linear_gradient(0.0, 0.0, w * 0.62, 0.0);
        g.add_color_stop(0.0, &a(0.86));
        g.add_color_stop(0.45, &a(0.5));
        g.add_color_stop(1.0, &a(0.0));
        p.c.set_fill_style(&g);
        p.c.fill_rect(0.0, 0.0, w, h);
        let g = p.c.create_linear_gradient(0.0, h * 0.5, 0.0, h);
        g.add_color_stop(0.0, &a(0.0));
        g.add_color_stop(1.0, &a(0.55));
        p.c.set_fill_style(&g);
        p.c.fill_rect(0.0, 0.0, w, h);
        p
    });
    f.push_canvas("menu-shade", canvas, id, ver, rect, 1.0, LayerBlend::Normal, true);
}

fn line(text: &str, font: Font, c: Rgba, max_w: Option<f64>) -> TextBlock {
    TextBlock::plain(text, c, &[], 0.0, font, if max_w.is_some() { Ws::Normal } else { Ws::NoWrap }, max_w, None)
}

/// The open list (title, confirmations), topmost
pub(crate) fn paint_menu(f: &mut Frame, g: &Game) {
    let m = &g.menu;
    let Some(screen) = g.live_menu() else { return };
    let (w, h) = (f.w, f.h);
    paint_shade(f, 1.0);
    let ink = col(INK);
    let x0 = margin(w);
    let bottom = h - (h * 0.11).max(44.0);
    // the list, from the bottom up
    let items: Vec<(TextBlock, TextBlock)> =
        m.items.iter().map(|i| (line(&i.label, Font::sans(16.0), ink, None), line(&i.detail, Font::sans(12.0), ink, None))).collect();
    let list_y = bottom - items.len() as f64 * LINE_H;
    // above it: the title and her line, or a question and its note
    let max_w = (w * 0.42).clamp(280.0, 520.0);
    let (head, sub) = match screen {
        Screen::Title => (line("The Backrooms", Font::sans_light(40.0), ink, None), line(&m.her_line, Font::hand(17.0), ink, Some(max_w))),
        _ => (line(&m.heading, Font::sans(22.0), ink, Some(max_w)), line(&m.note, Font::sans(13.0), ink, Some(max_w))),
    };
    let sub_y = list_y - 26.0 - sub.height();
    let head_y = sub_y - if sub.is_empty() { 0.0 } else { 10.0 } - head.height();
    // click targets: each line, a little wider than its words
    let rects: Vec<[f64; 4]> = items
        .iter()
        .enumerate()
        .map(|(k, (l, d))| {
            let wd = l.width() + if d.is_empty() { 0.0 } else { 10.0 + d.width() };
            [x0 - 28.0, list_y + k as f64 * LINE_H, wd + 56.0, LINE_H]
        })
        .collect();
    let hits: Vec<(&'static str, [f64; 4], bool)> = m.items.iter().zip(rects.iter()).map(|(i, r)| (i.id, *r, true)).collect();
    f.ui.hits.splice(0..0, hits);
    let sel = m.sel;
    let key = format!("{screen:?}|{:?}|{}|{}|{}|{sel}|{w}|{h}", m.items, m.heading, m.note, m.her_line);
    let top = head_y - 12.0;
    f.layer("menu", &key, 1.0, move |dpr| {
        let mut p = Painter::new(0.0, top, w, h - top, dpr);
        p.text(&head, x0, head_y, head.width(), "left", 0.94);
        p.text(&sub, x0, sub_y, max_w, "left", 0.62);
        for (k, (l, d)) in items.iter().enumerate() {
            let y = list_y + k as f64 * LINE_H + (LINE_H - l.height()) / 2.0;
            let on = k == sel;
            p.text(l, x0, y, l.width(), "left", if on { 1.0 } else { 0.48 });
            if !d.is_empty() {
                p.text(d, x0 + l.width() + 10.0, y + 3.0, d.width(), "left", if on { 0.62 } else { 0.32 });
            }
            if on {
                // a short pencil rule before the chosen line
                let ry = y + l.height() * 0.55;
                p.rect(x0 - 26.0, ry, 16.0, 1.0, rgba(226.0, 218.0, 189.0, 0.85));
            }
        }
        p
    });
}
