//! Batch `r15`'s props in this crate, made real (`docs/design/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Newspaper stack: 8% chance per level. Three to six dailies kept and
// stacked (Blender batch r15: a Berliner paper folded once, 315 x 235 mm),
// each a little askew, every other one turned round; the top one's front
// page printed (a local paper's masthead, a mundane lead, a photograph in
// halftone, columns of type), its corner curling, all of it yellowed and
// dusty. The old draws are kept call for call.
pub(super) fn place_newspaper_stacks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let paper_colors = [0xe8d8b0, 0xd8c8a0, 0xe8dcc0];
    let grp = group();
    let n_papers = 3 + (rnd() * 4.0).floor() as i32;
    // each paper's yellowing, slide and turn (the old draws)
    let mut papers = vec![];
    for _ in 0..n_papers {
        let tint: u32 = choice(&paper_colors);
        let px = (rnd() - 0.5) * 0.010;
        let pz = (rnd() - 0.5) * 0.010;
        let yaw = (rnd() - 0.5) * 0.15;
        papers.push((tint, px, pz, yaw));
    }
    scatter(g, &grp, x, z, 2.0);
    let salt = r15_news::salt(&grp);
    // every paper's front page; the ones under the top are seen at their edges alone
    let front = r15_news::front_tex(salt);
    let edges = r15_news::edges_tex();
    for (i, &(tint, px, pz, yaw)) in papers.iter().enumerate() {
        let top = i + 1 == papers.len();
        let paper = group();
        paper.position().set(px, i as f64 * 0.0075, pz);
        // turned round when its slide went one way: its fold the other side
        paper.rotation().y = yaw + if px > 0.0 { PI } else { 0.0 };
        let parts = [
            ("Real__newsBody", lambert!({ color: tint })),
            ("Real__newsEdges", lambert!({ map: &edges, color: tint })),
            (if top { "Real__newsFrontCurl" } else { "Real__newsFront" }, lambert!({ map: &front, color: tint, side: DoubleSide })),
        ];
        for (name, mat) in &parts {
            if let Some(geo) = prop_part_as_modelled(name) {
                paper.add(&mesh(geo, mat));
            }
        }
        grp.add(&paper);
    }
    g.world.add(&grp);
}

/// The newspaper's print (`place_newspaper_stacks`): a local daily's front
/// page above the fold, made up for the game (no real paper's name), its
/// news the town's; seeded by where the stack lies.
mod r15_news {
    use game::prelude::*;

    pub fn salt(o: &Obj) -> u32 {
        let p = o.pos();
        ((p.x * 977.0).floor() as i64 ^ ((p.z * 1181.0).floor() as i64).rotate_left(13)) as u32
    }

    use placers_kit::paint::rgba;

    fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &rgba(c, a));
        gr.add_color_stop(1, &rgba(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
    }

    const LEADS: [(&str, &str); 4] = [
        ("Council approves new", "bus timetable for spring"),
        ("Bridge repairs to close", "Mill Road for six weeks"),
        ("Market hall roof to be", "replaced next summer"),
        ("Rain expected to ease", "in time for the weekend"),
    ];
    const SECONDS: [&str; 4] =
        ["Library to open on Saturdays", "School fete raises \u{a3}1,240", "Allotment show results", "Car park charges unchanged"];
    const SMALL: [&str; 4] =
        ["Bowls club elects chairman", "Roadworks on Station Road", "Choir to sing at harvest service", "New opening hours at baths"];
    const BODY: [&str; 8] = [
        "The plans, first put forward last year, were approved at Tuesday\u{2019}s meeting of the highways committee.",
        "Councillors heard that the changes would mean more buses at busy times and an earlier first service on weekdays.",
        "A spokesman said copies would be available from the library and the post office from the end of the month.",
        "Work is expected to begin in the new year, weather permitting, and to take about six weeks in all.",
        "Residents who wish to comment may write to the clerk at the town hall before the end of November.",
        "The committee also agreed to look again at parking in the High Street at its next meeting.",
        "Mrs Joan Avery, who has run the stall for twenty years, said she was pleased with the turnout.",
        "Tickets are available from the newsagent on Bridge Street and on the door, price 50p.",
    ];

    /// Columns of body type, set ragged in a small serif, from `start` in
    /// the text (each word measured once a page: `widths`)
    fn column(
        ctx: &mut Canvas,
        widths: &mut std::collections::HashMap<&'static str, f64>,
        x: f64,
        y0: f64,
        y1: f64,
        w: f64,
        size: f64,
        start: usize,
    ) {
        ctx.set_font(&format!("{size:.1}px Times New Roman"));
        ctx.set_fill_style("rgba(40,36,34,0.82)");
        ctx.set_text_align("left");
        let words: Vec<&'static str> = (0..BODY.len()).flat_map(|k| BODY[(start + k) % BODY.len()].split(' ')).collect();
        let space = *widths.entry(" ").or_insert_with(|| ctx.measure_text(" ").width);
        let mut y = y0;
        let mut wi = 0;
        while y < y1 {
            let mut line = String::new();
            let mut used = 0.0;
            loop {
                let wd = words[wi % words.len()];
                let ww = *widths.entry(wd).or_insert_with(|| ctx.measure_text(wd).width);
                if !line.is_empty() && used + space + ww > w {
                    break;
                }
                if !line.is_empty() {
                    line.push(' ');
                    used += space;
                }
                line.push_str(wd);
                used += ww;
                wi += 1;
            }
            ctx.fill_text(&line, x, y);
            y += size * 1.18;
        }
    }

    /// A photograph in halftone: a street of terraced houses under a pale
    /// sky, a tree, a bus shelter at the kerb with someone waiting in it, a
    /// parked car, the road wet; drawn in greys, then screened into dots as
    /// a newspaper prints it
    fn photo(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64, r: &mut SplitMix64) {
        ctx.save();
        ctx.begin_path();
        ctx.rect(x0, y0, w, h);
        ctx.clip();
        let sky = ctx.create_linear_gradient(0, y0, 0, y0 + h * 0.45);
        sky.add_color_stop(0, "#d4d4d0");
        sky.add_color_stop(1, "#ececea");
        ctx.set_fill_style(&sky);
        ctx.fill_rect(x0, y0, w, h);
        // the terrace: brick fronts, slate roofs and chimneys, sash windows
        // with their bars, doors, receding a little toward the right
        let horizon = y0 + h * 0.66;
        let mut x = x0 - 20.0;
        let mut k = 0;
        while x < x0 + w {
            let hw = 74.0 + 12.0 * r.next();
            let ht = h * (0.46 - 0.10 * (x - x0) / w);
            ctx.set_fill_style(if k % 2 == 0 { "#a4a29e" } else { "#acaaa6" });
            ctx.fill_rect(x, horizon - ht, hw, ht);
            ctx.set_fill_style("#6a6866");
            ctx.begin_path();
            ctx.move_to(x - 3.0, horizon - ht);
            ctx.line_to(x + 8.0, horizon - ht - 22.0);
            ctx.line_to(x + hw - 8.0, horizon - ht - 22.0);
            ctx.line_to(x + hw + 3.0, horizon - ht);
            ctx.close_path();
            ctx.fill();
            ctx.set_fill_style("#5a5856");
            ctx.fill_rect(x + hw * 0.72, horizon - ht - 36.0, 10, 16);
            for (wx, wy, ww, wh) in [(0.14, 0.14, 0.26, 0.26), (0.58, 0.14, 0.26, 0.26), (0.14, 0.55, 0.30, 0.28)] {
                let (px, py, pw, ph) = (x + hw * wx, horizon - ht + ht * wy, hw * ww, ht * wh);
                ctx.set_fill_style("#e2e2de");
                ctx.fill_rect(px - 2.0, py - 2.0, pw + 4.0, ph + 4.0);
                ctx.set_fill_style("#585654");
                ctx.fill_rect(px, py, pw, ph);
                ctx.set_fill_style("#d8d8d4");
                ctx.fill_rect(px, py + ph / 2.0 - 1.0, pw, 2);
                ctx.fill_rect(px + pw / 2.0 - 1.0, py, 2, ph);
            }
            ctx.set_fill_style("#4a4846");
            ctx.fill_rect(x + hw * 0.62, horizon - ht * 0.40, hw * 0.2, ht * 0.40);
            x += hw;
            k += 1;
        }
        // a tree in front of the houses at the left
        let tx = x0 + w * (0.10 + 0.15 * r.next());
        ctx.set_fill_style("#585654");
        ctx.fill_rect(tx - 4.0, horizon - h * 0.30, 8, h * 0.30);
        for _ in 0..14 {
            let (cx, cy) = (tx + (r.next() - 0.5) * 90.0, horizon - h * 0.36 - r.next() * h * 0.22);
            ctx.set_fill_style(if r.next() < 0.5 { "#6c6a66" } else { "#7c7a76" });
            ctx.begin_path();
            ctx.arc(cx, cy, 18.0 + r.next() * 16.0, 0, 7);
            ctx.fill();
        }
        // the pavement, the kerb and the road, wet and lighter toward us
        ctx.set_fill_style("#c2c0ba");
        ctx.fill_rect(x0, horizon, w, h * 0.07);
        ctx.set_fill_style("#7a7874");
        ctx.fill_rect(x0, horizon + h * 0.07, w, 3);
        let road = ctx.create_linear_gradient(0, horizon + h * 0.07, 0, y0 + h);
        road.add_color_stop(0, "#9a9894");
        road.add_color_stop(1, "#c4c2be");
        ctx.set_fill_style(&road);
        ctx.fill_rect(x0, horizon + h * 0.07 + 3.0, w, h);
        ctx.set_fill_style("#e4e4e0");
        for k in 0..6 {
            ctx.fill_rect(x0 + 20.0 + k as f64 * 110.0, y0 + h * 0.86, 60, 4);
        }
        // a parked car at the kerb
        let cx = x0 + w * (0.30 + 0.1 * r.next());
        ctx.set_fill_style("#3e3c3a");
        ctx.fill_rect(cx, horizon + h * 0.02, 130, 30);
        ctx.fill_rect(cx + 26.0, horizon - h * 0.06, 74, 26);
        ctx.set_fill_style("#bcbcb8");
        ctx.fill_rect(cx + 32.0, horizon - h * 0.05, 30, 18);
        ctx.fill_rect(cx + 66.0, horizon - h * 0.05, 28, 18);
        ctx.set_fill_style("#222");
        for wx in [cx + 22.0, cx + 104.0] {
            ctx.begin_path();
            ctx.arc(wx, horizon + h * 0.02 + 30.0, 11, 0, 7);
            ctx.fill();
        }
        // the shelter, and someone waiting in it
        let sx = x0 + w * (0.62 + 0.15 * r.next());
        ctx.set_fill_style("#3a3836");
        ctx.fill_rect(sx, horizon - h * 0.22, 4, h * 0.27);
        ctx.fill_rect(sx + 84.0, horizon - h * 0.22, 4, h * 0.27);
        ctx.fill_rect(sx - 4.0, horizon - h * 0.24, 96, 7);
        ctx.set_fill_style("rgba(200,200,196,0.5)");
        ctx.fill_rect(sx + 4.0, horizon - h * 0.17, 80, h * 0.17);
        ctx.set_fill_style("#2a2826");
        ctx.fill_rect(sx + 36.0, horizon - h * 0.14, 12, h * 0.17);
        ctx.begin_path();
        ctx.arc(sx + 42.0, horizon - h * 0.16, 7, 0, 7);
        ctx.fill();
        ctx.restore();
        // screened: a dot a cell, as big as the cell is dark
        let (xi, yi, wi, hi) = (x0 as i64, y0 as i64, w as i64, h as i64);
        let img = ctx.get_image_data(xi, yi, wi, hi);
        let cell = 4.0;
        let (iw, ih) = (img.width as usize, img.height as usize);
        let lum = |x: usize, y: usize| {
            let i = (y.min(ih - 1) * iw + x.min(iw - 1)) * 4;
            (img.data[i] as f64 * 0.3 + img.data[i + 1] as f64 * 0.59 + img.data[i + 2] as f64 * 0.11) / 255.0
        };
        let mut out = ctx.create_image_data(wi, hi);
        for y in 0..ih {
            for x in 0..iw {
                let (cx, cy) = ((x as f64 / cell).floor() * cell + cell / 2.0, (y as f64 / cell).floor() * cell + cell / 2.0);
                let d = 1.0 - lum(cx as usize, cy as usize);
                let rad = d.sqrt() * cell * 0.72;
                let dist = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
                let ink = ((rad - dist + 0.5).clamp(0.0, 1.0)) * 0.85;
                let i = (y * iw + x) * 4;
                let v = (232.0 * (1.0 - ink) + 36.0 * ink) as u8;
                out.data[i] = v;
                out.data[i + 1] = v;
                out.data[i + 2] = (v as f64 * 0.97) as u8;
                out.data[i + 3] = 255;
            }
        }
        ctx.put_image_data(&out, xi, yi);
    }

    /// The front page above the fold (315 x 235 mm, its masthead at the top,
    /// the fold at the bottom), 1024 px across
    pub fn front_tex(salt: u32) -> Tex {
        canvas_tex_opt(1024, 764, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x9e75);
            ctx.set_fill_style("#e4dfd2");
            ctx.fill_rect(0, 0, w, h);
            ctx.save();
            ctx.scale(w / 1024.0, h / 764.0);
            let lead = LEADS[(salt as usize / 3) % LEADS.len()];
            let second = SECONDS[(salt as usize / 5) % SECONDS.len()];
            let small = SMALL[(salt as usize / 11) % SMALL.len()];
            let ink = "rgba(30,28,26,0.92)";
            // the masthead and its ears
            ctx.set_fill_style(ink);
            ctx.set_text_align("center");
            ctx.set_font("bold 70px Times New Roman");
            ctx.fill_text("The Saltham Gazette", 512, 98);
            ctx.set_font("13px Times New Roman");
            ctx.set_text_align("left");
            ctx.fill_text(&format!("No. {}", 31_000 + salt % 900), 34, 40);
            ctx.set_text_align("right");
            ctx.fill_text("40p", 990, 40);
            ctx.fill_rect(32, 116, 960, 2.5);
            ctx.fill_rect(32, 142, 960, 1.0);
            ctx.set_font("bold 13px Times New Roman");
            ctx.set_text_align("left");
            let days = ["MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY", "SATURDAY"];
            let months = ["MARCH", "APRIL", "MAY", "SEPTEMBER", "OCTOBER", "NOVEMBER"];
            ctx.fill_text(&format!("{}, {} {}", days[(salt % 6) as usize], 3 + salt % 25, months[(salt / 6 % 6) as usize]), 34, 135);
            ctx.set_text_align("right");
            ctx.fill_text("WEATHER: SHOWERS, BRIGHTER LATER", 990, 135);
            // the lead across four columns, its photograph and caption
            ctx.set_text_align("left");
            ctx.set_font("bold 46px Times New Roman");
            ctx.fill_text_max(lead.0, 34, 196, 640);
            ctx.fill_text_max(lead.1, 34, 244, 640);
            photo(ctx, 34.0, 266.0, 636.0, 300.0, &mut r);
            ctx.set_font("italic 13px Times New Roman");
            ctx.set_fill_style(ink);
            let caption = [
                "Station Road, where the new stop is to go. Picture by our photographer.",
                "Mill Road, to be closed from the bridge to the school. Picture by our photographer.",
                "Shoppers on Market Street, below the hall. Picture by our photographer.",
                "Tuesday\u{2019}s showers on Station Road. Picture by our photographer.",
            ][(salt as usize / 3) % LEADS.len()];
            ctx.fill_text_max(caption, 34, 586, 636);
            let mut widths = std::collections::HashMap::new();
            for k in 0..4 {
                column(ctx, &mut widths, 34.0 + k as f64 * 162.0, 612.0, 760.0, 150.0, 12.0, k * 2);
            }
            ctx.fill_rect(686, 160, 1.0, 600);
            // the second story and a short one
            ctx.set_fill_style(ink);
            ctx.set_font("bold 28px Times New Roman");
            let mut y = 190.0;
            for line in [second] {
                ctx.fill_text_max(line, 700, y, 290);
                y += 32.0;
            }
            column(ctx, &mut widths, 700.0, y + 4.0, 470.0, 140.0, 12.0, 3);
            column(ctx, &mut widths, 850.0, y + 4.0, 470.0, 140.0, 12.0, 6);
            ctx.set_fill_style(ink);
            ctx.fill_rect(700, 486, 290, 1.0);
            ctx.set_font("bold 21px Times New Roman");
            ctx.fill_text_max(small, 700, 516, 290);
            column(ctx, &mut widths, 700.0, 540.0, 760.0, 140.0, 12.0, 1);
            column(ctx, &mut widths, 850.0, 540.0, 760.0, 140.0, 12.0, 5);
            ctx.restore();
            // the paper yellowed from its edges in, sun-faded where it lay on
            // top, a ring where a cup stood on it, the dust of years
            for (x0, y0, x1, y1) in [(0.0, 0.0, 0.0, 70.0), (0.0, h, 0.0, h - 50.0), (0.0, 0.0, 70.0, 0.0), (w, 0.0, w - 70.0, 0.0)] {
                let gr = ctx.create_linear_gradient(x0, y0, x1, y1);
                gr.add_color_stop(0, "rgba(176,140,80,0.45)");
                gr.add_color_stop(1, "rgba(176,140,80,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x0.min(x1), y0.min(y1), if x0 == x1 { w } else { 70.0 }, if y0 == y1 { h } else { (y1 - y0).abs() });
            }
            for _ in 0..20 {
                let at = (r.next() * w, r.next() * h);
                soft(
                    ctx,
                    at,
                    w * (0.02 + 0.08 * r.next()),
                    0.5 + 0.5 * r.next(),
                    r.next() * 3.0,
                    (170.0, 140.0, 90.0),
                    0.06 + 0.1 * r.next(),
                );
            }
            {
                let (cx, cy, rr) = (w * (0.55 + 0.35 * r.next()), h * (0.5 + 0.4 * r.next()), 34.0 + 6.0 * r.next());
                ctx.set_stroke_style("rgba(110,76,40,0.35)");
                ctx.set_line_width(3.0);
                ctx.begin_path();
                ctx.arc(cx, cy, rr, 0.4, 6.0);
                ctx.stroke();
            }
            ctx.set_fill_style("rgba(124,116,100,0.22)");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..250 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.3 * r.next()));
                ctx.fill_rect(x, y, 1.0 + r.next(), 1.0 + r.next());
            }
        })
    }

    /// The open edges: the leaves' edges in fine lines, browner than the page
    pub fn edges_tex() -> Tex {
        canvas_tex_opt(64, 64, false, |ctx, w, h| {
            let mut r = SplitMix64::new(0x4e45, 0x0ed9);
            ctx.set_fill_style("#c8bea6");
            ctx.fill_rect(0, 0, w, h);
            let mut y = 0.0;
            while y < h {
                ctx.set_fill_style(format!("rgba(80,68,50,{:.2})", 0.1 + 0.2 * r.next()));
                ctx.fill_rect(0, y, w, 0.8);
                y += 1.5 + r.next() * 2.0;
            }
        })
    }
}
