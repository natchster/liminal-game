//! `place_elevator_panels`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;
use placers_kit::wall_plates::blob as r12_blob;
use placers_kit::wall_plates::part as r12_part;
use placers_kit::wall_plates::scratches as r12_scratches;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! r12_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

// Lift button panel: 4% chance per level. A car operating panel's
// faceplate (Blender batch `r12`): brushed stainless, twelve floor buttons in
// their bezels with the raised numeral and braille beside each, door open and
// close, the alarm, the fire service key switch; the lit buttons' rings glow.
pub(crate) fn place_elevator_panels(g: &mut Game, _dist: &[i32]) {
    g.pp.elevator_panel_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let panel = group();
    // the parts are modelled with the wall at z = 0: the plate's back on it
    let o = group();
    o.position().z = -0.008;
    panel.add(&o);
    let brushed = phong!({ map: &lift_plate_tex(), color: 0xb0b0ac, specular: 0x3c3c3a, shininess: 26 });
    o.add(&mesh(r12_part("Real__elevatorPanelPlate", 0.165, 0.36, 0.003), &brushed));
    let print = mesh(
        plane_geo!(0.165, 0.36),
        lambert!({ map: &lift_print_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().z = EP_T + 0.0002;
    o.add(&print);
    let polished = phong!({ color: 0x8e8e8a, specular: 0x66666a, shininess: 70 });
    o.add(&mesh(r12_part("Real__elevatorPanelBezels", 0.15, 0.32, 0.003), &polished));
    o.add(&mesh(r12_part("Real__elevatorPanelScrews", 0.15, 0.34, 0.002), &polished));
    o.add(&mesh(r12_part("Real__elevatorPanelKey", 0.016, 0.016, 0.004), phong!({ color: 0x9a8a62, specular: 0x4a4436, shininess: 50 })));
    o.add(&mesh(r12_part("Real__elevatorPanelRings", 0.13, 0.2, 0.001), lambert!({ color: 0x2e2d2b })));
    o.add(&mesh(r12_part("Real__elevatorPanelButtons", 0.13, 0.2, 0.003), phong!({ color: 0xa4a4a0, specular: 0x404040, shininess: 40 })));
    let numerals = mesh(
        plane_geo!(0.130, 0.210),
        lambert!({ map: &lift_numerals_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    numerals.position().set(0.009, -0.0125, EP_T + 0.0029);
    o.add(&numerals);
    o.add(&mesh(r12_part("Real__elevatorPanelAlarm", 0.027, 0.027, 0.004), lambert!({ color: 0x8c2318 })));
    let bell = mesh(
        plane_geo!(0.016, 0.016),
        lambert!({ map: &lift_bell_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    bell.position().set(EP_ALARM.0, EP_ALARM.1, EP_T + 0.0037);
    o.add(&bell);
    // 4×3 = 12 floor buttons, top row first; a 20% chance a button's ring is lit.
    let halo_mat = basic_mat!({ color: 0xffc274 });
    for by in 0..4 {
        for bx in 0..3 {
            if rnd() < 0.20 {
                let halo = mesh(r12_part("Real__elevatorPanelHalo", 0.022, 0.022, 0.001), &halo_mat);
                halo.position().set(EP_COL[bx], EP_ROW[by], EP_T + 0.0001);
                o.add(&halo);
            }
        }
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&panel, c, wall_x, wall_z, 1.30, Some(0.6));
    g.world.add(&panel);
    // Track for beep audio.
    let p = panel.pos();
    g.pp.elevator_panel_positions.push(v3(p.x, 1.30, p.z));
}

/// The lift panel's plate thickness, its columns and rows of floor buttons
/// (the top row first) and its alarm button (`r12.py`, EP_*).
const EP_T: f64 = 0.003;

const EP_COL: [f64; 3] = [-0.041, 0.009, 0.059];

const EP_ROW: [f64; 4] = [0.080, 0.035, -0.010, -0.055];

const EP_ALARM: (f64, f64) = (0.003, -0.150);

/// Brushed stainless: the grain across, fingers' smudges round the buttons,
/// the edges grimy (the whole canvas is the plate's face, 0.165 x 0.36 m).
fn lift_plate_tex() -> Tex {
    r12_tex!(200, 436, false, |ctx, w, h| {
        ctx.set_fill_style("#e6e6e2");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..900 {
            let y = rnd() * h;
            let x = rnd() * w;
            ctx.set_fill_style(&format!("rgba({},{},{},{})", if rnd() < 0.5 { 255 } else { 90 }, 255, 250, 0.05 + rnd() * 0.08));
            ctx.fill_rect(x, y, 6.0 + rnd() * 40.0, 0.6);
        }
        // smudges where hands go: over the buttons, heaviest on the lower floors
        let to = |x: f64, y: f64| ((x / 0.165 + 0.5) * w, (0.5 - y / 0.36) * h);
        for &(bx, by, a) in &[
            (-0.041, -0.055, 0.22),
            (0.009, -0.055, 0.25),
            (0.059, -0.055, 0.15),
            (0.009, 0.035, 0.12),
            (-0.022, -0.105, 0.2),
            (0.028, -0.105, 0.24),
            (0.003, -0.150, 0.1),
        ] {
            let (x, y) = to(bx, by);
            r12_blob(ctx, x, y, 18.0, 1.2, (70, 62, 52, a));
        }
        for _ in 0..14 {
            r12_blob(ctx, rnd() * w, h * (0.25 + rnd() * 0.6), 4.0 + rnd() * 6.0, 1.4, (80, 70, 60, 0.10 + rnd() * 0.08));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(120,110,95,0.25)");
        gd.add_color_stop(0.05, "rgba(120,110,95,0)");
        gd.add_color_stop(0.95, "rgba(50,44,36,0)");
        gd.add_color_stop(1, "rgba(50,44,36,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 30, (60, 58, 54), 0.12, 18.0);
    })
}

/// Braille: the dots of a digit (cells of the numeric indicator and the letter's).
fn braille_dots(c: char) -> &'static [u8] {
    match c {
        '#' => &[3, 4, 5, 6],
        '1' => &[1],
        '2' => &[1, 2],
        '3' => &[1, 4],
        '4' => &[1, 4, 5],
        '5' => &[1, 5],
        '6' => &[1, 2, 4],
        '7' => &[1, 2, 4, 5],
        '8' => &[1, 2, 5],
        '9' => &[2, 4],
        '0' => &[2, 4, 5],
        'B' => &[1, 2],
        'S' => &[2, 3, 4],
        'T' => &[2, 3, 4, 5],
        'A' => &[1],
        'L' => &[1, 2, 3],
        'M' => &[1, 3, 4],
        _ => &[],
    }
}

/// One Braille cell at (x, y), its dot pitch `p` px: raised dots, lit from
/// above (a light crescent over a shadow).
fn braille_cell(ctx: &mut Canvas, x: f64, y: f64, p: f64, dots: &[u8]) {
    for &d in dots {
        let (col, row) = (if d > 3 { 1.0 } else { 0.0 }, ((d - 1) % 3) as f64);
        let (cx, cy) = (x + col * p, y + row * p);
        ctx.set_fill_style("rgba(20,18,16,0.55)");
        ctx.begin_path();
        ctx.arc(cx + 0.4, cy + 0.5, p * 0.36, 0, 7);
        ctx.fill();
        ctx.set_fill_style("rgba(255,255,250,0.75)");
        ctx.begin_path();
        ctx.arc(cx - 0.2, cy - 0.3, p * 0.28, 0, 7);
        ctx.fill();
    }
}

/// What is on the plate beside the buttons, 2 px a mm: each floor's plate
/// (its numeral raised and its Braille under it) to the left of its button,
/// the star at the main floor, FIRE OPERATION round the key switch, OPEN and
/// CLOSE, ALARM, the capacity engraved at the foot.
fn lift_print_tex() -> Tex {
    r12_tex!(330, 720, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x / 0.165 + 0.5) * w, (0.5 - y / 0.36) * h);
        let names = [["9", "10", "11"], ["6", "7", "8"], ["3", "4", "5"], ["B", "1", "2"]];
        let cols = [-0.041, 0.009, 0.059];
        let rows = [0.080, 0.035, -0.010, -0.055];
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for (r, row) in names.iter().enumerate() {
            for (c, name) in row.iter().enumerate() {
                let (x, y) = to(cols[c] - 0.022, rows[r]);
                // the plate: black anodised, 13 x 24 mm
                ctx.set_fill_style("rgba(28,27,26,0.92)");
                ctx.fill_rect(x - 13.0, y - 24.0, 26, 48);
                ctx.set_fill_style("rgba(225,222,214,0.9)");
                ctx.set_font(if name.len() > 1 { "bold 15px Helvetica" } else { "bold 18px Helvetica" });
                ctx.fill_text(name, x, y - 9.0);
                // its Braille: the numeric indicator before a digit
                let mut cx = x - 9.0;
                if name.chars().all(|ch| ch.is_ascii_digit()) {
                    braille_cell(ctx, cx, y + 7.0, 4.6, braille_dots('#'));
                    cx += 10.0;
                }
                for ch in name.chars() {
                    braille_cell(ctx, cx, y + 7.0, 4.6, braille_dots(ch));
                    cx += 10.0;
                }
                if *name == "1" {
                    // the main entry floor's star
                    ctx.set_fill_style("rgba(225,222,214,0.9)");
                    ctx.set_font("bold 12px Helvetica");
                    ctx.fill_text("\u{2605}", x, y - 26.0 + 2.0);
                }
            }
        }
        ctx.set_fill_style("rgba(30,29,28,0.85)");
        ctx.set_font("bold 11px Helvetica");
        let (kx, ky) = to(0.040, 0.142);
        ctx.fill_text("FIRE OPERATION", to(-0.020, 0.150).0, to(0.0, 0.150).1);
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text("OFF", kx - 30.0, ky);
        ctx.fill_text("HOLD", kx, ky - 30.0);
        ctx.fill_text("ON", kx + 30.0, ky);
        // the fire hat pictogram under its words
        let (hx, hy) = to(-0.020, 0.128);
        ctx.begin_path();
        ctx.move_to(hx - 16.0, hy + 6.0);
        ctx.quadratic_curve_to(hx - 14.0, hy - 12.0, hx, hy - 12.0);
        ctx.quadratic_curve_to(hx + 14.0, hy - 12.0, hx + 16.0, hy + 6.0);
        ctx.close_path();
        ctx.fill();
        ctx.fill_rect(hx - 22.0, hy + 4.0, 44, 4);
        ctx.set_font("bold 9px Helvetica");
        for (i, l) in ["OPEN", "CLOSE"].iter().enumerate() {
            let (x, y) = to(if i == 0 { -0.022 } else { 0.028 }, -0.105);
            ctx.fill_text(l, x, y + 34.0);
        }
        let (ax, ay) = to(-0.032, -0.150);
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("ALARM", ax - 14.0, ay);
        ctx.set_font("bold 8px Helvetica");
        let (cx, cy) = to(0.0, -0.172);
        ctx.fill_text("CAPACITY 2500 LBS  16 PERSONS", cx, cy);
    })
}

/// The numerals engraved in the buttons and filled black, 2 px a mm over the
/// buttons' field (0.130 x 0.210 m, its middle at x 0.009, y -0.0125), and
/// the door buttons' arrows; rubbed paler where pressed most.
fn lift_numerals_tex() -> Tex {
    r12_tex!(260, 420, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x - 0.009 + 0.065) * 2000.0, (0.105 - (y + 0.0125)) * 2000.0);
        let names = [["9", "10", "11"], ["6", "7", "8"], ["3", "4", "5"], ["B", "1", "2"]];
        let cols = [-0.041, 0.009, 0.059];
        let rows = [0.080, 0.035, -0.010, -0.055];
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for (r, row) in names.iter().enumerate() {
            for (c, name) in row.iter().enumerate() {
                let (x, y) = to(cols[c], rows[r]);
                ctx.set_fill_style(&format!("rgba(24,22,20,{})", 0.9 - rnd() * 0.3));
                ctx.set_font(if name.len() > 1 { "bold 17px Helvetica" } else { "bold 21px Helvetica" });
                ctx.fill_text(name, x, y + 1.0);
            }
        }
        // the door buttons: open (outward arrows) and close (inward)
        ctx.set_fill_style("rgba(24,22,20,0.85)");
        for (i, &bx) in [-0.022, 0.028].iter().enumerate() {
            let (x, y) = to(bx, -0.105);
            ctx.fill_rect(x - 1.0, y - 8.0, 2, 16);
            for s in [-1.0, 1.0] {
                let dir = if i == 0 { s } else { -s };
                let tip = x + s * if i == 0 { 13.0 } else { 3.0 };
                ctx.begin_path();
                ctx.move_to(tip, y);
                ctx.line_to(tip - dir * 8.0, y - 6.0);
                ctx.line_to(tip - dir * 8.0, y + 6.0);
                ctx.close_path();
                ctx.fill();
            }
        }
    })
}

/// The alarm button's bell, white on its red.
fn lift_bell_tex() -> Tex {
    r12_tex!(64, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(236,230,220,0.92)");
        ctx.begin_path();
        ctx.move_to(14, 44);
        ctx.quadratic_curve_to(20, 40, 20, 28);
        ctx.quadratic_curve_to(20, 12, 32, 12);
        ctx.quadratic_curve_to(44, 12, 44, 28);
        ctx.quadratic_curve_to(44, 40, 50, 44);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.arc(32, 49, 5, 0, 7);
        ctx.fill();
        ctx.fill_rect(30, 7, 4, 6);
    })
}
