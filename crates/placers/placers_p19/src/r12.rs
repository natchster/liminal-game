//! Batch `r12`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Lift button panel: 4% chance per level. A car operating panel's
// faceplate (Blender batch `r12`): brushed stainless, twelve floor buttons in
// their bezels with the raised numeral and braille beside each, door open and
// close, the alarm, the fire service key switch; the lit buttons' rings glow.
pub(super) fn place_elevator_panels(g: &mut Game, _dist: &[i32]) {
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

// Wall extinguisher cabinet: 4% chance per level. A surface-mounted
// steel cabinet for a 5 lb extinguisher (Blender batch `r12`): white enamel
// gone yellow and chipped, a full pane in its door frame, FIRE EXTINGUISHER
// down the glass; in it the extinguisher stands on its foot, its hose in its
// band, the pin sealed, its tag hanging.
pub(super) fn place_extinguisher_cabinets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let cab = group();
    // the parts are modelled with the wall at z = 0: the tub's back on it
    let o = group();
    o.position().z = -0.04;
    cab.add(&o);
    let enamel = lambert!({ map: &ec_enamel_tex(), color: 0xd4cfc2 });
    o.add(&mesh(r12_part("Real__extCabinetTub", 0.205, 0.456, 0.134), &enamel));
    o.add(&mesh(r12_part("Real__extCabinetDoor", 0.205, 0.456, 0.022), &enamel));
    o.add(&mesh(r12_part("Real__extCabinetHardware", 0.21, 0.45, 0.04), phong!({ color: 0x75746f, specular: 0x3a3a38, shininess: 36 })));
    // the extinguisher, standing on the tub's floor, turned so its label
    // and gauge face out past the door's edge
    let ex = group();
    ex.position().set(-0.014, -EC_H / 2.0 + 0.0014, 0.073);
    ex.rotation().y = PI - 0.4;
    o.add(&ex);
    ex.add(&mesh(r12_part("Real__extCabinetExBody", 0.108, 0.24, 0.108), lambert!({ map: &ex_label_tex(), color: 0xffffff })));
    ex.add(&mesh(r12_part("Real__extCabinetExEnds", 0.108, 0.3, 0.108), lambert!({ color: 0x7e1c14 })));
    ex.add(&mesh(r12_part("Real__extCabinetExValve", 0.14, 0.07, 0.05), phong!({ color: 0x8a8984, specular: 0x505050, shininess: 44 })));
    ex.add(&mesh(r12_part("Real__extCabinetExSeal", 0.02, 0.02, 0.015), lambert!({ color: 0xb49a2e })));
    ex.add(&mesh(r12_part("Real__extCabinetExHose", 0.04, 0.2, 0.03), lambert!({ color: 0x1f1e1c })));
    let gauge = mesh(circle_geo!(0.0098, 20), lambert!({ map: &ex_gauge_tex() }));
    gauge.rotation().y = PI;
    gauge.position().set(0, EX_BODY + 0.034, -0.0252);
    ex.add(&gauge);
    let tag = mesh(plane_geo!(0.042, 0.078), lambert!({ map: &ex_tag_tex(), side: DoubleSide }));
    tag.rotation().set(0.08, PI + 0.25, 0.04);
    tag.position().set(0.030, EX_BODY - 0.040, -0.060);
    ex.add(&tag);
    // the pane, clear, and what is on it: the lettering, dust and old tape
    o.add(&mesh(
        r12_part("Real__extCabinetGlass", 0.169, 0.42, 0.003),
        lambert!({ color: 0xcdd9d4, transparent: true, opacity: 0.16, depthWrite: false }),
    ));
    let print = mesh(
        plane_geo!(0.169, 0.420),
        lambert!({ map: &ec_glass_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().z = EC_D - 0.0025 + 0.0002;
    o.add(&print);
    let (wall_x, wall_z) = wall_xz(g, c, 0.040);
    mount(&cab, c, wall_x, wall_z, 1.30, Some(0.4));
    g.world.add(&cab);
}

// Wall fuse box: 4% chance per level. A 6-space breaker panel on the
// surface (Blender batch `r12`): the grey enclosure, its front cover and dead
// front, the door hanging open, six single-pole breakers on or off, the
// circuits written on the card inside the door.
pub(super) fn place_fuse_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let fuse_box = group();
    // the parts are modelled with the wall at z = 0: the enclosure's back on it
    let o = group();
    o.position().z = -0.03;
    fuse_box.add(&o);
    let paint = lambert!({ map: &fb_enamel_tex(), color: 0x8e908b });
    o.add(&mesh(r12_part("Real__fuseBoxTub", 0.215, 0.32, 0.083), &paint));
    o.add(&mesh(r12_part("Real__fuseBoxCover", 0.229, 0.334, 0.012), &paint));
    o.add(&mesh(r12_part("Real__fuseBoxScrews", 0.22, 0.32, 0.002), phong!({ color: 0x77766f, specular: 0x333333, shininess: 30 })));
    let bakelite = lambert!({ color: 0x232220 });
    o.add(&mesh(r12_part("Real__fuseBoxBreakers", 0.13, 0.08, 0.02), &bakelite));
    let print = mesh(
        plane_geo!(FB_DOOR.0, FB_DOOR.1),
        lambert!({ map: &fb_print_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().set(FB_DOOR_C.0, FB_DOOR_C.1, FB_D + 0.0014);
    o.add(&print);
    // the door, hanging open on its hinge at the opening's left edge
    let hinge = group();
    hinge.position().set(FB_DOOR_C.0 - FB_DOOR.0 / 2.0, FB_DOOR_C.1, FB_D + 0.0035);
    hinge.rotation().y = -1.85;
    o.add(&hinge);
    hinge.add(&mesh(r12_part("Real__fuseBoxDoor", 0.15, 0.2, 0.006), &paint));
    let card = mesh(plane_geo!(0.112, 0.150), lambert!({ map: &fb_directory_tex(), side: DoubleSide }));
    card.rotation().y = PI;
    card.position().set(FB_DOOR.0 * 0.5, 0.0, -0.0014);
    hinge.add(&card);
    // six breakers, 1-3-5 down the left and 2-4-6 down the right, each on or off
    for i in 0..6 {
        let (col, row) = (i % 2, i / 2);
        let sw = mesh(r12_part("Real__fuseBoxHandle", 0.011, 0.018, 0.013), &bakelite);
        sw.position().set(FB_COL[col], FB_ROW[row], FB_D + 0.001);
        // Random tilt for on/off.
        sw.rotation().y = if rnd() < 0.5 { 0.3 } else { -0.3 };
        o.add(&sw);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.030);
    mount(&fuse_box, c, wall_x, wall_z, 1.50, Some(0.4));
    g.world.add(&fuse_box);
}

// Emergency phone: 4% chance per level. A ring-down phone (Blender batch
// `r12`): the 2554-class wall set in red with a blank plate where the keys
// would be, EMERGENCY on it, its handset on the hook and the cord looping
// under it; the engraved sign above.
pub(super) fn place_emergency_phones(g: &mut Game, _dist: &[i32]) {
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
    o.add(&mesh(r12_part("Real__wallPhoneHousing", 0.123, 0.232, 0.058), &red));
    o.add(&mesh(r12_part("Real__emergencyPhonePlate", 0.056, 0.088, 0.004), lambert!({ map: &em_plate_tex() })));
    o.add(&mesh(r12_part("Real__wallPhoneHook", 0.032, 0.03, 0.02), phong!({ color: 0x575752, specular: 0x3a3a3a, shininess: 22 })));
    let handset = mesh(r12_part("Real__wallPhoneHandset", 0.06, 0.21, 0.04), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e }));
    handset.position().set(-0.026, 0.001, 0.058 - 0.004);
    o.add(&handset);
    o.add(&mesh(r12_part("Real__wallPhoneCordOn", 0.07, 0.2, 0.05), lambert!({ map: &em_handset_tex(), color: 0x9a2a1e })));
    // the sign screwed to the wall over it
    let sign = mesh(r12_part("Real__emergencyPhoneSign", 0.2, 0.06, 0.003), lambert!({ map: &em_sign_tex() }));
    sign.position().y = 0.200;
    o.add(&sign);
    // box (local z −0.09…0) back on the wall, not half buried in it
    let (wall_x, wall_z) = wall_xz(g, c, 0.09);
    mount(&phone, c, wall_x, wall_z, 1.3, Some(0.4));
    g.world.add(&phone);
}

/// A Blender part of batch `r12` as modelled (a scale of one), or a box
/// standing in for it when the props are switched off.
fn r12_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// The lift panel's plate thickness, its columns and rows of floor buttons
/// (the top row first) and its alarm button (`r12.py`, EP_*).
const EP_T: f64 = 0.003;

const EP_COL: [f64; 3] = [-0.041, 0.009, 0.059];

const EP_ROW: [f64; 4] = [0.080, 0.035, -0.010, -0.055];

const EP_ALARM: (f64, f64) = (0.003, -0.150);

/// The extinguisher cabinet's height and depth, the extinguisher's cylinder (`r12.py`, EC_*, EX_*).
const EC_H: f64 = 0.456;

const EC_D: f64 = 0.156;

const EX_BODY: f64 = 0.300;

/// The breaker panel: the enclosure's depth, the door (w, h) and its middle,
/// the breakers' columns and rows (`r12.py`, FB_*).
const FB_D: f64 = 0.083;

const FB_DOOR: (f64, f64) = (0.150, 0.198);

const FB_DOOR_C: (f64, f64) = (0.0, -0.032);

const FB_COL: [f64; 2] = [-0.036, 0.036];

const FB_ROW: [f64; 3] = [0.000, -0.026, -0.052];

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! r12_tex {
    ($w:expr, $h:expr, $repeat:expr, $paint:expr) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

/// A soft round blob (squashed by the context, never the ellipse: no rim).
fn r12_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, rgba: (u8, u8, u8, f64)) {
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
fn r12_scratches(ctx: &mut Canvas, w: f64, h: f64, n: usize, rgb: (u8, u8, u8), a0: f64, len: f64) {
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

/// Red ABS gone dull: the housing's wear (the colour is the material's), hands
/// round the handset's seats, dust on top, grime at the foot.
fn em_plastic_tex() -> Tex {
    r12_tex!(128, 240, false, |ctx, w, h| {
        ctx.set_fill_style("#f2eeea");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(&format!("rgba(70,60,56,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        for &(x, y, r, a) in &[(0.29, 0.16, 0.30, 0.26), (0.30, 0.83, 0.26, 0.2), (0.6, 0.45, 0.3, 0.1)] {
            r12_blob(ctx, x * w, y * h, r * w, 1.3, (50, 36, 30, a));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,125,0.35)");
        gd.add_color_stop(0.06, "rgba(150,140,125,0)");
        gd.add_color_stop(0.9, "rgba(40,30,26,0)");
        gd.add_color_stop(1, "rgba(40,30,26,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 24, (200, 170, 160), 0.15, 14.0);
    })
}

/// The handset's and its cord's wear: grime at the grip.
fn em_handset_tex() -> Tex {
    r12_tex!(64, 160, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece8");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(50,36,30,0.12)");
        gr.add_color_stop(0.5, "rgba(50,36,30,0.32)");
        gr.add_color_stop(1, "rgba(50,36,30,0.14)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 10, (200, 170, 160), 0.15, 8.0);
    })
}

/// The ring-down set's plate (the whole canvas is its face): a printed card,
/// EMERGENCY over what to do, yellowed, a thumb's grime at its foot.
fn em_plate_tex() -> Tex {
    r12_tex!(112, 176, false, |ctx, w, h| {
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
        r12_scratches(ctx, w, h, 10, (60, 50, 40), 0.12, 10.0);
    })
}

/// The engraved sign: red laminate cut through to its white core, the
/// letters' edges dirty, its face scratched and dusty.
fn em_sign_tex() -> Tex {
    r12_tex!(400, 120, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2218");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..600 {
            ctx.set_fill_style(&format!("rgba(40,10,6,{})", 0.03 + rnd() * 0.05));
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
        r12_scratches(ctx, w, h, 30, (230, 200, 190), 0.12, 30.0);
    })
}

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

/// White baked enamel on steel, gone cream: chipped to the metal at the
/// corners and the door's edge, rust in the chips at the foot, grime where
/// the door is pulled, dust on top (the canvas is the cabinet's face).
fn ec_enamel_tex() -> Tex {
    r12_tex!(128, 284, false, |ctx, w, h| {
        ctx.set_fill_style("#f3efe4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(&format!("rgba(120,105,80,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(190,160,100,0.30)");
        gy.add_color_stop(0.5, "rgba(190,160,100,0.12)");
        gy.add_color_stop(1, "rgba(80,60,40,0.30)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        // the pull's grime (right edge, middle)
        r12_blob(ctx, w * 0.92, h * 0.5, w * 0.22, 2.0, (60, 50, 40, 0.3));
        // chips: grey metal, rust in the low ones
        for k in 0..22 {
            let edge = k % 4;
            let t = rnd();
            let (x, y) = match edge {
                0 => (t * w, 2.0 + rnd() * 4.0),
                1 => (t * w, h - 2.0 - rnd() * 5.0),
                2 => (2.0 + rnd() * 4.0, t * h),
                _ => (w - 2.0 - rnd() * 4.0, t * h),
            };
            let low = y > h * 0.7;
            let r = 1.5 + rnd() * 3.5;
            ctx.set_fill_style(if low { "rgba(120,62,30,0.85)" } else { "rgba(118,116,110,0.85)" });
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.5 + rnd() * 0.5), rnd() * 3.0, 0, 7);
            ctx.fill();
            if low {
                ctx.set_fill_style("rgba(130,70,30,0.25)");
                ctx.fill_rect(x - 1.0, y, 2.0, 6.0 + rnd() * 14.0);
            }
        }
        r12_scratches(ctx, w, h, 20, (90, 80, 70), 0.12, 16.0);
    })
}

/// The pane's print (the canvas is the pane, 0.169 x 0.420 m): FIRE
/// EXTINGUISHER in red vinyl down the glass, reading up, a letter or two
/// peeled; dust thick at the top and the corners, a hand's smear, old tape.
fn ec_glass_tex() -> Tex {
    r12_tex!(236, 588, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // the letters, turned to read up the pane
        ctx.save();
        ctx.translate(w * 0.5, h * 0.5);
        ctx.rotate(-PI / 2.0);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica");
        ctx.set_fill_style("rgba(150,30,22,0.95)");
        ctx.fill_text_max("FIRE EXTINGUISHER", 0, 0, h - 60.0);
        ctx.restore();
        // a peeled letter's ghost: some vinyl gone, its glue left
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..30 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.4 + rnd() * 0.6));
            ctx.begin_path();
            ctx.ellipse(w * (0.35 + rnd() * 0.3), h * (0.1 + rnd() * 0.8), 1.0 + rnd() * 4.0, 1.0 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_composite_operation("source-over");
        // dust: a film, thicker up top and in the corners
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(170,160,140,0.40)");
        gd.add_color_stop(0.3, "rgba(170,160,140,0.12)");
        gd.add_color_stop(1, "rgba(150,140,120,0.22)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        for &(x, y) in &[(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
            r12_blob(ctx, x, y, 60.0, 1.0, (160, 150, 130, 0.35));
        }
        // a hand's smear at the pull side, and a strip of old tape's glue
        r12_blob(ctx, w * 0.85, h * 0.5, 40.0, 1.6, (90, 80, 70, 0.18));
        ctx.set_fill_style("rgba(190,170,120,0.35)");
        ctx.fill_rect(w * 0.08, h * 0.06, 48, 14);
        r12_scratches(ctx, w, h, 16, (220, 220, 215), 0.15, 30.0);
    })
}

/// The extinguisher's wall (the canvas wraps it, u round from its back,
/// its front at the middle; v up its 0.238 m): red enamel, the label at the
/// front, scuffed where it stood among things, chipped low.
fn ex_label_tex() -> Tex {
    r12_tex!(512, 360, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2016");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..800 {
            ctx.set_fill_style(&format!("rgba({},20,12,{})", if rnd() < 0.5 { 40 } else { 180 }, 0.04 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        // the label: 0.10 m round, 0.15 m tall, its middle at u = 0.5
        let (lw, lh) = (152.0, 228.0);
        let (lx, ly) = (w / 2.0 - lw / 2.0, h * 0.5 - lh / 2.0 - 6.0);
        ctx.set_fill_style("#e2dccb");
        ctx.fill_rect(lx, ly, lw, lh);
        ctx.set_fill_style("#8a2016");
        ctx.fill_rect(lx, ly, lw, 30);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("#efe9dc");
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text_max("FIRE EXTINGUISHER", w / 2.0, ly + 15.0, lw - 8.0);
        ctx.set_fill_style("#24201c");
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text_max("MULTIPURPOSE DRY CHEMICAL", w / 2.0, ly + 41.0, lw - 8.0);
        // the classes: A in a triangle, B in a square, C in a circle
        let cy = ly + 66.0;
        for (i, (letter, col)) in [("A", "#3c6a3c"), ("B", "#8a2a20"), ("C", "#2e4a78")].iter().enumerate() {
            let cx = w / 2.0 - 40.0 + i as f64 * 40.0;
            ctx.set_fill_style(*col);
            ctx.begin_path();
            match i {
                0 => {
                    ctx.move_to(cx, cy - 14.0);
                    ctx.line_to(cx + 14.0, cy + 11.0);
                    ctx.line_to(cx - 14.0, cy + 11.0);
                    ctx.close_path();
                }
                1 => ctx.rect(cx - 12.0, cy - 12.0, 24, 24),
                _ => ctx.arc(cx, cy, 12.5, 0, 7),
            }
            ctx.fill();
            ctx.set_fill_style("#efe9dc");
            ctx.set_font("bold 13px Helvetica");
            ctx.fill_text(letter, cx, cy + if i == 0 { 3.0 } else { 1.0 });
        }
        ctx.set_fill_style("#24201c");
        ctx.set_font("bold 8px Helvetica");
        ctx.fill_text("RATING 2-A:10-B:C", w / 2.0, ly + 92.0);
        ctx.set_font("bold 9px Helvetica");
        ctx.fill_text("TO OPERATE", w / 2.0, ly + 108.0);
        ctx.set_text_align("left");
        ctx.set_font("8px Helvetica");
        for (i, l) in ["1  PULL PIN", "2  AIM AT BASE OF FIRE", "3  SQUEEZE LEVER", "4  SWEEP SIDE TO SIDE"].iter().enumerate() {
            ctx.fill_text(l, lx + 14.0, ly + 122.0 + i as f64 * 12.0);
        }
        ctx.set_text_align("center");
        ctx.set_font("7px Helvetica");
        for (i, l) in ["RECHARGE IMMEDIATELY AFTER ANY USE", "INSPECT MONTHLY", "CONTENTS UNDER PRESSURE"].iter().enumerate() {
            ctx.fill_text_max(l, w / 2.0, ly + 178.0 + i as f64 * 10.0, lw - 10.0);
        }
        ctx.fill_rect(lx + 18.0, ly + 210.0, lw - 36.0, 1);
        // the label yellowed and grimy, its edges lifting dark
        let gy = ctx.create_linear_gradient(0, ly, 0, ly + lh);
        gy.add_color_stop(0, "rgba(170,140,70,0.15)");
        gy.add_color_stop(1, "rgba(90,60,30,0.3)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(lx, ly, lw, lh);
        ctx.set_stroke_style("rgba(60,40,30,0.4)");
        ctx.set_line_width(1.5);
        ctx.stroke_rect(lx + 0.5, ly + 0.5, lw - 1.0, lh - 1.0);
        // scuffs, chips low down, dust settled on what faces up
        r12_scratches(ctx, w, h, 40, (200, 190, 180), 0.12, 20.0);
        for _ in 0..14 {
            ctx.set_fill_style("rgba(70,66,62,0.8)");
            ctx.begin_path();
            ctx.ellipse(rnd() * w, h - 4.0 - rnd() * 40.0, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(150,140,120,0.25)");
        gd.add_color_stop(0.15, "rgba(150,140,120,0)");
        gd.add_color_stop(0.85, "rgba(40,30,25,0)");
        gd.add_color_stop(1, "rgba(40,30,25,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// The gauge's dial: the green band of a charged cylinder between red, the
/// needle fallen into RECHARGE, its glass dusty.
fn ex_gauge_tex() -> Tex {
    r12_tex!(64, 64, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_fill_style("#d8d4c8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_line_width(7);
        for (a0, a1, col) in [(PI * 0.85, PI * 1.25, "#8a2a20"), (PI * 1.25, PI * 1.75, "#3c6a3c"), (PI * 1.75, PI * 2.15, "#8a2a20")] {
            ctx.set_stroke_style(col);
            ctx.begin_path();
            ctx.arc(cx, cy, 20, a0, a1);
            ctx.stroke();
        }
        ctx.set_fill_style("#222");
        ctx.set_font("bold 6px Helvetica");
        ctx.set_text_align("center");
        ctx.fill_text("PSI", cx, cy + 14.0);
        ctx.set_stroke_style("#1a1a1a");
        ctx.set_line_width(2);
        let a = PI * 0.95;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + a.cos() * 22.0, cy + a.sin() * 22.0);
        ctx.stroke();
        r12_blob(ctx, cx, cy, 34.0, 1.0, (160, 150, 130, 0.35));
    })
}

/// The inspection tag on the neck: a yellow card, its year's months
/// punched out until they stopped, initials in pen.
fn ex_tag_tex() -> Tex {
    r12_tex!(84, 156, false, |ctx, w, h| {
        ctx.set_fill_style("#c9b25a");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#2a241a");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 8px Helvetica");
        ctx.fill_text("INSPECTION", w / 2.0, 22);
        ctx.fill_text("RECORD", w / 2.0, 31);
        ctx.set_font("6px Helvetica");
        ctx.fill_text("DO NOT REMOVE", w / 2.0, 41);
        let months = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
        ctx.set_line_width(0.6);
        ctx.set_stroke_style("#2a241a");
        for (i, m) in months.iter().enumerate() {
            let (col, row) = ((i % 4) as f64, (i / 4) as f64);
            let (x, y) = (14.0 + col * 18.0, 58.0 + row * 18.0);
            ctx.stroke_rect(x - 7.0, y - 7.0, 14, 14);
            ctx.set_fill_style("#2a241a");
            ctx.fill_text(m, x, y);
            if i < 7 {
                ctx.set_fill_style("#3a3630");
                ctx.begin_path();
                ctx.arc(x, y, 4.0, 0, 7);
                ctx.fill();
            }
        }
        ctx.set_fill_style("rgba(30,30,70,0.85)");
        ctx.set_font("13px Marker Felt");
        ctx.fill_text("R.K.", w / 2.0, 126);
        // the hole and its brass eyelet
        ctx.set_fill_style("#5a4a2a");
        ctx.begin_path();
        ctx.arc(w / 2.0, 8, 4.5, 0, 7);
        ctx.fill();
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(90,70,40,0.1)");
        gd.add_color_stop(1, "rgba(60,40,20,0.35)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// Grey baked enamel (ANSI 61) on steel: rust blooming from the screws and
/// the knockouts, grime where the door is pulled and below it, dust on top.
fn fb_enamel_tex() -> Tex {
    r12_tex!(160, 232, false, |ctx, w, h| {
        ctx.set_fill_style("#efefea");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..600 {
            ctx.set_fill_style(&format!("rgba(80,80,76,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        for &(x, y) in &[(0.05, 0.04), (0.95, 0.04), (0.05, 0.96), (0.95, 0.96)] {
            r12_blob(ctx, x * w, y * h, 9.0, 1.0, (120, 60, 25, 0.55));
            ctx.set_fill_style("rgba(120,60,25,0.25)");
            ctx.fill_rect(x * w - 1.0, y * h, 2.0, 10.0 + rnd() * 20.0);
        }
        r12_blob(ctx, w * 0.8, h * 0.6, 30.0, 1.4, (50, 46, 40, 0.3));
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(140,130,110,0.35)");
        gd.add_color_stop(0.08, "rgba(140,130,110,0)");
        gd.add_color_stop(0.85, "rgba(40,36,30,0)");
        gd.add_color_stop(1, "rgba(40,36,30,0.28)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        r12_scratches(ctx, w, h, 26, (60, 58, 52), 0.12, 18.0);
        for _ in 0..10 {
            ctx.set_fill_style("rgba(110,70,40,0.7)");
            ctx.begin_path();
            ctx.ellipse(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 0.8 + rnd() * 1.5, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
    })
}

/// What is printed on the dead front (the canvas spans the door's opening,
/// 0.150 x 0.198 m): the circuit numbers by the breakers, the rating label.
fn fb_print_tex() -> Tex {
    r12_tex!(300, 396, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let to = |x: f64, y: f64| ((x + 0.075) * 2000.0, (0.099 - (y + 0.032)) * 2000.0);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("rgba(28,28,26,0.9)");
        ctx.set_font("bold 13px Helvetica");
        for (i, &y) in [0.000, -0.026, -0.052].iter().enumerate() {
            let (lx, ly) = to(-0.071, y);
            ctx.fill_text(&format!("{}", i * 2 + 1), lx, ly);
            let (rx, ry) = to(0.071, y);
            ctx.fill_text(&format!("{}", i * 2 + 2), rx, ry);
        }
        // the rating label above the breakers: paper, printed, curling
        let (x0, y0) = to(-0.050, 0.060);
        ctx.set_fill_style("rgba(226,222,208,0.95)");
        ctx.fill_rect(x0, y0, 200, 46);
        ctx.set_fill_style("rgba(30,30,28,0.9)");
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("LOAD CENTER", x0 + 100.0, y0 + 11.0);
        ctx.set_font("8px Helvetica");
        ctx.fill_text("120/240 VAC  1 PH  3 W  100 A MAX", x0 + 100.0, y0 + 24.0);
        ctx.fill_text("6 SPACES  USE COPPER WIRE ONLY", x0 + 100.0, y0 + 35.0);
        ctx.set_fill_style("rgba(90,70,40,0.25)");
        ctx.fill_rect(x0, y0 + 36.0, 200, 10);
        // DANGER at its foot
        let (dx, dy) = to(-0.040, -0.075);
        ctx.set_fill_style("rgba(150,40,26,0.9)");
        ctx.fill_rect(dx, dy, 160, 18);
        ctx.set_fill_style("rgba(235,230,220,0.95)");
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("DANGER  HIGH VOLTAGE", dx + 80.0, dy + 9.5);
    })
}

/// The circuit directory on the inside of the door: the printed card, what
/// each circuit fed written in, one crossed out, the card foxed.
fn fb_directory_tex() -> Tex {
    r12_tex!(224, 300, false, |ctx, w, h| {
        ctx.set_fill_style("#ddd6c2");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#26241f");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 14px Helvetica");
        ctx.fill_text("CIRCUIT DIRECTORY", w / 2.0, 18);
        ctx.set_stroke_style("rgba(40,38,34,0.8)");
        ctx.set_line_width(1);
        let entries = ["HALL LIGHTS", "RECEPT. NORTH", "COPIER", "COFFEE / SINK", "EXIT SIGNS", "SPARE"];
        for (i, e) in entries.iter().enumerate() {
            let y = 44.0 + i as f64 * 40.0;
            ctx.begin_path();
            ctx.move_to(10, y + 30.0);
            ctx.line_to(w - 10.0, y + 30.0);
            ctx.stroke();
            ctx.set_fill_style("#26241f");
            ctx.set_font("bold 12px Helvetica");
            ctx.set_text_align("left");
            ctx.fill_text(&format!("{}", i + 1), 12, y + 18.0);
            ctx.set_fill_style("rgba(30,32,80,0.85)");
            ctx.set_font("17px Marker Felt");
            ctx.fill_text(e, 34, y + 18.0);
        }
        // the copier struck out, something else written over it
        ctx.set_stroke_style("rgba(30,32,80,0.8)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(32, 44.0 + 2.0 * 40.0 + 18.0);
        ctx.line_to(104, 44.0 + 2.0 * 40.0 + 16.0);
        ctx.stroke();
        ctx.set_font("13px Marker Felt");
        ctx.set_fill_style("rgba(30,32,80,0.8)");
        ctx.fill_text("FRIDGE", 120, 44.0 + 2.0 * 40.0 + 18.0);
        for _ in 0..30 {
            r12_blob(ctx, rnd() * w, rnd() * h, 3.0 + rnd() * 8.0, 1.0, (140, 100, 50, 0.08 + rnd() * 0.12));
        }
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(160,130,70,0.15)");
        gd.add_color_stop(1, "rgba(90,70,40,0.3)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
    })
}
