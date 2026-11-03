//! Batch `r12`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// R540 fire alarm pull: 4% chance per level. A single-action pull station
// flush on its box in the wall (batch `r06`'s, the fire alarms'): FIRE raised
// over the lever's well, PULL DOWN on the lever, the glass rod across it, and
// an addressable station's LED blinking as the panel polls it.
pub(super) fn place_fire_alarm_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.alarm_pull_positions = vec![]; // R541 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    // the pull station of batch `r06` (the fire alarms'), here flush on a box
    // in the wall: its cover's back (z 0.035 in its parts' frame) on the wall,
    // and the LED of an addressable station (batch `r12`)
    let alarm = group();
    let o = group();
    o.position().z = -0.008 - 0.035;
    alarm.add(&o);
    let red = lambert!({ map: &pull_paint_tex(), color: 0xffffff });
    let white = lambert!({ color: 0xc4bfb1 });
    let steel = phong!({ color: 0x6a6964, specular: 0x2c2c2c, shininess: 34 });
    let add = |name: &str, mat: &Mat, x: f64, y: f64, z: f64| {
        let m = mesh(pull_part(name, 0.1, 0.14, 0.03), mat);
        m.position().set(x, y, z);
        o.add(&m);
    };
    add("Real__pullBody", &red, 0.0, 0.0, 0.0);
    add("Real__pullLetters", &white, 0.0, 0.0, 0.0);
    add("Real__pullScrews", &steel, 0.0, 0.0, 0.0);
    add("Real__pullLock", &steel, 0.030, -0.050, 0.035 + 0.030 + 0.0035);
    // the lever up in its well, the glass rod across it in its lugs
    let pivot = group();
    pivot.position().set(0, 0.020, 0.035 + 0.030);
    o.add(&pivot);
    pivot.add(&mesh(pull_part("Real__pullHandle", 0.07, 0.03, 0.016), &red));
    pivot.add(&mesh(pull_part("Real__pullHandleText", 0.05, 0.008, 0.001), &white));
    add("Real__pullRod", &lambert!({ color: 0xc8d6d0, transparent: true, opacity: 0.5 }), 0.0, 0.002, 0.035 + 0.030 + 0.0045);
    // R541 the station's LED, dark between the panel's polls.
    let led_mat = basic_mat!({ color: 0x3a0e0a });
    let led = mesh(pull_part("Real__fireAlarmPullLed", 0.007, 0.007, 0.002), &led_mat);
    led.position().set(0.036, 0.052, 0.035 + 0.030);
    o.add(&led);
    // Its blink: a short red flash every four seconds (the old throb's phase).
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let k = (t * 0.25 + pulse_phase / (PI * 2.0)) % 1.0;
        led_mat.m().color.set_hex(if k < 0.035 { 0xd8402c } else { 0x3a0e0a });
    });
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&alarm, c, wall_x, wall_z, 1.50, 0.6);
    g.world.add(&alarm);
    // R541 track for beep audio.
    let p = alarm.pos();
    g.pp.alarm_pull_positions.push(v3(p.x, 1.50, p.z));
}

// R537 wall pay phone: 4% chance per level. A single-slot payphone of the Western
// kind (Blender batch `r12`): the stainless upper housing with its instruction
// card, coin entry, keypad and coin return, the black vault under it, the
// handset hung on the switchhook at its side on its armoured cord.
pub(super) fn place_wall_pay_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.pay_phone_positions = vec![]; // R542 reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let phone = group();
    // the parts are modelled with the wall at z = 0: the housing's back on it
    let o = group();
    o.position().z = -0.05;
    phone.add(&o);
    let steel = phong!({ map: &pay_steel_tex(), color: 0xa6a6a0, specular: 0x383836, shininess: 22 });
    o.add(&mesh(pay_part("Real__wallPayPhoneUpper", 0.2, 0.3, 0.135), &steel));
    o.add(&mesh(pay_part("Real__wallPayPhoneVault", 0.196, 0.23, 0.124), lambert!({ map: &pay_vault_tex(), color: 0x2e2e2b })));
    let chrome = phong!({ color: 0x8c8c88, specular: 0x5a5a58, shininess: 60 });
    o.add(&mesh(pay_part("Real__wallPayPhoneKeys", 0.07, 0.095, 0.009), &chrome));
    o.add(&mesh(pay_part("Real__wallPayPhoneTrim", 0.26, 0.3, 0.14), &chrome));
    let decal = |tex: &Tex| lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let legends = mesh(plane_geo!(0.075, 0.100), decal(&pay_keys_tex()));
    legends.position().set(0.0, 0.357, PP_D + 0.0058);
    o.add(&legends);
    let card = mesh(plane_geo!(0.110, 0.072), lambert!({ map: &pay_card_tex() }));
    card.position().set(PP_CARD.0, PP_CARD.1, PP_D - 0.0027);
    o.add(&card);
    let window = mesh(plane_geo!(0.110, 0.072), decal(&pay_window_tex()));
    window.position().set(PP_CARD.0, PP_CARD.1, PP_D - 0.0006);
    o.add(&window);
    let handset =
        mesh(pay_part("Real__wallPayPhoneHandset", 0.065, 0.23, 0.045), phong!({ color: 0x1e1d1c, specular: 0x262626, shininess: 18 }));
    handset.position().set(-0.1025, 0.358, 0.070);
    handset.rotation().y = -PI / 2.0;
    o.add(&handset);
    o.add(&mesh(pay_part("Real__wallPayPhoneCord", 0.03, 0.16, 0.02), phong!({ color: 0x7a7a75, specular: 0x404040, shininess: 40 })));
    let (wall_x, wall_z) = wall_xz(g, c, 0.050);
    mount(&phone, c, wall_x, wall_z, 0.7, 0.4);
    g.world.add(&phone);
    // R542 track for ring audio.
    let p = phone.pos();
    g.pp.pay_phone_positions.push(v3(p.x, 1.0, p.z));
}

/// A Blender part as modelled (a scale of one), or a box standing in for it
/// when the props are switched off.
fn pay_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

fn pull_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    pay_part(name, w, h, d)
}

/// The payphone's depth and its card's window (x, y, w, h) (`r12.py`, PP_*).
const PP_D: f64 = 0.135;

const PP_CARD: (f64, f64, f64, f64) = (-0.026, 0.474, 0.112, 0.074);

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! r12_tex {
    ($w:expr, $h:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, false, $paint)).clone())
    }};
}

/// A soft round blob (squashed by the context, never the ellipse: no rim).
fn pay_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, rgba: (u8, u8, u8, f64)) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({},{},{},{})", rgba.0, rgba.1, rgba.2, rgba.3));
    gr.add_color_stop(1, &format!("rgba({},{},{},0)", rgba.0, rgba.1, rgba.2));
    ctx.set_fill_style(gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Short straight scratches.
fn pay_scratches(ctx: &mut Canvas, w: f64, h: f64, n: usize, rgb: (u8, u8, u8), a0: f64, len: f64) {
    ctx.set_line_cap("round");
    for _ in 0..n {
        let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, len * (0.2 + rnd()));
        ctx.set_line_width(0.5 + rnd() * 0.7);
        ctx.set_stroke_style(&format!("rgba({},{},{},{})", rgb.0, rgb.1, rgb.2, a0 + rnd() * a0));
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
}

/// The upper housing's stainless (the canvas is its face, 0.204 x 0.294 m):
/// the grain down it, keyed scratches, the ghosts of stickers peeled off,
/// grime round the keys and the coin return, COIN RETURN stamped over it.
fn pay_steel_tex() -> Tex {
    r12_tex!(204, 294, |ctx, w, h| {
        ctx.set_fill_style("#e8e8e4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..800 {
            ctx.set_fill_style(&format!("rgba({},{},{},{})", if rnd() < 0.5 { 255 } else { 80 }, 250, 245, 0.05 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 0.6, 5.0 + rnd() * 30.0);
        }
        // metres on the face to the canvas
        let to = |x: f64, y: f64| ((x + 0.102) / 0.204 * w, (0.530 - y) / 0.294 * h);
        // sticker ghosts: glue gone grey with dirt, a torn corner left
        for &(x, y, sw, sh) in &[(0.040, 0.300, 34.0, 22.0), (-0.085, 0.300, 26.0, 40.0)] {
            let (px, py) = to(x, y);
            ctx.set_fill_style("rgba(120,112,96,0.30)");
            ctx.fill_rect(px, py, sw, sh);
            ctx.set_fill_style("rgba(220,210,180,0.75)");
            ctx.fill_rect(px, py, 6, 5);
        }
        // grime where hands go
        let (kx, ky) = to(0.0, 0.357);
        pay_blob(ctx, kx, ky, 50.0, 1.3, (60, 54, 46, 0.30));
        let (rx, ry) = to(0.060, 0.266);
        pay_blob(ctx, rx, ry, 26.0, 1.0, (50, 44, 36, 0.35));
        let (cx, cy) = to(0.064, 0.480);
        pay_blob(ctx, cx, cy, 20.0, 1.0, (50, 44, 36, 0.25));
        // stamped labels
        ctx.set_fill_style("rgba(40,40,38,0.75)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 7px Helvetica");
        let (lx, ly) = to(0.060, 0.290);
        ctx.fill_text("COIN RETURN", lx, ly);
        let (dx, dy) = to(0.064, 0.448);
        ctx.fill_text("25¢ 10¢ 5¢", dx, dy);
        // keyed scratches: long, wandering, through the grain
        ctx.set_line_cap("round");
        for _ in 0..7 {
            let (mut x, mut y) = (rnd() * w, rnd() * h);
            ctx.set_line_width(0.8);
            ctx.set_stroke_style(&format!("rgba(70,68,64,{})", 0.25 + rnd() * 0.25));
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..6 {
                x += (rnd() - 0.5) * 30.0;
                y += (rnd() - 0.5) * 18.0;
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        pay_scratches(ctx, w, h, 40, (60, 58, 54), 0.12, 16.0);
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(130,120,100,0.3)");
        gd.add_color_stop(0.06, "rgba(130,120,100,0)");
        gd.add_color_stop(0.9, "rgba(50,44,36,0)");
        gd.add_color_stop(1, "rgba(50,44,36,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// The vault's black enamel: chipped to grey primer and the steel at its
/// edges and round the lock, rust weeping from the chips, kicked at the foot.
fn pay_vault_tex() -> Tex {
    r12_tex!(160, 190, |ctx, w, h| {
        ctx.set_fill_style("#f0f0ec");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            ctx.set_fill_style(&format!("rgba(60,60,58,{})", 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        // chips: pale primer and bare steel, at the edges and the lock
        for k in 0..30 {
            let (x, y) = if k < 8 {
                (w * 0.5 + (rnd() - 0.5) * 30.0, h * 0.2 + (rnd() - 0.5) * 30.0)
            } else if k % 2 == 0 {
                (rnd() * w, if rnd() < 0.5 { 3.0 } else { h - 3.0 })
            } else {
                (if rnd() < 0.5 { 3.0 } else { w - 3.0 }, rnd() * h)
            };
            let r = 1.5 + rnd() * 4.0;
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(255,255,255,0.9)" } else { "rgba(190,150,120,0.9)" });
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.4 + rnd() * 0.6), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // scuffed from kicks at the foot
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(255,255,255,0.0)");
        gd.add_color_stop(0.7, "rgba(255,255,255,0.0)");
        gd.add_color_stop(1, "rgba(255,255,255,0.35)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 40, (255, 255, 255), 0.25, 18.0);
    })
}

/// The instruction card (0.112 x 0.074 m): how to call, what it costs, the
/// emergency and the operator, this phone's number (555-01xx, kept for
/// fiction), sun-faded and foxed.
fn pay_card_tex() -> Tex {
    r12_tex!(336, 222, |ctx, w, h| {
        ctx.set_fill_style("#e3dfd2");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#2c4a6a");
        ctx.fill_rect(0, 0, w, 34);
        ctx.set_fill_style("#efebe0");
        ctx.set_text_align("left");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 19px Helvetica");
        ctx.fill_text("LOCAL CALLS 35¢", 12, 18);
        ctx.set_fill_style("#22201c");
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text("1  LIFT RECEIVER", 12, 52);
        ctx.fill_text("2  DEPOSIT COINS — LISTEN FOR DIAL TONE", 12, 70);
        ctx.fill_text("3  DIAL NUMBER", 12, 88);
        ctx.set_font("11px Helvetica");
        ctx.fill_text("COINS RETURNED IF CALL IS NOT COMPLETED", 12, 108);
        ctx.fill_text("LONG DISTANCE: DIAL 1 + AREA CODE + NUMBER", 12, 124);
        ctx.set_fill_style("#8a2a1e");
        ctx.fill_rect(10, 136, w - 20.0, 30);
        ctx.set_fill_style("#efebe0");
        ctx.set_font("bold 15px Helvetica");
        ctx.fill_text("EMERGENCY  911  NO COINS NEEDED", 18, 151);
        ctx.set_fill_style("#22201c");
        ctx.set_font("12px Helvetica");
        ctx.fill_text("OPERATOR  0          REPAIR  611", 12, 182);
        ctx.set_font("bold 12px Helvetica");
        ctx.fill_text("THIS PHONE  (555) 555-0182", 12, 204);
        // faded from the top, foxed, a water stain from the corner
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(200,190,160,0.35)");
        gy.add_color_stop(1, "rgba(120,100,60,0.18)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..24 {
            pay_blob(ctx, rnd() * w, rnd() * h, 3.0 + rnd() * 7.0, 1.0, (130, 90, 40, 0.08 + rnd() * 0.12));
        }
        pay_blob(ctx, w, h, 80.0, 1.0, (120, 90, 50, 0.28));
    })
}

/// What is printed on the keys, black in their chrome: the numeral and its
/// letters (the canvas spans the keypad, 0.075 x 0.100 m, 3 px a mm).
fn pay_keys_tex() -> Tex {
    r12_tex!(225, 300, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let keys = [
            ("1", ""),
            ("2", "ABC"),
            ("3", "DEF"),
            ("4", "GHI"),
            ("5", "JKL"),
            ("6", "MNO"),
            ("7", "PQRS"),
            ("8", "TUV"),
            ("9", "WXYZ"),
            ("*", ""),
            ("0", "OPER"),
            ("#", ""),
        ];
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        for (i, (num, letters)) in keys.iter().enumerate() {
            let (col, row) = ((i % 3) as f64, (i / 3) as f64);
            let cx = (0.0375 - 0.026 + col * 0.026) * 3000.0;
            let cy = (0.050 - (0.396 - 0.357) + row * 0.026) * 3000.0;
            ctx.set_fill_style(&format!("rgba(26,25,24,{})", 0.92 - rnd() * 0.4));
            if !letters.is_empty() {
                ctx.set_font("bold 8px Helvetica");
                ctx.fill_text(letters, cx, cy - 6.0);
            }
            ctx.set_font("bold 19px Helvetica");
            ctx.fill_text(num, cx, cy + if letters.is_empty() { 8.0 } else { 14.0 });
        }
    })
}

/// The card's clear window: scratched and clouded, a crack from a corner.
fn pay_window_tex() -> Tex {
    r12_tex!(168, 111, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(200,195,180,0.18)");
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 50, (235, 235, 230), 0.18, 26.0);
        ctx.set_stroke_style("rgba(240,240,235,0.6)");
        ctx.set_line_width(0.8);
        ctx.begin_path();
        ctx.move_to(w, 0);
        ctx.line_to(w * 0.82, h * 0.18);
        ctx.line_to(w * 0.74, h * 0.22);
        ctx.line_to(w * 0.6, h * 0.41);
        ctx.stroke();
        for &(x, y) in &[(0.0, 0.0), (w, h), (0.0, h)] {
            pay_blob(ctx, x, y, 30.0, 1.0, (150, 140, 120, 0.3));
        }
    })
}

/// The pull station's cover: red enamel over die-cast, rubbed to the metal at
/// its edges and round the lever's well where hands go, grimy, a few chips
/// (as batch `r06`'s fire alarms; laid on its front by the part's box
/// projection: the whole texture is the face).
fn pull_paint_tex() -> Tex {
    r12_tex!(128, 170, |ctx, w, h| {
        ctx.set_fill_style("#8a1c14");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            ctx.set_fill_style(&format!("rgba({},0,0,{})", if rnd() < 0.5 { 40 } else { 200 }, 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
        }
        ctx.set_stroke_style("rgba(150,120,110,0.35)");
        ctx.set_line_width(3);
        ctx.stroke_rect(1.5, 1.5, w - 3.0, h - 3.0);
        for &(x, y) in &[(3.0, 3.0), (w - 4.0, 3.0), (3.0, h - 4.0), (w - 4.0, h - 4.0), (w * 0.5, h - 3.0), (3.0, h * 0.4)] {
            ctx.set_fill_style("rgba(120,116,110,0.85)");
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 4.0, 1.5 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        pay_blob(ctx, w * 0.5, h * 0.55, w * 0.6, 1.0, (30, 20, 16, 0.30));
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(160,150,130,0.22)");
        gd.add_color_stop(0.08, "rgba(160,150,130,0)");
        gd.add_color_stop(0.8, "rgba(40,30,24,0)");
        gd.add_color_stop(1, "rgba(40,30,24,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        pay_scratches(ctx, w, h, 14, (170, 140, 130), 0.2, 16.0);
    })
}
