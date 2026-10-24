//! Builds every texture of `visuals::textures` and `visuals::surfaces`, checks
//! sizes and writes a contact sheet to target/texture-sheet.png (one PNG per
//! section in target/texture-sheet/) for visual inspection.

use engine::canvas::{Canvas, ImageData};
use engine::texture::{ColorSpace, Tex, Wrap};
use game::base::rng::{native_rnd, rnd, seeded_draws, with_native, with_seed};
use game::visuals::surfaces::*;
use game::visuals::textures::*;

struct Item {
    label: String,
    tex: Tex,
}
struct Section {
    name: &'static str,
    cw: f64,
    ch: f64,
    cols: usize,
    items: Vec<Item>,
}

fn item(label: impl Into<String>, tex: Tex) -> Item {
    Item { label: label.into(), tex }
}

const PAD: f64 = 4.0;
const TITLE_H: f64 = 24.0;
const LABEL_H: f64 = 16.0;

fn tex_canvas(t: &Tex) -> Canvas {
    let img = t.image();
    let im = img.borrow();
    let mut c = Canvas::new(im.width, im.height);
    c.put_image_data(&ImageData { width: im.width, height: im.height, data: im.data.clone() }, 0, 0);
    c
}

/// One section of the contact sheet: a title row, then a grid of labelled cells.
fn render_section(s: &Section) -> Canvas {
    let n = s.items.len();
    let rows = n.div_ceil(s.cols);
    let w = PAD + s.cols as f64 * (s.cw + PAD);
    let h = TITLE_H + rows as f64 * (s.ch + LABEL_H + PAD) + PAD;
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#202020");
    c.fill_rect(0, 0, w, h);
    c.set_fill_style("#ffffff");
    c.set_font("bold 14px sans-serif");
    c.fill_text(s.name, PAD, 17);
    for (i, it) in s.items.iter().enumerate() {
        let (col, row) = ((i % s.cols) as f64, (i / s.cols) as f64);
        let x0 = PAD + col * (s.cw + PAD);
        let y0 = TITLE_H + row * (s.ch + LABEL_H + PAD);
        // checkerboard so transparent textures read
        c.set_fill_style("#7a7a7a");
        c.fill_rect(x0, y0, s.cw, s.ch);
        c.set_fill_style("#9a9a9a");
        let mut yy = 0.0;
        while yy < s.ch {
            let mut xx = 0.0;
            while xx < s.cw {
                if ((xx / 16.0) as i64 + (yy / 16.0) as i64) % 2 == 1 {
                    c.fill_rect(x0 + xx, y0 + yy, 16f64.min(s.cw - xx), 16f64.min(s.ch - yy));
                }
                xx += 16.0;
            }
            yy += 16.0;
        }
        let (tw, th) = (it.tex.width() as f64, it.tex.height() as f64);
        let k = (s.cw / tw).min(s.ch / th);
        let (dw, dh) = (tw * k, th * k);
        let src = tex_canvas(&it.tex);
        c.draw_image_scaled(&src, x0 + (s.cw - dw) / 2.0, y0 + (s.ch - dh) / 2.0, dw, dh);
        c.set_fill_style("#dddddd");
        c.set_font("11px sans-serif");
        c.fill_text(&it.label, x0 + 2.0, y0 + s.ch + 12.0);
    }
    c
}

fn write_png(path: &str, c: &mut Canvas) {
    let (w, h) = (c.width() as u32, c.height() as u32);
    let img = c.get_image_data(0, 0, w, h);
    engine::png::write_rgba(path, w, h, &img.data).expect("write png");
}

fn assert_size(t: &Tex, w: u32, h: u32, what: &str) {
    assert_eq!((t.width(), t.height()), (w, h), "{what}: size");
    let img = t.image();
    assert_eq!(img.borrow().data.len(), (w * h * 4) as usize, "{what}: data length");
}

fn hand(color: &str, thickness: f64, jitter: f64, gap: f64, wobble: f64, double_stroke: bool, drip: bool, w: f64, h: f64) -> HandOpts {
    HandOpts { color: color.into(), thickness, jitter, gap, wobble, double_stroke, drip, w, h, ..Default::default() }
}
fn txt(w: f64, h: f64, font: &str, color: &str, glow: f64) -> TextTexOpts {
    TextTexOpts { w, h, font: font.into(), color: color.into(), glow, ..Default::default() }
}

#[test]
fn textures_and_surfaces() {
    // ---- startup textures; building them must not touch a seeded stream
    let mut tx = with_seed(7, || {
        let before = seeded_draws();
        let a = rnd();
        let t = Textures::new();
        assert_eq!(seeded_draws(), before + 1, "Textures::new drew from the seeded stream");
        let _ = a;
        t
    });
    assert_eq!(WALL_PALETTES.len(), 21);
    assert_eq!(PAL_MONO_YELLOW, WALL_PALETTES.len() - 1);
    assert_eq!(WALL_PALETTES[PAL_MONO_YELLOW].name, "mono-yellow");
    assert_eq!(WALL_PATTERNS.len(), 23);
    assert_eq!(tx.wallpaper_variants.len(), 21 * 4 + 2);
    for (i, v) in tx.wallpaper_variants.iter().enumerate() {
        assert_size(&v.tex, 512, 512, "wallpaper variant");
        if i < 84 {
            assert_eq!(v.palette_idx, i / 4);
            assert_eq!(v.grime_lvl, (i % 4) as i64);
            assert!(!v.moss);
            assert!(SOFT_PATTERN_POOL.iter().any(|p| p.0 == v.pattern));
        } else {
            assert!(v.moss);
            assert_eq!(v.grime_lvl, 3);
            assert!(v.palette_idx < 21);
        }
        let t = v.tex.r();
        assert_eq!((t.wrap_s, t.wrap_t, t.color_space, t.anisotropy), (Wrap::Repeat, Wrap::Repeat, ColorSpace::Srgb, 4));
    }
    assert_eq!(tx.wallpaper_tex.id(), tx.wallpaper_variants[0].tex.id());
    assert_eq!(tx.carpet_variants.len(), 5);
    for t in &tx.carpet_variants {
        assert_size(t, 512, 512, "carpet");
    }
    assert_eq!(tx.carpet_tex.id(), tx.carpet_variants[2].id());
    assert_eq!(tx.ceiling_variants.len(), 5);
    for t in &tx.ceiling_variants {
        assert_size(t, 512, 512, "ceiling");
    }
    assert_eq!(tx.ceiling_tex.id(), tx.ceiling_variants[0].id());
    assert_size(&tx.light_tex, 128, 64, "lightTex");
    assert_eq!(tx.light_tex.r().wrap_s, Wrap::Clamp);
    assert_size(&tx.dust_sprite, 64, 64, "dustSprite");
    assert_size(&tx.face_tex, 256, 256, "faceTex");
    assert_size(&tx.static_tex, 96, 64, "staticTex");
    assert_eq!(tx.static_tex.r().color_space, ColorSpace::Srgb);
    {
        // static: opaque grey noise; a redraw changes it and bumps the version
        let img = tx.static_tex.image();
        let (v0, d0) = {
            let b = img.borrow();
            (b.version, b.data.clone())
        };
        assert!(d0.chunks_exact(4).all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
        tx.update_static();
        let b = img.borrow();
        assert!(b.version > v0);
        assert_ne!(b.data, d0);
    }
    // the light panel's centre is the bright tube colour, its rim the frame
    {
        let img = tx.light_tex.image();
        let b = img.borrow();
        let px = |x: u32, y: u32| {
            let i = ((y * 128 + x) * 4) as usize;
            [b.data[i], b.data[i + 1], b.data[i + 2]]
        };
        assert_eq!(px(1, 1), [0x8f, 0x8a, 0x7a]);
        assert!(px(64, 20)[0] > 240);
    }

    // ---- run-time factories
    assert_eq!(strokes('A').map(|g| g.len()), Some(2));
    assert_eq!(strokes(' ').map(|g| g.len()), Some(0));
    assert!(strokes('a').is_none() && strokes('#').is_none());
    let hw = handwritten_tex("Out of service", &HandOpts::default());
    assert_size(&hw, 1024, 220, "handwrittenTex default");
    assert_eq!(hw.r().wrap_s, Wrap::Clamp);
    let tt = text_tex("HELLO", &TextTexOpts::default());
    assert_size(&tt, 256, 128, "textTex default");
    assert!(pick_soft_pattern_is_valid());
    // `wall_pattern` falls back to `plain`: an unknown kind draws what `plain` draws
    // (compared by output: fn-pointer addresses aren't stable across codegen units)
    let draw = |f: WallPatternFn| {
        let mut c = engine::canvas::Canvas::new(64, 64);
        f(&mut c, 64.0, 64.0, "rgba(80,60,20,0.5)", "rgba(40,30,10,0.5)");
        c.premultiplied().to_vec()
    };
    assert_eq!(draw(wall_pattern("nope")), draw(pattern_plain));

    // ---- surfaces: lazy and cached
    let surfaces = make_surfaces(native_rnd);
    assert_eq!(surfaces.cached(), 0);
    let walls = ["drywall", "paint", "concrete", "brick", "tile"];
    let floors = ["concrete", "office", "tile"];
    let ceils = ["slab", "plaster"];
    let mut surf_items = vec![];
    for k in walls {
        for g in 0..4 {
            let t = surfaces.wall(k, g).unwrap();
            assert_size(&t, 512, 512, k);
            surf_items.push(item(format!("wall {k} {g}"), t));
        }
    }
    for k in floors {
        for g in 0..4 {
            let t = surfaces.floor(k, g).unwrap();
            assert_size(&t, 512, 512, k);
            surf_items.push(item(format!("floor {k} {g}"), t));
        }
    }
    for k in ceils {
        for g in 0..4 {
            let t = surfaces.ceil(k, g).unwrap();
            assert_size(&t, 512, 512, k);
            surf_items.push(item(format!("ceil {k} {g}"), t));
        }
    }
    assert_eq!(surfaces.cached(), 40);
    assert_eq!(surfaces.wall("drywall", 9).unwrap().id(), surfaces.wall("drywall", 3).unwrap().id());
    assert_eq!(surfaces.wall("drywall", -2.5).unwrap().id(), surfaces.wall("drywall", 0).unwrap().id());
    assert_eq!(surfaces.wall("brick", f64::NAN).unwrap().id(), surfaces.wall("brick", 0).unwrap().id());
    assert!(surfaces.wall("marble", 0).is_none() && surfaces.floor("slab", 0).is_none() && surfaces.ceil("tile", 0).is_none());
    assert_eq!(surfaces.cached(), 40);
    assert_eq!(surfaces.scale("brick"), [2.0, 2.0]);
    assert_eq!(surfaces.scale("unknown"), [4.0, 3.1]);
    assert_eq!(shade("#d9d3c2", -0.08), "rgb(200,194,178)");

    // ---- contact sheet
    let sections = with_native(|| {
        let mut secs = vec![];
        secs.push(Section {
            name: "patterns",
            cw: 256.0,
            ch: 256.0,
            cols: 6,
            items: WALL_PATTERNS
                .iter()
                .map(|&(name, f)| {
                    item(
                        name,
                        game::canvas_tex(512, 512, |ctx, w, h| {
                            ctx.set_fill_style("#f4f0e0");
                            ctx.fill_rect(0, 0, w, h);
                            f(ctx, w, h, "rgba(0,0,0,0.85)", "rgba(170,20,20,0.7)");
                        }),
                    )
                })
                .collect(),
        });
        let mut wp = vec![];
        for p in 0..21 {
            for g in 0..4 {
                wp.push(item(format!("{p}:{g}"), make_wallpaper(&WALL_PALETTES[p], "plain", g as i64)));
            }
        }
        wp.push(item("moss 0", make_moss_wallpaper(&WALL_PALETTES[0])));
        wp.push(item("moss 20", make_moss_wallpaper(&WALL_PALETTES[20])));
        secs.push(Section { name: "wallpaper (plain, palette:grime)", cw: 128.0, ch: 128.0, cols: 12, items: wp });
        let detail: [(usize, &str, i64); 8] = [
            (20, "plain", 0),
            (20, "plain", 3),
            (0, "flock", 1),
            (5, "trellis", 2),
            (12, "hex", 1),
            (16, "paisley", 0),
            (1, "stars", 3),
            (9, "vines", 2),
        ];
        let mut wd: Vec<Item> =
            detail.iter().map(|&(p, k, g)| item(format!("{p} {k} {g}"), make_wallpaper(&WALL_PALETTES[p], k, g))).collect();
        wd.insert(2, item("moss 20", make_moss_wallpaper(&WALL_PALETTES[20])));
        secs.push(Section { name: "wallpaper detail", cw: 256.0, ch: 256.0, cols: 5, items: wd });
        let mut fc = vec![];
        for g in 0..5 {
            fc.push(item(format!("carpet {g}"), make_carpet(g)));
        }
        for (g, t) in make_ceiling_variants().into_iter().enumerate() {
            fc.push(item(format!("ceiling {g}"), t));
        }
        fc.push(item("lightTex", make_light_tex()));
        fc.push(item("dustSprite", make_dust_sprite()));
        fc.push(item("faceTex", make_face_tex()));
        fc.push(item("staticTex", tx.static_tex.clone()));
        secs.push(Section { name: "floors, ceilings, sprites", cw: 256.0, ch: 256.0, cols: 5, items: fc });
        let hand_items = vec![
            ("CALL ME", hand("rgba(10,10,10,0.9)", 4.0, 1.6, 0.02, 1.2, false, false, 256.0, 64.0)),
            ("OUT OF SERVICE", hand("rgba(160,30,20,0.98)", 8.0, 2.2, 0.02, 1.6, true, true, 1024.0, 220.0)),
            ("DO NOT ENTER", hand("rgba(160,30,20,0.98)", 6.0, 1.9, 0.02, 1.4, true, true, 1024.0, 220.0)),
            ("OUT OF ORDER", hand("rgba(20,15,10,0.98)", 6.0, 1.8, 0.02, 1.4, false, false, 512.0, 128.0)),
            ("M.KANE", hand("rgba(20,15,10,0.9)", 3.0, 1.5, 0.02, 1.2, false, false, 256.0, 128.0)),
            ("FRAGILE", hand("rgba(20,10,5,0.95)", 4.0, 1.6, 0.02, 1.2, false, false, 512.0, 96.0)),
            ("DON'T LOOK AT IT", hand("rgba(140,20,15,0.95)", 6.0, 1.6, 0.02, 1.2, true, true, 1024.0, 220.0)),
            ("IT HUNTS BY SOUND", hand("rgba(230,225,210,0.90)", 4.0, 1.9, 0.05, 1.5, false, false, 1024.0, 220.0)),
            ("HELP ME", hand("rgba(15,10,8,0.95)", 3.0, 1.2, 0.01, 0.8, true, false, 1024.0, 220.0)),
            ("DAY 47", hand("rgba(220,180,40,0.92)", 5.0, 1.4, 0.03, 1.4, false, false, 1024.0, 220.0)),
            ("RUN", hand("rgba(100,10,10,0.98)", 8.0, 2.4, 0.06, 2.0, false, true, 1024.0, 220.0)),
            ("HELLO ?", hand("rgba(30,50,120,0.95)", 4.0, 1.5, 0.03, 1.1, false, false, 1024.0, 220.0)),
            ("I MADE IT TO LEVEL 3", hand("rgba(35,30,25,0.88)", 3.0, 1.7, 0.04, 1.6, false, false, 1024.0, 220.0)),
            ("ABCDEFGHIJKLM", HandOpts::default()),
            ("NOPQRSTUVWXYZ", HandOpts::default()),
            ("0123456789!?.,'-/()", HandOpts::default()),
        ];
        secs.push(Section {
            name: "handwrittenTex",
            cw: 512.0,
            ch: 128.0,
            cols: 3,
            items: hand_items.iter().map(|(t, o)| item(*t, handwritten_tex(t, o))).collect(),
        });
        let text_items = vec![
            ("EXIT", txt(512.0, 128.0, "bold 60px Helvetica, Arial, sans-serif", "#1c2a44", 0.0)),
            ("STAIRS", txt(256.0, 76.0, "bold 44px Helvetica, Arial, sans-serif", "#e8e2d0", 2.0)),
            ("COLA-Z", txt(1024.0, 192.0, "bold 140px \"Impact\", sans-serif", "#c11515", 6.0)),
            ("$1.25", txt(128.0, 32.0, "bold 22px monospace", "#c11515", 0.0)),
            ("E - -", txt(256.0, 64.0, "bold 40px \"Courier New\", monospace", "#ff4020", 6.0)),
            ("INSERT COIN", txt(256.0, 32.0, "bold 18px sans-serif", "#e8e8e8", 0.0)),
            ("7", txt(128.0, 64.0, "bold 44px \"Courier New\", monospace", "#f04010", 6.0)),
            ("CLEAN 3000", txt(256.0, 64.0, "bold 26px sans-serif", "#4a5a70", 2.0)),
            ("DEFAULT", TextTexOpts::default()),
            ("TILT", TextTexOpts { rot: 0.3, ..Default::default() }),
        ];
        secs.push(Section {
            name: "textTex",
            cw: 512.0,
            ch: 128.0,
            cols: 3,
            items: text_items.iter().map(|(t, o)| item(*t, text_tex(t, o))).collect(),
        });
        secs.push(Section { name: "surfaces", cw: 256.0, ch: 256.0, cols: 8, items: surf_items });
        secs.push(Section {
            name: "wallpaperVariants (random patterns / moss palettes)",
            cw: 128.0,
            ch: 128.0,
            cols: 12,
            items: tx
                .wallpaper_variants
                .iter()
                .map(|v| item(format!("{}:{}{}", v.palette_idx, v.grime_lvl, if v.moss { "m" } else { "" }), v.tex.clone()))
                .collect(),
        });
        secs
    });
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/../target");
    std::fs::create_dir_all(format!("{out}/texture-sheet")).unwrap();
    // per-texture alpha-weighted mean colour, written to target/texture-stats.tsv
    let mut stats = String::new();
    for (si, s) in sections.iter().enumerate() {
        for it in &s.items {
            let [r, g, b, a] = mean_rgba(&it.tex);
            stats += &format!("{si}\t{}\t{r:.2}\t{g:.2}\t{b:.2}\t{a:.2}\n", it.label);
        }
    }
    std::fs::write(format!("{out}/texture-stats.tsv"), stats).unwrap();
    let mut canvases: Vec<Canvas> = sections.iter().map(render_section).collect();
    for (i, (s, c)) in sections.iter().zip(canvases.iter_mut()).enumerate() {
        let _ = s;
        write_png(&format!("{out}/texture-sheet/{i:02}.png"), c);
    }
    let sw = canvases.iter().map(|c| c.width()).fold(0.0, f64::max);
    let sh: f64 = canvases.iter().map(|c| c.height()).sum();
    let mut sheet = Canvas::new(sw as u32, sh as u32);
    sheet.set_fill_style("#101010");
    sheet.fill_rect(0, 0, sw, sh);
    let mut y = 0.0;
    for c in &canvases {
        sheet.draw_image(c, 0, y);
        y += c.height();
    }
    write_png(&format!("{out}/texture-sheet.png"), &mut sheet);
}

/// Alpha-weighted mean RGB and mean alpha of a texture (straight-alpha RGBA).
fn mean_rgba(t: &Tex) -> [f64; 4] {
    let img = t.image();
    let b = img.borrow();
    let (mut r, mut g, mut bl, mut a, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in b.data.chunks_exact(4) {
        let w = p[3] as f64;
        r += p[0] as f64 * w;
        g += p[1] as f64 * w;
        bl += p[2] as f64 * w;
        a += w;
        n += 1.0;
    }
    let k = if a > 0.0 { 1.0 / a } else { 0.0 };
    [r * k, g * k, bl * k, a / n]
}

/// `pick_soft_pattern` only returns pool entries.
fn pick_soft_pattern_is_valid() -> bool {
    with_native(|| {
        (0..200).all(|_| {
            let k = pick_soft_pattern();
            SOFT_PATTERN_POOL.iter().any(|p| p.0 == k)
        })
    })
}
