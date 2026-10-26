//! The HUD's three voices (`ui/faces.rs`): every face is the one asked for
//! (CoreText substitutes a missing face silently), a laid-out line is as
//! wide as CoreText draws it, and `crates/target/faces.png` shows them.
use game::ui::draw::{Painter, TextBlock, Ws};
use game::ui::faces::{line_width, resolved_name, Face, Font};

fn all() -> Vec<Font> {
    let mut v = vec![];
    for f in [Font::new(12.0), Font::sans(12.0), Font::sans_light(12.0), Font::hand(12.0)] {
        v.extend([f, f.bold(), f.italic(), f.bold().italic()]);
    }
    v
}

#[test]
fn every_face_is_the_one_asked_for() {
    for f in all() {
        assert_eq!(
            resolved_name(f),
            f.ps_name(),
            "{:?} {}{} resolved to another face",
            f.face,
            if f.bold { "bold " } else { "" },
            if f.italic { "italic" } else { "" }
        );
    }
}

#[test]
fn a_laid_out_line_is_as_wide_as_coretext_draws_it() {
    let lines = [
        "Continue — level 4",
        "New descent",
        "Take her page",
        "\u{201c}Folded twice, so I know it\u{2019}s mine.\u{201d}",
        "The door was where the hum was loudest. I left you the water.",
        "AVAWAY fi fl ffi Tö 1/2 ×3",
    ];
    for face in [Font::sans(1.0), Font::sans_light(1.0), Font::hand(1.0), Font::sans(1.0).bold(), Font::hand(1.0).italic()] {
        for size in [11.0, 15.0, 34.0] {
            let font = Font { size, ..face };
            for text in lines {
                let tb = TextBlock::plain(text, [1.0; 4], &[], 0.0, font, Ws::NoWrap, None, None);
                let ct = line_width(font, text, size, 0.0);
                assert!(
                    (tb.width() - ct).abs() < 0.05,
                    "{:?} {size} px: \"{text}\" lays out {:.3} px wide, CoreText draws it {ct:.3}",
                    face.face,
                    tb.width()
                );
                // spaced, each glyph moves on by the spacing (and is drawn where it was laid out)
                let spaced = TextBlock::plain(text, [1.0; 4], &[], 1.5, font, Ws::NoWrap, None, None);
                let n = text.chars().count() as f64;
                assert!((spaced.width() - (tb.width() + 1.5 * n)).abs() < 1e-6, "{text}: spacing is not per glyph");
                assert!(!font.draws_runs('a', 1.5) && font.draws_runs('a', 0.0));
            }
        }
    }
    // Courier New keeps the fixed metrics the HUD frames were laid out with
    let tb = TextBlock::plain("LEVEL 3", [1.0; 4], &[], 0.0, Font::new(15.0), Ws::NoWrap, None, None);
    assert!((tb.width() - 7.0 * 0.600109375 * 15.0).abs() < 1e-9);
    assert_eq!(Font::new(15.0).face, Face::Typed);
}

#[test]
fn the_voices_side_by_side() {
    let (w, h, dpr) = (720.0, 300.0, 2.0);
    let mut p = Painter::new(0.0, 0.0, w, h, dpr);
    p.rect(0.0, 0.0, w, h, [0.06, 0.055, 0.04, 1.0]);
    p.rect(360.0, 150.0, 360.0, 150.0, [0.90, 0.85, 0.72, 1.0]);
    let ink = [0.86, 0.82, 0.66, 1.0];
    let pencil = [0.13, 0.11, 0.09, 1.0];
    let put = |p: &mut Painter, t: &str, f: Font, c: [f64; 4], x: f64, y: f64| {
        let tb = TextBlock::plain(t, c, &[], 0.0, f, Ws::Normal, Some(330.0), None);
        p.text(&tb, x, y, 330.0, "left", 1.0);
    };
    put(&mut p, "The Backrooms", Font::sans_light(34.0), ink, 24.0, 24.0);
    put(&mut p, "Continue — level 4", Font::sans(15.0), ink, 24.0, 84.0);
    put(&mut p, "New descent", Font::sans(15.0), ink, 24.0, 108.0);
    put(&mut p, "E  Take her page", Font::sans(13.0).bold(), ink, 24.0, 160.0);
    put(&mut p, "Level 3 — the walls are thinner here", Font::sans(13.0), ink, 24.0, 190.0);
    put(&mut p, "Elias, Day 47", Font::new(13.0).bold(), ink, 380.0, 24.0);
    put(
        &mut p,
        "Made it to level three today. The plants down here are wet even where there is no water.",
        Font::new(12.0),
        ink,
        380.0,
        48.0,
    );
    put(&mut p, "The door was where the hum was loudest. I left you the water. — M.", Font::hand(17.0), pencil, 380.0, 170.0);
    let img = p.c.get_image_data(0, 0, w * dpr, h * dpr);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
    std::fs::create_dir_all(&dir).unwrap();
    engine::png::write_rgba(dir.join("faces.png").to_str().unwrap(), (w * dpr) as u32, (h * dpr) as u32, &img.data).unwrap();
    assert!(img.data.chunks_exact(4).any(|px| px[0] > 200 && px[3] > 200), "nothing drawn");
}
