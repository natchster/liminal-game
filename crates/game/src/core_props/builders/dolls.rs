//! The toys left on the floor, as the real ones (batch `r03`): the bath
//! duck and the bent-limb baby doll, their vinyl, paint and cloth painted.
use super::*;

/// The classic bath duck (Blender batch `r03`): blow-moulded vinyl, 9 cm
/// from tail to bill, its yellow faded and grimed, the bill's paint chipped
/// back to the vinyl, the painted eyes yellowed (docs/PROPS.md)
pub fn build_rubber_duck(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    use crate::assets::props::prop_part_as_modelled;
    // (no draws: the old duck made none, and it is built unplaced for the
    // draws a stacked top once made)
    let dg = group();
    let parts: [(&str, Mat); 4] = [
        ("Real__duckBody", lambert!({ color: 0xd2b866, map: &duck_vinyl_tex() })),
        ("Real__duckBill", lambert!({ color: 0xffffff, map: &duck_bill_tex() })),
        ("Real__duckEyeWhite", lambert!({ color: 0xcfc7b0, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 })),
        ("Real__duckEyePupil", lambert!({ color: 0x2a2622, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            dg.add(&mesh(geo, mat));
        }
    }
    Built::new(dg, 0.08)
}

/// The duck's vinyl: faded unevenly, grime gathered toward its base and in
/// specks, scuffed (multiplies its yellow; the parts' box mapping lays it on
/// each side)
fn duck_vinyl_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#f2f0ea");
        ctx.fill_rect(0, 0, w, h);
        // faded and darker patches
        for _ in 0..26 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 8.0 + rnd() * 26.0;
            let dark = rnd() < 0.5;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, if dark { "rgba(110,98,70,0.22)" } else { "rgba(255,252,240,0.25)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // grime toward the base, where it sat in the wet
        let grad = ctx.create_linear_gradient(0, h * 0.62, 0, h);
        grad.add_color_stop(0, "rgba(70,60,40,0)");
        grad.add_color_stop(0.7, "rgba(70,60,40,0.35)");
        grad.add_color_stop(1, "rgba(52,44,30,0.6)");
        ctx.set_fill_style(grad);
        ctx.fill_rect(0, h * 0.62, w, h * 0.38);
        // specks of dirt and scuffs
        for _ in 0..50 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(if rnd() < 0.75 { "rgba(70,60,42,0.25)" } else { "rgba(255,255,250,0.2)" });
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd());
        }
    })
}

/// The bill's paint: an orange-red worn thin, chipped back to the yellow
/// vinyl at its edges, darker where dirt keeps
fn duck_bill_tex() -> Tex {
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#a9542a");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(90,50,30,0.25)" } else { "rgba(220,140,90,0.2)" });
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 6.0, 1.0 + rnd() * 4.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // chips: the vinyl's faded yellow under the paint
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style("rgba(196,172,96,0.9)");
            ctx.begin_path();
            ctx.ellipse(x, y, 0.8 + rnd() * 2.4, 0.6 + rnd() * 1.6, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
    })
}

/// A bent-limb play baby doll of the 1970s–80s, lying on its back (Blender
/// batch `r03`): 38 cm, a soft-vinyl head as wide as its shoulders with
/// moulded, painted hair, sleep eyes (shut as it lies, one stuck half open)
/// and an open mouth for its bottle; chubby bent vinyl arms and legs; a cloth
/// body in a faded cotton romper with a Peter Pan collar. The vinyl yellowed
/// and grimed, the paint rubbed, a child's ballpoint on its brow
/// (docs/PROPS.md)
pub fn build_baby_doll(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    use crate::assets::props::prop_part_as_modelled;
    // (no draws: the old doll made none, and it is built unplaced for the
    // draws a stacked top once made)
    // the head frame's origin (under its neck) in the body's, and how far it
    // is rolled toward its right: DOLL_NECK and DOLL_TURN in r03.py
    const NECK: (f64, f64, f64) = (0.050, 0.0592, 0.0);
    const TURN: f64 = -0.30;
    let dg = group();
    let add = |parent: &Obj, name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            parent.add(&mesh(geo, mat));
        }
    };
    let vinyl = lambert!({ color: 0xffffff, map: &doll_vinyl_tex() });
    add(&dg, "Real__dollRomper", &lambert!({ color: 0xffffff, map: &doll_romper_tex() }));
    add(&dg, "Real__dollCollar", &lambert!({ color: 0xc9c4b6 }));
    add(&dg, "Real__dollArms", &vinyl);
    add(&dg, "Real__dollLegs", &vinyl);
    add(&dg, "Real__dollHead", &vinyl);
    add(
        &dg,
        "Real__dollHair",
        &lambert!({ color: 0xffffff, map: &doll_hair_tex(), polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
    );
    // the face, the eyes, their lids and lashes, from the head's own frame
    // (face +z, crown +y) laid face up, crown toward +x, rolled toward its
    // right
    let head = group();
    head.position().set(NECK.0, NECK.1, NECK.2);
    head.rotation().set(TURN - PI / 2.0, 0, -PI / 2.0);
    add(&head, "Real__dollFace", &lambert!({ color: 0xffffff, map: &doll_face_tex() }));
    let eye = phong!({ color: 0xffffff, map: &doll_eye_tex(), specular: 0x2a2a2a, shininess: 70 });
    add(&head, "Real__dollEyeR", &eye);
    add(&head, "Real__dollEyeL", &eye);
    add(&head, "Real__dollLids", &lambert!({ color: 0xc29a84 }));
    add(&head, "Real__dollLashes", &lambert!({ color: 0x3a312b, side: DoubleSide }));
    dg.add(&head);
    Built::new(dg, 0.08)
}

/// The face part's bounds in the head's frame (x across, y up; r03.py): its
/// paint is laid straight on along z (its frame is a plane's), so a point
/// (x, y) of the face is at these fractions of the canvas
const DOLL_FACE_LO: (f64, f64) = (-0.0501, -0.0125);
const DOLL_FACE_HI: (f64, f64) = (0.0501, 0.0961);
fn doll_face_px(x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let u = (x - DOLL_FACE_LO.0) / (DOLL_FACE_HI.0 - DOLL_FACE_LO.0);
    let v = (y - DOLL_FACE_LO.1) / (DOLL_FACE_HI.1 - DOLL_FACE_LO.1);
    (u * w, (1.0 - v) * h)
}

/// The doll's face: the vinyl (as `doll_vinyl_tex`, plain toward the edges
/// where it meets the head) and its paint, as a factory sprayed and brushed
/// it and as years rubbed it: blushed cheeks, fine brows, painted lower
/// lashes, the lips round the bottle's hole, the nostrils; dirt in the
/// creases round the eyes, the nose and the mouth, specks, a grey smudge, a
/// child's blue ballpoint on the brow
fn doll_face_tex() -> Tex {
    canvas_tex_opt(256, 288, false, |ctx, w, h| {
        ctx.set_fill_style("#d6b19a");
        ctx.fill_rect(0, 0, w, h);
        let px = |x: f64, y: f64| doll_face_px(x, y, w, h);
        let k = w / (DOLL_FACE_HI.0 - DOLL_FACE_LO.0); // px per metre
        let blot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, c0: &str, c1: &str| {
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, c0);
            gr.add_color_stop(1, c1);
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        };
        // the vinyl's own unevenness, faint (the head part's texture is as faint)
        for _ in 0..20 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 10.0 + rnd() * 26.0;
            blot(ctx, x, y, r, if rnd() < 0.6 { "rgba(120,96,74,0.10)" } else { "rgba(240,214,190,0.10)" }, "rgba(0,0,0,0)");
        }
        for s in [-1.0, 1.0] {
            // blush, sprayed: faded, and rubbed off the cheek's crown
            let (cx0, cy0) = px(s * 0.025, 0.038);
            blot(ctx, cx0, cy0, 0.016 * k, "rgba(196,118,100,0.42)", "rgba(196,118,100,0)");
            blot(ctx, cx0 + s * 2.0, cy0 - 3.0, 0.005 * k, "rgba(214,177,154,0.35)", "rgba(214,177,154,0)");
            // the brow: fine strokes along an arc, light brown, rubbed thin
            ctx.set_stroke_style("rgba(122,88,60,0.55)");
            ctx.set_line_width(1.2);
            for i in 0..9 {
                let t = i as f64 / 8.0;
                let x = s * (0.008 + 0.022 * t);
                let y = 0.0715 + 0.0035 * (1.0 - (2.0 * t - 0.8).powi(2));
                let (a, b) = px(x, y);
                ctx.begin_path();
                ctx.move_to(a, b + 1.0);
                ctx.line_to(a + s * 3.0, b - 1.4);
                ctx.stroke();
            }
            // painted lower lashes under the opening
            ctx.set_stroke_style("rgba(78,56,42,0.6)");
            ctx.set_line_width(0.9);
            for i in 0..6 {
                let t = i as f64 / 5.0;
                let x = s * (0.0125 + 0.0135 * t);
                let (a, b) = px(x, 0.0532 - 0.0012 * (1.0 - (2.0 * t - 1.0).powi(2)) + s * (x - s * 0.019) * 0.1);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.line_to(a + s * 1.2, b + 3.2);
                ctx.stroke();
            }
            // dirt in the crease round the eye's opening
            let (ex, ey) = px(s * 0.019, 0.0585);
            ctx.set_stroke_style("rgba(84,66,50,0.35)");
            ctx.set_line_width(2.4);
            ctx.begin_path();
            ctx.ellipse(ex, ey, 0.0102 * k, 0.0060 * k, 0, 0, 7);
            ctx.stroke();
            // the nostrils, and dirt round the nose's wings
            let (a, b) = px(s * 0.0034, 0.0442);
            ctx.set_fill_style("rgba(96,58,48,0.6)");
            ctx.begin_path();
            ctx.ellipse(a, b, 1.6, 1.1, s * 0.4, 0, 7);
            ctx.fill();
            let (a, b) = px(s * 0.0062, 0.0448);
            blot(ctx, a, b, 0.003 * k, "rgba(90,70,54,0.3)", "rgba(90,70,54,0)");
        }
        // the lips: a rose, worn pale at their crowns, round the bottle's
        // hole (dark inside), dirt at the corners
        let (mx, my) = px(0.0, 0.0333);
        ctx.set_fill_style("rgba(164,100,92,0.55)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0060 * k, 0.0051 * k, 0, 0, 7);
        ctx.fill();
        blot(ctx, mx, my - 0.0042 * k, 0.0022 * k, "rgba(214,170,150,0.5)", "rgba(214,170,150,0)");
        ctx.set_fill_style("rgba(54,26,24,0.95)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0027 * k, 0.0027 * k, 0, 0, 7);
        ctx.fill();
        for s in [-1.0, 1.0] {
            blot(ctx, mx + s * 0.0068 * k, my, 0.0025 * k, "rgba(80,58,44,0.35)", "rgba(80,58,44,0)");
        }
        // years: specks of dirt, scuffs where the paint and the vinyl's skin
        // rubbed off, a grey smudge on its right cheek
        for _ in 0..70 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(if rnd() < 0.8 { "rgba(84,68,54,0.35)" } else { "rgba(236,214,196,0.35)" });
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.6, 1.0 + rnd());
        }
        let (sx0, sy0) = px(-0.028, 0.030);
        blot(ctx, sx0, sy0, 0.013 * k, "rgba(74,66,60,0.32)", "rgba(74,66,60,0)");
        // a child's blue ballpoint: a few short strokes on the brow, faded
        ctx.set_stroke_style("rgba(58,70,122,0.45)");
        ctx.set_line_width(1.0);
        let (bx, by) = px(0.006 + rnd() * 0.01, 0.080 + rnd() * 0.004);
        ctx.begin_path();
        ctx.move_to(bx, by);
        for i in 0..7 {
            let fi = i as f64;
            ctx.line_to(bx + fi * 3.4 + rnd() * 1.5, by + if i % 2 == 0 { 0.0 } else { 3.5 + rnd() * 2.0 });
        }
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(bx - 4.0, by + 9.0);
        ctx.quadratic_curve_to(bx + 6.0, by + 14.0, bx + 16.0, by + 8.0);
        ctx.stroke();
    })
}

/// A sleep eye: an ivory ball yellowed with age, a grey-blue iris with a
/// darker ring and streaks, the pupil black (the iris faces +z: a quarter
/// of the way along)
fn doll_eye_tex() -> Tex {
    canvas_tex_opt(256, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#d2c8b2");
        ctx.fill_rect(0, 0, w, h);
        let (cx0, cy0) = (w * 0.25, h * 0.5);
        let gr = ctx.create_radial_gradient(cx0, cy0, 0, cx0, cy0, 21);
        gr.add_color_stop(0, "#3d5468");
        gr.add_color_stop(0.55, "#64829a");
        gr.add_color_stop(0.85, "#4b677e");
        gr.add_color_stop(1, "#26323e");
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 21, 21, 0, 0, 7);
        ctx.fill();
        ctx.set_stroke_style("rgba(30,44,58,0.5)");
        ctx.set_line_width(1);
        for k in 0..28 {
            let a = k as f64 / 28.0 * 6.283;
            ctx.begin_path();
            ctx.move_to(cx0 + a.cos() * 9.0, cy0 + a.sin() * 9.0);
            ctx.line_to(cx0 + a.cos() * 19.0, cy0 + a.sin() * 19.0);
            ctx.stroke();
        }
        ctx.set_fill_style("#1c1a1a");
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 8.5, 8.5, 0, 0, 7);
        ctx.fill();
    })
}

/// Moulded hair's paint: a mid brown, combed in fine streaks, rubbed back
/// to the vinyl in places
fn doll_hair_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#86684a");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_line_width(1);
        for _ in 0..160 {
            let x = rnd() * w;
            let y = rnd() * h;
            let l = 10.0 + rnd() * 30.0;
            ctx.set_stroke_style(if rnd() < 0.5 { "rgba(60,42,28,0.45)" } else { "rgba(150,116,84,0.35)" });
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + (rnd() - 0.5) * 4.0, y + l);
            ctx.stroke();
        }
        for _ in 0..14 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 2.0 + rnd() * 6.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, "rgba(206,172,146,0.65)");
            gr.add_color_stop(1, "rgba(206,172,146,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
    })
}

/// The head's, arms' and legs' vinyl: yellowed, faintly uneven, grimed
/// toward the floor, specks of dirt
fn doll_vinyl_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#d6b19a");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 6.0 + rnd() * 20.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, if rnd() < 0.65 { "rgba(110,88,66,0.16)" } else { "rgba(240,214,190,0.14)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        let grad = ctx.create_linear_gradient(0, h * 0.55, 0, h);
        grad.add_color_stop(0, "rgba(70,56,42,0)");
        grad.add_color_stop(1, "rgba(70,56,42,0.45)");
        ctx.set_fill_style(grad);
        ctx.fill_rect(0, h * 0.55, w, h * 0.45);
        for _ in 0..70 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style("rgba(70,58,46,0.35)");
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd());
        }
    })
}

/// The romper's cotton: a powder blue faded unevenly toward grey, its
/// weave, stains and the grey of the floor where it lay
fn doll_romper_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#9eaab2");
        ctx.fill_rect(0, 0, w, h);
        // the weave
        for y in 0..64 {
            ctx.set_fill_style(if y % 2 == 0 { "rgba(255,255,255,0.03)" } else { "rgba(0,0,0,0.03)" });
            ctx.fill_rect(0, y as f64 * 2.0, w, 1);
        }
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 6.0 + rnd() * 20.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, if rnd() < 0.6 { "rgba(110,92,74,0.25)" } else { "rgba(225,210,200,0.22)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // an old stain, dried brown at its edge
        for _ in 0..3 {
            let (x, y) = (w * 0.2 + rnd() * w * 0.6, h * 0.2 + rnd() * h * 0.4);
            let r = 5.0 + rnd() * 9.0;
            let gr = ctx.create_radial_gradient(x, y, r * 0.3, x, y, r);
            gr.add_color_stop(0, "rgba(120,96,70,0.08)");
            gr.add_color_stop(0.85, "rgba(110,84,60,0.28)");
            gr.add_color_stop(1, "rgba(110,84,60,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        let grad = ctx.create_linear_gradient(0, h * 0.6, 0, h);
        grad.add_color_stop(0, "rgba(70,62,54,0)");
        grad.add_color_stop(1, "rgba(70,62,54,0.5)");
        ctx.set_fill_style(grad);
        ctx.fill_rect(0, h * 0.6, w, h * 0.4);
    })
}
