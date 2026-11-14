//! `place_extinguisher_cabinets`
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
    ($w:expr_2021, $h:expr_2021, $repeat:expr_2021, $paint:expr_2021) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, $repeat, $paint)).clone())
    }};
}

// Wall extinguisher cabinet: 4% chance per level. A surface-mounted
// steel cabinet for a 5 lb extinguisher (Blender family `what_services_screw_to_walls`): white enamel
// gone yellow and chipped, a full pane in its door frame, FIRE EXTINGUISHER
// down the glass; in it the extinguisher stands on its foot, its hose in its
// band, the pin sealed, its tag hanging.
pub(crate) fn place_extinguisher_cabinets(g: &mut Game, _dist: &[i32]) {
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
    o.add(&mesh(part("Real__extCabinetTub", 0.205, 0.456, 0.134), &enamel));
    o.add(&mesh(part("Real__extCabinetDoor", 0.205, 0.456, 0.022), &enamel));
    o.add(&mesh(part("Real__extCabinetHardware", 0.21, 0.45, 0.04), phong!({ color: 0x75746f, specular: 0x3a3a38, shininess: 36 })));
    // the extinguisher, standing on the tub's floor, turned so its label
    // and gauge face out past the door's edge
    let ex = group();
    ex.position().set(-0.014, -EC_H / 2.0 + 0.0014, 0.073);
    ex.rotation().y = PI - 0.4;
    o.add(&ex);
    ex.add(&mesh(part("Real__extCabinetExBody", 0.108, 0.24, 0.108), lambert!({ map: &ex_label_tex(), color: 0xffffff })));
    ex.add(&mesh(part("Real__extCabinetExEnds", 0.108, 0.3, 0.108), lambert!({ color: 0x7e1c14 })));
    ex.add(&mesh(part("Real__extCabinetExValve", 0.14, 0.07, 0.05), phong!({ color: 0x8a8984, specular: 0x505050, shininess: 44 })));
    ex.add(&mesh(part("Real__extCabinetExSeal", 0.02, 0.02, 0.015), lambert!({ color: 0xb49a2e })));
    ex.add(&mesh(part("Real__extCabinetExHose", 0.04, 0.2, 0.03), lambert!({ color: 0x1f1e1c })));
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
        part("Real__extCabinetGlass", 0.169, 0.42, 0.003),
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

/// The extinguisher cabinet's height and depth, the extinguisher's cylinder (`core/what_services_screw_to_walls.py`, EC_*, EX_*).
const EC_H: f64 = 0.456;

const EC_D: f64 = 0.156;

const EX_BODY: f64 = 0.300;

/// White baked enamel on steel, gone cream: chipped to the metal at the
/// corners and the door's edge, rust in the chips at the foot, grime where
/// the door is pulled, dust on top (the canvas is the cabinet's face).
fn ec_enamel_tex() -> Tex {
    cached_tex!(128, 284, false, |ctx, w, h| {
        ctx.set_fill_style("#f3efe4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..500 {
            ctx.set_fill_style(format!("rgba(120,105,80,{})", 0.02 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
        }
        let gy = ctx.create_linear_gradient(0, 0, 0, h);
        gy.add_color_stop(0, "rgba(190,160,100,0.30)");
        gy.add_color_stop(0.5, "rgba(190,160,100,0.12)");
        gy.add_color_stop(1, "rgba(80,60,40,0.30)");
        ctx.set_fill_style(gy);
        ctx.fill_rect(0, 0, w, h);
        // the pull's grime (right edge, middle)
        blob(ctx, w * 0.92, h * 0.5, w * 0.22, 2.0, (60, 50, 40, 0.3));
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
        scratches(ctx, w, h, 20, (90, 80, 70), 0.12, 16.0);
    })
}

/// The pane's print (the canvas is the pane, 0.169 x 0.420 m): FIRE
/// EXTINGUISHER in red vinyl down the glass, reading up, a letter or two
/// peeled; dust thick at the top and the corners, a hand's smear, old tape.
fn ec_glass_tex() -> Tex {
    cached_tex!(236, 588, false, |ctx, w, h| {
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
            ctx.set_fill_style(format!("rgba(0,0,0,{})", 0.4 + rnd() * 0.6));
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
            blob(ctx, x, y, 60.0, 1.0, (160, 150, 130, 0.35));
        }
        // a hand's smear at the pull side, and a strip of old tape's glue
        blob(ctx, w * 0.85, h * 0.5, 40.0, 1.6, (90, 80, 70, 0.18));
        ctx.set_fill_style("rgba(190,170,120,0.35)");
        ctx.fill_rect(w * 0.08, h * 0.06, 48, 14);
        scratches(ctx, w, h, 16, (220, 220, 215), 0.15, 30.0);
    })
}

/// The extinguisher's wall (the canvas wraps it, u round from its back,
/// its front at the middle; v up its 0.238 m): red enamel, the label at the
/// front, scuffed where it stood among things, chipped low.
fn ex_label_tex() -> Tex {
    cached_tex!(512, 360, false, |ctx, w, h| {
        ctx.set_fill_style("#8a2016");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..800 {
            ctx.set_fill_style(format!("rgba({},20,12,{})", if rnd() < 0.5 { 40 } else { 180 }, 0.04 + rnd() * 0.05));
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
        scratches(ctx, w, h, 40, (200, 190, 180), 0.12, 20.0);
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
    cached_tex!(64, 64, false, |ctx, w, h| {
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
        ctx.line_to(cx + a.dcos() * 22.0, cy + a.dsin() * 22.0);
        ctx.stroke();
        blob(ctx, cx, cy, 34.0, 1.0, (160, 150, 130, 0.35));
    })
}

/// The inspection tag on the neck: a yellow card, its year's months
/// punched out until they stopped, initials in pen.
fn ex_tag_tex() -> Tex {
    cached_tex!(84, 156, false, |ctx, w, h| {
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
