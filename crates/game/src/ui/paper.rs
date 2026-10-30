//! The notebook as paper in your hands (`docs/UI.md`, design 7): a page low
//! on the screen, so the way ahead stays in view and the mouse still turns
//! you. Her pages are on ruled paper in her hand; the torn notes are typed
//! on a greyer photocopy with a torn top; the index, the medals and the
//! epilogue are on ruled paper in the building's voice. No dimming of the
//! world, no buttons: the arrows, the wheel or a click turn the pages, and
//! the notebook key closes it.
use crate::base::game::Game;
use crate::session::input::{key_label, Act};
use crate::ui::draw::{base_run, col, css_rgba, rgba, runs_of, Frame, Painter, Rgba, TextBlock, Ws};
use crate::ui::faces::Font;

/// Which kind of page is open
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    /// one of hers (or the torn edge where hers was)
    Hers,
    /// a torn note someone typed
    Typed,
    /// the index, the medals, the epilogue
    Index,
    /// what you took (`secrets`), in your own hand, in pen
    Mine,
}

pub(crate) fn paint_notebook(f: &mut Frame, g: &Game) {
    let d = &g.ui;
    let ids = g.journal_entries_in_order();
    let cur = ids.get(g.journal_cursor as usize).copied();
    let kind = match cur {
        Some(c) if c <= -100 => Kind::Hers,
        Some(c) if c >= 0 => Kind::Typed,
        Some(-4) => Kind::Mine,
        _ => Kind::Index,
    };
    let (w, h) = (f.w, f.h);
    let pw = (w * 0.46).clamp(380.0, 600.0);
    let (pad, top) = (34.0, 30.0);
    let iw = pw - 2.0 * pad;
    let ink: Rgba = match kind {
        Kind::Typed => col("#24211c"),
        _ => col("#2a2620"),
    };
    let pencil = col("#4a4a52");
    // the page's words in its voice
    let title_font = match kind {
        Kind::Hers => Font::sans(12.0),
        Kind::Typed => Font::new(14.0).bold(),
        Kind::Index | Kind::Mine => Font::sans(13.0).bold(),
    };
    // (typed paper keeps its capitals; her pages and the index are written)
    let title_text =
        if kind == Kind::Typed { d.journal_entry_title.text.clone() } else { crate::ui::rest::calm(&d.journal_entry_title.text) };
    let title =
        TextBlock::plain(&title_text, if kind == Kind::Hers { pencil } else { ink }, &[], 0.0, title_font, Ws::Normal, Some(iw), None);
    let (body_font, lh) = match kind {
        Kind::Hers | Kind::Mine => (Font::hand(19.0), 28.0),
        Kind::Typed => (Font::new(13.5), 13.5 * 1.65),
        Kind::Index => (Font::sans(14.0), 14.0 * 1.6),
    };
    let body = TextBlock::new(runs_of(&d.journal_entry_body, &base_run(ink, vec![], 0.0)), body_font, Ws::PreLine, Some(iw), Some(lh));
    let torn_edge = kind == Kind::Hers && d.journal_entry_body.text.starts_with("A torn edge");
    // the page is as tall as its words (and never more than the lower two thirds)
    let ph = (top + title.height() + 18.0 + body.height() + 58.0).clamp(240.0, (h * 0.66).max(240.0));
    let (px, py) = ((w - pw) / 2.0, h - ph - (h * 0.035).max(14.0));
    let n = ids.len().max(1);
    let count =
        TextBlock::plain(&format!("{} / {}", g.journal_cursor as usize + 1, n), pencil, &[], 0.0, Font::sans(11.0), Ws::NoWrap, None, None);
    let close = g.settings.keys(Act::Journal).first().map(|k| key_label(k)).unwrap_or_default();
    let sp = "\u{a0}\u{a0}";
    let hint = TextBlock::plain(
        &format!("←{sp}→{sp}turn{sp}{sp}{sp}{close}{sp}close"),
        pencil,
        &[],
        0.0,
        Font::sans(11.0),
        Ws::NoWrap,
        None,
        None,
    );
    let key = format!("{kind:?}|{:?}|{:?}|{}|{w}|{h}|{torn_edge}", title.width(), d.journal_entry_body.version, d.journal_entry_title.text);
    let key = format!("{key}|{}|{close}", g.journal_cursor);
    f.layer("journal", &key, 1.0, move |dpr| {
        let sm = 26.0;
        let mut p = Painter::new(px - sm, py - sm, pw + 2.0 * sm, ph + 2.0 * sm, dpr);
        // a soft shadow under the page
        for k in 0..6 {
            let e = 3.0 + k as f64 * 3.0;
            p.rect(px - e + 4.0, py - e + 8.0, pw + 2.0 * e - 8.0, ph + 2.0 * e, [0.0, 0.0, 0.0, 0.07]);
        }
        let paper = match kind {
            Kind::Typed => rgba(214.0, 210.0, 199.0, 0.97),
            _ => rgba(233.0, 224.0, 200.0, 0.97),
        };
        p.tr();
        // the sheet; a typed note's top edge is torn
        p.c.set_fill_style(css_rgba(paper).as_str());
        p.c.begin_path();
        if kind == Kind::Typed {
            p.c.move_to(px, py + 6.0);
            let mut x = px;
            let mut k = 0;
            while x < px + pw {
                let y = py + [3.0, 7.0, 1.0, 5.0, 8.0, 2.0, 6.0][k % 7];
                x = (x + 9.0 + (k % 3) as f64 * 4.0).min(px + pw);
                p.c.line_to(x, y);
                k += 1;
            }
        } else {
            p.c.move_to(px, py);
            p.c.line_to(px + pw, py);
        }
        p.c.line_to(px + pw, py + ph);
        p.c.line_to(px, py + ph);
        p.c.close_path();
        p.c.fill();
        // ruled lines (and a margin) on her paper, and yours
        if kind == Kind::Hers || kind == Kind::Mine {
            let first = py + top + title.height() + 18.0;
            let mut y = first + lh - 6.0;
            while y < py + ph - 30.0 {
                p.rect(px + 12.0, y, pw - 24.0, 1.0, rgba(110.0, 130.0, 160.0, 0.22));
                y += lh;
            }
            p.rect(px + pad - 10.0, py + 8.0, 1.0, ph - 16.0, rgba(170.0, 80.0, 70.0, 0.25));
        }
        p.text(&title, px + pad, py + top, iw, "left", if kind == Kind::Hers { 0.7 } else { 0.9 });
        let by = py + top + title.height() + 18.0;
        p.text(&body, px + pad, by, iw, "left", if torn_edge { 0.45 } else { 0.92 });
        p.text(&count, px + pw - pad - count.width(), py + ph - 24.0, count.width(), "left", 0.5);
        p.text(&hint, px + pad, py + ph - 24.0, hint.width(), "left", 0.38);
        p
    });
}
