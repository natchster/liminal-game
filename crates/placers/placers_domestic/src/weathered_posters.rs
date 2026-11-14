//! `place_weathered_posters`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;
use placers_kit::wall_plates::part as real_part;

// Weathered posters: 0-2 per level on random walls at eye level
// (1.2-1.9 m) (Blender family `wall_hole_pipe_run_posters_and_hooks`): what offices and halls put up (a safety
// poster, a dance's bill, a bake sale, a recycling notice, flyers with their
// tear-off tabs), printed as real ones are, sun-faded and foxed, taped at
// the corners, one corner curled off the wall and one torn away under its
// tape. Now and then one is behind the glass of a clip frame, and the glass
// is broken.
pub(crate) fn place_weathered_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.55 {
        return;
    } // 45% chance of posters
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // which poster (the old texture choice's draw)
        let design = ((rnd() * POSTER_DESIGNS as f64).floor() as usize).min(POSTER_DESIGNS - 1);
        // Per-weather tint, multiplied on the print
        let poster_tint = poster_tint_by_weather(g.weather_key());
        // how long the sun has had it (the old opacity's draw)
        let faded = rnd() < 0.5;
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.01);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.01);
        let mid_height = 1.2 + rnd() * 0.7; // 1.2 - 1.9 m
        let poster = group();
        if c.dx.abs() == 1 {
            poster.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            poster.position().set(wall_x, mid_height, wall_z + (rnd() - 0.5) * 1.4);
        } else {
            poster.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            poster.position().set(wall_x + (rnd() - 0.5) * 1.4, mid_height, wall_z);
        }
        // Slight tilt: hung by hand.
        let tilt = (rnd() - 0.5) * 0.06;
        // 8% chance: in a clip frame whose glass is broken
        let framed = rnd() < 0.08;
        let sheet = build_poster(design, faded, poster_tint, framed);
        sheet.rotation().z = tilt;
        sheet.position().z = -0.01;
        poster.add(&sheet);
        g.world.add(&poster);
        if framed {
            // 3-5 pieces of its glass on the floor beneath
            let glass = lambert!({ color: 0xb8c6c0, transparent: true, opacity: 0.45 });
            let ps_count = 3 + (rnd() * 3.0).floor() as i32;
            let pp = poster.pos();
            let th = poster.rot().y;
            let (along, out) = ((th.dcos(), -th.dsin()), (th.dsin(), th.dcos()));
            for _ in 0..ps_count {
                let sw = 0.014 + rnd() * 0.022;
                let shh = 0.018 + rnd() * 0.022;
                let turn = rnd() * PI * 2.0;
                let sh_ang = rnd() * PI * 2.0;
                let sh_rad = 0.10 + rnd() * 0.35;
                let shard = mesh(real_part("Real__glassShard", 0.03, 0.003, 0.02), &glass);
                let k = ((sw + shh) / 0.05).clamp(0.85, 1.15);
                shard.scale().set(k, k, k);
                shard.rotation().y = turn;
                // in front of the wall, not in it
                let (a, n) = (sh_ang.dcos() * sh_rad, 0.02 + sh_ang.dsin().abs() * sh_rad);
                shard.position().set(pp.x + along.0 * a + out.0 * n, 0.001, pp.z + along.1 * a + out.1 * n);
                g.world.add(&shard);
            }
        }
        i += 1;
    }
}

/// Poster tint per weather (else the normal entry)
fn poster_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa0b0c0, 0xe0c890, 0xb08080, 0xc0b090, 0xd8ccb0, 0xffffff)
}

/// How many posters there are to choose from.
const POSTER_DESIGNS: usize = 6;

/// Each design's sheet (0 A2, 1 tabloid, 2 letter).
const POSTER_SHEET: [usize; POSTER_DESIGNS] = [0, 0, 1, 1, 2, 2];

/// Sheet sizes (m), the corner each part has curled and the corner torn
/// away under its tape (0 bottom left, 1 bottom right, 2 top right, 3 top left).
const SHEETS: [(&str, f64, f64, usize, usize); 3] =
    [("Real__posterA2", 0.420, 0.594, 2, 0), ("Real__posterTabloid", 0.279, 0.432, 0, 2), ("Real__posterLetter", 0.216, 0.279, 1, 3)];

/// A poster on the wall: its group's origin on the wall's face at the
/// sheet's middle, +z out. `framed`: in a clip frame, flat behind its glass
/// (broken), the clips at its edges.
fn build_poster(design: usize, faded: bool, tint: u32, framed: bool) -> Obj {
    let pg_ = group();
    let (part, w, h, curl, _) = SHEETS[POSTER_SHEET[design]];
    let paper = lambert!({ map: &poster_tex(design, faded, framed), color: tint, alphaTest: 0.5 });
    let corners = [(-w / 2.0, -h / 2.0), (w / 2.0, -h / 2.0), (w / 2.0, h / 2.0), (-w / 2.0, h / 2.0)];
    if framed {
        let back = mesh(box_geo!(w, h, 0.003), lambert!({ color: 0x4a3a2a }));
        back.position().z = 0.0015;
        pg_.add(&back);
        let sheet = mesh(plane_geo!(w, h), &paper);
        sheet.position().z = 0.0032;
        pg_.add(&sheet);
        let glass = mesh(plane_geo!(w, h), lambert!({ map: &cracked_pane_tex(), transparent: true, depthWrite: false }));
        glass.position().z = 0.0046;
        pg_.add(&glass);
        let clip_mat = phong!({ color: 0x8a8a86, specular: 0x333333, shininess: 40 });
        for (cx, cy, rz) in [(0.0, h / 2.0, 0.0), (0.0, -h / 2.0, PI), (-w / 2.0, h * 0.2, PI / 2.0), (w / 2.0, -h * 0.2, -PI / 2.0)] {
            let cl = mesh(real_part("Real__posterClip", 0.016, 0.02, 0.012), &clip_mat);
            cl.rotation().z = rz;
            cl.position().set(cx, cy, 0.0);
            pg_.add(&cl);
        }
        return pg_;
    }
    let sheet = mesh(real_part(part, w, h, 0.002), &paper);
    pg_.add(&sheet);
    let back = mesh(real_part(&format!("{part}Back"), w, h, 0.002), lambert!({ color: 0xb5ad98 }));
    pg_.add(&back);
    // masking tape at the corners: over the paper where it holds, on the
    // wall alone where the corner curled away from it
    let tape = lambert!({ color: 0x9c8762 });
    for (k, &(cx, cy)) in corners.iter().enumerate() {
        let t = mesh(real_part("Real__posterTape", 0.048, 0.019, 0.0005), &tape);
        let (ix, iy) = (-cx.signum() * 0.008, -cy.signum() * 0.008);
        t.position().set(cx + ix, cy + iy, if k == curl { 0.0004 } else { 0.0022 });
        t.rotation().z = if (k % 2 == 0) == (cx * cy > 0.0) { PI / 4.0 } else { -PI / 4.0 };
        pg_.add(&t);
    }
    pg_
}

/// What a poster says, as real ones are set: title, body, small print, and
/// its wear (sun, damp, foxing, a fold, hands at its foot); the torn corner
/// and torn-off tabs cut away (alpha). Nothing about the place, no name.
fn poster_tex(design: usize, faded: bool, framed: bool) -> Tex {
    thread_local! { static T: std::cell::RefCell<std::collections::BTreeMap<(usize, bool, bool), Tex>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) }; }
    if let Some(t) = T.with(|c| c.borrow().get(&(design, faded, framed)).cloned()) {
        return t;
    }
    let (_, sw, sh, _, torn) = SHEETS[POSTER_SHEET[design]];
    let ppm = 1200.0; // pixels a metre
    let (pw, ph) = ((sw * ppm).round(), (sh * ppm).round());
    let t = canvas_tex_opt(pw, ph, false, move |ctx, w, h| {
        let s = w / 504.0; // a design's units: the A2 sheet is 504 wide
        // the paper, yellowed
        ctx.set_fill_style(if faded { "#d9cfb2" } else { "#e2dcc7" });
        ctx.fill_rect(0, 0, w, h);
        let ink = |a: f64| format!("rgba(26,22,18,{a})");
        let fade = if faded { 0.55 } else { 0.8 };
        match design {
            0 => {
                // a safety poster: LIFT SAFELY, the figure squatting to the box, three rules
                ctx.set_fill_style(format!("rgba(34,92,52,{fade})"));
                ctx.fill_rect(0, 0, w, 120.0 * s);
                ctx.set_fill_style("rgba(240,236,224,0.95)");
                ctx.set_text_align("center");
                ctx.set_font(&format!("bold {}px Helvetica", 30.0 * s));
                ctx.fill_text_max("SAFETY FIRST", w / 2.0, 44.0 * s, w - 40.0 * s);
                ctx.set_font(&format!("bold {}px Helvetica", 58.0 * s));
                ctx.fill_text_max("LIFT SAFELY", w / 2.0, 102.0 * s, w - 40.0 * s);
                // the figure: squatting, back straight, the box held close
                ctx.set_stroke_style(ink(fade));
                ctx.set_fill_style(ink(fade));
                ctx.set_line_cap("round");
                ctx.set_line_join("round");
                ctx.set_line_width(16.0 * s);
                let (fx, fy) = (w * 0.44, 300.0 * s);
                ctx.begin_path();
                ctx.arc(fx, fy - 108.0 * s, 20.0 * s, 0, 7);
                ctx.fill();
                ctx.begin_path(); // the back, upright
                ctx.move_to(fx, fy - 80.0 * s);
                ctx.line_to(fx - 6.0 * s, fy);
                ctx.stroke();
                ctx.begin_path(); // the legs, bent
                ctx.move_to(fx - 6.0 * s, fy);
                ctx.line_to(fx + 46.0 * s, fy + 14.0 * s);
                ctx.line_to(fx + 40.0 * s, fy + 74.0 * s);
                ctx.move_to(fx - 6.0 * s, fy);
                ctx.line_to(fx + 30.0 * s, fy + 30.0 * s);
                ctx.line_to(fx + 8.0 * s, fy + 76.0 * s);
                ctx.stroke();
                ctx.begin_path(); // the arms to the box
                ctx.move_to(fx - 2.0 * s, fy - 66.0 * s);
                ctx.line_to(fx + 40.0 * s, fy - 30.0 * s);
                ctx.line_to(fx + 62.0 * s, fy - 10.0 * s);
                ctx.stroke();
                ctx.fill_rect(fx + 58.0 * s, fy - 34.0 * s, 70.0 * s, 56.0 * s);
                ctx.set_line_width(5.0 * s);
                ctx.begin_path();
                ctx.move_to(40.0 * s, fy + 82.0 * s);
                ctx.line_to(w - 40.0 * s, fy + 82.0 * s);
                ctx.stroke();
                // the tick by it
                ctx.set_stroke_style(format!("rgba(34,92,52,{fade})"));
                ctx.set_line_width(14.0 * s);
                ctx.begin_path();
                ctx.move_to(w * 0.80, fy - 70.0 * s);
                ctx.line_to(w * 0.84, fy - 40.0 * s);
                ctx.line_to(w * 0.92, fy - 110.0 * s);
                ctx.stroke();
                ctx.set_text_align("left");
                ctx.set_fill_style(ink(fade + 0.1));
                let rules = [
                    ("1", "Bend your knees, not your back."),
                    ("2", "Keep the load close to your body."),
                    ("3", "Turn with your feet. Never twist."),
                ];
                for (k, (n, line)) in rules.iter().enumerate() {
                    let y = 470.0 * s + k as f64 * 62.0 * s;
                    ctx.set_font(&format!("bold {}px Helvetica", 34.0 * s));
                    ctx.fill_text(n, 40.0 * s, y);
                    ctx.set_font(&format!("{}px Helvetica", 25.0 * s));
                    ctx.fill_text_max(line, 82.0 * s, y - 4.0 * s, w - 120.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 21.0 * s));
                ctx.set_text_align("center");
                ctx.fill_text_max("Ask for help with anything over 50 lb.", w / 2.0, 680.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 12.0 * s));
                ctx.fill_text_max("Report every injury, however small, to your supervisor the same day.", w / 2.0, 702.0 * s, w - 60.0 * s);
            }
            1 => {
                // a dance's bill: two inks, a slab serif
                ctx.set_fill_style(format!("rgba(150,40,30,{fade})"));
                ctx.fill_rect(24.0 * s, 24.0 * s, w - 48.0 * s, h - 48.0 * s);
                ctx.set_fill_style(if faded { "#d9cfb2" } else { "#e2dcc7" });
                ctx.fill_rect(36.0 * s, 36.0 * s, w - 72.0 * s, h - 72.0 * s);
                ctx.set_text_align("center");
                ctx.set_fill_style(ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Rockwell", 40.0 * s));
                ctx.fill_text_max("SATURDAY NIGHT", w / 2.0, 120.0 * s, w - 100.0 * s);
                ctx.set_fill_style(format!("rgba(150,40,30,{fade})"));
                ctx.set_font(&format!("bold {}px Rockwell", 128.0 * s));
                ctx.fill_text_max("DANCE", w / 2.0, 250.0 * s, w - 90.0 * s);
                ctx.set_fill_style(ink(fade + 0.1));
                ctx.fill_rect(80.0 * s, 290.0 * s, w - 160.0 * s, 4.0 * s);
                ctx.set_font(&format!("bold {}px Rockwell", 44.0 * s));
                ctx.fill_text_max("LIVE MUSIC", w / 2.0, 360.0 * s, w - 100.0 * s);
                ctx.set_font(&format!("{}px Rockwell", 34.0 * s));
                ctx.fill_text_max("8 PM till midnight", w / 2.0, 420.0 * s, w - 100.0 * s);
                ctx.fill_rect(80.0 * s, 450.0 * s, w - 160.0 * s, 4.0 * s);
                ctx.set_font(&format!("{}px Rockwell", 28.0 * s));
                ctx.fill_text_max("Admission $5 at the door", w / 2.0, 510.0 * s, w - 100.0 * s);
                ctx.fill_text_max("Refreshments", w / 2.0, 550.0 * s, w - 100.0 * s);
                ctx.set_fill_style(format!("rgba(150,40,30,{fade})"));
                ctx.set_font(&format!("bold {}px Rockwell", 30.0 * s));
                ctx.fill_text_max("EVERYONE WELCOME", w / 2.0, 640.0 * s, w - 100.0 * s);
            }
            2 => {
                // a bake sale, from an office printer
                ctx.set_text_align("center");
                ctx.set_fill_style(ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Futura", 92.0 * s));
                ctx.fill_text_max("BAKE SALE", w / 2.0, 120.0 * s, w - 50.0 * s);
                ctx.set_font(&format!("bold {}px Futura", 64.0 * s));
                ctx.fill_text_max("FRIDAY", w / 2.0, 240.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 52.0 * s));
                ctx.fill_text_max("11:30 – 2:00", w / 2.0, 320.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 36.0 * s));
                ctx.fill_text_max("in the break room", w / 2.0, 380.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Futura", 30.0 * s));
                ctx.fill_text_max("Cookies · Brownies · Pie · Coffee", w / 2.0, 500.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("italic {}px Times New Roman", 28.0 * s));
                ctx.fill_text_max("All proceeds go to the holiday fund", w / 2.0, 640.0 * s, w - 60.0 * s);
                ctx.set_font(&format!("{}px Times New Roman", 24.0 * s));
                ctx.fill_text_max("Thank you!", w / 2.0, 690.0 * s, w - 60.0 * s);
            }
            3 => {
                // a recycling notice
                ctx.set_fill_style(format!("rgba(30,60,110,{fade})"));
                ctx.fill_rect(0, 0, w, 150.0 * s);
                ctx.set_text_align("center");
                ctx.set_fill_style("rgba(240,236,224,0.95)");
                ctx.set_font(&format!("bold {}px Helvetica", 96.0 * s));
                ctx.fill_text_max("PAPER ONLY", w / 2.0, 110.0 * s, w - 40.0 * s);
                ctx.set_text_align("left");
                ctx.set_fill_style(ink(fade + 0.1));
                ctx.set_font(&format!("bold {}px Helvetica", 40.0 * s));
                ctx.fill_text("YES", 40.0 * s, 230.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 30.0 * s));
                for (k, l) in ["Office paper", "Envelopes and folders", "Newspapers, magazines", "Flattened boxes"].iter().enumerate() {
                    ctx.fill_text_max(l, 70.0 * s, 280.0 * s + k as f64 * 42.0 * s, w - 100.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 40.0 * s));
                ctx.fill_text("NO", 40.0 * s, 490.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 30.0 * s));
                for (k, l) in ["Food or drink", "Cups and lids", "Plastic wrap", "Batteries"].iter().enumerate() {
                    ctx.fill_text_max(l, 70.0 * s, 540.0 * s + k as f64 * 42.0 * s, w - 100.0 * s);
                }
                ctx.set_font(&format!("italic {}px Helvetica", 22.0 * s));
                ctx.set_text_align("center");
                ctx.fill_text_max("Thank you for keeping it clean.", w / 2.0, 735.0 * s, w - 60.0 * s);
            }
            _ => {
                // a flyer with tear-off tabs
                let (title, lines, tab): (&str, [&str; 3], &str) = if design == 4 {
                    (
                        "GUITAR LESSONS",
                        ["Beginners welcome. All ages.", "Acoustic and electric.", "$15 a half hour."],
                        "Guitar lessons  555-0186",
                    )
                } else {
                    (
                        "ROOM FOR RENT",
                        ["Quiet, furnished, by the bus line.", "$300 a month, utilities included.", "Non-smoker, please."],
                        "Room for rent  555-0142",
                    )
                };
                ctx.set_text_align("center");
                ctx.set_fill_style(ink(fade + 0.15));
                ctx.set_font(&format!("bold {}px Helvetica", 70.0 * s));
                ctx.fill_text_max(title, w / 2.0, 110.0 * s, w - 40.0 * s);
                ctx.set_font(&format!("{}px Helvetica", 40.0 * s));
                for (k, l) in lines.iter().enumerate() {
                    ctx.fill_text_max(l, w / 2.0, 210.0 * s + k as f64 * 60.0 * s, w - 40.0 * s);
                }
                ctx.set_font(&format!("bold {}px Helvetica", 44.0 * s));
                ctx.fill_text_max(if design == 4 { "Call 555-0186" } else { "Call 555-0142" }, w / 2.0, 420.0 * s, w - 40.0 * s);
                // the tabs: eight, cut apart, the number on each turned up the tab
                let ty = h - 150.0 * s;
                let n = 8;
                let tw = w / n as f64;
                ctx.set_stroke_style(ink(0.5));
                ctx.set_line_width(1.0 * s);
                ctx.set_line_dash(&[6.0 * s, 4.0 * s]);
                ctx.begin_path();
                ctx.move_to(0, ty);
                ctx.line_to(w, ty);
                ctx.stroke();
                ctx.set_line_dash(&[]);
                for k in 0..n {
                    ctx.save();
                    ctx.translate((k as f64 + 0.5) * tw + 9.0 * s, h - 8.0 * s);
                    ctx.rotate(-PI / 2.0);
                    ctx.set_text_align("left");
                    ctx.set_font(&format!("{}px Helvetica", 19.0 * s));
                    ctx.fill_text_max(tab, 0, 0, 136.0 * s);
                    ctx.restore();
                }
                // three already taken: torn off along the line, a stub left
                ctx.set_global_composite_operation("destination-out");
                for k in [1usize, 4, 5] {
                    let x0 = k as f64 * tw;
                    ctx.begin_path();
                    ctx.move_to(x0 + 1.0, h);
                    ctx.line_to(x0 + 1.0, ty + 18.0 * s + rnd() * 20.0 * s);
                    let mut x = x0 + 1.0;
                    while x < x0 + tw - 1.0 {
                        x += 4.0 * s;
                        ctx.line_to(x.min(x0 + tw - 1.0), ty + 10.0 * s + rnd() * 26.0 * s);
                    }
                    ctx.line_to(x0 + tw - 1.0, h);
                    ctx.close_path();
                    ctx.fill();
                }
                ctx.set_global_composite_operation("source-over");
                // the slits between the tabs that are left
                ctx.set_global_composite_operation("destination-out");
                for k in 1..n {
                    ctx.fill_rect(k as f64 * tw - 1.0, ty, 2, h - ty);
                }
                ctx.set_global_composite_operation("source-over");
            }
        }
        // ---- its years on the wall
        // the sun: the ink and paper bleached toward the light
        if faded {
            ctx.set_global_composite_operation("source-atop");
            ctx.set_fill_style("rgba(226,216,190,0.22)");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_global_composite_operation("source-over");
        }
        ctx.set_global_composite_operation("source-atop");
        // foxing: small brown spots
        for _ in 0..(40.0 * w / 504.0) as i32 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 1.0 + rnd() * 4.0 * s;
            ctx.set_fill_style(format!("rgba(120,80,40,{})", 0.10 + rnd() * 0.25));
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.7 + rnd() * 0.3), 0, 0, 7);
            ctx.fill();
        }
        // damp: a tide line across its foot
        let ty = h * (0.82 + rnd() * 0.08);
        let gr = ctx.create_linear_gradient(0, ty - 30.0 * s, 0, h);
        gr.add_color_stop(0, "rgba(140,110,60,0)");
        gr.add_color_stop(0.25, "rgba(140,110,60,0.18)");
        gr.add_color_stop(0.3, "rgba(110,84,44,0.25)");
        gr.add_color_stop(1, "rgba(140,110,60,0.12)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, ty - 30.0 * s, w, h);
        // the folds it was posted in (in half each way): a pale crease
        for (x0, y0, x1, y1) in [(w / 2.0, 0.0, w / 2.0, h), (0.0, h / 2.0, w, h / 2.0)] {
            ctx.set_stroke_style("rgba(255,250,235,0.20)");
            ctx.set_line_width(2.0 * s);
            ctx.begin_path();
            ctx.move_to(x0 + 1.0, y0);
            ctx.line_to(x1 + 1.0, y1);
            ctx.stroke();
            ctx.set_stroke_style("rgba(60,46,30,0.16)");
            ctx.set_line_width(1.2 * s);
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x1, y1);
            ctx.stroke();
        }
        // grime: hands at its foot, dust settled along its top edge
        for _ in 0..8 {
            let (x, y) = (rnd() * w, h * (0.75 + rnd() * 0.25));
            let r = (20.0 + rnd() * 40.0) * s;
            let gs = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gs.add_color_stop(0, "rgba(60,48,34,0.12)");
            gs.add_color_stop(1, "rgba(60,48,34,0)");
            ctx.set_fill_style(gs);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // the paper's edge darkened where it met the air
        for (x0, y0, x1, y1) in [(0.0, 0.0, 14.0, 0.0), (w, 0.0, w - 14.0, 0.0)] {
            let ge = ctx.create_linear_gradient(x0 * s.max(1.0), y0, x1 * s.max(1.0), y1);
            ge.add_color_stop(0, "rgba(120,96,60,0.25)");
            ge.add_color_stop(1, "rgba(120,96,60,0)");
            ctx.set_fill_style(ge);
            ctx.fill_rect(0, 0, w, h);
        }
        ctx.set_global_composite_operation("source-over");
        // the torn corner: gone but for the scrap still under its tape
        if !framed {
            let cx = if torn == 0 || torn == 3 { 0.0 } else { w };
            let cy = if torn == 0 || torn == 1 { h } else { 0.0 };
            let (dx, dy) = (if cx == 0.0 { 1.0 } else { -1.0 }, if cy == 0.0 { 1.0 } else { -1.0 });
            let ppm = w / sw;
            let (a, b) = (0.024 * ppm, 0.065 * ppm);
            ctx.set_global_composite_operation("destination-out");
            ctx.begin_path();
            // a jagged band from edge to edge, between the scrap and the sheet
            let steps = 18;
            let mut pts = vec![];
            for q in 0..=steps {
                let t = q as f64 / steps as f64;
                let r = b + (rnd() - 0.5) * 0.012 * ppm;
                pts.push((cx + dx * r * (1.0 - t), cy + dy * r * t));
            }
            for q in (0..=steps).rev() {
                let t = q as f64 / steps as f64;
                let r = a + (rnd() - 0.5) * 0.008 * ppm;
                pts.push((cx + dx * r * (1.0 - t), cy + dy * r * t));
            }
            for (q, &(x, y)) in pts.iter().enumerate() {
                if q == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.close_path();
            ctx.fill();
            ctx.set_global_composite_operation("source-over");
        }
    });
    T.with(|c| c.borrow_mut().insert((design, faded, framed), t.clone()));
    t
}

/// A clip frame's broken glass: the cracks from where it was struck, rings
/// round it, a few panes of it dirtier (transparent but for the cracks).
fn cracked_pane_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 360, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                // dust on the glass
                ctx.set_fill_style("rgba(200,196,180,0.10)");
                ctx.fill_rect(0, 0, w, h);
                let (cx, cy) = (w * 0.62, h * 0.38);
                let mut ends = vec![];
                for k in 0..11 {
                    let mut a = k as f64 / 11.0 * 6.283 + rnd() * 0.4;
                    let (mut x, mut y) = (cx, cy);
                    ctx.set_stroke_style("rgba(235,240,236,0.55)");
                    ctx.set_line_width(1.0);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    let steps = 10 + (rnd() * 10.0) as i32;
                    for _ in 0..steps {
                        a += (rnd() - 0.5) * 0.35;
                        x += a.dcos() * 14.0;
                        y += a.dsin() * 14.0;
                        ctx.line_to(x, y);
                    }
                    ctx.stroke();
                    ends.push(a);
                }
                for r in [18.0, 44.0, 80.0] {
                    ctx.set_stroke_style("rgba(235,240,236,0.4)");
                    ctx.begin_path();
                    let n = 11;
                    for k in 0..=n {
                        let a = k as f64 / n as f64 * 6.283;
                        let rr = r * (0.8 + rnd() * 0.4);
                        if k == 0 {
                            ctx.move_to(cx + a.dcos() * rr, cy + a.dsin() * rr);
                        } else {
                            ctx.line_to(cx + a.dcos() * rr, cy + a.dsin() * rr);
                        }
                    }
                    ctx.stroke();
                }
                // the impact: crushed white
                let gr = ctx.create_radial_gradient(cx, cy, 0, cx, cy, 10);
                gr.add_color_stop(0, "rgba(240,240,236,0.7)");
                gr.add_color_stop(1, "rgba(240,240,236,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(cx - 10.0, cy - 10.0, 20, 20);
            })
        })
        .clone()
    })
}
