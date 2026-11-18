//! The signs' pictures, painted on the engine's canvas (CoreText, the
//! system's Korean face): the green way-out sign, the blue plates that
//! number the walkway's escape points and give the way out, the portal's
//! name, the gate's notice, the lane signal's arrow. Drawn here, not
//! photographed: the set's own shapes and words. Every picture is a
//! function of its arguments only (no random draw).
use engine::prelude::*;

const FACE: &str = "'Apple SD Gothic Neo'";

fn tex(c: &Canvas) -> Tex {
    let t = c.to_tex();
    {
        let mut tt = t.t();
        tt.color_space = ColorSpace::Srgb;
        tt.flip_y = false;
        tt.anisotropy = 8;
    }
    t
}

/// A figure running to the right, in white strokes, its head at (x, y),
/// `s` its height.
fn runner(c: &mut Canvas, x: f64, y: f64, s: f64) {
    c.set_fill_style("#f4f6f2");
    c.begin_path();
    c.arc(x, y, s * 0.085, 0.0, std::f64::consts::TAU);
    c.fill();
    c.set_stroke_style("#f4f6f2");
    c.set_line_width(s * 0.085);
    c.set_line_cap("round");
    c.set_line_join("round");
    // torso, leaning into the run
    let (hx, hy) = (x - s * 0.06, y + s * 0.42);
    c.begin_path();
    c.move_to(x - s * 0.01, y + s * 0.12);
    c.line_to(hx, hy);
    c.stroke();
    // arms
    c.begin_path();
    c.move_to(x - s * 0.18, y + s * 0.36);
    c.line_to(x - s * 0.13, y + s * 0.2);
    c.line_to(x + s * 0.06, y + s * 0.18);
    c.line_to(x + s * 0.17, y + s * 0.3);
    c.stroke();
    // legs: one forward, one back
    c.begin_path();
    c.move_to(x + s * 0.2, y + s * 0.84);
    c.line_to(x + s * 0.13, y + s * 0.63);
    c.line_to(hx, hy);
    c.line_to(x - s * 0.2, y + s * 0.6);
    c.line_to(x - s * 0.33, y + s * 0.66);
    c.stroke();
}

/// The green way-out sign: a figure running to a doorway, an arrow under.
pub fn exit_sign() -> Tex {
    let (w, h) = (256.0, 256.0);
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#0d7f45");
    c.fill_rect(0, 0, w, h);
    // a thin white border, as on the enamelled plates
    c.set_stroke_style("#e8ece6");
    c.set_line_width(5);
    c.stroke_rect(9, 9, w - 18.0, h - 18.0);
    // the doorway
    c.set_fill_style("#f4f6f2");
    c.fill_rect(170, 34, 52, 128);
    c.set_fill_style("#0d7f45");
    c.fill_rect(180, 44, 32, 118);
    runner(&mut c, 112.0, 46.0, 120.0);
    // the arrow
    c.set_fill_style("#f4f6f2");
    c.begin_path();
    c.move_to(44, 200);
    c.line_to(170, 200);
    c.line_to(170, 184);
    c.line_to(212, 210);
    c.line_to(170, 236);
    c.line_to(170, 220);
    c.line_to(44, 220);
    c.close_path();
    c.fill();
    tex(&c)
}

/// The blue plate at an escape point: its number, and how far the way out
/// is each way.
pub fn info_plate(number: i64, back_m: i64, ahead_m: i64) -> Tex {
    let (w, h) = (256.0, 512.0);
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#1b2f8c");
    c.fill_rect(0, 0, w, h);
    c.set_stroke_style("#dfe4f2");
    c.set_line_width(4);
    c.stroke_rect(8, 8, w - 16.0, h - 16.0);
    c.set_fill_style("#f2f4fa");
    c.set_text_align("center");
    c.set_text_baseline("middle");
    c.set_font(&format!("bold 30px {FACE}"));
    c.fill_text("피난연결", w / 2.0, 52);
    c.fill_text("통로", w / 2.0, 90);
    c.set_font(&format!("bold 190px {FACE}"));
    c.fill_text(&number.to_string(), w / 2.0, 236);
    c.set_fill_style("#dfe4f2");
    c.fill_rect(28, 340, w - 56.0, 3);
    c.set_fill_style("#f2f4fa");
    c.set_font(&format!("bold 30px {FACE}"));
    c.fill_text(&format!("← {back_m}m"), w / 2.0, 392);
    c.fill_text(&format!("{ahead_m}m →"), w / 2.0, 446);
    tex(&c)
}

/// The portal's name, cut in the granite plate.
pub fn name_plate() -> Tex {
    let (w, h) = (1024.0, 156.0);
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#1c1c1b");
    c.fill_rect(0, 0, w, h);
    c.set_text_align("center");
    c.set_text_baseline("middle");
    c.set_fill_style("#d9d4c6");
    c.set_font(&format!("bold 82px {FACE}"));
    c.fill_text("자 하 문 터 널", w / 2.0, 62);
    c.set_font(&format!("28px {FACE}"));
    c.fill_text("JAHAMUN  TUNNEL      L = 485 m", w / 2.0, 128);
    tex(&c)
}

/// The gate's notice: no way through at night.
pub fn gate_notice() -> Tex {
    let (w, h) = (384.0, 256.0);
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#efefe9");
    c.fill_rect(0, 0, w, h);
    c.set_fill_style("#c21d16");
    c.fill_rect(0, 0, w, 70);
    c.set_text_align("center");
    c.set_text_baseline("middle");
    c.set_fill_style("#ffffff");
    c.set_font(&format!("bold 48px {FACE}"));
    c.fill_text("출 입 금 지", w / 2.0, 36);
    c.set_fill_style("#222222");
    c.set_font(&format!("bold 30px {FACE}"));
    c.fill_text("야간 보행로 폐쇄", w / 2.0, 120);
    c.set_font(&format!("26px {FACE}"));
    c.fill_text("23:00 ~ 05:00", w / 2.0, 166);
    c.set_font(&format!("20px {FACE}"));
    c.fill_text("종로구청 도로과", w / 2.0, 216);
    tex(&c)
}

/// The lane signal's face: a green arrow pointing down at the lane.
pub fn lane_arrow() -> Tex {
    let (w, h) = (128.0, 128.0);
    let mut c = Canvas::new(w as u32, h as u32);
    c.set_fill_style("#050605");
    c.fill_rect(0, 0, w, h);
    c.set_fill_style("#7dffa8");
    c.begin_path();
    c.move_to(50, 18);
    c.line_to(78, 18);
    c.line_to(78, 66);
    c.line_to(102, 66);
    c.line_to(64, 110);
    c.line_to(26, 66);
    c.line_to(50, 66);
    c.close_path();
    c.fill();
    tex(&c)
}
