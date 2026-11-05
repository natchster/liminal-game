//! Batch `t10`'s prop in this crate, made real (`docs/PROPS.md` P3): the
//! stack of 45s left on the floor; its placer and what only it uses
//! (`assets/src/blender/props/t10.py`, `t10.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

/// The Blender part `name`, as modelled, in `grp` with `mat`
fn t10_put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn t10_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

fn t10_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// The single (t10.py): a sleeve's thickness, one sheet of it, the disc's
/// radius, the sleeve's half width
const RS_SLEEVE_T: f64 = 2.0 * 0.0003 + 0.0019 + 0.0001;
const RS_SHEET: f64 = 0.0003;
const RS_VINYL_R: f64 = 0.0875;
const RS_HALF: f64 = 0.092;

/// The record companies of the sleeves and labels (invented)
const RS_LABELS: [&str; 8] = ["MERIDIAN", "BELLWOOD", "ARCADIA", "HALCYON", "SILVERLINE", "NORTHSTAR", "LANTERN", "CASCADE"];
const RS_SONGS: [&str; 12] = [
    "LONELY STREET",
    "BLUE CORNER",
    "AFTER THE DANCE",
    "SATURDAY LIGHTS",
    "WAITING ROOM",
    "SLOW TRAIN HOME",
    "THE OTHER SIDE",
    "ONE MORE TIME",
    "SUMMER RAIN",
    "MIDNIGHT HALL",
    "DON'T LOOK BACK",
    "STRANGER'S WALTZ",
];
const RS_ARTISTS: [&str; 8] =
    ["THE DELMONTS", "RAY HOLLIS", "THE STARLINERS", "JUNE MARLOW", "EDDIE CROSS ORCH.", "THE VALIANTS", "PEGGY LANE", "THE FOUR KEYS"];

/// The sleeves' inks (sRGB) for the old sleeve colours in their order,
/// faded: red, blue, green, gold, black, plum, violet, rust, grey, yellow,
/// teal
const RS_INK: [[u8; 3]; 11] = [
    [150, 52, 58],
    [52, 86, 128],
    [66, 104, 82],
    [176, 140, 70],
    [36, 34, 32],
    [112, 64, 86],
    [92, 90, 132],
    [146, 76, 50],
    [96, 96, 94],
    [184, 168, 76],
    [62, 120, 106],
];
/// The labels' grounds for the old centre colours: red, blue, green, gold
const RS_LABEL_GROUND: [[u8; 3]; 4] = [[138, 40, 42], [40, 62, 112], [44, 86, 62], [168, 132, 58]];
/// Their print for the old label colours: cream, silver, beige, grey
const RS_LABEL_PRINT: [[u8; 3]; 4] = [[226, 214, 186], [196, 198, 196], [214, 192, 160], [168, 168, 164]];

/// One single: its sleeve's ink, its label's print and ground
struct RsSingle {
    ink: usize,
    print: usize,
    ground: usize,
}

// Record stack: 4% chance per level. A few 45s left on the floor, as
// they were dropped: 7 in singles in their paper company sleeves (Blender
// batch `t10`), stacked askew, the labels showing through the sleeves'
// holes; often one out of its sleeve, lying across the stack. The sleeves
// printed in the labels' inks, ring-worn, creased, stained; the vinyl's
// grooves, its dust and its scratches.
pub(super) fn place_record_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old draws, call for call: how many, each sleeve's colour, its
    // label's and its centre's; one out of its sleeve; where and turned
    let sleeve_colors = [0xa02040, 0x2060a0, 0x408060, 0xd0a040, 0x101010, 0x804060, 0x6060a0, 0xa04020, 0x606060, 0xd0d040, 0x40a080];
    let label_colors = [0xf0e0c0, 0xd0d0d0, 0xe0c0a0, 0xa0a0a0];
    let centre_colors = [0xa02040, 0x2060a0, 0x408060, 0xd0a040];
    let record_count = 3 + (rnd() * 3.0).floor() as i32;
    let mut singles = vec![];
    for _ in 0..record_count {
        let s = choice(&sleeve_colors);
        let l = choice(&label_colors);
        let c = choice(&centre_colors);
        let at = |a: &[u32], v: u32| a.iter().position(|&k| k == v).unwrap_or(0);
        singles.push(RsSingle { ink: at(&sleeve_colors, s), print: at(&label_colors, l), ground: at(&centre_colors, c) });
    }
    let bare = rnd() < 0.6;
    let px = g.wx(x) + (rnd() - 0.5) * 1.8;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.8;
    let turn = rnd() * PI * 2.0;
    let rs = group();
    rs.position().set(px, 0, pz);
    rs.rotation().y = turn;
    let vary = t10_vary(px, pz, turn);
    let n = singles.len();
    let sleeves = rs_sleeve_atlas(&singles, vary);
    let labels = rs_label_atlas(&singles, bare, vary);
    let vinyl = phong!({ map: &rs_vinyl_tex(), specular: 0x3a3a3a, shininess: 70 });
    // a cell of an atlas (cols x rows of cells), inset a pixel
    let cell = |t: &Tex, cols: f64, rows: f64, k: usize, px: f64| {
        let (cx, cy) = ((k % cols as usize) as f64, (k / cols as usize) as f64);
        let c = t.duplicate();
        c.set_offset((cx * px + 1.0) / (cols * px), 1.0 - ((cy + 1.0) * px - 1.0) / (rows * px));
        c.set_repeat((px - 2.0) / (cols * px), (px - 2.0) / (rows * px));
        c
    };
    for (i, _) in singles.iter().enumerate() {
        // each laid a little askew on the one under it
        let one = group();
        let (h1, h2, h3) = (t10_hash(vary, i as i32, 1), t10_hash(vary, i as i32, 2), t10_hash(vary, i as i32, 3));
        one.position().set((h1 - 0.5) * 0.016, i as f64 * RS_SLEEVE_T, (h2 - 0.5) * 0.016);
        one.rotation().y = (h3 - 0.5) * 0.24;
        t10_put(&one, "Real__recordSleeve", &lambert!({ map: &cell(&sleeves, 3.0, 2.0, i, 256.0) }));
        let disc = group();
        disc.position().y = RS_SHEET;
        disc.rotation().y = t10_hash(vary, i as i32, 4) * PI * 2.0;
        t10_put(&disc, "Real__recordVinylLow", &vinyl);
        t10_put(&disc, "Real__recordLabel", &lambert!({ map: &cell(&labels, 4.0, 2.0, i, 128.0) }));
        one.add(&disc);
        rs.add(&one);
    }
    if bare {
        // out of its sleeve, lying across the stack: on its edge at one side,
        // its far rim on the floor
        let top = n as f64 * RS_SLEEVE_T;
        let d = 0.095;
        let tilt = (top / (d + RS_VINYL_R - RS_HALF)).atan();
        let side = group();
        side.rotation().y = t10_hash(vary, 9, 9) * PI * 2.0;
        let disc = group();
        disc.position().set(d, top, 0);
        disc.rotation().z = -tilt;
        t10_put(&disc, "Real__recordVinyl", &vinyl);
        t10_put(&disc, "Real__recordLabel", &lambert!({ map: &cell(&labels, 4.0, 2.0, n, 128.0) }));
        side.add(&disc);
        rs.add(&side);
    }
    g.world.add(&rs);
}

fn rgb(c: [u8; 3], a: f64) -> String {
    format!("rgba({},{},{},{a:.3})", c[0], c[1], c[2])
}

/// A soft round spot, squashed whole so its fade is never cut
fn rs_soft(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, col: [u8; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &rgb(col, a));
    gr.add_color_stop(1, &rgb(col, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The sleeves (256 px cells, three by two, the top face as it is mapped:
/// its mouth at the right): white or kraft paper printed in the company's
/// ink, its name and its slogan; the record's ring worn into it, creases,
/// stains, a corner turned, a name in pen
fn rs_sleeve_atlas(singles: &[RsSingle], vary: u32) -> Tex {
    let looks: Vec<usize> = singles.iter().map(|s| s.ink).collect();
    canvas_tex_opt(768, 512, false, move |ctx, _, _| {
        let mut r = SplitMix64::new(vary, 0x5ee7);
        ctx.set_fill_style("#bdb29a");
        ctx.fill_rect(0, 0, 768, 512);
        for (i, &ink) in looks.iter().enumerate() {
            let (x0, y0) = ((i % 3) as f64 * 256.0, (i / 3) as f64 * 256.0);
            let ink = RS_INK[ink];
            let kraft = r.next() < 0.35;
            let paper: [u8; 3] = if kraft { [160, 128, 92] } else { [198, 188, 164] };
            let name = RS_LABELS[((r.next() * 8.0) as usize).min(7)];
            let (cx, cy) = (x0 + 128.0, y0 + 128.0);
            ctx.save();
            ctx.begin_path();
            ctx.rect(x0, y0, 256, 256);
            ctx.clip();
            ctx.set_fill_style(&rgb(paper, 1.0));
            ctx.fill_rect(x0, y0, 256, 256);
            // the paper's grain and foxing
            for _ in 0..500 {
                ctx.set_fill_style(if r.next() < 0.5 { "rgba(255,250,235,0.06)" } else { "rgba(60,40,20,0.06)" });
                ctx.fill_rect(x0 + r.next() * 256.0, y0 + r.next() * 256.0, 1.0 + 2.0 * r.next(), 1);
            }
            match (r.next() * 3.0) as usize {
                0 => {
                    // rays from the hole, every other one printed
                    ctx.set_fill_style(&rgb(ink, 0.85));
                    for k in 0..24 {
                        let a0 = k as f64 * PI / 12.0;
                        ctx.begin_path();
                        ctx.move_to(cx, cy);
                        ctx.arc(cx, cy, 240, a0, a0 + PI / 24.0);
                        ctx.close_path();
                        ctx.fill();
                    }
                    ctx.set_fill_style(&rgb(paper, 1.0));
                    ctx.fill_rect(x0 + 16.0, y0 + 200.0, 224, 34);
                    ctx.set_fill_style(&rgb(ink, 0.95));
                    ctx.set_font("bold 22px Helvetica");
                    ctx.set_text_align("center");
                    ctx.set_text_baseline("middle");
                    ctx.fill_text_max(name, cx, y0 + 218.0, 210);
                }
                1 => {
                    // the company's name, rows of it on the slant
                    ctx.save();
                    ctx.translate(cx, cy);
                    ctx.rotate(-0.5);
                    ctx.set_fill_style(&rgb(ink, 0.8));
                    ctx.set_font("bold 17px Helvetica");
                    ctx.set_text_align("center");
                    ctx.set_text_baseline("middle");
                    for row in -7..=7 {
                        let off = if row % 2 == 0 { 0.0 } else { 60.0 };
                        for col in -2..=2 {
                            ctx.fill_text(name, col as f64 * 125.0 + off, row as f64 * 26.0);
                        }
                    }
                    ctx.restore();
                }
                _ => {
                    // a band across its head, the slogan at its foot
                    ctx.set_fill_style(&rgb(ink, 0.9));
                    ctx.fill_rect(x0, y0 + 12.0, 256, 44);
                    ctx.fill_rect(x0, y0 + 222.0, 256, 3);
                    ctx.set_fill_style(&rgb(paper, 0.95));
                    ctx.set_font("bold 26px Times New Roman");
                    ctx.set_text_align("center");
                    ctx.set_text_baseline("middle");
                    ctx.fill_text_max(name, cx, y0 + 35.0, 230);
                    ctx.set_fill_style(&rgb(ink, 0.9));
                    ctx.set_font("bold 12px Helvetica");
                    ctx.fill_text_max("HIGH FIDELITY  ·  45 R.P.M.", cx, y0 + 238.0, 230);
                    ctx.set_font("10px Helvetica");
                    ctx.fill_text_max("LOOK FOR THIS SLEEVE ON THE LATEST HITS", cx, y0 + 208.0, 230);
                }
            }
            // the record's ring pressed into it, rubbed pale on its crest
            ctx.set_stroke_style("rgba(255,250,236,0.16)");
            ctx.set_line_width(5);
            ctx.begin_path();
            ctx.arc(cx, cy, 121, 0, 7);
            ctx.stroke();
            ctx.set_stroke_style("rgba(30,20,10,0.10)");
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.arc(cx, cy, 116, 0, 7);
            ctx.stroke();
            // the mouth's edge worn by fingers (at the right)
            let gr = ctx.create_linear_gradient(x0 + 236.0, 0, x0 + 256.0, 0);
            gr.add_color_stop(0, "rgba(60,44,30,0)");
            gr.add_color_stop(1, "rgba(60,44,30,0.22)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x0 + 236.0, y0, 20, 256);
            // creases
            for _ in 0..(1 + (r.next() * 3.0) as usize) {
                let (ax, ay) = (x0 + r.next() * 256.0, y0 + r.next() * 256.0);
                let a = r.next() * PI;
                let l = 40.0 + r.next() * 110.0;
                ctx.set_stroke_style("rgba(255,250,240,0.20)");
                ctx.set_line_width(1.2);
                ctx.begin_path();
                ctx.move_to(ax, ay);
                ctx.line_to(ax + a.cos() * l, ay + a.sin() * l);
                ctx.stroke();
                ctx.set_stroke_style("rgba(40,28,18,0.18)");
                ctx.begin_path();
                ctx.move_to(ax + 1.0, ay + 1.0);
                ctx.line_to(ax + a.cos() * l + 1.0, ay + a.sin() * l + 1.0);
                ctx.stroke();
            }
            // a water stain, the floor's grime
            if r.next() < 0.6 {
                let (sx, sy) = (x0 + 30.0 + r.next() * 196.0, y0 + 30.0 + r.next() * 196.0);
                let rr = 20.0 + r.next() * 50.0;
                rs_soft(ctx, sx, sy, rr, 0.7 + 0.3 * r.next(), r.next() * PI, [120, 92, 50], 0.22);
                ctx.set_stroke_style("rgba(110,80,40,0.25)");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.arc(sx, sy, rr * 0.8, 0, 7);
                ctx.stroke();
            }
            for _ in 0..3 {
                rs_soft(ctx, x0 + r.next() * 256.0, y0 + r.next() * 256.0, 30.0 + r.next() * 60.0, 0.6, r.next() * PI, [70, 60, 46], 0.12);
            }
            // years on the floor: the dust settled on it, scuffed by feet
            for _ in 0..4 {
                rs_soft(
                    ctx,
                    x0 + r.next() * 256.0,
                    y0 + r.next() * 256.0,
                    50.0 + r.next() * 80.0,
                    0.5 + 0.5 * r.next(),
                    r.next() * PI,
                    [150, 140, 122],
                    0.16,
                );
            }
            // someone's name on it in ballpoint
            if r.next() < 0.3 {
                ctx.save();
                ctx.translate(x0 + 30.0 + r.next() * 60.0, y0 + 70.0 + r.next() * 30.0);
                ctx.rotate(-0.08);
                ctx.set_fill_style("rgba(40,52,110,0.7)");
                ctx.set_font("italic 15px Times New Roman");
                ctx.set_text_align("left");
                ctx.fill_text(["Linda", "D. Ross", "Property of Sue", "Carol K.", "Jim"][(r.next() * 5.0) as usize % 5], 0, 0);
                ctx.restore();
            }
            ctx.restore();
        }
    })
}

/// The labels (128 px cells, four by two): the company's ground, its name,
/// the song and the artist, the speed and a number; rubbed, a fingerprint
fn rs_label_atlas(singles: &[RsSingle], bare: bool, vary: u32) -> Tex {
    let mut looks: Vec<(usize, usize)> = singles.iter().map(|s| (s.ground, s.print)).collect();
    if bare {
        let k = (vary % 4) as usize;
        looks.push((k, (k + 1) % 4));
    }
    canvas_tex_opt(512, 256, false, move |ctx, _, _| {
        let mut r = SplitMix64::new(vary, 0x1abe);
        ctx.set_fill_style("#2a2624");
        ctx.fill_rect(0, 0, 512, 256);
        for (i, &(ground, print)) in looks.iter().enumerate() {
            let (cx, cy) = ((i % 4) as f64 * 128.0 + 64.0, (i / 4) as f64 * 128.0 + 64.0);
            let (gc, pc) = (RS_LABEL_GROUND[ground], RS_LABEL_PRINT[print]);
            ctx.set_fill_style(&rgb(gc, 1.0));
            ctx.begin_path();
            ctx.arc(cx, cy, 64, 0, 7);
            ctx.fill();
            ctx.set_stroke_style(&rgb(pc, 0.8));
            ctx.set_line_width(1.2);
            ctx.begin_path();
            ctx.arc(cx, cy, 58, 0, 7);
            ctx.stroke();
            ctx.set_fill_style(&rgb(pc, 0.95));
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_font("bold 13px Times New Roman");
            ctx.fill_text_max(RS_LABELS[((r.next() * 8.0) as usize).min(7)], cx, cy - 40.0, 80);
            ctx.set_font("bold 7px Helvetica");
            ctx.fill_text("45 RPM", cx - 38.0, cy);
            ctx.fill_text(
                &format!("{}-{}", ["M", "K", "B", "S"][(r.next() * 4.0) as usize % 4], 100 + (r.next() * 900.0) as u32),
                cx + 38.0,
                cy,
            );
            ctx.set_font("bold 9px Helvetica");
            ctx.fill_text_max(RS_SONGS[((r.next() * 12.0) as usize).min(11)], cx, cy + 28.0, 92);
            ctx.set_font("7px Helvetica");
            ctx.fill_text_max(RS_ARTISTS[((r.next() * 8.0) as usize).min(7)], cx, cy + 40.0, 84);
            // rubbed, a thumb's print
            rs_soft(ctx, cx + (r.next() - 0.5) * 60.0, cy + (r.next() - 0.5) * 60.0, 22.0, 0.7, r.next() * PI, [220, 210, 190], 0.12);
            rs_soft(ctx, cx, cy, 64.0, 1.0, 0.0, [20, 16, 12], 0.15);
        }
    })
}

/// A record's face as its top is mapped (u, v across its 175 mm): black
/// vinyl, the grooves in bands between the tracks (a fine sheen, rings),
/// the run-out smooth, dust and hairline scratches; shared by every disc
fn rs_vinyl_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 256, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = ((i as f64 + 0.5) / w - 0.5, (j as f64 + 0.5) / h - 0.5);
                        let rr = (x * x + y * y).sqrt() * 0.175; // metres from the middle
                        let a = y.atan2(x);
                        let mut v = 0.075;
                        if (0.046..0.0835).contains(&rr) {
                            // the grooves: a sheen in rings, the track gaps darker and smooth
                            let gap = ((rr - 0.062).abs() < 0.0006) || ((rr - 0.074).abs() < 0.0006);
                            let ring = ((rr * 9000.0).sin() * 0.5 + 0.5) * 0.04;
                            v = if gap { 0.05 } else { 0.10 + ring + 0.03 * (a * 2.0).cos().abs() };
                        }
                        if rr > 0.0855 {
                            v = 0.09;
                        }
                        // dust and scratches
                        let hsh = t10_hash(0x51ad, i as i32, j as i32);
                        if hsh > 0.992 {
                            v = 0.45;
                        }
                        let s = ((x * 3.1 + y * 1.3) * 160.0).sin().abs() < 0.008
                            && t10_hash(0x51ae, (x * 20.0) as i32, (y * 20.0) as i32) > 0.6;
                        if s {
                            v += 0.12;
                        }
                        let b = (v * 255.0).min(255.0) as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[b, b, b, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}
