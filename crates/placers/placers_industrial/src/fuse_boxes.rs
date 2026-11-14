//! `place_fuse_boxes`
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

// Wall fuse box: 4% chance per level. A 6-space breaker panel on the
// surface (Blender batch `r12`): the grey enclosure, its front cover and dead
// front, the door hanging open, six single-pole breakers on or off, the
// circuits written on the card inside the door.
pub(crate) fn place_fuse_boxes(g: &mut Game, _dist: &[i32]) {
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

/// The breaker panel: the enclosure's depth, the door (w, h) and its middle,
/// the breakers' columns and rows (`core/what_services_screw_to_walls.py`, FB_*).
const FB_D: f64 = 0.083;

const FB_DOOR: (f64, f64) = (0.150, 0.198);

const FB_DOOR_C: (f64, f64) = (0.0, -0.032);

const FB_COL: [f64; 2] = [-0.036, 0.036];

const FB_ROW: [f64; 3] = [0.000, -0.026, -0.052];

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
