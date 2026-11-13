//! The hole in the floor (batch `r03`), drawn through the floor: the dark
//! under it, a steel joist, the torn deck, the slab's broken face.
use super::*;

/// A hole broken through the floor (Blender batch `r03`): the covering
/// torn back, the concrete slab broken through to its steel deck, the deck
/// torn and bent down, the mesh's wires cut, a steel joist in the dark under
/// one side, rubble and dust at the rim, and dark below (docs/design/PROPS.md).
///
/// The floor cannot be cut, so the hole is drawn through it: what is under
/// the floor goes first, deepest first, without depth (a ray from above the
/// floor down into the hole meets what is higher first, so that order is
/// right); then a mask over the opening, in depth only, at the floor's
/// surface, which keeps the floor (drawn after it) out of the opening.
/// None of that writes depth, so the placement pass does not see it under
/// the floor (it lifted the old shaft 1.2 m out of the carpet). What it
/// does see: the rubble, and the old prop's boxes, kept as an invisible
/// footprint so nothing else is placed over the hole and nothing else moves.
pub fn build_floor_hole(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    use crate::assets::props::prop_part_as_modelled;
    let hg = group();
    hg.user().no_instance = true;
    let under = |depth_order: i32, mat: Mat| {
        mat.depthWrite(false);
        (depth_order, mat)
    };
    let face_tex = hole_concrete_tex(true);
    let parts: [(&str, (i32, Mat)); 8] = [
        ("Real__floorHoleShaft", under(-9, basic_mat!({ color: 0x000000 }))),
        ("Real__floorHoleJoist", under(-8, lambert!({ color: 0xffffff, map: &hole_joist_tex() }))),
        ("Real__floorHoleDeck", under(-7, lambert!({ color: 0x6f716f, map: &hole_deck_tex(), side: DoubleSide }))),
        ("Real__floorHoleFace", under(-6, lambert!({ color: 0x8e8a83, map: &face_tex }))),
        ("Real__floorHoleLedge", under(-6, lambert!({ color: 0xffffff, map: &hole_ledge_tex() }))),
        ("Real__floorHoleWire", under(-5, lambert!({ color: 0x4a3426 }))),
        ("Real__floorHoleCover", under(-5, lambert!({ color: 0x3a342d }))),
        // the opening, in depth only: the floor is not drawn in it
        (
            "Real__floorHoleMask",
            (-1, basic_mat!({ colorWrite: false, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -4 })),
        ),
    ];
    // the old prop's boxes (its shaft and its 14 rim blocks, as the placement
    // pass lifted them): drawn by nothing, seen by the placement pass only
    let footprint = basic_mat!({ visible: false });
    for i in 0..14 {
        let a = (i as f64 / 14.0) * PI * 2.0;
        let b = mesh(box_geo!(0.14, 0.06, 0.14), &footprint);
        b.position().set(a.cos() * 0.88, 1.23, a.sin() * 0.88);
        b.rotation().y = a;
        hg.add(&b);
    }
    let shaft_box = mesh(cyl_geo!(0.82, 0.82, 1.2, 20, 1, true), &footprint);
    shaft_box.position().y = 0.6;
    hg.add(&shaft_box);
    for (name, (order, mat)) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.set_render_order(order);
            hg.add(&m);
        }
    }
    // on the floor: the rubble, and the dust it left
    if let Some(geo) = prop_part_as_modelled("Real__floorHoleRubble") {
        hg.add(&mesh(geo, lambert!({ color: 0x9a968e, map: &hole_concrete_tex(false) })));
    }
    if let Some(geo) = prop_part_as_modelled("Real__floorHoleDust") {
        let dust = lambert!({ map: &hole_dust_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
        hg.add(&mesh(geo, dust));
    }
    Built::new(hg, 0.0)
}

/// Broken concrete: grey with its aggregate showing, grimed; for the slab's
/// face (`face`) darker toward its foot, where the floor's shadow keeps the
/// light out, and rust run down from the mesh's wires
fn hole_concrete_tex(face: bool) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        ctx.set_fill_style("#e4e1da");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 6.0 + rnd() * 22.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, if rnd() < 0.6 { "rgba(80,74,64,0.25)" } else { "rgba(255,252,245,0.25)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // the aggregate: stones broken through, light and dark
        for _ in 0..260 {
            let (x, y) = (rnd() * w, rnd() * h);
            let s = 0.8 + rnd() * 2.6;
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(70,66,60,0.55)" } else { "rgba(250,246,236,0.5)" });
            ctx.begin_path();
            ctx.ellipse(x, y, s, s * (0.5 + rnd() * 0.5), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        if face {
            // rust run down from the wires (a sixth of the way down)
            for _ in 0..7 {
                let x = rnd() * w;
                let len = h * (0.2 + rnd() * 0.5);
                let grad = ctx.create_linear_gradient(0, h * 0.16, 0, h * 0.16 + len);
                grad.add_color_stop(0, "rgba(110,60,30,0.55)");
                grad.add_color_stop(1, "rgba(110,60,30,0)");
                ctx.set_fill_style(grad);
                ctx.fill_rect(x, h * 0.16, 1.5 + rnd() * 3.0, len);
            }
            let grad = ctx.create_linear_gradient(0, 0, 0, h);
            grad.add_color_stop(0, "rgba(0,0,0,0)");
            grad.add_color_stop(0.35, "rgba(10,8,6,0.25)");
            grad.add_color_stop(1, "rgba(5,4,3,0.82)");
            ctx.set_fill_style(grad);
            ctx.fill_rect(0, 0, w, h);
        }
    })
}

/// The concrete's top where the covering tore off: grey, the adhesive left
/// in black smears, dust and grit (laid on from above)
fn hole_ledge_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#7e7a72");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 6.0 + rnd() * 20.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(28,24,20,0.55)" } else { "rgba(170,165,156,0.4)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        for _ in 0..500 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(40,36,30,0.5)" } else { "rgba(190,185,175,0.5)" });
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
    })
}

/// The joist's shop primer (a dull grey-brown), rust at its edges, going
/// dark with depth in the shadow under the floor (the parts' box mapping
/// runs v up its height)
fn hole_joist_tex() -> Tex {
    canvas_tex_opt(64, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#5e5650");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 2.0 + rnd() * 7.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, "rgba(104,58,30,0.45)");
            gr.add_color_stop(1, "rgba(104,58,30,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "rgba(8,7,6,0.35)");
        grad.add_color_stop(0.5, "rgba(8,7,6,0.75)");
        grad.add_color_stop(1, "rgba(4,4,3,0.95)");
        ctx.set_fill_style(grad);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// The galvanised deck: dull grey, spangled, rusting along the tear, in the
/// slab's shadow
fn hole_deck_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#bfc1bf");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..120 {
            let (x, y) = (rnd() * w, rnd() * h);
            let s = 2.0 + rnd() * 6.0;
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(90,94,96,0.25)" } else { "rgba(230,232,232,0.25)" });
            ctx.fill_rect(x, y, s, s * (0.6 + rnd() * 0.6));
        }
        for _ in 0..24 {
            let (x, y) = (rnd() * w, rnd() * h);
            let r = 3.0 + rnd() * 12.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, "rgba(120,62,28,0.7)");
            gr.add_color_stop(1, "rgba(120,62,28,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
    })
}

/// Concrete dust round the opening: thick at its rim, thinning out over the
/// floor, with grit and a few drag marks (the decal's centre is the hole's)
fn hole_dust_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let (cx0, cy0) = (w / 2.0, h / 2.0);
        // the opening's rim is at about 0.28 of the width from the centre;
        // the dust lies in a narrow, patchy band round it
        let rim = w * 0.275;
        let gr = ctx.create_radial_gradient(cx0, cy0, rim * 0.95, cx0, cy0, w * 0.42);
        gr.add_color_stop(0, "rgba(170,164,152,0.32)");
        gr.add_color_stop(0.3, "rgba(165,159,147,0.14)");
        gr.add_color_stop(1, "rgba(160,154,142,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        // patchy: thicker in drifts where pieces fell and broke, swept thin elsewhere
        for _ in 0..50 {
            let a = rnd() * 6.283;
            let d = rim + rnd().powf(1.8) * w * 0.16;
            let (x, y) = (cx0 + a.cos() * d, cy0 + a.sin() * d);
            let r = 3.0 + rnd() * 10.0;
            let g2 = ctx.create_radial_gradient(x, y, 0, x, y, r);
            g2.add_color_stop(0, "rgba(176,170,158,0.3)");
            g2.add_color_stop(1, "rgba(176,170,158,0)");
            ctx.set_fill_style(g2);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        }
        // grit
        for _ in 0..500 {
            let a = rnd() * 6.283;
            let d = rim + rnd().powf(2.4) * w * 0.2;
            let (x, y) = (cx0 + a.cos() * d, cy0 + a.sin() * d);
            ctx.set_fill_style(if rnd() < 0.6 { "rgba(120,116,108,0.8)" } else { "rgba(200,196,186,0.8)" });
            ctx.fill_rect(x, y, 0.8 + rnd() * 1.6, 0.8 + rnd() * 1.4);
        }
    })
}
