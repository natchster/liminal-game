//! `place_emergency_phones`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;
use placers_kit::wall_plates::blob;
use placers_kit::wall_plates::part;
use placers_kit::wall_plates::scratches;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! cached_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

// Emergency phone: 4% chance per level. A ring-down phone (Blender batch
// `r12`): the 2554-class wall set in red with a blank plate where the keys
// would be, EMERGENCY on it, its handset on the hook and the cord looping
// under it; the engraved sign above.
pub(crate) fn place_emergency_phones(g: &mut Game, _dist: &[i32]) {
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
    o.position().z = -0.09;
    phone.add(&o);
    let red = lambert!({ map: &em_plastic_tex(), color: 0x9a2a1e });
    o.add(&mesh(part("Real__wallPhoneHousing", 0.123, 0.232, 0.058), &red));
    o.add(&mesh(part("Real__emergencyPhonePlate", 0.056, 0.088, 0.004), lambert!({ map: &em_plate_tex() })));
    o.add(&mesh(part("Real__wallPhoneHook", 0.032, 0.03, 0.02), phong!({ color: 0x575752, specular: 0x3a3a3a, shininess: 22 })));
    let handset = mesh(part("Real__wallPhoneHandset", 0.06, 0.21, 0.04), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e }));
    handset.position().set(-0.026, 0.001, 0.058 - 0.004);
    o.add(&handset);
    o.add(&mesh(part("Real__wallPhoneCordOn", 0.07, 0.2, 0.05), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e })));
    // the sign screwed to the wall over it
    let sign = mesh(part("Real__emergencyPhoneSign", 0.2, 0.06, 0.003), lambert!({ map: &em_sign_tex() }));
    sign.position().y = 0.200;
    o.add(&sign);
    // box (local z −0.09…0) back on the wall, not half buried in it
    let (wall_x, wall_z) = wall_xz(g, c, 0.09);
    mount(&phone, c, wall_x, wall_z, 1.3, Some(0.4));
    g.world.add(&phone);
}

/// Red ABS gone dull: the housing's wear (the colour is the material's), hands
/// round the handset's seats, dust on top, grime at the foot.
fn em_plastic_tex() -> Tex {
    cached_tex!(128, 240, false, |ctx, w, h| {
        ctx.set_fill_style("#f2eeea");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(format!("rgba(70,60,56,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        for &(x, y, r, a) in &[(0.29, 0.16, 0.30, 0.26), (0.30, 0.83, 0.26, 0.2), (0.6, 0.45, 0.3, 0.1)] {
            blob(ctx, x * w, y * h, r * w, 1.3, (50, 36, 30, a));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,125,0.35)");
        gd.add_color_stop(0.06, "rgba(150,140,125,0)");
        gd.add_color_stop(0.9, "rgba(40,30,26,0)");
        gd.add_color_stop(1, "rgba(40,30,26,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        scratches(ctx, w, h, 24, (200, 170, 160), 0.15, 14.0);
    })
}

/// The handset's and its cord's wear: grime at the grip.
fn em_handset_tex() -> Tex {
    cached_tex!(64, 160, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece8");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(50,36,30,0.12)");
        gr.add_color_stop(0.5, "rgba(50,36,30,0.32)");
        gr.add_color_stop(1, "rgba(50,36,30,0.14)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        scratches(ctx, w, h, 10, (200, 170, 160), 0.15, 8.0);
    })
}

/// The ring-down set's plate (the whole canvas is its face): a printed card,
/// EMERGENCY over what to do, yellowed, a thumb's grime at its foot.
fn em_plate_tex() -> Tex {
    cached_tex!(112, 176, false, |ctx, w, h| {
        ctx.set_fill_style("#e4ddcc");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#8e2016");
        ctx.fill_rect(0, 0, w, 44);
        ctx.set_fill_style("#eee8da");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 17px Helvetica");
        ctx.fill_text_max("EMERGENCY", w / 2.0, 23, w - 10.0);
        ctx.set_fill_style("#2a2622");
        ctx.set_font("bold 12px Helvetica");
        for (i, l) in ["LIFT", "RECEIVER", "TO CALL"].iter().enumerate() {
            ctx.fill_text(l, w / 2.0, 66.0 + i as f64 * 15.0);
        }
        ctx.set_font("9px Helvetica");
        for (i, l) in ["STAY ON", "THE LINE", "UNTIL", "ANSWERED"].iter().enumerate() {
            ctx.fill_text(l, w / 2.0, 120.0 + i as f64 * 11.0);
        }
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(180,150,80,0.12)");
        gy.add_color_stop(1, "rgba(80,60,40,0.28)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        scratches(ctx, w, h, 10, (60, 50, 40), 0.12, 10.0);
    })
}

/// The engraved sign: red laminate cut through to its white core, the
/// letters' edges dirty, its face scratched and dusty.
fn em_sign_tex() -> Tex {
    cached_tex!(400, 120, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2218");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..600 {
            ctx.set_fill_style(format!("rgba(40,10,6,{})", 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        ctx.set_fill_style("#e9e4d8");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica");
        ctx.fill_text_max("EMERGENCY", w / 2.0, 42, w - 40.0);
        ctx.set_font("bold 30px Helvetica");
        ctx.fill_text_max("TELEPHONE", w / 2.0, 88, w - 60.0);
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,120,0.30)");
        gd.add_color_stop(0.25, "rgba(150,140,120,0.05)");
        gd.add_color_stop(1, "rgba(30,20,16,0.15)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        scratches(ctx, w, h, 30, (230, 200, 190), 0.12, 30.0);
    })
}
