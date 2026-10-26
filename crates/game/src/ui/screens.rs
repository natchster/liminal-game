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

/// The open list, topmost
pub(crate) fn paint_menu(f: &mut Frame, g: &Game) {
    let m = &g.menu;
    let Some(screen) = g.live_menu() else { return };
    let (w, h) = (f.w, f.h);
    // over the frozen frame the place stays, darker
    if g.state == "paused" {
        f.fill(rgba(4.0, 3.0, 2.0, 0.38), LayerBlend::Normal);
    }
    paint_shade(f, 1.0);
    let ink = col(INK);
    let x0 = margin(w);
    let bottom = h - (h * 0.11).max(44.0);
    let valued = matches!(screen, Screen::Settings | Screen::Controls);
    let n = m.items.len().max(1) as f64;
    let line_h = if valued { (h * 0.62 / n).clamp(20.0, LINE_H) } else { LINE_H };
    let size = if line_h < 26.0 { 14.0 } else { 16.0 };
    // the list, from the bottom up
    let items: Vec<(TextBlock, TextBlock)> = m
        .items
        .iter()
        .map(|i| {
            (line(&i.label, Font::sans(size), ink, None), line(&i.detail, Font::sans(if valued { size - 1.0 } else { 12.0 }), ink, None))
        })
        .collect();
    let list_y = bottom - items.len() as f64 * line_h;
    // a setting's value sits in a column after the longest label
    let vcol = items.iter().map(|(l, _)| l.width()).fold(0.0, f64::max) + 36.0;
    // above it: the title and her line, or a heading and its note
    let max_w = (w * 0.42).clamp(280.0, 520.0);
    let (head, sub) = match screen {
        Screen::Title => (line("The Backrooms", Font::sans_light(40.0), ink, None), line(&m.her_line, Font::hand(17.0), ink, Some(max_w))),
        Screen::Pause => (line(&m.heading, Font::sans_light(32.0), ink, None), line(&m.note, Font::sans(13.0).italic(), ink, Some(max_w))),
        _ => (line(&m.heading, Font::sans(22.0), ink, Some(max_w)), line(&m.note, Font::sans(13.0), ink, Some(max_w))),
    };
    let sub_y = list_y - 26.0 - sub.height();
    let head_y = sub_y - if sub.is_empty() { 0.0 } else { 10.0 } - head.height();
    // click targets: each line, a little wider than its words
    let rects: Vec<[f64; 4]> = items
        .iter()
        .enumerate()
        .map(|(k, (l, d))| {
            let wd = if valued && !d.is_empty() { vcol + d.width() } else { l.width() + if d.is_empty() { 0.0 } else { 10.0 + d.width() } };
            [x0 - 28.0, list_y + k as f64 * line_h, wd + 56.0, line_h]
        })
        .collect();
    let hits: Vec<(&'static str, [f64; 4], bool)> = m.items.iter().zip(rects.iter()).map(|(i, r)| (i.id, *r, true)).collect();
    f.ui.hits.splice(0..0, hits);
    let sel = m.sel;
    // the stepped settings show their arrows on the chosen line
    let stepped = valued && screen == Screen::Settings;
    let key = format!("{screen:?}|{:?}|{}|{}|{}|{sel}|{w}|{h}|{line_h}", m.items, m.heading, m.note, m.her_line);
    let top = head_y - 12.0;
    f.layer("menu", &key, 1.0, move |dpr| {
        let mut p = Painter::new(0.0, top, w, h - top, dpr);
        p.text(&head, x0, head_y, head.width(), "left", 0.94);
        p.text(&sub, x0, sub_y, max_w, "left", 0.62);
        for (k, (l, d)) in items.iter().enumerate() {
            let y = list_y + k as f64 * line_h + (line_h - l.height()) / 2.0;
            let on = k == sel;
            p.text(l, x0, y, l.width(), "left", if on { 1.0 } else { 0.48 });
            if !d.is_empty() {
                let (dx, dy) = if valued { (x0 + vcol, y + (l.height() - d.height()) / 2.0) } else { (x0 + l.width() + 10.0, y + 3.0) };
                p.text(
                    d,
                    dx,
                    dy,
                    d.width(),
                    "left",
                    if on {
                        if valued {
                            0.95
                        } else {
                            0.62
                        }
                    } else if valued {
                        0.55
                    } else {
                        0.32
                    },
                );
                if on && stepped {
                    let a = line("‹", Font::sans(size), ink, None);
                    let b = line("›", Font::sans(size), ink, None);
                    p.text(&a, dx - 14.0, dy, a.width(), "left", 0.6);
                    p.text(&b, dx + d.width() + 6.0, dy, b.width(), "left", 0.6);
                }
            }
            if on {
                // a short pencil rule before the chosen line
                let ry = y + l.height() * 0.55;
                p.rect(x0 - 26.0, ry, 16.0, 1.0, rgba(226.0, 218.0, 189.0, 0.85));
            }
        }
        p
    });
    // paused: where you are, quietly (the seed is for telling someone about a floor)
    if screen == Screen::Pause {
        let t = format!("Level {}  ·  seed {:08x}", g.level_num, g.run_seed);
        let tb = line(&t, Font::sans(11.0), ink, None);
        let (tw, th) = (tb.width(), tb.height());
        f.layer("menu-where", &t, 1.0, move |dpr| {
            let mut p = Painter::new(w - x0 - tw - 4.0, bottom - th - 4.0, tw + 8.0, th + 8.0, dpr);
            p.text(&tb, w - x0 - tw, bottom - th, tw, "left", 0.38);
            p
        });
    }
    // Brightness chosen: a mark that should be barely seen on black
    if screen == Screen::Settings && m.items.get(sel).map(|i| i.id) == Some("set:brightness") {
        let (bw, bh) = (200.0, 130.0);
        let (bx, by) = ((w * 0.66).min(w - bw - x0), (h - bh) / 2.0);
        let hint = line("Turn it until the mark is only just seen.", Font::sans(12.0), ink, Some(bw + 40.0));
        f.layer("menu-mark", "mark", 1.0, move |dpr| {
            let mut p = Painter::new(bx, by, bw + 40.0, bh + 40.0, dpr);
            p.rect(bx, by, bw, bh, [0.0, 0.0, 0.0, 1.0]);
            // the exit sign's shape, a hair above black
            let c = rgba(5.0, 5.0, 4.0, 1.0);
            p.rect(bx + bw / 2.0 - 34.0, by + bh / 2.0 - 16.0, 68.0, 26.0, c);
            p.text(&hint, bx, by + bh + 10.0, bw + 40.0, "left", 0.6);
            p
        });
    }
}

/// Settings' brightness, over everything: a lift (screen) or a dim (multiply)
pub(crate) fn paint_brightness(f: &mut Frame, g: &Game) {
    let k = g.settings.brightness as f64;
    if k > 0.0 {
        let c = k * 0.025;
        f.fill([c, c, c, 1.0], LayerBlend::Screen);
    } else if k < 0.0 {
        let c = 1.0 + k * 0.05;
        f.fill([c, c, c, 1.0], LayerBlend::Multiply);
    }
}
