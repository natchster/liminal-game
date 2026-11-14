//! `place_school_desks`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::opaque as rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::skip;

pub(crate) fn place_school_desks(g: &mut Game, _dist: &[i32]) {
    // A combination chair desk (batch `r18`): a laminate top on particleboard
    // in its edge moulding, carved and written on, a corner knocked off; the
    // seat and back of moulded plywood worn to the wood; a chromed steel
    // frame on two runners gone to rust at its feet; the book rack under the
    // seat. On it a textbook left open, sometimes a pencil, sometimes an apple
    // dried to a husk (docs/research/props-remodelling/washroom_janitor_classroom_and_plant.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let sd = group();
    let put = |grp: &Obj, name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            grp.add(&mesh(geo, mat));
        }
    };
    // the old book's draws: its cloth's colour, its turn on the desk, the 14
    // lines of print on its pages (what is on the pages)
    let book_colors = [0x603030, 0x2060a0, 0x408060, 0x804020, 0x604040];
    let book_kind = {
        let c = choice(&book_colors);
        book_colors.iter().position(|&k| k == c).unwrap_or(0)
    };
    let br = (rnd() - 0.5) * 0.3;
    let mut text_salt = 0x5c40_0d35u32;
    for _side in 0..2 {
        for _line in 0..7 {
            let lw = rnd();
            let lx = rnd();
            text_salt = text_salt.rotate_left(7) ^ ((lw * 65535.0) as u32) ^ ((lx * 65535.0) as u32).rotate_left(16);
        }
    }
    let salt = text_salt ^ (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b);
    put(&sd, "Real__schoolDeskTop", &lambert!({ map: &top_tex(salt) }));
    put(&sd, "Real__schoolDeskEdge", &phong!({ color: 0x3b2c22, specular: 0x1a1816, shininess: 20 }));
    put(&sd, "Real__schoolDeskCore", &lambert!({ color: 0x9a8462 }));
    let chrome = phong!({ map: &chrome_tex(salt), specular: 0x6c6c68, shininess: 50 });
    put(&sd, "Real__schoolDeskFrame", &chrome);
    put(&sd, "Real__schoolDeskRack", &chrome);
    put(&sd, "Real__schoolDeskScrews", &phong!({ color: 0x6a6964, specular: 0x6a6a66, shininess: 40 }));
    put(&sd, "Real__schoolDeskGlides", &phong!({ color: 0x1c1b1a, specular: 0x121212, shininess: 12 }));
    put(&sd, "Real__schoolDeskSeat", &phong!({ map: &ply_tex(false, salt), specular: 0x1e1a14, shininess: 18 }));
    put(&sd, "Real__schoolDeskBack", &phong!({ map: &ply_tex(true, salt), specular: 0x1e1a14, shininess: 18 }));
    // the book, open, turned as it was left (the pencil on its pages)
    let book = group();
    book.rotation().y = br;
    put(&book, "Real__schoolDeskBookCover", &lambert!({ color: CLOTH[book_kind] }));
    put(&book, "Real__schoolDeskBookPages", &lambert!({ map: &pages_tex(salt) }));
    sd.add(&book);
    // 70% chance: an apple left on the desk, dried to a brown husk
    if rnd() < 0.7 {
        let apple_colors = [0xa02020, 0xa04020, 0xc03020, 0x408030];
        let ak = {
            let c = choice(&apple_colors);
            apple_colors.iter().position(|&k| k == c).unwrap_or(0)
        };
        let apple = group();
        put(&apple, "Real__schoolDeskApple", &lambert!({ map: &apple_tex(ak, salt) }));
        put(&apple, "Real__schoolDeskAppleStem", &lambert!({ color: 0x3a2c1e }));
        apple.position().set(-0.235, TOP_Y, -0.150);
        apple.rotation().y = ak as f64 * 1.7 + 0.4;
        sd.add(&apple);
    }
    // 60% chance: a yellow HB pencil, sharpened short, on the pages
    if rnd() < 0.6 {
        let turn = (rnd() - 0.5) * 0.5;
        let pencil = group();
        put(&pencil, "Real__schoolDeskPencil", &phong!({ map: &pencil_tex(salt), specular: 0x2a2620, shininess: 30 }));
        put(&pencil, "Real__schoolDeskPencilWood", &lambert!({ color: 0xb3916a }));
        put(&pencil, "Real__schoolDeskPencilLead", &phong!({ color: 0x2b2b2d, specular: 0x5a5a5c, shininess: 50 }));
        put(&pencil, "Real__schoolDeskPencilFerrule", &phong!({ color: 0x8e8670, specular: 0x6a6658, shininess: 40 }));
        put(&pencil, "Real__schoolDeskPencilEraser", &lambert!({ color: 0x9c6c62 }));
        pencil.rotation().y = turn;
        pencil.position().set(0.084, TOP_Y + page_top(0.084), -0.06);
        book.add(&pencil);
    }
    sd.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    sd.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sd);
}

/// The desk top's surface
const TOP_Y: f64 = 0.745;

/// The book's cloth: maroon, blue, green, brown, plum, faded (sRGB hex)
const CLOTH: [u32; 5] = [0x5a2e2a, 0x2c4560, 0x34503e, 0x6a4428, 0x50363a];

const BK_W: f64 = 0.168;

const BK_T: f64 = 0.011;

/// The height of the open pages over the desk at x across the spread (as
/// `industrial/washroom_janitor_classroom_and_plant.py` models them)
fn page_top(x: f64) -> f64 {
    let ax = x.abs();
    0.0035 + BK_T * (1.0 - (-ax / 0.022).exp()) - 0.0018 * (ax / BK_W).powi(3) + 0.0012
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
/// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let (iw, _) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({},{},{},{:.3})", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a)
}

/// The top's laminate (laid from above: u across, v from the back edge to
/// the front): a medium oak print worn pale where arms rested, carved
/// with initials and a heart cut through to the paper under it, written
/// on in ballpoint and pencil, ringed where cans stood, dusty
fn top_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 384, false, move |ctx, w, h| {
        skip(salt);
        let seed = salt ^ 0x70b5;
        // laid from above, the texture's top is at the desk's back (the
        // student's side) and its left at the student's right: turned
        // half round, so what is carved reads from the seat
        fill(ctx, w, h, |u, v| {
            let (u, v) = (1.0 - u, 1.0 - v);
            let (x, y) = (u * 0.61, v * 0.457);
            let wob = fbm(seed ^ 9, x * 6.0, y * 6.0, 2) * 0.02;
            let grain = fbm(seed, x * 4.0, (y + wob) * 160.0, 3);
            let figure = fbm(seed ^ 1, x * 1.5, (y + wob) * 25.0, 3);
            let mut c = mix([0.5, 0.36, 0.22], [0.62, 0.47, 0.31], figure);
            c = mix(c, [0.38, 0.26, 0.15], (grain - 0.5).max(0.0) * 1.6);
            let pore = (vn(seed ^ 2, x * 900.0, y * 300.0) - 0.82).max(0.0) * 4.0;
            c = mix(c, [0.3, 0.2, 0.12], pore);
            // worn where forearms lay: paler, matte
            let wear = (-(((x - 0.305) / 0.18).powi(2) + ((y - 0.08) / 0.09).powi(2))).exp() * 0.5;
            c = mix(c, [0.7, 0.6, 0.47], wear * (0.6 + 0.4 * fbm(seed ^ 3, x * 40.0, y * 40.0, 2)));
            // dust and grime
            c = mix(c, [0.48, 0.45, 0.4], 0.12 + 0.12 * fbm(seed ^ 4, x * 10.0, y * 10.0, 3));
            rgb(c)
        });
        ctx.translate(w, h);
        ctx.rotate(PI);
        // rings where cans and cups stood
        for _ in 0..3 {
            let (cx, cy, r) = (w * (0.15 + 0.7 * rnd()), h * (0.2 + 0.6 * rnd()), 26.0 + rnd() * 8.0);
            ctx.set_stroke_style(css([0.3, 0.22, 0.14], 0.22 + 0.15 * rnd()).as_str());
            ctx.set_line_width(2.0 + rnd() * 2.0);
            ctx.begin_path();
            ctx.arc(cx, cy, r, rnd() * 2.0, 4.0 + rnd() * 2.3);
            ctx.stroke();
        }
        // carved: cut through the print to the pale paper under it,
        // its edges dark with dirt
        let carve = |ctx: &mut Canvas, text: &str, x: f64, y: f64, size: f64, a: f64| {
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(a);
            ctx.set_font(format!("bold {size:.0}px Helvetica Neue").as_str());
            ctx.set_line_join("round");
            ctx.set_line_width(size * 0.12);
            ctx.set_stroke_style(css([0.2, 0.14, 0.09], 0.6).as_str());
            ctx.stroke_text(text, 0.8, 0.8);
            ctx.set_line_width(size * 0.06);
            ctx.set_stroke_style(css([0.68, 0.6, 0.48], 0.75).as_str());
            ctx.stroke_text(text, 0.0, 0.0);
            ctx.restore();
        };
        let pick = |k: usize, list: &[&'static str]| list[(hash(seed ^ 0x51, k as i32, 3) * list.len() as f64) as usize % list.len()];
        let initials = ["JR", "T.M.", "KL", "D+S", "R.W.", "AJ"];
        carve(ctx, pick(0, &initials), w * 0.74, h * 0.3, 30.0, -0.2);
        carve(ctx, pick(1, &initials), w * 0.12, h * 0.82, 24.0, 0.15);
        // a heart, gouged
        ctx.save();
        ctx.translate(w * 0.86, h * 0.72);
        ctx.rotate(0.3);
        ctx.set_line_width(4.0);
        ctx.set_stroke_style(css([0.22, 0.15, 0.1], 0.7).as_str());
        ctx.begin_path();
        ctx.move_to(0.0, 14.0);
        ctx.bezier_curve_to(-22.0, -2.0, -12.0, -20.0, 0.0, -8.0);
        ctx.bezier_curve_to(12.0, -20.0, 22.0, -2.0, 0.0, 14.0);
        ctx.stroke();
        ctx.set_line_width(2.0);
        ctx.set_stroke_style(css([0.78, 0.7, 0.58], 0.9).as_str());
        ctx.stroke();
        ctx.restore();
        // ballpoint: a doodled box, a name, a game of noughts and crosses
        ctx.set_stroke_style(css([0.16, 0.2, 0.42], 0.55).as_str());
        ctx.set_line_width(1.3);
        ctx.begin_path();
        let (gx, gy) = (w * 0.2, h * 0.28);
        for k in 1..3 {
            ctx.move_to(gx + k as f64 * 14.0, gy);
            ctx.line_to(gx + k as f64 * 14.0 + 2.0, gy + 42.0);
            ctx.move_to(gx - 2.0, gy + k as f64 * 14.0);
            ctx.line_to(gx + 42.0, gy + k as f64 * 14.0 - 2.0);
        }
        ctx.stroke();
        ctx.set_font("16px Marker Felt");
        ctx.set_fill_style(css([0.16, 0.2, 0.42], 0.5).as_str());
        ctx.fill_text(pick(2, &["Lisa", "Danny", "Mike W.", "Tina", "Rob"]), w * 0.45, h * 0.9);
        ctx.set_fill_style(css([0.3, 0.3, 0.3], 0.35).as_str());
        ctx.fill_text(pick(3, &["1987", "1991", "1994", "'89"]), w * 0.6, h * 0.62);
        // scratches all over
        for _ in 0..160 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = (rnd() - 0.5) * 0.6 + if rnd() < 0.3 { PI / 2.0 } else { 0.0 };
            let l = 5.0 + rnd() * 40.0;
            ctx.set_stroke_style(css([0.74, 0.66, 0.54], 0.12 + 0.2 * rnd()).as_str());
            ctx.set_line_width(0.6 + rnd() * 0.8);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The chrome on the steel (laid as a box): its shine gone to grey, rust
/// coming through it in pits, thickest at the feet and the runners
fn chrome_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xc407;
        fill(ctx, w, h, |u, v| {
            let mut c = mix([0.36, 0.36, 0.35], [0.44, 0.44, 0.42], fbm(seed, u * 20.0, v * 20.0, 3));
            // the plating dulled to a grey film, blotched
            c = mix(c, [0.3, 0.29, 0.27], (fbm(seed ^ 3, u * 9.0, v * 9.0, 3) - 0.4).max(0.0) * 1.2);
            let low = sstep(0.3, 0.0, v);
            let pit = (fbm(seed ^ 1, u * 90.0, v * 90.0, 3) - 0.6 + 0.3 * low).max(0.0) * 4.0;
            c = mix(c, [0.33, 0.2, 0.12], pit.min(1.0));
            c = mix(c, [0.24, 0.21, 0.17], low * 0.5);
            rgb(c)
        });
    })
}

/// Moulded plywood under its varnish (the seat's grain front to back,
/// the back's across), worn through to the paler wood where it was sat
/// on and leant on, its edges dark with hands
fn ply_tex(back: bool, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        skip(salt);
        let seed = salt ^ if back { 0xbac4 } else { 0x5ea7 };
        fill(ctx, w, h, |u, v| {
            let (a, b) = if back { (v, u) } else { (u, v) };
            let grain = fbm(seed, a * 60.0, b * 3.0, 3);
            let mut c = mix([0.46, 0.3, 0.16], [0.56, 0.38, 0.21], fbm(seed ^ 1, a * 8.0, b * 2.0, 3));
            c = mix(c, [0.36, 0.22, 0.11], (grain - 0.5).max(0.0) * 1.5);
            let worn =
                (-(((u - 0.5) / 0.28).powi(2) + ((v - 0.5) / 0.3).powi(2))).exp() * (0.5 + 0.5 * fbm(seed ^ 2, u * 30.0, v * 30.0, 2));
            c = mix(c, [0.66, 0.53, 0.38], worn * 0.55);
            let edge = sstep(0.4, 0.5, (u - 0.5).abs().max((v - 0.5).abs()));
            c = mix(c, [0.22, 0.16, 0.1], edge * 0.5);
            rgb(c)
        });
        for _ in 0..60 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 4.0 + rnd() * 20.0;
            ctx.set_stroke_style(css([0.7, 0.58, 0.42], 0.2 + 0.2 * rnd()).as_str());
            ctx.set_line_width(0.7);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The open spread (from above: u across both pages, v from the foot of
/// the pages to the head): a geography textbook's chapter on the water
/// cycle in two columns, its figure, running heads and page numbers; the
/// paper gone yellow, foxed, a tide mark where damp got in at the foot
fn pages_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 368, false, move |ctx, w, h| {
        let seed = salt ^ 0x9a6e;
        // turned half round to read from the seat (as the desk top's)
        fill(ctx, w, h, |u, v| {
            let (x, y) = (1.0 - u, 1.0 - v);
            let mut c = mix([0.8, 0.76, 0.64], [0.86, 0.83, 0.72], fbm(seed, x * 8.0, y * 8.0, 3));
            // the gutter darker, the fore-edges browned
            c = mix(c, [0.62, 0.56, 0.44], sstep(0.04, 0.0, (x - 0.5).abs()) * 0.6);
            c = mix(c, [0.66, 0.58, 0.42], sstep(0.44, 0.5, (x - 0.5).abs()) * 0.5);
            // foxing
            let fox = (vn(seed ^ 1, x * 60.0, y * 45.0) - 0.84).max(0.0) * 5.0;
            c = mix(c, [0.62, 0.48, 0.32], fox * 0.6);
            // the tide mark at the foot
            let tide = 0.12 + 0.05 * fbm(seed ^ 2, x * 9.0, 0.5, 2);
            c = mix(c, [0.7, 0.62, 0.48], sstep(tide + 0.01, tide - 0.03, y) * 0.35);
            c = mix(c, [0.52, 0.44, 0.32], (-((y - tide) / 0.006).powi(2)).exp() * 0.45);
            rgb(c)
        });
        ctx.translate(w, h);
        ctx.rotate(PI);
        let ink = css([0.12, 0.11, 0.1], 0.86);
        let page = |ctx: &mut Canvas, x0: f64, right: bool, n: u32| {
            ctx.set_fill_style(ink.as_str());
            ctx.set_text_align("left");
            ctx.set_font("italic 8px serif");
            let head = if right { "THE WATER CYCLE" } else { "Unit 3  Weather and Climate" };
            ctx.fill_text(head, x0 + 14.0, 18.0);
            ctx.set_text_align(if right { "right" } else { "left" });
            ctx.fill_text(&format!("{n}"), if right { x0 + 242.0 } else { x0 + 14.0 }, 352.0);
            ctx.set_text_align("left");
            let body = [
                "Water is always on the move. It falls as rain",
                "and snow, runs off the land in rivers and soaks",
                "into the ground. The sun warms the surface of",
                "lakes and the sea, and some of the water turns",
                "to vapour and rises into the air. This is called",
                "evaporation. Plants give off water vapour from",
                "their leaves too: this is transpiration.",
                "As the vapour rises it cools, and condenses into",
                "tiny drops of water that form clouds. When the",
                "drops join together and grow too heavy to stay",
                "up, they fall as precipitation: rain, sleet, hail",
                "or snow. Most of it falls back into the sea.",
                "The rest falls on the land, where it collects in",
                "lakes, flows to the sea in rivers, or sinks into",
                "the ground to become groundwater.",
            ];
            if right {
                ctx.set_font("bold 13px Helvetica Neue");
                ctx.fill_text("3.4  Where does the rain go?", x0 + 14.0, 44.0);
                ctx.set_font("9px serif");
                for (k, l) in body.iter().enumerate().take(9) {
                    ctx.fill_text(l, x0 + 14.0, 64.0 + k as f64 * 11.5);
                }
                // the figure: sea, land, a cloud, the sun, the arrows round
                ctx.set_stroke_style(ink.as_str());
                ctx.set_line_width(1.2);
                ctx.stroke_rect(x0 + 16.0, 176.0, 226.0, 128.0);
                ctx.begin_path();
                ctx.move_to(x0 + 16.0, 280.0);
                ctx.bezier_curve_to(x0 + 90.0, 276.0, x0 + 120.0, 250.0, x0 + 170.0, 238.0);
                ctx.line_to(x0 + 242.0, 228.0);
                ctx.stroke();
                ctx.begin_path();
                ctx.arc(x0 + 40.0, 200.0, 10.0, 0.0, 6.3);
                ctx.stroke();
                ctx.begin_path();
                ctx.arc(x0 + 150.0, 200.0, 12.0, 3.1, 6.3);
                ctx.arc(x0 + 168.0, 196.0, 14.0, 3.4, 6.3);
                ctx.arc(x0 + 186.0, 202.0, 10.0, 3.6, 6.4);
                ctx.stroke();
                for k in 0..3 {
                    let xx = x0 + 70.0 + k as f64 * 18.0;
                    ctx.begin_path();
                    ctx.move_to(xx, 274.0);
                    ctx.line_to(xx + 14.0, 218.0);
                    ctx.stroke();
                    let xr = x0 + 180.0 + k as f64 * 8.0;
                    ctx.begin_path();
                    ctx.move_to(xr, 214.0);
                    ctx.line_to(xr - 4.0, 236.0);
                    ctx.stroke();
                }
                ctx.set_font("italic 8px serif");
                ctx.fill_text("Fig. 3.7  The water cycle.", x0 + 16.0, 318.0);
            } else {
                ctx.set_font("9px serif");
                for (k, l) in body.iter().enumerate() {
                    ctx.fill_text(l, x0 + 14.0, 40.0 + k as f64 * 11.5);
                }
                ctx.set_font("bold 10px Helvetica Neue");
                ctx.fill_text("Questions", x0 + 14.0, 232.0);
                ctx.set_font("9px serif");
                let q = [
                    "1  What is evaporation?",
                    "2  Name two kinds of precipitation.",
                    "3  Where does most rain fall?",
                    "4  Draw and label the water cycle.",
                ];
                for (k, l) in q.iter().enumerate() {
                    ctx.fill_text(l, x0 + 14.0, 250.0 + k as f64 * 13.0);
                }
            }
        };
        let n = 40 + 2 * (hash(seed ^ 3, 1, 1) * 60.0) as u32;
        page(ctx, 0.0, false, n);
        page(ctx, 256.0, true, n + 1);
        // pencil underlinings and a note in the margin
        ctx.set_stroke_style(css([0.3, 0.3, 0.3], 0.5).as_str());
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(14.0, 98.0);
        ctx.line_to(190.0, 97.0);
        ctx.stroke();
    })
}

/// The pencil's paint: chrome yellow worn through to the wood at its
/// edges and bitten near its end, its stamp gone grey
fn pencil_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 16, false, move |ctx, w, h| {
        let seed = salt ^ 0x9e11;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.78, 0.6, 0.18];
            c = mix(c, [0.6, 0.48, 0.3], (fbm(seed, u * 30.0, v * 6.0, 2) - 0.55).max(0.0) * 2.0);
            c = mix(c, [0.62, 0.5, 0.36], sstep(0.1, 0.0, u) * (0.5 + 0.5 * vn(seed ^ 1, u * 80.0, v * 10.0)));
            rgb(c)
        });
    })
}

/// The apple dried: a dark brown husk, its old colour still in patches,
/// a white bloom of mould dried in its folds
fn apple_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xa991 ^ kind as u32;
        let old = [[0.3, 0.12, 0.08], [0.32, 0.17, 0.08], [0.34, 0.12, 0.07], [0.24, 0.24, 0.1]][kind.min(3)];
        fill(ctx, w, h, |u, v| {
            let mut c = mix([0.2, 0.13, 0.08], [0.3, 0.2, 0.12], fbm(seed, u * 10.0, v * 10.0, 3));
            c = mix(c, old, (fbm(seed ^ 1, u * 4.0, v * 4.0, 2) - 0.5).max(0.0) * 1.6);
            c = mix(c, [0.62, 0.6, 0.55], (vn(seed ^ 2, u * 40.0, v * 40.0) - 0.82).max(0.0) * 3.0);
            rgb(c)
        });
    })
}
